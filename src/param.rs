use std::slice::{Iter, IterMut};

use crate::error::ParseError;
use crate::parsing::{Parser, is_coord, is_epoch, is_number, is_number_unit};

/// A Vex parameter
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize), derive(serde::Deserialize))]
pub struct Param {
    /// The name of this parameter
    name: String,
    /// The values associated with this parameter
    values: Vec<ParamValue>,
}

impl Param {
    /// Construct a new [`Param`] from a `name` and a series of [`ParamValue`]s.
    pub fn new(name: impl AsRef<str>, values: impl Into<Vec<ParamValue>>) -> Self {
        Self {
            name: name.as_ref().to_string(),
            values: values.into(),
        }
    }

    /// Construct a new parameter with no values
    pub fn new_empty(name: impl AsRef<str>) -> Self {
        Self::new(name, [])
    }

    /// Get the name of this parameter
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Add a value to this parameter
    pub fn add_value(&mut self, value: ParamValue) {
        self.values.push(value);
    }

    /// Return an iterator over the values of this parameter
    pub fn values(&self) -> Iter<'_, ParamValue> {
        self.values.iter()
    }

    /// Return a mutable iterator over the values of this parameter
    pub fn values_mut(&mut self) -> IterMut<'_, ParamValue> {
        self.values.iter_mut()
    }

    /// View this parameter as a slice of values
    pub fn as_slice(&self) -> &[ParamValue] {
        self.values.as_slice()
    }

    /// View this parameter as a mutable slice of values
    pub fn as_mut_slice(&mut self) -> &mut [ParamValue] {
        self.values.as_mut_slice()
    }

    /// Get a representation of this parameter as a Vex formatted string
    pub fn to_vex(&self) -> String {
        format!("{} = {};", self.name, self.values.iter().map(|v| v.to_vex()).collect::<Vec<String>>().join(" : "))
    }

    /// Get a representation of this parameter as a Vex formatted string, with unneccesary whitespace removed
    pub fn to_vex_minified(&self) -> String {
        format!("{}={};", self.name, self.values.iter().map(|v| v.to_vex_minified()).collect::<Vec<String>>().join(":"))
    }

    /// Parse a Vex parameter block from a string slice
    pub fn parse(vex: &str) -> Result<Self, ParseError> {
        Parser::new(vex).parse_param()
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
    ($name:literal) => {
        $crate::Param::new_empty(format!($name))
    };
}

/// A single value of a defined Vex [`Param`].
///
/// Parameter values are contained in a colon separated list in the Vex format
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize), derive(serde::Deserialize))]
pub struct ParamValue {
    /// The source text of the value
    source: String,
    /// The kind of this value
    kind: ParamValueKind,
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

    /// Get the source Vex of this value
    pub fn source(&self) -> &str {
        &self.source
    }

    /// Construct a new [`ParamValue`] of kind [`ParamValueKind::Empty`].
    pub fn empty() -> Self {
        Self {
            source: " ".to_string(),
            kind: ParamValueKind::Empty,
        }
    }

    /// Return the [`ParamValueKind`] of this [`ParamValue`]
    pub fn kind(&self) -> ParamValueKind {
        self.kind
    }

    /// Get a representation of this value as a Vex formatted string
    pub fn to_vex(&self) -> String {
        match self.kind {
            ParamValueKind::Link => format!("&{}", self.source),
            _ => self.source.clone(),
        }
    }

    /// Get a representation of this value as a Vex formatted string, with unneccesary whitespace removed
    pub fn to_vex_minified(&self) -> String {
        match self.kind {
            ParamValueKind::Link => format!("&{}", self.source),
            _ => self.source.clone(),
        }
    }

    /// Parse a value from a string slice
    pub fn parse(vex: &str) -> Result<Self, ParseError> {
        Parser::new(vex).parse_param_value()
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
    use crate::VexError;

    use super::*;

    #[test]
    fn test_param_macro() -> Result<(), VexError> {
        let p = param!("my_param" = "keyword");
        assert_eq!(p, Param::new("my_param", [ParamValue::new("keyword")]));
        Ok(())
    }
}
