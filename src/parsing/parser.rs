use std::collections::HashMap;

use crate::error::{ParseError, VexError};
use crate::parsing::{Token, TokenKind, Tokenizer};
use crate::*;

/// Parses Vex [`Token`]s into Vex data structures.
pub struct Parser<'de> {
    tokenizer: Tokenizer<'de>,
}

impl<'de> Parser<'de> {
    /// Construct a new parser from a string reference
    pub fn new(input: &'de str) -> Self {
        Self { tokenizer: Tokenizer::new(input) }
    }

    /// Parse the referenced string into [`Vex`].
    pub fn parse(mut self) -> Result<Vex, VexError> {
        let rev = self.parse_vex_version()?;

        let mut global: Option<Global> = None;
        let mut mode: Option<HighLevelBlock> = None;
        let mut station: Option<HighLevelBlock> = None;
        let mut schedule: Option<Schedule> = None;
        let mut primitives: HashMap<String, Block> = HashMap::new();

        while let Some(block) = self.peek() {
            match block.source {
                "GLOBAL" => global = Some(self.parse_global()?),
                "STATION" => station = Some(self.parse_highlevel_block()?),
                "MODE" => mode = Some(self.parse_highlevel_block()?),
                "SCHED" => schedule = Some(self.parse_schedule()?),
                _ => {
                    if let Some(dup) = primitives.insert(block.source.to_string(), self.parse_primitive_block()?) {
                        return Err(VexError::DuplicateBlock(dup.name().to_string()));
                    }
                }
            }
        }

        Ok(Vex {
            rev,
            global: global.ok_or(VexError::MissingBlock("GLOBAL".to_string()))?,
            station: station.ok_or(VexError::MissingBlock("STATION".to_string()))?,
            mode: mode.ok_or(VexError::MissingBlock("MODE".to_string()))?,
            primitives,
            schedule: schedule.ok_or(VexError::MissingBlock("SCHED".to_string()))?,
        })
    }

    /// Peek ahead at the next token in the token stream
    pub fn peek(&mut self) -> Option<&Token<'de>> {
        self.tokenizer.peek()
    }

    /// Peek ahead at the kind of the next token in the token stream
    pub fn peek_kind(&mut self) -> Option<TokenKind> {
        self.peek().map(|t| t.kind)
    }

    /// Peek ahead at the byte location of the next token in the token stream
    pub fn peek_loc(&mut self) -> Option<usize> {
        self.peek().map(|t| t.loc)
    }

    /// Peek ahead at the byte length of the next token in the token stream
    pub fn peek_len(&mut self) -> Option<usize> {
        self.peek().map(|t| t.source.len())
    }

    /// Expect to find a [`Token`] of token kind `kind`, and return that token. An error is returned if `kind` does not match the found token kind.
    ///
    /// Note that this function consumes the token even in the case of an error. If this behavior is not desired, consider using [`peek`](Self::peek) or [`expect_if_kind`](Self::expect_if_kind) instead.
    pub fn expect(&mut self, kind: TokenKind) -> Result<Token<'de>, ParseError> {
        let curr_pos = self.tokenizer.byte;
        let next_token = self
            .tokenizer
            .next()
            .ok_or(ParseError::new_at("Unexpected end of token stream", self.tokenizer.whole, curr_pos))?;
        if next_token.kind != kind {
            Err(ParseError::new_range(
                format!("Expected to find a token of kind {} but found {}", kind, next_token.kind),
                self.tokenizer.whole,
                next_token.loc_range(),
            ))
        } else {
            Ok(next_token)
        }
    }

    /// Peek ahead at the next token and if a token of kind `kind` is found, call [`expect`](Self::expect).
    pub fn expect_if_kind(&mut self, kind: TokenKind) -> Option<Token<'de>> {
        if self.peek_kind()? == kind { self.expect(kind).ok() } else { None }
    }

    /// Like [`expect`](Self::expect), but just dismiss the token instead of returning it.
    pub fn expect_dismiss(&mut self, kind: TokenKind) -> Result<(), ParseError> {
        self.expect(kind).map(|_| ())
    }

    /// Expect to find a token in the stream, then peek at its kind.
    pub fn expect_peek_kind(&mut self) -> Result<TokenKind, ParseError> {
        self.peek_kind()
            .ok_or(ParseError::new_at("Unexpected end of token stream", self.tokenizer.whole, self.tokenizer.byte))
    }

    /// Expect to find a token of any kind within `kinds`, then return that token
    pub fn expect_oneof<const N: usize>(&mut self, kinds: [TokenKind; N]) -> Result<Token<'de>, ParseError> {
        let (peeked_kind, peeked_range) = match self.peek() {
            Some(t) => (t.kind, t.loc_range()),
            None => return Err(ParseError::new_at("Unexpected end of token stream", self.tokenizer.whole, self.tokenizer.byte)),
        };

        if kinds.contains(&peeked_kind) {
            Ok(self.expect(peeked_kind).unwrap())
        } else {
            Err(ParseError::new_range(
                format!("Expected to find one of {} but found a {} token", kinds.map(|k| k.to_string()).join(", "), peeked_kind),
                self.tokenizer.whole,
                peeked_range,
            ))
        }
    }

    /// Parse a [`VexVersion`] from the token stream
    pub fn parse_vex_version(&mut self) -> Result<VexVersion, ParseError> {
        match self.peek() {
            Some(t) => {
                if t.source != "VEX_rev" {
                    return Err(ParseError::new_at("Expected to find VEX_rev", self.tokenizer.whole, self.tokenizer.byte));
                }
            }
            None => return Err(ParseError::new_at("Unexpected end of token stream", self.tokenizer.whole, self.tokenizer.byte)),
        };

        self.expect_dismiss(TokenKind::Ident)?;
        self.expect_dismiss(TokenKind::Equals)?;

        let version = self.expect(TokenKind::Number)?;

        let (lhs, rhs) = version
            .source
            .split_once(".")
            .ok_or(ParseError::new_range("Failed to parse a Vex version", self.tokenizer.whole, version.loc_range()))?;
        let maj: u8 = lhs
            .parse()
            .map_err(|_| ParseError::new_range("Failed to parse a Vex version", self.tokenizer.whole, version.loc_range()))?;
        let min: u8 = rhs
            .parse()
            .map_err(|_| ParseError::new_range("Failed to parse a Vex version", self.tokenizer.whole, version.loc_range()))?;

        self.expect_dismiss(TokenKind::SemiColon)?;
        Ok(VexVersion::new(maj, min))
    }

    /// Parse a [`VexRef`] from the token stream
    pub fn parse_ref(&mut self) -> Result<VexRef, ParseError> {
        self.expect_dismiss(TokenKind::Ref)?;
        let blockname = self.expect(TokenKind::BlockName)?.source.to_string();
        self.expect_dismiss(TokenKind::Equals)?;
        let keyword = self.expect(TokenKind::Ident)?.source.to_string();

        let mut qualifiers = Vec::new();
        loop {
            let next = self.expect_oneof([TokenKind::Ident, TokenKind::Colon, TokenKind::SemiColon])?;
            match next.kind {
                TokenKind::Colon => continue,
                TokenKind::SemiColon => break,
                TokenKind::Ident => qualifiers.push(next.source.to_string()),
                _ => unreachable!(),
            }
        }

        Ok(VexRef::new_with_quals(blockname, keyword, qualifiers))
    }

    /// Parse a [`Param`] from the token stream
    pub fn parse_param(&mut self) -> Result<Param, ParseError> {
        let name = self.expect(TokenKind::Ident)?.source.to_string();
        self.expect_dismiss(TokenKind::Equals)?;

        let mut values = Vec::new();
        let mut prev_kind = TokenKind::Equals;
        loop {
            let peek_kind = self.expect_peek_kind()?;
            match peek_kind {
                TokenKind::Colon => {
                    if prev_kind == TokenKind::Colon || prev_kind == TokenKind::Equals {
                        values.push(ParamValue::empty());
                    }
                    prev_kind = TokenKind::Colon;
                    self.expect(TokenKind::Colon)?;
                }
                TokenKind::Ident | TokenKind::Literal | TokenKind::Epoch | TokenKind::Coord | TokenKind::Number | TokenKind::Link => {
                    let value = self.parse_param_value()?;
                    values.push(value);
                    let sep = self.expect_oneof([TokenKind::Colon, TokenKind::SemiColon])?;
                    prev_kind = sep.kind;
                    if sep.is_line_end() {
                        break;
                    }
                }
                TokenKind::SemiColon => {
                    if prev_kind == TokenKind::Colon || prev_kind == TokenKind::Equals {
                        values.push(ParamValue::empty());
                    }
                    self.expect(TokenKind::SemiColon)?;
                    break;
                }
                _ => unreachable!(),
            }
        }

        Ok(Param::new(name, values))
    }

    /// Parse a [`ParamValue`] from the token stream
    pub fn parse_param_value(&mut self) -> Result<ParamValue, ParseError> {
        let next = self.expect_oneof([TokenKind::Ident, TokenKind::Literal, TokenKind::Epoch, TokenKind::Coord, TokenKind::Number, TokenKind::Link])?;

        match next.kind {
            TokenKind::Ident => Ok(ParamValue::new_unchecked(next.source, ParamValueKind::Ident)),
            TokenKind::Literal => Ok(ParamValue::new_unchecked(next.source, ParamValueKind::Literal)),
            TokenKind::Epoch => Ok(ParamValue::new_unchecked(next.source, ParamValueKind::Epoch)),
            TokenKind::Coord => Ok(ParamValue::new_unchecked(next.source, ParamValueKind::Coord)),
            TokenKind::Link => Ok(ParamValue::new_unchecked(next.source, ParamValueKind::Link)),
            TokenKind::Number => {
                let maybe_unit = self.expect_if_kind(TokenKind::Unit);
                if let Some(unit) = maybe_unit {
                    Ok(ParamValue::new_unchecked(format!("{} {}", next.source, unit.source), ParamValueKind::NumberUnit))
                } else {
                    Ok(ParamValue::new_unchecked(next.source, ParamValueKind::Number))
                }
            }
            _ => unreachable!(),
        }
    }

    /// Parse a [`HighLevelDef`] from the token stream
    pub fn parse_highlevel_def(&mut self) -> Result<HighLevelDef, ParseError> {
        self.expect_dismiss(TokenKind::Def)?;
        let name = self.expect_oneof([TokenKind::Ident, TokenKind::Number])?.source.to_string();
        self.expect_dismiss(TokenKind::SemiColon)?;
        let mut refs = Vec::new();
        while self.expect_peek_kind()? != TokenKind::EndDef {
            refs.push(self.parse_ref()?);
        }
        self.expect_dismiss(TokenKind::EndDef)?;
        self.expect_dismiss(TokenKind::SemiColon)?;
        Ok(HighLevelDef::new(name, refs))
    }

    /// Parse a [`Def`] from the token stream
    pub fn parse_primitive_def(&mut self) -> Result<Def, ParseError> {
        self.expect_dismiss(TokenKind::Def)?;
        let name = self.expect_oneof([TokenKind::Ident, TokenKind::Number, TokenKind::Coord])?.source.to_string();
        self.expect_dismiss(TokenKind::SemiColon)?;
        let mut params = Vec::new();
        while self.expect_peek_kind()? != TokenKind::EndDef {
            params.push(self.parse_param()?);
        }
        self.expect_dismiss(TokenKind::EndDef)?;
        self.expect_dismiss(TokenKind::SemiColon)?;
        Ok(Def::new(name, params))
    }

    /// Parse a [`Scan`] from the token stream
    pub fn parse_scan(&mut self) -> Result<Scan, ParseError> {
        self.expect_dismiss(TokenKind::Scan)?;
        let name = self.expect_oneof([TokenKind::Ident, TokenKind::Number, TokenKind::Coord])?.source.to_string();
        self.expect_dismiss(TokenKind::SemiColon)?;
        let mut params = Vec::new();
        while self.expect_peek_kind()? != TokenKind::EndScan {
            params.push(self.parse_param()?);
        }
        self.expect_dismiss(TokenKind::EndScan)?;
        self.expect_dismiss(TokenKind::SemiColon)?;
        Ok(Scan::new(name, params))
    }

    /// Parse a [`Global`] from the token stream
    pub fn parse_global(&mut self) -> Result<Global, ParseError> {
        match self.peek() {
            Some(t) => {
                if t.source != "GLOBAL" {
                    return Err(ParseError::new_at("Expected to find a GLOBAL block", self.tokenizer.whole, self.tokenizer.byte));
                }
            }
            None => return Err(ParseError::new_at("Unexpected end of token stream", self.tokenizer.whole, self.tokenizer.byte)),
        };

        self.expect_dismiss(TokenKind::BlockName)?;
        self.expect_dismiss(TokenKind::SemiColon)?;
        let mut refs = Vec::new();
        while self.peek_kind() == Some(TokenKind::Ref) && self.peek_kind().is_some() {
            refs.push(self.parse_ref()?);
        }

        Ok(Global::from(refs))
    }

    /// Parse a [`HighLevelBlock`] from the token stream
    pub fn parse_highlevel_block(&mut self) -> Result<HighLevelBlock, ParseError> {
        let name = self.expect(TokenKind::BlockName)?.source.to_string();
        self.expect_dismiss(TokenKind::SemiColon)?;
        let mut defs = Vec::new();
        while self.peek_kind() != Some(TokenKind::BlockName) && self.peek_kind().is_some() {
            defs.push(self.parse_highlevel_def()?);
        }

        Ok(HighLevelBlock::new(name, defs))
    }

    /// Parse a [`Block`] from the token stream
    pub fn parse_primitive_block(&mut self) -> Result<Block, ParseError> {
        let name = self.expect(TokenKind::BlockName)?.source.to_string();
        self.expect_dismiss(TokenKind::SemiColon)?;
        let mut defs = Vec::new();
        while self.peek_kind() != Some(TokenKind::BlockName) && self.peek_kind().is_some() {
            defs.push(self.parse_primitive_def()?);
        }

        Ok(Block::new(name, defs))
    }

    /// Parse a [`Schedule`] from the token stream
    pub fn parse_schedule(&mut self) -> Result<Schedule, ParseError> {
        match self.peek() {
            Some(t) => {
                if t.source != "SCHED" {
                    return Err(ParseError::new_at("Expected to find a SCHED block", self.tokenizer.whole, self.tokenizer.byte));
                }
            }
            None => return Err(ParseError::new_at("Unexpected end of token stream", self.tokenizer.whole, self.tokenizer.byte)),
        };

        self.expect_dismiss(TokenKind::BlockName)?;
        self.expect_dismiss(TokenKind::SemiColon)?;
        let mut scans = Vec::new();
        while self.peek_kind() != Some(TokenKind::BlockName) && self.peek_kind().is_some() {
            scans.push(self.parse_scan()?);
        }

        Ok(Schedule::from(scans))
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_parse_param() -> Result<(), VexError> {
        let inp = "some_param = : with : 20 m : values;";
        let mut parser = Parser::new(inp);
        let expected = Param::new(
            "some_param",
            [
                ParamValue::empty(),
                ParamValue::new_unchecked("with", ParamValueKind::Ident),
                ParamValue::new_unchecked("20 m", ParamValueKind::NumberUnit),
                ParamValue::new_unchecked("values", ParamValueKind::Ident),
            ],
        );
        assert_eq!(parser.parse_param()?, expected);
        Ok(())
    }

    #[test]
    fn test_parse_param_values_link() -> Result<(), VexError> {
        let inp = "&mylink";
        let mut parser = Parser::new(inp);
        let expected = ParamValue::new_unchecked("mylink", crate::param::ParamValueKind::Link);
        assert_eq!(parser.parse_param_value()?, expected);
        Ok(())
    }

    #[test]
    fn test_parse_ref() -> Result<(), VexError> {
        let inp = "ref $MYBLOCKNAME = keyword;";
        let mut parser = Parser::new(inp);
        let expected = VexRef::new("MYBLOCKNAME", "keyword");
        assert_eq!(parser.parse_ref()?, expected);
        Ok(())
    }

    #[test]
    fn test_parse_ref_with_qualifiers() -> Result<(), VexError> {
        let inp = "ref $MYBLOCKNAME = keyword : with : some : qualifiers;";
        let mut parser = Parser::new(inp);
        let expected = VexRef::new_with_quals("MYBLOCKNAME", "keyword", ["with", "some", "qualifiers"]);
        assert_eq!(parser.parse_ref()?, expected);
        Ok(())
    }

    #[test]
    fn test_parse_highlevel_def() -> Result<(), VexError> {
        let inp = "
            def my_def;
                ref $MYBLOCK1 = mykey1;
                ref $MYBLOCK2 = mykey2 : with : some: qualifiers;
            enddef;
        ";
        let mut parser = Parser::new(inp);
        let expected = HighLevelDef::new(
            "my_def",
            [
                VexRef::new("MYBLOCK1", "mykey1"),
                VexRef::new_with_quals("MYBLOCK2", "mykey2", ["with", "some", "qualifiers"]),
            ],
        );
        assert_eq!(parser.parse_highlevel_def()?, expected);
        Ok(())
    }

    #[test]
    fn test_parse_primitive_def() -> Result<(), VexError> {
        let inp = "
            def my_def;
                param1 = myvalue1;
                param2 = myvalue2 : with: 20 m : values : ;
            enddef;
        ";
        let mut parser = Parser::new(inp);
        let expected = Def::new(
            "my_def",
            [
                Param::new("param1", [ParamValue::new_unchecked("myvalue1", ParamValueKind::Ident)]),
                Param::new(
                    "param2",
                    [
                        ParamValue::new_unchecked("myvalue2", ParamValueKind::Ident),
                        ParamValue::new_unchecked("with", ParamValueKind::Ident),
                        ParamValue::new_unchecked("20 m", ParamValueKind::NumberUnit),
                        ParamValue::new_unchecked("values", ParamValueKind::Ident),
                        ParamValue::empty(),
                    ],
                ),
            ],
        );
        assert_eq!(parser.parse_primitive_def()?, expected);
        Ok(())
    }

    #[test]
    fn test_parse_scan() -> Result<(), VexError> {
        let inp = "
            scan my_scan;
                station = station1;
                source = mysource2 : with: 20 m : values : ;
            endscan;
        ";
        let mut parser = Parser::new(inp);
        let expected = Scan::new(
            "my_scan",
            [
                Param::new("station", [ParamValue::new_unchecked("station1", ParamValueKind::Ident)]),
                Param::new(
                    "source",
                    [
                        ParamValue::new_unchecked("mysource2", ParamValueKind::Ident),
                        ParamValue::new_unchecked("with", ParamValueKind::Ident),
                        ParamValue::new_unchecked("20 m", ParamValueKind::NumberUnit),
                        ParamValue::new_unchecked("values", ParamValueKind::Ident),
                        ParamValue::empty(),
                    ],
                ),
            ],
        );
        assert_eq!(parser.parse_scan()?, expected);
        Ok(())
    }

    #[test]
    fn test_parse_global_terminated() -> Result<(), VexError> {
        let inp = "
            $GLOBAL;
                ref $BLOCK1 = MYKEY1;
                ref $BLOCK2 = some_other_value;

            $TERMINATINGBLOCK;
        ";
        let mut parser = Parser::new(inp);
        let expected = Global::from([VexRef::new("BLOCK1", "MYKEY1"), VexRef::new("BLOCK2", "some_other_value")]);
        assert_eq!(parser.parse_global()?, expected);
        Ok(())
    }

    #[test]
    fn test_parse_global_unterminated() -> Result<(), VexError> {
        let inp = "
            $GLOBAL;
                ref $BLOCK1 = MYKEY1;
                ref $BLOCK2 = some_other_value;
        ";
        let mut parser = Parser::new(inp);
        let expected = Global::from([VexRef::new("BLOCK1", "MYKEY1"), VexRef::new("BLOCK2", "some_other_value")]);
        assert_eq!(parser.parse_global()?, expected);
        Ok(())
    }

    #[test]
    fn test_parse_highlevel_block() -> Result<(), VexError> {
        let inp = "
            $MYBLOCK;
            def mydef1;
                ref $BLOCK1 = somekey1;
                ref $BLOCK2 = somekey2 : with : qualifiers;
            enddef;

            def mydef2;
                ref $BLOCK3 = somekey3;
                ref $BLOCK4 = somekey4 : with : qualifiers;
            enddef;
        ";
        let mut parser = Parser::new(inp);
        let expected = HighLevelBlock::new(
            "MYBLOCK",
            [
                HighLevelDef::new(
                    "mydef1",
                    [VexRef::new("BLOCK1", "somekey1"), VexRef::new_with_quals("BLOCK2", "somekey2", ["with", "qualifiers"])],
                ),
                HighLevelDef::new(
                    "mydef2",
                    [VexRef::new("BLOCK3", "somekey3"), VexRef::new_with_quals("BLOCK4", "somekey4", ["with", "qualifiers"])],
                ),
            ],
        );
        assert_eq!(parser.parse_highlevel_block()?, expected);
        Ok(())
    }
}
