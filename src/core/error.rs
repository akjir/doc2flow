//! Error handling and diagnostic reporting module for Doc2Flow.

use std::borrow::Cow;
use std::fmt::{self, Display, Formatter, Write};
use std::path::PathBuf;

/// Result type alias for Doc2Flow operations.
pub type Result<T, E = Doc2FlowError> = std::result::Result<T, E>;

/// Centralized domain error type for all Doc2Flow operations.
#[derive(Debug)]
pub enum Doc2FlowError {
    /// Rendered compiler-style diagnostic error string.
    Diagnostic(String),
    /// Missing or insufficient identity fields (title, version, date).
    MissingIdentityFields,
    /// Image resource not found or unreadable.
    ImageNotFound(PathBuf),
    /// Image processing or encoding error.
    ImageProcess(String),
    /// Standard I/O operation error.
    Io {
        path: Option<PathBuf>,
        source: std::io::Error,
    },
    /// JSON serialization or deserialization error.
    Json(String),
    /// General application CLI or processing message error.
    Message(String),
}

impl Display for Doc2FlowError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Doc2FlowError::Diagnostic(msg) => write!(f, "{msg}"),
            Doc2FlowError::MissingIdentityFields => write!(
                f,
                "Fatal: At least 2 of the required identity fields (title, version, date) are missing in frontmatter."
            ),
            Doc2FlowError::ImageNotFound(path) => write!(f, "Image file not found: {}", path.display()),
            Doc2FlowError::ImageProcess(msg) => write!(f, "{msg}"),
            Doc2FlowError::Io { path: Some(path), source } => {
                write!(f, "I/O error at {}: {}", path.display(), source)
            }
            Doc2FlowError::Io { path: None, source } => write!(f, "I/O error: {source}"),
            Doc2FlowError::Json(msg) => write!(f, "{msg}"),
            Doc2FlowError::Message(msg) => write!(f, "{msg}"),
        }
    }
}

impl std::error::Error for Doc2FlowError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Doc2FlowError::Io { source, .. } => Some(source),
            _ => None,
        }
    }
}

impl From<std::io::Error> for Doc2FlowError {
    fn from(err: std::io::Error) -> Self {
        Doc2FlowError::Io {
            path: None,
            source: err,
        }
    }
}

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

    /// Converts the diagnostic error into a `Doc2FlowError`.
    pub fn to_doc2flow(&self) -> Doc2FlowError {
        Doc2FlowError::Diagnostic(self.render())
    }



    /// Builder for local image size exceeding maximum limit errors.
    pub fn image_too_large(
        file_path: &'a str,
        line_no: usize,
        col_no: usize,
        line_snippet: &'a str,
        src_val: &'a str,
        size_bytes: u64,
    ) -> Doc2FlowError {
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
        .to_doc2flow()
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

    #[test]
    fn test_diagnostic_error_render_multi_digit_lines() {
        let diag_2digit = DiagnosticError {
            message: "2 digit line".into(),
            file_path: "spec.md".into(),
            line_number: 42,
            col_number: 5,
            line_snippet: "company: ''".into(),
            annotation_carets: "^^^^^^^^^^^".into(),
            annotation_text: "empty".into(),
            help_text: "fix company".into(),
        };
        let rendered_2 = diag_2digit.render();
        assert!(rendered_2.contains("42 | company: ''"));

        let diag_4digit = DiagnosticError {
            message: "4 digit line".into(),
            file_path: "large_spec.md".into(),
            line_number: 1234,
            col_number: 1,
            line_snippet: "some text".into(),
            annotation_carets: "^".into(),
            annotation_text: "text".into(),
            help_text: "fix text".into(),
        };
        let rendered_4 = diag_4digit.render();
        assert!(rendered_4.contains("1234 | some text"));
    }

    #[test]
    fn test_print_warning_does_not_panic() {
        print_warning("Test warning message");
    }
}

