import {localizeDocument, t} from "./localization.mjs";
localizeDocument(document);
document.getElementById("about-version").textContent = t("about.version", {version: "0.1.0"});
