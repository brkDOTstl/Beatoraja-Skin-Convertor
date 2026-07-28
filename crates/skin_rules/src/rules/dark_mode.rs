use std::{
    collections::{BTreeSet, VecDeque},
    ffi::OsString,
    path::{Path, PathBuf},
};

use skin_core::{CsvRecord, SkinFileKind, TextDocument, csv::trim_ascii};

use crate::{
    edit::{document_has_marker, insert_lines, replace_line, warn},
    model::{EditableFile, FileChange, GeneratedAsset, RuleContext, RuleWarning},
};

const RULE_ID: &str = "lr2-dark-mode";
const MARKER: &[u8] = b"//bms-skin-generator:R010:LR2_DARK_MODE";
const ROUTED_INCLUDE: &[u8] = b"//bms-skin-generator:R010:ROUTED_INCLUDE,";
const GENERATED_DIR: &str = "_bms-skin-generator/dark";
const CUSTOM_NAME: &str = "LR2 DARK MODE";
const DARK1_SUFFIX: &str = ".bmsg-dark1";
const DARK2_SUFFIX: &str = ".bmsg-dark2";
const BLACK_BOARD_PATH: &str = "_bms-skin-generator/dark/lane-cover-black.png";
const PLAY_TYPES: &[i32] = &[0, 1, 2, 3, 4, 12, 13, 14];
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DarkVariant {
    Off,
    Dark,
    Dark2,
}

impl DarkVariant {
    fn file_name(self) -> &'static str {
        match self {
            Self::Off => "OFF",
            Self::Dark => "DARK",
            Self::Dark2 => "DARK2",
        }
    }

    fn suffix(self) -> Option<&'static str> {
        match self {
            Self::Off => None,
            Self::Dark => Some(DARK1_SUFFIX),
            Self::Dark2 => Some(DARK2_SUFFIX),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ObjectAction {
    Keep,
    Hide,
    BlackLaneCover,
}

#[derive(Debug, Default)]
struct VariantStats {
    hidden_objects: usize,
    black_lane_covers: usize,
    lane_cover_fallbacks: usize,
    rewritten_includes: usize,
}

#[derive(Debug, Default)]
struct LaneCoverPlan {
    direct_images: BTreeSet<i32>,
    appended_image: Option<(usize, i32)>,
}

impl LaneCoverPlan {
    fn replacement_image(&self, source_image: i32) -> Option<i32> {
        self.appended_image
            .map(|(_, image_index)| image_index)
            .or_else(|| {
                self.direct_images
                    .contains(&source_image)
                    .then_some(source_image)
            })
    }

    fn replacement_images(&self) -> impl Iterator<Item = i32> + '_ {
        self.direct_images
            .iter()
            .copied()
            .chain(self.appended_image.map(|(_, image_index)| image_index))
    }

    fn has_replacement(&self) -> bool {
        self.appended_image.is_some() || !self.direct_images.is_empty()
    }
}

#[derive(Debug)]
struct HeaderPlan {
    file_index: usize,
    original: TextDocument,
    include_indices: Vec<usize>,
    selector_dir: PathBuf,
}

#[derive(Debug)]
struct CsvPathEntry {
    file_index: usize,
    normalized_path: String,
}

#[derive(Debug, Default)]
struct CsvPathIndex {
    entries: Vec<CsvPathEntry>,
}

impl CsvPathIndex {
    fn from_files(files: &[EditableFile]) -> Self {
        Self {
            entries: files
                .iter()
                .enumerate()
                .filter(|(_, file)| file.kind == SkinFileKind::Csv)
                .map(|(file_index, file)| CsvPathEntry {
                    file_index,
                    normalized_path: normalize_path(&file.relative_path.to_string_lossy()),
                })
                .collect(),
        }
    }

    fn matches(&self, include_path: &[u8], theme: &str) -> Option<Vec<usize>> {
        let (pattern, explicitly_external) = include_pattern(include_path, theme);
        let matches = self
            .entries
            .iter()
            .filter(|entry| wildcard_matches(&pattern, &entry.normalized_path))
            .map(|entry| entry.file_index)
            .collect::<Vec<_>>();
        if explicitly_external && matches.is_empty() {
            None
        } else {
            Some(matches)
        }
    }
}

pub(crate) fn apply(
    files: &mut [EditableFile],
    context: &RuleContext,
    generated_assets: &mut Vec<GeneratedAsset>,
    changes: &mut Vec<FileChange>,
    warnings: &mut Vec<RuleWarning>,
) {
    let Some(theme) = context.output_theme_dir.as_deref() else {
        warn(
            warnings,
            RULE_ID,
            None,
            "Could not determine the output theme folder for DARK mode routing.",
        );
        return;
    };

    let mut external_includes = BTreeSet::new();
    let csv_index = CsvPathIndex::from_files(files);
    let mut header_plans = Vec::new();
    for (file_index, file) in files
        .iter()
        .enumerate()
        .filter(|(_, file)| file.kind == SkinFileKind::Lr2Skin && is_play_header(&file.document))
    {
        if document_has_marker(&file.document, MARKER) {
            continue;
        }
        let original = file.document.clone();
        let include_indices = command_indices(&original, "#INCLUDE");
        if include_indices.is_empty() {
            warn(
                warnings,
                RULE_ID,
                Some(file),
                "Play header has no #INCLUDE route; inline draw objects were left unchanged.",
            );
            continue;
        }
        header_plans.push(HeaderPlan {
            file_index,
            original,
            include_indices,
            selector_dir: selector_directory(&file.relative_path),
        });
    }

    if header_plans.is_empty() {
        warn(
            warnings,
            RULE_ID,
            None,
            "No compatible LR2 Play header was available for DARK mode.",
        );
        return;
    }

    let reachable_csv = reachable_csv_files(
        files,
        &header_plans,
        &csv_index,
        theme,
        &mut external_includes,
    );
    if reachable_csv.is_empty() {
        warn(
            warnings,
            RULE_ID,
            None,
            "Play routes did not resolve to any local CSV files; DARK mode was not added.",
        );
        return;
    }

    if reachable_csv
        .iter()
        .any(|file_index| lane_cover_plan(&files[*file_index].document).has_replacement())
    {
        let relative_path = PathBuf::from(BLACK_BOARD_PATH);
        generated_assets.push(GeneratedAsset {
            relative_path: relative_path.clone(),
            bytes: BLACK_PNG.to_vec(),
        });
        changes.push(FileChange {
            rule_id: RULE_ID.to_owned(),
            path: relative_path,
            line: None,
            message: "generate opaque DARK2 lane-cover board".to_owned(),
            before: String::new(),
            after: "generated opaque 1x1 black PNG".to_owned(),
        });
    }

    for plan in &header_plans {
        let selector_dir = &plan.selector_dir;
        let selector_path = theme_path(theme, selector_dir, "*");
        for variant in [DarkVariant::Off, DarkVariant::Dark, DarkVariant::Dark2] {
            let bytes = routing_variant(
                &plan.original,
                variant,
                theme,
                &csv_index,
                &mut external_includes,
            );
            let relative_path = selector_dir.join(variant.file_name());
            generated_assets.push(GeneratedAsset {
                relative_path: relative_path.clone(),
                bytes,
            });
            changes.push(FileChange {
                rule_id: RULE_ID.to_owned(),
                path: relative_path,
                line: None,
                message: format!("generate {} routing file", variant.file_name()),
                before: String::new(),
                after: "generated mode-specific include routing".to_owned(),
            });
        }

        route_header(
            &mut files[plan.file_index],
            &plan.include_indices,
            &selector_path,
            changes,
        );
    }

    for file_index in reachable_csv {
        let file = &files[file_index];
        for variant in [DarkVariant::Dark, DarkVariant::Dark2] {
            let (bytes, stats) = build_csv_variant(
                &file.document,
                variant,
                theme,
                &csv_index,
                &mut external_includes,
            );
            let relative_path = suffixed_path(
                &file.relative_path,
                variant
                    .suffix()
                    .expect("only DARK variants generate CSV copies"),
            );
            generated_assets.push(GeneratedAsset {
                relative_path: relative_path.clone(),
                bytes,
            });
            changes.push(FileChange {
                rule_id: RULE_ID.to_owned(),
                path: relative_path,
                line: None,
                message: format!("generate {} CSV variant", variant.file_name()),
                before: String::new(),
                after: format!(
                    "hidden objects: {}; black lane covers: {}; lane-cover fallbacks: {}; rewritten includes: {}",
                    stats.hidden_objects,
                    stats.black_lane_covers,
                    stats.lane_cover_fallbacks,
                    stats.rewritten_includes
                ),
            });
            if stats.lane_cover_fallbacks > 0 {
                warn(
                    warnings,
                    RULE_ID,
                    Some(file),
                    format!(
                        "{} DARK2 lane-cover slider(s) could not be bound to an exclusive local image and use black tinting instead.",
                        stats.lane_cover_fallbacks
                    ),
                );
            }
        }
    }

    if !external_includes.is_empty() {
        warn(
            warnings,
            RULE_ID,
            None,
            format!(
                "{} external or unresolved include path(s) were preserved without DARK filtering: {}",
                external_includes.len(),
                external_includes
                    .iter()
                    .take(3)
                    .map(|path| String::from_utf8_lossy(path).into_owned())
                    .collect::<Vec<_>>()
                    .join("; ")
            ),
        );
    }
}

fn reachable_csv_files(
    files: &[EditableFile],
    header_plans: &[HeaderPlan],
    csv_index: &CsvPathIndex,
    theme: &str,
    external_includes: &mut BTreeSet<Vec<u8>>,
) -> BTreeSet<usize> {
    let mut reachable = BTreeSet::new();
    let mut queue = VecDeque::new();

    for plan in header_plans {
        enqueue_document_includes(
            &plan.original,
            csv_index,
            theme,
            external_includes,
            &mut reachable,
            &mut queue,
        );
    }

    while let Some(file_index) = queue.pop_front() {
        enqueue_document_includes(
            &files[file_index].document,
            csv_index,
            theme,
            external_includes,
            &mut reachable,
            &mut queue,
        );
    }

    reachable
}

fn enqueue_document_includes(
    document: &TextDocument,
    csv_index: &CsvPathIndex,
    theme: &str,
    external_includes: &mut BTreeSet<Vec<u8>>,
    reachable: &mut BTreeSet<usize>,
    queue: &mut VecDeque<usize>,
) {
    for index in command_indices(document, "#INCLUDE") {
        let Some(content) = document.line_content(index) else {
            continue;
        };
        let record = CsvRecord::from_line(index, content);
        let Some(path) = record.field(1).map(trim_ascii) else {
            continue;
        };
        if path.is_empty() {
            continue;
        }
        let Some(matches) = csv_index.matches(path, theme) else {
            external_includes.insert(path.to_vec());
            continue;
        };
        if matches.is_empty() {
            external_includes.insert(path.to_vec());
            continue;
        }
        for file_index in matches {
            if reachable.insert(file_index) {
                queue.push_back(file_index);
            }
        }
    }
}

fn is_play_header(document: &TextDocument) -> bool {
    (0..document.line_count()).any(|index| {
        let Some(content) = document.line_content(index) else {
            return false;
        };
        let record = CsvRecord::from_line(index, content);
        record.is_command("#INFORMATION")
            && record
                .field_i32(1)
                .is_some_and(|skin_type| PLAY_TYPES.contains(&skin_type))
    })
}

fn command_indices(document: &TextDocument, command: &str) -> Vec<usize> {
    (0..document.line_count())
        .filter(|index| {
            document
                .line_content(*index)
                .is_some_and(|content| CsvRecord::from_line(*index, content).is_command(command))
        })
        .collect()
}

fn selector_directory(header_path: &Path) -> PathBuf {
    let normalized = header_path
        .to_string_lossy()
        .replace('\\', "/")
        .to_ascii_lowercase();
    let mut hash = 0xcbf2_9ce4_8422_2325u64;
    for byte in normalized.as_bytes() {
        hash ^= u64::from(*byte);
        hash = hash.wrapping_mul(0x0000_0100_0000_01b3);
    }
    Path::new(GENERATED_DIR).join(format!("header-{hash:016x}"))
}

fn theme_path(theme: &str, relative_dir: &Path, file: &str) -> String {
    let relative = relative_dir.to_string_lossy().replace('/', "\\");
    format!("LR2files\\Theme\\{theme}\\{relative}\\{file}")
}

fn route_header(
    file: &mut EditableFile,
    include_indices: &[usize],
    selector_path: &str,
    changes: &mut Vec<FileChange>,
) {
    for index in include_indices {
        let Some(before) = file.document.line_content(*index).map(Vec::from) else {
            continue;
        };
        let mut after = ROUTED_INCLUDE.to_vec();
        after.extend_from_slice(&before);
        replace_line(
            file,
            *index,
            before,
            after,
            RULE_ID,
            "route original include through LR2 DARK MODE",
            changes,
        );
    }

    let insert_at = command_indices(&file.document, "#ENDOFHEADER")
        .into_iter()
        .next()
        .or_else(|| {
            (0..file.document.line_count()).find(|index| {
                file.document.line_content(*index).is_some_and(|content| {
                    let record = CsvRecord::from_line(*index, content);
                    record.is_command("#IF")
                        || record.is_command("#ELSEIF")
                        || record.is_command("#ELSE")
                        || record.is_command("#INCLUDE")
                })
            })
        })
        .unwrap_or(file.document.line_count());
    insert_lines(
        file,
        insert_at,
        [
            MARKER.to_vec(),
            format!("#CUSTOMFILE,{CUSTOM_NAME},{selector_path},OFF").into_bytes(),
        ],
        RULE_ID,
        changes,
    );
    let append_at = file.document.line_count();
    insert_lines(
        file,
        append_at,
        [
            b"//bms-skin-generator:R010:LR2_DARK_MODE_ROUTE".to_vec(),
            format!("#INCLUDE,{selector_path}").into_bytes(),
        ],
        RULE_ID,
        changes,
    );
}

fn routing_variant(
    document: &TextDocument,
    variant: DarkVariant,
    theme: &str,
    csv_index: &CsvPathIndex,
    external_includes: &mut BTreeSet<Vec<u8>>,
) -> Vec<u8> {
    let mut output = Vec::new();
    push_generated_line(
        &mut output,
        format!("//bms-skin-generator:R010:{}_ROUTING", variant.file_name()).as_bytes(),
        document,
    );
    for (index, line) in document.lines().iter().enumerate() {
        let mut record = CsvRecord::from_line(index, &line.content);
        let Some(command) = record.command() else {
            continue;
        };
        if !matches!(
            command.as_str(),
            "#IF" | "#ELSEIF" | "#ELSE" | "#ENDIF" | "#SETOPTION" | "#INCLUDE"
        ) {
            continue;
        }
        if record.is_command("#INCLUDE")
            && let Some(suffix) = variant.suffix()
        {
            rewrite_include(&mut record, suffix, theme, csv_index, external_includes);
        }
        output.extend_from_slice(&record.to_line());
        output.extend_from_slice(
            line.ending
                .unwrap_or(document.dominant_line_ending())
                .as_bytes(),
        );
    }
    output
}

fn push_generated_line(output: &mut Vec<u8>, content: &[u8], document: &TextDocument) {
    output.extend_from_slice(content);
    output.extend_from_slice(document.dominant_line_ending().as_bytes());
}

fn build_csv_variant(
    document: &TextDocument,
    variant: DarkVariant,
    theme: &str,
    csv_index: &CsvPathIndex,
    external_includes: &mut BTreeSet<Vec<u8>>,
) -> (Vec<u8>, VariantStats) {
    let mut transformed = document.clone();
    let mut active = ObjectAction::Keep;
    let mut active_destination = None;
    let mut stats = VariantStats::default();
    let judge_line_images = judge_line_image_sources(document);
    let lane_cover_plan = if variant == DarkVariant::Dark2 {
        lane_cover_plan_with_protected_images(document, &judge_line_images)
    } else {
        LaneCoverPlan::default()
    };
    let black_images = lane_cover_plan
        .replacement_images()
        .collect::<BTreeSet<_>>();
    if !black_images.is_empty() {
        let asset_path = BLACK_BOARD_PATH.replace('/', "\\");
        for (image_index, line_index) in command_indices(document, "#IMAGE").into_iter().enumerate()
        {
            if !black_images.contains(&(image_index as i32)) {
                continue;
            }
            let Some(content) = document.line_content(line_index) else {
                continue;
            };
            let mut image = CsvRecord::from_line(line_index, content);
            image.set_field_ascii(1, &format!("LR2files\\Theme\\{theme}\\{asset_path}"));
            let _ = transformed.replace_line_content(line_index, image.to_line());
        }
    }

    for index in 0..document.line_count() {
        let Some(before) = document.line_content(index).map(Vec::from) else {
            continue;
        };
        let mut record = CsvRecord::from_line(index, &before);
        let Some(command) = record.command() else {
            continue;
        };

        if record.is_command("#INCLUDE")
            && let Some(suffix) = variant.suffix()
            && rewrite_include(&mut record, suffix, theme, csv_index, external_includes)
        {
            let _ = transformed.replace_line_content(index, record.to_line());
            stats.rewritten_includes = stats.rewritten_includes.saturating_add(1);
            continue;
        }

        if command.starts_with("#SRC_") {
            active = source_action(
                &record,
                variant,
                effective_image_timer(document, index, &record),
                judge_line_images.contains(&index),
            );
            active_destination = destination_for_source(&command);
            match active {
                ObjectAction::Hide => {
                    stats.hidden_objects = stats.hidden_objects.saturating_add(1);
                }
                ObjectAction::BlackLaneCover => {
                    if let Some(image_index) = record
                        .field_i32(2)
                        .and_then(|image_index| lane_cover_plan.replacement_image(image_index))
                    {
                        record.set_field_ascii(2, &image_index.to_string());
                        blacken_lane_cover_source(&mut record);
                        let _ = transformed.replace_line_content(index, record.to_line());
                    } else {
                        stats.lane_cover_fallbacks = stats.lane_cover_fallbacks.saturating_add(1);
                    }
                    stats.black_lane_covers = stats.black_lane_covers.saturating_add(1);
                }
                ObjectAction::Keep => {}
            }
            continue;
        }

        if !command.starts_with("#DST_") {
            continue;
        }
        if active_destination.is_none_or(|destination| !record.is_command(destination)) {
            continue;
        }
        match active {
            ObjectAction::Hide => {
                make_destination_transparent(&mut record);
                let _ = transformed.replace_line_content(index, record.to_line());
            }
            ObjectAction::BlackLaneCover
                if record.is_command("#DST_SLIDER") || record.is_command("#DST_LIFT") =>
            {
                blacken_lane_cover_destination(&mut record);
                let _ = transformed.replace_line_content(index, record.to_line());
            }
            ObjectAction::Keep | ObjectAction::BlackLaneCover => {}
        }
    }

    if let Some((line_index, image_index)) = lane_cover_plan.appended_image {
        let asset_path = BLACK_BOARD_PATH.replace('/', "\\");
        transformed.insert_generated_line(
            line_index,
            format!("#IMAGE,LR2files\\Theme\\{theme}\\{asset_path},,,,,//{image_index}")
                .into_bytes(),
        );
    }

    (transformed.to_bytes(), stats)
}

fn lane_cover_plan(document: &TextDocument) -> LaneCoverPlan {
    let protected_images = judge_line_image_sources(document);
    lane_cover_plan_with_protected_images(document, &protected_images)
}

fn lane_cover_plan_with_protected_images(
    document: &TextDocument,
    protected_images: &BTreeSet<usize>,
) -> LaneCoverPlan {
    let image_lines = command_indices(document, "#IMAGE");
    let image_count = image_lines.len() as i32;
    let mut lane_cover_images = BTreeSet::new();
    let mut other_source_images = BTreeSet::new();

    for index in 0..document.line_count() {
        let Some(content) = document.line_content(index) else {
            continue;
        };
        let source = CsvRecord::from_line(index, content);
        if !source
            .command()
            .is_some_and(|command| command.starts_with("#SRC_"))
        {
            continue;
        }
        let Some(image_index) = source.field_i32(2) else {
            continue;
        };
        if is_lane_cover_source(&source) {
            lane_cover_images.insert(image_index);
        } else if source_action(
            &source,
            DarkVariant::Dark2,
            effective_image_timer(document, index, &source),
            protected_images.contains(&index),
        ) != ObjectAction::Hide
        {
            other_source_images.insert(image_index);
        }
    }

    lane_cover_images.retain(|image_index| {
        (0..image_count).contains(image_index) && !other_source_images.contains(image_index)
    });

    if let Some(line_index) = static_image_append_position(document, &image_lines) {
        LaneCoverPlan {
            direct_images: BTreeSet::new(),
            appended_image: Some((line_index, image_count)),
        }
    } else {
        LaneCoverPlan {
            direct_images: lane_cover_images,
            appended_image: None,
        }
    }
}

fn static_image_append_position(document: &TextDocument, image_lines: &[usize]) -> Option<usize> {
    let first_source = (0..document.line_count()).find(|index| {
        document.line_content(*index).is_some_and(|content| {
            CsvRecord::from_line(*index, content)
                .command()
                .is_some_and(|command| command.starts_with("#SRC_"))
        })
    })?;
    if image_lines
        .iter()
        .any(|line_index| *line_index >= first_source)
    {
        return None;
    }

    let mut conditional_depth = 0usize;
    for index in 0..first_source {
        let Some(content) = document.line_content(index) else {
            continue;
        };
        let record = CsvRecord::from_line(index, content);
        if record.is_command("#IF") {
            conditional_depth = conditional_depth.saturating_add(1);
        }
        if record.is_command("#IMAGE") && conditional_depth != 0 {
            return None;
        }
        if record.is_command("#ENDIF") {
            conditional_depth = conditional_depth.saturating_sub(1);
        }
    }
    if conditional_depth != 0 {
        return None;
    }

    Some(
        image_lines
            .last()
            .map_or(first_source, |line_index| line_index + 1),
    )
}

fn is_lane_cover_source(source: &CsvRecord) -> bool {
    source.is_command("#SRC_LIFT")
        || (source.is_command("#SRC_SLIDER")
            && source
                .field_i32(13)
                .is_some_and(|value_id| value_id == 4 || value_id == 5))
}

fn is_lane_cover_number(source: &CsvRecord) -> bool {
    source.is_command("#SRC_NUMBER")
        && source
            .field_i32(11)
            .is_some_and(|num| matches!(num, 14 | 15 | 313 | 314))
}

fn source_action(
    source: &CsvRecord,
    variant: DarkVariant,
    effective_image_timer: Option<i32>,
    protected_judge_line: bool,
) -> ObjectAction {
    if protected_judge_line {
        return ObjectAction::Keep;
    }

    if source.is_command("#SRC_IMAGE") {
        let timer = effective_image_timer.unwrap_or_else(|| source.field_i32(10).unwrap_or(0));
        return match variant {
            DarkVariant::Off => ObjectAction::Keep,
            DarkVariant::Dark => {
                if timer == 0 {
                    ObjectAction::Hide
                } else {
                    ObjectAction::Keep
                }
            }
            DarkVariant::Dark2 => {
                if dark2_keeps_timer(timer) {
                    ObjectAction::Keep
                } else {
                    ObjectAction::Hide
                }
            }
        };
    }

    if is_lane_cover_source(source) {
        return if variant == DarkVariant::Dark2 {
            ObjectAction::BlackLaneCover
        } else {
            ObjectAction::Keep
        };
    }

    if is_lane_cover_number(source) {
        return ObjectAction::Keep;
    }

    let secondary = matches!(
        source.command().as_deref(),
        Some("#SRC_NUMBER" | "#SRC_SLIDER" | "#SRC_BARGRAPH" | "#SRC_BUTTON" | "#SRC_TEXT")
    );
    if secondary && variant == DarkVariant::Dark2 {
        ObjectAction::Hide
    } else {
        ObjectAction::Keep
    }
}

fn judge_line_image_sources(document: &TextDocument) -> BTreeSet<usize> {
    let mut source_signatures = BTreeSet::new();
    let mut destination_signatures = BTreeSet::new();
    let mut protected = BTreeSet::new();

    for index in 0..document.line_count() {
        let Some(content) = document.line_content(index) else {
            continue;
        };
        let record = CsvRecord::from_line(index, content);
        if record.is_command("#SRC_IMAGE")
            && content
                .windows(super::judge_line::IMAGE_FALLBACK_MARKER.len())
                .any(|window| window == super::judge_line::IMAGE_FALLBACK_MARKER)
        {
            protected.insert(index);
        }
        if record.is_command("#SRC_JUDGELINE")
            && let Some(signature) = source_signature(&record)
        {
            source_signatures.insert(signature);
        }
        if record.is_command("#DST_JUDGELINE")
            && let Some(signature) = destination_signature(&record)
        {
            destination_signatures.insert(signature);
        }
    }

    if source_signatures.is_empty() || destination_signatures.is_empty() {
        return protected;
    }

    for source_index in 0..document.line_count() {
        let Some(content) = document.line_content(source_index) else {
            continue;
        };
        let source = CsvRecord::from_line(source_index, content);
        if !source.is_command("#SRC_IMAGE")
            || source_signature(&source)
                .is_none_or(|signature| !source_signatures.contains(&signature))
        {
            continue;
        }

        for destination_index in source_index.saturating_add(1)..document.line_count() {
            let Some(content) = document.line_content(destination_index) else {
                continue;
            };
            let destination = CsvRecord::from_line(destination_index, content);
            if destination
                .command()
                .is_some_and(|command| command.starts_with("#SRC_"))
            {
                break;
            }
            if destination.is_command("#DST_IMAGE")
                && destination_signature(&destination)
                    .is_some_and(|signature| destination_signatures.contains(&signature))
            {
                protected.insert(source_index);
                break;
            }
        }
    }
    protected
}

fn source_signature(record: &CsvRecord) -> Option<[i32; 9]> {
    Some([
        record.field_i32(1)?,
        record.field_i32(2)?,
        record.field_i32(3)?,
        record.field_i32(4)?,
        record.field_i32(5)?,
        record.field_i32(6)?,
        record.field_i32(7)?,
        record.field_i32(8)?,
        record.field_i32(9)?,
    ])
}

fn destination_signature(record: &CsvRecord) -> Option<[i32; 5]> {
    Some([
        record.field_i32(1)?,
        record.field_i32(3)?,
        record.field_i32(4)?,
        record.field_i32(5)?,
        record.field_i32(6)?,
    ])
}

fn effective_image_timer(
    document: &TextDocument,
    source_index: usize,
    source: &CsvRecord,
) -> Option<i32> {
    if !source.is_command("#SRC_IMAGE") {
        return None;
    }
    let source_timer = source.field_i32(10).unwrap_or(0);
    if source_timer != 0 {
        return Some(source_timer);
    }

    let mut destination_timer = None;
    for index in source_index.saturating_add(1)..document.line_count() {
        let Some(content) = document.line_content(index) else {
            continue;
        };
        let record = CsvRecord::from_line(index, content);
        if record
            .command()
            .is_some_and(|command| command.starts_with("#SRC_"))
        {
            break;
        }
        if !record.is_command("#DST_IMAGE") {
            continue;
        }
        let timer = record.field_i32(17).unwrap_or(0);
        if timer == 140 {
            return Some(timer);
        }
        if timer != 0 && destination_timer.is_none() {
            destination_timer = Some(timer);
        }
    }
    Some(destination_timer.unwrap_or(source_timer))
}

fn dark2_keeps_timer(timer: i32) -> bool {
    matches!(timer, 2 | 3 | 48 | 49 | 140) || (50..=89).contains(&timer)
}

fn destination_for_source(source: &str) -> Option<&'static str> {
    match source {
        "#SRC_IMAGE" => Some("#DST_IMAGE"),
        "#SRC_NUMBER" => Some("#DST_NUMBER"),
        "#SRC_SLIDER" | "#SRC_SLIDER_REFNUMBER" => Some("#DST_SLIDER"),
        "#SRC_LIFT" => Some("#DST_LIFT"),
        "#SRC_BARGRAPH" | "#SRC_BARGRAPH_REFNUMBER" => Some("#DST_BARGRAPH"),
        "#SRC_BUTTON" => Some("#DST_BUTTON"),
        "#SRC_TEXT" => Some("#DST_TEXT"),
        _ => None,
    }
}

fn make_destination_transparent(record: &mut CsvRecord) {
    record.set_field_ascii(8, "0");
}

fn blacken_lane_cover_destination(record: &mut CsvRecord) {
    record.set_field_ascii(8, "255");
    record.set_field_ascii(9, "0");
    record.set_field_ascii(10, "0");
    record.set_field_ascii(11, "0");
    record.set_field_ascii(12, "0");
}

fn blacken_lane_cover_source(record: &mut CsvRecord) {
    record.set_field_ascii(3, "0");
    record.set_field_ascii(4, "0");
    record.set_field_ascii(5, "1");
    record.set_field_ascii(6, "1");
    record.set_field_ascii(7, "1");
    record.set_field_ascii(8, "1");
    record.set_field_ascii(9, "0");
}

fn rewrite_include(
    record: &mut CsvRecord,
    suffix: &str,
    theme: &str,
    csv_index: &CsvPathIndex,
    external_includes: &mut BTreeSet<Vec<u8>>,
) -> bool {
    let Some(field) = record.field(1).map(Vec::from) else {
        return false;
    };
    let path = trim_ascii(&field);
    if path.is_empty() {
        return false;
    }
    let Some(matches) = csv_index.matches(path, theme) else {
        external_includes.insert(path.to_vec());
        return false;
    };
    if matches.is_empty() {
        external_includes.insert(path.to_vec());
        return false;
    }

    let leading = field.len().saturating_sub(field.trim_ascii_start().len());
    let trailing = field.len().saturating_sub(field.trim_ascii_end().len());
    let mut rewritten = Vec::with_capacity(field.len() + suffix.len());
    rewritten.extend_from_slice(&field[..leading]);
    rewritten.extend_from_slice(path);
    rewritten.extend_from_slice(suffix.as_bytes());
    if trailing > 0 {
        rewritten.extend_from_slice(&field[field.len() - trailing..]);
    }
    record.fields[1] = rewritten;
    true
}

fn include_pattern(path: &[u8], theme: &str) -> (String, bool) {
    let normalized = normalize_path(&String::from_utf8_lossy(path));
    let prefix = "lr2files/theme/";
    if let Some(start) = normalized.find(prefix) {
        let remainder = &normalized[start + prefix.len()..];
        let (included_theme, relative) = remainder.split_once('/').unwrap_or((remainder, ""));
        (relative.to_owned(), included_theme != normalize_path(theme))
    } else {
        (
            normalized
                .trim_start_matches("./")
                .trim_start_matches('/')
                .to_owned(),
            false,
        )
    }
}

fn normalize_path(path: &str) -> String {
    path.trim().replace('\\', "/").to_ascii_lowercase()
}

fn wildcard_matches(pattern: &str, value: &str) -> bool {
    let pattern = pattern.as_bytes();
    let value = value.as_bytes();
    let (mut pattern_index, mut value_index) = (0usize, 0usize);
    let (mut wildcard_index, mut wildcard_value_index) = (None, 0usize);

    while value_index < value.len() {
        if pattern_index < pattern.len()
            && (pattern[pattern_index] == b'?' || pattern[pattern_index] == value[value_index])
        {
            pattern_index += 1;
            value_index += 1;
        } else if pattern_index < pattern.len() && pattern[pattern_index] == b'*' {
            wildcard_index = Some(pattern_index);
            wildcard_value_index = value_index;
            pattern_index += 1;
        } else if let Some(index) = wildcard_index {
            pattern_index = index + 1;
            wildcard_value_index += 1;
            value_index = wildcard_value_index;
        } else {
            return false;
        }
    }

    while pattern_index < pattern.len() && pattern[pattern_index] == b'*' {
        pattern_index += 1;
    }
    pattern_index == pattern.len()
}

fn suffixed_path(path: &Path, suffix: &str) -> PathBuf {
    let mut value = OsString::from(path.as_os_str());
    value.push(suffix);
    PathBuf::from(value)
}
