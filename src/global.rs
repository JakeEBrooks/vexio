use crate::VexRef;
use crate::error::ParseError;
use crate::parsing::Parser;

/// Represents the unique `GLOBAL` block within a Vex file.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize), derive(serde::Deserialize))]
pub struct Global {
    refs: Vec<VexRef>,
}

impl Global {
    pub fn new() -> Self {
        Self { refs: Vec::new() }
    }

    pub fn add_ref(&mut self, refr: VexRef) {
        self.refs.push(refr);
    }

    pub fn as_slice(&self) -> &[VexRef] {
        self.refs.as_slice()
    }

    pub fn as_mut_slice(&mut self) -> &mut [VexRef] {
        self.refs.as_mut_slice()
    }

    pub fn to_vex(&self) -> String {
        format!("$GLOBAL;\n{}\n", self.refs.iter().map(|r| format!("\t{}", r.to_vex())).collect::<Vec<String>>().join("\n"))
    }

    pub fn to_vex_minified(&self) -> String {
        format!("$GLOBAL;{}", self.refs.iter().map(|r| r.to_vex_minified()).collect::<Vec<String>>().join(""))
    }

    pub fn parse(vex: &str) -> Result<Self, ParseError> {
        Parser::new(vex).parse_global()
    }
}

impl<I: Into<Vec<VexRef>>> From<I> for Global {
    fn from(value: I) -> Self {
        Self { refs: value.into() }
    }
}

#[cfg(test)]
mod tests {
    use crate::refr;

    use super::*;

    #[test]
    fn test_parse_tofrom_global() -> miette::Result<()> {
        let start = Global::from([refr!("MY_BLOCK" = "some_key" : "qual1"), refr!("MY_BLOCK2" = "some_key2")]);
        assert_eq!(start, Global::parse(start.to_vex().as_str())?);
        Ok(())
    }
}
