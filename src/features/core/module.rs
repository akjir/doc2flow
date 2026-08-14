//! Core vertical slice feature module.

use crate::core::document::DocumentElement;
use crate::core::feature::Feature;

/// Embedded core CSS styles for layout and components.
pub const CSS: &str = include_str!("core.css");

/// Core feature renderer handling text and section elements.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct CoreFeature;

impl CoreFeature {
    /// Creates a new core feature instance.
    pub const fn new() -> Self {
        Self
    }
}

/// Appends leading whitespace indentation to a buffer based on the specified indent level.
fn push_indent(out: &mut String, indent: usize) {
    for _ in 0..indent {
        out.push_str("  ");
    }
}

impl Feature for CoreFeature {
    /// Converts a document element and inner content into an HTML string representation.
    fn to_html(&self, element: &DocumentElement, content: &str, indent: usize) -> String {
        match element {
            DocumentElement::Text(text) => {
                let spaces = indent * 2;
                let inner_spaces = (indent + 1) * 2;
                let mut out = String::with_capacity(text.len() + spaces * 2 + inner_spaces + 32);
                push_indent(&mut out, indent);
                out.push_str("<p class=\"txt-default\">\n");
                for line in text.lines() {
                    push_indent(&mut out, indent + 1);
                    out.push_str(line);
                    out.push('\n');
                }
                push_indent(&mut out, indent);
                out.push_str("</p>\n");
                out
            }
            DocumentElement::Section {
                level,
                title,
                ..
            } => {
                let spaces = indent * 2;
                let inner_spaces = (indent + 1) * 2;
                let mut out = String::with_capacity(
                    title.len() + content.len() + spaces * 2 + inner_spaces * 3 + 128,
                );
                push_indent(&mut out, indent);
                out.push_str("<section class=\"section\" data-level=\"");
                out.push_str(&level.to_string());
                out.push_str("\">\n");

                push_indent(&mut out, indent + 1);
                out.push_str("<h");
                out.push_str(&level.to_string());
                out.push('>');
                out.push_str(title);
                out.push_str("</h");
                out.push_str(&level.to_string());
                out.push_str(">\n");

                push_indent(&mut out, indent + 1);
                out.push_str("<div class=\"section-body\">\n");

                out.push_str(content);

                push_indent(&mut out, indent + 1);
                out.push_str("</div>\n");

                push_indent(&mut out, indent);
                out.push_str("</section>\n");
                out
            }
            _ => String::new(),
        }
    }

    /// Returns the embedded CSS stylesheet for the core feature.
    fn css(&self) -> Option<&'static str> {
        Some(CSS)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_core_feature_css() {
        let feature = CoreFeature::new();
        let css = feature.css().expect("core css should exist");
        assert!(css.contains("--bg-body:"));
        assert!(css.contains(".txt-default"));
        assert!(css.contains(".section"));
        assert!(css.contains(".section-header"));
        assert!(css.contains(".section-body"));
        assert!(css.contains(".section-subheading"));
        assert!(css.contains("--section-bg-header:"));
    }

    #[test]
    fn test_core_feature_renders_text() {
        let feature = CoreFeature::new();
        let element = DocumentElement::text("Hello, world!");
        assert_eq!(
            feature.to_html(&element, "", 1),
            "  <p class=\"txt-default\">\n    Hello, world!\n  </p>\n"
        );
    }

    #[test]
    fn test_core_feature_renders_section_with_content() {
        let feature = CoreFeature::new();
        let section = DocumentElement::section(
            1,
            "Overview",
            vec![DocumentElement::text("Section body content")],
        );
        let child_html =
            "        <p class=\"txt-default\">\n          Section body content\n        </p>\n";
        let html = feature.to_html(&section, child_html, 2);
        let expected = concat!(
            "    <section class=\"section\" data-level=\"1\">\n",
            "      <h1>Overview</h1>\n",
            "      <div class=\"section-body\">\n",
            "        <p class=\"txt-default\">\n",
            "          Section body content\n",
            "        </p>\n",
            "      </div>\n",
            "    </section>\n"
        );
        assert_eq!(html, expected);
    }

    #[test]
    fn test_core_feature_empty_for_unsupported_elements() {
        let feature = CoreFeature::new();
        let code = DocumentElement::code_block(Some("rust"), "fn main() {}");
        assert_eq!(feature.to_html(&code, "", 0), "");
    }
}
