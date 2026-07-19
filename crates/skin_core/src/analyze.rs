use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::{
    CsvRecord, LayoutDetection, TextDocument,
    capabilities::{build_capabilities, finding},
    csv::records_for_document,
    error::CoreResult,
    layout::detect_layout,
    scan::{SkinFileKind, read_skin_files},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Confidence {
    SafeAuto,
    RecommendedConfirm,
    RiskyManual,
    Unsupported,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Finding {
    pub code: String,
    pub message: String,
    pub confidence: Confidence,
    pub file: Option<PathBuf>,
    pub line: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleCapability {
    pub rule_id: String,
    pub title: String,
    pub confidence: Confidence,
    pub status: String,
    pub detail: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AnalysisCounters {
    pub total_csv_files: usize,
    pub play_csv_files: usize,
    pub lr2skin_files: usize,
    pub has_resolution: bool,
    pub has_src_lift: bool,
    pub has_dst_lift: bool,
    pub has_green_number: bool,
    pub has_lift_number: bool,
    pub judge_line_count: usize,
    pub judge_line_missing_op4: usize,
    pub follow_lift_candidates: usize,
    pub key_laser_candidates: usize,
    pub bomb_or_grow_candidates: usize,
    pub rhythm_line_candidates: usize,
    pub blend5_candidates: usize,
    pub gauge_label_assets: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalysisReport {
    pub input_path: PathBuf,
    pub layout: LayoutDetection,
    pub counters: AnalysisCounters,
    pub capabilities: Vec<RuleCapability>,
    pub findings: Vec<Finding>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FollowLiftKind {
    BombOrGrow,
    KeyLaser,
    RhythmLine,
}

pub fn analyze_path(root: impl AsRef<Path>) -> CoreResult<AnalysisReport> {
    let root = root.as_ref();
    let files = read_skin_files(root)?;
    let layout = detect_layout(files.iter().map(|file| &file.document));
    let mut counters = AnalysisCounters::default();

    for file in &files {
        match file.kind {
            SkinFileKind::Csv => {
                counters.total_csv_files += 1;
                counters.play_csv_files +=
                    usize::from(is_play_document(&file.relative_path, &file.document));
            }
            SkinFileKind::Lr2Skin => counters.lr2skin_files += 1,
        }
        inspect_document(&file.document, &mut counters);
    }

    let mut findings = Vec::new();
    if counters.play_csv_files == 0 {
        findings.push(finding(
            "no_play_csv",
            "No structurally valid Play CSV files were detected.",
            Confidence::Unsupported,
        ));
    }
    if !counters.has_lift_number {
        findings.push(finding(
            "missing_lift_number",
            "No beatoraja LIFT number num=314 was detected.",
            Confidence::RecommendedConfirm,
        ));
    }

    let capabilities = build_capabilities(&counters);
    Ok(AnalysisReport {
        input_path: root.to_path_buf(),
        layout,
        counters,
        capabilities,
        findings,
    })
}

pub fn is_play_document(_path: &Path, document: &TextDocument) -> bool {
    records_for_document(document).iter().any(|record| {
        record.is_command("#DST_JUDGELINE")
            || record.is_command("#SRC_NOTE")
            || record.is_command("#SRC_LN_START")
    })
}

pub fn follow_lift_kind(source: &CsvRecord) -> Option<FollowLiftKind> {
    if !source.is_command("#SRC_IMAGE") {
        return None;
    }
    let timer = source.field_i32(10)?;
    let height = source.field_i32(6).unwrap_or(0).abs();
    if is_bomb_or_grow_timer(timer) {
        Some(FollowLiftKind::BombOrGrow)
    } else if is_key_laser_timer(timer) && height >= 128 {
        Some(FollowLiftKind::KeyLaser)
    } else if timer == 140 {
        Some(FollowLiftKind::RhythmLine)
    } else {
        None
    }
}

pub fn is_key_laser_timer(timer: i32) -> bool {
    (100..=137).contains(&timer)
}

pub fn is_bomb_or_grow_timer(timer: i32) -> bool {
    (50..=89).contains(&timer)
}

fn inspect_document(document: &TextDocument, counters: &mut AnalysisCounters) {
    let mut active_kind = None;
    for record in records_for_document(document) {
        if record
            .command()
            .is_some_and(|command| command.starts_with("#SRC_"))
        {
            active_kind = follow_lift_kind(&record);
        }
        if active_kind.is_none()
            && record.is_command("#DST_IMAGE")
            && record.field_equals_ascii(17, "140")
        {
            active_kind = Some(FollowLiftKind::RhythmLine);
        }
        inspect_record(&record, active_kind, counters);
    }
}

fn inspect_record(
    record: &CsvRecord,
    active_kind: Option<FollowLiftKind>,
    counters: &mut AnalysisCounters,
) {
    counters.has_resolution |= record.is_command("#RESOLUTION");
    counters.has_src_lift |= record.is_command("#SRC_LIFT");
    counters.has_dst_lift |= record.is_command("#DST_LIFT");
    counters.has_green_number |=
        record.is_command("#SRC_NUMBER") && record.field_equals_ascii(11, "313");
    counters.has_lift_number |=
        record.is_command("#SRC_NUMBER") && record.field_equals_ascii(11, "314");
    if record.is_command("#DST_JUDGELINE") {
        counters.judge_line_count += 1;
        counters.judge_line_missing_op4 += usize::from(!record.field_equals_ascii(21, "3"));
    }
    if record.is_command("#DST_IMAGE") {
        counters.blend5_candidates += usize::from(record.field_equals_ascii(12, "5"));
        if let Some(kind) = active_kind {
            counters.follow_lift_candidates += usize::from(!record.field_equals_ascii(21, "3"));
            match kind {
                FollowLiftKind::BombOrGrow => counters.bomb_or_grow_candidates += 1,
                FollowLiftKind::KeyLaser => counters.key_laser_candidates += 1,
                FollowLiftKind::RhythmLine => counters.rhythm_line_candidates += 1,
            }
        }
    }
    if record.is_command("#IMAGE") {
        let path = record
            .field(1)
            .map(|field| String::from_utf8_lossy(field).to_ascii_lowercase())
            .unwrap_or_default();
        counters.gauge_label_assets +=
            usize::from(path.contains("_parts\\mn.png") || path.contains("_parts/mn.png"));
    }
}
