//! Ordered list vertical slice feature module.

use std::fmt::Write as _;

use crate::core::document::{DocumentElement, DocumentElementId, DocumentParameters};
use crate::core::feature::FeatureModule;
use crate::core::format::{format_inline_into, push_indent};
use crate::core::renderer::{DocumentElementRenderer, HtmlRenderer};

/// Embedded ordered list CSS stylesheet.
pub const CSS: &str = include_str!("ordered.css");

/// Supported document element identifiers for ordered list items.
const ORDERED_SUPPORTED: [DocumentElementId; 1] = [DocumentElementId::OrderedListItem];

/// Ordered list feature renderer handling numbered and ordered list items.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct OrderedFeature;

impl OrderedFeature {
    /// Creates a new ordered list feature instance.
    ///
    /// # Examples
    ///
    /// ```
    /// use doc2flow::features::ordered::OrderedFeature;
    ///
    /// let feature = OrderedFeature::new();
    /// ```
    #[must_use]
    pub fn new() -> Self {
        Self
    }
}

impl DocumentElementRenderer for OrderedFeature {
    fn supported(&self) -> &[DocumentElementId] {
        &ORDERED_SUPPORTED
    }

    fn render_element(
        &self,
        element: &DocumentElement,
        indent: usize,
        depth: usize,
        parameters: &DocumentParameters,
        out: &mut String,
        renderer: &HtmlRenderer,
    ) {
        if let DocumentElement::OrderedListItem {
            content,
            position,
            children,
        } = element
        {
            push_indent(out, indent);
            out.push_str("<div class=\"item order-item item-selectable\"");
            if depth > 0 {
                let _ = write!(out, " style=\"--indent: {depth};\"");
            }
            out.push_str(">\n");

            push_indent(out, indent + 1);
            out.push_str("<span class=\"order-marker\">");
            let _ = write!(out, "{position}.");
            out.push_str("</span>\n");

            push_indent(out, indent + 1);
            out.push_str("<span class=\"order-content\">\n");

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
        }
    }
}

impl FeatureModule for OrderedFeature {
    fn name(&self) -> &'static str {
        "ordered"
    }

    fn css(&self) -> Option<&'static str> {
        Some(CSS)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ordered_empty_for_unsupported_elements() {
        let feature = OrderedFeature::new();
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

    #[test]
    fn test_ordered_feature_constructor_new() {
        let feature = OrderedFeature::new();
        assert_eq!(feature, OrderedFeature);
    }

    #[test]
    fn test_ordered_feature_css() {
        let feature = OrderedFeature::new();
        let css = feature.css().expect("ordered list css should exist");
        assert!(css.contains("--order-marker-color:"));
        assert!(css.contains(".order-marker"));
        assert!(css.contains(".order-content"));
    }

    #[test]
    fn test_ordered_renders_root() {
        let feature = OrderedFeature::new();
        let element = DocumentElement::ordered_list_item(1, "First numbered item");
        let mut out = String::new();
        let renderer = HtmlRenderer::default_renderer();
        feature.render_element(
            &element,
            1,
            0,
            &DocumentParameters::default(),
            &mut out,
            &renderer,
        );
        let expected = concat!(
            "  <div class=\"item order-item item-selectable\">\n",
            "    <span class=\"order-marker\">1.</span>\n",
            "    <span class=\"order-content\">\n",
            "      First numbered item\n",
            "    </span>\n",
            "  </div>\n"
        );
        assert_eq!(out, expected);
    }

    #[test]
    fn test_ordered_renders_inline_formatting() {
        let feature = OrderedFeature::new();
        let element = DocumentElement::ordered_list_item(
            2,
            "Step with **bold**, *italic*, ~~strike~~, `code`, [link](https://example.com), and <special> & characters",
        );
        let mut out = String::new();
        let renderer = HtmlRenderer::default_renderer();
        feature.render_element(
            &element,
            0,
            0,
            &DocumentParameters::default(),
            &mut out,
            &renderer,
        );
        let expected = concat!(
            "<div class=\"item order-item item-selectable\">\n",
            "  <span class=\"order-marker\">2.</span>\n",
            "  <span class=\"order-content\">\n",
            "    Step with <strong>bold</strong>, <em>italic</em>, <s>strike</s>, <code>code</code>, <a href=\"https://example.com\">link</a>, and &lt;special&gt; &amp; characters\n",
            "  </span>\n",
            "</div>\n"
        );
        assert_eq!(out, expected);
    }

    #[test]
    fn test_ordered_renders_with_depth() {
        let feature = OrderedFeature::new();
        let element = DocumentElement::ordered_list_item(1, "Sub-step item");
        let mut out = String::new();
        let renderer = HtmlRenderer::default_renderer();
        feature.render_element(
            &element,
            2,
            1,
            &DocumentParameters::default(),
            &mut out,
            &renderer,
        );
        let expected = concat!(
            "    <div class=\"item order-item item-selectable\" style=\"--indent: 1;\">\n",
            "      <span class=\"order-marker\">1.</span>\n",
            "      <span class=\"order-content\">\n",
            "        Sub-step item\n",
            "      </span>\n",
            "    </div>\n"
        );
        assert_eq!(out, expected);
    }

    #[test]
    fn test_ordered_renders_with_children() {
        let feature = OrderedFeature::new();
        let mut element = DocumentElement::ordered_list_item(1, "Parent order");
        let child = DocumentElement::ordered_list_item(1, "Child item");
        element.push_child(child).unwrap();

        let mut out = String::new();
        let renderer = HtmlRenderer::default_renderer();
        feature.render_element(
            &element,
            1,
            0,
            &DocumentParameters::default(),
            &mut out,
            &renderer,
        );
        let expected = concat!(
            "  <div class=\"item order-item item-selectable\">\n",
            "    <span class=\"order-marker\">1.</span>\n",
            "    <span class=\"order-content\">\n",
            "      Parent order\n",
            "    </span>\n",
            "  </div>\n",
            "  <div class=\"item order-item item-selectable\" style=\"--indent: 1;\">\n",
            "    <span class=\"order-marker\">1.</span>\n",
            "    <span class=\"order-content\">\n",
            "      Child item\n",
            "    </span>\n",
            "  </div>\n",
        );
        assert_eq!(out, expected);
    }
}
