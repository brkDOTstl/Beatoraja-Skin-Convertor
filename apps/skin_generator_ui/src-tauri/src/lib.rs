use std::path::{Path, PathBuf};

use skin_core::{AnalysisReport, analyze_path};
use skin_rules::{
    AspectProtection, ConversionOptions, ConversionReport, RuleDefinition, TargetAspect,
    all_rule_definitions, dry_run, generate,
};

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

pub fn run() -> Result<(), tauri::Error> {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            rule_definitions,
            analyze_skin,
            select_folder,
            dry_run_skin,
            generate_skin,
        ])
        .run(tauri::generate_context!())
}
