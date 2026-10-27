import test from "node:test";
import assert from "node:assert/strict";
import {readFileSync} from "node:fs";
import {fileURLToPath} from "node:url";
import path from "node:path";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const read = file => readFileSync(path.join(root, file), "utf8");

test("Windows card uses one anchored native geometry path", () => {
  const main = read("apps/desktop/src-tauri/src/main.rs");
  const card = read("apps/desktop/src-tauri/src/windows_card.rs");
  assert.match(main, /windows_card::restore_for_show/);
  assert.match(main, /windows_card::resize_anchored/);
  assert.match(main, /windows_card::collapse_hidden/);
  assert.match(main, /windows_card::begin_drag/);
  assert.match(main, /windows_card::end_drag/);
  assert.match(main, /windows_card::on_window_event/);
  assert.match(card, /windows-card-position\.json/);
  assert.match(card, /work_area\(\)/);
  assert.match(card, /anchor\.bottom/);
  assert.match(card, /Duration::from_millis\(300\)/);
  assert.doesNotMatch(card, /linux_surface|gtk_layer_shell/);
  assert.match(main, /is_visible\(\)\.unwrap_or\(false\)/);
  assert.match(main, /hide_card\(tray\.app_handle\(\)\.clone\(\)\)/);
});

test("Windows release binary is GUI subsystem and does not show a console", () => {
  const main = read("apps/desktop/src-tauri/src/main.rs");
  assert.match(main, /cfg_attr\(all\(target_os = "windows", not\(debug_assertions\)\), windows_subsystem = "windows"\)/);
});

test("Settings saves asynchronously with visible progress and masked token state", () => {
  const main = read("apps/desktop/src-tauri/src/main.rs");
  const ui = read("apps/desktop/ui/settings.mjs");
  const html = read("apps/desktop/ui/settings.html");
  assert.match(main, /async fn get_token_status\(\) -> Result<bool, String>/);
  assert.match(main, /config_revision: AtomicU64/);
  assert.match(main, /fn superseded_revision\(revision: &AtomicU64, captured: u64\) -> bool/);
  assert.match(main, /revision\.load\(Ordering::Acquire\) != captured/);
  assert.equal((main.match(/if superseded_revision\(&state\.config_revision, revision\)/g) ?? []).length,2,
    "both error and successful provider replies must discard a superseded credential revision");
  assert.match(ui, /spinner\.hidden = !busy/);
  assert.match(ui, /if \(saving\) return/);
  assert.match(ui, /token\.placeholder = present \? "••••••••"/);
  assert.match(ui, /void refreshTokenStatus\(\)/);
  assert.match(html, /id="token-status"/);
  assert.match(html, /id="save-spinner"/);
  assert.doesNotMatch(ui, /localStorage|sessionStorage/);
});

test("Windows card uses measured outer size and its own resize completion anchor", () => {
  const card = read("apps/desktop/src-tauri/src/windows_card.rs");
  assert.match(card, /anchored_position\(anchor, measured_height\)/);
  assert.match(card, /window\.outer_size\(\)/);
  assert.match(card, /window\.inner_size\(\)\.ok\(\) == Some\(expected\)/);
  assert.match(card, /WindowEvent::Resized/);
  assert.match(card, /windows-card-geometry\.log/);
});

test("Windows card has no white native frame and hide icon is centered SVG", () => {
  const config = JSON.parse(read("apps/desktop/src-tauri/tauri.conf.json"));
  const main = config.app.windows.find(window => window.label === "main");
  assert.equal(main.transparent, true);
  assert.equal(main.shadow, false);
  const html = read("apps/desktop/ui/index.html");
  const css = read("apps/desktop/ui/style.css");
  assert.match(html, /id="hide"[^>]*><span class="glyph glyph-chevron"/);
  assert.match(html, /<svg viewBox="0 0 24 24"/);
  assert.doesNotMatch(html, />⌄<\/span>/);
  assert.match(css, /\.glyph-chevron\{display:grid;place-items:center;width:26px;height:26px/);
  assert.match(css, /\.glyph-chevron svg\{display:block;width:22px;height:22px/);
  assert.match(css, /html,body\{[^}]*background:transparent/);
});
