import test from "node:test";
import assert from "node:assert/strict";
import {expectedNsisFilename,selectNsis,validPeHeader} from "../scripts/windows-candidate-assets.mjs";

test("Windows candidates have one exact versioned NSIS installer",()=>{
  assert.equal(expectedNsisFilename("0.1.0"),"CFVigil_0.1.0_x64-setup.exe");
  assert.equal(selectNsis(["CFVigil_0.1.0_x64-setup.exe"],"0.1.0"),"CFVigil_0.1.0_x64-setup.exe");
  assert.throws(()=>selectNsis(["CFVigil_0.1.0_x64-setup.exe","other.exe"],"0.1.0"));
  assert.throws(()=>selectNsis(["CFVigil_0.2.0_x64-setup.exe"],"0.1.0"));
  assert.throws(()=>selectNsis(["CFVigil_0.1.0_x64-setup.exe","other_0.1.0.exe"],"0.1.0"));
});

test("reject non-PE bytes and out of bounds PE signatures",()=>{
  assert.equal(validPeHeader(Buffer.alloc(4096)),false);
  const data=Buffer.alloc(4096);
  data.write("MZ",0);
  data.writeUInt32LE(128,60);
  data.write("PE\0\0",128,"binary");
  assert.equal(validPeHeader(data),true);
  data.writeUInt32LE(8192,60);
  assert.equal(validPeHeader(data),false);
});
