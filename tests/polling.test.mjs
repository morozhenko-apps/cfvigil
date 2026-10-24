import test from "node:test";
import assert from "node:assert/strict";
import {readFileSync} from "node:fs";
import {normalizeSnapshot} from "../apps/desktop/ui/snapshot.mjs";

test("missing and stale provider snapshots never claim verified resource health", () => {
  assert.deepEqual(normalizeSnapshot(null), {resources:[],ready:false,configured:false});
  const unknown = [{name:"worker",status:"unknown"}];
  assert.deepEqual(normalizeSnapshot({ready:false,configured:true,resources:unknown}), {
    resources:unknown,ready:false,configured:true,
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
});
