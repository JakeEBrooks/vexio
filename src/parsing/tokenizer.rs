use crate::parsing::token::{Token, TokenKind};

/// Produces a series of [`Token`]s from a &[`str`]
///
/// The tokenizer takes a string slice and implements an [`Iterator`] that yields Vex [`Token`]s.
/// It is important to note that the Tokenizer cannot fail, hence the lack of any [`Result`] return type.
/// Instead, the fallback [`TokenKind`] is an identifier, so any substring that cannot be parsed into any other
/// kind of token will simply be produced as a [`TokenKind::Ident`].
#[derive(Debug, PartialEq, Eq)]
pub struct Tokenizer<'de> {
    /// The entire string reference
    pub whole: &'de str,
    /// The remaining string reference to be tokenized
    pub rest: &'de str,
    /// The current byte location
    pub byte: usize,
    peeked: Option<Token<'de>>,
}

impl<'de> Tokenizer<'de> {
    /// Construct a new Tokenizer
    pub fn new(input: &'de str) -> Self {
        Self {
            whole: input,
            rest: input,
            byte: 0,
            peeked: None,
        }
    }

    /// Peek ahead to the next token in the token stream
    pub fn peek(&mut self) -> Option<&Token<'de>> {
        if self.peeked.is_some() {
            return self.peeked.as_ref();
        }

        self.peeked = self.next();
        self.peeked.as_ref()
    }
}

impl<'de> Iterator for Tokenizer<'de> {
    type Item = Token<'de>;

    fn next(&mut self) -> Option<Self::Item> {
        if let Some(next) = self.peeked.take() {
            return Some(next);
        }

        loop {
            let mut chars = self.rest.chars();
            let c = chars.next()?;
            let c_onwards = self.rest;

            let startswith = match c {
                ';' => {
                    let token = Token {
                        source: &c_onwards[..1],
                        loc: self.byte,
                        kind: TokenKind::SemiColon,
                    };
                    self.rest = chars.as_str();
                    self.byte += c.len_utf8();
                    return Some(token);
                }
                ':' => {
                    let token = Token {
                        source: &c_onwards[..1],
                        loc: self.byte,
                        kind: TokenKind::Colon,
                    };
                    self.rest = chars.as_str();
                    self.byte += c.len_utf8();
                    return Some(token);
                }
                '=' => {
                    let token = Token {
                        source: &c_onwards[..1],
                        loc: self.byte,
                        kind: TokenKind::Equals,
                    };
                    self.rest = chars.as_str();
                    self.byte += c.len_utf8();
                    return Some(token);
                }
                '$' => StartsWith::Dollar,
                '"' => StartsWith::DoubleQuotation,
                '&' => StartsWith::Ampersand,
                '*' => StartsWith::Asterisk,
                '-' | '+' | '0'..='9' => StartsWith::MaybeNumber,
                c if c.is_whitespace() => {
                    self.rest = chars.as_str();
                    self.byte += c.len_utf8();
                    continue;
                }
                _ => StartsWith::MaybeIdent,
            };

            break match startswith {
                // A Block Name starts with a $ sign
                StartsWith::Dollar => {
                    let first_illegal = c_onwards[1..].find(|c| !is_valid_blockname_char(c)).unwrap_or(c_onwards.len() - 1);
                    let whole_blkname = &c_onwards[..first_illegal + 1];
                    let token = Token {
                        source: &whole_blkname[1..],
                        loc: self.byte,
                        kind: TokenKind::BlockName,
                    };
                    self.byte += whole_blkname.len();
                    self.rest = &self.rest[whole_blkname.len()..];

                    Some(token)
                }
                // A comment starts with a asterisk (*)
                StartsWith::Asterisk => {
                    let first_newline = c_onwards[1..].find('\n').unwrap_or(c_onwards.len() - 1);
                    let comment = &c_onwards[..first_newline];
                    self.byte += comment.len();
                    self.rest = &self.rest[comment.len()..];

                    self.next()
                }
                // A link name starts with an ampersand
                StartsWith::Ampersand => {
                    let first_illegal = c_onwards[1..].find(|c| !is_valid_linkname_char(c)).unwrap_or(c_onwards.len() - 1);
                    let whole_linkname = &c_onwards[..first_illegal + 1];
                    let token = Token {
                        source: &whole_linkname[1..],
                        loc: self.byte,
                        kind: TokenKind::Link,
                    };
                    self.byte += whole_linkname.len();
                    self.rest = &self.rest[whole_linkname.len()..];

                    Some(token)
                }
                // A literal string starts with double quotation marks
                StartsWith::DoubleQuotation => {
                    let first_dqm = c_onwards[1..].find('"').unwrap_or(c_onwards.len());
                    let whole_literal = &c_onwards[..first_dqm + 2];
                    let token = Token {
                        source: whole_literal,
                        loc: self.byte,
                        kind: TokenKind::Literal,
                    };
                    self.byte += whole_literal.len();
                    self.rest = &self.rest[whole_literal.len()..];

                    Some(token)
                }
                StartsWith::MaybeNumber => {
                    let first_illegal = c_onwards.find(|c: char| !is_valid_ident_char(c)).unwrap_or(c_onwards.len());
                    let whole_ident = &c_onwards[..first_illegal];
                    let curr_loc = self.byte;

                    self.byte += whole_ident.len();
                    self.rest = &self.rest[whole_ident.len()..];

                    // If it's a number return that
                    if is_number(whole_ident) {
                        return Some(Token {
                            source: whole_ident,
                            loc: curr_loc,
                            kind: TokenKind::Number,
                        });
                    }

                    // At this point it is either a Epoch or a Coordinate or a generic Identifier, so test both
                    if is_epoch(whole_ident) {
                        return Some(Token {
                            source: whole_ident,
                            loc: curr_loc,
                            kind: TokenKind::Epoch,
                        });
                    }

                    if is_coord(whole_ident) {
                        return Some(Token {
                            source: whole_ident,
                            loc: curr_loc,
                            kind: TokenKind::Coord,
                        });
                    }

                    // If it was none of the above, assume it's just an identifier
                    Some(Token {
                        source: whole_ident,
                        loc: curr_loc,
                        kind: TokenKind::Ident,
                    })
                }
                StartsWith::MaybeIdent => {
                    // We've accounted for other possibilities, so it's a keyword, a unit, or an identifier
                    let first_illegal = c_onwards.find(|c: char| !is_valid_ident_char(c)).unwrap_or(c_onwards.len());
                    let whole_ident = &c_onwards[..first_illegal];
                    let curr_loc = self.byte;

                    self.byte += whole_ident.len();
                    self.rest = &self.rest[whole_ident.len()..];

                    if let Some(k) = is_keyword(whole_ident) {
                        return Some(Token {
                            source: whole_ident,
                            loc: curr_loc,
                            kind: k,
                        });
                    }

                    if is_unit(whole_ident) {
                        return Some(Token {
                            source: whole_ident,
                            loc: curr_loc,
                            kind: TokenKind::Unit,
                        });
                    }

                    // Otherwise it's an identifier
                    Some(Token {
                        source: whole_ident,
                        loc: curr_loc,
                        kind: TokenKind::Ident,
                    })
                }
            };
        }
    }
}

enum StartsWith {
    Dollar,
    MaybeIdent,
    MaybeNumber,
    Asterisk,
    Ampersand,
    DoubleQuotation,
}

fn is_valid_blockname_char(c: char) -> bool {
    !matches!(c, ';' | ':' | '=') && !c.is_ascii_whitespace()
}

fn is_valid_linkname_char(c: char) -> bool {
    !matches!(c, ';' | ':' | '=' | '&' | '*' | '$') && !c.is_ascii_whitespace()
}

// Name OR Value
fn is_valid_ident_char(c: char) -> bool {
    !matches!(c, ';' | ':' | '=' | '&' | '*' | '$') && !c.is_ascii_whitespace()
}

pub(crate) fn is_keyword(source: &str) -> Option<TokenKind> {
    match source {
        "def" => Some(TokenKind::Def),
        "enddef" => Some(TokenKind::EndDef),
        "ref" => Some(TokenKind::Ref),
        "scan" => Some(TokenKind::Scan),
        "endscan" => Some(TokenKind::EndScan),
        _ => None,
    }
}

pub(crate) fn is_number(source: &str) -> bool {
    source.parse::<f64>().is_ok()
}

#[rustfmt::skip]
pub(crate) const VALID_UNITS: [&str; 36] = [
    "psec", "sec", "usec", "msec", "sec", "min", "hr", "day", "yr",
    "mHz", "Hz", "kHz", "MHz", "GHz",
    "ks", "Ms",
    "um", "mm", "cm", "m", "km", "in", "ft",
    "mdeg", "deg", "amin", "asec", "rad",
    "mJy", "Jy",
    "B", "kB", "MB", "GB", "TB", "PB"
];

pub(crate) fn is_unit(source: &str) -> bool {
    if let Some((lhs, rhs)) = source.split_once('/') {
        VALID_UNITS.contains(&lhs) && VALID_UNITS.contains(&rhs)
    } else {
        VALID_UNITS.contains(&source)
    }
}

// pub(crate) fn is_number_unit(source: &str) -> bool {
//     if let Some((lhs, rhs)) = source.split_once(" ") {
//         is_number(lhs) && is_unit(rhs)
//     } else {
//         false
//     }
// }

pub(crate) fn is_epoch(source: &str) -> bool {
    source.chars().all(|c| matches!(c, '0'..='9' | '.' | 'y' | 'd' | 'h' | 'm' | 's'))
}

pub(crate) fn is_coord(source: &str) -> bool {
    let is_ra = source.chars().all(|c| matches!(c, '+' | '-' | '0'..='9' | 'h' | 'm' | 's'));
    let is_dec = source.chars().all(|c| matches!(c, '+' | '-' | '0'..='9' | 'd' | '\'' | '"'));

    is_ra || is_dec
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tokenize_controls() {
        let inp = "; : =";
        let tokenizer = Tokenizer::new(inp);
        let tokens = tokenizer.map(|t| t.kind).collect::<Vec<TokenKind>>();
        assert_eq!(tokens, vec![TokenKind::SemiColon, TokenKind::Colon, TokenKind::Equals])
    }

    #[test]
    fn test_tokenize_keywords() {
        let inp = "def enddef ref scan endscan";
        let tokenizer = Tokenizer::new(inp);
        let tokens = tokenizer.map(|t| t.kind).collect::<Vec<TokenKind>>();
        assert_eq!(tokens, vec![TokenKind::Def, TokenKind::EndDef, TokenKind::Ref, TokenKind::Scan, TokenKind::EndScan])
    }

    #[test]
    fn test_tokenize_param_types() {
        let inp = "1234 m asec TB some_ident \"a literal\" 2024y200d12h14m35s +15d30'45\" 20h30m35.144s &my_link";
        let tokenizer = Tokenizer::new(inp);
        let tokens = tokenizer.collect::<Vec<Token>>();
        assert_eq!(
            tokens,
            vec![
                Token {
                    source: "1234",
                    loc: 0,
                    kind: TokenKind::Number
                },
                Token {
                    source: "m",
                    loc: 5,
                    kind: TokenKind::Unit
                },
                Token {
                    source: "asec",
                    loc: 7,
                    kind: TokenKind::Unit
                },
                Token {
                    source: "TB",
                    loc: 12,
                    kind: TokenKind::Unit
                },
                Token {
                    source: "some_ident",
                    loc: 15,
                    kind: TokenKind::Ident
                },
                Token {
                    source: "\"a literal\"",
                    loc: 26,
                    kind: TokenKind::Literal
                },
                Token {
                    source: "2024y200d12h14m35s",
                    loc: 38,
                    kind: TokenKind::Epoch
                },
                Token {
                    source: "+15d30'45\"",
                    loc: 57,
                    kind: TokenKind::Coord
                },
                Token {
                    source: "20h30m35.144s",
                    loc: 68,
                    kind: TokenKind::Epoch
                },
                Token {
                    source: "my_link",
                    loc: 82,
                    kind: TokenKind::Link
                },
            ]
        )
    }
}
