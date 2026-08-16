//! Central feature registry exposing available vertical slices.

#[path = "bullet/module.rs"]
pub mod bullet;

#[path = "code/module.rs"]
pub mod code;

#[path = "core/module.rs"]
pub mod core;

#[path = "image/module.rs"]
pub mod image;

#[path = "ordered/module.rs"]
pub mod ordered;

#[path = "table/module.rs"]
pub mod table;

#[path = "task/module.rs"]
pub mod task;

#[path = "unknown/module.rs"]
pub mod unknown;

pub use bullet::BulletFeature;
pub use code::CodeFeature;
pub use core::CoreFeature;
pub use image::ImageFeature;
pub use ordered::OrderedFeature;
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

/// Static instance of the image feature to avoid runtime allocations.
pub static IMAGE_FEATURE: ImageFeature = ImageFeature;

/// Static instance of the ordered list feature to avoid runtime allocations.
pub static ORDERED_FEATURE: OrderedFeature = OrderedFeature;

/// Static instance of the table feature to avoid runtime allocations.
pub static TABLE_FEATURE: TableFeature = TableFeature;

/// Static instance of the task feature to avoid runtime allocations.
pub static TASK_FEATURE: TaskFeature = TaskFeature;

/// Static instance of the unknown feature to avoid runtime allocations.
pub static UNKNOWN_FEATURE: UnknownFeature = UnknownFeature;

/// Static collection of all standard feature modules for default dispatch and inspection.
pub static ALL_FEATURE_MODULES: [&'static dyn FeatureModule; 8] = [
    &CORE_FEATURE,
    &BULLET_FEATURE,
    &CODE_FEATURE,
    &IMAGE_FEATURE,
    &ORDERED_FEATURE,
    &TABLE_FEATURE,
    &TASK_FEATURE,
    &UNKNOWN_FEATURE,
];

/// Returns a reference to the feature module instance matching the given name with zero allocations.
///
/// # Examples
///
/// ```
/// use doc2flow::features::get_feature;
///
/// assert!(get_feature("bullet").is_some());
/// assert!(get_feature("code").is_some());
/// assert!(get_feature("core").is_some());
/// assert!(get_feature("image").is_some());
/// assert!(get_feature("ordered").is_some());
/// assert!(get_feature("table").is_some());
/// assert!(get_feature("task").is_some());
/// assert!(get_feature("unknown").is_some());
/// assert!(get_feature("non_existent").is_none());
/// ```
#[must_use]
pub fn get_feature(name: &str) -> Option<&'static dyn FeatureModule> {
    match name {
        "bullet" => Some(&BULLET_FEATURE),
        "code" => Some(&CODE_FEATURE),
        "core" => Some(&CORE_FEATURE),
        "image" => Some(&IMAGE_FEATURE),
        "ordered" => Some(&ORDERED_FEATURE),
        "table" => Some(&TABLE_FEATURE),
        "task" => Some(&TASK_FEATURE),
        "unknown" => Some(&UNKNOWN_FEATURE),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::document::{DocumentElement, DocumentParameters};
    use crate::core::renderer::HtmlRenderer;

    #[test]
    fn test_get_feature_returns_bullet() {
        let bullet_feature = get_feature("bullet").expect("bullet feature should exist");
        let element = DocumentElement::bullet_list_item("Bullet item");
        let mut out = String::new();
        let renderer = HtmlRenderer::default_renderer();
        bullet_feature.render_element(
            &element,
            1,
            0,
            &DocumentParameters::default(),
            &mut out,
            &renderer,
        );
        assert_eq!(
            out,
            "  <div class=\"item bullet-item\">\n    <span class=\"bullet-marker\">&bull;</span>\n    <span class=\"bullet-content\">\n      Bullet item\n    </span>\n  </div>\n"
        );
    }

    #[test]
    fn test_get_feature_returns_task() {
        let task_feature = get_feature("task").expect("task feature should exist");
        let unchecked = DocumentElement::check_box_item(false, "Pending task");
        let mut out = String::new();
        let renderer = HtmlRenderer::default_renderer();
        task_feature.render_element(
            &unchecked,
            1,
            0,
            &DocumentParameters::default(),
            &mut out,
            &renderer,
        );
        assert_eq!(
            out,
            "  <div class=\"item check-item\">\n    <span class=\"check-marker\">\n      <input type=\"checkbox\" class=\"check-box\" />\n    </span>\n    <span class=\"check-content\">\n      Pending task\n    </span>\n  </div>\n"
        );

        let checked = DocumentElement::check_box_item(true, "Done task");
        let mut out_checked = String::new();
        task_feature.render_element(
            &checked,
            1,
            0,
            &DocumentParameters::default(),
            &mut out_checked,
            &renderer,
        );
        assert_eq!(
            out_checked,
            "  <div class=\"item check-item checked\">\n    <span class=\"check-marker\">\n      <input type=\"checkbox\" class=\"check-box\" checked />\n    </span>\n    <span class=\"check-content\">\n      Done task\n    </span>\n  </div>\n"
        );
    }

    #[test]
    fn test_get_feature_returns_code() {
        let code_feature = get_feature("code").expect("code feature should exist");
        let element = DocumentElement::code_block(Some("rust"), "fn main() {}");
        let mut out = String::new();
        let renderer = HtmlRenderer::default_renderer();
        code_feature.render_element(
            &element,
            1,
            0,
            &DocumentParameters::default(),
            &mut out,
            &renderer,
        );
        assert_eq!(
            out,
            "  <pre class=\"code-default\"><code>fn main() {}</code></pre>\n"
        );
    }

    #[test]
    fn test_get_feature_returns_core() {
        let core_feature = get_feature("core").expect("core feature should exist");
        let element = DocumentElement::text("Hello");
        let mut out = String::new();
        let renderer = HtmlRenderer::default_renderer();
        core_feature.render_element(
            &element,
            1,
            0,
            &DocumentParameters::default(),
            &mut out,
            &renderer,
        );
        assert_eq!(
            out,
            "  <div class=\"item text-item\">\n    <span class=\"text-content\">\n      Hello\n    </span>\n  </div>\n"
        );
    }

    #[test]
    fn test_get_feature_returns_image() {
        let image_feature = get_feature("image").expect("image feature should exist");
        let element = DocumentElement::image("Alt", "pic.png");
        let mut out = String::new();
        let renderer = HtmlRenderer::default_renderer();
        image_feature.render_element(
            &element,
            1,
            0,
            &DocumentParameters::default(),
            &mut out,
            &renderer,
        );
        assert_eq!(
            out,
            "  <div class=\"image-item\">\n    <img src=\"pic.png\" alt=\"Alt\" />\n  </div>\n"
        );
    }

    #[test]
    fn test_get_feature_returns_ordered() {
        let ordered_feature = get_feature("ordered").expect("ordered feature should exist");
        let element = DocumentElement::ordered_list_item(1, "Ordered item");
        let mut out = String::new();
        let renderer = HtmlRenderer::default_renderer();
        ordered_feature.render_element(
            &element,
            1,
            0,
            &DocumentParameters::default(),
            &mut out,
            &renderer,
        );
        assert_eq!(
            out,
            "  <div class=\"item order-item\">\n    <span class=\"order-marker\">1.</span>\n    <span class=\"order-content\">\n      Ordered item\n    </span>\n  </div>\n"
        );
    }

    #[test]
    fn test_get_feature_returns_table() {
        let table_feature = get_feature("table").expect("table feature should exist");
        let element = DocumentElement::table(
            vec![crate::core::document::TableAlignment::None],
            vec![vec!["Col".into()]],
        );
        let mut out = String::new();
        let renderer = HtmlRenderer::default_renderer();
        table_feature.render_element(
            &element,
            1,
            0,
            &DocumentParameters::default(),
            &mut out,
            &renderer,
        );
        assert_eq!(
            out,
            "  <div class=\"table-wrap\">\n    <table class=\"table-default\">\n      <thead>\n        <tr>\n          <th>Col</th>\n        </tr>\n      </thead>\n    </table>\n  </div>\n"
        );
    }

    #[test]
    fn test_get_feature_returns_unknown() {
        let unknown_feature = get_feature("unknown").expect("unknown feature should exist");
        let element = DocumentElement::unknown("Raw line");
        let mut out = String::new();
        let renderer = HtmlRenderer::default_renderer();
        unknown_feature.render_element(
            &element,
            1,
            0,
            &DocumentParameters::default(),
            &mut out,
            &renderer,
        );
        assert_eq!(
            out,
            "  <p class=\"unknown-default\">\n    Raw line\n  </p>\n"
        );
    }

    #[test]
    fn test_get_feature_unknown_returns_none() {
        assert!(get_feature("non_existent").is_none());
        assert!(get_feature("bullet_list").is_none());
        assert!(get_feature("ordered_list").is_none());
        assert!(get_feature("code_block").is_none());
        assert!(get_feature("images").is_none());
        assert!(get_feature("tables").is_none());
        assert!(get_feature("bullet_list_item").is_none());
        assert!(get_feature("check_box_item").is_none());
        assert!(get_feature("ordered_list_item").is_none());
    }
}
