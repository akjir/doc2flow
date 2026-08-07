//! Image lightbox and embedding feature slice.

use crate::core::feature::{DocumentContext, Feature};

/// Unified image feature slice providing image lightbox modal viewing, zoom styling, and print formatting.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ImageFeature;

impl ImageFeature {
    /// Creates a new image feature instance.
    ///
    /// # Examples
    ///
    /// ```
    /// use doc2flow::features::image::ImageFeature;
    ///
    /// let feature = ImageFeature::new();
    /// ```
    #[inline]
    pub const fn new() -> Self {
        Self
    }
}

impl Feature for ImageFeature {
    /// Returns the unique feature identifier "image".
    #[inline]
    fn name(&self) -> &'static str {
        "image"
    }

    /// Evaluates if the image feature is enabled based on presence of markdown images or html img tags.
    #[inline]
    fn is_enabled(&self, ctx: &DocumentContext) -> bool {
        ctx.raw_markdown.contains("![") || ctx.raw_markdown.contains("<img")
    }

    /// Returns embedded TypeScript client script for lightbox modal and zoom viewing.
    #[inline]
    fn javascript(&self) -> Option<&'static str> {
        Some(include_str!("image.ts"))
    }

    /// Returns embedded CSS styles for image containers, lightbox modal, and print styles.
    #[inline]
    fn css(&self) -> Option<&'static str> {
        Some(include_str!("image.css"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_image_feature_activation_logic() {
        let feature = ImageFeature::new();
        let fm = HashMap::new();

        // 1. Markdown with image syntax: enabled
        let ctx_with_md_img = DocumentContext::new(&fm, "Here is an image: ![Diagram](img/arch.png)");
        assert!(feature.is_enabled(&ctx_with_md_img));

        let ctx_with_html_img =
            DocumentContext::new(&fm, "<p><img src=\"photo.jpg\" alt=\"Photo\"></p>");
        assert!(feature.is_enabled(&ctx_with_html_img));

        // 2. Markdown without images: disabled
        let ctx_no_image = DocumentContext::new(&fm, "# Hello World\nJust normal documentation.");
        assert!(!feature.is_enabled(&ctx_no_image));

        let ctx_empty = DocumentContext::new(&fm, "");
        assert!(!feature.is_enabled(&ctx_empty));
    }

    #[test]
    fn test_image_assets_embedded() {
        let feature = ImageFeature::new();
        assert_eq!(feature.name(), "image");
        let js = feature.javascript().expect("JavaScript must be embedded");
        let css = feature.css().expect("CSS must be embedded");

        assert!(js.contains("openLightbox") || js.contains("open"));
        assert!(js.contains("closeLightbox") || js.contains("close"));
        assert!(css.contains(".lightbox"));
        assert!(css.contains(".doc-body img"));
        assert!(css.contains(".img-item"));
    }
}
