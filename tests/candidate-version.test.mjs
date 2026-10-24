import test from "node:test";
import assert from "node:assert/strict";
import { sourceVersion, readVersion, tagMatches, ROOT } from "../scripts/candidate-version.mjs";
import {readFileSync} from "node:fs";
import path from "node:path";

const source = name => readFileSync(path.join(ROOT, name), "utf8");
const cargo = source("apps/desktop/src-tauri/Cargo.toml");
const config = JSON.parse(source("apps/desktop/src-tauri/tauri.conf.json"));

test("bundle versions, identity, license and native Linux dependency agree", () => {
  assert.equal(sourceVersion(), "0.1.0");
  assert.equal(readVersion(cargo, config), "0.1.0");
  assert.equal(tagMatches("v0.1.0", "0.1.0"), true);
  for (const tag of ["0.1.0", "v0.1.1", "v00.1.0", "v0.1.0-rc1"]) {
    assert.equal(tagMatches(tag, "0.1.0"), false);
  }
});

test("version or bundle metadata drift cannot pass the package gate", () => {
  assert.throws(() => readVersion(cargo, {...config, version:"0.2.0"}), /differ/);
  assert.throws(() => readVersion(cargo, {...config, productName:"Other"}), /identity/);
  assert.throws(() => readVersion(cargo, {...config, bundle:{...config.bundle, active:false}}), /enabled/);
  assert.throws(() => readVersion(cargo, {...config, bundle:{...config.bundle, linux:{deb:{depends:[]}}}}), /libgtk/);
  assert.throws(() => readVersion(cargo, {...config, bundle:{...config.bundle, license:"MIT"}}), /Apache/);
  assert.throws(() => readVersion(cargo.replace('version = "0.1.0"', 'version = "1.0.0-beta"'), config), /Invalid/);
});
