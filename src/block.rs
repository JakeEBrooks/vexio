use crate::error::{ParseError, VexError};
use crate::parsing::Parser;
use crate::{Def, HighLevelDef};
use std::collections::HashMap;
use std::collections::hash_map::{Keys, Values};
use std::ops::Index;

/// A primitive Vex block.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize), derive(serde::Deserialize))]
pub struct Block {
    /// The name of this primitive block
    name: String,
    /// The [`Def`]s within this primitive block
    defs: HashMap<String, Def>,
}

impl Block {
    /// Construct a new primitive Vex [`Block`] from a `name` and a series of [`Def`]s
    pub fn new<S: AsRef<str>, I: Into<Vec<Def>>>(name: S, defs: I) -> Self {
        let blkname = name.as_ref().to_string();
        let mut defmap = HashMap::new();
        for d in defs.into() {
            let _ = defmap.insert(d.name().to_string(), d);
        }

        Self { name: blkname, defs: defmap }
    }

    /// Construct a new primitive Vex [`Block`] with no [`Def`]s from a `name`
    pub fn new_empty(name: impl AsRef<str>) -> Self {
        Self::new(name, [])
    }

    /// Get the name of this block
    ///
    /// This may be something like "EXPER" or "ANTENNA"
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Add a [`Def`] to this [`Block`]
    pub fn add_block(&mut self, def: Def) -> Result<(), VexError> {
        match self.defs.insert(def.name().to_string(), def) {
            Some(dup) => Err(VexError::DuplicateDef(dup.name().to_string())),
            None => Ok(()),
        }
    }

    /// Get a reference to the [`Def`] block with name `key`, returning [`None`] if it doesn't exist
    pub fn get(&self, key: impl AsRef<str>) -> Option<&Def> {
        self.defs.get(key.as_ref())
    }

    /// Get a mutable reference to the Def block with name `key`, returning [`None`] if it doesn't exist
    pub fn get_mut(&mut self, key: impl AsRef<str>) -> Option<&mut Def> {
        self.defs.get_mut(key.as_ref())
    }

    /// Return an iterator over all the names of [`Def`] blocks within this [`Block`]
    pub fn def_names(&self) -> Keys<'_, String, Def> {
        self.defs.keys()
    }

    /// Return an iterator over all the [`Def`] blocks within this [`Block`]
    pub fn defs(&self) -> Values<'_, String, Def> {
        self.defs.values()
    }

    /// Get a representation of this block as a Vex formatted string
    pub fn to_vex(&self) -> String {
        format!("${};\n{}", self.name, self.defs.values().map(|v| v.to_vex()).collect::<Vec<String>>().join("\n"))
    }

    /// Get a representation of this block as a Vex formatted string, with unnecessary whitespace removed
    pub fn to_vex_minified(&self) -> String {
        format!("${};{}", self.name, self.defs.values().map(|v| v.to_vex_minified()).collect::<Vec<String>>().join(""))
    }

    /// Parse a [`Block`] from a string slice
    pub fn parse(vex: &str) -> Result<Self, ParseError> {
        Parser::new(vex).parse_primitive_block()
    }
}

impl<T: AsRef<str>> Index<T> for Block {
    type Output = Def;

    fn index(&self, index: T) -> &Self::Output {
        self.defs.index(index.as_ref())
    }
}

/// A high level Vex block. Currently this is either a `$MODE` or a `$STATION` block.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize), derive(serde::Deserialize))]
pub struct HighLevelBlock {
    /// The name of this high level block
    pub name: String,
    /// The [`HighLevelDef`]s within this block
    pub defs: HashMap<String, HighLevelDef>,
}

impl HighLevelBlock {
    /// Construct a new [`HighLevelBlock`] from a `name` and a series of [`HighLevelDef`]s.
    pub fn new<S: AsRef<str>, I: Into<Vec<HighLevelDef>>>(name: S, defs: I) -> Self {
        let blkname = name.as_ref().to_string();
        let mut defmap = HashMap::new();
        for d in defs.into() {
            let _ = defmap.insert(d.name().to_string(), d);
        }

        Self { name: blkname, defs: defmap }
    }

    /// Construct a new [`HighLevelBlock`] with no [`HighLevelDef`]s from a `name`
    pub fn new_empty(name: impl AsRef<str>) -> Self {
        Self::new(name, [])
    }

    /// Get the name of this block
    ///
    /// Currently this is either "MODE" or "STATION"
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Get a reference to the [`HighLevelDef`] block with name `key`, returning [`None`] if it doesn't exist.
    pub fn get(&self, key: impl AsRef<str>) -> Option<&HighLevelDef> {
        self.defs.get(key.as_ref())
    }

    /// Get a mutable reference to the Def block with name `key`, returning [`None`] if it doesn't exist.
    pub fn get_mut(&mut self, key: impl AsRef<str>) -> Option<&mut HighLevelDef> {
        self.defs.get_mut(key.as_ref())
    }

    /// Return an iterator over all the names of [`HighLevelDef`] blocks within this [`HighLevelBlock`]
    pub fn def_names(&self) -> Keys<'_, String, HighLevelDef> {
        self.defs.keys()
    }

    /// Return an iterator over all the [`HighLevelDef`] blocks within this [`HighLevelBlock`]
    pub fn defs(&self) -> Values<'_, String, HighLevelDef> {
        self.defs.values()
    }

    /// Get a representation of this block as a Vex formatted string
    pub fn to_vex(&self) -> String {
        format!("${};\n{}", self.name, self.defs.values().map(|v| v.to_vex()).collect::<Vec<String>>().join("\n"))
    }

    /// Get a representation of this block as a Vex formatted string, with unnecessary whitespace removed
    pub fn to_vex_minified(&self) -> String {
        format!("${};{}", self.name, self.defs.values().map(|v| v.to_vex_minified()).collect::<Vec<String>>().join(""))
    }

    /// Parse a [`HighLevelBlock`] from a string slice
    pub fn parse(vex: &str) -> Result<Self, ParseError> {
        Parser::new(vex).parse_highlevel_block()
    }
}

impl<T: AsRef<str>> Index<T> for HighLevelBlock {
    type Output = HighLevelDef;

    fn index(&self, index: T) -> &Self::Output {
        self.defs.index(index.as_ref())
    }
}

#[cfg(test)]
mod tests {
    use crate::{def, refr};

    use super::*;

    #[test]
    fn test_parse_tofrom_block() -> Result<(), VexError> {
        let start = Block::new(
            "my_block",
            [
                def!("my_def1";
                    "my_param1" = "value1" : "value2";
                    "my_param2" = "value3" : "value4";
                ),
                def!("my_def2";
                    "my_param3" = "value1" : "value2";
                    "my_param4" = "value3" : "value4";
                ),
            ],
        );
        assert_eq!(start, Block::parse(start.to_vex().as_str())?);
        Ok(())
    }

    #[test]
    fn test_parse_tofrom_highlevelblock() -> Result<(), VexError> {
        let start = HighLevelBlock::new("MY_BLOCK", [HighLevelDef::new("my_def", [refr!("BLK1" = "my_key"), refr!("BLK2" = "my_key2")])]);
        assert_eq!(start, HighLevelBlock::parse(start.to_vex().as_str())?);
        Ok(())
    }
}
