export function createBatchController({
  invoke,
  getTranslator,
  getPayload,
  onBusy,
  onReport,
}) {
  const jobs = [];
  let busy = false;

  document.getElementById("addBatchBtn").addEventListener("click", addCurrent);
  document.getElementById("runBatchBtn").addEventListener("click", run);
  document.getElementById("clearBatchBtn").addEventListener("click", () => {
    jobs.length = 0;
    render();
  });

  function addCurrent() {
    const t = getTranslator();
    const job = getPayload(true);
    const error = validateGenerationPaths(job.input, job.output, t);
    if (error) {
      onReport({ error });
      return;
    }
    const outputKey = normalizeWindowsPath(job.output);
    if (jobs.some((item) => normalizeWindowsPath(item.output) === outputKey)) {
      onReport({ error: t("duplicateBatchOutput") });
      return;
    }
    jobs.push(job);
    render();
  }

  async function run() {
    const t = getTranslator();
    if (!invoke || !jobs.length) {
      onReport({ error: invoke ? t("emptyBatch") : t("staticMessage") });
      return;
    }
    onBusy(true);
    const results = [];
    try {
      for (const job of jobs) {
        try {
          const report = await invoke("generate_skin", job);
          results.push({
            input: job.input,
            output: job.output,
            ok: true,
            changed_files: report.manifest?.changed_files?.length || 0,
            warnings: report.warnings?.length || 0,
          });
        } catch (error) {
          results.push({
            input: job.input,
            output: job.output,
            ok: false,
            changed_files: 0,
            warnings: 0,
            error: String(error),
          });
        }
      }
      onReport({ batch_results: results });
    } finally {
      onBusy(false);
    }
  }

  function render() {
    const t = getTranslator();
    const list = document.getElementById("batchList");
    list.replaceChildren();
    document.getElementById("batchCount").textContent = String(jobs.length);
    const clear = document.getElementById("clearBatchBtn");
    clear.title = t("clearBatch");
    clear.setAttribute("aria-label", t("clearBatch"));
    jobs.forEach((job, index) => list.append(batchRow(job, index, t, remove)));
    document.getElementById("batchQueue").hidden = jobs.length === 0;
    document.getElementById("runBatchBtn").disabled = busy || jobs.length === 0;
    document.getElementById("clearBatchBtn").disabled = busy || jobs.length === 0;
    document.getElementById("addBatchBtn").disabled = busy;
  }

  function remove(index) {
    jobs.splice(index, 1);
    render();
  }

  function setBusy(value) {
    busy = value;
    render();
  }

  render();
  return { render, setBusy };
}

export function validateGenerationPaths(input, output, t) {
  const inputKey = normalizeWindowsPath(input);
  const outputKey = normalizeWindowsPath(output);
  if (!inputKey || !outputKey) return t("pathsRequired");
  if (inputKey === outputKey) return t("outputEqualsInput");
  if (outputKey.startsWith(`${inputKey}\\`)) return t("outputInsideInput");
  return null;
}

function normalizeWindowsPath(value) {
  const parts = value.trim().replaceAll("/", "\\").split("\\");
  const normalized = [];
  for (const part of parts) {
    if (!part || part === ".") continue;
    if (part === "..") normalized.pop();
    else normalized.push(part);
  }
  return normalized.join("\\").toLocaleLowerCase();
}

function batchRow(job, index, t, onRemove) {
  const row = document.createElement("div");
  row.className = "batch-row";
  const copy = document.createElement("div");
  copy.className = "batch-paths";
  const source = document.createElement("strong");
  source.textContent = job.input;
  source.title = job.input;
  const output = document.createElement("span");
  output.textContent = `→ ${job.output}`;
  output.title = job.output;
  const strategyKey = job.liftCoverStrategy === "copy-sudden"
    ? "copySudden"
    : job.liftCoverStrategy === "transparent"
      ? "transparentLift"
      : "keepLift";
  const options = document.createElement("small");
  const protectedCount = Object.values(job.aspectProtection || {}).filter(Boolean).length;
  const layout = job.targetAspect || (job.resolution == null ? "" : t("keepAspect"));
  options.textContent = `${job.rules.length} ${t("rulesShort")} · ${t(strategyKey)}${layout ? ` · ${layout}` : ""} · ${protectedCount} ${t("protectedShort")}`;
  copy.append(source, output, options);
  const remove = document.createElement("button");
  remove.type = "button";
  remove.className = "icon-button";
  remove.textContent = "×";
  remove.title = t("removeBatchItem");
  remove.setAttribute("aria-label", t("removeBatchItem"));
  remove.addEventListener("click", () => onRemove(index));
  row.append(copy, remove);
  return row;
}
