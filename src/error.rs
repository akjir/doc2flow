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
            message: Cow::Borrowed("missing required frontmatter field 'company'"),
            file_path: Cow::Borrowed(file_path),
            line_number: line_no,
            col_number: 1,
            line_snippet: Cow::Borrowed("---"),
            annotation_carets: Cow::Borrowed("^^^"),
            annotation_text: Cow::Borrowed(
                "frontmatter block defined here is missing required field 'company'",
            ),
            help_text: Cow::Borrowed(
                "add 'company: \"Company Name\"' to the YAML frontmatter block at the top of your Markdown file.",
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
            message: Cow::Borrowed("required frontmatter field 'company' cannot be empty"),
            file_path: Cow::Borrowed(file_path),
            line_number: line_no,
            col_number: 1,
            line_snippet: Cow::Borrowed(line_content),
            annotation_carets: carets,
            annotation_text: Cow::Borrowed("'company' field value cannot be empty"),
            help_text: Cow::Borrowed("provide a valid company name, e.g. company: \"Acme Corp\""),
        }
        .to_anyhow()
    }

    /// Builder for missing frontmatter block errors.
    pub fn missing_frontmatter_block(file_path: &'a str, first_line: &'a str) -> Error {
        DiagnosticError {
            message: Cow::Borrowed(
                "missing YAML frontmatter block with required field 'company'",
            ),
            file_path: Cow::Borrowed(file_path),
            line_number: 1,
            col_number: 1,
            line_snippet: Cow::Borrowed(first_line),
            annotation_carets: Cow::Borrowed("^"),
            annotation_text: Cow::Borrowed(
                "missing frontmatter section '---' with required field 'company' at start of file",
            ),
            help_text: Cow::Borrowed(
                "add YAML frontmatter at the top of your Markdown file:\n          ---\n          title: \"Document Title\"\n          company: \"Company Name\"\n          date: \"YYYY-MM-DD\"\n          ---",
            ),
        }
        .to_anyhow()
    }

    /// Builder for local image size exceeding maximum limit errors.
    pub fn image_too_large(
        file_path: &'a str,
        line_no: usize,
        col_no: usize,
        line_snippet: &'a str,
        src_val: &'a str,
        size_bytes: u64,
    ) -> Error {
        let size_kb = size_bytes as f64 / 1024.0;
        let line_len = line_snippet.len().max(1);

        let carets = if col_no > 0 && !line_snippet.is_empty() {
            let padding_len = col_no.saturating_sub(1);
            let span_len = src_val
                .len()
                .max(1)
                .min(line_len.saturating_sub(padding_len).max(1));
            let mut s = String::with_capacity(padding_len + span_len);
            for _ in 0..padding_len {
                s.push(' ');
            }
            for _ in 0..span_len {
                s.push('^');
            }
            Cow::Owned(s)
        } else {
            let carets_len = src_val.len().max(1).min(line_len);
            Cow::Borrowed(&STATIC_CARETS[..carets_len.min(STATIC_CARETS.len())])
        };

        DiagnosticError {
            message: Cow::Owned(format!(
                "image '{src_val}' exceeds maximum allowed size of 250 KB ({size_kb:.1} KB)"
            )),
            file_path: Cow::Borrowed(file_path),
            line_number: line_no,
            col_number: col_no,
            line_snippet: Cow::Borrowed(line_snippet),
            annotation_carets: carets,
            annotation_text: Cow::Owned(format!(
                "local image size ({size_kb:.1} KB) exceeds 250 KB limit"
            )),
            help_text: Cow::Owned(format!(
                "reduce image resolution or compress '{src_val}' below 250 KB before embedding."
            )),
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
        assert!(err_str.contains("error: missing required frontmatter field 'company'"));
        assert!(err_str.contains("--> doc.md:1:1"));
        assert!(err_str.contains("1 | ---"));
    }

    #[test]
    fn test_empty_frontmatter_field_builder() {
        let err = DiagnosticError::empty_frontmatter_field("doc.md", 3, "company: \"\"");
        let err_str = err.to_string();
        assert!(err_str.contains("error: required frontmatter field 'company' cannot be empty"));
        assert!(err_str.contains("--> doc.md:3:1"));
        assert!(err_str.contains("3 | company: \"\""));
    }

    #[test]
    fn test_missing_frontmatter_block_builder() {
        let err = DiagnosticError::missing_frontmatter_block("doc.md", "# Title");
        let err_str = err.to_string();
        assert!(
            err_str.contains("error: missing YAML frontmatter block with required field 'company'")
        );
        assert!(err_str.contains("--> doc.md:1:1"));
        assert!(err_str.contains("1 | # Title"));
    }

    #[test]
    fn test_image_too_large_builder() {
        let err = DiagnosticError::image_too_large(
            "doc.md",
            12,
            16,
            "![Diagram](images/large_photo.png)",
            "images/large_photo.png",
            300 * 1024,
        );
        let err_str = err.to_string();
        assert!(err_str.contains("error: image 'images/large_photo.png' exceeds maximum allowed size of 250 KB (300.0 KB)"));
        assert!(err_str.contains("--> doc.md:12:16"));
        assert!(err_str.contains("12 | ![Diagram](images/large_photo.png)"));
        assert!(err_str.contains("^^^^^^^^^^^^^^^^^^^ local image size (300.0 KB) exceeds 250 KB limit"));
        assert!(err_str.contains("= help: reduce image resolution or compress 'images/large_photo.png' below 250 KB before embedding."));
    }
}
