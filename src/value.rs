use std::fmt::Display;
use std::str::FromStr;

use crate::error::ParseError;
use crate::parsing::Parser;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize),
    derive(serde::Deserialize)
)]
/// The type of a VEX value.
pub enum VexValueKind {
    /// A numeric value without a unit.
    Number,
    /// A numeric value with a unit.
    NumberUnit,
    /// An unquoted identifier.
    Ident,
    /// A quoted string literal.
    Literal,
    /// An epoch value.
    Epoch,
    /// A right-ascension value.
    RightAscension,
    /// A declination value.
    Declination,
    /// A reference link, written with a leading `&` in VEX.
    Link,
    /// An empty value.
    Empty,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize),
    derive(serde::Deserialize)
)]
/// A VEX value retaining its source text, optional unit, and type.
pub struct VexValue {
    source: String,
    unit: Option<String>,
    kind: VexValueKind,
}

impl VexValue {
    /// Creates a value from its source text, optional unit, and value kind.
    pub fn new(source: impl AsRef<str>, unit: Option<String>, kind: VexValueKind) -> Self {
        Self {
            source: source.as_ref().to_string(),
            unit,
            kind,
        }
    }

    /// Creates a unitless numeric value from a floating-point number.
    pub fn new_float(number: f64) -> Self {
        Self {
            source: number.to_string(),
            unit: None,
            kind: VexValueKind::Number,
        }
    }

    /// Creates a unitless numeric value from an integer.
    pub fn new_integer(number: i64) -> Self {
        Self {
            source: number.to_string(),
            unit: None,
            kind: VexValueKind::Number,
        }
    }

    /// Creates a floating-point numeric value with a unit.
    pub fn new_float_unit(number: f64, unit: impl AsRef<str>) -> Self {
        Self {
            source: number.to_string(),
            unit: Some(unit.as_ref().to_string()),
            kind: VexValueKind::NumberUnit,
        }
    }

    /// Creates an integer numeric value with a unit.
    pub fn new_integer_unit(number: i64, unit: impl AsRef<str>) -> Self {
        Self {
            source: number.to_string(),
            unit: Some(unit.as_ref().to_string()),
            kind: VexValueKind::NumberUnit,
        }
    }

    /// Creates an identifier value from its source text.
    pub fn new_ident(source: impl AsRef<str>) -> Self {
        Self {
            source: source.as_ref().to_string(),
            unit: None,
            kind: VexValueKind::Ident,
        }
    }

    /// Creates a literal value from its source text.
    ///
    /// This should _not_ include the double quotation marks
    pub fn new_literal(source: impl AsRef<str>) -> Self {
        Self {
            source: source.as_ref().to_string(),
            unit: None,
            kind: VexValueKind::Literal,
        }
    }

    /// Creates an epoch value from its source text.
    pub fn new_epoch(source: impl AsRef<str>) -> Self {
        Self {
            source: source.as_ref().to_string(),
            unit: None,
            kind: VexValueKind::Epoch,
        }
    }

    /// Creates a right-ascension value from its source text.
    pub fn new_ra(source: impl AsRef<str>) -> Self {
        Self {
            source: source.as_ref().to_string(),
            unit: None,
            kind: VexValueKind::RightAscension,
        }
    }

    /// Creates a declination value from its source text.
    pub fn new_dec(source: impl AsRef<str>) -> Self {
        Self {
            source: source.as_ref().to_string(),
            unit: None,
            kind: VexValueKind::Declination,
        }
    }

    /// Creates a link value from its target text, without a leading `&`.
    pub fn new_link(source: impl AsRef<str>) -> Self {
        Self {
            source: source.as_ref().to_string(),
            unit: None,
            kind: VexValueKind::Link,
        }
    }

    /// Creates an empty value.
    pub fn empty() -> Self {
        Self {
            source: String::new(),
            unit: None,
            kind: VexValueKind::Empty,
        }
    }

    /// Returns the value's source text.
    pub fn source(&self) -> &str {
        &self.source
    }

    /// Replaces the value's source text.
    pub fn set_source(&mut self, source: impl AsRef<str>) {
        self.source = source.as_ref().to_string()
    }

    /// Returns the value's type.
    pub fn kind(&self) -> VexValueKind {
        self.kind
    }

    /// Sets the value's type.
    pub fn set_kind(&mut self, kind: VexValueKind) {
        self.kind = kind
    }

    /// Returns the value's unit, if it has one.
    pub fn unit(&self) -> Option<&str> {
        self.unit.as_deref()
    }

    /// Sets the value's unit and marks its kind as [`VexValueKind::NumberUnit`].
    pub fn set_unit(&mut self, unit: impl AsRef<str>) {
        self.set_kind(VexValueKind::NumberUnit);
        self.unit = Some(unit.as_ref().to_string())
    }

    /// Removes the value's unit, if present.
    pub fn clear_unit(&mut self) {
        if self.kind == VexValueKind::NumberUnit {
            self.set_kind(VexValueKind::Number)
        };
        self.unit.take();
    }

    fn to_vex(&self) -> String {
        match self.kind {
            VexValueKind::Link => format!("&{}", self.source),
            VexValueKind::NumberUnit => format!(
                "{} {}",
                self.source,
                self.unit
                    .clone()
                    .expect("A VexValue with kind NumberUnit must have a unit!")
            ),
            _ => self.source.clone(),
        }
    }
}

impl Display for VexValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_vex())
    }
}

impl FromStr for VexValue {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Parser::new(s)?.parse_vex_value()
    }
}
