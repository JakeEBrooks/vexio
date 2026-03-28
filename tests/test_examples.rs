use vexio::{ToFromVex, Vex};

#[test]
fn test_vex_parse_files() -> miette::Result<()> {
    let testfiles = std::fs::read_dir("tests/examples").unwrap();
    for f in testfiles {
        let _vex = Vex::open(f.unwrap().path())?;
    }
    Ok(())
}

#[test]
fn test_minified_vex() -> miette::Result<()> {
    let testfiles = std::fs::read_dir("tests/examples").unwrap();
    for f in testfiles {
        let vex = Vex::open(f.unwrap().path())?;
        let vex_str = vex.to_vex();
        let vex_min_str = vex.to_vex_minified();

        let vex_str_parsed = Vex::from_vex(&vex_str)?;
        let vex_min_str_parsed = Vex::from_vex(&vex_min_str)?;
        assert_eq!(vex_str_parsed, vex_min_str_parsed)
    }
    Ok(())
}

#[test]
fn test_json_serde() -> miette::Result<()> {
    let testfiles = std::fs::read_dir("tests/examples").unwrap();
    for f in testfiles {
        let vex = Vex::open(f.unwrap().path())?;
        let json_str = serde_json::to_string_pretty(&vex).unwrap();
        let vex_parsed: Vex = serde_json::from_str(&json_str).unwrap();
        assert_eq!(vex, vex_parsed)
    }
    Ok(())
}
