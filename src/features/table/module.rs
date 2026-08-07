//! Section table and tabular layout feature slice.

use crate::core::feature::{DocumentContext, Feature};

/// Unified section table feature slice providing responsive table wrappers, hover highlights, and print formatting.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct TableFeature;

impl TableFeature {
    /// Creates a new table feature instance.
    ///
    /// # Examples
    ///
    /// ```
    /// use doc2flow::features::table::TableFeature;
    ///
    /// let feature = TableFeature::new();
    /// ```
    #[inline]
    pub const fn new() -> Self {
        Self
    }
}

impl Feature for TableFeature {
    /// Returns the unique feature identifier "table".
    #[inline]
    fn name(&self) -> &'static str {
        "table"
    }

    /// Evaluates if the table feature is enabled based on presence of markdown tables or html table tags.
    #[inline]
    fn is_enabled(&self, ctx: &DocumentContext) -> bool {
        ctx.raw_markdown.contains('|') || ctx.raw_markdown.contains("<table")
    }

    /// Returns embedded TypeScript client script for section table hover and formatting.
    #[inline]
    fn javascript(&self) -> Option<&'static str> {
        Some(include_str!("table.ts"))
    }

    /// Returns embedded CSS styles for section table containers, rows, and print styles.
    #[inline]
    fn css(&self) -> Option<&'static str> {
        Some(include_str!("table.css"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_table_feature_activation_logic() {
        let feature = TableFeature::new();
        let fm = HashMap::new();

        // 1. Markdown with pipe table syntax: enabled
        let ctx_with_pipe_table = DocumentContext::new(
            &fm,
            "| Header 1 | Header 2 |\n| --- | --- |\n| Cell 1 | Cell 2 |",
        );
        assert!(feature.is_enabled(&ctx_with_pipe_table));

        let ctx_with_html_table =
            DocumentContext::new(&fm, "<table class=\"custom-table\"><tr><td>Data</td></tr></table>");
        assert!(feature.is_enabled(&ctx_with_html_table));

        // 2. Markdown without tables: disabled
        let ctx_no_table = DocumentContext::new(&fm, "# Hello World\nJust normal documentation.");
        assert!(!feature.is_enabled(&ctx_no_table));

        let ctx_empty = DocumentContext::new(&fm, "");
        assert!(!feature.is_enabled(&ctx_empty));
    }

    #[test]
    fn test_table_assets_embedded() {
        let feature = TableFeature::new();
        assert_eq!(feature.name(), "table");
        let js = feature.javascript().expect("JavaScript must be embedded");
        let css = feature.css().expect("CSS must be embedded");

        assert!(js.contains("initSectionTables") || js.contains("init"));
        assert!(js.contains("item-table"));
        assert!(css.contains(".item-table"));
        assert!(css.contains(".item-table-wrap"));
    }
}
