//! Document build and rendering module.

use crate::core::constants::{APP_VERSION, LICENSE_URL, REPOSITORY_URL};
use crate::core::document::{Document, DocumentElement, DocumentParameters};
use crate::core::feature::DocumentFeature;
use crate::core::language::get_language_json;
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

    if let Some(feature) = get_feature("core") {
        if let Some(css) = feature.css() {
            append_indented(&mut css_out, css);
        }
        for js in feature.javascript() {
            append_indented(&mut js_out, js);
        }
    }

    for (name, is_active) in [
        ("bullet", features.bullet),
        ("code", features.code),
        ("image", features.image),
        ("ordered", features.ordered),
        ("shoutout", features.shoutout),
        ("table", features.table),
        ("task", features.task),
        ("unknown", features.unknown),
    ] {
        if is_active {
            if let Some(feature) = get_feature(name) {
                if let Some(css) = feature.css() {
                    append_indented(&mut css_out, css);
                }
                for js in feature.javascript() {
                    append_indented(&mut js_out, js);
                }
            }
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
    let features_str = features.to_features_string();
    let (css_content, js_content) = assemble_assets(features);
    let i18n_json = get_language_json(lang_code);

    let mut html_content = String::new();
    if let Some(ref variables) = document.header.variables {
        html_content.push_str(&render_element(variables, 2, &document.parameters));
    }
    for element in &document.body {
        html_content.push_str(&render_element(element, 2, &document.parameters));
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

/// Renders a document element and its children into an HTML string representation.
///
/// Recursively processes nested child elements and delegates to registered feature renderers.
///
/// # Examples
///
/// ```
/// use doc2flow::core::builder::render_element;
/// use doc2flow::core::document::{DocumentElement, DocumentParameters};
///
/// let element = DocumentElement::text("Hello world");
/// let params = DocumentParameters::default();
/// let html = render_element(&element, 1, &params);
/// assert_eq!(html, "  <div class=\"item text-item\">\n    <span class=\"text-content\">\n      Hello world\n    </span>\n  </div>\n");
/// ```
pub fn render_element(
    element: &DocumentElement,
    indent: usize,
    parameters: &DocumentParameters,
) -> String {
    render_element_with_depth(element, indent, 0, parameters)
}

/// Renders a document element and its children with an explicit list nesting depth.
fn render_element_with_depth(
    element: &DocumentElement,
    indent: usize,
    depth: usize,
    parameters: &DocumentParameters,
) -> String {
    let (feature_name, inner_content) = match element {
        DocumentElement::BlockDirective { children, name } => {
            let mut inner = String::new();
            for child in children {
                inner.push_str(&render_element_with_depth(child, indent + 1, 0, parameters));
            }
            (name.as_str(), inner)
        }
        DocumentElement::BulletListItem { children, .. } => {
            let mut inner = String::new();
            for child in children {
                inner.push_str(&render_element_with_depth(
                    child,
                    indent,
                    depth + 1,
                    parameters,
                ));
            }
            ("bullet", inner)
        }
        DocumentElement::CheckBoxItem { children, .. } => {
            let mut inner = String::new();
            for child in children {
                inner.push_str(&render_element_with_depth(
                    child,
                    indent,
                    depth + 1,
                    parameters,
                ));
            }
            ("task", inner)
        }
        DocumentElement::CodeBlock { .. } => ("code", String::new()),
        DocumentElement::HorizontalRule => ("core", String::new()),
        DocumentElement::Image { .. } => ("image", String::new()),
        DocumentElement::OrderedListItem { children, .. } => {
            let mut inner = String::new();
            for child in children {
                inner.push_str(&render_element_with_depth(
                    child,
                    indent,
                    depth + 1,
                    parameters,
                ));
            }
            ("ordered", inner)
        }
        DocumentElement::Section { children, .. } => {
            let mut inner = String::new();
            for child in children {
                inner.push_str(&render_element_with_depth(child, indent + 2, 0, parameters));
            }
            ("core", inner)
        }
        DocumentElement::Shoutout { .. } => ("shoutout", String::new()),
        DocumentElement::Table { .. } => ("table", String::new()),
        DocumentElement::Text(_) => ("core", String::new()),
        DocumentElement::Unknown(_) => ("unknown", String::new()),
    };

    match get_feature(feature_name) {
        Some(feature) => feature.to_html(element, &inner_content, indent, depth, parameters),
        None => inner_content,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_builder_build_as_ref_u8() {
        let mut doc = Document::new();
        doc.push_body(DocumentElement::text("Document body text"));
        let features = DocumentFeature::default();
        let content = build(&doc, &features);
        assert!(!content.as_bytes().is_empty());
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
        let mut features = DocumentFeature::default();
        features.unknown = true;
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
        doc.push_body(crate::core::document::DocumentElement::code_block(
            None::<String>,
            "test code",
        ));
        let features = DocumentFeature::from(&doc);
        let content = build(&doc, &features);
        assert!(content.contains("<meta name=\"features\" content=\"core, code\">"));
        assert!(!content.contains("{{FEATURES}}"));
    }

    #[test]
    fn test_render_element_text() {
        let text = DocumentElement::text("Sample paragraph text");
        assert_eq!(
            render_element(&text, 1, &DocumentParameters::default()),
            "  <div class=\"item text-item\">\n    <span class=\"text-content\">\n      Sample paragraph text\n    </span>\n  </div>\n"
        );
    }

    #[test]
    fn test_render_element_unknown() {
        let unknown = DocumentElement::unknown("Unrecognized markdown");
        assert_eq!(
            render_element(&unknown, 1, &DocumentParameters::default()),
            "  <p class=\"unknown-default\">\n    Unrecognized markdown\n  </p>\n"
        );
    }

    #[test]
    fn test_render_element_section_with_children() {
        let section = DocumentElement::section(
            2,
            "Details",
            vec![
                DocumentElement::text("First paragraph"),
                DocumentElement::text("Second paragraph"),
            ],
        );
        let expected = concat!(
            "  <section class=\"section\" data-level=\"2\">\n",
            "    <h2>Details</h2>\n",
            "    <div class=\"section-body\">\n",
            "      <div class=\"item text-item\">\n",
            "        <span class=\"text-content\">\n",
            "          First paragraph\n",
            "        </span>\n",
            "      </div>\n",
            "      <div class=\"item text-item\">\n",
            "        <span class=\"text-content\">\n",
            "          Second paragraph\n",
            "        </span>\n",
            "      </div>\n",
            "    </div>\n",
            "  </section>\n"
        );
        assert_eq!(
            render_element(&section, 1, &DocumentParameters::default()),
            expected
        );
    }

    #[test]
    fn test_render_element_nested_sections() {
        let inner_section =
            DocumentElement::section(3, "Inner", vec![DocumentElement::text("Inner content")]);
        let outer_section = DocumentElement::section(1, "Outer", vec![inner_section]);
        let expected = concat!(
            "  <section class=\"section\" data-level=\"1\">\n",
            "    <h1>Outer</h1>\n",
            "    <div class=\"section-body\">\n",
            "      <section class=\"section\" data-level=\"3\">\n",
            "        <h3>Inner</h3>\n",
            "        <div class=\"section-body\">\n",
            "          <div class=\"item text-item\">\n",
            "            <span class=\"text-content\">\n",
            "              Inner content\n",
            "            </span>\n",
            "          </div>\n",
            "        </div>\n",
            "      </section>\n",
            "    </div>\n",
            "  </section>\n"
        );
        assert_eq!(
            render_element(&outer_section, 1, &DocumentParameters::default()),
            expected
        );
    }

    #[test]
    fn test_assemble_styles_with_code_feature() {
        let mut features = DocumentFeature::default();
        features.code = true;
        let (css, _) = assemble_assets(&features);
        assert!(css.contains("    --bg-body:"));
        assert!(css.contains("    --code-bg:"));
        assert!(css.contains("    .code-default"));
    }

    #[test]
    fn test_render_element_code_block() {
        let code_block = DocumentElement::code_block(Some("rust"), "fn main() {}");
        assert_eq!(
            render_element(&code_block, 1, &DocumentParameters::default()),
            "  <pre class=\"code-default\"><code>fn main() {}</code></pre>\n"
        );
    }

    #[test]
    fn test_render_element_horizontal_rule() {
        let hr = DocumentElement::horizontal_rule();
        assert_eq!(
            render_element(&hr, 1, &DocumentParameters::default()),
            "  <hr />\n"
        );
        assert_eq!(
            render_element(&hr, 2, &DocumentParameters::default()),
            "    <hr />\n"
        );
    }

    #[test]
    fn test_render_element_bullet_list_item() {
        let mut bullet = DocumentElement::bullet_list_item("Parent bullet");
        let child = DocumentElement::bullet_list_item("Child bullet");
        bullet.push_child(child).unwrap();

        let html = render_element(&bullet, 1, &DocumentParameters::default());
        assert!(html.contains(
            "<div class=\"item bullet-item\">\n    <span class=\"bullet-marker\">&bull;</span>"
        ));
        assert!(html.contains("<div class=\"item bullet-item\" style=\"--indent: 1;\">"));
        assert!(html.contains("Parent bullet"));
        assert!(html.contains("Child bullet"));
    }

    #[test]
    fn test_render_element_check_box_item() {
        let mut check = DocumentElement::check_box_item(true, "Done task");
        let child = DocumentElement::check_box_item(false, "Sub task");
        check.push_child(child).unwrap();

        let html = render_element(&check, 1, &DocumentParameters::default());
        assert!(html.contains("<div class=\"item check-item checked\">"));
        assert!(html.contains("<div class=\"item check-item\" style=\"--indent: 1;\">"));
        assert!(html.contains("<input type=\"checkbox\" class=\"check-box\" checked />"));
        assert!(html.contains("<input type=\"checkbox\" class=\"check-box\" />"));
        assert!(html.contains("Done task"));
        assert!(html.contains("Sub task"));
    }

    #[test]
    fn test_render_element_ordered_list_item() {
        let mut order = DocumentElement::ordered_list_item(1, "First step");
        let child = DocumentElement::ordered_list_item(1, "Sub step");
        order.push_child(child).unwrap();

        let html = render_element(&order, 1, &DocumentParameters::default());
        assert!(html.contains(
            "<div class=\"item order-item\">\n    <span class=\"order-marker\">1.</span>"
        ));
        assert!(html.contains("<div class=\"item order-item\" style=\"--indent: 1;\">"));
        assert!(html.contains("First step"));
        assert!(html.contains("Sub step"));
    }

    #[test]
    fn test_render_element_image() {
        let image = DocumentElement::image("alt text", "test.png");
        assert_eq!(
            render_element(&image, 1, &DocumentParameters::default()),
            "  <div class=\"image-item\">\n    <img src=\"test.png\" alt=\"alt text\" />\n  </div>\n"
        );
    }

    #[test]
    fn test_render_element_unregistered_feature_fallback() {
        let shoutout = DocumentElement::shoutout(
            crate::core::document::ShoutoutElementKind::Note,
            "Unregistered shoutout",
        );
        assert_eq!(
            render_element(&shoutout, 1, &DocumentParameters::default()),
            ""
        );

        let directive = DocumentElement::block_directive(
            "unregistered_directive",
            vec![DocumentElement::text("Directive child")],
        );
        assert_eq!(
            render_element(&directive, 1, &DocumentParameters::default()),
            "    <div class=\"item text-item\">\n      <span class=\"text-content\">\n        Directive child\n      </span>\n    </div>\n"
        );
    }

    #[test]
    fn test_builder_build_with_header_variables() {
        let mut doc = Document::new();
        doc.header.variables = Some(DocumentElement::table(
            vec![crate::core::document::TableAlignment::None, crate::core::document::TableAlignment::None],
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
        assert!(features.code);
        assert!(features.table);
    }
}
