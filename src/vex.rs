use std::collections::HashMap;
use std::io::{Read, Seek};
use std::path::Path;

use crate::parsing::Parser;
use crate::{Block, Global, HighLevelBlock, HighLevelDef, Schedule, VexError};

/// A Vex file
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
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self, VexError> {
        let path = path.as_ref().to_path_buf();
        let fname = path.file_name().ok_or(VexError::new(format!("Unable to open file at: {}", path.display())))?;
        let mut file = std::fs::File::open(path.clone())?;
        let mut contents = String::new();

        // The first 9 bytes should be "VEX_rev ="
        // Don't bother reading the whole file and attempting to parse
        // unless it is
        let mut vex_rev_bytes: [u8; 9] = [0u8; 9];
        file.read_exact(&mut vex_rev_bytes)?;

        if &vex_rev_bytes == b"VEX_rev =" {
            file.rewind()?;
            file.read_to_string(&mut contents)?;
            Vex::from_vex(&contents).map_err(|e| e.with_filename(fname.to_string_lossy()))
        } else {
            Err(VexError::new_at("Missing VEX_rev", 0)
                .with_src(std::str::from_utf8(&vex_rev_bytes).map_err(|_| VexError::new("Encountered invalid utf-8"))?)
                .with_filename(fname.to_string_lossy())
                .with_help("Are you sure this is a Vex file?"))
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
        let _ = self.primitives.insert(block.name.clone(), block);
        self
    }

    /// Add a `SCHED` block, overwriting the current one
    pub fn with_schedule(mut self, sched: Schedule) -> Self {
        self.schedule = sched;
        self
    }
}

impl Default for Vex {
    fn default() -> Self {
        Vex::new()
    }
}

impl ToFromVex for Vex {
    fn to_vex(&self) -> String {
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

    fn to_vex_minified(&self) -> String {
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

    fn from_vex(vex: &str) -> Result<Self, VexError> {
        Parser::new(vex).parse()
    }
}

/// Represents any type that can be converted into Vex formatted text, or parsed from it
pub trait ToFromVex
where
    Self: Sized,
{
    /// Produce a Vex formatted [`String`]
    fn to_vex(&self) -> String;
    /// Produce a Vex formatted [`String`] with unnecessary whitespace removed
    fn to_vex_minified(&self) -> String;
    /// Parse [`Self`] from a Vex formatted string slice
    fn from_vex(vex: &str) -> Result<Self, VexError>;
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
}

impl Default for VexVersion {
    fn default() -> Self {
        Self { maj: 2, min: 0 }
    }
}

impl ToFromVex for VexVersion {
    fn to_vex(&self) -> String {
        format!("VEX_rev = {}.{};", self.maj, self.min)
    }

    fn to_vex_minified(&self) -> String {
        format!("VEX_rev={}.{};", self.maj, self.min)
    }

    fn from_vex(vex: &str) -> Result<Self, VexError> {
        let (lhs, rhs) = vex.split_once(".").ok_or(VexError::new(format!("Unable to parse a Vex version from {}", vex)))?;
        let maj: u8 = lhs.parse().map_err(|_| VexError::new(format!("Unable to parse a Vex major version from {}", lhs)))?;
        let min: u8 = lhs.parse().map_err(|_| VexError::new(format!("Unable to parse a Vex minor version from {}", rhs)))?;
        Ok(Self { maj, min })
    }
}
