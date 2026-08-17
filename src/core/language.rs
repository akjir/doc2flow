//! Language dictionary and internationalization module.

/// German locale JSON dictionary embedded at compile time.
pub const DE_JSON: &str = include_str!("../../resources/locales/de.json");

/// English locale JSON dictionary embedded at compile time.
pub const EN_JSON: &str = include_str!("../../resources/locales/en.json");

/// Fallback empty JSON dictionary for unknown language codes.
pub const EMPTY_JSON: &str = "{}";

/// Returns the serialized JSON dictionary string for the specified language code.
///
/// Returns an empty JSON object (`"{}"`) if the language code is unrecognized.
///
/// # Examples
///
/// ```
/// use doc2flow::core::language::get_language_json;
///
/// assert!(get_language_json("en").contains("\"lang_code\": \"en\""));
/// assert!(get_language_json("de").contains("\"lang_code\": \"de\""));
/// assert_eq!(get_language_json("fr"), "{}");
/// assert_eq!(get_language_json(""), "{}");
/// ```
pub fn get_language_json(code: &str) -> &'static str {
    match code.trim() {
        c if c.eq_ignore_ascii_case("de") => DE_JSON,
        c if c.eq_ignore_ascii_case("en") => EN_JSON,
        _ => EMPTY_JSON,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_language_json_returns_embedded_dictionaries() {
        let en = get_language_json("en");
        assert!(en.contains("\"lang_code\": \"en\""));
        assert!(en.contains("\"export_pdf\": \"Export as PDF\""));

        let de = get_language_json("de");
        assert!(de.contains("\"lang_code\": \"de\""));
        assert!(de.contains("\"export_pdf\": \"Als PDF exportieren\""));
    }

    #[test]
    fn test_get_language_json_case_insensitive_and_whitespace() {
        assert_eq!(get_language_json("EN"), EN_JSON);
        assert_eq!(get_language_json("DE"), DE_JSON);
        assert_eq!(get_language_json("  en  "), EN_JSON);
        assert_eq!(get_language_json("  de  "), DE_JSON);
    }

    #[test]
    fn test_get_language_json_returns_empty_map_for_unknown() {
        assert_eq!(get_language_json("fr"), "{}");
        assert_eq!(get_language_json("es"), "{}");
        assert_eq!(get_language_json("xyz"), "{}");
        assert_eq!(get_language_json(""), "{}");
        assert_eq!(get_language_json("   "), "{}");
    }
}
