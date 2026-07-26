import assert from "node:assert/strict";
import test from "node:test";

import { presetRules } from "./presets.js";

test("experimental LR2 DARK MODE remains opt-in for every shipped preset", () => {
  for (const [preset, rules] of Object.entries(presetRules)) {
    assert.equal(
      rules.includes("lr2-dark-mode"),
      false,
      `${preset} must leave lr2-dark-mode disabled`,
    );
  }
});
