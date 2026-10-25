import {t, currentLocale, ACTIVE_LOCALES, LANGUAGE_NAMES} from "./localization.mjs";
import {initializeLanguage} from "./runtime-language.mjs";
const invoke = (name, args={}) => window.__TAURI__.core.invoke(name, args);
const message = document.getElementById("message-text");
const spinner = document.getElementById("save-spinner");
const token = document.getElementById("token");
const tokenStatus = document.getElementById("token-status");
const submit = document.querySelector('button[type="submit"]');
let saving = false;
let tokenPresent = false;
function showTokenPresence(present) {
  tokenPresent = present;
  token.placeholder = present ? "••••••••" : t("settings.tokenPlaceholder");
  tokenStatus.textContent = present ? t("settings.tokenStored") : "";
}
function setSaving(busy) {
  saving = busy;
  form.setAttribute("aria-busy", String(busy));
  spinner.hidden = !busy;
  submit.disabled = busy;
  document.getElementById("cancel").disabled = busy;
  document.getElementById("close-settings").disabled = busy;
}
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
    showTokenPresence(tokenPresent);
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
  token.value = "";
  showTokenPresence(tokenPresent);
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
    // Last content element's intrinsic box includes footer and padding.
    // Unlike body.scrollHeight, its offset is independent of viewport size.
    const content = document.querySelector(".settings-content");
    if (!content || !content.offsetHeight) return;
    const height = Math.ceil(content.offsetTop + content.offsetHeight + 2);
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
// Only the empty topbar starts a compositor-managed window drag.
document.getElementById("settings-drag-area").addEventListener("pointerdown", event => {
  if (event.button !== 0 || event.target.closest("button")) return;
  void invoke("start_settings_drag").catch(error => { console.warn(String(error)); });
});
const closeSettings = () => {
  if (saving) return;
  restoreDraft();
  void window.__TAURI__.window.getCurrentWindow().hide();
};
document.getElementById("close-settings").onclick = closeSettings;
document.getElementById("cancel").onclick = () => {
  if (saving) return;
  restoreDraft();
  void window.__TAURI__.window.getCurrentWindow().hide();
};
form.addEventListener("submit", async event => {
  event.preventDefault();
  if (saving) return;
  setSaving(true);
  message.textContent = t("settings.saving");
  const nextAccount = document.getElementById("account").value.trim();
  const nextRefresh = Number(document.getElementById("interval").value);
  const replacement = token.value.trim();
  const nextLanguage = language.value;
  try {
    await invoke("save_settings", {
      accountId: nextAccount,
      token: replacement || null,
      refreshSeconds: nextRefresh,
      autostart: autostartLoaded ? autostart.checked : null,
      language: nextLanguage
    });
    storedSettings = {account_id: nextAccount, refresh_seconds: nextRefresh};
    token.value = "";
    confirmedLanguage = nextLanguage;
    // Never return the secret to JS. Only the native bool state is used.
    try {
      showTokenPresence(await invoke("get_token_status"));
    } catch {
      tokenStatus.textContent = t("settings.tokenUnavailable");
    }
    message.textContent = t("settings.saved");
    if (autostartLoaded) {
      try {
        autostart.checked = await invoke("get_autostart");
        storedAutostart = autostart.checked;
        autostartHint.textContent = autostart.checked
          ? t("settings.autostartEnabled")
          : t("settings.autostartDisabled");
      } catch {
        autostartLoaded = false;
        autostart.disabled = true;
        autostartHint.textContent = t("settings.autostartUnavailable");
      }
    }
  } catch (error) {
    message.textContent = String(error);
  } finally {
    setSaving(false);
    scheduleFit();
  }
});

try {
  showTokenPresence(await invoke("get_token_status"));
} catch {
  tokenStatus.textContent = t("settings.tokenUnavailable");
}

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
