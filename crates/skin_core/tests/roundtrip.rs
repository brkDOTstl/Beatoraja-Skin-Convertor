use std::path::PathBuf;

use skin_core::{TextDocument, records_for_document};

#[test]
fn roundtrips_crlf_and_trailing_commas() {
    let input = b"#IMAGE,foo,,\r\n#DST_IMAGE,1,50,10,20,,,,,,,,,,,,,,,,,\r\n".to_vec();
    let document = TextDocument::from_bytes(PathBuf::from("sample.csv"), input.clone());
    assert_eq!(document.to_bytes(), input);

    let records = records_for_document(&document);
    assert_eq!(records[0].fields.len(), 4);
    assert_eq!(records[1].field_i32(2), Some(50));
}

#[test]
fn roundtrips_mixed_invalid_bytes_without_utf8_conversion() {
    let input = vec![0xff, b',', b'a', b'\n', b'#', b'E', b'N', b'D'];
    let document = TextDocument::from_bytes(PathBuf::from("bad.csv"), input.clone());
    assert_eq!(document.to_bytes(), input);
}

#[test]
fn appending_generated_lines_separates_an_unterminated_last_line() {
    let mut document = TextDocument::from_bytes(
        PathBuf::from("unterminated.csv"),
        b"#IF,900\r\n#ENDIF".to_vec(),
    );

    document.push_generated_line(b"#INCLUDE,route".to_vec());

    assert_eq!(
        document.to_bytes(),
        b"#IF,900\r\n#ENDIF\r\n#INCLUDE,route\r\n"
    );
}
