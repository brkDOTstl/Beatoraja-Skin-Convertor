use skin_core::{CsvRecord, FollowLiftKind, canvas_for_resolution, follow_lift_kind};

use crate::{
    edit::{replace_record, warn},
    model::{ConversionOptions, EditableFile, FileChange, RuleContext, RuleWarning},
    rules::{
        aspect_schema::{destination_geometry, slider_range_axis, source_dimensions},
        aspect_transform::{scale, transform_geometry, transform_source_dimensions},
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
                transform_geometry(
                    &mut record,
                    transform,
                    geometry,
                    !offset_excluded(&command),
                    preserve,
                    preserve && command.contains("NOWCOMBO"),
                )
            } else if command == "#SRC_SLIDER" {
                slider_range_axis(&record).is_some_and(|axis| {
                    transform_source_dimensions(&mut record, transform, &[(12, axis)])
                })
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
