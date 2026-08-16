//! Document AST feature module trait definitions.

use std::fmt;

use crate::core::renderer::DocumentElementRenderer;

/// Trait defining a vertical slice feature module with lifecycle, metadata, and asset capabilities.
pub trait FeatureModule: DocumentElementRenderer + Send + Sync + fmt::Debug {
    /// Returns the canonical name identifier of this feature (e.g., `"bullet"`, `"code"`, `"core"`).
    fn name(&self) -> &'static str;

    /// Returns optional CSS stylesheet rules for this feature, defaulting to `None`.
    fn css(&self) -> Option<&'static str> {
        None
    }

    /// Returns optional JavaScript client logic files for this feature, defaulting to an empty slice.
    fn javascript(&self) -> &[&'static str] {
        &[]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::document::{DocumentElement, DocumentElementId, DocumentParameters};
    use crate::core::renderer::HtmlRenderer;

    #[derive(Debug)]
    struct MockFeature;

    impl DocumentElementRenderer for MockFeature {
        fn supported(&self) -> &[DocumentElementId] {
            &[DocumentElementId::Text]
        }

        fn render_element(
            &self,
            _element: &DocumentElement,
            _indent: usize,
            _depth: usize,
            _parameters: &DocumentParameters,
            out: &mut String,
            _renderer: &HtmlRenderer,
        ) {
            out.push_str("mock text\n");
        }
    }

    impl FeatureModule for MockFeature {
        fn name(&self) -> &'static str {
            "mock"
        }

        fn css(&self) -> Option<&'static str> {
            Some(".mock { color: red; }")
        }

        fn javascript(&self) -> &[&'static str] {
            &["console.log('mock');"]
        }
    }

    #[test]
    fn test_feature_module_defaults_and_overrides() {
        let feature = MockFeature;
        assert_eq!(feature.name(), "mock");
        assert_eq!(feature.css(), Some(".mock { color: red; }"));
        assert_eq!(feature.javascript(), &["console.log('mock');"]);
        assert_eq!(feature.supported(), &[DocumentElementId::Text]);
    }
}
