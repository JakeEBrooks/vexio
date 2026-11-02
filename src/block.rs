use crate::IntersperseIterator;
use crate::error::ParseError;
use crate::parsing::{Lexer, Parser, TokenKind};
use crate::value::VexValue;
use std::collections::HashMap;
use std::collections::hash_map::{Iter, IterMut};
use std::fmt::Display;
use std::ops::{Index, IndexMut};
use std::str::FromStr;

/// The type of a Vex block.
///
/// The high level blocks are currently MODE and STATION, the schedule block is SCHED,
/// and all other blocks are primitive.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize),
    derive(serde::Deserialize)
)]
pub enum BlockKind {
    /// A block whose definitions contain references to other blocks or definitions.
    HighLevel,
    /// A block containing primitive parameter definitions.
    Primitive,
    /// The schedule block, containing scan definitions.
    Schedule,
}

/// A Vex block containing definitions, references, or scans.
///
/// This represents any entry within a Vex file defined by a $\[BLOCKNAME\]
/// declaration. If the definitions inside contain `ref $[BLOCKNAME] = ...` statements
/// then it is of kind [`BlockKind::HighLevel`]. If it contains `scan [SCAN];` definitions
/// then it is of kind [`BlockKind::Schedule`]. Otherwise it contains standard `def [DEFNAME];`
/// definitions and it is ok kind [`BlockKind::Primitive`].
#[derive(Debug, Clone)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize),
    derive(serde::Deserialize)
)]
pub struct Block {
    kind: BlockKind,
    name: String,
    defs: HashMap<String, BlockDef>,
}

impl Block {
    /// Creates an empty high-level block with the given name.
    pub fn new_highlevel(name: impl AsRef<str>) -> Self {
        Self {
            kind: BlockKind::HighLevel,
            name: name.as_ref().to_ascii_uppercase(),
            defs: HashMap::new(),
        }
    }

    /// Creates an empty primitive block with the given name.
    pub fn new_primitive(name: impl AsRef<str>) -> Self {
        Self {
            kind: BlockKind::Primitive,
            name: name.as_ref().to_ascii_uppercase(),
            defs: HashMap::new(),
        }
    }

    /// Creates an empty schedule block named `SCHED`.
    pub fn new_schedule() -> Self {
        Self {
            kind: BlockKind::Schedule,
            name: "SCHED".to_string(),
            defs: HashMap::new(),
        }
    }

    /// Returns this block's kind.
    pub fn kind(&self) -> BlockKind {
        self.kind
    }

    /// Returns whether this is a high-level block.
    pub fn is_highlevel(&self) -> bool {
        self.kind == BlockKind::HighLevel
    }

    /// Returns whether this is a primitive block.
    pub fn is_primitive(&self) -> bool {
        self.kind == BlockKind::Primitive
    }

    /// Returns whether this is the schedule block.
    pub fn is_schedule(&self) -> bool {
        self.kind == BlockKind::Schedule
    }

    /// Returns this block's name.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Iterates over definitions as `(name, definition)` pairs.
    pub fn iter(&self) -> Iter<'_, String, BlockDef> {
        self.defs.iter()
    }

    /// Mutably iterates over definitions as `(name, definition)` pairs.
    pub fn iter_mut(&mut self) -> IterMut<'_, String, BlockDef> {
        self.defs.iter_mut()
    }

    /// Inserts a definition, returning any previous definition with the same name.
    pub fn add(&mut self, def: BlockDef) -> Option<BlockDef> {
        self.defs.insert(def.name.clone(), def)
    }

    /// Removes and returns the definition with the given name, if present.
    pub fn remove(&mut self, name: impl AsRef<str>) -> Option<BlockDef> {
        self.defs.remove(name.as_ref())
    }

    /// Returns the definition with the given name, if present.
    pub fn get(&self, name: impl AsRef<str>) -> Option<&BlockDef> {
        self.defs.get(name.as_ref())
    }

    /// Returns mutable access to the definition with the given name, if present.
    pub fn get_mut(&mut self, name: impl AsRef<str>) -> Option<&mut BlockDef> {
        self.defs.get_mut(name.as_ref())
    }

    /// Returns whether this block contains a definition with the given name.
    pub fn contains_key(&self, key: impl AsRef<str>) -> bool {
        self.defs.contains_key(key.as_ref())
    }

    fn to_vex(&self) -> String {
        let defs_vex = self.defs.values().map(|d| d.to_vex()).collect::<String>();
        format!("\n${};\n{}", self.name.to_uppercase(), defs_vex)
    }
}

impl Display for Block {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_vex())
    }
}

impl FromStr for Block {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Parser::new(s)?.parse_block()
    }
}

impl<I: AsRef<str>> Index<I> for Block {
    type Output = BlockDef;

    fn index(&self, index: I) -> &Self::Output {
        self.get(index.as_ref()).unwrap_or_else(|| {
            panic!(
                "Def block '{}' does not exist in {}",
                index.as_ref(),
                self.name()
            )
        })
    }
}

impl<I: AsRef<str>> IndexMut<I> for Block {
    fn index_mut(&mut self, index: I) -> &mut Self::Output {
        let blk_name = self.name.clone();
        let def_name = index.as_ref();
        self.get_mut(index.as_ref())
            .unwrap_or_else(|| panic!("Def block '{}' does not exist in {}", def_name, blk_name))
    }
}

impl<'a> IntoIterator for &'a Block {
    type Item = (&'a String, &'a BlockDef);
    type IntoIter = Iter<'a, String, BlockDef>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'a> IntoIterator for &'a mut Block {
    type Item = (&'a String, &'a mut BlockDef);
    type IntoIter = IterMut<'a, String, BlockDef>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter_mut()
    }
}

/// A named definition or scan containing one or more parameter entries.
///
/// The exact Vex formatting of this block is defined by its parent [`BlockKind`], but
/// in any case it can be viewed as a map of key-value pairs. Each key can have multiple definitions
/// which are collected together under the same entry in the map, hence the return type of [`Vec<Vec<VexValue>>`].
/// The inner `Vec` contains the parameter values usually defined on a single line.
///
/// As an example, the following Vex:
/// ```text
///    sample_rate = 64.000000 Ms/sec;
///    chan_def =   : 4830.49000000 MHz : L : 32.00000000 MHz : &CH01 : &BBC01 : &NoCal;
///    chan_def =   : 4830.49000000 MHz : L : 32.00000000 MHz : &CH02 : &BBC09 : &NoCal;
///    chan_def =   : 4830.49000000 MHz : U : 32.00000000 MHz : &CH03 : &BBC01 : &NoCal;
/// ```
/// Would be parsed into a map containing two keys: `sample_rate` and `chan_def`. Getting the `sample_rate` entry would
/// return a reference to an outer `Vec` of length one, whereas getting the `chan_def` entry would return an outer `Vec` of length 3.
///
/// The first and only inner `Vec` of the `sample_rate` entry would contain one element: the single defined [`VexValue`]. The three inner `Vec`s of the
/// `chan_def` entry will all be of length 7, representing the values between colons (including the empty value at the beginning).
#[derive(Debug, Clone)]
#[cfg_attr(
    feature = "serde",
    derive(serde::Serialize),
    derive(serde::Deserialize)
)]
pub struct BlockDef {
    parent: BlockKind,
    name: String,
    values: HashMap<String, Vec<Vec<VexValue>>>,
}

impl BlockDef {
    /// Creates an empty definition with a name and its parent block kind.
    pub fn new(name: impl AsRef<str>, parentkind: BlockKind) -> Self {
        Self {
            parent: parentkind,
            name: name.as_ref().to_string(),
            values: HashMap::new(),
        }
    }

    /// Creates an empty definition belonging to a high-level block.
    pub fn new_highlevel(name: impl AsRef<str>) -> Self {
        Self::new(name, BlockKind::HighLevel)
    }

    /// Creates an empty definition belonging to a primitive block.
    pub fn new_primitive(name: impl AsRef<str>) -> Self {
        Self::new(name, BlockKind::Primitive)
    }

    /// Creates an empty scan definition.
    pub fn new_scan(name: impl AsRef<str>) -> Self {
        Self::new(name, BlockKind::Schedule)
    }

    /// Returns the kind of block this definition belongs to.
    pub fn parentkind(&self) -> BlockKind {
        self.parent
    }

    /// Sets the kind of block this definition belongs to.
    pub fn set_parentkind(&mut self, kind: BlockKind) {
        self.parent = kind
    }

    /// Returns this definition's name.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Sets this definition's name.
    pub fn set_name(&mut self, name: impl AsRef<str>) {
        self.name = name.as_ref().to_string()
    }

    /// Returns all occurrences of the named entry, if present.
    ///
    /// Each inner vector contains the colon-separated values for one occurrence.
    pub fn get(&self, name: impl AsRef<str>) -> Option<&Vec<Vec<VexValue>>> {
        self.values.get(name.as_ref())
    }

    /// Returns mutable access to all occurrences of the named entry, if present.
    pub fn get_mut(&mut self, name: impl AsRef<str>) -> Option<&mut Vec<Vec<VexValue>>> {
        self.values.get_mut(name.as_ref())
    }

    /// Returns the first occurrence of an entry, if present.
    pub fn get_first(&self, name: impl AsRef<str>) -> Option<&Vec<VexValue>> {
        let found = self.get(name.as_ref())?;
        found.first()
    }

    /// Returns mutable access to the first occurrence of an entry, if present.
    pub fn get_first_mut(&mut self, name: impl AsRef<str>) -> Option<&mut Vec<VexValue>> {
        let found = self.get_mut(name.as_ref())?;
        found.first_mut()
    }

    /// Returns whether this definition contains the named entry.
    pub fn contains_key(&self, name: impl AsRef<str>) -> bool {
        self.values.contains_key(name.as_ref())
    }

    /// Appends one occurrence of an entry, using `values` as its colon-separated values.
    pub fn add(&mut self, name: impl AsRef<str>, values: impl AsRef<[VexValue]>) {
        match self.get_mut(name.as_ref()) {
            Some(found) => found.push(values.as_ref().to_vec()),
            None => {
                self.values
                    .insert(name.as_ref().to_string(), vec![values.as_ref().to_vec()]);
            }
        }
    }

    /// Removes and returns all occurrences of an entry, if present.
    pub fn remove(&mut self, name: impl AsRef<str>) -> Option<Vec<Vec<VexValue>>> {
        self.values.remove(name.as_ref())
    }

    /// Removes and returns one occurrence by index, or `None` if the entry or index is absent.
    pub fn remove_single(&mut self, name: impl AsRef<str>, index: usize) -> Option<Vec<VexValue>> {
        let map = self.values.get_mut(name.as_ref())?;
        if index >= map.len() {
            None
        } else {
            Some(map.remove(index))
        }
    }

    /// Iterates over entries as `(name, occurrences)` pairs.
    pub fn iter(&self) -> Iter<'_, String, Vec<Vec<VexValue>>> {
        self.values.iter()
    }

    /// Mutably iterates over entries as `(name, occurrences)` pairs.
    pub fn iter_mut(&mut self) -> IterMut<'_, String, Vec<Vec<VexValue>>> {
        self.values.iter_mut()
    }

    /// Iterates over every entry occurrence as `(name, values)` pairs.
    pub fn iter_flattened(&self) -> impl Iterator<Item = (&String, &Vec<VexValue>)> {
        self.iter()
            .flat_map(|(k, v)| v.iter().map(move |inner| (k, inner)))
    }

    /// Mutably iterates over every entry occurrence as `(name, values)` pairs.
    pub fn iter_mut_flattened(&mut self) -> impl Iterator<Item = (&String, &mut Vec<VexValue>)> {
        self.iter_mut()
            .flat_map(|(k, v)| v.iter_mut().map(move |inner| (k, inner)))
    }

    pub(crate) fn set_values(&mut self, entries: HashMap<String, Vec<Vec<VexValue>>>) {
        self.values = entries
    }

    fn to_vex(&self) -> String {
        if self.parent == BlockKind::Schedule {
            format!(
                "scan {};\n{}endscan;\n",
                self.name(),
                match self.parent {
                    BlockKind::HighLevel => self.ref_entries_to_vex(),
                    BlockKind::Primitive | BlockKind::Schedule => self.primitive_entries_to_vex(),
                }
            )
        } else {
            format!(
                "def {};\n{}enddef;\n",
                self.name(),
                match self.parent {
                    BlockKind::HighLevel => self.ref_entries_to_vex(),
                    BlockKind::Primitive | BlockKind::Schedule => self.primitive_entries_to_vex(),
                }
            )
        }
    }

    fn primitive_entries_to_vex(&self) -> String {
        let mut out = String::new();
        for (k, v) in self.iter_flattened() {
            let it = IntersperseIterator::new(v.iter().map(|v| v.to_string()), " : ".to_string())
                .collect::<String>();
            out += &format!("\t{k} = {it};\n")
        }
        out
    }

    fn ref_entries_to_vex(&self) -> String {
        let mut out = String::new();
        for (k, v) in self.iter_flattened() {
            let it = IntersperseIterator::new(v.iter().map(|v| v.to_string()), " : ".to_string())
                .collect::<String>();
            out += &format!("\tref ${} = {it};\n", k.to_uppercase())
        }
        out
    }
}

fn infer_parentkind(source: &str) -> BlockKind {
    let lexer = Lexer::new(source);
    if let Ok(tokens) = lexer.lex() {
        let kinds: Vec<TokenKind> = tokens.into_iter().map(|t| t.kind).collect();
        if kinds.contains(&TokenKind::Scan) && kinds.contains(&TokenKind::EndScan) {
            BlockKind::Schedule
        } else if kinds.contains(&TokenKind::Ref)
            && kinds.contains(&TokenKind::Def)
            && kinds.contains(&TokenKind::EndDef)
        {
            BlockKind::HighLevel
        } else {
            BlockKind::Primitive
        }
    } else {
        BlockKind::Primitive
    }
}

impl Display for BlockDef {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.to_vex())
    }
}

impl FromStr for BlockDef {
    type Err = ParseError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut parser = Parser::new(s)?;
        let parentkind = infer_parentkind(s);
        match parentkind {
            BlockKind::Schedule => parser.parse_scan(),
            BlockKind::HighLevel | BlockKind::Primitive => parser.parse_blockdef(parentkind),
        }
    }
}

impl<I: AsRef<str>> Index<I> for BlockDef {
    type Output = Vec<Vec<VexValue>>;

    fn index(&self, index: I) -> &Self::Output {
        self.get(index.as_ref()).unwrap_or_else(|| {
            panic!(
                "Entry '{}' does not exist in {}",
                index.as_ref(),
                self.name()
            )
        })
    }
}

impl<I: AsRef<str>> IndexMut<I> for BlockDef {
    fn index_mut(&mut self, index: I) -> &mut Self::Output {
        let def_name = self.name.clone();
        let entry_name = index.as_ref();
        self.get_mut(index.as_ref())
            .unwrap_or_else(|| panic!("Entry '{}' does not exist in {}", entry_name, def_name))
    }
}

impl<'a> IntoIterator for &'a BlockDef {
    type Item = (&'a String, &'a Vec<Vec<VexValue>>);
    type IntoIter = Iter<'a, String, Vec<Vec<VexValue>>>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}

impl<'a> IntoIterator for &'a mut BlockDef {
    type Item = (&'a String, &'a mut Vec<Vec<VexValue>>);
    type IntoIter = IterMut<'a, String, Vec<Vec<VexValue>>>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter_mut()
    }
}
