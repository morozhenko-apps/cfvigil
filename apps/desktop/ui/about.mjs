import {t} from "./localization.mjs";
import {initializeLanguage} from "./runtime-language.mjs";
const invoke = (name, args={}) => window.__TAURI__.core.invoke(name, args);
const listen = (name, callback) => window.__TAURI__.event.listen(name, callback);
const updateVersion = () => {
  document.getElementById("about-version").textContent = t("about.version", {version:"0.1.0"});
};
await initializeLanguage({invoke,listen,doc:document,onChange:updateVersion});
