//! Language dictionary and internationalization module.

use std::collections::HashMap;
use std::sync::LazyLock;

/// German locale JSON dictionary embedded at compile time.
pub const DE_JSON: &str = include_str!("../../resources/locales/de.json");

/// English locale JSON dictionary embedded at compile time.
pub const EN_JSON: &str = include_str!("../../resources/locales/en.json");

static DE_DICT: LazyLock<HashMap<&'static str, &'static str>> =
    LazyLock::new(|| serde_json::from_str(DE_JSON).unwrap_or_default());

static EN_DICT: LazyLock<HashMap<&'static str, &'static str>> =
    LazyLock::new(|| serde_json::from_str(EN_JSON).unwrap_or_default());

/// Translates a localization key for the specified language code.
///
/// Falls back to English if the key is not present in the requested language,
/// or returns an empty string if the key does not exist in any dictionary.
///
/// # Examples
///
/// ```
/// use doc2flow::core::language::translate;
///
/// assert_eq!(translate("export_pdf", "en"), "Export as PDF");
/// assert_eq!(translate("export_pdf", "de"), "Als PDF exportieren");
/// assert_eq!(translate("callout_note", "de"), "Hinweis");
/// assert_eq!(translate("callout_note", "en"), "Note");
/// assert_eq!(translate("nonexistent_key", "en"), "");
/// ```
#[must_use]
pub fn translate(key: &str, lang: &str) -> &'static str {
    let dict = match lang.trim() {
        c if c.eq_ignore_ascii_case("de") => &*DE_DICT,
        _ => &*EN_DICT,
    };
    if let Some(&val) = dict.get(key) {
        return val;
    }
    if let Some(&val) = EN_DICT.get(key) {
        return val;
    }
    ""
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_translate_english_and_german() {
        assert_eq!(translate("export_pdf", "en"), "Export as PDF");
        assert_eq!(translate("export_pdf", "de"), "Als PDF exportieren");
        assert_eq!(translate("callout_note", "en"), "Note");
        assert_eq!(translate("callout_note", "de"), "Hinweis");
        assert_eq!(translate("callout_caution", "de"), "Achtung");
    }

    #[test]
    fn test_translate_case_insensitive_and_whitespace() {
        assert_eq!(translate("export_pdf", "DE"), "Als PDF exportieren");
        assert_eq!(translate("export_pdf", "  de  "), "Als PDF exportieren");
        assert_eq!(translate("export_pdf", "EN"), "Export as PDF");
        assert_eq!(translate("export_pdf", "  en  "), "Export as PDF");
    }

    #[test]
    fn test_translate_fallback_for_unknown_language_or_key() {
        assert_eq!(translate("export_pdf", "fr"), "Export as PDF");
        assert_eq!(translate("export_pdf", ""), "Export as PDF");
        assert_eq!(translate("export_pdf", "   "), "Export as PDF");

        assert_eq!(translate("nonexistent_key", "de"), "");
        assert_eq!(translate("nonexistent_key", "en"), "");
        assert_eq!(translate("nonexistent_key", "fr"), "");
    }
}
