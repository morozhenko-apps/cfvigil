import test from "node:test";
import assert from "node:assert/strict";
import {readFileSync} from "node:fs";

const source = file => readFileSync(new URL("../" + file, import.meta.url),"utf8");
const native = source("apps/desktop/src-tauri/src/main.rs");
const persistence = source("apps/desktop/src-tauri/src/storage.rs");
const card = source("apps/desktop/ui/app.mjs");
const settings = source("apps/desktop/ui/settings.mjs");

test("all three local JSON records use the same atomic write and checked restore", () => {
  assert.match(native,/storage::read_or_default\(/);
  assert.match(native,/storage::write_atomic\(/);
  for (const file of ["settings.json","events.json","incident-state.json"]) {
    assert.match(native,new RegExp('\\"' + file.replaceAll(".", "\\.") + '\\"'));
  }
  assert.doesNotMatch(native,/\.and_then\(\|bytes\| serde_json::from_slice\(&bytes\)\.ok\(\)\)/);
  assert.match(native,/events\.truncate\(500\)/);
  assert.match(native,/settings\.quarantined \|\| stored\.quarantined \|\| history\.quarantined/);
});

test("recovery never overwrites damaged JSON before backing it up", () => {
  assert.match(persistence,/pub struct Loaded<T>/);
  assert.match(persistence,/fs::rename\(path, backup\)/);
  assert.match(persistence,/fs::rename\(&temp, path\)/);
  assert.match(persistence,/file\.sync_all\(\)/);
  assert.match(persistence,/create_new\(true\)/);
  assert.match(persistence,/MAX_JSON_BYTES: u64 = 4 \* 1024 \* 1024/);
  assert.match(persistence,/fn symlinked_state_does_not_follow_or_overwrite_other_files/);
});

test("damaged-state warning is accessible without changing the compact card", () => {
  assert.match(card,/payload\?\.storage_warning === true/);
  assert.match(card,/announce\(warning\)/);
  assert.match(settings,/snapshot\?\.storage_warning/);
  assert.match(settings,/\.corrupt\.\*/);
});
