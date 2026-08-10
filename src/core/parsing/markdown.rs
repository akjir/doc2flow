//! Markdown parser without external dependencies.

use crate::core::document::{Document, DocumentElement, ShoutoutElementKind, TableAlignment};
use crate::core::error::{build_caret_annotation, DiagnosticError};
use crate::core::{Error, Result};



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
                    if current.trim().is_empty() {
                        return None;
                    }
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
        if buf.trim().is_empty() {
            None
        } else {
            Some(buf.as_str())
        }
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

/// Constructs a standardized diagnostic error for invalid block directive names.
fn build_invalid_block_directive_name_err(line_number: usize, line_snippet: &str) -> Error {
    let snippet = if line_snippet.is_empty() { "" } else { line_snippet };
    let carets = build_caret_annotation(1, snippet.len().max(1), snippet.len().max(1));
    DiagnosticError {
        message: "invalid block directive name".into(),
        file_path: "<input>".into(),
        line_number,
        col_number: 1,
        line_snippet: snippet.into(),
        annotation_carets: carets,
        annotation_text: "directive name must contain only alphanumeric characters (a-z, 0-9)".into(),
        help_text: "use only alphanumeric characters for directive names, e.g. ':::variables'.".into(),
    }
    .into()
}

/// Constructs a standardized diagnostic error for missing block directive names.
fn build_missing_block_directive_name_err(line_number: usize, line_snippet: &str) -> Error {
    let snippet = if line_snippet.is_empty() { "" } else { line_snippet };
    let carets = build_caret_annotation(1, snippet.len().max(1), snippet.len().max(1));
    DiagnosticError {
        message: "missing block directive name".into(),
        file_path: "<input>".into(),
        line_number,
        col_number: 1,
        line_snippet: snippet.into(),
        annotation_carets: carets,
        annotation_text: "expected directive name after colons".into(),
        help_text: "provide an alphanumeric name for the block directive, e.g. ':::variables'.".into(),
    }
    .into()
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

/// Constructs a standardized diagnostic error for nested block directives.
fn build_nested_block_directive_err(line_number: usize, line_snippet: &str) -> Error {
    let snippet = if line_snippet.is_empty() { "" } else { line_snippet };
    let carets = build_caret_annotation(1, snippet.len().max(1), snippet.len().max(1));
    DiagnosticError {
        message: "nested block directives are not supported".into(),
        file_path: "<input>".into(),
        line_number,
        col_number: 1,
        line_snippet: snippet.into(),
        annotation_carets: carets,
        annotation_text: "nested block directive opening found here".into(),
        help_text: "close the active block directive with ':::' before starting a new one.".into(),
    }
    .into()
}

/// Constructs a standardized diagnostic error for unclosed block directives.
fn build_unclosed_block_directive_err(line_number: usize, line_snippet: &str) -> Error {
    let snippet = if line_snippet.is_empty() { "" } else { line_snippet };
    let carets = build_caret_annotation(1, snippet.len().max(3), snippet.len().max(3));
    DiagnosticError {
        message: "unclosed block directive".into(),
        file_path: "<input>".into(),
        line_number,
        col_number: 1,
        line_snippet: snippet.into(),
        annotation_carets: carets,
        annotation_text: "block directive starting here is never closed".into(),
        help_text: "close the block directive with a closing ':::' line.".into(),
    }
    .into()
}

/// Returns the heading level if the element is a Section.
fn get_section_level(elem: &DocumentElement) -> usize {
    match elem {
        DocumentElement::Section { level, .. } => *level,
        _ => 0,
    }
}

/// Parses a markdown heading line into its normalized section level (1, 2, or 3) and title.
fn parse_heading_line(line: &str) -> Option<(usize, &str)> {
    let trimmed = line.trim_start();
    if !trimmed.starts_with('#') {
        return None;
    }
    let hash_count = trimmed.bytes().take_while(|&b| b == b'#').count();
    let rest = &trimmed[hash_count..];
    if rest.is_empty() {
        let level = match hash_count {
            1 => 1,
            2 => 2,
            _ => 3,
        };
        Some((level, ""))
    } else if rest.starts_with([' ', '\t']) {
        let level = match hash_count {
            1 => 1,
            2 => 2,
            _ => 3,
        };
        Some((level, rest.trim()))
    } else {
        None
    }
}

/// Unwinds closed sections from the stack and adds the new section to the hierarchy.
fn push_section(
    doc: &mut Document,
    section_stack: &mut Vec<DocumentElement>,
    level: usize,
    title: &str,
) {
    while let Some(top) = section_stack.last() {
        if get_section_level(top) >= level {
            let popped = section_stack.pop().unwrap();
            if let Some(parent) = section_stack.last_mut() {
                parent.push_child(popped);
            } else {
                doc.push_body(popped);
            }
        } else {
            break;
        }
    }
    section_stack.push(DocumentElement::section(level, title, Vec::new()));
}

/// Unwinds all active sections on the stack into their parent containers or document body.
fn flush_section_stack(doc: &mut Document, section_stack: &mut Vec<DocumentElement>) {
    while let Some(popped) = section_stack.pop() {
        if let Some(parent) = section_stack.last_mut() {
            parent.push_child(popped);
        } else {
            doc.push_body(popped);
        }
    }
}

/// Classifies a non-table line and appends it to target buffer or document body.
fn classify_and_push_line(
    doc: &mut Document,
    section_stack: &mut Vec<DocumentElement>,
    in_block: bool,
    block_children: &mut Vec<DocumentElement>,
    effective_line: &str,
    ordered_list_stack: &mut Vec<(usize, usize)>,
) {
    let trimmed = effective_line.trim();
    if trimmed.is_empty() {
        ordered_list_stack.clear();
        return;
    }

    if let Some((level, title)) = parse_heading_line(effective_line) {
        ordered_list_stack.clear();
        if in_block {
            push_element(
                doc,
                section_stack,
                in_block,
                block_children,
                DocumentElement::section(level, title, Vec::new()),
            );
        } else {
            push_section(doc, section_stack, level, title);
        }
    } else if let Some((depth, checked, content)) = parse_check_box_item(effective_line) {
        ordered_list_stack.clear();
        push_element(
            doc,
            section_stack,
            in_block,
            block_children,
            DocumentElement::check_box_item(depth, checked, parse_item_content(content)),
        );
    } else if let Some((depth, content)) = parse_bullet_list_item(effective_line) {
        ordered_list_stack.clear();
        push_element(
            doc,
            section_stack,
            in_block,
            block_children,
            DocumentElement::bullet_list_item(depth, parse_item_content(content)),
        );
    } else if let Some(elem) = try_process_ordered_list_item(effective_line, ordered_list_stack) {
        push_element(doc, section_stack, in_block, block_children, elem);
    } else if let Some((alt, url)) = parse_image(trimmed) {
        ordered_list_stack.clear();
        push_element(
            doc,
            section_stack,
            in_block,
            block_children,
            DocumentElement::image(alt, url),
        );
    } else if trimmed.starts_with('>') {
        ordered_list_stack.clear();
        let (shoutout_kind, content) = parse_shoutout_line(trimmed);
        push_element(
            doc,
            section_stack,
            in_block,
            block_children,
            DocumentElement::shoutout(shoutout_kind, content),
        );
    } else if is_plain_text(trimmed) {
        ordered_list_stack.clear();
        push_element(
            doc,
            section_stack,
            in_block,
            block_children,
            DocumentElement::text(trimmed),
        );
    } else {
        ordered_list_stack.clear();
        push_element(
            doc,
            section_stack,
            in_block,
            block_children,
            DocumentElement::unknown(trimmed),
        );
    }
}

/// Checks whether a line represents plain text.
fn is_plain_text(line: &str) -> bool {
    !line.starts_with(['#', '>', '-', '*', '`', '|'])
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

/// Parses Markdown content into a structured document model.
///
/// # Errors
///
/// Returns a diagnostic error if frontmatter is missing, unclosed, or improperly placed,
/// or if block directives are malformed, unclosed, or nested.
pub fn parse_d2f_markdown(md_content: &str) -> Result<Document, Error> {
    let mut doc = Document::new();
    let mut phase = FrontmatterPhase::SeekingStart;
    let mut comment_filter = CommentFilterState::new();
    let mut ordered_list_stack: Vec<(usize, usize)> = Vec::new();
    let mut section_stack: Vec<DocumentElement> = Vec::new();

    let mut in_code_block = false;
    let mut code_block_lang: Option<String> = None;
    let mut code_block_lines: Vec<&str> = Vec::new();

    let mut in_block_directive = false;
    let mut block_name = String::new();
    let mut block_start_line = 0;
    let mut block_start_snippet = "";
    let mut block_children: Vec<DocumentElement> = Vec::new();

    let mut active_table: Option<(Vec<TableAlignment>, Vec<Vec<String>>)> = None;
    let mut pending_table_header: Option<String> = None;

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
                push_element(
                    &mut doc,
                    &mut section_stack,
                    in_block_directive,
                    &mut block_children,
                    DocumentElement::code_block(code_block_lang.take(), content),
                );
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

                if trimmed_start.starts_with(":::") {
                    if let Some((alignments, rows)) = active_table.take() {
                        push_element(
                            &mut doc,
                            &mut section_stack,
                            in_block_directive,
                            &mut block_children,
                            DocumentElement::table(alignments, rows),
                        );
                    }
                    if let Some(pending) = pending_table_header.take() {
                        classify_and_push_line(
                            &mut doc,
                            &mut section_stack,
                            in_block_directive,
                            &mut block_children,
                            &pending,
                            &mut ordered_list_stack,
                        );
                    }
                    ordered_list_stack.clear();

                    let colon_count = trimmed_start.bytes().take_while(|&b| b == b':').count();
                    let rest = trimmed_start[colon_count..].trim();

                    if in_block_directive {
                        if rest.is_empty() {
                            let children = std::mem::take(&mut block_children);
                            let name = std::mem::take(&mut block_name);
                            push_element(
                                &mut doc,
                                &mut section_stack,
                                false,
                                &mut block_children,
                                DocumentElement::block_directive(name, children),
                            );
                            in_block_directive = false;
                            continue;
                        } else {
                            return Err(build_nested_block_directive_err(line_no, line));
                        }
                    } else if rest.is_empty() {
                        return Err(build_missing_block_directive_name_err(line_no, line));
                    } else if !rest.chars().all(|c| c.is_ascii_alphanumeric()) {
                        return Err(build_invalid_block_directive_name_err(line_no, line));
                    } else {
                        in_block_directive = true;
                        block_name = rest.to_string();
                        block_start_line = line_no;
                        block_start_snippet = line;
                        block_children.clear();
                        continue;
                    }
                }

                if trimmed_start.starts_with("```") {
                    if let Some((alignments, rows)) = active_table.take() {
                        push_element(
                            &mut doc,
                            &mut section_stack,
                            in_block_directive,
                            &mut block_children,
                            DocumentElement::table(alignments, rows),
                        );
                    }
                    if let Some(pending) = pending_table_header.take() {
                        classify_and_push_line(
                            &mut doc,
                            &mut section_stack,
                            in_block_directive,
                            &mut block_children,
                            &pending,
                            &mut ordered_list_stack,
                        );
                    }
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

                if let Some((_, ref mut rows)) = active_table {
                    let trimmed = effective_line.trim();
                    if !trimmed.is_empty()
                        && trimmed.contains('|')
                        && !trimmed.starts_with(['#', '>'])
                        && !trimmed.starts_with("```")
                        && !trimmed.starts_with(":::")
                    {
                        rows.push(parse_table_row(effective_line));
                        continue;
                    } else {
                        let (alignments, rows) = active_table.take().unwrap();
                        push_element(
                            &mut doc,
                            &mut section_stack,
                            in_block_directive,
                            &mut block_children,
                            DocumentElement::table(alignments, rows),
                        );
                    }
                }

                if let Some(pending_line) = pending_table_header.take() {
                    if let Some(alignments) = parse_table_delimiter_row(effective_line) {
                        let header_row = parse_table_row(&pending_line);
                        active_table = Some((alignments, vec![header_row]));
                        continue;
                    } else {
                        classify_and_push_line(
                            &mut doc,
                            &mut section_stack,
                            in_block_directive,
                            &mut block_children,
                            &pending_line,
                            &mut ordered_list_stack,
                        );
                    }
                }

                let trimmed = effective_line.trim();
                if trimmed.is_empty() {
                    ordered_list_stack.clear();
                    continue;
                }

                if trimmed.contains('|')
                    && !trimmed.starts_with(['#', '>'])
                    && !trimmed.starts_with("```")
                    && !trimmed.starts_with(":::")
                    && !trimmed.starts_with("- ")
                    && !trimmed.starts_with("- [")
                {
                    ordered_list_stack.clear();
                    pending_table_header = Some(effective_line.to_string());
                    continue;
                }

                classify_and_push_line(
                    &mut doc,
                    &mut section_stack,
                    in_block_directive,
                    &mut block_children,
                    effective_line,
                    &mut ordered_list_stack,
                );
            }
        }
    }

    if in_code_block {
        let content = trim_code_block_lines(&code_block_lines);
        push_element(
            &mut doc,
            &mut section_stack,
            in_block_directive,
            &mut block_children,
            DocumentElement::code_block(code_block_lang.take(), content),
        );
    }

    if let Some((alignments, rows)) = active_table.take() {
        push_element(
            &mut doc,
            &mut section_stack,
            in_block_directive,
            &mut block_children,
            DocumentElement::table(alignments, rows),
        );
    }
    if let Some(pending) = pending_table_header.take() {
        classify_and_push_line(
            &mut doc,
            &mut section_stack,
            in_block_directive,
            &mut block_children,
            &pending,
            &mut ordered_list_stack,
        );
    }

    if in_block_directive {
        return Err(build_unclosed_block_directive_err(
            block_start_line,
            block_start_snippet,
        ));
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
        FrontmatterPhase::Complete => {
            flush_section_stack(&mut doc, &mut section_stack);
            Ok(doc)
        }
    }
}

/// Parses a standalone image markdown pattern `![alt](url)` into alt text and url slices.
fn parse_image(input: &str) -> Option<(&str, &str)> {
    let trimmed = input.trim();
    let rest = trimmed.strip_prefix("![")?;
    let close_bracket_idx = rest.find("](")?;
    let alt = &rest[..close_bracket_idx];
    let after_bracket = &rest[close_bracket_idx + 2..];
    let url = after_bracket.strip_suffix(')')?;
    Some((alt, url.trim()))
}

/// Resolves inner list item content into a structured DocumentElement (e.g. Image or Text).
fn parse_item_content(content: &str) -> DocumentElement {
    if let Some((alt, url)) = parse_image(content) {
        DocumentElement::image(alt, url)
    } else {
        DocumentElement::text(content)
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

/// Parses a single cell in a table delimiter row into its `TableAlignment`.
fn parse_table_alignment(cell: &str) -> Option<TableAlignment> {
    let trimmed = cell.trim();
    if trimmed.is_empty() {
        return None;
    }

    let has_prefix_colon = trimmed.starts_with(':');
    let has_suffix_colon = trimmed.ends_with(':');

    let inner = if has_prefix_colon {
        &trimmed[1..]
    } else {
        trimmed
    };

    let inner = if has_suffix_colon && !inner.is_empty() {
        &inner[..inner.len() - 1]
    } else {
        inner
    };

    if inner.is_empty() || !inner.chars().all(|c| c == '-') {
        return None;
    }

    match (has_prefix_colon, has_suffix_colon) {
        (true, true) => Some(TableAlignment::Center),
        (true, false) => Some(TableAlignment::Left),
        (false, true) => Some(TableAlignment::Right),
        (false, false) => Some(TableAlignment::None),
    }
}

/// Parses a markdown table delimiter row (e.g. `| :--- | :---: | ---: |`) into column alignments.
fn parse_table_delimiter_row(line: &str) -> Option<Vec<TableAlignment>> {
    let trimmed = line.trim();
    if !trimmed.contains('|') && !trimmed.starts_with('-') {
        return None;
    }

    let inner = if let Some(stripped) = trimmed.strip_prefix('|') {
        stripped
    } else {
        trimmed
    };

    let inner = if inner.ends_with('|') && !inner.ends_with("\\|") {
        &inner[..inner.len() - 1]
    } else {
        inner
    };

    if inner.trim().is_empty() {
        return None;
    }

    let raw_cells: Vec<&str> = inner.split('|').collect();
    if raw_cells.is_empty() {
        return None;
    }

    let mut alignments = Vec::with_capacity(raw_cells.len());
    for raw_cell in raw_cells {
        let align = parse_table_alignment(raw_cell)?;
        alignments.push(align);
    }

    Some(alignments)
}

/// Parses a markdown table row into trimmed cell string contents, unescaping escaped pipes (`\|`).
fn parse_table_row(line: &str) -> Vec<String> {
    let trimmed = line.trim();
    let inner = if let Some(stripped) = trimmed.strip_prefix('|') {
        stripped
    } else {
        trimmed
    };

    let inner = if inner.ends_with('|') && !inner.ends_with("\\|") {
        &inner[..inner.len() - 1]
    } else {
        inner
    };

    let mut cells = Vec::new();
    let mut current_cell = String::with_capacity(32);
    let mut chars = inner.chars().peekable();

    while let Some(ch) = chars.next() {
        if ch == '\\' {
            if let Some(&next_ch) = chars.peek() {
                if next_ch == '|' {
                    current_cell.push('|');
                    chars.next();
                    continue;
                }
            }
            current_cell.push('\\');
        } else if ch == '|' {
            cells.push(current_cell.trim().to_string());
            current_cell.clear();
        } else {
            current_cell.push(ch);
        }
    }
    cells.push(current_cell.trim().to_string());
    cells
}

/// Appends an element either to the active block children buffer, the topmost active section, or document body.
fn push_element(
    doc: &mut Document,
    section_stack: &mut [DocumentElement],
    in_block: bool,
    block_children: &mut Vec<DocumentElement>,
    elem: DocumentElement,
) {
    if in_block {
        block_children.push(elem);
    } else if let Some(active_section) = section_stack.last_mut() {
        active_section.push_child(elem);
    } else {
        doc.push_body(elem);
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

/// Attempts to process an ordered list item line against the active list depth stack.
fn try_process_ordered_list_item(
    line: &str,
    stack: &mut Vec<(usize, usize)>,
) -> Option<DocumentElement> {
    let (depth, parsed_num, content) = parse_ordered_list_candidate(line)?;
    let content_elem = parse_item_content(content);

    if stack.is_empty() {
        if parsed_num == 1 {
            stack.push((depth, 1));
            Some(DocumentElement::ordered_list_item(
                depth,
                1,
                content_elem,
            ))
        } else {
            None
        }
    } else {
        let last_depth = stack.last().unwrap().0;
        if depth > last_depth {
            stack.push((depth, 1));
            Some(DocumentElement::ordered_list_item(
                depth,
                1,
                content_elem,
            ))
        } else if depth == last_depth {
            let entry = stack.last_mut().unwrap();
            entry.1 += 1;
            let position = entry.1;
            Some(DocumentElement::ordered_list_item(
                depth,
                position,
                content_elem,
            ))
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
                    Some(DocumentElement::ordered_list_item(
                        depth,
                        position,
                        content_elem,
                    ))
                } else if parsed_num == 1 {
                    stack.push((depth, 1));
                    Some(DocumentElement::ordered_list_item(
                        depth,
                        1,
                        content_elem,
                    ))
                } else {
                    stack.clear();
                    None
                }
            } else if parsed_num == 1 {
                stack.push((depth, 1));
                Some(DocumentElement::ordered_list_item(
                    depth,
                    1,
                    content_elem,
                ))
            } else {
                None
            }
        }
    }
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
        assert_eq!(doc.body.len(), 1);

        assert_eq!(
            doc.body[0],
            DocumentElement::section(
                1,
                "Heading 1",
                vec![
                    DocumentElement::Text("This is plain text paragraph.".into()),
                    DocumentElement::Shoutout {
                        kind: ShoutoutElementKind::Note,
                        content: "Callout note".into(),
                    },
                    DocumentElement::Text("Another text.".into()),
                ]
            )
        );
    }

    #[test]
    fn test_parse_d2f_markdown_comments_before_frontmatter() {
        let md = "<!-- Comment at top -->\n\n<!--\nMultiline comment\n-->\n<!-- inline -->\n---\ntitle: \"Doc\"\n---\n# Heading 1\nVisible text";
        let doc = parse_d2f_markdown(md).unwrap();

        assert_eq!(doc.parameters.get("title").map(|s| s.as_str()), Some("Doc"));
        assert_eq!(doc.body.len(), 1);
        assert_eq!(
            doc.body[0],
            DocumentElement::section(
                1,
                "Heading 1",
                vec![DocumentElement::Text("Visible text".into())]
            )
        );
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
        assert_eq!(doc.body.len(), 1);
        assert_eq!(
            doc.body[0],
            DocumentElement::section(
                1,
                "Heading",
                vec![DocumentElement::Text("Text".into())]
            )
        );
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
        assert_eq!(doc.body[0], DocumentElement::bullet_list_item(0, DocumentElement::text("Level 0")));
        assert_eq!(doc.body[1], DocumentElement::bullet_list_item(1, DocumentElement::text("Level 1a")));
        assert_eq!(doc.body[2], DocumentElement::bullet_list_item(1, DocumentElement::text("Level 1b")));
        assert_eq!(doc.body[3], DocumentElement::bullet_list_item(2, DocumentElement::text("Level 2a")));
        assert_eq!(doc.body[4], DocumentElement::bullet_list_item(2, DocumentElement::text("Level 2b")));
        assert_eq!(doc.body[5], DocumentElement::text("Regular text"));
        assert_eq!(doc.body[6], DocumentElement::bullet_list_item(0, DocumentElement::text("Another root")));
    }

    #[test]
    fn test_parse_d2f_markdown_bullet_list_with_comments() {
        let md = "---\ntitle: \"Comments in List\"\n---\n  - Item 1 <!-- inline comment -->\n<!-- multiline\ncomment -->\n    - Item 2";
        let doc = parse_d2f_markdown(md).unwrap();

        assert_eq!(doc.body.len(), 2);
        assert_eq!(doc.body[0], DocumentElement::bullet_list_item(1, DocumentElement::text("Item 1")));
        assert_eq!(doc.body[1], DocumentElement::bullet_list_item(2, DocumentElement::text("Item 2")));
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
        assert_eq!(doc.body[0], DocumentElement::check_box_item(0, false, DocumentElement::text("Task 1")));
        assert_eq!(doc.body[1], DocumentElement::check_box_item(1, true, DocumentElement::text("Subtask 1.1")));
        assert_eq!(doc.body[2], DocumentElement::check_box_item(1, true, DocumentElement::text("Subtask 1.2")));
        assert_eq!(doc.body[3], DocumentElement::check_box_item(2, false, DocumentElement::text("Sub-subtask")));
        assert_eq!(doc.body[4], DocumentElement::check_box_item(0, true, DocumentElement::text("Task 2")));
        assert_eq!(doc.body[5], DocumentElement::check_box_item(0, false, DocumentElement::text("")));
    }

    #[test]
    fn test_parse_d2f_markdown_mixed_lists() {
        let md = "---\ntitle: \"Mixed\"\n---\n- Bullet 1\n- [ ] Task 1\n  - Bullet nested\n  - [x] Task nested\n- Bullet 2";
        let doc = parse_d2f_markdown(md).unwrap();

        assert_eq!(doc.body.len(), 5);
        assert_eq!(doc.body[0], DocumentElement::bullet_list_item(0, DocumentElement::text("Bullet 1")));
        assert_eq!(doc.body[1], DocumentElement::check_box_item(0, false, DocumentElement::text("Task 1")));
        assert_eq!(doc.body[2], DocumentElement::bullet_list_item(1, DocumentElement::text("Bullet nested")));
        assert_eq!(doc.body[3], DocumentElement::check_box_item(1, true, DocumentElement::text("Task nested")));
        assert_eq!(doc.body[4], DocumentElement::bullet_list_item(0, DocumentElement::text("Bullet 2")));
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
        assert_eq!(doc.body[0], DocumentElement::ordered_list_item(0, 1, DocumentElement::text("First")));
        assert_eq!(doc.body[1], DocumentElement::ordered_list_item(0, 2, DocumentElement::text("Second")));
        assert_eq!(doc.body[2], DocumentElement::ordered_list_item(0, 3, DocumentElement::text("Third")));
    }

    #[test]
    fn test_parse_d2f_markdown_ordered_lists_arbitrary_numbers() {
        let md1 = "---\ntitle: \"Repeated 1s\"\n---\n1. Apple\n1. Banana\n1. Cherry";
        let doc1 = parse_d2f_markdown(md1).unwrap();
        assert_eq!(doc1.body.len(), 3);
        assert_eq!(doc1.body[0], DocumentElement::ordered_list_item(0, 1, DocumentElement::text("Apple")));
        assert_eq!(doc1.body[1], DocumentElement::ordered_list_item(0, 2, DocumentElement::text("Banana")));
        assert_eq!(doc1.body[2], DocumentElement::ordered_list_item(0, 3, DocumentElement::text("Cherry")));

        let md2 = "---\ntitle: \"Random Numbers\"\n---\n1. Red\n5. Green\n99. Blue";
        let doc2 = parse_d2f_markdown(md2).unwrap();
        assert_eq!(doc2.body.len(), 3);
        assert_eq!(doc2.body[0], DocumentElement::ordered_list_item(0, 1, DocumentElement::text("Red")));
        assert_eq!(doc2.body[1], DocumentElement::ordered_list_item(0, 2, DocumentElement::text("Green")));
        assert_eq!(doc2.body[2], DocumentElement::ordered_list_item(0, 3, DocumentElement::text("Blue")));
    }

    #[test]
    fn test_parse_d2f_markdown_ordered_lists_nested_and_returns() {
        let md = "---\ntitle: \"Nested\"\n---\n1. L0_1\n  1. L1_1\n  2. L1_2\n    1. L2_1\n  3. L1_3\n2. L0_2";
        let doc = parse_d2f_markdown(md).unwrap();

        assert_eq!(doc.body.len(), 6);
        assert_eq!(doc.body[0], DocumentElement::ordered_list_item(0, 1, DocumentElement::text("L0_1")));
        assert_eq!(doc.body[1], DocumentElement::ordered_list_item(1, 1, DocumentElement::text("L1_1")));
        assert_eq!(doc.body[2], DocumentElement::ordered_list_item(1, 2, DocumentElement::text("L1_2")));
        assert_eq!(doc.body[3], DocumentElement::ordered_list_item(2, 1, DocumentElement::text("L2_1")));
        assert_eq!(doc.body[4], DocumentElement::ordered_list_item(1, 3, DocumentElement::text("L1_3")));
        assert_eq!(doc.body[5], DocumentElement::ordered_list_item(0, 2, DocumentElement::text("L0_2")));
    }

    #[test]
    fn test_parse_d2f_markdown_ordered_lists_new_depth_arbitrary_number() {
        let md = "---\ntitle: \"New Depth Start\"\n---\n1. Root 1\n  7. Subitem 1\n  8. Subitem 2\n2. Root 2";
        let doc = parse_d2f_markdown(md).unwrap();

        assert_eq!(doc.body.len(), 4);
        assert_eq!(doc.body[0], DocumentElement::ordered_list_item(0, 1, DocumentElement::text("Root 1")));
        assert_eq!(doc.body[1], DocumentElement::ordered_list_item(1, 1, DocumentElement::text("Subitem 1")));
        assert_eq!(doc.body[2], DocumentElement::ordered_list_item(1, 2, DocumentElement::text("Subitem 2")));
        assert_eq!(doc.body[3], DocumentElement::ordered_list_item(0, 2, DocumentElement::text("Root 2")));
    }

    #[test]
    fn test_parse_d2f_markdown_ordered_lists_interrupted_by_text() {
        let md = "---\ntitle: \"Interrupted\"\n---\n1. Item 1\n2. Item 2\n\nParagraph text\n\n1. New list 1\n2. New list 2\n\nAnother paragraph\n\n2. Not a list";
        let doc = parse_d2f_markdown(md).unwrap();

        assert_eq!(doc.body.len(), 7);
        assert_eq!(doc.body[0], DocumentElement::ordered_list_item(0, 1, DocumentElement::text("Item 1")));
        assert_eq!(doc.body[1], DocumentElement::ordered_list_item(0, 2, DocumentElement::text("Item 2")));
        assert_eq!(doc.body[2], DocumentElement::text("Paragraph text"));
        assert_eq!(doc.body[3], DocumentElement::ordered_list_item(0, 1, DocumentElement::text("New list 1")));
        assert_eq!(doc.body[4], DocumentElement::ordered_list_item(0, 2, DocumentElement::text("New list 2")));
        assert_eq!(doc.body[5], DocumentElement::text("Another paragraph"));
        assert_eq!(doc.body[6], DocumentElement::text("2. Not a list"));
    }

    #[test]
    fn test_parse_d2f_markdown_ordered_lists_interrupted_by_other_elements() {
        let md = "---\ntitle: \"Interrupted by Elements\"\n---\n1. Item 1\n- Bullet item\n1. Item after bullet\n- [ ] Check item\n1. Item after check\n> Note shoutout\n1. Item after shoutout\n```\ncode\n```\n1. Item after code";
        let doc = parse_d2f_markdown(md).unwrap();

        assert_eq!(doc.body.len(), 9);
        assert_eq!(doc.body[0], DocumentElement::ordered_list_item(0, 1, DocumentElement::text("Item 1")));
        assert_eq!(doc.body[1], DocumentElement::bullet_list_item(0, DocumentElement::text("Bullet item")));
        assert_eq!(doc.body[2], DocumentElement::ordered_list_item(0, 1, DocumentElement::text("Item after bullet")));
        assert_eq!(doc.body[3], DocumentElement::check_box_item(0, false, DocumentElement::text("Check item")));
        assert_eq!(doc.body[4], DocumentElement::ordered_list_item(0, 1, DocumentElement::text("Item after check")));
        assert_eq!(
            doc.body[5],
            DocumentElement::shoutout(ShoutoutElementKind::Note, "Note shoutout")
        );
        assert_eq!(doc.body[6], DocumentElement::ordered_list_item(0, 1, DocumentElement::text("Item after shoutout")));
        assert_eq!(
            doc.body[7],
            DocumentElement::code_block(None::<String>, "code")
        );
        assert_eq!(doc.body[8], DocumentElement::ordered_list_item(0, 1, DocumentElement::text("Item after code")));
    }

    #[test]
    fn test_parse_d2f_markdown_ordered_lists_with_comments() {
        let md = "---\ntitle: \"Comments in Ordered List\"\n---\n1. Item 1 <!-- inline -->\n<!-- multiline\ncomment -->\n2. Item 2\n  1. Subitem <!-- comment -->";
        let doc = parse_d2f_markdown(md).unwrap();

        assert_eq!(doc.body.len(), 3);
        assert_eq!(doc.body[0], DocumentElement::ordered_list_item(0, 1, DocumentElement::text("Item 1")));
        assert_eq!(doc.body[1], DocumentElement::ordered_list_item(0, 2, DocumentElement::text("Item 2")));
        assert_eq!(doc.body[2], DocumentElement::ordered_list_item(1, 1, DocumentElement::text("Subitem")));
    }

    #[test]
    fn test_parse_table_alignment_cases() {
        assert_eq!(parse_table_alignment("---"), Some(TableAlignment::None));
        assert_eq!(parse_table_alignment(" :--- "), Some(TableAlignment::Left));
        assert_eq!(parse_table_alignment(":---:"), Some(TableAlignment::Center));
        assert_eq!(parse_table_alignment("---:"), Some(TableAlignment::Right));
        assert_eq!(parse_table_alignment("-"), Some(TableAlignment::None));
        assert_eq!(parse_table_alignment(":-"), Some(TableAlignment::Left));
        assert_eq!(parse_table_alignment(":-:"), Some(TableAlignment::Center));
        assert_eq!(parse_table_alignment("-:"), Some(TableAlignment::Right));
        assert_eq!(parse_table_alignment(""), None);
        assert_eq!(parse_table_alignment(":::"), None);
        assert_eq!(parse_table_alignment("abc"), None);
        assert_eq!(parse_table_alignment("--x--"), None);
    }

    #[test]
    fn test_parse_table_delimiter_row_cases() {
        let align1 = parse_table_delimiter_row("| :--- | :---: | ---: | --- |");
        assert_eq!(
            align1,
            Some(vec![
                TableAlignment::Left,
                TableAlignment::Center,
                TableAlignment::Right,
                TableAlignment::None,
            ])
        );

        let align2 = parse_table_delimiter_row(":-: | -:");
        assert_eq!(
            align2,
            Some(vec![TableAlignment::Center, TableAlignment::Right])
        );

        assert_eq!(parse_table_delimiter_row("Not a delimiter"), None);
        assert_eq!(parse_table_delimiter_row("| --- | abc |"), None);
    }

    #[test]
    fn test_parse_table_row_cases() {
        assert_eq!(
            parse_table_row("| A | B | C |"),
            vec!["A", "B", "C"]
        );
        assert_eq!(
            parse_table_row("A | B"),
            vec!["A", "B"]
        );
        assert_eq!(
            parse_table_row("| A \\| B | C |"),
            vec!["A | B", "C"]
        );
        assert_eq!(
            parse_table_row("|   Spaced   |   Content   |"),
            vec!["Spaced", "Content"]
        );
        assert_eq!(
            parse_table_row("| Empty | | End |"),
            vec!["Empty", "", "End"]
        );
    }

    #[test]
    fn test_parse_d2f_markdown_table_basic() {
        let md = "---\ntitle: \"Table Doc\"\n---\n| Name | Role | Location |\n| :--- | :---: | ---: |\n| Alice | Engineer | Berlin |\n| Bob | Designer | London |";
        let doc = parse_d2f_markdown(md).unwrap();

        assert_eq!(doc.body.len(), 1);
        assert_eq!(
            doc.body[0],
            DocumentElement::table(
                vec![
                    TableAlignment::Left,
                    TableAlignment::Center,
                    TableAlignment::Right,
                ],
                vec![
                    vec!["Name".into(), "Role".into(), "Location".into()],
                    vec!["Alice".into(), "Engineer".into(), "Berlin".into()],
                    vec!["Bob".into(), "Designer".into(), "London".into()],
                ]
            )
        );
    }

    #[test]
    fn test_parse_d2f_markdown_table_with_escaped_pipes_and_no_outer_pipes() {
        let md = "---\ntitle: \"Escaped Pipes\"\n---\nSyntax | Description\n---|---\n`grep \\| sort` | Pipeline\n`d2f -o file` | CLI output";
        let doc = parse_d2f_markdown(md).unwrap();

        assert_eq!(doc.body.len(), 1);
        assert_eq!(
            doc.body[0],
            DocumentElement::table(
                vec![TableAlignment::None, TableAlignment::None],
                vec![
                    vec!["Syntax".into(), "Description".into()],
                    vec!["`grep | sort`".into(), "Pipeline".into()],
                    vec!["`d2f -o file`".into(), "CLI output".into()],
                ]
            )
        );
    }

    #[test]
    fn test_parse_d2f_markdown_table_interrupted_by_blank_line() {
        let md = "---\ntitle: \"Multiple Tables\"\n---\n| T1H1 | T1H2 |\n| --- | --- |\n| R1 | R2 |\n\n| T2H1 | T2H2 |\n| :--- | ---: |\n| V1 | V2 |";
        let doc = parse_d2f_markdown(md).unwrap();

        assert_eq!(doc.body.len(), 2);
        assert_eq!(
            doc.body[0],
            DocumentElement::table(
                vec![TableAlignment::None, TableAlignment::None],
                vec![
                    vec!["T1H1".into(), "T1H2".into()],
                    vec!["R1".into(), "R2".into()],
                ]
            )
        );
        assert_eq!(
            doc.body[1],
            DocumentElement::table(
                vec![TableAlignment::Left, TableAlignment::Right],
                vec![
                    vec!["T2H1".into(), "T2H2".into()],
                    vec!["V1".into(), "V2".into()],
                ]
            )
        );
    }

    #[test]
    fn test_parse_d2f_markdown_table_interrupted_by_heading_and_lists() {
        let md = "---\ntitle: \"Table With Other Elements\"\n---\n| Col A | Col B |\n| --- | --- |\n| A1 | B1 |\n# Next Section\n- Bullet item\n\n| Next Table |\n| --- |\n| Only Row |";
        let doc = parse_d2f_markdown(md).unwrap();

        assert_eq!(doc.body.len(), 2);
        assert_eq!(
            doc.body[0],
            DocumentElement::table(
                vec![TableAlignment::None, TableAlignment::None],
                vec![
                    vec!["Col A".into(), "Col B".into()],
                    vec!["A1".into(), "B1".into()],
                ]
            )
        );
        assert_eq!(
            doc.body[1],
            DocumentElement::section(
                1,
                "Next Section",
                vec![
                    DocumentElement::bullet_list_item(0, DocumentElement::text("Bullet item")),
                    DocumentElement::table(
                        vec![TableAlignment::None],
                        vec![
                            vec!["Next Table".into()],
                            vec!["Only Row".into()],
                        ]
                    ),
                ]
            )
        );
    }

    #[test]
    fn test_parse_d2f_markdown_table_with_comments() {
        let md = "---\ntitle: \"Table With Comments\"\n---\n| Col 1 <!-- inline --> | Col 2 |\n<!-- multiline\ncomment -->\n| --- | --- |\n| Val 1 | Val 2 <!-- comment --> |";
        let doc = parse_d2f_markdown(md).unwrap();

        assert_eq!(doc.body.len(), 1);
        assert_eq!(
            doc.body[0],
            DocumentElement::table(
                vec![TableAlignment::None, TableAlignment::None],
                vec![
                    vec!["Col 1".into(), "Col 2".into()],
                    vec!["Val 1".into(), "Val 2".into()],
                ]
            )
        );
    }

    #[test]
    fn test_parse_d2f_markdown_pipe_in_text_fallback() {
        let md = "---\ntitle: \"Non Table Pipes\"\n---\nThis is plain text with | pipe character.\nAnother regular sentence.\n\nOption A | Option B\nNot a table.";
        let doc = parse_d2f_markdown(md).unwrap();

        assert_eq!(doc.body.len(), 4);
        assert_eq!(
            doc.body[0],
            DocumentElement::text("This is plain text with | pipe character.")
        );
        assert_eq!(
            doc.body[1],
            DocumentElement::text("Another regular sentence.")
        );
        assert_eq!(
            doc.body[2],
            DocumentElement::text("Option A | Option B")
        );
        assert_eq!(
            doc.body[3],
            DocumentElement::text("Not a table.")
        );
    }

    #[test]
    fn test_parse_d2f_markdown_table_at_eof() {
        let md = "---\ntitle: \"Table at EOF\"\n---\n| H1 | H2 |\n| --- | --- |\n| D1 | D2 |";
        let doc = parse_d2f_markdown(md).unwrap();

        assert_eq!(doc.body.len(), 1);
        assert_eq!(
            doc.body[0],
            DocumentElement::table(
                vec![TableAlignment::None, TableAlignment::None],
                vec![
                    vec!["H1".into(), "H2".into()],
                    vec!["D1".into(), "D2".into()],
                ]
            )
        );
    }

    #[test]
    fn test_parse_d2f_markdown_block_directive_variables_example() {
        let md = "---\ntitle: \"Block Directive Doc\"\n---\n:::variables\n| Variable | Value |\n| --- | --- |\n| TARGET_HOST | 192.168.1.100 |\n| SERVICE_PORT | 8080 |\nDas ist ein Text.\n\nDas auch.\n:::";
        let doc = parse_d2f_markdown(md).unwrap();

        assert_eq!(doc.body.len(), 1);
        match &doc.body[0] {
            DocumentElement::BlockDirective { name, children } => {
                assert_eq!(name, "variables");
                assert_eq!(children.len(), 3);
                assert_eq!(
                    children[0],
                    DocumentElement::table(
                        vec![TableAlignment::None, TableAlignment::None],
                        vec![
                            vec!["Variable".into(), "Value".into()],
                            vec!["TARGET_HOST".into(), "192.168.1.100".into()],
                            vec!["SERVICE_PORT".into(), "8080".into()],
                        ]
                    )
                );
                assert_eq!(children[1], DocumentElement::text("Das ist ein Text."));
                assert_eq!(children[2], DocumentElement::text("Das auch."));
            }
            other => panic!("Expected BlockDirective, got {other:?}"),
        }
    }

    #[test]
    fn test_parse_d2f_markdown_block_directive_with_mixed_elements() {
        let md = "---\ntitle: \"Mixed Block Directive\"\n---\n:::custom123\n- Bullet 1\n- [x] Checkbox\n1. Numbered 1\n>! Important shoutout\n```bash\necho test\n```\n:::";
        let doc = parse_d2f_markdown(md).unwrap();

        assert_eq!(doc.body.len(), 1);
        if let DocumentElement::BlockDirective { name, children } = &doc.body[0] {
            assert_eq!(name, "custom123");
            assert_eq!(children.len(), 5);
            assert_eq!(children[0], DocumentElement::bullet_list_item(0, DocumentElement::text("Bullet 1")));
            assert_eq!(children[1], DocumentElement::check_box_item(0, true, DocumentElement::text("Checkbox")));
            assert_eq!(children[2], DocumentElement::ordered_list_item(0, 1, DocumentElement::text("Numbered 1")));
            assert_eq!(children[3], DocumentElement::shoutout(ShoutoutElementKind::Important, "Important shoutout"));
            assert_eq!(children[4], DocumentElement::code_block(Some("bash"), "echo test"));
        } else {
            panic!("Expected BlockDirective");
        }
    }

    #[test]
    fn test_parse_d2f_markdown_block_directive_with_extra_colons() {
        let md = "---\ntitle: \"Extra Colons\"\n---\n:::::::::config\nConfig text.\n:::::";
        let doc = parse_d2f_markdown(md).unwrap();

        assert_eq!(doc.body.len(), 1);
        assert_eq!(
            doc.body[0],
            DocumentElement::block_directive("config", vec![DocumentElement::text("Config text.")])
        );
    }

    #[test]
    fn test_parse_d2f_markdown_multiple_block_directives() {
        let md = "---\ntitle: \"Multiple Directives\"\n---\n:::blockA\nText A\n:::\n\nMiddle text\n\n:::blockB\nText B\n:::";
        let doc = parse_d2f_markdown(md).unwrap();

        assert_eq!(doc.body.len(), 3);
        assert_eq!(
            doc.body[0],
            DocumentElement::block_directive("blockA", vec![DocumentElement::text("Text A")])
        );
        assert_eq!(doc.body[1], DocumentElement::text("Middle text"));
        assert_eq!(
            doc.body[2],
            DocumentElement::block_directive("blockB", vec![DocumentElement::text("Text B")])
        );
    }

    #[test]
    fn test_parse_d2f_markdown_block_directive_unclosed_error() {
        let md = "---\ntitle: \"Unclosed Directive\"\n---\n:::variables\n| A | B |\n| --- | --- |\n| 1 | 2 |";
        let err = parse_d2f_markdown(md).unwrap_err();
        let err_str = err.to_string();
        assert!(err_str.contains("unclosed block directive"));
        assert!(err_str.contains("block directive starting here is never closed"));
    }

    #[test]
    fn test_parse_d2f_markdown_block_directive_missing_name_error() {
        let md = "---\ntitle: \"Missing Name\"\n---\n:::\nSome text\n:::";
        let err = parse_d2f_markdown(md).unwrap_err();
        let err_str = err.to_string();
        assert!(err_str.contains("missing block directive name"));
        assert!(err_str.contains("expected directive name after colons"));
    }

    #[test]
    fn test_parse_d2f_markdown_block_directive_invalid_name_error() {
        let invalid_names = [
            ":::var-name\ntext\n:::",
            ":::var_name\ntext\n:::",
            ":::var.name\ntext\n:::",
            ":::var!name\ntext\n:::",
            ":::var name\ntext\n:::",
        ];

        for md in invalid_names {
            let full_md = format!("---\ntitle: \"Invalid Name\"\n---\n{md}");
            let err = parse_d2f_markdown(&full_md).unwrap_err();
            let err_str = err.to_string();
            assert!(err_str.contains("invalid block directive name"));
            assert!(err_str.contains("directive name must contain only alphanumeric characters"));
        }
    }

    #[test]
    fn test_parse_d2f_markdown_block_directive_nested_error() {
        let md = "---\ntitle: \"Nested Directive\"\n---\n:::outer\nText\n:::inner\nNested text\n:::\n:::";
        let err = parse_d2f_markdown(md).unwrap_err();
        let err_str = err.to_string();
        assert!(err_str.contains("nested block directives are not supported"));
        assert!(err_str.contains("nested block directive opening found here"));
    }

    #[test]
    fn test_parse_d2f_markdown_standalone_images() {
        let md = "---\ntitle: \"Images Test\"\n---\n![Architecture Diagram](assets/arch.png)\n![](https://example.com/logo.svg)\n![Empty URL]()\n![ With Spaces ](  https://example.com/pic.webp  )";
        let doc = parse_d2f_markdown(md).unwrap();

        assert_eq!(doc.body.len(), 4);
        assert_eq!(
            doc.body[0],
            DocumentElement::image("Architecture Diagram", "assets/arch.png")
        );
        assert_eq!(
            doc.body[1],
            DocumentElement::image("", "https://example.com/logo.svg")
        );
        assert_eq!(
            doc.body[2],
            DocumentElement::image("Empty URL", "")
        );
        assert_eq!(
            doc.body[3],
            DocumentElement::image(" With Spaces ", "https://example.com/pic.webp")
        );
    }

    #[test]
    fn test_parse_d2f_markdown_images_in_lists() {
        let md = "---\ntitle: \"List Images\"\n---\n- ![Bullet Img](img/bullet.png)\n- [x] ![Check Img](img/check.png)\n- [ ] Normal text\n1. ![Ordered Img 1](img/step1.png)\n2. ![Ordered Img 2](img/step2.png)";
        let doc = parse_d2f_markdown(md).unwrap();

        assert_eq!(doc.body.len(), 5);
        assert_eq!(
            doc.body[0],
            DocumentElement::bullet_list_item(
                0,
                DocumentElement::image("Bullet Img", "img/bullet.png")
            )
        );
        assert_eq!(
            doc.body[1],
            DocumentElement::check_box_item(
                0,
                true,
                DocumentElement::image("Check Img", "img/check.png")
            )
        );
        assert_eq!(
            doc.body[2],
            DocumentElement::check_box_item(
                0,
                false,
                DocumentElement::text("Normal text")
            )
        );
        assert_eq!(
            doc.body[3],
            DocumentElement::ordered_list_item(
                0,
                1,
                DocumentElement::image("Ordered Img 1", "img/step1.png")
            )
        );
        assert_eq!(
            doc.body[4],
            DocumentElement::ordered_list_item(
                0,
                2,
                DocumentElement::image("Ordered Img 2", "img/step2.png")
            )
        );
    }

    #[test]
    fn test_parse_d2f_markdown_images_in_block_directive() {
        let md = "---\ntitle: \"Block Directive Images\"\n---\n:::gallery\n![Pic 1](pic1.jpg)\n- ![Nested Pic](pic2.jpg)\nText\n:::";
        let doc = parse_d2f_markdown(md).unwrap();

        assert_eq!(doc.body.len(), 1);
        if let DocumentElement::BlockDirective { name, children } = &doc.body[0] {
            assert_eq!(name, "gallery");
            assert_eq!(children.len(), 3);
            assert_eq!(
                children[0],
                DocumentElement::image("Pic 1", "pic1.jpg")
            );
            assert_eq!(
                children[1],
                DocumentElement::bullet_list_item(
                    0,
                    DocumentElement::image("Nested Pic", "pic2.jpg")
                )
            );
            assert_eq!(
                children[2],
                DocumentElement::text("Text")
            );
        } else {
            panic!("expected BlockDirective");
        }
    }

    #[test]
    fn test_parse_d2f_markdown_image_edge_cases() {
        let md = "---\ntitle: \"Image Edge Cases\"\n---\n! Not an image\n![Unclosed bracket(url)\n![Alt]no paren\n![Alt](no closing paren\nPrefix ![Alt](url) suffix";
        let doc = parse_d2f_markdown(md).unwrap();

        assert_eq!(doc.body.len(), 5);
        assert_eq!(doc.body[0], DocumentElement::text("! Not an image"));
        assert_eq!(doc.body[1], DocumentElement::text("![Unclosed bracket(url)"));
        assert_eq!(doc.body[2], DocumentElement::text("![Alt]no paren"));
        assert_eq!(doc.body[3], DocumentElement::text("![Alt](no closing paren"));
        assert_eq!(doc.body[4], DocumentElement::text("Prefix ![Alt](url) suffix"));
    }

    #[test]
    fn test_parse_heading_line() {
        assert_eq!(parse_heading_line("# Title"), Some((1, "Title")));
        assert_eq!(parse_heading_line("## Subtitle"), Some((2, "Subtitle")));
        assert_eq!(parse_heading_line("### Sub-sub"), Some((3, "Sub-sub")));
        assert_eq!(parse_heading_line("#### Level 4 capped"), Some((3, "Level 4 capped")));
        assert_eq!(parse_heading_line("########## Level 10 capped"), Some((3, "Level 10 capped")));
        assert_eq!(parse_heading_line("#"), Some((1, "")));
        assert_eq!(parse_heading_line("##"), Some((2, "")));
        assert_eq!(parse_heading_line("###"), Some((3, "")));
        assert_eq!(parse_heading_line("   # Indented"), Some((1, "Indented")));
        assert_eq!(parse_heading_line("#hashtag"), None);
        assert_eq!(parse_heading_line("Not a heading"), None);
    }

    #[test]
    fn test_parse_d2f_markdown_section_hierarchy_and_capping() {
        let md = "---\ntitle: \"Sections Doc\"\n---\nIntro text\n\n# Section 1\nText in 1\n\n## Section 1.1\nText in 1.1\n\n### Section 1.1.1\nText in 1.1.1\n\n#### Section 1.1.2 Capped\nText in 1.1.2\n\n########## Section 1.1.3 Capped\nText in 1.1.3\n\n## Section 1.2\nText in 1.2\n\n# Section 2\nText in 2";
        let doc = parse_d2f_markdown(md).unwrap();

        assert_eq!(doc.body.len(), 3);
        // Intro text
        assert_eq!(doc.body[0], DocumentElement::text("Intro text"));

        // Section 1 (Level 1)
        if let DocumentElement::Section { level, title, children } = &doc.body[1] {
            assert_eq!(*level, 1);
            assert_eq!(title, "Section 1");
            assert_eq!(children.len(), 3);
            assert_eq!(children[0], DocumentElement::text("Text in 1"));

            // Section 1.1 (Level 2)
            if let DocumentElement::Section { level: l2_1, title: t2_1, children: ch2_1 } = &children[1] {
                assert_eq!(*l2_1, 2);
                assert_eq!(t2_1, "Section 1.1");
                assert_eq!(ch2_1.len(), 4);
                assert_eq!(ch2_1[0], DocumentElement::text("Text in 1.1"));

                // Section 1.1.1 (Level 3)
                assert_eq!(
                    ch2_1[1],
                    DocumentElement::section(3, "Section 1.1.1", vec![DocumentElement::text("Text in 1.1.1")])
                );
                // Section 1.1.2 (Level 3, capped from 4)
                assert_eq!(
                    ch2_1[2],
                    DocumentElement::section(3, "Section 1.1.2 Capped", vec![DocumentElement::text("Text in 1.1.2")])
                );
                // Section 1.1.3 (Level 3, capped from 10)
                assert_eq!(
                    ch2_1[3],
                    DocumentElement::section(3, "Section 1.1.3 Capped", vec![DocumentElement::text("Text in 1.1.3")])
                );
            } else {
                panic!("expected Section 1.1");
            }

            // Section 1.2 (Level 2)
            assert_eq!(
                children[2],
                DocumentElement::section(2, "Section 1.2", vec![DocumentElement::text("Text in 1.2")])
            );
        } else {
            panic!("expected Section 1");
        }

        // Section 2 (Level 1)
        assert_eq!(
            doc.body[2],
            DocumentElement::section(1, "Section 2", vec![DocumentElement::text("Text in 2")])
        );
    }

    #[test]
    fn test_parse_d2f_markdown_section_with_mixed_elements() {
        let md = "---\ntitle: \"Mixed Elements\"\n---\n# Main Heading\nParagraph line\n- [ ] Task 1\n- Bullet 1\n>! Important note\n```rust\nfn main() {}\n```\n| H1 | H2 |\n| --- | --- |\n| D1 | D2 |\n:::variables\n| KEY | VAL |\n| --- | --- |\n| PORT | 8080 |\n:::\n![Diagram](arch.png)";
        let doc = parse_d2f_markdown(md).unwrap();

        assert_eq!(doc.body.len(), 1);
        if let DocumentElement::Section { level, title, children } = &doc.body[0] {
            assert_eq!(*level, 1);
            assert_eq!(title, "Main Heading");
            assert_eq!(children.len(), 8);
            assert_eq!(children[0], DocumentElement::text("Paragraph line"));
            assert_eq!(
                children[1],
                DocumentElement::check_box_item(0, false, DocumentElement::text("Task 1"))
            );
            assert_eq!(
                children[2],
                DocumentElement::bullet_list_item(0, DocumentElement::text("Bullet 1"))
            );
            assert_eq!(
                children[3],
                DocumentElement::shoutout(ShoutoutElementKind::Important, "Important note")
            );
            assert_eq!(
                children[4],
                DocumentElement::code_block(Some("rust"), "fn main() {}")
            );
            assert_eq!(
                children[5],
                DocumentElement::table(
                    vec![TableAlignment::None, TableAlignment::None],
                    vec![vec!["H1".into(), "H2".into()], vec!["D1".into(), "D2".into()]]
                )
            );
            assert_eq!(
                children[6],
                DocumentElement::block_directive(
                    "variables",
                    vec![DocumentElement::table(
                        vec![TableAlignment::None, TableAlignment::None],
                        vec![vec!["KEY".into(), "VAL".into()], vec!["PORT".into(), "8080".into()]]
                    )]
                )
            );
            assert_eq!(
                children[7],
                DocumentElement::image("Diagram", "arch.png")
            );
        } else {
            panic!("expected Section");
        }
    }
}



