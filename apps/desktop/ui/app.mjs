import { counters, filterResources, relativeTime } from "./status.mjs";
const invoke = (command, args = {}) => window.__TAURI__.core.invoke(command, args);
const listen = (event, callback) => window.__TAURI__.event.listen(event, callback);
const byId = id => document.getElementById(id);
let resources = [];
let expanded = false;
let focusedKey = null;
let openMenuKey = null;
let syncAvailable = false;

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
    message.textContent = syncAvailable ? "No resources found" : "Connect Cloudflare in Settings";
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
      open.onclick = async () => { clearMenu(); try { await invoke("open_resource", { key: resource.key }); } catch { showIncident("Unable to open Cloudflare"); } };
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
function showIncident(message) {
  const node = byId("incident");
  node.textContent = message;
  node.hidden = false;
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
      byId("incident").hidden = true;
      await invoke("acknowledge_incident");
      render();
    } else {
      focusedKey = null;
      byId("search").value = "";
      clearMenu();
    }
  } catch {
    showIncident("Unable to resize the card");
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
  byId("incident").hidden = true;
  byId("toggle-list").classList.remove("blink");
  clearMenu();
});
await listen("cfvigil-resources", ({payload}) => {
  resources = payload;
  syncAvailable = true;
  render();
});
function presentIncident(payload) {
  if (!payload?.key || !payload?.message) return;
  focusedKey = payload.key;
  // The active text search must not hide the incident target.
  byId("search").value = "";
  showIncident(payload.message);
  if (!expanded) byId("toggle-list").classList.add("blink");
  else render();
}
await listen("cfvigil-incident", ({payload}) => presentIncident(payload));
await listen("cfvigil-sync-error", ({payload}) => { showIncident(payload || "Cloudflare API unavailable"); });
try {
  const snapshot = await invoke("get_snapshot");
  resources = snapshot.resources;
  syncAvailable = snapshot.ready;
  render();
  if (snapshot.pending_incident) presentIncident(snapshot.pending_incident);
} catch { showIncident("Unable to load resources"); }
