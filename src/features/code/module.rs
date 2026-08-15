//! Code vertical slice feature module.

use crate::core::document::DocumentElement;
use crate::core::feature::Feature;
use crate::core::format::{escape_html_into, push_indent};

/// Embedded code CSS styles for code blocks.
pub const CSS: &str = include_str!("code.css");

/// Code feature renderer handling syntax and fenced code block elements.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct CodeFeature;

impl CodeFeature {
    /// Creates a new code feature instance.
    pub const fn new() -> Self {
        Self
    }
}

impl Feature for CodeFeature {
    /// Converts a document element and inner content into an HTML string representation.
    fn to_html(
        &self,
        element: &DocumentElement,
        _content: &str,
        indent: usize,
        _depth: usize,
    ) -> String {
        match element {
            DocumentElement::CodeBlock { content, .. } => {
                let spaces = indent * 2;
                let mut out = String::with_capacity(content.len() + spaces + 48);
                push_indent(&mut out, indent);
                out.push_str("<pre class=\"code-default\"><code>");
                escape_html_into(&mut out, content);
                out.push_str("</code></pre>\n");
                out
            }
            _ => String::new(),
        }
    }

    /// Returns the embedded CSS stylesheet for the code feature.
    fn css(&self) -> Option<&'static str> {
        Some(CSS)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_code_feature_constructor_new() {
        let feature = CodeFeature::new();
        assert_eq!(feature, CodeFeature);
    }

    #[test]
    fn test_code_feature_css() {
        let feature = CodeFeature::new();
        let css = feature.css().expect("code css should exist");
        assert!(css.contains("--code-font-size:"));
        assert!(css.contains("--code-line-height:"));
        assert!(css.contains("--code-radius:"));
        assert!(css.contains(".code-default"));
    }

    #[test]
    fn test_code_feature_renders_code_block() {
        let feature = CodeFeature::new();
        let element =
            DocumentElement::code_block(Some("rust"), "fn main() {\n    println!(\"hi\");\n}");
        assert_eq!(
            feature.to_html(&element, "", 1, 0),
            "  <pre class=\"code-default\"><code>fn main() {\n    println!(&quot;hi&quot;);\n}</code></pre>\n"
        );
    }

    #[test]
    fn test_code_feature_escapes_html_entities() {
        let feature = CodeFeature::new();
        let element =
            DocumentElement::code_block(None::<String>, "<div class=\"foo\"> && 'bar'</div>");
        assert_eq!(
            feature.to_html(&element, "", 2, 0),
            "    <pre class=\"code-default\"><code>&lt;div class=&quot;foo&quot;&gt; &amp;&amp; &#39;bar&#39;&lt;/div&gt;</code></pre>\n"
        );
    }

    #[test]
    fn test_code_feature_empty_for_unsupported_elements() {
        let feature = CodeFeature::new();
        let text = DocumentElement::text("Regular text");
        assert_eq!(feature.to_html(&text, "", 0, 0), "");
    }
}
