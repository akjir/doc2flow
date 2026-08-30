//! Document HTML template builder and asset assembly engine.

use crate::core::constants::{APP_VERSION, LICENSE_URL, REPOSITORY_URL};
use crate::core::document::Document;
use crate::core::feature::FeatureModule;
use crate::core::format::append_indented;
use crate::core::renderer::{HtmlRenderer, MAX_ACTIVE_FEATURES};
use crate::utils::format_iso8601_utc;

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
/// use doc2flow::features::CORE_FEATURE;
///
/// let (css, js) = assemble_assets(&[&CORE_FEATURE]);
/// assert!(css.contains("    --bg-body:"));
/// assert!(js.contains("    window.d2f"));
/// ```
#[must_use]
pub fn assemble_assets(active_modules: &[&'static dyn FeatureModule]) -> (String, String) {
    let mut css_out = String::with_capacity(12288);
    let mut js_out = String::with_capacity(4096);

    for module in active_modules {
        if let Some(css) = module.css() {
            append_indented(&mut css_out, css);
        }
        for js in module.javascript() {
            append_indented(&mut js_out, js);
        }
    }

    (css_out, js_out)
}

/// Formats a comma-separated list of active feature names.
fn format_features_str(active_modules: &[&'static dyn FeatureModule]) -> String {
    let mut s = String::with_capacity(64);
    for (i, module) in active_modules.iter().enumerate() {
        if i > 0 {
            s.push_str(", ");
        }
        s.push_str(module.name());
    }
    s
}

/// Builds output content from a structured [`Document`].
///
/// Returns a formatted string implementing `AsRef<[u8]>`.
///
/// # Examples
///
/// ```
/// use doc2flow::core::builder::build;
/// use doc2flow::core::document::Document;
///
/// let doc = Document::new();
/// let content = build(&doc);
/// assert!(!content.is_empty());
/// ```
#[must_use]
pub fn build(document: &Document) -> String {
    crate::core::language::init(&document.parameters.language);
    let app_version_raw = APP_VERSION.strip_prefix('v').unwrap_or(APP_VERSION);
    let created_at = format_iso8601_utc(std::time::SystemTime::now());
    let document_id = crate::core::id::generate_document_id(&document.parameters);
    let lang_code = if document.parameters.language.is_empty() {
        "en"
    } else {
        &document.parameters.language
    };
    let title = &document.parameters.title;

    let mut html_content = String::with_capacity(32768);
    let renderer = HtmlRenderer::default_renderer();

    if let Some(ref header) = document.head.header {
        renderer.render_element(header, 2, 0, &document.parameters, &mut html_content);
    }
    if let Some(ref variables) = document.head.variables {
        renderer.render_element(variables, 2, 0, &document.parameters, &mut html_content);
    }
    for element in &document.body {
        renderer.render_element(element, 2, 0, &document.parameters, &mut html_content);
    }

    if document.parameters.comments {
        renderer.mark_feature_active("comment");
        renderer.mark_feature_active("input");
    }
    if document.head.variables.is_some() {
        renderer.mark_feature_active("input");
    }

    let export_pdf_label = crate::core::language::localize("export_pdf");
    let save_state_label = crate::core::language::localize("save_state");
    let reset_all_label = crate::core::language::localize("reset_all");
    let confirm_reset_msg = crate::core::language::localize("confirm_reset");

    let mut active_buffer =
        [&crate::features::CORE_FEATURE as &'static dyn FeatureModule; MAX_ACTIVE_FEATURES];
    let active_modules = renderer.active_features(&mut active_buffer);
    let features_str = format_features_str(active_modules);
    let (css_content, js_content) = assemble_assets(active_modules);

    TEMPLATE_HTML
        .replace("{{APP_VERSION}}", APP_VERSION)
        .replace("{{APP_VERSION_RAW}}", app_version_raw)
        .replace("{{REPOSITORY_URL}}", REPOSITORY_URL)
        .replace("{{LICENSE_URL}}", LICENSE_URL)
        .replace("{{CREATED_AT}}", &created_at)
        .replace("{{LANG_CODE}}", lang_code)
        .replace("{{LANGUAGE_CODE}}", lang_code)
        .replace("{{TITLE}}", title)
        .replace("{{DOCUMENT_ID}}", &document_id)
        .replace("{{FEATURES}}", &features_str)
        .replace("{{L_EXPORT_PDF}}", &export_pdf_label)
        .replace("{{L_SAVE_STATE}}", &save_state_label)
        .replace("{{L_RESET_ALL}}", &reset_all_label)
        .replace("{{L_CONFIRM_RESET}}", &confirm_reset_msg)
        .replace("{{CSS}}", &css_content)
        .replace("{{JS}}", &js_content)
        .replace("{{CONTENT}}", &html_content)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::document::DocumentElement;
    use crate::features::{CODE_FEATURE, CORE_FEATURE, UNKNOWN_FEATURE};

    #[test]
    fn test_builder_build_as_ref_u8() {
        let _guard = crate::core::language::TEST_I18N_LOCK
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        let mut doc = Document::new();
        doc.push_body(DocumentElement::text("Document body text"));
        let content = build(&doc);
        assert!(!content.is_empty());
        assert!(content.contains("<!DOCTYPE html>"));
        assert!(content.contains(APP_VERSION));
        assert!(content.contains(REPOSITORY_URL));
        assert!(content.contains(LICENSE_URL));
        assert!(content.contains("<html lang=\"en\">"));
        assert!(content.contains("<meta name=\"features\" content=\"core, comment, input\">"));
        assert!(
            content.contains("    <div class=\"item text-item item-selectable\">\n      <span class=\"text-content\">\n        Document body text\n      </span>\n      <span class=\"item-comment-icon\">")
        );
        assert!(!content.contains("{{CONTENT}}"));
        assert!(!content.contains("{{APP_VERSION}}"));
        assert!(!content.contains("{{APP_VERSION_RAW}}"));
        assert!(!content.contains("{{REPOSITORY_URL}}"));
        assert!(!content.contains("{{LICENSE_URL}}"));
        assert!(!content.contains("{{CREATED_AT}}"));
        assert!(!content.contains("{{LANG_CODE}}"));
        assert!(!content.contains("{{LANGUAGE_CODE}}"));
        assert!(!content.contains("{{DOCUMENT_ID}}"));
        assert!(!content.contains("{{FEATURES}}"));
        assert!(!content.contains("{{L_EXPORT_PDF}}"));
        assert!(!content.contains("{{L_SAVE_STATE}}"));
        assert!(!content.contains("{{L_RESET_ALL}}"));
        assert!(!content.contains("{{L_CONFIRM_RESET}}"));
        assert!(!content.contains("{{CSS}}"));
        assert!(!content.contains("{{JS}}"));
        assert!(!content.contains("{{TITLE}}"));
        assert!(content.contains("<title></title>"));
        assert!(content.contains("<div class=\"doc-body-buttons\">"));
        assert!(content.contains("<button type=\"button\" class=\"item-button-pdf\" id=\"item-button-pdf\">Export as PDF</button>"));
        assert!(content.contains("<button type=\"button\" class=\"item-button-save\" id=\"item-button-save\">Save State</button>"));
        assert!(content.contains("<button type=\"button\" class=\"item-button-reset\" id=\"item-button-reset\" data-confirm=\"Are you sure you want to reset all inputs and markings and expand all sections?\">Reset</button>"));
        assert!(content.contains("--bg-body:"));
        assert!(content.contains("window.d2f"));
        assert!(content.contains("window.d2f.document.id = 'd2f_id_"));
        assert!(content.contains("window.d2f.document.language = 'en';"));
        assert!(!content.contains("--unknown-bg:"));
    }

    #[test]
    fn test_builder_build_deterministic_document_id() {
        let _guard = crate::core::language::TEST_I18N_LOCK
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        let mut doc = Document::new();
        doc.parameters.title = "Release Notes".into();
        doc.parameters.version = "v1.0.0".into();
        doc.parameters.date = "2026-08-17".into();
        let expected_id = crate::core::id::generate_document_id(&doc.parameters);

        let content = build(&doc);
        assert!(content.contains(&format!("window.d2f.document.id = '{expected_id}';")));
    }

    #[test]
    fn test_assemble_assets_core_default() {
        let (css, js) = assemble_assets(&[&CORE_FEATURE]);
        assert!(css.contains("    --bg-body:"));
        assert!(css.contains("    .txt-default"));
        assert!(js.contains("    window.d2f"));
    }

    #[test]
    fn test_assemble_scripts_core_default() {
        let (_, js) = assemble_assets(&[&CORE_FEATURE]);
        assert!(js.contains("    window.d2f"));
        assert!(js.contains("utils"));
        assert!(js.contains("storage"));
    }

    #[test]
    fn test_assemble_styles_core_default() {
        let (css, _) = assemble_assets(&[&CORE_FEATURE]);
        assert!(css.contains("    --bg-body:"));
        assert!(css.contains("    .txt-default"));
        assert!(!css.contains("--unknown-bg:"));
        assert!(!css.contains(".unknown-default"));
    }

    #[test]
    fn test_assemble_styles_with_unknown_feature() {
        let (css, _) = assemble_assets(&[&CORE_FEATURE, &UNKNOWN_FEATURE]);
        assert!(css.contains("    --bg-body:"));
        assert!(css.contains("    .txt-default"));
        assert!(css.contains("    --unknown-bg:"));
        assert!(css.contains("    .unknown-default"));
    }

    #[test]
    fn test_builder_build_includes_unknown_css_when_active() {
        let _guard = crate::core::language::TEST_I18N_LOCK
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        let mut doc = Document::new();
        doc.parameters.comments = false;
        doc.push_body(crate::core::document::DocumentElement::unknown(
            "unrecognized",
        ));
        let content = build(&doc);
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
        let _guard = crate::core::language::TEST_I18N_LOCK
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        let mut doc = Document::new();
        doc.parameters.title = "Custom Title".into();
        let content = build(&doc);
        assert!(content.contains("<title>Custom Title</title>"));
        assert!(!content.contains("{{TITLE}}"));
    }

    #[test]
    fn test_builder_build_empty_title() {
        let _guard = crate::core::language::TEST_I18N_LOCK
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        let doc = Document::new();
        let content = build(&doc);
        assert!(content.contains("<title></title>"));
        assert!(!content.contains("{{TITLE}}"));
    }

    #[test]
    fn test_builder_build_custom_language() {
        let _guard = crate::core::language::TEST_I18N_LOCK
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        let mut doc = Document::new();
        doc.parameters.language = "de".into();
        doc.push_body(DocumentElement::shoutout(
            crate::core::document::ShoutoutElementKind::Note,
            "Hinweistext",
        ));
        let content = build(&doc);
        assert!(content.contains("<html lang=\"de\">"));
        assert!(content.contains("window.d2f.document.language = 'de';"));
        assert!(content.contains("data-label=\"Hinweis\""));
        assert!(content.contains("<button type=\"button\" class=\"item-button-pdf\" id=\"item-button-pdf\">Als PDF exportieren</button>"));
        assert!(content.contains("<button type=\"button\" class=\"item-button-save\" id=\"item-button-save\">Stand sichern</button>"));
        assert!(content.contains("<button type=\"button\" class=\"item-button-reset\" id=\"item-button-reset\" data-confirm=\"Sind Sie sicher, dass Sie alle Eingaben und Markierungen zurücksetzen und alle Abschnitte ausklappen möchten?\">Zurücksetzen</button>"));
        assert!(!content.contains("{{LANG_CODE}}"));
        assert!(!content.contains("{{LANGUAGE_CODE}}"));
        assert!(!content.contains("{{L_EXPORT_PDF}}"));
        assert!(!content.contains("{{L_SAVE_STATE}}"));
        assert!(!content.contains("{{L_RESET_ALL}}"));
        assert!(!content.contains("{{L_CONFIRM_RESET}}"));
    }

    #[test]
    fn test_builder_build_unknown_language() {
        let _guard = crate::core::language::TEST_I18N_LOCK
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        let mut doc = Document::new();
        doc.parameters.language = "fr".into();
        doc.push_body(DocumentElement::shoutout(
            crate::core::document::ShoutoutElementKind::Note,
            "Texte",
        ));
        let content = build(&doc);
        assert!(content.contains("<html lang=\"fr\">"));
        assert!(content.contains("window.d2f.document.language = 'fr';"));
        assert!(content.contains("data-label=\"{{callout_note}}\""));
        assert!(content.contains("<button type=\"button\" class=\"item-button-pdf\" id=\"item-button-pdf\">{{export_pdf}}</button>"));
        assert!(content.contains("<button type=\"button\" class=\"item-button-save\" id=\"item-button-save\">{{save_state}}</button>"));
        assert!(content.contains("<button type=\"button\" class=\"item-button-reset\" id=\"item-button-reset\" data-confirm=\"{{confirm_reset}}\">{{reset_all}}</button>"));
    }

    #[test]
    fn test_builder_build_empty_language_fallback() {
        let _guard = crate::core::language::TEST_I18N_LOCK
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        let mut doc = Document::new();
        doc.parameters.language.clear();
        let content = build(&doc);
        assert!(content.contains("<html lang=\"en\">"));
        assert!(content.contains("window.d2f.document.language = 'en';"));
        assert!(!content.contains("{{LANG_CODE}}"));
        assert!(!content.contains("{{LANGUAGE_CODE}}"));
    }

    #[test]
    fn test_builder_build_with_features() {
        let _guard = crate::core::language::TEST_I18N_LOCK
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        let mut doc = Document::new();
        doc.parameters.comments = false;
        doc.push_body(DocumentElement::code_block(None::<String>, "test code"));
        let content = build(&doc);
        assert!(content.contains("<meta name=\"features\" content=\"core, code\">"));
        assert!(!content.contains("{{FEATURES}}"));
    }

    #[test]
    fn test_assemble_styles_with_code_feature() {
        let (css, _) = assemble_assets(&[&CORE_FEATURE, &CODE_FEATURE]);
        assert!(css.contains("    --bg-body:"));
        assert!(css.contains("    --code-bg:"));
        assert!(css.contains("    .code-default"));
    }

    #[test]
    fn test_builder_build_with_header_variables() {
        let _guard = crate::core::language::TEST_I18N_LOCK
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        let mut doc = Document::new();
        doc.parameters.comments = false;
        let mut vars = std::collections::HashMap::new();
        vars.insert("PORT".into(), "8080".into());
        doc.head.variables = Some(DocumentElement::table_variables(vars));
        doc.push_body(DocumentElement::text("Body text"));
        let content = build(&doc);
        assert!(content.contains("code-table-wrap"));
        assert!(content.contains("value=\"8080\""));
        assert!(content.contains("Body text"));
        assert!(content.contains("<meta name=\"features\" content=\"core, code, input\">"));
    }

    #[test]
    fn test_builder_build_with_header_element() {
        let _guard = crate::core::language::TEST_I18N_LOCK
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        let mut doc = Document::new();
        doc.parameters.comments = false;
        doc.head.header = Some(DocumentElement::header(
            "Document Header Title",
            Some("Header Subtitle"),
            None::<String>,
        ));
        doc.push_body(DocumentElement::text("Document body text"));
        let content = build(&doc);
        assert!(content.contains("<meta name=\"features\" content=\"core, header\">"));
        assert!(content.contains("<section class=\"section header-container\" id=\"header\">"));
        assert!(content.contains("<h1 class=\"header-title\">Document Header Title</h1>"));
        assert!(content.contains("<div class=\"header-sub\">Header Subtitle</div>"));
        assert!(content.contains("--header-bg:"));
    }

    #[test]
    fn test_builder_build_with_comments_disabled() {
        let _guard = crate::core::language::TEST_I18N_LOCK
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        let mut doc = Document::new();
        doc.parameters.comments = false;
        doc.push_body(DocumentElement::text("Plain body text"));
        let content = build(&doc);
        assert!(content.contains("<meta name=\"features\" content=\"core\">"));
        assert!(!content.contains("<span class=\"item-comment-icon\">"));
        assert!(!content.contains("window.d2f.comments"));
        assert!(!content.contains("window.d2f.input"));
    }

    #[test]
    fn test_builder_build_header_before_variables_and_body() {
        let _guard = crate::core::language::TEST_I18N_LOCK
            .lock()
            .unwrap_or_else(|e| e.into_inner());
        let mut doc = Document::new();
        doc.head.header = Some(DocumentElement::header(
            "Header First",
            None::<String>,
            None::<String>,
        ));
        let mut vars = std::collections::HashMap::new();
        vars.insert("K".into(), "V".into());
        doc.head.variables = Some(DocumentElement::table_variables(vars));
        doc.push_body(DocumentElement::text("Body Last"));

        let content = build(&doc);
        let header_pos = content.find("header-container").unwrap();
        let vars_pos = content.find("code-table-wrap").unwrap();
        let body_pos = content.find("Body Last").unwrap();

        assert!(header_pos < vars_pos, "header must precede variables table");
        assert!(vars_pos < body_pos, "variables table must precede body");
    }
}
