//! Document HTML template builder and asset assembly engine.

use crate::core::constants::{APP_VERSION, LICENSE_URL, REPOSITORY_URL};
use crate::core::document::Document;
use crate::core::feature::{DocumentFeature, Feature};
use crate::core::format::append_indented;
use crate::core::language::get_language_json;
use crate::core::renderer::HtmlRenderer;
use crate::core::utils::format_iso8601_utc;
use crate::features::{CORE_FEATURE, get_active_features};

/// Embedded base HTML template.
pub const TEMPLATE_HTML: &str = include_str!("../../resources/templates/template.html");

/// Assembles active CSS stylesheets and JavaScript client scripts from core and enabled features.
///
/// Returns a tuple of `(css, javascript)` strings indented with 4 spaces.
///
/// # Examples
///
/// ```
/// use doc2flow::core::builder::assemble_assets;
/// use doc2flow::core::feature::DocumentFeature;
///
/// let features = DocumentFeature::default();
/// let (css, js) = assemble_assets(&features);
/// assert!(css.contains("    --bg-body:"));
/// assert!(js.contains("    window.d2f"));
/// ```
pub fn assemble_assets(features: &DocumentFeature) -> (String, String) {
    let mut css_out = String::with_capacity(12288);
    let mut js_out = String::with_capacity(4096);

    let mut active_buffer = [&CORE_FEATURE as &'static dyn Feature; 8];
    let active_features = get_active_features(features, &mut active_buffer);

    for feature in active_features {
        if let Some(css) = feature.css() {
            append_indented(&mut css_out, css);
        }
        for js in feature.javascript() {
            append_indented(&mut js_out, js);
        }
    }

    (css_out, js_out)
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
    let lang_code = if document.parameters.language.is_empty() {
        "en"
    } else {
        &document.parameters.language
    };
    let title = &document.parameters.title;
    let features_str = features.to_string();
    let (css_content, js_content) = assemble_assets(features);
    let i18n_json = get_language_json(lang_code);

    let mut html_content = String::with_capacity(32768);
    let mut active_buffer = [&CORE_FEATURE as &'static dyn Feature; 8];
    let active_features = get_active_features(features, &mut active_buffer);
    let renderer = HtmlRenderer::new(active_features);

    if let Some(ref variables) = document.header.variables {
        renderer.render_element(variables, 2, 0, &document.parameters, &mut html_content);
    }
    for element in &document.body {
        renderer.render_element(element, 2, 0, &document.parameters, &mut html_content);
    }

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
        .replace("{{JS}}", &js_content)
        .replace("{{I18N_JSON}}", i18n_json)
        .replace("{{CONTENT}}", &html_content)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::document::DocumentElement;

    #[test]
    fn test_builder_build_as_ref_u8() {
        let mut doc = Document::new();
        doc.push_body(DocumentElement::text("Document body text"));
        let features = DocumentFeature::default();
        let content = build(&doc, &features);
        assert!(!content.is_empty());
        assert!(content.contains("<!DOCTYPE html>"));
        assert!(content.contains(APP_VERSION));
        assert!(content.contains(REPOSITORY_URL));
        assert!(content.contains(LICENSE_URL));
        assert!(content.contains("<html lang=\"en\">"));
        assert!(content.contains("<meta name=\"features\" content=\"core\">"));
        assert!(
            content.contains("    <div class=\"item text-item\">\n      <span class=\"text-content\">\n        Document body text\n      </span>\n    </div>")
        );
        assert!(!content.contains("{{CONTENT}}"));
        assert!(!content.contains("{{APP_VERSION}}"));
        assert!(!content.contains("{{APP_VERSION_RAW}}"));
        assert!(!content.contains("{{REPOSITORY_URL}}"));
        assert!(!content.contains("{{LICENSE_URL}}"));
        assert!(!content.contains("{{CREATED_AT}}"));
        assert!(!content.contains("{{LANG_CODE}}"));
        assert!(!content.contains("{{FEATURES}}"));
        assert!(!content.contains("{{CSS}}"));
        assert!(!content.contains("{{JS}}"));
        assert!(!content.contains("{{I18N_JSON}}"));
        assert!(!content.contains("{{TITLE}}"));
        assert!(content.contains("<title></title>"));
        assert!(content.contains("--bg-body:"));
        assert!(content.contains("window.d2f"));
        assert!(content.contains("window.d2f.lang.dictionary = {};"));
        assert!(!content.contains("--unknown-bg:"));
    }

    #[test]
    fn test_assemble_assets_core_default() {
        let features = DocumentFeature::default();
        let (css, js) = assemble_assets(&features);
        assert!(css.contains("    --bg-body:"));
        assert!(css.contains("    .txt-default"));
        assert!(js.contains("    window.d2f"));
    }

    #[test]
    fn test_assemble_scripts_core_default() {
        let features = DocumentFeature::default();
        let (_, js) = assemble_assets(&features);
        assert!(js.contains("    window.d2f"));
        assert!(js.contains("utils"));
        assert!(js.contains("storage"));
    }

    #[test]
    fn test_assemble_styles_core_default() {
        let features = DocumentFeature::default();
        let (css, _) = assemble_assets(&features);
        assert!(css.contains("    --bg-body:"));
        assert!(css.contains("    .txt-default"));
        assert!(!css.contains("--unknown-bg:"));
        assert!(!css.contains(".unknown-default"));
    }

    #[test]
    fn test_assemble_styles_with_unknown_feature() {
        let features = DocumentFeature::UNKNOWN;
        let (css, _) = assemble_assets(&features);
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
        assert!(
            content.contains("    <p class=\"unknown-default\">\n      unrecognized\n    </p>")
        );
        assert!(content.contains("    --bg-body:"));
        assert!(content.contains("    --unknown-bg:"));
        assert!(content.contains("    .unknown-default"));
        assert!(!content.contains("{{CSS}}"));
        assert!(!content.contains("{{JS}}"));
    }

    #[test]
    fn test_builder_build_custom_title() {
        let mut doc = Document::new();
        doc.parameters.title = "Custom Title".into();
        let features = DocumentFeature::default();
        let content = build(&doc, &features);
        assert!(content.contains("<title>Custom Title</title>"));
        assert!(!content.contains("{{TITLE}}"));
    }

    #[test]
    fn test_builder_build_empty_title() {
        let doc = Document::new();
        let features = DocumentFeature::default();
        let content = build(&doc, &features);
        assert!(content.contains("<title></title>"));
        assert!(!content.contains("{{TITLE}}"));
    }

    #[test]
    fn test_builder_build_custom_language() {
        let mut doc = Document::new();
        doc.parameters.language = "de".into();
        let features = DocumentFeature::default();
        let content = build(&doc, &features);
        assert!(content.contains("<html lang=\"de\">"));
        assert!(content.contains("window.d2f.lang.dictionary = {};"));
        assert!(!content.contains("{{LANG_CODE}}"));
        assert!(!content.contains("{{I18N_JSON}}"));
    }

    #[test]
    fn test_builder_build_empty_language_fallback() {
        let mut doc = Document::new();
        doc.parameters.language.clear();
        let features = DocumentFeature::default();
        let content = build(&doc, &features);
        assert!(content.contains("<html lang=\"en\">"));
        assert!(!content.contains("{{LANG_CODE}}"));
    }

    #[test]
    fn test_builder_build_with_features() {
        let mut doc = Document::new();
        doc.push_body(DocumentElement::code_block(
            None::<String>,
            "test code",
        ));
        let features = DocumentFeature::from(&doc);
        let content = build(&doc, &features);
        assert!(content.contains("<meta name=\"features\" content=\"core, code\">"));
        assert!(!content.contains("{{FEATURES}}"));
    }

    #[test]
    fn test_assemble_styles_with_code_feature() {
        let features = DocumentFeature::CODE;
        let (css, _) = assemble_assets(&features);
        assert!(css.contains("    --bg-body:"));
        assert!(css.contains("    --code-bg:"));
        assert!(css.contains("    .code-default"));
    }

    #[test]
    fn test_builder_build_with_header_variables() {
        let mut doc = Document::new();
        doc.header.variables = Some(DocumentElement::table(
            vec![
                crate::core::document::TableAlignment::None,
                crate::core::document::TableAlignment::None,
            ],
            vec![
                vec!["Variable".into(), "Value".into()],
                vec!["PORT".into(), "8080".into()],
            ],
        ));
        doc.push_body(DocumentElement::text("Body text"));
        let features = DocumentFeature::from(&doc);
        let content = build(&doc, &features);
        assert!(content.contains("<div class=\"table-wrap\">"));
        assert!(content.contains("<th>Variable</th>"));
        assert!(content.contains("<td>8080</td>"));
        assert!(content.contains("Body text"));
        assert!(features.contains(DocumentFeature::CODE));
        assert!(features.contains(DocumentFeature::TABLE));
    }
}
