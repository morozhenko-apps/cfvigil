//! Localized native shell labels and isolated, durable language preference.
//! An unsupported locale never activates incomplete UI strings.
use serde::{Deserialize, Serialize};

pub const ACTIVE: &[&str] = &["en", "ru", "es", "pt", "de", "fr", "it", "pl", "hi", "zh-Hant", "ja", "nl", "sv", "tr", "ko", "bn", "th", "id", "fil", "ar"];

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
        ("tr", "open") => "Kartı göster",
        ("tr", "history") => "Olay geçmişi",
        ("tr", "settings") => "Ayarlar",
        ("tr", "about") => "CFVigil hakkında",
        ("tr", "quit") => "Çıkış",
        ("id", "open") => "Tampilkan panel",
        ("id", "history") => "Riwayat peristiwa",
        ("id", "settings") => "Pengaturan",
        ("id", "about") => "Tentang CFVigil",
        ("id", "quit") => "Keluar",
        ("fil", "open") => "Ipakita ang card",
        ("fil", "history") => "History ng mga event",
        ("fil", "settings") => "Mga Setting",
        ("fil", "about") => "Tungkol sa CFVigil",
        ("fil", "quit") => "Lumabas",
        ("zh-Hant", "open") => "顯示卡片",
        ("zh-Hant", "history") => "事件記錄",
        ("zh-Hant", "settings") => "設定",
        ("zh-Hant", "about") => "關於 CFVigil",
        ("zh-Hant", "quit") => "離開",
        ("ja", "open") => "カードを表示",
        ("ja", "history") => "イベント履歴",
        ("ja", "settings") => "設定",
        ("ja", "about") => "CFVigil について",
        ("ja", "quit") => "終了",
        ("ko", "open") => "카드 표시",
        ("ko", "history") => "이벤트 기록",
        ("ko", "settings") => "설정",
        ("ko", "about") => "CFVigil 정보",
        ("ko", "quit") => "종료",
        ("hi", "open") => "कार्ड दिखाएँ",
        ("hi", "history") => "इवेंट इतिहास",
        ("hi", "settings") => "सेटिंग्स",
        ("hi", "about") => "CFVigil के बारे में",
        ("hi", "quit") => "बाहर निकलें",
        ("bn", "open") => "কার্ড দেখান",
        ("bn", "history") => "ইভেন্টের ইতিহাস",
        ("bn", "settings") => "সেটিংস",
        ("bn", "about") => "CFVigil সম্পর্কে",
        ("bn", "quit") => "প্রস্থান",
        ("th", "open") => "แสดงการ์ด",
        ("th", "history") => "ประวัติเหตุการณ์",
        ("th", "settings") => "การตั้งค่า",
        ("th", "about") => "เกี่ยวกับ CFVigil",
        ("th", "quit") => "ออก",
        ("ar", "open") => "عرض البطاقة",
        ("ar", "history") => "سجل الأحداث",
        ("ar", "settings") => "الإعدادات",
        ("ar", "about") => "حول CFVigil",
        ("ar", "quit") => "خروج",
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
            "tr" => "CFVigil hakkında",
            "id" => "Tentang CFVigil",
            "fil" => "Tungkol sa CFVigil",
            "zh-Hant" => "關於 CFVigil",
            "ja" => "CFVigil について",
            "ko" => "CFVigil 정보",
            "hi" => "CFVigil के बारे में",
            "bn" => "CFVigil সম্পর্কে",
            "th" => "เกี่ยวกับ CFVigil",
            "ar" => "حول CFVigil",
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
