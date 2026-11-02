use std::fmt::Display;

/// A Vex Token
///
/// A token is fundamentally a reference into some other string slice, with an associated byte location (`loc`),
/// and an associated [`TokenKind`]. Tokens are produced by the [`Lexer`](crate::parsing::Lexer).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Token<'de> {
    /// The referenced string slice
    pub source: &'de str,
    /// The byte location of this token
    pub loc: usize,
    /// The kind of this token
    pub kind: TokenKind,
}

impl<'de> Token<'de> {
    /// Return true if this token is of kind `SemiColon`.
    pub fn is_line_end(&self) -> bool {
        self.kind == TokenKind::SemiColon
    }

    //     /// Return true if this token is of kind `Colon`.
    //     pub fn is_param_separator(&self) -> bool {
    //         self.kind == TokenKind::Colon
    //     }
    //
    //     /// Return true if this token is of kind `Equals`.
    //     pub fn is_equals(&self) -> bool {
    //         self.kind == TokenKind::Equals
    //     }
    //
    //     /// Return true if this token is of kind `Comment`.
    //     pub fn is_comment(&self) -> bool {
    //         self.kind == TokenKind::Comment
    //     }

    /// Return the length of the string slice that this token refers to.
    pub fn len(&self) -> usize {
        self.source.len()
    }

    //     /// Returns true if the referenced slice has a length of zero bytes.
    //     pub fn is_empty(&self) -> bool {
    //         self.source.is_empty()
    //     }
    //
    //     /// Return the byte range of the string slice this token refers to.
    //     pub fn loc_range(&self) -> Range<usize> {
    //         self.loc..self.loc + self.len()
    //     }
}

#[allow(missing_docs)]
/// The types of tokens found in the Vex format
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TokenKind {
    // Punctuation
    SemiColon,
    Colon,
    Equals,
    Dollar,
    Ampersand,
    Plus,
    Minus,

    // Keywords
    Def,
    EndDef,
    Ref,
    Scan,
    EndScan,

    // Standard Types
    Integer,
    Float,
    Epoch,
    RightAscension,
    Declination,
    Ident,
    Literal,

    // Comment
    Comment,
}

impl Display for TokenKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::SemiColon => "SEMICOLON",
                Self::Colon => "COLON",
                Self::Equals => "EQUALS",
                Self::Dollar => "DOLLAR",
                Self::Ampersand => "AMPERSAND",
                Self::Plus => "PLUS",
                Self::Minus => "MINUS",
                Self::Def => "DEF",
                Self::EndDef => "ENDDEF",
                Self::Ref => "REF",
                Self::Scan => "SCAN",
                Self::EndScan => "ENDSCAN",
                Self::Integer => "INTEGER",
                Self::Float => "FLOAT",
                Self::Epoch => "EPOCH",
                Self::RightAscension => "RIGHTASCENSION",
                Self::Declination => "DECLINATION",
                Self::Ident => "IDENTIFIER",
                Self::Literal => "LITERAL",
                Self::Comment => "COMMENT",
            }
        )
    }
}
