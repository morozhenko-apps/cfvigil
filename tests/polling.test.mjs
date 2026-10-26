import test from "node:test";
import assert from "node:assert/strict";
import {readFileSync} from "node:fs";
import {normalizeSnapshot} from "../apps/desktop/ui/snapshot.mjs";

test("missing and stale provider snapshots never claim verified resource health", () => {
  assert.deepEqual(normalizeSnapshot(null), {resources:[],ready:false,configured:false,provider_warnings:[]});
  const unknown = [{name:"worker",status:"unknown"}];
  assert.deepEqual(normalizeSnapshot({ready:false,configured:true,resources:unknown}), {
    resources:unknown,ready:false,configured:true,provider_warnings:[],
  });
  assert.equal(normalizeSnapshot({ready:"true",configured:true,resources:unknown}).ready,false);
  assert.deepEqual(normalizeSnapshot({ready:true,resources:{status:"healthy"}}).resources,[]);
});

test("one scheduler handles both timer and settings wake-ups", () => {
  const main = readFileSync(new URL("../apps/desktop/src-tauri/src/main.rs",import.meta.url),"utf8");
  assert.match(main,/poll_refresh: tokio::sync::watch::Sender<u64>/);
  assert.match(main,/poll_refresh\.send_modify\(/);
  assert.match(main,/tokio::select!/);
  assert.match(main,/poll_delay\(interval, failures\)/);
  assert.doesNotMatch(main,/spawn\(async move \{ refresh_once\(/);
  assert.match(main,/report_sync_failure\(&app, &state, msg::DATA_CREDENTIALS_UNAVAILABLE\)/);
  assert.match(main,/publish_snapshot\(&app, &state\)/);
  assert.ok(main.includes("let catalog_available = discovery.workers_complete || discovery.pages_complete"));
  assert.ok(main.includes("catalog_available && !budget.exhausted()"));
  assert.ok(!main.includes("core_complete && !budget.exhausted()"));
});

test("partial Cloudflare API errors survive snapshot normalization without HTML or secrets", () => {
  const warnings = ["Pages: Cloudflare HTTP 403 (requires Pages Read)"];
  const snapshot = normalizeSnapshot({
    ready:false,configured:true,
    resources:[{key:"worker:one",status:"healthy"}],
    provider_warnings: warnings
  });
  assert.deepEqual(snapshot.provider_warnings, warnings);
  assert.equal(snapshot.resources[0].status, "healthy");
  assert.deepEqual(normalizeSnapshot({provider_warnings:[{}, null, 17]}).provider_warnings, []);
  const card = readFileSync(new URL("../apps/desktop/ui/app.mjs",import.meta.url),"utf8");
  assert.ok(card.includes('warning.replaceChildren()'));
  assert.match(card, /entry\.textContent = diagnosticText\(item\)/);
  assert.ok(card.includes('resources.some(item =>'));
  assert.ok(!card.includes('closest("button,input,select,a,.resource-row'));
});
