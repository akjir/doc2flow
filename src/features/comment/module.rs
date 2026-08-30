//! Comment vertical slice feature module.

use crate::core::document::{DocumentElement, DocumentElementId, DocumentParameters};
use crate::core::feature::FeatureModule;
use crate::core::renderer::{DocumentElementRenderer, HtmlRenderer};

/// Embedded comment CSS stylesheet.
pub const CSS: &str = include_str!("comment.css");

/// Embedded comment JavaScript client script.
pub const JS: &str = include_str!("comment.js");

/// Embedded SVG comment icon for interactive elements.
pub const COMMENT_ICON_SVG: &str = "<span class=\"item-comment-icon\"></span>";

/// Supported document element identifiers for comment elements.
const COMMENT_SUPPORTED: [DocumentElementId; 0] = [];

/// Comment feature renderer handling interactive item comments.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct CommentFeature;

impl CommentFeature {
    /// Creates a new comment feature instance.
    ///
    /// # Examples
    ///
    /// ```
    /// use doc2flow::features::comment::CommentFeature;
    ///
    /// let feature = CommentFeature::new();
    /// ```
    #[must_use]
    pub fn new() -> Self {
        Self
    }
}

impl DocumentElementRenderer for CommentFeature {
    fn supported(&self) -> &[DocumentElementId] {
        &COMMENT_SUPPORTED
    }

    fn render_element(
        &self,
        _element: &DocumentElement,
        _indent: usize,
        _depth: usize,
        _parameters: &DocumentParameters,
        _out: &mut String,
        _renderer: &HtmlRenderer,
    ) {
    }
}

impl FeatureModule for CommentFeature {
    fn name(&self) -> &'static str {
        "comment"
    }

    fn css(&self) -> Option<&'static str> {
        Some(CSS)
    }

    fn javascript(&self) -> &[&'static str] {
        &[JS]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_comment_feature_constructor_new() {
        let feature = CommentFeature::new();
        assert_eq!(feature, CommentFeature);
    }

    #[test]
    fn test_comment_feature_name() {
        let feature = CommentFeature::new();
        assert_eq!(feature.name(), "comment");
    }

    #[test]
    fn test_comment_feature_css() {
        let feature = CommentFeature::new();
        let css = feature.css().expect("comment css should exist");
        assert!(css.contains("--icon-comment-mask:"));
        assert!(css.contains(".item-comment-icon"));
        assert!(css.contains(".item-comment-box"));
        assert!(css.contains(".item-comment-input"));
        assert!(css.contains(".item-comment-del"));
    }

    #[test]
    fn test_comment_feature_javascript() {
        let feature = CommentFeature::new();
        let js_files = feature.javascript();
        assert_eq!(js_files.len(), 1);
        let js = js_files[0];
        assert!(js.contains("saveComments"));
        assert!(js.contains("loadComments"));
        assert!(js.contains("resetComments"));
        assert!(js.contains("registerSaveHandler"));
        assert!(js.contains("registerLoadHandler"));
        assert!(js.contains("registerResetHandler"));
        assert!(js.contains("window.d2f.comments"));
    }

    #[test]
    fn test_comment_icon_svg_constant() {
        assert_eq!(COMMENT_ICON_SVG, "<span class=\"item-comment-icon\"></span>");
    }

    #[test]
    fn test_comment_feature_empty_for_unsupported_elements() {
        let feature = CommentFeature::new();
        let text = DocumentElement::text("Regular text");
        let mut out = String::new();
        let renderer = HtmlRenderer::default_renderer();
        feature.render_element(
            &text,
            0,
            0,
            &DocumentParameters::default(),
            &mut out,
            &renderer,
        );
        assert!(out.is_empty());
    }
}
