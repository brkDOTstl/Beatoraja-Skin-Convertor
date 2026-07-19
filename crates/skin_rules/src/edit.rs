use skin_core::{CsvRecord, SkinFileKind, TextDocument, analyze::is_play_document};

use crate::model::{EditableFile, FileChange, RuleWarning};

pub(crate) const DST_BLEND_FIELD: usize = 12;
pub(crate) const DST_OP1_FIELD: usize = 18;
pub(crate) const DST_OP2_FIELD: usize = 19;
pub(crate) const DST_OP3_FIELD: usize = 20;
pub(crate) const DST_OP4_FIELD: usize = 21;

pub(crate) fn is_play_file(file: &EditableFile) -> bool {
    file.kind == SkinFileKind::Csv && is_play_document(&file.relative_path, &file.document)
}

pub(crate) fn document_has_command(document: &TextDocument, command: &str) -> bool {
    (0..document.line_count()).any(|index| {
        document
            .line_content(index)
            .is_some_and(|content| CsvRecord::from_line(index, content).is_command(command))
    })
}

pub(crate) fn document_has_marker(document: &TextDocument, marker: &[u8]) -> bool {
    document.lines().iter().any(|line| {
        line.content
            .windows(marker.len())
            .any(|window| window == marker)
    })
}

pub(crate) fn find_last_command(document: &TextDocument, command: &str) -> Option<usize> {
    (0..document.line_count()).rev().find(|index| {
        document
            .line_content(*index)
            .is_some_and(|content| CsvRecord::from_line(*index, content).is_command(command))
    })
}

pub(crate) fn find_insert_after_command(document: &TextDocument, command: &str) -> Option<usize> {
    (0..document.line_count()).find_map(|index| {
        let content = document.line_content(index)?;
        CsvRecord::from_line(index, content)
            .is_command(command)
            .then_some(index + 1)
    })
}

pub(crate) fn replace_record(
    file: &mut EditableFile,
    line_index: usize,
    before: Vec<u8>,
    record: CsvRecord,
    rule_id: &str,
    message: &str,
    changes: &mut Vec<FileChange>,
) {
    replace_line(
        file,
        line_index,
        before,
        record.to_line(),
        rule_id,
        message,
        changes,
    );
}

pub(crate) fn replace_line(
    file: &mut EditableFile,
    line_index: usize,
    before: Vec<u8>,
    after: Vec<u8>,
    rule_id: &str,
    message: &str,
    changes: &mut Vec<FileChange>,
) {
    if before == after {
        return;
    }
    let _ = file
        .document
        .replace_line_content(line_index, after.clone());
    file.dirty = true;
    changes.push(FileChange {
        rule_id: rule_id.to_owned(),
        path: file.relative_path.clone(),
        line: Some(line_index + 1),
        message: message.to_owned(),
        before: String::from_utf8_lossy(&before).into_owned(),
        after: String::from_utf8_lossy(&after).into_owned(),
    });
}

pub(crate) fn insert_lines(
    file: &mut EditableFile,
    line_index: usize,
    lines: impl IntoIterator<Item = Vec<u8>>,
    rule_id: &str,
    changes: &mut Vec<FileChange>,
) {
    for (offset, content) in lines.into_iter().enumerate() {
        let index = line_index + offset;
        file.document.insert_generated_line(index, content.clone());
        changes.push(FileChange {
            rule_id: rule_id.to_owned(),
            path: file.relative_path.clone(),
            line: Some(index + 1),
            message: "insert generated line".to_owned(),
            before: String::new(),
            after: String::from_utf8_lossy(&content).into_owned(),
        });
    }
    file.dirty = true;
}

pub(crate) fn warn(
    warnings: &mut Vec<RuleWarning>,
    rule_id: &str,
    file: Option<&EditableFile>,
    message: impl Into<String>,
) {
    warnings.push(RuleWarning {
        rule_id: rule_id.to_owned(),
        path: file.map(|item| item.relative_path.clone()),
        message: message.into(),
    });
}
