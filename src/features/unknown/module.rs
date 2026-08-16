//! Unknown vertical slice feature module.

use crate::core::document::{DocumentElement, DocumentElementId, DocumentParameters};
use crate::core::feature::FeatureModule;
use crate::core::format::push_indent;
use crate::core::renderer::{DocumentElementRenderer, HtmlRenderer};

/// Embedded unknown CSS styles for fallback elements.
pub const CSS: &str = include_str!("unknown.css");

/// Supported document element identifiers for unrecognized unknown elements.
const UNKNOWN_SUPPORTED: [DocumentElementId; 1] = [DocumentElementId::Unknown];

/// Unknown feature renderer handling unrecognized fallback elements.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct UnknownFeature;

impl UnknownFeature {
    /// Creates a new unknown feature instance.
    #[must_use]
    pub fn new() -> Self {
        Self
    }
}

impl DocumentElementRenderer for UnknownFeature {
    fn supported(&self) -> &[DocumentElementId] {
        &UNKNOWN_SUPPORTED
    }

    fn render_element(
        &self,
        element: &DocumentElement,
        indent: usize,
        _depth: usize,
        _parameters: &DocumentParameters,
        out: &mut String,
        _renderer: &HtmlRenderer,
    ) {
        if let DocumentElement::Unknown(text) = element {
            push_indent(out, indent);
            out.push_str("<p class=\"unknown-default\">\n");
            for line in text.lines() {
                push_indent(out, indent + 1);
                out.push_str(line);
                out.push('\n');
            }
            push_indent(out, indent);
            out.push_str("</p>\n");
        }
    }
}

impl FeatureModule for UnknownFeature {
    fn name(&self) -> &'static str {
        "unknown"
    }

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
        feature.render_element(
            &element,
            1,
            0,
            &DocumentParameters::default(),
            &mut out,
            &renderer,
        );
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
        feature.render_element(
            &text,
            0,
            0,
            &DocumentParameters::default(),
            &mut out,
            &renderer,
        );
        assert!(out.is_empty());
    }
}
