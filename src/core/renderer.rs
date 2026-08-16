//! HTML AST renderer and element formatting engine.

use std::fmt::Write as _;

use crate::core::document::{DocumentElement, DocumentParameters};
use crate::core::feature::Feature;
use crate::core::format::{format_inline_into, push_indent};

/// Document AST HTML renderer managing active features and element interception.
#[derive(Clone, Copy, Debug)]
pub struct HtmlRenderer<'a> {
    features: &'a [&'static dyn Feature],
}

impl<'a> HtmlRenderer<'a> {
    /// Creates a new HTML renderer configured with the specified slice of active features.
    #[must_use]
    pub const fn new(features: &'a [&'static dyn Feature]) -> Self {
        Self { features }
    }

    /// Creates an HTML renderer with all default features enabled.
    #[must_use]
    pub fn default_renderer() -> HtmlRenderer<'static> {
        HtmlRenderer::new(&crate::features::ALL_FEATURES)
    }

    /// Renders a single document element and its children into the output buffer.
    pub fn render_element(
        &self,
        element: &DocumentElement,
        indent: usize,
        depth: usize,
        parameters: &DocumentParameters,
        out: &mut String,
    ) {
        for feature in self.features {
            if feature.try_render_body(element, indent, depth, parameters, out, self) {
                return;
            }
        }

        self.render_fallback(element, indent, depth, parameters, out);
    }

    /// Renders a slice of document elements sequentially into the output buffer.
    pub fn render_children(
        &self,
        children: &[DocumentElement],
        indent: usize,
        depth: usize,
        parameters: &DocumentParameters,
        out: &mut String,
    ) {
        for child in children {
            self.render_element(child, indent, depth, parameters, out);
        }
    }

    /// Fallback rendering for elements not intercepted by any registered feature.
    fn render_fallback(
        &self,
        element: &DocumentElement,
        indent: usize,
        _depth: usize,
        parameters: &DocumentParameters,
        out: &mut String,
    ) {
        match element {
            DocumentElement::BlockDirective { children, .. } => {
                self.render_children(children, indent + 1, 0, parameters, out);
            }
            DocumentElement::Section {
                children,
                level,
                title,
            } => {
                push_indent(out, indent);
                out.push_str("<section class=\"section\" data-level=\"");
                let _ = write!(out, "{level}");
                out.push_str("\">\n");

                push_indent(out, indent + 1);
                let _ = writeln!(out, "<h{level}>{title}</h{level}>");

                push_indent(out, indent + 1);
                out.push_str("<div class=\"section-body\">\n");

                self.render_children(children, indent + 2, 0, parameters, out);

                push_indent(out, indent + 1);
                out.push_str("</div>\n");

                push_indent(out, indent);
                out.push_str("</section>\n");
            }
            DocumentElement::Text(text) => {
                push_indent(out, indent);
                out.push_str("<div class=\"item text-item\">\n");
                push_indent(out, indent + 1);
                out.push_str("<span class=\"text-content\">\n");
                for line in text.lines() {
                    push_indent(out, indent + 2);
                    format_inline_into(out, line);
                    out.push('\n');
                }
                push_indent(out, indent + 1);
                out.push_str("</span>\n");
                push_indent(out, indent);
                out.push_str("</div>\n");
            }
            DocumentElement::HorizontalRule => {
                push_indent(out, indent);
                out.push_str("<hr />\n");
            }
            _ => {}
        }
    }
}

/// Renders a document element and its children directly into an output buffer.
///
/// Recursively processes nested child elements and delegates to registered feature renderers.
pub fn render_element_into(
    element: &DocumentElement,
    indent: usize,
    parameters: &DocumentParameters,
    out: &mut String,
) {
    let renderer = HtmlRenderer::default_renderer();
    renderer.render_element(element, indent, 0, parameters, out);
}

/// Renders a document element and its children into an HTML string representation.
///
/// Recursively processes nested child elements and delegates to registered feature renderers.
///
/// # Examples
///
/// ```
/// use doc2flow::core::renderer::render_element;
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
    let mut out = String::with_capacity(512);
    render_element_into(element, indent, parameters, &mut out);
    out
}

#[cfg(test)]
mod tests {
    use super::*;

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
}