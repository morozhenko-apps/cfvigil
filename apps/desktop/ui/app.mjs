import { counters, filterResources, relativeTime } from "./status.mjs";
import { normalizeSnapshot } from "./snapshot.mjs";
const invoke = (command, args = {}) => window.__TAURI__.core.invoke(command, args);
const listen = (event, callback) => window.__TAURI__.event.listen(event, callback);
const byId = id => document.getElementById(id);
let resources = [];
let expanded = false;
let focusedKey = null;
let openMenuKey = null;
let syncAvailable = false;
let accountConfigured = false;

function clearMenu() { document.querySelectorAll(".resource-menu").forEach(node => node.remove()); openMenuKey = null; }

function render() {
  const totals = counters(resources);
  for (const [id, value] of Object.entries({healthy: totals.healthy, deploying: totals.deploying, issues: totals.failed})) {
    byId(id).textContent = syncAvailable ? String(value) : "—";
  }
  clearMenu();
  const list = byId("resource-list");
  list.replaceChildren();
  if (!resources.length) {
    const message = document.createElement("div");
    message.className = "empty";
    message.textContent = !accountConfigured ? "Connect Cloudflare in Settings" : syncAvailable ? "No resources found" : "Cloudflare data unavailable";
    list.append(message);
    return;
  }
  const filtered = filterResources(resources, byId("search").value);
  if (!filtered.length) {
    const empty = document.createElement("div");
    empty.className = "empty";
    empty.textContent = "No matching resources";
    list.append(empty);
  }
  for (const resource of filtered) {
    const row = document.createElement("div");
    row.className = "resource-row";
    row.dataset.key = resource.key;
    if (resource.key === focusedKey) row.classList.add("focused");
    const name = document.createElement("span");
    name.className = "resource-name";
    name.textContent = resource.name;
    name.title = resource.name;
    const time = document.createElement("span");
    time.className = "resource-time";
    time.textContent = relativeTime(resource.changed_at);
    const lamp = document.createElement("span");
    lamp.className = `lamp ${resource.status}`;
    lamp.title = resource.detail || resource.status;
    const button = document.createElement("button");
    button.className = "more";
    button.type = "button";
    button.textContent = "⋯";
    button.setAttribute("aria-label", "Resource options: " + resource.name);
    button.onclick = event => {
      event.stopPropagation();
      const wasOpen = openMenuKey === resource.key;
      clearMenu();
      if (wasOpen) return;
      const menu = document.createElement("div");
      menu.className = "resource-menu";
      const open = document.createElement("button");
      open.textContent = "Open in Cloudflare ↗";
      open.onclick = async () => { clearMenu(); try { await invoke("open_resource", { key: resource.key }); } catch { alertOnResources("Unable to open Cloudflare"); } };
      menu.append(open);
      menu.style.top = Math.max(3, Math.min(window.innerHeight - 45, row.getBoundingClientRect().bottom)) + "px";
      row.append(menu);
      openMenuKey = resource.key;
    };
    row.append(name, time, lamp, button);
    list.append(row);
  }
  if (focusedKey) list.querySelectorAll(".resource-row").forEach(row => {
    if (row.dataset.key === focusedKey) row.scrollIntoView({block:"nearest"});
  });
}
// An accessible status message, not a visible overlay over the five-cell card.
function announce(message) {
  byId("announcement").textContent = message;
}
function alertOnResources(message) {
  announce(message);
  const button = byId("toggle-list");
  button.title = message;
  if (!expanded) button.classList.add("blink");
}
async function toggleList() {
  const next = !expanded;
  try {
    await invoke("set_card_expanded", { expanded: next });
    expanded = next;
    byId("drawer").hidden = !expanded;
    byId("search-row").hidden = !expanded;
    byId("toggle-list").setAttribute("aria-pressed", String(expanded));
    if (expanded) {
      byId("toggle-list").classList.remove("blink");
      byId("toggle-list").title = "Toggle resource list";
      await invoke("acknowledge_incident");
      render();
    } else {
      focusedKey = null;
      byId("search").value = "";
      clearMenu();
    }
  } catch {
    alertOnResources("Unable to resize the card");
  }
}
byId("toggle-list").onclick = toggleList;
byId("hide").onclick = async () => { await invoke("hide_card"); };
byId("search").oninput = render;
document.addEventListener("click", event => { if (!event.target.closest(".resource-row")) clearMenu(); });
document.addEventListener("keydown", event => {
  if (event.key !== "Escape") return;
  if (expanded) void toggleList(); else void invoke("hide_card");
});
await listen("cfvigil-card-hidden", () => {
  expanded = false;
  focusedKey = null;
  byId("search").value = "";
  byId("drawer").hidden = true;
  byId("search-row").hidden = true;
  byId("toggle-list").setAttribute("aria-pressed","false");
  byId("toggle-list").classList.remove("blink");
  byId("toggle-list").title = "Toggle resource list";
  announce("");
  clearMenu();
});
function applySnapshot(payload) {
  const snapshot = normalizeSnapshot(payload);
  resources = snapshot.resources;
  syncAvailable = snapshot.ready;
  accountConfigured = snapshot.configured;
  if (syncAvailable && !byId("toggle-list").classList.contains("blink")) {
    byId("toggle-list").title = "Toggle resource list";
    announce("");
  }
  render();
}
await listen("cfvigil-resources", ({payload}) => applySnapshot(payload));
function presentIncident(payload) {
  if (!payload?.key || !payload?.message) return;
  focusedKey = payload.key;
  // The active text search must not hide the incident target.
  byId("search").value = "";
  alertOnResources(payload.message);
  if (expanded) render();
}
await listen("cfvigil-incident", ({payload}) => presentIncident(payload));
await listen("cfvigil-sync-error", ({payload}) => {
  announce(payload || "Cloudflare API unavailable");
  byId("toggle-list").title = payload || "Cloudflare API unavailable";
});
try {
  const snapshot = await invoke("get_snapshot");
  applySnapshot(snapshot);
  if (snapshot.pending_incident) presentIncident(snapshot.pending_incident);
} catch { alertOnResources("Unable to load resources"); }
