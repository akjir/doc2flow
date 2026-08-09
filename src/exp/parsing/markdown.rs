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

/// Parses a checkbox list item into its nesting depth, checked status, and inner content.
fn parse_check_box_item(line: &str) -> Option<(usize, bool, &str)> {
    let leading_spaces = line.bytes().take_while(|&b| b == b' ').count();
    let rest = &line[leading_spaces..];

    let after_dash = rest.strip_prefix('-')?;
    let after_space = after_dash.strip_prefix(' ')?;
    let depth = (leading_spaces + 1) / 2;

    let (checked, after_box) = if let Some(after_box) = after_space.strip_prefix("[ ]") {
        (false, after_box)
    } else if let Some(after_box) = after_space
        .strip_prefix("[x]")
        .or_else(|| after_space.strip_prefix("[X]"))
    {
        (true, after_box)
    } else {
        return None;
    };

    if after_box.is_empty() {
        Some((depth, checked, ""))
    } else if let Some(content) = after_box.strip_prefix(' ') {
        Some((depth, checked, content.trim()))
    } else {
        None
    }
}

/// Parses a bullet list item into its nesting depth and inner content.
fn parse_bullet_list_item(line: &str) -> Option<(usize, &str)> {
    let leading_spaces = line.bytes().take_while(|&b| b == b' ').count();
    let rest = &line[leading_spaces..];

    let after_dash = rest.strip_prefix('-')?;
    if after_dash.is_empty() {
        let depth = (leading_spaces + 1) / 2;
        Some((depth, ""))
    } else if let Some(content) = after_dash.strip_prefix(' ') {
        let depth = (leading_spaces + 1) / 2;
        Some((depth, content.trim()))
    } else {
        None
    }
}

/// Parses an ordered list item candidate into its nesting depth, leading number, and inner content.
fn parse_ordered_list_candidate(line: &str) -> Option<(usize, u64, &str)> {
    let leading_spaces = line.bytes().take_while(|&b| b == b' ').count();
    let rest = &line[leading_spaces..];

    let digits_len = rest.bytes().take_while(|b| b.is_ascii_digit()).count();
    if digits_len == 0 {
        return None;
    }

    let after_digits = &rest[digits_len..];
    let after_dot = after_digits.strip_prefix('.')?;
    let parsed_num: u64 = rest[..digits_len].parse().ok()?;
    let depth = (leading_spaces + 1) / 2;

    if after_dot.is_empty() {
        Some((depth, parsed_num, ""))
    } else if let Some(content) = after_dot.strip_prefix(' ') {
        Some((depth, parsed_num, content.trim()))
    } else {
        None
    }
}

/// Attempts to process an ordered list item line against the active list depth stack.
fn try_process_ordered_list_item(
    line: &str,
    stack: &mut Vec<(usize, usize)>,
) -> Option<DocumentElement> {
    let (depth, parsed_num, content) = parse_ordered_list_candidate(line)?;

    if stack.is_empty() {
        if parsed_num == 1 {
            stack.push((depth, 1));
            Some(DocumentElement::ordered_list_item(depth, 1, content))
        } else {
            None
        }
    } else {
        let last_depth = stack.last().unwrap().0;
        if depth > last_depth {
            stack.push((depth, 1));
            Some(DocumentElement::ordered_list_item(depth, 1, content))
        } else if depth == last_depth {
            let entry = stack.last_mut().unwrap();
            entry.1 += 1;
            let position = entry.1;
            Some(DocumentElement::ordered_list_item(depth, position, content))
        } else {
            while let Some(top) = stack.last() {
                if top.0 > depth {
                    stack.pop();
                } else {
                    break;
                }
            }

            if let Some(top) = stack.last_mut() {
                if top.0 == depth {
                    top.1 += 1;
                    let position = top.1;
                    Some(DocumentElement::ordered_list_item(depth, position, content))
                } else if parsed_num == 1 {
                    stack.push((depth, 1));
                    Some(DocumentElement::ordered_list_item(depth, 1, content))
                } else {
                    stack.clear();
                    None
                }
            } else if parsed_num == 1 {
                stack.push((depth, 1));
                Some(DocumentElement::ordered_list_item(depth, 1, content))
            } else {
                None
            }
        }
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

/// Trims leading and trailing empty lines from a slice of code lines.
fn trim_code_block_lines(lines: &[&str]) -> String {
    let start = lines.iter().position(|line| !line.trim().is_empty());
    let end = lines.iter().rposition(|line| !line.trim().is_empty());

    match (start, end) {
        (Some(start_idx), Some(end_idx)) if start_idx <= end_idx => {
            let slice = &lines[start_idx..=end_idx];
            let total_len =
                slice.iter().map(|l| l.len()).sum::<usize>() + slice.len().saturating_sub(1);
            let mut result = String::with_capacity(total_len);
            for (i, line) in slice.iter().enumerate() {
                if i > 0 {
                    result.push('\n');
                }
                result.push_str(line);
            }
            result
        }
        _ => String::new(),
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
    let mut ordered_list_stack: Vec<(usize, usize)> = Vec::new();

    let mut in_code_block = false;
    let mut code_block_lang: Option<String> = None;
    let mut code_block_lines: Vec<&str> = Vec::new();

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

        // 2. Active code block collection phase (verbatim lines, no comment filtering)
        if in_code_block {
            let trimmed = line.trim_start();
            if trimmed.starts_with("```") {
                let content = trim_code_block_lines(&code_block_lines);
                doc.push_body(DocumentElement::code_block(code_block_lang.take(), content));
                code_block_lines.clear();
                in_code_block = false;
            } else {
                code_block_lines.push(line);
            }
            continue;
        }

        // 3. HTML comment filtering
        let Some(effective_line) = comment_filter.process_line(line, &mut line_buf) else {
            continue;
        };

        // 4. Frontmatter start detection or document body classification
        match phase {
            FrontmatterPhase::SeekingStart => {
                let trimmed = effective_line.trim();
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
                let trimmed_start = effective_line.trim_start();
                if trimmed_start.starts_with("```") {
                    ordered_list_stack.clear();
                    let info = trimmed_start.strip_prefix("```").unwrap_or("").trim();
                    code_block_lang = if info.is_empty() {
                        None
                    } else {
                        Some(info.to_string())
                    };
                    code_block_lines.clear();
                    in_code_block = true;
                    continue;
                }

                let trimmed = effective_line.trim();
                if !trimmed.is_empty() {
                    if let Some((depth, checked, content)) = parse_check_box_item(effective_line) {
                        ordered_list_stack.clear();
                        doc.push_body(DocumentElement::check_box_item(depth, checked, content));
                    } else if let Some((depth, content)) = parse_bullet_list_item(effective_line) {
                        ordered_list_stack.clear();
                        doc.push_body(DocumentElement::bullet_list_item(depth, content));
                    } else if let Some(elem) =
                        try_process_ordered_list_item(effective_line, &mut ordered_list_stack)
                    {
                        doc.push_body(elem);
                    } else if trimmed.starts_with('>') {
                        ordered_list_stack.clear();
                        let (shoutout_kind, content) = parse_shoutout_line(trimmed);
                        doc.push_body(DocumentElement::shoutout(shoutout_kind, content));
                    } else if is_plain_text(trimmed) {
                        ordered_list_stack.clear();
                        doc.push_body(DocumentElement::text(trimmed));
                    } else {
                        ordered_list_stack.clear();
                        doc.push_body(DocumentElement::unknown(trimmed));
                    }
                }
            }
        }
    }

    if in_code_block {
        let content = trim_code_block_lines(&code_block_lines);
        doc.push_body(DocumentElement::code_block(code_block_lang.take(), content));
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
    !line.starts_with(['#', '>', '-', '*', '`', '|'])
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
    fn test_is_plain_text() {
        assert!(is_plain_text("Regular sentence."));
        assert!(is_plain_text("2. Non-list plain text"));
        assert!(!is_plain_text("# Heading"));
        assert!(!is_plain_text("> Quote"));
        assert!(!is_plain_text("- Bullet"));
        assert!(!is_plain_text("* Italic / Bullet"));
        assert!(!is_plain_text("`Code`"));
        assert!(!is_plain_text("| Table |"));
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

    #[test]
    fn test_parse_bullet_list_item_depths() {
        let cases = [
            ("- Root item", Some((0, "Root item"))),
            ("-NoSpace", None),
            ("-- Not a bullet", None),
            ("---", None),
            ("-", Some((0, ""))),
            ("- ", Some((0, ""))),
            (" - Depth 1 (1 space)", Some((1, "Depth 1 (1 space)"))),
            ("  - Depth 1 (2 spaces)", Some((1, "Depth 1 (2 spaces)"))),
            ("   - Depth 2 (3 spaces)", Some((2, "Depth 2 (3 spaces)"))),
            ("    - Depth 2 (4 spaces)", Some((2, "Depth 2 (4 spaces)"))),
            ("     - Depth 3 (5 spaces)", Some((3, "Depth 3 (5 spaces)"))),
            ("      - Depth 3 (6 spaces)", Some((3, "Depth 3 (6 spaces)"))),
            ("       - Depth 4 (7 spaces)", Some((4, "Depth 4 (7 spaces)"))),
            ("        - Depth 4 (8 spaces)", Some((4, "Depth 4 (8 spaces)"))),
            ("  -   Spaced content   ", Some((1, "Spaced content"))),
            ("  -", Some((1, ""))),
        ];

        for (input, expected) in cases {
            assert_eq!(
                parse_bullet_list_item(input),
                expected,
                "Mismatch for input: {input:?}"
            );
        }
    }

    #[test]
    fn test_parse_d2f_markdown_bullet_lists() {
        let md = "---\ntitle: \"List Doc\"\n---\n- Level 0\n - Level 1a\n  - Level 1b\n   - Level 2a\n    - Level 2b\nRegular text\n- Another root";
        let doc = parse_d2f_markdown(md).unwrap();

        assert_eq!(doc.body.len(), 7);
        assert_eq!(doc.body[0], DocumentElement::bullet_list_item(0, "Level 0"));
        assert_eq!(doc.body[1], DocumentElement::bullet_list_item(1, "Level 1a"));
        assert_eq!(doc.body[2], DocumentElement::bullet_list_item(1, "Level 1b"));
        assert_eq!(doc.body[3], DocumentElement::bullet_list_item(2, "Level 2a"));
        assert_eq!(doc.body[4], DocumentElement::bullet_list_item(2, "Level 2b"));
        assert_eq!(doc.body[5], DocumentElement::text("Regular text"));
        assert_eq!(doc.body[6], DocumentElement::bullet_list_item(0, "Another root"));
    }

    #[test]
    fn test_parse_d2f_markdown_bullet_list_with_comments() {
        let md = "---\ntitle: \"Comments in List\"\n---\n  - Item 1 <!-- inline comment -->\n<!-- multiline\ncomment -->\n    - Item 2";
        let doc = parse_d2f_markdown(md).unwrap();

        assert_eq!(doc.body.len(), 2);
        assert_eq!(doc.body[0], DocumentElement::bullet_list_item(1, "Item 1"));
        assert_eq!(doc.body[1], DocumentElement::bullet_list_item(2, "Item 2"));
    }

    #[test]
    fn test_trim_code_block_lines() {
        let lines1 = ["", "", "Test", "", "Test 2", "", ""];
        assert_eq!(trim_code_block_lines(&lines1), "Test\n\nTest 2");

        let lines2 = ["   ", "\t", ""];
        assert_eq!(trim_code_block_lines(&lines2), "");

        let lines3 = ["", "  let x = 1;", "    let y = 2;", ""];
        assert_eq!(
            trim_code_block_lines(&lines3),
            "  let x = 1;\n    let y = 2;"
        );
    }

    #[test]
    fn test_parse_d2f_markdown_code_block_with_language() {
        let md = "---\ntitle: \"Code Doc\"\n---\n```bash\n# Init script\nd2f --init\n```";
        let doc = parse_d2f_markdown(md).unwrap();

        assert_eq!(doc.body.len(), 1);
        assert_eq!(
            doc.body[0],
            DocumentElement::code_block(Some("bash"), "# Init script\nd2f --init")
        );
    }

    #[test]
    fn test_parse_d2f_markdown_code_block_without_language() {
        let md = "---\ntitle: \"Code Doc\"\n---\n```\n\nTest\n\nTest 2\n\n\n```";
        let doc = parse_d2f_markdown(md).unwrap();

        assert_eq!(doc.body.len(), 1);
        assert_eq!(
            doc.body[0],
            DocumentElement::code_block(None::<String>, "Test\n\nTest 2")
        );
    }

    #[test]
    fn test_parse_d2f_markdown_code_block_preserves_comments_and_formatting() {
        let md = "---\ntitle: \"Code Doc\"\n---\n```html\n<!-- Inside code block -->\n<div>\n  <p>Hello</p>\n</div>\n```";
        let doc = parse_d2f_markdown(md).unwrap();

        assert_eq!(doc.body.len(), 1);
        assert_eq!(
            doc.body[0],
            DocumentElement::code_block(
                Some("html"),
                "<!-- Inside code block -->\n<div>\n  <p>Hello</p>\n</div>"
            )
        );
    }

    #[test]
    fn test_parse_d2f_markdown_unclosed_code_block_at_eof() {
        let md = "---\ntitle: \"Unclosed Code\"\n---\n```rust\nfn main() {\n    println!(\"hi\");\n}";
        let doc = parse_d2f_markdown(md).unwrap();

        assert_eq!(doc.body.len(), 1);
        assert_eq!(
            doc.body[0],
            DocumentElement::code_block(
                Some("rust"),
                "fn main() {\n    println!(\"hi\");\n}"
            )
        );
    }

    #[test]
    fn test_parse_check_box_item_depths_and_checked() {
        let cases = [
            ("- [ ] Root unchecked", Some((0, false, "Root unchecked"))),
            ("- [x] Root checked", Some((0, true, "Root checked"))),
            ("- [X] Root checked upper", Some((0, true, "Root checked upper"))),
            ("- [ ]", Some((0, false, ""))),
            ("- [ ] ", Some((0, false, ""))),
            ("- [x]", Some((0, true, ""))),
            ("- [x] ", Some((0, true, ""))),
            ("- [X]", Some((0, true, ""))),
            (" - [ ] Depth 1 (1 space)", Some((1, false, "Depth 1 (1 space)"))),
            ("  - [ ] Depth 1 (2 spaces)", Some((1, false, "Depth 1 (2 spaces)"))),
            ("   - [x] Depth 2 (3 spaces)", Some((2, true, "Depth 2 (3 spaces)"))),
            ("    - [x] Depth 2 (4 spaces)", Some((2, true, "Depth 2 (4 spaces)"))),
            ("     - [X] Depth 3 (5 spaces)", Some((3, true, "Depth 3 (5 spaces)"))),
            ("      - [X] Depth 3 (6 spaces)", Some((3, true, "Depth 3 (6 spaces)"))),
            ("       - [ ] Depth 4 (7 spaces)", Some((4, false, "Depth 4 (7 spaces)"))),
            ("        - [x] Depth 4 (8 spaces)", Some((4, true, "Depth 4 (8 spaces)"))),
            ("  - [ ]   Spaced task content   ", Some((1, false, "Spaced task content"))),
            ("- [ ]NoSpace", None),
            ("- [x]NoSpace", None),
            ("- [y] Invalid char", None),
            ("- [] Empty brackets", None),
            ("- Plain bullet", None),
            ("-- [ ] Invalid dash", None),
        ];

        for (input, expected) in cases {
            assert_eq!(
                parse_check_box_item(input),
                expected,
                "Mismatch for input: {input:?}"
            );
        }
    }

    #[test]
    fn test_parse_d2f_markdown_check_box_items() {
        let md = "---\ntitle: \"Checkboxes\"\n---\n- [ ] Task 1\n - [x] Subtask 1.1\n  - [X] Subtask 1.2\n   - [ ] Sub-subtask\n- [x] Task 2\n- [ ]";
        let doc = parse_d2f_markdown(md).unwrap();

        assert_eq!(doc.body.len(), 6);
        assert_eq!(doc.body[0], DocumentElement::check_box_item(0, false, "Task 1"));
        assert_eq!(doc.body[1], DocumentElement::check_box_item(1, true, "Subtask 1.1"));
        assert_eq!(doc.body[2], DocumentElement::check_box_item(1, true, "Subtask 1.2"));
        assert_eq!(doc.body[3], DocumentElement::check_box_item(2, false, "Sub-subtask"));
        assert_eq!(doc.body[4], DocumentElement::check_box_item(0, true, "Task 2"));
        assert_eq!(doc.body[5], DocumentElement::check_box_item(0, false, ""));
    }

    #[test]
    fn test_parse_d2f_markdown_mixed_lists() {
        let md = "---\ntitle: \"Mixed\"\n---\n- Bullet 1\n- [ ] Task 1\n  - Bullet nested\n  - [x] Task nested\n- Bullet 2";
        let doc = parse_d2f_markdown(md).unwrap();

        assert_eq!(doc.body.len(), 5);
        assert_eq!(doc.body[0], DocumentElement::bullet_list_item(0, "Bullet 1"));
        assert_eq!(doc.body[1], DocumentElement::check_box_item(0, false, "Task 1"));
        assert_eq!(doc.body[2], DocumentElement::bullet_list_item(1, "Bullet nested"));
        assert_eq!(doc.body[3], DocumentElement::check_box_item(1, true, "Task nested"));
        assert_eq!(doc.body[4], DocumentElement::bullet_list_item(0, "Bullet 2"));
    }

    #[test]
    fn test_parse_ordered_list_candidate_depths_and_numbers() {
        let cases = [
            ("1. Root item", Some((0, 1, "Root item"))),
            ("1.", Some((0, 1, ""))),
            ("1. ", Some((0, 1, ""))),
            (" 1. Depth 1 (1 space)", Some((1, 1, "Depth 1 (1 space)"))),
            ("  1. Depth 1 (2 spaces)", Some((1, 1, "Depth 1 (2 spaces)"))),
            ("   2. Depth 2 (3 spaces)", Some((2, 2, "Depth 2 (3 spaces)"))),
            ("    42. Depth 2 (4 spaces)", Some((2, 42, "Depth 2 (4 spaces)"))),
            ("     999. Depth 3 (5 spaces)", Some((3, 999, "Depth 3 (5 spaces)"))),
            ("      1. Depth 3 (6 spaces)", Some((3, 1, "Depth 3 (6 spaces)"))),
            ("       5. Depth 4 (7 spaces)", Some((4, 5, "Depth 4 (7 spaces)"))),
            ("        10. Depth 4 (8 spaces)", Some((4, 10, "Depth 4 (8 spaces)"))),
            ("  1.   Spaced content   ", Some((1, 1, "Spaced content"))),
            ("1.NoSpace", None),
            ("1.1 Decimal", None),
            ("- Dash", None),
            ("Text 1.", None),
            ("", None),
        ];

        for (input, expected) in cases {
            assert_eq!(
                parse_ordered_list_candidate(input),
                expected,
                "Mismatch for candidate input: {input:?}"
            );
        }
    }

    #[test]
    fn test_parse_d2f_markdown_ordered_lists_basic() {
        let md = "---\ntitle: \"Ordered Lists\"\n---\n1. First\n2. Second\n3. Third";
        let doc = parse_d2f_markdown(md).unwrap();

        assert_eq!(doc.body.len(), 3);
        assert_eq!(doc.body[0], DocumentElement::ordered_list_item(0, 1, "First"));
        assert_eq!(doc.body[1], DocumentElement::ordered_list_item(0, 2, "Second"));
        assert_eq!(doc.body[2], DocumentElement::ordered_list_item(0, 3, "Third"));
    }

    #[test]
    fn test_parse_d2f_markdown_ordered_lists_arbitrary_numbers() {
        let md1 = "---\ntitle: \"Repeated 1s\"\n---\n1. Apple\n1. Banana\n1. Cherry";
        let doc1 = parse_d2f_markdown(md1).unwrap();
        assert_eq!(doc1.body.len(), 3);
        assert_eq!(doc1.body[0], DocumentElement::ordered_list_item(0, 1, "Apple"));
        assert_eq!(doc1.body[1], DocumentElement::ordered_list_item(0, 2, "Banana"));
        assert_eq!(doc1.body[2], DocumentElement::ordered_list_item(0, 3, "Cherry"));

        let md2 = "---\ntitle: \"Random Numbers\"\n---\n1. Red\n5. Green\n99. Blue";
        let doc2 = parse_d2f_markdown(md2).unwrap();
        assert_eq!(doc2.body.len(), 3);
        assert_eq!(doc2.body[0], DocumentElement::ordered_list_item(0, 1, "Red"));
        assert_eq!(doc2.body[1], DocumentElement::ordered_list_item(0, 2, "Green"));
        assert_eq!(doc2.body[2], DocumentElement::ordered_list_item(0, 3, "Blue"));
    }

    #[test]
    fn test_parse_d2f_markdown_ordered_lists_nested_and_returns() {
        let md = "---\ntitle: \"Nested\"\n---\n1. L0_1\n  1. L1_1\n  2. L1_2\n    1. L2_1\n  3. L1_3\n2. L0_2";
        let doc = parse_d2f_markdown(md).unwrap();

        assert_eq!(doc.body.len(), 6);
        assert_eq!(doc.body[0], DocumentElement::ordered_list_item(0, 1, "L0_1"));
        assert_eq!(doc.body[1], DocumentElement::ordered_list_item(1, 1, "L1_1"));
        assert_eq!(doc.body[2], DocumentElement::ordered_list_item(1, 2, "L1_2"));
        assert_eq!(doc.body[3], DocumentElement::ordered_list_item(2, 1, "L2_1"));
        assert_eq!(doc.body[4], DocumentElement::ordered_list_item(1, 3, "L1_3"));
        assert_eq!(doc.body[5], DocumentElement::ordered_list_item(0, 2, "L0_2"));
    }

    #[test]
    fn test_parse_d2f_markdown_ordered_lists_new_depth_arbitrary_number() {
        let md = "---\ntitle: \"New Depth Start\"\n---\n1. Root 1\n  7. Subitem 1\n  8. Subitem 2\n2. Root 2";
        let doc = parse_d2f_markdown(md).unwrap();

        assert_eq!(doc.body.len(), 4);
        assert_eq!(doc.body[0], DocumentElement::ordered_list_item(0, 1, "Root 1"));
        assert_eq!(doc.body[1], DocumentElement::ordered_list_item(1, 1, "Subitem 1"));
        assert_eq!(doc.body[2], DocumentElement::ordered_list_item(1, 2, "Subitem 2"));
        assert_eq!(doc.body[3], DocumentElement::ordered_list_item(0, 2, "Root 2"));
    }

    #[test]
    fn test_parse_d2f_markdown_ordered_lists_interrupted_by_text() {
        let md = "---\ntitle: \"Interrupted\"\n---\n1. Item 1\n2. Item 2\n\nParagraph text\n\n1. New list 1\n2. New list 2\n\nAnother paragraph\n\n2. Not a list";
        let doc = parse_d2f_markdown(md).unwrap();

        assert_eq!(doc.body.len(), 7);
        assert_eq!(doc.body[0], DocumentElement::ordered_list_item(0, 1, "Item 1"));
        assert_eq!(doc.body[1], DocumentElement::ordered_list_item(0, 2, "Item 2"));
        assert_eq!(doc.body[2], DocumentElement::text("Paragraph text"));
        assert_eq!(doc.body[3], DocumentElement::ordered_list_item(0, 1, "New list 1"));
        assert_eq!(doc.body[4], DocumentElement::ordered_list_item(0, 2, "New list 2"));
        assert_eq!(doc.body[5], DocumentElement::text("Another paragraph"));
        assert_eq!(doc.body[6], DocumentElement::text("2. Not a list"));
    }

    #[test]
    fn test_parse_d2f_markdown_ordered_lists_interrupted_by_other_elements() {
        let md = "---\ntitle: \"Interrupted by Elements\"\n---\n1. Item 1\n- Bullet item\n1. Item after bullet\n- [ ] Check item\n1. Item after check\n> Note shoutout\n1. Item after shoutout\n```\ncode\n```\n1. Item after code";
        let doc = parse_d2f_markdown(md).unwrap();

        assert_eq!(doc.body.len(), 9);
        assert_eq!(doc.body[0], DocumentElement::ordered_list_item(0, 1, "Item 1"));
        assert_eq!(doc.body[1], DocumentElement::bullet_list_item(0, "Bullet item"));
        assert_eq!(doc.body[2], DocumentElement::ordered_list_item(0, 1, "Item after bullet"));
        assert_eq!(doc.body[3], DocumentElement::check_box_item(0, false, "Check item"));
        assert_eq!(doc.body[4], DocumentElement::ordered_list_item(0, 1, "Item after check"));
        assert_eq!(
            doc.body[5],
            DocumentElement::shoutout(ShoutoutElementKind::Note, "Note shoutout")
        );
        assert_eq!(doc.body[6], DocumentElement::ordered_list_item(0, 1, "Item after shoutout"));
        assert_eq!(
            doc.body[7],
            DocumentElement::code_block(None::<String>, "code")
        );
        assert_eq!(doc.body[8], DocumentElement::ordered_list_item(0, 1, "Item after code"));
    }

    #[test]
    fn test_parse_d2f_markdown_ordered_lists_with_comments() {
        let md = "---\ntitle: \"Comments in Ordered List\"\n---\n1. Item 1 <!-- inline -->\n<!-- multiline\ncomment -->\n2. Item 2\n  1. Subitem <!-- comment -->";
        let doc = parse_d2f_markdown(md).unwrap();

        assert_eq!(doc.body.len(), 3);
        assert_eq!(doc.body[0], DocumentElement::ordered_list_item(0, 1, "Item 1"));
        assert_eq!(doc.body[1], DocumentElement::ordered_list_item(0, 2, "Item 2"));
        assert_eq!(doc.body[2], DocumentElement::ordered_list_item(1, 1, "Subitem"));
    }
}
