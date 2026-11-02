use std::collections::HashMap;
use std::collections::hash_map::{Iter, IterMut};
use std::fmt::Display;
use std::fs::File;
use std::io::{Read, Seek};
use std::ops::{Index, IndexMut};
use std::path::Path;
use std::str::FromStr;

use crate::error::{ParseError, ParseErrorKind, VexError};
use crate::parsing::Parser;
use crate::{Block, Global};

/// A complete VEX document, including its version, global block, and named blocks.
///
/// A [`Vex`] can be constructed manually, opened from a file, or parsed from a string
/// using [`str::parse`].
#[derive(Debug, Clone)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize),
    derive(serde::Deserialize)
)]
pub struct Vex {
    version: VexVersion,
    global: Global,
    blocks: HashMap<String, Block>,
}

impl Vex {
    /// Creates an empty VEX document using the default VEX version.
    pub fn new() -> Self {
        Self {
            version: VexVersion::default(),
            global: Global::default(),
            blocks: HashMap::new(),
        }
    }

    /// Opens and parses a VEX document from a file path.
    pub fn open<P: AsRef<Path>>(path: P) -> Result<Self, VexError> {
        let mut file = File::open(path)?;

        // Read a small amount to check for a valid vex file
        let mut header_bytes = [0u8; 128];
        let n = file.read(&mut header_bytes)?;
        // Now try to parse a vex_rev
        let _vexrev = Parser::new(std::str::from_utf8(&header_bytes[..n]).map_err(|_| {
            ParseError::new(
                ParseErrorKind::InvalidVexRev,
                String::from_utf8_lossy(&header_bytes),
                0,
                0,
            )
        })?)?
        .parse_vex_version()?;

        // If we did then start over and attempt to parse the whole thing
        file.rewind()?;
        let mut contents = String::new();
        file.read_to_string(&mut contents)?;
        let vex = Parser::new(&contents)?.parse_vex()?;
        Ok(vex)
    }

    /// Return the GLOBAL block.
    pub fn global(&self) -> &Global {
        &self.global
    }

    /// Returns mutable access to the GLOBAL block.
    pub fn global_mut(&mut self) -> &mut Global {
        &mut self.global
    }

    /// Returns the VEX version.
    pub fn version(&self) -> VexVersion {
        self.version
    }

    /// Sets the major and minor VEX version numbers.
    pub fn set_version(&mut self, maj: usize, min: usize) {
        self.version = VexVersion {
            maj: maj as u8,
            min: min as u8,
        }
    }

    /// Inserts a block, returning any previous block with the same name.
    pub fn add(&mut self, block: Block) -> Option<Block> {
        self.blocks.insert(block.name().to_string(), block)
    }

    /// Returns the block with the given name, if present.
    pub fn get(&self, name: impl AsRef<str>) -> Option<&Block> {
        self.blocks.get(name.as_ref())
    }

    /// Returns mutable access to the block with the given name, if present.
    pub fn get_mut(&mut self, name: impl AsRef<str>) -> Option<&mut Block> {
        self.blocks.get_mut(name.as_ref())
    }

    /// Iterates over blocks as `(name, block)` pairs.
    pub fn iter(&self) -> Iter<'_, String, Block> {
        self.blocks.iter()
    }

    /// Mutably iterates over blocks as `(name, block)` pairs.
    pub fn iter_mut(&mut self) -> IterMut<'_, String, Block> {
        self.blocks.iter_mut()
    }

    /// Returns whether a block with the given name is present.
    pub fn contains_key(&self, name: impl AsRef<str>) -> bool {
        self.blocks.contains_key(name.as_ref())
    }

    /// Removes and returns the block with the given name, if present.
    pub fn remove(&mut self, name: impl AsRef<str>) -> Option<Block> {
        self.blocks.remove(name.as_ref())
    }

    fn to_vex(&self) -> String {
        let mut vex = format!("{}\n{}", self.version, self.global);
        // Print MODE and STATION first, then SCHED last
        if let Some(mode) = self.get("MODE") {
            vex += &mode.to_string();
        }

        if let Some(station) = self.get("STATION") {
            vex += &station.to_string();
        }

        for (k, v) in self.blocks.iter() {
            if !matches!(k.as_str(), "MODE" | "STATION" | "SCHED") {
                vex += &v.to_string();
            }
        }

        if let Some(sched) = self.get("SCHED") {
            vex += &sched.to_string();
        }

        vex
    }
}

impl Default for Vex {
    fn default() -> Self {
        Vex::new()
    }
}

impl Display for Vex {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_vex())
    }
}

impl FromStr for Vex {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Parser::new(s)?.parse_vex()
    }
}

impl<I: AsRef<str>> Index<I> for Vex {
    type Output = Block;

    fn index(&self, index: I) -> &Self::Output {
        self.get(index.as_ref())
            .unwrap_or_else(|| panic!("Vex block '{}' does not exist", index.as_ref()))
    }
}

impl<I: AsRef<str>> IndexMut<I> for Vex {
    fn index_mut(&mut self, index: I) -> &mut Self::Output {
        self.get_mut(index.as_ref())
            .unwrap_or_else(|| panic!("Vex block '{}' does not exist", index.as_ref()))
    }
}

impl<'a> IntoIterator for &'a Vex {
    type Item = (&'a String, &'a Block);
    type IntoIter = Iter<'a, String, Block>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'a> IntoIterator for &'a mut Vex {
    type Item = (&'a String, &'a mut Block);
    type IntoIter = IterMut<'a, String, Block>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter_mut()
    }
}

/// The version of a Vex file. Typically parsed from a `VEX_rev` parameter.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize),
    derive(serde::Deserialize)
)]
pub struct VexVersion {
    /// The major version
    pub maj: u8,
    /// The minor version
    pub min: u8,
}

impl VexVersion {
    /// Create a new vex version from a major and minor version
    pub fn new(maj: u8, min: u8) -> Self {
        Self { maj, min }
    }

    fn to_vex(self) -> String {
        format!("VEX_rev = {}.{};", self.maj, self.min)
    }
}

impl Default for VexVersion {
    fn default() -> Self {
        Self { maj: 2, min: 0 }
    }
}

impl Display for VexVersion {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_vex())
    }
}

impl FromStr for VexVersion {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Parser::new(s)?.parse_vex_version()
    }
}
