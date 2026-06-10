use std::slice::{Iter, IterMut};

use crate::VexRef;
use crate::error::ParseError;
use crate::parsing::Parser;

/// Represents the unique `GLOBAL` block within a Vex file.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize), derive(serde::Deserialize))]
pub struct Global {
    refs: Vec<VexRef>,
}

impl Global {
    /// Make a new, empty, Global block
    pub fn new() -> Self {
        Self { refs: Vec::new() }
    }

    /// Add a [`VexRef`] to this global block
    pub fn add_ref(&mut self, refr: VexRef) {
        self.refs.push(refr);
    }

    /// View this global block as a slice of references
    pub fn as_slice(&self) -> &[VexRef] {
        self.refs.as_slice()
    }

    /// View this global block as a mutable slice of references
    pub fn as_mut_slice(&mut self) -> &mut [VexRef] {
        self.refs.as_mut_slice()
    }

    /// Retrieve references to all [`VexRef`]s with block names matching matching `blk`.
    ///
    /// For example:
    /// ```rust
    /// use vexio::{refr, Global};
    ///
    /// let mut global = Global::new();
    /// global.add_ref(refr!("BLK1" = "my_key2"));
    /// global.add_ref(refr!("BLK2" = "key2" : "qual1"));
    /// assert_eq!(global.get("BLK1"), vec![&refr!("BLK1" = "my_key2")])
    /// ```
    pub fn get(&self, blk: impl AsRef<str>) -> Vec<&VexRef> {
        self.refs.iter().filter(|r| r.blockname() == blk.as_ref()).collect()
    }

    /// Retrieve mutable references to all [`VexRef`]s with block names matching matching `blk`.
    ///
    /// For example:
    /// ```rust
    /// use vexio::{refr, Global};
    ///
    /// let mut global = Global::new();
    /// global.add_ref(refr!("BLK1" = "my_key2"));
    /// global.add_ref(refr!("BLK2" = "key2" : "qual1"));
    /// assert_eq!(global.get_mut("BLK1"), vec![&mut refr!("BLK1" = "my_key2")])
    /// ```
    pub fn get_mut(&mut self, blk: impl AsRef<str>) -> Vec<&mut VexRef> {
        self.refs.iter_mut().filter(|r| r.blockname() == blk.as_ref()).collect()
    }

    /// Return an iterator over all [`VexRef`]s in this [`Global`].
    pub fn refs(&self) -> Iter<'_, VexRef> {
        self.refs.iter()
    }

    /// Return an iterator of mutable references over all [`VexRef`]s in this [`Global`].
    pub fn refs_mut(&mut self) -> IterMut<'_, VexRef> {
        self.refs.iter_mut()
    }

    /// Get a representation of this global block as a Vex formatted string
    pub fn to_vex(&self) -> String {
        format!("$GLOBAL;\n{}\n", self.refs.iter().map(|r| format!("\t{}", r.to_vex())).collect::<Vec<String>>().join("\n"))
    }

    /// Get a representation of this global block as a Vex formatted string, with unneccesary whitespace removed
    pub fn to_vex_minified(&self) -> String {
        format!("$GLOBAL;{}", self.refs.iter().map(|r| r.to_vex_minified()).collect::<Vec<String>>().join(""))
    }

    /// Parse a [`Global`] block from a string slice
    pub fn parse(vex: &str) -> Result<Self, ParseError> {
        Parser::new(vex).parse_global()
    }
}

impl<I: Into<Vec<VexRef>>> From<I> for Global {
    fn from(value: I) -> Self {
        Self { refs: value.into() }
    }
}

#[cfg(test)]
mod tests {
    use crate::error::VexError;
    use crate::refr;

    use super::*;

    #[test]
    fn test_parse_tofrom_global() -> Result<(), VexError> {
        let start = Global::from([refr!("MY_BLOCK" = "some_key" : "qual1"), refr!("MY_BLOCK2" = "some_key2")]);
        assert_eq!(start, Global::parse(start.to_vex().as_str())?);
        Ok(())
    }
}
