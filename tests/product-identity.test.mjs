import test from "node:test";
import assert from "node:assert/strict";
import {readFileSync} from "node:fs";
import {fileURLToPath} from "node:url";
import path from "node:path";

const root=path.resolve(path.dirname(fileURLToPath(import.meta.url)),"..");
const read=file=>readFileSync(path.join(root,file),"utf8");
const tauri=JSON.parse(read("apps/desktop/src-tauri/tauri.conf.json"));
const cargo=read("apps/desktop/src-tauri/Cargo.toml");
const lock=read("Cargo.lock");
const native=read("apps/desktop/src-tauri/src/main.rs");
const scopes=[
  ".github/workflows/package-candidate.yml",
  ".github/workflows/windows-candidate.yml",
  ".github/workflows/protected-release.yml"
];

test("Mode B identity: product, executable, package and main bundle agree",()=>{
  assert.equal(tauri.productName,"CFVigil");
  assert.equal(tauri.identifier,"com.morozhenko.cfvigil");
  assert.match(cargo,/^\[package\]\s*\nname = "cfvigil"\s*$/m);
  assert.match(lock,/^\[\[package\]\]\s*\nname = "cfvigil"\s*\nversion = "0\.1\.0"/m);
  assert.doesNotMatch(cargo,/cfvigil-desktop/);
  assert.doesNotMatch(lock,/name = "cfvigil-desktop"/);
});

test("Mode B identity: native KDE History and About icons match the actual Cargo binary",()=>{
  const icons=native.match(/native\.set_icon_name\(Some\("cfvigil"\)\)/g)??[];
  assert.equal(icons.length,2,"startup + show-time for all four windows");
  assert.doesNotMatch(native,/native\.set_icon_name\(Some\("cfvigil-desktop"\)\)/);
  assert.match(native,/for label in \["main", "settings", "history", "about"\]/);
  assert.match(native,/window\.set_icon\(icon\)/);
  assert.equal(tauri.app.windows.find(w=>w.label==="about").decorations,false);
});

test("Mode B identity: all Linux, Windows and protected release jobs select one canonical Rust package",()=>{
  for(const file of scopes){
    const script=read(file);
    assert.match(script,/cargo check --locked -p cfvigil(?:\s|$)/m,file);
    assert.match(script,/cargo test --locked -p cfvigil(?:\s|$)/m,file);
    assert.doesNotMatch(script,/cfvigil-desktop/,file);
    assert.match(script,/cargo tauri build --bundles /,file);
  }
  assert.match(read(".github/workflows/checks.yml"),/cargo check --locked -p cfvigil/);
  assert.match(read(".github/workflows/checks.yml"),/cargo test --locked -p cfvigil/);
});

test("Mode B identity: new binary must NOT create a new user profile or credential namespace",()=>{
  assert.match(native,/const KEYRING_SERVICE: &str = "com\.morozhenko\.cfvigil\.cloudflare";/);
  assert.equal(tauri.identifier,"com.morozhenko.cfvigil");
  assert.equal(tauri.productName,"CFVigil");
  assert.match(native,/TrayIconBuilder::with_id\("cfvigil"\)/);
});

test("Mode B identity: installed package name and download URLs remain backward-compatible",()=>{
  const linux=read("scripts/candidate-assets.mjs");
  const windows=read("scripts/windows-candidate-assets.mjs");
  const release=read(".github/workflows/protected-release.yml");
  assert.match(linux,/metadata\.Package !== "cf-vigil"/);
  assert.match(linux,/Icon= must be a safe themed icon name/);
  assert.match(linux,/name !== "cfvigil"/);
  assert.match(windows,/CFVigil-Setup-\$\{version\}-win-x64\.exe/);
  assert.match(release,/download\.morozhenko\.com\/dist\/CFVigil\//);
  assert.doesNotMatch(release,/dist\/CFVigilDesktop/);
});
