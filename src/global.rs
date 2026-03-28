use crate::{ToFromVex, VexError, VexRef};

/// Represents the unique `GLOBAL` block within a Vex file.
pub type Global = Vec<VexRef>;

impl ToFromVex for Global {
    fn to_vex(&self) -> String {
        format!("$GLOBAL;\n{}\n", self.iter().map(|r| format!("\t{}", r.to_vex())).collect::<Vec<String>>().join("\n"))
    }

    fn to_vex_minified(&self) -> String {
        format!("$GLOBAL;{}", self.iter().map(|r| r.to_vex_minified()).collect::<Vec<String>>().join(""))
    }

    fn from_vex(vex: &str) -> Result<Self, VexError> {
        crate::parsing::Parser::new(vex).parse_global()
    }
}

#[cfg(test)]
mod tests {
    use crate::refr;

    use super::*;

    #[test]
    fn test_parse_tofrom_global() -> miette::Result<()> {
        let start: Global = [refr!("MY_BLOCK" = "some_key" : "qual1"), refr!("MY_BLOCK2" = "some_key2")].into();
        assert_eq!(start, Global::from_vex(start.to_vex().as_str())?);
        Ok(())
    }
}
