use std::ops::Range;

use skin_core::{CsvRecord, FollowLiftKind, Line, TextDocument, follow_lift_kind};

use crate::{
    edit::{is_play_file, warn},
    model::{EditableFile, FileChange, RuleWarning},
};

const RULE_ID: &str = "key-laser-layer-compat";

#[derive(Debug, Clone, Copy)]
struct ConditionalBlock {
    start: usize,
    end: usize,
}

pub(crate) fn apply(
    files: &mut [EditableFile],
    changes: &mut Vec<FileChange>,
    warnings: &mut Vec<RuleWarning>,
) {
    for file in files.iter_mut().filter(|file| is_play_file(file)) {
        apply_to_file(file, changes, warnings);
    }
}

fn apply_to_file(
    file: &mut EditableFile,
    changes: &mut Vec<FileChange>,
    warnings: &mut Vec<RuleWarning>,
) {
    let records = (0..file.document.line_count())
        .filter_map(|index| {
            file.document
                .line_content(index)
                .map(|content| CsvRecord::from_line(index, content))
        })
        .collect::<Vec<_>>();
    let Some(first_note_destination) = records
        .iter()
        .position(|record| record.is_command("#DST_NOTE"))
    else {
        return;
    };

    let (conditional_blocks, depth_before, malformed_conditions) =
        parse_conditional_blocks(&records);
    let candidates = records
        .iter()
        .enumerate()
        .filter_map(|(index, record)| {
            (index > first_note_destination
                && follow_lift_kind(record) == Some(FollowLiftKind::KeyLaser))
            .then_some(index)
        })
        .collect::<Vec<_>>();
    if candidates.is_empty() {
        return;
    }

    if malformed_conditions {
        warn(
            warnings,
            RULE_ID,
            Some(file),
            "Skipped key-laser layering because the CSV has unbalanced #IF/#ENDIF directives.",
        );
        return;
    }

    let insertion_index = conditional_blocks
        .iter()
        .filter(|block| block.start < first_note_destination && first_note_destination < block.end)
        .map(|block| block.start)
        .min()
        .unwrap_or(first_note_destination);

    let mut ranges = Vec::<Range<usize>>::new();
    let mut skipped = Vec::<usize>::new();
    for candidate in candidates {
        if ranges
            .iter()
            .any(|range| range.start <= candidate && candidate < range.end)
        {
            continue;
        }

        if depth_before[candidate] == 0 {
            if let Some(range) = unguarded_laser_group(&records, candidate) {
                ranges.push(range);
            } else {
                skipped.push(candidate + 1);
            }
            continue;
        }

        let outermost_block = conditional_blocks
            .iter()
            .filter(|block| block.start < candidate && candidate < block.end)
            .min_by_key(|block| block.start);
        if let Some(block) = outermost_block.filter(|block| {
            block.start > first_note_destination
                && block_contains_only_key_lasers(&records, **block)
        }) {
            ranges.push(block.start..block.end + 1);
        } else {
            skipped.push(candidate + 1);
        }
    }

    let ranges = normalize_ranges(ranges, &records);
    if !skipped.is_empty() {
        warn(
            warnings,
            RULE_ID,
            Some(file),
            format!(
                "Skipped key-laser groups in mixed or unsupported conditional scopes at lines {}.",
                skipped
                    .iter()
                    .map(usize::to_string)
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
        );
    }
    if ranges.is_empty() {
        return;
    }

    relocate_ranges(
        file,
        &ranges,
        insertion_index,
        first_note_destination,
        changes,
    );
}

fn parse_conditional_blocks(records: &[CsvRecord]) -> (Vec<ConditionalBlock>, Vec<usize>, bool) {
    let mut blocks = Vec::new();
    let mut stack = Vec::new();
    let mut depth_before = Vec::with_capacity(records.len());
    let mut malformed = false;

    for (index, record) in records.iter().enumerate() {
        depth_before.push(stack.len());
        if record.is_command("#IF") {
            stack.push(index);
        } else if record.is_command("#ENDIF") {
            if let Some(start) = stack.pop() {
                blocks.push(ConditionalBlock { start, end: index });
            } else {
                malformed = true;
            }
        } else if (record.is_command("#ELSEIF") || record.is_command("#ELSE")) && stack.is_empty() {
            malformed = true;
        }
    }
    malformed |= !stack.is_empty();
    (blocks, depth_before, malformed)
}

fn block_contains_only_key_lasers(records: &[CsvRecord], block: ConditionalBlock) -> bool {
    let mut found_laser = false;
    for record in &records[block.start + 1..block.end] {
        let Some(command) = record.command() else {
            continue;
        };
        if !command.starts_with("#SRC_") {
            continue;
        }
        if follow_lift_kind(record) != Some(FollowLiftKind::KeyLaser) {
            return false;
        }
        found_laser = true;
    }
    found_laser
}

fn unguarded_laser_group(records: &[CsvRecord], source_index: usize) -> Option<Range<usize>> {
    let mut last_destination = None;
    for (index, record) in records.iter().enumerate().skip(source_index + 1) {
        let Some(command) = record.command() else {
            continue;
        };
        if command.starts_with("#SRC_") || is_conditional_command(&command) {
            break;
        }
        if command == "#DST_IMAGE" {
            last_destination = Some(index);
        } else {
            return None;
        }
    }
    last_destination.map(|end| source_index..end + 1)
}

fn is_conditional_command(command: &str) -> bool {
    matches!(command, "#IF" | "#ELSEIF" | "#ELSE" | "#ENDIF")
}

fn normalize_ranges(mut ranges: Vec<Range<usize>>, records: &[CsvRecord]) -> Vec<Range<usize>> {
    ranges.sort_by_key(|range| (range.start, range.end));
    let mut normalized = Vec::<Range<usize>>::new();
    for range in ranges {
        if let Some(previous) = normalized.last_mut() {
            let comments_only_gap = range.start >= previous.end
                && records[previous.end..range.start]
                    .iter()
                    .all(|record| record.command().is_none());
            if range.start <= previous.end || comments_only_gap {
                previous.end = previous.end.max(range.end);
                continue;
            }
        }
        normalized.push(range);
    }
    normalized
}

fn relocate_ranges(
    file: &mut EditableFile,
    ranges: &[Range<usize>],
    insertion_index: usize,
    first_note_destination: usize,
    changes: &mut Vec<FileChange>,
) {
    let original = file.document.lines().to_vec();
    let mut moved = vec![false; original.len()];
    for range in ranges {
        moved[range.clone()].fill(true);
        let first_command = range
            .clone()
            .find_map(|index| {
                CsvRecord::from_line(index, &original[index].content)
                    .command()
                    .map(|_| index)
            })
            .unwrap_or(range.start);
        changes.push(FileChange {
            rule_id: RULE_ID.to_owned(),
            path: file.relative_path.clone(),
            line: Some(first_command + 1),
            message: "register key-laser objects before the note renderer".to_owned(),
            before: String::from_utf8_lossy(&original[first_command].content).into_owned(),
            after: format!(
                "moved before original #DST_NOTE at line {}",
                first_note_destination + 1
            ),
        });
    }

    let mut reordered = Vec::<Line>::with_capacity(original.len());
    reordered.extend(
        original[..insertion_index]
            .iter()
            .enumerate()
            .filter(|(index, _)| !moved[*index])
            .map(|(_, line)| line.clone()),
    );
    for range in ranges {
        reordered.extend(original[range.clone()].iter().cloned());
    }
    reordered.extend(
        original[insertion_index..]
            .iter()
            .enumerate()
            .filter(|(offset, _)| !moved[insertion_index + *offset])
            .map(|(_, line)| line.clone()),
    );

    let mut bytes = Vec::new();
    for line in reordered {
        bytes.extend_from_slice(&line.content);
        if let Some(ending) = line.ending {
            bytes.extend_from_slice(ending.as_bytes());
        }
    }
    file.document = TextDocument::from_bytes(file.document.path().clone(), bytes);
    file.dirty = true;
}
