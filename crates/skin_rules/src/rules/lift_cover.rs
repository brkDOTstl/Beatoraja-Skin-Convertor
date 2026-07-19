use crate::{
    edit::{
        document_has_command, document_has_marker, find_last_command, insert_lines, is_play_file,
        warn,
    },
    model::{EditableFile, FileChange, RuleWarning},
    rules::{
        geometry::judgment_geometries,
        lift_source::{copy_lines, lane_cover_image_index, lane_cover_lines, sudden_sliders},
    },
};

const MARK_TRANSPARENT: &[u8] = b"//bms-skin-generator:R003:LIFT_TRANSPARENT";

pub(crate) fn apply_copy_sudden(
    files: &mut [EditableFile],
    changes: &mut Vec<FileChange>,
    warnings: &mut Vec<RuleWarning>,
) {
    let mut applied = false;
    for file in files.iter_mut().filter(|file| is_play_file(file)) {
        if document_has_command(&file.document, "#SRC_LIFT") {
            continue;
        }
        let judgments = judgment_geometries(&file.document);
        let sliders = sudden_sliders(file);
        let lane_cover_image = lane_cover_image_index(file);
        if judgments.is_empty()
            || (!sliders.is_empty() && judgments.len() != sliders.len())
            || (sliders.is_empty() && lane_cover_image.is_none())
        {
            warn(
                warnings,
                "lift-cover-copy-sudden",
                Some(file),
                format!(
                    "Judgment/SUDDEN count mismatch: {} judgment sides, {} sliders.",
                    judgments.len(),
                    sliders.len()
                ),
            );
            continue;
        }
        let Some(index) =
            find_last_command(&file.document, "#DST_JUDGELINE").map(|value| value + 1)
        else {
            continue;
        };
        let lines = if sliders.is_empty() {
            judgments
                .iter()
                .flat_map(|judgment| lane_cover_lines(judgment, lane_cover_image.unwrap_or(0)))
                .collect::<Vec<_>>()
        } else {
            judgments
                .iter()
                .zip(&sliders)
                .flat_map(|(judgment, slider)| copy_lines(judgment, slider))
                .collect::<Vec<_>>()
        };
        insert_lines(file, index, lines, "lift-cover-copy-sudden", changes);
        applied = true;
    }
    if !applied {
        warn(
            warnings,
            "lift-cover-copy-sudden",
            None,
            "No structurally matched SUDDEN cover was available.",
        );
    }
}

pub(crate) fn apply_transparent(
    files: &mut [EditableFile],
    changes: &mut Vec<FileChange>,
    warnings: &mut Vec<RuleWarning>,
) {
    let mut applied = false;
    for file in files.iter_mut().filter(|file| is_play_file(file)) {
        if document_has_command(&file.document, "#SRC_LIFT")
            || document_has_marker(&file.document, MARK_TRANSPARENT)
        {
            continue;
        }
        let judgments = judgment_geometries(&file.document);
        let Some(index) =
            find_last_command(&file.document, "#DST_JUDGELINE").map(|value| value + 1)
        else {
            continue;
        };
        let mut lines = vec![MARK_TRANSPARENT.to_vec()];
        for judgment in judgments {
            let edge = judgment.lower_edge();
            lines.push(
                format!(
                    "#SRC_LIFT,{},0,0,0,1,1,1,1,0,0,{edge},0,0,0,,,,,,,",
                    judgment.side
                )
                .into_bytes(),
            );
            lines.push(
                format!(
                    "#DST_LIFT,{},0,{},{},1,1,0,0,255,255,255,0,0,0,0,0,0,0,0,0,3",
                    judgment.side, judgment.x, edge
                )
                .into_bytes(),
            );
        }
        insert_lines(file, index, lines, "lift-cover-transparent", changes);
        applied = true;
    }
    if !applied {
        warn(
            warnings,
            "lift-cover-transparent",
            None,
            "No Play CSV needed a transparent LIFT cover.",
        );
    }
}
