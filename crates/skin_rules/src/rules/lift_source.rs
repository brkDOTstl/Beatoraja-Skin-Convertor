use skin_core::CsvRecord;

use crate::{
    model::EditableFile,
    rules::geometry::{JudgmentGeometry, next_destinations},
};

const MARK_COPY: &str = "//bms-skin-generator:R006:LIFT_COPY_SUDDEN";

#[derive(Debug)]
pub(crate) struct SuddenSlider {
    gr: i32,
    source_x: i32,
    source_y: i32,
    source_width: i32,
    source_height: i32,
    div_x: i32,
    div_y: i32,
    cycle: i32,
    timer: i32,
    destination_x: i32,
    destination_width: i32,
    destination_height: i32,
}

pub(crate) fn lane_cover_image_index(file: &EditableFile) -> Option<i32> {
    let mut image_index = 0;
    for line_index in 0..file.document.line_count() {
        let content = file.document.line_content(line_index)?;
        let record = CsvRecord::from_line(line_index, content);
        if !record.is_command("#IMAGE") {
            continue;
        }
        let is_lane_cover = record.field(1).is_some_and(|value| {
            let path = String::from_utf8_lossy(value).to_ascii_lowercase();
            path.contains("lanecover") || path.contains("lane_cover")
        });
        if is_lane_cover {
            return Some(image_index);
        }
        image_index += 1;
    }
    None
}

pub(crate) fn sudden_sliders(file: &EditableFile) -> Vec<SuddenSlider> {
    let mut output = Vec::new();
    for line_index in 0..file.document.line_count() {
        let Some(content) = file.document.line_content(line_index) else {
            continue;
        };
        let source = CsvRecord::from_line(line_index, content);
        if !source.is_command("#SRC_SLIDER")
            || !source
                .field_i32(13)
                .is_some_and(|kind| kind == 4 || kind == 5)
        {
            continue;
        }
        let Some((_, destination)) = next_destinations(&file.document, line_index, "#DST_SLIDER")
            .into_iter()
            .next()
        else {
            continue;
        };
        let values = (
            source.field_i32(2),
            source.field_i32(3),
            source.field_i32(4),
            source.field_i32(5),
            source.field_i32(6),
            source.field_i32(7),
            source.field_i32(8),
            source.field_i32(9),
            source.field_i32(10),
            destination.field_i32(3),
            destination.field_i32(5),
            destination.field_i32(6),
        );
        if let (
            Some(gr),
            Some(source_x),
            Some(source_y),
            Some(source_width),
            Some(source_height),
            Some(div_x),
            Some(div_y),
            Some(cycle),
            Some(timer),
            Some(destination_x),
            Some(destination_width),
            Some(destination_height),
        ) = values
        {
            output.push(SuddenSlider {
                gr,
                source_x,
                source_y,
                source_width,
                source_height,
                div_x,
                div_y,
                cycle,
                timer,
                destination_x,
                destination_width,
                destination_height,
            });
        }
    }
    output
}

pub(crate) fn copy_lines(judgment: &JudgmentGeometry, slider: &SuddenSlider) -> [Vec<u8>; 3] {
    let edge = judgment.lower_edge();
    [
        format!("{MARK_COPY}:{}", judgment.side).into_bytes(),
        format!(
            "#SRC_LIFT,{},{},{},{},{},{},{},{},{},{},{},0,0,0,,,,,,,,",
            judgment.side,
            slider.gr,
            slider.source_x,
            slider.source_y,
            slider.source_width,
            slider.source_height,
            slider.div_x,
            slider.div_y,
            slider.cycle,
            slider.timer,
            edge
        )
        .into_bytes(),
        format!(
            "#DST_LIFT,{},0,{},{},{},{},2,255,255,255,255,1,0,0,0,0,0,0,0,0,3",
            judgment.side,
            slider.destination_x,
            edge,
            slider.destination_width,
            slider.destination_height
        )
        .into_bytes(),
    ]
}

pub(crate) fn lane_cover_lines(judgment: &JudgmentGeometry, image_index: i32) -> [Vec<u8>; 3] {
    let edge = judgment.lower_edge();
    [
        format!("{MARK_COPY}:{}:IMAGE", judgment.side).into_bytes(),
        format!(
            "#SRC_LIFT,{},{image_index},0,0,-1,-1,1,1,0,0,{edge},0,0,0,,,,,,,,",
            judgment.side
        )
        .into_bytes(),
        format!(
            "#DST_LIFT,{},0,{},{},{},{},2,255,255,255,255,1,0,0,0,0,0,0,0,0",
            judgment.side,
            judgment.x.saturating_sub(1),
            edge,
            judgment.width.saturating_add(1),
            edge
        )
        .into_bytes(),
    ]
}
