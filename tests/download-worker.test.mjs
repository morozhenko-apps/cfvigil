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
    "releases/v1.2.3/linux/CFVigil-1.2.4-linux-x64.AppImage",
    "releases/v1.2.3/windows/CFVigil-Setup-1.2.4-win-x64.exe.sha256",
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


test("Mode B: unsupported HTTP methods never access private R2 storage",async()=>{
  const calls=[];
  const env={ARCHIVE:{
    get:async key=>{calls.push(["get",key]);throw new Error("unexpected storage call");},
    head:async key=>{calls.push(["head",key]);throw new Error("unexpected storage call");},
    put:async()=>{throw new Error("mutation forbidden");}
  }};
  for(const method of ["POST","PUT","DELETE","PATCH","OPTIONS"]){
    const response=await worker.fetch(new Request(base+"apt/dists/stable/InRelease",{method}),env);
    assert.equal(response.status,405,method);
    assert.equal(response.headers.get("allow"),"GET, HEAD");
    assert.equal(response.headers.get("cache-control"),"no-store");
  }
  assert.deepEqual(calls,[],"unsupported methods cannot touch R2 at all");
});

test("Mode B: invalid, foreign and path-smuggling URLs fail closed before any R2 read",async()=>{
  const called=[];
  const env={ARCHIVE:{
    get:async key=>{called.push(key);throw new Error("unexpected get");},
    head:async key=>{called.push(key);throw new Error("unexpected head");}
  }};
  const invalid=[
    "apt/dists/stable/InRelease?x=1",
    "apt/dists/stable/InRelease#fragment",
    "apt/%2e%2e/private",
    "apt//dists/stable/InRelease",
    "releases/v01.0.0/windows/CFVigil-Setup-01.0.0-win-x64.exe",
    "releases/v1.2.3/windows/CFVigil-Setup-1.2.3-win-x86.exe",
    "releases/v1.2.3/linux/../../secret",
    "releases/v1.2.3/linux/%2fhidden",
    "apt/dists/stable/main/binary-amd64/by-hash/SHA256/"+"F".repeat(64),
    "apt/dists/stable/main/binary-amd64/by-hash/SHA256/"+"g".repeat(64),
    "releases/v1.2.3/linux/cfvigil_1.2.3_amd64.deb/download"
  ];
  for(const relative of invalid){
    const url=base+relative;
    const response=await worker.fetch(new Request(url),env);
    assert.equal(response.status,404,relative);
  }
  assert.deepEqual(called,[]);
});

test("Mode B: immutable release binaries and mutable APT manifests use distinct cache policies",async()=>{
  const seen=[];
  const env={ARCHIVE:{
    get:async key=>{seen.push(["get",key]);return {
      body:new TextEncoder().encode("payload"),size:7,httpEtag:'"immutable"'
    };},
    head:async key=>{seen.push(["head",key]);return {size:7,httpEtag:'"immutable"'};}
  }};
  const cases=[
    ["releases/v1.2.3/linux/cfvigil_1.2.3_amd64.deb",
      "application/vnd.debian.binary-package","public, max-age=31536000, immutable"],
    ["releases/v1.2.3/windows/CFVigil-Setup-1.2.3-win-x64.exe",
      "application/octet-stream","public, max-age=31536000, immutable"],
    ["apt/dists/stable/InRelease","text/plain; charset=utf-8","no-store"],
    ["apt/dists/stable/main/binary-amd64/Packages.gz","application/gzip","no-store"],
    ["apt/cfvigil-archive-keyring.gpg","application/pgp-keys",
      "public, max-age=31536000, immutable"]
  ];
  for(const [path,contentType,cacheControl] of cases){
    const get=await worker.fetch(new Request(base+path),env);
    assert.equal(get.status,200,path);
    assert.equal(get.headers.get("content-type"),contentType,path);
    assert.equal(get.headers.get("cache-control"),cacheControl,path);
    assert.equal(get.headers.get("x-content-type-options"),"nosniff",path);
    assert.equal(get.headers.get("etag"),'"immutable"',path);
    assert.equal(await get.text(),"payload",path);
    const head=await worker.fetch(new Request(base+path,{method:"HEAD"}),env);
    assert.equal(head.status,200,path);
    assert.equal(head.headers.get("content-type"),contentType,path);
    assert.equal(await head.text(),"",path);
  }
  assert.equal(seen.length,cases.length*2);
});

test("Mode B: missing R2 objects remain 404 for both allowed methods",async()=>{
  const called=[];
  const env={ARCHIVE:{
    get:async key=>{called.push(["get",key]);return null;},
    head:async key=>{called.push(["head",key]);return null;}
  }};
  const url=base+"releases/v1.2.3/windows/CFVigil-Setup-1.2.3-win-x64.exe";
  assert.equal((await worker.fetch(new Request(url),env)).status,404);
  assert.equal((await worker.fetch(new Request(url,{method:"HEAD"}),env)).status,404);
  assert.deepEqual(called.map(x=>x[0]),["get","head"]);
});
