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
    const job = snapshotBatchJob(getPayload(true));
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
    try {
      const jobsForRun = applyCurrentBatchLayout(jobs, getPayload(false));
      jobs.splice(0, jobs.length, ...jobsForRun);
      render();
      const results = await invoke("generate_batch", { jobs: jobsForRun });
      onReport({ batch_results: results });
    } catch (error) {
      onReport({ error: String(error) });
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
    const displayJobs = applyCurrentBatchLayout(jobs, getPayload(false));
    displayJobs.forEach((job, index) => list.append(batchRow(job, index, t, remove)));
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

export function snapshotBatchJob(payload) {
  const resolution = payload.resolution == null || payload.resolution === ""
    ? null
    : Number(payload.resolution);
  return {
    input: String(payload.input ?? ""),
    output: String(payload.output ?? ""),
    preset: payload.preset ?? null,
    rules: Array.isArray(payload.rules) ? [...payload.rules] : [],
    liftCoverStrategy: payload.liftCoverStrategy ?? null,
    resolution,
    targetAspect: payload.targetAspect || null,
    aspectProtection: {
      bomb_effects: Boolean(payload.aspectProtection?.bomb_effects),
      nowjudge: Boolean(payload.aspectProtection?.nowjudge),
      nowcombo: Boolean(payload.aspectProtection?.nowcombo),
    },
  };
}

export function applyCurrentBatchLayout(jobs, payload) {
  const current = snapshotBatchJob(payload);
  const hasExplicitLayout = current.resolution != null || current.targetAspect != null;
  return jobs.map((job) => {
    const prepared = snapshotBatchJob(job);
    if (!hasExplicitLayout) return prepared;

    const rules = new Set(prepared.rules);
    if (current.resolution != null) rules.add("resolution-declaration");
    rules.add("aspect-ratio-convert");
    return {
      ...prepared,
      rules: [...rules],
      resolution: current.resolution,
      targetAspect: current.targetAspect,
      aspectProtection: { ...current.aspectProtection },
    };
  });
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
  const resolutions = {
    0: "SD 640×480",
    1: "HD 1280×720",
    2: "FHD 1920×1080",
    3: "4K 3840×2160",
  };
  const resolution = job.resolution == null
    ? t("keepResolution")
    : resolutions[job.resolution] || `#RESOLUTION,${job.resolution}`;
  const aspect = job.targetAspect || t("keepAspect");
  options.textContent = `${job.rules.length} ${t("rulesShort")} · ${t(strategyKey)} · ${resolution} · ${aspect} · ${protectedCount} ${t("protectedShort")}`;
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
