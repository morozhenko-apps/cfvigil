// Verify committed desktop icon formats all originate from the SAME owner PNG.
// No image library, network access, temp files or side effects in CI.
import {readFileSync, existsSync} from "node:fs";
import {createHash} from "node:crypto";
import {fileURLToPath} from "node:url";
import path from "node:path";

const ROOT = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const source = "assets/branding/cfvigil_icon.png";
const icons = "apps/desktop/src-tauri/icons";
const ui = "apps/desktop/ui/icon.png";
const manifestPath = icons + "/source-manifest.json";
const sizes = [16,24,32,48,64,128,256,512];
const expected = new Map(sizes.map(s => [icons + "/icon-" + s + ".png", s]));
expected.set(icons + "/icon.png", 256);
expected.set(ui, 128);
expected.set(icons + "/icon.ico", "ICO");
expected.set(icons + "/icon.icns", "ICNS");

function content(relative) {
  const file = path.join(ROOT, relative);
  if (!existsSync(file)) throw Error("Brand asset missing: " + relative);
  return readFileSync(file);
}
function digest(data) {
  return createHash("sha256").update(data).digest("hex");
}
function png(data, expectedSize = null) {
  if (data.length < 33 || !data.subarray(0, 8).equals(
    Buffer.from([137,80,78,71,13,10,26,10]))) {
    throw Error("Invalid PNG signature");
  }
  const width = data.readUInt32BE(16);
  const height = data.readUInt32BE(20);
  if (width !== height || width < 1 || data[25] !== 6) {
    throw Error("PNG must be square and RGBA");
  }
  if (expectedSize !== null && width !== expectedSize) {
    throw Error("Incorrect generated icon size: " + width + " vs " + expectedSize);
  }
  return width;
}

const sourceBytes = content(source);
const sourceSize = png(sourceBytes);
if (sourceSize < 512) throw Error("Icon source must be >=512px");
const manifest = JSON.parse(content(manifestPath).toString("utf8"));
if (manifest.source !== source ||
    manifest.source_sha256 !== digest(sourceBytes) ||
    JSON.stringify(manifest.source_size) !== JSON.stringify([sourceSize,sourceSize])) {
  throw Error("Source PNG changed: regenerate platform icons with tools/import_icon.py");
}
const entries = Object.entries(manifest.files ?? {});
if (entries.length !== expected.size ||
    entries.some(([name]) => !expected.has(name))) {
  throw Error("Incomplete/unknown icon manifest outputs");
}
for (const [relative, type] of expected) {
  const data = content(relative);
  if (digest(data) !== manifest.files[relative]) {
    throw Error("Stale or modified icon: " + relative);
  }
  if (typeof type === "number") {
    png(data, type);
  } else if (type === "ICO") {
    if (data.length < 6 || !data.subarray(0, 4).equals(
      Buffer.from([0,0,1,0]))) throw Error("Invalid Windows ICO");
  } else if (type === "ICNS") {
    if (data.length < 8 || data.toString("ascii", 0, 4) !== "icns") {
      throw Error("Invalid macOS ICNS");
    }
  }
}
console.log("PASS: " + expected.size + " native/WebView icons match the original source PNG");
