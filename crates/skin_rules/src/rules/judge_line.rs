use std::{
    collections::{BTreeMap, BTreeSet},
    path::{Path, PathBuf},
};

use skin_core::{CsvRecord, SkinFileKind};

use crate::{
    edit::{DST_OP4_FIELD, insert_lines, replace_record, warn},
    model::{EditableFile, FileChange, GeneratedAsset, RuleContext, RuleWarning},
};

const RULE_ID: &str = "judge-line-follow";
const GENERATED_ASSET_DIR: &str = "_bms-skin-generator";
const BLACK_REFERENCE: i32 = 110;
const WHITE_REFERENCE: i32 = 111;
pub(crate) const IMAGE_FALLBACK_MARKER: &[u8] = b"//bms-skin-generator:JUDGE_LINE_IMAGE_FALLBACK";

// Opaque 1x1 RGBA PNGs. LR2's gr=110/111 references represent these two colors,
// but beatoraja cannot resolve those references from SRC_JUDGELINE.
const BLACK_PNG: &[u8] = &[
    0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a, 0x00, 0x00, 0x00, 0x0d, 0x49, 0x48, 0x44, 0x52,
    0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, 0x08, 0x06, 0x00, 0x00, 0x00, 0x1f, 0x15, 0xc4,
    0x89, 0x00, 0x00, 0x00, 0x01, 0x73, 0x52, 0x47, 0x42, 0x00, 0xae, 0xce, 0x1c, 0xe9, 0x00, 0x00,
    0x00, 0x04, 0x67, 0x41, 0x4d, 0x41, 0x00, 0x00, 0xb1, 0x8f, 0x0b, 0xfc, 0x61, 0x05, 0x00, 0x00,
    0x00, 0x09, 0x70, 0x48, 0x59, 0x73, 0x00, 0x00, 0x0e, 0xc3, 0x00, 0x00, 0x0e, 0xc3, 0x01, 0xc7,
    0x6f, 0xa8, 0x64, 0x00, 0x00, 0x00, 0x0d, 0x49, 0x44, 0x41, 0x54, 0x18, 0x57, 0x63, 0x60, 0x60,
    0x60, 0xf8, 0x0f, 0x00, 0x01, 0x04, 0x01, 0x00, 0x70, 0x20, 0x65, 0x0b, 0x00, 0x00, 0x00, 0x00,
    0x49, 0x45, 0x4e, 0x44, 0xae, 0x42, 0x60, 0x82,
];
const WHITE_PNG: &[u8] = &[
    0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a, 0x00, 0x00, 0x00, 0x0d, 0x49, 0x48, 0x44, 0x52,
    0x00, 0x00, 0x00, 0x01, 0x00, 0x00, 0x00, 0x01, 0x08, 0x06, 0x00, 0x00, 0x00, 0x1f, 0x15, 0xc4,
    0x89, 0x00, 0x00, 0x00, 0x01, 0x73, 0x52, 0x47, 0x42, 0x00, 0xae, 0xce, 0x1c, 0xe9, 0x00, 0x00,
    0x00, 0x04, 0x67, 0x41, 0x4d, 0x41, 0x00, 0x00, 0xb1, 0x8f, 0x0b, 0xfc, 0x61, 0x05, 0x00, 0x00,
    0x00, 0x09, 0x70, 0x48, 0x59, 0x73, 0x00, 0x00, 0x0e, 0xc3, 0x00, 0x00, 0x0e, 0xc3, 0x01, 0xc7,
    0x6f, 0xa8, 0x64, 0x00, 0x00, 0x00, 0x0d, 0x49, 0x44, 0x41, 0x54, 0x18, 0x57, 0x63, 0xf8, 0xff,
    0xff, 0xff, 0x7f, 0x00, 0x09, 0xfb, 0x03, 0xfd, 0x05, 0x43, 0x45, 0xca, 0x00, 0x00, 0x00, 0x00,
    0x49, 0x45, 0x4e, 0x44, 0xae, 0x42, 0x60, 0x82,
];

pub(crate) fn apply(
    files: &mut [EditableFile],
    context: &RuleContext,
    generated_assets: &mut Vec<GeneratedAsset>,
    changes: &mut Vec<FileChange>,
    warnings: &mut Vec<RuleWarning>,
) {
    let required_references = required_solid_references(files);
    let asset_paths = materialize_references(
        &required_references,
        context,
        generated_assets,
        changes,
        warnings,
    );

    for file in files
        .iter_mut()
        .filter(|file| file.kind == SkinFileKind::Csv)
    {
        let mappings = add_image_declarations(file, context, &asset_paths, changes);
        update_judge_lines(file, &mappings, changes);
    }
}

fn required_solid_references(files: &[EditableFile]) -> BTreeSet<i32> {
    let mut required = BTreeSet::new();
    for file in files.iter().filter(|file| file.kind == SkinFileKind::Csv) {
        for index in 0..file.document.line_count() {
            let Some(content) = file.document.line_content(index) else {
                continue;
            };
            let record = CsvRecord::from_line(index, content);
            if record.is_command("#SRC_JUDGELINE")
                && record.field_i32(2).is_some_and(is_solid_reference)
            {
                required.insert(record.field_i32(2).unwrap_or_default());
            }
        }
    }
    required
}

fn materialize_references(
    required: &BTreeSet<i32>,
    context: &RuleContext,
    generated_assets: &mut Vec<GeneratedAsset>,
    changes: &mut Vec<FileChange>,
    warnings: &mut Vec<RuleWarning>,
) -> BTreeMap<i32, PathBuf> {
    let mut paths = BTreeMap::new();
    if required.is_empty() {
        return paths;
    }
    if context.output_theme_dir.is_none() {
        warn(
            warnings,
            RULE_ID,
            None,
            "Could not determine the output theme folder; solid judge-line references use the image-object fallback.",
        );
        return paths;
    }

    for reference in required {
        let (name, bytes, color_name) = match *reference {
            BLACK_REFERENCE => ("judgeline-black.png", BLACK_PNG, "black"),
            WHITE_REFERENCE => ("judgeline-white.png", WHITE_PNG, "white"),
            _ => continue,
        };
        let relative_path = reserve_asset_path(context, generated_assets, name);
        generated_assets.push(GeneratedAsset {
            relative_path: relative_path.clone(),
            bytes: bytes.to_vec(),
        });
        changes.push(FileChange {
            rule_id: RULE_ID.to_owned(),
            path: relative_path.clone(),
            line: None,
            message: format!(
                "materialize LR2 gr={reference} as a 1x1 {color_name} judge-line asset"
            ),
            before: String::new(),
            after: format!("generated opaque 1x1 {color_name} PNG"),
        });
        paths.insert(*reference, relative_path);
    }
    paths
}

fn reserve_asset_path(
    context: &RuleContext,
    generated_assets: &[GeneratedAsset],
    file_name: &str,
) -> PathBuf {
    let stem = file_name.strip_suffix(".png").unwrap_or(file_name);
    for suffix in 0usize.. {
        let candidate_name = if suffix == 0 {
            file_name.to_owned()
        } else {
            format!("{stem}-{suffix}.png")
        };
        let candidate = Path::new(GENERATED_ASSET_DIR).join(candidate_name);
        let reserved = generated_assets
            .iter()
            .any(|asset| asset.relative_path == candidate);
        if !reserved && !context.output_root.join(&candidate).exists() {
            return candidate;
        }
    }
    unreachable!("an unused generated asset name must exist")
}

fn add_image_declarations(
    file: &mut EditableFile,
    context: &RuleContext,
    asset_paths: &BTreeMap<i32, PathBuf>,
    changes: &mut Vec<FileChange>,
) -> BTreeMap<i32, usize> {
    let Some(theme) = context.output_theme_dir.as_deref() else {
        return BTreeMap::new();
    };
    let mut image_count = 0usize;
    let mut last_image = None;
    let mut first_special_source = None;
    let mut needed = BTreeSet::new();

    for index in 0..file.document.line_count() {
        let Some(content) = file.document.line_content(index) else {
            continue;
        };
        let record = CsvRecord::from_line(index, content);
        if record.is_command("#IMAGE") {
            image_count = image_count.saturating_add(1);
            last_image = Some(index);
        }
        if record.is_command("#SRC_JUDGELINE")
            && let Some(reference) = record.field_i32(2)
            && asset_paths.contains_key(&reference)
        {
            needed.insert(reference);
            first_special_source.get_or_insert(index);
        }
    }
    if needed.is_empty() {
        return BTreeMap::new();
    }

    let mut mappings = BTreeMap::new();
    let mut declarations = Vec::new();
    for reference in needed {
        let image_index = image_count + mappings.len();
        let asset_path = asset_paths
            .get(&reference)
            .expect("required judge-line asset path");
        let relative = asset_path.to_string_lossy().replace('/', "\\");
        declarations.push(
            format!("#IMAGE,LR2files\\Theme\\{theme}\\{relative},,,,,//{image_index}").into_bytes(),
        );
        mappings.insert(reference, image_index);
    }
    let insert_at = last_image
        .map(|index| index + 1)
        .or(first_special_source)
        .unwrap_or(file.document.line_count());
    insert_lines(file, insert_at, declarations, RULE_ID, changes);
    mappings
}

fn update_judge_lines(
    file: &mut EditableFile,
    mappings: &BTreeMap<i32, usize>,
    changes: &mut Vec<FileChange>,
) {
    let mut image_fallback_group = false;
    for index in 0..file.document.line_count() {
        let Some(content) = file.document.line_content(index).map(Vec::from) else {
            continue;
        };
        let mut record = CsvRecord::from_line(index, &content);

        if record.is_command("#SRC_JUDGELINE") {
            let reference = record.field_i32(2);
            image_fallback_group = reference.is_some_and(|value| value >= 100)
                && !reference.is_some_and(|value| mappings.contains_key(&value));
            if let Some(image_index) = reference.and_then(|value| mappings.get(&value)) {
                record.set_field_ascii(2, &image_index.to_string());
                replace_record(
                    file,
                    index,
                    content,
                    record,
                    RULE_ID,
                    "bind solid reference to generated judge-line asset",
                    changes,
                );
            } else if image_fallback_group {
                record.set_field_ascii(0, "#SRC_IMAGE");
                record.fields.push(IMAGE_FALLBACK_MARKER.to_vec());
                replace_record(
                    file,
                    index,
                    content,
                    record,
                    RULE_ID,
                    "use image-object fallback for non-materialized reference",
                    changes,
                );
            }
            continue;
        }

        if record
            .command()
            .is_some_and(|command| command.starts_with("#SRC_"))
        {
            image_fallback_group = false;
            continue;
        }
        if !record.is_command("#DST_JUDGELINE") {
            continue;
        }
        if image_fallback_group {
            record.set_field_ascii(0, "#DST_IMAGE");
        }
        record.set_field_ascii(DST_OP4_FIELD, "3");
        replace_record(
            file,
            index,
            content,
            record,
            RULE_ID,
            if image_fallback_group {
                "use image-object fallback and move destination with LIFT"
            } else {
                "move judge-line destination with LIFT"
            },
            changes,
        );
    }
}

fn is_solid_reference(reference: i32) -> bool {
    matches!(reference, BLACK_REFERENCE | WHITE_REFERENCE)
}
