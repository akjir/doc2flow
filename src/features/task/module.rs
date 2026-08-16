//! Task vertical slice feature module.

use std::fmt::Write as _;

use crate::core::builder::HtmlRenderer;
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
    /// Intercepts the rendering of checkbox task elements into the output buffer.
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
            DocumentElement::CheckBoxItem {
                checked,
                content,
                children,
            } => {
                push_indent(out, indent);

                if *checked {
                    out.push_str("<div class=\"item check-item checked\"");
                } else {
                    out.push_str("<div class=\"item check-item\"");
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
                true
            }
            _ => false,
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
        assert!(feature.try_render_body(
            &element,
            1,
            0,
            &DocumentParameters::default(),
            &mut out,
            &renderer,
        ));
        let expected = concat!(
            "  <div class=\"item check-item\">\n",
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
        assert!(feature.try_render_body(
            &element,
            1,
            0,
            &DocumentParameters::default(),
            &mut out,
            &renderer,
        ));
        let expected = concat!(
            "  <div class=\"item check-item checked\">\n",
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
        assert!(feature.try_render_body(
            &element,
            0,
            0,
            &DocumentParameters::default(),
            &mut out,
            &renderer,
        ));
        let expected = concat!(
            "<div class=\"item check-item checked\">\n",
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
        assert!(feature.try_render_body(
            &element,
            2,
            1,
            &DocumentParameters::default(),
            &mut out,
            &renderer,
        ));
        let expected = concat!(
            "    <div class=\"item check-item\" style=\"--indent: 1;\">\n",
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
        assert!(feature.try_render_body(
            &checked,
            2,
            2,
            &DocumentParameters::default(),
            &mut checked_out,
            &renderer,
        ));
        let checked_expected = concat!(
            "    <div class=\"item check-item checked\" style=\"--indent: 2;\">\n",
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
        assert!(feature.try_render_body(
            &element,
            1,
            0,
            &DocumentParameters::default(),
            &mut out,
            &renderer,
        ));
        let expected = concat!(
            "  <div class=\"item check-item\">\n",
            "    <span class=\"check-marker\">\n",
            "      <input type=\"checkbox\" class=\"check-box\" />\n",
            "    </span>\n",
            "    <span class=\"check-content\">\n",
            "      Parent task\n",
            "    </span>\n",
            "  </div>\n",
            "  <div class=\"item check-item\" style=\"--indent: 1;\">\n",
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
