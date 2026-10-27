import test from "node:test";
import assert from "node:assert/strict";
import { sourceVersion, readVersion, validateCargoLock, tagMatches, ROOT } from "../scripts/candidate-version.mjs";
import {readFileSync} from "node:fs";
import path from "node:path";

const source = name => readFileSync(path.join(ROOT, name), "utf8");
const cargo = source("apps/desktop/src-tauri/Cargo.toml");
const config = JSON.parse(source("apps/desktop/src-tauri/tauri.conf.json"));

test("bundle versions, identity, license and native Linux dependency agree", () => {
  assert.equal(sourceVersion(), "1.0.0");
  assert.equal(readVersion(cargo, config), "1.0.0");
  assert.match(cargo, /^name = "cfvigil"$/m);
  assert.doesNotMatch(cargo, /^name = "cfvigil-desktop"$/m);
  assert.equal(tagMatches("v1.0.0", "1.0.0"), true);
  for (const tag of ["1.0.0", "v1.0.1", "v01.0.0", "v1.0.0-rc1", "v0.1.0"]) {
    assert.equal(tagMatches(tag, "1.0.0"), false);
  }
});

test("version or bundle metadata drift cannot pass the package gate", () => {
  assert.throws(() => readVersion(cargo, {...config, version:"1.0.1"}), /differ/);
  assert.throws(() => readVersion(cargo.replace('name = "cfvigil"', 'name = "cfvigil-desktop"'), config), /Cargo\/binary identity/);
  assert.throws(() => readVersion(cargo, {...config, productName:"Other"}), /identity/);
  assert.throws(() => readVersion(cargo, {...config, bundle:{...config.bundle, active:false}}), /enabled/);
  assert.throws(() => readVersion(cargo, {...config, bundle:{...config.bundle, linux:{deb:{depends:[]}}}}), /libgtk/);
  assert.throws(() => readVersion(cargo, {...config, bundle:{...config.bundle, license:"MIT"}}), /Apache/);
  assert.throws(() => readVersion(cargo.replace('version = "1.0.0"', 'version = "1.0.0-beta"'), config), /Invalid/);
});

test("generated Cargo.lock is valid with Linux LF and Windows CRLF", () => {
  const lock = source("Cargo.lock");
  assert.doesNotThrow(() => validateCargoLock(lock));
  assert.doesNotThrow(() => validateCargoLock(lock, "1.0.0"));
  assert.throws(() => validateCargoLock(lock, "0.1.0"), /package version differs/);
  // A Windows Git checkout may contain CRLF: mutate the actual CFVigil
  // stanza, not whichever "version = 1.0.0" occurs first in Cargo.lock.
  const staleLock=lock.replace(/(name = "cfvigil"\r?\nversion = ")1\.0\.0"/,
    (_, prefix)=>prefix+'1.0.1"');
  assert.notEqual(staleLock, lock, "fixture must actually mutate CFVigil's version");
  assert.throws(() => validateCargoLock(staleLock, "1.0.0"), /package version differs/);
  assert.match(lock, /^\[\[package\]\]\r?\nname = "cfvigil"\r?\nversion = "/m);
  const crlf = lock.replace(/\r?\n/g, "\r\n");
  assert.doesNotThrow(() => validateCargoLock(crlf, "1.0.0"));
  assert.throws(() => validateCargoLock(lock.replace("version = 4", "version = 3")), /Invalid Cargo.lock/);
  assert.throws(() => validateCargoLock(lock.replace('name = "cfvigil"', 'name = "cfvigil-desktop"')), /package identity/);
  assert.throws(() => validateCargoLock(lock.replace("# It is not intended for manual editing.", "# modified")), /Invalid Cargo.lock/);
  assert.throws(() => validateCargoLock("version = 4\n"), /Invalid Cargo.lock/);
});
