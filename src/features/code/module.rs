//! Code vertical slice feature module.

use std::fmt::Write as _;

use crate::core::document::{DocumentElement, DocumentElementId, DocumentParameters};
use crate::core::feature::FeatureModule;
use crate::core::format::{escape_html_into, push_indent};
use crate::core::renderer::{DocumentElementRenderer, HtmlRenderer};

/// Embedded code CSS styles for code blocks.
pub const CSS: &str = include_str!("code.css");

/// Embedded code JavaScript client script.
pub const JS: &str = include_str!("code.js");

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
                let copy_label = crate::core::language::localize("copy_code");
                let copied_label = crate::core::language::localize("copied");
                push_indent(out, indent);
                let _ = write!(
                    out,
                    "<pre class=\"code-default\" data-label-copy=\"{copy_label}\" data-label-copied=\"{copied_label}\"><code>"
                );
                escape_html_into(out, content);
                out.push_str("</code></pre>\n");
            }
            DocumentElement::TableVariables { variables } => {
                if variables.is_empty() {
                    return;
                }

                let var_header = crate::core::language::localize("var_table_variable");
                let val_header = crate::core::language::localize("var_table_value");

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
                let _ = write!(out, "<th>{var_header}</th>\n");

                push_indent(out, indent + 4);
                let _ = write!(out, "<th>{val_header}</th>\n");

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

    fn javascript(&self) -> &[&'static str] {
        &[JS]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_code_feature_javascript() {
        let feature = CodeFeature::new();
        let js_files = feature.javascript();
        assert_eq!(js_files.len(), 1);
        let js = js_files[0];
        assert!(js.contains("copyCode"));
        assert!(js.contains("updateAllCodeVariables"));
        assert!(js.contains("resetCodeVariables"));
        assert!(js.contains("data-raw-code"));
        assert!(js.contains("code-copy-btn"));
        assert!(js.contains("copied"));
        assert!(js.contains("code-table-input"));
        assert!(js.contains("navigator.clipboard"));
    }

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
        assert!(css.contains(".code-copy-btn"));
        assert!(css.contains(".code-copy-btn.copied"));
        assert!(css.contains("--code-table-bg:"));
        assert!(css.contains(".code-table-wrap"));
        assert!(css.contains(".code-table-default"));
        assert!(css.contains(".code-table-input"));
    }

    #[test]
    fn test_code_feature_renders_code_block() {
        let _guard = crate::core::language::TEST_I18N_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        crate::core::language::init("en");
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
            "  <pre class=\"code-default\" data-label-copy=\"Copy code\" data-label-copied=\"Copied!\"><code>fn main() {\n    println!(&quot;hi&quot;);\n}</code></pre>\n"
        );
    }

    #[test]
    fn test_code_feature_escapes_html_entities() {
        let _guard = crate::core::language::TEST_I18N_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        crate::core::language::init("en");
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
            "    <pre class=\"code-default\" data-label-copy=\"Copy code\" data-label-copied=\"Copied!\"><code>&lt;div class=&quot;foo&quot;&gt; &amp;&amp; &#39;bar&#39;&lt;/div&gt;</code></pre>\n"
        );
    }

    #[test]
    fn test_code_feature_renders_table_variables() {
        let _guard = crate::core::language::TEST_I18N_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        crate::core::language::init("en");
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
    fn test_code_feature_renders_german_labels() {
        let _guard = crate::core::language::TEST_I18N_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        crate::core::language::init("de");
        let feature = CodeFeature::new();
        let code_el = DocumentElement::code_block(Some("rust"), "let x = 1;");
        let mut out = String::new();
        let renderer = HtmlRenderer::default_renderer();
        feature.render_element(
            &code_el,
            0,
            0,
            &DocumentParameters::default(),
            &mut out,
            &renderer,
        );
        assert!(out.contains("data-label-copy=\"Code kopieren\""));
        assert!(out.contains("data-label-copied=\"Kopiert!\""));

        let mut vars = std::collections::HashMap::new();
        vars.insert("HOST".into(), "localhost".into());
        let var_el = DocumentElement::table_variables(vars);
        let mut var_out = String::new();
        feature.render_element(
            &var_el,
            0,
            0,
            &DocumentParameters::default(),
            &mut var_out,
            &renderer,
        );
        assert!(var_out.contains("<th>Variable</th>"));
        assert!(var_out.contains("<th>Wert</th>"));
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
        let _guard = crate::core::language::TEST_I18N_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        crate::core::language::init("en");
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
