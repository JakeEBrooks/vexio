//! Contains the two error types provided by `vexio`, a top level error [`VexError`], and a lower level
//! [`ParseError`] containing detailed information about the souce of a syntax error

use std::fmt::Debug;
use std::ops::Range;

#[cfg(feature = "miette")]
use miette::{Diagnostic, LabeledSpan, SourceCode, SourceOffset, SourceSpan};
use thiserror::Error;

/// Indicates a range of failures when interacting with Vex files
///
/// This is the primary error type of `vexio` and includes [`ParseError`].
///
/// If the crate feature `miette` is enabled, VexError implements [`miette::Diagnostic`] and can be used to print
/// detailed error messages to the terminal for user facing applications
#[allow(missing_docs)]
#[derive(Debug, Error)]
pub enum VexError {
    #[error("Failed to parse VEX_rev, are you sure this is a Vex file?")]
    MissingVexRev,
    #[error("Could not find a valid {} block", .0)]
    MissingBlock(String),
    #[error("Found duplicate definitions of {}", .0)]
    DuplicateDef(String),
    #[error("Found duplicate block definitions of {}", .0)]
    DuplicateBlock(String),
    #[error(transparent)]
    ParseError(#[from] ParseError),
    #[error(transparent)]
    IoError(#[from] std::io::Error),
}

#[cfg(feature = "miette")]
impl Diagnostic for VexError {
    fn code<'a>(&'a self) -> Option<Box<dyn std::fmt::Display + 'a>> {
        match self {
            Self::MissingVexRev => Some(Box::new("vexio::missing_vex_rev")),
            Self::DuplicateDef(_) => Some(Box::new("vexio::duplicate_def")),
            Self::DuplicateBlock(_) => Some(Box::new("vexio::duplicate_block")),
            Self::MissingBlock(_) => Some(Box::new("vexio::missing_block")),
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

/// Indicates a failure to parse some Vex data structure from
/// a string of characters
///
/// Contains detailed information on the location of the syntax error, and provides a description
/// of the cause of the error
///
/// If the crate feature `miette` is enabled, ParseError implements [`miette::Diagnostic`] and can be used to print
/// detailed error messages to the terminal for user facing applications
#[derive(Debug, Error)]
#[error("Failed to parse Vex at line {line}, column {col}: {desc}")]
pub struct ParseError {
    desc: String,
    src: String,
    start: usize,
    end: usize,
    line: usize,
    col: usize,
}

impl ParseError {
    /// Make a new error with a textual description of the error `desc`, the source of the error `source`, and the _byte_ start and end locations
    pub fn new(desc: impl AsRef<str>, source: impl AsRef<str>, start: usize, end: usize) -> Self {
        let (line, col) = count_lines_cols(&source, start);
        Self {
            desc: desc.as_ref().to_string(),
            src: source.as_ref().to_string(),
            start,
            end,
            line,
            col,
        }
    }

    /// Make a new error with a textual description of the error `desc`, the source of the error `source`, and the _byte_ location
    pub fn new_at(desc: impl AsRef<str>, source: impl AsRef<str>, loc: usize) -> Self {
        Self::new(desc, source, loc, loc)
    }

    /// Make a new error with a textual description of the error `desc`, the source of the error `source`, and the _byte_ start and end locations as a [`Range`]
    pub fn new_range(desc: impl AsRef<str>, source: impl AsRef<str>, at: Range<usize>) -> Self {
        Self::new(desc, source, at.start, at.end)
    }

    /// The error description
    pub fn description(&self) -> &str {
        &self.desc
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
        let label = LabeledSpan::new(Some("here".to_string()), span.offset(), self.end - self.start);
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
