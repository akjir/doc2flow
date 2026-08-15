//! Core vertical slice feature module.

use crate::core::document::{DocumentElement, DocumentParameters};
use crate::core::feature::Feature;
use crate::core::format::{format_inline_into, push_indent};

/// Embedded core CSS styles for layout and components.
pub const CSS: &str = include_str!("core.css");

/// Embedded core JavaScript bundle for client runtime.
pub const JS: &str = include_str!("core.js");

/// Core feature renderer handling text, horizontal rules, and section elements.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct CoreFeature;

impl CoreFeature {
    /// Creates a new core feature instance.
    pub const fn new() -> Self {
        Self
    }
}

impl Feature for CoreFeature {
    /// Converts a document element and inner content into an HTML string representation.
    fn to_html(
        &self,
        element: &DocumentElement,
        content: &str,
        indent: usize,
        _depth: usize,
        _parameters: &DocumentParameters,
    ) -> String {
        match element {
            DocumentElement::HorizontalRule => {
                let spaces = indent * 2;
                let mut out = String::with_capacity(spaces + 8);
                push_indent(&mut out, indent);
                out.push_str("<hr />\n");
                out
            }
            DocumentElement::Section { level, title, .. } => {
                let spaces = indent * 2;
                let inner_spaces = (indent + 1) * 2;
                let mut out = String::with_capacity(
                    title.len() + content.len() + spaces * 2 + inner_spaces * 3 + 128,
                );
                push_indent(&mut out, indent);
                out.push_str("<section class=\"section\" data-level=\"");
                out.push_str(&level.to_string());
                out.push_str("\">\n");

                push_indent(&mut out, indent + 1);
                out.push_str("<h");
                out.push_str(&level.to_string());
                out.push('>');
                out.push_str(title);
                out.push_str("</h");
                out.push_str(&level.to_string());
                out.push_str(">\n");

                push_indent(&mut out, indent + 1);
                out.push_str("<div class=\"section-body\">\n");

                out.push_str(content);

                push_indent(&mut out, indent + 1);
                out.push_str("</div>\n");

                push_indent(&mut out, indent);
                out.push_str("</section>\n");
                out
            }
            DocumentElement::Text(text) => {
                let spaces = indent * 2;
                let inner_spaces = (indent + 1) * 2;
                let mut out =
                    String::with_capacity(text.len() * 2 + spaces * 2 + inner_spaces * 2 + 64);
                push_indent(&mut out, indent);
                out.push_str("<div class=\"item item-text\">\n");
                push_indent(&mut out, indent + 1);
                out.push_str("<span class=\"text-content\">\n");
                for line in text.lines() {
                    push_indent(&mut out, indent + 2);
                    format_inline_into(&mut out, line);
                    out.push('\n');
                }
                push_indent(&mut out, indent + 1);
                out.push_str("</span>\n");
                push_indent(&mut out, indent);
                out.push_str("</div>\n");
                out
            }
            _ => String::new(),
        }
    }

    /// Returns the embedded CSS stylesheet for the core feature.
    fn css(&self) -> Option<&'static str> {
        Some(CSS)
    }

    /// Returns the embedded JavaScript client script for the core feature.
    fn javascript(&self) -> Option<&'static str> {
        Some(JS)
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
        let js = feature.javascript().expect("core javascript should exist");
        assert!(js.contains("window.d2f"));
        assert!(js.contains("core"));
    }

    #[test]
    fn test_core_feature_empty_for_unsupported_elements() {
        let feature = CoreFeature::new();
        let code = DocumentElement::code_block(Some("rust"), "fn main() {}");
        assert_eq!(
            feature.to_html(&code, "", 0, 0, &DocumentParameters::default()),
            ""
        );
    }

    #[test]
    fn test_core_feature_escapes_plain_text_html_entities() {
        let feature = CoreFeature::new();
        let elem = DocumentElement::text("5 < 10 & 20 > 15 \"quoted\" 'single'");
        assert_eq!(
            feature.to_html(&elem, "", 0, 0, &DocumentParameters::default()),
            "<div class=\"item item-text\">\n  <span class=\"text-content\">\n    5 &lt; 10 &amp; 20 &gt; 15 &quot;quoted&quot; &#39;single&#39;\n  </span>\n</div>\n"
        );
    }

    #[test]
    fn test_core_feature_inline_code_escapes_html_and_preserves_literals() {
        let feature = CoreFeature::new();
        let elem =
            DocumentElement::text("Example `<div class=\"box\"> && **not bold**</div>` here.");
        assert_eq!(
            feature.to_html(&elem, "", 0, 0, &DocumentParameters::default()),
            "<div class=\"item item-text\">\n  <span class=\"text-content\">\n    Example <code>&lt;div class=&quot;box&quot;&gt; &amp;&amp; **not bold**&lt;/div&gt;</code> here.\n  </span>\n</div>\n"
        );
    }

    #[test]
    fn test_core_feature_nested_formatting() {
        let feature = CoreFeature::new();
        let elem = DocumentElement::text("Formatted ~~**bold strikethrough**~~ with `code`.");
        assert_eq!(
            feature.to_html(&elem, "", 0, 0, &DocumentParameters::default()),
            "<div class=\"item item-text\">\n  <span class=\"text-content\">\n    Formatted <s><strong>bold strikethrough</strong></s> with <code>code</code>.\n  </span>\n</div>\n"
        );
    }

    #[test]
    fn test_core_feature_preserves_intra_word_underscores() {
        let feature = CoreFeature::new();
        let elem = DocumentElement::text("Host {{SERVER_NAME}}:{{PORT}} with key {{API_KEY}}.");
        assert_eq!(
            feature.to_html(&elem, "", 0, 0, &DocumentParameters::default()),
            "<div class=\"item item-text\">\n  <span class=\"text-content\">\n    Host {{SERVER_NAME}}:{{PORT}} with key {{API_KEY}}.\n  </span>\n</div>\n"
        );
    }

    #[test]
    fn test_core_feature_renders_bold_and_italic_combined() {
        let feature = CoreFeature::new();
        let elem = DocumentElement::text("This is ***bold and italic*** text.");
        assert_eq!(
            feature.to_html(&elem, "", 0, 0, &DocumentParameters::default()),
            "<div class=\"item item-text\">\n  <span class=\"text-content\">\n    This is <strong><em>bold and italic</em></strong> text.\n  </span>\n</div>\n"
        );

        let elem_underscores = DocumentElement::text("This is ___bold and italic___ text.");
        assert_eq!(
            feature.to_html(&elem_underscores, "", 0, 0, &DocumentParameters::default()),
            "<div class=\"item item-text\">\n  <span class=\"text-content\">\n    This is <strong><em>bold and italic</em></strong> text.\n  </span>\n</div>\n"
        );
    }

    #[test]
    fn test_core_feature_renders_bold_asterisks_and_underscores() {
        let feature = CoreFeature::new();
        let elem_asterisk = DocumentElement::text("This is **bold** text.");
        assert_eq!(
            feature.to_html(&elem_asterisk, "", 0, 0, &DocumentParameters::default()),
            "<div class=\"item item-text\">\n  <span class=\"text-content\">\n    This is <strong>bold</strong> text.\n  </span>\n</div>\n"
        );

        let elem_underscore = DocumentElement::text("This is __bold__ text.");
        assert_eq!(
            feature.to_html(&elem_underscore, "", 0, 0, &DocumentParameters::default()),
            "<div class=\"item item-text\">\n  <span class=\"text-content\">\n    This is <strong>bold</strong> text.\n  </span>\n</div>\n"
        );
    }

    #[test]
    fn test_core_feature_renders_horizontal_rule() {
        let feature = CoreFeature::new();
        let element = DocumentElement::horizontal_rule();
        assert_eq!(
            feature.to_html(&element, "", 0, 0, &DocumentParameters::default()),
            "<hr />\n"
        );
        assert_eq!(
            feature.to_html(&element, "", 1, 0, &DocumentParameters::default()),
            "  <hr />\n"
        );
        assert_eq!(
            feature.to_html(&element, "", 2, 0, &DocumentParameters::default()),
            "    <hr />\n"
        );
    }

    #[test]
    fn test_core_feature_renders_inline_code_and_strips_backticks() {
        let feature = CoreFeature::new();
        let elem = DocumentElement::text("Run `cargo test --all` now.");
        assert_eq!(
            feature.to_html(&elem, "", 0, 0, &DocumentParameters::default()),
            "<div class=\"item item-text\">\n  <span class=\"text-content\">\n    Run <code>cargo test --all</code> now.\n  </span>\n</div>\n"
        );
    }

    #[test]
    fn test_core_feature_renders_italic_asterisks_and_underscores() {
        let feature = CoreFeature::new();
        let elem_asterisk = DocumentElement::text("This is *italic* text.");
        assert_eq!(
            feature.to_html(&elem_asterisk, "", 0, 0, &DocumentParameters::default()),
            "<div class=\"item item-text\">\n  <span class=\"text-content\">\n    This is <em>italic</em> text.\n  </span>\n</div>\n"
        );

        let elem_underscore = DocumentElement::text("This is _italic_ text.");
        assert_eq!(
            feature.to_html(&elem_underscore, "", 0, 0, &DocumentParameters::default()),
            "<div class=\"item item-text\">\n  <span class=\"text-content\">\n    This is <em>italic</em> text.\n  </span>\n</div>\n"
        );
    }

    #[test]
    fn test_core_feature_renders_plain_text() {
        let feature = CoreFeature::new();
        let element = DocumentElement::text("Hello, world!");
        assert_eq!(
            feature.to_html(&element, "", 1, 0, &DocumentParameters::default()),
            "  <div class=\"item item-text\">\n    <span class=\"text-content\">\n      Hello, world!\n    </span>\n  </div>\n"
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
        let child_html = "        <div class=\"item item-text\">\n          <span class=\"text-content\">\n            Section body content\n          </span>\n        </div>\n";
        let html = feature.to_html(
            &section,
            child_html,
            2,
            0,
            &DocumentParameters::default(),
        );
        let expected = concat!(
            "    <section class=\"section\" data-level=\"1\">\n",
            "      <h1>Overview</h1>\n",
            "      <div class=\"section-body\">\n",
            "        <div class=\"item item-text\">\n",
            "          <span class=\"text-content\">\n",
            "            Section body content\n",
            "          </span>\n",
            "        </div>\n",
            "      </div>\n",
            "    </section>\n"
        );
        assert_eq!(html, expected);
    }

    #[test]
    fn test_core_feature_renders_strikethrough() {
        let feature = CoreFeature::new();
        let elem = DocumentElement::text("Replaces ~~legacy procedures~~ with modern.");
        assert_eq!(
            feature.to_html(&elem, "", 0, 0, &DocumentParameters::default()),
            "<div class=\"item item-text\">\n  <span class=\"text-content\">\n    Replaces <s>legacy procedures</s> with modern.\n  </span>\n</div>\n"
        );
    }

    #[test]
    fn test_core_feature_renders_links() {
        let feature = CoreFeature::new();
        let elem = DocumentElement::text("Visit [Doc2Flow](https://doc2flow.dev) for guides.");
        assert_eq!(
            feature.to_html(&elem, "", 0, 0, &DocumentParameters::default()),
            "<div class=\"item item-text\">\n  <span class=\"text-content\">\n    Visit <a href=\"https://doc2flow.dev\">Doc2Flow</a> for guides.\n  </span>\n</div>\n"
        );
    }

    #[test]
    fn test_core_feature_unclosed_delimiters() {
        let feature = CoreFeature::new();
        let elem = DocumentElement::text("Unclosed **bold and ~~strike and `code");
        assert_eq!(
            feature.to_html(&elem, "", 0, 0, &DocumentParameters::default()),
            "<div class=\"item item-text\">\n  <span class=\"text-content\">\n    Unclosed **bold and ~~strike and `code\n  </span>\n</div>\n"
        );
    }
}
