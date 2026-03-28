use crate::{Def, HighLevelDef, ToFromVex, VexError};
use std::collections::HashMap;

/// A primitive Vex block.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize), derive(serde::Deserialize))]
pub struct Block {
    /// The name of this primitive block
    pub name: String,
    /// The [`Def`]s within this primitive block
    pub defs: HashMap<String, Def>,
}

impl Block {
    /// Construct a new primitive Vex [`Block`] from a `name` and a series of [`Def`]s.
    pub fn new<S: AsRef<str>, I: Into<Vec<Def>>>(name: S, defs: I) -> Self {
        let blkname = name.as_ref().to_string();
        let mut defmap = HashMap::new();
        for d in defs.into() {
            let _ = defmap.insert(d.name.clone(), d);
        }

        Self { name: blkname, defs: defmap }
    }
}

impl ToFromVex for Block {
    fn to_vex(&self) -> String {
        format!("${};\n{}", self.name, self.defs.values().map(|v| v.to_vex()).collect::<Vec<String>>().join("\n"))
    }

    fn to_vex_minified(&self) -> String {
        format!("${};{}", self.name, self.defs.values().map(|v| v.to_vex_minified()).collect::<Vec<String>>().join(""))
    }

    fn from_vex(vex: &str) -> Result<Self, VexError> {
        crate::parsing::Parser::new(vex).parse_primitive_block()
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
            let _ = defmap.insert(d.name.clone(), d);
        }

        Self { name: blkname, defs: defmap }
    }
}

impl ToFromVex for HighLevelBlock {
    fn to_vex(&self) -> String {
        format!("${};\n{}", self.name, self.defs.values().map(|v| v.to_vex()).collect::<Vec<String>>().join("\n"))
    }

    fn to_vex_minified(&self) -> String {
        format!("${};{}", self.name, self.defs.values().map(|v| v.to_vex_minified()).collect::<Vec<String>>().join(""))
    }

    fn from_vex(vex: &str) -> Result<Self, VexError> {
        crate::parsing::Parser::new(vex).parse_highlevel_block()
    }
}

#[cfg(test)]
mod tests {
    use crate::{def, refr};

    use super::*;

    #[test]
    fn test_parse_tofrom_block() -> miette::Result<()> {
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
        assert_eq!(start, Block::from_vex(start.to_vex().as_str())?);
        Ok(())
    }

    #[test]
    fn test_parse_tofrom_highlevelblock() -> miette::Result<()> {
        let start = HighLevelBlock::new("MY_BLOCK", [HighLevelDef::new("my_def", [refr!("BLK1" = "my_key"), refr!("BLK2" = "my_key2")])]);
        assert_eq!(start, HighLevelBlock::from_vex(start.to_vex().as_str())?);
        Ok(())
    }
}
