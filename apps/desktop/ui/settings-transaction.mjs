// Settings are drafts until Apply. Omitting an unchanged OS side-effect is
// essential: a locale-only edit must not re-check/write the token or autostart.
export function planSettingsApply(previous, draft) {
  const prev = previous?.config ?? {account_id:"",refresh_seconds:180};
  const accountId = String(draft.accountId ?? "").trim();
  const refreshSeconds = Number(draft.refreshSeconds);
  const language = String(draft.language ?? "en");
  const oldLanguage = String(previous?.language ?? "en");
  const replacement = String(draft.token ?? "").trim();
  const autostart = previous?.autostart === null ||
    previous?.autostart === undefined ||
    draft.autostart === previous.autostart ? null : Boolean(draft.autostart);
  const changed = accountId !== prev.account_id ||
    refreshSeconds !== prev.refresh_seconds ||
    language !== oldLanguage || replacement !== "" || autostart !== null;
  return {
    changed,
    args: {
      accountId,
      refreshSeconds,
      token: replacement || null,
      language,
      autostart
    }
  };
}
