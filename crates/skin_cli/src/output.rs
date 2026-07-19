use anyhow::Result;
use skin_core::AnalysisReport;
use skin_rules::ConversionReport;

pub(crate) fn print_conversion_report(report: &ConversionReport, json: bool) -> Result<()> {
    if json {
        println!("{}", serde_json::to_string_pretty(report)?);
        return Ok(());
    }
    println!("Selected rules: {}", report.selected_rules.join(", "));
    println!("Changed files: {}", report.manifest.changed_files.len());
    println!("Changes: {}", report.changes.len());
    for warning in &report.warnings {
        println!("WARN [{}] {}", warning.rule_id, warning.message);
    }
    for change in report.changes.iter().take(40) {
        let line = change
            .line
            .map(|line| line.to_string())
            .unwrap_or_else(|| "-".to_owned());
        println!(
            "CHANGE [{}] {}:{} {}",
            change.rule_id,
            change.path.display(),
            line,
            change.message
        );
    }
    Ok(())
}

pub(crate) fn print_analysis(report: &AnalysisReport) {
    println!("Input: {}", report.input_path.display());
    println!("CSV files: {}", report.counters.total_csv_files);
    println!("Play CSV files: {}", report.counters.play_csv_files);
    println!("lr2skin files: {}", report.counters.lr2skin_files);
    println!(
        "Detected layout: resolution={:?}, canvas={:?}x{:?}, aspect={}, source={}",
        report.layout.resolution,
        report.layout.canvas_width,
        report.layout.canvas_height,
        report.layout.aspect_ratio,
        report.layout.source
    );
    println!("Green number present: {}", report.counters.has_green_number);
    println!("LIFT number present: {}", report.counters.has_lift_number);
    println!(
        "LIFT cover present: {}",
        report.counters.has_src_lift && report.counters.has_dst_lift
    );
    println!(
        "Follow-LIFT candidates: {}",
        report.counters.follow_lift_candidates
    );
    println!(
        "Blend fallback candidates: {}",
        report.counters.blend5_candidates
    );
    for finding in &report.findings {
        println!("FINDING [{}] {}", finding.code, finding.message);
    }
}
