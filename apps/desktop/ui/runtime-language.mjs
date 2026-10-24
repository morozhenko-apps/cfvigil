import {setLocale, localizeDocument} from "./localization.mjs";

// Subscribe before taking the native snapshot. If the user switches locale
// during the asynchronous read, the event is newer and takes precedence.
// No polling, IPC retries or local storage mirrors are needed.
export async function initializeLanguage({invoke, listen, doc, onChange = () => {}}) {
  const apply = language => {
    setLocale(language);
    localizeDocument(doc);
    onChange();
  };
  let receivedChange = false;
  await listen("cfvigil-language-changed", ({payload}) => {
    if (typeof payload !== "string") return;
    receivedChange = true;
    apply(payload);
  });
  try {
    const initial = await invoke("get_language");
    if (!receivedChange) apply(initial);
  } catch (error) {
    if (!receivedChange) apply("en");
  }
}
