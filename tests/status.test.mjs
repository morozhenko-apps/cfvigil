import test from "node:test";
import assert from "node:assert/strict";
import {classifyWorkerBuild, classifyPagesDeployment, counters, filterResources, relativeTime} from "../apps/desktop/ui/status.mjs";

test("failed previous Workers build does not override newer running build", () => {
  const previous = {status:"stopped",build_outcome:"fail"};
  const latest = {status:"running",build_outcome:null};
  assert.equal(classifyWorkerBuild(previous),"failed");
  assert.equal(classifyWorkerBuild(latest),"deploying");
});
test("unknown Workers outcomes never become green", () => {
  for (const outcome of ["skipped",undefined,"other"]) assert.equal(classifyWorkerBuild({status:"stopped",build_outcome:outcome}),"unknown");
});
test("Workers failed / successful outcomes", () => {
  assert.equal(classifyWorkerBuild({status:"stopped",build_outcome:"fail"}),"failed");
  assert.equal(classifyWorkerBuild({status:"stopped",build_outcome:"success"}),"healthy");
  assert.equal(classifyWorkerBuild({status:"initializing",build_outcome:"fail"}),"deploying");
  assert.equal(classifyWorkerBuild(null),"unknown");
});
test("latest Pages attempt controls indicator", () => {
  assert.equal(classifyPagesDeployment({latest_stage:{status:"active"}}),"deploying");
  assert.equal(classifyPagesDeployment({latest_stage:{status:"failure"}}),"failed");
  assert.equal(classifyPagesDeployment({latest_stage:{status:"success"}}),"healthy");
  assert.equal(classifyPagesDeployment({latest_stage:{status:"skipped"}}),"unknown");
  assert.equal(classifyPagesDeployment({latest_stage:{status:"canceled"}}),"unknown");
  assert.equal(classifyPagesDeployment({is_skipped:true,latest_stage:{status:"success"}}),"unknown");
  assert.equal(classifyPagesDeployment({is_skipped:false,latest_stage:{status:"success"}}),"healthy");
  assert.equal(classifyPagesDeployment(null),"unknown");
});
test("aggregate counts never mix unknown with healthy", () => {
  assert.deepEqual(counters([{status:"healthy"},{status:"deploying"},{status:"failed"},{status:"unknown"},{status:"unhandled"}]), {healthy:1,deploying:1,failed:1,unknown:2});
});
test("search filters case insensitively and relative time handles missing timestamps", () => {
  assert.deepEqual(filterResources([{name:"Growth-API"},{name:"Marketing"}]," growth ").map(x=>x.name),["Growth-API"]);
  assert.equal(relativeTime(null),"—");
  assert.equal(relativeTime("2026-10-22T10:00:00Z", Date.parse("2026-10-22T10:02:00Z")),"2m");
});
