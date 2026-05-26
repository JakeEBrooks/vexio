use std::collections::HashMap;
use std::io::{Read, Seek};
use std::path::Path;

use crate::error::ParseError;
use crate::parsing::Parser;
use crate::{Block, Global, HighLevelBlock, HighLevelDef, Schedule, VexError};

/// A Vex file
///
/// This is the top level type provided by this crate, and represents the full contents of a Vex file.
/// A [`Vex`] can be constructed manually, or parsed from a string slice (see [`Self::parse`])
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize), derive(serde::Deserialize))]
pub struct Vex {
    /// The value of the VEX_rev definition
    pub rev: VexVersion,
    /// The [`Global`] block for this [`Vex`]
    pub global: Global,
    /// The `STATION` [`HighLevelBlock`] for this [`Vex`]
    pub station: HighLevelBlock,
    /// The `MODE` [`HighLevelBlock`] for this [`Vex`]
    pub mode: HighLevelBlock,
    /// The primitive blocks for this [`Vex`]
    pub primitives: HashMap<String, Block>,
    /// The [`Schedule`] for this [`Vex`]
    pub schedule: Schedule,
}

impl Vex {
    /// Construct an empty [`Vex`]
    pub fn new() -> Self {
        Self {
            rev: VexVersion::default(),
            global: Global::new(),
            station: HighLevelBlock::new("STATION", []),
            mode: HighLevelBlock::new("MODE", []),
            primitives: HashMap::new(),
            schedule: Schedule::new(),
        }
    }

    /// Open a Vex file using the provided file path
    pub fn open(path: impl AsRef<Path>) -> Result<Self, VexError> {
        let mut file = std::fs::File::open(path)?;
        let mut contents = String::new();

        // The first 9 bytes should be "VEX_rev ="
        // Don't bother reading the whole file and attempting to parse
        // unless it is
        let mut vex_rev_bytes: [u8; 9] = [0u8; 9];
        file.read_exact(&mut vex_rev_bytes)?;

        if &vex_rev_bytes == b"VEX_rev =" {
            file.rewind()?;
            file.read_to_string(&mut contents)?;
            Vex::parse(&contents)
        } else {
            Err(VexError::MissingVexRev)
        }
    }

    /// Add a `GLOBAL` high level block, overwriting the current one
    pub fn with_global(mut self, global: Global) -> Self {
        self.global = global;
        self
    }

    /// Add a `STATION` high level block, overwriting the current one
    pub fn with_station(mut self, defs: impl Into<Vec<HighLevelDef>>) -> Self {
        self.station = HighLevelBlock::new("STATION", defs.into());
        self
    }

    /// Add a `MODE` high level block, overwriting the current one
    pub fn with_mode(mut self, defs: impl Into<Vec<HighLevelDef>>) -> Self {
        self.mode = HighLevelBlock::new("MODE", defs.into());
        self
    }

    /// Add a primitive block.
    ///
    /// This will overwrite a block with the same name
    pub fn with_block(mut self, block: Block) -> Self {
        let _ = self.primitives.insert(block.name().to_string(), block);
        self
    }

    /// Add a `SCHED` block, overwriting the current one
    pub fn with_schedule(mut self, sched: Schedule) -> Self {
        self.schedule = sched;
        self
    }

    /// Get a representation of this Vex as a Vex formatted string
    pub fn to_vex(&self) -> String {
        format!(
            "{}\n{}\n{}\n{}\n{}\n{}",
            self.rev.to_vex(),
            self.global.to_vex(),
            self.mode.to_vex(),
            self.station.to_vex(),
            self.primitives.values().map(|b| b.to_vex()).collect::<Vec<String>>().join("\n"),
            self.schedule.to_vex()
        )
    }

    /// Get a representation of this Vex as a Vex formatted string, with unneccesary whitespace removed
    pub fn to_vex_minified(&self) -> String {
        format!(
            "{}{}{}{}{}{}",
            self.rev.to_vex_minified(),
            self.global.to_vex_minified(),
            self.mode.to_vex_minified(),
            self.station.to_vex_minified(),
            self.primitives.values().map(|b| b.to_vex_minified()).collect::<Vec<String>>().join(""),
            self.schedule.to_vex_minified()
        )
    }

    /// Parse a [`Vex`] from a Vex formatted string slice
    pub fn parse(vex: &str) -> Result<Self, VexError> {
        Parser::new(vex).parse()
    }

    /// Save this [`Vex`] to a file, overwriting the file at `path` if it already exists
    pub fn save(&self, path: impl AsRef<Path>) -> Result<(), VexError> {
        std::fs::write(path, self.to_vex())?;
        Ok(())
    }
}

impl Default for Vex {
    fn default() -> Self {
        Vex::new()
    }
}

/// The version of a Vex file. Typically parsed from a `VEX_rev` parameter.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize), derive(serde::Deserialize))]
pub struct VexVersion {
    /// The major version
    pub maj: u8,
    /// The minor version
    pub min: u8,
}

impl VexVersion {
    /// Create a new [`VexVersion`] from a major and minor version
    pub fn new(maj: u8, min: u8) -> Self {
        Self { maj, min }
    }

    /// Get a representation of this vex version as a Vex formatted string
    pub fn to_vex(&self) -> String {
        format!("VEX_rev = {}.{};", self.maj, self.min)
    }

    /// Get a representation of this vex version as a Vex formatted string, with unneccesary whitespace removed
    pub fn to_vex_minified(&self) -> String {
        format!("VEX_rev={}.{};", self.maj, self.min)
    }

    /// Parse a vex version from a string slice
    pub fn parse(vex: &str) -> Result<Self, ParseError> {
        Parser::new(vex).parse_vex_version()
    }
}

impl Default for VexVersion {
    fn default() -> Self {
        Self { maj: 2, min: 0 }
    }
}
