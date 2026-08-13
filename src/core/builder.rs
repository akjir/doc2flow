//! Document build and rendering module.

use crate::core::constants::{APP_VERSION, LICENSE_URL, REPOSITORY_URL};
use crate::core::document::Document;
use crate::core::document_json::document_to_json;
use crate::core::feature::DocumentFeature;

/// Embedded base HTML template.
pub const TEMPLATE_HTML: &str = include_str!("../../resources/templates/template.html");

/// Builds output content from a structured [`Document`] and active [`DocumentFeature`] flags.
///
/// Returns a formatted string implementing `AsRef<[u8]>`.
///
/// # Examples
///
/// ```
/// use doc2flow::core::builder::build;
/// use doc2flow::core::document::Document;
/// use doc2flow::core::feature::DocumentFeature;
///
/// let doc = Document::new();
/// let features = DocumentFeature::default();
/// let content = build(&doc, &features);
/// assert!(!content.is_empty());
/// ```
pub fn build(document: &Document, features: &DocumentFeature) -> String {
    let app_version_raw = APP_VERSION.strip_prefix('v').unwrap_or(APP_VERSION);
    let json_content = document_to_json(document);
    let lang_code = document
        .parameters
        .get("language")
        .map(String::as_str)
        .filter(|s| !s.is_empty())
        .unwrap_or("en");
    let features_str = features.to_features_string();

    TEMPLATE_HTML
        .replace("{{APP_VERSION}}", APP_VERSION)
        .replace("{{APP_VERSION_RAW}}", app_version_raw)
        .replace("{{REPOSITORY_URL}}", REPOSITORY_URL)
        .replace("{{LICENSE_URL}}", LICENSE_URL)
        .replace("{{LANG_CODE}}", lang_code)
        .replace("{{FEATURES}}", &features_str)
        .replace("{{CONTENT}}", &json_content)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_builder_build_as_ref_u8() {
        let doc = Document::new();
        let features = DocumentFeature::default();
        let content = build(&doc, &features);
        assert!(!content.as_bytes().is_empty());
        assert!(content.contains("<!DOCTYPE html>"));
        assert!(content.contains(APP_VERSION));
        assert!(content.contains(REPOSITORY_URL));
        assert!(content.contains(LICENSE_URL));
        assert!(content.contains("<html lang=\"en\">"));
        assert!(content.contains("<meta name=\"features\" content=\"core\">"));
        assert!(content.contains("\"parameters\":"));
        assert!(!content.contains("{{CONTENT}}"));
        assert!(!content.contains("{{APP_VERSION}}"));
        assert!(!content.contains("{{APP_VERSION_RAW}}"));
        assert!(!content.contains("{{REPOSITORY_URL}}"));
        assert!(!content.contains("{{LICENSE_URL}}"));
        assert!(!content.contains("{{LANG_CODE}}"));
        assert!(!content.contains("{{FEATURES}}"));
    }

    #[test]
    fn test_builder_build_custom_language() {
        let mut doc = Document::new();
        doc.insert_parameter("language", "de");
        let features = DocumentFeature::default();
        let content = build(&doc, &features);
        assert!(content.contains("<html lang=\"de\">"));
        assert!(!content.contains("{{LANG_CODE}}"));
    }

    #[test]
    fn test_builder_build_empty_language_fallback() {
        let mut doc = Document::new();
        doc.insert_parameter("language", "");
        let features = DocumentFeature::default();
        let content = build(&doc, &features);
        assert!(content.contains("<html lang=\"en\">"));
        assert!(!content.contains("{{LANG_CODE}}"));
    }

    #[test]
    fn test_builder_build_with_features() {
        let mut doc = Document::new();
        doc.push_body(crate::core::document::DocumentElement::code_block(
            None::<String>,
            "test code",
        ));
        let features = DocumentFeature::from(&doc);
        let content = build(&doc, &features);
        assert!(content.contains("<meta name=\"features\" content=\"core, code_block\">"));
        assert!(!content.contains("{{FEATURES}}"));
    }
}
