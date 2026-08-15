//! Bullet list vertical slice feature module.

use std::fmt::Write as _;

use crate::core::document::{DocumentElement, DocumentParameters};
use crate::core::feature::Feature;
use crate::core::format::{format_inline_into, push_indent};

/// Embedded bullet list CSS stylesheet.
pub const CSS: &str = include_str!("bullet_list.css");

/// Bullet list feature renderer handling bullet and unordered list items.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct BulletListFeature;

impl BulletListFeature {
    /// Creates a new bullet list feature instance.
    ///
    /// # Examples
    ///
    /// ```
    /// use doc2flow::features::bullet_list::BulletListFeature;
    ///
    /// let feature = BulletListFeature::new();
    /// ```
    pub const fn new() -> Self {
        Self
    }
}

impl Feature for BulletListFeature {
    /// Converts a bullet list item document element into an HTML string representation.
    ///
    /// # Examples
    ///
    /// ```
    /// use doc2flow::core::document::{DocumentElement, DocumentParameters};
    /// use doc2flow::core::feature::Feature;
    /// use doc2flow::features::bullet_list::BulletListFeature;
    ///
    /// let feature = BulletListFeature::new();
    /// let elem = DocumentElement::bullet_list_item("List item");
    /// let params = DocumentParameters::default();
    /// let html = feature.to_html(&elem, "", 1, 0, &params);
    /// assert!(html.contains("class=\"item item-bullet\""));
    /// ```
    fn to_html(
        &self,
        element: &DocumentElement,
        _content: &str,
        indent: usize,
        depth: usize,
        _parameters: &DocumentParameters,
    ) -> String {
        match element {
            DocumentElement::BulletListItem { content, .. } => {
                let spaces = indent * 2;
                let inner_spaces = (indent + 1) * 2;
                let content_len = content.len() * 2;

                let mut out = String::with_capacity(
                    content_len + _content.len() + spaces * 2 + inner_spaces * 2 + 128,
                );
                push_indent(&mut out, indent);
                out.push_str("<div class=\"item item-bullet\"");
                if depth > 0 {
                    let _ = write!(out, " style=\"--indent: {depth};\"");
                }
                out.push_str(">\n");

                push_indent(&mut out, indent + 1);
                out.push_str("<span class=\"bullet-marker\">&bull;</span>\n");

                push_indent(&mut out, indent + 1);
                out.push_str("<span class=\"bullet-content\">\n");

                for line in content.lines() {
                    push_indent(&mut out, indent + 2);
                    format_inline_into(&mut out, line);
                    out.push('\n');
                }

                push_indent(&mut out, indent + 1);
                out.push_str("</span>\n");

                push_indent(&mut out, indent);
                out.push_str("</div>\n");
                out.push_str(_content);

                out
            }
            _ => String::new(),
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
    fn test_bullet_list_empty_for_unsupported_elements() {
        let feature = BulletListFeature::new();
        let text = DocumentElement::text("Regular text");
        assert_eq!(
            feature.to_html(&text, "", 0, 0, &DocumentParameters::default()),
            ""
        );
    }

    #[test]
    fn test_bullet_list_feature_constructor_new() {
        let feature = BulletListFeature::new();
        assert_eq!(feature, BulletListFeature);
    }

    #[test]
    fn test_bullet_list_feature_css() {
        let feature = BulletListFeature::new();
        let css = feature.css().expect("bullet list css should exist");
        assert!(css.contains("--bullet-marker-color:"));
        assert!(css.contains(".bullet-marker"));
        assert!(css.contains(".bullet-content"));
    }

    #[test]
    fn test_bullet_list_renders_inline_formatting() {
        let feature = BulletListFeature::new();
        let element = DocumentElement::bullet_list_item(
            "Item with **bold**, *italic*, ~~strike~~, and `code` span",
        );
        let html = feature.to_html(&element, "", 0, 0, &DocumentParameters::default());
        let expected = concat!(
            "<div class=\"item item-bullet\">\n",
            "  <span class=\"bullet-marker\">&bull;</span>\n",
            "  <span class=\"bullet-content\">\n",
            "    Item with <strong>bold</strong>, <em>italic</em>, <s>strike</s>, and <code>code</code> span\n",
            "  </span>\n",
            "</div>\n"
        );
        assert_eq!(html, expected);
    }

    #[test]
    fn test_bullet_list_renders_with_indent() {
        let feature = BulletListFeature::new();
        let element = DocumentElement::bullet_list_item("Nested level 2 item");
        let html = feature.to_html(&element, "", 2, 0, &DocumentParameters::default());
        let expected = concat!(
            "    <div class=\"item item-bullet\">\n",
            "      <span class=\"bullet-marker\">&bull;</span>\n",
            "      <span class=\"bullet-content\">\n",
            "        Nested level 2 item\n",
            "      </span>\n",
            "    </div>\n"
        );
        assert_eq!(html, expected);
    }

    #[test]
    fn test_bullet_list_renders_with_depth() {
        let feature = BulletListFeature::new();
        let element = DocumentElement::bullet_list_item("Indented child item");
        let html = feature.to_html(&element, "", 2, 1, &DocumentParameters::default());
        let expected = concat!(
            "    <div class=\"item item-bullet\" style=\"--indent: 1;\">\n",
            "      <span class=\"bullet-marker\">&bull;</span>\n",
            "      <span class=\"bullet-content\">\n",
            "        Indented child item\n",
            "      </span>\n",
            "    </div>\n"
        );
        assert_eq!(html, expected);
    }

    #[test]
    fn test_bullet_list_renders_with_children() {
        let feature = BulletListFeature::new();
        let element = DocumentElement::bullet_list_item("Parent item");
        let child_html = "    <div class=\"item item-bullet\" style=\"--indent: 1;\">\n      <span class=\"bullet-marker\">&bull;</span>\n      <span class=\"bullet-content\">\n        Child item\n      </span>\n    </div>\n";
        let html = feature.to_html(
            &element,
            child_html,
            1,
            0,
            &DocumentParameters::default(),
        );
        let expected_prefix = "  <div class=\"item item-bullet\">\n    <span class=\"bullet-marker\">&bull;</span>\n    <span class=\"bullet-content\">\n      Parent item\n    </span>\n  </div>\n";
        assert_eq!(html, format!("{expected_prefix}{child_html}"));
    }
}
