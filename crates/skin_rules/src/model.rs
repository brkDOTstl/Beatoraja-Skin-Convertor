use std::{fmt, path::PathBuf, str::FromStr};

use serde::{Deserialize, Serialize};
use skin_core::{AnalysisReport, SkinFileKind, TextDocument};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ConversionOptions {
    pub preset: Option<String>,
    #[serde(default)]
    pub rules: Vec<String>,
    pub lift_cover_strategy: Option<String>,
    pub resolution: Option<u8>,
    pub target_aspect: Option<TargetAspect>,
    #[serde(default)]
    pub aspect_protection: AspectProtection,
}

impl ConversionOptions {
    pub(crate) fn requested_resolution(&self) -> Option<u8> {
        self.resolution.or_else(|| {
            match self
                .preset
                .as_deref()
                .map(str::to_ascii_lowercase)
                .as_deref()
            {
                Some("girlish" | "girlish-cafe" | "girlish_cafe") => Some(0),
                Some(
                    "askin" | "askin-like" | "beatgb" | "beatgb-like" | "bms-no-tatsujin"
                    | "bms_no_tatsujin",
                ) => Some(1),
                Some("fhd-4x3") => Some(2),
                _ => None,
            }
        })
    }

    pub(crate) fn requested_aspect(&self) -> Option<TargetAspect> {
        self.target_aspect.or_else(|| {
            self.preset
                .as_deref()
                .is_some_and(|preset| preset.eq_ignore_ascii_case("fhd-4x3"))
                .then_some(TargetAspect::FourThree)
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TargetAspect {
    #[serde(rename = "4:3")]
    FourThree,
    #[serde(rename = "16:9")]
    SixteenNine,
}

impl TargetAspect {
    pub const fn ratio(self) -> (i32, i32) {
        match self {
            Self::FourThree => (4, 3),
            Self::SixteenNine => (16, 9),
        }
    }
}

impl fmt::Display for TargetAspect {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::FourThree => formatter.write_str("4:3"),
            Self::SixteenNine => formatter.write_str("16:9"),
        }
    }
}

impl FromStr for TargetAspect {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "4:3" => Ok(Self::FourThree),
            "16:9" => Ok(Self::SixteenNine),
            _ => Err(format!("unsupported aspect ratio: {value}")),
        }
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct AspectProtection {
    pub bomb_effects: bool,
    pub nowjudge: bool,
    pub nowcombo: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleWarning {
    pub rule_id: String,
    pub path: Option<PathBuf>,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileChange {
    pub rule_id: String,
    pub path: PathBuf,
    pub line: Option<usize>,
    pub message: String,
    pub before: String,
    pub after: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutputManifest {
    pub app_version: String,
    pub source_path: PathBuf,
    pub output_path: Option<PathBuf>,
    pub source_fingerprint: String,
    pub generated_at_unix: u64,
    pub preset: Option<String>,
    pub active_rules: Vec<String>,
    pub changed_files: Vec<PathBuf>,
    pub warnings: Vec<RuleWarning>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversionReport {
    pub analysis: AnalysisReport,
    pub selected_rules: Vec<String>,
    pub changes: Vec<FileChange>,
    pub warnings: Vec<RuleWarning>,
    pub manifest: OutputManifest,
}

pub(crate) struct EditableFile {
    pub relative_path: PathBuf,
    pub kind: SkinFileKind,
    pub document: TextDocument,
    pub dirty: bool,
}

#[derive(Debug, Clone)]
pub(crate) struct RuleContext {
    pub output_root: PathBuf,
    pub source_theme_dir: Option<String>,
    pub output_theme_dir: Option<String>,
    pub detected_resolution: Option<u8>,
    pub canvas_width: Option<i32>,
    pub canvas_height: Option<i32>,
}
