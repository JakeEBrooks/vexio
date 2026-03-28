use crate::vex::ToFromVex;

/// A Vex `ref` declaration
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize), derive(serde::Deserialize))]
pub struct VexRef {
    /// The name of the [`Block`](crate::Block) this [`VexRef`] refers to
    pub blockname: String,
    /// The name of the [`Def`](crate::Def) block this [`VexRef`] refers to
    pub keyword: String,
    /// The associated qualifiers of this [`VexRef`]
    pub qualifiers: Vec<String>,
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
}

impl ToFromVex for VexRef {
    fn to_vex(&self) -> String {
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

    fn to_vex_minified(&self) -> String {
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

    fn from_vex(vex: &str) -> Result<Self, crate::VexError> {
        crate::parsing::Parser::new(vex).parse_ref()
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
    use super::*;

    #[test]
    fn test_refr_macro() -> miette::Result<()> {
        let refr = refr!("MYBLOCK" = "keyword");
        assert_eq!(refr, VexRef::new("MYBLOCK", "keyword"));
        Ok(())
    }

    #[test]
    fn test_refr_macro_with_quals() -> miette::Result<()> {
        let refr = refr!("MYBLOCK" = "keyword" : "qual1" : "qual2");
        assert_eq!(refr, VexRef::new_with_quals("MYBLOCK", "keyword", ["qual1", "qual2"]));
        Ok(())
    }
}
