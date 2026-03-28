use std::fmt::{Debug, Display};
use std::ops::Range;

use miette::{Diagnostic, MietteSpanContents, SourceCode, SourceSpan};
use thiserror::Error;

/// A Vex error type.
///
/// This error type uses `miette` to annotate parts of a [`String`] containing data in the Vex format,
/// that has caused an error of some kind.
#[derive(Debug, Error, Diagnostic)]
#[diagnostic(severity(Error))]
pub struct VexError {
    desc: String,

    #[source_code]
    src: Option<OptionalNamedSource<String>>,
    #[label = "here"]
    src_span: Option<SourceSpan>,
    #[help]
    help: Option<String>,
}

impl VexError {
    /// Construct a new generic [`VexError`] with the description `desc`].
    ///
    /// This will not produce any text annotations by default.
    pub fn new(desc: impl AsRef<str>) -> Self {
        Self {
            desc: desc.as_ref().to_string(),
            src: None,
            src_span: None,
            help: None,
        }
    }

    /// Construct a new generic [`VexError`] with the description `desc`.
    ///
    /// This will point to the byte location `loc`, but will not produce any text annotations by default.
    pub fn new_at(desc: impl AsRef<str>, loc: usize) -> Self {
        Self {
            desc: desc.as_ref().to_string(),
            src: None,
            src_span: Some(SourceSpan::from(loc)),
            help: None,
        }
    }

    /// Construct a new generic [`VexError`] with the description `desc`.
    ///
    /// This will point to the byte span at `span`, but will not produce any text annotations by default.
    pub fn new_span(desc: impl AsRef<str>, span: Range<usize>) -> Self {
        Self {
            desc: desc.as_ref().to_string(),
            src: None,
            src_span: Some(SourceSpan::from(span)),
            help: None,
        }
    }

    /// Set the description for this error.
    pub fn with_desc(mut self, desc: impl AsRef<str>) -> Self {
        self.desc = desc.as_ref().to_string();
        self
    }

    /// Enable text annotations in the returned error by providing the source text
    pub fn with_src(mut self, src: impl AsRef<str>) -> Self {
        self.src = Some(OptionalNamedSource {
            source: src.as_ref().to_string(),
            name: None,
        });
        self
    }

    /// Give the returned error an associated file name.
    pub fn with_filename(mut self, name: impl AsRef<str>) -> Self {
        if let Some(named) = self.src.as_mut() {
            named.set_name(name.as_ref());
        }
        self
    }

    /// Add a help message to the returned error
    pub fn with_help(mut self, help: impl AsRef<str>) -> Self {
        self.help = Some(help.as_ref().to_string());
        self
    }
}

impl Display for VexError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.desc)
    }
}

impl From<std::io::Error> for VexError {
    fn from(value: std::io::Error) -> Self {
        Self {
            desc: format!("An IO error occurred: {}", value),
            src: None,
            src_span: None,
            help: None,
        }
    }
}

/// An implementation of [`miette::NamedSource`] when the name isn't known in advance and can be added later
///
/// This means that the Vex parser doesn't need to know a filename to generate valid VexErrors
/// but we can still give miette a filename later for proper annotation of an error that occurred
/// from a file
struct OptionalNamedSource<S: SourceCode + 'static> {
    source: S,
    name: Option<String>,
}

impl<S: SourceCode> Debug for OptionalNamedSource<S> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("OptionalNamedSource").field("name", &self.name).field("source", &"<redacted>");
        Ok(())
    }
}

impl<S: SourceCode + Send + Sync + 'static> OptionalNamedSource<S> {
    pub fn inner(&self) -> &S {
        &self.source
    }

    pub fn set_name(&mut self, name: impl AsRef<str>) {
        self.name = Some(name.as_ref().to_string())
    }
}

impl<S: SourceCode + 'static> SourceCode for OptionalNamedSource<S> {
    fn read_span<'a>(
        &'a self,
        span: &miette::SourceSpan,
        context_lines_before: usize,
        context_lines_after: usize,
    ) -> Result<Box<dyn miette::SpanContents<'a> + 'a>, miette::MietteError> {
        let inner_contents = self.inner().read_span(span, context_lines_before, context_lines_after)?;
        let contents = if let Some(ref name) = self.name {
            MietteSpanContents::new_named(
                (*name).clone(),
                inner_contents.data(),
                *inner_contents.span(),
                inner_contents.line(),
                inner_contents.column(),
                inner_contents.line_count(),
            )
        } else {
            MietteSpanContents::new(
                inner_contents.data(),
                *inner_contents.span(),
                inner_contents.line(),
                inner_contents.column(),
                inner_contents.line_count(),
            )
        };

        Ok(Box::new(contents))
    }
}
