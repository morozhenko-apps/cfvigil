import { counters, filterResources, relativeTime } from "./status.mjs";
import { normalizeSnapshot } from "./snapshot.mjs";
import { nextResourceIndex } from "./resource-navigation.mjs";
import {t, resourceDetail, incidentText, verboseRelativeTime, currentLocale, intlLocale} from "./localization.mjs";
import {initializeLanguage} from "./runtime-language.mjs";
const invoke = (command, args = {}) => window.__TAURI__.core.invoke(command, args);
const listen = (event, callback) => window.__TAURI__.event.listen(event, callback);
const byId = id => document.getElementById(id);
const statusLabelKey = Object.freeze({
  "healthy": "card.healthy",
  "deploying": "card.deploying",
  "failed": "card.issues",
  "unknown": "card.statusUnknown"
});
let resources = [];
let expanded = false;
let focusedKey = null;
let openMenuKey = null;
let openMenuTrigger = null;
let syncAvailable = false;
let accountConfigured = false;
let storageWarning = false;
let resizing = false;

function clearMenu(restoreFocus = false) {
  const trigger = openMenuTrigger;
  document.querySelectorAll(".resource-menu").forEach(node => node.remove());
  if (trigger?.isConnected) trigger.setAttribute("aria-expanded", "false");
  openMenuKey = null;
  openMenuTrigger = null;
  if (restoreFocus && trigger?.isConnected) trigger.focus();
}

function moreButtons() {
  return Array.from(byId("resource-list").querySelectorAll(".resource-row > .more"));
}

function focusResource(key) {
  const button = moreButtons().find(node => node.closest(".resource-row")?.dataset.key === key);
  if (!button) return false;
  button.focus({preventScroll: true});
  button.scrollIntoView({block: "nearest"});
  return true;
}

function openResourceMenu(resource, trigger) {
  if (openMenuKey === resource.key) {
    clearMenu(true);
    return;
  }
  clearMenu();
  const card = byId("card");
  const menu = document.createElement("div");
  menu.id = "resource-action-menu";
  menu.className = "resource-menu";
  menu.setAttribute("role", "menu");
  menu.setAttribute("aria-label", t("card.menuLabel", {name: resource.name}));
  const open = document.createElement("button");
  open.type = "button";
  open.setAttribute("role", "menuitem");
  open.textContent = t("card.openCloudflare");
  open.onclick = async () => {
    clearMenu(true);
    try {
      await invoke("open_resource", {key: resource.key});
    } catch {
      alertOnResources(t("card.openError"));
    }
  };
  menu.append(open);
  card.append(menu);
  const rect = card.getBoundingClientRect();
  const triggerRect = trigger.getBoundingClientRect();
  // Place outside the scrollable row list, but keep it inside the card.
  menu.style.left = Math.max(4, Math.min(triggerRect.right - rect.left - menu.offsetWidth,
    rect.width - menu.offsetWidth - 4)) + "px";
  const below = triggerRect.bottom - rect.top;
  menu.style.top = (below + menu.offsetHeight + 4 <= rect.height - 4
    ? below + 2
    : Math.max(4, triggerRect.top - rect.top - menu.offsetHeight - 2)) + "px";
  openMenuKey = resource.key;
  openMenuTrigger = trigger;
  trigger.setAttribute("aria-expanded", "true");
  open.focus();
}

function render() {
  const totals = counters(resources);
  for (const [id, value] of Object.entries({healthy: totals.healthy, deploying: totals.deploying, issues: totals.failed})) {
    byId(id).textContent = syncAvailable ? String(value) : "—";
  }
  // Replacing rows on polling must not strand keyboard focus on a detached
  // button. Preserve the selected action by stable resource key.
  const active = document.activeElement;
  const focusKey = active?.closest(".resource-row")?.dataset.key
    ?? (active?.closest(".resource-menu") ? openMenuKey : null);
  clearMenu();
  const list = byId("resource-list");
  list.replaceChildren();
  if (!resources.length) {
    const message = document.createElement("div");
    message.className = "empty";
    message.textContent = !accountConfigured ? t("card.noAccount") : syncAvailable ? t("card.noResources") : t("card.noData");
    list.append(message);
    return;
  }
  const filtered = filterResources(resources, byId("search").value)
    .sort((a,b) => new Intl.Collator(intlLocale(currentLocale)).compare(a.name,b.name));
  if (!filtered.length) {
    const empty = document.createElement("div");
    empty.className = "empty";
    empty.textContent = t("card.noMatches");
    list.append(empty);
  }
  for (const resource of filtered) {
    const row = document.createElement("div");
    row.className = "resource-row";
    row.setAttribute("role", "listitem");
    row.dataset.key = resource.key;
    if (resource.key === focusedKey) row.classList.add("focused");
    const name = document.createElement("bdi");
    name.dir = "auto";
    name.className = "resource-name";
    name.textContent = resource.name;
    name.title = resource.name;
    const time = document.createElement("span");
    time.className = "resource-time";
    time.textContent = relativeTime(resource.changed_at);
    time.title = verboseRelativeTime(resource.changed_at);
    const lamp = document.createElement("span");
    lamp.className = `lamp ${resource.status}`;
    lamp.title = resourceDetail(resource.detail || resource.status);
    lamp.setAttribute("role", "img");
    const localizedStatus = t(statusLabelKey[resource.status] ?? "card.statusUnknown");
    lamp.setAttribute("aria-label", localizedStatus + ": " + resourceDetail(resource.detail || resource.name));
    const button = document.createElement("button");
    button.className = "more";
    button.type = "button";
    button.textContent = "⋯";
    button.setAttribute("aria-label", t("card.optionsLabel", {name: resource.name}));
    button.setAttribute("aria-haspopup", "menu");
    button.setAttribute("aria-controls", "resource-action-menu");
    button.setAttribute("aria-expanded", "false");
    button.onclick = event => {
      event.stopPropagation();
      openResourceMenu(resource, button);
    };
    row.append(name, time, lamp, button);
    list.append(row);
  }
  if (focusedKey) list.querySelectorAll(".resource-row").forEach(row => {
    if (row.dataset.key === focusedKey) row.scrollIntoView({block:"nearest"});
  });
  if (focusKey) focusResource(focusKey);
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
  if (resizing) return;
  resizing = true;
  const next = !expanded;
  try {
    await invoke("set_card_expanded", { expanded: next });
    expanded = next;
    byId("drawer").hidden = !expanded;
    byId("search-row").hidden = !expanded;
    byId("toggle-list").setAttribute("aria-pressed", String(expanded));
    byId("toggle-list").setAttribute("aria-expanded", String(expanded));
    if (expanded) {
      byId("toggle-list").classList.remove("blink");
      byId("toggle-list").title = t("card.toggleTitle");
      await invoke("acknowledge_incident");
      render();
      if (!focusedKey || !focusResource(focusedKey)) byId("search").focus();
    } else {
      focusedKey = null;
      byId("search").value = "";
      clearMenu();
      byId("toggle-list").focus();
    }
  } catch {
    alertOnResources(t("card.resizeError"));
  } finally {
    resizing = false;
  }
}
byId("toggle-list").onclick = toggleList;
byId("hide").onclick = async () => { await invoke("hide_card"); };
byId("search").oninput = render;
byId("search").addEventListener("keydown", event => {
  if (event.key !== "ArrowDown") return;
  const first = moreButtons()[0];
  if (first) {
    event.preventDefault();
    first.focus();
  }
});
byId("resource-list").addEventListener("keydown", event => {
  if (event.altKey || event.ctrlKey || event.metaKey) return;
  if (event.target.closest(".resource-menu")) return;
  const buttons = moreButtons();
  const next = nextResourceIndex(buttons.indexOf(document.activeElement),
    buttons.length, event.key);
  if (next < 0) return;
  event.preventDefault();
  buttons[next].focus();
});
document.addEventListener("click", event => {
  if (!event.target.closest(".resource-menu, .more")) clearMenu();
});
document.addEventListener("keydown", event => {
  if (event.key !== "Escape") return;
  if (openMenuKey) {
    event.preventDefault();
    clearMenu(true);
    return;
  }
  if (expanded) {
    event.preventDefault();
    void toggleList();
  } else {
    void invoke("hide_card");
  }
});
await initializeLanguage({invoke,listen,doc:document,onChange:render});
await listen("cfvigil-card-hidden", () => {
  expanded = false;
  focusedKey = null;
  byId("search").value = "";
  byId("drawer").hidden = true;
  byId("search-row").hidden = true;
  byId("toggle-list").setAttribute("aria-pressed","false");
  byId("toggle-list").setAttribute("aria-expanded","false");
  byId("toggle-list").classList.remove("blink");
  byId("toggle-list").title = t("card.toggleTitle");
  announce("");
  clearMenu();
});
function applySnapshot(payload) {
  const snapshot = normalizeSnapshot(payload);
  resources = snapshot.resources;
  syncAvailable = snapshot.ready;
  accountConfigured = snapshot.configured;
  storageWarning = payload?.storage_warning === true;
  if (!byId("toggle-list").classList.contains("blink")) {
    if (storageWarning) {
      const warning = t("card.storageWarning");
      byId("toggle-list").title = warning;
      announce(warning);
    } else if (syncAvailable) {
      byId("toggle-list").title = t("card.toggleTitle");
      announce("");
    }
  }
  render();
}
await listen("cfvigil-resources", ({payload}) => applySnapshot(payload));
function presentIncident(payload) {
  if (!payload?.key || !payload?.message) return;
  focusedKey = payload.key;
  // The active text search must not hide the incident target.
  byId("search").value = "";
  alertOnResources(incidentText(payload.message));
  if (expanded) {
    render();
    focusResource(focusedKey);
  }
}
await listen("cfvigil-incident", ({payload}) => presentIncident(payload));
await listen("cfvigil-sync-error", ({payload}) => {
  announce(payload || t("card.apiError"));
  byId("toggle-list").title = payload || t("card.apiError");
});
try {
  const snapshot = await invoke("get_snapshot");
  applySnapshot(snapshot);
  if (snapshot.pending_incident) presentIncident(snapshot.pending_incident);
} catch { alertOnResources(t("card.loadError")); }
