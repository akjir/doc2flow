use super::*;
use crate::core::document::*;

/// State tracker for filtering HTML comments across lines.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub(crate) struct CommentFilterState {
    /// Indicates whether parsing is currently within a multi-line HTML comment block.
    pub(crate) in_comment: bool,
}

impl CommentFilterState {
    /// Creates a new comment filter in standard mode.
    #[must_use]
    pub(crate) fn new() -> Self {
        Self::default()
    }

    /// Filters HTML comments from a single line.
    ///
    /// Returns `Some(processed_line)` or `None` if the line was fully swallowed by comments.
    pub(crate) fn process_line<'a>(&mut self, mut current: &'a str, buf: &'a mut String) -> Option<&'a str> {
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

/// Classifies a non-table line and appends it to target buffer or document body.
pub(crate) fn classify_and_push_line(
    doc: &mut Document,
    section_stack: &mut Vec<DocumentElement>,
    effective_line: &str,
    list_state: &mut ListState,
) {
    let trimmed = effective_line.trim();
    if trimmed.is_empty() {
        list_state.flush(doc, section_stack);
        list_state.root_ordered_position = None;
        return;
    }

    if let Some((spaces, checked, content)) = parse_check_box_item(effective_line) {
        let processed = replace_text_directives(content);
        list_state.process_item(
            doc,
            section_stack,
            spaces,
            ListItemKind::CheckBox(checked, processed),
        );
    } else if let Some((spaces, content)) = parse_bullet_list_item(effective_line) {
        let processed = replace_text_directives(content);
        list_state.process_item(
            doc,
            section_stack,
            spaces,
            ListItemKind::Bullet(processed),
        );
    } else if let Some((spaces, _parsed_num, content)) =
        parse_ordered_list_candidate(effective_line)
    {
        let processed = replace_text_directives(content);
        list_state.process_item(
            doc,
            section_stack,
            spaces,
            ListItemKind::Ordered(processed),
        );
    } else {
        list_state.flush(doc, section_stack);
        list_state.root_ordered_position = None;

        if is_horizontal_rule(trimmed) {
            push_element(
                doc,
                section_stack,
                DocumentElement::horizontal_rule(),
            );
        } else if let Some((level, title)) = parse_heading_line(effective_line) {
            push_section(doc, section_stack, level, title);
        } else if let Some((alt, url)) = parse_image(trimmed) {
            push_element(
                doc,
                section_stack,
                DocumentElement::image(alt, url),
            );
        } else if trimmed.starts_with('>') {
            let (shoutout_kind, content) = parse_shoutout_line(trimmed);
            let processed = replace_text_directives(content);
            push_element(
                doc,
                section_stack,
                DocumentElement::shoutout(shoutout_kind, processed),
            );
        } else if is_plain_text(trimmed) {
            let processed = replace_text_directives(trimmed);
            push_element(
                doc,
                section_stack,
                DocumentElement::text(processed),
            );
        } else {
            push_element(
                doc,
                section_stack,
                DocumentElement::unknown(trimmed),
            );
        }
    }
}

/// Unwinds all active sections on the stack into their parent containers or document body.
pub(crate) fn flush_section_stack(doc: &mut Document, section_stack: &mut Vec<DocumentElement>) {
    while let Some(popped) = section_stack.pop() {
        if let Some(parent) = section_stack.last_mut() {
            let _ = parent.push_child(popped);
        } else {
            doc.push_body(popped);
        }
    }
}

/// Returns the heading level if the element is a Section.
#[must_use]
pub(crate) fn get_section_level(elem: &DocumentElement) -> usize {
    match elem {
        DocumentElement::Section { level, .. } => *level,
        _ => 0,
    }
}

/// Checks whether a line represents a markdown horizontal rule (`---`, `----`, etc.).
#[must_use]
pub(crate) fn is_horizontal_rule(line: &str) -> bool {
    let trimmed = line.trim();
    trimmed.len() >= 3 && trimmed.chars().all(|c| c == '-')
}

/// Checks whether a line represents plain text.
#[must_use]
pub(crate) fn is_plain_text(line: &str) -> bool {
    !line.starts_with(['#', '>', '-', '*', '`', '|'])
}

/// Parses a markdown heading line into its normalized section level (1, 2, or 3) and title.
#[must_use]
pub(crate) fn parse_heading_line(line: &str) -> Option<(usize, &str)> {
    let trimmed = line.trim_start();
    if !trimmed.starts_with('#') {
        return None;
    }
    let hash_count = trimmed.chars().take_while(|&c| c == '#').count();
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
#[must_use]
pub(crate) fn parse_image(input: &str) -> Option<(&str, &str)> {
    let trimmed = input.trim();
    let rest = trimmed.strip_prefix("![")?;
    let close_bracket_idx = rest.find("](")?;
    let alt = &rest[..close_bracket_idx];
    let after_bracket = &rest[close_bracket_idx + 2..];
    let url = after_bracket.strip_suffix(')')?;
    Some((alt, url.trim()))
}

/// Parses a shoutout line starting with `>` into its element kind and inner content.
#[must_use]
pub(crate) fn parse_shoutout_line(line: &str) -> (ShoutoutElementKind, &str) {
    debug_assert!(line.starts_with('>'));
    let inner = line[1..].trim_start();
    const PREFIX_MAP: [(&str, ShoutoutElementKind); 4] = [
        ("!!!", ShoutoutElementKind::Caution),
        ("!!", ShoutoutElementKind::Warning),
        ("!", ShoutoutElementKind::Important),
        ("?", ShoutoutElementKind::Tip),
    ];
    for (prefix, kind) in PREFIX_MAP {
        if let Some(rest) = inner.strip_prefix(prefix) {
            let content = rest.strip_prefix(' ').unwrap_or(rest);
            return (kind, content);
        }
    }
    (ShoutoutElementKind::Note, inner)
}

/// Appends an element either to the topmost active section or document body.
pub(crate) fn push_element(
    doc: &mut Document,
    section_stack: &mut [DocumentElement],
    elem: DocumentElement,
) {
    if let Some(active_section) = section_stack.last_mut() {
        let _ = active_section.push_child(elem);
    } else {
        doc.push_body(elem);
    }
}

/// Unwinds closed sections from the stack and adds the new section to the hierarchy.
pub(crate) fn push_section(
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

/// Replaces all inline text directives (`:name[label]{attr}`) with `UNKNOWN DIRECTIVE (<name>)`.
#[must_use]
pub(crate) fn replace_text_directives(text: &str) -> String {
    if !text.contains(':') {
        return text.to_string();
    }
    let mut result = String::with_capacity(text.len());
    let bytes = text.as_bytes();
    let mut cursor = 0;

    while cursor < bytes.len() {
        if bytes[cursor] == b':' {
            let is_valid_start = if cursor > 0 {
                let prev = bytes[cursor - 1];
                !prev.is_ascii_alphanumeric() && prev != b':'
            } else {
                true
            };

            if is_valid_start {
                let rest = &text[cursor..];
                if let Some((consumed_len, name)) = try_scan_text_directive(rest) {
                    result.push_str("UNKNOWN DIRECTIVE (");
                    result.push_str(name);
                    result.push(')');
                    cursor += consumed_len;
                    continue;
                }
            }
        }

        let ch = text[cursor..].chars().next().unwrap();
        result.push(ch);
        cursor += ch.len_utf8();
    }

    result
}

/// Trims surrounding quotes only if enclosed by identical matching single or double quotes.
#[must_use]
pub(crate) fn trim_matching_quotes(input: &str) -> &str {
    let s = input.trim();
    if (s.starts_with('"') && s.ends_with('"') && s.len() >= 2)
        || (s.starts_with('\'') && s.ends_with('\'') && s.len() >= 2)
    {
        &s[1..s.len() - 1]
    } else {
        s
    }
}

/// Scans a text directive candidate starting with `:` and returns consumed byte length and directive name.
#[must_use]
pub(crate) fn try_scan_text_directive(rest: &str) -> Option<(usize, &str)> {
    if !rest.starts_with(':') || rest.starts_with("::") {
        return None;
    }
    let after_colon = &rest[1..];
    let first_char = after_colon.chars().next()?;
    if !first_char.is_ascii_alphanumeric() {
        return None;
    }
    let name_len = after_colon
        .chars()
        .take_while(|&c| is_directive_name_char(c))
        .count();
    if name_len == 0 {
        return None;
    }
    let name = &after_colon[..name_len];
    let mut pos = 1 + name_len;

    if rest[pos..].starts_with('[')
        && let Some(close_idx) = rest[pos + 1..].find(']')
    {
        pos = pos + 1 + close_idx + 1;
    }

    if rest[pos..].starts_with('{')
        && let Some(close_idx) = rest[pos + 1..].find('}')
    {
        pos = pos + 1 + close_idx + 1;
    }

    Some((pos, name))
}

