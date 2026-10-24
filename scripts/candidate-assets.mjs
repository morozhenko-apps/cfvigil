#!/usr/bin/env node
// Validate and stage unsigned local Linux candidates. Never uploads remotely.
import { openSync, readSync, closeSync, readFileSync, readdirSync, existsSync, lstatSync, mkdirSync, copyFileSync, writeFileSync } from "node:fs";
import path from "node:path";
import { createHash } from "node:crypto";
import { execFileSync } from "node:child_process";
import { fileURLToPath } from "node:url";
import { sourceVersion } from "./candidate-version.mjs";

const MAX_BYTES = 350 * 1024 * 1024;
const MIN_BYTES = 1024;
const SUFFIXES = [".deb", ".AppImage"];

export function canonicalName(version, suffix) {
  if (!/^\d+\.\d+\.\d+$/.test(version)) throw Error("Invalid candidate version");
  if (suffix === ".deb") return `cfvigil_${version}_amd64.deb`;
  if (suffix === ".AppImage") return `CFVigil-${version}-linux-x64.AppImage`;
  throw Error("Unexpected bundle type");
}

// Read only final Tauri bundle files. AppImage builds legitimately contain
// symlinks in intermediate AppDir directories; they are not distributables.
export function selectArtifacts(names, version) {
  const selected = {};
  for (const [dir, suffix] of [["deb", ".deb"], ["appimage", ".AppImage"]]) {
    const candidates = names.filter(name => name.startsWith(dir + "/") &&
      !name.slice(dir.length + 1).includes("/") && name.endsWith(suffix));
    if (candidates.length !== 1) {
      throw Error(`Expected exactly one ${suffix} package, found ${candidates.length}`);
    }
    const expected = `${dir}/CFVigil_${version}_amd64${suffix}`;
    if (candidates[0] !== expected) {
      throw Error(`Unexpected Tauri ${suffix} bundle filename/version`);
    }
    selected[suffix] = candidates[0];
  }
  return selected;
}

export function bundleFiles(root) {
  if (lstatSync(root).isSymbolicLink() || !lstatSync(root).isDirectory()) {
    throw Error("Unsafe package output root");
  }
  const result = [];
  for (const [dir, suffix] of [["deb", ".deb"], ["appimage", ".AppImage"]]) {
    const folder = path.join(root, dir);
    if (lstatSync(folder).isSymbolicLink() || !lstatSync(folder).isDirectory()) {
      throw Error("Unsafe package output directory");
    }
    for (const filename of readdirSync(folder)) {
      // Never traverse AppDir, AppImage build tools or their symlink trees.
      if (!filename.endsWith(suffix)) continue;
      const candidate = path.join(folder, filename);
      const info = lstatSync(candidate);
      if (info.isSymbolicLink() || !info.isFile()) {
        throw Error("Unsafe final package (not a regular file)");
      }
      result.push(`${dir}/${filename}`);
    }
  }
  return result;
}

function sha256(filename) {
  const hash = createHash("sha256");
  const fd = openSync(filename, "r");
  try {
    const buffer = Buffer.allocUnsafe(1024 * 1024);
    let read;
    while ((read = readSync(fd, buffer, 0, buffer.length, null)) !== 0) {
      hash.update(buffer.subarray(0, read));
    }
    return hash.digest("hex");
  } finally { closeSync(fd); }
}

function prefix(filename, length) {
  const fd = openSync(filename, "r");
  try {
    const buffer = Buffer.alloc(length);
    if (readSync(fd, buffer, 0, length, 0) !== length) {
      throw Error("Truncated package header");
    }
    return buffer;
  } finally { closeSync(fd); }
}

function field(filename, fieldName) {
  return execFileSync("dpkg-deb", ["-f", filename, fieldName],
    {encoding:"utf8", maxBuffer:65536}).trim();
}

export function validateDebMetadata(metadata, version) {
  // Tauri 2 uses heck::AsKebabCase(productName) for Debian's Package
  // field. "CFVigil" becomes "cf-vigil", regardless of Cargo's binary name.
  if (metadata.Package !== "cf-vigil" || metadata.Version !== version ||
      metadata.Architecture !== "amd64") {
    const {Package, Version, Architecture} = metadata;
    throw Error(`Unexpected Debian identity: Package=${JSON.stringify(Package)}, Version=${JSON.stringify(Version)}, Architecture=${JSON.stringify(Architecture)}`);
  }
  if (!/(?:^|[, ])libgtk-layer-shell0(?:\s|\(|,|$)/.test(metadata.Depends ?? "")) {
    throw Error("Missing libgtk-layer-shell0 runtime dependency");
  }
}

function collect(source, target) {
  const version = sourceVersion();
  if (!existsSync(source) || !lstatSync(source).isDirectory()) {
    throw Error("Bundle output directory missing");
  }
  if (existsSync(target) && readdirSync(target).length) {
    throw Error("Refusing nonempty candidate staging directory");
  }
  const paths = bundleFiles(source);
  const choice = selectArtifacts(paths, version);
  const prepared = [];
  for (const suffix of SUFFIXES) {
    const rel = choice[suffix];
    const from = path.join(source, rel);
    const size = lstatSync(from).size;
    if (size < MIN_BYTES || size > MAX_BYTES) throw Error("Invalid bundle size");
    if (suffix === ".deb") {
      validateDebMetadata({
        Package:field(from,"Package"), Version:field(from,"Version"),
        Architecture:field(from,"Architecture"), Depends:field(from,"Depends"),
      }, version);
      if (prefix(from,8).toString("ascii") !== "!<arch>\n") {
        throw Error("Invalid Debian archive header");
      }
    } else {
      const magic = prefix(from,4);
      if (!magic.equals(Buffer.from([0x7f,0x45,0x4c,0x46]))) {
        throw Error("Invalid AppImage ELF header");
      }
    }
    prepared.push({from,name:canonicalName(version,suffix),size});
  }
  mkdirSync(target,{recursive:true});
  const files = [];
  for (const item of prepared) {
    const to = path.join(target,item.name);
    copyFileSync(item.from,to);
    const digest = sha256(to);
    writeFileSync(to+".sha256",`${digest}  ${item.name}\n`,{flag:"wx"});
    files.push({name:item.name,sha256:digest,size:item.size});
  }
  const manifest = {
    schema:1, product:"CFVigil", version, platform:"linux-amd64",
    release:false, commit:process.env.GITHUB_SHA || null, files
  };
  writeFileSync(path.join(target,"candidate.json"),
    JSON.stringify(manifest,null,2)+"\n",{flag:"wx"});
  console.log(`Verified unsigned CFVigil ${version} candidate: DEB and AppImage`);
}

function main(args) {
  if (args.length !== 5 || args[0] !== "collect" ||
      args[1] !== "--source" || args[3] !== "--target") {
    throw Error("Usage: candidate-assets.mjs collect --source PATH --target PATH");
  }
  collect(path.resolve(args[2]),path.resolve(args[4]));
}
if (process.argv[1] && path.resolve(process.argv[1]) === fileURLToPath(import.meta.url)) {
  try { main(process.argv.slice(2)); }
  catch(error) { console.error(error.message); process.exitCode=1; }
}
