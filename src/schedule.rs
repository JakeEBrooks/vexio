use std::slice::{Iter, IterMut};

use crate::Param;
use crate::error::ParseError;
use crate::parsing::Parser;

/// Represents the unique `SCHED` block within a Vex file.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize), derive(serde::Deserialize))]
pub struct Schedule {
    scans: Vec<Scan>,
}

impl Schedule {
    /// Create a new empty schedule
    pub fn new() -> Self {
        Self { scans: Vec::new() }
    }

    /// Add a scan to this schedule
    pub fn add_scan(&mut self, scan: Scan) {
        self.scans.push(scan);
    }

    /// Return an iterator over the scans in this schedule
    pub fn scans(&self) -> Iter<'_, Scan> {
        self.scans.iter()
    }

    /// Return a mutable iterator over the scans in this schedule
    pub fn scans_mut(&mut self) -> IterMut<'_, Scan> {
        self.scans.iter_mut()
    }

    /// View this schedule as a slice of scans
    pub fn scan_slice(&self) -> &[Scan] {
        self.scans.as_slice()
    }

    /// View this schedule as a mutable slice of scans
    pub fn scan_mut_slice(&mut self) -> &mut [Scan] {
        self.scans.as_mut_slice()
    }

    /// Get a representation of this schedule as a Vex formatted string
    pub fn to_vex(&self) -> String {
        format!("$SCHED;\n{}", self.scans.iter().map(|s| s.to_vex()).collect::<Vec<String>>().join("\n"))
    }

    /// Get a representation of this schedule as a Vex formatted string, with unneccesary whitespace removed
    pub fn to_vex_minified(&self) -> String {
        format!("$SCHED;{}", self.scans.iter().map(|s| s.to_vex_minified()).collect::<Vec<String>>().join(""))
    }

    /// Parse a schedule from a string slice
    pub fn parse(vex: &str) -> Result<Self, ParseError> {
        Parser::new(vex).parse_schedule()
    }
}

impl<I: Into<Vec<Scan>>> From<I> for Schedule {
    fn from(value: I) -> Self {
        Schedule { scans: value.into() }
    }
}

/// A single scan within a [`Schedule`]
#[derive(Debug, Default, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize), derive(serde::Deserialize))]
pub struct Scan {
    /// The name of this [`Scan`]
    name: String,
    /// The [`Param`]s associated with this [`Scan`]
    params: Vec<Param>,
}

impl Scan {
    /// Construct a new [`Scan`] from a `name` and a series of [`Param`]s.
    pub fn new<S: AsRef<str>, I: Into<Vec<Param>>>(name: S, params: I) -> Self {
        Self {
            name: name.as_ref().to_string(),
            params: params.into(),
        }
    }

    /// Construct a new [`Scan`] with no [`Param`]s from a `name`
    pub fn new_empty(name: impl AsRef<str>) -> Self {
        Self::new(name, [])
    }

    /// Get the name of this scan
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Retrieve references to all [`Param`]s with parameter names matching matching `name`.
    pub fn get(&self, name: impl AsRef<str>) -> Vec<&Param> {
        self.params.iter().filter(|p| p.name() == name.as_ref()).collect()
    }

    /// Retrieve mutable references to all [`Param`]s with parameter names matching matching `name`.
    pub fn get_mut(&mut self, name: impl AsRef<str>) -> Vec<&mut Param> {
        self.params.iter_mut().filter(|p| p.name() == name.as_ref()).collect()
    }

    /// Return an iterator over all the parameters in this scan
    pub fn params(&self) -> Iter<'_, Param> {
        self.params.iter()
    }

    /// Return a mutable iterator over all the parameters in this scan
    pub fn params_mut(&mut self) -> IterMut<'_, Param> {
        self.params.iter_mut()
    }

    /// Get a representation of this scan as a Vex formatted string
    pub fn to_vex(&self) -> String {
        format!(
            "scan {};\n{}\nendscan;\n",
            self.name,
            self.params.iter().map(|s| format!("\t{}", s.to_vex())).collect::<Vec<String>>().join("\n")
        )
    }

    /// Get a representation of this scan as a Vex formatted string, with unneccesary whitespace removed
    pub fn to_vex_minified(&self) -> String {
        format!(
            "scan {};{}endscan;",
            self.name,
            self.params.iter().map(|s| s.to_vex_minified()).collect::<Vec<String>>().join("")
        )
    }

    /// Parse a scan from a string slice
    pub fn parse(vex: &str) -> Result<Self, ParseError> {
        Parser::new(vex).parse_scan()
    }
}

/// Convenience macro for constructing a [`Scan`].
///
/// The required syntax is the same as the [`crate::def`] macro
#[macro_export]
macro_rules! scan {
    ($name:literal; $($param_name:literal = $($param:literal):*;)+) => {
        $crate::Scan::new(
            format!($name),
            [$( $crate::Param::new(format!($param_name), [$( $crate::ParamValue::new_ident_default(format!($param)) ),*]) ),*]
        )
    };
}
