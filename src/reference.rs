use std::slice::{Iter, IterMut};

use crate::error::ParseError;
use crate::parsing::Parser;

/// A Vex `ref` declaration
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize), derive(serde::Deserialize))]
pub struct VexRef {
    /// The name of the [`Block`](crate::Block) this [`VexRef`] refers to
    blockname: String,
    /// The name of the [`Def`](crate::Def) block this [`VexRef`] refers to
    keyword: String,
    /// The associated qualifiers of this [`VexRef`]
    qualifiers: Vec<String>,
}

impl VexRef {
    /// Construct a new [`VexRef`] from a `blockname` and a `keyword` (the name of a [`Def`](crate::Def) within `blockname`).
    pub fn new<S: AsRef<str>>(blockname: S, keyword: S) -> Self {
        let blkname: &str = if blockname.as_ref().starts_with('$') {
            &blockname.as_ref()[1..]
        } else {
            blockname.as_ref()
        };
        Self {
            blockname: blkname.to_uppercase(),
            keyword: keyword.as_ref().to_string(),
            qualifiers: Vec::new(),
        }
    }

    /// Construct a new [`VexRef`] from a `blockname`, a `keyword` (the name of a [`Def`](crate::Def) within `blockname`), and a series of qualifiers
    pub fn new_with_quals<S: AsRef<str>>(blockname: S, keyword: S, qualifiers: impl Into<Vec<S>>) -> Self {
        let blkname: &str = if blockname.as_ref().starts_with('$') {
            &blockname.as_ref()[1..]
        } else {
            blockname.as_ref()
        };
        Self {
            blockname: blkname.to_uppercase(),
            keyword: keyword.as_ref().to_string(),
            qualifiers: qualifiers.into().iter().map(|s| s.as_ref().to_string()).collect(),
        }
    }

    pub fn blockname(&self) -> &str {
        &self.blockname
    }

    pub fn defname(&self) -> &str {
        &self.keyword
    }

    pub fn quals(&self) -> Iter<'_, String> {
        self.qualifiers.iter()
    }

    pub fn quals_mut(&mut self) -> IterMut<'_, String> {
        self.qualifiers.iter_mut()
    }

    pub fn quals_slice(&self) -> &[String] {
        self.qualifiers.as_slice()
    }

    pub fn quals_mut_slice(&mut self) -> &mut [String] {
        self.qualifiers.as_mut_slice()
    }

    pub fn add_quals(&mut self, qual: impl AsRef<str>) {
        self.qualifiers.push(qual.as_ref().to_string());
    }

    pub fn to_vex(&self) -> String {
        if self.qualifiers.is_empty() {
            format!("ref ${} = {};", self.blockname, self.keyword)
        } else {
            format!(
                "ref ${} = {}:{};",
                self.blockname,
                self.keyword,
                self.qualifiers.iter().map(|q| q.to_string()).collect::<Vec<String>>().join(" : ")
            )
        }
    }

    pub fn to_vex_minified(&self) -> String {
        if self.qualifiers.is_empty() {
            format!("ref ${}={};", self.blockname, self.keyword)
        } else {
            format!(
                "ref ${}={}:{};",
                self.blockname,
                self.keyword,
                self.qualifiers.iter().map(|q| q.to_string()).collect::<Vec<String>>().join(":")
            )
        }
    }

    pub fn parse(vex: &str) -> Result<Self, ParseError> {
        Parser::new(vex).parse_ref()
    }
}

/// Convenience macro for constructing a [`VexRef`].
///
/// # Examples
/// ```rust
/// use vexio::refr;
///
/// let refr = refr!("MY_BLOCK" = "my_def_keyword");
/// let refr_q = refr!("MY_BLOCK" = "my_def_keyword" : "with" : "qualifiers");
/// ```
///
/// You can also use variables from the local scope:
/// ```rust
/// use vexio::refr;
///
/// let my_def = "my_def_block";
/// let refr = refr!("MY_BLOCK" = "{my_def}");
/// let refr_q = refr!("MY_BLOCK" = "my_def_keyword" : "with" : "qualifiers");
/// ```
#[macro_export]
macro_rules! refr {
    ($name:literal = $keyword:literal) => {
        $crate::VexRef::new(format!($name), format!($keyword))
    };

    ($name:literal = $keyword:literal:$($qualifier:literal):+) => {
        $crate::VexRef::new_with_quals(format!($name), format!($keyword), [$(format!($qualifier)),+])
    };
}

#[cfg(test)]
mod tests {
    use crate::VexError;

    use super::*;

    #[test]
    fn test_refr_macro() -> Result<(), VexError> {
        let refr = refr!("MYBLOCK" = "keyword");
        assert_eq!(refr, VexRef::new("MYBLOCK", "keyword"));
        Ok(())
    }

    #[test]
    fn test_refr_macro_with_quals() -> Result<(), VexError> {
        let refr = refr!("MYBLOCK" = "keyword" : "qual1" : "qual2");
        assert_eq!(refr, VexRef::new_with_quals("MYBLOCK", "keyword", ["qual1", "qual2"]));
        Ok(())
    }
}
