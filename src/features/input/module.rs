//! Input form field vertical slice feature module.

use crate::core::document::{DocumentElement, DocumentElementId, DocumentParameters};
use crate::core::feature::FeatureModule;
use crate::core::format::{escape_html_into, push_indent};
use crate::core::renderer::{DocumentElementRenderer, HtmlRenderer};

/// Embedded input CSS stylesheet.
pub const CSS: &str = include_str!("input.css");

/// Embedded input JavaScript client script.
pub const JS: &str = include_str!("input.js");

/// Supported document element identifiers for input form field elements.
const INPUT_SUPPORTED: [DocumentElementId; 1] = [DocumentElementId::Input];

/// Input feature renderer handling standalone editable input form field elements.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct InputFeature;

impl InputFeature {
    /// Creates a new input feature instance.
    ///
    /// # Examples
    ///
    /// ```
    /// use doc2flow::features::input::InputFeature;
    ///
    /// let feature = InputFeature::new();
    /// ```
    #[must_use]
    pub fn new() -> Self {
        Self
    }
}

impl DocumentElementRenderer for InputFeature {
    fn supported(&self) -> &[DocumentElementId] {
        &INPUT_SUPPORTED
    }

    fn render_element(
        &self,
        element: &DocumentElement,
        indent: usize,
        _depth: usize,
        _parameters: &DocumentParameters,
        out: &mut String,
        _renderer: &HtmlRenderer,
    ) {
        if let DocumentElement::Input { text } = element {
            push_indent(out, indent);
            out.push_str("<div class=\"input-wrap\">\n");

            push_indent(out, indent + 1);
            out.push_str("<input type=\"text\" class=\"input-field\" value=\"");
            escape_html_into(out, text);
            out.push_str("\" />\n");

            push_indent(out, indent);
            out.push_str("</div>\n");
        }
    }
}

impl FeatureModule for InputFeature {
    fn name(&self) -> &'static str {
        "input"
    }

    fn css(&self) -> Option<&'static str> {
        Some(CSS)
    }

    fn javascript(&self) -> &[&'static str] {
        &[JS]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_input_feature_javascript() {
        let feature = InputFeature::new();
        let js_files = feature.javascript();
        assert_eq!(js_files.len(), 1);
        let js = js_files[0];
        assert!(js.contains("saveFields"));
        assert!(js.contains("loadFields"));
        assert!(js.contains("resetFields"));
        assert!(js.contains("registerSaveHandler"));
        assert!(js.contains("registerLoadHandler"));
        assert!(js.contains("registerResetHandler"));
        assert!(js.contains("input-field"));
    }


    #[test]
    fn test_input_feature_constructor_new() {
        let feature = InputFeature::new();
        assert_eq!(feature, InputFeature);
    }

    #[test]
    fn test_input_feature_css() {
        let feature = InputFeature::new();
        let css = feature.css().expect("input css should exist");
        assert!(css.contains("--input-bg:"));
        assert!(css.contains("--input-border:"));
        assert!(css.contains("--input-color:"));
        assert!(css.contains("--input-radius:"));
        assert!(css.contains("--input-font-size:"));
        assert!(css.contains("--input-padding:"));
        assert!(css.contains("--input-focus-border:"));
        assert!(css.contains(".input-wrap"));
        assert!(css.contains(".input-field"));
    }

    #[test]
    fn test_input_feature_renders_input_element() {
        let feature = InputFeature::new();
        let element = DocumentElement::input("production.db.internal");
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
            "  <div class=\"input-wrap\">\n",
            "    <input type=\"text\" class=\"input-field\" value=\"production.db.internal\" />\n",
            "  </div>\n"
        );
        assert_eq!(out, expected);
    }

    #[test]
    fn test_input_feature_renders_empty_text() {
        let feature = InputFeature::new();
        let element = DocumentElement::input("");
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
            "<div class=\"input-wrap\">\n",
            "  <input type=\"text\" class=\"input-field\" value=\"\" />\n",
            "</div>\n"
        );
        assert_eq!(out, expected);
    }

    #[test]
    fn test_input_feature_escapes_html_entities() {
        let feature = InputFeature::new();
        let element = DocumentElement::input("<script>alert(\"XSS & 'quotes'\")</script>");
        let mut out = String::new();
        let renderer = HtmlRenderer::default_renderer();
        feature.render_element(
            &element,
            2,
            0,
            &DocumentParameters::default(),
            &mut out,
            &renderer,
        );
        let expected = concat!(
            "    <div class=\"input-wrap\">\n",
            "      <input type=\"text\" class=\"input-field\" value=\"&lt;script&gt;alert(&quot;XSS &amp; &#39;quotes&#39;&quot;)&lt;/script&gt;\" />\n",
            "    </div>\n"
        );
        assert_eq!(out, expected);
    }

    #[test]
    fn test_input_feature_renders_with_indent() {
        let feature = InputFeature::new();
        let element = DocumentElement::input("port 8080");
        let mut out = String::new();
        let renderer = HtmlRenderer::default_renderer();
        feature.render_element(
            &element,
            3,
            0,
            &DocumentParameters::default(),
            &mut out,
            &renderer,
        );
        let expected = concat!(
            "      <div class=\"input-wrap\">\n",
            "        <input type=\"text\" class=\"input-field\" value=\"port 8080\" />\n",
            "      </div>\n"
        );
        assert_eq!(out, expected);
    }

    #[test]
    fn test_input_feature_empty_for_unsupported_elements() {
        let feature = InputFeature::new();
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
}
