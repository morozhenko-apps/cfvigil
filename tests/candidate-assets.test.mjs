import test from "node:test";
import assert from "node:assert/strict";
import {canonicalName, selectArtifacts, bundleFiles, validateDebMetadata, validateDebLauncherLayout} from "../scripts/candidate-assets.mjs";
import {mkdtempSync, mkdirSync, writeFileSync, symlinkSync, rmSync} from "node:fs";
import {tmpdir} from "node:os";
import path from "node:path";

test("candidate names are deterministic and versioned", () => {
  assert.equal(canonicalName("0.1.0",".deb"),"cfvigil_0.1.0_amd64.deb");
  assert.equal(canonicalName("0.1.0",".AppImage"),"CFVigil-0.1.0-linux-x64.AppImage");
  assert.throws(()=>canonicalName("0.1.0",".exe"),/Unexpected/);
  assert.throws(()=>canonicalName("../bad",".deb"),/Invalid/);
});

test("reject empty, duplicate, stale or incomplete platform bundles", () => {
  const filenames=["deb/CFVigil_0.1.0_amd64.deb","appimage/CFVigil_0.1.0_amd64.AppImage"];
  assert.deepEqual(selectArtifacts(filenames,"0.1.0"),{
    ".deb":filenames[0],".AppImage":filenames[1]
  });
  assert.throws(()=>selectArtifacts(filenames.slice(0,1),"0.1.0"),/AppImage/);
  assert.throws(()=>selectArtifacts([...filenames,"deb/another_0.1.0_amd64.deb"],"0.1.0"),/exactly one/);
  assert.deepEqual(selectArtifacts([...filenames,"other/ignored_0.1.0_amd64.deb"],"0.1.0"),{
    ".deb":filenames[0],".AppImage":filenames[1]
  });
  assert.throws(()=>selectArtifacts(filenames,"0.2.0"),/Unexpected/);
});

test("DEB package identity and mandatory native KDE dependency are checked", () => {
  const okay={Package:"cf-vigil",Version:"0.1.0",Architecture:"amd64",
    Depends:"libgtk-layer-shell0, libwebkit2gtk-4.1-0"};
  assert.doesNotThrow(()=>validateDebMetadata(okay,"0.1.0"));
  for(const diff of [
    {...okay,Package:"cfvigil"},
    {...okay,Package:"repoticker"},
    {...okay,Version:"0.2.0"},
    {...okay,Architecture:"arm64"},
    {...okay,Depends:"libwebkit2gtk-4.1-0"}
  ]) assert.throws(()=>validateDebMetadata(diff,"0.1.0"));
});

test("ignore AppImage staging symlinks but reject symlinks replacing final packages", () => {
  const root = mkdtempSync(path.join(tmpdir(),"cfvigil-bundles-"));
  try {
    mkdirSync(path.join(root,"deb"));
    mkdirSync(path.join(root,"appimage"));
    const deb = path.join(root,"deb","CFVigil_0.1.0_amd64.deb");
    const appimage = path.join(root,"appimage","CFVigil_0.1.0_amd64.AppImage");
    writeFileSync(deb,"debian");
    writeFileSync(appimage,"appimage");
    mkdirSync(path.join(root,"appimage","CFVigil.AppDir"));
    symlinkSync(root,path.join(root,"appimage","CFVigil.AppDir","lib"));
    const names=bundleFiles(root);
    assert.deepEqual(names.sort(),[
      "appimage/CFVigil_0.1.0_amd64.AppImage",
      "deb/CFVigil_0.1.0_amd64.deb",
    ]);
    assert.deepEqual(selectArtifacts(names,"0.1.0"),{
      ".deb":"deb/CFVigil_0.1.0_amd64.deb",
      ".AppImage":"appimage/CFVigil_0.1.0_amd64.AppImage",
    });
    rmSync(appimage);
    symlinkSync(deb,appimage);
    assert.throws(()=>bundleFiles(root),/Unsafe final package/);
  } finally { rmSync(root,{recursive:true,force:true}); }
});

test("Debian menu entry must reference an installed themed CFVigil PNG", () => {
  const files = [
    "16x16/apps/cfvigil.png",
    "32x32/apps/cfvigil.png",
    "256x256/apps/cfvigil.png",
    "scalable/apps/unrelated.svg",
  ];
  assert.deepEqual(validateDebLauncherLayout(
    "[Desktop Entry]\nName=CFVigil\nExec=cfvigil\nIcon=cfvigil\n", files), {
    name:"cfvigil", paths:files.slice(0,3),
  });
  assert.throws(() => validateDebLauncherLayout(
    "[Desktop Entry]\nName=CFVigil Desktop\nExec=cfvigil\nIcon=cfvigil\n",files),
    /display CFVigil/);
  assert.throws(() => validateDebLauncherLayout(
    "[Desktop Entry]\nName=CFVigil\nExec=cfvigil-desktop\nIcon=cfvigil\n",files),
    /execute canonical cfvigil binary/);
  assert.doesNotThrow(() => validateDebLauncherLayout(
    "[Desktop Entry]\nName=CFVigil\nExec=\/usr\/bin\/cfvigil %U\nIcon=cfvigil\n",files));
  assert.throws(() => validateDebLauncherLayout(
    "[Desktop Entry]\nName=CFVigil\nExec=cfvigil\nIcon=old-default\n", files), /canonical CFVigil icon name/);
  assert.throws(() => validateDebLauncherLayout(
    "[Desktop Entry]\nName=CFVigil\nExec=cfvigil\nIcon=cfvigil-desktop\n", ["32x32/apps/cfvigil-desktop.png"]), /canonical CFVigil icon name/);
  assert.throws(() => validateDebLauncherLayout(
    "[Desktop Entry]\nName=CFVigil\nExec=cfvigil\n", files), /exactly one Icon/);
  assert.throws(() => validateDebLauncherLayout(
    "[Desktop Entry]\nName=CFVigil\nExec=cfvigil\nIcon=../../outside\n", files), /safe themed icon/);
  assert.throws(() => validateDebLauncherLayout(
    "[Desktop Entry]\nName=CFVigil\nExec=cfvigil\nIcon=cfvigil\nIcon=old-default\n", files),
    /exactly one Icon/);
});
