use skin_core::CsvRecord;

#[derive(Debug, Clone, Copy)]
pub(super) enum Axis {
    Horizontal,
    Vertical,
}

#[derive(Debug, Clone, Copy)]
pub(super) enum DestinationGeometry {
    Rectangle([usize; 4]),
    Position { x: usize, y: usize },
    VerticalPosition { y: usize },
}

pub(super) fn destination_geometry(command: &str) -> Option<DestinationGeometry> {
    use DestinationGeometry::{Position, Rectangle, VerticalPosition};

    match command {
        "#DST_PM_CHARA_1P" | "#DST_PM_CHARA_2P" | "#DST_PM_CHARA_ANIMATION" => {
            Some(Rectangle([1, 2, 3, 4]))
        }
        "#DST_NOTECHART"
        | "#DST_NOTECHART_1P"
        | "#DST_BPMCHART"
        | "#DST_TIMING_1P"
        | "#DST_GAUGECHART_1P"
        | "#DST_TIMINGCHART_1P" => Some(Position { x: 3, y: 4 }),
        "#DST_NOTE2" => Some(VerticalPosition { y: 1 }),
        command if standard_rectangle(command) => Some(Rectangle([3, 4, 5, 6])),
        _ => None,
    }
}

pub(super) fn source_dimensions(command: &str) -> &'static [(usize, Axis)] {
    use Axis::{Horizontal, Vertical};

    match command {
        "#SRC_GROOVEGAUGE" => &[(11, Horizontal)],
        "#SRC_NOTECHART" | "#SRC_NOTECHART_1P" | "#SRC_GAUGECHART_1P" => {
            &[(11, Horizontal), (12, Vertical)]
        }
        "#SRC_BPMCHART" => &[(1, Horizontal), (2, Vertical)],
        "#SRC_TIMING_1P" | "#SRC_TIMINGCHART_1P" => &[(4, Horizontal), (5, Vertical)],
        _ => &[],
    }
}

pub(super) fn slider_range_axis(record: &CsvRecord) -> Option<Axis> {
    match record.field_i32(11)? {
        0 | 2 => Some(Axis::Vertical),
        1 | 3 => Some(Axis::Horizontal),
        _ => None,
    }
}

fn standard_rectangle(command: &str) -> bool {
    matches!(
        command,
        "#DST_IMAGE"
            | "#DST_NUMBER"
            | "#DST_TEXT"
            | "#DST_SLIDER"
            | "#DST_BARGRAPH"
            | "#DST_BUTTON"
            | "#DST_ONMOUSE"
            | "#DST_GROOVEGAUGE"
            | "#DST_BGA"
            | "#DST_LINE"
            | "#DST_NOTE"
            | "#DST_NOWJUDGE_1P"
            | "#DST_NOWJUDGE_2P"
            | "#DST_NOWJUDGE_3P"
            | "#DST_NOWCOMBO_1P"
            | "#DST_NOWCOMBO_2P"
            | "#DST_NOWCOMBO_3P"
            | "#DST_JUDGELINE"
            | "#DST_HIDDEN"
            | "#DST_LIFT"
            | "#DST_PM_CHARA_IMAGE"
            | "#DST_BAR_BODY_OFF"
            | "#DST_BAR_BODY_ON"
            | "#DST_BAR_LEVEL"
            | "#DST_BAR_LAMP"
            | "#DST_BAR_MY_LAMP"
            | "#DST_BAR_RIVAL_LAMP"
            | "#DST_BAR_TROPHY"
            | "#DST_BAR_LABEL"
            | "#DST_BAR_GRAPH"
            | "#DST_BAR_TITLE"
    )
}
