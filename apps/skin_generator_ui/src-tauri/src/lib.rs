use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use skin_core::{AnalysisReport, analyze_path};
use skin_rules::{
    AspectProtection, ConversionOptions, ConversionReport, RuleDefinition, TargetAspect,
    all_rule_definitions, dry_run, generate,
};

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
struct BatchJob {
    input: String,
    output: String,
    preset: Option<String>,
    #[serde(default)]
    rules: Vec<String>,
    lift_cover_strategy: Option<String>,
    resolution: Option<u8>,
    target_aspect: Option<TargetAspect>,
    #[serde(default)]
    aspect_protection: AspectProtection,
}

impl BatchJob {
    fn options(&self) -> ConversionOptions {
        ConversionOptions {
            preset: self.preset.clone(),
            rules: self.rules.clone(),
            lift_cover_strategy: self.lift_cover_strategy.clone(),
            resolution: self.resolution,
            target_aspect: self.target_aspect,
            aspect_protection: self.aspect_protection,
        }
    }
}

#[derive(Debug, Serialize)]
struct BatchResult {
    input: String,
    output: String,
    ok: bool,
    applied_resolution: Option<u8>,
    applied_target_aspect: Option<TargetAspect>,
    changed_files: usize,
    warnings: usize,
    error: Option<String>,
}

#[tauri::command]
fn rule_definitions() -> Vec<RuleDefinition> {
    all_rule_definitions()
}

#[tauri::command]
fn analyze_skin(input: String) -> Result<AnalysisReport, String> {
    analyze_path(PathBuf::from(input)).map_err(|error| error.to_string())
}

#[tauri::command]
fn select_folder(initial_path: Option<String>) -> Option<String> {
    let mut dialog = rfd::FileDialog::new();
    if let Some(path) = initial_path.filter(|path| Path::new(path).is_dir()) {
        dialog = dialog.set_directory(path);
    }
    dialog
        .pick_folder()
        .map(|path| path.to_string_lossy().into_owned())
}

#[tauri::command]
fn dry_run_skin(
    input: String,
    preset: Option<String>,
    rules: Vec<String>,
    lift_cover_strategy: Option<String>,
    resolution: Option<u8>,
    target_aspect: Option<TargetAspect>,
    aspect_protection: Option<AspectProtection>,
) -> Result<ConversionReport, String> {
    dry_run(
        PathBuf::from(input),
        ConversionOptions {
            preset,
            rules,
            lift_cover_strategy,
            resolution,
            target_aspect,
            aspect_protection: aspect_protection.unwrap_or_default(),
        },
    )
    .map_err(|error| error.to_string())
}

#[tauri::command]
fn generate_skin(
    input: String,
    output: String,
    preset: Option<String>,
    rules: Vec<String>,
    lift_cover_strategy: Option<String>,
    resolution: Option<u8>,
    target_aspect: Option<TargetAspect>,
    aspect_protection: Option<AspectProtection>,
) -> Result<ConversionReport, String> {
    generate(
        PathBuf::from(input),
        PathBuf::from(output),
        ConversionOptions {
            preset,
            rules,
            lift_cover_strategy,
            resolution,
            target_aspect,
            aspect_protection: aspect_protection.unwrap_or_default(),
        },
    )
    .map_err(|error| error.to_string())
}

#[tauri::command]
fn generate_batch(jobs: Vec<BatchJob>) -> Vec<BatchResult> {
    jobs.into_iter().map(generate_batch_job).collect()
}

fn generate_batch_job(job: BatchJob) -> BatchResult {
    let report = generate(
        PathBuf::from(&job.input),
        PathBuf::from(&job.output),
        job.options(),
    );
    match report {
        Ok(report) => BatchResult {
            input: job.input,
            output: job.output,
            ok: true,
            applied_resolution: job.resolution,
            applied_target_aspect: job.target_aspect,
            changed_files: report.manifest.changed_files.len(),
            warnings: report.warnings.len(),
            error: None,
        },
        Err(error) => BatchResult {
            input: job.input,
            output: job.output,
            ok: false,
            applied_resolution: job.resolution,
            applied_target_aspect: job.target_aspect,
            changed_files: 0,
            warnings: 0,
            error: Some(error.to_string()),
        },
    }
}

pub fn run() -> Result<(), tauri::Error> {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            rule_definitions,
            analyze_skin,
            select_folder,
            dry_run_skin,
            generate_skin,
            generate_batch,
        ])
        .run(tauri::generate_context!())
}

#[cfg(test)]
mod tests {
    use std::{fs, path::PathBuf};

    use super::*;

    #[test]
    fn batch_job_deserializes_all_layout_options_from_ui_payload() {
        let job: BatchJob = serde_json::from_str(
            r#"{
                "input":"source",
                "output":"output",
                "preset":null,
                "rules":["resolution-declaration","aspect-ratio-convert"],
                "liftCoverStrategy":"copy-sudden",
                "resolution":2,
                "targetAspect":"4:3",
                "aspectProtection":{
                    "bomb_effects":true,
                    "nowjudge":false,
                    "nowcombo":true
                }
            }"#,
        )
        .expect("deserialize UI batch job");

        let options = job.options();
        assert_eq!(options.resolution, Some(2));
        assert_eq!(options.target_aspect, Some(TargetAspect::FourThree));
        assert_eq!(options.lift_cover_strategy.as_deref(), Some("copy-sudden"));
        assert!(options.aspect_protection.bomb_effects);
        assert!(!options.aspect_protection.nowjudge);
        assert!(options.aspect_protection.nowcombo);
    }

    #[test]
    fn native_batch_generation_compensates_resolution_zero_for_four_three() {
        let root = temporary_root("layout-options");
        let input = root.join("source");
        let output = root.join("generated");
        fs::create_dir_all(input.join("Play")).expect("create source");
        fs::write(
            input.join("play.lr2skin"),
            concat!(
                "#INFORMATION,0,title,maker,\r\n",
                "#RESOLUTION,0\r\n",
                "#INCLUDE,Play\\layout.csv\r\n",
            ),
        )
        .expect("write header");
        fs::write(
            input.join("Play").join("layout.csv"),
            concat!(
                "#SRC_JUDGELINE,0,0,0,0,100,4,1,1,0,0,0,0,0\r\n",
                "#DST_JUDGELINE,0,0,20,300,100,4,0,255,255,255,255,0,0,0,0,0,0,0,0,0\r\n",
            ),
        )
        .expect("write layout");

        let result = generate_batch_job(BatchJob {
            input: input.to_string_lossy().into_owned(),
            output: output.to_string_lossy().into_owned(),
            preset: None,
            rules: Vec::new(),
            lift_cover_strategy: None,
            resolution: Some(0),
            target_aspect: Some(TargetAspect::FourThree),
            aspect_protection: AspectProtection {
                bomb_effects: true,
                nowjudge: true,
                nowcombo: true,
            },
        });

        assert!(result.ok, "{:?}", result.error);
        assert_eq!(result.applied_resolution, Some(0));
        assert_eq!(result.applied_target_aspect, Some(TargetAspect::FourThree));
        let header = fs::read_to_string(output.join("play.lr2skin")).expect("read output header");
        assert!(header.contains("#RESOLUTION,0"));
        let layout =
            fs::read_to_string(output.join("Play").join("layout.csv")).expect("read output layout");
        assert!(layout.contains("#DST_JUDGELINE,0,0,95,300,75,4"));
        let manifest = fs::read_to_string(output.join("bms-skin-generator-manifest.json"))
            .expect("read manifest");
        assert!(manifest.contains("\"aspect-ratio-convert\""));
        fs::remove_dir_all(root).expect("remove fixture");
    }

    fn temporary_root(name: &str) -> PathBuf {
        let root = std::env::temp_dir().join(format!(
            "beatoraja-skin-convertor-batch-{name}-{}",
            std::process::id()
        ));
        if root.exists() {
            fs::remove_dir_all(&root).expect("remove old fixture");
        }
        root
    }
}
