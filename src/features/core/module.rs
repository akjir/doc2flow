//! Core vertical slice feature module.

use crate::core::document::DocumentElement;
use crate::core::feature::Feature;

/// Core feature renderer handling text and section elements.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct CoreFeature;

impl CoreFeature {
    /// Creates a new core feature instance.
    pub const fn new() -> Self {
        Self
    }
}

impl Feature for CoreFeature {
    /// Converts a document element into an HTML string representation.
    fn to_html(&self, element: &DocumentElement) -> String {
        match element {
            DocumentElement::Text(text) => {
                let mut out = String::with_capacity(text.len() + 32);
                out.push_str("<p class=\"txt-default\">");
                out.push_str(text);
                out.push_str("</p>");
                out
            }
            DocumentElement::Section {
                children,
                level,
                title,
            } => {
                let mut out = String::with_capacity(title.len() + 128);
                out.push_str("<section class=\"section\" data-level=\"");
                out.push_str(&level.to_string());
                out.push_str("\"><h");
                out.push_str(&level.to_string());
                out.push('>');
                out.push_str(title);
                out.push_str("</h");
                out.push_str(&level.to_string());
                out.push_str("><div class=\"section-body\">");
                for child in children {
                    out.push_str(&self.to_html(child));
                }
                out.push_str("</div></section>");
                out
            }
            _ => String::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_core_feature_renders_text() {
        let feature = CoreFeature::new();
        let element = DocumentElement::text("Hello, world!");
        assert_eq!(
            feature.to_html(&element),
            "<p class=\"txt-default\">Hello, world!</p>"
        );
    }

    #[test]
    fn test_core_feature_renders_section_placeholder() {
        let feature = CoreFeature::new();
        let section = DocumentElement::section(
            1,
            "Overview",
            vec![DocumentElement::text("Section body content")],
        );
        let html = feature.to_html(&section);
        assert_eq!(
            html,
            "<section class=\"section\" data-level=\"1\"><h1>Overview</h1><div class=\"section-body\"><p class=\"txt-default\">Section body content</p></div></section>"
        );
    }

    #[test]
    fn test_core_feature_empty_for_unsupported_elements() {
        let feature = CoreFeature::new();
        let code = DocumentElement::code_block(Some("rust"), "fn main() {}");
        assert_eq!(feature.to_html(&code), "");
    }
}
