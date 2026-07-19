import { ruleText } from "./i18n.js";

const categoryKeys = {
  base: "categoryBase",
  lift: "categoryLift",
  numbers: "categoryNumbers",
  layout: "categoryLayout",
  compatibility: "categoryCompatibility",
};

export function renderRules(state, t, onToggle) {
  const container = document.getElementById("ruleList");
  container.replaceChildren();
  const categories = new Map();
  for (const rule of state.rules) {
    const category = rule.category || "compatibility";
    if (!categories.has(category)) categories.set(category, []);
    categories.get(category).push(rule);
  }
  for (const [category, rules] of categories) {
    const section = document.createElement("section");
    section.className = "rule-section";
    const heading = document.createElement("h2");
    heading.textContent = t(categoryKeys[category] || "categoryCompatibility");
    section.append(heading);
    for (const rule of rules) section.append(ruleRow(rule, state, t, onToggle));
    container.append(section);
  }
}

export function renderOutput(state, t) {
  const output = document.getElementById("outputText");
  const report = state.lastReport;
  if (!report) {
    output.textContent = "";
    return;
  }
  if (report.batch_results) {
    output.textContent = formatBatchResults(report.batch_results, t);
    return;
  }
  if (report.error) {
    output.textContent = `${t("error")}\n${report.error}`;
    return;
  }
  if (state.activeTab === "changes") {
    output.textContent = formatChanges(report.changes || [], state.language, t);
  } else if (state.activeTab === "warnings") {
    output.textContent = formatWarnings(report.warnings || report.findings || [], state.language, t);
  } else {
    output.textContent = formatSummary(report, t);
  }
}

function formatBatchResults(results, t) {
  const succeeded = results.filter((result) => result.ok).length;
  const lines = [
    `${t("batchResults")}: ${succeeded} ${t("succeeded")} / ${results.length - succeeded} ${t("failed")}`,
    "",
  ];
  for (const result of results) {
    lines.push(
      `[${result.ok ? t("succeeded") : t("failed")}] ${result.input}`,
      `  -> ${result.output}`,
    );
    if (result.ok) {
      lines.push(
        `  ${t("changedFiles")}: ${result.changed_files}, ${t("warningCount")}: ${result.warnings}`,
      );
    } else {
      lines.push(`  ${t("error")}: ${result.error}`);
    }
    lines.push("");
  }
  return lines.join("\n");
}

export function renderDetectedLayout(report, t) {
  const element = document.getElementById("detectedLayout");
  const analysis = report?.analysis || report;
  const layout = analysis?.layout;
  if (!layout) {
    element.hidden = true;
    return;
  }
  const resolution = layout.resolution == null ? t("unknown") : `#RESOLUTION,${layout.resolution}`;
  const canvas = layout.canvas_width && layout.canvas_height
    ? `${layout.canvas_width} × ${layout.canvas_height}`
    : t("unknown");
  element.textContent = `${t("resolutionDetected")}: ${resolution}  ·  ${t("canvas")}: ${canvas}  ·  ${t("aspect")}: ${layout.aspect_ratio}  ·  ${t("detection")}: ${t(layout.source)}`;
  element.hidden = false;
}

function ruleRow(rule, state, t, onToggle) {
  const row = document.createElement("label");
  row.className = `rule-row${rule.implemented ? "" : " unavailable"}`;
  const checkbox = document.createElement("input");
  checkbox.type = "checkbox";
  checkbox.value = rule.id;
  checkbox.checked = state.selectedRules.has(rule.id);
  checkbox.disabled = !rule.implemented;
  checkbox.addEventListener("change", () => onToggle(rule.id, checkbox.checked));
  const copy = document.createElement("div");
  const localized = ruleText[state.language]?.[rule.id] || [rule.title, rule.description];
  const title = document.createElement("div");
  title.className = "rule-title";
  title.textContent = localized[0];
  const description = document.createElement("div");
  description.className = "rule-description";
  description.textContent = localized[1];
  copy.append(title, description);
  const badge = document.createElement("span");
  badge.className = `badge ${rule.confidence}`;
  badge.textContent = rule.implemented ? t(rule.confidence) : t("risky_manual");
  row.append(checkbox, copy, badge);
  return row;
}

function formatSummary(report, t) {
  const analysis = report.analysis || (report.counters ? report : null);
  const lines = [];
  if (analysis) {
    const c = analysis.counters;
    const layout = analysis.layout || {};
    lines.push(
      `${t("input")}: ${analysis.input_path}`,
      `${t("csvFiles")}: ${c.total_csv_files}`,
      `${t("playCsvFiles")}: ${c.play_csv_files}`,
      `${t("skinFiles")}: ${c.lr2skin_files}`,
      `${t("resolutionDetected")}: ${layout.resolution == null ? t("unknown") : `#RESOLUTION,${layout.resolution}`}`,
      `${t("aspect")}: ${layout.aspect_ratio || t("unknown")}`,
      `${t("greenNumber")}: ${formatBoolean(c.has_green_number, t)}`,
      `${t("liftNumber")}: ${formatBoolean(c.has_lift_number, t)}`,
      `${t("followCandidates")}: ${c.follow_lift_candidates}`,
    );
  }
  if (report.manifest) {
    const manifest = report.manifest;
    lines.push(
      "",
      `${t("selectedRules")}: ${(report.selected_rules || []).join(", ")}`,
      `${t("changedFiles")}: ${(manifest.changed_files || []).length}`,
      `${t("changeCount")}: ${(report.changes || []).length}`,
      `${t("warningCount")}: ${(report.warnings || []).length}`,
      `${t("output")}: ${manifest.output_path || t("dryRunOutput")}`,
      "",
      ...(manifest.changed_files || []).map((path) => `- ${path}`),
    );
  }
  return lines.join("\n");
}

function formatChanges(changes, language, t) {
  if (!changes.length) return t("noChanges");
  return changes.map((change) => {
    const title = ruleText[language]?.[change.rule_id]?.[0] || change.rule_id;
    return `[${t("applied")}: ${title}] ${change.path}:${change.line ?? "-"}\n- ${change.before}\n+ ${change.after}`;
  }).join("\n\n");
}

function formatWarnings(warnings, language, t) {
  if (!warnings.length) return t("noWarnings");
  return warnings.map((warning) => {
    const id = warning.rule_id || warning.code || "warning";
    const title = ruleText[language]?.[id]?.[0] || id;
    const path = warning.path ? ` (${warning.path})` : "";
    return `[${title}] ${localizeWarning(id, warning.message, language, t)}${path}`;
  }).join("\n");
}

function formatBoolean(value, t) {
  return value ? t("yes") : t("no");
}

function localizeWarning(id, message, language, t) {
  if (language !== "ja") return message;
  const key = `warning_${id.replaceAll("-", "_")}`;
  const localized = t(key);
  return localized === key ? message : localized;
}
