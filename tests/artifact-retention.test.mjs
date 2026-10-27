import test from "node:test";
import assert from "node:assert/strict";
import {readFileSync} from "node:fs";
import {validateR2Receipt,deletePublishedArtifacts} from "../scripts/cleanup-published-artifacts.mjs";

const BASE = "https://download.morozhenko.com/dist/CFVigil/releases/v0.1.0/";
const HASH = "a".repeat(64);
const COMMIT = "b".repeat(40);
// Mirror the exact receipt file lists emitted by r2_release.py.
const filesFor = (name, ext) => {
  if (name === "cfvigil-release-linux") return [
    "linux/cfvigil_0.1.0_amd64.deb",
    "linux/cfvigil_0.1.0_amd64.deb.sha256",
    "linux/CFVigil-0.1.0-linux-x64.AppImage",
    "linux/CFVigil-0.1.0-linux-x64.AppImage.sha256"
  ];
  if (name === "cfvigil-release-windows") return [
    "windows/CFVigil-Setup-0.1.0-win-x64.exe",
    "windows/CFVigil-Setup-0.1.0-win-x64.exe.sha256"
  ];
  return [name + ext];
};
const record = (name, run_id, ext) => ({
  name,run_id,files:filesFor(name,ext).map(relative => ({
    url:BASE + relative,sha256:HASH,readback_sha256:HASH,
    size:12345,readback_size:12345,readback_status:200
  }))
});
const receipt = () => ({
  schema:1,product:"CFVigil",version:"0.1.0",commit:COMMIT,
  base_url:"https://download.morozhenko.com/dist/CFVigil/",
  artifacts:[
    record("cfvigil-release-linux",51,".deb"),
    record("cfvigil-release-windows",52,".exe")
  ]
});

test("cleanup requires verified public R2 receipt before any network call", async () => {
  validateR2Receipt(receipt());
  for (const invalid of [
    {...receipt(),artifacts:[]},
    {...receipt(),base_url:"https://download.morozhenko.com/dist/RepoTicker/"},
    {...receipt(),commit:"bad"},
    {...receipt(),artifacts:[receipt().artifacts[0],receipt().artifacts[0]]},
    {...receipt(),artifacts:[receipt().artifacts[0],record("repoticker-release",52,".exe")]},
    {...receipt(),artifacts:[receipt().artifacts[0],{...receipt().artifacts[1],
      files:[{...receipt().artifacts[1].files[0],readback_sha256:"f".repeat(64)}]}]}
  ]) {
    let called=false;
    await assert.rejects(deletePublishedArtifacts(invalid,"token",()=>{
      called=true;throw Error("network");
    }));
    assert.equal(called,false);
  }
});

test("cleanup deletes only matching artifacts and independently confirms removal", async () => {
  const items = new Map([
    [51,[{id:101,name:"cfvigil-release-linux",workflow_run:{id:51,head_sha:COMMIT}},
      {id:111,name:"unrelated-build",workflow_run:{id:51,head_sha:COMMIT}}]],
    [52,[{id:102,name:"cfvigil-release-windows",workflow_run:{id:52,head_sha:COMMIT}}]]
  ]);
  const deleted=[];
  const request=async (url,options={})=>{
    const list=url.match(/\/actions\/runs\/(\d+)\/artifacts\?/);
    if(list)return {status:200,json:async()=>({artifacts:items.get(Number(list[1]))})};
    const del=url.match(/\/actions\/artifacts\/(\d+)$/);
    assert.equal(options.method,"DELETE");
    assert.ok(del);
    const id=Number(del[1]);
    deleted.push(id);
    for(const [run,arr] of items) items.set(run,arr.filter(a=>a.id!==id));
    return {status:204};
  };
  assert.equal(await deletePublishedArtifacts(receipt(),"token",request),2);
  assert.deepEqual(deleted,[101,102]);
  assert.equal(items.get(51).length,1,"unrelated artifacts untouched");
  assert.equal(await deletePublishedArtifacts(receipt(),"token",request),0);
});

test("foreign commit SHA refuses deletion", async () => {
  let deleted=false;
  const request=async (url,options={})=>{
    if(options.method==="DELETE")deleted=true;
    return {status:200,json:async()=>({artifacts:[{
      id:101,name:"cfvigil-release-linux",
      workflow_run:{id:51,head_sha:"f".repeat(40)}
    }]})};
  };
  await assert.rejects(deletePublishedArtifacts(receipt(),"token",request),/different build/);
  assert.equal(deleted,false);
});

test("candidates use minimum GitHub artifact retention and no release publishing", () => {
  const read=p=>readFileSync(new URL("../"+p,import.meta.url),"utf8");
  for(const path of [".github/workflows/package-candidate.yml",
    ".github/workflows/windows-candidate.yml"]) {
    const workflow=read(path).replace(/\r\n/g,"\n");
    assert.match(workflow,/retention-days: 1/);
    assert.doesNotMatch(workflow,/retention-days:\s*[2-9]/);
    assert.doesNotMatch(workflow,/gh release create|contents:\s*write/);
  }
  assert.doesNotMatch(read(".github/workflows/package-candidate.yml"),/cfvigil-cargo-lock-for-review/);
});


test("Mode B: missing or malformed publication receipts never trigger GitHub DELETE",async()=>{
  const invalidFiles=[
    {...receipt(),artifacts:[receipt().artifacts[0],{
      ...receipt().artifacts[1],files:[]
    }]},
    {...receipt(),artifacts:[receipt().artifacts[0],{
      ...receipt().artifacts[1],files:[{
        ...receipt().artifacts[1].files[0],readback_status:404
      }]
    }]},
    {...receipt(),artifacts:[receipt().artifacts[0],{
      ...receipt().artifacts[1],files:[{
        ...receipt().artifacts[1].files[0],readback_size:12344
      }]
    }]},
    {...receipt(),artifacts:[receipt().artifacts[0],{
      ...receipt().artifacts[1],files:[{
        ...receipt().artifacts[1].files[0],
        url:"https://evil.example/release.exe"
      }]
    }]},
    {...receipt(),artifacts:[receipt().artifacts[0],{
      ...receipt().artifacts[1],files:[{
        ...receipt().artifacts[1].files[0],sha256:"invalid"
      }]
    }]},
    {...receipt(),artifacts:[receipt().artifacts[0],{
      ...receipt().artifacts[1],run_id:0
    }]},
  ];
  for(const invalid of invalidFiles) {
    let calls=0;
    await assert.rejects(
      deletePublishedArtifacts(invalid,"test-only-token",()=>{calls++;throw Error("network forbidden");})
    );
    assert.equal(calls,0,"no network call before receipt validation");
  }
});

test("Mode B: GitHub artifact listing errors and ambiguous matches block cleanup",async()=>{
  const scenarios=[
    {status:403,data:{artifacts:[]},message:/Could not list/},
    {status:200,data:{artifacts:null},message:/Malformed/},
    {status:200,data:{artifacts:Array.from({length:101},(_,id)=>({id}))},message:/Malformed/},
    {status:200,data:{artifacts:[
      {id:11,name:"cfvigil-release-linux",workflow_run:{id:51,head_sha:COMMIT}},
      {id:12,name:"cfvigil-release-linux",workflow_run:{id:51,head_sha:COMMIT}}
    ]},message:/Ambiguous/},
  ];
  for(const scenario of scenarios){
    let deleted=false;
    const request=async (_url,options={})=>{
      if(options.method==="DELETE")deleted=true;
      return {status:scenario.status,json:async()=>scenario.data};
    };
    await assert.rejects(
      deletePublishedArtifacts(receipt(),"test-only-token",request),scenario.message
    );
    assert.equal(deleted,false,"never delete after unverified GitHub listing");
  }
});

test("Mode B: failed delete and failed post-delete confirmation cannot report completion",async()=>{
  const artifact={id:101,name:"cfvigil-release-linux",
    workflow_run:{id:51,head_sha:COMMIT}};
  for(const failure of ["delete-fails","still-listed"]) {
    let calls=0;
    const request=async(url,options={})=>{
      calls++;
      if(options.method==="DELETE")
        return {status:failure==="delete-fails"?503:204};
      if(url.includes("/runs/51/"))
        return {status:200,json:async()=>({artifacts:[artifact]})};
      return {status:200,json:async()=>({artifacts:[]})};
    };
    await assert.rejects(
      deletePublishedArtifacts(receipt(),"test-only-token",request),
      failure==="delete-fails"?/cleanup failed/:/remains/
    );
    assert.ok(calls>=2);
  }
});

test("Mode B: cleanup with no credentials or missing archived runs must fail closed or be idempotent",async()=>{
  let calls=0;
  await assert.rejects(deletePublishedArtifacts(receipt(),"",()=>{calls++;}),/Missing Actions-write/);
  assert.equal(calls,0);
  const request=async()=>({status:200,json:async()=>({artifacts:[]})});
  assert.equal(await deletePublishedArtifacts(receipt(),"test-only-token",request),0);
});
