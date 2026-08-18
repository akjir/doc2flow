//! Central feature registry exposing available vertical slices.

#[path = "bullet/module.rs"]
pub mod bullet;

#[path = "code/module.rs"]
pub mod code;

#[path = "core/module.rs"]
pub mod core;

#[path = "header/module.rs"]
pub mod header;

#[path = "image/module.rs"]
pub mod image;

#[path = "input/module.rs"]
pub mod input;

#[path = "ordered/module.rs"]
pub mod ordered;

#[path = "shoutout/module.rs"]
pub mod shoutout;

#[path = "table/module.rs"]
pub mod table;

#[path = "task/module.rs"]
pub mod task;

#[path = "unknown/module.rs"]
pub mod unknown;

pub use bullet::BulletFeature;
pub use code::CodeFeature;
pub use core::CoreFeature;
pub use header::HeaderFeature;
pub use image::{ImageFeature, embed_images_as_base64, embed_images_as_base64_with_source};
pub use input::InputFeature;
pub use ordered::OrderedFeature;
pub use shoutout::ShoutoutFeature;
pub use table::TableFeature;
pub use task::TaskFeature;
pub use unknown::UnknownFeature;

use crate::core::feature::FeatureModule;

/// Static instance of the bullet list feature to avoid runtime allocations.
pub static BULLET_FEATURE: BulletFeature = BulletFeature;

/// Static instance of the code feature to avoid runtime allocations.
pub static CODE_FEATURE: CodeFeature = CodeFeature;

/// Static instance of the core feature to avoid runtime allocations.
pub static CORE_FEATURE: CoreFeature = CoreFeature;

/// Static instance of the header feature to avoid runtime allocations.
pub static HEADER_FEATURE: HeaderFeature = HeaderFeature;

/// Static instance of the image feature to avoid runtime allocations.
pub static IMAGE_FEATURE: ImageFeature = ImageFeature;

/// Static instance of the input feature to avoid runtime allocations.
pub static INPUT_FEATURE: InputFeature = InputFeature;

/// Static instance of the ordered list feature to avoid runtime allocations.
pub static ORDERED_FEATURE: OrderedFeature = OrderedFeature;

/// Static instance of the shoutout feature to avoid runtime allocations.
pub static SHOUTOUT_FEATURE: ShoutoutFeature = ShoutoutFeature;

/// Static instance of the table feature to avoid runtime allocations.
pub static TABLE_FEATURE: TableFeature = TableFeature;

/// Static instance of the task feature to avoid runtime allocations.
pub static TASK_FEATURE: TaskFeature = TaskFeature;

/// Static instance of the unknown feature to avoid runtime allocations.
pub static UNKNOWN_FEATURE: UnknownFeature = UnknownFeature;

/// Static collection of all standard feature modules for default dispatch and inspection.
pub static ALL_FEATURE_MODULES: [&'static dyn FeatureModule; 11] = [
    &CORE_FEATURE,
    &HEADER_FEATURE,
    &BULLET_FEATURE,
    &CODE_FEATURE,
    &IMAGE_FEATURE,
    &INPUT_FEATURE,
    &ORDERED_FEATURE,
    &SHOUTOUT_FEATURE,
    &TABLE_FEATURE,
    &TASK_FEATURE,
    &UNKNOWN_FEATURE,
];

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_all_feature_modules_count_and_registration() {
        assert_eq!(ALL_FEATURE_MODULES.len(), 11);
        let names: Vec<&str> = ALL_FEATURE_MODULES.iter().map(|m| m.name()).collect();
        assert_eq!(
            names,
            [
                "core", "header", "bullet", "code", "image", "input", "ordered", "shoutout", "table", "task", "unknown"
            ]
        );
    }

    #[test]
    fn test_static_feature_instances_match_defaults() {
        assert_eq!(BULLET_FEATURE, BulletFeature::default());
        assert_eq!(CODE_FEATURE, CodeFeature::default());
        assert_eq!(CORE_FEATURE, CoreFeature::default());
        assert_eq!(HEADER_FEATURE, HeaderFeature::default());
        assert_eq!(IMAGE_FEATURE, ImageFeature::default());
        assert_eq!(INPUT_FEATURE, InputFeature::default());
        assert_eq!(ORDERED_FEATURE, OrderedFeature::default());
        assert_eq!(SHOUTOUT_FEATURE, ShoutoutFeature::default());
        assert_eq!(TABLE_FEATURE, TableFeature::default());
        assert_eq!(TASK_FEATURE, TaskFeature::default());
        assert_eq!(UNKNOWN_FEATURE, UnknownFeature::default());
    }
}
