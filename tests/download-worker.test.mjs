import test from "node:test";
import assert from "node:assert/strict";
import worker, {keyFor} from "../infra/download-worker/src/worker.mjs";

const base = "https://download.morozhenko.com/dist/CFVigil/";
test("CFVigil Worker maps only precisely named public R2 files",()=>{
  const allowed=[
    "releases/v1.2.3/linux/cfvigil_1.2.3_amd64.deb",
    "releases/v1.2.3/linux/CFVigil-1.2.3-linux-x64.AppImage.sha256",
    "releases/v1.2.3/windows/CFVigil-Setup-1.2.3-win-x64.exe",
    "apt/dists/stable/InRelease",
    "apt/dists/stable/main/binary-amd64/Packages.gz",
    "apt/dists/stable/main/binary-amd64/by-hash/SHA256/"+"a".repeat(64),
    "apt/pool/main/c/cf-vigil/cfvigil_1.2.3_amd64.deb",
    "apt/cfvigil-archive-keyring.gpg",
  ];
  for(const item of allowed) assert.equal(keyFor(base+item),"dist/CFVigil/"+item);
  for(const bad of [
    "../RepoTicker/releases/v1.2.3/linux/a.deb",
    "releases/v1.2.3/linux/other.deb",
    "releases/v1.2.3/linux/cfvigil_1.2.4_amd64.deb",
    "releases/v1.2.3/windows/CFVigil-Setup-1.2.3-win-x86.exe",
    "apt/dists/stable/Release",
    "releases/v1.2.3/linux/%2e%2e/a",
    "releases/v1.2.3/linux//cfvigil_1.2.3_amd64.deb",
    "apt/../../RepoTicker/secret",
    "apt/dists/stable/InRelease?download=1",
  ]) assert.equal(keyFor(base+bad),null,bad);
  assert.equal(keyFor("https://download.morozhenko.com/dist/RepoTicker/apt/dists/stable/InRelease"),null);
});

test("read-only Worker only exposes GET and HEAD, caches mutable APT safely",async()=>{
  const events=[];
  const content=new TextEncoder().encode("demo");
  const env={ARCHIVE:{
    get:async key=>{events.push(["get",key]);return {body:content,size:4,httpEtag:'"123"'};},
    head:async key=>{events.push(["head",key]);return {size:4,httpEtag:'"123"'};}
  }};
  const url=base+"apt/dists/stable/InRelease";
  const post=await worker.fetch(new Request(url,{method:"POST"}),env);
  assert.equal(post.status,405);
  assert.equal(post.headers.get("allow"),"GET, HEAD");
  assert.equal(events.length,0);
  const get=await worker.fetch(new Request(url),env);
  assert.equal(get.status,200);
  assert.equal(get.headers.get("cache-control"),"no-store");
  assert.equal(get.headers.get("x-content-type-options"),"nosniff");
  assert.equal(await get.text(),"demo");
  const head=await worker.fetch(new Request(url,{method:"HEAD"}),env);
  assert.equal(head.status,200);
  assert.equal(await head.text(),"");
  assert.deepEqual(events.map(event=>event[0]),["get","head"]);
  const absent=await worker.fetch(new Request(base+"other"),env);
  assert.equal(absent.status,404);
  assert.equal(events.length,2);
});
