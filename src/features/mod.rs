//! Central feature registry exposing available vertical slices.

#[path = "bullet_list/module.rs"]
pub mod bullet_list;

#[path = "code/module.rs"]
pub mod code;

#[path = "core/module.rs"]
pub mod core;

#[path = "ordered_list/module.rs"]
pub mod ordered_list;

#[path = "task/module.rs"]
pub mod task;

#[path = "unknown/module.rs"]
pub mod unknown;

use crate::core::feature::Feature;
pub use bullet_list::BulletListFeature;
pub use code::CodeFeature;
pub use core::CoreFeature;
pub use ordered_list::OrderedListFeature;
pub use task::TaskFeature;
pub use unknown::UnknownFeature;

/// Static instance of the bullet list feature to avoid runtime allocations.
static BULLET_LIST_FEATURE: BulletListFeature = BulletListFeature;

/// Static instance of the code feature to avoid runtime allocations.
static CODE_FEATURE: CodeFeature = CodeFeature;

/// Static instance of the core feature to avoid runtime allocations.
static CORE_FEATURE: CoreFeature = CoreFeature;

/// Static instance of the ordered list feature to avoid runtime allocations.
static ORDERED_LIST_FEATURE: OrderedListFeature = OrderedListFeature;

/// Static instance of the task feature to avoid runtime allocations.
static TASK_FEATURE: TaskFeature = TaskFeature;

/// Static instance of the unknown feature to avoid runtime allocations.
static UNKNOWN_FEATURE: UnknownFeature = UnknownFeature;

/// Returns a reference to the feature instance matching the given name with zero allocations.
///
/// # Examples
///
/// ```
/// use doc2flow::features::get_feature;
///
/// assert!(get_feature("bullet_list").is_some());
/// assert!(get_feature("code").is_some());
/// assert!(get_feature("code_block").is_some());
/// assert!(get_feature("core").is_some());
/// assert!(get_feature("ordered_list").is_some());
/// assert!(get_feature("task").is_some());
/// assert!(get_feature("unknown").is_some());
/// assert!(get_feature("non_existent").is_none());
/// ```
pub fn get_feature(name: &str) -> Option<&'static dyn Feature> {
    match name {
        "bullet_list" => Some(&BULLET_LIST_FEATURE),
        "code" | "code_block" => Some(&CODE_FEATURE),
        "core" => Some(&CORE_FEATURE),
        "ordered_list" => Some(&ORDERED_LIST_FEATURE),
        "task" => Some(&TASK_FEATURE),
        "unknown" => Some(&UNKNOWN_FEATURE),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::document::{DocumentElement, DocumentParameters};

    #[test]
    fn test_get_feature_returns_bullet_list() {
        let bullet_feature = get_feature("bullet_list").expect("bullet_list feature should exist");
        let element = DocumentElement::bullet_list_item("Bullet item");
        assert_eq!(
            bullet_feature.to_html(&element, "", 1, 0, &DocumentParameters::default()),
            "  <div class=\"item item-bullet\">\n    <span class=\"bullet-marker\">&bull;</span>\n    <span class=\"bullet-content\">\n      Bullet item\n    </span>\n  </div>\n"
        );
    }

    #[test]
    fn test_get_feature_returns_task() {
        let task_feature = get_feature("task").expect("task feature should exist");
        let unchecked = DocumentElement::check_box_item(false, "Pending task");
        assert_eq!(
            task_feature.to_html(&unchecked, "", 1, 0, &DocumentParameters::default()),
            "  <div class=\"item item-check\">\n    <span class=\"check-marker\">\n      <input type=\"checkbox\" class=\"check-box\" />\n    </span>\n    <span class=\"check-content\">\n      Pending task\n    </span>\n  </div>\n"
        );

        let checked = DocumentElement::check_box_item(true, "Done task");
        assert_eq!(
            task_feature.to_html(&checked, "", 1, 0, &DocumentParameters::default()),
            "  <div class=\"item item-check checked\">\n    <span class=\"check-marker\">\n      <input type=\"checkbox\" class=\"check-box\" checked />\n    </span>\n    <span class=\"check-content\">\n      Done task\n    </span>\n  </div>\n"
        );
    }

    #[test]
    fn test_get_feature_returns_code() {
        let code_feature = get_feature("code").expect("code feature should exist");
        let element = DocumentElement::code_block(Some("rust"), "fn main() {}");
        assert_eq!(
            code_feature.to_html(&element, "", 1, 0, &DocumentParameters::default()),
            "  <pre class=\"code-default\"><code>fn main() {}</code></pre>\n"
        );

        let code_block_feature = get_feature("code_block").expect("code_block alias should exist");
        assert_eq!(
            code_block_feature.to_html(&element, "", 1, 0, &DocumentParameters::default()),
            "  <pre class=\"code-default\"><code>fn main() {}</code></pre>\n"
        );
    }

    #[test]
    fn test_get_feature_returns_core() {
        let core_feature = get_feature("core").expect("core feature should exist");
        let element = DocumentElement::text("Hello");
        assert_eq!(
            core_feature.to_html(&element, "", 1, 0, &DocumentParameters::default()),
            "  <div class=\"item item-text\">\n    <span class=\"text-content\">\n      Hello\n    </span>\n  </div>\n"
        );
    }

    #[test]
    fn test_get_feature_returns_ordered_list() {
        let ordered_feature =
            get_feature("ordered_list").expect("ordered_list feature should exist");
        let element = DocumentElement::ordered_list_item(1, "Ordered item");
        assert_eq!(
            ordered_feature.to_html(&element, "", 1, 0, &DocumentParameters::default()),
            "  <div class=\"item item-order\">\n    <span class=\"order-marker\">1.</span>\n    <span class=\"order-content\">\n      Ordered item\n    </span>\n  </div>\n"
        );
    }

    #[test]
    fn test_get_feature_returns_unknown() {
        let unknown_feature = get_feature("unknown").expect("unknown feature should exist");
        let element = DocumentElement::unknown("Raw line");
        assert_eq!(
            unknown_feature.to_html(&element, "", 1, 0, &DocumentParameters::default()),
            "  <p class=\"unknown-default\">\n    Raw line\n  </p>\n"
        );
    }

    #[test]
    fn test_get_feature_unknown_returns_none() {
        assert!(get_feature("non_existent").is_none());
        assert!(get_feature("bullet_list_item").is_none());
        assert!(get_feature("check_box_item").is_none());
        assert!(get_feature("ordered_list_item").is_none());
    }
}
