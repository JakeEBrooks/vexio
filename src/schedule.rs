use crate::Param;
use crate::vex::ToFromVex;

/// Represents the unique `SCHED` block within a Vex file.
pub type Schedule = Vec<Scan>;

impl ToFromVex for Schedule {
    fn to_vex(&self) -> String {
        format!("$SCHED;\n{}", self.iter().map(|s| s.to_vex()).collect::<Vec<String>>().join("\n"))
    }

    fn to_vex_minified(&self) -> String {
        format!("$SCHED;{}", self.iter().map(|s| s.to_vex_minified()).collect::<Vec<String>>().join(""))
    }

    fn from_vex(vex: &str) -> Result<Self, crate::VexError> {
        crate::parsing::Parser::new(vex).parse_schedule()
    }
}

/// A single scan within a [`Schedule`]
#[derive(Debug, Default, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize), derive(serde::Deserialize))]
pub struct Scan {
    /// The name of this [`Scan`]
    pub name: String,
    /// The [`Param`]s associated with this [`Scan`]
    pub params: Vec<Param>,
}

impl Scan {
    /// Construct a new [`Scan`] from a series of [`Param`]s
    pub fn new<S: AsRef<str>, P: Into<Vec<Param>>>(name: S, params: P) -> Self {
        Self {
            name: name.as_ref().to_string(),
            params: params.into(),
        }
    }
}

impl ToFromVex for Scan {
    fn to_vex(&self) -> String {
        format!(
            "scan {};\n{}\nendscan;\n",
            self.name,
            self.params.iter().map(|s| format!("\t{}", s.to_vex())).collect::<Vec<String>>().join("\n")
        )
    }

    fn to_vex_minified(&self) -> String {
        format!(
            "scan {};{}endscan;",
            self.name,
            self.params.iter().map(|s| s.to_vex_minified()).collect::<Vec<String>>().join("")
        )
    }

    fn from_vex(vex: &str) -> Result<Self, crate::VexError> {
        crate::parsing::Parser::new(vex).parse_scan()
    }
}

/// Convenience macro for constructing a [`Scan`].
///
/// The required syntax is the same as the [`crate::def`] macro
#[macro_export]
macro_rules! scan {
    ($name:literal; $($param_name:literal = $($param:literal):*;)+) => {
        $crate::Scan::new(
            format!($name),
            [$( $crate::Param::new(format!($param_name), [$( $crate::ParamValue::new(format!($param)) ),*]) ),*]
        )
    };
}
