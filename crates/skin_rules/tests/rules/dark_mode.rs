use std::{
    fs,
    path::{Path, PathBuf},
};

use skin_rules::{ConversionOptions, generate};

use crate::support::make_temp_root;

#[test]
fn dark_mode_generates_routed_variants_with_lr2_filtering() {
    let root = make_temp_root("dark-mode-source");
    fs::create_dir_all(root.join("Play")).expect("create Play directory");
    fs::write(
        root.join("play.lr2skin"),
        concat!(
            "#INFORMATION,0,title,maker,\r\n",
            "#CUSTOMOPTION,SIDE,900,LEFT,RIGHT\r\n",
            "#ENDOFHEADER\r\n",
            "#IF,900\r\n",
            "#INCLUDE,Play\\layout.csv\r\n",
            "#ENDIF",
        ),
    )
    .expect("write play header");
    fs::write(
        root.join("Play").join("layout.csv"),
        concat!(
            "#IMAGE,parts.png\r\n",
            "#IMAGE,lane.png,,,,,//1\r\n",
            "#SRC_IMAGE,0,0,0,0,10,10,1,1,0,0,0,0,0\r\n",
            "#DST_IMAGE,0,0,10,10,10,10,0,255,255,255,255,0,0,0,0,0,0,0,0,0\r\n",
            "#SRC_IMAGE,0,0,0,0,10,10,1,1,0,2,0,0,0\r\n",
            "#DST_IMAGE,0,0,20,10,10,10,0,255,255,255,255,0,0,0,0,0,0,0,0,0\r\n",
            "#SRC_IMAGE,0,0,0,0,10,10,1,1,0,10,0,0,0\r\n",
            "#DST_IMAGE,0,0,30,10,10,10,0,255,255,255,255,0,0,0,0,0,0,0,0,0\r\n",
            "#SRC_NUMBER,0,0,0,0,100,10,10,1,1,0,0,0,0,0\r\n",
            "#DST_NUMBER,0,0,40,10,10,10,0,255,255,255,255,0,0,0,0,0,0,0,0,0\r\n",
            "#DST_NOWCOMBO_1P,0,0,45,10,10,10,0,255,255,255,255,0,0,0,0,0,0,0,0,0\r\n",
            "#SRC_SLIDER,0,1,0,0,10,10,1,1,0,0,2,100,4,1\r\n",
            "#DST_SLIDER,0,0,10,20,100,200,0,128,255,255,255,2,0,0,0,0,0,0,0,0\r\n",
            "#SRC_LIFT,0,1,0,0,10,10,1,1,0,0,300,0,0,0\r\n",
            "#DST_LIFT,0,0,10,300,100,200,2,128,255,255,255,1,0,0,0,0,0,0,0,0,3\r\n",
            "#SRC_NOTE,0,0,0,0,10,10,1,1,0,0,0,0,0\r\n",
            "#DST_NOTE,0,0,10,10,10,10,0,255,255,255,255,0,0,0,0,0,0,0,0,0\r\n",
            "#SRC_JUDGELINE,0,0,0,0,10,2,1,1,0,0,0,0,0\r\n",
            "#DST_JUDGELINE,0,0,10,300,100,2,0,255,255,255,255,0,0,0,0,0,0,0,0,3\r\n",
            "#SRC_GROOVEGAUGE,0,0,0,0,10,10,1,1,0,0,0,0,0\r\n",
            "#DST_GROOVEGAUGE,0,0,10,400,100,10,0,255,255,255,255,0,0,0,0,0,0,0,0,0\r\n",
            "#SRC_NOWJUDGE_1P,0,0,0,0,10,10,1,1,0,0,0,0,0\r\n",
            "#DST_NOWJUDGE_1P,0,0,10,200,100,20,0,255,255,255,255,0,0,0,0,0,0,0,0,0\r\n",
            "#INCLUDE,Play\\helper.csv\r\n",
        ),
    )
    .expect("write play layout");
    fs::write(
        root.join("Play").join("helper.csv"),
        concat!(
            "#SRC_IMAGE,0,0,0,0,10,10,1,1,0,3,0,0,0\r\n",
            "#DST_IMAGE,0,0,50,10,10,10,0,255,255,255,255,0,0,0,0,0,0,0,0,0\r\n",
        ),
    )
    .expect("write helper layout");
    fs::create_dir_all(root.join("Result")).expect("create Result directory");
    fs::write(
        root.join("Result").join("unrelated.csv"),
        "#SRC_IMAGE,0,0,0,0,10,10,1,1,0,0,0,0,0\r\n",
    )
    .expect("write unrelated layout");

    let output = make_temp_root("dark-mode-output");
    generate(
        &root,
        &output,
        ConversionOptions {
            rules: vec!["lr2-dark-mode".to_owned()],
            ..ConversionOptions::default()
        },
    )
    .expect("generation should succeed");

    let header = fs::read_to_string(output.join("play.lr2skin")).expect("read routed header");
    assert!(header.contains("#CUSTOMFILE,LR2 DARK MODE,"));
    assert!(header.contains(",OFF"));
    assert!(
        header
            .find("#CUSTOMFILE,LR2 DARK MODE")
            .expect("custom file")
            < header.find("#ENDOFHEADER").expect("end of header")
    );
    assert!(header.contains("//bms-skin-generator:R010:ROUTED_INCLUDE,#INCLUDE,Play\\layout.csv"));
    assert!(header.contains("#INCLUDE,LR2files\\Theme\\"));
    assert!(!header.contains("#ENDIF//bms-skin-generator"));
    assert!(header.contains("#ENDIF\r\n//bms-skin-generator:R010:LR2_DARK_MODE_ROUTE"));

    let selector = selector_directory(&output);
    let off = fs::read_to_string(selector.join("OFF")).expect("read OFF route");
    let dark_route = fs::read_to_string(selector.join("DARK")).expect("read DARK route");
    let dark2_route = fs::read_to_string(selector.join("DARK2")).expect("read DARK2 route");
    assert!(off.contains("#IF,900\r\n#INCLUDE,Play\\layout.csv\r\n#ENDIF"));
    assert!(dark_route.contains("#INCLUDE,Play\\layout.csv.bmsg-dark1"));
    assert!(dark2_route.contains("#INCLUDE,Play\\layout.csv.bmsg-dark2"));

    let dark = fs::read_to_string(output.join("Play").join("layout.csv.bmsg-dark1"))
        .expect("read DARK CSV");
    assert!(dark.contains("#SRC_IMAGE,0,0,0,0,10,10,1,1,0,0,"));
    assert!(dark.contains("#DST_IMAGE,0,0,10,10,10,10,0,0,255,255,255,0,"));
    assert!(dark.contains("#SRC_IMAGE,0,0,0,0,10,10,1,1,0,2,"));
    assert!(dark.contains("#SRC_IMAGE,0,0,0,0,10,10,1,1,0,10,"));
    assert!(dark.contains("#SRC_NUMBER,"));
    assert!(dark.contains("#SRC_SLIDER,0,1,0,0,10,10,1,1,0,0,2,100,4,1"));
    assert!(dark.contains("#INCLUDE,Play\\helper.csv.bmsg-dark1"));

    let dark2 = fs::read_to_string(output.join("Play").join("layout.csv.bmsg-dark2"))
        .expect("read DARK2 CSV");
    assert!(dark2.contains("#SRC_IMAGE,0,0,0,0,10,10,1,1,0,0,"));
    assert!(dark2.contains("#DST_IMAGE,0,0,10,10,10,10,0,0,255,255,255,0,"));
    assert!(dark2.contains("#SRC_IMAGE,0,0,0,0,10,10,1,1,0,2,"));
    assert!(dark2.contains("#SRC_IMAGE,0,0,0,0,10,10,1,1,0,10,"));
    assert!(dark2.contains("#DST_IMAGE,0,0,30,10,10,10,0,0,255,255,255,0,"));
    assert!(dark2.contains("#SRC_NUMBER,"));
    assert!(dark2.contains("#DST_NUMBER,0,0,40,10,10,10,0,0,255,255,255,0,"));
    assert!(dark2.contains("#DST_NOWCOMBO_1P,0,0,45,10,10,10,0,255,255,255,255,"));
    assert!(dark2.contains("#IMAGE,LR2files\\Theme\\"));
    assert!(dark2.contains("_bms-skin-generator\\dark\\lane-cover-black.png,,,,,//2"));
    assert!(dark2.contains("#SRC_SLIDER,0,2,0,0,1,1,1,1,0,0,2,100,4,1"));
    assert!(dark2.contains("#DST_SLIDER,0,0,10,20,100,200,0,255,0,0,0,0,"));
    assert!(dark2.contains("#SRC_LIFT,0,2,0,0,1,1,1,1,0,0,300,0,0,0"));
    assert!(dark2.contains("#DST_LIFT,0,0,10,300,100,200,2,255,0,0,0,0,"));
    assert!(dark2.contains("#SRC_NOTE,"));
    assert!(dark2.contains("#SRC_JUDGELINE,"));
    assert!(dark2.contains("#SRC_GROOVEGAUGE,"));
    assert!(dark2.contains("#SRC_NOWJUDGE_1P,"));
    assert!(dark2.contains("#INCLUDE,Play\\helper.csv.bmsg-dark2"));
    assert!(output.join("Play").join("helper.csv.bmsg-dark1").is_file());
    assert!(output.join("Play").join("helper.csv.bmsg-dark2").is_file());
    assert!(
        output
            .join("_bms-skin-generator")
            .join("dark")
            .join("lane-cover-black.png")
            .is_file()
    );
    assert!(
        !output
            .join("Result")
            .join("unrelated.csv.bmsg-dark1")
            .exists()
    );
    assert!(
        !output
            .join("Result")
            .join("unrelated.csv.bmsg-dark2")
            .exists()
    );
}

#[test]
fn dark_modes_keep_a_rhythm_line_with_timer_140_on_its_destination() {
    let root = make_temp_root("dark-mode-destination-timer-source");
    fs::create_dir_all(root.join("Play")).expect("create Play directory");
    fs::write(
        root.join("play.lr2skin"),
        "#INFORMATION,0,title,maker,\r\n#INCLUDE,Play\\layout.csv\r\n",
    )
    .expect("write header");
    fs::write(
        root.join("Play").join("layout.csv"),
        concat!(
            "#IMAGE,parts.png\r\n",
            "#SRC_IMAGE,0,0,0,0,100,4,1,1,0,0,0,0,0\r\n",
            "#DST_IMAGE,0,0,10,10,100,4,0,255,255,255,255,1,0,0,0,0,0,0,0,0,0\r\n",
            "#SRC_IMAGE,0,0,0,4,100,4,1,1,0,0,0,0,0\r\n",
            "#DST_IMAGE,0,0,10,20,100,4,0,255,255,255,255,1,0,0,0,0,140,0,0,0,0\r\n",
        ),
    )
    .expect("write layout");

    let output = make_temp_root("dark-mode-destination-timer-output");
    generate(
        &root,
        &output,
        ConversionOptions {
            rules: vec!["lr2-dark-mode".to_owned()],
            ..ConversionOptions::default()
        },
    )
    .expect("generation should succeed");

    for suffix in ["dark1", "dark2"] {
        let csv = fs::read_to_string(
            output
                .join("Play")
                .join(format!("layout.csv.bmsg-{suffix}")),
        )
        .expect("read DARK CSV");
        assert!(csv.contains("#DST_IMAGE,0,0,10,10,100,4,0,0,255,255,255,1,0,0,0,0,0,0,0,0,0"));
        assert!(csv.contains("#DST_IMAGE,0,0,10,20,100,4,0,255,255,255,255,1,0,0,0,0,140,0,0,0,0"));
    }
}

#[test]
fn dark_modes_keep_static_judge_line_compatibility_images() {
    let root = make_temp_root("dark-mode-judge-line-source");
    fs::create_dir_all(root.join("Play")).expect("create Play directory");
    fs::write(
        root.join("play.lr2skin"),
        "#INFORMATION,0,title,maker,\r\n#INCLUDE,Play\\layout.csv\r\n",
    )
    .expect("write header");
    fs::write(
        root.join("Play").join("layout.csv"),
        concat!(
            "#IMAGE,parts.png\r\n",
            "#SRC_JUDGELINE,0,0,0,0,144,14,1,1,0,0,0,0,0\r\n",
            "#DST_JUDGELINE,0,250,448,468,384,0,0,255,255,255,255,1,0,0,0,500,0,0,0,0,3\r\n",
            "#DST_JUDGELINE,0,250,448,468,384,14,0,255,255,255,255,1,0,0,0,0,0,0,0,0,3\r\n",
            "#SRC_IMAGE,0,0,0,0,144,14,1,1,0,0,0,0,0\r\n",
            "#DST_IMAGE,0,0,448,468,384,14,0,255,255,255,255,1,0,0,0,0,0,0,0,0,3\r\n",
            "#SRC_IMAGE,0,0,0,20,144,14,1,1,0,0,0,0,0\r\n",
            "#DST_IMAGE,0,0,448,430,384,14,0,255,255,255,255,1,0,0,0,0,0,0,0,0,0\r\n",
        ),
    )
    .expect("write layout");

    let output = make_temp_root("dark-mode-judge-line-output");
    generate(
        &root,
        &output,
        ConversionOptions {
            rules: vec!["lr2-dark-mode".to_owned()],
            ..ConversionOptions::default()
        },
    )
    .expect("generation should succeed");

    for suffix in ["dark1", "dark2"] {
        let csv = fs::read_to_string(
            output
                .join("Play")
                .join(format!("layout.csv.bmsg-{suffix}")),
        )
        .expect("read DARK CSV");
        assert!(
            csv.contains("#DST_IMAGE,0,0,448,468,384,14,0,255,255,255,255,1,0,0,0,0,0,0,0,0,3")
        );
        assert!(csv.contains("#DST_IMAGE,0,0,448,430,384,14,0,0,255,255,255,1,0,0,0,0,0,0,0,0,0"));
    }
}

#[test]
fn dark_modes_keep_marked_judge_line_image_fallbacks() {
    let root = make_temp_root("dark-mode-marked-judge-line-source");
    fs::create_dir_all(root.join("Play")).expect("create Play directory");
    fs::write(
        root.join("play.lr2skin"),
        "#INFORMATION,0,title,maker,\r\n#INCLUDE,Play\\layout.csv\r\n",
    )
    .expect("write header");
    fs::write(
        root.join("Play").join("layout.csv"),
        concat!(
            "#SRC_JUDGELINE,0,112,0,0,1,1,1,1,0,0,0,0,0\r\n",
            "#DST_JUDGELINE,0,0,44,428,256,8,0,255,255,255,255,1,0,0,0,0,0,0,0,0,0\r\n",
        ),
    )
    .expect("write layout");

    let output = make_temp_root("dark-mode-marked-judge-line-output");
    generate(
        &root,
        &output,
        ConversionOptions {
            rules: vec!["judge-line-follow".to_owned(), "lr2-dark-mode".to_owned()],
            ..ConversionOptions::default()
        },
    )
    .expect("generation should succeed");

    for path in [
        output.join("Play").join("layout.csv"),
        output.join("Play").join("layout.csv.bmsg-dark1"),
        output.join("Play").join("layout.csv.bmsg-dark2"),
    ] {
        let csv = fs::read_to_string(path).expect("read generated CSV");
        assert!(csv.contains("#SRC_IMAGE,0,112,"));
        assert!(csv.contains("//bms-skin-generator:JUDGE_LINE_IMAGE_FALLBACK"));
        assert!(csv.contains("#DST_IMAGE,0,0,44,428,256,8,0,255,255,255,255,"));
    }
}

#[test]
fn dark2_keeps_sudden_green_and_lift_numbers() {
    let root = make_temp_root("dark-mode-lane-cover-numbers-source");
    fs::create_dir_all(root.join("Play")).expect("create Play directory");
    fs::write(
        root.join("play.lr2skin"),
        "#INFORMATION,0,title,maker,\r\n#INCLUDE,Play\\layout.csv\r\n",
    )
    .expect("write header");
    let mut layout = String::new();
    for num in [14, 15, 313, 314, 100] {
        layout.push_str(&format!("#SRC_NUMBER,0,0,0,0,10,10,10,1,0,0,{num},0,3\r\n"));
        layout.push_str(&format!(
            "#DST_NUMBER,0,0,{num},0,10,10,0,255,255,255,255,1,0,0,0,0,0,0,0,0,0\r\n"
        ));
    }
    fs::write(root.join("Play").join("layout.csv"), layout).expect("write layout");

    let output = make_temp_root("dark-mode-lane-cover-numbers-output");
    generate(
        &root,
        &output,
        ConversionOptions {
            rules: vec!["lr2-dark-mode".to_owned()],
            ..ConversionOptions::default()
        },
    )
    .expect("generation should succeed");

    let dark2 = fs::read_to_string(output.join("Play").join("layout.csv.bmsg-dark2"))
        .expect("read DARK2 CSV");
    for num in [14, 15, 313, 314] {
        assert!(dark2.contains(&format!("#DST_NUMBER,0,0,{num},0,10,10,0,255,255,255,255,")));
    }
    assert!(dark2.contains("#DST_NUMBER,0,0,100,0,10,10,0,0,255,255,255,"));
}

#[test]
fn dark2_uses_the_exact_lr2_primary_timer_allowlist() {
    let kept = [2, 3, 42, 43, 44, 45, 48, 49, 50, 69, 70, 89, 140];
    let hidden = [0, 1, 4, 41, 46, 47, 90, 139, 141];
    let mut layout = String::new();
    for timer in kept.into_iter().chain(hidden) {
        layout.push_str(&format!("#SRC_IMAGE,0,0,0,0,1,1,1,1,0,{timer},0,0,0\r\n"));
        layout.push_str(&format!(
            "#DST_IMAGE,0,0,{timer},0,1,1,0,255,255,255,255,0,0,0,0,0,0,0,0,0\r\n"
        ));
    }

    let root = make_temp_root("dark-mode-timers-source");
    fs::create_dir_all(root.join("Play")).expect("create Play directory");
    fs::write(
        root.join("play.lr2skin"),
        "#INFORMATION,0,title,maker,\r\n#INCLUDE,Play\\layout.csv\r\n",
    )
    .expect("write header");
    fs::write(root.join("Play").join("layout.csv"), layout).expect("write layout");
    let output = make_temp_root("dark-mode-timers-output");
    generate(
        &root,
        &output,
        ConversionOptions {
            rules: vec!["lr2-dark-mode".to_owned()],
            ..ConversionOptions::default()
        },
    )
    .expect("generation should succeed");

    let dark2 = fs::read_to_string(output.join("Play").join("layout.csv.bmsg-dark2"))
        .expect("read DARK2 CSV");
    for timer in kept {
        assert!(dark2.contains(&format!("#SRC_IMAGE,0,0,0,0,1,1,1,1,0,{timer},")));
        assert!(dark2.contains(&format!("#DST_IMAGE,0,0,{timer},0,1,1,0,255,255,255,255,")));
    }
    for timer in hidden {
        assert!(dark2.contains(&format!("#SRC_IMAGE,0,0,0,0,1,1,1,1,0,{timer},")));
        assert!(dark2.contains(&format!("#DST_IMAGE,0,0,{timer},0,1,1,0,0,255,255,255,")));
    }
}

#[test]
fn dark_modes_keep_gauge_type_value_and_animation_images() {
    let root = make_temp_root("dark-mode-gauge-images-source");
    fs::create_dir_all(root.join("Play")).expect("create Play directory");
    fs::write(
        root.join("play.lr2skin"),
        "#INFORMATION,0,title,maker,\r\n#INCLUDE,Play\\layout.csv\r\n",
    )
    .expect("write header");
    fs::write(
        root.join("Play").join("layout.csv"),
        concat!(
            "#IMAGE,gauge.png\r\n",
            "#SRC_GROOVEGAUGE,0,0,0,0,4,4,4,1,0,0,1,0\r\n",
            "#DST_GROOVEGAUGE,0,0,0,0,100,10,0,255,255,255,255,1,0,0,0,0,0,0,0,0\r\n",
            "#SRC_IMAGE,0,0,0,0,1,1,1,1,0,0,0,0,0\r\n",
            "#DST_IMAGE,0,0,10,0,1,1,0,255,255,255,255,1,0,0,0,0,0,230,42,0\r\n",
            "#SRC_IMAGE,0,0,0,0,1,1,1,1,0,0,0,0,0\r\n",
            "#DST_IMAGE,0,0,20,0,1,1,0,255,255,255,255,1,0,0,0,0,0,-250,-63,0\r\n",
            "#SRC_IMAGE,0,0,0,0,1,1,1,1,0,44,0,0,0\r\n",
            "#DST_IMAGE,0,0,30,0,1,1,0,255,255,255,255,1,0,0,0,0,44,0,0,0,0\r\n",
            "#SRC_IMAGE,0,0,0,0,1,1,1,1,0,0,0,0,0\r\n",
            "#DST_IMAGE,0,0,40,0,1,1,0,255,255,255,255,1,0,0,0,0,0,0,0,0,0\r\n",
        ),
    )
    .expect("write layout");

    let output = make_temp_root("dark-mode-gauge-images-output");
    generate(
        &root,
        &output,
        ConversionOptions {
            rules: vec!["lr2-dark-mode".to_owned()],
            ..ConversionOptions::default()
        },
    )
    .expect("generation should succeed");

    for suffix in ["dark1", "dark2"] {
        let csv = fs::read_to_string(
            output
                .join("Play")
                .join(format!("layout.csv.bmsg-{suffix}")),
        )
        .expect("read generated CSV");
        for x in [10, 20, 30] {
            assert!(csv.contains(&format!("#DST_IMAGE,0,0,{x},0,1,1,0,255,255,255,255,")));
        }
        assert!(csv.contains("#DST_IMAGE,0,0,40,0,1,1,0,0,255,255,255,"));
    }
}

#[test]
fn dark_mode_preserves_lr2_flat_if_routes_and_resolves_stale_theme_names() {
    let root = make_temp_root("dark-mode-flat-if-source");
    fs::create_dir_all(root.join("Play")).expect("create Play directory");
    fs::write(
        root.join("play.lr2skin"),
        concat!(
            "#INFORMATION,0,title,maker,\r\n",
            "#IF,900\r\n",
            "#INCLUDE,LR2files\\Theme\\Old Folder Name\\Play\\left.csv\r\n",
            "#IF,901\r\n",
            "#INCLUDE,LR2files\\Theme\\Old Folder Name\\Play\\right.csv\r\n",
            "#ENDIF\r\n",
        ),
    )
    .expect("write flat IF header");
    for name in ["left.csv", "right.csv"] {
        fs::write(
            root.join("Play").join(name),
            "#SRC_IMAGE,0,0,0,0,1,1,1,1,0,0,0,0,0\r\n",
        )
        .expect("write routed layout");
    }

    let output = make_temp_root("dark-mode-flat-if-output");
    let report = generate(
        &root,
        &output,
        ConversionOptions {
            rules: vec!["lr2-dark-mode".to_owned()],
            ..ConversionOptions::default()
        },
    )
    .expect("generation should succeed");

    assert!(report.warnings.is_empty());
    assert!(output.join("Play").join("left.csv.bmsg-dark1").is_file());
    assert!(output.join("Play").join("right.csv.bmsg-dark2").is_file());
    let selector = selector_directory(&output);
    let dark = fs::read_to_string(selector.join("DARK")).expect("read DARK route");
    assert!(dark.contains("#IF,900\r\n"));
    assert!(dark.contains("#IF,901\r\n"));
    assert_eq!(dark.matches("#ENDIF").count(), 1);
    assert!(dark.contains("Play\\left.csv.bmsg-dark1"));
    assert!(dark.contains("Play\\right.csv.bmsg-dark1"));
}

#[test]
fn dark2_appends_a_black_image_for_a_lane_cover_shared_by_other_objects() {
    let root = make_temp_root("dark-mode-shared-slider-source");
    fs::create_dir_all(root.join("Play")).expect("create Play directory");
    fs::write(
        root.join("play.lr2skin"),
        "#INFORMATION,0,title,maker,\r\n#INCLUDE,Play\\layout.csv\r\n",
    )
    .expect("write header");
    fs::write(
        root.join("Play").join("layout.csv"),
        concat!(
            "#IMAGE,shared.png\r\n",
            "#SRC_IMAGE,0,0,0,0,1,1,1,1,0,2,0,0,0\r\n",
            "#DST_IMAGE,0,0,0,0,1,1,0,255,255,255,255,0,0,0,0,0,0,0,0,0\r\n",
            "#SRC_SLIDER,0,0,0,0,10,10,1,1,0,0,2,100,4,1\r\n",
            "#DST_SLIDER,0,0,10,20,100,200,0,128,255,255,255,2,0,0,0,0,0,0,0,0\r\n",
        ),
    )
    .expect("write shared-image layout");

    let output = make_temp_root("dark-mode-shared-slider-output");
    let report = generate(
        &root,
        &output,
        ConversionOptions {
            rules: vec!["lr2-dark-mode".to_owned()],
            ..ConversionOptions::default()
        },
    )
    .expect("generation should succeed");

    let dark2 = fs::read_to_string(output.join("Play").join("layout.csv.bmsg-dark2"))
        .expect("read DARK2 CSV");
    assert!(dark2.contains("#IMAGE,shared.png"));
    assert!(dark2.contains("\\_bms-skin-generator\\dark\\lane-cover-black.png,,,,,//1"));
    assert!(dark2.contains("#SRC_SLIDER,0,1,0,0,1,1,1,1,0,0,2,100,4,1"));
    assert!(dark2.contains("#DST_SLIDER,0,0,10,20,100,200,0,255,0,0,0,0,"));
    assert!(report.warnings.is_empty());
}

fn selector_directory(output: &Path) -> PathBuf {
    let root = output.join("_bms-skin-generator").join("dark");
    fs::read_dir(&root)
        .expect("read selector root")
        .map(|entry| entry.expect("read selector entry").path())
        .find(|path| path.is_dir())
        .expect("selector directory")
}
