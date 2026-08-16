//! Shoutout callout vertical slice feature module.

use crate::core::document::{DocumentElement, DocumentElementId, DocumentParameters};
use crate::core::feature::FeatureModule;
use crate::core::renderer::{DocumentElementRenderer, HtmlRenderer};

/// Supported document element identifiers for shoutout elements.
const SHOUTOUT_SUPPORTED: [DocumentElementId; 1] = [DocumentElementId::Shoutout];

/// Shoutout feature renderer handling callout and shoutout panel elements.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct ShoutoutFeature;

impl ShoutoutFeature {
    /// Creates a new shoutout feature instance.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }
}

impl DocumentElementRenderer for ShoutoutFeature {
    fn supported(&self) -> &[DocumentElementId] {
        &SHOUTOUT_SUPPORTED
    }

    fn render_element(
        &self,
        _element: &DocumentElement,
        _indent: usize,
        _depth: usize,
        _parameters: &DocumentParameters,
        _out: &mut String,
        _renderer: &HtmlRenderer,
    ) {
        // No-op: Shoutouts currently render empty string output.
    }
}

impl FeatureModule for ShoutoutFeature {
    fn name(&self) -> &'static str {
        "shoutout"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::document::ShoutoutElementKind;

    #[test]
    fn test_shoutout_feature_renders_empty() {
        let feature = ShoutoutFeature::new();
        let shoutout = DocumentElement::shoutout(ShoutoutElementKind::Note, "Note content");
        let mut out = String::new();
        let renderer = HtmlRenderer::default_renderer();
        feature.render_element(
            &shoutout,
            1,
            0,
            &DocumentParameters::default(),
            &mut out,
            &renderer,
        );
        assert!(out.is_empty());
    }

    #[test]
    fn test_shoutout_feature_metadata() {
        let feature = ShoutoutFeature::new();
        assert_eq!(feature.name(), "shoutout");
        assert_eq!(feature.css(), None);
        assert_eq!(feature.javascript(), &[] as &[&str]);
        assert_eq!(feature.supported(), &[DocumentElementId::Shoutout]);
    }
}
