//! Central feature registry exposing available vertical slices.

#[path = "code/module.rs"]
pub mod code;

#[path = "core/module.rs"]
pub mod core;

#[path = "unknown/module.rs"]
pub mod unknown;

use crate::core::feature::Feature;
pub use code::CodeFeature;
pub use core::CoreFeature;
pub use unknown::UnknownFeature;

/// Static instance of the code feature to avoid runtime allocations.
static CODE_FEATURE: CodeFeature = CodeFeature;

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
/// assert!(get_feature("code").is_some());
/// assert!(get_feature("code_block").is_some());
/// assert!(get_feature("core").is_some());
/// assert!(get_feature("unknown").is_some());
/// assert!(get_feature("non_existent").is_none());
/// ```
pub fn get_feature(name: &str) -> Option<&'static dyn Feature> {
    match name {
        "code" | "code_block" => Some(&CODE_FEATURE),
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
    fn test_get_feature_returns_code() {
        let code_feature = get_feature("code").expect("code feature should exist");
        let element = DocumentElement::code_block(Some("rust"), "fn main() {}");
        assert_eq!(
            code_feature.to_html(&element, "", 1),
            "  <pre class=\"code-default\"><code>fn main() {}</code></pre>\n"
        );

        let code_block_feature = get_feature("code_block").expect("code_block alias should exist");
        assert_eq!(
            code_block_feature.to_html(&element, "", 1),
            "  <pre class=\"code-default\"><code>fn main() {}</code></pre>\n"
        );
    }

    #[test]
    fn test_get_feature_returns_core() {
        let core_feature = get_feature("core").expect("core feature should exist");
        let element = DocumentElement::text("Hello");
        assert_eq!(
            core_feature.to_html(&element, "", 1),
            "  <p class=\"txt-default\">\n    Hello\n  </p>\n"
        );
    }

    #[test]
    fn test_get_feature_returns_unknown() {
        let unknown_feature = get_feature("unknown").expect("unknown feature should exist");
        let element = DocumentElement::unknown("Raw line");
        assert_eq!(
            unknown_feature.to_html(&element, "", 1),
            "  <p class=\"unknown-default\">\n    Raw line\n  </p>\n"
        );
    }

    #[test]
    fn test_get_feature_unknown_returns_none() {
        assert!(get_feature("non_existent").is_none());
    }
}
