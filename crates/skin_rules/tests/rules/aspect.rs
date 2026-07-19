use std::{
    fs,
    path::{Path, PathBuf},
};

use skin_rules::{AspectProtection, ConversionOptions, TargetAspect, generate};

use crate::support::make_temp_root;

#[test]
fn centered_four_three_scales_destination_edges_on_fhd_canvas() {
    let root = fixture(
        "aspect-source",
        2,
        b"#DST_IMAGE,0,0,0,0,1920,1080,0,255,255,255,255,1,0,0,0,0,0,0,0,0,0\r\n#DST_IMAGE,0,0,100,20,200,40,0,255,255,255,255,1,0,0,0,0,0,0,0,0,0\r\n",
    );
    let csv = convert(
        &root,
        "aspect-output",
        ConversionOptions {
            rules: vec!["aspect-ratio-convert".to_owned()],
            target_aspect: Some(TargetAspect::FourThree),
            ..ConversionOptions::default()
        },
    );

    assert!(csv.contains("#DST_IMAGE,0,0,240,0,1440,1080,"));
    assert!(csv.contains("#DST_IMAGE,0,0,315,20,150,40,"));
}

#[test]
fn sd_four_three_scales_to_fhd_sixteen_nine() {
    let root = fixture(
        "aspect-sd-source",
        0,
        b"#DST_IMAGE,0,0,100,20,200,40,0,255,255,255,255,1,0,0,0,0,0,0,0,0,0\r\n",
    );
    let output = make_temp_root("aspect-fhd-output");

    generate(
        &root,
        &output,
        ConversionOptions {
            rules: vec![
                "resolution-declaration".to_owned(),
                "aspect-ratio-convert".to_owned(),
            ],
            resolution: Some(2),
            target_aspect: Some(TargetAspect::SixteenNine),
            ..ConversionOptions::default()
        },
    )
    .expect("generation should succeed");

    let header = fs::read_to_string(output.join("play.lr2skin")).expect("read generated header");
    let csv = read_layout(&output);
    assert!(header.contains("#RESOLUTION,2"));
    assert!(csv.contains("#DST_IMAGE,0,0,300,45,600,90,"));
}

#[test]
fn sd_four_three_scales_to_centered_fhd_four_three() {
    let root = fixture(
        "aspect-sd-centered-source",
        0,
        b"#DST_IMAGE,0,0,100,20,200,40,0,255,255,255,255,1,0,0,0,0,0,0,0,0,0\r\n",
    );
    let csv = convert(
        &root,
        "aspect-fhd-centered-output",
        ConversionOptions {
            rules: vec![
                "resolution-declaration".to_owned(),
                "aspect-ratio-convert".to_owned(),
            ],
            resolution: Some(2),
            ..ConversionOptions::default()
        },
    );

    assert!(csv.contains("#DST_IMAGE,0,0,465,45,450,90,"));
}

#[test]
fn resolution_zero_compensates_beatoraja_stretch_for_four_three_output() {
    let root = fixture(
        "aspect-resolution-zero-four-three-source",
        0,
        b"#DST_IMAGE,0,0,0,0,640,480,0,255,255,255,255,1,0,0,0,0,0,0,0,0,0\r\n#DST_IMAGE,0,0,100,20,200,40,0,255,255,255,255,1,0,0,0,0,0,0,0,0,0\r\n",
    );
    let csv = convert(
        &root,
        "aspect-resolution-zero-four-three-output",
        ConversionOptions {
            target_aspect: Some(TargetAspect::FourThree),
            ..ConversionOptions::default()
        },
    );

    assert!(csv.contains("#DST_IMAGE,0,0,80,0,480,480,"));
    assert!(csv.contains("#DST_IMAGE,0,0,155,20,150,40,"));
}

#[test]
fn resolution_zero_uses_full_logical_canvas_for_sixteen_nine_output() {
    let root = fixture(
        "aspect-resolution-zero-sixteen-nine-source",
        0,
        b"#DST_IMAGE,0,0,0,0,640,480,0,255,255,255,255,1,0,0,0,0,0,0,0,0,0\r\n#DST_IMAGE,0,0,100,20,200,40,0,255,255,255,255,1,0,0,0,0,0,0,0,0,0\r\n",
    );
    let csv = convert(
        &root,
        "aspect-resolution-zero-sixteen-nine-output",
        ConversionOptions {
            target_aspect: Some(TargetAspect::SixteenNine),
            ..ConversionOptions::default()
        },
    );

    assert!(csv.contains("#DST_IMAGE,0,0,0,0,640,480,"));
    assert!(csv.contains("#DST_IMAGE,0,0,100,20,200,40,"));
}

#[test]
fn resolution_zero_same_aspect_ignores_protection_and_wide_conversion_preserves_it() {
    let layout = concat!(
        "#SRC_IMAGE,0,3,0,0,3600,240,18,0,300,50,0,0,0\r\n",
        "#DST_IMAGE,0,0,100,20,200,40,0,255,255,255,255,1,0,0,0,0,50,0,0,0\r\n",
        "#DST_NOWJUDGE_1P,0,0,100,20,200,40,0,255,255,255,255,1,0,0,0,0,46,0,0,0\r\n",
        "#DST_NOWCOMBO_1P,0,0,100,20,200,40,0,255,255,255,255,1,0,0,0,0,46,0,0,0\r\n",
    );
    let options = ConversionOptions {
        target_aspect: Some(TargetAspect::FourThree),
        aspect_protection: AspectProtection {
            bomb_effects: true,
            nowjudge: true,
            nowcombo: true,
        },
        ..ConversionOptions::default()
    };
    let four_three = convert(
        &fixture(
            "aspect-resolution-zero-protected-four-three-source",
            0,
            layout.as_bytes(),
        ),
        "aspect-resolution-zero-protected-four-three-output",
        options.clone(),
    );
    let sixteen_nine = convert(
        &fixture(
            "aspect-resolution-zero-protected-sixteen-nine-source",
            0,
            layout.as_bytes(),
        ),
        "aspect-resolution-zero-protected-sixteen-nine-output",
        ConversionOptions {
            target_aspect: Some(TargetAspect::SixteenNine),
            ..options
        },
    );

    assert!(four_three.contains("#DST_IMAGE,0,0,155,20,150,40,"));
    assert!(four_three.contains("#DST_NOWJUDGE_1P,0,0,155,20,150,40,"));
    assert!(four_three.contains("#DST_NOWCOMBO_1P,0,0,75,20,150,40,"));
    assert!(sixteen_nine.contains("#DST_IMAGE,0,0,125,20,150,40,"));
    assert!(sixteen_nine.contains("#DST_NOWJUDGE_1P,0,0,125,20,150,40,"));
    assert!(sixteen_nine.contains("#DST_NOWCOMBO_1P,0,0,75,20,150,40,"));
}

#[test]
fn slider_range_uses_the_movement_axis() {
    let root = fixture(
        "aspect-slider-axis-source",
        0,
        concat!(
            "#SRC_SLIDER,0,0,320,6,144,321,1,1,0,0,2,356,4,1\r\n",
            "#DST_SLIDER,0,0,44,-356,256,356,0,255,255,255,255,1,0,0,0,0,0,0,0,0,0\r\n",
            "#SRC_SLIDER,0,0,0,0,16,16,1,1,0,0,1,356,0,1\r\n",
            "#DST_SLIDER,0,0,0,0,16,16,0,255,255,255,255,1,0,0,0,0,0,0,0,0,0\r\n",
        )
        .as_bytes(),
    );
    let csv = convert(
        &root,
        "aspect-slider-axis-output",
        ConversionOptions {
            target_aspect: Some(TargetAspect::FourThree),
            ..ConversionOptions::default()
        },
    );

    assert!(csv.contains("#SRC_SLIDER,0,0,320,6,144,321,1,1,0,0,2,356,4,1"));
    assert!(csv.contains("#SRC_SLIDER,0,0,0,0,16,16,1,1,0,0,1,267,0,1"));
}

#[test]
fn selected_gameplay_objects_keep_aspect_during_wide_conversion() {
    let root = fixture(
        "aspect-protection-source",
        0,
        b"#SRC_IMAGE,0,3,0,0,3600,240,18,0,300,50,0,0,0\r\n#DST_IMAGE,0,0,100,20,200,40,0,255,255,255,255,1,0,0,0,0,50,0,0,0\r\n#SRC_NOTE,0,4,0,0,10,10,1,1,0,0,0,0,0\r\n#DST_IMAGE,0,0,100,20,200,40,0,255,255,255,255,1,0,0,0,0,0,0,0,0\r\n#DST_NOWJUDGE_1P,0,0,100,20,200,40,0,255,255,255,255,1,0,0,0,0,46,0,0,0\r\n#DST_NOWCOMBO_1P,0,0,100,20,200,40,0,255,255,255,255,1,0,0,0,0,46,0,0,0\r\n",
    );
    let csv = convert(
        &root,
        "aspect-protection-output",
        ConversionOptions {
            rules: vec!["aspect-ratio-convert".to_owned()],
            resolution: Some(2),
            target_aspect: Some(TargetAspect::SixteenNine),
            aspect_protection: AspectProtection {
                bomb_effects: true,
                nowjudge: true,
                nowcombo: true,
            },
            ..ConversionOptions::default()
        },
    );

    assert!(csv.contains("#DST_IMAGE,0,0,375,45,450,90,"));
    assert!(csv.contains("#DST_IMAGE,0,0,300,45,600,90,"));
    assert!(csv.contains("#DST_NOWJUDGE_1P,0,0,375,45,450,90,"));
    assert!(csv.contains("#DST_NOWCOMBO_1P,0,0,225,45,450,90,"));
}

#[test]
fn protected_nowcombo_keeps_its_position_relative_to_nowjudge() {
    let root = fixture(
        "aspect-relative-combo-source",
        2,
        concat!(
            "#DST_NOWJUDGE_1P,0,0,400,300,200,40,0,255,255,255,255,1,0,0,0,0,46,0,0,0\r\n",
            "#DST_NOWCOMBO_1P,0,0,210,0,30,40,0,255,255,255,255,1,0,0,0,0,46,0,0,0\r\n",
        )
        .as_bytes(),
    );
    let csv = convert(
        &root,
        "aspect-relative-combo-output",
        ConversionOptions {
            target_aspect: Some(TargetAspect::FourThree),
            aspect_protection: AspectProtection {
                nowjudge: true,
                nowcombo: true,
                ..AspectProtection::default()
            },
            ..ConversionOptions::default()
        },
    );

    assert!(csv.contains("#DST_NOWJUDGE_1P,0,0,515,300,200,40,"));
    assert!(csv.contains("#DST_NOWCOMBO_1P,0,0,210,0,30,40,"));
}

#[test]
fn aspect_protection_categories_are_independent() {
    let root = fixture(
        "aspect-independent-source",
        0,
        b"#SRC_IMAGE,0,3,0,0,3600,240,18,0,300,50,0,0,0\r\n#DST_IMAGE,0,0,100,20,200,40,0,255,255,255,255,1,0,0,0,0,50,0,0,0\r\n#SRC_NOWJUDGE_1P,0,23,0,0,200,40,1,1,0,46,0,0,0\r\n#DST_NOWJUDGE_1P,0,0,100,20,200,40,0,255,255,255,255,1,0,0,0,0,46,0,0,0\r\n#DST_NOWCOMBO_1P,0,0,100,20,200,40,0,255,255,255,255,1,0,0,0,0,46,0,0,0\r\n",
    );
    let csv = convert(
        &root,
        "aspect-independent-output",
        ConversionOptions {
            rules: vec!["aspect-ratio-convert".to_owned()],
            resolution: Some(2),
            target_aspect: Some(TargetAspect::SixteenNine),
            aspect_protection: AspectProtection {
                nowjudge: true,
                ..AspectProtection::default()
            },
            ..ConversionOptions::default()
        },
    );

    assert!(csv.contains("#DST_IMAGE,0,0,300,45,600,90,"));
    assert!(csv.contains("#DST_NOWJUDGE_1P,0,0,375,45,450,90,"));
    assert!(csv.contains("#DST_NOWCOMBO_1P,0,0,300,45,600,90,"));
}

fn fixture(name: &str, resolution: u8, layout: &[u8]) -> PathBuf {
    let root = make_temp_root(name);
    fs::create_dir_all(root.join("Play")).expect("create fixture");
    fs::write(
        root.join("play.lr2skin"),
        format!("#INFORMATION,0,title,maker,\r\n#RESOLUTION,{resolution}\r\n"),
    )
    .expect("write skin header");
    fs::write(root.join("Play").join("layout.csv"), layout).expect("write layout");
    root
}

fn convert(root: &Path, output_name: &str, options: ConversionOptions) -> String {
    let output = make_temp_root(output_name);
    generate(root, &output, options).expect("generation should succeed");
    read_layout(&output)
}

fn read_layout(root: &Path) -> String {
    fs::read_to_string(root.join("Play").join("layout.csv")).expect("read transformed layout")
}
