import test from "node:test";
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";

const root = new URL("../", import.meta.url);
const read = (path) => readFileSync(new URL(path, root), "utf8");
const html = read("apps/desktop/ui/index.html");
const css = read("apps/desktop/ui/style.css");
const js = read("apps/desktop/ui/app.mjs");

test("compact card retains exactly three counters and two actions", () => {
  for (const id of ["healthy", "deploying", "issues", "toggle-list", "hide"]) {
    assert.match(html, new RegExp('id="' + id + '"'));
  }
  assert.equal((html.match(/class="metric /g) || []).length, 3);
  assert.equal((html.match(/class="action"/g) || []).length, 2);
  assert.match(css, /grid-template-columns:repeat\(5,/);
});

test("incident announcements do not render an overlay on the compact card", () => {
  assert.doesNotMatch(html, /id="incident"/);
  assert.doesNotMatch(html, /class="incident"/);
  assert.doesNotMatch(css, /\.incident\s*\{/);
  assert.match(html, /id="announcement"[^>]*class="sr-only"|id="announcement" class="sr-only"/);
  assert.match(html, /aria-live="polite"/);
  assert.match(css, /\.sr-only\{[^}]*clip-path:inset\(50%\)/);
  assert.doesNotMatch(js, /showIncident\(/);
});

test("incident focuses the resource without a visible status toast", () => {
  assert.match(js, /focusedKey = payload\.key/);
  assert.match(js, /byId\("search"\)\.value = ""/);
  assert.match(js, /button\.classList\.add\("blink"\)/);
  assert.match(js, /await invoke\("acknowledge_incident"\)/);
  assert.match(js, /if \(snapshot\.pending_incident\) presentIncident/);
  assert.match(js, /byId\("announcement"\)\.textContent = message/);
});
