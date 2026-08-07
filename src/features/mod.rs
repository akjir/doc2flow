//! Central feature registry exposing available vertical slices.

#[path = "code/module.rs"]
pub mod code;
#[path = "image/module.rs"]
pub mod image;

pub use code::CodeFeature;
pub use image::ImageFeature;
use crate::core::feature::Feature;

/// Returns all available vertical slice feature instances in registration order.
///
/// # Examples
///
/// ```
/// use doc2flow::features::get_all_features;
///
/// let features = get_all_features();
/// assert_eq!(features.len(), 2);
/// assert_eq!(features[0].name(), "code");
/// assert_eq!(features[1].name(), "image");
/// ```
pub fn get_all_features() -> Vec<Box<dyn Feature>> {
    vec![
        Box::new(CodeFeature::new()),
        Box::new(ImageFeature::new()),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_feature_registry_contains_registered_features() {
        let list = get_all_features();
        assert_eq!(list.len(), 2);
        assert_eq!(list[0].name(), "code");
        assert_eq!(list[1].name(), "image");
    }
}
