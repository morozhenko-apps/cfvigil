import {t, currentLocale, ACTIVE_LOCALES, LANGUAGE_NAMES} from "./localization.mjs";
import {initializeLanguage} from "./runtime-language.mjs";
const invoke = (name, args={}) => window.__TAURI__.core.invoke(name, args);
const message = document.getElementById("message");
const form = document.getElementById("form");
const autostart = document.getElementById("autostart");
const autostartHint = document.getElementById("autostart-hint");
const language = document.getElementById("language");
let autostartLoaded = false;
let confirmedLanguage = "en";
let fitFrame = null;
let lastRequestedHeight = null;
for (const code of ACTIVE_LOCALES) {
  const option = document.createElement("option");
  option.value = code;
  option.textContent = LANGUAGE_NAMES[code];
  language.append(option);
}

await initializeLanguage({invoke,listen: (event, callback) => window.__TAURI__.event.listen(event, callback),
  doc:document,onChange: () => {
    confirmedLanguage = currentLocale;
    language.value = currentLocale;
    scheduleFit();
  }});
language.disabled = false;
// Select changes are drafts. Only form Submit commits native settings and locale.
language.addEventListener("change", () => { message.textContent = ""; });
let storedSettings = null;
let storedAutostart = null;
function restoreDraft() {
  if (storedSettings) {
    document.getElementById("account").value = storedSettings.account_id || "";
    document.getElementById("interval").value = String(storedSettings.refresh_seconds);
  }
  document.getElementById("token").value = "";
  if (storedAutostart !== null) autostart.checked = storedAutostart;
  language.value = confirmedLanguage;
  message.textContent = "";
}
// The form is the last real content in Settings. Do not use body.scrollHeight:
// it includes the current WebView viewport and can only grow after an oversized
// window, causing a self-reinforcing bottom gap.
function scheduleFit() {
  if (fitFrame !== null) return;
  fitFrame = requestAnimationFrame(() => {
    fitFrame = null;
    const style = getComputedStyle(document.body);
    const bottomMargin = Number.parseFloat(style.marginBottom) || 0;
    // offsetTop + offsetHeight is independent of the native window height
    // and remains stable even if the user scrolled a long locale's form.
    const intrinsicBottom = form.offsetTop + form.offsetHeight;
    if (!intrinsicBottom) return; // Hidden WebView has not laid out yet.
    const height = Math.ceil(intrinsicBottom + bottomMargin + 2);
    if (height === lastRequestedHeight) return;
    lastRequestedHeight = height;
    void invoke("fit_settings", {height}).catch(() => {
      // A subsequent focus/open can retry an unavailable native window.
      lastRequestedHeight = null;
    });
  });
}
window.addEventListener("focus", scheduleFit);
await window.__TAURI__.event.listen("cfvigil-settings-opened", () => {
  // The OS close button also means Cancel, not a stored draft that
  // reappears when the same hidden WebView is later shown.
  restoreDraft();
  scheduleFit();
});
document.getElementById("cancel").onclick = () => {
  restoreDraft();
  void window.__TAURI__.window.getCurrentWindow().hide();
};
form.addEventListener("submit", async event => {
  event.preventDefault();
  message.textContent = t("settings.saving");
  try {
    await invoke("save_settings", {
      accountId: document.getElementById("account").value.trim(),
      token: document.getElementById("token").value.trim() || null,
      refreshSeconds: Number(document.getElementById("interval").value),
      // Null means no change when the OS startup state is unavailable.
      autostart: autostartLoaded ? autostart.checked : null,
      language: language.value
    });
    storedSettings = {
      account_id: document.getElementById("account").value.trim(),
      refresh_seconds: Number(document.getElementById("interval").value)
    };
    document.getElementById("token").value = "";
    confirmedLanguage = language.value;
    message.textContent = t("settings.saved");
    scheduleFit();
    if (autostartLoaded) {
      autostart.checked = await invoke("get_autostart");
      storedAutostart = autostart.checked;
      autostartHint.textContent = autostart.checked
        ? t("settings.autostartEnabled")
        : t("settings.autostartDisabled");
    }
  } catch (error) { message.textContent = String(error); }
});
try {
  const config = await invoke("get_settings");
  storedSettings = config;
  document.getElementById("account").value = config.account_id || "";
  document.getElementById("interval").value = String(config.refresh_seconds);
} catch { message.textContent = t("settings.readError"); }

try {
  autostart.checked = await invoke("get_autostart");
  autostartLoaded = true;
  storedAutostart = autostart.checked;
  autostart.disabled = false;
  autostartHint.textContent = autostart.checked
    ? t("settings.autostartDetectedOn")
    : t("settings.autostartDetectedOff");
} catch {
  // Never interpret a failed OS check as false and disable an existing entry.
  autostart.disabled = true;
  autostartHint.textContent = t("settings.autostartUnavailable");
}

try {
  const snapshot = await invoke("get_snapshot");
  if (snapshot?.storage_warning) {
    message.textContent =
      t("settings.storageWarning");
  }
} catch {
  // Connection settings remain available when the card snapshot cannot load.
}

scheduleFit();
