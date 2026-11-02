use std::str::FromStr;

use pyo3::exceptions::{PyKeyError, PyOSError, PyValueError};
use pyo3::prelude::*;
use vexio::{
    Block as RustBlock, BlockDef as RustBlockDef, BlockKind as RustBlockKind, Global as RustGlobal,
    ParseError, Vex as RustVex, VexError, VexValue as RustVexValue,
    VexValueKind as RustVexValueKind, VexVersion as RustVexVersion,
};

fn parse_error(error: ParseError) -> PyErr {
    PyValueError::new_err(error.to_string())
}

fn vex_error(error: VexError) -> PyErr {
    match error {
        VexError::IoError(error) => PyOSError::new_err(error.to_string()),
        VexError::ParseError(error) => parse_error(error),
    }
}

fn wrap_value(py: Python<'_>, value: RustVexValue) -> PyResult<Py<PyVexValue>> {
    Py::new(py, PyVexValue { inner: value })
}

fn wrap_values(py: Python<'_>, values: Vec<RustVexValue>) -> PyResult<Vec<Py<PyVexValue>>> {
    values
        .into_iter()
        .map(|value| wrap_value(py, value))
        .collect()
}

fn clone_values(py: Python<'_>, values: Vec<Py<PyVexValue>>) -> Vec<RustVexValue> {
    values
        .into_iter()
        .map(|value| value.borrow(py).inner.clone())
        .collect()
}

#[pyclass(name = "BlockKind", eq, eq_int)]
#[derive(Clone, Copy, PartialEq, Eq)]
enum PyBlockKind {
    HighLevel,
    Primitive,
    Schedule,
}

impl From<PyBlockKind> for RustBlockKind {
    fn from(kind: PyBlockKind) -> Self {
        match kind {
            PyBlockKind::HighLevel => Self::HighLevel,
            PyBlockKind::Primitive => Self::Primitive,
            PyBlockKind::Schedule => Self::Schedule,
        }
    }
}

impl From<RustBlockKind> for PyBlockKind {
    fn from(kind: RustBlockKind) -> Self {
        match kind {
            RustBlockKind::HighLevel => Self::HighLevel,
            RustBlockKind::Primitive => Self::Primitive,
            RustBlockKind::Schedule => Self::Schedule,
        }
    }
}

#[pyclass(name = "VexValueKind", eq, eq_int)]
#[derive(Clone, Copy, PartialEq, Eq)]
enum PyVexValueKind {
    Number,
    NumberUnit,
    Ident,
    Literal,
    Epoch,
    RightAscension,
    Declination,
    Link,
    Empty,
}

impl From<PyVexValueKind> for RustVexValueKind {
    fn from(kind: PyVexValueKind) -> Self {
        match kind {
            PyVexValueKind::Number => Self::Number,
            PyVexValueKind::NumberUnit => Self::NumberUnit,
            PyVexValueKind::Ident => Self::Ident,
            PyVexValueKind::Literal => Self::Literal,
            PyVexValueKind::Epoch => Self::Epoch,
            PyVexValueKind::RightAscension => Self::RightAscension,
            PyVexValueKind::Declination => Self::Declination,
            PyVexValueKind::Link => Self::Link,
            PyVexValueKind::Empty => Self::Empty,
        }
    }
}

impl From<RustVexValueKind> for PyVexValueKind {
    fn from(kind: RustVexValueKind) -> Self {
        match kind {
            RustVexValueKind::Number => Self::Number,
            RustVexValueKind::NumberUnit => Self::NumberUnit,
            RustVexValueKind::Ident => Self::Ident,
            RustVexValueKind::Literal => Self::Literal,
            RustVexValueKind::Epoch => Self::Epoch,
            RustVexValueKind::RightAscension => Self::RightAscension,
            RustVexValueKind::Declination => Self::Declination,
            RustVexValueKind::Link => Self::Link,
            RustVexValueKind::Empty => Self::Empty,
        }
    }
}

#[pyclass(name = "VexValue")]
#[derive(Clone)]
struct PyVexValue {
    inner: RustVexValue,
}

#[pymethods]
impl PyVexValue {
    #[new]
    #[pyo3(signature = (source, kind, unit=None))]
    fn new(source: String, kind: PyRef<'_, PyVexValueKind>, unit: Option<String>) -> Self {
        Self {
            inner: RustVexValue::new(source, unit, (*kind).into()),
        }
    }

    #[staticmethod]
    fn number(value: f64) -> Self {
        Self {
            inner: RustVexValue::new_float(value),
        }
    }

    #[staticmethod]
    fn integer(value: i64) -> Self {
        Self {
            inner: RustVexValue::new_integer(value),
        }
    }

    #[staticmethod]
    fn number_with_unit(value: f64, unit: &str) -> Self {
        Self {
            inner: RustVexValue::new_float_unit(value, unit),
        }
    }

    #[staticmethod]
    fn integer_with_unit(value: i64, unit: &str) -> Self {
        Self {
            inner: RustVexValue::new_integer_unit(value, unit),
        }
    }

    #[staticmethod]
    fn ident(source: &str) -> Self {
        Self {
            inner: RustVexValue::new_ident(source),
        }
    }

    #[staticmethod]
    fn literal(source: &str) -> Self {
        Self {
            inner: RustVexValue::new_literal(source),
        }
    }

    #[staticmethod]
    fn epoch(source: &str) -> Self {
        Self {
            inner: RustVexValue::new_epoch(source),
        }
    }

    #[staticmethod]
    fn right_ascension(source: &str) -> Self {
        Self {
            inner: RustVexValue::new_ra(source),
        }
    }

    #[staticmethod]
    fn declination(source: &str) -> Self {
        Self {
            inner: RustVexValue::new_dec(source),
        }
    }

    #[staticmethod]
    fn link(source: &str) -> Self {
        Self {
            inner: RustVexValue::new_link(source),
        }
    }

    #[staticmethod]
    fn empty() -> Self {
        Self {
            inner: RustVexValue::empty(),
        }
    }

    #[staticmethod]
    fn parse(source: &str) -> PyResult<Self> {
        RustVexValue::from_str(source)
            .map(|inner| Self { inner })
            .map_err(parse_error)
    }

    #[getter]
    fn source(&self) -> &str {
        self.inner.source()
    }

    #[setter]
    fn set_source(&mut self, source: &str) {
        self.inner.set_source(source);
    }

    #[getter]
    fn kind(&self) -> PyVexValueKind {
        self.inner.kind().into()
    }

    #[setter]
    fn set_kind(&mut self, kind: PyRef<'_, PyVexValueKind>) {
        self.inner.set_kind((*kind).into());
    }

    #[getter]
    fn unit(&self) -> Option<&str> {
        self.inner.unit()
    }

    #[setter]
    fn set_unit(&mut self, unit: Option<&str>) {
        match unit {
            Some(unit) => self.inner.set_unit(unit),
            None => self.inner.clear_unit(),
        }
    }

    fn clear_unit(&mut self) {
        self.inner.clear_unit();
    }

    fn to_vex(&self) -> String {
        self.inner.to_string()
    }

    fn __str__(&self) -> String {
        self.inner.to_string()
    }

    fn __repr__(&self) -> String {
        format!(
            "VexValue({:?}, {:?}, {:?})",
            self.inner.source(),
            self.inner.kind(),
            self.inner.unit()
        )
    }
}

#[pyclass(name = "VexVersion")]
#[derive(Clone)]
struct PyVexVersion {
    inner: RustVexVersion,
}

#[pymethods]
impl PyVexVersion {
    #[new]
    fn new(major: u8, minor: u8) -> Self {
        Self {
            inner: RustVexVersion::new(major, minor),
        }
    }

    #[getter]
    fn major(&self) -> u8 {
        self.inner.maj
    }

    #[getter]
    fn minor(&self) -> u8 {
        self.inner.min
    }

    fn __str__(&self) -> String {
        self.inner.to_string()
    }

    fn __repr__(&self) -> String {
        format!("VexVersion({}, {})", self.inner.maj, self.inner.min)
    }
}

#[pyclass(name = "Global")]
#[derive(Clone, Default)]
struct PyGlobal {
    inner: RustGlobal,
}

#[pymethods]
impl PyGlobal {
    #[new]
    fn new() -> Self {
        Self::default()
    }

    #[staticmethod]
    fn parse(source: &str) -> PyResult<Self> {
        RustGlobal::from_str(source)
            .map(|inner| Self { inner })
            .map_err(parse_error)
    }

    fn add(&mut self, key: &str, value: PyRef<'_, PyVexValue>) -> Option<PyVexValue> {
        self.inner
            .add(key, value.inner.clone())
            .map(|inner| PyVexValue { inner })
    }

    fn get(&self, key: &str) -> Option<PyVexValue> {
        self.inner
            .get(key)
            .cloned()
            .map(|inner| PyVexValue { inner })
    }

    fn remove(&mut self, key: &str) -> Option<PyVexValue> {
        self.inner.remove(key).map(|inner| PyVexValue { inner })
    }

    fn contains(&self, key: &str) -> bool {
        self.inner.contains_key(key)
    }

    fn keys(&self) -> Vec<String> {
        let mut keys: Vec<_> = self.inner.iter().map(|(key, _)| key.clone()).collect();
        keys.sort();
        keys
    }

    fn values(&self) -> Vec<PyVexValue> {
        self.inner
            .iter()
            .map(|(_, value)| PyVexValue {
                inner: value.clone(),
            })
            .collect()
    }

    fn to_vex(&self) -> String {
        self.inner.to_string()
    }

    fn __len__(&self) -> usize {
        self.inner.iter().count()
    }

    fn __contains__(&self, key: &str) -> bool {
        self.contains(key)
    }

    fn __getitem__(&self, key: &str) -> PyResult<PyVexValue> {
        self.get(key)
            .ok_or_else(|| PyKeyError::new_err(key.to_owned()))
    }

    fn __setitem__(&mut self, key: &str, value: PyRef<'_, PyVexValue>) {
        self.inner.add(key, value.inner.clone());
    }

    fn __str__(&self) -> String {
        self.to_vex()
    }
}

#[pyclass(name = "BlockDef")]
#[derive(Clone)]
struct PyBlockDef {
    inner: RustBlockDef,
}

#[pymethods]
impl PyBlockDef {
    #[new]
    fn new(name: &str, parent_kind: PyRef<'_, PyBlockKind>) -> Self {
        Self {
            inner: RustBlockDef::new(name, (*parent_kind).into()),
        }
    }

    #[staticmethod]
    fn highlevel(name: &str) -> Self {
        Self {
            inner: RustBlockDef::new_highlevel(name),
        }
    }

    #[staticmethod]
    fn primitive(name: &str) -> Self {
        Self {
            inner: RustBlockDef::new_primitive(name),
        }
    }

    #[staticmethod]
    fn scan(name: &str) -> Self {
        Self {
            inner: RustBlockDef::new_scan(name),
        }
    }

    #[staticmethod]
    fn parse(source: &str) -> PyResult<Self> {
        RustBlockDef::from_str(source)
            .map(|inner| Self { inner })
            .map_err(parse_error)
    }

    #[getter]
    fn name(&self) -> &str {
        self.inner.name()
    }

    #[getter]
    fn parent_kind(&self) -> PyBlockKind {
        self.inner.parentkind().into()
    }

    #[setter]
    fn set_parent_kind(&mut self, kind: PyRef<'_, PyBlockKind>) {
        self.inner.set_parentkind((*kind).into());
    }

    fn add(&mut self, py: Python<'_>, name: &str, values: Vec<Py<PyVexValue>>) {
        self.inner.add(name, clone_values(py, values));
    }

    fn get(&self, py: Python<'_>, name: &str) -> PyResult<Option<Vec<Vec<Py<PyVexValue>>>>> {
        self.inner
            .get(name)
            .map(|occurrences| {
                occurrences
                    .iter()
                    .map(|values| wrap_values(py, values.clone()))
                    .collect()
            })
            .transpose()
    }

    fn get_first(&self, py: Python<'_>, name: &str) -> PyResult<Option<Vec<Py<PyVexValue>>>> {
        self.inner
            .get_first(name)
            .map(|values| wrap_values(py, values.clone()))
            .transpose()
    }

    fn remove(&mut self, py: Python<'_>, name: &str) -> PyResult<Option<Vec<Vec<Py<PyVexValue>>>>> {
        self.inner
            .remove(name)
            .map(|occurrences| {
                occurrences
                    .into_iter()
                    .map(|values| wrap_values(py, values))
                    .collect()
            })
            .transpose()
    }

    fn remove_single(
        &mut self,
        py: Python<'_>,
        name: &str,
        index: usize,
    ) -> PyResult<Option<Vec<Py<PyVexValue>>>> {
        self.inner
            .remove_single(name, index)
            .map(|values| wrap_values(py, values))
            .transpose()
    }

    fn contains(&self, name: &str) -> bool {
        self.inner.contains_key(name)
    }

    fn keys(&self) -> Vec<String> {
        let mut keys: Vec<_> = self.inner.iter().map(|(key, _)| key.clone()).collect();
        keys.sort();
        keys
    }

    fn to_vex(&self) -> String {
        self.inner.to_string()
    }

    fn __len__(&self) -> usize {
        self.inner.iter().count()
    }

    fn __contains__(&self, name: &str) -> bool {
        self.contains(name)
    }

    fn __getitem__(&self, py: Python<'_>, name: &str) -> PyResult<Vec<Vec<Py<PyVexValue>>>> {
        self.get(py, name)?
            .ok_or_else(|| PyKeyError::new_err(name.to_owned()))
    }

    fn __setitem__(&mut self, py: Python<'_>, name: &str, occurrences: Vec<Vec<Py<PyVexValue>>>) {
        self.inner.remove(name);
        for occurrence in occurrences {
            self.inner.add(name, clone_values(py, occurrence));
        }
    }

    fn __str__(&self) -> String {
        self.to_vex()
    }
}

#[pyclass(name = "Block")]
#[derive(Clone)]
struct PyBlock {
    inner: RustBlock,
}

#[pymethods]
impl PyBlock {
    #[new]
    fn new(name: &str, kind: PyRef<'_, PyBlockKind>) -> PyResult<Self> {
        let inner = match *kind {
            PyBlockKind::HighLevel => RustBlock::new_highlevel(name),
            PyBlockKind::Primitive => RustBlock::new_primitive(name),
            PyBlockKind::Schedule if name.eq_ignore_ascii_case("SCHED") => {
                RustBlock::new_schedule()
            }
            PyBlockKind::Schedule => {
                return Err(PyValueError::new_err(
                    "schedule blocks must use the name 'SCHED'",
                ));
            }
        };
        Ok(Self { inner })
    }

    #[staticmethod]
    fn highlevel(name: &str) -> Self {
        Self {
            inner: RustBlock::new_highlevel(name),
        }
    }

    #[staticmethod]
    fn primitive(name: &str) -> Self {
        Self {
            inner: RustBlock::new_primitive(name),
        }
    }

    #[staticmethod]
    fn schedule() -> Self {
        Self {
            inner: RustBlock::new_schedule(),
        }
    }

    #[staticmethod]
    fn parse(source: &str) -> PyResult<Self> {
        RustBlock::from_str(source)
            .map(|inner| Self { inner })
            .map_err(parse_error)
    }

    #[getter]
    fn name(&self) -> &str {
        self.inner.name()
    }

    #[getter]
    fn kind(&self) -> PyBlockKind {
        self.inner.kind().into()
    }

    fn add(&mut self, definition: PyRef<'_, PyBlockDef>) -> Option<PyBlockDef> {
        self.inner
            .add(definition.inner.clone())
            .map(|inner| PyBlockDef { inner })
    }

    fn get(&self, name: &str) -> Option<PyBlockDef> {
        self.inner
            .get(name)
            .cloned()
            .map(|inner| PyBlockDef { inner })
    }

    fn remove(&mut self, name: &str) -> Option<PyBlockDef> {
        self.inner.remove(name).map(|inner| PyBlockDef { inner })
    }

    fn contains(&self, name: &str) -> bool {
        self.inner.contains_key(name)
    }

    fn keys(&self) -> Vec<String> {
        let mut keys: Vec<_> = self.inner.iter().map(|(key, _)| key.clone()).collect();
        keys.sort();
        keys
    }

    fn definitions(&self) -> Vec<PyBlockDef> {
        self.keys()
            .into_iter()
            .filter_map(|name| self.get(&name))
            .collect()
    }

    fn to_vex(&self) -> String {
        self.inner.to_string()
    }

    fn __len__(&self) -> usize {
        self.inner.iter().count()
    }

    fn __contains__(&self, name: &str) -> bool {
        self.contains(name)
    }

    fn __getitem__(&self, name: &str) -> PyResult<PyBlockDef> {
        self.get(name)
            .ok_or_else(|| PyKeyError::new_err(name.to_owned()))
    }

    fn __setitem__(&mut self, name: &str, definition: PyRef<'_, PyBlockDef>) {
        let mut replacement = definition.inner.clone();
        replacement.set_name(name);
        self.inner.add(replacement);
    }

    fn __str__(&self) -> String {
        self.to_vex()
    }
}

#[pyclass(name = "Vex")]
#[derive(Clone)]
struct PyVex {
    inner: RustVex,
}

#[pymethods]
impl PyVex {
    #[new]
    fn new() -> Self {
        Self {
            inner: RustVex::new(),
        }
    }

    #[staticmethod]
    fn parse(source: &str) -> PyResult<Self> {
        RustVex::from_str(source)
            .map(|inner| Self { inner })
            .map_err(parse_error)
    }

    #[staticmethod]
    fn open(path: &str) -> PyResult<Self> {
        RustVex::open(path)
            .map(|inner| Self { inner })
            .map_err(vex_error)
    }

    fn save(&self, path: &str) -> PyResult<()> {
        std::fs::write(path, self.inner.to_string())
            .map_err(|error| PyOSError::new_err(error.to_string()))
    }

    fn to_vex(&self) -> String {
        self.inner.to_string()
    }

    #[getter]
    fn version(&self, py: Python<'_>) -> PyResult<Py<PyVexVersion>> {
        Py::new(
            py,
            PyVexVersion {
                inner: self.inner.version(),
            },
        )
    }

    #[setter(version)]
    fn set_version(&mut self, version: PyRef<'_, PyVexVersion>) {
        self.inner.set_version(
            usize::from(version.inner.maj),
            usize::from(version.inner.min),
        );
    }

    #[getter]
    fn global_block(&self, py: Python<'_>) -> PyResult<Py<PyGlobal>> {
        Py::new(
            py,
            PyGlobal {
                inner: self.inner.global().clone(),
            },
        )
    }

    #[setter(global_block)]
    fn set_global_block(&mut self, global: PyRef<'_, PyGlobal>) {
        *self.inner.global_mut() = global.inner.clone();
    }

    fn get_block(&self, py: Python<'_>, name: &str) -> PyResult<Option<Py<PyBlock>>> {
        let name = name.to_ascii_uppercase();
        self.inner
            .get(&name)
            .cloned()
            .map(|inner| Py::new(py, PyBlock { inner }))
            .transpose()
    }

    fn add_block(
        &mut self,
        py: Python<'_>,
        block: PyRef<'_, PyBlock>,
    ) -> PyResult<Option<Py<PyBlock>>> {
        self.inner
            .add(block.inner.clone())
            .map(|inner| Py::new(py, PyBlock { inner }))
            .transpose()
    }

    fn remove_block(&mut self, py: Python<'_>, name: &str) -> PyResult<Option<Py<PyBlock>>> {
        let name = name.to_ascii_uppercase();
        self.inner
            .remove(&name)
            .map(|inner| Py::new(py, PyBlock { inner }))
            .transpose()
    }

    fn block_names(&self) -> Vec<String> {
        let mut names: Vec<_> = self.inner.iter().map(|(name, _)| name.clone()).collect();
        names.sort();
        names
    }

    fn blocks(&self, py: Python<'_>) -> PyResult<Vec<Py<PyBlock>>> {
        self.block_names()
            .into_iter()
            .filter_map(|name| self.inner.get(&name).cloned())
            .map(|inner| Py::new(py, PyBlock { inner }))
            .collect()
    }

    fn __len__(&self) -> usize {
        self.inner.iter().count()
    }

    fn __contains__(&self, name: &str) -> bool {
        self.inner.contains_key(name.to_ascii_uppercase())
    }

    fn __getitem__(&self, py: Python<'_>, name: &str) -> PyResult<Py<PyBlock>> {
        self.get_block(py, name)?
            .ok_or_else(|| PyKeyError::new_err(name.to_owned()))
    }

    fn __setitem__(&mut self, name: &str, block: PyRef<'_, PyBlock>) -> PyResult<()> {
        if !name.eq_ignore_ascii_case(block.inner.name()) {
            return Err(PyValueError::new_err(format!(
                "mapping key {name:?} does not match block name {:?}",
                block.inner.name()
            )));
        }
        self.inner.add(block.inner.clone());
        Ok(())
    }

    fn __str__(&self) -> String {
        self.to_vex()
    }
}

#[pymodule]
fn _vexio(module: &Bound<'_, PyModule>) -> PyResult<()> {
    module.add_class::<PyBlockKind>()?;
    module.add_class::<PyVexValueKind>()?;
    module.add_class::<PyVexValue>()?;
    module.add_class::<PyVexVersion>()?;
    module.add_class::<PyGlobal>()?;
    module.add_class::<PyBlockDef>()?;
    module.add_class::<PyBlock>()?;
    module.add_class::<PyVex>()?;
    Ok(())
}
