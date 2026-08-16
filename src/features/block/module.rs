//! Block directive vertical slice feature module.

use crate::core::document::{DocumentElement, DocumentElementId, DocumentParameters};
use crate::core::feature::FeatureModule;
use crate::core::renderer::{DocumentElementRenderer, HtmlRenderer};

/// Supported document element identifiers for block directive elements.
const BLOCK_SUPPORTED: [DocumentElementId; 1] = [DocumentElementId::BlockDirective];

/// Block directive feature renderer handling block directive elements.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct BlockFeature;

impl BlockFeature {
    /// Creates a new block directive feature instance.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }
}

impl DocumentElementRenderer for BlockFeature {
    fn supported(&self) -> &[DocumentElementId] {
        &BLOCK_SUPPORTED
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
        // No-op: Block directives currently render empty string output.
    }
}

impl FeatureModule for BlockFeature {
    fn name(&self) -> &'static str {
        "block"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_block_feature_renders_empty() {
        let feature = BlockFeature::new();
        let directive = DocumentElement::block_directive("variables", vec![]);
        let mut out = String::new();
        let renderer = HtmlRenderer::default_renderer();
        feature.render_element(
            &directive,
            1,
            0,
            &DocumentParameters::default(),
            &mut out,
            &renderer,
        );
        assert!(out.is_empty());
    }

    #[test]
    fn test_block_feature_metadata() {
        let feature = BlockFeature::new();
        assert_eq!(feature.name(), "block");
        assert_eq!(feature.css(), None);
        assert_eq!(feature.javascript(), &[] as &[&str]);
        assert_eq!(feature.supported(), &[DocumentElementId::BlockDirective]);
    }
}
