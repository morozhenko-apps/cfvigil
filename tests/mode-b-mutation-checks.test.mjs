import test from "node:test";
import assert from "node:assert/strict";
import {readFileSync} from "node:fs";
import {planSettingsApply} from "../apps/desktop/ui/settings-transaction.mjs";
import {classifyWorkerBuild,classifyPagesDeployment,counters,filterResources}
  from "../apps/desktop/ui/status.mjs";
import {normalizeSnapshot} from "../apps/desktop/ui/snapshot.mjs";

// Small, executable mutation checks of high-risk pure decision code.
// Mutations run only in memory. No source files, OS settings, CI fixtures
// or production interfaces are rewritten.
const source = file => readFileSync(new URL("../apps/desktop/ui/"+file,import.meta.url),"utf8");
const load = async text => import("data:text/javascript;base64,"+Buffer.from(text).toString("base64"));

const previous={
  config:{account_id:"a".repeat(32),refresh_seconds:180},
  language:"ru",autostart:false
};
const draft={
  accountId:"a".repeat(32),refreshSeconds:"180",token:"",
  language:"ru",autostart:false
};
function assertSettingsRules(plan) {
  assert.equal(plan(previous,draft).changed,false,"unchanged OK must not persist");
  assert.equal(plan(previous,{...draft,accountId:"b".repeat(32)}).changed,true,
    "account change must trigger update");
  assert.equal(plan(previous,{...draft,accountId:"  "+"a".repeat(32)+" "}).changed,false,
    "trimmed account identity must be stable");
  assert.equal(plan(previous,{...draft,refreshSeconds:"300"}).args.refreshSeconds,300,
    "UI interval strings must become numbers");
  assert.equal(plan(previous,{...draft,language:"pt"}).changed,true,
    "locale-only update must commit");
  assert.equal(plan(previous,{...draft,language:"pt"}).args.token,null,
    "locale-only update must not rotate credentials");
  assert.equal(plan(previous,{...draft,token:"  new-token "}).args.token,"new-token",
    "only explicit token replacement should reach keyring");
  assert.equal(plan(previous,{...draft,autostart:true}).args.autostart,true,
    "changed autostart must apply");
  assert.equal(plan(previous,{...draft}).args.autostart,null,
    "unchanged autostart must not be reapplied");
}
function assertStatusRules({classifyWorkerBuild:worker,classifyPagesDeployment:pages,
  counters:count,filterResources:filter}) {
  assert.equal(worker({status:"queued",build_outcome:"fail"}),"deploying",
    "queued build must override previous failure");
  assert.equal(worker({status:"stopped",build_outcome:"success"}),"healthy",
    "only completed successful build can be green");
  assert.equal(worker({status:"stopped",build_outcome:"fail"}),"failed");
  assert.equal(pages({is_skipped:true,latest_stage:{status:"success"}}),"unknown",
    "skipped success is unverified");
  assert.equal(pages({latest_stage:{status:"success"}}),"healthy");
  assert.equal(pages({latest_stage:{status:"failure"}}),"failed");
  assert.deepEqual(count([{status:"not-real"}]),
    {healthy:0,deploying:0,failed:0,unknown:1},
    "unexpected state cannot mutate the counters");
  assert.deepEqual(filter([{name:"Alpha"},{name:"BETA"}],"  beta "),
    [{name:"BETA"}],"search must trim and case-fold");
}
function assertSnapshotRules(normalize) {
  assert.equal(normalize({ready:false}).ready,false);
  assert.equal(normalize({ready:true}).ready,true);
  assert.deepEqual(normalize({resources:[{status:"unknown"}]}).resources,
    [{status:"unknown"}]);
  assert.deepEqual(normalize({resources:"invalid"}).resources,[]);
  assert.deepEqual(normalize({provider_warnings:["warning",null,123,"safe"]}).provider_warnings,
    ["warning","safe"],"untrusted warning types are rejected");
}

test("Mode B Stage 10: live Settings oracle kills targeted state and credential mutations",async()=>{
  assertSettingsRules(planSettingsApply);
  const original=source("settings-transaction.mjs");
  const mutants=[
    ["invert account comparison","accountId !== prev.account_id","accountId === prev.account_id"],
    ["invert interval comparison","refreshSeconds !== prev.refresh_seconds","refreshSeconds === prev.refresh_seconds"],
    ["invert locale comparison","language !== oldLanguage","language === oldLanguage"],
    ["invert token replacement","replacement !== \"\"","replacement === \"\""],
    ["reapply unchanged autostart","draft.autostart === previous.autostart",
      "draft.autostart !== previous.autostart"],
    ["remove account trim","String(draft.accountId ?? \"\").trim()",
      "String(draft.accountId ?? \"\")"],
    ["preserve string interval","Number(draft.refreshSeconds)","String(draft.refreshSeconds)"],
    ["drop token replacement","token: replacement || null","token: null"],
    ["invert autostart boolean","Boolean(draft.autostart)","Boolean(!draft.autostart)"]
  ];
  for(const [name,needle,replacement] of mutants) {
    assert.equal(original.split(needle).length,2,`mutation anchor: ${name}`);
    const mutated=await load(original.replace(needle,replacement));
    assert.throws(()=>assertSettingsRules(mutated.planSettingsApply),
      {name:"AssertionError"},`mutant survived: ${name}`);
  }
});

test("Mode B Stage 10: Worker/Pages status oracle kills wrong success and latest-attempt mutations",async()=>{
  assertStatusRules({classifyWorkerBuild,classifyPagesDeployment,counters,filterResources});
  const original=source("status.mjs").replace(
    'export {relativeTime} from "./localization.mjs";',"");
  const mutants=[
    ["queued no longer deploys",'"queued", "initializing", "running"',
      '"initializing", "running"'],
    ["completed success becomes failure",'build.build_outcome === "success"',
      'build.build_outcome === "fail"'],
    ["skipped pages become healthy","deployment.is_skipped === true",
      "deployment.is_skipped === false"],
    ["pages success misclassified",'status === "success"','status === "failure"'],
    ["stray state increments as healthy","result.unknown += 1","result.healthy += 1"],
    ["resource search no longer trims","query.trim().toLocaleLowerCase()",
      "query.toLocaleLowerCase()"],
  ];
  for(const [name,needle,replacement] of mutants) {
    assert.equal(original.split(needle).length,2,`mutation anchor: ${name}`);
    const mutated=await load(original.replace(needle,replacement));
    assert.throws(()=>assertStatusRules(mutated),
      {name:"AssertionError"},`mutant survived: ${name}`);
  }
});

test("Mode B Stage 10: partial provider snapshots never mutate into verified or unsanitized data",async()=>{
  assertSnapshotRules(normalizeSnapshot);
  const original=source("snapshot.mjs");
  const mutants=[
    ["invert native readiness","payload?.ready === true","payload?.ready !== true"],
    ["trust nonarray resource payload","Array.isArray(payload?.resources)",
      "Array.isArray(payload?.resources) === false"],
    ["drop warning type validation",'filter(value => typeof value === "string")',
      'filter(value => true)'],
  ];
  for(const [name,needle,replacement] of mutants) {
    assert.equal(original.split(needle).length,2,`mutation anchor: ${name}`);
    const mutated=await load(original.replace(needle,replacement));
    assert.throws(()=>assertSnapshotRules(mutated.normalizeSnapshot),
      {name:"AssertionError"},`mutant survived: ${name}`);
  }
});
