use std::fs;

use skin_rules::{AspectProtection, ConversionOptions, TargetAspect, generate};

use crate::support::make_temp_root;

#[test]
fn bom_prefixed_source_still_protects_following_bomb() {
    let csv = convert(
        "bom-bomb",
        concat!(
            "\u{feff}#SRC_IMAGE,0,3,0,0,3600,240,18,0,300,50,0,0,0\r\n",
            "#DST_IMAGE,0,0,100,20,200,40,0,255,255,255,255,1,0,0,0,0,50,0,0,0\r\n",
        ),
        all_protected_options(),
    );

    assert!(csv.contains("#DST_IMAGE,0,0,375,45,450,90,"));
}

#[test]
fn nonstandard_destinations_only_transform_owned_geometry_fields() {
    let csv = convert(
        "command-ownership",
        concat!(
            "#DST_PM_CHARA_1P,100,20,200,40,1,7,folder\r\n",
            "#DST_NOTE2,100\r\n",
            "#DST_NOTE_EXPANSION_RATE,120,130\r\n",
            "#SRC_NOTECHART_1P,0,0,0,0,0,0,0,0,0,0,200,40,0,0,0,0,0,0\r\n",
            "#DST_NOTECHART_1P,0,0,100,20,999,888,0,255,255,255,255,1,0,0,0,0,0,0,0,0\r\n",
            "#SRC_GAUGECHART_1P,0,0,0,0,0,0,0,0,0,0,200,40,0,0\r\n",
            "#DST_GAUGECHART_1P,0,0,100,20,999,888,0,255,255,255,255,1,0,0,0,0,0,0,0,0\r\n",
            "#SRC_TIMINGCHART_1P,0,0,0,200,40,1,ffffff,ffffff,ffffff\r\n",
            "#DST_TIMINGCHART_1P,0,0,100,20,999,888,0,255,255,255,255,1,0,0,0,0,0,0,0,0\r\n",
            "#DST_FUTURE_COMMAND,0,0,100,20,200,40,0,255,255,255,255\r\n",
        ),
        ConversionOptions {
            resolution: Some(2),
            target_aspect: Some(TargetAspect::SixteenNine),
            ..ConversionOptions::default()
        },
    );

    assert!(csv.contains("#DST_PM_CHARA_1P,300,45,600,90,1,7,folder"));
    assert!(csv.contains("#DST_NOTE2,225"));
    assert!(csv.contains("#DST_NOTE_EXPANSION_RATE,120,130"));
    assert!(csv.contains("#SRC_NOTECHART_1P,0,0,0,0,0,0,0,0,0,0,600,90,"));
    assert!(csv.contains("#DST_NOTECHART_1P,0,0,300,45,999,888,"));
    assert!(csv.contains("#SRC_GAUGECHART_1P,0,0,0,0,0,0,0,0,0,0,600,90,"));
    assert!(csv.contains("#DST_GAUGECHART_1P,0,0,300,45,999,888,"));
    assert!(csv.contains("#SRC_TIMINGCHART_1P,0,0,0,600,90,"));
    assert!(csv.contains("#DST_TIMINGCHART_1P,0,0,300,45,999,888,"));
    assert!(csv.contains("#DST_FUTURE_COMMAND,0,0,100,20,200,40,"));
}

#[test]
fn resolution_zero_sixteen_nine_keeps_select_bar_coordinates() {
    let csv = convert(
        "select-bar-relative",
        concat!(
            "#DST_BAR_BODY_OFF,0,0,100,20,200,40,0,255,255,255,255,1,0,0,0,0,0,0,0,0\r\n",
            "#DST_BAR_MY_LAMP,0,0,100,20,200,40,0,255,255,255,255,1,0,0,0,0,0,0,0,0\r\n",
            "#DST_BAR_TROPHY,0,0,100,20,200,40,0,255,255,255,255,1,0,0,0,0,0,0,0,0\r\n",
            "#DST_BAR_LABEL,0,0,100,20,200,40,0,255,255,255,255,1,0,0,0,0,0,0,0,0\r\n",
            "#DST_BAR_GRAPH,0,0,100,20,200,40,0,255,255,255,255,1,0,0,0,0,0,0,0,0\r\n",
            "#DST_BAR_FLASH,0,0,100,20,200,40,0,255,255,255,255,1,0,0,0,0,0,0,0,0\r\n",
        ),
        ConversionOptions {
            target_aspect: Some(TargetAspect::SixteenNine),
            ..ConversionOptions::default()
        },
    );

    assert!(csv.contains("#DST_BAR_BODY_OFF,0,0,100,20,200,40,"));
    for command in [
        "#DST_BAR_MY_LAMP",
        "#DST_BAR_TROPHY",
        "#DST_BAR_LABEL",
        "#DST_BAR_GRAPH",
    ] {
        assert!(csv.contains(&format!("{command},0,0,100,20,200,40,")));
    }
    assert!(csv.contains("#DST_BAR_FLASH,0,0,100,20,200,40,"));
}

fn all_protected_options() -> ConversionOptions {
    ConversionOptions {
        resolution: Some(2),
        target_aspect: Some(TargetAspect::SixteenNine),
        aspect_protection: AspectProtection {
            bomb_effects: true,
            nowjudge: true,
            nowcombo: true,
        },
        ..ConversionOptions::default()
    }
}

fn convert(name: &str, layout: &str, options: ConversionOptions) -> String {
    let root = make_temp_root(&format!("{name}-source"));
    let output = make_temp_root(&format!("{name}-output"));
    fs::create_dir_all(root.join("Play")).expect("create fixture");
    fs::write(
        root.join("play.lr2skin"),
        "#INFORMATION,0,title,maker,\r\n#RESOLUTION,0\r\n",
    )
    .expect("write header");
    fs::write(root.join("Play").join("layout.csv"), layout.as_bytes()).expect("write layout");
    generate(&root, &output, options).expect("generation should succeed");
    fs::read_to_string(output.join("Play").join("layout.csv")).expect("read output")
}
