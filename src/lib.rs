//! `vexio` is a library for parsing, modifying, and creating Very Long Baseline Interferometry (VLBI) Experiment files, or VEX files, commonly used for describing radio astronomy VLBI experiments.
//!
//! For examples on how to use this library, see the `examples/` directory in the crate source.

#![warn(missing_docs)]

mod block;
mod def;
mod error;
mod global;
mod param;
pub mod parsing;
mod reference;
mod schedule;
mod vex;
pub use block::*;
pub use def::{Def, HighLevelDef};
pub use error::VexError;
pub use global::Global;
pub use param::{Param, ParamValue, ParamValueKind};
pub use reference::VexRef;
pub use schedule::{Scan, Schedule};
pub use vex::{Vex, VexVersion};

/// Valid high level Vex blocks
pub const HIGH_LEVEL_BLOCKS: [&str; 3] = ["GLOBAL", "STATION", "MODE"];

/// Valid primitive Vex blocks
pub const PRIMITIVE_BLOCKS: [&str; 19] = [
    "ANTENNA",
    "BBC",
    "BITSTREAMS",
    "CLOCK",
    "DAS",
    "DATASTREAMS",
    "EOP",
    "EXPER",
    "EXTENSIONS",
    "FREQ",
    "PHASE_CAL_DETECT",
    "IF",
    "SEFD",
    "SITE",
    "SCHEDULING_PARAMS",
    "SOURCE",
    "PROCEDURES",
    "TRACKS",
    "TAPELOG_OBS",
];
