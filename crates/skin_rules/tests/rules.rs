use std::{fs, path::Path};

use skin_core::analyze_path;
use skin_rules::{ConversionOptions, dry_run, generate};

#[path = "rules/aspect.rs"]
mod aspect;
#[path = "rules/aspect_edge.rs"]
mod aspect_edge;
#[path = "rules/assets.rs"]
mod assets;
#[path = "rules/dark_mode.rs"]
mod dark_mode;
#[path = "rules/dependencies.rs"]
mod dependencies;
#[path = "rules/support.rs"]
mod support;

use support::{make_fixture, make_temp_root, populate_fixture, write_oadx_layout};

#[cfg(windows)]
#[test]
fn generate_always_unpacks_dxa_font_archives() {
    let root = make_temp_root("dxa-font-source");
    fs::create_dir_all(root.join("Font")).expect("create Font directory");
    fs::write(root.join("play.lr2skin"), "#INFORMATION,0,title,maker,\r\n")
        .expect("write skin header");
    fs::copy(
        Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests")
            .join("fixtures")
            .join("tiny-font.dxa"),
        root.join("Font").join("SystemFont.dxa"),
    )
    .expect("copy DXA fixture");

    let output = make_temp_root("dxa-font-output");
    let report = generate(
        &root,
        &output,
        ConversionOptions {
            rules: vec!["blend-fallback".to_owned()],
            ..ConversionOptions::default()
        },
    )
    .expect("generation should unpack DXA archives");

    assert!(output.join("Font").join("SystemFont.dxa").is_file());
    assert_eq!(
        fs::read_to_string(output.join("Font").join("SystemFont").join("font.lr2font"))
            .expect("read unpacked LR2 font"),
        "#S,12,12\r\n"
    );
    assert!(
        output
            .join("Font")
            .join("SystemFont")
            .join("font.png")
            .is_file()
    );
    assert!(
        report
            .changes
            .iter()
            .any(|change| change.rule_id == "dxa-font-unpack")
    );
    assert!(
        report
            .manifest
            .changed_files
            .contains(&Path::new("Font").join("SystemFont").join("font.lr2font"))
    );
}

#[test]
fn generate_always_maps_lr2_poor_to_beatoraja_poor_plus_miss() {
    let root = make_temp_root("lr2-poor-count-source");
    fs::create_dir_all(root.join("Play")).expect("create Play directory");
    fs::write(
        root.join("play.lr2skin"),
        "#INFORMATION,0,title,maker,\r\n#INCLUDE,Play\\layout.csv\r\n",
    )
    .expect("write header");
    fs::write(
        root.join("Play").join("layout.csv"),
        concat!(
            "#SRC_NUMBER,0,0,0,0,100,10,10,1,0,0,113,0,4\r\n",
            "#DST_NUMBER,0,0,10,10,10,10,0,255,255,255,255,0,0,0,0,0,0,0,0,0\r\n",
            "#SRC_NUMBER,0,0,0,0,100,10,10,1,0,0,114,0,4\r\n",
            "#DST_NUMBER,0,0,20,10,10,10,0,255,255,255,255,0,0,0,0,0,0,0,0,0\r\n",
            "#SRC_NUMBER,0,0,0,0,100,10,10,1,0,0,84,0,4\r\n",
            "#DST_NUMBER,0,0,30,10,10,10,0,255,255,255,255,0,0,0,0,0,0,0,0,0\r\n",
        ),
    )
    .expect("write layout");

    let output = make_temp_root("lr2-poor-count-output");
    let report = generate(
        &root,
        &output,
        ConversionOptions {
            rules: vec!["blend-fallback".to_owned()],
            ..ConversionOptions::default()
        },
    )
    .expect("generation should succeed");

    let csv =
        fs::read_to_string(output.join("Play").join("layout.csv")).expect("read generated layout");
    assert!(csv.contains(",113,0,4"));
    assert!(csv.contains(",426,0,4"));
    assert!(csv.contains(",84,0,4"));
    assert!(!csv.contains(",114,0,4"));
    assert!(
        report
            .changes
            .iter()
            .any(|change| change.rule_id == "lr2-poor-count-compat")
    );
}

#[test]
fn dry_run_adds_lift_number_from_green_number_anchor() {
    let root = make_fixture("lift-number");
    let report = dry_run(
        &root,
        ConversionOptions {
            rules: vec!["lift-number".to_owned()],
            ..ConversionOptions::default()
        },
    )
    .expect("dry-run should succeed");

    assert!(
        report
            .changes
            .iter()
            .any(|change| change.after.contains(",314,"))
    );
    assert!(
        report
            .changes
            .iter()
            .any(|change| change.after.contains(",272,"))
    );
}

#[test]
fn generate_rebases_all_source_theme_references() {
    let parent = make_temp_root("theme-rebase-source");
    let source = parent.join("source-folder-name-differs");
    populate_fixture(&source);
    fs::write(
        source.join("play.lr2skin"),
        b"#INFORMATION,0,title,maker,LR2files\\Theme\\OA_DX_03\\.png\r\n#CUSTOMFILE,OBJECT,.\\LR2files\\Theme\\OA_DX_03\\object\\*.png,DEFAULT\r\n#CUSTOMFILE,SHARED,.\\LR2files\\Theme\\LR2\\shared\\*.png,DEFAULT\r\n#IMAGE,.\\LR2files\\Theme\\OA_DX_03\\object\\*.png,//0\r\n#INCLUDE,LR2files\\Theme\\OA_DX_03\\Play\\_csv\\play.csv\r\n",
    )
    .expect("write fixture skin");
    fs::write(
        source.join("Play").join("_csv").join("play.csv"),
        b"#IMAGE,LR2files\\Theme\\OA_DX_03\\parts\\*.png,//0\r\n",
    )
    .expect("write fixture CSV");

    let output = make_temp_root("theme-rebase-output").join("OADX3");
    let report = generate(
        &source,
        &output,
        ConversionOptions {
            rules: vec!["theme-path-rebase".to_owned()],
            ..ConversionOptions::default()
        },
    )
    .expect("generate should succeed");

    let skin = fs::read_to_string(output.join("play.lr2skin")).expect("read generated skin");
    let csv = fs::read_to_string(output.join("Play").join("_csv").join("play.csv"))
        .expect("read generated CSV");
    assert!(skin.contains("LR2files\\Theme\\OADX3\\Play\\_csv\\play.csv"));
    assert!(skin.contains(".\\LR2files\\Theme\\OADX3\\object\\*.png,DEFAULT"));
    assert!(skin.contains(".\\LR2files\\Theme\\OADX3\\object\\*.png,//0"));
    assert!(skin.contains(".\\LR2files\\Theme\\LR2\\shared\\*.png"));
    assert!(!skin.contains("OA_DX_03"));
    assert!(!csv.contains("OA_DX_03"));
    assert!(
        report
            .changes
            .iter()
            .any(|change| change.path.ends_with("play.csv"))
    );
}

#[test]
fn analyzer_detects_structural_play_csv_outside_play_directory() {
    let root = make_temp_root("structural-play");
    fs::create_dir_all(root.join("_csv")).expect("create fixture");
    fs::create_dir_all(root.join("Play")).expect("create named Play directory");
    fs::write(
        root.join("_csv").join("layout.csv"),
        b"#DST_JUDGELINE,0,0,25,315,144,6,0,255,255,255,255,1,0,0,0,0,0,0,0,0,0\r\n",
    )
    .expect("write structural play fixture");
    fs::write(
        root.join("Play").join("menu.csv"),
        b"#IMAGE,0,LR2files\\Theme\\Example\\menu.png\r\n",
    )
    .expect("write non-play CSV in named Play directory");

    let report = analyze_path(&root).expect("analysis should succeed");
    assert_eq!(report.counters.total_csv_files, 2);
    assert_eq!(report.counters.play_csv_files, 1);
}

#[test]
fn follow_lift_uses_source_object_ownership() {
    let root = make_temp_root("object-ownership");
    write_oadx_layout(&root);

    let report = dry_run(
        &root,
        ConversionOptions {
            rules: vec!["follow-lift-op4".to_owned()],
            ..ConversionOptions::default()
        },
    )
    .expect("dry-run should succeed");

    let changed = report
        .changes
        .iter()
        .filter(|change| change.rule_id == "follow-lift-op4")
        .collect::<Vec<_>>();
    assert_eq!(changed.len(), 7);
    assert!(changed.iter().all(|change| change.after.ends_with(",3")));
    assert!(
        changed
            .iter()
            .all(|change| !change.before.starts_with("#DST_LINE,"))
    );
    assert!(
        changed
            .iter()
            .all(|change| !change.before.contains(",777,30,"))
    );
}

#[test]
fn judge_line_materializes_solid_reference_and_keeps_native_commands() {
    let root = make_temp_root("judge-line-reference-image");
    fs::create_dir_all(root.join("Play")).expect("create fixture");
    fs::write(
        root.join("Play").join("layout.csv"),
        concat!(
            "#SRC_JUDGELINE,0,111,0,0,1,1,1,1,0,0,0,0,0,,,,,,,\r\n",
            "#DST_JUDGELINE,0,0,44,428,256,0,0,255,255,0,0,1,0,0,0,30,0,0,,0\r\n",
            "#DST_JUDGELINE,0,30,44,422,256,8,0,255,255,0,0,1,0,0,0,,,,,\r\n",
            "#SRC_JUDGELINE,1,2,0,0,256,8,1,1,0,0,0,0,0,,,,,,,\r\n",
            "#DST_JUDGELINE,1,0,340,350,256,8,0,255,255,255,255,1,0,0,0,,,,,\r\n",
        ),
    )
    .expect("write fixture");
    let output = make_temp_root("judge-line-reference-image-output");

    generate(
        &root,
        &output,
        ConversionOptions {
            rules: vec!["judge-line-follow".to_owned()],
            ..ConversionOptions::default()
        },
    )
    .expect("generation should succeed");

    let csv =
        fs::read_to_string(output.join("Play").join("layout.csv")).expect("read generated CSV");
    assert!(csv.contains("\\_bms-skin-generator\\judgeline-white.png,,,,,//0"));
    assert!(csv.contains("#SRC_JUDGELINE,0,0,0,0,1,1,1,1,0,0,0,0,0"));
    assert!(csv.contains("#DST_JUDGELINE,0,0,44,428,256,0,0,255,255,0,0,1,0,0,0,30,0,0,,0,3"));
    assert!(csv.contains("#DST_JUDGELINE,0,30,44,422,256,8,0,255,255,0,0,1,0,0,0,,,,,,3"));
    assert!(csv.contains("#SRC_JUDGELINE,1,2,0,0,256,8,1,1,0,0,0,0,0"));
    assert!(csv.contains("#DST_JUDGELINE,1,0,340,350,256,8,0,255,255,255,255,1,0,0,0,,,,,,3"));
    assert!(!csv.contains("#SRC_IMAGE,0,111"));

    let png = fs::read(
        output
            .join("_bms-skin-generator")
            .join("judgeline-white.png"),
    )
    .expect("read generated judge-line asset");
    assert_eq!(&png[..8], b"\x89PNG\r\n\x1a\n");
    assert_eq!(&png[16..24], &[0, 0, 0, 1, 0, 0, 0, 1]);
}

#[test]
fn judge_line_uses_actual_image_count_for_black_and_white_assets() {
    let root = make_temp_root("judge-line-image-indices");
    fs::create_dir_all(root.join("Play")).expect("create fixture");
    fs::write(
        root.join("Play").join("layout.csv"),
        concat!(
            "#IMAGE,LR2files\\Theme\\Source\\first.png,,,,,//0\r\n",
            "#IMAGE,LR2files\\Theme\\Source\\second.png,,,,,//duplicate-comment\r\n",
            "#SRC_JUDGELINE,0,111,0,0,1,1,1,1,0,0,0,0,0,,,,,,,\r\n",
            "#DST_JUDGELINE,0,0,44,428,256,1,0,255,255,255,255,1,0,0,0,,,,,\r\n",
            "#SRC_JUDGELINE,1,110,0,0,1,1,1,1,0,0,0,0,0,,,,,,,\r\n",
            "#DST_JUDGELINE,1,0,340,428,256,1,0,255,255,255,255,1,0,0,0,,,,,\r\n",
        ),
    )
    .expect("write fixture");
    let output = make_temp_root("judge-line-image-indices-output");

    generate(
        &root,
        &output,
        ConversionOptions {
            rules: vec!["judge-line-follow".to_owned()],
            ..ConversionOptions::default()
        },
    )
    .expect("generation should succeed");

    let csv =
        fs::read_to_string(output.join("Play").join("layout.csv")).expect("read generated CSV");
    assert!(csv.contains("judgeline-black.png,,,,,//2"));
    assert!(csv.contains("judgeline-white.png,,,,,//3"));
    assert!(csv.contains("#SRC_JUDGELINE,0,3,"));
    assert!(csv.contains("#SRC_JUDGELINE,1,2,"));
    assert!(
        output
            .join("_bms-skin-generator")
            .join("judgeline-black.png")
            .is_file()
    );
    assert!(
        output
            .join("_bms-skin-generator")
            .join("judgeline-white.png")
            .is_file()
    );
}

#[test]
fn judge_line_dry_run_reports_but_does_not_write_generated_asset() {
    let root = make_temp_root("judge-line-dry-run");
    fs::create_dir_all(root.join("Play")).expect("create fixture");
    fs::write(
        root.join("Play").join("layout.csv"),
        concat!(
            "#SRC_JUDGELINE,0,111,0,0,1,1,1,1,0,0,0,0,0,,,,,,,\r\n",
            "#DST_JUDGELINE,0,0,44,428,256,1,0,255,255,255,255,1,0,0,0,,,,,\r\n",
        ),
    )
    .expect("write fixture");

    let report = dry_run(
        &root,
        ConversionOptions {
            rules: vec!["judge-line-follow".to_owned()],
            ..ConversionOptions::default()
        },
    )
    .expect("dry-run should succeed");

    let relative = std::path::Path::new("_bms-skin-generator").join("judgeline-white.png");
    assert!(report.manifest.changed_files.contains(&relative));
    assert!(!root.join(relative).exists());
}

#[test]
fn judge_line_compat_runs_after_lift_geometry_consumers() {
    let root = make_temp_root("judge-line-rule-order");
    fs::create_dir_all(root.join("Play")).expect("create fixture");
    fs::write(
        root.join("Play").join("layout.csv"),
        concat!(
            "#SRC_JUDGELINE,0,111,0,0,1,1,1,1,0,0,0,0,0,,,,,,,\r\n",
            "#DST_JUDGELINE,0,0,44,356,256,8,0,255,255,0,0,1,0,0,0,,,,,\r\n",
            "#SRC_SLIDER,0,5,0,0,256,348,1,1,0,0,0,0,4,348,,,,,,,\r\n",
            "#DST_SLIDER,0,0,44,0,256,356,0,255,255,255,255,1,0,0,0,0,0,0,0,0,0\r\n",
            "#SRC_NUMBER,0,9,0,0,70,12,10,1,0,0,14,0,3,,,,,,,\r\n",
            "#DST_NUMBER,0,0,88,100,10,12,0,255,255,255,255,1,0,0,0,0,0,0,0,0,0\r\n",
        ),
    )
    .expect("write fixture");
    let output = make_temp_root("judge-line-rule-order-output");

    generate(
        &root,
        &output,
        ConversionOptions {
            rules: vec![
                "judge-line-follow".to_owned(),
                "lift-cover-copy-sudden".to_owned(),
                "green-number".to_owned(),
                "lift-number".to_owned(),
            ],
            lift_cover_strategy: Some("copy-sudden".to_owned()),
            ..ConversionOptions::default()
        },
    )
    .expect("generation should succeed");

    let csv =
        fs::read_to_string(output.join("Play").join("layout.csv")).expect("read generated CSV");
    assert!(csv.contains("#SRC_JUDGELINE,0,0"));
    assert!(csv.contains("#DST_JUDGELINE,0,0,44,356,256,8"));
    assert!(csv.contains("#SRC_LIFT,0,5"));
    assert!(csv.contains(",313,0,3"));
    assert!(csv.contains(",314,0,3"));
}

#[test]
fn key_laser_layer_registers_beams_before_notes_without_moving_other_images() {
    let root = make_temp_root("key-laser-layer");
    fs::create_dir_all(root.join("_csv")).expect("create fixture");
    fs::write(
        root.join("_csv").join("layout.csv"),
        concat!(
            "#SRC_NOTE,0,1,0,0,32,16,1,1,0,0,0,0,0\r\n",
            "#IF,935\r\n",
            "#DST_NOTE,0,0,10,300,32,16,0,255,255,255,255,0,0,0,0,0,0,0,0,0\r\n",
            "#ENDIF\r\n",
            "#SRC_IMAGE,0,2,0,0,32,256,1,1,0,100,0,0,0\r\n",
            "#DST_IMAGE,0,0,10,316,32,-256,0,255,255,255,255,1,0,0,0,0,100,0,0,0,3\r\n",
            "#SRC_IMAGE,0,2,32,0,32,256,1,1,0,120,0,0,0\r\n",
            "#DST_IMAGE,0,0,10,316,32,-256,0,255,255,255,255,1,0,0,0,0,120,0,0,0,3\r\n",
            "#SRC_IMAGE,0,3,0,0,64,64,1,1,0,0,0,0,0\r\n",
            "#DST_IMAGE,0,0,100,100,64,64,0,255,255,255,255,1,0,0,0,0,0,0,0,0\r\n",
        ),
    )
    .expect("write fixture");
    let output = make_temp_root("key-laser-layer-output");

    let report = generate(
        &root,
        &output,
        ConversionOptions {
            rules: vec!["key-laser-layer-compat".to_owned()],
            ..ConversionOptions::default()
        },
    )
    .expect("generation should succeed");

    assert!(report.warnings.is_empty());
    assert_eq!(
        report
            .changes
            .iter()
            .filter(|change| change.rule_id == "key-laser-layer-compat")
            .count(),
        1
    );
    let csv =
        fs::read_to_string(output.join("_csv").join("layout.csv")).expect("read generated CSV");
    let note_source = csv.find("#SRC_NOTE").expect("note source");
    let laser_on = csv.find("#SRC_IMAGE,0,2,0,0,32,256").expect("laser on");
    let laser_off = csv.find("#SRC_IMAGE,0,2,32,0,32,256").expect("laser off");
    let note_condition = csv.find("#IF,935").expect("note condition");
    let note_destination = csv.find("#DST_NOTE").expect("note destination");
    let unrelated = csv
        .find("#SRC_IMAGE,0,3,0,0,64,64")
        .expect("unrelated image");
    assert!(note_source < laser_on);
    assert!(laser_on < laser_off);
    assert!(laser_off < note_condition);
    assert!(note_condition < note_destination);
    assert!(note_destination < unrelated);
}

#[test]
fn key_laser_layer_moves_pure_conditional_blocks_intact() {
    let root = make_temp_root("conditional-key-laser-layer");
    fs::create_dir_all(root.join("_csv")).expect("create fixture");
    fs::write(
        root.join("_csv").join("layout.csv"),
        concat!(
            "#SRC_NOTE,0,1,0,0,32,16,1,1,0,0,0,0,0\r\n",
            "#DST_NOTE,0,0,10,300,32,16,0,255,255,255,255,0,0,0,0,0,0,0,0,0\r\n",
            "#IF,901,920\r\n",
            "#SRC_IMAGE,0,2,0,0,32,222,1,1,0,100,0,0,0\r\n",
            "#DST_IMAGE,0,0,10,300,32,-222,0,255,255,255,255,1,0,0,0,0,100,0,0,0\r\n",
            "#SRC_IMAGE,0,2,32,0,32,222,1,1,0,101,0,0,0\r\n",
            "#DST_IMAGE,0,0,42,300,32,-222,0,255,255,255,255,1,0,0,0,0,101,0,0,0\r\n",
            "#ENDIF\r\n",
        ),
    )
    .expect("write fixture");
    let output = make_temp_root("conditional-key-laser-layer-output");

    let report = generate(
        &root,
        &output,
        ConversionOptions {
            rules: vec!["key-laser-layer-compat".to_owned()],
            ..ConversionOptions::default()
        },
    )
    .expect("generation should succeed");

    assert!(report.warnings.is_empty());
    let csv =
        fs::read_to_string(output.join("_csv").join("layout.csv")).expect("read generated CSV");
    let condition = csv.find("#IF,901,920").expect("laser condition");
    let condition_end = csv.find("#ENDIF").expect("laser condition end");
    let note_destination = csv.find("#DST_NOTE").expect("note destination");
    assert!(condition < condition_end);
    assert!(condition_end < note_destination);
}

#[test]
fn key_laser_layer_leaves_mixed_conditional_blocks_unchanged() {
    let root = make_temp_root("mixed-key-laser-layer");
    fs::create_dir_all(root.join("_csv")).expect("create fixture");
    let original = concat!(
        "#SRC_NOTE,0,1,0,0,32,16,1,1,0,0,0,0,0\r\n",
        "#DST_NOTE,0,0,10,300,32,16,0,255,255,255,255,0,0,0,0,0,0,0,0,0\r\n",
        "#IF,900\r\n",
        "#IF,901\r\n",
        "#SRC_IMAGE,0,2,0,0,32,222,1,1,0,100,0,0,0\r\n",
        "#DST_IMAGE,0,0,10,300,32,-222,0,255,255,255,255,1,0,0,0,0,100,0,0,0\r\n",
        "#ENDIF\r\n",
        "#SRC_IMAGE,0,3,0,0,64,64,1,1,0,0,0,0,0\r\n",
        "#DST_IMAGE,0,0,100,100,64,64,0,255,255,255,255,1,0,0,0,0,0,0,0,0\r\n",
        "#ENDIF\r\n",
    );
    fs::write(root.join("_csv").join("layout.csv"), original).expect("write fixture");
    let output = make_temp_root("mixed-key-laser-layer-output");

    let report = generate(
        &root,
        &output,
        ConversionOptions {
            rules: vec!["key-laser-layer-compat".to_owned()],
            ..ConversionOptions::default()
        },
    )
    .expect("generation should succeed");

    assert!(report.changes.is_empty());
    assert!(
        report
            .warnings
            .iter()
            .any(|warning| warning.rule_id == "key-laser-layer-compat")
    );
    assert_eq!(
        fs::read(output.join("_csv").join("layout.csv")).expect("read generated CSV"),
        original.as_bytes()
    );
}

#[test]
fn copy_sudden_lift_and_numbers_match_judgment_geometry() {
    let root = make_temp_root("oadx-modules");
    write_oadx_layout(&root);
    let output = make_temp_root("oadx-modules-output");

    generate(
        &root,
        &output,
        ConversionOptions {
            rules: vec![
                "lift-cover-copy-sudden".to_owned(),
                "green-number".to_owned(),
                "lift-number".to_owned(),
            ],
            lift_cover_strategy: Some("copy-sudden".to_owned()),
            ..ConversionOptions::default()
        },
    )
    .expect("generation should succeed");

    let csv =
        fs::read_to_string(output.join("_csv").join("layout.csv")).expect("read generated layout");
    assert!(csv.contains("#SRC_LIFT,0,5,0,0,144,321,1,1,0,0,321,0,0,0"));
    assert!(csv.contains("#DST_LIFT,0,0,25,321,256,356,2,255,255,255,255,1,0,0,0,0,0,0,0,0,3",));
    assert!(csv.contains("#SRC_NUMBER,0,9,0,0,70,12,10,1,0,0,313,0,3"));
    assert!(csv.contains("#DST_NUMBER,0,0,88,-9,10,12,0,255,0,255,0,1,0,0,0,0,0,270,271,0,4",));
    assert!(csv.contains("#DST_NUMBER,0,700,88,-9,10,12,0,255,0,255,0,1,0,0,0,0,0,270,271,0,4",));
    assert!(csv.contains("#SRC_NUMBER,0,9,0,0,70,12,10,1,0,0,314,0,3"));
    assert!(
        csv.contains("#DST_NUMBER,0,0,88,301,10,12,0,255,255,255,255,1,0,0,0,0,0,270,272,0,0",)
    );
    assert!(
        csv.contains("#DST_NUMBER,0,700,88,301,10,12,0,255,255,255,255,1,0,0,0,0,0,270,272,0,0",)
    );
}

#[test]
fn number_rules_fall_back_to_an_existing_digit_sheet() {
    let root = make_temp_root("number-fallback");
    fs::create_dir_all(root.join("Play")).expect("create fixture");
    fs::write(
        root.join("play.lr2skin"),
        b"#INFORMATION,0,title,maker,\r\n#RESOLUTION,0\r\n",
    )
    .expect("write header");
    fs::write(
        root.join("Play").join("layout.csv"),
        concat!(
            "#DST_JUDGELINE,0,0,44,356,256,0,0,255,255,255,255,1,0,0,0,0,0,0,0,0,0\r\n",
            "#IF,912\r\n",
            "#SRC_NUMBER,0,0,0,345,130,17,10,1,0,0,107,0,3\r\n",
            "#DST_NUMBER,0,0,316,407,17,17,0,255,255,255,255,1,0,0,0,0,0,230,43,0\r\n",
            "#ENDIF\r\n",
        ),
    )
    .expect("write layout");
    let output = make_temp_root("number-fallback-output");

    generate(
        &root,
        &output,
        ConversionOptions {
            rules: vec!["green-number".to_owned(), "lift-number".to_owned()],
            ..ConversionOptions::default()
        },
    )
    .expect("generation should succeed");

    let csv =
        fs::read_to_string(output.join("Play").join("layout.csv")).expect("read generated layout");
    assert!(csv.contains("//bms-skin-generator:R005:GREEN_NUMBER:FALLBACK"));
    assert!(csv.contains("//bms-skin-generator:R008:SUDDEN_NUMBER:FALLBACK"));
    assert!(csv.contains("#SRC_NUMBER,0,0,0,345,130,17,10,1,0,0,14,0,3"));
    assert!(csv.contains("#DST_NUMBER,0,0,146,9,17,17,0,255,255,255,255,1,0,0,0,0,0,270,0,0,0"));
    assert!(csv.contains("#SRC_NUMBER,0,0,0,345,130,17,10,1,0,0,313,0,3"));
    assert!(csv.contains("#DST_NUMBER,0,0,146,-9,17,17,0,255,0,255,0,1,0,0,0,0,0,270,271,0,4"));
    assert!(csv.contains("//bms-skin-generator:R004:LIFT_NUMBER:FALLBACK"));
    assert!(csv.contains("#SRC_NUMBER,0,0,0,345,130,17,10,1,0,0,314,0,3"));
    assert!(
        csv.contains("#DST_NUMBER,0,0,146,337,17,17,0,255,255,255,255,1,0,0,0,0,0,270,272,0,0")
    );
    assert!(
        csv.find("#ENDIF")
            .is_some_and(|endif| csv[endif..].contains("GREEN_NUMBER:FALLBACK"))
    );
}

#[test]
fn blend_fallback_disables_only_duplicate_button_fades() {
    let root = make_temp_root("button-fade");
    fs::create_dir_all(root.join("Play")).expect("create fixture");
    fs::write(
        root.join("play.lr2skin"),
        b"#INFORMATION,0,title,maker,\r\n",
    )
    .expect("write header");
    fs::write(
        root.join("Play").join("layout.csv"),
        concat!(
            "#DST_JUDGELINE,0,0,44,356,256,0,0,255,255,255,255,1,0,0,0,0,0,0,0,0,0\r\n",
            "#SRC_BUTTON,0,0,120,453,119,156,1,6,0,0,40,0,0,0\r\n",
            "#DST_BUTTON,0,50,424,459,96,21,0,255,255,255,255,1,1,0,0,0,0,0,0,0\r\n",
            "#SRC_BUTTON,0,0,120,453,119,156,1,6,0,0,40,0,0,0\r\n",
            "#DST_BUTTON,0,1500,424,459,96,21,0,150,255,255,255,2,1,0,0,-1,0,0,0,0\r\n",
            "#DST_BUTTON,0,1600,424,459,96,21,0,0,255,255,255,2,1,0,0\r\n",
            "#SRC_IMAGE,0,0,0,0,10,10,1,1,0,0,0,0,0\r\n",
            "#DST_IMAGE,0,50,0,0,10,10,0,255,255,255,255,1,1,0,0,0,0,33,0,0\r\n",
            "#SRC_IMAGE,0,0,0,0,10,10,1,1,0,0,0,0,0\r\n",
            "#DST_IMAGE,0,1500,0,0,10,10,0,150,255,255,255,2,1,0,0,-1,0,0,0,0\r\n",
            "#DST_IMAGE,0,1600,0,0,10,10,0,0,255,255,255,2,1,0,0\r\n",
        ),
    )
    .expect("write layout");
    let output = make_temp_root("button-fade-output");

    generate(
        &root,
        &output,
        ConversionOptions {
            rules: vec!["blend-fallback".to_owned()],
            ..ConversionOptions::default()
        },
    )
    .expect("generation should succeed");

    let csv =
        fs::read_to_string(output.join("Play").join("layout.csv")).expect("read generated layout");
    assert!(csv.contains("#DST_BUTTON,0,0,424,459,96,21"));
    assert!(csv.contains("#DST_IMAGE,0,0,0,0,10,10"));
    assert_eq!(csv.matches("R007:DISABLED_FADE").count(), 6);
    assert!(csv.contains("R007:DISABLED_FADE,#SRC_IMAGE"));
}

#[test]
fn generate_refuses_input_as_output() {
    let root = make_fixture("same-output");
    let result = generate(&root, &root, ConversionOptions::default());
    assert!(result.is_err());
}

#[test]
fn generate_refuses_lexical_output_inside_input() {
    let root = make_fixture("inside-output");
    let folder = root.file_name().expect("fixture folder").to_owned();
    let output = root.join("..").join(folder).join("generated");
    let result = generate(&root, &output, ConversionOptions::default());
    assert!(result.is_err());
    assert!(!output.exists());
}
