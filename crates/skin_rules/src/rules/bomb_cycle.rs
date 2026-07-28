use skin_core::{CsvRecord, SkinFileKind};

use crate::{
    edit::replace_record,
    model::{EditableFile, FileChange},
};

const RULE_ID: &str = "lr2-bomb-cycle-boundary-compat";
const SRC_CYCLE_FIELD: usize = 9;
const SRC_TIMER_FIELD: usize = 10;
const DST_TIME_FIELD: usize = 2;
const DST_LOOP_FIELD: usize = 16;

#[derive(Debug)]
struct BombGroup {
    cycle: i32,
    active_loop: i32,
    destinations: Vec<(usize, i32)>,
}

pub(crate) fn apply(files: &mut [EditableFile], changes: &mut Vec<FileChange>) {
    for file in files
        .iter_mut()
        .filter(|file| file.kind == SkinFileKind::Csv)
    {
        let mut active = None;
        let mut fixes = Vec::new();

        for line_index in 0..file.document.line_count() {
            let Some(content) = file.document.line_content(line_index) else {
                continue;
            };
            let record = CsvRecord::from_line(line_index, content);
            let Some(command) = record.command() else {
                continue;
            };

            if command.starts_with("#SRC_") {
                collect_boundary_fixes(active.take(), &mut fixes);
                active = bomb_group(&record);
                continue;
            }

            let Some(group) = active.as_mut() else {
                continue;
            };
            if !record.is_command("#DST_IMAGE") {
                continue;
            }

            if group.active_loop == 0
                && let Some(loop_time) = record.field_i32(DST_LOOP_FIELD)
            {
                group.active_loop = loop_time;
            }
            if let Some(time) = record.field_i32(DST_TIME_FIELD) {
                group.destinations.push((line_index, time));
            }
        }
        collect_boundary_fixes(active, &mut fixes);

        for (line_index, end_time) in fixes {
            let Some(before) = file.document.line_content(line_index).map(Vec::from) else {
                continue;
            };
            let mut record = CsvRecord::from_line(line_index, &before);
            record.set_field_ascii(DST_TIME_FIELD, &end_time.to_string());
            replace_record(
                file,
                line_index,
                before,
                record,
                RULE_ID,
                "end one-shot LR2 bomb before beatoraja wraps to frame zero",
                changes,
            );
        }
    }
}

fn bomb_group(source: &CsvRecord) -> Option<BombGroup> {
    if !source.is_command("#SRC_IMAGE") {
        return None;
    }
    let timer = source.field_i32(SRC_TIMER_FIELD)?;
    let cycle = source.field_i32(SRC_CYCLE_FIELD)?;
    ((50..=69).contains(&timer) && cycle > 1).then_some(BombGroup {
        cycle,
        active_loop: 0,
        destinations: Vec::new(),
    })
}

fn collect_boundary_fixes(group: Option<BombGroup>, fixes: &mut Vec<(usize, i32)>) {
    let Some(group) = group else {
        return;
    };
    if group.active_loop != -1
        || group.destinations.iter().map(|(_, time)| *time).max() != Some(group.cycle)
    {
        return;
    }

    fixes.extend(
        group
            .destinations
            .into_iter()
            .filter(|(_, time)| *time == group.cycle)
            .map(|(line_index, _)| (line_index, group.cycle - 1)),
    );
}
