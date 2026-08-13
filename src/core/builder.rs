//! Document build and rendering module.

use crate::core::constants::{APP_VERSION, LICENSE_URL, REPOSITORY_URL};
use crate::core::document::{Document, DocumentElement};
use crate::core::document_json::document_to_json;
use crate::core::feature::DocumentFeature;
use crate::core::utils::format_iso8601_utc;
use crate::features::get_feature;

/// Embedded base HTML template.
pub const TEMPLATE_HTML: &str = include_str!("../../resources/templates/template.html");

/// Appends multiline text to a buffer, indenting every non-empty line by 4 spaces.
fn append_indented(out: &mut String, text: &str) {
    for line in text.lines() {
        if !line.is_empty() {
            out.push_str("    ");
            out.push_str(line);
        }
        out.push('\n');
    }
}

/// Assembles active CSS stylesheets from core and enabled features indented with 4 spaces.
///
/// Core CSS is always included first, followed by active features in canonical order.
///
/// # Examples
///
/// ```
/// use doc2flow::core::builder::assemble_styles;
/// use doc2flow::core::feature::DocumentFeature;
///
/// let features = DocumentFeature::default();
/// let css = assemble_styles(&features);
/// assert!(css.contains("    --bg-body:"));
/// ```
pub fn assemble_styles(features: &DocumentFeature) -> String {
    let mut out = String::with_capacity(12288);
    if let Some(feature) = get_feature("core") {
        if let Some(css) = feature.css() {
            append_indented(&mut out, css);
        }
    }

    for (name, is_active) in [
        ("bullet_list_item", features.bullet_list_item),
        ("check_box_item", features.check_box_item),
        ("code_block", features.code_block),
        ("image", features.image),
        ("ordered_list_item", features.ordered_list_item),
        ("shoutout", features.shoutout),
        ("table", features.table),
        ("unknown", features.unknown),
    ] {
        if is_active {
            if let Some(feature) = get_feature(name) {
                if let Some(css) = feature.css() {
                    append_indented(&mut out, css);
                }
            }
        }
    }

    out
}

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
    let created_at = format_iso8601_utc(std::time::SystemTime::now());
    let json_content = document_to_json(document);
    let text_element = DocumentElement::text(json_content);
    let html_content = match get_feature("core") {
        Some(feature) => feature.to_html(&text_element),
        None => String::new(),
    };
    let lang_code = document
        .parameters
        .get("language")
        .map(String::as_str)
        .filter(|s| !s.is_empty())
        .unwrap_or("en");
    let title = document
        .parameters
        .get("title")
        .map(String::as_str)
        .unwrap_or("");
    let features_str = features.to_features_string();
    let css_content = assemble_styles(features);

    TEMPLATE_HTML
        .replace("{{APP_VERSION}}", APP_VERSION)
        .replace("{{APP_VERSION_RAW}}", app_version_raw)
        .replace("{{REPOSITORY_URL}}", REPOSITORY_URL)
        .replace("{{LICENSE_URL}}", LICENSE_URL)
        .replace("{{CREATED_AT}}", &created_at)
        .replace("{{LANG_CODE}}", lang_code)
        .replace("{{TITLE}}", title)
        .replace("{{FEATURES}}", &features_str)
        .replace("{{CSS}}", &css_content)
        .replace("{{CONTENT}}", &html_content)
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
        assert!(content.contains("<p class=\"txt-default\">"));
        assert!(content.contains("\"parameters\":"));
        assert!(!content.contains("{{CONTENT}}"));
        assert!(!content.contains("{{APP_VERSION}}"));
        assert!(!content.contains("{{APP_VERSION_RAW}}"));
        assert!(!content.contains("{{REPOSITORY_URL}}"));
        assert!(!content.contains("{{LICENSE_URL}}"));
        assert!(!content.contains("{{CREATED_AT}}"));
        assert!(!content.contains("{{LANG_CODE}}"));
        assert!(!content.contains("{{FEATURES}}"));
        assert!(!content.contains("{{CSS}}"));
        assert!(!content.contains("{{TITLE}}"));
        assert!(content.contains("<title></title>"));
        assert!(content.contains("--bg-body:"));
        assert!(!content.contains("--unknown-bg:"));
    }

    #[test]
    fn test_assemble_styles_core_default() {
        let features = DocumentFeature::default();
        let css = assemble_styles(&features);
        assert!(css.contains("    --bg-body:"));
        assert!(css.contains("    .txt-default"));
        assert!(!css.contains("--unknown-bg:"));
        assert!(!css.contains(".unknown-default"));
    }

    #[test]
    fn test_assemble_styles_with_unknown_feature() {
        let mut features = DocumentFeature::default();
        features.unknown = true;
        let css = assemble_styles(&features);
        assert!(css.contains("    --bg-body:"));
        assert!(css.contains("    .txt-default"));
        assert!(css.contains("    --unknown-bg:"));
        assert!(css.contains("    .unknown-default"));
    }

    #[test]
    fn test_builder_build_includes_unknown_css_when_active() {
        let mut doc = Document::new();
        doc.push_body(crate::core::document::DocumentElement::unknown(
            "unrecognized",
        ));
        let features = DocumentFeature::from(&doc);
        let content = build(&doc, &features);
        assert!(content.contains("<meta name=\"features\" content=\"core, unknown\">"));
        assert!(content.contains("    --bg-body:"));
        assert!(content.contains("    --unknown-bg:"));
        assert!(content.contains("    .unknown-default"));
        assert!(!content.contains("{{CSS}}"));
    }

    #[test]
    fn test_builder_build_custom_title() {
        let mut doc = Document::new();
        doc.insert_parameter("title", "Custom Title");
        let features = DocumentFeature::default();
        let content = build(&doc, &features);
        assert!(content.contains("<title>Custom Title</title>"));
        assert!(!content.contains("{{TITLE}}"));
    }

    #[test]
    fn test_builder_build_empty_title() {
        let mut doc = Document::new();
        doc.insert_parameter("title", "");
        let features = DocumentFeature::default();
        let content = build(&doc, &features);
        assert!(content.contains("<title></title>"));
        assert!(!content.contains("{{TITLE}}"));
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
