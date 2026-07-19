use super::*;

#[test]
fn repair_missing_local_references_handles_oadx_mistakes() {
    let parent = make_temp_root("asset-repair-source");
    let source = parent.join("OA_DX_03");
    fs::create_dir_all(source.join("loadmovie")).expect("create movie folder");
    fs::write(source.join("loadmovie").join("clip.mpg"), b"movie").expect("write movie");
    for index in [0, 1, 3, 4] {
        let directory = source.join("font").join(format!("{index:02}"));
        fs::create_dir_all(&directory).expect("create font folder");
        fs::write(directory.join("font.lr2font"), b"font").expect("write font");
    }
    fs::write(
        source.join("play.lr2skin"),
        b"#INFORMATION,0,title,maker,LR2files\\Theme\\OA_DX_03\\.png\r\n",
    )
    .expect("write header");
    fs::write(
        source.join("layout.csv"),
        concat!(
            "#LR2FONT,LR2files\\Theme\\OA_DX_03\\font\\00\\font.lr2font\r\n",
            "#LR2FONT,LR2files\\Theme\\OA_DX_03\\font\\01\\font.lr2font\r\n",
            "#LR2FONT,LR2files\\Theme\\OA_DX_03\\font\\02\\font.lr2font\r\n",
            "#LR2FONT,LR2files\\Theme\\OA_DX_03\\font\\03\\font.lr2font\r\n",
            "#LR2FONT,LR2files\\Theme\\OA_DX_03\\font\\04\\font.lr2font\r\n",
            "#LR2FONT,LR2files\\Theme\\OA_DX_03\\font\\05\\font.lr2font\r\n",
            "#IMAGE,LR2files\\Theme\\OA_DX_03\\loadmovie\\*.png,//20\r\n",
            "#SRC_TEXT,0,4,15,1,0\r\n",
        ),
    )
    .expect("write layout");
    let output = make_temp_root("asset-repair-output").join("OADX3");

    let report = generate(
        &source,
        &output,
        ConversionOptions {
            rules: vec![
                "theme-path-rebase".to_owned(),
                "asset-reference-repair".to_owned(),
            ],
            ..ConversionOptions::default()
        },
    )
    .expect("generation should succeed");

    let header = fs::read_to_string(output.join("play.lr2skin")).expect("read header");
    let csv = fs::read_to_string(output.join("layout.csv")).expect("read layout");
    assert!(header.contains("#INFORMATION,0,title,maker,"));
    assert!(!header.contains(".png"));
    assert!(csv.contains("font\\02\\font.lr2font"));
    assert!(report.warnings.iter().any(|warning| {
        warning
            .message
            .contains("preserved to keep subsequent font indices stable")
    }));
    assert!(csv.contains("R009:REMOVED_MISSING_UNUSED_FONT"));
    assert!(!csv.contains("font\\05\\font.lr2font"));
    assert!(csv.contains("loadmovie\\*.mpg"));
}
