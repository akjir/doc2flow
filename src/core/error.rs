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

impl<'a> From<DiagnosticError<'a>> for Doc2FlowError {
    fn from(err: DiagnosticError<'a>) -> Self {
        Doc2FlowError::Diagnostic(err.render())
    }
}

impl From<&DiagnosticError<'_>> for Doc2FlowError {
    fn from(err: &DiagnosticError<'_>) -> Self {
        Doc2FlowError::Diagnostic(err.render())
    }
}

/// Extension trait for attaching path context to I/O results.
///
/// # Examples
///
/// ```no_run
/// use std::fs::File;
/// use doc2flow::core::error::IoResultExt;
///
/// let res = File::open("missing_file.txt").with_path("missing_file.txt");
/// assert!(res.is_err());
/// ```
pub trait IoResultExt<T> {
    /// Attaches a filesystem path to the error variant if the result is an error.
    fn with_path(self, path: impl Into<PathBuf>) -> Result<T>;
}

impl<T> IoResultExt<T> for std::result::Result<T, std::io::Error> {
    fn with_path(self, path: impl Into<PathBuf>) -> Result<T> {
        self.map_err(|source| Doc2FlowError::Io {
            path: Some(path.into()),
            source,
        })
    }
}

/// Static buffer of carets for zero-allocation caret borrowing on typical line lengths.
const STATIC_CARETS: &str =
    "^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^";

/// Constructs a diagnostic caret annotation string pointing to a source location.
///
/// # Examples
///
/// ```
/// use doc2flow::core::error::build_caret_annotation;
///
/// let carets = build_caret_annotation(1, 3, 10);
/// assert_eq!(carets, "^^^");
///
/// let padded = build_caret_annotation(4, 3, 10);
/// assert_eq!(padded, "   ^^^");
/// ```
pub fn build_caret_annotation(
    col_no: usize,
    span_len: usize,
    max_len: usize,
) -> Cow<'static, str> {
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

        let _ = out.write_str("error: ");
        let _ = out.write_str(&self.message);
        let _ = out.write_str("\n --> ");
        let _ = writeln!(
            out,
            "{}:{}:{}",
            self.file_path, self.line_number, self.col_number
        );
        let _ = writeln!(out, "  {:>width$}|", "", width = line_len);
        let _ = writeln!(out, "{} | {}", self.line_number, self.line_snippet);
        let _ = writeln!(
            out,
            "  {:>width$}| {} {}",
            "",
            self.annotation_carets,
            self.annotation_text,
            width = line_len
        );
        let _ = writeln!(out, "  {:>width$}|", "", width = line_len);
        let _ = out.write_str("= help: ");
        let _ = out.write_str(&self.help_text);

        out
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
        let carets = build_caret_annotation(col_no, src_val.len(), line_len);

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
        .into()
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
        assert_eq!(col_over_max.len(), 199 + 1);
        assert_eq!(&col_over_max[..199], " ".repeat(199));
        assert_eq!(&col_over_max[199..], "^");
    }

    #[test]
    fn test_image_too_large_long_snippet_does_not_panic() {
        let src_val = "a".repeat(150);
        let long_snippet = format!("![Photo]({src_val})");
        let err = DiagnosticError::image_too_large(
            "doc.md",
            100,
            130,
            &long_snippet,
            &src_val,
            500 * 1024,
        );
        let err_str = err.to_string();
        assert!(err_str.contains("--> doc.md:100:130"));
        assert!(err_str.contains("100 | "));
    }

    #[test]
    fn test_doc2flow_error_source() {
        use std::error::Error;

        let io_err = std::io::Error::new(std::io::ErrorKind::NotFound, "file not found");
        let d2f_io = Doc2FlowError::Io {
            path: Some(PathBuf::from("missing.txt")),
            source: io_err,
        };
        assert!(d2f_io.source().is_some());
        assert_eq!(
            d2f_io.source().unwrap().to_string(),
            "file not found"
        );

        let diag_err = Doc2FlowError::Diagnostic("diag error".to_string());
        assert!(diag_err.source().is_none());

        let missing_id = Doc2FlowError::MissingIdentityFields;
        assert!(missing_id.source().is_none());

        let img_not_found = Doc2FlowError::ImageNotFound(PathBuf::from("img.png"));
        assert!(img_not_found.source().is_none());

        let img_proc = Doc2FlowError::ImageProcess("process error".to_string());
        assert!(img_proc.source().is_none());

        let json_err = Doc2FlowError::Json("json error".to_string());
        assert!(json_err.source().is_none());

        let msg_err = Doc2FlowError::Message("msg error".to_string());
        assert!(msg_err.source().is_none());
    }

    #[test]
    fn test_diagnostic_error_display_and_from_conversion() {
        use std::error::Error;

        let diag = DiagnosticError {
            message: "invalid frontmatter".into(),
            file_path: "input.md".into(),
            line_number: 3,
            col_number: 2,
            line_snippet: "title:".into(),
            annotation_carets: " ^^^^^".into(),
            annotation_text: "expected value".into(),
            help_text: "provide title value".into(),
        };

        let displayed = format!("{}", diag);
        assert_eq!(displayed, diag.render());
        assert!(diag.source().is_none());

        let d2f_owned: Doc2FlowError = diag.clone().into();
        assert_eq!(format!("{}", d2f_owned), diag.render());

        let d2f_borrowed: Doc2FlowError = (&diag).into();
        assert_eq!(format!("{}", d2f_borrowed), diag.render());
    }

    #[test]
    fn test_io_result_ext_with_path() {
        let io_err = std::io::Error::new(std::io::ErrorKind::PermissionDenied, "access denied");
        let io_res: std::result::Result<(), std::io::Error> = Err(io_err);

        let d2f_res = io_res.with_path("config.json");
        match d2f_res {
            Err(Doc2FlowError::Io { path, source }) => {
                assert_eq!(path, Some(PathBuf::from("config.json")));
                assert_eq!(source.kind(), std::io::ErrorKind::PermissionDenied);
            }
            _ => panic!("Expected Doc2FlowError::Io error variant"),
        }

        let ok_res: std::result::Result<u32, std::io::Error> = Ok(42);
        assert_eq!(ok_res.with_path("config.json").unwrap(), 42);
    }
}
