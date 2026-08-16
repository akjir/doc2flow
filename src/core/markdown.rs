//! Markdown parser without external dependencies.

use std::collections::HashMap;
use std::mem;

use crate::core::document::{
    Document, DocumentElement, DocumentParameters, ShoutoutElementKind, TableAlignment,
};
use crate::core::error::{DiagnosticError, build_caret_annotation};
use crate::core::{Error, Result};

/// Block directive names allowed before the first level-1 heading.
const ALLOWED_PRE_H1_DIRECTIVES: &[&str] = &["variables"];

/// Active list item held on the hierarchical list tracking stack.
#[derive(Debug)]
struct ActiveListItem {
    /// Element data model payload.
    element: DocumentElement,
    /// Indentation spaces offset.
    indent_spaces: usize,
    /// Last child ordered list numerical position.
    last_child_ordered_position: Option<usize>,
}

/// State tracker for filtering HTML comments across lines.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
struct CommentFilterState {
    /// Indicates whether parsing is currently within a multi-line HTML comment block.
    in_comment: bool,
}

impl CommentFilterState {
    /// Creates a new comment filter in standard mode.
    #[must_use]
    fn new() -> Self {
        Self { in_comment: false }
    }

    /// Filters HTML comments from a single line.
    ///
    /// Returns `Some(processed_line)` or `None` if the line was fully swallowed by comments.
    fn process_line<'a>(&mut self, mut current: &'a str, buf: &'a mut String) -> Option<&'a str> {
        if self.in_comment {
            let end_idx = current.find("-->")?;
            self.in_comment = false;
            current = &current[end_idx + 3..];
            if current.trim().is_empty() {
                return None;
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
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum FrontmatterPhase {
    /// Seeking opening frontmatter delimiter `---`.
    SeekingStart,
    /// Inside frontmatter block, collecting content until closing `---`.
    Inside,
    /// Frontmatter parsed, processing standard document body.
    Complete,
}

/// Intermediate list item classification before AST node construction.
#[derive(Debug)]
enum ListItemKind<'a> {
    /// Bullet list item with raw inner text/image slice.
    Bullet(&'a str),
    /// Checkbox task item with checked state and raw inner slice.
    CheckBox(bool, &'a str),
    /// Ordered list item with raw inner text/image slice.
    Ordered(&'a str),
}

/// Hierarchical list parsing state tracking active parent list items and sequential numbering.
#[derive(Debug, Default)]
struct ListState {
    /// Next sequential position counter for root-level ordered lists.
    root_ordered_position: Option<usize>,
    /// Stack of currently open parent list items.
    stack: Vec<ActiveListItem>,
}

impl ListState {
    /// Creates a new empty list parsing state.
    #[must_use]
    const fn new() -> Self {
        Self {
            root_ordered_position: None,
            stack: Vec::new(),
        }
    }

    /// Flushes all active list items on the stack into their parent elements or target container.
    fn flush(
        &mut self,
        doc: &mut Document,
        section_stack: &mut [DocumentElement],
        in_block: bool,
        block_children: &mut Vec<DocumentElement>,
    ) {
        while let Some(popped) = self.stack.pop() {
            if let Some(parent) = self.stack.last_mut() {
                let _ = parent.element.push_child(popped.element);
            } else {
                push_element(doc, section_stack, in_block, block_children, popped.element);
            }
        }
    }

    /// Processes an incoming list item line into the active list hierarchy.
    fn process_item(
        &mut self,
        doc: &mut Document,
        section_stack: &mut [DocumentElement],
        in_block: bool,
        block_children: &mut Vec<DocumentElement>,
        indent_spaces: usize,
        kind: ListItemKind,
    ) {
        while let Some(top) = self.stack.last() {
            if top.indent_spaces >= indent_spaces {
                let popped = self.stack.pop().unwrap();
                if let Some(parent) = self.stack.last_mut() {
                    let _ = parent.element.push_child(popped.element);
                } else {
                    push_element(doc, section_stack, in_block, block_children, popped.element);
                }
            } else {
                break;
            }
        }

        if let Some(parent) = self.stack.last_mut() {
            let (elem, next_ordered) = match kind {
                ListItemKind::Bullet(content) => (DocumentElement::bullet_list_item(content), None),
                ListItemKind::CheckBox(checked, content) => {
                    (DocumentElement::check_box_item(checked, content), None)
                }
                ListItemKind::Ordered(content) => {
                    let next_pos = parent.last_child_ordered_position.map_or(1, |p| p + 1);
                    (
                        DocumentElement::ordered_list_item(next_pos, content),
                        Some(next_pos),
                    )
                }
            };
            parent.last_child_ordered_position = next_ordered;
            self.stack.push(ActiveListItem {
                element: elem,
                indent_spaces,
                last_child_ordered_position: None,
            });
        } else {
            let elem = match kind {
                ListItemKind::Bullet(content) => {
                    self.root_ordered_position = None;
                    DocumentElement::bullet_list_item(content)
                }
                ListItemKind::CheckBox(checked, content) => {
                    self.root_ordered_position = None;
                    DocumentElement::check_box_item(checked, content)
                }
                ListItemKind::Ordered(content) => {
                    let next_pos = self.root_ordered_position.map_or(1, |p| p + 1);
                    self.root_ordered_position = Some(next_pos);
                    DocumentElement::ordered_list_item(next_pos, content)
                }
            };
            self.stack.push(ActiveListItem {
                element: elem,
                indent_spaces,
                last_child_ordered_position: None,
            });
        }
    }
}

/// Constructs a standardized diagnostic error for content defined before the first level-1 heading.
fn build_content_before_h1_err(line_number: usize, line_snippet: &str) -> Error {
    let carets = build_caret_annotation(1, line_snippet.len().max(1), line_snippet.len().max(1));
    DiagnosticError {
        message: "content defined before first level-1 heading".into(),
        file_path: "<input>".into(),
        line_number,
        col_number: 1,
        line_snippet: line_snippet.into(),
        annotation_carets: carets,
        annotation_text: "unexpected content before first '# Heading'".into(),
        help_text: "document content must begin with a level-1 heading '# Heading' after frontmatter (only ':::variables' is permitted before it).".into(),
    }
    .into()
}

/// Constructs a standardized diagnostic error for block directives not allowed before the first level-1 heading.
fn build_disallowed_pre_h1_block_directive_err(
    line_number: usize,
    line_snippet: &str,
    name: &str,
) -> Error {
    let carets = build_caret_annotation(1, line_snippet.len().max(1), line_snippet.len().max(1));
    DiagnosticError {
        message: format!(
            "block directive ':::{name}' is not permitted before first level-1 heading"
        )
        .into(),
        file_path: "<input>".into(),
        line_number,
        col_number: 1,
        line_snippet: line_snippet.into(),
        annotation_carets: carets,
        annotation_text: "directive not allowed before first '# Heading'".into(),
        help_text: "only ':::variables' is permitted before the first level-1 heading '# Heading'."
            .into(),
    }
    .into()
}

/// Constructs a standardized diagnostic error for invalid block directive names.
fn build_invalid_block_directive_name_err(line_number: usize, line_snippet: &str) -> Error {
    let carets = build_caret_annotation(1, line_snippet.len().max(1), line_snippet.len().max(1));
    DiagnosticError {
        message: "invalid block directive name".into(),
        file_path: "<input>".into(),
        line_number,
        col_number: 1,
        line_snippet: line_snippet.into(),
        annotation_carets: carets,
        annotation_text: "directive name must contain only alphanumeric characters (a-z, 0-9)"
            .into(),
        help_text: "use only alphanumeric characters for directive names, e.g. ':::variables'."
            .into(),
    }
    .into()
}

/// Constructs a standardized diagnostic error for malformed variables block directives.
fn build_invalid_variables_block_directive_err(line_number: usize, line_snippet: &str) -> Error {
    let carets = build_caret_annotation(1, line_snippet.len().max(3), line_snippet.len().max(3));
    DiagnosticError {
        message: "block directive ':::variables' must contain only a single table".into(),
        file_path: "<input>".into(),
        line_number,
        col_number: 1,
        line_snippet: line_snippet.into(),
        annotation_carets: carets,
        annotation_text: "directive must contain only a table and no other content".into(),
        help_text: "define exactly one markdown table inside the ':::variables' block directive."
            .into(),
    }
    .into()
}

/// Constructs a standardized diagnostic error for missing block directive names.
fn build_missing_block_directive_name_err(line_number: usize, line_snippet: &str) -> Error {
    let carets = build_caret_annotation(1, line_snippet.len().max(1), line_snippet.len().max(1));
    DiagnosticError {
        message: "missing block directive name".into(),
        file_path: "<input>".into(),
        line_number,
        col_number: 1,
        line_snippet: line_snippet.into(),
        annotation_carets: carets,
        annotation_text: "expected directive name after colons".into(),
        help_text: "provide an alphanumeric name for the block directive, e.g. ':::variables'."
            .into(),
    }
    .into()
}

/// Constructs a standardized diagnostic error for missing frontmatter delimiters.
fn build_missing_frontmatter_err(line_number: usize, line_snippet: &str) -> Error {
    let carets = build_caret_annotation(1, line_snippet.len().max(1), line_snippet.len().max(1));
    DiagnosticError {
        message: "missing frontmatter delimiter '---'".into(),
        file_path: "<input>".into(),
        line_number,
        col_number: 1,
        line_snippet: line_snippet.into(),
        annotation_carets: carets,
        annotation_text: "expected '---' to begin frontmatter".into(),
        help_text: "document must begin with frontmatter enclosed by '---' delimiters.".into(),
    }
    .into()
}

/// Constructs a standardized diagnostic error for missing mandatory level-1 heading.
fn build_missing_h1_err(line_number: usize, line_snippet: &str) -> Error {
    let carets = build_caret_annotation(1, line_snippet.len().max(1), line_snippet.len().max(1));
    DiagnosticError {
        message: "missing level-1 heading '# Heading'".into(),
        file_path: "<input>".into(),
        line_number,
        col_number: 1,
        line_snippet: line_snippet.into(),
        annotation_carets: carets,
        annotation_text: "document must contain at least one level-1 heading".into(),
        help_text: "add a level-1 heading '# Your Title' after the frontmatter.".into(),
    }
    .into()
}

/// Constructs a standardized diagnostic error for nested block directives.
fn build_nested_block_directive_err(line_number: usize, line_snippet: &str) -> Error {
    let carets = build_caret_annotation(1, line_snippet.len().max(1), line_snippet.len().max(1));
    DiagnosticError {
        message: "nested block directives are not supported".into(),
        file_path: "<input>".into(),
        line_number,
        col_number: 1,
        line_snippet: line_snippet.into(),
        annotation_carets: carets,
        annotation_text: "nested block directive opening found here".into(),
        help_text: "close the active block directive with ':::' before starting a new one.".into(),
    }
    .into()
}

/// Constructs a standardized diagnostic error for unclosed block directives.
fn build_unclosed_block_directive_err(line_number: usize, line_snippet: &str) -> Error {
    let carets = build_caret_annotation(1, line_snippet.len().max(3), line_snippet.len().max(3));
    DiagnosticError {
        message: "unclosed block directive".into(),
        file_path: "<input>".into(),
        line_number,
        col_number: 1,
        line_snippet: line_snippet.into(),
        annotation_carets: carets,
        annotation_text: "block directive starting here is never closed".into(),
        help_text: "close the block directive with a closing ':::' line.".into(),
    }
    .into()
}

/// Classifies a non-table line and appends it to target buffer or document body.
fn classify_and_push_line(
    doc: &mut Document,
    section_stack: &mut Vec<DocumentElement>,
    in_block: bool,
    block_children: &mut Vec<DocumentElement>,
    effective_line: &str,
    list_state: &mut ListState,
) {
    let trimmed = effective_line.trim();
    if trimmed.is_empty() {
        list_state.flush(doc, section_stack, in_block, block_children);
        list_state.root_ordered_position = None;
        return;
    }

    if let Some((spaces, checked, content)) = parse_check_box_item(effective_line) {
        list_state.process_item(
            doc,
            section_stack,
            in_block,
            block_children,
            spaces,
            ListItemKind::CheckBox(checked, content),
        );
    } else if let Some((spaces, content)) = parse_bullet_list_item(effective_line) {
        list_state.process_item(
            doc,
            section_stack,
            in_block,
            block_children,
            spaces,
            ListItemKind::Bullet(content),
        );
    } else if let Some((spaces, _parsed_num, content)) =
        parse_ordered_list_candidate(effective_line)
    {
        list_state.process_item(
            doc,
            section_stack,
            in_block,
            block_children,
            spaces,
            ListItemKind::Ordered(content),
        );
    } else {
        list_state.flush(doc, section_stack, in_block, block_children);
        list_state.root_ordered_position = None;

        if is_horizontal_rule(trimmed) {
            push_element(
                doc,
                section_stack,
                in_block,
                block_children,
                DocumentElement::horizontal_rule(),
            );
        } else if let Some((level, title)) = parse_heading_line(effective_line) {
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
        } else if let Some((alt, url)) = parse_image(trimmed) {
            push_element(
                doc,
                section_stack,
                in_block,
                block_children,
                DocumentElement::image(alt, url),
            );
        } else if trimmed.starts_with('>') {
            let (shoutout_kind, content) = parse_shoutout_line(trimmed);
            push_element(
                doc,
                section_stack,
                in_block,
                block_children,
                DocumentElement::shoutout(shoutout_kind, content),
            );
        } else if is_plain_text(trimmed) {
            push_element(
                doc,
                section_stack,
                in_block,
                block_children,
                DocumentElement::text(trimmed),
            );
        } else {
            push_element(
                doc,
                section_stack,
                in_block,
                block_children,
                DocumentElement::unknown(trimmed),
            );
        }
    }
}

/// Unwinds all active sections on the stack into their parent containers or document body.
fn flush_section_stack(doc: &mut Document, section_stack: &mut Vec<DocumentElement>) {
    while let Some(popped) = section_stack.pop() {
        if let Some(parent) = section_stack.last_mut() {
            let _ = parent.push_child(popped);
        } else {
            doc.push_body(popped);
        }
    }
}

/// Returns the heading level if the element is a Section.
fn get_section_level(elem: &DocumentElement) -> usize {
    match elem {
        DocumentElement::Section { level, .. } => *level,
        _ => 0,
    }
}

/// Checks whether a block directive name is permitted before the first level-1 heading.
fn is_allowed_pre_h1_directive(name: &str) -> bool {
    ALLOWED_PRE_H1_DIRECTIVES.contains(&name)
}

/// Checks whether a line represents a markdown horizontal rule (`---`, `----`, etc.).
fn is_horizontal_rule(line: &str) -> bool {
    let trimmed = line.trim();
    trimmed.len() >= 3 && trimmed.bytes().all(|b| b == b'-')
}

/// Checks whether a line represents plain text.
fn is_plain_text(line: &str) -> bool {
    !line.starts_with(['#', '>', '-', '*', '`', '|'])
}

/// Parses a bullet list item into its leading spaces and inner content.
fn parse_bullet_list_item(line: &str) -> Option<(usize, &str)> {
    let leading_spaces = line.bytes().take_while(|&b| b == b' ').count();
    let rest = &line[leading_spaces..];

    let after_dash = rest.strip_prefix('-')?;
    if after_dash.is_empty() {
        Some((leading_spaces, ""))
    } else if let Some(content) = after_dash.strip_prefix(' ') {
        Some((leading_spaces, content.trim()))
    } else {
        None
    }
}

/// Parses a checkbox list item into its leading spaces, checked status, and inner content.
fn parse_check_box_item(line: &str) -> Option<(usize, bool, &str)> {
    let leading_spaces = line.bytes().take_while(|&b| b == b' ').count();
    let rest = &line[leading_spaces..];

    let after_dash = rest.strip_prefix('-')?;
    let after_space = after_dash.strip_prefix(' ')?;

    let (checked, after_box) = if let Some(after_box) = after_space.strip_prefix("[ ]") {
        (false, after_box)
    } else {
        let after_box = after_space
            .strip_prefix("[x]")
            .or_else(|| after_space.strip_prefix("[X]"))?;
        (true, after_box)
    };

    if after_box.is_empty() {
        Some((leading_spaces, checked, ""))
    } else if let Some(content) = after_box.strip_prefix(' ') {
        Some((leading_spaces, checked, content.trim()))
    } else {
        None
    }
}

/// Parses Markdown content into a structured document model.
///
/// # Errors
///
/// Returns a diagnostic error if frontmatter or level-1 heading is missing or unclosed,
/// if content is defined before the first level-1 heading without authorization,
/// or if block directives are malformed, unclosed, or nested.
pub fn parse_d2f_markdown(md_content: &str) -> Result<Document, Error> {
    let mut doc = Document::new();
    let mut frontmatter_map: HashMap<String, String> = HashMap::new();
    let mut phase = FrontmatterPhase::SeekingStart;
    let mut comment_filter = CommentFilterState::new();
    let mut list_state = ListState::new();
    let mut section_stack: Vec<DocumentElement> = Vec::new();
    let mut has_seen_first_h1 = false;

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
                doc.parameters = DocumentParameters::from(mem::take(&mut frontmatter_map));
            } else if let Some((key, val)) = line.split_once(':') {
                let key = key.trim();
                let val_trimmed = trim_matching_quotes(val);
                if !key.is_empty() {
                    frontmatter_map.insert(key.to_string(), val_trimmed.to_string());
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
                            &mut list_state,
                        );
                    }
                    list_state.flush(
                        &mut doc,
                        &mut section_stack,
                        in_block_directive,
                        &mut block_children,
                    );
                    list_state.root_ordered_position = None;

                    let colon_count = trimmed_start.bytes().take_while(|&b| b == b':').count();
                    let rest = trimmed_start[colon_count..].trim();

                    if in_block_directive {
                        if rest.is_empty() {
                            let mut children = mem::take(&mut block_children);
                            let name = mem::take(&mut block_name);
                            if name == "variables" {
                                if children.len() != 1
                                    || !matches!(&children[0], DocumentElement::Table { .. })
                                {
                                    return Err(build_invalid_variables_block_directive_err(
                                        block_start_line,
                                        block_start_snippet,
                                    ));
                                }
                                doc.header.variables = Some(children.remove(0));
                            } else {
                                push_element(
                                    &mut doc,
                                    &mut section_stack,
                                    false,
                                    &mut block_children,
                                    DocumentElement::block_directive(name, children),
                                );
                            }
                            in_block_directive = false;
                            continue;
                        } else {
                            return Err(build_nested_block_directive_err(line_no, line));
                        }
                    } else if rest.is_empty() {
                        return Err(build_missing_block_directive_name_err(line_no, line));
                    } else if !rest.chars().all(|c| c.is_ascii_alphanumeric()) {
                        return Err(build_invalid_block_directive_name_err(line_no, line));
                    } else if !has_seen_first_h1 && !is_allowed_pre_h1_directive(rest) {
                        return Err(build_disallowed_pre_h1_block_directive_err(
                            line_no, line, rest,
                        ));
                    } else {
                        in_block_directive = true;
                        block_name = rest.to_string();
                        block_start_line = line_no;
                        block_start_snippet = line;
                        block_children.clear();
                        continue;
                    }
                }

                if !in_block_directive && !has_seen_first_h1 {
                    let trimmed = effective_line.trim();
                    if trimmed.is_empty() {
                        continue;
                    }
                    if let Some((1, title)) = parse_heading_line(effective_line) {
                        has_seen_first_h1 = true;
                        push_section(&mut doc, &mut section_stack, 1, title);
                        continue;
                    }
                    return Err(build_content_before_h1_err(line_no, line));
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
                            &mut list_state,
                        );
                    }
                    list_state.flush(
                        &mut doc,
                        &mut section_stack,
                        in_block_directive,
                        &mut block_children,
                    );
                    list_state.root_ordered_position = None;
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
                            &mut list_state,
                        );
                    }
                }

                let trimmed = effective_line.trim();
                if trimmed.is_empty() {
                    list_state.flush(
                        &mut doc,
                        &mut section_stack,
                        in_block_directive,
                        &mut block_children,
                    );
                    list_state.root_ordered_position = None;
                    continue;
                }

                if trimmed.contains('|')
                    && !trimmed.starts_with(['#', '>'])
                    && !trimmed.starts_with("```")
                    && !trimmed.starts_with(":::")
                    && !trimmed.starts_with("- ")
                    && !trimmed.starts_with("- [")
                {
                    list_state.flush(
                        &mut doc,
                        &mut section_stack,
                        in_block_directive,
                        &mut block_children,
                    );
                    list_state.root_ordered_position = None;
                    pending_table_header = Some(effective_line.to_string());
                    continue;
                }

                classify_and_push_line(
                    &mut doc,
                    &mut section_stack,
                    in_block_directive,
                    &mut block_children,
                    effective_line,
                    &mut list_state,
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
            &mut list_state,
        );
    }

    list_state.flush(
        &mut doc,
        &mut section_stack,
        in_block_directive,
        &mut block_children,
    );

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
            if !has_seen_first_h1 {
                let (line_no, snippet) = last_line_info;
                return Err(build_missing_h1_err(line_no, snippet));
            }
            flush_section_stack(&mut doc, &mut section_stack);
            Ok(doc)
        }
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
    let level = match hash_count {
        1 => 1,
        2 => 2,
        _ => 3,
    };
    if rest.is_empty() {
        Some((level, ""))
    } else if rest.starts_with([' ', '\t']) {
        Some((level, rest.trim()))
    } else {
        None
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

/// Parses an ordered list item candidate into its leading spaces, leading number, and inner content.
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

    if after_dot.is_empty() {
        Some((leading_spaces, parsed_num, ""))
    } else if let Some(content) = after_dot.strip_prefix(' ') {
        Some((leading_spaces, parsed_num, content.trim()))
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

    if inner.is_empty() || !inner.bytes().all(|b| b == b'-') {
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

    let mut alignments = Vec::with_capacity(inner.matches('|').count() + 1);
    for raw_cell in inner.split('|') {
        let align = parse_table_alignment(raw_cell)?;
        alignments.push(align);
    }

    if alignments.is_empty() {
        None
    } else {
        Some(alignments)
    }
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

    let mut cells = Vec::with_capacity(inner.matches('|').count() + 1);
    let mut current_cell = String::with_capacity(32);
    let mut chars = inner.chars().peekable();

    while let Some(ch) = chars.next() {
        if ch == '\\' {
            if chars.peek() == Some(&'|') {
                current_cell.push('|');
                chars.next();
                continue;
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
        let _ = active_section.push_child(elem);
    } else {
        doc.push_body(elem);
    }
}

/// Unwinds closed sections from the stack and adds the new section to the hierarchy.
fn push_section(
    doc: &mut Document,
    section_stack: &mut Vec<DocumentElement>,
    level: usize,
    title: &str,
) {
    let min_unwind_level = if level <= 2 { 1 } else { level };
    while let Some(top) = section_stack.last() {
        if get_section_level(top) >= min_unwind_level {
            let popped = section_stack.pop().unwrap();
            if let Some(parent) = section_stack.last_mut() {
                let _ = parent.push_child(popped);
            } else {
                doc.push_body(popped);
            }
        } else {
            break;
        }
    }
    section_stack.push(DocumentElement::section(level, title, Vec::new()));
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_d2f_markdown_valid_frontmatter_and_elements() {
        let md = "---\ntitle: \"Test Title\"\nversion: \"1.0.0\"\n---\n# Heading 1\n\nThis is plain text paragraph.\n\n> Callout note\nAnother text.";
        let doc = parse_d2f_markdown(md).unwrap();

        assert_eq!(doc.parameters.title, "Test Title");
        assert_eq!(doc.parameters.version, "1.0.0");
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
    fn test_parse_d2f_markdown_full_frontmatter_options() {
        let md = r#"---
title: "Full Spec"
subtitle: "Sub Spec"
date: "2026-08-15"
version: "2.0.0"
language: "de"
logo: "images/logo.svg"
header: "flex"
numbered_sections: false
author: "Admin"
---
# Main Section
Body text
"#;
        let doc = parse_d2f_markdown(md).unwrap();
        assert_eq!(doc.parameters.title, "Full Spec");
        assert_eq!(doc.parameters.subtitle, "Sub Spec");
        assert_eq!(doc.parameters.date, "2026-08-15");
        assert_eq!(doc.parameters.version, "2.0.0");
        assert_eq!(doc.parameters.language, "de");
        assert_eq!(doc.parameters.logo, "images/logo.svg");
        assert_eq!(doc.parameters.header, "flex");
        assert!(!doc.parameters.numbered_sections);
        assert_eq!(
            doc.parameters.variables.get("author").map(|s| s.as_str()),
            Some("Admin")
        );
        assert_eq!(doc.parameters.get_variable("author"), Some("Admin"));
    }

    #[test]
    fn test_parse_d2f_markdown_comments_before_frontmatter() {
        let md = "<!-- Comment at top -->\n\n<!--\nMultiline comment\n-->\n<!-- inline -->\n---\ntitle: \"Doc\"\n---\n# Heading 1\nVisible text";
        let doc = parse_d2f_markdown(md).unwrap();

        assert_eq!(doc.parameters.title, "Doc");
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
        let md = "---\n---\n# Title\nBody text";
        let doc = parse_d2f_markdown(md).unwrap();

        assert_eq!(doc.parameters.title, "");
        assert_eq!(doc.parameters.language, "en");
        assert!(doc.parameters.numbered_sections);
        assert!(doc.parameters.variables.is_empty());
        assert_eq!(doc.body.len(), 1);
        assert_eq!(
            doc.body[0],
            DocumentElement::section(1, "Title", vec![DocumentElement::Text("Body text".into())])
        );
    }

    #[test]
    fn test_parse_d2f_markdown_horizontal_rule_in_body() {
        let md = "---\ntitle: \"Test\"\n---\n# Main\nLine 1\n---\nLine 2";
        let doc = parse_d2f_markdown(md).unwrap();

        assert_eq!(doc.parameters.title, "Test");
        assert_eq!(doc.body.len(), 1);
        assert_eq!(
            doc.body[0],
            DocumentElement::section(
                1,
                "Main",
                vec![
                    DocumentElement::Text("Line 1".into()),
                    DocumentElement::HorizontalRule,
                    DocumentElement::Text("Line 2".into()),
                ]
            )
        );
    }

    #[test]
    fn test_parse_d2f_markdown_horizontal_rule_variations() {
        let md = "---\n---\n# Main\n---\n----\n-----\n----------\n   ---   \n   ------   ";
        let doc = parse_d2f_markdown(md).unwrap();

        assert_eq!(doc.body.len(), 1);
        match &doc.body[0] {
            DocumentElement::Section { children, .. } => {
                assert_eq!(children.len(), 6);
                for elem in children {
                    assert_eq!(*elem, DocumentElement::HorizontalRule);
                }
            }
            other => panic!("expected section, got {other:?}"),
        }
    }

    #[test]
    fn test_parse_d2f_markdown_horizontal_rule_in_section() {
        let md = "---\n---\n# Main\n## Section 1\nText before\n---\nText after\n";
        let doc = parse_d2f_markdown(md).unwrap();

        assert_eq!(doc.body.len(), 2);
        match &doc.body[1] {
            DocumentElement::Section {
                children,
                level,
                title,
            } => {
                assert_eq!(*level, 2);
                assert_eq!(title, "Section 1");
                assert_eq!(children.len(), 3);
                assert_eq!(children[0], DocumentElement::Text("Text before".into()));
                assert_eq!(children[1], DocumentElement::HorizontalRule);
                assert_eq!(children[2], DocumentElement::Text("Text after".into()));
            }
            other => panic!("expected section, got {other:?}"),
        }
    }

    #[test]
    fn test_is_horizontal_rule_helper() {
        assert!(is_horizontal_rule("---"));
        assert!(is_horizontal_rule("----"));
        assert!(is_horizontal_rule("-----"));
        assert!(is_horizontal_rule("----------"));
        assert!(is_horizontal_rule("   ---   "));
        assert!(is_horizontal_rule("  -----  "));

        assert!(!is_horizontal_rule("--"));
        assert!(!is_horizontal_rule("-"));
        assert!(!is_horizontal_rule(""));
        assert!(!is_horizontal_rule("--- text"));
        assert!(!is_horizontal_rule("---123"));
        assert!(!is_horizontal_rule("***"));
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
    fn test_parse_d2f_markdown_error_missing_h1_after_frontmatter() {
        let md = "---\ntitle: \"No H1 Doc\"\n---";
        let err = parse_d2f_markdown(md).unwrap_err();
        let err_str = err.to_string();

        assert!(err_str.contains("error: missing level-1 heading '# Heading'"));
        assert!(err_str.contains("document must contain at least one level-1 heading"));
    }

    #[test]
    fn test_parse_d2f_markdown_error_missing_h1_only_variables_block() {
        let md = "---\ntitle: \"Only Variables\"\n---\n:::variables\n| A | B |\n| --- | --- |\n| 1 | 2 |\n:::";
        let err = parse_d2f_markdown(md).unwrap_err();
        let err_str = err.to_string();

        assert!(err_str.contains("error: missing level-1 heading '# Heading'"));
    }

    #[test]
    fn test_parse_d2f_markdown_error_content_before_first_h1_text() {
        let md = "---\ntitle: \"Text Before H1\"\n---\nSome text before heading\n# Main Heading";
        let err = parse_d2f_markdown(md).unwrap_err();
        let err_str = err.to_string();

        assert!(err_str.contains("error: content defined before first level-1 heading"));
        assert!(err_str.contains("4 | Some text before heading"));
        assert!(err_str.contains("unexpected content before first '# Heading'"));
    }

    #[test]
    fn test_parse_d2f_markdown_error_content_before_first_h1_h2() {
        let md = "---\ntitle: \"H2 Before H1\"\n---\n## Subheading First\n# Main Heading";
        let err = parse_d2f_markdown(md).unwrap_err();
        let err_str = err.to_string();

        assert!(err_str.contains("error: content defined before first level-1 heading"));
        assert!(err_str.contains("4 | ## Subheading First"));
    }

    #[test]
    fn test_parse_d2f_markdown_error_content_before_first_h1_table() {
        let md = "---\ntitle: \"Table Before H1\"\n---\n| Col A | Col B |\n| --- | --- |\n| A1 | B1 |\n# Main Heading";
        let err = parse_d2f_markdown(md).unwrap_err();
        let err_str = err.to_string();

        assert!(err_str.contains("error: content defined before first level-1 heading"));
        assert!(err_str.contains("4 | | Col A | Col B |"));
    }

    #[test]
    fn test_parse_d2f_markdown_error_content_before_first_h1_code_block() {
        let md = "---\ntitle: \"Code Before H1\"\n---\n```bash\necho 1\n```\n# Main Heading";
        let err = parse_d2f_markdown(md).unwrap_err();
        let err_str = err.to_string();

        assert!(err_str.contains("error: content defined before first level-1 heading"));
        assert!(err_str.contains("4 | ```bash"));
    }

    #[test]
    fn test_parse_d2f_markdown_error_content_before_first_h1_list() {
        let md = "---\ntitle: \"List Before H1\"\n---\n- Bullet item\n# Main Heading";
        let err = parse_d2f_markdown(md).unwrap_err();
        let err_str = err.to_string();

        assert!(err_str.contains("error: content defined before first level-1 heading"));
        assert!(err_str.contains("4 | - Bullet item"));
    }

    #[test]
    fn test_parse_d2f_markdown_error_content_before_first_h1_shoutout() {
        let md = "---\ntitle: \"Shoutout Before H1\"\n---\n> Note callout\n# Main Heading";
        let err = parse_d2f_markdown(md).unwrap_err();
        let err_str = err.to_string();

        assert!(err_str.contains("error: content defined before first level-1 heading"));
        assert!(err_str.contains("4 | > Note callout"));
    }

    #[test]
    fn test_parse_d2f_markdown_error_content_before_first_h1_disallowed_directive() {
        let md = "---\ntitle: \"Disallowed Directive Before H1\"\n---\n:::custom\nText\n:::\n# Main Heading";
        let err = parse_d2f_markdown(md).unwrap_err();
        let err_str = err.to_string();

        assert!(err_str.contains(
            "error: block directive ':::custom' is not permitted before first level-1 heading"
        ));
        assert!(err_str.contains("4 | :::custom"));
        assert!(err_str.contains("directive not allowed before first '# Heading'"));
    }

    #[test]
    fn test_parse_d2f_markdown_crlf_endings() {
        let md = "---\r\ntitle: \"CRLF\"\r\n---\r\n# Heading\r\nText";
        let doc = parse_d2f_markdown(md).unwrap();

        assert_eq!(doc.parameters.title, "CRLF");
        assert_eq!(doc.body.len(), 1);
        assert_eq!(
            doc.body[0],
            DocumentElement::section(1, "Heading", vec![DocumentElement::Text("Text".into())])
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
            (
                ">Note without space",
                ShoutoutElementKind::Note,
                "Note without space",
            ),
            (">   Spaced note", ShoutoutElementKind::Note, "Spaced note"),
            (">?", ShoutoutElementKind::Tip, ""),
            (">? Tip content", ShoutoutElementKind::Tip, "Tip content"),
            (
                ">?Tip without space",
                ShoutoutElementKind::Tip,
                "Tip without space",
            ),
            ("> ? Spaced tip", ShoutoutElementKind::Tip, "Spaced tip"),
            (
                ">! Important note",
                ShoutoutElementKind::Important,
                "Important note",
            ),
            (">!Important", ShoutoutElementKind::Important, "Important"),
            (
                "> ! Spaced important",
                ShoutoutElementKind::Important,
                "Spaced important",
            ),
            (
                ">!! Warning note",
                ShoutoutElementKind::Warning,
                "Warning note",
            ),
            (">!!Warning", ShoutoutElementKind::Warning, "Warning"),
            (
                "> !! Spaced warning",
                ShoutoutElementKind::Warning,
                "Spaced warning",
            ),
            (
                ">!!! Caution alert",
                ShoutoutElementKind::Caution,
                "Caution alert",
            ),
            (">!!!Caution", ShoutoutElementKind::Caution, "Caution"),
            (
                "> !!! Spaced caution",
                ShoutoutElementKind::Caution,
                "Spaced caution",
            ),
        ];

        for (input, expected_kind, expected_content) in cases {
            let (kind, content) = parse_shoutout_line(input);
            assert_eq!(kind, expected_kind, "Mismatch for input: {input}");
            assert_eq!(content, expected_content, "Mismatch for input: {input}");
        }
    }

    #[test]
    fn test_parse_d2f_markdown_shoutout_kinds() {
        let md = "---\ntitle: \"Shoutouts\"\n---\n# Main\n> Note\n>? Tip\n>! Important\n>!! Warning\n>!!! Caution";
        let doc = parse_d2f_markdown(md).unwrap();

        assert_eq!(doc.body.len(), 1);

        let expected = [
            (ShoutoutElementKind::Note, "Note"),
            (ShoutoutElementKind::Tip, "Tip"),
            (ShoutoutElementKind::Important, "Important"),
            (ShoutoutElementKind::Warning, "Warning"),
            (ShoutoutElementKind::Caution, "Caution"),
        ];

        match &doc.body[0] {
            DocumentElement::Section { children, .. } => {
                assert_eq!(children.len(), 5);
                for (i, (kind, content)) in expected.into_iter().enumerate() {
                    assert_eq!(
                        children[i],
                        DocumentElement::Shoutout {
                            kind,
                            content: content.into(),
                        }
                    );
                }
            }
            other => panic!("expected section, got {other:?}"),
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
            ("  - Depth 1 (2 spaces)", Some((2, "Depth 1 (2 spaces)"))),
            ("   - Depth 2 (3 spaces)", Some((3, "Depth 2 (3 spaces)"))),
            ("    - Depth 2 (4 spaces)", Some((4, "Depth 2 (4 spaces)"))),
            ("     - Depth 3 (5 spaces)", Some((5, "Depth 3 (5 spaces)"))),
            (
                "      - Depth 3 (6 spaces)",
                Some((6, "Depth 3 (6 spaces)")),
            ),
            (
                "       - Depth 4 (7 spaces)",
                Some((7, "Depth 4 (7 spaces)")),
            ),
            (
                "        - Depth 4 (8 spaces)",
                Some((8, "Depth 4 (8 spaces)")),
            ),
            ("  -   Spaced content   ", Some((2, "Spaced content"))),
            ("  -", Some((2, ""))),
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
        let md = "---\ntitle: \"List Doc\"\n---\n# Main\n- Level 0\n - Level 1a\n  - Level 1b\n   - Level 2a\n    - Level 2b\nRegular text\n- Another root";
        let doc = parse_d2f_markdown(md).unwrap();

        assert_eq!(doc.body.len(), 1);
        let mut expected_tree = DocumentElement::bullet_list_item("Level 0");
        let mut l1a = DocumentElement::bullet_list_item("Level 1a");
        let mut l1b = DocumentElement::bullet_list_item("Level 1b");
        let mut l2a = DocumentElement::bullet_list_item("Level 2a");
        let l2b = DocumentElement::bullet_list_item("Level 2b");
        l2a.push_child(l2b).unwrap();
        l1b.push_child(l2a).unwrap();
        l1a.push_child(l1b).unwrap();
        expected_tree.push_child(l1a).unwrap();

        match &doc.body[0] {
            DocumentElement::Section { children, .. } => {
                assert_eq!(children.len(), 3);
                assert_eq!(children[0], expected_tree);
                assert_eq!(children[1], DocumentElement::text("Regular text"));
                assert_eq!(
                    children[2],
                    DocumentElement::bullet_list_item("Another root")
                );
            }
            other => panic!("expected section, got {other:?}"),
        }
    }

    #[test]
    fn test_parse_d2f_markdown_bullet_list_with_comments() {
        let md = "---\ntitle: \"Comments in List\"\n---\n# Main\n  - Item 1 <!-- inline comment -->\n<!-- multiline\ncomment -->\n    - Item 2";
        let doc = parse_d2f_markdown(md).unwrap();

        assert_eq!(doc.body.len(), 1);
        let mut expected = DocumentElement::bullet_list_item("Item 1");
        expected
            .push_child(DocumentElement::bullet_list_item("Item 2"))
            .unwrap();
        match &doc.body[0] {
            DocumentElement::Section { children, .. } => {
                assert_eq!(children.len(), 1);
                assert_eq!(children[0], expected);
            }
            other => panic!("expected section, got {other:?}"),
        }
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
        let md = "---\ntitle: \"Code Doc\"\n---\n# Main\n```bash\n# Init script\nd2f --init\n```";
        let doc = parse_d2f_markdown(md).unwrap();

        assert_eq!(doc.body.len(), 1);
        match &doc.body[0] {
            DocumentElement::Section { children, .. } => {
                assert_eq!(children.len(), 1);
                assert_eq!(
                    children[0],
                    DocumentElement::code_block(Some("bash"), "# Init script\nd2f --init")
                );
            }
            other => panic!("expected section, got {other:?}"),
        }
    }

    #[test]
    fn test_parse_d2f_markdown_code_block_without_language() {
        let md = "---\ntitle: \"Code Doc\"\n---\n# Main\n```\n\nTest\n\nTest 2\n\n\n```";
        let doc = parse_d2f_markdown(md).unwrap();

        assert_eq!(doc.body.len(), 1);
        match &doc.body[0] {
            DocumentElement::Section { children, .. } => {
                assert_eq!(children.len(), 1);
                assert_eq!(
                    children[0],
                    DocumentElement::code_block(None::<String>, "Test\n\nTest 2")
                );
            }
            other => panic!("expected section, got {other:?}"),
        }
    }

    #[test]
    fn test_parse_d2f_markdown_code_block_preserves_comments_and_formatting() {
        let md = "---\ntitle: \"Code Doc\"\n---\n# Main\n```html\n<!-- Inside code block -->\n<div>\n  <p>Hello</p>\n</div>\n```";
        let doc = parse_d2f_markdown(md).unwrap();

        assert_eq!(doc.body.len(), 1);
        match &doc.body[0] {
            DocumentElement::Section { children, .. } => {
                assert_eq!(children.len(), 1);
                assert_eq!(
                    children[0],
                    DocumentElement::code_block(
                        Some("html"),
                        "<!-- Inside code block -->\n<div>\n  <p>Hello</p>\n</div>"
                    )
                );
            }
            other => panic!("expected section, got {other:?}"),
        }
    }

    #[test]
    fn test_parse_d2f_markdown_unclosed_code_block_at_eof() {
        let md = "---\ntitle: \"Unclosed Code\"\n---\n# Main\n```rust\nfn main() {";
        let doc = parse_d2f_markdown(md).unwrap();

        assert_eq!(doc.body.len(), 1);
        match &doc.body[0] {
            DocumentElement::Section { children, .. } => {
                assert_eq!(children.len(), 1);
                assert_eq!(
                    children[0],
                    DocumentElement::code_block(Some("rust"), "fn main() {")
                );
            }
            other => panic!("expected section, got {other:?}"),
        }
    }

    #[test]
    fn test_parse_check_box_item_depths_and_checked() {
        let cases = [
            ("- [ ] Root unchecked", Some((0, false, "Root unchecked"))),
            ("- [x] Root checked", Some((0, true, "Root checked"))),
            (
                "- [X] Root checked upper",
                Some((0, true, "Root checked upper")),
            ),
            ("- [ ]", Some((0, false, ""))),
            ("- [ ] ", Some((0, false, ""))),
            ("- [x]", Some((0, true, ""))),
            ("- [x] ", Some((0, true, ""))),
            ("- [X]", Some((0, true, ""))),
            (
                " - [ ] Depth 1 (1 space)",
                Some((1, false, "Depth 1 (1 space)")),
            ),
            (
                "  - [ ] Depth 1 (2 spaces)",
                Some((2, false, "Depth 1 (2 spaces)")),
            ),
            (
                "   - [x] Depth 2 (3 spaces)",
                Some((3, true, "Depth 2 (3 spaces)")),
            ),
            (
                "    - [x] Depth 2 (4 spaces)",
                Some((4, true, "Depth 2 (4 spaces)")),
            ),
            (
                "     - [X] Depth 3 (5 spaces)",
                Some((5, true, "Depth 3 (5 spaces)")),
            ),
            (
                "      - [X] Depth 3 (6 spaces)",
                Some((6, true, "Depth 3 (6 spaces)")),
            ),
            (
                "       - [ ] Depth 4 (7 spaces)",
                Some((7, false, "Depth 4 (7 spaces)")),
            ),
            (
                "        - [x] Depth 4 (8 spaces)",
                Some((8, true, "Depth 4 (8 spaces)")),
            ),
            (
                "  - [ ]   Spaced task content   ",
                Some((2, false, "Spaced task content")),
            ),
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
        let md = "---\ntitle: \"Checkboxes\"\n---\n# Main\n- [ ] Task 1\n - [x] Subtask 1.1\n  - [X] Subtask 1.2\n   - [ ] Sub-subtask\n- [x] Task 2\n- [ ]";
        let doc = parse_d2f_markdown(md).unwrap();

        assert_eq!(doc.body.len(), 1);
        let mut expected_task1 = DocumentElement::check_box_item(false, "Task 1");
        let mut sub1 = DocumentElement::check_box_item(true, "Subtask 1.1");
        let mut sub2 = DocumentElement::check_box_item(true, "Subtask 1.2");
        let sub3 = DocumentElement::check_box_item(false, "Sub-subtask");
        sub2.push_child(sub3).unwrap();
        sub1.push_child(sub2).unwrap();
        expected_task1.push_child(sub1).unwrap();

        match &doc.body[0] {
            DocumentElement::Section { children, .. } => {
                assert_eq!(children.len(), 3);
                assert_eq!(children[0], expected_task1);
                assert_eq!(children[1], DocumentElement::check_box_item(true, "Task 2"));
                assert_eq!(children[2], DocumentElement::check_box_item(false, ""));
            }
            other => panic!("expected section, got {other:?}"),
        }
    }

    #[test]
    fn test_parse_d2f_markdown_mixed_lists() {
        let md = "---\ntitle: \"Mixed\"\n---\n# Main\n- Bullet 1\n- [ ] Task 1\n  - Bullet nested\n  - [x] Task nested\n- Bullet 2";
        let doc = parse_d2f_markdown(md).unwrap();

        assert_eq!(doc.body.len(), 1);
        let mut expected_task = DocumentElement::check_box_item(false, "Task 1");
        expected_task
            .push_child(DocumentElement::bullet_list_item("Bullet nested"))
            .unwrap();
        expected_task
            .push_child(DocumentElement::check_box_item(true, "Task nested"))
            .unwrap();

        match &doc.body[0] {
            DocumentElement::Section { children, .. } => {
                assert_eq!(children.len(), 3);
                assert_eq!(children[0], DocumentElement::bullet_list_item("Bullet 1"));
                assert_eq!(children[1], expected_task);
                assert_eq!(children[2], DocumentElement::bullet_list_item("Bullet 2"));
            }
            other => panic!("expected section, got {other:?}"),
        }
    }

    #[test]
    fn test_parse_ordered_list_candidate_depths_and_numbers() {
        let cases = [
            ("1. Root item", Some((0, 1, "Root item"))),
            ("1.", Some((0, 1, ""))),
            ("1. ", Some((0, 1, ""))),
            (" 1. Depth 1 (1 space)", Some((1, 1, "Depth 1 (1 space)"))),
            (
                "  1. Depth 1 (2 spaces)",
                Some((2, 1, "Depth 1 (2 spaces)")),
            ),
            (
                "   2. Depth 2 (3 spaces)",
                Some((3, 2, "Depth 2 (3 spaces)")),
            ),
            (
                "    42. Depth 2 (4 spaces)",
                Some((4, 42, "Depth 2 (4 spaces)")),
            ),
            (
                "     999. Depth 3 (5 spaces)",
                Some((5, 999, "Depth 3 (5 spaces)")),
            ),
            (
                "      1. Depth 3 (6 spaces)",
                Some((6, 1, "Depth 3 (6 spaces)")),
            ),
            (
                "       5. Depth 4 (7 spaces)",
                Some((7, 5, "Depth 4 (7 spaces)")),
            ),
            (
                "        10. Depth 4 (8 spaces)",
                Some((8, 10, "Depth 4 (8 spaces)")),
            ),
            ("  1.   Spaced content   ", Some((2, 1, "Spaced content"))),
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
        let md = "---\ntitle: \"Ordered Lists\"\n---\n# Main\n1. First\n2. Second\n3. Third";
        let doc = parse_d2f_markdown(md).unwrap();

        assert_eq!(doc.body.len(), 1);
        match &doc.body[0] {
            DocumentElement::Section { children, .. } => {
                assert_eq!(children.len(), 3);
                assert_eq!(children[0], DocumentElement::ordered_list_item(1, "First"));
                assert_eq!(children[1], DocumentElement::ordered_list_item(2, "Second"));
                assert_eq!(children[2], DocumentElement::ordered_list_item(3, "Third"));
            }
            other => panic!("expected section, got {other:?}"),
        }
    }

    #[test]
    fn test_parse_d2f_markdown_ordered_lists_arbitrary_numbers() {
        let md1 = "---\ntitle: \"Repeated 1s\"\n---\n# Main\n1. Apple\n1. Banana\n1. Cherry";
        let doc1 = parse_d2f_markdown(md1).unwrap();
        assert_eq!(doc1.body.len(), 1);
        match &doc1.body[0] {
            DocumentElement::Section { children, .. } => {
                assert_eq!(children.len(), 3);
                assert_eq!(children[0], DocumentElement::ordered_list_item(1, "Apple"));
                assert_eq!(children[1], DocumentElement::ordered_list_item(2, "Banana"));
                assert_eq!(children[2], DocumentElement::ordered_list_item(3, "Cherry"));
            }
            other => panic!("expected section, got {other:?}"),
        }

        let md2 = "---\ntitle: \"Random Numbers\"\n---\n# Main\n1. Red\n5. Green\n99. Blue";
        let doc2 = parse_d2f_markdown(md2).unwrap();
        assert_eq!(doc2.body.len(), 1);
        match &doc2.body[0] {
            DocumentElement::Section { children, .. } => {
                assert_eq!(children.len(), 3);
                assert_eq!(children[0], DocumentElement::ordered_list_item(1, "Red"));
                assert_eq!(children[1], DocumentElement::ordered_list_item(2, "Green"));
                assert_eq!(children[2], DocumentElement::ordered_list_item(3, "Blue"));
            }
            other => panic!("expected section, got {other:?}"),
        }
    }

    #[test]
    fn test_parse_d2f_markdown_ordered_lists_nested_and_returns() {
        let md = "---\ntitle: \"Nested\"\n---\n# Main\n1. L0_1\n  1. L1_1\n  2. L1_2\n    1. L2_1\n  3. L1_3\n2. L0_2";
        let doc = parse_d2f_markdown(md).unwrap();

        assert_eq!(doc.body.len(), 1);
        let mut expected_l0_1 = DocumentElement::ordered_list_item(1, "L0_1");
        let l1_1 = DocumentElement::ordered_list_item(1, "L1_1");
        let mut l1_2 = DocumentElement::ordered_list_item(2, "L1_2");
        let l2_1 = DocumentElement::ordered_list_item(1, "L2_1");
        let l1_3 = DocumentElement::ordered_list_item(3, "L1_3");
        l1_2.push_child(l2_1).unwrap();
        expected_l0_1.push_child(l1_1).unwrap();
        expected_l0_1.push_child(l1_2).unwrap();
        expected_l0_1.push_child(l1_3).unwrap();

        match &doc.body[0] {
            DocumentElement::Section { children, .. } => {
                assert_eq!(children.len(), 2);
                assert_eq!(children[0], expected_l0_1);
                assert_eq!(children[1], DocumentElement::ordered_list_item(2, "L0_2"));
            }
            other => panic!("expected section, got {other:?}"),
        }
    }

    #[test]
    fn test_parse_d2f_markdown_ordered_lists_new_depth_arbitrary_number() {
        let md = "---\ntitle: \"New Depth Start\"\n---\n# Main\n1. Root 1\n  7. Subitem 1\n  8. Subitem 2\n2. Root 2";
        let doc = parse_d2f_markdown(md).unwrap();

        assert_eq!(doc.body.len(), 1);
        let mut expected_root1 = DocumentElement::ordered_list_item(1, "Root 1");
        expected_root1
            .push_child(DocumentElement::ordered_list_item(1, "Subitem 1"))
            .unwrap();
        expected_root1
            .push_child(DocumentElement::ordered_list_item(2, "Subitem 2"))
            .unwrap();

        match &doc.body[0] {
            DocumentElement::Section { children, .. } => {
                assert_eq!(children.len(), 2);
                assert_eq!(children[0], expected_root1);
                assert_eq!(children[1], DocumentElement::ordered_list_item(2, "Root 2"));
            }
            other => panic!("expected section, got {other:?}"),
        }
    }

    #[test]
    fn test_parse_d2f_markdown_ordered_lists_interrupted_by_text() {
        let md = "---\ntitle: \"Interrupted\"\n---\n# Main\n1. Item 1\n2. Item 2\n\nParagraph text\n\n1. New list 1\n2. New list 2\n\nAnother paragraph\n\n2. Not a list";
        let doc = parse_d2f_markdown(md).unwrap();

        assert_eq!(doc.body.len(), 1);
        match &doc.body[0] {
            DocumentElement::Section { children, .. } => {
                assert_eq!(children.len(), 7);
                assert_eq!(children[0], DocumentElement::ordered_list_item(1, "Item 1"));
                assert_eq!(children[1], DocumentElement::ordered_list_item(2, "Item 2"));
                assert_eq!(children[2], DocumentElement::text("Paragraph text"));
                assert_eq!(
                    children[3],
                    DocumentElement::ordered_list_item(1, "New list 1")
                );
                assert_eq!(
                    children[4],
                    DocumentElement::ordered_list_item(2, "New list 2")
                );
                assert_eq!(children[5], DocumentElement::text("Another paragraph"));
                assert_eq!(
                    children[6],
                    DocumentElement::ordered_list_item(1, "Not a list")
                );
            }
            other => panic!("expected section, got {other:?}"),
        }
    }

    #[test]
    fn test_parse_d2f_markdown_ordered_lists_interrupted_by_other_elements() {
        let md = "---\ntitle: \"Interrupted by Elements\"\n---\n# Main\n1. Item 1\n- Bullet item\n1. Item after bullet\n- [ ] Check item\n1. Item after check\n> Note shoutout\n1. Item after shoutout\n```\ncode\n```\n1. Item after code";
        let doc = parse_d2f_markdown(md).unwrap();

        assert_eq!(doc.body.len(), 1);
        match &doc.body[0] {
            DocumentElement::Section { children, .. } => {
                assert_eq!(children.len(), 9);
                assert_eq!(children[0], DocumentElement::ordered_list_item(1, "Item 1"));
                assert_eq!(
                    children[1],
                    DocumentElement::bullet_list_item("Bullet item")
                );
                assert_eq!(
                    children[2],
                    DocumentElement::ordered_list_item(1, "Item after bullet")
                );
                assert_eq!(
                    children[3],
                    DocumentElement::check_box_item(false, "Check item")
                );
                assert_eq!(
                    children[4],
                    DocumentElement::ordered_list_item(1, "Item after check")
                );
                assert_eq!(
                    children[5],
                    DocumentElement::shoutout(ShoutoutElementKind::Note, "Note shoutout")
                );
                assert_eq!(
                    children[6],
                    DocumentElement::ordered_list_item(1, "Item after shoutout")
                );
                assert_eq!(
                    children[7],
                    DocumentElement::code_block(None::<String>, "code")
                );
                assert_eq!(
                    children[8],
                    DocumentElement::ordered_list_item(1, "Item after code")
                );
            }
            other => panic!("expected section, got {other:?}"),
        }
    }

    #[test]
    fn test_parse_d2f_markdown_ordered_lists_with_comments() {
        let md = "---\ntitle: \"Comments in Ordered List\"\n---\n# Main\n1. Item 1 <!-- inline -->\n<!-- multiline\ncomment -->\n2. Item 2\n  1. Subitem <!-- comment -->";
        let doc = parse_d2f_markdown(md).unwrap();

        assert_eq!(doc.body.len(), 1);
        let mut expected_item2 = DocumentElement::ordered_list_item(2, "Item 2");
        expected_item2
            .push_child(DocumentElement::ordered_list_item(1, "Subitem"))
            .unwrap();
        match &doc.body[0] {
            DocumentElement::Section { children, .. } => {
                assert_eq!(children.len(), 2);
                assert_eq!(children[0], DocumentElement::ordered_list_item(1, "Item 1"));
                assert_eq!(children[1], expected_item2);
            }
            other => panic!("expected section, got {other:?}"),
        }
    }

    #[test]
    fn test_parse_d2f_markdown_user_example_hierarchical_lists() {
        let md =
            "---\ntitle: \"User Example\"\n---\n# Main\n1. A\n - B\n - C\n  - D\n2. E\nText\n3. F";
        let doc = parse_d2f_markdown(md).unwrap();

        assert_eq!(doc.body.len(), 1);
        let mut a = DocumentElement::ordered_list_item(1, "A");
        let b = DocumentElement::bullet_list_item("B");
        let mut c = DocumentElement::bullet_list_item("C");
        let d = DocumentElement::bullet_list_item("D");
        c.push_child(d).unwrap();
        a.push_child(b).unwrap();
        a.push_child(c).unwrap();

        let e = DocumentElement::ordered_list_item(2, "E");
        let text = DocumentElement::text("Text");
        let f = DocumentElement::ordered_list_item(1, "F");

        match &doc.body[0] {
            DocumentElement::Section { children, .. } => {
                assert_eq!(children.len(), 4);
                assert_eq!(children[0], a);
                assert_eq!(children[1], e);
                assert_eq!(children[2], text);
                assert_eq!(children[3], f);
            }
            other => panic!("expected section, got {other:?}"),
        }
    }

    #[test]
    fn test_parse_d2f_markdown_indented_text_not_added_to_list() {
        let md = "---\ntitle: \"Indented Text Test\"\n---\n# Main\n1. Step one\n   Indented text not in list\n2. Step two";
        let doc = parse_d2f_markdown(md).unwrap();

        assert_eq!(doc.body.len(), 1);
        match &doc.body[0] {
            DocumentElement::Section { children, .. } => {
                assert_eq!(children.len(), 3);
                assert_eq!(
                    children[0],
                    DocumentElement::ordered_list_item(1, "Step one")
                );
                assert_eq!(
                    children[1],
                    DocumentElement::text("Indented text not in list")
                );
                assert_eq!(
                    children[2],
                    DocumentElement::ordered_list_item(1, "Step two")
                );
            }
            other => panic!("expected section, got {other:?}"),
        }
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
        assert_eq!(parse_table_row("| A | B | C |"), vec!["A", "B", "C"]);
        assert_eq!(parse_table_row("A | B"), vec!["A", "B"]);
        assert_eq!(parse_table_row("| A \\| B | C |"), vec!["A | B", "C"]);
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
        let md = "---\ntitle: \"Table Doc\"\n---\n# Main\n| Name | Role | Location |\n| :--- | :---: | ---: |\n| Alice | Engineer | Berlin |\n| Bob | Designer | London |";
        let doc = parse_d2f_markdown(md).unwrap();

        assert_eq!(doc.body.len(), 1);
        match &doc.body[0] {
            DocumentElement::Section { children, .. } => {
                assert_eq!(children.len(), 1);
                assert_eq!(
                    children[0],
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
            other => panic!("expected section, got {other:?}"),
        }
    }

    #[test]
    fn test_parse_d2f_markdown_table_with_escaped_pipes_and_no_outer_pipes() {
        let md = "---\ntitle: \"Escaped Pipes\"\n---\n# Main\nSyntax | Description\n---|---\n`grep \\| sort` | Pipeline\n`d2f -o file` | CLI output";
        let doc = parse_d2f_markdown(md).unwrap();

        assert_eq!(doc.body.len(), 1);
        match &doc.body[0] {
            DocumentElement::Section { children, .. } => {
                assert_eq!(children.len(), 1);
                assert_eq!(
                    children[0],
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
            other => panic!("expected section, got {other:?}"),
        }
    }

    #[test]
    fn test_parse_d2f_markdown_table_interrupted_by_blank_line() {
        let md = "---\ntitle: \"Multiple Tables\"\n---\n# Main\n| T1H1 | T1H2 |\n| --- | --- |\n| R1 | R2 |\n\n| T2H1 | T2H2 |\n| :--- | ---: |\n| V1 | V2 |";
        let doc = parse_d2f_markdown(md).unwrap();

        assert_eq!(doc.body.len(), 1);
        match &doc.body[0] {
            DocumentElement::Section { children, .. } => {
                assert_eq!(children.len(), 2);
                assert_eq!(
                    children[0],
                    DocumentElement::table(
                        vec![TableAlignment::None, TableAlignment::None],
                        vec![
                            vec!["T1H1".into(), "T1H2".into()],
                            vec!["R1".into(), "R2".into()],
                        ]
                    )
                );
                assert_eq!(
                    children[1],
                    DocumentElement::table(
                        vec![TableAlignment::Left, TableAlignment::Right],
                        vec![
                            vec!["T2H1".into(), "T2H2".into()],
                            vec!["V1".into(), "V2".into()],
                        ]
                    )
                );
            }
            other => panic!("expected section, got {other:?}"),
        }
    }

    #[test]
    fn test_parse_d2f_markdown_table_interrupted_by_heading_and_lists() {
        let md = "---\ntitle: \"Table With Other Elements\"\n---\n# First Section\n| Col A | Col B |\n| --- | --- |\n| A1 | B1 |\n# Next Section\n- Bullet item\n\n| Next Table |\n| --- |\n| Only Row |";
        let doc = parse_d2f_markdown(md).unwrap();

        assert_eq!(doc.body.len(), 2);
        assert_eq!(
            doc.body[0],
            DocumentElement::section(
                1,
                "First Section",
                vec![DocumentElement::table(
                    vec![TableAlignment::None, TableAlignment::None],
                    vec![
                        vec!["Col A".into(), "Col B".into()],
                        vec!["A1".into(), "B1".into()],
                    ]
                )]
            )
        );
        assert_eq!(
            doc.body[1],
            DocumentElement::section(
                1,
                "Next Section",
                vec![
                    DocumentElement::bullet_list_item("Bullet item"),
                    DocumentElement::table(
                        vec![TableAlignment::None],
                        vec![vec!["Next Table".into()], vec!["Only Row".into()]]
                    ),
                ]
            )
        );
    }

    #[test]
    fn test_parse_d2f_markdown_table_with_comments() {
        let md = "---\ntitle: \"Table With Comments\"\n---\n# Main\n| Col 1 <!-- inline --> | Col 2 |\n<!-- multiline\ncomment -->\n| --- | --- |\n| Val 1 | Val 2 <!-- comment --> |";
        let doc = parse_d2f_markdown(md).unwrap();

        assert_eq!(doc.body.len(), 1);
        match &doc.body[0] {
            DocumentElement::Section { children, .. } => {
                assert_eq!(children.len(), 1);
                assert_eq!(
                    children[0],
                    DocumentElement::table(
                        vec![TableAlignment::None, TableAlignment::None],
                        vec![
                            vec!["Col 1".into(), "Col 2".into()],
                            vec!["Val 1".into(), "Val 2".into()],
                        ]
                    )
                );
            }
            other => panic!("expected section, got {other:?}"),
        }
    }

    #[test]
    fn test_parse_d2f_markdown_pipe_in_text_fallback() {
        let md = "---\ntitle: \"Non Table Pipes\"\n---\n# Main\nThis is plain text with | pipe character.\nAnother regular sentence.\n\nOption A | Option B\nNot a table.";
        let doc = parse_d2f_markdown(md).unwrap();

        assert_eq!(doc.body.len(), 1);
        match &doc.body[0] {
            DocumentElement::Section { children, .. } => {
                assert_eq!(children.len(), 4);
                assert_eq!(
                    children[0],
                    DocumentElement::text("This is plain text with | pipe character.")
                );
                assert_eq!(
                    children[1],
                    DocumentElement::text("Another regular sentence.")
                );
                assert_eq!(children[2], DocumentElement::text("Option A | Option B"));
                assert_eq!(children[3], DocumentElement::text("Not a table."));
            }
            other => panic!("expected section, got {other:?}"),
        }
    }

    #[test]
    fn test_parse_d2f_markdown_table_at_eof() {
        let md =
            "---\ntitle: \"Table at EOF\"\n---\n# Main\n| H1 | H2 |\n| --- | --- |\n| D1 | D2 |";
        let doc = parse_d2f_markdown(md).unwrap();

        assert_eq!(doc.body.len(), 1);
        match &doc.body[0] {
            DocumentElement::Section { children, .. } => {
                assert_eq!(children.len(), 1);
                assert_eq!(
                    children[0],
                    DocumentElement::table(
                        vec![TableAlignment::None, TableAlignment::None],
                        vec![
                            vec!["H1".into(), "H2".into()],
                            vec!["D1".into(), "D2".into()],
                        ]
                    )
                );
            }
            other => panic!("expected section, got {other:?}"),
        }
    }

    #[test]
    fn test_parse_d2f_markdown_block_directive_variables_example() {
        let md = "---\ntitle: \"Block Directive Doc\"\n---\n:::variables\n| Variable | Value |\n| --- | --- |\n| TARGET_HOST | 192.168.1.100 |\n| SERVICE_PORT | 8080 |\n:::\n# Main Section\nContent under main";
        let doc = parse_d2f_markdown(md).unwrap();

        assert_eq!(
            doc.header.variables,
            Some(DocumentElement::table(
                vec![TableAlignment::None, TableAlignment::None],
                vec![
                    vec!["Variable".into(), "Value".into()],
                    vec!["TARGET_HOST".into(), "192.168.1.100".into()],
                    vec!["SERVICE_PORT".into(), "8080".into()],
                ]
            ))
        );
        assert_eq!(doc.body.len(), 1);
        match &doc.body[0] {
            DocumentElement::Section { title, level, .. } => {
                assert_eq!(*level, 1);
                assert_eq!(title, "Main Section");
            }
            other => panic!("Expected Section, got {other:?}"),
        }
    }

    #[test]
    fn test_parse_d2f_markdown_block_directive_variables_error_when_empty() {
        let md = "---\ntitle: \"Empty Variables\"\n---\n:::variables\n:::\n# Main Section\nContent";
        let err = parse_d2f_markdown(md).unwrap_err();
        let err_str = err.to_string();
        assert!(
            err_str.contains("block directive ':::variables' must contain only a single table")
        );
    }

    #[test]
    fn test_parse_d2f_markdown_block_directive_variables_error_with_text() {
        let md = "---\ntitle: \"Invalid Variables\"\n---\n:::variables\n| Variable | Value |\n| --- | --- |\n| TARGET_HOST | 192.168.1.100 |\nExtra text\n:::\n# Main\nContent";
        let err = parse_d2f_markdown(md).unwrap_err();
        let err_str = err.to_string();
        assert!(
            err_str.contains("block directive ':::variables' must contain only a single table")
        );
    }

    #[test]
    fn test_parse_d2f_markdown_block_directive_variables_error_with_non_table() {
        let md = "---\ntitle: \"Invalid Variables\"\n---\n:::variables\n- [x] Task\n:::\n# Main\nContent";
        let err = parse_d2f_markdown(md).unwrap_err();
        let err_str = err.to_string();
        assert!(
            err_str.contains("block directive ':::variables' must contain only a single table")
        );
    }

    #[test]
    fn test_parse_d2f_markdown_block_directive_variables_error_with_multiple_tables() {
        let md = "---\ntitle: \"Invalid Variables\"\n---\n:::variables\n| A | B |\n| --- | --- |\n| 1 | 2 |\n\n| C | D |\n| --- | --- |\n| 3 | 4 |\n:::\n# Main\nContent";
        let err = parse_d2f_markdown(md).unwrap_err();
        let err_str = err.to_string();
        assert!(
            err_str.contains("block directive ':::variables' must contain only a single table")
        );
    }

    #[test]
    fn test_parse_d2f_markdown_block_directive_with_mixed_elements() {
        let md = "---\ntitle: \"Mixed Block Directive\"\n---\n# Main\n:::custom123\n- Bullet 1\n- [x] Checkbox\n1. Numbered 1\n>! Important shoutout\n```bash\necho test\n```\n:::";
        let doc = parse_d2f_markdown(md).unwrap();

        assert_eq!(doc.body.len(), 1);
        match &doc.body[0] {
            DocumentElement::Section { children, .. } => {
                assert_eq!(children.len(), 1);
                if let DocumentElement::BlockDirective { name, children } = &children[0] {
                    assert_eq!(name, "custom123");
                    assert_eq!(children.len(), 5);
                    assert_eq!(children[0], DocumentElement::bullet_list_item("Bullet 1"));
                    assert_eq!(
                        children[1],
                        DocumentElement::check_box_item(true, "Checkbox")
                    );
                    assert_eq!(
                        children[2],
                        DocumentElement::ordered_list_item(1, "Numbered 1")
                    );
                    assert_eq!(
                        children[3],
                        DocumentElement::shoutout(
                            ShoutoutElementKind::Important,
                            "Important shoutout"
                        )
                    );
                    assert_eq!(
                        children[4],
                        DocumentElement::code_block(Some("bash"), "echo test")
                    );
                } else {
                    panic!("Expected BlockDirective child");
                }
            }
            other => panic!("expected section, got {other:?}"),
        }
    }

    #[test]
    fn test_parse_d2f_markdown_block_directive_with_extra_colons() {
        let md = "---\ntitle: \"Extra Colons\"\n---\n# Main\n:::::::::config\nConfig text.\n:::::";
        let doc = parse_d2f_markdown(md).unwrap();

        assert_eq!(doc.body.len(), 1);
        match &doc.body[0] {
            DocumentElement::Section { children, .. } => {
                assert_eq!(children.len(), 1);
                assert_eq!(
                    children[0],
                    DocumentElement::block_directive(
                        "config",
                        vec![DocumentElement::text("Config text.")]
                    )
                );
            }
            other => panic!("expected section, got {other:?}"),
        }
    }

    #[test]
    fn test_parse_d2f_markdown_multiple_block_directives() {
        let md = "---\ntitle: \"Multiple Directives\"\n---\n:::variables\n| Var | Val |\n| --- | --- |\n| A | 1 |\n:::\n# Main\n:::blockA\nText A\n:::\n\nMiddle text\n\n:::blockB\nText B\n:::";
        let doc = parse_d2f_markdown(md).unwrap();

        assert_eq!(
            doc.header.variables,
            Some(DocumentElement::table(
                vec![TableAlignment::None, TableAlignment::None],
                vec![
                    vec!["Var".into(), "Val".into()],
                    vec!["A".into(), "1".into()]
                ]
            ))
        );
        assert_eq!(doc.body.len(), 1);
        match &doc.body[0] {
            DocumentElement::Section { children, .. } => {
                assert_eq!(children.len(), 3);
                assert_eq!(
                    children[0],
                    DocumentElement::block_directive(
                        "blockA",
                        vec![DocumentElement::text("Text A")]
                    )
                );
                assert_eq!(children[1], DocumentElement::text("Middle text"));
                assert_eq!(
                    children[2],
                    DocumentElement::block_directive(
                        "blockB",
                        vec![DocumentElement::text("Text B")]
                    )
                );
            }
            other => panic!("expected section, got {other:?}"),
        }
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
        let md = "---\ntitle: \"Nested Directive\"\n---\n# Main\n:::outer\nText\n:::inner\nNested text\n:::\n:::";
        let err = parse_d2f_markdown(md).unwrap_err();
        let err_str = err.to_string();
        assert!(err_str.contains("nested block directives are not supported"));
        assert!(err_str.contains("nested block directive opening found here"));
    }

    #[test]
    fn test_parse_d2f_markdown_standalone_images() {
        let md = "---\ntitle: \"Images Test\"\n---\n# Main\n![Architecture Diagram](assets/arch.png)\n![](https://example.com/logo.svg)\n![Empty URL]()\n![ With Spaces ](  https://example.com/pic.webp  )";
        let doc = parse_d2f_markdown(md).unwrap();

        assert_eq!(doc.body.len(), 1);
        match &doc.body[0] {
            DocumentElement::Section { children, .. } => {
                assert_eq!(children.len(), 4);
                assert_eq!(
                    children[0],
                    DocumentElement::image("Architecture Diagram", "assets/arch.png")
                );
                assert_eq!(
                    children[1],
                    DocumentElement::image("", "https://example.com/logo.svg")
                );
                assert_eq!(children[2], DocumentElement::image("Empty URL", ""));
                assert_eq!(
                    children[3],
                    DocumentElement::image(" With Spaces ", "https://example.com/pic.webp")
                );
            }
            other => panic!("expected section, got {other:?}"),
        }
    }

    #[test]
    fn test_parse_d2f_markdown_images_in_lists() {
        let md = "---\ntitle: \"List Images\"\n---\n# Main\n- ![Bullet Img](img/bullet.png)\n- [x] ![Check Img](img/check.png)\n- [ ] Normal text\n1. ![Ordered Img 1](img/step1.png)\n2. ![Ordered Img 2](img/step2.png)";
        let doc = parse_d2f_markdown(md).unwrap();

        assert_eq!(doc.body.len(), 1);
        match &doc.body[0] {
            DocumentElement::Section { children, .. } => {
                assert_eq!(children.len(), 5);
                assert_eq!(
                    children[0],
                    DocumentElement::bullet_list_item("![Bullet Img](img/bullet.png)")
                );
                assert_eq!(
                    children[1],
                    DocumentElement::check_box_item(true, "![Check Img](img/check.png)")
                );
                assert_eq!(
                    children[2],
                    DocumentElement::check_box_item(false, "Normal text")
                );
                assert_eq!(
                    children[3],
                    DocumentElement::ordered_list_item(1, "![Ordered Img 1](img/step1.png)")
                );
                assert_eq!(
                    children[4],
                    DocumentElement::ordered_list_item(2, "![Ordered Img 2](img/step2.png)")
                );
            }
            other => panic!("expected section, got {other:?}"),
        }
    }

    #[test]
    fn test_parse_d2f_markdown_images_in_block_directive() {
        let md = "---\ntitle: \"Block Directive Images\"\n---\n# Main\n:::gallery\n![Pic 1](pic1.jpg)\n- ![Nested Pic](pic2.jpg)\nText\n:::";
        let doc = parse_d2f_markdown(md).unwrap();

        assert_eq!(doc.body.len(), 1);
        match &doc.body[0] {
            DocumentElement::Section { children, .. } => {
                assert_eq!(children.len(), 1);
                if let DocumentElement::BlockDirective { name, children } = &children[0] {
                    assert_eq!(name, "gallery");
                    assert_eq!(children.len(), 3);
                    assert_eq!(children[0], DocumentElement::image("Pic 1", "pic1.jpg"));
                    assert_eq!(
                        children[1],
                        DocumentElement::bullet_list_item("![Nested Pic](pic2.jpg)")
                    );
                    assert_eq!(children[2], DocumentElement::text("Text"));
                } else {
                    panic!("expected BlockDirective");
                }
            }
            other => panic!("expected section, got {other:?}"),
        }
    }

    #[test]
    fn test_parse_d2f_markdown_image_edge_cases() {
        let md = "---\ntitle: \"Image Edge Cases\"\n---\n# Main\n! Not an image\n![Unclosed bracket(url)\n![Alt]no paren\n![Alt](no closing paren\nPrefix ![Alt](url) suffix";
        let doc = parse_d2f_markdown(md).unwrap();

        assert_eq!(doc.body.len(), 1);
        match &doc.body[0] {
            DocumentElement::Section { children, .. } => {
                assert_eq!(children.len(), 5);
                assert_eq!(children[0], DocumentElement::text("! Not an image"));
                assert_eq!(
                    children[1],
                    DocumentElement::text("![Unclosed bracket(url)")
                );
                assert_eq!(children[2], DocumentElement::text("![Alt]no paren"));
                assert_eq!(
                    children[3],
                    DocumentElement::text("![Alt](no closing paren")
                );
                assert_eq!(
                    children[4],
                    DocumentElement::text("Prefix ![Alt](url) suffix")
                );
            }
            other => panic!("expected section, got {other:?}"),
        }
    }

    #[test]
    fn test_parse_heading_line() {
        assert_eq!(parse_heading_line("# Title"), Some((1, "Title")));
        assert_eq!(parse_heading_line("## Subtitle"), Some((2, "Subtitle")));
        assert_eq!(parse_heading_line("### Sub-sub"), Some((3, "Sub-sub")));
        assert_eq!(
            parse_heading_line("#### Level 4 capped"),
            Some((3, "Level 4 capped"))
        );
        assert_eq!(
            parse_heading_line("########## Level 10 capped"),
            Some((3, "Level 10 capped"))
        );
        assert_eq!(parse_heading_line("#"), Some((1, "")));
        assert_eq!(parse_heading_line("##"), Some((2, "")));
        assert_eq!(parse_heading_line("###"), Some((3, "")));
        assert_eq!(parse_heading_line("   # Indented"), Some((1, "Indented")));
        assert_eq!(parse_heading_line("#hashtag"), None);
        assert_eq!(parse_heading_line("Not a heading"), None);
    }

    #[test]
    fn test_parse_d2f_markdown_section_hierarchy_and_capping() {
        let md = "---\ntitle: \"Sections Doc\"\n---\n# Section 1\nText in 1\n\n## Section 1.1\nText in 1.1\n\n### Section 1.1.1\nText in 1.1.1\n\n#### Section 1.1.2 Capped\nText in 1.1.2\n\n########## Section 1.1.3 Capped\nText in 1.1.3\n\n## Section 1.2\nText in 1.2\n\n# Section 2\nText in 2";
        let doc = parse_d2f_markdown(md).unwrap();

        assert_eq!(doc.body.len(), 4);

        // Section 1 (Level 1) - closes when Level 2 starts
        assert_eq!(
            doc.body[0],
            DocumentElement::section(1, "Section 1", vec![DocumentElement::text("Text in 1")])
        );

        // Section 1.1 (Level 2) - receives Level 3 children
        if let DocumentElement::Section {
            level,
            title,
            children,
        } = &doc.body[1]
        {
            assert_eq!(*level, 2);
            assert_eq!(title, "Section 1.1");
            assert_eq!(children.len(), 4);
            assert_eq!(children[0], DocumentElement::text("Text in 1.1"));

            // Section 1.1.1 (Level 3)
            assert_eq!(
                children[1],
                DocumentElement::section(
                    3,
                    "Section 1.1.1",
                    vec![DocumentElement::text("Text in 1.1.1")]
                )
            );
            // Section 1.1.2 (Level 3, capped from 4)
            assert_eq!(
                children[2],
                DocumentElement::section(
                    3,
                    "Section 1.1.2 Capped",
                    vec![DocumentElement::text("Text in 1.1.2")]
                )
            );
            // Section 1.1.3 (Level 3, capped from 10)
            assert_eq!(
                children[3],
                DocumentElement::section(
                    3,
                    "Section 1.1.3 Capped",
                    vec![DocumentElement::text("Text in 1.1.3")]
                )
            );
        } else {
            panic!("expected Section 1.1");
        }

        // Section 1.2 (Level 2)
        assert_eq!(
            doc.body[2],
            DocumentElement::section(2, "Section 1.2", vec![DocumentElement::text("Text in 1.2")])
        );

        // Section 2 (Level 1)
        assert_eq!(
            doc.body[3],
            DocumentElement::section(1, "Section 2", vec![DocumentElement::text("Text in 2")])
        );
    }

    #[test]
    fn test_parse_d2f_markdown_h1_and_h2_with_h3_children() {
        let md = "---\ntitle: \"H1 and H2 with H3\"\n---\n# Level 1 Heading\nL1 Body\n### Subheading in L1\nSub 1 Body\n\n## Level 2 Heading\nL2 Body\n### Subheading in L2\nSub 2 Body";
        let doc = parse_d2f_markdown(md).unwrap();

        assert_eq!(doc.body.len(), 2);

        // Level 1 Section with H3 child
        assert_eq!(
            doc.body[0],
            DocumentElement::section(
                1,
                "Level 1 Heading",
                vec![
                    DocumentElement::text("L1 Body"),
                    DocumentElement::section(
                        3,
                        "Subheading in L1",
                        vec![DocumentElement::text("Sub 1 Body")]
                    )
                ]
            )
        );

        // Level 2 Section with H3 child
        assert_eq!(
            doc.body[1],
            DocumentElement::section(
                2,
                "Level 2 Heading",
                vec![
                    DocumentElement::text("L2 Body"),
                    DocumentElement::section(
                        3,
                        "Subheading in L2",
                        vec![DocumentElement::text("Sub 2 Body")]
                    )
                ]
            )
        );
    }

    #[test]
    fn test_parse_d2f_markdown_consecutive_h3_sections() {
        let md = "---\ntitle: \"Consecutive H3\"\n---\n# Main\n### Sub 1\nText 1\n### Sub 2\nText 2\n### Sub 3\nText 3";
        let doc = parse_d2f_markdown(md).unwrap();

        assert_eq!(doc.body.len(), 1);
        assert_eq!(
            doc.body[0],
            DocumentElement::section(
                1,
                "Main",
                vec![
                    DocumentElement::section(3, "Sub 1", vec![DocumentElement::text("Text 1")]),
                    DocumentElement::section(3, "Sub 2", vec![DocumentElement::text("Text 2")]),
                    DocumentElement::section(3, "Sub 3", vec![DocumentElement::text("Text 3")]),
                ]
            )
        );
    }

    #[test]
    fn test_parse_d2f_markdown_section_with_mixed_elements() {
        let md = "---\ntitle: \"Mixed Elements\"\n---\n# Main Heading\nParagraph line\n- [ ] Task 1\n- Bullet 1\n>! Important note\n```rust\nfn main() {}\n```\n| H1 | H2 |\n| --- | --- |\n| D1 | D2 |\n:::customblock\n| KEY | VAL |\n| --- | --- |\n| PORT | 8080 |\n:::\n![Diagram](arch.png)";
        let doc = parse_d2f_markdown(md).unwrap();

        assert_eq!(doc.body.len(), 1);
        if let DocumentElement::Section {
            level,
            title,
            children,
        } = &doc.body[0]
        {
            assert_eq!(*level, 1);
            assert_eq!(title, "Main Heading");
            assert_eq!(children.len(), 8);
            assert_eq!(children[0], DocumentElement::text("Paragraph line"));
            assert_eq!(
                children[1],
                DocumentElement::check_box_item(false, "Task 1")
            );
            assert_eq!(children[2], DocumentElement::bullet_list_item("Bullet 1"));
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
                    vec![
                        vec!["H1".into(), "H2".into()],
                        vec!["D1".into(), "D2".into()]
                    ]
                )
            );
            assert_eq!(
                children[6],
                DocumentElement::block_directive(
                    "customblock",
                    vec![DocumentElement::table(
                        vec![TableAlignment::None, TableAlignment::None],
                        vec![
                            vec!["KEY".into(), "VAL".into()],
                            vec!["PORT".into(), "8080".into()]
                        ]
                    )]
                )
            );
            assert_eq!(children[7], DocumentElement::image("Diagram", "arch.png"));
        } else {
            panic!("expected Section");
        }
    }
}
