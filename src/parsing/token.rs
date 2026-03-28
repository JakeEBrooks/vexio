use std::fmt::Display;
use std::ops::Range;

/// A Vex Token
///
/// A token is fundamentally a reference into some other string slice, with an associated byte location (`loc`),
/// and an associated [`TokenKind`]. Tokens are produced primarily by the [`Tokenizer`](crate::parsing::Tokenizer).
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

    /// Return true if this token is of kind `Colon`.
    pub fn is_param_separator(&self) -> bool {
        self.kind == TokenKind::Colon
    }

    /// Return true if this token is of kind `Equals`.
    pub fn is_equals(&self) -> bool {
        self.kind == TokenKind::Equals
    }

    /// Return true if this token is of kind `BlockName`.
    pub fn is_blockname(&self) -> bool {
        self.kind == TokenKind::BlockName
    }

    /// Return true if this token refers to a valid high level Vex block.
    pub fn is_high_level_blockname(&self) -> bool {
        crate::HIGH_LEVEL_BLOCKS.contains(&self.source)
    }

    /// Return true if this token refers to a valid primitive Vex block.
    pub fn is_primitive_blockname(&self) -> bool {
        crate::PRIMITIVE_BLOCKS.contains(&self.source)
    }

    /// Return true if this token is of kind `Comment`.
    pub fn is_comment(&self) -> bool {
        self.kind == TokenKind::Comment
    }

    /// Return the length of the string slice that this token refers to.
    pub fn len(&self) -> usize {
        self.source.len()
    }

    /// Return the `len` of the referenced string slice.
    pub fn is_empty(&self) -> bool {
        self.source.is_empty()
    }

    /// Return the byte range of the string slice this token refers to.
    pub fn loc_range(&self) -> Range<usize> {
        self.loc..self.loc + self.len()
    }
}

#[allow(missing_docs)]
/// The types of tokens found in the Vex format
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TokenKind {
    // Control characters
    SemiColon,
    Colon,
    Equals,

    // Keywords
    Def,
    EndDef,
    Ref,
    Scan,
    EndScan,

    // Parameter types
    Number,
    Unit,
    Ident,
    Literal,
    Epoch,
    Coord,
    Link,

    // Other
    BlockName,
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
                Self::Def => "DEF",
                Self::EndDef => "ENDDEF",
                Self::Ref => "REF",
                Self::Scan => "SCAN",
                Self::EndScan => "ENDSCAN",
                Self::Number => "NUMBER",
                Self::Unit => "UNIT",
                Self::Ident => "IDENTIFIER",
                Self::Literal => "LITERAL",
                Self::Epoch => "EPOCH",
                Self::Coord => "COORDINATE",
                Self::Link => "LINK",
                Self::BlockName => "BLOCKNAME",
                Self::Comment => "COMMENT",
            }
        )
    }
}
