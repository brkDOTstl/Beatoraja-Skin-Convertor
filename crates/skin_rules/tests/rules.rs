use std::fs;

use skin_core::analyze_path;
use skin_rules::{ConversionOptions, dry_run, generate};

#[path = "rules/aspect.rs"]
mod aspect;
#[path = "rules/aspect_edge.rs"]
mod aspect_edge;
#[path = "rules/assets.rs"]
mod assets;
#[path = "rules/dependencies.rs"]
mod dependencies;
#[path = "rules/support.rs"]
mod support;

use support::{make_fixture, make_temp_root, populate_fixture, write_oadx_layout};

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
