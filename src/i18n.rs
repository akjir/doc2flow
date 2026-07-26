//! Internationalization module for Doc2Flow static UI terms.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

include!(concat!(env!("OUT_DIR"), "/locales_gen.rs"));

/// Localized terms for generated HTML UI layout.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Locale {
    /// Language identifier for the locale.
    pub lang_code: String,
    /// Dynamic key-value pairs for localized UI terms.
    pub entries: HashMap<String, String>,
}

impl Locale {
    /// Constructs a `Locale` by parsing a raw JSON string.
    ///
    /// # Examples
    ///
    /// ```
    /// use doc2flow::i18n::Locale;
    ///
    /// let json = r#"{"lang_code": "en", "company": "Company"}"#;
    /// let locale = Locale::from_json(json);
    /// assert_eq!(locale.get("company"), "Company");
    /// ```
    pub fn from_json(json_str: &str) -> Self {
        let entries: HashMap<String, String> =
            serde_json::from_str(json_str).expect("Failed to deserialize locale JSON");
        let lang_code = entries
            .get("lang_code")
            .cloned()
            .unwrap_or_else(|| "en".to_string());
        Locale { lang_code, entries }
    }

    /// Loads an embedded locale corresponding to the normalized language code.
    ///
    /// Normalizes `code` by trimming whitespace and converting to lowercase. If no matching
    /// locale file is embedded, falls back to default English (`"en"`).
    pub fn from_lang_code(code: &str) -> Self {
        let normalized = code.trim().to_lowercase();
        if let Some(json_str) = get_embedded_locale(&normalized) {
            Self::from_json(json_str)
        } else if let Some(fallback_str) = get_embedded_locale("en") {
            Self::from_json(fallback_str)
        } else {
            Locale {
                lang_code: normalized,
                entries: HashMap::new(),
            }
        }
    }

    /// Safe getter returning the string entry for `key`, or empty string if missing.
    pub fn get(&self, key: &str) -> &str {
        self.entries.get(key).map(|s| s.as_str()).unwrap_or("")
    }

    /// Returns the localized entry value for `key` ignoring ASCII case.
    pub fn get_ignore_ascii_case(&self, key: &str) -> Option<&str> {
        self.entries
            .iter()
            .find(|(k, _)| k.eq_ignore_ascii_case(key))
            .map(|(_, v)| v.as_str())
    }
}

impl Default for Locale {
    fn default() -> Self {
        Self::from_lang_code("en")
    }
}

/// Validates that all `{{L_KEY}}` placeholders in `template` exist in `locale.entries`.
///
/// Prints a non-blocking warning message to `stderr` for any missing key.
/// Performs zero heap allocations during scanning.
pub fn validate_locale_coverage(template: &str, locale: &Locale) {
    let mut cursor = 0;
    while let Some(start) = template[cursor..].find("{{L_") {
        let abs_start = cursor + start;
        if let Some(end) = template[abs_start + 4..].find("}}") {
            let abs_end = abs_start + 4 + end;
            let key_name = &template[abs_start + 4..abs_end];

            if locale.get_ignore_ascii_case(key_name).is_none() {
                eprintln!(
                    "Warning: Missing translation key 'L_{}' in locale '{}'",
                    key_name, locale.lang_code
                );
            }
            cursor = abs_end + 2;
        } else {
            break;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_json_dynamic_loading() {
        let json = r#"{
            "lang_code": "fr",
            "custom_key": "Bonjour",
            "company": "Client"
        }"#;
        let locale = Locale::from_json(json);
        assert_eq!(locale.lang_code, "fr");
        assert_eq!(locale.get("custom_key"), "Bonjour");
        assert_eq!(locale.get("company"), "Client");
    }

    #[test]
    fn test_from_lang_code_resolution() {
        let de_locale = Locale::from_lang_code("de");
        assert_eq!(de_locale.lang_code, "de");
        assert_eq!(de_locale.get("company"), "Firma");

        let en_locale = Locale::from_lang_code("en");
        assert_eq!(en_locale.lang_code, "en");
        assert_eq!(en_locale.get("company"), "Company");

        let unknown = Locale::from_lang_code("xyz");
        assert_eq!(unknown.lang_code, "en");
    }

    #[test]
    fn test_get_fallback_unknown_key() {
        let locale = Locale::from_lang_code("de");
        assert_eq!(locale.get("nonexistent_key_123"), "");
    }

    #[test]
    fn test_validate_locale_coverage() {
        let mut entries = HashMap::new();
        entries.insert("company".into(), "Firma".into());
        let locale = Locale {
            lang_code: "de".into(),
            entries,
        };

        let tmpl = "<div>{{L_COMPANY}}</div><div>{{L_MISSING_KEY}}</div>";
        // Call validation; missing key prints to stderr without panicking
        validate_locale_coverage(tmpl, &locale);
    }
}
