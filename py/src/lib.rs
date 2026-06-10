use pyo3::exceptions::{PyOSError, PyValueError};
use pyo3::prelude::*;
use std::convert::From;
use vexio::error::VexError;
use vexio::{Block, Def, Global, HighLevelBlock, HighLevelDef, Param, ParamValue, ParamValueKind, Scan, Schedule, Vex, VexRef};

/// `vexio` is a library for parsing, modifying, and creating Very Long Baseline Interferometry (VLBI) Experiment files, or VEX files, commonly used for describing radio astronomy VLBI experiments.
///
/// For examples on how to use this library, see the `examples/` directory in the library source.
#[pymodule(name = "_vexio")]
fn my_module(_py: Python, m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<PyVex>()?;
    m.add_class::<PyParam>()?;
    m.add_class::<PyParamValue>()?;
    m.add_class::<PyParamValueKind>()?;
    m.add_class::<PyVexRef>()?;
    m.add_class::<PyDef>()?;
    m.add_class::<PyGlobal>()?;
    m.add_class::<PyHighLevelDef>()?;
    m.add_class::<PyHighLevelBlock>()?;
    m.add_class::<PyBlock>()?;
    m.add_class::<PyScan>()?;
    m.add_class::<PySchedule>()?;
    Ok(())
}

#[pyclass(name = "Vex")]
/// A Vex file.
///
/// This is the top level type provided by this crate, and represents the full
/// contents of a Vex file. A ``Vex`` can be constructed manually, or parsed
/// from a string slice (see :meth:`parse`).
struct PyVex {
    inner: Vex,
}

#[pymethods]
impl PyVex {
    /// Construct an empty ``Vex``.
    #[new]
    fn new() -> Self {
        PyVex { inner: Vex::new() }
    }

    /// Open a Vex file using the provided file path.
    #[staticmethod]
    fn open(path: String) -> Result<Self, PyVexError> {
        let vex = Vex::open(path)?;
        Ok(Self { inner: vex })
    }

    /// Get a representation of this Vex as a Vex formatted string.
    fn to_vex(&self) -> String {
        self.inner.to_vex()
    }

    /// Get a representation of this Vex as a Vex formatted string, with
    /// unneccesary whitespace removed.
    fn to_vex_minified(&self) -> String {
        self.inner.to_vex_minified()
    }

    /// Parse a ``Vex`` from a Vex formatted string slice.
    #[staticmethod]
    fn parse(vex: &str) -> Result<Self, PyVexError> {
        let parsed = Vex::parse(vex)?;
        Ok(Self { inner: parsed })
    }

    /// Save this ``Vex`` to a file, overwriting the file at ``path`` if it
    /// already exists.
    fn save(&self, path: String) -> Result<(), PyVexError> {
        self.inner.save(path)?;
        Ok(())
    }

    /// Get the VEX_rev version of this Vex file.
    fn version(&self) -> (usize, usize) {
        (self.inner.rev.maj as usize, self.inner.rev.min as usize)
    }

    /// Set the VEX_rev version of this Vex file.
    fn set_version(&mut self, maj: usize, min: usize) {
        self.inner.rev = vexio::VexVersion { maj: maj as u8, min: min as u8 }
    }

    /// Add a `GLOBAL` high level block, overwriting the current one
    fn set_global(&mut self, global: &PyGlobal) {
        self.inner.set_global(global.inner.clone());
    }

    /// Add a `STATION` high level block, overwriting the current one
    fn set_station(&mut self, station: &PyHighLevelBlock) {
        self.inner.set_station(station.inner.defs().cloned().collect::<Vec<HighLevelDef>>());
    }

    /// Add a `MODE` high level block, overwriting the current one
    fn set_mode(&mut self, mode: &PyHighLevelBlock) {
        self.inner.set_mode(mode.inner.defs().cloned().collect::<Vec<HighLevelDef>>());
    }

    /// Add a primitive block (e.g. $EOP), overwriting the current one
    fn set_primitive(&mut self, primitive: &PyBlock) {
        self.inner.set_primitive(primitive.inner.clone());
    }

    /// Add a `SCHED` block, overwriting the current one
    fn set_schedule(&mut self, schedule: &PySchedule) {
        self.inner.set_schedule(schedule.inner.clone());
    }
}

struct PyVexError(VexError);

impl From<PyVexError> for PyErr {
    fn from(value: PyVexError) -> Self {
        match value.0 {
            VexError::IoError(e) => PyOSError::new_err(e.to_string()),
            VexError::ParseError(e) => PyValueError::new_err(e.description().to_string()),
            VexError::DuplicateBlock(e) => PyValueError::new_err(e),
            VexError::DuplicateDef(e) => PyValueError::new_err(e),
            VexError::MissingBlock(e) => PyValueError::new_err(e),
            VexError::MissingVexRev => PyValueError::new_err(value.0.to_string()),
        }
    }
}

impl From<VexError> for PyVexError {
    fn from(value: VexError) -> Self {
        Self(value)
    }
}

#[pyclass(name = "Param")]
/// A Vex parameter.
struct PyParam {
    inner: Param,
}

#[pymethods]
impl PyParam {
    /// Construct a new ``Param`` from a name and a series of values.
    #[new]
    fn new(name: String, values: Vec<String>) -> Self {
        Self { inner: Param::new(name, values) }
    }

    /// Construct a new parameter with no values.
    #[staticmethod]
    fn new_empty(name: String) -> Self {
        Self { inner: Param::new_empty(name) }
    }

    /// Get the name of this parameter.
    fn name(&self) -> String {
        self.inner.name().to_string()
    }

    /// Add a value to this parameter.
    fn add_value<'py>(&mut self, value: &Bound<'py, PyAny>) {
        // Convert whatever it was to a string and attempt to parse it
        self.inner.add_value(value.extract::<String>().unwrap_or("UNKNOWN_VALUE".to_string()));
    }

    /// Return an iterator over the values of this parameter.
    fn values(&self) -> Vec<String> {
        self.inner.values().map(|v| v.source().to_string()).collect()
    }

    /// Get a representation of this parameter as a Vex formatted string.
    fn to_vex(&self) -> String {
        self.inner.to_vex()
    }

    /// Get a representation of this parameter as a Vex formatted string, with
    /// unneccesary whitespace removed.
    fn to_vex_minified(&self) -> String {
        self.inner.to_vex_minified()
    }

    /// Parse a Vex parameter from a string slice.
    #[staticmethod]
    fn parse(vex: &str) -> Result<Self, PyVexError> {
        let parsed = Param::parse(vex).map_err(VexError::from)?;
        Ok(Self { inner: parsed })
    }
}

#[pyclass(name = "ParamValue")]
/// A single value of a defined Vex ``Param``.
///
/// Parameter values are contained in a colon separated list in the Vex format.
struct PyParamValue {
    inner: ParamValue,
}

#[pymethods]
impl PyParamValue {
    /// Get the source Vex of this value.
    fn source(&self) -> String {
        self.inner.source().to_string()
    }

    /// Return the ``ParamValueKind`` of this ``ParamValue``.
    fn kind(&self) -> PyParamValueKind {
        PyParamValueKind::from(self.inner.kind())
    }

    /// Get a representation of this value as a Vex formatted string.
    fn to_vex(&self) -> String {
        self.inner.to_vex()
    }

    /// Get a representation of this value as a Vex formatted string, with
    /// unneccesary whitespace removed.
    fn to_vex_minified(&self) -> String {
        self.inner.to_vex_minified()
    }

    /// Parse a value from a string slice.
    #[staticmethod]
    fn parse(vex: &str) -> Result<Self, PyVexError> {
        match ParamValue::parse(vex) {
            Ok(parsed) => Ok(PyParamValue { inner: parsed }),
            Err(e) => Err(PyVexError(VexError::ParseError(e))),
        }
    }
}

#[pyclass(name = "ParamValueKind")]
/// The type of a particular ``ParamValue``.
enum PyParamValueKind {
    Number,
    NumberUnit,
    Ident,
    Literal,
    Epoch,
    Coord,
    Link,
    Empty,
}

impl From<ParamValueKind> for PyParamValueKind {
    fn from(value: ParamValueKind) -> Self {
        match value {
            ParamValueKind::Coord => PyParamValueKind::Coord,
            ParamValueKind::Empty => PyParamValueKind::Empty,
            ParamValueKind::Epoch => PyParamValueKind::Epoch,
            ParamValueKind::Ident => PyParamValueKind::Ident,
            ParamValueKind::Link => PyParamValueKind::Link,
            ParamValueKind::Literal => PyParamValueKind::Literal,
            ParamValueKind::Number => PyParamValueKind::Number,
            ParamValueKind::NumberUnit => PyParamValueKind::NumberUnit,
        }
    }
}

#[pyclass(name = "VexRef")]
/// A Vex `ref` declaration.
struct PyVexRef {
    inner: VexRef,
}

#[pymethods]
impl PyVexRef {
    /// Construct a new ``VexRef`` from a ``blockname`` and a ``keyword``
    /// (the name of a ``Def`` within ``blockname``).
    #[new]
    fn new(blockname: String, keyword: String) -> Self {
        Self {
            inner: VexRef::new(blockname, keyword),
        }
    }

    /// Construct a new ``VexRef`` from a ``blockname``, a ``keyword``
    /// (the name of a ``Def`` within ``blockname``), and a series of qualifiers.
    #[staticmethod]
    fn new_with_quals(blockname: String, keyword: String, qualifiers: Vec<String>) -> Self {
        Self {
            inner: VexRef::new_with_quals(blockname, keyword, qualifiers),
        }
    }

    /// Get the blockname referenced by this ref.
    #[getter]
    fn blockname(&self) -> String {
        self.inner.blockname().to_string()
    }

    /// Get the def block referenced by this ref.
    #[getter]
    fn defname(&self) -> String {
        self.inner.defname().to_string()
    }

    /// Get an iterator over the qualifiers in this reference.
    #[getter]
    fn qualifiers(&self) -> Vec<String> {
        self.inner.quals().cloned().collect()
    }

    /// Add a qualifier to this reference.
    fn add_quals(&mut self, qual: String) {
        self.inner.add_quals(qual);
    }

    /// Get a representation of this reference as a Vex formatted string.
    fn to_vex(&self) -> String {
        self.inner.to_vex()
    }

    /// Get a representation of this reference as a Vex formatted string,
    /// with unneccesary whitespace removed.
    fn to_vex_minified(&self) -> String {
        self.inner.to_vex_minified()
    }

    /// Parse a ``VexRef`` from a string slice.
    #[staticmethod]
    fn parse(vex: &str) -> Result<Self, PyVexError> {
        match VexRef::parse(vex) {
            Ok(parsed) => Ok(PyVexRef { inner: parsed }),
            Err(e) => Err(PyVexError(VexError::ParseError(e))),
        }
    }
}

#[pyclass(name = "Def")]
/// A `def` block within a primitive Vex block.
struct PyDef {
    inner: Def,
}

#[pymethods]
impl PyDef {
    /// Construct a new ``Def`` block from a name and a series of ``Param``s.
    #[new]
    fn new(name: String) -> Self {
        Self { inner: Def::new_empty(name) }
    }

    /// Get the name of this def block.
    #[getter]
    fn name(&self) -> String {
        self.inner.name().to_string()
    }

    /// Add a new parameter to this ``Def``.
    fn add_param(&mut self, param: &PyParam) {
        self.inner.add_param(param.inner.clone());
    }

    /// Retrieve all ``Param``s with parameter names matching ``name``.
    fn get_param(&self, name: String) -> Option<PyParam> {
        self.inner.get(&name).first().map(|p| PyParam { inner: (*p).clone() })
    }

    /// Retrieve all ``Param``s with parameter names matching ``name``.
    fn get_params(&self, name: String) -> Vec<PyParam> {
        self.inner.get(&name).into_iter().map(|p| PyParam { inner: (*p).clone() }).collect()
    }

    /// Return an iterator over the names of parameters in this def.
    fn param_names(&self) -> Vec<String> {
        self.inner.params().map(|p| p.name().to_string()).collect()
    }

    /// Return an iterator over the ``Param``s in this def.
    fn params(&self) -> Vec<PyParam> {
        self.inner.params().cloned().map(|p| PyParam { inner: p }).collect()
    }

    /// Return a Vex formatted string representing this structure.
    fn to_vex(&self) -> String {
        self.inner.to_vex()
    }

    /// Return a Vex formatted string representing this structure,
    /// with unnecessary whitespace removed.
    fn to_vex_minified(&self) -> String {
        self.inner.to_vex_minified()
    }

    /// Attempt to parse a ``Def`` from a Vex formatted string.
    #[staticmethod]
    fn parse(vex: &str) -> Result<Self, PyVexError> {
        match Def::parse(vex) {
            Ok(parsed) => Ok(PyDef { inner: parsed }),
            Err(e) => Err(PyVexError(VexError::ParseError(e))),
        }
    }
}

#[pyclass(name = "Global")]
/// Represents the unique ``GLOBAL`` block within a Vex file.
struct PyGlobal {
    inner: Global,
}

#[pymethods]
impl PyGlobal {
    /// Make a new, empty, Global block.
    #[new]
    fn new() -> Self {
        Self { inner: Global::new() }
    }

    /// Add a ``VexRef`` to this global block.
    fn add_ref(&mut self, refr: &PyVexRef) {
        self.inner.add_ref(refr.inner.clone());
    }

    /// Retrieve references to all ``VexRef``s with block names matching ``blk``.
    fn get(&self, blk: String) -> Vec<PyVexRef> {
        self.inner.get(blk).into_iter().cloned().map(|r| PyVexRef { inner: r }).collect()
    }

    /// Return an iterator over all ``VexRef``s in this ``Global``.
    fn refs(&self) -> Vec<PyVexRef> {
        self.inner.as_slice().iter().cloned().map(|r| PyVexRef { inner: r }).collect()
    }

    /// Get a representation of this global block as a Vex formatted string.
    fn to_vex(&self) -> String {
        self.inner.to_vex()
    }

    /// Get a representation of this global block as a Vex formatted string,
    /// with unneccesary whitespace removed.
    fn to_vex_minified(&self) -> String {
        self.inner.to_vex_minified()
    }

    /// Parse a ``Global`` block from a string slice.
    #[staticmethod]
    fn parse(vex: &str) -> Result<Self, PyVexError> {
        match Global::parse(vex) {
            Ok(parsed) => Ok(PyGlobal { inner: parsed }),
            Err(e) => Err(PyVexError(VexError::ParseError(e))),
        }
    }
}

#[pyclass(name = "HighLevelDef")]
/// A `def` block within a high level Vex block.
struct PyHighLevelDef {
    inner: HighLevelDef,
}

#[pymethods]
impl PyHighLevelDef {
    /// Construct a new ``HighLevelDef`` from a name and a series of ``VexRef``s.
    #[new]
    fn new(name: String) -> Self {
        Self {
            inner: HighLevelDef::new_empty(name),
        }
    }

    /// Get the name of this def block.
    #[getter]
    fn name(&self) -> String {
        self.inner.name().to_string()
    }

    /// Add a ``VexRef`` to this ``HighLevelDef``.
    fn add_ref(&mut self, refr: &PyVexRef) {
        self.inner.add_ref(refr.inner.clone());
    }

    /// Retrieve references to all ``VexRef``s with block names matching ``blk``.
    fn get(&self, blk: String) -> Vec<PyVexRef> {
        self.inner.get(blk).into_iter().cloned().map(|r| PyVexRef { inner: r }).collect()
    }

    /// Return an iterator over all ``VexRef``s in this ``HighLevelDef``.
    fn refs(&self) -> Vec<PyVexRef> {
        self.inner.as_slice().iter().cloned().map(|r| PyVexRef { inner: r }).collect()
    }

    /// Get a representation of this high level def as a Vex formatted string.
    fn to_vex(&self) -> String {
        self.inner.to_vex()
    }

    /// Get a representation of this block as a Vex formatted string,
    /// with unnecessary whitespace removed.
    fn to_vex_minified(&self) -> String {
        self.inner.to_vex_minified()
    }

    /// Parse a ``HighLevelDef`` from a string slice.
    #[staticmethod]
    fn parse(vex: &str) -> Result<Self, PyVexError> {
        match HighLevelDef::parse(vex) {
            Ok(parsed) => Ok(PyHighLevelDef { inner: parsed }),
            Err(e) => Err(PyVexError(VexError::ParseError(e))),
        }
    }
}

#[pyclass(name = "HighLevelBlock")]
/// A high level Vex block. Currently this is either a `$MODE` or a `$STATION` block.
struct PyHighLevelBlock {
    inner: HighLevelBlock,
}

#[pymethods]
impl PyHighLevelBlock {
    /// Construct a new ``HighLevelBlock`` from a name and a series of ``HighLevelDef``s.
    #[new]
    fn new(name: String) -> Self {
        Self {
            inner: HighLevelBlock::new_empty(name),
        }
    }

    /// Get the name of this block. Currently this is either "MODE" or "STATION".
    #[getter]
    fn name(&self) -> String {
        self.inner.name().to_string()
    }

    /// Get a reference to the ``HighLevelDef`` block with name ``key``,
    /// returning ``None`` if it doesn't exist.
    fn get(&self, name: String) -> Option<PyHighLevelDef> {
        self.inner.get(name).map(|d| PyHighLevelDef { inner: d.clone() })
    }

    /// Return an iterator over all the names of ``HighLevelDef`` blocks
    /// within this ``HighLevelBlock``.
    #[getter]
    fn def_names(&self) -> Vec<String> {
        self.inner.def_names().cloned().collect()
    }

    /// Return an iterator over all the ``HighLevelDef`` blocks within this
    /// ``HighLevelBlock``.
    #[getter]
    fn defs(&self) -> Vec<PyHighLevelDef> {
        self.inner.defs().map(|d| PyHighLevelDef { inner: d.clone() }).collect()
    }

    /// Get a representation of this block as a Vex formatted string.
    fn to_vex(&self) -> String {
        self.inner.to_vex()
    }

    /// Get a representation of this block as a Vex formatted string,
    /// with unnecessary whitespace removed.
    fn to_vex_minified(&self) -> String {
        self.inner.to_vex_minified()
    }

    /// Parse a ``HighLevelBlock`` from a string slice.
    #[staticmethod]
    fn parse(vex: &str) -> Result<Self, PyVexError> {
        match HighLevelBlock::parse(vex) {
            Ok(parsed) => Ok(PyHighLevelBlock { inner: parsed }),
            Err(e) => Err(PyVexError(VexError::ParseError(e))),
        }
    }
}

#[pyclass(name = "Block")]
/// A primitive Vex block.
struct PyBlock {
    inner: Block,
}

#[pymethods]
impl PyBlock {
    /// Construct a new primitive Vex ``Block`` from a name and a series of
    /// ``Def``s.
    #[new]
    fn new(name: String) -> Self {
        Self { inner: Block::new_empty(name) }
    }

    /// Get the name of this block. This may be something like "EXPER" or
    /// "ANTENNA".
    #[getter]
    fn name(&self) -> String {
        self.inner.name().to_string()
    }

    /// Get a reference to the ``Def`` block with name ``key``, returning
    /// ``None`` if it doesn't exist.
    fn get(&self, name: String) -> Option<PyDef> {
        self.inner.get(name).map(|d| PyDef { inner: d.clone() })
    }

    /// Return an iterator over all the names of ``Def`` blocks within this
    /// ``Block``.
    #[getter]
    fn def_names(&self) -> Vec<String> {
        self.inner.def_names().cloned().collect()
    }

    /// Return an iterator over all the ``Def`` blocks within this ``Block``.
    #[getter]
    fn defs(&self) -> Vec<PyDef> {
        self.inner.defs().map(|d| PyDef { inner: d.clone() }).collect()
    }

    /// Add a ``Def`` to this block, overwriting one with the same name if it exists
    fn add_def(&mut self, def: &PyDef) {
        self.inner.add_def(def.inner.clone())
    }

    /// Get a representation of this block as a Vex formatted string.
    fn to_vex(&self) -> String {
        self.inner.to_vex()
    }

    /// Get a representation of this block as a Vex formatted string,
    /// with unnecessary whitespace removed.
    fn to_vex_minified(&self) -> String {
        self.inner.to_vex_minified()
    }

    /// Parse a ``Block`` from a string slice.
    #[staticmethod]
    fn parse(vex: &str) -> Result<Self, PyVexError> {
        match Block::parse(vex) {
            Ok(parsed) => Ok(PyBlock { inner: parsed }),
            Err(e) => Err(PyVexError(VexError::ParseError(e))),
        }
    }
}

#[pyclass(name = "Scan")]
/// A single scan within a ``Schedule``.
struct PyScan {
    inner: Scan,
}

#[pymethods]
impl PyScan {
    /// Construct a new ``Scan`` from a name and a series of ``Param``s.
    #[new]
    fn new(name: String) -> Self {
        Self { inner: Scan::new_empty(name) }
    }

    /// Get the name of this scan.
    #[getter]
    fn name(&self) -> String {
        self.inner.name().to_string()
    }

    /// Add a new parameter to this ``Scan``.
    fn add_param(&mut self, param: &PyParam) {
        self.inner.add_param(param.inner.clone());
    }

    /// Retrieve all ``Param``s with parameter names matching ``name``.
    fn get_param(&self, name: String) -> Option<PyParam> {
        self.inner.get(&name).first().map(|p| PyParam { inner: (*p).clone() })
    }

    /// Retrieve all ``Param``s with parameter names matching ``name``.
    fn get_params(&self, name: String) -> Vec<PyParam> {
        self.inner.get(&name).into_iter().map(|p| PyParam { inner: (*p).clone() }).collect()
    }

    /// Return an iterator over the names of parameters in this scan.
    fn param_names(&self) -> Vec<String> {
        self.inner.params().map(|p| p.name().to_string()).collect()
    }

    /// Return an iterator over all the parameters in this scan.
    fn params(&self) -> Vec<PyParam> {
        self.inner.params().cloned().map(|p| PyParam { inner: p }).collect()
    }

    /// Get a representation of this scan as a Vex formatted string.
    fn to_vex(&self) -> String {
        self.inner.to_vex()
    }

    /// Get a representation of this scan as a Vex formatted string,
    /// with unneccesary whitespace removed.
    fn to_vex_minified(&self) -> String {
        self.inner.to_vex_minified()
    }

    /// Parse a scan from a string slice.
    #[staticmethod]
    fn parse(vex: &str) -> Result<Self, PyVexError> {
        match Scan::parse(vex) {
            Ok(parsed) => Ok(PyScan { inner: parsed }),
            Err(e) => Err(PyVexError(VexError::ParseError(e))),
        }
    }
}

#[pyclass(name = "Schedule")]
/// Represents the unique ``SCHED`` block within a Vex file.
struct PySchedule {
    inner: Schedule,
}

#[pymethods]
impl PySchedule {
    /// Create a new empty schedule.
    #[new]
    fn new() -> Self {
        Self { inner: Schedule::new() }
    }

    /// Add a scan to this schedule.
    fn add_scan(&mut self, scan: &PyScan) {
        self.inner.add_scan(scan.inner.clone());
    }

    /// Return an iterator over the scans in this schedule.
    #[getter]
    fn scans(&self) -> Vec<PyScan> {
        self.inner.scans().map(|s| PyScan { inner: s.clone() }).collect()
    }

    /// Get a representation of this schedule as a Vex formatted string.
    fn to_vex(&self) -> String {
        self.inner.to_vex()
    }

    /// Get a representation of this schedule as a Vex formatted string,
    /// with unneccesary whitespace removed.
    fn to_vex_minified(&self) -> String {
        self.inner.to_vex_minified()
    }

    /// Parse a schedule from a string slice.
    #[staticmethod]
    fn parse(vex: &str) -> Result<Self, PyVexError> {
        match Schedule::parse(vex) {
            Ok(parsed) => Ok(PySchedule { inner: parsed }),
            Err(e) => Err(PyVexError(VexError::ParseError(e))),
        }
    }
}
