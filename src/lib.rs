//! `vexio` is a library for parsing, modifying, and creating Very Long Baseline Interferometry (VLBI) Experiment files, or VEX files, commonly used for describing radio astronomy VLBI experiments.
//!
//! For examples on how to use this library, see the `examples/` directory in the crate source.

// #![warn(missing_docs)]

mod block;
mod error;
mod global;
mod parsing;
mod value;
mod vex;

pub use block::{Block, BlockDef, BlockKind};
pub use error::{ParseError, ParseErrorKind, VexError};
pub use global::Global;
pub use value::{VexValue, VexValueKind};
pub use vex::{Vex, VexVersion};

use std::iter::Peekable;

pub(crate) struct IntersperseIterator<I>
where
    I: Iterator,
    I::Item: Clone,
{
    inner: Peekable<I>,
    index: usize,
    separator: I::Item,
}

impl<I> IntersperseIterator<I>
where
    I: Iterator,
    I::Item: Clone,
{
    fn new(iter: I, separator: I::Item) -> Self {
        Self {
            inner: iter.peekable(),
            index: 0,
            separator,
        }
    }
}

impl<I> Iterator for IntersperseIterator<I>
where
    I: Iterator,
    I::Item: Clone,
{
    type Item = I::Item;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index % 2 == 1 && self.inner.peek().is_some() {
            self.index += 1;
            Some(self.separator.clone())
        } else {
            self.index += 1;
            self.inner.next()
        }
    }
}
