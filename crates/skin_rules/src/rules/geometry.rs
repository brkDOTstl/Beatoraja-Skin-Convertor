use std::collections::BTreeMap;

use skin_core::{CsvRecord, TextDocument, records_for_document};

#[derive(Debug, Clone)]
pub(crate) struct JudgmentGeometry {
    pub side: i32,
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
}

impl JudgmentGeometry {
    pub fn lower_edge(&self) -> i32 {
        self.y.saturating_add(self.height.abs())
    }
}

pub(crate) fn judgment_geometries(document: &TextDocument) -> Vec<JudgmentGeometry> {
    let mut order = Vec::new();
    let mut by_side: BTreeMap<i32, JudgmentGeometry> = BTreeMap::new();
    for record in records_for_document(document) {
        if !record.is_command("#DST_JUDGELINE") {
            continue;
        }
        let (Some(side), Some(x), Some(y), Some(width), Some(height)) = (
            record.field_i32(1),
            record.field_i32(3),
            record.field_i32(4),
            record.field_i32(5),
            record.field_i32(6),
        ) else {
            continue;
        };
        let candidate = JudgmentGeometry {
            side,
            x,
            y,
            width,
            height: height.abs(),
        };
        if let Some(existing) = by_side.get_mut(&side) {
            if candidate.y > existing.y
                || (candidate.y == existing.y && candidate.height > existing.height)
            {
                *existing = candidate;
            }
        } else {
            order.push(side);
            by_side.insert(side, candidate);
        }
    }
    order
        .into_iter()
        .filter_map(|side| by_side.remove(&side))
        .collect()
}

pub(crate) fn next_destinations(
    document: &TextDocument,
    source_line: usize,
    command: &str,
) -> Vec<(usize, CsvRecord)> {
    let mut output = Vec::new();
    for index in (source_line + 1)..document.line_count() {
        let Some(content) = document.line_content(index) else {
            break;
        };
        let record = CsvRecord::from_line(index, content);
        if record
            .command()
            .is_some_and(|value| value.starts_with("#SRC_"))
        {
            break;
        }
        if record.is_command(command) {
            output.push((index, record));
        }
    }
    output
}
