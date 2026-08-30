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

/// Maximum buffer capacity for base-26 alphabetic representation.
const MAX_ALPHA_DIGITS: usize = 16;

/// Converts a 1-based number to alphabetic representation without heap allocations.
fn write_alpha(mut n: usize, out: &mut String) {
    if n == 0 {
        out.push('a');
        return;
    }
    if n <= 26 {
        out.push((b'a' + (n - 1) as u8) as char);
        return;
    }
    let mut buf = [0u8; MAX_ALPHA_DIGITS];
    let mut pos = MAX_ALPHA_DIGITS;
    while n > 0 {
        n -= 1;
        pos -= 1;
        debug_assert!(pos < MAX_ALPHA_DIGITS, "alpha buffer position out of bounds");
        buf[pos] = b'a' + (n % 26) as u8;
        n /= 26;
    }
    if let Ok(s) = std::str::from_utf8(&buf[pos..]) {
        out.push_str(s);
    }
}

/// Converts a 1-based number to lowercase Roman numerals without heap allocations.
fn write_roman(n: usize, out: &mut String) {
    if n == 0 {
        out.push('i');
        return;
    }
    if (1..=10).contains(&n) {
        const ROMANS: [&str; 10] = ["i", "ii", "iii", "iv", "v", "vi", "vii", "viii", "ix", "x"];
        out.push_str(ROMANS[n - 1]);
        return;
    }
    const ROMAN_MAPPING: [(usize, &str); 13] = [
        (1000, "m"),
        (900, "cm"),
        (500, "d"),
        (400, "cd"),
        (100, "c"),
        (90, "xc"),
        (50, "l"),
        (40, "xl"),
        (10, "x"),
        (9, "ix"),
        (5, "v"),
        (4, "iv"),
        (1, "i"),
    ];
    let mut num = n;
    for (val, sym) in ROMAN_MAPPING {
        while num >= val {
            out.push_str(sym);
            num -= val;
        }
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
            match depth % 3 {
                0 => {
                    let _ = write!(out, "{position}.");
                }
                1 => {
                    write_alpha(*position, out);
                    out.push('.');
                }
                _ => {
                    write_roman(*position, out);
                    out.push('.');
                }
            }
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

            if parameters.comments {
                push_indent(out, indent + 1);
                out.push_str(crate::features::comment::COMMENT_ICON_SVG);
                out.push('\n');
            }

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
    use crate::features::comment::COMMENT_ICON_SVG;

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
        let expected = format!(
            concat!(
                "  <div class=\"item order-item item-selectable\">\n",
                "    <span class=\"order-marker\">1.</span>\n",
                "    <span class=\"order-content\">\n",
                "      First numbered item\n",
                "    </span>\n",
                "    {COMMENT_ICON_SVG}\n",
                "  </div>\n"
            ),
            COMMENT_ICON_SVG = COMMENT_ICON_SVG
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
        let expected = format!(
            concat!(
                "<div class=\"item order-item item-selectable\">\n",
                "  <span class=\"order-marker\">2.</span>\n",
                "  <span class=\"order-content\">\n",
                "    Step with <strong>bold</strong>, <em>italic</em>, <s>strike</s>, <code>code</code>, <a href=\"https://example.com\">link</a>, and &lt;special&gt; &amp; characters\n",
                "  </span>\n",
                "  {COMMENT_ICON_SVG}\n",
                "</div>\n"
            ),
            COMMENT_ICON_SVG = COMMENT_ICON_SVG
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
        let expected = format!(
            concat!(
                "    <div class=\"item order-item item-selectable\" style=\"--indent: 1;\">\n",
                "      <span class=\"order-marker\">a.</span>\n",
                "      <span class=\"order-content\">\n",
                "        Sub-step item\n",
                "      </span>\n",
                "      {COMMENT_ICON_SVG}\n",
                "    </div>\n"
            ),
            COMMENT_ICON_SVG = COMMENT_ICON_SVG
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
        let expected = format!(
            concat!(
                "  <div class=\"item order-item item-selectable\">\n",
                "    <span class=\"order-marker\">1.</span>\n",
                "    <span class=\"order-content\">\n",
                "      Parent order\n",
                "    </span>\n",
                "    {COMMENT_ICON_SVG}\n",
                "  </div>\n",
                "  <div class=\"item order-item item-selectable\" style=\"--indent: 1;\">\n",
                "    <span class=\"order-marker\">a.</span>\n",
                "    <span class=\"order-content\">\n",
                "      Child item\n",
                "    </span>\n",
                "    {COMMENT_ICON_SVG}\n",
                "  </div>\n",
            ),
            COMMENT_ICON_SVG = COMMENT_ICON_SVG
        );
        assert_eq!(out, expected);
    }

    #[test]
    fn test_ordered_hierarchical_numbering_all_depth_levels() {
        let feature = OrderedFeature::new();
        let renderer = HtmlRenderer::default_renderer();
        let params = DocumentParameters::default();

        let cases = [
            (0, 1, "1."),
            (0, 5, "5."),
            (1, 1, "a."),
            (1, 2, "b."),
            (1, 26, "z."),
            (2, 1, "i."),
            (2, 2, "ii."),
            (2, 3, "iii."),
            (2, 4, "iv."),
            (2, 5, "v."),
            (2, 9, "ix."),
            (2, 10, "x."),
            (3, 1, "1."),
            (4, 1, "a."),
            (5, 1, "i."),
            (6, 1, "1."),
        ];

        for (depth, pos, expected_marker) in cases {
            let mut out = String::new();
            let element = DocumentElement::ordered_list_item(pos, "Content");
            feature.render_element(&element, 0, depth, &params, &mut out, &renderer);
            assert!(
                out.contains(&format!("<span class=\"order-marker\">{expected_marker}</span>")),
                "Expected marker '{expected_marker}' for depth {depth} and position {pos}, got:\n{out}"
            );
        }
    }

    #[test]
    fn test_ordered_alphabetic_conversion_edge_cases() {
        let mut buf = String::new();
        write_alpha(0, &mut buf);
        assert_eq!(buf, "a");

        buf.clear();
        write_alpha(1, &mut buf);
        assert_eq!(buf, "a");

        buf.clear();
        write_alpha(26, &mut buf);
        assert_eq!(buf, "z");

        buf.clear();
        write_alpha(27, &mut buf);
        assert_eq!(buf, "aa");

        buf.clear();
        write_alpha(28, &mut buf);
        assert_eq!(buf, "ab");

        buf.clear();
        write_alpha(52, &mut buf);
        assert_eq!(buf, "az");

        buf.clear();
        write_alpha(53, &mut buf);
        assert_eq!(buf, "ba");

        buf.clear();
        write_alpha(702, &mut buf);
        assert_eq!(buf, "zz");

        buf.clear();
        write_alpha(703, &mut buf);
        assert_eq!(buf, "aaa");
    }

    #[test]
    fn test_ordered_roman_conversion_edge_cases() {
        let cases = [
            (0, "i"),
            (1, "i"),
            (2, "ii"),
            (3, "iii"),
            (4, "iv"),
            (5, "v"),
            (6, "vi"),
            (7, "vii"),
            (8, "viii"),
            (9, "ix"),
            (10, "x"),
            (14, "xiv"),
            (40, "xl"),
            (90, "xc"),
            (100, "c"),
            (400, "cd"),
            (500, "d"),
            (900, "cm"),
            (1000, "m"),
            (1984, "mcmlxxxiv"),
            (2026, "mmxxvi"),
        ];

        for (num, expected) in cases {
            let mut buf = String::new();
            write_roman(num, &mut buf);
            assert_eq!(buf, expected, "Failed for Roman conversion of {num}");
        }
    }

    #[test]
    fn test_ordered_renders_deeply_nested_hierarchy() {
        let feature = OrderedFeature::new();
        let renderer = HtmlRenderer::default_renderer();
        let params = DocumentParameters::default();

        let mut root = DocumentElement::ordered_list_item(1, "Top level");
        let mut level1 = DocumentElement::ordered_list_item(1, "Level 1 alpha");
        let level2 = DocumentElement::ordered_list_item(1, "Level 2 roman");
        level1.push_child(level2).unwrap();
        root.push_child(level1).unwrap();

        let mut out = String::new();
        feature.render_element(&root, 0, 0, &params, &mut out, &renderer);

        assert!(out.contains("<span class=\"order-marker\">1.</span>"));
        assert!(out.contains("<span class=\"order-marker\">a.</span>"));
        assert!(out.contains("<span class=\"order-marker\">i.</span>"));
        assert!(out.contains("style=\"--indent: 1;\""));
        assert!(out.contains("style=\"--indent: 2;\""));
    }
}
