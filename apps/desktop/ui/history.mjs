const invoke = (name,args={}) => window.__TAURI__.core.invoke(name,args);
const list = document.getElementById("history");
try {
  const events = await invoke("get_history");
  list.replaceChildren();
  if (!events.length) list.textContent = "No events yet.";
  for (const event of events) {
    const line = document.createElement("p");
    line.setAttribute("role","listitem");
    const label = document.createElement("span");
    label.textContent = `${new Date(event.at).toLocaleString()} · ${event.name} · ${event.message} `;
    const button = document.createElement("button");
    button.textContent = "Cloudflare ↗";
    button.onclick = () => invoke("open_history_resource", { key: event.key });
    line.append(label,button);
    list.append(line);
  }
} catch { list.textContent = "Unable to load event history."; }
