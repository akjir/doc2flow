//! Table vertical slice feature module.

use crate::core::document::{DocumentElement, DocumentParameters, TableAlignment};
use crate::core::feature::Feature;
use crate::core::format::{format_inline_into, push_indent};
use crate::core::renderer::HtmlRenderer;

/// Embedded table CSS stylesheet.
pub const CSS: &str = include_str!("table.css");

/// Embedded table JavaScript client script.
pub const JS: &str = include_str!("table.js");

/// Table feature renderer handling tabular layout and column alignments.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
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
    #[must_use]
    pub const fn new() -> Self {
        Self
    }
}

impl Feature for TableFeature {
    /// Intercepts the rendering of table elements into the output buffer.
    fn try_render_body(
        &self,
        element: &DocumentElement,
        indent: usize,
        _depth: usize,
        _parameters: &DocumentParameters,
        out: &mut String,
        _renderer: &HtmlRenderer,
    ) -> bool {
        match element {
            DocumentElement::Table { alignments, rows } => {
                if rows.is_empty() {
                    return true;
                }

                push_indent(out, indent);
                out.push_str("<div class=\"table-wrap\">\n");

                push_indent(out, indent + 1);
                out.push_str("<table class=\"table-default\">\n");

                if let Some(header_row) = rows.first() {
                    push_indent(out, indent + 2);
                    out.push_str("<thead>\n");

                    push_indent(out, indent + 3);
                    out.push_str("<tr>\n");

                    for (col_idx, cell) in header_row.iter().enumerate() {
                        push_indent(out, indent + 4);
                        out.push_str("<th");
                        match alignments.get(col_idx) {
                            Some(TableAlignment::Left) => {
                                out.push_str(" style=\"text-align: left;\"")
                            }
                            Some(TableAlignment::Center) => {
                                out.push_str(" style=\"text-align: center;\"")
                            }
                            Some(TableAlignment::Right) => {
                                out.push_str(" style=\"text-align: right;\"")
                            }
                            _ => {}
                        }
                        out.push('>');
                        format_inline_into(out, cell);
                        out.push_str("</th>\n");
                    }

                    push_indent(out, indent + 3);
                    out.push_str("</tr>\n");

                    push_indent(out, indent + 2);
                    out.push_str("</thead>\n");
                }

                if rows.len() > 1 {
                    push_indent(out, indent + 2);
                    out.push_str("<tbody>\n");

                    for row in &rows[1..] {
                        push_indent(out, indent + 3);
                        out.push_str("<tr>\n");

                        for (col_idx, cell) in row.iter().enumerate() {
                            push_indent(out, indent + 4);
                            out.push_str("<td");
                            match alignments.get(col_idx) {
                                Some(TableAlignment::Left) => {
                                    out.push_str(" style=\"text-align: left;\"")
                                }
                                Some(TableAlignment::Center) => {
                                    out.push_str(" style=\"text-align: center;\"")
                                }
                                Some(TableAlignment::Right) => {
                                    out.push_str(" style=\"text-align: right;\"")
                                }
                                _ => {}
                            }
                            out.push('>');
                            format_inline_into(out, cell);
                            out.push_str("</td>\n");
                        }

                        push_indent(out, indent + 3);
                        out.push_str("</tr>\n");
                    }

                    push_indent(out, indent + 2);
                    out.push_str("</tbody>\n");
                }

                push_indent(out, indent + 1);
                out.push_str("</table>\n");

                push_indent(out, indent);
                out.push_str("</div>\n");

                true
            }
            _ => false,
        }
    }

    /// Returns the embedded CSS stylesheet for the table feature.
    fn css(&self) -> Option<&'static str> {
        Some(CSS)
    }

    /// Returns the embedded JavaScript client scripts for the table feature.
    fn javascript(&self) -> &[&'static str] {
        &[JS]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_table_feature_constructor_new() {
        let feature = TableFeature::new();
        assert_eq!(feature, TableFeature);
    }

    #[test]
    fn test_table_feature_css() {
        let feature = TableFeature::new();
        let css = feature.css().expect("table css should exist");
        assert!(css.contains("--table-border-color:"));
        assert!(css.contains("--table-header-bg:"));
        assert!(css.contains("--table-row-even-bg:"));
        assert!(css.contains("--table-row-hover-bg:"));
        assert!(css.contains(".table-wrap"));
        assert!(css.contains(".table-default"));
    }

    #[test]
    fn test_table_feature_javascript() {
        let feature = TableFeature::new();
        let js = feature.javascript();
        assert_eq!(js.len(), 1);
        let script = js[0];
        assert!(script.contains("window.d2f.table"));
        assert!(script.contains("table-row-hover"));
        assert!(script.contains("table-default"));
    }

    #[test]
    fn test_table_feature_empty_for_unsupported_elements() {
        let feature = TableFeature::new();
        let text = DocumentElement::text("Regular text");
        let mut out = String::new();
        let renderer = HtmlRenderer::default_renderer();
        assert!(!feature.try_render_body(
            &text,
            0,
            0,
            &DocumentParameters::default(),
            &mut out,
            &renderer,
        ));
        assert!(out.is_empty());
    }

    #[test]
    fn test_table_feature_empty_rows() {
        let feature = TableFeature::new();
        let element = DocumentElement::table(vec![], vec![]);
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
        assert!(out.is_empty());
    }

    #[test]
    fn test_table_feature_header_only() {
        let feature = TableFeature::new();
        let element = DocumentElement::table(
            vec![TableAlignment::None, TableAlignment::None],
            vec![vec!["Col 1".into(), "Col 2".into()]],
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
            "<div class=\"table-wrap\">\n",
            "  <table class=\"table-default\">\n",
            "    <thead>\n",
            "      <tr>\n",
            "        <th>Col 1</th>\n",
            "        <th>Col 2</th>\n",
            "      </tr>\n",
            "    </thead>\n",
            "  </table>\n",
            "</div>\n"
        );
        assert_eq!(out, expected);
    }

    #[test]
    fn test_table_feature_renders_alignments() {
        let feature = TableFeature::new();
        let element = DocumentElement::table(
            vec![
                TableAlignment::Left,
                TableAlignment::Center,
                TableAlignment::Right,
                TableAlignment::None,
            ],
            vec![
                vec!["L".into(), "C".into(), "R".into(), "N".into()],
                vec!["1".into(), "2".into(), "3".into(), "4".into()],
            ],
        );
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
            "  <div class=\"table-wrap\">\n",
            "    <table class=\"table-default\">\n",
            "      <thead>\n",
            "        <tr>\n",
            "          <th style=\"text-align: left;\">L</th>\n",
            "          <th style=\"text-align: center;\">C</th>\n",
            "          <th style=\"text-align: right;\">R</th>\n",
            "          <th>N</th>\n",
            "        </tr>\n",
            "      </thead>\n",
            "      <tbody>\n",
            "        <tr>\n",
            "          <td style=\"text-align: left;\">1</td>\n",
            "          <td style=\"text-align: center;\">2</td>\n",
            "          <td style=\"text-align: right;\">3</td>\n",
            "          <td>4</td>\n",
            "        </tr>\n",
            "      </tbody>\n",
            "    </table>\n",
            "  </div>\n"
        );
        assert_eq!(out, expected);
    }

    #[test]
    fn test_table_feature_renders_inline_formatting() {
        let feature = TableFeature::new();
        let element = DocumentElement::table(
            vec![TableAlignment::None, TableAlignment::None],
            vec![
                vec!["**Header Bold**".into(), "*Header Italic*".into()],
                vec![
                    "`code` and [link](https://example.com)".into(),
                    "<special> & ~~strike~~".into(),
                ],
            ],
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
            "<div class=\"table-wrap\">\n",
            "  <table class=\"table-default\">\n",
            "    <thead>\n",
            "      <tr>\n",
            "        <th><strong>Header Bold</strong></th>\n",
            "        <th><em>Header Italic</em></th>\n",
            "      </tr>\n",
            "    </thead>\n",
            "    <tbody>\n",
            "      <tr>\n",
            "        <td><code>code</code> and <a href=\"https://example.com\">link</a></td>\n",
            "        <td>&lt;special&gt; &amp; <s>strike</s></td>\n",
            "      </tr>\n",
            "    </tbody>\n",
            "  </table>\n",
            "</div>\n"
        );
        assert_eq!(out, expected);
    }
}
