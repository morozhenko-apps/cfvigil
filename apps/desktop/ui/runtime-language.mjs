import {setLocale, localizeDocument} from "./localization.mjs";

// Register first to avoid missing a language change while bootstrapping.
// Each WebView owns one JS locale instance; native preferences are authoritative.
export async function initializeLanguage({invoke, listen, doc, onChange = () => {}}) {
  const apply = language => {
    setLocale(language);
    localizeDocument(doc);
    onChange();
  };
  await listen("cfvigil-language-changed", ({payload}) => apply(payload));
  try {
    apply(await invoke("get_language"));
  } catch {
    apply("en");
  }
}
