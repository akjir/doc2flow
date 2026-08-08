//! Central feature registry exposing available vertical slices.

#[path = "code/module.rs"]
pub mod code;
#[path = "header/module.rs"]
pub mod header;
#[path = "image/module.rs"]
pub mod image;
#[path = "table/module.rs"]
pub mod table;
#[path = "tasks/module.rs"]
pub mod tasks;

pub use code::CodeFeature;
pub use header::HeaderFeature;
pub use image::ImageFeature;
pub use table::TableFeature;
pub use tasks::TasksFeature;
use crate::core::feature::Feature;

/// Returns all available vertical slice feature instances in registration order.
///
/// # Examples
///
/// ```
/// use doc2flow::features::get_all_features;
///
/// let features = get_all_features();
/// assert_eq!(features.len(), 5);
/// assert_eq!(features[0].name(), "code");
/// assert_eq!(features[1].name(), "header");
/// assert_eq!(features[2].name(), "image");
/// assert_eq!(features[3].name(), "table");
/// assert_eq!(features[4].name(), "tasks");
/// ```
pub fn get_all_features() -> Vec<Box<dyn Feature>> {
    vec![
        Box::new(CodeFeature::new()),
        Box::new(HeaderFeature::new()),
        Box::new(ImageFeature::new()),
        Box::new(TableFeature::new()),
        Box::new(TasksFeature::new()),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_feature_registry_contains_registered_features() {
        let list = get_all_features();
        assert_eq!(list.len(), 5);
        assert_eq!(list[0].name(), "code");
        assert_eq!(list[1].name(), "header");
        assert_eq!(list[2].name(), "image");
        assert_eq!(list[3].name(), "table");
        assert_eq!(list[4].name(), "tasks");
    }
}
