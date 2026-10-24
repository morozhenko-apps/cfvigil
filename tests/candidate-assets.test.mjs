import test from "node:test";
import assert from "node:assert/strict";
import {canonicalName, selectArtifacts, validateDebMetadata} from "../scripts/candidate-assets.mjs";

test("candidate names are deterministic and versioned", () => {
  assert.equal(canonicalName("0.1.0",".deb"),"cfvigil_0.1.0_amd64.deb");
  assert.equal(canonicalName("0.1.0",".AppImage"),"CFVigil-0.1.0-linux-x64.AppImage");
  assert.throws(()=>canonicalName("0.1.0",".exe"),/Unexpected/);
  assert.throws(()=>canonicalName("../bad",".deb"),/Invalid/);
});

test("reject empty, duplicate, stale or incomplete platform bundles", () => {
  const filenames=["deb/cfvigil_0.1.0_amd64.deb","appimage/cfvigil_0.1.0_amd64.AppImage"];
  assert.deepEqual(selectArtifacts(filenames,"0.1.0"),{
    ".deb":filenames[0],".AppImage":filenames[1]
  });
  assert.throws(()=>selectArtifacts(filenames.slice(0,1),"0.1.0"),/AppImage/);
  assert.throws(()=>selectArtifacts([...filenames,"other/cfvigil_0.1.0_amd64.deb"],"0.1.0"),/exactly one/);
  assert.throws(()=>selectArtifacts(filenames,"0.2.0"),/exactly one/);
});

test("DEB package identity and mandatory native KDE dependency are checked", () => {
  const okay={Package:"cfvigil",Version:"0.1.0",Architecture:"amd64",
    Depends:"libgtk-layer-shell0, libwebkit2gtk-4.1-0"};
  assert.doesNotThrow(()=>validateDebMetadata(okay,"0.1.0"));
  for(const diff of [
    {...okay,Package:"repoticker"},
    {...okay,Version:"0.2.0"},
    {...okay,Architecture:"arm64"},
    {...okay,Depends:"libwebkit2gtk-4.1-0"}
  ]) assert.throws(()=>validateDebMetadata(diff,"0.1.0"));
});
