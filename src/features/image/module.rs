//! Image vertical slice feature module.

use crate::core::builder::HtmlRenderer;
use crate::core::document::{DocumentElement, DocumentParameters};
use crate::core::feature::Feature;
use crate::core::format::{escape_html_into, push_indent};

/// Embedded image CSS stylesheet.
pub const CSS: &str = include_str!("image.css");

/// Embedded image JavaScript client script.
pub const JS: &str = include_str!("image.js");

/// Image feature renderer handling image embedding and lightbox preview.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
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
    pub const fn new() -> Self {
        Self
    }
}

impl Feature for ImageFeature {
    /// Intercepts the rendering of image elements into the output buffer.
    fn try_render_body(
        &self,
        element: &DocumentElement,
        indent: usize,
        _depth: usize,
        _parameters: &DocumentParameters,
        out: &mut String,
        _renderer: &HtmlRenderer,
    ) -> bool {
        match element {
            DocumentElement::Image { alt, url } => {
                push_indent(out, indent);
                out.push_str("<div class=\"image-item\">\n");

                push_indent(out, indent + 1);
                out.push_str("<img src=\"");
                escape_html_into(out, url);
                out.push_str("\" alt=\"");
                escape_html_into(out, alt);
                out.push_str("\" />\n");

                push_indent(out, indent);
                out.push_str("</div>\n");
                true
            }
            _ => false,
        }
    }

    /// Returns the embedded CSS stylesheet for the image feature.
    fn css(&self) -> Option<&'static str> {
        Some(CSS)
    }

    /// Returns the embedded JavaScript client scripts for the image feature.
    fn javascript(&self) -> &[&'static str] {
        &[JS]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_image_feature_constructor_new() {
        let feature = ImageFeature::new();
        assert_eq!(feature, ImageFeature);
    }

    #[test]
    fn test_image_feature_css() {
        let feature = ImageFeature::new();
        let css = feature.css().expect("image css should exist");
        assert!(css.contains("--image-radius:"));
        assert!(css.contains("--image-border:"));
        assert!(css.contains("--image-fallback-bg:"));
        assert!(css.contains("--image-lightbox-bg:"));
        assert!(css.contains(".image-item"));
        assert!(css.contains(".image-fallback"));
        assert!(css.contains(".image-lightbox"));
    }

    #[test]
    fn test_image_feature_javascript() {
        let feature = ImageFeature::new();
        let js = feature.javascript();
        assert_eq!(js.len(), 1);
        let script = js[0];
        assert!(script.contains("window.d2f.image"));
        assert!(script.contains("openLightbox"));
        assert!(script.contains("closeLightbox"));
        assert!(script.contains("applyImageFallback"));
        assert!(script.contains("image-lightbox"));
    }

    #[test]
    fn test_image_feature_renders_image_element() {
        let feature = ImageFeature::new();
        let element = DocumentElement::image("Architecture Diagram", "assets/arch.png");
        let mut out = String::new();
        let renderer = HtmlRenderer::default_renderer();
        assert!(feature.try_render_body(
            &element,
            1,
            0,
            &DocumentParameters::default(),
            &mut out,
            &renderer,
        ));
        let expected = concat!(
            "  <div class=\"image-item\">\n",
            "    <img src=\"assets/arch.png\" alt=\"Architecture Diagram\" />\n",
            "  </div>\n"
        );
        assert_eq!(out, expected);
    }

    #[test]
    fn test_image_feature_escapes_html_attributes() {
        let feature = ImageFeature::new();
        let element = DocumentElement::image(
            "Picture <with> \"quotes\" & symbols",
            "https://example.com/pic.png?a=1&b=2",
        );
        let mut out = String::new();
        let renderer = HtmlRenderer::default_renderer();
        assert!(feature.try_render_body(
            &element,
            2,
            0,
            &DocumentParameters::default(),
            &mut out,
            &renderer,
        ));
        let expected = concat!(
            "    <div class=\"image-item\">\n",
            "      <img src=\"https://example.com/pic.png?a=1&amp;b=2\" alt=\"Picture &lt;with&gt; &quot;quotes&quot; &amp; symbols\" />\n",
            "    </div>\n"
        );
        assert_eq!(out, expected);
    }

    #[test]
    fn test_image_feature_empty_for_unsupported_elements() {
        let feature = ImageFeature::new();
        let text = DocumentElement::text("Regular text");
        let mut out = String::new();
        let renderer = HtmlRenderer::default_renderer();
        assert!(!feature.try_render_body(
            &text,
            0,
            0,
            &DocumentParameters::default(),
            &mut out,
            &renderer,
        ));
        assert!(out.is_empty());
    }
}
