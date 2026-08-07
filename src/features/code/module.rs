//! Code block and clipboard copying feature slice.

use crate::core::feature::{DocumentContext, Feature};
use std::fmt::Write;

/// Code copy SVG icon button element.
pub const COPY_ICON_SVG: &str = r#"<svg aria-hidden="true" class="svg-icon iconCopy" width="14" height="15" viewBox="0 0 17 18"><path fill="currentColor" d="M5 6c0-1.09.91-2 2-2h4.5L15 7.5V15c0 1.09-.91 2-2 2H7c-1.09 0-2-.91-2-2zm6-1.25V8h3.25z"/><path fill="currentColor" d="M10 1a2 2 0 0 1 2 2H6a2 2 0 0 0-2 2v9a2 2 0 0 1-2-2V4a3 3 0 0 1 3-3z" opacity=".4"/></svg>"#;

/// Renders a code block component with header, copy button, and syntax language badge.
#[inline]
pub fn render_code_block(
    out: &mut impl Write,
    lang_opt: Option<&str>,
    escaped_code: &str,
    copy_label: &str,
) {
    let _ = out.write_str("<div class=\"code-block-wrap\"><div class=\"code-header\">");
    if let Some(lang) = lang_opt {
        let _ = write!(out, "<span class=\"code-lang\">{lang}</span>");
    }
    let _ = write!(
        out,
        "<button class=\"copy-btn\" onclick=\"window.d2f_code.copy(this)\" title=\"{copy_label}\" aria-label=\"{copy_label}\">{COPY_ICON_SVG}</button></div><pre class=\"code-block"
    );
    if let Some(lang) = lang_opt {
        let _ = write!(out, " language-{lang}");
    }
    let _ = writeln!(out, "\"><code>{escaped_code}</code></pre></div>");
}

/// Renders an annotated `[Variables]` key-value table component into the output buffer.
#[inline]
pub fn render_variable_table<K: AsRef<str>, V: AsRef<str>>(
    out: &mut impl Write,
    col_variable: &str,
    col_value: &str,
    rows: &[(K, V)],
    json_payload: &str,
) {
    let escaped_json = crate::converter::html_escape(json_payload);
    let escaped_col_var = crate::converter::html_escape(col_variable);
    let escaped_col_val = crate::converter::html_escape(col_value);

    let _ = writeln!(
        out,
        "<div class=\"item-table-var-wrap\"><table class=\"item-table-var\" data-variables=\"{escaped_json}\"><thead><tr><th>{escaped_col_var}</th><th>{escaped_col_val}</th></tr></thead><tbody>"
    );
    for (k, v) in rows {
        let escaped_k = crate::converter::html_escape(k.as_ref());
        let escaped_v = crate::converter::html_escape(v.as_ref());
        let _ = write!(
            out,
            "<tr><td>{escaped_k}</td><td><input type=\"text\" class=\"item-table-var-input persistent-field\" id=\"f_var_{escaped_k}\" data-var-key=\"{escaped_k}\" data-default-value=\"{escaped_v}\" value=\"{escaped_v}\"></td></tr>\n"
        );
    }
    let _ = out.write_str("</tbody></table></div>\n");
}

/// Unified code feature slice providing syntax block styling, variable substitution, and clipboard copying.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct CodeFeature;

impl CodeFeature {
    /// Creates a new code feature instance.
    ///
    /// # Examples
    ///
    /// ```
    /// use doc2flow::features::code::CodeFeature;
    ///
    /// let feature = CodeFeature::new();
    /// ```
    #[inline]
    pub const fn new() -> Self {
        Self
    }
}

impl Feature for CodeFeature {
    /// Returns the unique feature identifier "code".
    #[inline]
    fn name(&self) -> &'static str {
        "code"
    }

    /// Evaluates if the code feature is enabled based solely on the presence of code blocks in markdown.
    #[inline]
    fn is_enabled(&self, ctx: &DocumentContext) -> bool {
        ctx.raw_markdown.contains("```")
    }

    /// Returns embedded TypeScript client script for code block variables and clipboard copying.
    #[inline]
    fn javascript(&self) -> Option<&'static str> {
        Some(include_str!("code.ts"))
    }

    /// Returns embedded CSS styles for code blocks, copy button, and variables table.
    #[inline]
    fn css(&self) -> Option<&'static str> {
        Some(include_str!("code.css"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_code_feature_activation_logic() {
        let feature = CodeFeature::new();
        let fm = HashMap::new();

        // 1. Markdown with fenced code blocks: enabled
        let ctx_with_code = DocumentContext::new(&fm, "```rust\nfn main() {}\n```");
        assert!(feature.is_enabled(&ctx_with_code));

        let ctx_inline_fence = DocumentContext::new(&fm, "Here is ```bash\necho test\n``` snippet.");
        assert!(feature.is_enabled(&ctx_inline_fence));

        // 2. Markdown without code blocks: disabled
        let ctx_no_code = DocumentContext::new(&fm, "# Hello World\nJust normal documentation.");
        assert!(!feature.is_enabled(&ctx_no_code));

        let ctx_empty = DocumentContext::new(&fm, "");
        assert!(!feature.is_enabled(&ctx_empty));
    }

    #[test]
    fn test_code_assets_embedded() {
        let feature = CodeFeature::new();
        assert_eq!(feature.name(), "code");
        let js = feature.javascript().expect("JavaScript must be embedded");
        let css = feature.css().expect("CSS must be embedded");

        assert!(js.contains("copyCode") || js.contains("copy"));
        assert!(css.contains(".copy-btn"));
        assert!(css.contains(".code-block"));
        assert!(css.contains(".item-table-var"));
    }

    #[test]
    fn test_render_code_block() {
        let mut buf = String::new();
        render_code_block(&mut buf, Some("rust"), "fn main() {}", "Copy");
        assert!(buf.contains("<span class=\"code-lang\">rust</span>"));
        assert!(buf.contains("language-rust"));
        assert!(buf.contains("<code>fn main() {}</code>"));
    }

    #[test]
    fn test_render_variable_table() {
        let mut buf = String::new();
        let rows = vec![("BLOCK".to_string(), "prod-server".to_string())];
        let json = "{\"BLOCK\":\"prod-server\"}";
        render_variable_table(&mut buf, "Variable", "Value", &rows, json);
        assert!(buf.contains("<div class=\"item-table-var-wrap\">"));
        assert!(buf.contains("<th>Variable</th><th>Value</th>"));
        assert!(buf.contains("<td>BLOCK</td>"));
        assert!(buf.contains("class=\"item-table-var-input persistent-field\""));
        assert!(buf.contains("data-var-key=\"BLOCK\""));
        assert!(buf.contains("value=\"prod-server\""));
    }
}
