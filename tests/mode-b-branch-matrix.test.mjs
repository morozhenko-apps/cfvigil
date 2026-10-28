import test from "node:test";
import assert from "node:assert/strict";
import {
  classifyWorkerBuild, classifyPagesDeployment, counters, filterResources
} from "../apps/desktop/ui/status.mjs";
import {normalizeSnapshot} from "../apps/desktop/ui/snapshot.mjs";
import {filterHistory, historyTimestamp} from "../apps/desktop/ui/history-model.mjs";
import {nextResourceIndex} from "../apps/desktop/ui/resource-navigation.mjs";
import {createNativeDragGesture} from "../apps/desktop/ui/native-drag.mjs";
import {plural, relativeTime, incidentText} from "../apps/desktop/ui/localization.mjs";

test("Given every supported Worker build branch When classified Then status is never inferred from previous completion", () => {
  // Arrange: exact branches for queued/running/finished/unknown.
  const cases = [
    [null,"unknown"], [undefined,"unknown"], [false,"unknown"],
    [{}, "unknown"], [{status:"queued",build_outcome:"fail"},"deploying"],
    [{status:"initializing",build_outcome:"terminated"},"deploying"],
    [{status:"running",build_outcome:"success"},"deploying"],
    [{status:"stopped",build_outcome:"success"},"healthy"],
    [{status:"stopped",build_outcome:"fail"},"failed"],
    [{status:"stopped",build_outcome:"cancelled"},"failed"],
    [{status:"stopped",build_outcome:"terminated"},"failed"],
    [{status:"stopped",build_outcome:"skipped"},"unknown"],
    [{status:"stopped",build_outcome:null},"unknown"],
    [{status:"stopped",build_outcome:"unknown"},"unknown"],
    [{status:"deploying",build_outcome:"success"},"unknown"]
  ];
  // Act/Assert: a sign or state-order mutation breaks the expected table.
  for (const [input,expected] of cases) assert.equal(classifyWorkerBuild(input),expected,
    JSON.stringify(input));
});

test("Given every Pages stage branch When classified Then only non-skipped verified success is green", () => {
  const cases=[
    [null,"unknown"], [{}, "unknown"],
    [{is_skipped:true,latest_stage:{status:"success"}},"unknown"],
    [{is_skipped:false,latest_stage:{status:"success"}},"healthy"],
    [{latest_stage:{status:"active"}},"deploying"],
    [{latest_stage:{status:"idle"}},"deploying"],
    [{latest_stage:{status:"failure"}},"failed"],
    [{latest_stage:{status:"canceled"}},"unknown"],
    [{latest_stage:{status:"skipped"}},"unknown"],
    [{latest_stage:null},"unknown"]
  ];
  for(const [input,expected] of cases) assert.equal(classifyPagesDeployment(input),expected);
});

test("Given verified and unverified resource mixtures When counted Then totals preserve every category and do not mutate rows", () => {
  const resources=[
    {status:"healthy"}, {status:"healthy"}, {status:"deploying"},
    {status:"failed"}, {status:"unknown"}, {status:"upstream-unknown"},
    {status:"__proto__"}
  ];
  const copy=structuredClone(resources);
  assert.deepEqual(counters(resources),{healthy:2,deploying:1,failed:1,unknown:3});
  assert.deepEqual(resources,copy);
  assert.deepEqual(counters([]),{healthy:0,deploying:0,failed:0,unknown:0});
});

test("Given resources with mixed-case names When searched Then trim and case-fold are applied only to filtering", () => {
  const resources=[{name:"Growth-API"},{name:"PersonalLanding"},{name:"GROWTH-Worker"}];
  assert.deepEqual(filterResources(resources,"  growth "),[resources[0],resources[2]]);
  assert.deepEqual(filterResources(resources,""),resources);
  assert.deepEqual(filterResources(resources,"missing"),[]);
  assert.deepEqual(resources.map(x=>x.name),["Growth-API","PersonalLanding","GROWTH-Worker"]);
});

test("Given event history When filtered by category, key, name, and locale Then only matching immutable events remain", () => {
  const records=[
    {id:"a",name:"Alpha",key:"worker:alpha",category:"incident",
      message:"Workers runtime error spike detected"},
    {id:"b",name:"Beta",key:"zone:beta",category:"recovery",
      message:"WAF block spike cleared"},
    {id:"c",name:"Gamma",key:"pages:gamma",category:"update",
      message:"Latest Cloudflare deployment succeeded"}
  ];
  assert.deepEqual(filterHistory(records,"","all").map(x=>x.id),["a","b","c"]);
  assert.deepEqual(filterHistory(records,"", "incident").map(x=>x.id),["a"]);
  assert.deepEqual(filterHistory(records,"", "recovery").map(x=>x.id),["b"]);
  assert.deepEqual(filterHistory(records,"", "update").map(x=>x.id),["c"]);
  assert.deepEqual(filterHistory(records,"","invalid"),[]);
  assert.deepEqual(filterHistory(records," ZONE:BETA ").map(x=>x.id),["b"]);
  assert.deepEqual(filterHistory(records," Gamma ").map(x=>x.id),["c"]);
  assert.deepEqual(filterHistory(records,"Обнаружен рост ошибок","all","ru").map(x=>x.id),["a"]);
  assert.deepEqual(filterHistory(records,"Нет совпадений","all","ru"),[]);
  assert.equal(records[2].message,"Latest Cloudflare deployment succeeded");
});

test("Given resource keyboard navigation When keys and boundaries vary Then all focus branches remain deterministic", () => {
  const cases=[
    [0,0,"Home",-1],[0,5,"Home",0],[3,5,"Home",0],
    [0,5,"End",4],[3,5,"End",4],
    [-1,5,"ArrowDown",0],[4,5,"ArrowDown",0],[2,5,"ArrowDown",3],
    [-1,5,"ArrowUp",4],[0,5,"ArrowUp",4],[3,5,"ArrowUp",2],
    [2,5,"Escape",-1],[2,5,"Tab",-1],[2,5,"ArrowLeft",-1],
    [0,-1,"Home",-1],[0,1.2,"Home",-1],[0,NaN,"End",-1]
  ];
  for(const [current,count,key,expected] of cases) {
    assert.equal(nextResourceIndex(current,count,key),expected,
      `current=${current}, count=${count}, key=${key}`);
  }
});

test("Given malformed and partial native snapshots When normalized Then status cannot be fabricated", () => {
  assert.deepEqual(normalizeSnapshot(null),{
    resources:[],ready:false,pending:false,configured:false,provider_warnings:[]
  });
  assert.deepEqual(normalizeSnapshot({resources:"bad",ready:1,configured:"true",
    provider_warnings:[null,42]}),{
    resources:[],ready:false,pending:false,configured:false,provider_warnings:[]
  });
  const snapshot=normalizeSnapshot({
    resources:[{status:"unknown"}],ready:false,configured:true,
    provider_warnings:["one",null,"two","three","four","five","six","seven"]
  });
  assert.deepEqual(snapshot.provider_warnings,["one","two","three","four","five","six"]);
  assert.equal(snapshot.resources[0].status,"unknown");
  assert.equal(snapshot.ready,false);
  assert.equal(snapshot.pending,false,"missing pending is never an active request");
  assert.equal(normalizeSnapshot({configured:true,pending:"true"}).pending,false);
  assert.equal(normalizeSnapshot({configured:true,pending:true}).pending,true);
});

test("Given real timezone-offset timestamps When calculating relative age Then elapsed time is instant-based", () => {
  const now=Date.parse("2026-03-29T03:00:00+01:00");
  assert.equal(relativeTime("2026-03-29T01:00:00Z",now,"en"),"1h");
  assert.equal(relativeTime("2026-03-29T03:30:00+02:00",now,"en"),"30m");
  assert.equal(relativeTime("2026-03-29T04:00:00+02:00",now,"en"),"now");
  assert.equal(relativeTime("invalid",now,"en"),"—");
  assert.equal(historyTimestamp("invalid","pt"),"Hora desconhecida");
});

test("Given multiple CLDR quantity boundaries When rendering event counts Then Russian/Polish/Arabic keep valid plural forms", () => {
  for(const locale of ["ru","pl","ar"]) {
    for(const count of [0,1,2,3,4,5,11,21,22,101]) {
      const rendered=plural("history.eventCount",count,{shown:count,total:count},locale);
      assert.ok(rendered.length>0,`${locale} ${count}`);
      assert.ok(!rendered.includes("{shown}")&&!rendered.includes("{total}"),locale);
    }
  }
});

test("Given native drag session When duplicated or mismatched Then only matching end queues exactly one stop", async () => {
  const actions=[];
  let resolveStopped;
  const stopped=new Promise(resolve=>{resolveStopped=resolve;});
  const gesture=createNativeDragGesture({
    start:async()=>{actions.push("start");},
    stop:async()=>{actions.push("stop");resolveStopped();},
    onError:error=>{throw error;}
  });
  assert.equal(gesture.begin("pointer-1"),true);
  assert.equal(gesture.begin("pointer-2"),false);
  gesture.end("pointer-2");
  assert.equal(gesture.begin("pointer-3"),false);
  gesture.end("pointer-1");
  await stopped;
  assert.deepEqual(actions,["start","stop"]);
  assert.equal(gesture.begin("pointer-3"),true);
  gesture.end("pointer-2"); // Old or unrelated pointer cannot stop the new session.
  assert.equal(gesture.begin("pointer-4"),false);
});

test("Given failing native drag start When the failure is delivered Then the session releases without swallowing error", async () => {
  let attempts=0;
  let resolveReported;
  let resolveRecovered;
  const reported=new Promise(resolve=>{resolveReported=resolve;});
  const recovered=new Promise(resolve=>{resolveRecovered=resolve;});
  const gesture=createNativeDragGesture({
    start:()=>{
      if(++attempts===1)throw new Error("native drag denied");
      resolveRecovered();
    },
    stop:()=>{},
    onError:error=>resolveReported(error)
  });
  assert.equal(gesture.begin("first"),true);
  const error=await reported;
  assert.match(error.message,/native drag denied/);
  assert.equal(gesture.begin("second"),true);
  await recovered;
  assert.equal(attempts,2); // A fresh session successfully starts after failure.
});

test("Given old incident prefix and attacker-controlled name When rendering Then only known suffix changes", () => {
  const attacker="worker:<img src=x onerror=alert(1)>";
  const source=attacker+": Latest Cloudflare build/deployment failed";
  const result=incidentText(source,"ru");
  assert.ok(result.startsWith(attacker+": "));
  assert.match(result,/завершилась ошибкой$/);
  assert.equal(source,attacker+": Latest Cloudflare build/deployment failed");
});
