use std::{collections::BTreeSet, fs, path::Path};

use skin_core::{AnalysisReport, analyze_path};

use crate::{
    definitions::{all_rule_definitions, preset_rule_ids},
    dxa,
    error::{RuleError, RuleResult},
    model::{
        ConversionOptions, ConversionReport, FileChange, GeneratedAsset, RuleContext, RuleWarning,
    },
    rules::{
        aspect, basic, dark_mode, follow_lift, judge_line, key_laser_layer, lift_cover,
        numbers::{self, NumberKind},
    },
    theme::{detect_internal_theme, theme_dir_name},
    workspace::{
        build_manifest, copy_dir_recursive, load_editable_files, validate_output_path,
        write_dirty_files, write_generated_assets,
    },
};

pub fn dry_run(
    input: impl AsRef<Path>,
    options: ConversionOptions,
) -> RuleResult<ConversionReport> {
    let input = input.as_ref();
    let analysis = analyze_path(input)?;
    let mut files = load_editable_files(input)?;
    let selected_rules = resolve_rule_ids(&options);
    let context = context_from(input, input, &analysis, &files);
    let (changes, warnings, generated_assets) =
        apply_rules(&mut files, &selected_rules, &context, &options);
    let manifest = build_manifest(
        input,
        None,
        &options,
        &selected_rules,
        &files,
        &generated_assets,
        &[],
        &warnings,
    )?;
    Ok(ConversionReport {
        analysis,
        selected_rules,
        changes,
        warnings,
        manifest,
    })
}

pub fn generate(
    input: impl AsRef<Path>,
    output: impl AsRef<Path>,
    options: ConversionOptions,
) -> RuleResult<ConversionReport> {
    let input = input.as_ref();
    let output = output.as_ref();
    validate_output_path(input, output)?;
    let analysis = analyze_path(input)?;
    copy_dir_recursive(input, output)?;
    let dxa_report = dxa::unpack_archives(output)?;
    let mut files = load_editable_files(output)?;
    let selected_rules = resolve_rule_ids(&options);
    let context = context_from(input, output, &analysis, &files);
    let (rule_changes, warnings, generated_assets) =
        apply_rules(&mut files, &selected_rules, &context, &options);
    let mut changes = dxa_report.changes();
    changes.extend(rule_changes);
    write_dirty_files(output, &files)?;
    write_generated_assets(output, &generated_assets)?;
    let manifest = build_manifest(
        input,
        Some(output),
        &options,
        &selected_rules,
        &files,
        &generated_assets,
        &dxa_report.extracted_files,
        &warnings,
    )?;
    let manifest_path = output.join("bms-skin-generator-manifest.json");
    let manifest_bytes = serde_json::to_vec_pretty(&manifest)?;
    fs::write(&manifest_path, manifest_bytes)
        .map_err(|source| RuleError::io(manifest_path, source))?;
    Ok(ConversionReport {
        analysis,
        selected_rules,
        changes,
        warnings,
        manifest,
    })
}

fn apply_rules(
    files: &mut [crate::model::EditableFile],
    selected_rules: &[String],
    context: &RuleContext,
    options: &ConversionOptions,
) -> (Vec<FileChange>, Vec<RuleWarning>, Vec<GeneratedAsset>) {
    let mut changes = Vec::new();
    let mut warnings = Vec::new();
    let mut generated_assets = Vec::new();
    let selected = selected_rules
        .iter()
        .map(String::as_str)
        .collect::<BTreeSet<_>>();

    for rule_id in RULE_ORDER {
        if !selected.contains(rule_id) {
            continue;
        }
        match *rule_id {
            "resolution-declaration" => {
                basic::apply_resolution(files, context, options, &mut changes, &mut warnings);
            }
            "theme-path-rebase" => {
                basic::apply_path_rebase(files, context, &mut changes, &mut warnings);
            }
            "asset-reference-repair" => {
                crate::rules::asset_repair::apply(files, context, &mut changes, &mut warnings);
            }
            "follow-lift-op4" => follow_lift::apply(files, &mut changes),
            "key-laser-layer-compat" => {
                key_laser_layer::apply(files, &mut changes, &mut warnings);
            }
            "judge-line-follow" => judge_line::apply(
                files,
                context,
                &mut generated_assets,
                &mut changes,
                &mut warnings,
            ),
            "blend-fallback" => basic::apply_blend(files, &mut changes),
            "lift-cover-transparent" => {
                lift_cover::apply_transparent(files, &mut changes, &mut warnings);
            }
            "lift-cover-copy-sudden" => {
                lift_cover::apply_copy_sudden(files, &mut changes, &mut warnings);
            }
            "green-number" => numbers::apply(
                files,
                context,
                NumberKind::Green,
                &mut changes,
                &mut warnings,
            ),
            "lift-number" => numbers::apply(
                files,
                context,
                NumberKind::Lift,
                &mut changes,
                &mut warnings,
            ),
            "aspect-ratio-convert" => {
                aspect::apply(files, context, options, &mut changes, &mut warnings);
            }
            "lr2-dark-mode" => dark_mode::apply(
                files,
                context,
                &mut generated_assets,
                &mut changes,
                &mut warnings,
            ),
            "key-laser-rewrite" | "gauge-label-helper" | "lift-cover-custom-image" => {
                warnings.push(RuleWarning {
                    rule_id: (*rule_id).to_owned(),
                    path: None,
                    message:
                        "This visual compatibility helper requires a skin-specific asset mapping."
                            .to_owned(),
                });
            }
            _ => {}
        }
    }
    for unknown in selected.difference(&RULE_ORDER.iter().copied().collect()) {
        warnings.push(RuleWarning {
            rule_id: (*unknown).to_owned(),
            path: None,
            message: "Unknown rule id; skipped.".to_owned(),
        });
    }
    (changes, warnings, generated_assets)
}

fn resolve_rule_ids(options: &ConversionOptions) -> Vec<String> {
    let mut output = Vec::new();
    let mut seen = BTreeSet::new();
    if let Some(preset) = &options.preset {
        for id in preset_rule_ids(preset) {
            push_rule(&mut output, &mut seen, id);
        }
    } else if options.rules.is_empty() {
        for definition in all_rule_definitions()
            .into_iter()
            .filter(|definition| definition.default_enabled)
        {
            push_rule(&mut output, &mut seen, definition.id);
        }
    }
    if options.resolution.is_some() {
        push_rule(&mut output, &mut seen, "resolution-declaration".to_owned());
    }
    if options.resolution.is_some() || options.target_aspect.is_some() {
        push_rule(&mut output, &mut seen, "aspect-ratio-convert".to_owned());
    }
    for id in &options.rules {
        push_rule(&mut output, &mut seen, id.clone());
    }
    if let Some(strategy) = options.lift_cover_strategy.as_deref() {
        let id = match strategy {
            "transparent" => Some("lift-cover-transparent"),
            "copy-sudden" => Some("lift-cover-copy-sudden"),
            "custom-image" => Some("lift-cover-custom-image"),
            _ => None,
        };
        if let Some(id) = id {
            push_rule(&mut output, &mut seen, id.to_owned());
        }
    }
    output
}

fn push_rule(output: &mut Vec<String>, seen: &mut BTreeSet<String>, id: String) {
    if seen.insert(id.clone()) {
        output.push(id);
    }
}

fn context_from(
    source: &Path,
    output: &Path,
    analysis: &AnalysisReport,
    files: &[crate::model::EditableFile],
) -> RuleContext {
    RuleContext {
        output_root: output.to_path_buf(),
        source_theme_dir: detect_internal_theme(files).or_else(|| theme_dir_name(source)),
        output_theme_dir: theme_dir_name(output),
        detected_resolution: analysis.layout.resolution,
        canvas_width: analysis.layout.canvas_width,
        canvas_height: analysis.layout.canvas_height,
    }
}

const RULE_ORDER: &[&str] = &[
    "resolution-declaration",
    "theme-path-rebase",
    "asset-reference-repair",
    "follow-lift-op4",
    "key-laser-layer-compat",
    "blend-fallback",
    "lift-cover-transparent",
    "lift-cover-copy-sudden",
    "green-number",
    "lift-number",
    "judge-line-follow",
    "aspect-ratio-convert",
    "lr2-dark-mode",
    "key-laser-rewrite",
    "gauge-label-helper",
    "lift-cover-custom-image",
];
