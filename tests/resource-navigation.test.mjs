import test from "node:test";
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { nextResourceIndex } from "../apps/desktop/ui/resource-navigation.mjs";

const read = p => readFileSync(new URL("../" + p, import.meta.url), "utf8");

test("arrow, home and end navigation wrap only among visible resource actions", () => {
  assert.equal(nextResourceIndex(0, 4, "ArrowUp"), 3);
  assert.equal(nextResourceIndex(3, 4, "ArrowDown"), 0);
  assert.equal(nextResourceIndex(-1, 4, "ArrowDown"), 0);
  assert.equal(nextResourceIndex(-1, 4, "ArrowUp"), 3);
  assert.equal(nextResourceIndex(2, 4, "Home"), 0);
  assert.equal(nextResourceIndex(2, 4, "End"), 3);
  assert.equal(nextResourceIndex(2, 4, "Enter"), -1);
  assert.equal(nextResourceIndex(0, 0, "ArrowDown"), -1);
  assert.equal(nextResourceIndex(0, -1, "Home"), -1);
});

test("resource action menu is accessible and keyboard focused", () => {
  const js = read("apps/desktop/ui/app.mjs");
  const css = read("apps/desktop/ui/style.css");
  assert.match(js, /role", "menu"/);
  assert.match(js, /role", "menuitem"/);
  assert.match(js, /aria-haspopup", "menu"/);
  assert.match(js, /aria-expanded", "false"/);
  assert.match(js, /key === "Escape"/);
  assert.match(js, /nextResourceIndex\(/);
  assert.match(js, /menu\.append\(open\)/);
  assert.match(js, /open\.focus\(\)/);
  assert.match(css, /\.resource-row:focus-within/);
  assert.match(css, /:focus-visible/);
});

test("incident focus and collapsed five-cell contract remain intact", () => {
  const js = read("apps/desktop/ui/app.mjs");
  const html = read("apps/desktop/ui/index.html");
  assert.match(js, /focusedKey = payload\.key/);
  assert.match(js, /byId\("search"\)\.value = ""/);
  assert.match(js, /await invoke\("acknowledge_incident"\)/);
  assert.match(js, /focusResource\(focusedKey\)/);
  assert.match(html, /class="actions"/);
  assert.equal((html.match(/class="metric /g)||[]).length, 3);
  assert.equal((html.match(/class="action"/g)||[]).length, 2);
});
