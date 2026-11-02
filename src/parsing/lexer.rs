use crate::error::{ParseError, ParseErrorKind};
use crate::parsing::token::{Token, TokenKind};

/// Produces a series of [`Token`]s from a &[`str`]
///
/// The lexer takes a string slice and implements an [`Iterator`] that yields Vex [`Token`]s.
#[derive(Debug, PartialEq, Eq)]
pub struct Lexer<'de> {
    whole: &'de str,
    // The currently hovered byte in whole
    byte: usize,
}

impl<'de> Lexer<'de> {
    /// Construct a new Lexer
    pub fn new(input: &'de str) -> Self {
        Self {
            whole: input,
            byte: 0,
        }
    }

    /// Convert the entire input into tokens
    pub fn lex(self) -> Result<Vec<Token<'de>>, ParseError> {
        self.into_iter().collect()
    }

    /// Form a token spanning `whole[start..start + len]` and advance the cursor
    /// past it.
    ///
    /// `start` is passed explicitly because several lexers (numbers, floats, RA,
    /// Dec, epoch) advance the cursor while scanning the token's extent and then
    /// call `make` with the already-computed length. Slicing from `self.byte` in
    /// that case would read the bytes that follow the token.
    fn make(&mut self, start: usize, len: usize, kind: TokenKind) -> Token<'de> {
        debug_assert!(
            start + len <= self.whole.len(),
            "token length out of bounds"
        );

        let tok = Token {
            source: &self.whole[start..start + len],
            loc: start,
            kind,
        };
        self.byte = start + len;
        tok
    }

    fn remaining(&self) -> &'de str {
        &self.whole[self.byte..]
    }

    /// Peek at the next character without advancing.
    fn peek_char(&self) -> Option<char> {
        self.remaining().chars().next()
    }

    /// Expect to find a character that matches the predicate, returning it if
    /// `pred` evaluates to `true`. Also advance the internal cursor.
    /// Return an error on EOF or if the character
    /// does not match the predicate
    fn expect_char(&mut self, pred: impl Fn(char) -> bool) -> Result<char, ParseError> {
        match self.peek_char() {
            Some(peeked) => {
                if pred(peeked) {
                    self.byte += peeked.len_utf8();
                    Ok(peeked)
                } else {
                    Err(ParseError::new(
                        ParseErrorKind::UnexpectedCharacter(peeked),
                        self.whole,
                        self.byte,
                        self.byte,
                    ))
                }
            }
            None => Err(ParseError::new(
                ParseErrorKind::UnexpectedEof,
                self.whole,
                self.byte,
                self.byte,
            )),
        }
    }

    /// Advance the cursor past the next character and return it.
    fn bump(&mut self) -> Option<char> {
        let c = self.peek_char()?;
        self.byte += c.len_utf8();
        Some(c)
    }

    /// Advance the cursor past the next character only if it matches a predicate.
    fn bump_if(&mut self, pred: impl Fn(char) -> bool) -> Option<char> {
        let c = self.peek_char()?;
        if pred(c) {
            self.byte += c.len_utf8();
            Some(c)
        } else {
            None
        }
    }

    /// Consume and discard every character for which `pred` is true.
    fn consume(&mut self, pred: impl Fn(char) -> bool) {
        while let Some(c) = self.peek_char() {
            if pred(c) {
                self.byte += c.len_utf8();
            } else {
                return;
            }
        }
    }

    fn lex_literal(&mut self) -> Result<Token<'de>, ParseError> {
        // We're currently pointing at a '"' char

        // Consume the opening quote
        let start = self.byte;
        self.bump();
        // Consume everything that isn't the closing quote
        self.consume(|c| c != '"');
        // We must find a closing quote
        self.expect_char(|c| c == '"')?;

        let len = self.byte - start;

        // Form a token from the whole literal, excluding both DQMs
        let tok = self.make(start + 1, len - 2, TokenKind::Literal);
        // Correct the cursor
        self.byte = start + len;
        Ok(tok)
    }

    fn lex_comment(&mut self) -> Result<Token<'de>, ParseError> {
        // We're currently pointing at a '*' char
        //
        // This is a comment, so grab everything until
        // a newline

        // Advance past the '*'
        self.bump();
        // Now the comment starts
        let start = self.byte;
        // Take everything until the line end
        self.consume(|c| c != '\n');

        let len = self.byte - start;
        Ok(self.make(start, len, TokenKind::Comment))
    }

    fn lex_number(&mut self, start: usize) -> Result<Token<'de>, ParseError> {
        // We're currently pointing at a number
        // character: '0' .. '9'
        //
        // This might be:
        // - Integer
        // - Float
        // - RightAscension
        // - Declination
        // - Epoch
        // - Identifier

        self.byte = start;

        // First take all the leading digits
        self.consume(|c| c.is_ascii_digit());

        if let Some(c) = self.peek_char() {
            match c {
                // It's definitely a float
                '.' | 'e' | 'E' => self.lex_float(start),
                // It's the 'h' in the hms format -> RA
                'h' => self.lex_ra(start),
                // It's the 'd' in the dms or d'" format -> Dec
                'd' => self.lex_dec(start),
                // It's the 'y' in the ydhms format
                'y' => self.lex_epoch(start),
                // It's just an integer
                ' ' => Ok(self.make(start, self.byte - start, TokenKind::Integer)),
                // Otherwise it's definitely an identifier
                _ => self.lex_ident(start),
            }
        } else {
            // Otherwise we reached EOF, and it's just an integer
            Ok(self.make(start, self.byte - start, TokenKind::Integer))
        }
    }

    fn lex_float(&mut self, start: usize) -> Result<Token<'de>, ParseError> {
        // This function is only called when already
        // know we're looking at a float, so rewind and walk through
        // it as a float

        self.byte = start;

        // Take all the leading digits again
        self.consume(|c| c.is_ascii_digit());

        // There must be an '.' or 'e' or 'E' otherwise this function wouldn't be called
        // so take it
        self.expect_char(|c| matches!(c, '.' | 'e' | 'E'))?;

        // Take any more digits
        self.consume(|c| c.is_ascii_digit());

        // Now we must be looking at a 'e' or 'E' or we're done
        if self.bump_if(|c| matches!(c, 'e' | 'E')).is_none() {
            return Ok(self.make(start, self.byte - start, TokenKind::Float));
        }

        // Now there must be an optional '+' or '-', followed by at least one digit
        self.bump_if(|c| matches!(c, '+' | '-'));
        self.expect_char(|c| c.is_ascii_digit())?;
        self.consume(|c| c.is_ascii_digit());

        Ok(self.make(start, self.byte - start, TokenKind::Float))
    }

    fn lex_ra(&mut self, start: usize) -> Result<Token<'de>, ParseError> {
        // We're definitely looking at a right ascension in
        // hms format

        self.byte = start;

        self.consume(|c| c.is_ascii_digit());
        self.expect_char(|c| c == 'h')?;
        self.consume(|c| c.is_ascii_digit());
        self.expect_char(|c| c == 'm')?;
        self.consume(|c| c.is_ascii_digit());
        if self.bump_if(|c| c == '.').is_some() {
            self.consume(|c| c.is_ascii_digit());
        }
        self.expect_char(|c| c == 's')?;

        Ok(self.make(start, self.byte - start, TokenKind::RightAscension))
    }

    fn lex_dec(&mut self, start: usize) -> Result<Token<'de>, ParseError> {
        // We're definitely looking at a declination in
        // d'" format
        self.byte = start;

        self.consume(|c| c.is_ascii_digit());
        self.expect_char(|c| c == 'd')?;
        self.consume(|c| c.is_ascii_digit());
        self.expect_char(|c| c == '\'')?;
        self.consume(|c| c.is_ascii_digit());
        if self.bump_if(|c| c == '.').is_some() {
            self.consume(|c| c.is_ascii_digit());
        }
        self.expect_char(|c| c == '"')?;

        Ok(self.make(start, self.byte - start, TokenKind::Declination))
    }

    fn lex_epoch(&mut self, start: usize) -> Result<Token<'de>, ParseError> {
        // We're definitely looking at an epoch in ydhms format

        self.byte = start;

        self.consume(|c| c.is_ascii_digit());
        self.expect_char(|c| c == 'y')?;
        self.consume(|c| c.is_ascii_digit());
        self.expect_char(|c| c == 'd')?;
        self.consume(|c| c.is_ascii_digit());
        self.expect_char(|c| c == 'h')?;
        self.consume(|c| c.is_ascii_digit());
        self.expect_char(|c| c == 'm')?;
        self.consume(|c| c.is_ascii_digit());
        if self.bump_if(|c| c == '.').is_some() {
            self.consume(|c| c.is_ascii_digit());
        }
        self.expect_char(|c| c == 's')?;

        Ok(self.make(start, self.byte - start, TokenKind::Epoch))
    }

    fn lex_ident(&mut self, start: usize) -> Result<Token<'de>, ParseError> {
        self.byte = start;
        self.consume(is_ident_char);
        Ok(self.make(start, self.byte - start, TokenKind::Ident))
    }

    fn lex_kw_or_ident(&mut self, start: usize) -> Result<Token<'de>, ParseError> {
        self.byte = start;
        self.consume(is_ident_char);
        let source = &self.whole[start..self.byte];
        match get_kw(source) {
            Some(kind) => Ok(self.make(start, self.byte - start, kind)),
            None => Ok(self.make(start, self.byte - start, TokenKind::Ident)),
        }
    }
}

fn is_ident_char(c: char) -> bool {
    !c.is_whitespace() && !matches!(c, ':' | ';' | '*' | '=' | '&' | '$' | '"')
}

impl<'de> Iterator for Lexer<'de> {
    type Item = Result<Token<'de>, ParseError>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            let c = self.peek_char()?;

            return match c {
                ';' => Some(Ok(self.make(self.byte, 1, TokenKind::SemiColon))),
                ':' => Some(Ok(self.make(self.byte, 1, TokenKind::Colon))),
                '=' => Some(Ok(self.make(self.byte, 1, TokenKind::Equals))),
                '$' => Some(Ok(self.make(self.byte, 1, TokenKind::Dollar))),
                '&' => Some(Ok(self.make(self.byte, 1, TokenKind::Ampersand))),
                '+' => Some(Ok(self.make(self.byte, 1, TokenKind::Plus))),
                '-' => Some(Ok(self.make(self.byte, 1, TokenKind::Minus))),
                '"' => Some(self.lex_literal()),
                '*' => Some(self.lex_comment()),
                '0'..='9' => Some(self.lex_number(self.byte)),
                // Whitespace is skipped; an end-of-line counts as a single space.
                c if c.is_whitespace() => {
                    self.bump();
                    continue;
                }
                _ => Some(self.lex_kw_or_ident(self.byte)),
            };
        }
    }
}

fn get_kw(s: &str) -> Option<TokenKind> {
    match s {
        "def" => Some(TokenKind::Def),
        "enddef" => Some(TokenKind::EndDef),
        "scan" => Some(TokenKind::Scan),
        "endscan" => Some(TokenKind::EndScan),
        "ref" => Some(TokenKind::Ref),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // For shorter token construction
    macro_rules! token {
        ($src:literal, $loc:literal, $kind:expr) => {
            Token {
                source: $src,
                loc: $loc,
                kind: $kind,
            }
        };
    }

    #[test]
    fn test_lex_numbers() {
        let s = "1234 0.1234 1.234e5 1.234E-5";
        let mut t = Lexer::new(s).map(|t| t.unwrap());

        assert_eq!(t.next(), Some(token!("1234", 0, TokenKind::Integer)));
        assert_eq!(t.next(), Some(token!("0.1234", 5, TokenKind::Float)));
        assert_eq!(t.next(), Some(token!("1.234e5", 12, TokenKind::Float)));
        assert_eq!(t.next(), Some(token!("1.234E-5", 20, TokenKind::Float)));
        assert_eq!(t.next(), None)
    }

    #[test]
    fn test_lex_kws() {
        let s = "def enddef scan endscan ref";
        let mut t = Lexer::new(s).map(|t| t.unwrap());

        assert_eq!(t.next(), Some(token!("def", 0, TokenKind::Def)));
        assert_eq!(t.next(), Some(token!("enddef", 4, TokenKind::EndDef)));
        assert_eq!(t.next(), Some(token!("scan", 11, TokenKind::Scan)));
        assert_eq!(t.next(), Some(token!("endscan", 16, TokenKind::EndScan)));
        assert_eq!(t.next(), Some(token!("ref", 24, TokenKind::Ref)));
        assert_eq!(t.next(), None)
    }

    #[test]
    fn test_lex_literal() {
        let s = "\"Foo Bar\"; \"Bar Foo\"";
        let mut t = Lexer::new(s).map(|t| t.unwrap());

        assert_eq!(t.next(), Some(token!("Foo Bar", 1, TokenKind::Literal)));
        assert_eq!(t.next(), Some(token!(";", 9, TokenKind::SemiColon)));
        assert_eq!(t.next(), Some(token!("Bar Foo", 12, TokenKind::Literal)));
        assert_eq!(t.next(), None)
    }

    #[test]
    fn test_lex_idents_with_comment() {
        let s = "* Some really important comment\nsome_ident myident _testident what#even/;";
        let mut t = Lexer::new(s).map(|t| t.unwrap());

        assert_eq!(
            t.next(),
            Some(token!(
                " Some really important comment",
                1,
                TokenKind::Comment
            ))
        );
        assert_eq!(t.next(), Some(token!("some_ident", 32, TokenKind::Ident)));
        assert_eq!(t.next(), Some(token!("myident", 43, TokenKind::Ident)));
        assert_eq!(t.next(), Some(token!("_testident", 51, TokenKind::Ident)));
        assert_eq!(t.next(), Some(token!("what#even/", 62, TokenKind::Ident)));
        assert_eq!(t.next(), Some(token!(";", 72, TokenKind::SemiColon)));
        assert_eq!(t.next(), None)
    }

    #[test]
    fn test_lex_source_name() {
        let s = "0123+4567 0123-4567";
        let mut t = Lexer::new(s).map(|t| t.unwrap());

        assert_eq!(t.next(), Some(token!("0123+4567", 0, TokenKind::Ident)));
        assert_eq!(t.next(), Some(token!("0123-4567", 10, TokenKind::Ident)));
        assert_eq!(t.next(), None)
    }

    #[test]
    fn test_lex_ra_dec() {
        let s = "05h23m2.56s 12h23m21s -01d59'23\" 23d23'23.333\"";
        let mut t = Lexer::new(s).map(|t| t.unwrap());

        assert_eq!(
            t.next(),
            Some(token!("05h23m2.56s", 0, TokenKind::RightAscension))
        );
        assert_eq!(
            t.next(),
            Some(token!("12h23m21s", 12, TokenKind::RightAscension))
        );
        assert_eq!(t.next(), Some(token!("-", 22, TokenKind::Minus)));
        assert_eq!(
            t.next(),
            Some(token!("01d59'23\"", 23, TokenKind::Declination))
        );
        assert_eq!(
            t.next(),
            Some(token!("23d23'23.333\"", 33, TokenKind::Declination))
        );
        assert_eq!(t.next(), None)
    }

    #[test]
    fn test_lex_epoch() {
        let s = "2020y014d12h00m00s 2025y014d12h00m30.00s";
        let mut t = Lexer::new(s).map(|t| t.unwrap());

        assert_eq!(
            t.next(),
            Some(token!("2020y014d12h00m00s", 0, TokenKind::Epoch))
        );
        assert_eq!(
            t.next(),
            Some(token!("2025y014d12h00m30.00s", 19, TokenKind::Epoch))
        );
        assert_eq!(t.next(), None)
    }
}
