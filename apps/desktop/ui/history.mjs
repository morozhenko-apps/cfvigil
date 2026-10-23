const invoke = (name, args={}) => window.__TAURI__.core.invoke(name, args);
const listen = (name, callback) => window.__TAURI__.event.listen(name, callback);
const list = document.getElementById("history");

async function refreshHistory() {
  try {
    const events = await invoke("get_history");
    const output = document.createDocumentFragment();
    if (!events.length) {
      const empty = document.createElement("p");
      empty.textContent = "No events yet.";
      output.append(empty);
    }
    for (const event of events) {
      const line = document.createElement("p");
      line.setAttribute("role", "listitem");
      const label = document.createElement("span");
      label.textContent = `${new Date(event.at).toLocaleString()} · ${event.name} · ${event.message} `;
      const button = document.createElement("button");
      button.textContent = "Cloudflare ↗";
      button.onclick = async () => {
        try {
          await invoke("open_history_resource", {key: event.key});
        } catch {
          button.title = "Resource is unavailable in the current account";
        }
      };
      line.append(label, button);
      output.append(line);
    }
    list.replaceChildren(output);
  } catch {
    list.textContent = "Unable to load event history.";
  }
}

window.addEventListener("focus", () => { void refreshHistory(); });
await listen("cfvigil-history-changed", () => { void refreshHistory(); });
await refreshHistory();
