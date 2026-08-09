//! Markdown parser without external dependencies.

use crate::exp::document::{Document, DocumentElement, ShoutoutElementKind};
use crate::exp::error::{build_caret_annotation, DiagnosticError};
use crate::exp::{Error, Result};

/// State tracker for filtering HTML comments across lines.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
struct CommentFilterState {
    /// Indicates whether parsing is currently within a multi-line HTML comment block.
    in_comment: bool,
}

impl CommentFilterState {
    /// Creates a new comment filter in standard mode.
    fn new() -> Self {
        Self { in_comment: false }
    }

    /// Filters HTML comments from a single line.
    ///
    /// Returns `Some(processed_line)` or `None` if the line was fully swallowed by comments.
    fn process_line<'a>(&mut self, mut current: &'a str, buf: &'a mut String) -> Option<&'a str> {
        if self.in_comment {
            match current.find("-->") {
                Some(end_idx) => {
                    self.in_comment = false;
                    current = &current[end_idx + 3..];
                }
                None => return None,
            }
        }

        if !current.contains("<!--") {
            return Some(current);
        }

        buf.clear();
        while let Some(start_idx) = current.find("<!--") {
            buf.push_str(&current[..start_idx]);
            let after_start = &current[start_idx + 4..];
            match after_start.find("-->") {
                Some(end_idx) => {
                    current = &after_start[end_idx + 3..];
                }
                None => {
                    self.in_comment = true;
                    current = "";
                    break;
                }
            }
        }
        buf.push_str(current);
        Some(buf.as_str())
    }
}

/// Frontmatter parsing lifecycle state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum FrontmatterPhase {
    /// Seeking opening frontmatter delimiter `---`.
    SeekingStart,
    /// Inside frontmatter block, collecting content until closing `---`.
    Inside,
    /// Frontmatter parsed, processing standard document body.
    Complete,
}

/// Trims surrounding quotes only if enclosed by identical matching single or double quotes.
fn trim_matching_quotes(input: &str) -> &str {
    let s = input.trim();
    if (s.starts_with('"') && s.ends_with('"') && s.len() >= 2)
        || (s.starts_with('\'') && s.ends_with('\'') && s.len() >= 2)
    {
        &s[1..s.len() - 1]
    } else {
        s
    }
}

/// Parses a shoutout line starting with `>` into its element kind and inner content.
fn parse_shoutout_line(line: &str) -> (ShoutoutElementKind, &str) {
    debug_assert!(line.starts_with('>'));
    let inner = line[1..].trim_start();

    if let Some(rest) = inner.strip_prefix("!!!") {
        (
            ShoutoutElementKind::Caution,
            rest.strip_prefix(' ').unwrap_or(rest),
        )
    } else if let Some(rest) = inner.strip_prefix("!!") {
        (
            ShoutoutElementKind::Warning,
            rest.strip_prefix(' ').unwrap_or(rest),
        )
    } else if let Some(rest) = inner.strip_prefix('!') {
        (
            ShoutoutElementKind::Important,
            rest.strip_prefix(' ').unwrap_or(rest),
        )
    } else if let Some(rest) = inner.strip_prefix('?') {
        (
            ShoutoutElementKind::Tip,
            rest.strip_prefix(' ').unwrap_or(rest),
        )
    } else {
        (ShoutoutElementKind::Note, inner)
    }
}

/// Parses Markdown content into a structured document model.
///
/// # Errors
///
/// Returns a diagnostic error if frontmatter is missing, unclosed, or improperly placed.
pub fn parse_d2f_markdown(md_content: &str) -> Result<Document, Error> {
    let mut doc = Document::new();
    let mut phase = FrontmatterPhase::SeekingStart;
    let mut comment_filter = CommentFilterState::new();

    let mut fm_start_line = 0;
    let mut fm_start_snippet = "";

    let mut line_buf = String::new();
    let mut last_line_info = (1, "");

    for (idx, line) in md_content.lines().enumerate() {
        let line_no = idx + 1;
        last_line_info = (line_no, line);

        // 1. Frontmatter collection phase
        if phase == FrontmatterPhase::Inside {
            if line.trim() == "---" {
                phase = FrontmatterPhase::Complete;
            } else if let Some((key, val)) = line.split_once(':') {
                let key = key.trim();
                let val_trimmed = trim_matching_quotes(val);
                if !key.is_empty() {
                    doc.insert_parameter(key, val_trimmed);
                }
            }
            continue;
        }

        // 2. HTML comment filtering
        let Some(effective_line) = comment_filter.process_line(line, &mut line_buf) else {
            continue;
        };

        let trimmed = effective_line.trim();

        // 3. Frontmatter start detection or document body classification
        match phase {
            FrontmatterPhase::SeekingStart => {
                if trimmed.is_empty() {
                    continue;
                }
                if trimmed == "---" {
                    phase = FrontmatterPhase::Inside;
                    fm_start_line = line_no;
                    fm_start_snippet = line;
                    continue;
                }
                return Err(build_missing_frontmatter_err(line_no, line));
            }
            FrontmatterPhase::Inside => unreachable!(),
            FrontmatterPhase::Complete => {
                if !trimmed.is_empty() {
                    if trimmed.starts_with('>') {
                        let (shoutout_kind, content) = parse_shoutout_line(trimmed);
                        doc.push_body(DocumentElement::shoutout(shoutout_kind, content));
                    } else if is_plain_text(trimmed) {
                        doc.push_body(DocumentElement::text(trimmed));
                    } else {
                        doc.push_body(DocumentElement::unknown(trimmed));
                    }
                }
            }
        }
    }

    // Final state validation
    match phase {
        FrontmatterPhase::SeekingStart => {
            let (line_no, snippet) = last_line_info;
            Err(build_missing_frontmatter_err(line_no, snippet))
        }
        FrontmatterPhase::Inside => Err(DiagnosticError {
            message: "unclosed frontmatter block".into(),
            file_path: "<input>".into(),
            line_number: fm_start_line,
            col_number: 1,
            line_snippet: fm_start_snippet.into(),
            annotation_carets: build_caret_annotation(
                1,
                fm_start_snippet.len().max(3),
                fm_start_snippet.len().max(3),
            ),
            annotation_text: "frontmatter starting here is never closed".into(),
            help_text: "close the frontmatter block with a closing '---' line.".into(),
        }
        .into()),
        FrontmatterPhase::Complete => Ok(doc),
    }
}

/// Constructs a standardized diagnostic error for missing frontmatter delimiters.
fn build_missing_frontmatter_err(line_number: usize, line_snippet: &str) -> Error {
    let snippet = if line_snippet.is_empty() { "" } else { line_snippet };
    let carets = build_caret_annotation(1, snippet.len().max(1), snippet.len().max(1));
    DiagnosticError {
        message: "missing frontmatter delimiter '---'".into(),
        file_path: "<input>".into(),
        line_number,
        col_number: 1,
        line_snippet: snippet.into(),
        annotation_carets: carets,
        annotation_text: "expected '---' to begin frontmatter".into(),
        help_text: "document must begin with frontmatter enclosed by '---' delimiters.".into(),
    }
    .into()
}

/// Checks whether a line represents plain text.
fn is_plain_text(line: &str) -> bool {
    if line.starts_with(['#', '>', '-', '*', '`', '|']) {
        return false;
    }
    // Prefix check for numbered lists ("0." through "9.")
    if line.len() >= 2 && line.as_bytes()[0].is_ascii_digit() && line.as_bytes()[1] == b'.' {
        return false;
    }
    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_d2f_markdown_valid_frontmatter_and_elements() {
        let md = "---\ntitle: \"Test Title\"\nversion: \"1.0.0\"\n---\n# Heading 1\n\nThis is plain text paragraph.\n\n> Callout note\nAnother text.";
        let doc = parse_d2f_markdown(md).unwrap();

        assert_eq!(
            doc.parameters.get("title").map(|s| s.as_str()),
            Some("Test Title")
        );
        assert_eq!(
            doc.parameters.get("version").map(|s| s.as_str()),
            Some("1.0.0")
        );
        assert_eq!(doc.body.len(), 4);

        assert_eq!(doc.body[0], DocumentElement::Unknown("# Heading 1".into()));
        assert_eq!(
            doc.body[1],
            DocumentElement::Text("This is plain text paragraph.".into())
        );
        assert_eq!(
            doc.body[2],
            DocumentElement::Shoutout {
                kind: ShoutoutElementKind::Note,
                content: "Callout note".into(),
            }
        );
        assert_eq!(doc.body[3], DocumentElement::Text("Another text.".into()));
    }

    #[test]
    fn test_parse_d2f_markdown_comments_before_frontmatter() {
        let md = "<!-- Comment at top -->\n\n<!--\nMultiline comment\n-->\n<!-- inline -->\n---\ntitle: \"Doc\"\n---\n# Heading 1\nVisible text";
        let doc = parse_d2f_markdown(md).unwrap();

        assert_eq!(doc.parameters.get("title").map(|s| s.as_str()), Some("Doc"));
        assert_eq!(doc.body.len(), 2);
        assert_eq!(doc.body[0], DocumentElement::Unknown("# Heading 1".into()));
        assert_eq!(doc.body[1], DocumentElement::Text("Visible text".into()));
    }

    #[test]
    fn test_parse_d2f_markdown_empty_frontmatter() {
        let md = "---\n---\nBody text";
        let doc = parse_d2f_markdown(md).unwrap();

        assert!(doc.parameters.is_empty());
        assert_eq!(doc.body.len(), 1);
        assert_eq!(doc.body[0], DocumentElement::Text("Body text".into()));
    }

    #[test]
    fn test_parse_d2f_markdown_horizontal_rule_in_body() {
        let md = "---\ntitle: \"Test\"\n---\nLine 1\n---\nLine 2";
        let doc = parse_d2f_markdown(md).unwrap();

        assert_eq!(doc.parameters.get("title").map(|s| s.as_str()), Some("Test"));
        assert_eq!(doc.body.len(), 3);
        assert_eq!(doc.body[0], DocumentElement::Text("Line 1".into()));
        assert_eq!(doc.body[1], DocumentElement::Unknown("---".into()));
        assert_eq!(doc.body[2], DocumentElement::Text("Line 2".into()));
    }

    #[test]
    fn test_parse_d2f_markdown_error_missing_frontmatter() {
        let md = "# Heading 1\nSome text";
        let err = parse_d2f_markdown(md).unwrap_err();
        let err_str = err.to_string();

        assert!(err_str.contains("error: missing frontmatter delimiter '---'"));
        assert!(err_str.contains("1 | # Heading 1"));
        assert!(err_str.contains("expected '---' to begin frontmatter"));
    }

    #[test]
    fn test_parse_d2f_markdown_error_text_before_frontmatter() {
        let md = "Some text before\n---\ntitle: \"Doc\"\n---";
        let err = parse_d2f_markdown(md).unwrap_err();
        let err_str = err.to_string();

        assert!(err_str.contains("error: missing frontmatter delimiter '---'"));
        assert!(err_str.contains("1 | Some text before"));
    }

    #[test]
    fn test_parse_d2f_markdown_error_unclosed_frontmatter() {
        let md = "---\ntitle: \"Unclosed Doc\"\nversion: \"1.0.0\"";
        let err = parse_d2f_markdown(md).unwrap_err();
        let err_str = err.to_string();

        assert!(err_str.contains("error: unclosed frontmatter block"));
        assert!(err_str.contains("1 | ---"));
        assert!(err_str.contains("frontmatter starting here is never closed"));
    }

    #[test]
    fn test_parse_d2f_markdown_error_empty_input() {
        let err = parse_d2f_markdown("").unwrap_err();
        let err_str = err.to_string();

        assert!(err_str.contains("error: missing frontmatter delimiter '---'"));
    }

    #[test]
    fn test_parse_d2f_markdown_error_only_comments() {
        let md = "<!-- comment 1 -->\n<!--\nmultiline comment\n-->\n<!-- comment 3 -->";
        let err = parse_d2f_markdown(md).unwrap_err();
        let err_str = err.to_string();

        assert!(err_str.contains("error: missing frontmatter delimiter '---'"));
    }

    #[test]
    fn test_parse_d2f_markdown_crlf_endings() {
        let md = "---\r\ntitle: \"CRLF\"\r\n---\r\n# Heading\r\nText";
        let doc = parse_d2f_markdown(md).unwrap();

        assert_eq!(doc.parameters.get("title").map(|s| s.as_str()), Some("CRLF"));
        assert_eq!(doc.body.len(), 2);
        assert_eq!(doc.body[0], DocumentElement::Unknown("# Heading".into()));
        assert_eq!(doc.body[1], DocumentElement::Text("Text".into()));
    }

    #[test]
    fn test_is_plain_text_numbered_lists() {
        assert!(!is_plain_text("1. Item"));
        assert!(!is_plain_text("9. Item"));
        assert!(is_plain_text("10. Item"));
        assert!(is_plain_text("Regular sentence."));
    }

    #[test]
    fn test_parse_shoutout_line_prefixes() {
        let cases = [
            ("> Note content", ShoutoutElementKind::Note, "Note content"),
            (">Note without space", ShoutoutElementKind::Note, "Note without space"),
            (">   Spaced note", ShoutoutElementKind::Note, "Spaced note"),
            (">?", ShoutoutElementKind::Tip, ""),
            (">? Tip content", ShoutoutElementKind::Tip, "Tip content"),
            (">?Tip without space", ShoutoutElementKind::Tip, "Tip without space"),
            ("> ? Spaced tip", ShoutoutElementKind::Tip, "Spaced tip"),
            (">! Important note", ShoutoutElementKind::Important, "Important note"),
            (">!Important", ShoutoutElementKind::Important, "Important"),
            ("> ! Spaced important", ShoutoutElementKind::Important, "Spaced important"),
            (">!! Warning note", ShoutoutElementKind::Warning, "Warning note"),
            (">!!Warning", ShoutoutElementKind::Warning, "Warning"),
            ("> !! Spaced warning", ShoutoutElementKind::Warning, "Spaced warning"),
            (">!!! Caution alert", ShoutoutElementKind::Caution, "Caution alert"),
            (">!!!Caution", ShoutoutElementKind::Caution, "Caution"),
            ("> !!! Spaced caution", ShoutoutElementKind::Caution, "Spaced caution"),
        ];

        for (input, expected_kind, expected_content) in cases {
            let (kind, content) = parse_shoutout_line(input);
            assert_eq!(kind, expected_kind, "Mismatch for input: {input}");
            assert_eq!(content, expected_content, "Mismatch for input: {input}");
        }
    }

    #[test]
    fn test_parse_d2f_markdown_shoutout_kinds() {
        let md = "---\ntitle: \"Shoutouts\"\n---\n> Note\n>? Tip\n>! Important\n>!! Warning\n>!!! Caution";
        let doc = parse_d2f_markdown(md).unwrap();

        assert_eq!(doc.body.len(), 5);

        let expected = [
            (ShoutoutElementKind::Note, "Note"),
            (ShoutoutElementKind::Tip, "Tip"),
            (ShoutoutElementKind::Important, "Important"),
            (ShoutoutElementKind::Warning, "Warning"),
            (ShoutoutElementKind::Caution, "Caution"),
        ];

        for (i, (kind, content)) in expected.into_iter().enumerate() {
            assert_eq!(
                doc.body[i],
                DocumentElement::Shoutout {
                    kind,
                    content: content.into(),
                }
            );
        }
    }
}
