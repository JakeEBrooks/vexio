use vexio::{Vex, VexError};

#[test]
fn test_vex_parse_files() -> Result<(), VexError> {
    let testfiles = std::fs::read_dir("tests/examples").unwrap();
    for f in testfiles {
        let _vex = Vex::open(f.unwrap().path())?;
    }
    Ok(())
}

#[test]
fn test_minified_vex() -> Result<(), VexError> {
    let testfiles = std::fs::read_dir("tests/examples").unwrap();
    for f in testfiles {
        let vex = Vex::open(f.unwrap().path())?;
        let vex_str = vex.to_vex();
        let vex_min_str = vex.to_vex_minified();

        let vex_str_parsed = Vex::parse(&vex_str)?;
        let vex_min_str_parsed = Vex::parse(&vex_min_str)?;
        assert_eq!(vex_str_parsed, vex_min_str_parsed)
    }
    Ok(())
}
