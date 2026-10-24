import test from "node:test";
import assert from "node:assert/strict";
import {readFileSync} from "node:fs";
import english from "../apps/desktop/ui/locales/en.mjs";

const root = new URL("../", import.meta.url);
const read = path => readFileSync(new URL(path, root), "utf8");
const native = read("apps/desktop/src-tauri/src/native_messages.rs");

function constants(source) {
  const result = new Map();
  for (const match of source.matchAll(/pub const ([A-Z_][A-Z_0-9]*): &str = ("(?:[^"\\]|\\.)*");/g)) {
    result.set(match[1], JSON.parse(match[2]));
  }
  return result;
}

test("Rust presentation messages have a single native catalog and valid references", () => {
  const declared = constants(native);
  assert.ok(declared.size >= 100, "Native diagnostics and menu text are centralized");
  for (const path of [
    "apps/desktop/src-tauri/src/main.rs",
    "apps/desktop/src-tauri/src/linux_tray.rs",
    "apps/desktop/src-tauri/src/incident_policy.rs",
    "apps/desktop/src-tauri/src/cloudflare.rs",
    "apps/desktop/src-tauri/src/analytics.rs",
    "apps/desktop/src-tauri/src/security.rs",
    "apps/desktop/src-tauri/src/api_budget.rs",
    "apps/desktop/src-tauri/src/storage.rs"
  ]) {
    const source = read(path);
    for (const match of source.matchAll(/msg::([A-Z_][A-Z_0-9]*)/g)) {
      assert.ok(declared.has(match[1]), `${path}: undefined native message ${match[1]}`);
    }
  }
  for (const name of ["TRAY_SHOW_CARD","TRAY_EVENT_HISTORY","TRAY_SETTINGS","TRAY_ABOUT","TRAY_EXIT"]) {
    assert.ok(declared.has(name), `Missing tray label: ${name}`);
  }
});

test("legacy persisted incident strings stay byte-for-byte compatible with English catalog", () => {
  const declared = constants(native);
  const keys = {
    EVENT_BUILD_FAILURE:"event.buildFailure",
    EVENT_RUNTIME_SPIKE:"event.runtimeSpike",
    EVENT_WAF_SPIKE:"event.wafSpike",
    EVENT_RUNTIME_RECOVERY:"event.runtimeRecovery",
    EVENT_WAF_RECOVERY:"event.wafRecovery",
    EVENT_BUILD_STARTED:"event.buildStarted",
    EVENT_BUILD_SUCCESS:"event.buildSuccess"
  };
  for (const [nativeKey, translatedKey] of Object.entries(keys)) {
    assert.equal(declared.get(nativeKey), english[translatedKey]);
  }
  assert.match(read("apps/desktop/src-tauri/src/main.rs"), /fn classify_history\(message: &str\)/);
  assert.match(read("apps/desktop/src-tauri/src/main.rs"), /msg::EVENT_BUILD_FAILURE/);
  assert.match(native, /pub fn worker_spike_detail\(/);
  assert.match(native, /pub fn waf_spike_detail\(/);
});
