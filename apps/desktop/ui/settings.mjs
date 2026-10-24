const invoke = (name, args={}) => window.__TAURI__.core.invoke(name, args);
const message = document.getElementById("message");
const form = document.getElementById("form");
const autostart = document.getElementById("autostart");
const autostartHint = document.getElementById("autostart-hint");
let autostartLoaded = false;
document.getElementById("cancel").onclick = () => window.__TAURI__.window.getCurrentWindow().hide();
form.addEventListener("submit", async event => {
  event.preventDefault();
  message.textContent = "Saving...";
  try {
    await invoke("save_settings", {
      accountId: document.getElementById("account").value.trim(),
      token: document.getElementById("token").value.trim() || null,
      refreshSeconds: Number(document.getElementById("interval").value),
      // Null means no change when the OS startup state is unavailable.
      autostart: autostartLoaded ? autostart.checked : null
    });
    document.getElementById("token").value = "";
    message.textContent = "Saved.";
    if (autostartLoaded) {
      autostart.checked = await invoke("get_autostart");
      autostartHint.textContent = autostart.checked
        ? "CFVigil will launch after sign-in, minimized to the tray."
        : "CFVigil will not launch automatically.";
    }
  } catch (error) { message.textContent = String(error); }
});
try {
  const config = await invoke("get_settings");
  document.getElementById("account").value = config.account_id || "";
  document.getElementById("interval").value = String(config.refresh_seconds);
} catch { message.textContent = "Unable to read settings"; }

try {
  autostart.checked = await invoke("get_autostart");
  autostartLoaded = true;
  autostart.disabled = false;
  autostartHint.textContent = autostart.checked
    ? "Launch at sign-in is currently enabled."
    : "Off by default. Enable only if you want CFVigil to start automatically.";
} catch {
  // Never interpret a failed OS check as false and disable an existing entry.
  autostart.disabled = true;
  autostartHint.textContent = "System startup state unavailable; other settings remain editable.";
}

try {
  const snapshot = await invoke("get_snapshot");
  if (snapshot?.storage_warning) {
    message.textContent =
      "Damaged local state was preserved as a .corrupt.* file in the CFVigil settings folder. Please verify your connection and history.";
  }
} catch {
  // Connection settings remain available when the card snapshot cannot load.
}
