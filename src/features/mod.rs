//! Central feature registry exposing available vertical slices.

#[path = "core/module.rs"]
pub mod core;

#[path = "unknown/module.rs"]
pub mod unknown;

use crate::core::feature::Feature;
pub use core::CoreFeature;
pub use unknown::UnknownFeature;

/// Static instance of the core feature to avoid runtime allocations.
static CORE_FEATURE: CoreFeature = CoreFeature;

/// Static instance of the unknown feature to avoid runtime allocations.
static UNKNOWN_FEATURE: UnknownFeature = UnknownFeature;

/// Returns a reference to the feature instance matching the given name with zero allocations.
///
/// # Examples
///
/// ```
/// use doc2flow::features::get_feature;
///
/// assert!(get_feature("core").is_some());
/// assert!(get_feature("unknown").is_some());
/// assert!(get_feature("non_existent").is_none());
/// ```
pub fn get_feature(name: &str) -> Option<&'static dyn Feature> {
    match name {
        "core" => Some(&CORE_FEATURE),
        "unknown" => Some(&UNKNOWN_FEATURE),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::document::DocumentElement;

    #[test]
    fn test_get_feature_returns_core() {
        let core_feature = get_feature("core").expect("core feature should exist");
        let element = DocumentElement::text("Hello");
        assert_eq!(
            core_feature.to_html(&element),
            "<p class=\"txt-default\">Hello</p>"
        );
    }

    #[test]
    fn test_get_feature_returns_unknown() {
        let unknown_feature = get_feature("unknown").expect("unknown feature should exist");
        let element = DocumentElement::unknown("Raw line");
        assert_eq!(
            unknown_feature.to_html(&element),
            "<p class=\"unknown-default\">Raw line</p>"
        );
    }

    #[test]
    fn test_get_feature_unknown_returns_none() {
        assert!(get_feature("non_existent").is_none());
    }
}
