use crate::parsing::{is_coord, is_epoch, is_number, is_number_unit};
use crate::vex::ToFromVex;

/// A Vex parameter
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize), derive(serde::Deserialize))]
pub struct Param {
    /// The name of this parameter
    pub name: String,
    /// The values associated with this parameter
    pub values: Vec<ParamValue>,
}

impl Param {
    /// Construct a new [`Param`] from a `name` and a series of [`ParamValue`]s.
    pub fn new<S, I>(name: S, values: I) -> Self
    where
        S: AsRef<str>,
        I: Into<Vec<ParamValue>>,
    {
        Self {
            name: name.as_ref().to_string(),
            values: values.into(),
        }
    }
}

impl ToFromVex for Param {
    fn to_vex(&self) -> String {
        format!("{} = {};", self.name, self.values.iter().map(|v| v.to_vex()).collect::<Vec<String>>().join(" : "))
    }

    fn to_vex_minified(&self) -> String {
        format!("{}={};", self.name, self.values.iter().map(|v| v.to_vex_minified()).collect::<Vec<String>>().join(":"))
    }

    fn from_vex(vex: &str) -> Result<Self, crate::VexError> {
        crate::parsing::Parser::new(vex).parse_param()
    }
}

/// Convenience macro for constructing a [`Param`].
///
/// # Examples
/// ```rust
/// use vexio::param;
///
/// let par1 = param!("my_parameter" = "my_value");
/// let par2 = param!("my_parameter_2" = "my_value" : "with" : "" : "other_values");
/// ```
///
/// You can also use variables from the local scope:
/// ```rust
/// use vexio::param;
///
/// let s = "some_value";
/// let par1 = param!("my_parameter" = "my_value");
/// let par2 = param!("my_parameter_2" = "{s}" : "with" : "" : "other_values");
/// ```
#[macro_export]
macro_rules! param {
    ($name:literal = $($value:literal):*) => {
        $crate::Param::new(format!($name), [$($crate::ParamValue::new($value)),*])
    };
}

/// A single value of a defined Vex [`Param`].
///
/// Parameter values are contained in a colon separated list in the Vex format
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize), derive(serde::Deserialize))]
pub struct ParamValue {
    /// The source text of the value
    pub source: String,
    /// The kind of this value
    pub kind: ParamValueKind,
}

impl ParamValue {
    /// Construct a new [`ParamValue`].
    ///
    /// The [`ParamValueKind`] is automatically determined
    pub fn new(source: impl AsRef<str>) -> Self {
        let sref = source.as_ref();
        let kind = determine_param_value_kind(sref);
        let s = match kind {
            ParamValueKind::Number => sref,
            ParamValueKind::NumberUnit => sref,
            ParamValueKind::Coord => sref,
            ParamValueKind::Epoch => sref,
            ParamValueKind::Link => &sref[1..],
            ParamValueKind::Empty => " ",
            ParamValueKind::Literal => sref,
            ParamValueKind::Ident => sref,
        };
        Self { source: s.to_string(), kind }
    }

    /// Construct a new [`ParamValue`].
    ///
    /// The user asserts that this value is of kind `kind`.
    pub fn new_unchecked(source: impl AsRef<str>, kind: ParamValueKind) -> Self {
        Self {
            source: source.as_ref().to_string(),
            kind,
        }
    }

    /// Construct a new [`ParamValue`] of kind [`ParamValueKind::Empty`].
    pub fn empty() -> Self {
        Self {
            source: String::new(),
            kind: ParamValueKind::Empty,
        }
    }

    /// Return `true` if this value is of kind [`ParamValueKind::Empty`].
    pub fn is_empty(&self) -> bool {
        self.kind == ParamValueKind::Empty
    }

    /// Return the [`ParamValueKind`] of this [`ParamValue`]
    pub fn kind(&self) -> ParamValueKind {
        self.kind
    }
}

fn determine_param_value_kind(src: impl AsRef<str>) -> ParamValueKind {
    let r = src.as_ref();
    if is_number(r) {
        ParamValueKind::Number
    } else if is_number_unit(r) {
        ParamValueKind::NumberUnit
    } else if is_coord(r) {
        ParamValueKind::Coord
    } else if is_epoch(r) {
        ParamValueKind::Epoch
    } else if r.starts_with('"') && r.ends_with('"') {
        ParamValueKind::Literal
    } else if r.starts_with('&') {
        ParamValueKind::Link
    } else if r.chars().all(|c: char| c.is_whitespace()) {
        ParamValueKind::Empty
    } else {
        ParamValueKind::Ident
    }
}

impl ToFromVex for ParamValue {
    fn to_vex(&self) -> String {
        match self.kind {
            ParamValueKind::Link => format!("&{}", self.source),
            _ => self.source.clone(),
        }
    }

    fn to_vex_minified(&self) -> String {
        match self.kind {
            ParamValueKind::Link => format!("&{}", self.source),
            _ => self.source.clone(),
        }
    }

    fn from_vex(vex: &str) -> Result<Self, crate::VexError> {
        crate::parsing::Parser::new(vex).parse_param_value()
    }
}

#[allow(missing_docs)]
/// The type of a particular [`ParamValue`]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize), derive(serde::Deserialize))]
pub enum ParamValueKind {
    Number,
    NumberUnit,
    Ident,
    Literal,
    Epoch,
    Coord,
    Link,
    Empty,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_param_macro() -> miette::Result<()> {
        let p = param!("my_param" = "keyword");
        assert_eq!(p, Param::new("my_param", [ParamValue::new("keyword")]));
        Ok(())
    }
}
