use std::collections::BTreeSet;

use skin_core::CsvRecord;

use crate::{
    edit::{replace_line, replace_record, warn},
    model::{EditableFile, FileChange, RuleContext, RuleWarning},
    rules::reference::reference_exists,
};

const REMOVED_FONT: &[u8] = b"//bms-skin-generator:R009:REMOVED_MISSING_UNUSED_FONT";

pub(crate) fn apply(
    files: &mut [EditableFile],
    context: &RuleContext,
    changes: &mut Vec<FileChange>,
    warnings: &mut Vec<RuleWarning>,
) {
    let Some(theme) = context.output_theme_dir.as_deref() else {
        warn(
            warnings,
            "asset-reference-repair",
            None,
            "Output theme folder is unknown.",
        );
        return;
    };
    for file in files {
        let used_fonts = used_font_indices(file);
        let mut font_index = 0;
        for line_index in 0..file.document.line_count() {
            let Some(content) = file.document.line_content(line_index).map(Vec::from) else {
                continue;
            };
            let mut record = CsvRecord::from_line(line_index, &content);
            if record.is_command("#INFORMATION") {
                clear_missing_thumbnail(
                    file,
                    line_index,
                    content,
                    &mut record,
                    context,
                    theme,
                    changes,
                );
                continue;
            }
            if record.is_command("#LR2FONT") {
                let request = FontRepair {
                    line_index,
                    content,
                    record: &record,
                    font_index,
                    used_fonts: &used_fonts,
                    context,
                    theme,
                };
                repair_font(file, request, changes, warnings);
                font_index += 1;
                continue;
            }
            if record.is_command("#IMAGE") {
                repair_movie_extension(
                    file,
                    line_index,
                    content,
                    &mut record,
                    context,
                    theme,
                    changes,
                );
            }
        }
    }
}

fn used_font_indices(file: &EditableFile) -> BTreeSet<i32> {
    (0..file.document.line_count())
        .filter_map(|index| {
            let content = file.document.line_content(index)?;
            let record = CsvRecord::from_line(index, content);
            record
                .is_command("#SRC_TEXT")
                .then(|| record.field_i32(2))
                .flatten()
        })
        .collect()
}

fn clear_missing_thumbnail(
    file: &mut EditableFile,
    line_index: usize,
    content: Vec<u8>,
    record: &mut CsvRecord,
    context: &RuleContext,
    theme: &str,
    changes: &mut Vec<FileChange>,
) {
    let Some(reference) = field_text(record, 4) else {
        return;
    };
    if reference.is_empty() || reference_exists(context, theme, &reference) {
        return;
    }
    record.set_field_ascii(4, "");
    replace_record(
        file,
        line_index,
        content,
        record.clone(),
        "asset-reference-repair",
        "clear missing local thumbnail",
        changes,
    );
}

struct FontRepair<'a> {
    line_index: usize,
    content: Vec<u8>,
    record: &'a CsvRecord,
    font_index: i32,
    used_fonts: &'a BTreeSet<i32>,
    context: &'a RuleContext,
    theme: &'a str,
}

fn repair_font(
    file: &mut EditableFile,
    request: FontRepair<'_>,
    changes: &mut Vec<FileChange>,
    warnings: &mut Vec<RuleWarning>,
) {
    let Some(reference) = field_text(request.record, 1) else {
        return;
    };
    if reference_exists(request.context, request.theme, &reference) {
        return;
    }
    if request.used_fonts.contains(&request.font_index) {
        warn(
            warnings,
            "asset-reference-repair",
            Some(file),
            format!(
                "Missing font index {} is used and was preserved: {reference}",
                request.font_index
            ),
        );
        return;
    }
    if request
        .used_fonts
        .iter()
        .any(|used| *used > request.font_index)
    {
        warn(
            warnings,
            "asset-reference-repair",
            Some(file),
            format!(
                "Missing font index {} was preserved to keep subsequent font indices stable: {reference}",
                request.font_index
            ),
        );
        return;
    }
    replace_line(
        file,
        request.line_index,
        request.content,
        format!(
            "{}:{}",
            String::from_utf8_lossy(REMOVED_FONT),
            request.font_index
        )
        .into_bytes(),
        "asset-reference-repair",
        "remove missing unused local font",
        changes,
    );
}

fn repair_movie_extension(
    file: &mut EditableFile,
    line_index: usize,
    content: Vec<u8>,
    record: &mut CsvRecord,
    context: &RuleContext,
    theme: &str,
    changes: &mut Vec<FileChange>,
) {
    let Some(reference) = field_text(record, 1) else {
        return;
    };
    if reference_exists(context, theme, &reference)
        || !reference.to_ascii_lowercase().ends_with(".png")
    {
        return;
    }
    let replacement = format!("{}.mpg", &reference[..reference.len().saturating_sub(4)]);
    if !reference_exists(context, theme, &replacement) {
        return;
    }
    record.set_field_ascii(1, &replacement);
    replace_record(
        file,
        line_index,
        content,
        record.clone(),
        "asset-reference-repair",
        "replace missing PNG reference with available MPG",
        changes,
    );
}

fn field_text(record: &CsvRecord, field: usize) -> Option<String> {
    record
        .field(field)
        .map(|value| String::from_utf8_lossy(value).trim().to_owned())
}
