const invoke = (name, args={}) => window.__TAURI__.core.invoke(name, args);
const message = document.getElementById("message");
const form = document.getElementById("form");
document.getElementById("cancel").onclick = () => window.__TAURI__.core.getCurrentWindow().hide();
form.addEventListener("submit", async event => {
  event.preventDefault();
  message.textContent = "Saving...";
  try {
    await invoke("save_settings", {
      accountId: document.getElementById("account").value.trim(),
      token: document.getElementById("token").value.trim() || null,
      refreshSeconds: Number(document.getElementById("interval").value)
    });
    document.getElementById("token").value = "";
    message.textContent = "Saved. Resource discovery is starting.";
  } catch (error) { message.textContent = String(error); }
});
try {
  const config = await invoke("get_settings");
  document.getElementById("account").value = config.account_id || "";
  document.getElementById("interval").value = String(config.refresh_seconds);
} catch { message.textContent = "Unable to read settings"; }
