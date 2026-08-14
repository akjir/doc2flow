//! Unknown vertical slice feature module.

use crate::core::document::DocumentElement;
use crate::core::feature::Feature;

/// Embedded unknown CSS styles for fallback elements.
pub const CSS: &str = include_str!("unknown.css");

/// Unknown feature renderer handling unrecognized fallback elements.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct UnknownFeature;

impl UnknownFeature {
    /// Creates a new unknown feature instance.
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

impl Feature for UnknownFeature {
    /// Converts a document element and inner content into an HTML string representation.
    fn to_html(&self, element: &DocumentElement, _content: &str, indent: usize, _depth: usize) -> String {
        match element {
            DocumentElement::Unknown(text) => {
                let spaces = indent * 2;
                let inner_spaces = (indent + 1) * 2;
                let mut out = String::with_capacity(text.len() + spaces * 2 + inner_spaces + 32);
                push_indent(&mut out, indent);
                out.push_str("<p class=\"unknown-default\">\n");
                for line in text.lines() {
                    push_indent(&mut out, indent + 1);
                    out.push_str(line);
                    out.push('\n');
                }
                push_indent(&mut out, indent);
                out.push_str("</p>\n");
                out
            }
            _ => String::new(),
        }
    }

    /// Returns the embedded CSS stylesheet for the unknown feature.
    fn css(&self) -> Option<&'static str> {
        Some(CSS)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_unknown_feature_css() {
        let feature = UnknownFeature::new();
        let css = feature.css().expect("unknown css should exist");
        assert!(css.contains("--unknown-bg:"));
        assert!(css.contains(".unknown-default"));
    }

    #[test]
    fn test_unknown_feature_renders_unknown_element() {
        let feature = UnknownFeature::new();
        let element = DocumentElement::unknown("Unrecognized raw markdown line");
        assert_eq!(
            feature.to_html(&element, "", 1, 0),
            "  <p class=\"unknown-default\">\n    Unrecognized raw markdown line\n  </p>\n"
        );
    }

    #[test]
    fn test_unknown_feature_empty_for_unsupported_elements() {
        let feature = UnknownFeature::new();
        let text = DocumentElement::text("Regular text");
        assert_eq!(feature.to_html(&text, "", 0, 0), "");
    }
}
