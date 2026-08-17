//! Language dictionary and internationalization module.

use std::collections::HashMap;
use std::sync::{LazyLock, RwLock};

/// German locale JSON dictionary embedded at compile time.
pub const DE_JSON: &str = include_str!("../../resources/locales/de.json");

/// English locale JSON dictionary embedded at compile time.
pub const EN_JSON: &str = include_str!("../../resources/locales/en.json");

static ACTIVE_DICT: LazyLock<RwLock<HashMap<String, String>>> =
    LazyLock::new(|| RwLock::new(HashMap::new()));

/// Initializes the active localization dictionary from a language code.
///
/// Selects German for `"de"`, English for `"en"`, and an empty dictionary for any other code.
/// Case-insensitive and trims leading and trailing whitespace.
///
/// # Examples
///
/// ```
/// use doc2flow::core::language::{init, localize};
///
/// init("de");
/// assert_eq!(localize("export_pdf"), "Als PDF exportieren");
///
/// init("en");
/// assert_eq!(localize("export_pdf"), "Export as PDF");
///
/// init("fr");
/// assert_eq!(localize("export_pdf"), "{{export_pdf}}");
/// ```
pub fn init(lang: &str) {
    let raw_json = match lang.trim() {
        c if c.eq_ignore_ascii_case("de") => Some(DE_JSON),
        c if c.eq_ignore_ascii_case("en") => Some(EN_JSON),
        _ => None,
    };

    let map = match raw_json {
        Some(json) => crate::core::json::parse_json_map(json)
            .expect("Failed to parse locale JSON: invalid format or unsupported escapes"),
        None => HashMap::new(),
    };

    let mut lock = ACTIVE_DICT.write().unwrap_or_else(|e| e.into_inner());
    *lock = map;
}

/// Localizes a key using the active dictionary.
///
/// Returns the localized string value if present in the active dictionary,
/// or `"{{<key>}}"` if missing.
///
/// # Examples
///
/// ```
/// use doc2flow::core::language::{init, localize, t};
///
/// init("en");
/// assert_eq!(localize("callout_note"), "Note");
/// assert_eq!(t("callout_note"), "Note");
/// assert_eq!(localize("nonexistent_key"), "{{nonexistent_key}}");
/// ```
#[must_use]
pub fn localize(key: &str) -> String {
    let lock = ACTIVE_DICT.read().unwrap_or_else(|e| e.into_inner());
    if let Some(val) = lock.get(key) {
        return val.clone();
    }
    format!("{{{{{key}}}}}")
}

/// Convenience alias for [`localize`].
#[must_use]
pub fn t(key: &str) -> String {
    localize(key)
}

#[cfg(test)]
pub static TEST_I18N_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_localize_english_and_german() {
        let _guard = TEST_I18N_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        init("en");
        assert_eq!(localize("export_pdf"), "Export as PDF");
        assert_eq!(localize("callout_note"), "Note");
        assert_eq!(t("callout_note"), "Note");

        init("de");
        assert_eq!(localize("export_pdf"), "Als PDF exportieren");
        assert_eq!(localize("callout_note"), "Hinweis");
        assert_eq!(localize("callout_caution"), "Achtung");
        assert_eq!(t("callout_caution"), "Achtung");
    }

    #[test]
    fn test_localize_case_insensitive_and_whitespace() {
        let _guard = TEST_I18N_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        init("DE");
        assert_eq!(localize("export_pdf"), "Als PDF exportieren");
        init("  de  ");
        assert_eq!(localize("export_pdf"), "Als PDF exportieren");
        init("EN");
        assert_eq!(localize("export_pdf"), "Export as PDF");
        init("  en  ");
        assert_eq!(localize("export_pdf"), "Export as PDF");
    }

    #[test]
    fn test_localize_fallback_for_unknown_language_or_key() {
        let _guard = TEST_I18N_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        init("fr");
        assert_eq!(localize("export_pdf"), "{{export_pdf}}");
        init("");
        assert_eq!(localize("export_pdf"), "{{export_pdf}}");
        init("   ");
        assert_eq!(localize("export_pdf"), "{{export_pdf}}");

        init("de");
        assert_eq!(localize("nonexistent_key"), "{{nonexistent_key}}");
        init("en");
        assert_eq!(localize("nonexistent_key"), "{{nonexistent_key}}");
        init("fr");
        assert_eq!(localize("nonexistent_key"), "{{nonexistent_key}}");
    }

    #[test]
    fn test_json_deserialization_with_escapes() {
        let mock_json = r#"{
            "escaped_newline": "Line 1\nLine 2",
            "escaped_quote": "He said \"hello\"",
            "unicode_escape": "Sample \u2713",
            "escaped_backslash": "C:\\path\\file"
        }"#;
        let dict: HashMap<String, String> = crate::core::json::parse_json_map(mock_json)
            .expect("Should deserialize JSON with escapes into HashMap<String, String>");
        assert_eq!(
            dict.get("escaped_newline"),
            Some(&"Line 1\nLine 2".to_string())
        );
        assert_eq!(
            dict.get("escaped_quote"),
            Some(&"He said \"hello\"".to_string())
        );
        assert_eq!(dict.get("unicode_escape"), Some(&"Sample ✓".to_string()));
        assert_eq!(
            dict.get("escaped_backslash"),
            Some(&"C:\\path\\file".to_string())
        );
    }

    #[test]
    fn test_localize_thread_safe_cross_thread_propagation() {
        let _guard = TEST_I18N_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        init("de");
        let handle = std::thread::spawn(|| localize("export_pdf"));
        assert_eq!(handle.join().unwrap(), "Als PDF exportieren");

        init("en");
        let handle = std::thread::spawn(|| localize("export_pdf"));
        assert_eq!(handle.join().unwrap(), "Export as PDF");

        init("fr");
        let handle = std::thread::spawn(|| localize("export_pdf"));
        assert_eq!(handle.join().unwrap(), "{{export_pdf}}");
    }

    #[test]
    fn test_rapid_reinitialization() {
        let _guard = TEST_I18N_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        init("en");
        assert_eq!(localize("export_pdf"), "Export as PDF");
        init("de");
        assert_eq!(localize("export_pdf"), "Als PDF exportieren");
        init("fr");
        assert_eq!(localize("export_pdf"), "{{export_pdf}}");
        init("en");
        assert_eq!(localize("export_pdf"), "Export as PDF");
    }
}
