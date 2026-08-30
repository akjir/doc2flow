//! Code vertical slice feature module.

use crate::core::document::{DocumentElement, DocumentElementId, DocumentParameters};
use crate::core::feature::FeatureModule;
use crate::core::format::{escape_html_into, push_indent};
use crate::core::renderer::{DocumentElementRenderer, HtmlRenderer};

/// Embedded code CSS styles for code blocks.
pub const CSS: &str = include_str!("code.css");

/// Supported document element identifiers for code blocks and code block variables.
const CODE_SUPPORTED: [DocumentElementId; 2] = [
    DocumentElementId::CodeBlock,
    DocumentElementId::TableVariables,
];

/// Code feature renderer handling syntax and fenced code block elements.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct CodeFeature;

impl CodeFeature {
    /// Creates a new code feature instance.
    #[must_use]
    pub fn new() -> Self {
        Self
    }
}

impl DocumentElementRenderer for CodeFeature {
    fn supported(&self) -> &[DocumentElementId] {
        &CODE_SUPPORTED
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
        match element {
            DocumentElement::CodeBlock { content, .. } => {
                push_indent(out, indent);
                out.push_str("<pre class=\"code-default\"><code>");
                escape_html_into(out, content);
                out.push_str("</code></pre>\n");
            }
            DocumentElement::TableVariables { variables } => {
                if variables.is_empty() {
                    return;
                }

                let mut entries: Vec<_> = variables.iter().collect();
                entries.sort_by_key(|(k, _)| *k);

                push_indent(out, indent);
                out.push_str("<div class=\"code-table-wrap\">\n");

                push_indent(out, indent + 1);
                out.push_str("<table class=\"code-table-default\">\n");

                push_indent(out, indent + 2);
                out.push_str("<thead>\n");

                push_indent(out, indent + 3);
                out.push_str("<tr>\n");

                push_indent(out, indent + 4);
                out.push_str("<th>Variable</th>\n");

                push_indent(out, indent + 4);
                out.push_str("<th>Value</th>\n");

                push_indent(out, indent + 3);
                out.push_str("</tr>\n");

                push_indent(out, indent + 2);
                out.push_str("</thead>\n");

                push_indent(out, indent + 2);
                out.push_str("<tbody>\n");

                for (key, val) in entries {
                    push_indent(out, indent + 3);
                    out.push_str("<tr>\n");

                    push_indent(out, indent + 4);
                    out.push_str("<td>");
                    escape_html_into(out, key);
                    out.push_str("</td>\n");

                    push_indent(out, indent + 4);
                    out.push_str("<td><input type=\"text\" class=\"code-table-input input-field\" id=\"f_var_");
                    escape_html_into(out, key);
                    out.push_str("\" data-var-key=\"");
                    escape_html_into(out, key);
                    out.push_str("\" value=\"");
                    escape_html_into(out, val);
                    out.push_str("\"></td>\n");

                    push_indent(out, indent + 3);
                    out.push_str("</tr>\n");
                }

                push_indent(out, indent + 2);
                out.push_str("</tbody>\n");

                push_indent(out, indent + 1);
                out.push_str("</table>\n");

                push_indent(out, indent);
                out.push_str("</div>\n");
            }
            _ => {}
        }
    }
}

impl FeatureModule for CodeFeature {
    fn name(&self) -> &'static str {
        "code"
    }

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
        assert!(css.contains("--code-table-bg:"));
        assert!(css.contains(".code-table-wrap"));
        assert!(css.contains(".code-table-default"));
        assert!(css.contains(".code-table-input"));
    }

    #[test]
    fn test_code_feature_renders_code_block() {
        let feature = CodeFeature::new();
        let element =
            DocumentElement::code_block(Some("rust"), "fn main() {\n    println!(\"hi\");\n}");
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
            "  <pre class=\"code-default\"><code>fn main() {\n    println!(&quot;hi&quot;);\n}</code></pre>\n"
        );
    }

    #[test]
    fn test_code_feature_escapes_html_entities() {
        let feature = CodeFeature::new();
        let element =
            DocumentElement::code_block(None::<String>, "<div class=\"foo\"> && 'bar'</div>");
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
        assert_eq!(
            out,
            "    <pre class=\"code-default\"><code>&lt;div class=&quot;foo&quot;&gt; &amp;&amp; &#39;bar&#39;&lt;/div&gt;</code></pre>\n"
        );
    }

    #[test]
    fn test_code_feature_renders_table_variables() {
        let feature = CodeFeature::new();
        let mut vars = std::collections::HashMap::new();
        vars.insert("PORT".into(), "8080".into());
        let element = DocumentElement::table_variables(vars);
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
            "    <div class=\"code-table-wrap\">\n",
            "      <table class=\"code-table-default\">\n",
            "        <thead>\n",
            "          <tr>\n",
            "            <th>Variable</th>\n",
            "            <th>Value</th>\n",
            "          </tr>\n",
            "        </thead>\n",
            "        <tbody>\n",
            "          <tr>\n",
            "            <td>PORT</td>\n",
            "            <td><input type=\"text\" class=\"code-table-input input-field\" id=\"f_var_PORT\" data-var-key=\"PORT\" value=\"8080\"></td>\n",
            "          </tr>\n",
            "        </tbody>\n",
            "      </table>\n",
            "    </div>\n"
        );
        assert_eq!(out, expected);
    }

    #[test]
    fn test_code_feature_renders_table_variables_empty() {
        let feature = CodeFeature::new();
        let element = DocumentElement::table_variables(std::collections::HashMap::new());
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
        assert!(out.is_empty());
    }

    #[test]
    fn test_code_feature_renders_table_variables_sorted_and_escaped() {
        let feature = CodeFeature::new();
        let mut vars = std::collections::HashMap::new();
        vars.insert("Z_KEY".into(), "<script>alert('xss')</script>".into());
        vars.insert("A_KEY".into(), "value & \"quotes\"".into());
        let element = DocumentElement::table_variables(vars);
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
            "  <div class=\"code-table-wrap\">\n",
            "    <table class=\"code-table-default\">\n",
            "      <thead>\n",
            "        <tr>\n",
            "          <th>Variable</th>\n",
            "          <th>Value</th>\n",
            "        </tr>\n",
            "      </thead>\n",
            "      <tbody>\n",
            "        <tr>\n",
            "          <td>A_KEY</td>\n",
            "          <td><input type=\"text\" class=\"code-table-input input-field\" id=\"f_var_A_KEY\" data-var-key=\"A_KEY\" value=\"value &amp; &quot;quotes&quot;\"></td>\n",
            "        </tr>\n",
            "        <tr>\n",
            "          <td>Z_KEY</td>\n",
            "          <td><input type=\"text\" class=\"code-table-input input-field\" id=\"f_var_Z_KEY\" data-var-key=\"Z_KEY\" value=\"&lt;script&gt;alert(&#39;xss&#39;)&lt;/script&gt;\"></td>\n",
            "        </tr>\n",
            "      </tbody>\n",
            "    </table>\n",
            "  </div>\n"
        );
        assert_eq!(out, expected);
    }

    #[test]
    fn test_code_feature_empty_for_unsupported_elements() {
        let feature = CodeFeature::new();
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
