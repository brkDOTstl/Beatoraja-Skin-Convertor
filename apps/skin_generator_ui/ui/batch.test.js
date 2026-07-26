import assert from "node:assert/strict";
import test from "node:test";

import { applyCurrentBatchLayout, snapshotBatchJob } from "./batch.js";

test("batch snapshot preserves layout and aspect-protection options", () => {
  const payload = {
    input: "D:\\skins\\source",
    output: "D:\\skins\\generated\\source",
    preset: null,
    rules: ["resolution-declaration", "aspect-ratio-convert"],
    liftCoverStrategy: "copy-sudden",
    resolution: 2,
    targetAspect: "4:3",
    aspectProtection: {
      bomb_effects: true,
      nowjudge: false,
      nowcombo: true,
    },
  };

  const snapshot = snapshotBatchJob(payload);
  payload.rules.push("later-change");
  payload.resolution = 0;
  payload.targetAspect = "16:9";
  payload.aspectProtection.bomb_effects = false;

  assert.deepEqual(snapshot, {
    input: "D:\\skins\\source",
    output: "D:\\skins\\generated\\source",
    preset: null,
    rules: ["resolution-declaration", "aspect-ratio-convert"],
    liftCoverStrategy: "copy-sudden",
    resolution: 2,
    targetAspect: "4:3",
    aspectProtection: {
      bomb_effects: true,
      nowjudge: false,
      nowcombo: true,
    },
  });
});

test("batch snapshot normalizes empty target options to null", () => {
  const snapshot = snapshotBatchJob({
    input: "source",
    output: "output",
    rules: [],
    resolution: "",
    targetAspect: "",
  });

  assert.equal(snapshot.resolution, null);
  assert.equal(snapshot.targetAspect, null);
  assert.deepEqual(snapshot.aspectProtection, {
    bomb_effects: false,
    nowjudge: false,
    nowcombo: false,
  });
});

test("batch run applies an explicit current layout to jobs queued before selection", () => {
  const queued = snapshotBatchJob({
    input: "D:\\skins\\source-a",
    output: "D:\\generated\\source-a",
    rules: ["theme-path-rebase"],
    resolution: null,
    targetAspect: null,
  });

  const [prepared] = applyCurrentBatchLayout([queued], {
    resolution: 0,
    targetAspect: "4:3",
    aspectProtection: {
      bomb_effects: true,
      nowjudge: false,
      nowcombo: true,
    },
  });

  assert.equal(prepared.resolution, 0);
  assert.equal(prepared.targetAspect, "4:3");
  assert.deepEqual(prepared.aspectProtection, {
    bomb_effects: true,
    nowjudge: false,
    nowcombo: true,
  });
  assert.ok(prepared.rules.includes("resolution-declaration"));
  assert.ok(prepared.rules.includes("aspect-ratio-convert"));
  assert.equal(queued.resolution, null);
  assert.equal(queued.targetAspect, null);
});

test("blank current layout preserves each queued job layout", () => {
  const queued = snapshotBatchJob({
    input: "D:\\skins\\source-a",
    output: "D:\\generated\\source-a",
    rules: ["aspect-ratio-convert"],
    resolution: 2,
    targetAspect: "16:9",
  });

  const [prepared] = applyCurrentBatchLayout([queued], {
    resolution: null,
    targetAspect: null,
  });

  assert.equal(prepared.resolution, 2);
  assert.equal(prepared.targetAspect, "16:9");
});
