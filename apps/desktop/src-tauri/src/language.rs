//! Localized native shell labels and isolated, durable language preference.
//! An unsupported locale never activates incomplete UI strings.
use serde::{Deserialize, Serialize};

pub const ACTIVE: &[&str] = &["en", "ru", "pt", "es", "de", "fr", "it", "pl", "nl", "sv"];

fn default_locale() -> String { "en".to_owned() }

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LanguagePreference {
    #[serde(default = "default_locale")]
    pub locale: String,
}
impl Default for LanguagePreference {
    fn default() -> Self { Self { locale: default_locale() } }
}
pub fn supported(locale: &str) -> bool { ACTIVE.contains(&locale) }
pub fn sanitize(value: LanguagePreference) -> LanguagePreference {
    if supported(&value.locale) { value } else { LanguagePreference::default() }
}

pub fn menu(locale: &str, action: &str) -> &'static str {
    match (locale, action) {
        ("ru", "open") => "Показать карточку",
        ("ru", "history") => "История событий",
        ("ru", "settings") => "Настройки",
        ("ru", "about") => "О программе",
        ("ru", "quit") => "Выход",
        ("pt", "open") => "Mostrar painel",
        ("pt", "history") => "Histórico de eventos",
        ("pt", "settings") => "Definições",
        ("pt", "about") => "Sobre",
        ("pt", "quit") => "Sair",
        ("es", "open") => "Mostrar panel",
        ("es", "history") => "Historial de eventos",
        ("es", "settings") => "Configuración",
        ("es", "about") => "Acerca de",
        ("es", "quit") => "Salir",
        ("de", "open") => "Karte anzeigen",
        ("de", "history") => "Ereignisverlauf",
        ("de", "settings") => "Einstellungen",
        ("de", "about") => "Über",
        ("de", "quit") => "Beenden",
        ("fr", "open") => "Afficher la carte",
        ("fr", "history") => "Historique des événements",
        ("fr", "settings") => "Paramètres",
        ("fr", "about") => "À propos",
        ("fr", "quit") => "Quitter",
        ("it", "open") => "Mostra pannello",
        ("it", "history") => "Cronologia eventi",
        ("it", "settings") => "Impostazioni",
        ("it", "about") => "Informazioni",
        ("it", "quit") => "Esci",
        ("pl", "open") => "Pokaż panel",
        ("pl", "history") => "Historia zdarzeń",
        ("pl", "settings") => "Ustawienia",
        ("pl", "about") => "O programie",
        ("pl", "quit") => "Zakończ",
        ("nl", "open") => "Paneel tonen",
        ("nl", "history") => "Gebeurtenisgeschiedenis",
        ("nl", "settings") => "Instellingen",
        ("nl", "about") => "Over",
        ("nl", "quit") => "Afsluiten",
        ("sv", "open") => "Visa panel",
        ("sv", "history") => "Händelsehistorik",
        ("sv", "settings") => "Inställningar",
        ("sv", "about") => "Om",
        ("sv", "quit") => "Avsluta",
        (_, "open") => "Show Card",
        (_, "history") => "Event History",
        (_, "settings") => "Settings",
        (_, "about") => "About",
        (_, "quit") => "Exit",
        _ => "CFVigil",
    }
}
pub fn window_title(locale: &str, label: &str) -> &'static str {
    match label {
        "main" => "CFVigil",
        "about" => match locale {
            "ru" => "О CFVigil",
            "pt" => "Sobre o CFVigil",
            "es" => "Acerca de CFVigil",
            "de" => "Über CFVigil",
            "fr" => "À propos de CFVigil",
            "it" => "Informazioni su CFVigil",
            "pl" => "O programie CFVigil",
            "nl" => "Over CFVigil",
            "sv" => "Om CFVigil",
            _ => "About CFVigil"
        },
        "settings" => menu(locale, "settings"),
        "history" => menu(locale, "history"),
        _ => "CFVigil",
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn unknown_languages_fail_closed() {
        assert!(ACTIVE.iter().all(|l| supported(l)));
        assert!(!supported("pt-BR"));
        assert!(!supported("zh-Hans"));
        assert_eq!(sanitize(LanguagePreference {locale: "zz".into()}).locale, "en");
        assert_eq!(sanitize(LanguagePreference {locale: "pt".into()}).locale, "pt");
    }
}
