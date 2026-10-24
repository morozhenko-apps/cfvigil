import test from "node:test";
import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { filterHistory, historyTimestamp } from "../apps/desktop/ui/history-model.mjs";

const events = [
  {id:"old",at:"2026-10-23T11:00:00Z",name:"worker",key:"worker:alpha",
    message:"Latest Cloudflare build/deployment failed",category:"incident",url:"https://dash.cloudflare.com/a/old"},
  {id:"new",at:"2026-10-23T12:00:00Z",name:"worker",key:"worker:alpha",
    message:"Latest Cloudflare deployment succeeded",category:"update",url:"https://dash.cloudflare.com/a/new"},
  {id:"waf",at:"2026-10-23T12:30:00Z",name:"example.net",key:"zone:some",
    message:"WAF block spike cleared",category:"recovery",url:"https://dash.cloudflare.com/a/waf"}
];

test("history filters are independent of the selected historical event", () => {
  assert.deepEqual(filterHistory(events,"worker","incident").map(x=>x.id),["old"]);
  assert.deepEqual(filterHistory(events,"EXAMPLE","recovery").map(x=>x.id),["waf"]);
  assert.deepEqual(filterHistory(events,"  alpha  ").map(x=>x.id),["old","new"]);
  assert.deepEqual(filterHistory(events,"","update").map(x=>x.id),["new"]);
  assert.equal(filterHistory(events,"","unsupported").length,0);
});

test("history timestamp handles missing/invalid data", () => {
  assert.equal(historyTimestamp(null),"Unknown time");
  assert.equal(historyTimestamp("nonsense"),"Unknown time");
  assert.notEqual(historyTimestamp("2026-10-23T12:00:00Z"),"Unknown time");
});

test("historical link command uses event ID and never current resource key", () => {
  const ui = readFileSync(new URL("../apps/desktop/ui/history.mjs",import.meta.url),"utf8");
  const native = readFileSync(new URL("../apps/desktop/src-tauri/src/main.rs",import.meta.url),"utf8");
  assert.match(ui,/invoke\("open_history_event", \{id: event\.id\}\)/);
  assert.doesNotMatch(ui,/open_history_resource|open_resource.*key/);
  assert.match(native,/fn open_history_event\(/);
  assert.match(native,/history_url\(&history, &id\)/);
  assert.doesNotMatch(native,/fn open_history_resource\(/);
});

test("history matches translated incident text without changing stored English events", () => {
  assert.deepEqual(filterHistory(events, "ошибкой", "incident", "ru").map(x => x.id), ["old"]);
  assert.deepEqual(filterHistory(events, "falhou", "incident", "pt").map(x => x.id), ["old"]);
});
