use std::collections::{BTreeMap, BTreeSet};

use skin_core::{CsvRecord, SkinFileKind};

use crate::{
    edit::{
        DST_BLEND_FIELD, find_insert_after_command, insert_lines, is_play_file, replace_line,
        replace_record, warn,
    },
    model::{ConversionOptions, EditableFile, FileChange, RuleContext, RuleWarning},
    path_rewrite::rebase_theme_path_line,
    rules::geometry::next_destinations,
};

const MARK_RESOLUTION: &[u8] = b"//bms-skin-generator:R001:RESOLUTION";

pub(crate) fn apply_resolution(
    files: &mut [EditableFile],
    context: &RuleContext,
    options: &ConversionOptions,
    changes: &mut Vec<FileChange>,
    warnings: &mut Vec<RuleWarning>,
) {
    let requested_resolution = options.requested_resolution();
    let resolution = requested_resolution.or(context.detected_resolution);
    let Some(resolution) = resolution else {
        warn(
            warnings,
            "resolution-declaration",
            None,
            "Could not detect a resolution value.",
        );
        return;
    };
    for file in files
        .iter_mut()
        .filter(|file| file.kind == SkinFileKind::Lr2Skin)
    {
        let mut found = false;
        for line_index in 0..file.document.line_count() {
            let Some(content) = file.document.line_content(line_index).map(Vec::from) else {
                continue;
            };
            let mut record = CsvRecord::from_line(line_index, &content);
            if !record.is_command("#RESOLUTION") {
                continue;
            }
            found = true;
            if requested_resolution.is_some() && record.field_i32(1) != Some(i32::from(resolution))
            {
                record.set_field_ascii(1, &resolution.to_string());
                replace_record(
                    file,
                    line_index,
                    content,
                    record,
                    "resolution-declaration",
                    "set target resolution declaration",
                    changes,
                );
            }
        }
        if found {
            continue;
        }
        let index = find_insert_after_command(&file.document, "#INFORMATION").unwrap_or(0);
        insert_lines(
            file,
            index,
            [
                MARK_RESOLUTION.to_vec(),
                format!("#RESOLUTION,{resolution}").into_bytes(),
            ],
            "resolution-declaration",
            changes,
        );
    }
}

pub(crate) fn apply_path_rebase(
    files: &mut [EditableFile],
    context: &RuleContext,
    changes: &mut Vec<FileChange>,
    warnings: &mut Vec<RuleWarning>,
) {
    let (Some(source), Some(output)) = (
        context.source_theme_dir.as_deref(),
        context.output_theme_dir.as_deref(),
    ) else {
        warn(
            warnings,
            "theme-path-rebase",
            None,
            "Could not determine theme folder names.",
        );
        return;
    };
    if source.eq_ignore_ascii_case(output) {
        return;
    }
    for file in files {
        for index in 0..file.document.line_count() {
            let Some(content) = file.document.line_content(index).map(Vec::from) else {
                continue;
            };
            let Some(rebased) = rebase_theme_path_line(index, &content, source, output) else {
                continue;
            };
            replace_line(
                file,
                index,
                rebased.before,
                rebased.after,
                "theme-path-rebase",
                "rebase source theme path",
                changes,
            );
        }
    }
}

pub(crate) fn apply_blend(files: &mut [EditableFile], changes: &mut Vec<FileChange>) {
    replace_dst_field(
        files,
        "#DST_IMAGE",
        DST_BLEND_FIELD,
        "2",
        "blend-fallback",
        changes,
    );
    stabilize_option_labels(files, changes);
}

fn stabilize_option_labels(files: &mut [EditableFile], changes: &mut Vec<FileChange>) {
    for file in files.iter_mut().filter(|file| is_play_file(file)) {
        let edits = option_label_edits(file);
        for line_index in edits.persistent_destinations {
            let Some(before) = file.document.line_content(line_index).map(Vec::from) else {
                continue;
            };
            let mut record = CsvRecord::from_line(line_index, &before);
            record.set_field_ascii(2, "0");
            replace_record(
                file,
                line_index,
                before,
                record,
                "blend-fallback",
                "make option-label base frame persistent",
                changes,
            );
        }
        for line_index in edits.disabled_lines {
            let Some(before) = file.document.line_content(line_index).map(Vec::from) else {
                continue;
            };
            let mut after = b"//bms-skin-generator:R007:DISABLED_FADE,".to_vec();
            after.extend_from_slice(&before);
            replace_line(
                file,
                line_index,
                before,
                after,
                "blend-fallback",
                "disable duplicate option-label fade overlay",
                changes,
            );
        }
    }
}

#[derive(Default)]
struct OptionLabelEdits {
    persistent_destinations: BTreeSet<usize>,
    disabled_lines: BTreeSet<usize>,
}

fn option_label_edits(file: &EditableFile) -> OptionLabelEdits {
    let mut persistent_sources = BTreeMap::new();
    let mut output = OptionLabelEdits::default();
    for line_index in 0..file.document.line_count() {
        let Some(content) = file.document.line_content(line_index) else {
            continue;
        };
        let source = CsvRecord::from_line(line_index, content);
        let destination_command = if source.is_command("#SRC_BUTTON") {
            "#DST_BUTTON"
        } else if source.is_command("#SRC_IMAGE") {
            "#DST_IMAGE"
        } else {
            continue;
        };
        let destinations = next_destinations(&file.document, line_index, destination_command);
        if destinations.is_empty() {
            continue;
        }
        let signature = source.to_line();
        if is_persistent_label_group(&destinations) {
            persistent_sources.insert(signature, destinations[0].0);
        } else if let Some(persistent_destination) = persistent_sources.get(&signature)
            && is_fading_label_group(&destinations)
        {
            output
                .persistent_destinations
                .insert(*persistent_destination);
            output.disabled_lines.insert(line_index);
            output
                .disabled_lines
                .extend(destinations.into_iter().map(|(index, _)| index));
        }
    }
    output
}

fn is_persistent_label_group(destinations: &[(usize, CsvRecord)]) -> bool {
    destinations.len() == 1
        && destinations[0]
            .1
            .field_i32(2)
            .is_some_and(|time| time <= 50)
        && destinations[0].1.field_i32(8) == Some(255)
        && destinations[0].1.field_i32(12) == Some(1)
        && destinations[0].1.field_i32(16).unwrap_or(0) == 0
}

fn is_fading_label_group(destinations: &[(usize, CsvRecord)]) -> bool {
    let first = &destinations[0].1;
    first.field_i32(2).is_some_and(|time| time >= 1000)
        && first
            .field_i32(8)
            .is_some_and(|alpha| (1..255).contains(&alpha))
        && first.field_i32(12) == Some(2)
        && first.field_i32(16) == Some(-1)
        && destinations
            .iter()
            .skip(1)
            .any(|(_, destination)| destination.field_i32(8) == Some(0))
}

fn replace_dst_field(
    files: &mut [EditableFile],
    command: &str,
    field: usize,
    value: &str,
    rule_id: &str,
    changes: &mut Vec<FileChange>,
) {
    for file in files
        .iter_mut()
        .filter(|file| file.kind == SkinFileKind::Csv)
    {
        for index in 0..file.document.line_count() {
            let Some(content) = file.document.line_content(index).map(Vec::from) else {
                continue;
            };
            let mut record = CsvRecord::from_line(index, &content);
            let matches = if rule_id == "blend-fallback" {
                record.is_command(command) && record.field_equals_ascii(field, "5")
            } else {
                record.is_command(command) && !record.field_equals_ascii(field, value)
            };
            if matches {
                record.set_field_ascii(field, value);
                replace_record(
                    file,
                    index,
                    content,
                    record,
                    rule_id,
                    "update destination field",
                    changes,
                );
            }
        }
    }
}
