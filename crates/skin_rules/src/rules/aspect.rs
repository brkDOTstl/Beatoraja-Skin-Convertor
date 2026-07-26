use std::collections::BTreeMap;

use skin_core::{CsvRecord, FollowLiftKind, canvas_for_resolution, follow_lift_kind};

use crate::{
    edit::{is_play_file, replace_record, warn},
    model::{ConversionOptions, EditableFile, FileChange, RuleContext, RuleWarning},
    rules::{
        aspect_schema::{
            DestinationGeometry, destination_geometry, slider_range_axis, source_dimensions,
            source_vertical_cutoff,
        },
        aspect_transform::{
            SymmetricHorizontalFrame, preserve_symmetric_horizontal_frame, scale,
            transform_geometry, transform_source_dimensions, transform_source_vertical_cutoff,
        },
        geometry::judgment_geometries,
    },
};

pub(crate) fn apply(
    files: &mut [EditableFile],
    context: &RuleContext,
    options: &ConversionOptions,
    changes: &mut Vec<FileChange>,
    warnings: &mut Vec<RuleWarning>,
) {
    let Some(transform) = LayoutTransform::from_options(context, options) else {
        warn(
            warnings,
            "aspect-ratio-convert",
            None,
            "Source or target canvas dimensions could not be determined.",
        );
        return;
    };

    for file in files {
        let symmetric_frames = symmetric_horizontal_frames(file);
        let mut bomb_group = false;
        for index in 0..file.document.line_count() {
            let Some(content) = file.document.line_content(index).map(Vec::from) else {
                continue;
            };
            let mut record = CsvRecord::from_line(index, &content);
            let Some(command) = record.command() else {
                continue;
            };
            if command.starts_with("#SRC_") {
                bomb_group = follow_lift_kind(&record) == Some(FollowLiftKind::BombOrGrow);
            }
            let changed = if let Some(geometry) = destination_geometry(&command) {
                let preserve =
                    transform.aspect_changed && preserve_aspect(&command, bomb_group, options);
                let apply_offset = !offset_excluded(&command);
                let mut changed = transform_geometry(
                    &mut record,
                    transform,
                    geometry,
                    apply_offset,
                    preserve,
                    preserve && command.contains("NOWCOMBO"),
                );
                if !preserve
                    && let DestinationGeometry::Rectangle(fields) = geometry
                    && let Some(frame) = symmetric_frames.get(&index)
                {
                    changed = preserve_symmetric_horizontal_frame(
                        &mut record,
                        transform,
                        fields,
                        apply_offset,
                        *frame,
                    ) || changed;
                }
                changed
            } else if command == "#SRC_SLIDER" {
                slider_range_axis(&record).is_some_and(|axis| {
                    transform_source_dimensions(&mut record, transform, &[(12, axis)])
                })
            } else if let Some(field) = source_vertical_cutoff(&command) {
                transform_source_vertical_cutoff(&mut record, transform, field)
            } else {
                transform_source_dimensions(&mut record, transform, source_dimensions(&command))
            };
            if changed {
                replace_record(
                    file,
                    index,
                    content,
                    record,
                    "aspect-ratio-convert",
                    "convert coordinates to target resolution and aspect",
                    changes,
                );
            }
        }
    }
}

fn symmetric_horizontal_frames(file: &EditableFile) -> BTreeMap<usize, SymmetricHorizontalFrame> {
    let mut frames = BTreeMap::new();
    if !is_play_file(file) {
        return frames;
    }
    let judgments = judgment_geometries(&file.document);
    if judgments.is_empty() {
        return frames;
    }

    for index in 0..file.document.line_count() {
        let Some(content) = file.document.line_content(index) else {
            continue;
        };
        let record = CsvRecord::from_line(index, content);
        let Some(command) = record.command() else {
            continue;
        };
        let Some(DestinationGeometry::Rectangle([x_field, _, width_field, _])) =
            destination_geometry(&command)
        else {
            continue;
        };
        let (Some(x), Some(width)) = (record.field_i32(x_field), record.field_i32(width_field))
        else {
            continue;
        };
        if width <= 0 {
            continue;
        }
        let rectangle_right = i64::from(x) + i64::from(width);
        let Some(frame) = judgments
            .iter()
            .filter_map(|judgment| {
                let judgment_right = i64::from(judgment.x) + i64::from(judgment.width);
                let left_margin = i64::from(judgment.x) - i64::from(x);
                let right_margin = rectangle_right - judgment_right;
                (left_margin > 0 && left_margin == right_margin).then_some(
                    SymmetricHorizontalFrame {
                        inner_x: judgment.x,
                        inner_width: judgment.width,
                        margin: left_margin.clamp(0, i64::from(i32::MAX)) as i32,
                    },
                )
            })
            .min_by_key(|frame| frame.margin)
        else {
            continue;
        };
        frames.insert(index, frame);
    }
    frames
}

#[derive(Debug, Clone, Copy)]
pub(super) struct LayoutTransform {
    pub(super) source_width: i32,
    pub(super) source_height: i32,
    pub(super) target: ActiveRect,
    pub(super) aspect_changed: bool,
    pub(super) protected_width_numerator: i32,
    pub(super) protected_width_denominator: i32,
}

impl LayoutTransform {
    fn from_options(context: &RuleContext, options: &ConversionOptions) -> Option<Self> {
        let (source_width, source_height) = (context.canvas_width?, context.canvas_height?);
        let target_resolution = options
            .requested_resolution()
            .or(context.detected_resolution)?;
        let (target_width, target_height) = canvas_for_resolution(target_resolution);
        let (target_width, target_height) = (target_width?, target_height?);
        let (aspect_width, aspect_height) = options
            .requested_aspect()
            .map_or((source_width, source_height), |aspect| aspect.ratio());
        let aspect_changed = i64::from(source_width) * i64::from(aspect_height)
            != i64::from(source_height) * i64::from(aspect_width);
        // beatoraja displays #RESOLUTION,0 with a 4:3-to-16:9 horizontal stretch.
        // Fit the desired on-screen aspect into its pre-stretched logical canvas.
        let (aspect_width, aspect_height) = if target_resolution == 0 {
            (
                aspect_width.saturating_mul(3),
                aspect_height.saturating_mul(4),
            )
        } else {
            (aspect_width, aspect_height)
        };
        Some(Self {
            source_width,
            source_height,
            target: fit_aspect(target_width, target_height, aspect_width, aspect_height)?,
            aspect_changed,
            protected_width_numerator: if target_resolution == 0 { 3 } else { 1 },
            protected_width_denominator: if target_resolution == 0 { 4 } else { 1 },
        })
    }
}

#[derive(Debug, Clone, Copy)]
pub(super) struct ActiveRect {
    pub(super) x: i32,
    pub(super) y: i32,
    pub(super) width: i32,
    pub(super) height: i32,
}

fn fit_aspect(
    canvas_width: i32,
    canvas_height: i32,
    aspect_width: i32,
    aspect_height: i32,
) -> Option<ActiveRect> {
    if canvas_width <= 0 || canvas_height <= 0 || aspect_width <= 0 || aspect_height <= 0 {
        return None;
    }
    let canvas_is_wider = i64::from(canvas_width) * i64::from(aspect_height)
        >= i64::from(canvas_height) * i64::from(aspect_width);
    if canvas_is_wider {
        let width = scale(canvas_height, aspect_width, aspect_height);
        Some(ActiveRect {
            x: canvas_width.saturating_sub(width).div_euclid(2),
            y: 0,
            width,
            height: canvas_height,
        })
    } else {
        let height = scale(canvas_width, aspect_height, aspect_width);
        Some(ActiveRect {
            x: 0,
            y: canvas_height.saturating_sub(height).div_euclid(2),
            width: canvas_width,
            height,
        })
    }
}

fn preserve_aspect(command: &str, bomb_group: bool, options: &ConversionOptions) -> bool {
    (bomb_group && command == "#DST_IMAGE" && options.aspect_protection.bomb_effects)
        || (command.contains("NOWJUDGE") && options.aspect_protection.nowjudge)
        || (command.contains("NOWCOMBO") && options.aspect_protection.nowcombo)
}

fn offset_excluded(command: &str) -> bool {
    [
        "NOWCOMBO",
        "BAR_FLASH",
        "BAR_LEVEL",
        "BAR_LAMP",
        "BAR_TITLE",
        "BAR_RANK",
        "BAR_RIVAL",
        "BAR_MY_LAMP",
        "BAR_TROPHY",
        "BAR_LABEL",
        "BAR_GRAPH",
        "MOUSECURSOR",
    ]
    .iter()
    .any(|name| command.contains(name))
}
