use crate::error::ParseError;
use crate::parsing::Parser;
use crate::value::VexValue;
use std::collections::HashMap;
use std::collections::hash_map::{Iter, IterMut};
use std::fmt::Display;
use std::ops::{Index, IndexMut};
use std::str::FromStr;

/// Represents the unique `GLOBAL` block within a Vex file.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize),
    derive(serde::Deserialize)
)]
pub struct Global {
    refs: HashMap<String, VexValue>,
}

impl Global {
    /// Make a new, empty, GLOBAL block
    pub fn new() -> Self {
        Self {
            refs: HashMap::new(),
        }
    }

    /// Inserts a reference, returning any previous value stored under the same key.
    pub fn add(&mut self, key: impl AsRef<str>, value: VexValue) -> Option<VexValue> {
        self.refs.insert(key.as_ref().to_string(), value)
    }

    /// Returns the reference value for `key`, if present.
    pub fn get(&self, key: impl AsRef<str>) -> Option<&VexValue> {
        self.refs.get(key.as_ref())
    }

    /// Returns mutable access to the reference value for `key`, if present.
    pub fn get_mut(&mut self, key: impl AsRef<str>) -> Option<&mut VexValue> {
        self.refs.get_mut(key.as_ref())
    }

    /// Removes and returns the reference value for `key`, if present.
    pub fn remove(&mut self, key: impl AsRef<str>) -> Option<VexValue> {
        self.refs.remove(key.as_ref())
    }

    /// Returns whether a reference with `key` is present.
    pub fn contains_key(&self, key: impl AsRef<str>) -> bool {
        self.refs.contains_key(key.as_ref())
    }

    /// Iterates over references as `(key, value)` pairs.
    pub fn iter(&self) -> Iter<'_, String, VexValue> {
        self.refs.iter()
    }

    /// Mutably iterates over references as `(key, value)` pairs.
    pub fn iter_mut(&mut self) -> IterMut<'_, String, VexValue> {
        self.refs.iter_mut()
    }

    fn to_vex(&self) -> String {
        format!(
            "$GLOBAL;{}\n",
            self.refs
                .iter()
                .map(|(k, v)| format!("\n\tref ${} = {};", k.to_uppercase(), v))
                .collect::<String>()
        )
    }
}

impl Display for Global {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_vex())
    }
}

impl FromStr for Global {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Parser::new(s)?.parse_global()
    }
}

impl<I: AsRef<str>> Index<I> for Global {
    type Output = VexValue;

    fn index(&self, index: I) -> &Self::Output {
        self.get(index.as_ref())
            .unwrap_or_else(|| panic!("Entry '{}' does not exist in GLOBAL", index.as_ref()))
    }
}

impl<I: AsRef<str>> IndexMut<I> for Global {
    fn index_mut(&mut self, index: I) -> &mut Self::Output {
        self.get_mut(index.as_ref())
            .unwrap_or_else(|| panic!("Entry '{}' does not exist in GLOBAL", index.as_ref()))
    }
}

impl<'a> IntoIterator for &'a Global {
    type Item = (&'a String, &'a VexValue);
    type IntoIter = Iter<'a, String, VexValue>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'a> IntoIterator for &'a mut Global {
    type Item = (&'a String, &'a mut VexValue);
    type IntoIter = IterMut<'a, String, VexValue>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter_mut()
    }
}
