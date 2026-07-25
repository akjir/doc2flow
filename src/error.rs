//! Error handling and diagnostic reporting module for Doc2Flow.

use anyhow::Error;
use std::borrow::Cow;
use std::fmt::Write;

/// Static buffer of carets for zero-allocation caret borrowing on typical line lengths.
const STATIC_CARETS: &str =
    "^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^";

/// Compiler-style diagnostic error representation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DiagnosticError<'a> {
    /// High-level error summary.
    pub message: Cow<'a, str>,
    /// Path to the file where the error occurred.
    pub file_path: Cow<'a, str>,
    /// Line number where the error occurred (1-based).
    pub line_number: usize,
    /// Column number where the error occurred (1-based).
    pub col_number: usize,
    /// Raw line content snippet from the source file.
    pub line_snippet: Cow<'a, str>,
    /// Caret string pointing to the error location (e.g. `^^^`).
    pub annotation_carets: Cow<'a, str>,
    /// Explanation accompanying the carets.
    pub annotation_text: Cow<'a, str>,
    /// Actionable advice on how to resolve the error.
    pub help_text: Cow<'a, str>,
}

impl<'a> DiagnosticError<'a> {
    /// Formats the diagnostic error into a rustc-style string.
    pub fn render(&self) -> String {
        let line_len = self.line_number.checked_ilog10().unwrap_or(0) as usize + 1;
        let cap = 80
            + self.message.len()
            + self.file_path.len()
            + self.line_snippet.len()
            + self.annotation_carets.len()
            + self.annotation_text.len()
            + self.help_text.len()
            + line_len * 4;

        let mut out = String::with_capacity(cap);

        let _ = write!(
            out,
            "error: {}\n --> {}:{}:{}\n  {:>width$}|\n{} | {}\n  {:>width$}| {} {}\n  {:>width$}|\n= help: {}",
            self.message,
            self.file_path,
            self.line_number,
            self.col_number,
            "",
            self.line_number,
            self.line_snippet,
            "",
            self.annotation_carets,
            self.annotation_text,
            "",
            self.help_text,
            width = line_len
        );

        out
    }

    /// Converts the diagnostic error into an `anyhow::Error`.
    pub fn to_anyhow(&self) -> Error {
        anyhow::anyhow!("{}", self.render())
    }

    /// Builder for missing frontmatter field errors.
    pub fn missing_frontmatter_field(file_path: &'a str, line_no: usize) -> Error {
        DiagnosticError {
            message: Cow::Borrowed("missing required frontmatter field 'customer'"),
            file_path: Cow::Borrowed(file_path),
            line_number: line_no,
            col_number: 1,
            line_snippet: Cow::Borrowed("---"),
            annotation_carets: Cow::Borrowed("^^^"),
            annotation_text: Cow::Borrowed(
                "frontmatter block defined here is missing required field 'customer'",
            ),
            help_text: Cow::Borrowed(
                "add 'customer: \"Company Name\"' to the YAML frontmatter block at the top of your Markdown file.",
            ),
        }
        .to_anyhow()
    }

    /// Builder for empty frontmatter field errors.
    pub fn empty_frontmatter_field(
        file_path: &'a str,
        line_no: usize,
        line_content: &'a str,
    ) -> Error {
        let line_len = line_content
            .trim_end()
            .len()
            .max(1)
            .min(STATIC_CARETS.len());
        let carets = Cow::Borrowed(&STATIC_CARETS[..line_len]);

        DiagnosticError {
            message: Cow::Borrowed("required frontmatter field 'customer' cannot be empty"),
            file_path: Cow::Borrowed(file_path),
            line_number: line_no,
            col_number: 1,
            line_snippet: Cow::Borrowed(line_content),
            annotation_carets: carets,
            annotation_text: Cow::Borrowed("'customer' field value cannot be empty"),
            help_text: Cow::Borrowed("provide a valid company name, e.g. customer: \"Acme Corp\""),
        }
        .to_anyhow()
    }

    /// Builder for missing frontmatter block errors.
    pub fn missing_frontmatter_block(file_path: &'a str, first_line: &'a str) -> Error {
        DiagnosticError {
            message: Cow::Borrowed(
                "missing YAML frontmatter block with required field 'customer'",
            ),
            file_path: Cow::Borrowed(file_path),
            line_number: 1,
            col_number: 1,
            line_snippet: Cow::Borrowed(first_line),
            annotation_carets: Cow::Borrowed("^"),
            annotation_text: Cow::Borrowed(
                "missing frontmatter section '---' with required field 'customer' at start of file",
            ),
            help_text: Cow::Borrowed(
                "add YAML frontmatter at the top of your Markdown file:\n          ---\n          title: \"Document Title\"\n          customer: \"Company Name\"\n          date: \"YYYY-MM-DD\"\n          ---",
            ),
        }
        .to_anyhow()
    }
}

/// Prints a standardized warning message to stderr.
pub fn print_warning(message: &str) {
    eprintln!("Warning: {}", message);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_diagnostic_error_render() {
        let diag = DiagnosticError {
            message: "test error".into(),
            file_path: "test.md".into(),
            line_number: 5,
            col_number: 1,
            line_snippet: "test line".into(),
            annotation_carets: "^^^".into(),
            annotation_text: "annotation text".into(),
            help_text: "help text".into(),
        };

        let rendered = diag.render();
        assert!(rendered.contains("error: test error"));
        assert!(rendered.contains("--> test.md:5:1"));
        assert!(rendered.contains("5 | test line"));
        assert!(rendered.contains("^^^ annotation text"));
        assert!(rendered.contains("= help: help text"));
    }

    #[test]
    fn test_missing_frontmatter_field_builder() {
        let err = DiagnosticError::missing_frontmatter_field("doc.md", 1);
        let err_str = err.to_string();
        assert!(err_str.contains("error: missing required frontmatter field 'customer'"));
        assert!(err_str.contains("--> doc.md:1:1"));
        assert!(err_str.contains("1 | ---"));
    }

    #[test]
    fn test_empty_frontmatter_field_builder() {
        let err = DiagnosticError::empty_frontmatter_field("doc.md", 3, "customer: \"\"");
        let err_str = err.to_string();
        assert!(err_str.contains("error: required frontmatter field 'customer' cannot be empty"));
        assert!(err_str.contains("--> doc.md:3:1"));
        assert!(err_str.contains("3 | customer: \"\""));
    }

    #[test]
    fn test_missing_frontmatter_block_builder() {
        let err = DiagnosticError::missing_frontmatter_block("doc.md", "# Title");
        let err_str = err.to_string();
        assert!(
            err_str
                .contains("error: missing YAML frontmatter block with required field 'customer'")
        );
        assert!(err_str.contains("--> doc.md:1:1"));
        assert!(err_str.contains("1 | # Title"));
    }
}
