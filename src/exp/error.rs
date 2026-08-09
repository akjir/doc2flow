//! Error types and diagnostic definitions for experimental pipeline.

use std::borrow::Cow;
use std::fmt::{self, Display, Formatter, Write};

/// Result type alias for experimental operations.
pub type Result<T, E = Error> = std::result::Result<T, E>;

/// Centralized error type for experimental operations.
#[derive(Debug)]
pub enum Error {
    /// Rendered compiler-style diagnostic error string.
    Diagnostic(String),
    /// General message error.
    Message(String),
}

impl std::error::Error for Error {}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::Diagnostic(msg) => write!(f, "{msg}"),
            Self::Message(msg) => write!(f, "{msg}"),
        }
    }
}

impl From<&str> for Error {
    fn from(msg: &str) -> Self {
        Self::Message(msg.to_string())
    }
}

impl From<String> for Error {
    fn from(msg: String) -> Self {
        Self::Message(msg)
    }
}

impl<'a> From<DiagnosticError<'a>> for Error {
    fn from(err: DiagnosticError<'a>) -> Self {
        Self::Diagnostic(err.render())
    }
}

impl From<&DiagnosticError<'_>> for Error {
    fn from(err: &DiagnosticError<'_>) -> Self {
        Self::Diagnostic(err.render())
    }
}

/// Static buffer of carets for zero-allocation caret borrowing on typical line lengths.
const STATIC_CARETS: &str =
    "^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^";

/// Constructs a diagnostic caret annotation string pointing to a source location.
pub fn build_caret_annotation(col_no: usize, span_len: usize, max_len: usize) -> Cow<'static, str> {
    let max_len = max_len.max(1);
    let span = span_len.max(1);
    let padding_len = col_no.saturating_sub(1);

    if padding_len == 0 {
        let effective_span = span.min(max_len);
        if effective_span <= STATIC_CARETS.len() {
            Cow::Borrowed(&STATIC_CARETS[..effective_span.min(STATIC_CARETS.len())])
        } else {
            let mut s = String::with_capacity(effective_span);
            for _ in 0..effective_span {
                s.push('^');
            }
            Cow::Owned(s)
        }
    } else {
        let effective_span = span.min(max_len.saturating_sub(padding_len).max(1));
        let total_len = padding_len.saturating_add(effective_span);
        let mut s = String::with_capacity(total_len);
        for _ in 0..padding_len {
            s.push(' ');
        }
        for _ in 0..effective_span {
            s.push('^');
        }
        Cow::Owned(s)
    }
}

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

impl Display for DiagnosticError<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_str(&self.render())
    }
}

impl std::error::Error for DiagnosticError<'_> {}

impl DiagnosticError<'_> {
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

        let _ = out.write_str("error: ");
        let _ = out.write_str(&self.message);
        let _ = out.write_str("\n --> ");
        let _ = writeln!(
            out,
            "{}:{}:{}",
            self.file_path, self.line_number, self.col_number
        );
        let _ = writeln!(out, "{:>width$} |", "", width = line_len);
        let _ = writeln!(
            out,
            "{:>width$} | {}",
            self.line_number,
            self.line_snippet,
            width = line_len
        );
        let _ = writeln!(
            out,
            "{:>width$} | {} {}",
            "",
            self.annotation_carets,
            self.annotation_text,
            width = line_len
        );
        let _ = writeln!(out, "{:>width$} |", "", width = line_len);
        let _ = out.write_str("= help: ");
        let _ = out.write_str(&self.help_text);

        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let err = Error::Message("test error".to_string());
        assert_eq!(err.to_string(), "test error");

        let diag_err = Error::Diagnostic("diagnostic error".to_string());
        assert_eq!(diag_err.to_string(), "diagnostic error");
    }

    #[test]
    fn test_error_from_str() {
        let err: Error = "test from str".into();
        assert_eq!(err.to_string(), "test from str");
    }

    #[test]
    fn test_diagnostic_error_render() {
        let diag = DiagnosticError {
            message: "missing frontmatter delimiter '---'".into(),
            file_path: "<input>".into(),
            line_number: 1,
            col_number: 1,
            line_snippet: "# Title".into(),
            annotation_carets: "^^^^^^^".into(),
            annotation_text: "expected '---' to begin frontmatter".into(),
            help_text: "document must begin with frontmatter enclosed by '---' delimiters.".into(),
        };

        let rendered = diag.render();
        assert!(rendered.contains("error: missing frontmatter delimiter '---'"));
        assert!(rendered.contains("--> <input>:1:1"));
        assert!(rendered.contains("1 | # Title"));
        assert!(rendered.contains("^^^^^^^ expected '---' to begin frontmatter"));
        assert!(rendered.contains(
            "= help: document must begin with frontmatter enclosed by '---' delimiters."
        ));
    }

    #[test]
    fn test_diagnostic_error_from_conversion() {
        let diag = DiagnosticError {
            message: "unclosed frontmatter block".into(),
            file_path: "<input>".into(),
            line_number: 1,
            col_number: 1,
            line_snippet: "---".into(),
            annotation_carets: "^^^".into(),
            annotation_text: "frontmatter starting here is never closed".into(),
            help_text: "close the frontmatter block with a closing '---' line.".into(),
        };

        let err: Error = diag.into();
        assert!(err.to_string().contains("unclosed frontmatter block"));
    }

    #[test]
    fn test_caret_annotation() {
        let carets = build_caret_annotation(1, 3, 10);
        assert_eq!(carets, "^^^");

        let padded = build_caret_annotation(4, 3, 10);
        assert_eq!(padded, "   ^^^");
    }

    #[test]
    fn test_diagnostic_error_gutter_alignment() {
        let diag_2digit = DiagnosticError {
            message: "unclosed frontmatter block".into(),
            file_path: "<input>".into(),
            line_number: 58,
            col_number: 1,
            line_snippet: "---".into(),
            annotation_carets: "^^^".into(),
            annotation_text: "frontmatter starting here is never closed".into(),
            help_text: "close the frontmatter block with a closing '---' line.".into(),
        };

        let rendered = diag_2digit.render();
        assert!(
            rendered.contains(
                "   |\n58 | ---\n   | ^^^ frontmatter starting here is never closed\n   |"
            )
        );
    }
}
