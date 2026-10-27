//! Localized native shell labels and isolated, durable language preference.
//! An unsupported locale never activates incomplete UI strings.
use serde::{Deserialize, Serialize};

pub const ACTIVE: &[&str] = &["en", "ru", "es", "pt", "de", "fr", "it", "pl", "hi", "zh-Hant", "ja", "nl", "sv", "tr", "ko", "bn", "th", "id", "fil", "ar"];

fn default_locale() -> String { "en".to_owned() }

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(try_from = "serde_json::Map<String, serde_json::Value>")]
pub struct LanguagePreference {
    pub locale: String,
}
// The on-disk format has always been a JSON object. Serde's derived
// struct visitor otherwise accepts [] as an empty field sequence and
// silently bypasses read_or_default's corrupt-data quarantine.
impl TryFrom<serde_json::Map<String, serde_json::Value>> for LanguagePreference {
    type Error = String;
    fn try_from(mut fields: serde_json::Map<String, serde_json::Value>) -> Result<Self, Self::Error> {
        match fields.remove("locale") {
            None => Ok(Self::default()), // Compatible with legacy {}.
            Some(serde_json::Value::String(locale)) => Ok(Self { locale }),
            Some(_) => Err("Invalid saved language preference".into()),
        }
    }
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

    const LABELS:&[(&str,[&str;5])] = &[
        ("en",["Show Card","Event History","Settings","About","Exit"]),
        ("ru",["Показать карточку","История событий","Настройки","О программе","Выход"]),
        ("es",["Mostrar panel","Historial de eventos","Configuración","Acerca de","Salir"]),
        ("pt",["Mostrar painel","Histórico de eventos","Definições","Sobre","Sair"]),
        ("de",["Karte anzeigen","Ereignisverlauf","Einstellungen","Über","Beenden"]),
        ("fr",["Afficher la carte","Historique des événements","Paramètres","À propos","Quitter"]),
        ("it",["Mostra pannello","Cronologia eventi","Impostazioni","Informazioni","Esci"]),
        ("pl",["Pokaż panel","Historia zdarzeń","Ustawienia","O programie","Zakończ"]),
        ("hi",["कार्ड दिखाएँ","इवेंट इतिहास","सेटिंग्स","CFVigil के बारे में","बाहर निकलें"]),
        ("zh-Hant",["顯示卡片","事件記錄","設定","關於 CFVigil","離開"]),
        ("ja",["カードを表示","イベント履歴","設定","CFVigil について","終了"]),
        ("nl",["Paneel tonen","Gebeurtenisgeschiedenis","Instellingen","Over","Afsluiten"]),
        ("sv",["Visa panel","Händelsehistorik","Inställningar","Om","Avsluta"]),
        ("tr",["Kartı göster","Olay geçmişi","Ayarlar","CFVigil hakkında","Çıkış"]),
        ("ko",["카드 표시","이벤트 기록","설정","CFVigil 정보","종료"]),
        ("bn",["কার্ড দেখান","ইভেন্টের ইতিহাস","সেটিংস","CFVigil সম্পর্কে","প্রস্থান"]),
        ("th",["แสดงการ์ด","ประวัติเหตุการณ์","การตั้งค่า","เกี่ยวกับ CFVigil","ออก"]),
        ("id",["Tampilkan panel","Riwayat peristiwa","Pengaturan","Tentang CFVigil","Keluar"]),
        ("fil",["Ipakita ang card","History ng mga event","Mga Setting","Tungkol sa CFVigil","Lumabas"]),
        ("ar",["عرض البطاقة","سجل الأحداث","الإعدادات","حول CFVigil","خروج"])
    ];
    const ACTIONS:[&str;5]=["open","history","settings","about","quit"];

    #[test]
    fn given_all_registered_native_locales_when_selecting_then_each_exact_locale_is_supported() {
        assert_eq!(ACTIVE.len(),20);
        assert_eq!(LABELS.len(),ACTIVE.len());
        let mut seen=std::collections::HashSet::new();
        for ((locale,_),registered) in LABELS.iter().zip(ACTIVE.iter()) {
            assert_eq!(locale,registered,"native locale ordering must be stable");
            assert!(seen.insert(*locale),"duplicate native locale: {locale}");
            assert!(supported(locale),"{locale}");
            assert_eq!(sanitize(LanguagePreference {locale:(*locale).into()}).locale,*locale);
        }
    }

    #[test]
    fn given_invalid_native_locale_when_sanitizing_then_fallback_does_not_activate_partial_catalog() {
        for locale in [
            ""," ","EN","PT","pt-BR","pt-PT","zh-Hans","zh-hant","ar-EG",
            "fil-PH","unknown","../../en","en\n","en\0",
        ] {
            assert!(!supported(locale),"unsupported {locale:?}");
            assert_eq!(sanitize(LanguagePreference {locale:locale.into()}).locale,"en");
        }
    }

    #[test]
    fn given_language_preference_when_serializing_then_all_twenty_choices_roundtrip() {
        for locale in ACTIVE {
            let original=LanguagePreference{locale:(*locale).into()};
            let json=serde_json::to_string(&original).unwrap();
            let loaded:LanguagePreference=serde_json::from_str(&json).unwrap();
            assert_eq!(loaded.locale,*locale);
            assert_eq!(sanitize(loaded).locale,*locale);
        }
    }

    #[test]
    fn given_legacy_or_corrupt_locale_json_when_loading_then_missing_defaults_and_invalid_shapes_reject() {
        let default:LanguagePreference=serde_json::from_str("{}").unwrap();
        assert_eq!(default.locale,"en");
        let extra:LanguagePreference=serde_json::from_str(
            r#"{"locale":"pt","legacySetting":100}"#).unwrap();
        assert_eq!(extra.locale,"pt");
        for content in [
            r#"{"locale":null}"#,r#"{"locale":42}"#,r#"{"locale":[]}"#,
            r#"{"locale":{}}"#,r#"[]"#,r#"true"#,"{corrupt",
        ] {
            assert!(serde_json::from_str::<LanguagePreference>(content).is_err(),
                "malformed native language preference: {content}");
        }
        let unknown:LanguagePreference=serde_json::from_str(
            r#"{"locale":"zz"}"#).unwrap();
        assert_eq!(sanitize(unknown).locale,"en");
    }

    #[test]
    fn given_each_active_locale_when_building_native_menu_then_five_actions_have_exact_labels() {
        for (locale,expected) in LABELS {
            for (index,action) in ACTIONS.iter().enumerate() {
                assert_eq!(menu(locale,action),expected[index],
                    "native {locale} action {action}");
                assert!(!menu(locale,action).trim().is_empty());
            }
        }
    }

    #[test]
    fn given_unknown_action_or_unregistered_locale_when_building_menu_then_english_fallback_is_safe() {
        for locale in ["zz","en-GB","pt-BR","zh-Hans",""] {
            for action in ACTIONS {
                assert_eq!(menu(locale,action),menu("en",action),
                    "unsupported {locale:?} action {action}");
            }
        }
        for action in ["","unknown","Open","exit","destroy","../../settings"] {
            for locale in ACTIVE {
                assert_eq!(menu(locale,action),"CFVigil",
                    "unknown command action must not pretend to be executable");
            }
        }
    }

    #[test]
    fn given_native_window_titles_when_switching_twenty_locales_then_actions_update_consistently() {
        let about_titles=[
            "About CFVigil","О CFVigil","Acerca de CFVigil","Sobre o CFVigil",
            "Über CFVigil","À propos de CFVigil","Informazioni su CFVigil",
            "O programie CFVigil","CFVigil के बारे में","關於 CFVigil",
            "CFVigil について","Over CFVigil","Om CFVigil","CFVigil hakkında",
            "CFVigil 정보","CFVigil সম্পর্কে","เกี่ยวกับ CFVigil",
            "Tentang CFVigil","Tungkol sa CFVigil","حول CFVigil"
        ];
        assert_eq!(about_titles.len(),ACTIVE.len());
        for (i,locale) in ACTIVE.iter().enumerate() {
            assert_eq!(window_title(locale,"main"),"CFVigil");
            assert_eq!(window_title(locale,"about"),about_titles[i],"{locale} About");
            assert_eq!(window_title(locale,"settings"),menu(locale,"settings"));
            assert_eq!(window_title(locale,"history"),menu(locale,"history"));
            for unrecognized in ["","unknown","main2","quit"] {
                assert_eq!(window_title(locale,unrecognized),"CFVigil");
            }
        }
        for unsupported in ["","zz","pt-BR","zh-Hans"] {
            assert_eq!(window_title(unsupported,"about"),"About CFVigil");
            assert_eq!(window_title(unsupported,"history"),"Event History");
            assert_eq!(window_title(unsupported,"settings"),"Settings");
        }
    }

}
