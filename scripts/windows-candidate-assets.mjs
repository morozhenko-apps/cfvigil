#!/usr/bin/env node
// Nonpublishing Windows NSIS candidate artifact validation.
import {openSync,readSync,closeSync,readdirSync,lstatSync,mkdirSync,copyFileSync,writeFileSync,existsSync} from "node:fs";
import {createHash} from "node:crypto";
import path from "node:path";
import {fileURLToPath} from "node:url";
import {sourceVersion} from "./candidate-version.mjs";

export function expectedNsisFilename(version) {
  if (!/^(0|[1-9]\d*)\.(0|[1-9]\d*)\.(0|[1-9]\d*)$/.test(version)) {
    throw Error("Invalid candidate version");
  }
  return `CFVigil_${version}_x64-setup.exe`;
}

export function selectNsis(names,version) {
  const executables = names.filter(name => name.toLowerCase().endsWith(".exe"));
  const expected = expectedNsisFilename(version);
  if (executables.length !== 1 || executables[0] !== expected) {
    throw Error(`Unexpected Windows NSIS candidates: ${JSON.stringify(executables)} (expected ${expected})`);
  }
  return expected;
}

function head(file,size) {
  const fd=openSync(file,"r");
  try {
    const bytes=Buffer.alloc(size);
    if (readSync(fd,bytes,0,size,0)!==size) throw Error("Truncated Windows executable");
    return bytes;
  } finally {closeSync(fd);}
}

export function validPeHeader(buffer) {
  if (buffer.length<64 || buffer.toString("ascii",0,2)!=="MZ") return false;
  const offset=buffer.readUInt32LE(60);
  // For fixed 4 KiB header probe, PE signature must be in inspected region.
  return offset>=64 && offset+4<=buffer.length
      && buffer.toString("latin1",offset,offset+4)==="PE\0\0";
}

function sha256(filename) {
  const hash=createHash("sha256");
  const fd=openSync(filename,"r");
  try {
    const buffer=Buffer.allocUnsafe(1024*1024);
    let bytes=0;
    while ((bytes=readSync(fd,buffer,0,buffer.length,null))>0) {
      hash.update(buffer.subarray(0,bytes));
    }
    return hash.digest("hex");
  } finally {closeSync(fd);}
}

export function collect(source,target,version=sourceVersion()) {
  const folder=path.join(source,"nsis");
  if (lstatSync(folder).isSymbolicLink() || !lstatSync(folder).isDirectory()) {
    throw Error("NSIS output missing or unsafe");
  }
  const actual=selectNsis(readdirSync(folder),version);
  const original=path.join(folder,actual);
  const info=lstatSync(original);
  if (!info.isFile() || info.isSymbolicLink() || info.size<1024 ||
      info.size>350*1024*1024) throw Error("Invalid NSIS package");
  if (!validPeHeader(head(original,4096))) throw Error("NSIS installer is not valid PE");
  if (existsSync(target) && readdirSync(target).length) {
    throw Error("Refusing nonempty candidate output");
  }
  mkdirSync(target,{recursive:true});
  const filename=`CFVigil-Setup-${version}-win-x64.exe`;
  const destination=path.join(target,filename);
  copyFileSync(original,destination);
  const digest=sha256(destination);
  writeFileSync(destination+".sha256",`${digest}  ${filename}\n`,{flag:"wx"});
  writeFileSync(path.join(target,"candidate.json"),
    JSON.stringify({schema:1,product:"CFVigil",version,platform:"windows-x64",
      release:false, signed:false, commit:process.env.GITHUB_SHA||null,
      files:[{name:filename,sha256:digest,size:info.size}]},null,2)+"\n",{flag:"wx"});
  console.log(`Verified unsigned CFVigil ${version} NSIS candidate`);
}

if (process.argv[1] && path.resolve(process.argv[1])===fileURLToPath(import.meta.url)) {
  try {
    const args=process.argv.slice(2);
    if (args.length!==5 || args[0]!=="collect" ||
        args[1]!=="--source" || args[3]!=="--target") throw Error("Usage: windows-candidate-assets.mjs collect --source FOLDER --target FOLDER");
    collect(path.resolve(args[2]),path.resolve(args[4]));
  } catch(e) {
    console.error(e.message);
    process.exitCode=1;
  }
}
