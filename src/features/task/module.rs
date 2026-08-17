//! Task vertical slice feature module.

use std::fmt::Write as _;

use crate::core::document::{DocumentElement, DocumentElementId, DocumentParameters};
use crate::core::feature::FeatureModule;
use crate::core::format::{format_inline_into, push_indent};
use crate::core::renderer::{DocumentElementRenderer, HtmlRenderer};

/// Embedded task CSS stylesheet.
pub const CSS: &str = include_str!("task.css");

/// Supported document element identifiers for checkbox task items.
const TASK_SUPPORTED: [DocumentElementId; 1] = [DocumentElementId::CheckBoxItem];

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
    #[must_use]
    pub fn new() -> Self {
        Self
    }
}

impl DocumentElementRenderer for TaskFeature {
    fn supported(&self) -> &[DocumentElementId] {
        &TASK_SUPPORTED
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
        if let DocumentElement::CheckBoxItem {
            checked,
            content,
            children,
        } = element
        {
            push_indent(out, indent);

            if *checked {
                out.push_str("<div class=\"item check-item item-selectable checked\"");
            } else {
                out.push_str("<div class=\"item check-item item-selectable\"");
            }
            if depth > 0 {
                let _ = write!(out, " style=\"--indent: {depth};\"");
            }
            out.push_str(">\n");

            push_indent(out, indent + 1);
            out.push_str("<span class=\"check-marker\">\n");
            push_indent(out, indent + 2);
            if *checked {
                out.push_str("<input type=\"checkbox\" class=\"check-box\" checked />\n");
            } else {
                out.push_str("<input type=\"checkbox\" class=\"check-box\" />\n");
            }
            push_indent(out, indent + 1);
            out.push_str("</span>\n");

            push_indent(out, indent + 1);
            out.push_str("<span class=\"check-content\">\n");

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

impl FeatureModule for TaskFeature {
    fn name(&self) -> &'static str {
        "task"
    }

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
        assert!(css.contains(".check-item.checked"));
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
            "  <div class=\"item check-item item-selectable\">\n",
            "    <span class=\"check-marker\">\n",
            "      <input type=\"checkbox\" class=\"check-box\" />\n",
            "    </span>\n",
            "    <span class=\"check-content\">\n",
            "      Pending task\n",
            "    </span>\n",
            "  </div>\n"
        );
        assert_eq!(out, expected);
    }

    #[test]
    fn test_task_renders_checked() {
        let feature = TaskFeature::new();
        let element = DocumentElement::check_box_item(true, "Completed task");
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
            "  <div class=\"item check-item item-selectable checked\">\n",
            "    <span class=\"check-marker\">\n",
            "      <input type=\"checkbox\" class=\"check-box\" checked />\n",
            "    </span>\n",
            "    <span class=\"check-content\">\n",
            "      Completed task\n",
            "    </span>\n",
            "  </div>\n"
        );
        assert_eq!(out, expected);
    }

    #[test]
    fn test_task_renders_inline_formatting() {
        let feature = TaskFeature::new();
        let element = DocumentElement::check_box_item(
            true,
            "Item with **bold**, *italic*, ~~strike~~, `code`, and [link](https://example.com) span",
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
            "<div class=\"item check-item item-selectable checked\">\n",
            "  <span class=\"check-marker\">\n",
            "    <input type=\"checkbox\" class=\"check-box\" checked />\n",
            "  </span>\n",
            "  <span class=\"check-content\">\n",
            "    Item with <strong>bold</strong>, <em>italic</em>, <s>strike</s>, <code>code</code>, and <a href=\"https://example.com\">link</a> span\n",
            "  </span>\n",
            "</div>\n"
        );
        assert_eq!(out, expected);
    }

    #[test]
    fn test_task_renders_with_depth() {
        let feature = TaskFeature::new();
        let element = DocumentElement::check_box_item(false, "Sub task");
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
            "    <div class=\"item check-item item-selectable\" style=\"--indent: 1;\">\n",
            "      <span class=\"check-marker\">\n",
            "        <input type=\"checkbox\" class=\"check-box\" />\n",
            "      </span>\n",
            "      <span class=\"check-content\">\n",
            "        Sub task\n",
            "      </span>\n",
            "    </div>\n"
        );
        assert_eq!(out, expected);

        let checked = DocumentElement::check_box_item(true, "Checked sub task");
        let mut checked_out = String::new();
        feature.render_element(
            &checked,
            2,
            2,
            &DocumentParameters::default(),
            &mut checked_out,
            &renderer,
        );
        let checked_expected = concat!(
            "    <div class=\"item check-item item-selectable checked\" style=\"--indent: 2;\">\n",
            "      <span class=\"check-marker\">\n",
            "        <input type=\"checkbox\" class=\"check-box\" checked />\n",
            "      </span>\n",
            "      <span class=\"check-content\">\n",
            "        Checked sub task\n",
            "      </span>\n",
            "    </div>\n"
        );
        assert_eq!(checked_out, checked_expected);
    }

    #[test]
    fn test_task_renders_with_children() {
        let feature = TaskFeature::new();
        let mut element = DocumentElement::check_box_item(false, "Parent task");
        let child = DocumentElement::check_box_item(false, "Child task");
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
            "  <div class=\"item check-item item-selectable\">\n",
            "    <span class=\"check-marker\">\n",
            "      <input type=\"checkbox\" class=\"check-box\" />\n",
            "    </span>\n",
            "    <span class=\"check-content\">\n",
            "      Parent task\n",
            "    </span>\n",
            "  </div>\n",
            "  <div class=\"item check-item item-selectable\" style=\"--indent: 1;\">\n",
            "    <span class=\"check-marker\">\n",
            "      <input type=\"checkbox\" class=\"check-box\" />\n",
            "    </span>\n",
            "    <span class=\"check-content\">\n",
            "      Child task\n",
            "    </span>\n",
            "  </div>\n",
        );
        assert_eq!(out, expected);
    }
}
