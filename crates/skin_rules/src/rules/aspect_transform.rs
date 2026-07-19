use skin_core::CsvRecord;

use super::{
    aspect::LayoutTransform,
    aspect_schema::{Axis, DestinationGeometry},
};

#[derive(Debug, Clone, Copy)]
pub(super) struct SymmetricHorizontalFrame {
    pub(super) inner_x: i32,
    pub(super) inner_width: i32,
    pub(super) margin: i32,
}

pub(super) fn transform_geometry(
    record: &mut CsvRecord,
    transform: LayoutTransform,
    geometry: DestinationGeometry,
    apply_offset: bool,
    preserve: bool,
    preserve_relative_x: bool,
) -> bool {
    match geometry {
        DestinationGeometry::Rectangle(fields) => transform_rectangle(
            record,
            transform,
            fields,
            apply_offset,
            preserve,
            preserve_relative_x,
        ),
        DestinationGeometry::Position { x, y } => {
            transform_position(record, transform, x, y, apply_offset)
        }
        DestinationGeometry::VerticalPosition { y } => transform_axis_position(
            record,
            y,
            transform.target.y,
            transform.target.height,
            transform.source_height,
        ),
    }
}

pub(super) fn transform_source_dimensions(
    record: &mut CsvRecord,
    transform: LayoutTransform,
    dimensions: &[(usize, Axis)],
) -> bool {
    dimensions.iter().fold(false, |changed, (field, axis)| {
        let (numerator, denominator) = match axis {
            Axis::Horizontal => (transform.target.width, transform.source_width),
            Axis::Vertical => (transform.target.height, transform.source_height),
        };
        scale_field(record, *field, numerator, denominator) || changed
    })
}

pub(super) fn scale(value: i32, numerator: i32, denominator: i32) -> i32 {
    if denominator == 0 {
        return value;
    }
    let scaled = (i64::from(value) * i64::from(numerator)).div_euclid(i64::from(denominator));
    scaled.clamp(i64::from(i32::MIN), i64::from(i32::MAX)) as i32
}

pub(super) fn preserve_symmetric_horizontal_frame(
    record: &mut CsvRecord,
    transform: LayoutTransform,
    [x_field, _, width_field, _]: [usize; 4],
    apply_offset: bool,
    frame: SymmetricHorizontalFrame,
) -> bool {
    let offset = if apply_offset { transform.target.x } else { 0 };
    let inner_left = scale(
        frame.inner_x,
        transform.target.width,
        transform.source_width,
    )
    .saturating_add(offset);
    let inner_right = scale(
        frame.inner_x.saturating_add(frame.inner_width),
        transform.target.width,
        transform.source_width,
    )
    .saturating_add(offset);
    let margin = scale_nearest(frame.margin, transform.target.width, transform.source_width);
    let target_x = inner_left.saturating_sub(margin);
    let target_width = inner_right
        .saturating_sub(inner_left)
        .saturating_add(margin.saturating_mul(2));
    if record.field_i32(x_field) == Some(target_x)
        && record.field_i32(width_field) == Some(target_width)
    {
        return false;
    }
    record.set_field_ascii(x_field, &target_x.to_string());
    record.set_field_ascii(width_field, &target_width.to_string());
    true
}

fn scale_nearest(value: i32, numerator: i32, denominator: i32) -> i32 {
    if denominator <= 0 || value < 0 || numerator < 0 {
        return scale(value, numerator, denominator);
    }
    let product = i64::from(value) * i64::from(numerator);
    let scaled = (product + i64::from(denominator) / 2).div_euclid(i64::from(denominator));
    scaled.clamp(i64::from(i32::MIN), i64::from(i32::MAX)) as i32
}

fn transform_rectangle(
    record: &mut CsvRecord,
    transform: LayoutTransform,
    [x_field, y_field, width_field, height_field]: [usize; 4],
    apply_offset: bool,
    preserve: bool,
    preserve_relative_x: bool,
) -> bool {
    let (Some(x), Some(y), Some(width), Some(height)) = (
        record.field_i32(x_field),
        record.field_i32(y_field),
        record.field_i32(width_field),
        record.field_i32(height_field),
    ) else {
        return false;
    };
    let scaled_x = scale(x, transform.target.width, transform.source_width);
    let scaled_right = scale(
        x.saturating_add(width),
        transform.target.width,
        transform.source_width,
    );
    let scaled_y = scale(y, transform.target.height, transform.source_height);
    let scaled_bottom = scale(
        y.saturating_add(height),
        transform.target.height,
        transform.source_height,
    );
    let layout_width = scaled_right.saturating_sub(scaled_x);
    let target_width = if preserve {
        let height_scaled_width = scale(width, transform.target.height, transform.source_height);
        scale(
            height_scaled_width,
            transform.protected_width_numerator,
            transform.protected_width_denominator,
        )
    } else {
        layout_width
    };
    let target_x = if preserve_relative_x {
        let height_scaled_x = scale(x, transform.target.height, transform.source_height);
        scale(
            height_scaled_x,
            transform.protected_width_numerator,
            transform.protected_width_denominator,
        )
    } else {
        scaled_x
            .saturating_add(if apply_offset { transform.target.x } else { 0 })
            .saturating_add(layout_width.saturating_sub(target_width).div_euclid(2))
    };
    let target_y = scaled_y.saturating_add(if apply_offset { transform.target.y } else { 0 });
    let target_height = scaled_bottom.saturating_sub(scaled_y);
    if [x, y, width, height] == [target_x, target_y, target_width, target_height] {
        return false;
    }
    record.set_field_ascii(x_field, &target_x.to_string());
    record.set_field_ascii(y_field, &target_y.to_string());
    record.set_field_ascii(width_field, &target_width.to_string());
    record.set_field_ascii(height_field, &target_height.to_string());
    true
}

fn transform_position(
    record: &mut CsvRecord,
    transform: LayoutTransform,
    x_field: usize,
    y_field: usize,
    apply_offset: bool,
) -> bool {
    let x_offset = if apply_offset { transform.target.x } else { 0 };
    let y_offset = if apply_offset { transform.target.y } else { 0 };
    let x_changed = transform_axis_position(
        record,
        x_field,
        x_offset,
        transform.target.width,
        transform.source_width,
    );
    let y_changed = transform_axis_position(
        record,
        y_field,
        y_offset,
        transform.target.height,
        transform.source_height,
    );
    x_changed || y_changed
}

fn transform_axis_position(
    record: &mut CsvRecord,
    field: usize,
    offset: i32,
    numerator: i32,
    denominator: i32,
) -> bool {
    let Some(value) = record.field_i32(field) else {
        return false;
    };
    let target = scale(value, numerator, denominator).saturating_add(offset);
    if target == value {
        return false;
    }
    record.set_field_ascii(field, &target.to_string());
    true
}

fn scale_field(record: &mut CsvRecord, field: usize, numerator: i32, denominator: i32) -> bool {
    let Some(value) = record.field_i32(field) else {
        return false;
    };
    let target = scale(value, numerator, denominator);
    if target == value {
        return false;
    }
    record.set_field_ascii(field, &target.to_string());
    true
}
