//! This example shows how to construct a simple [`Vex`] file manually, using some of the examples from the Vex specification

use vexio::error::VexError;
use vexio::{Block, Global, HighLevelDef, Schedule, Vex, def, refr, scan};

fn main() -> Result<(), VexError> {
    let global: Global = Global::from(vec![
        refr!("EXPER" = "EXP1387"),
        refr!("SCHEDULING_PARAMS" = "SKED1"),
        refr!("PROCEDURES" = "STANDARD1"),
        refr!("EOP" = "EOP129"),
    ]);

    let mode = [HighLevelDef::new(
        "SX",
        [
            refr!("FREQ" = "S4:HS"),
            refr!("BBC" = "HS/S4:HS"),
            refr!("IF" = "CDP/SX_WIDE:HS"),
            refr!("SEFD" = "HS:HS"),
            refr!("TRACKS" = "MARK5A/XX-4-2/8:HS"),
            refr!("TRACKS" = "MARK4_TRK_FORMAT:HS"),
            refr!("PROCEDURES" = "STANDARD_CAL:HS"),
            refr!("DAS" = "EARLY_START_20:HS"),
        ],
    )];

    let station = [
        HighLevelDef::new(
            "JB",
            [
                refr!("SITE" = "JODRELL_MK2"),
                refr!("ANTENNA" = "JODRELL_MK2"),
                refr!("CLOCK" = "JB"),
                refr!("DAS" = "MARK4"),
                refr!("DAS" = "56KBPI"),
                refr!("DAS" = "17640FT"),
                refr!("DAS" = "JB_MARK4_ID"),
            ],
        ),
        HighLevelDef::new(
            "HS",
            [
                refr!("SITE" = "HAYSTACK"),
                refr!("ANTENNA" = "HAYSTACK"),
                refr!("CLOCK" = "HS"),
                refr!("DAS" = "MARK5A"),
                refr!("DAS" = "HS_MARK5A_ID"),
                refr!("PHASE_CAL_DETECT" = "STANDARD"),
            ],
        ),
    ];

    let antenna = Block::new(
        "ANTENNA",
        [
            def!("EFLSBERG";
                "antenna_diam" = "100 m";
                "axis_type" = "az" : "el";
                "axis_offset" = "0 m";
            ),
            def!("VLBA";
                "antenna_diam" = "25 m";
                "axis_type" = "az" : "el";
                "axis_offset" = "0.123 m";
            ),
        ],
    );

    let sched: Schedule = [
        scan!("scan001";
            "start" = "1995y263d06h15m00s";
            "mode" = "SX";
            "source" = "HD123456";
            "station" = "EF" : "0 sec" : "180 sec" : "0 ft" : "" : "&n" : "1";
            "station" = "FD" : "0 sec" : "180 sec" : "0 ft" : "" : "&n" : "1";
        ),
        scan!("scan002";
            "start" = "1995y263d06h20m00s";
            "mode" = "SX";
            "source" = "HD123456";
            "station" = "EF" : "20 sec" : "120 sec" : "" : "" : "&n" : "1";
            "station" = "FD" : "0 sec" : "150 sec" : "0 ft" : "" : "" : "1";
        ),
    ]
    .into();

    let vex = Vex::new()
        .with_global(global)
        .with_station(station)
        .with_mode(mode)
        .with_block(antenna)
        .with_schedule(sched)
        .to_vex();

    print!("{}", vex);

    let _parsed_vex = Vex::parse(&vex)?;
    Ok(())
}
