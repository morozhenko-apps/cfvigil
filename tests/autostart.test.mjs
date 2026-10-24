import test from "node:test";
import assert from "node:assert/strict";
import {readFileSync} from "node:fs";

const root = new URL("../", import.meta.url);
const get = file => readFileSync(new URL(file,root), "utf8");
const main = get("apps/desktop/src-tauri/src/main.rs");
const ui = get("apps/desktop/ui/settings.mjs");
const html = get("apps/desktop/ui/settings.html");
const caps = get("apps/desktop/src-tauri/capabilities/default.json");

test("launch at login is opt-in, native only, and survives missing OS state", () => {
  assert.match(html, /id="autostart" type="checkbox" disabled/);
  assert.match(ui, /autostartLoaded \? autostart\.checked : null/);
  assert.match(ui, /autostart\.disabled = true/);
  assert.match(main, /fn get_autostart\(app: AppHandle\)/);
  assert.match(main, /fn update_autostart\(app: &AppHandle, requested: Option<bool>\)/);
  assert.match(main, /\.plugin\(tauri_plugin_autostart::Builder::new\(\)\.build\(\)\)/);
  assert.doesNotMatch(caps, /autostart:allow-(enable|disable|is-enabled)/);
});

test("autostart-only saves never trigger Cloudflare refreshes", () => {
  assert.match(main, /let should_refresh = needs_refresh\(&previous, &next, token_replaced\)/);
  assert.match(main, /if should_refresh \{/);
  assert.match(main, /update_autostart\(&app, autostart\)/);
});
