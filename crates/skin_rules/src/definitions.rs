use serde::{Deserialize, Serialize};
use skin_core::Confidence;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuleDefinition {
    pub id: String,
    pub category: String,
    pub title: String,
    pub confidence: Confidence,
    pub implemented: bool,
    pub default_enabled: bool,
    pub description: String,
}

pub fn all_rule_definitions() -> Vec<RuleDefinition> {
    vec![
        rule(
            "resolution-declaration",
            "base",
            "Resolution declaration",
            Confidence::SafeAuto,
            true,
            "Detect and insert #RESOLUTION,0/1/2/3 when missing.",
        ),
        rule(
            "theme-path-rebase",
            "base",
            "Theme path rebase",
            Confidence::SafeAuto,
            true,
            "Rebase source-theme references in lr2skin and CSV files.",
        ),
        rule(
            "asset-reference-repair",
            "base",
            "Local reference repair",
            Confidence::RecommendedConfirm,
            true,
            "Clear missing thumbnails, remove unused missing fonts, and repair movie extensions.",
        ),
        rule(
            "follow-lift-op4",
            "lift",
            "Lane effects follow LIFT",
            Confidence::SafeAuto,
            true,
            "Move bomb, hold, tall laser, and rhythm-pulse object groups with LIFT.",
        ),
        rule(
            "key-laser-layer-compat",
            "compatibility",
            "Key laser behind notes",
            Confidence::SafeAuto,
            true,
            "Preserve LR2 note visibility by registering tall key-laser groups before beatoraja creates the note renderer.",
        ),
        rule(
            "judge-line-follow",
            "lift",
            "Judge line follows LIFT",
            Confidence::SafeAuto,
            true,
            "Materialize LR2 black/white references as standalone 1x1 assets, preserve native judge-line rendering, and move every frame with LIFT.",
        ),
        rule(
            "lift-cover-transparent",
            "lift",
            "Transparent LIFT cover",
            Confidence::RecommendedConfirm,
            false,
            "Add functional invisible LIFT objects for sparse play-field UI.",
        ),
        rule(
            "lift-cover-copy-sudden",
            "lift",
            "Copy SUDDEN as LIFT cover",
            Confidence::RecommendedConfirm,
            false,
            "Pair SUDDEN sliders with judgment sides and clip at the line lower edge.",
        ),
        rule(
            "green-number",
            "numbers",
            "Green number",
            Confidence::RecommendedConfirm,
            false,
            "Clone complete lane-cover number groups as beatoraja num=313.",
        ),
        rule(
            "lift-number",
            "numbers",
            "LIFT number",
            Confidence::RecommendedConfirm,
            false,
            "Clone complete lane-cover number groups as beatoraja num=314.",
        ),
        rule(
            "aspect-ratio-convert",
            "layout",
            "Resolution and aspect conversion",
            Confidence::RecommendedConfirm,
            false,
            "Scale coordinates to a target resolution and 4:3 or 16:9 layout, preserve symmetric play-field margins, and optionally protect gameplay-material aspect ratios.",
        ),
        rule(
            "blend-fallback",
            "compatibility",
            "Blend fallback",
            Confidence::RecommendedConfirm,
            true,
            "Convert LR2-only blend=5 frames and disable incompatible duplicate option-label fade overlays.",
        ),
        rule(
            "lr2-dark-mode",
            "compatibility",
            "LR2 DARK MODE",
            Confidence::RecommendedConfirm,
            false,
            "Add OFF, DARK, and DARK2 skin variants using LR2 object-category and timer filtering while preserving an opaque black lane cover in DARK2.",
        ),
        manual_rule(
            "key-laser-rewrite",
            "compatibility",
            "Key laser asset rewrite",
            "Requires a skin-specific source-sheet and animation mapping.",
        ),
        manual_rule(
            "gauge-label-helper",
            "compatibility",
            "Gauge label asset helper",
            "Requires a skin-specific gauge-label sprite order mapping.",
        ),
        manual_rule(
            "lift-cover-custom-image",
            "lift",
            "Custom LIFT cover image",
            "Requires an explicitly selected image and source rectangle.",
        ),
    ]
}

pub fn preset_rule_ids(preset: &str) -> Vec<String> {
    let base = [
        "resolution-declaration",
        "theme-path-rebase",
        "asset-reference-repair",
        "follow-lift-op4",
        "key-laser-layer-compat",
        "judge-line-follow",
        "blend-fallback",
    ];
    let ids: Vec<&str> = match preset.to_ascii_lowercase().as_str() {
        "transparent-lift" | "askin" | "askin-like" => base
            .into_iter()
            .chain(["lift-cover-transparent", "green-number", "lift-number"])
            .collect(),
        "visible-lift" | "oadx" | "remi" | "beatgb" | "beatgb-like" => base
            .into_iter()
            .chain(["lift-cover-copy-sudden", "green-number", "lift-number"])
            .collect(),
        "fhd-4x3" => base.into_iter().chain(["aspect-ratio-convert"]).collect(),
        _ => base.to_vec(),
    };
    ids.into_iter().map(str::to_owned).collect()
}

fn rule(
    id: &str,
    category: &str,
    title: &str,
    confidence: Confidence,
    default_enabled: bool,
    description: &str,
) -> RuleDefinition {
    RuleDefinition {
        id: id.to_owned(),
        category: category.to_owned(),
        title: title.to_owned(),
        confidence,
        implemented: true,
        default_enabled,
        description: description.to_owned(),
    }
}

fn manual_rule(id: &str, category: &str, title: &str, description: &str) -> RuleDefinition {
    RuleDefinition {
        id: id.to_owned(),
        category: category.to_owned(),
        title: title.to_owned(),
        confidence: Confidence::RiskyManual,
        implemented: false,
        default_enabled: false,
        description: description.to_owned(),
    }
}
