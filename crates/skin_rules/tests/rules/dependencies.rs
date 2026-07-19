use std::fs;

use skin_rules::{ConversionOptions, TargetAspect, dry_run};

use crate::support::make_temp_root;

#[test]
fn explicit_resolution_activates_geometry_and_declaration_rules() {
    let root = fixture("explicit-resolution", 0);
    let report = dry_run(
        &root,
        ConversionOptions {
            rules: vec!["aspect-ratio-convert".to_owned()],
            resolution: Some(2),
            ..ConversionOptions::default()
        },
    )
    .expect("dry-run should succeed");

    assert!(
        report
            .selected_rules
            .contains(&"resolution-declaration".to_owned())
    );
    assert!(
        report
            .selected_rules
            .contains(&"aspect-ratio-convert".to_owned())
    );
    assert!(report.changes.iter().any(|change| {
        change.rule_id == "resolution-declaration" && change.after == "#RESOLUTION,2"
    }));
}

#[test]
fn explicit_aspect_activates_geometry_without_changing_resolution() {
    let root = fixture("explicit-aspect", 2);
    let report = dry_run(
        &root,
        ConversionOptions {
            rules: vec!["follow-lift-op4".to_owned()],
            target_aspect: Some(TargetAspect::FourThree),
            ..ConversionOptions::default()
        },
    )
    .expect("dry-run should succeed");

    assert!(
        report
            .selected_rules
            .contains(&"aspect-ratio-convert".to_owned())
    );
    assert!(
        !report
            .selected_rules
            .contains(&"resolution-declaration".to_owned())
    );
}

fn fixture(name: &str, resolution: u8) -> std::path::PathBuf {
    let root = make_temp_root(name);
    fs::create_dir_all(root.join("Play")).expect("create fixture");
    fs::write(
        root.join("play.lr2skin"),
        format!("#INFORMATION,0,title,maker,\r\n#RESOLUTION,{resolution}\r\n"),
    )
    .expect("write header");
    fs::write(
        root.join("Play").join("layout.csv"),
        b"#DST_IMAGE,0,0,100,20,200,40,0,255,255,255,255,1,0,0,0,0,0,0,0,0,0\r\n",
    )
    .expect("write layout");
    root
}
