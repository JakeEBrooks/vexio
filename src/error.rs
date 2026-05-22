use std::fmt::Debug;
use std::ops::Range;

#[cfg(feature = "miette")]
use miette::{Diagnostic, LabeledSpan, SourceCode, SourceOffset, SourceSpan};
use thiserror::Error;

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

#[derive(Debug, Error)]
#[error("Failed to parse Vex: {desc}")]
pub struct ParseError {
    desc: String,
    src: String,
    start: usize,
    end: usize,
}

impl ParseError {
    pub fn new(desc: impl AsRef<str>, source: impl AsRef<str>, start: usize, end: usize) -> Self {
        Self {
            desc: desc.as_ref().to_string(),
            src: source.as_ref().to_string(),
            start,
            end,
        }
    }

    pub fn new_at(desc: impl AsRef<str>, source: impl AsRef<str>, loc: usize) -> Self {
        Self {
            desc: desc.as_ref().to_string(),
            src: source.as_ref().to_string(),
            start: loc,
            end: loc,
        }
    }

    pub fn new_range(desc: impl AsRef<str>, source: impl AsRef<str>, at: Range<usize>) -> Self {
        Self {
            desc: desc.as_ref().to_string(),
            src: source.as_ref().to_string(),
            start: at.start,
            end: at.end,
        }
    }

    pub fn description(&self) -> &str {
        &self.desc
    }

    pub fn byte_start(&self) -> usize {
        self.start
    }

    pub fn byte_end(&self) -> usize {
        self.end
    }

    pub fn byte_range(&self) -> Range<usize> {
        self.start..self.end
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
