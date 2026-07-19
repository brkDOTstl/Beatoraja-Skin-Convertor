use serde::{Deserialize, Serialize};

use crate::{CsvRecord, TextDocument, csv::records_for_document};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LayoutDetection {
    pub resolution: Option<u8>,
    pub canvas_width: Option<i32>,
    pub canvas_height: Option<i32>,
    pub aspect_ratio: String,
    pub source: String,
}

impl Default for LayoutDetection {
    fn default() -> Self {
        Self {
            resolution: None,
            canvas_width: None,
            canvas_height: None,
            aspect_ratio: "unknown".to_owned(),
            source: "unknown".to_owned(),
        }
    }
}

pub fn detect_layout<'a>(documents: impl IntoIterator<Item = &'a TextDocument>) -> LayoutDetection {
    let mut explicit = None;
    let mut canvas_votes = [0usize; 4];
    let mut max_right = 0;
    let mut max_bottom = 0;
    for document in documents {
        for record in records_for_document(document) {
            if record.is_command("#RESOLUTION") {
                explicit = explicit.or_else(|| {
                    record
                        .field_i32(1)
                        .and_then(|value| u8::try_from(value).ok())
                });
            }
            record_canvas_vote(&record, &mut canvas_votes);
            update_extent(&record, &mut max_right, &mut max_bottom);
        }
    }

    if let Some(resolution) = explicit {
        return from_resolution(resolution, "declaration");
    }
    if let Some(resolution) = strongest_canvas_vote(&canvas_votes) {
        return from_resolution(resolution, "full_canvas");
    }
    match infer_resolution(max_right, max_bottom) {
        Some(resolution) => from_resolution(resolution, "coordinates"),
        None => LayoutDetection::default(),
    }
}

fn record_canvas_vote(record: &CsvRecord, votes: &mut [usize; 4]) {
    let Some(command) = record.command() else {
        return;
    };
    if !command.starts_with("#DST_") {
        return;
    }
    let pair = (
        record.field_i32(5).map(i32::abs),
        record.field_i32(6).map(i32::abs),
    );
    let resolution = match pair {
        (Some(640), Some(480)) => Some(0),
        (Some(1280), Some(720)) => Some(1),
        (Some(1920), Some(1080)) => Some(2),
        (Some(3840), Some(2160)) => Some(3),
        _ => None,
    };
    if let Some(resolution) = resolution {
        votes[resolution] = votes[resolution].saturating_add(1);
    }
}

fn strongest_canvas_vote(votes: &[usize; 4]) -> Option<u8> {
    votes
        .iter()
        .enumerate()
        .max_by_key(|(_, count)| *count)
        .and_then(|(resolution, count)| (*count > 0).then_some(resolution as u8))
}

pub fn canvas_for_resolution(resolution: u8) -> (Option<i32>, Option<i32>) {
    match resolution {
        0 => (Some(640), Some(480)),
        1 => (Some(1280), Some(720)),
        2 => (Some(1920), Some(1080)),
        3 => (Some(3840), Some(2160)),
        _ => (None, None),
    }
}

fn from_resolution(resolution: u8, source: &str) -> LayoutDetection {
    let (canvas_width, canvas_height) = canvas_for_resolution(resolution);
    LayoutDetection {
        resolution: Some(resolution),
        canvas_width,
        canvas_height,
        aspect_ratio: aspect_for(canvas_width, canvas_height),
        source: source.to_owned(),
    }
}

fn update_extent(record: &CsvRecord, max_right: &mut i32, max_bottom: &mut i32) {
    let Some(command) = record.command() else {
        return;
    };
    if !command.starts_with("#DST_") {
        return;
    }
    let (Some(x), Some(y), Some(width), Some(height)) = (
        record.field_i32(3),
        record.field_i32(4),
        record.field_i32(5),
        record.field_i32(6),
    ) else {
        return;
    };
    *max_right = (*max_right).max(x.saturating_add(width.abs()));
    *max_bottom = (*max_bottom).max(y.saturating_add(height.abs()));
}

fn infer_resolution(max_right: i32, max_bottom: i32) -> Option<u8> {
    if max_right <= 0 || max_bottom <= 0 {
        None
    } else if max_right <= 700 && max_bottom <= 520 {
        Some(0)
    } else if max_right <= 1400 && max_bottom <= 800 {
        Some(1)
    } else if max_right <= 2100 && max_bottom <= 1200 {
        Some(2)
    } else {
        Some(3)
    }
}

fn aspect_for(width: Option<i32>, height: Option<i32>) -> String {
    match (width, height) {
        (Some(width), Some(height)) if width.saturating_mul(9) == height.saturating_mul(16) => {
            "16:9".to_owned()
        }
        (Some(width), Some(height)) if width.saturating_mul(3) == height.saturating_mul(4) => {
            "4:3".to_owned()
        }
        (Some(width), Some(height)) => format!("{width}:{height}"),
        _ => "unknown".to_owned(),
    }
}
