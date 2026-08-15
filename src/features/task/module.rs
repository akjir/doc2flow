//! Task vertical slice feature module.

use std::fmt::Write as _;

use crate::core::document::{DocumentElement, DocumentParameters};
use crate::core::feature::Feature;
use crate::core::format::{format_inline_into, push_indent};

/// Embedded task CSS stylesheet.
pub const CSS: &str = include_str!("task.css");

/// Task feature renderer handling task items and checkboxes.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct TaskFeature;

impl TaskFeature {
    /// Creates a new task feature instance.
    ///
    /// # Examples
    ///
    /// ```
    /// use doc2flow::features::task::TaskFeature;
    ///
    /// let feature = TaskFeature::new();
    /// ```
    pub const fn new() -> Self {
        Self
    }
}

impl Feature for TaskFeature {
    /// Converts a checkbox list item document element into an HTML string representation.
    ///
    /// # Examples
    ///
    /// ```
    /// use doc2flow::core::document::{DocumentElement, DocumentParameters};
    /// use doc2flow::core::feature::Feature;
    /// use doc2flow::features::task::TaskFeature;
    ///
    /// let feature = TaskFeature::new();
    /// let elem = DocumentElement::check_box_item(false, "Todo task");
    /// let params = DocumentParameters::default();
    /// let html = feature.to_html(&elem, "", 1, 0, &params);
    /// assert!(html.contains("class=\"item item-check\""));
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
            DocumentElement::CheckBoxItem {
                checked, content, ..
            } => {
                let spaces = indent * 2;
                let inner_spaces = (indent + 1) * 2;
                let content_len = content.len() * 2;

                let mut out = String::with_capacity(
                    content_len + _content.len() + spaces * 2 + inner_spaces * 2 + 160,
                );
                push_indent(&mut out, indent);

                if *checked {
                    out.push_str("<div class=\"item item-check checked\"");
                } else {
                    out.push_str("<div class=\"item item-check\"");
                }
                if depth > 0 {
                    let _ = write!(out, " style=\"--indent: {depth};\"");
                }
                out.push_str(">\n");

                push_indent(&mut out, indent + 1);
                out.push_str("<span class=\"check-marker\">\n");
                push_indent(&mut out, indent + 2);
                if *checked {
                    out.push_str("<input type=\"checkbox\" class=\"check-box\" checked />\n");
                } else {
                    out.push_str("<input type=\"checkbox\" class=\"check-box\" />\n");
                }
                push_indent(&mut out, indent + 1);
                out.push_str("</span>\n");

                push_indent(&mut out, indent + 1);
                out.push_str("<span class=\"check-content\">\n");

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

    /// Returns the embedded CSS stylesheet for the task feature.
    fn css(&self) -> Option<&'static str> {
        Some(CSS)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_task_feature_css() {
        let feature = TaskFeature::new();
        let css = feature.css().expect("task css should exist");
        assert!(css.contains("--check-marker-size:"));
        assert!(css.contains("--check-accent-color:"));
        assert!(css.contains(".check-marker"));
        assert!(css.contains(".check-box"));
        assert!(css.contains(".check-content"));
        assert!(css.contains(".item-check.checked"));
    }

    #[test]
    fn test_task_feature_javascript() {
        let feature = TaskFeature::new();
        assert!(feature.javascript().is_empty());
    }

    #[test]
    fn test_task_renders_unchecked() {
        let feature = TaskFeature::new();
        let element = DocumentElement::check_box_item(false, "Pending task");
        let html = feature.to_html(&element, "", 1, 0, &DocumentParameters::default());
        let expected = concat!(
            "  <div class=\"item item-check\">\n",
            "    <span class=\"check-marker\">\n",
            "      <input type=\"checkbox\" class=\"check-box\" />\n",
            "    </span>\n",
            "    <span class=\"check-content\">\n",
            "      Pending task\n",
            "    </span>\n",
            "  </div>\n"
        );
        assert_eq!(html, expected);
    }

    #[test]
    fn test_task_renders_checked() {
        let feature = TaskFeature::new();
        let element = DocumentElement::check_box_item(true, "Completed task");
        let html = feature.to_html(&element, "", 1, 0, &DocumentParameters::default());
        let expected = concat!(
            "  <div class=\"item item-check checked\">\n",
            "    <span class=\"check-marker\">\n",
            "      <input type=\"checkbox\" class=\"check-box\" checked />\n",
            "    </span>\n",
            "    <span class=\"check-content\">\n",
            "      Completed task\n",
            "    </span>\n",
            "  </div>\n"
        );
        assert_eq!(html, expected);
    }

    #[test]
    fn test_task_renders_inline_formatting() {
        let feature = TaskFeature::new();
        let element = DocumentElement::check_box_item(
            true,
            "Item with **bold**, *italic*, ~~strike~~, `code`, and [link](https://example.com) span",
        );
        let html = feature.to_html(&element, "", 0, 0, &DocumentParameters::default());
        let expected = concat!(
            "<div class=\"item item-check checked\">\n",
            "  <span class=\"check-marker\">\n",
            "    <input type=\"checkbox\" class=\"check-box\" checked />\n",
            "  </span>\n",
            "  <span class=\"check-content\">\n",
            "    Item with <strong>bold</strong>, <em>italic</em>, <s>strike</s>, <code>code</code>, and <a href=\"https://example.com\">link</a> span\n",
            "  </span>\n",
            "</div>\n"
        );
        assert_eq!(html, expected);
    }

    #[test]
    fn test_task_renders_with_depth() {
        let feature = TaskFeature::new();
        let element = DocumentElement::check_box_item(false, "Sub task");
        let html = feature.to_html(&element, "", 2, 1, &DocumentParameters::default());
        let expected = concat!(
            "    <div class=\"item item-check\" style=\"--indent: 1;\">\n",
            "      <span class=\"check-marker\">\n",
            "        <input type=\"checkbox\" class=\"check-box\" />\n",
            "      </span>\n",
            "      <span class=\"check-content\">\n",
            "        Sub task\n",
            "      </span>\n",
            "    </div>\n"
        );
        assert_eq!(html, expected);

        let checked = DocumentElement::check_box_item(true, "Checked sub task");
        let checked_html = feature.to_html(&checked, "", 2, 2, &DocumentParameters::default());
        let checked_expected = concat!(
            "    <div class=\"item item-check checked\" style=\"--indent: 2;\">\n",
            "      <span class=\"check-marker\">\n",
            "        <input type=\"checkbox\" class=\"check-box\" checked />\n",
            "      </span>\n",
            "      <span class=\"check-content\">\n",
            "        Checked sub task\n",
            "      </span>\n",
            "    </div>\n"
        );
        assert_eq!(checked_html, checked_expected);
    }

    #[test]
    fn test_task_renders_with_children() {
        let feature = TaskFeature::new();
        let element = DocumentElement::check_box_item(false, "Parent task");
        let child_html = "    <div class=\"item item-check\" style=\"--indent: 1;\">\n      <span class=\"check-marker\">\n        <input type=\"checkbox\" class=\"check-box\" />\n      </span>\n      <span class=\"check-content\">\n        Child task\n      </span>\n    </div>\n";
        let html = feature.to_html(
            &element,
            child_html,
            1,
            0,
            &DocumentParameters::default(),
        );
        let expected_prefix = "  <div class=\"item item-check\">\n    <span class=\"check-marker\">\n      <input type=\"checkbox\" class=\"check-box\" />\n    </span>\n    <span class=\"check-content\">\n      Parent task\n    </span>\n  </div>\n";
        assert_eq!(html, format!("{expected_prefix}{child_html}"));
    }
}
