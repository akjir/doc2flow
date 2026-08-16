//! Error types and diagnostic definitions for Doc2Flow core engine.

use std::borrow::Cow;
use std::fmt::{self, Display, Formatter};

/// Static buffer of carets for zero-allocation caret borrowing on typical line lengths.
const STATIC_CARETS: &str =
    "^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^";

/// Errors that can occur during CLI argument parsing.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CliError {
    /// An invalid argument or flag value was provided.
    InvalidArgument(String),
    /// An option flag was provided without a required value.
    MissingArgument(String),
    /// An unexpected extra positional argument was provided.
    UnexpectedPositional(String),
}

impl Display for CliError {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidArgument(msg)
            | Self::MissingArgument(msg)
            | Self::UnexpectedPositional(msg) => write!(f, "{msg}"),
        }
    }
}

impl std::error::Error for CliError {}

/// Compiler-style diagnostic error representation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DiagnosticError<'a> {
    /// Caret string pointing to the error location (e.g. `^^^`).
    pub annotation_carets: Cow<'a, str>,
    /// Explanation accompanying the carets.
    pub annotation_text: Cow<'a, str>,
    /// Column number where the error occurred (1-based).
    pub col_number: usize,
    /// Path to the file where the error occurred.
    pub file_path: Cow<'a, str>,
    /// Actionable advice on how to resolve the error.
    pub help_text: Cow<'a, str>,
    /// Line number where the error occurred (1-based).
    pub line_number: usize,
    /// Raw line content snippet from the source file.
    pub line_snippet: Cow<'a, str>,
    /// High-level error summary.
    pub message: Cow<'a, str>,
}

impl DiagnosticError<'_> {
    /// Formats the diagnostic error into a rustc-style string.
    #[must_use]
    pub fn render(&self) -> String {
        self.to_string()
    }
}

impl Display for DiagnosticError<'_> {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        let line_len = self.line_number.checked_ilog10().unwrap_or(0) as usize + 1;

        writeln!(f, "error: {}", self.message)?;
        writeln!(
            f,
            " --> {}:{}:{}",
            self.file_path, self.line_number, self.col_number
        )?;
        writeln!(f, "{:>width$} |", "", width = line_len)?;
        writeln!(
            f,
            "{:>width$} | {}",
            self.line_number,
            self.line_snippet,
            width = line_len
        )?;
        writeln!(
            f,
            "{:>width$} | {} {}",
            "",
            self.annotation_carets,
            self.annotation_text,
            width = line_len
        )?;
        writeln!(f, "{:>width$} |", "", width = line_len)?;
        write!(f, "= help: {}", self.help_text)
    }
}

impl std::error::Error for DiagnosticError<'_> {}

/// Centralized error type for core operations.
#[derive(Debug)]
pub enum Error {
    /// Rendered compiler-style diagnostic error string.
    Diagnostic(String),
    /// General message error.
    Message(String),
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Self::Diagnostic(msg) => write!(f, "{msg}"),
            Self::Message(msg) => write!(f, "{msg}"),
        }
    }
}

impl From<&DiagnosticError<'_>> for Error {
    fn from(err: &DiagnosticError<'_>) -> Self {
        Self::Diagnostic(err.to_string())
    }
}

impl From<&str> for Error {
    fn from(msg: &str) -> Self {
        Self::Message(msg.to_string())
    }
}

impl From<CliError> for Error {
    fn from(err: CliError) -> Self {
        Self::Message(err.to_string())
    }
}

impl<'a> From<DiagnosticError<'a>> for Error {
    fn from(err: DiagnosticError<'a>) -> Self {
        Self::Diagnostic(err.to_string())
    }
}

impl From<String> for Error {
    fn from(msg: String) -> Self {
        Self::Message(msg)
    }
}

impl std::error::Error for Error {}

/// Result type alias for core operations.
pub type Result<T, E = Error> = std::result::Result<T, E>;

/// Constructs a diagnostic caret annotation string pointing to a source location.
#[must_use]
pub fn build_caret_annotation(col_no: usize, span_len: usize, max_len: usize) -> Cow<'static, str> {
    let max_len = max_len.max(1);
    let span = span_len.max(1);
    let padding_len = col_no.saturating_sub(1).min(max_len);

    if padding_len == 0 {
        let effective_span = span.min(max_len);
        if effective_span <= STATIC_CARETS.len() {
            Cow::Borrowed(&STATIC_CARETS[..effective_span])
        } else {
            Cow::Owned("^".repeat(effective_span))
        }
    } else {
        let effective_span = span.min(max_len.saturating_sub(padding_len).max(1));
        Cow::Owned(format!(
            "{}{}",
            " ".repeat(padding_len),
            "^".repeat(effective_span)
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_caret_annotation() {
        let carets = build_caret_annotation(1, 3, 10);
        assert_eq!(carets, "^^^");

        let padded = build_caret_annotation(4, 3, 10);
        assert_eq!(padded, "   ^^^");
    }

    #[test]
    fn test_caret_annotation_col_zero() {
        let carets_0 = build_caret_annotation(0, 5, 20);
        assert_eq!(carets_0, "^^^^^");
        assert!(matches!(carets_0, Cow::Borrowed(_)));

        let carets_1 = build_caret_annotation(1, 5, 20);
        assert_eq!(carets_1, "^^^^^");
        assert!(matches!(carets_1, Cow::Borrowed(_)));

        let carets_0_clamped = build_caret_annotation(0, 50, 10);
        assert_eq!(carets_0_clamped, "^^^^^^^^^^");
        assert_eq!(carets_0_clamped.len(), 10);

        let carets_0_zero_span = build_caret_annotation(0, 0, 0);
        assert_eq!(carets_0_zero_span, "^");
    }

    #[test]
    fn test_caret_annotation_extreme_col_no() {
        let max_len = 80;
        let carets = build_caret_annotation(usize::MAX, 5, max_len);
        assert_eq!(carets.len(), max_len + 1);
        assert_eq!(&carets[..max_len], " ".repeat(max_len));
        assert_eq!(&carets[max_len..], "^");
    }

    #[test]
    fn test_caret_annotation_long_snippet_exceeding_static_carets() {
        let long_len = 120;
        let carets = build_caret_annotation(0, long_len, 200);
        assert_eq!(carets.len(), long_len);
        assert!(carets.chars().all(|c| c == '^'));
        assert!(matches!(carets, Cow::Owned(_)));

        let padded_long = build_caret_annotation(50, 90, 200);
        assert_eq!(padded_long.len(), 49 + 90);
        assert_eq!(&padded_long[..49], " ".repeat(49));
        assert_eq!(&padded_long[49..], "^".repeat(90));

        let max_len_long = 300;
        let carets_huge = build_caret_annotation(1, 300, max_len_long);
        assert_eq!(carets_huge.len(), 300);
        assert!(carets_huge.chars().all(|c| c == '^'));

        let col_huge = build_caret_annotation(150, 60, 250);
        assert_eq!(col_huge.len(), 149 + 60);
        assert_eq!(&col_huge[..149], " ".repeat(149));
        assert_eq!(&col_huge[149..], "^".repeat(60));

        let col_over_max = build_caret_annotation(200, 50, 120);
        assert_eq!(col_over_max.len(), 120 + 1);
        assert_eq!(&col_over_max[..120], " ".repeat(120));
        assert_eq!(&col_over_max[120..], "^");
    }

    #[test]
    fn test_diagnostic_error_display_and_from_conversion() {
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

        let displayed = format!("{diag}");
        assert_eq!(displayed, diag.render());

        let err_owned: Error = diag.clone().into();
        assert_eq!(err_owned.to_string(), diag.render());

        let err_borrowed: Error = (&diag).into();
        assert_eq!(err_borrowed.to_string(), diag.render());
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
    fn test_cli_error_display_and_trait() {
        let err_missing =
            CliError::MissingArgument("Option '--output' requires a path value".into());
        assert_eq!(
            err_missing.to_string(),
            "Option '--output' requires a path value"
        );

        let err_invalid = CliError::InvalidArgument("Unrecognized option '--bad'".into());
        assert_eq!(err_invalid.to_string(), "Unrecognized option '--bad'");

        let err_pos =
            CliError::UnexpectedPositional("Unexpected positional argument 'extra'".into());
        assert_eq!(
            err_pos.to_string(),
            "Unexpected positional argument 'extra'"
        );

        let std_err: &dyn std::error::Error = &err_missing;
        assert!(std_err.source().is_none());
    }

    #[test]
    fn test_diagnostic_error_trait_impl() {
        let diag = DiagnosticError {
            message: "sample".into(),
            file_path: "test.md".into(),
            line_number: 1,
            col_number: 1,
            line_snippet: "snippet".into(),
            annotation_carets: "^".into(),
            annotation_text: "note".into(),
            help_text: "fix it".into(),
        };
        let std_err: &dyn std::error::Error = &diag;
        assert!(std_err.source().is_none());
    }

    #[test]
    fn test_error_display() {
        let err = Error::Message("test error".to_string());
        assert_eq!(err.to_string(), "test error");

        let diag_err = Error::Diagnostic("diagnostic error".to_string());
        assert_eq!(diag_err.to_string(), "diagnostic error");
    }

    #[test]
    fn test_error_from_cli_error() {
        let cli_err = CliError::MissingArgument("Option '--output' requires a path value".into());
        let err: Error = cli_err.into();
        assert_eq!(err.to_string(), "Option '--output' requires a path value");
    }

    #[test]
    fn test_error_from_str() {
        let err: Error = "test from str".into();
        assert_eq!(err.to_string(), "test from str");
    }

    #[test]
    fn test_error_from_string() {
        let err: Error = String::from("test from String").into();
        assert_eq!(err.to_string(), "test from String");
        let std_err: &dyn std::error::Error = &err;
        assert!(std_err.source().is_none());
    }
}
