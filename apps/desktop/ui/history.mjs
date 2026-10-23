import { filterHistory, historyTimestamp } from "./history-model.mjs";

const invoke = (name, args={}) => window.__TAURI__.core.invoke(name, args);
const listen = (name, callback) => window.__TAURI__.event.listen(name, callback);
const list = document.getElementById("history");
const search = document.getElementById("history-search");
const category = document.getElementById("history-category");
const summary = document.getElementById("history-summary");
let events = [];

function render() {
  const filtered = filterHistory(events, search.value, category.value);
  summary.textContent = `${filtered.length} of ${events.length} events`;
  const output = document.createDocumentFragment();
  if (!filtered.length) {
    const empty = document.createElement("p");
    empty.textContent = events.length ? "No matching events." : "No events yet.";
    output.append(empty);
  }
  for (const event of filtered) {
    const row = document.createElement("div");
    row.className = "history-row";
    row.setAttribute("role", "listitem");
    const content = document.createElement("div");
    content.className = "history-details";
    const heading = document.createElement("strong");
    heading.textContent = event.name;
    const meta = document.createElement("time");
    meta.textContent = historyTimestamp(event.at);
    if (Number.isFinite(Date.parse(event.at))) meta.dateTime = event.at;
    const message = document.createElement("span");
    message.textContent = event.message;
    content.append(heading, meta, message);
    const button = document.createElement("button");
    button.type = "button";
    button.textContent = "Cloudflare ↗";
    button.title = "Open this event in Cloudflare";
    // Legacy records without a stored URL cannot target their original attempt.
    button.disabled = !event.id || !event.url;
    button.onclick = async () => {
      try {
        await invoke("open_history_event", {id: event.id});
      } catch {
        button.title = "This historical Cloudflare link is unavailable";
        button.setAttribute("aria-label", button.title);
      }
    };
    row.append(content, button);
    output.append(row);
  }
  list.replaceChildren(output);
}

async function refreshHistory() {
  try {
    const snapshot = await invoke("get_history");
    events = Array.isArray(snapshot) ? snapshot : [];
    render();
  } catch {
    summary.textContent = "Event history unavailable";
    list.textContent = "Unable to load event history.";
  }
}

search.addEventListener("input", render);
category.addEventListener("change", render);
window.addEventListener("focus", () => { void refreshHistory(); });
await listen("cfvigil-history-changed", () => { void refreshHistory(); });
await refreshHistory();
