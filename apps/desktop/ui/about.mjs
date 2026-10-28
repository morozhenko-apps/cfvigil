import "./suppress-context-menu.mjs";
import {t} from "./localization.mjs";
import {initializeLanguage} from "./runtime-language.mjs";

const invoke = (name, args={}) => window.__TAURI__.core.invoke(name, args);
const listen = (event, callback) => window.__TAURI__.event.listen(event, callback);
let version = "—";

function renderVersion() {
  document.getElementById("about-version").textContent =
    t("about.version", {version});
}

await initializeLanguage({invoke,listen,doc:document,onChange:renderVersion});
try {
  version = await invoke("get_app_version");
} catch {
  // About text remains usable without a native version response.
}
renderVersion();

document.getElementById("about-website").addEventListener("click", () => {
  void invoke("open_about_website").catch(() => {
    document.getElementById("about-website").title = t("card.openError");
  });
});
document.getElementById("about-source").addEventListener("click", () => {
  void invoke("open_about_source").catch(() => {
    document.getElementById("about-source").title = t("card.openError");
  });
});

const closeAbout = () => { void window.__TAURI__.window.getCurrentWindow().hide(); };
document.getElementById("about-close").addEventListener("click", closeAbout);
document.addEventListener("keydown", event => {
  if (event.key === "Escape") { event.preventDefault(); closeAbout(); }
});
