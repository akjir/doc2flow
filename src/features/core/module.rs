use std::fmt::Write as _;

use crate::core::document::{DocumentElement, DocumentElementId, DocumentParameters};
use crate::core::feature::FeatureModule;
use crate::core::format::{format_inline_into, push_indent};
use crate::core::renderer::{DocumentElementRenderer, HtmlRenderer};

/// Embedded core CSS styles for layout and components.
pub const CSS: &str = include_str!("core.css");

/// Embedded core JavaScript utility functions.
pub const JS_UTILS: &str = include_str!("utils.js");

/// Embedded core JavaScript state storage handlers.
pub const JS_STORAGE: &str = include_str!("storage.js");

/// Embedded core JavaScript section collapse handlers.
pub const JS_SECTIONS: &str = include_str!("sections.js");

/// Supported document element identifiers for core elements.
const CORE_SUPPORTED: [DocumentElementId; 3] = [
    DocumentElementId::HorizontalRule,
    DocumentElementId::Section,
    DocumentElementId::Text,
];

/// Core feature renderer handling text, horizontal rules, and section elements.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct CoreFeature;

impl CoreFeature {
    /// Creates a new core feature instance.
    #[must_use]
    pub fn new() -> Self {
        Self
    }
}

impl DocumentElementRenderer for CoreFeature {
    fn supported(&self) -> &[DocumentElementId] {
        &CORE_SUPPORTED
    }

    fn render_element(
        &self,
        element: &DocumentElement,
        indent: usize,
        _depth: usize,
        parameters: &DocumentParameters,
        out: &mut String,
        renderer: &HtmlRenderer,
    ) {
        match element {
            DocumentElement::HorizontalRule => {
                push_indent(out, indent);
                out.push_str("<hr />\n");
            }
            DocumentElement::Section {
                level,
                title,
                children,
            } => {
                let is_container = *level <= 2;
                let is_empty = children.is_empty();

                push_indent(out, indent);
                out.push_str("<section class=\"section\" data-level=\"");
                let _ = write!(out, "{level}");
                out.push_str("\">\n");

                push_indent(out, indent + 1);
                if is_container {
                    let h1_class = if *level == 1 { " section-header-h1" } else { "" };
                    let (empty_class, a11y_attrs) = if is_empty {
                        (" section-no-toggle", "")
                    } else {
                        ("", " role=\"button\" tabindex=\"0\" aria-expanded=\"true\"")
                    };

                    let _ = write!(
                        out,
                        "<h{level} class=\"section-header{h1_class}{empty_class}\"{a11y_attrs}>\n"
                    );
                    push_indent(out, indent + 2);
                    out.push_str("<span class=\"section-title\">");
                    format_inline_into(out, title);
                    out.push_str("</span>\n");
                    push_indent(out, indent + 2);
                    out.push_str("<span class=\"section-toggler\">&#9660;</span>\n");
                    push_indent(out, indent + 1);
                    let _ = writeln!(out, "</h{level}>");
                } else {
                    let _ = write!(out, "<h{level} class=\"section-subheading\">");
                    format_inline_into(out, title);
                    let _ = writeln!(out, "</h{level}>");
                }

                push_indent(out, indent + 1);
                out.push_str("<div class=\"section-body\">\n");

                renderer.render_children(children, indent + 2, 0, parameters, out);

                push_indent(out, indent + 1);
                out.push_str("</div>\n");

                push_indent(out, indent);
                out.push_str("</section>\n");
            }
            DocumentElement::Text(text) => {
                push_indent(out, indent);
                out.push_str("<div class=\"item text-item\">\n");
                push_indent(out, indent + 1);
                out.push_str("<span class=\"text-content\">\n");
                for line in text.lines() {
                    push_indent(out, indent + 2);
                    format_inline_into(out, line);
                    out.push('\n');
                }
                push_indent(out, indent + 1);
                out.push_str("</span>\n");
                push_indent(out, indent);
                out.push_str("</div>\n");
            }
            _ => {}
        }
    }
}

impl FeatureModule for CoreFeature {
    fn name(&self) -> &'static str {
        "core"
    }

    fn css(&self) -> Option<&'static str> {
        Some(CSS)
    }

    fn javascript(&self) -> &[&'static str] {
        &[JS_UTILS, JS_STORAGE, JS_SECTIONS]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_core_feature_css() {
        let feature = CoreFeature::new();
        let css = feature.css().expect("core css should exist");
        assert!(css.contains("--bg-body:"));
        assert!(css.contains("--code-bg:"));
        assert!(css.contains("--code-border:"));
        assert!(css.contains("--code-color:"));
        assert!(css.contains("--code-inline-font-size:"));
        assert!(css.contains("--item-hover-bg:"));
        assert!(css.contains("--item-done-bg:"));
        assert!(css.contains(".doc-body"));
        assert!(css.contains(".item"));
        assert!(css.contains(".text-content"));
        assert!(css.contains(".text-item"));
        assert!(css.contains(".txt-default"));
        assert!(css.contains(".txt-code"));
        assert!(css.contains(".txt-strike"));
        assert!(css.contains(".section"));
        assert!(css.contains(".section-header"));
        assert!(css.contains(".section-body"));
        assert!(css.contains(".section-subheading"));
        assert!(css.contains("--section-bg-header:"));
        assert!(css.contains("hr {"));
    }

    #[test]
    fn test_core_feature_javascript() {
        let feature = CoreFeature::new();
        let js = feature.javascript();
        assert_eq!(js.len(), 3);
        assert!(js[0].contains("window.d2f"));
        assert!(js[0].contains("utils"));
        assert!(js[0].contains("debounce"));
        assert!(js[0].contains("isRecord"));
        assert!(!js[0].contains("\nt,") && !js[0].contains(" t,"));
        assert!(js[1].contains("window.d2f"));
        assert!(js[1].contains("storage"));
        assert!(js[2].contains("window.d2f"));
        assert!(js[2].contains("sections"));
    }

    #[test]
    fn test_core_feature_empty_for_unsupported_elements() {
        let feature = CoreFeature::new();
        let code = DocumentElement::code_block(Some("rust"), "fn main() {}");
        let mut out = String::new();
        let renderer = HtmlRenderer::default_renderer();
        feature.render_element(
            &code,
            0,
            0,
            &DocumentParameters::default(),
            &mut out,
            &renderer,
        );
        assert!(out.is_empty());
    }

    #[test]
    fn test_core_feature_escapes_plain_text_html_entities() {
        let feature = CoreFeature::new();
        let elem = DocumentElement::text("5 < 10 & 20 > 15 \"quoted\" 'single'");
        let mut out = String::new();
        let renderer = HtmlRenderer::default_renderer();
        feature.render_element(
            &elem,
            0,
            0,
            &DocumentParameters::default(),
            &mut out,
            &renderer,
        );
        assert_eq!(
            out,
            "<div class=\"item text-item\">\n  <span class=\"text-content\">\n    5 &lt; 10 &amp; 20 &gt; 15 &quot;quoted&quot; &#39;single&#39;\n  </span>\n</div>\n"
        );
    }

    #[test]
    fn test_core_feature_inline_code_escapes_html_and_preserves_literals() {
        let feature = CoreFeature::new();
        let elem =
            DocumentElement::text("Example `<div class=\"box\"> && **not bold**</div>` here.");
        let mut out = String::new();
        let renderer = HtmlRenderer::default_renderer();
        feature.render_element(
            &elem,
            0,
            0,
            &DocumentParameters::default(),
            &mut out,
            &renderer,
        );
        assert_eq!(
            out,
            "<div class=\"item text-item\">\n  <span class=\"text-content\">\n    Example <code>&lt;div class=&quot;box&quot;&gt; &amp;&amp; **not bold**&lt;/div&gt;</code> here.\n  </span>\n</div>\n"
        );
    }

    #[test]
    fn test_core_feature_nested_formatting() {
        let feature = CoreFeature::new();
        let elem = DocumentElement::text("Formatted ~~**bold strikethrough**~~ with `code`.");
        let mut out = String::new();
        let renderer = HtmlRenderer::default_renderer();
        feature.render_element(
            &elem,
            0,
            0,
            &DocumentParameters::default(),
            &mut out,
            &renderer,
        );
        assert_eq!(
            out,
            "<div class=\"item text-item\">\n  <span class=\"text-content\">\n    Formatted <s><strong>bold strikethrough</strong></s> with <code>code</code>.\n  </span>\n</div>\n"
        );
    }

    #[test]
    fn test_core_feature_preserves_intra_word_underscores() {
        let feature = CoreFeature::new();
        let elem = DocumentElement::text("Host {{SERVER_NAME}}:{{PORT}} with key {{API_KEY}}.");
        let mut out = String::new();
        let renderer = HtmlRenderer::default_renderer();
        feature.render_element(
            &elem,
            0,
            0,
            &DocumentParameters::default(),
            &mut out,
            &renderer,
        );
        assert_eq!(
            out,
            "<div class=\"item text-item\">\n  <span class=\"text-content\">\n    Host {{SERVER_NAME}}:{{PORT}} with key {{API_KEY}}.\n  </span>\n</div>\n"
        );
    }

    #[test]
    fn test_core_feature_renders_bold_and_italic_combined() {
        let feature = CoreFeature::new();
        let elem = DocumentElement::text("This is ***bold and italic*** text.");
        let mut out = String::new();
        let renderer = HtmlRenderer::default_renderer();
        feature.render_element(
            &elem,
            0,
            0,
            &DocumentParameters::default(),
            &mut out,
            &renderer,
        );
        assert_eq!(
            out,
            "<div class=\"item text-item\">\n  <span class=\"text-content\">\n    This is <strong><em>bold and italic</em></strong> text.\n  </span>\n</div>\n"
        );

        let elem_underscores = DocumentElement::text("This is ___bold and italic___ text.");
        let mut out_underscores = String::new();
        feature.render_element(
            &elem_underscores,
            0,
            0,
            &DocumentParameters::default(),
            &mut out_underscores,
            &renderer,
        );
        assert_eq!(
            out_underscores,
            "<div class=\"item text-item\">\n  <span class=\"text-content\">\n    This is <strong><em>bold and italic</em></strong> text.\n  </span>\n</div>\n"
        );
    }

    #[test]
    fn test_core_feature_renders_bold_asterisks_and_underscores() {
        let feature = CoreFeature::new();
        let elem_asterisk = DocumentElement::text("This is **bold** text.");
        let mut out_asterisk = String::new();
        let renderer = HtmlRenderer::default_renderer();
        feature.render_element(
            &elem_asterisk,
            0,
            0,
            &DocumentParameters::default(),
            &mut out_asterisk,
            &renderer,
        );
        assert_eq!(
            out_asterisk,
            "<div class=\"item text-item\">\n  <span class=\"text-content\">\n    This is <strong>bold</strong> text.\n  </span>\n</div>\n"
        );

        let elem_underscore = DocumentElement::text("This is __bold__ text.");
        let mut out_underscore = String::new();
        feature.render_element(
            &elem_underscore,
            0,
            0,
            &DocumentParameters::default(),
            &mut out_underscore,
            &renderer,
        );
        assert_eq!(
            out_underscore,
            "<div class=\"item text-item\">\n  <span class=\"text-content\">\n    This is <strong>bold</strong> text.\n  </span>\n</div>\n"
        );
    }

    #[test]
    fn test_core_feature_renders_horizontal_rule() {
        let feature = CoreFeature::new();
        let element = DocumentElement::horizontal_rule();
        let renderer = HtmlRenderer::default_renderer();

        let mut out0 = String::new();
        feature.render_element(
            &element,
            0,
            0,
            &DocumentParameters::default(),
            &mut out0,
            &renderer,
        );
        assert_eq!(out0, "<hr />\n");

        let mut out1 = String::new();
        feature.render_element(
            &element,
            1,
            0,
            &DocumentParameters::default(),
            &mut out1,
            &renderer,
        );
        assert_eq!(out1, "  <hr />\n");

        let mut out2 = String::new();
        feature.render_element(
            &element,
            2,
            0,
            &DocumentParameters::default(),
            &mut out2,
            &renderer,
        );
        assert_eq!(out2, "    <hr />\n");
    }

    #[test]
    fn test_core_feature_renders_inline_code_and_strips_backticks() {
        let feature = CoreFeature::new();
        let elem = DocumentElement::text("Run `cargo test --all` now.");
        let mut out = String::new();
        let renderer = HtmlRenderer::default_renderer();
        feature.render_element(
            &elem,
            0,
            0,
            &DocumentParameters::default(),
            &mut out,
            &renderer,
        );
        assert_eq!(
            out,
            "<div class=\"item text-item\">\n  <span class=\"text-content\">\n    Run <code>cargo test --all</code> now.\n  </span>\n</div>\n"
        );
    }

    #[test]
    fn test_core_feature_renders_italic_asterisks_and_underscores() {
        let feature = CoreFeature::new();
        let elem_asterisk = DocumentElement::text("This is *italic* text.");
        let mut out_asterisk = String::new();
        let renderer = HtmlRenderer::default_renderer();
        feature.render_element(
            &elem_asterisk,
            0,
            0,
            &DocumentParameters::default(),
            &mut out_asterisk,
            &renderer,
        );
        assert_eq!(
            out_asterisk,
            "<div class=\"item text-item\">\n  <span class=\"text-content\">\n    This is <em>italic</em> text.\n  </span>\n</div>\n"
        );

        let elem_underscore = DocumentElement::text("This is _italic_ text.");
        let mut out_underscore = String::new();
        feature.render_element(
            &elem_underscore,
            0,
            0,
            &DocumentParameters::default(),
            &mut out_underscore,
            &renderer,
        );
        assert_eq!(
            out_underscore,
            "<div class=\"item text-item\">\n  <span class=\"text-content\">\n    This is <em>italic</em> text.\n  </span>\n</div>\n"
        );
    }

    #[test]
    fn test_core_feature_renders_plain_text() {
        let feature = CoreFeature::new();
        let element = DocumentElement::text("Hello, world!");
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
        assert_eq!(
            out,
            "  <div class=\"item text-item\">\n    <span class=\"text-content\">\n      Hello, world!\n    </span>\n  </div>\n"
        );
    }

    #[test]
    fn test_core_feature_renders_section_with_content() {
        let feature = CoreFeature::new();
        let section = DocumentElement::section(
            1,
            "Overview",
            vec![DocumentElement::text("Section body content")],
        );
        let mut out = String::new();
        let renderer = HtmlRenderer::default_renderer();
        feature.render_element(
            &section,
            2,
            0,
            &DocumentParameters::default(),
            &mut out,
            &renderer,
        );
        let expected = concat!(
            "    <section class=\"section\" data-level=\"1\">\n",
            "      <h1 class=\"section-header section-header-h1\" role=\"button\" tabindex=\"0\" aria-expanded=\"true\">\n",
            "        <span class=\"section-title\">Overview</span>\n",
            "        <span class=\"section-toggler\">&#9660;</span>\n",
            "      </h1>\n",
            "      <div class=\"section-body\">\n",
            "        <div class=\"item text-item\">\n",
            "          <span class=\"text-content\">\n",
            "            Section body content\n",
            "          </span>\n",
            "        </div>\n",
            "      </div>\n",
            "    </section>\n"
        );
        assert_eq!(out, expected);
    }

    #[test]
    fn test_core_feature_renders_level_2_collapsible_section() {
        let feature = CoreFeature::new();
        let section = DocumentElement::section(
            2,
            "Sub Section",
            vec![DocumentElement::text("Sub content")],
        );
        let mut out = String::new();
        let renderer = HtmlRenderer::default_renderer();
        feature.render_element(
            &section,
            1,
            0,
            &DocumentParameters::default(),
            &mut out,
            &renderer,
        );
        let expected = concat!(
            "  <section class=\"section\" data-level=\"2\">\n",
            "    <h2 class=\"section-header\" role=\"button\" tabindex=\"0\" aria-expanded=\"true\">\n",
            "      <span class=\"section-title\">Sub Section</span>\n",
            "      <span class=\"section-toggler\">&#9660;</span>\n",
            "    </h2>\n",
            "    <div class=\"section-body\">\n",
            "      <div class=\"item text-item\">\n",
            "        <span class=\"text-content\">\n",
            "          Sub content\n",
            "        </span>\n",
            "      </div>\n",
            "    </div>\n",
            "  </section>\n"
        );
        assert_eq!(out, expected);
    }

    #[test]
    fn test_core_feature_renders_empty_section_no_toggle() {
        let feature = CoreFeature::new();
        let section = DocumentElement::section(1, "Empty", Vec::new());
        let mut out = String::new();
        let renderer = HtmlRenderer::default_renderer();
        feature.render_element(
            &section,
            1,
            0,
            &DocumentParameters::default(),
            &mut out,
            &renderer,
        );
        let expected = concat!(
            "  <section class=\"section\" data-level=\"1\">\n",
            "    <h1 class=\"section-header section-header-h1 section-no-toggle\">\n",
            "      <span class=\"section-title\">Empty</span>\n",
            "      <span class=\"section-toggler\">&#9660;</span>\n",
            "    </h1>\n",
            "    <div class=\"section-body\">\n",
            "    </div>\n",
            "  </section>\n"
        );
        assert_eq!(out, expected);
    }

    #[test]
    fn test_core_feature_renders_subheading_level_3() {
        let feature = CoreFeature::new();
        let section = DocumentElement::section(
            3,
            "Deep Header",
            vec![DocumentElement::text("Deep content")],
        );
        let mut out = String::new();
        let renderer = HtmlRenderer::default_renderer();
        feature.render_element(
            &section,
            1,
            0,
            &DocumentParameters::default(),
            &mut out,
            &renderer,
        );
        let expected = concat!(
            "  <section class=\"section\" data-level=\"3\">\n",
            "    <h3 class=\"section-subheading\">Deep Header</h3>\n",
            "    <div class=\"section-body\">\n",
            "      <div class=\"item text-item\">\n",
            "        <span class=\"text-content\">\n",
            "          Deep content\n",
            "        </span>\n",
            "      </div>\n",
            "    </div>\n",
            "  </section>\n"
        );
        assert_eq!(out, expected);
    }

    #[test]
    fn test_core_feature_renders_strikethrough() {
        let feature = CoreFeature::new();
        let elem = DocumentElement::text("Replaces ~~legacy procedures~~ with modern.");
        let mut out = String::new();
        let renderer = HtmlRenderer::default_renderer();
        feature.render_element(
            &elem,
            0,
            0,
            &DocumentParameters::default(),
            &mut out,
            &renderer,
        );
        assert_eq!(
            out,
            "<div class=\"item text-item\">\n  <span class=\"text-content\">\n    Replaces <s>legacy procedures</s> with modern.\n  </span>\n</div>\n"
        );
    }

    #[test]
    fn test_core_feature_renders_links() {
        let feature = CoreFeature::new();
        let elem = DocumentElement::text("Visit [Doc2Flow](https://doc2flow.dev) for guides.");
        let mut out = String::new();
        let renderer = HtmlRenderer::default_renderer();
        feature.render_element(
            &elem,
            0,
            0,
            &DocumentParameters::default(),
            &mut out,
            &renderer,
        );
        assert_eq!(
            out,
            "<div class=\"item text-item\">\n  <span class=\"text-content\">\n    Visit <a href=\"https://doc2flow.dev\">Doc2Flow</a> for guides.\n  </span>\n</div>\n"
        );
    }

    #[test]
    fn test_core_feature_unclosed_delimiters() {
        let feature = CoreFeature::new();
        let elem = DocumentElement::text("Unclosed **bold and ~~strike and `code");
        let mut out = String::new();
        let renderer = HtmlRenderer::default_renderer();
        feature.render_element(
            &elem,
            0,
            0,
            &DocumentParameters::default(),
            &mut out,
            &renderer,
        );
        assert_eq!(
            out,
            "<div class=\"item text-item\">\n  <span class=\"text-content\">\n    Unclosed **bold and ~~strike and `code\n  </span>\n</div>\n"
        );
    }
}
