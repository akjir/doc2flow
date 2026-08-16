//! Unknown vertical slice feature module.

use crate::core::document::{DocumentElement, DocumentParameters};
use crate::core::feature::Feature;
use crate::core::format::push_indent;
use crate::core::renderer::HtmlRenderer;

/// Embedded unknown CSS styles for fallback elements.
pub const CSS: &str = include_str!("unknown.css");

/// Unknown feature renderer handling unrecognized fallback elements.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct UnknownFeature;

impl UnknownFeature {
    /// Creates a new unknown feature instance.
    #[must_use]
    pub const fn new() -> Self {
        Self
    }
}

impl Feature for UnknownFeature {
    /// Intercepts the rendering of unknown fallback elements into the output buffer.
    fn try_render_body(
        &self,
        element: &DocumentElement,
        indent: usize,
        _depth: usize,
        _parameters: &DocumentParameters,
        out: &mut String,
        _renderer: &HtmlRenderer,
    ) -> bool {
        match element {
            DocumentElement::Unknown(text) => {
                push_indent(out, indent);
                out.push_str("<p class=\"unknown-default\">\n");
                for line in text.lines() {
                    push_indent(out, indent + 1);
                    out.push_str(line);
                    out.push('\n');
                }
                push_indent(out, indent);
                out.push_str("</p>\n");
                true
            }
            _ => false,
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
        let mut out = String::new();
        let renderer = HtmlRenderer::default_renderer();
        assert!(feature.try_render_body(
            &element,
            1,
            0,
            &DocumentParameters::default(),
            &mut out,
            &renderer,
        ));
        assert_eq!(
            out,
            "  <p class=\"unknown-default\">\n    Unrecognized raw markdown line\n  </p>\n"
        );
    }

    #[test]
    fn test_unknown_feature_empty_for_unsupported_elements() {
        let feature = UnknownFeature::new();
        let text = DocumentElement::text("Regular text");
        let mut out = String::new();
        let renderer = HtmlRenderer::default_renderer();
        assert!(!feature.try_render_body(
            &text,
            0,
            0,
            &DocumentParameters::default(),
            &mut out,
            &renderer,
        ));
        assert!(out.is_empty());
    }
}
