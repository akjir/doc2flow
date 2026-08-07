//! Central feature registry exposing available vertical slices.

#[path = "code/module.rs"]
pub mod code;

pub use code::CodeFeature;
use crate::core::feature::Feature;

/// Returns all available vertical slice feature instances in registration order.
///
/// # Examples
///
/// ```
/// use doc2flow::features::get_all_features;
///
/// let features = get_all_features();
/// assert!(!features.is_empty());
/// assert_eq!(features[0].name(), "code");
/// ```
pub fn get_all_features() -> Vec<Box<dyn Feature>> {
    vec![Box::new(CodeFeature::new())]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_feature_registry_contains_code_feature() {
        let list = get_all_features();
        assert_eq!(list.len(), 1);
        assert_eq!(list[0].name(), "code");
    }
}
