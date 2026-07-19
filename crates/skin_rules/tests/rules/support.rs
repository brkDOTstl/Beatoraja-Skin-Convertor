use std::{
    fs,
    path::{Path, PathBuf},
};

pub(super) fn make_fixture(name: &str) -> PathBuf {
    let root = make_temp_root(name);
    populate_fixture(&root);
    root
}

pub(super) fn make_temp_root(name: &str) -> PathBuf {
    let mut root = std::env::temp_dir();
    root.push(format!("bms-skin-generator-{name}-{}", std::process::id()));
    if root.exists() {
        fs::remove_dir_all(&root).expect("remove old fixture");
    }
    root
}

pub(super) fn populate_fixture(root: &Path) {
    fs::create_dir_all(root.join(Path::new("Play").join("_csv"))).expect("create fixture");
    fs::write(
        root.join(Path::new("Play").join("_csv").join("play.csv")),
        b"#DST_JUDGELINE,0,0,10,100,96,4,0,255,255,255,255,1,0,0,0,0,0,0,0,0,0\r\n#SRC_NUMBER,0,9,0,0,319,22,11,1,0,0,14,0,3,,,,,,,\r\n#DST_NUMBER,0,0,10,20,96,22,0,255,255,255,255,1,0,0,0,0,0,0,0,0,0\r\n",
    )
    .expect("write fixture csv");
}

pub(super) fn write_oadx_layout(root: &Path) {
    fs::create_dir_all(root.join("_csv")).expect("create fixture");
    fs::write(
        root.join("_csv").join("layout.csv"),
        concat!(
            "#DST_JUDGELINE,0,0,25,315,144,6,0,255,255,255,255,1,0,0,0,0,0,0,0,0,0\r\n",
            "#SRC_SLIDER,0,5,0,0,144,321,1,1,0,0,0,0,4,321,,,,,,,\r\n",
            "#DST_SLIDER,0,0,25,0,256,356,0,255,255,255,255,1,0,0,0,0,0,0,0,0,0\r\n",
            "#SRC_NUMBER,0,9,0,0,70,12,10,1,0,0,14,0,3,,,,,,,\r\n",
            "#DST_NUMBER,0,0,88,100,10,12,0,255,255,255,255,1,0,0,0,0,0,0,0,0,0\r\n",
            "#DST_NUMBER,0,700,88,100,10,12,0,255,255,255,255,1,0,0,0,0,0,0,0,0,0\r\n",
            "#SRC_IMAGE,0,3,0,0,64,64,1,1,0,50,0,0,0,,,,,,,\r\n",
            "#DST_IMAGE,0,0,10,10,64,64,0,255,255,255,255,1,0,0,0,0,0,0,0,0,0\r\n",
            "#DST_IMAGE,0,100,10,10,64,64,0,255,255,255,255,1,0,0,0,0,0,0,0,0,0\r\n",
            "#SRC_IMAGE,0,3,777,30,30,30,1,1,0,100,0,0,0,,,,,,,\r\n",
            "#DST_IMAGE,0,0,20,20,30,30,0,255,255,255,255,1,0,0,0,0,0,0,0,0,0\r\n",
            "#SRC_IMAGE,0,3,0,0,30,256,1,1,0,130,0,0,0,,,,,,,\r\n",
            "#DST_IMAGE,0,0,30,30,30,321,0,255,255,255,255,1,0,0,0,0,0,0,0,0,0\r\n",
            "#DST_IMAGE,0,100,30,30,30,321,0,255,255,255,255,1,0,0,0,0,0,0,0,0,0\r\n",
            "#SRC_IMAGE,0,3,0,0,144,20,1,1,0,0,0,0,0,,,,,,,\r\n",
            "#DST_IMAGE,0,0,25,315,144,20,0,255,255,255,255,1,0,0,0,0,140,0,0,0,0\r\n",
            "#DST_IMAGE,0,700,25,315,144,20,0,255,255,255,255,1,0,0,0,0,0,0,0,0,0\r\n",
            "#DST_IMAGE,0,1000,25,315,144,20,0,255,255,255,255,1,0,0,0,0,0,0,0,0,0\r\n",
            "#DST_LINE,0,0,25,315,144,1,0,255,255,255,255,1,0,0,0,0,0,0,0,0,0\r\n",
        ),
    )
    .expect("write OADX fixture");
}
