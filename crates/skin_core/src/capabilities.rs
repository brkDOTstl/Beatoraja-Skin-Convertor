use crate::analyze::{AnalysisCounters, Confidence, Finding, RuleCapability};

pub(crate) fn build_capabilities(counters: &AnalysisCounters) -> Vec<RuleCapability> {
    [
        ("resolution-declaration", counters.has_resolution),
        ("follow-lift-op4", counters.follow_lift_candidates == 0),
        (
            "judge-line-follow",
            counters.judge_line_count > 0 && counters.judge_line_missing_op4 == 0,
        ),
        (
            "lift-cover-copy-sudden",
            counters.has_src_lift && counters.has_dst_lift,
        ),
        ("green-number", counters.has_green_number),
        ("lift-number", counters.has_lift_number),
        ("aspect-ratio-convert", false),
        ("blend-fallback", counters.blend5_candidates == 0),
    ]
    .into_iter()
    .map(|(id, present)| RuleCapability {
        rule_id: id.to_owned(),
        title: id.to_owned(),
        confidence: Confidence::RecommendedConfirm,
        status: if present { "present" } else { "available" }.to_owned(),
        detail: id.to_owned(),
    })
    .collect()
}

pub(crate) fn finding(code: &str, message: &str, confidence: Confidence) -> Finding {
    Finding {
        code: code.to_owned(),
        message: message.to_owned(),
        confidence,
        file: None,
        line: None,
    }
}
