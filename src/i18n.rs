//! Internationalization module for Doc2Flow static UI terms.

use serde::{Deserialize, Serialize};

/// Represents localized static terms for generated HTML UI layout.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Locale {
    pub lang_code: String,
    pub customer: String,
    pub employee: String,
    pub technician: String,
    pub date: String,
    pub setup_completed: String,
    pub name_placeholder: String,
    pub signature_technician: String,
    pub date_placeholder: String,
    pub signature_date: String,
    pub export_pdf: String,
    pub reset_all: String,
    pub copy_code: String,
    pub copied: String,
    pub progress_template: String,
    pub loading: String,
    pub confirm_reset: String,
    pub callout_note: String,
    pub callout_tip: String,
    pub callout_important: String,
    pub callout_warning: String,
    pub callout_caution: String,
}

impl Locale {
    /// Loads English locale embedded at compile time.
    pub fn english() -> Self {
        let json_str = include_str!("../locales/en.json");
        serde_json::from_str(json_str).expect("Failed to deserialize embedded English locale")
    }

    /// Loads German locale embedded at compile time.
    pub fn german() -> Self {
        let json_str = include_str!("../locales/de.json");
        serde_json::from_str(json_str).expect("Failed to deserialize embedded German locale")
    }

    /// Returns a locale based on the given language code string.
    ///
    /// If the language code is unknown, empty, or set to English variations,
    /// this function defaults to English.
    pub fn from_lang_code(code: &str) -> Self {
        let normalized = code.trim().to_lowercase();
        match normalized.as_str() {
            "de" | "de-de" | "de_de" | "german" | "deutsch" => Self::german(),
            _ => Self::english(),
        }
    }
}

impl Default for Locale {
    fn default() -> Self {
        Self::english()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_from_lang_code() {
        assert_eq!(Locale::from_lang_code("de").lang_code, "de");
        assert_eq!(Locale::from_lang_code("DE-DE").lang_code, "de");
        assert_eq!(Locale::from_lang_code("german").lang_code, "de");
        assert_eq!(Locale::from_lang_code("en").lang_code, "en");
        assert_eq!(Locale::from_lang_code("").lang_code, "en");
        assert_eq!(Locale::from_lang_code("unknown").lang_code, "en");
    }
}
