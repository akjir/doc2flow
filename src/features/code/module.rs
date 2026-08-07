//! Code block and clipboard copying feature slice.

use crate::core::feature::{DocumentContext, Feature};

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
}
