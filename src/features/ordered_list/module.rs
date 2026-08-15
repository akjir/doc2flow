//! Ordered list vertical slice feature module.

use std::fmt::Write as _;

use crate::core::document::{DocumentElement, DocumentParameters};
use crate::core::feature::Feature;
use crate::core::format::{format_inline_into, push_indent};

/// Embedded ordered list CSS stylesheet.
pub const CSS: &str = include_str!("ordered_list.css");

/// Ordered list feature renderer handling numbered and ordered list items.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct OrderedListFeature;

impl OrderedListFeature {
    /// Creates a new ordered list feature instance.
    ///
    /// # Examples
    ///
    /// ```
    /// use doc2flow::features::ordered_list::OrderedListFeature;
    ///
    /// let feature = OrderedListFeature::new();
    /// ```
    pub const fn new() -> Self {
        Self
    }
}

impl Feature for OrderedListFeature {
    /// Converts an ordered list item document element into an HTML string representation.
    ///
    /// # Examples
    ///
    /// ```
    /// use doc2flow::core::document::{DocumentElement, DocumentParameters};
    /// use doc2flow::core::feature::Feature;
    /// use doc2flow::features::ordered_list::OrderedListFeature;
    ///
    /// let feature = OrderedListFeature::new();
    /// let elem = DocumentElement::ordered_list_item(1, "First item");
    /// let params = DocumentParameters::default();
    /// let html = feature.to_html(&elem, "", 1, 0, &params);
    /// assert!(html.contains("class=\"item order-item\""));
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
            DocumentElement::OrderedListItem {
                content, position, ..
            } => {
                let spaces = indent * 2;
                let inner_spaces = (indent + 1) * 2;
                let content_len = content.len() * 2;

                let mut out = String::with_capacity(
                    content_len + _content.len() + spaces * 2 + inner_spaces * 2 + 128,
                );
                push_indent(&mut out, indent);
                out.push_str("<div class=\"item order-item\"");
                if depth > 0 {
                    let _ = write!(out, " style=\"--indent: {depth};\"");
                }
                out.push_str(">\n");

                push_indent(&mut out, indent + 1);
                out.push_str("<span class=\"order-marker\">");
                let _ = write!(out, "{position}.");
                out.push_str("</span>\n");

                push_indent(&mut out, indent + 1);
                out.push_str("<span class=\"order-content\">\n");

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

    /// Returns the embedded CSS stylesheet for the ordered list feature.
    fn css(&self) -> Option<&'static str> {
        Some(CSS)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ordered_list_empty_for_unsupported_elements() {
        let feature = OrderedListFeature::new();
        let text = DocumentElement::text("Regular text");
        assert_eq!(
            feature.to_html(&text, "", 0, 0, &DocumentParameters::default()),
            ""
        );
    }

    #[test]
    fn test_ordered_list_feature_constructor_new() {
        let feature = OrderedListFeature::new();
        assert_eq!(feature, OrderedListFeature);
    }

    #[test]
    fn test_ordered_list_feature_css() {
        let feature = OrderedListFeature::new();
        let css = feature.css().expect("ordered list css should exist");
        assert!(css.contains("--order-marker-color:"));
        assert!(css.contains(".order-marker"));
        assert!(css.contains(".order-content"));
    }

    #[test]
    fn test_ordered_list_renders_root() {
        let feature = OrderedListFeature::new();
        let element = DocumentElement::ordered_list_item(1, "First numbered item");
        let html = feature.to_html(&element, "", 1, 0, &DocumentParameters::default());
        let expected = concat!(
            "  <div class=\"item order-item\">\n",
            "    <span class=\"order-marker\">1.</span>\n",
            "    <span class=\"order-content\">\n",
            "      First numbered item\n",
            "    </span>\n",
            "  </div>\n"
        );
        assert_eq!(html, expected);
    }

    #[test]
    fn test_ordered_list_renders_inline_formatting() {
        let feature = OrderedListFeature::new();
        let element = DocumentElement::ordered_list_item(
            2,
            "Step with **bold**, *italic*, ~~strike~~, `code`, [link](https://example.com), and <special> & characters",
        );
        let html = feature.to_html(&element, "", 0, 0, &DocumentParameters::default());
        let expected = concat!(
            "<div class=\"item order-item\">\n",
            "  <span class=\"order-marker\">2.</span>\n",
            "  <span class=\"order-content\">\n",
            "    Step with <strong>bold</strong>, <em>italic</em>, <s>strike</s>, <code>code</code>, <a href=\"https://example.com\">link</a>, and &lt;special&gt; &amp; characters\n",
            "  </span>\n",
            "</div>\n"
        );
        assert_eq!(html, expected);
    }

    #[test]
    fn test_ordered_list_renders_with_depth() {
        let feature = OrderedListFeature::new();
        let element = DocumentElement::ordered_list_item(1, "Sub-step item");
        let html = feature.to_html(&element, "", 2, 1, &DocumentParameters::default());
        let expected = concat!(
            "    <div class=\"item order-item\" style=\"--indent: 1;\">\n",
            "      <span class=\"order-marker\">1.</span>\n",
            "      <span class=\"order-content\">\n",
            "        Sub-step item\n",
            "      </span>\n",
            "    </div>\n"
        );
        assert_eq!(html, expected);
    }

    #[test]
    fn test_ordered_list_renders_with_children() {
        let feature = OrderedListFeature::new();
        let element = DocumentElement::ordered_list_item(1, "Parent order");
        let child_html = "    <div class=\"item order-item\" style=\"--indent: 1;\">\n      <span class=\"order-marker\">1.</span>\n      <span class=\"order-content\">\n        Child item\n      </span>\n    </div>\n";
        let html = feature.to_html(
            &element,
            child_html,
            1,
            0,
            &DocumentParameters::default(),
        );
        let expected_prefix = "  <div class=\"item order-item\">\n    <span class=\"order-marker\">1.</span>\n    <span class=\"order-content\">\n      Parent order\n    </span>\n  </div>\n";
        assert_eq!(html, format!("{expected_prefix}{child_html}"));
    }
}
