use skin_core::CsvRecord;

use crate::{
    edit::{
        DST_OP1_FIELD, DST_OP2_FIELD, DST_OP3_FIELD, DST_OP4_FIELD, document_has_marker,
        insert_lines, is_play_file, warn,
    },
    model::{EditableFile, FileChange, RuleContext, RuleWarning},
    rules::geometry::{JudgmentGeometry, judgment_geometries, next_destinations},
};

const SRC_NUM_FIELD: usize = 11;
const SRC_KETA_FIELD: usize = 13;
const MARK_SUDDEN_FALLBACK: &[u8] = b"//bms-skin-generator:R008:SUDDEN_NUMBER:FALLBACK";
const MARK_GREEN: &[u8] = b"//bms-skin-generator:R005:GREEN_NUMBER";
const MARK_LIFT: &[u8] = b"//bms-skin-generator:R004:LIFT_NUMBER";

#[derive(Debug, Clone, Copy)]
pub(crate) enum NumberKind {
    Green,
    Lift,
}

#[derive(Debug, Clone)]
struct NumberAnchor {
    source: CsvRecord,
    destinations: Vec<(usize, CsvRecord)>,
    insert_at: usize,
    fallback: bool,
}

pub(crate) fn apply(
    files: &mut [EditableFile],
    context: &RuleContext,
    kind: NumberKind,
    changes: &mut Vec<FileChange>,
    warnings: &mut Vec<RuleWarning>,
) {
    let (rule_id, marker) = match kind {
        NumberKind::Green => ("green-number", MARK_GREEN),
        NumberKind::Lift => ("lift-number", MARK_LIFT),
    };
    let mut applied = false;
    for file in files.iter_mut().filter(|file| is_play_file(file)) {
        if document_has_marker(&file.document, marker) || has_target_number(file, kind) {
            continue;
        }
        let judgments = judgment_geometries(&file.document);
        let mut anchors = number_anchors(file);
        if anchors.is_empty() {
            if let Some(anchor) = fallback_number_anchor(file) {
                anchors = vec![anchor; judgments.len()];
            }
        }
        if judgments.is_empty() || judgments.len() != anchors.len() {
            warn(
                warnings,
                rule_id,
                Some(file),
                format!(
                    "Judgment/number count mismatch: {} judgment sides, {} num=14/15 anchors.",
                    judgments.len(),
                    anchors.len()
                ),
            );
            continue;
        }
        let mut insertions = judgments
            .iter()
            .zip(&anchors)
            .map(|(judgment, anchor)| {
                (
                    anchor.insert_at,
                    build_lines(anchor, judgment, context, kind),
                )
            })
            .collect::<Vec<_>>();
        insertions.sort_by_key(|(index, _)| std::cmp::Reverse(*index));
        for (index, lines) in insertions {
            insert_lines(file, index, lines, rule_id, changes);
        }
        applied = true;
    }
    if !applied {
        warn(
            warnings,
            rule_id,
            None,
            "No matched lane-cover number object group was available.",
        );
    }
}

fn number_anchors(file: &EditableFile) -> Vec<NumberAnchor> {
    let mut output = Vec::new();
    for line_index in 0..file.document.line_count() {
        let Some(content) = file.document.line_content(line_index) else {
            continue;
        };
        let source = CsvRecord::from_line(line_index, content);
        if !source.is_command("#SRC_NUMBER")
            || !source
                .field_i32(SRC_NUM_FIELD)
                .is_some_and(|num| num == 14 || num == 15)
            || line_index.checked_sub(1).is_some_and(|previous| {
                file.document.line_content(previous).is_some_and(|line| {
                    line.windows(MARK_SUDDEN_FALLBACK.len())
                        .any(|window| window == MARK_SUDDEN_FALLBACK)
                })
            })
        {
            continue;
        }
        let destinations = next_destinations(&file.document, line_index, "#DST_NUMBER");
        if !destinations.is_empty() {
            let insert_at = destinations
                .last()
                .map_or(line_index + 1, |item| item.0 + 1);
            output.push(NumberAnchor {
                source,
                destinations,
                insert_at,
                fallback: false,
            });
        }
    }
    output
}

fn fallback_number_anchor(file: &EditableFile) -> Option<NumberAnchor> {
    for line_index in 0..file.document.line_count() {
        let content = file.document.line_content(line_index)?;
        let source = CsvRecord::from_line(line_index, content);
        if !source.is_command("#SRC_NUMBER")
            || source
                .field_i32(SRC_NUM_FIELD)
                .is_some_and(|num| num == 313 || num == 314)
            || !source.field_i32(5).is_some_and(|value| value > 0)
            || !source.field_i32(6).is_some_and(|value| value > 0)
            || !source.field_i32(7).is_some_and(|value| value >= 10)
            || !source.field_i32(8).is_some_and(|value| value >= 1)
        {
            continue;
        }
        let Some(destination) = next_destinations(&file.document, line_index, "#DST_NUMBER")
            .into_iter()
            .find(|(_, record)| {
                record.field_i32(5).is_some_and(|value| value != 0)
                    && record.field_i32(6).is_some_and(|value| value != 0)
            })
        else {
            continue;
        };
        return Some(NumberAnchor {
            source,
            destinations: vec![destination],
            insert_at: file.document.line_count(),
            fallback: true,
        });
    }
    None
}

fn build_lines(
    anchor: &NumberAnchor,
    judgment: &JudgmentGeometry,
    context: &RuleContext,
    kind: NumberKind,
) -> Vec<Vec<u8>> {
    let mut lines = if anchor.fallback && matches!(kind, NumberKind::Green) {
        build_sudden_fallback_lines(anchor, judgment)
    } else {
        Vec::new()
    };
    let mut source = anchor.source.clone();
    let (marker, num, keta) = match kind {
        NumberKind::Green => (MARK_GREEN, "313", 3),
        NumberKind::Lift => (MARK_LIFT, "314", 3),
    };
    source.set_field_ascii(1, &judgment.side.to_string());
    source.set_field_ascii(SRC_NUM_FIELD, num);
    source.set_field_ascii(SRC_KETA_FIELD, &keta.to_string());
    let marker = if anchor.fallback {
        let mut line = marker.to_vec();
        line.extend_from_slice(b":FALLBACK");
        line
    } else {
        marker.to_vec()
    };
    lines.extend([marker, source.to_line()]);
    for (_, destination) in &anchor.destinations {
        let mut generated = destination.clone();
        style_destination(
            &mut generated,
            judgment,
            context,
            kind,
            anchor.fallback,
            keta,
        );
        lines.push(generated.to_line());
    }
    lines
}

fn build_sudden_fallback_lines(anchor: &NumberAnchor, judgment: &JudgmentGeometry) -> Vec<Vec<u8>> {
    let mut source = anchor.source.clone();
    source.set_field_ascii(1, &judgment.side.to_string());
    source.set_field_ascii(SRC_NUM_FIELD, if judgment.side == 0 { "14" } else { "15" });
    source.set_field_ascii(SRC_KETA_FIELD, "3");
    let mut lines = vec![MARK_SUDDEN_FALLBACK.to_vec(), source.to_line()];
    for (_, destination) in &anchor.destinations {
        let mut generated = destination.clone();
        style_fallback_base(&mut generated, judgment, 3);
        generated.set_field_ascii(4, "9");
        generated.set_field_ascii(9, "255");
        generated.set_field_ascii(10, "255");
        generated.set_field_ascii(11, "255");
        generated.set_field_ascii(DST_OP1_FIELD, "270");
        generated.set_field_ascii(DST_OP2_FIELD, "0");
        generated.set_field_ascii(DST_OP3_FIELD, "0");
        generated.set_field_ascii(DST_OP4_FIELD, "0");
        lines.push(generated.to_line());
    }
    lines
}

fn style_destination(
    destination: &mut CsvRecord,
    judgment: &JudgmentGeometry,
    context: &RuleContext,
    kind: NumberKind,
    fallback: bool,
    keta: i32,
) {
    destination.set_field_ascii(1, &judgment.side.to_string());
    if fallback {
        style_fallback_base(destination, judgment, keta);
    }
    destination.set_field_ascii(DST_OP1_FIELD, "270");
    destination.set_field_ascii(DST_OP3_FIELD, "0");
    match kind {
        NumberKind::Green => {
            destination.set_field_ascii(4, "-9");
            destination.set_field_ascii(9, "0");
            destination.set_field_ascii(10, "255");
            destination.set_field_ascii(11, "0");
            destination.set_field_ascii(DST_OP2_FIELD, "271");
            destination.set_field_ascii(DST_OP4_FIELD, "4");
        }
        NumberKind::Lift => {
            let width = destination.field_i32(5).unwrap_or(0).abs();
            let height = destination.field_i32(6).unwrap_or(0).abs();
            let original_x = destination.field_i32(3).unwrap_or(judgment.x);
            let (x, y) = if context.detected_resolution == Some(0) {
                (original_x, judgment.y - height - 2)
            } else {
                (original_x + width, judgment.lower_edge() + 10)
            };
            destination.set_field_ascii(3, &x.to_string());
            destination.set_field_ascii(4, &y.to_string());
            destination.set_field_ascii(9, "255");
            destination.set_field_ascii(10, "255");
            destination.set_field_ascii(11, "255");
            destination.set_field_ascii(DST_OP2_FIELD, "272");
            destination.set_field_ascii(DST_OP4_FIELD, "0");
        }
    }
}

fn style_fallback_base(destination: &mut CsvRecord, judgment: &JudgmentGeometry, keta: i32) {
    destination.set_field_ascii(1, &judgment.side.to_string());
    destination.set_field_ascii(2, "0");
    destination.set_field_ascii(7, "0");
    destination.set_field_ascii(8, "255");
    destination.set_field_ascii(12, "1");
    destination.set_field_ascii(13, "0");
    destination.set_field_ascii(14, "0");
    destination.set_field_ascii(15, "0");
    destination.set_field_ascii(16, "0");
    destination.set_field_ascii(17, "0");
    let digit_width = destination.field_i32(5).unwrap_or(0).abs();
    let total_width = digit_width.saturating_mul(keta);
    let x = judgment
        .x
        .saturating_add(judgment.width.saturating_sub(total_width).div_euclid(2));
    destination.set_field_ascii(3, &x.to_string());
}

fn has_target_number(file: &EditableFile, kind: NumberKind) -> bool {
    let target = match kind {
        NumberKind::Green => "313",
        NumberKind::Lift => "314",
    };
    (0..file.document.line_count()).any(|index| {
        file.document.line_content(index).is_some_and(|content| {
            let record = CsvRecord::from_line(index, content);
            record.is_command("#SRC_NUMBER") && record.field_equals_ascii(SRC_NUM_FIELD, target)
        })
    })
}
