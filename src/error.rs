//! Contains the two error types provided by `vexio`, a top level error [`VexError`], and a lower level
//! [`ParseError`] containing detailed information about the souce of a syntax error

use std::fmt::Debug;
use std::ops::Range;

#[cfg(feature = "miette")]
use miette::{Diagnostic, LabeledSpan, SourceCode, SourceOffset, SourceSpan};
use thiserror::Error;

use crate::parsing::TokenKind;

/// Indicates a range of failures when interacting with Vex files
///
/// If the crate feature `miette` is enabled, VexError implements [`miette::Diagnostic`] and can be used to print
/// detailed error messages to the terminal for user facing applications
#[allow(missing_docs)]
#[derive(Debug, Error)]
pub enum VexError {
    #[error(transparent)]
    ParseError(#[from] ParseError),
    #[error(transparent)]
    IoError(#[from] std::io::Error),
}

#[cfg(feature = "miette")]
impl Diagnostic for VexError {
    fn code<'a>(&'a self) -> Option<Box<dyn std::fmt::Display + 'a>> {
        match self {
            // Self::MissingVexRev => Some(Box::new("vexio::missing_vex_rev")),
            // Self::DuplicateDef(_) => Some(Box::new("vexio::duplicate_def")),
            // Self::DuplicateBlock(_) => Some(Box::new("vexio::duplicate_block")),
            // Self::MissingBlock(_) => Some(Box::new("vexio::missing_block")),
            Self::IoError(_) => Some(Box::new("std::io::Error")),
            Self::ParseError(e) => e.code(),
        }
    }

    fn labels(&self) -> Option<Box<dyn Iterator<Item = miette::LabeledSpan> + '_>> {
        match self {
            Self::ParseError(e) => e.labels(),
            _ => None,
        }
    }

    fn source_code(&self) -> Option<&dyn SourceCode> {
        match self {
            Self::ParseError(e) => e.source_code(),
            _ => None,
        }
    }
}

/// Describes the type of [`ParseError`].
#[derive(Debug, Error, Clone)]
pub enum ParseErrorKind {
    /// The required `GLOBAL` block is absent or invalid.
    #[error("Missing GLOBAL block definition")]
    MissingGlobal,
    /// An entry does not contain the expected value structure.
    #[error("Invalid entry")]
    InvalidEntry,
    /// The `VEX_rev` declaration is absent or invalid.
    #[error("Missing or invalid VEX_rev definition")]
    InvalidVexRev,
    /// A token other than the expected token was encountered.
    #[error("Unexpected token, expecting {} but found {}", .expecting, .found)]
    UnexpectedToken {
        /// The token expected by the parser.
        expecting: TokenKind,
        /// The token encountered by the parser.
        found: TokenKind,
    },
    /// A token was encountered where no token was expected.
    #[error("Unexpected token found: {}", .found)]
    UnexpectedTokenFound {
        /// The unexpected token.
        found: TokenKind,
    },
    /// An invalid character was encountered.
    #[error("Unexpected character: {}", .0)]
    UnexpectedCharacter(char),
    /// A string literal was not terminated.
    #[error("Unterminated string literal")]
    UnterminatedLiteral,
    /// A numeric value could not be parsed.
    #[error("Malformed number")]
    MalformedNumber,
    /// The input ended before parsing was complete.
    #[error("Unexpected EOF")]
    UnexpectedEof,
}

/// Indicates a failure to parse a Vex data structure from some input
///
/// Contains detailed information on the location of the error, and provides a description
/// of the cause of the error
///
/// If the crate feature `miette` is enabled, ParseError implements [`miette::Diagnostic`] and can be used to print
/// detailed error messages to the terminal for user facing applications
#[derive(Debug, Clone, Error)]
#[error("Failed to parse Vex at line {line}, column {col}: {}", .kind.to_string())]
pub struct ParseError {
    kind: ParseErrorKind,
    src: String,
    start: usize,
    end: usize,
    line: usize,
    col: usize,
}

impl ParseError {
    /// Make a new error with a textual description of the error `desc`, the source of the error `src`, and the _byte_ start and end locations
    pub fn new(kind: ParseErrorKind, src: impl AsRef<str>, start: usize, end: usize) -> Self {
        let (line, col) = count_lines_cols(&src, start);
        Self {
            kind,
            src: src.as_ref().to_string(),
            start,
            end,
            line,
            col,
        }
    }

    /// The source text that generated the error
    pub fn source(&self) -> &str {
        &self.src
    }

    /// The [`ParseErrorKind`] of this error
    pub fn kind(&self) -> &ParseErrorKind {
        &self.kind
    }

    /// The byte start location of the error
    pub fn byte_start(&self) -> usize {
        self.start
    }

    /// The byte end location of the error
    pub fn byte_end(&self) -> usize {
        self.end
    }

    /// The byte range of the error
    pub fn byte_range(&self) -> Range<usize> {
        self.start..self.end
    }

    /// The line number containing the start of the error
    pub fn line(&self) -> usize {
        self.line
    }

    /// The column containing the start of the error
    pub fn col(&self) -> usize {
        self.col
    }
}

#[cfg(feature = "miette")]
impl Diagnostic for ParseError {
    fn code<'a>(&'a self) -> Option<Box<dyn std::fmt::Display + 'a>> {
        Some(Box::new("vexio::error::ParseError"))
    }

    fn labels(&self) -> Option<Box<dyn Iterator<Item = miette::LabeledSpan> + '_>> {
        let span = SourceSpan::new(SourceOffset::from(self.start), self.end - self.start);
        let label = LabeledSpan::new(
            Some("here".to_string()),
            span.offset(),
            self.end - self.start,
        );
        Some(Box::new(std::iter::once(label)))
    }

    fn source_code(&self) -> Option<&dyn SourceCode> {
        Some(&self.src)
    }
}

fn count_lines_cols(src: impl AsRef<str>, byte_offset: usize) -> (usize, usize) {
    let mut lines = 0usize;
    let mut cols = 0usize;
    let mut offset = 0usize;

    let char_iter = src.as_ref().chars();
    for c in char_iter {
        if offset >= byte_offset {
            break;
        } else {
            if c == '\n' {
                lines += 1;
                cols = 0;
            } else {
                cols += 1;
            }
        }
        offset += c.len_utf8()
    }

    (lines, cols)
}
