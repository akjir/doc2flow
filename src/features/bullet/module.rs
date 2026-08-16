//! Bullet list vertical slice feature module.

use std::fmt::Write as _;

use crate::core::builder::HtmlRenderer;
use crate::core::document::{DocumentElement, DocumentParameters};
use crate::core::feature::Feature;
use crate::core::format::{format_inline_into, push_indent};

/// Embedded bullet list CSS stylesheet.
pub const CSS: &str = include_str!("bullet.css");

/// Bullet list feature renderer handling bullet and unordered list items.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct BulletFeature;

impl BulletFeature {
    /// Creates a new bullet list feature instance.
    ///
    /// # Examples
    ///
    /// ```
    /// use doc2flow::features::bullet::BulletFeature;
    ///
    /// let feature = BulletFeature::new();
    /// ```
    pub const fn new() -> Self {
        Self
    }
}

impl Feature for BulletFeature {
    /// Intercepts the rendering of bullet list elements into the output buffer.
    fn try_render_body(
        &self,
        element: &DocumentElement,
        indent: usize,
        depth: usize,
        parameters: &DocumentParameters,
        out: &mut String,
        renderer: &HtmlRenderer,
    ) -> bool {
        match element {
            DocumentElement::BulletListItem { content, children } => {
                push_indent(out, indent);
                out.push_str("<div class=\"item bullet-item\"");
                if depth > 0 {
                    let _ = write!(out, " style=\"--indent: {depth};\"");
                }
                out.push_str(">\n");

                push_indent(out, indent + 1);
                out.push_str("<span class=\"bullet-marker\">&bull;</span>\n");

                push_indent(out, indent + 1);
                out.push_str("<span class=\"bullet-content\">\n");

                for line in content.lines() {
                    push_indent(out, indent + 2);
                    format_inline_into(out, line);
                    out.push('\n');
                }

                push_indent(out, indent + 1);
                out.push_str("</span>\n");

                push_indent(out, indent);
                out.push_str("</div>\n");

                renderer.render_children(children, indent, depth + 1, parameters, out);
                true
            }
            _ => false,
        }
    }

    /// Returns the embedded CSS stylesheet for the bullet list feature.
    fn css(&self) -> Option<&'static str> {
        Some(CSS)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bullet_empty_for_unsupported_elements() {
        let feature = BulletFeature::new();
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

    #[test]
    fn test_bullet_feature_constructor_new() {
        let feature = BulletFeature::new();
        assert_eq!(feature, BulletFeature);
    }

    #[test]
    fn test_bullet_feature_css() {
        let feature = BulletFeature::new();
        let css = feature.css().expect("bullet list css should exist");
        assert!(css.contains("--bullet-marker-color:"));
        assert!(css.contains(".bullet-marker"));
        assert!(css.contains(".bullet-content"));
    }

    #[test]
    fn test_bullet_renders_inline_formatting() {
        let feature = BulletFeature::new();
        let element = DocumentElement::bullet_list_item(
            "Item with **bold**, *italic*, ~~strike~~, `code`, and [link](https://example.com) span",
        );
        let mut out = String::new();
        let renderer = HtmlRenderer::default_renderer();
        assert!(feature.try_render_body(
            &element,
            0,
            0,
            &DocumentParameters::default(),
            &mut out,
            &renderer,
        ));
        let expected = concat!(
            "<div class=\"item bullet-item\">\n",
            "  <span class=\"bullet-marker\">&bull;</span>\n",
            "  <span class=\"bullet-content\">\n",
            "    Item with <strong>bold</strong>, <em>italic</em>, <s>strike</s>, <code>code</code>, and <a href=\"https://example.com\">link</a> span\n",
            "  </span>\n",
            "</div>\n"
        );
        assert_eq!(out, expected);
    }

    #[test]
    fn test_bullet_renders_with_indent() {
        let feature = BulletFeature::new();
        let element = DocumentElement::bullet_list_item("Nested level 2 item");
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
            "    <div class=\"item bullet-item\">\n",
            "      <span class=\"bullet-marker\">&bull;</span>\n",
            "      <span class=\"bullet-content\">\n",
            "        Nested level 2 item\n",
            "      </span>\n",
            "    </div>\n"
        );
        assert_eq!(out, expected);
    }

    #[test]
    fn test_bullet_renders_with_depth() {
        let feature = BulletFeature::new();
        let element = DocumentElement::bullet_list_item("Indented child item");
        let mut out = String::new();
        let renderer = HtmlRenderer::default_renderer();
        assert!(feature.try_render_body(
            &element,
            2,
            1,
            &DocumentParameters::default(),
            &mut out,
            &renderer,
        ));
        let expected = concat!(
            "    <div class=\"item bullet-item\" style=\"--indent: 1;\">\n",
            "      <span class=\"bullet-marker\">&bull;</span>\n",
            "      <span class=\"bullet-content\">\n",
            "        Indented child item\n",
            "      </span>\n",
            "    </div>\n"
        );
        assert_eq!(out, expected);
    }

    #[test]
    fn test_bullet_renders_with_children() {
        let feature = BulletFeature::new();
        let mut element = DocumentElement::bullet_list_item("Parent item");
        let child = DocumentElement::bullet_list_item("Child item");
        element.push_child(child).unwrap();

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
            "  <div class=\"item bullet-item\">\n",
            "    <span class=\"bullet-marker\">&bull;</span>\n",
            "    <span class=\"bullet-content\">\n",
            "      Parent item\n",
            "    </span>\n",
            "  </div>\n",
            "  <div class=\"item bullet-item\" style=\"--indent: 1;\">\n",
            "    <span class=\"bullet-marker\">&bull;</span>\n",
            "    <span class=\"bullet-content\">\n",
            "      Child item\n",
            "    </span>\n",
            "  </div>\n",
        );
        assert_eq!(out, expected);
    }
}
