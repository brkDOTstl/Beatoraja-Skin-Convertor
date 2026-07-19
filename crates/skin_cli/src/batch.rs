use std::{
    fs,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use skin_rules::{AspectProtection, ConversionOptions, TargetAspect, generate};

#[derive(Debug, Deserialize)]
struct BatchJob {
    entries: Vec<BatchEntry>,
}

#[derive(Debug, Deserialize)]
struct BatchEntry {
    input: PathBuf,
    output: PathBuf,
    preset: Option<String>,
    #[serde(default)]
    rules: Vec<String>,
    lift_cover_strategy: Option<String>,
    resolution: Option<u8>,
    target_aspect: Option<TargetAspect>,
    #[serde(default)]
    aspect_protection: AspectProtection,
}

#[derive(Debug, Serialize)]
pub(crate) struct BatchSummary {
    pub(crate) entries: Vec<BatchEntrySummary>,
}

#[derive(Debug, Serialize)]
pub(crate) struct BatchEntrySummary {
    pub(crate) input: PathBuf,
    pub(crate) output: PathBuf,
    pub(crate) ok: bool,
    pub(crate) changed_files: usize,
    pub(crate) warnings: usize,
    pub(crate) error: Option<String>,
}

pub(crate) fn run_batch(job_path: &Path, fail_fast: bool) -> Result<BatchSummary> {
    let bytes = fs::read(job_path).with_context(|| format!("read job {}", job_path.display()))?;
    let job = parse_batch_job(&bytes)?;
    let mut entries = Vec::new();

    for entry in job.entries {
        let options = ConversionOptions {
            preset: entry.preset.clone(),
            rules: entry.rules.clone(),
            lift_cover_strategy: entry.lift_cover_strategy.clone(),
            resolution: entry.resolution,
            target_aspect: entry.target_aspect,
            aspect_protection: entry.aspect_protection,
        };
        match generate(&entry.input, &entry.output, options) {
            Ok(report) => entries.push(BatchEntrySummary {
                input: entry.input,
                output: entry.output,
                ok: true,
                changed_files: report.manifest.changed_files.len(),
                warnings: report.warnings.len(),
                error: None,
            }),
            Err(error) => {
                entries.push(BatchEntrySummary {
                    input: entry.input,
                    output: entry.output,
                    ok: false,
                    changed_files: 0,
                    warnings: 0,
                    error: Some(error.to_string()),
                });
                if fail_fast {
                    break;
                }
            }
        }
    }

    Ok(BatchSummary { entries })
}

fn parse_batch_job(bytes: &[u8]) -> Result<BatchJob> {
    const UTF8_BOM: &[u8] = b"\xEF\xBB\xBF";
    let json = bytes.strip_prefix(UTF8_BOM).unwrap_or(bytes);
    serde_json::from_slice(json).context("parse batch job JSON")
}

#[cfg(test)]
mod tests {
    use super::parse_batch_job;
    use skin_rules::TargetAspect;

    #[test]
    fn accepts_utf8_bom() {
        let job = parse_batch_job(b"\xEF\xBB\xBF{\"entries\":[]}").expect("parse BOM JSON");
        assert!(job.entries.is_empty());
    }

    #[test]
    fn parses_aspect_conversion_options() {
        let job = parse_batch_job(
            br#"{"entries":[{"input":"source","output":"output","preset":null,"rules":[],"lift_cover_strategy":null,"resolution":2,"target_aspect":"16:9","aspect_protection":{"bomb_effects":true,"nowjudge":false,"nowcombo":true}}]}"#,
        )
        .expect("parse aspect options");
        let entry = &job.entries[0];
        assert_eq!(entry.resolution, Some(2));
        assert_eq!(entry.target_aspect, Some(TargetAspect::SixteenNine));
        assert!(entry.aspect_protection.bomb_effects);
        assert!(!entry.aspect_protection.nowjudge);
        assert!(entry.aspect_protection.nowcombo);
    }
}
