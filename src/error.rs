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

// impl VexError {
//     /// Construct a new generic [`VexError`] with the description `desc`].
//     ///
//     /// This will not produce any text annotations by default.
//     pub fn new(desc: impl AsRef<str>) -> Self {
//         Self {
//             desc: desc.as_ref().to_string(),
//             src: None,
//             src_span: None,
//             help: None,
//         }
//     }

//     /// Construct a new generic [`VexError`] with the description `desc`.
//     ///
//     /// This will point to the byte location `loc`, but will not produce any text annotations by default.
//     pub fn new_at(desc: impl AsRef<str>, loc: usize) -> Self {
//         Self {
//             desc: desc.as_ref().to_string(),
//             src: None,
//             src_span: Some(SourceSpan::from(loc)),
//             help: None,
//         }
//     }

//     /// Construct a new generic [`VexError`] with the description `desc`.
//     ///
//     /// This will point to the byte span at `span`, but will not produce any text annotations by default.
//     pub fn new_span(desc: impl AsRef<str>, span: Range<usize>) -> Self {
//         Self {
//             desc: desc.as_ref().to_string(),
//             src: None,
//             src_span: Some(SourceSpan::from(span)),
//             help: None,
//         }
//     }

//     /// Set the description for this error.
//     pub fn with_desc(mut self, desc: impl AsRef<str>) -> Self {
//         self.desc = desc.as_ref().to_string();
//         self
//     }

//     /// Enable text annotations in the returned error by providing the source text
//     pub fn with_src(mut self, src: impl AsRef<str>) -> Self {
//         self.src = Some(OptionalNamedSource {
//             source: src.as_ref().to_string(),
//             name: None,
//         });
//         self
//     }

//     /// Give the returned error an associated file name.
//     pub fn with_filename(mut self, name: impl AsRef<str>) -> Self {
//         if let Some(named) = self.src.as_mut() {
//             named.set_name(name.as_ref());
//         }
//         self
//     }

//     /// Add a help message to the returned error
//     pub fn with_help(mut self, help: impl AsRef<str>) -> Self {
//         self.help = Some(help.as_ref().to_string());
//         self
//     }
// }

// impl Display for VexError {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         write!(f, "{}", self.desc)
//     }
// }

// impl From<std::io::Error> for VexError {
//     fn from(value: std::io::Error) -> Self {
//         Self {
//             desc: format!("An IO error occurred: {}", value),
//             src: None,
//             src_span: None,
//             help: None,
//         }
//     }
// }

// /// An implementation of [`miette::NamedSource`] when the name isn't known in advance and can be added later
// ///
// /// This means that the Vex parser doesn't need to know a filename to generate valid VexErrors
// /// but we can still give miette a filename later for proper annotation of an error that occurred
// /// from a file
// struct OptionalNamedSource<S: SourceCode + 'static> {
//     source: S,
//     name: Option<String>,
// }

// impl<S: SourceCode> Debug for OptionalNamedSource<S> {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         f.debug_struct("OptionalNamedSource").field("name", &self.name).field("source", &"<redacted>");
//         Ok(())
//     }
// }

// impl<S: SourceCode + Send + Sync + 'static> OptionalNamedSource<S> {
//     pub fn inner(&self) -> &S {
//         &self.source
//     }

//     pub fn set_name(&mut self, name: impl AsRef<str>) {
//         self.name = Some(name.as_ref().to_string())
//     }
// }

// impl<S: SourceCode + 'static> SourceCode for OptionalNamedSource<S> {
//     fn read_span<'a>(
//         &'a self,
//         span: &miette::SourceSpan,
//         context_lines_before: usize,
//         context_lines_after: usize,
//     ) -> Result<Box<dyn miette::SpanContents<'a> + 'a>, miette::MietteError> {
//         let inner_contents = self.inner().read_span(span, context_lines_before, context_lines_after)?;
//         let contents = if let Some(ref name) = self.name {
//             MietteSpanContents::new_named(
//                 (*name).clone(),
//                 inner_contents.data(),
//                 *inner_contents.span(),
//                 inner_contents.line(),
//                 inner_contents.column(),
//                 inner_contents.line_count(),
//             )
//         } else {
//             MietteSpanContents::new(
//                 inner_contents.data(),
//                 *inner_contents.span(),
//                 inner_contents.line(),
//                 inner_contents.column(),
//                 inner_contents.line_count(),
//             )
//         };

//         Ok(Box::new(contents))
//     }
// }
