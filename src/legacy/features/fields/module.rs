//! Form field state persistence and reset feature slice.

use crate::legacy::core::feature::{DocumentContext, Feature};

/// Unified fields feature slice providing persistent form input handling.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct FieldsFeature;

impl FieldsFeature {
    /// Creates a new fields feature instance.
    ///
    /// # Examples
    ///
    /// ```
    /// use doc2flow::legacy::features::fields::FieldsFeature;

    ///
    /// let feature = FieldsFeature::new();
    /// ```
    #[must_use]
    pub const fn new() -> Self {
        Self
    }
}

impl Feature for FieldsFeature {
    /// Returns the unique feature identifier "fields".
    fn name(&self) -> &'static str {
        "fields"
    }

    /// Evaluates if the fields feature is directly enabled.
    ///
    /// Currently returns false as fields is activated via dependency resolution (e.g. by `code`).
    fn is_enabled(&self, _ctx: &DocumentContext) -> bool {
        false
    }

    /// Returns embedded JavaScript client script for form input persistence.
    fn javascript(&self) -> Option<&'static str> {
        Some(include_str!("fields.js"))
    }

    /// Returns optional CSS stylesheet rules, none for fields.
    fn css(&self) -> Option<&'static str> {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_fields_feature_activation_logic() {
        let feature = FieldsFeature::new();
        let fm = HashMap::new();

        let ctx_with_inputs = DocumentContext::new(&fm, "<input class=\"persistent-field\">");
        assert!(!feature.is_enabled(&ctx_with_inputs));

        let ctx_plain = DocumentContext::new(&fm, "# Plain text");
        assert!(!feature.is_enabled(&ctx_plain));

        let ctx_empty = DocumentContext::new(&fm, "");
        assert!(!feature.is_enabled(&ctx_empty));
    }

    #[test]
    fn test_fields_assets_embedded() {
        let feature = FieldsFeature::new();
        assert_eq!(feature.name(), "fields");
        assert_eq!(feature.css(), None);
        let js = feature.javascript().expect("JavaScript must be embedded");

        assert!(js.contains("saveFields") || js.contains("loadFields"));
        assert!(js.contains("persistent-field"));
    }
}
