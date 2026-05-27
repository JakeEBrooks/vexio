use std::slice::{Iter, IterMut};

use crate::error::ParseError;
use crate::parsing::Parser;

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
    pub fn new<P: Into<ParamValue>, C: Into<Vec<P>>>(name: impl AsRef<str>, values: C) -> Self {
        let values = values.into().into_iter().map(|p| p.into()).collect();
        Self {
            name: name.as_ref().to_string(),
            values,
        }
    }

    /// Construct a new parameter with no values
    pub fn new_empty(name: impl AsRef<str>) -> Self {
        Self {
            name: name.as_ref().to_string(),
            values: Vec::new(),
        }
    }

    /// Get the name of this parameter
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Add a value to this parameter
    pub fn add_value(&mut self, value: impl Into<ParamValue>) {
        self.values.push(value.into());
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
        $crate::Param::new(format!($name), [$($crate::ParamValue::from($value)),*])
    };
    ($name:literal) => {
        $crate::Param::new_empty(format!($name))
    };
}

/// A single value of a defined Vex [`Param`].
///
/// Parameter values are contained in a colon separated list in the Vex format
///
/// [`ParamValue`]s implement [`From`] for many of the standard types and this is the preferred way of constructing them.
/// Passing a [`String`] or &[`str`] to the [`Self::from`] method will first attempt to parse it properly, and then
/// implicitly assume it is a value of kind [`ParamValueKind::Ident`] if it cannot be parsed. This gives the user a lot of
/// freedom when it comes to constructing values.
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
    /// The user asserts that this value is of kind `kind`.
    pub fn new_unchecked(source: impl AsRef<str>, kind: ParamValueKind) -> Self {
        Self {
            source: source.as_ref().to_string(),
            kind,
        }
    }

    /// Construct a new [`ParamValue`]
    ///
    /// If `source` cannot be parsed properly, assume the input source is of kind
    /// [`ParamValueKind::Ident`] and accept the source implicitly
    pub fn new_ident_default(source: impl AsRef<str>) -> Self {
        match Self::parse(source.as_ref()) {
            Ok(parsed) => parsed,
            Err(_) => Self::new_unchecked(source, ParamValueKind::Ident),
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

impl From<String> for ParamValue {
    fn from(value: String) -> Self {
        Self::new_ident_default(value)
    }
}

impl From<&str> for ParamValue {
    fn from(value: &str) -> Self {
        Self::new_ident_default(value)
    }
}

macro_rules! make_value_from {
    ($type:ty, $kind:expr) => {
        impl From<$type> for ParamValue {
            fn from(value: $type) -> Self {
                Self::new_unchecked(value.to_string(), $kind)
            }
        }
    };
}

make_value_from!(u8, ParamValueKind::Number);
make_value_from!(u16, ParamValueKind::Number);
make_value_from!(u32, ParamValueKind::Number);
make_value_from!(u64, ParamValueKind::Number);
make_value_from!(usize, ParamValueKind::Number);
make_value_from!(i8, ParamValueKind::Number);
make_value_from!(i16, ParamValueKind::Number);
make_value_from!(i32, ParamValueKind::Number);
make_value_from!(i64, ParamValueKind::Number);
make_value_from!(isize, ParamValueKind::Number);
make_value_from!(f32, ParamValueKind::Number);
make_value_from!(f64, ParamValueKind::Number);

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
    use crate::error::VexError;

    #[test]
    fn test_param_macro() -> Result<(), VexError> {
        let p = param!("my_param" = "keyword");
        assert_eq!(p, Param::new("my_param", ["keyword"]));
        Ok(())
    }

    #[test]
    fn test_param_value_from_0() -> Result<(), VexError> {
        let v = ParamValue::from(0.1);
        assert_eq!(v, ParamValue::new_unchecked("0.1", ParamValueKind::Number));
        Ok(())
    }

    #[test]
    fn test_param_value_from_1() -> Result<(), VexError> {
        let mut p = Param::new_empty("my_param");
        p.add_value(0.25);
        p.add_value("some_ident");
        p.add_value(2);
        let instead_parsed = Param::parse("my_param = 0.25 : some_ident : 2;")?;
        assert_eq!(p, instead_parsed);
        Ok(())
    }

    #[test]
    fn test_param_value_from_2() -> Result<(), VexError> {
        let mut p = Param::new_empty("my_param");
        p.add_value(0.25);
        p.add_value("some_ident");
        p.add_value(2);
        let instead = param!("my_param" = 0.25 : "some_ident" : 2);
        assert_eq!(p, instead);
        Ok(())
    }
}
