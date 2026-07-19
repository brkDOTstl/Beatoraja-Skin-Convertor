import { createTranslator } from "./i18n.js";
import { renderDetectedLayout, renderOutput, renderRules } from "./render.js";
import { createBatchController, validateGenerationPaths } from "./batch.js";

const invoke = window.__TAURI__?.core?.invoke;
const $ = (id) => document.getElementById(id);
const presetRules = {
  "safe-auto": ["resolution-declaration", "theme-path-rebase", "asset-reference-repair", "follow-lift-op4", "judge-line-follow", "blend-fallback"],
  "visible-lift": ["resolution-declaration", "theme-path-rebase", "asset-reference-repair", "follow-lift-op4", "judge-line-follow", "blend-fallback", "lift-cover-copy-sudden", "green-number", "lift-number"],
  "transparent-lift": ["resolution-declaration", "theme-path-rebase", "asset-reference-repair", "follow-lift-op4", "judge-line-follow", "blend-fallback", "lift-cover-transparent", "green-number", "lift-number"],
  "fhd-4x3": ["resolution-declaration", "theme-path-rebase", "asset-reference-repair", "follow-lift-op4", "judge-line-follow", "blend-fallback", "aspect-ratio-convert"],
};

const state = {
  rules: [],
  selectedRules: new Set(),
  lastReport: null,
  activeTab: "summary",
  runtimeKey: "checkingRuntime",
  language: localStorage.getItem("bmsSkinGenerator.language") || (navigator.language.startsWith("ja") ? "ja" : "en"),
};

let t = createTranslator(state.language);
let batchController;

async function boot() {
  batchController = createBatchController({
    invoke,
    getTranslator: () => t,
    getPayload: payload,
    onBusy: setBusy,
    onReport: showReport,
  });
  wireActions();
  wireTabs();
  wireLanguage();
  wireOptions();
  applyLanguage();
  if (!invoke) {
    state.rules = fallbackRules();
    applyPreset("visible-lift");
    setRuntime("staticPreview");
    $("outputText").textContent = t("staticMessage");
    return;
  }
  try {
    state.rules = await invoke("rule_definitions");
    applyPreset($("preset").value);
    setRuntime("runtimeReady");
  } catch (error) {
    state.rules = fallbackRules();
    applyPreset("visible-lift");
    setRuntime("loadFailed");
    state.lastReport = { error: String(error) };
    refresh();
  }
}

function wireActions() {
  $("analyzeBtn").addEventListener("click", () => runCommand("analyze_skin", { input: $("inputPath").value }));
  $("dryRunBtn").addEventListener("click", () => runCommand("dry_run_skin", payload(false)));
  $("generateBtn").addEventListener("click", runGenerate);
  $("browseInputBtn").addEventListener("click", () => chooseFolder("inputPath"));
  $("browseDestinationBtn").addEventListener("click", () => chooseFolder("destinationRoot"));
}

function wireTabs() {
  for (const button of document.querySelectorAll(".tab")) {
    button.addEventListener("click", () => {
      state.activeTab = button.dataset.tab;
      syncTabs();
      renderOutput(state, t);
    });
  }
}

function wireLanguage() {
  for (const button of document.querySelectorAll(".language-button")) {
    button.addEventListener("click", () => {
      state.language = button.dataset.language;
      localStorage.setItem("bmsSkinGenerator.language", state.language);
      t = createTranslator(state.language);
      applyLanguage();
      refresh();
    });
  }
}

function wireOptions() {
  $("preset").addEventListener("change", () => applyPreset($("preset").value));
  $("liftCoverStrategy").addEventListener("change", syncLiftRule);
  $("resolution").addEventListener("change", () => {
    syncAspectRule();
    syncResolutionZeroAspectNote();
  });
  $("targetAspect").addEventListener("change", syncAspectRule);
}

async function runCommand(command, args) {
  if (!invoke) return;
  setBusy(true);
  try {
    state.lastReport = await invoke(command, args);
    state.activeTab = "summary";
    renderDetectedLayout(state.lastReport, t);
  } catch (error) {
    state.lastReport = { error: String(error) };
    state.activeTab = "summary";
  } finally {
    setBusy(false);
    refresh();
  }
}

function runGenerate() {
  const args = payload(true);
  const error = validateGenerationPaths(args.input, args.output, t);
  if (error) {
    showReport({ error });
    return;
  }
  runCommand("generate_skin", args);
}

function showReport(report) {
  state.lastReport = report;
  state.activeTab = "summary";
  refresh();
}

function payload(includeOutput) {
  updateOutputPath();
  const result = {
    input: $("inputPath").value,
    preset: null,
    rules: [...state.selectedRules],
    liftCoverStrategy: $("liftCoverStrategy").value || null,
    resolution: $("resolution").value === "" ? null : Number($("resolution").value),
    targetAspect: $("targetAspect").value || null,
    aspectProtection: {
      bomb_effects: $("preserveBombAspect").checked,
      nowjudge: $("preserveNowjudgeAspect").checked,
      nowcombo: $("preserveNowcomboAspect").checked,
    },
  };
  if (includeOutput) result.output = $("outputPath").value;
  return result;
}

function applyPreset(name) {
  state.selectedRules = new Set(presetRules[name] || []);
  if (name === "visible-lift") $("liftCoverStrategy").value = "copy-sudden";
  if (name === "transparent-lift") $("liftCoverStrategy").value = "transparent";
  if (!name || name === "safe-auto" || name === "fhd-4x3") $("liftCoverStrategy").value = "";
  $("resolution").value = name === "fhd-4x3" ? "2" : "";
  $("targetAspect").value = name === "fhd-4x3" ? "4:3" : "";
  renderRules(state, t, toggleRule);
  syncAspectProtectionState();
  syncResolutionZeroAspectNote();
}

async function chooseFolder(fieldId) {
  if (!invoke) return;
  setBusy(true);
  try {
    const path = await invoke("select_folder", { initialPath: $(fieldId).value || null });
    if (path) {
      $(fieldId).value = path;
      updateOutputPath();
    }
  } catch (error) {
    showReport({ error: String(error) });
  } finally {
    setBusy(false);
  }
}

function updateOutputPath() {
  const sourceName = pathLeaf($("inputPath").value);
  const destination = trimTrailingSeparators($("destinationRoot").value);
  $("outputPath").value = sourceName && destination ? `${destination}\\${sourceName}` : "";
}

function pathLeaf(path) {
  const normalized = trimTrailingSeparators(path);
  const separator = Math.max(normalized.lastIndexOf("\\"), normalized.lastIndexOf("/"));
  return separator === -1 ? normalized : normalized.slice(separator + 1);
}

function trimTrailingSeparators(path) {
  return path.trim().replace(/[\\/]+$/, "");
}

function syncLiftRule() {
  const strategy = $("liftCoverStrategy").value;
  toggleRule("lift-cover-copy-sudden", strategy === "copy-sudden", false);
  toggleRule("lift-cover-transparent", strategy === "transparent", false);
  renderRules(state, t, toggleRule);
}

function toggleRule(id, enabled, rerender = true) {
  if (enabled) state.selectedRules.add(id);
  else state.selectedRules.delete(id);
  if (id === "aspect-ratio-convert") syncAspectProtectionState();
  if (rerender) renderRules(state, t, toggleRule);
}

function syncAspectRule() {
  const hasTargetResolution = $("resolution").value !== "";
  const enabled = hasTargetResolution || $("targetAspect").value !== "";
  if (hasTargetResolution) toggleRule("resolution-declaration", true, false);
  toggleRule("aspect-ratio-convert", enabled);
}

function syncAspectProtectionState() {
  $("aspectProtection").disabled = !state.selectedRules.has("aspect-ratio-convert");
}

function syncResolutionZeroAspectNote() {
  const explicitResolution = $("resolution").value;
  const detectedResolution = state.lastReport?.analysis?.layout?.resolution;
  $("resolutionZeroAspectNote").hidden = !(
    explicitResolution === "0" || (explicitResolution === "" && detectedResolution === 0)
  );
}

function applyLanguage() {
  document.documentElement.lang = state.language;
  for (const element of document.querySelectorAll("[data-i18n]")) {
    element.textContent = t(element.dataset.i18n);
  }
  for (const button of document.querySelectorAll(".language-button")) {
    const active = button.dataset.language === state.language;
    button.classList.toggle("active", active);
    button.setAttribute("aria-pressed", String(active));
  }
  $("runtimeState").textContent = t(state.runtimeKey);
}

function refresh() {
  renderRules(state, t, toggleRule);
  renderDetectedLayout(state.lastReport, t);
  renderOutput(state, t);
  if (!invoke && !state.lastReport) $("outputText").textContent = t("staticMessage");
  batchController?.render();
  syncTabs();
  syncResolutionZeroAspectNote();
}

function syncTabs() {
  for (const tab of document.querySelectorAll(".tab")) {
    tab.classList.toggle("active", tab.dataset.tab === state.activeTab);
  }
}

function setBusy(busy) {
  for (const id of ["analyzeBtn", "dryRunBtn", "generateBtn", "browseInputBtn", "browseDestinationBtn"]) $(id).disabled = busy;
  batchController?.setBusy(busy);
  setRuntime(busy ? "working" : invoke ? "runtimeReady" : "staticPreview");
}

function setRuntime(key) {
  state.runtimeKey = key;
  $("runtimeState").textContent = t(key);
}

function fallbackRules() {
  const catalog = [
    ["resolution-declaration", "base", "safe_auto", true],
    ["theme-path-rebase", "base", "safe_auto", true],
    ["asset-reference-repair", "base", "recommended_confirm", true],
    ["follow-lift-op4", "lift", "safe_auto", true],
    ["judge-line-follow", "lift", "safe_auto", true],
    ["lift-cover-transparent", "lift", "recommended_confirm", true],
    ["lift-cover-copy-sudden", "lift", "recommended_confirm", true],
    ["lift-cover-custom-image", "lift", "risky_manual", false],
    ["green-number", "numbers", "recommended_confirm", true],
    ["lift-number", "numbers", "recommended_confirm", true],
    ["aspect-ratio-convert", "layout", "recommended_confirm", true],
    ["blend-fallback", "compatibility", "recommended_confirm", true],
    ["key-laser-rewrite", "compatibility", "risky_manual", false],
    ["gauge-label-helper", "compatibility", "risky_manual", false],
  ];
  return catalog.map(([id, category, confidence, implemented]) => ({
    id, category, confidence, implemented,
    default_enabled: false,
    title: id,
    description: id,
  }));
}

updateOutputPath();
boot();
