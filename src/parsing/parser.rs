use std::collections::HashMap;

use crate::error::{ParseError, ParseErrorKind};
use crate::parsing::{Lexer, Token, TokenKind};
use crate::value::{VexValue, VexValueKind};
use crate::{Block, BlockDef, BlockKind, Global, Vex, VexVersion};

/// Parses Vex [`Token`]s into Vex data structures.
pub struct Parser<'de> {
    whole: &'de str,
    tokens: Vec<Token<'de>>,
    cursor: usize,
}

impl<'de> Parser<'de> {
    pub fn parse_vex(&mut self) -> Result<Vex, ParseError> {
        let version = self.parse_vex_version()?;
        let global = self.parse_global()?;

        let mut vex = Vex::new();
        vex.set_version(version.maj as usize, version.min as usize);
        *vex.global_mut() = global;
        while let Some(peeked) = self.peek()
            && peeked.kind == TokenKind::Dollar
        {
            let block = self.parse_block()?;
            vex.add(block);
        }
        Ok(vex)
    }

    pub fn parse_vex_version(&mut self) -> Result<VexVersion, ParseError> {
        let vexrev = *self.expect(TokenKind::Ident)?;
        self.expect_dismiss(TokenKind::Equals)?;
        let version = *self.expect(TokenKind::Float)?;
        self.expect_dismiss(TokenKind::SemiColon)?;

        if vexrev.source != "VEX_rev" {
            return Err(self.error(ParseErrorKind::InvalidVexRev));
        }

        let (lhs, rhs) = version
            .source
            .split_once('.')
            .ok_or(self.error(ParseErrorKind::InvalidVexRev))?;
        let maj: u8 = lhs
            .parse()
            .map_err(|_| self.error(ParseErrorKind::InvalidVexRev))?;
        let min: u8 = rhs
            .parse()
            .map_err(|_| self.error(ParseErrorKind::InvalidVexRev))?;
        Ok(VexVersion { maj, min })
    }

    pub fn parse_global(&mut self) -> Result<Global, ParseError> {
        self.expect_dismiss(TokenKind::Dollar)?;
        let blkname = *self.expect(TokenKind::Ident)?;
        if blkname.source != "GLOBAL" {
            return Err(self.error(ParseErrorKind::MissingGlobal));
        }
        self.expect_dismiss(TokenKind::SemiColon)?;

        let mut global = Global::new();
        let map = self.collect_entries()?;
        for (k, v) in map.into_iter() {
            global.add(
                k,
                v.first()
                    .ok_or_else(|| self.error(ParseErrorKind::InvalidEntry))?
                    .first()
                    .ok_or_else(|| self.error(ParseErrorKind::InvalidEntry))?
                    .clone(),
            );
        }

        Ok(global)
    }

    pub fn parse_block(&mut self) -> Result<Block, ParseError> {
        self.expect_dismiss(TokenKind::Dollar)?;
        let blkname = *self.expect(TokenKind::Ident)?;
        self.expect_dismiss(TokenKind::SemiColon)?;

        let mut block = match blkname.source {
            "SCHED" => Block::new_schedule(),
            "MODE" | "STATION" => Block::new_highlevel(blkname.source),
            _ => Block::new_primitive(blkname.source),
        };

        if block.is_schedule() {
            while let Some(peeked) = self.peek()
                && peeked.kind == TokenKind::Scan
            {
                block.add(self.parse_scan()?);
            }
        } else {
            while let Some(peeked) = self.peek()
                && peeked.kind == TokenKind::Def
            {
                block.add(self.parse_blockdef(block.kind())?);
            }
        }

        Ok(block)
    }

    pub fn parse_blockdef(&mut self, parentkind: BlockKind) -> Result<BlockDef, ParseError> {
        self.expect_dismiss(TokenKind::Def)?;
        let defname = *self.expect(TokenKind::Ident)?;
        self.expect_dismiss(TokenKind::SemiColon)?;

        let mut blockdef = BlockDef::new(defname.source, parentkind);
        blockdef.set_values(self.collect_entries()?);

        self.expect_dismiss(TokenKind::EndDef)?;
        self.expect_dismiss(TokenKind::SemiColon)?;

        Ok(blockdef)
    }

    pub fn parse_scan(&mut self) -> Result<BlockDef, ParseError> {
        self.expect_dismiss(TokenKind::Scan)?;
        let defname = *self.expect(TokenKind::Ident)?;
        self.expect_dismiss(TokenKind::SemiColon)?;

        let mut blockdef = BlockDef::new(defname.source, BlockKind::Schedule);
        blockdef.set_values(self.collect_entries()?);

        self.expect_dismiss(TokenKind::EndScan)?;
        self.expect_dismiss(TokenKind::SemiColon)?;

        Ok(blockdef)
    }

    pub fn parse_vex_value(&mut self) -> Result<VexValue, ParseError> {
        let peeked = self.tokens.get(self.cursor).ok_or_else(|| {
            ParseError::new(
                ParseErrorKind::UnexpectedEof,
                self.whole,
                self.whole.len(),
                self.whole.len(),
            )
        })?;

        if peeked.kind == TokenKind::Colon || peeked.kind == TokenKind::SemiColon {
            return Ok(VexValue::empty());
        }

        let token = *self.expect_if(|k| {
            matches!(
                k,
                TokenKind::Declination
                    | TokenKind::Epoch
                    | TokenKind::Float
                    | TokenKind::Ident
                    | TokenKind::Integer
                    | TokenKind::RightAscension
                    | TokenKind::Ampersand
                    | TokenKind::Plus
                    | TokenKind::Minus
                    | TokenKind::Literal
            )
        })?;

        let (source, unit, kind) = match token.kind {
            TokenKind::Declination => (token.source.to_string(), None, VexValueKind::Declination),
            TokenKind::RightAscension => {
                (token.source.to_string(), None, VexValueKind::RightAscension)
            }
            TokenKind::Epoch => (token.source.to_string(), None, VexValueKind::Epoch),
            TokenKind::Ident => (token.source.to_string(), None, VexValueKind::Ident),
            TokenKind::Literal => (token.source.to_string(), None, VexValueKind::Literal),
            TokenKind::Float | TokenKind::Integer => {
                match self.bump_if(|k| k == TokenKind::Ident) {
                    Some(unit) => (
                        token.source.to_string(),
                        Some(unit.source.to_string()),
                        VexValueKind::NumberUnit,
                    ),
                    None => (token.source.to_string(), None, VexValueKind::Number),
                }
            }
            TokenKind::Ampersand => {
                let link = self.expect(TokenKind::Ident)?;
                (link.source.to_string(), None, VexValueKind::Link)
            }
            TokenKind::Plus | TokenKind::Minus => {
                let t = *self.expect_if(|k| {
                    matches!(
                        k,
                        TokenKind::Declination | TokenKind::Float | TokenKind::Integer
                    )
                })?;
                if t.kind == TokenKind::Declination {
                    (
                        token.source.to_string() + t.source,
                        None,
                        VexValueKind::Declination,
                    )
                } else {
                    match self.bump_if(|k| k == TokenKind::Ident) {
                        Some(unit) => (
                            token.source.to_string() + t.source,
                            Some(unit.source.to_string()),
                            VexValueKind::NumberUnit,
                        ),
                        None => (
                            token.source.to_string() + t.source,
                            None,
                            VexValueKind::Number,
                        ),
                    }
                }
            }
            _ => unreachable!(),
        };

        Ok(VexValue::new(source, unit, kind))
    }

    pub fn parse_ref(&mut self) -> Result<(String, Vec<VexValue>), ParseError> {
        self.expect_dismiss(TokenKind::Ref)?;
        self.expect_dismiss(TokenKind::Dollar)?;
        let blk = *self.expect(TokenKind::Ident)?;
        self.expect_dismiss(TokenKind::Equals)?;

        let mut values = Vec::new();
        while let Some(peeked) = self.peek()
            && !peeked.is_line_end()
        {
            values.push(self.parse_vex_value()?);
            let _ = self.expect_if(|k| k == TokenKind::Colon);
        }

        self.expect_dismiss(TokenKind::SemiColon)?;

        Ok((blk.source.to_string(), values))
    }

    pub fn parse_key_value(&mut self) -> Result<(String, Vec<VexValue>), ParseError> {
        let key = *self.expect(TokenKind::Ident)?;
        self.expect_dismiss(TokenKind::Equals)?;

        let mut values = Vec::new();
        while let Some(peeked) = self.peek()
            && !peeked.is_line_end()
        {
            values.push(self.parse_vex_value()?);
            let _ = self.expect_if(|k| k == TokenKind::Colon);
        }

        self.expect_dismiss(TokenKind::SemiColon)?;

        Ok((key.source.to_string(), values))
    }

    pub fn parse_entry(&mut self) -> Result<(String, Vec<VexValue>), ParseError> {
        let peeked = self.tokens.get(self.cursor).ok_or_else(|| {
            ParseError::new(
                ParseErrorKind::UnexpectedEof,
                self.whole,
                self.whole.len(),
                self.whole.len(),
            )
        })?;

        match peeked.kind {
            TokenKind::Ref => self.parse_ref(),
            TokenKind::Ident => self.parse_key_value(),
            _ => Err(self.error(ParseErrorKind::InvalidEntry)),
        }
    }

    pub fn new(input: &'de str) -> Result<Self, ParseError> {
        Ok(Self {
            whole: input,
            tokens: Lexer::new(input).lex()?,
            cursor: 0,
        })
    }

    fn collect_entries(&mut self) -> Result<HashMap<String, Vec<Vec<VexValue>>>, ParseError> {
        let mut entries = HashMap::new();
        while let Some(peeked) = self.peek()
            && !matches!(
                peeked.kind,
                TokenKind::EndDef | TokenKind::EndScan | TokenKind::Dollar
            )
        {
            let (key, values) = self.parse_entry()?;
            entries.entry(key).or_insert_with(Vec::new).push(values);
        }
        Ok(entries)
    }

    fn error(&self, kind: ParseErrorKind) -> ParseError {
        let offending = self.tokens.get(self.cursor).unwrap();
        ParseError::new(
            kind,
            self.whole,
            offending.loc,
            offending.loc + offending.len(),
        )
    }

    fn skip_comments(&mut self) {
        while let Some(peeked) = self.peek()
            && peeked.kind == TokenKind::Comment
        {
            self.cursor += 1
        }
    }

    fn peek(&self) -> Option<&Token<'de>> {
        self.tokens.get(self.cursor)
    }

    fn expect(&mut self, kind: TokenKind) -> Result<&Token<'de>, ParseError> {
        self.skip_comments();

        let peeked = self.tokens.get(self.cursor).ok_or_else(|| {
            ParseError::new(
                ParseErrorKind::UnexpectedEof,
                self.whole,
                self.whole.len(),
                self.whole.len(),
            )
        })?;

        if peeked.kind == kind {
            self.cursor += 1;
            Ok(peeked)
        } else {
            Err(ParseError::new(
                ParseErrorKind::UnexpectedToken {
                    expecting: kind,
                    found: peeked.kind,
                },
                self.whole,
                peeked.loc,
                peeked.loc + peeked.len(),
            ))
        }
    }

    fn expect_dismiss(&mut self, kind: TokenKind) -> Result<(), ParseError> {
        self.expect(kind).map(|_| ())
    }

    fn expect_if(&mut self, pred: impl Fn(TokenKind) -> bool) -> Result<&Token<'de>, ParseError> {
        self.skip_comments();
        let token = self.tokens.get(self.cursor).ok_or_else(|| {
            ParseError::new(
                ParseErrorKind::UnexpectedEof,
                self.whole,
                self.whole.len(),
                self.whole.len(),
            )
        })?;

        if !pred(token.kind) {
            return Err(self.error(ParseErrorKind::UnexpectedTokenFound { found: token.kind }));
        }

        self.cursor += 1;
        Ok(token)
    }

    fn bump_if(&mut self, pred: impl Fn(TokenKind) -> bool) -> Option<&Token<'de>> {
        self.skip_comments();
        let token = self.tokens.get(self.cursor)?;
        if !pred(token.kind) {
            return None;
        }

        self.cursor += 1;
        Some(token)
    }
}
