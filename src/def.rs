use std::slice::{Iter, IterMut};

use crate::error::ParseError;
use crate::parsing::Parser;
use crate::{Param, VexRef};

/// A `def` block within a high level Vex block
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize), derive(serde::Deserialize))]
pub struct HighLevelDef {
    /// The name of this [`HighLevelDef`]
    name: String,
    /// The [`VexRef`]s within this high level def block
    refs: Vec<VexRef>,
}

impl HighLevelDef {
    /// Construct a new [`HighLevelDef`] from a `name` and a series of [`VexRef`]s.
    pub fn new<S: AsRef<str>, I: Into<Vec<VexRef>>>(name: S, refs: I) -> Self {
        Self {
            name: name.as_ref().to_string(),
            refs: refs.into(),
        }
    }

    /// Construct a new [`HighLevelDef`] with no [`VexRef`]s from a `name`
    pub fn new_empty(name: impl AsRef<str>) -> Self {
        Self::new(name, [])
    }

    /// Get the name of this def block
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Add a [`VexRef`] to this [`HighLevelDef`]
    pub fn add_ref(&mut self, refr: VexRef) {
        self.refs.push(refr);
    }

    /// Retrieve references to all [`VexRef`]s with block names matching matching `blk`.
    ///
    /// For example:
    /// ```rust
    /// use vexio::{refr, HighLevelDef};
    ///
    /// let hldef = HighLevelDef::new("my_def", [refr!("BLK1" = "my_key"), refr!("BLK1" = "my_key2"), refr!("BLK2" = "key2" : "qual1")]);
    /// assert_eq!(hldef.get("BLK1"), vec![&refr!("BLK1" = "my_key"), &refr!("BLK1" = "my_key2")])
    /// ```
    pub fn get(&self, blk: impl AsRef<str>) -> Vec<&VexRef> {
        self.refs.iter().filter(|r| r.blockname() == blk.as_ref()).collect()
    }

    /// Retrieve mutable references to all [`VexRef`]s with block names matching matching `blk`.
    ///
    /// For example:
    /// ```rust
    /// use vexio::{refr, HighLevelDef};
    ///
    /// let mut hldef = HighLevelDef::new("my_def", [refr!("BLK1" = "my_key"), refr!("BLK1" = "my_key2"), refr!("BLK2" = "key2" : "qual1")]);
    /// assert_eq!(hldef.get_mut("BLK1"), vec![&mut refr!("BLK1" = "my_key"), &mut refr!("BLK1" = "my_key2")])
    /// ```
    pub fn get_mut(&mut self, blk: impl AsRef<str>) -> Vec<&mut VexRef> {
        self.refs.iter_mut().filter(|r| r.blockname() == blk.as_ref()).collect()
    }

    /// Return an iterator over all [`VexRef`]s in this [`HighLevelDef`].
    pub fn refs(&self) -> Iter<'_, VexRef> {
        self.refs.iter()
    }

    /// Return an iterator of mutable references over all [`VexRef`]s in this [`HighLevelDef`].
    pub fn refs_mut(&mut self) -> IterMut<'_, VexRef> {
        self.refs.iter_mut()
    }

    /// Get a representation of this high level def as a Vex formatted string
    pub fn to_vex(&self) -> String {
        format!(
            "def {};\n{}\nenddef;\n",
            self.name,
            self.refs.iter().map(|r| format!("\t{}", r.to_vex())).collect::<Vec<String>>().join("\n")
        )
    }

    /// Get a representation of this block as a Vex formatted string, with unnecessary whitespace removed
    pub fn to_vex_minified(&self) -> String {
        format!(
            "def {};{}enddef;",
            self.name,
            self.refs.iter().map(|r| r.to_vex_minified()).collect::<Vec<String>>().join("")
        )
    }

    /// Parse a [`HighLevelDef`] from a string slice
    pub fn parse(vex: &str) -> Result<Self, ParseError> {
        Parser::new(vex).parse_highlevel_def()
    }

    /// View a slice of [`VexRef`]s
    pub fn as_slice(&self) -> &[VexRef] {
        self.refs.as_slice()
    }

    /// View a mutable slice of [`VexRef`]s
    pub fn as_mut_slice(&mut self) -> &mut [VexRef] {
        self.refs.as_mut_slice()
    }
}

/// A `def` block within a primitive Vex block
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize), derive(serde::Deserialize))]
pub struct Def {
    /// The name of this [`Def`] block
    name: String,
    /// The [`Param`]s within this def block
    params: Vec<Param>,
}

impl Def {
    /// Construct a new [`Def`] block from a `name` and a series of [`Param`]s.
    pub fn new<S: AsRef<str>, I: Into<Vec<Param>>>(name: S, params: I) -> Self {
        Self {
            name: name.as_ref().to_string(),
            params: params.into(),
        }
    }

    /// Construct a new [`Def`] with no [`Param`]s from a `name`
    pub fn new_empty(name: impl AsRef<str>) -> Self {
        Self::new(name, [])
    }

    /// Get the name of this def block
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Retrieve references to all [`Param`]s with parameter names matching matching `name`.
    ///
    /// For example:
    /// ```rust
    /// use vexio::{def, Param, ParamValue};
    ///
    /// let mydef = def!("my_def_block";
    ///     "param1" = "value" : "value_1";
    ///     "param1" = "other_value" : "value_2";
    ///     "param2" = "&my_link" : "64 TB";
    /// );
    /// assert_eq!(mydef.get("param1"), vec![
    ///     &Param::new("param1", [ParamValue::new("value"), ParamValue::new("value_1")]),
    ///     &Param::new("param1", [ParamValue::new("other_value"), ParamValue::new("value_2")]),
    /// ])
    /// ```
    pub fn get(&self, name: impl AsRef<str>) -> Vec<&Param> {
        self.params.iter().filter(|p| p.name() == name.as_ref()).collect()
    }

    /// Retrieve mutable references to all [`Param`]s with parameter names matching matching `name`.
    ///
    /// For example:
    /// ```rust
    /// use vexio::{def, Param, ParamValue};
    ///
    /// let mut mydef = def!("my_def_block";
    ///     "param1" = "value" : "value_1";
    ///     "param1" = "other_value" : "value_2";
    ///     "param2" = "&my_link" : "64 TB";
    /// );
    /// assert_eq!(mydef.get_mut("param1"), vec![
    ///     &mut Param::new("param1", [ParamValue::new("value"), ParamValue::new("value_1")]),
    ///     &mut Param::new("param1", [ParamValue::new("other_value"), ParamValue::new("value_2")]),
    /// ])
    /// ```
    pub fn get_mut(&mut self, name: impl AsRef<str>) -> Vec<&mut Param> {
        self.params.iter_mut().filter(|p| p.name() == name.as_ref()).collect()
    }

    /// Return an iterator over the [`Param`]s in this def
    pub fn params(&self) -> Iter<'_, Param> {
        self.params.iter()
    }

    /// Return a mutable iterator over the [`Param`]s in this def
    pub fn params_mut(&mut self) -> IterMut<'_, Param> {
        self.params.iter_mut()
    }

    /// Return a Vex formatted string representing this structure
    pub fn to_vex(&self) -> String {
        format!(
            "def {};\n{}\nenddef;\n",
            self.name,
            self.params.iter().map(|p| format!("\t{}", p.to_vex())).collect::<Vec<String>>().join("\n")
        )
    }

    /// Return a Vex formatted string representing this structure, with unnecessary whitespace removed
    pub fn to_vex_minified(&self) -> String {
        format!(
            "def {};{}enddef;",
            self.name,
            self.params.iter().map(|p| p.to_vex_minified()).collect::<Vec<String>>().join("")
        )
    }

    /// Attempt to parse a [`Def`] from a Vex formatted string
    pub fn parse(vex: &str) -> Result<Self, ParseError> {
        Parser::new(vex).parse_primitive_def()
    }

    /// Return a slice of parameters in this def
    pub fn as_slice(&self) -> &[Param] {
        self.params.as_slice()
    }

    /// Return a mutable slice of parameters in this def
    pub fn as_mut_slice(&mut self) -> &mut [Param] {
        self.params.as_mut_slice()
    }
}

/// Convenience macro for constructing a [`Def`].
///
/// The required syntax is somewhat similar to the Vex syntax
///
/// # Examples
/// ```rust
/// use vexio::def;
///
/// let def = def!("my_def_block";
///     "param1" = "value" : "value_2";
///     "param2" = "&my_link" : "64 TB";
/// );
/// ```
///
/// You can also use variables from the local scope:
/// ```rust
/// use vexio::def;
///
/// let my_param = "some_param_name";
/// let def = def!("my_def_block";
///     "{my_param}" = "value" : "value_2";
///     "param2" = "&my_link" : "64 TB";
/// );
/// ```
#[macro_export]
macro_rules! def {
    ($name:literal; $($param_name:literal = $($param:literal):*;)+) => {
        $crate::Def::new(
            format!($name),
            [$( $crate::Param::new(format!($param_name), [$( $crate::ParamValue::new(format!($param)) ),*]) ),*]
        )
    };
    ($name:literal) => {
        $crate::Def::new_empty(format!($name))
    };
}

#[cfg(test)]
mod tests {
    use crate::error::VexError;
    use crate::param::{ParamValue, ParamValueKind};
    use crate::refr;

    use super::*;

    #[test]
    fn test_def_macro() -> Result<(), VexError> {
        let def = def!("some_name";
            "param1" = "val" : "val2";
        );

        assert_eq!(
            def,
            Def::new(
                "some_name",
                [Param::new(
                    "param1",
                    [
                        ParamValue::new_unchecked("val", ParamValueKind::Ident),
                        ParamValue::new_unchecked("val2", ParamValueKind::Ident)
                    ]
                )]
            )
        );
        Ok(())
    }

    #[test]
    fn test_def_macro_2() -> Result<(), VexError> {
        let param_name = "param_subst";
        let def = def!("some_name";
            "{param_name}" = "val" : "val2";
        );
        assert_eq!(
            def,
            Def::new(
                "some_name",
                [Param::new(
                    "param_subst",
                    [
                        ParamValue::new_unchecked("val", ParamValueKind::Ident),
                        ParamValue::new_unchecked("val2", ParamValueKind::Ident)
                    ]
                )]
            )
        );
        Ok(())
    }

    #[test]
    fn test_def_macro_3() -> Result<(), VexError> {
        let param_name = "my_parameter".to_string();
        let num = 64;
        let def = def!("my_def_block";
            "{param_name}" = "value" : "{num}";
            "param2" = "&my_link" : "64 TB";
        );
        assert_eq!(
            def,
            Def::new(
                "my_def_block",
                [
                    Param::new(
                        "my_parameter",
                        [
                            ParamValue::new_unchecked("value", ParamValueKind::Ident),
                            ParamValue::new_unchecked("64", ParamValueKind::Number)
                        ]
                    ),
                    Param::new(
                        "param2",
                        [
                            ParamValue::new_unchecked("my_link", ParamValueKind::Link),
                            ParamValue::new_unchecked("64 TB", ParamValueKind::NumberUnit)
                        ]
                    )
                ],
            )
        );
        Ok(())
    }

    #[test]
    fn test_parse_tofrom_hldef() -> Result<(), VexError> {
        let start = HighLevelDef::new("my_def", [refr!("BLK1" = "my_key"), refr!("BLK2" = "key2" : "qual1")]);
        assert_eq!(start, HighLevelDef::parse(start.to_vex().as_str())?);
        Ok(())
    }

    #[test]
    fn test_parse_tofrom_def() -> Result<(), VexError> {
        let start = def!("my_def"; "param1" = "val1"; "param2" = "val2" : "val3";);
        assert_eq!(start, Def::parse(start.to_vex().as_str())?);
        Ok(())
    }
}
