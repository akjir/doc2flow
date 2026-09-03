use super::*;
use crate::core::document::*;
use crate::core::error::*;
use crate::core::{Error, Result};
use std::collections::HashMap;

/// Block directive names allowed before the first level-1 heading.
pub(crate) const ALLOWED_PRE_H1_DIRECTIVES: &[&str] = &["variables"];

/// Constructs a standardized diagnostic error for block directives not allowed before the first level-1 heading.
pub(crate) fn build_disallowed_pre_h1_block_directive_err(
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
pub(crate) fn build_invalid_block_directive_name_err(line_number: usize, line_snippet: &str) -> Error {
    let carets = build_caret_annotation(1, line_snippet.len().max(1), line_snippet.len().max(1));
    DiagnosticError {
        message: "invalid block directive name".into(),
        file_path: "<input>".into(),
        line_number,
        col_number: 1,
        line_snippet: line_snippet.into(),
        annotation_carets: carets,
        annotation_text: "directive name must contain only alphanumeric characters, hyphens, or underscores (a-z, 0-9, -, _)"
            .into(),
        help_text: "use only alphanumeric characters, hyphens, or underscores for directive names, e.g. ':::unknown-directive'."
            .into(),
    }
    .into()
}

/// Constructs a standardized diagnostic error for malformed variables block directives.
pub(crate) fn build_invalid_variables_block_directive_err(line_number: usize, line_snippet: &str) -> Error {
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
pub(crate) fn build_missing_block_directive_name_err(line_number: usize, line_snippet: &str) -> Error {
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

/// Constructs a standardized diagnostic error for nested block directives.
pub(crate) fn build_nested_block_directive_err(line_number: usize, line_snippet: &str) -> Error {
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
pub(crate) fn build_unclosed_block_directive_err(line_number: usize, line_snippet: &str) -> Error {
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

/// Extracts dynamic `{{VAR_NAME}}` placeholders from a code block slice into the document header.
pub(crate) fn extract_code_block_variables(content: &str, header: &mut DocumentHeader) {
    let mut parts = content.split("{{");
    let _ = parts.next();

    for part in parts {
        if let Some((raw_var, _remainder)) = part.split_once("}}") {
            let var_name = raw_var.trim();
            if !var_name.is_empty()
                && var_name
                    .chars()
                    .all(|c| c.is_ascii_alphanumeric() || c == '_')
            {
                header.ensure_variable(var_name, "");
            }
        }
    }
}

/// Checks whether a block directive name is permitted before the first level-1 heading.
#[must_use]
pub(crate) fn is_allowed_pre_h1_directive(name: &str) -> bool {
    ALLOWED_PRE_H1_DIRECTIVES.contains(&name)
}

/// Checks whether a character is permitted in a directive name identifier (alphanumeric, `-`, `_`).
#[must_use]
pub(crate) fn is_directive_name_char(c: char) -> bool {
    c.is_ascii_alphanumeric() || c == '-' || c == '_'
}

/// Parses a directive header line into its category kind, name, label, and attributes.
#[must_use]
pub(crate) fn parse_directive_header(line: &str) -> Option<(DirectiveKind, &str, &str, &str)> {
    let trimmed = line.trim();
    let colon_count = trimmed.chars().take_while(|&c| c == ':').count();
    if colon_count == 0 {
        return None;
    }
    let kind = match colon_count {
        1 => DirectiveKind::Text,
        2 => DirectiveKind::Leaf,
        _ => DirectiveKind::Block,
    };
    let rest = &trimmed[colon_count..];
    let first_char = rest.chars().next()?;
    if !first_char.is_ascii_alphanumeric() {
        return None;
    }
    let name_len = rest.chars().take_while(|&c| is_directive_name_char(c)).count();
    if name_len == 0 {
        return None;
    }
    let name = &rest[..name_len];
    let mut remainder = &rest[name_len..];

    let mut label = "";
    if remainder.starts_with('[')
        && let Some(close_idx) = remainder.find(']')
    {
        label = &remainder[1..close_idx];
        remainder = &remainder[close_idx + 1..];
    }

    let mut attributes = "";
    if remainder.starts_with('{')
        && let Some(close_idx) = remainder.find('}')
    {
        attributes = &remainder[1..close_idx];
        remainder = &remainder[close_idx + 1..];
    }

    if (kind == DirectiveKind::Leaf || kind == DirectiveKind::Block) && !remainder.trim().is_empty() {
        return None;
    }

    Some((kind, name, label, attributes))
}

/// Processes a variables block directive by extracting key-value pairs from a markdown table.
pub(crate) fn process_variables_block(
    raw_lines: &[&str],
    doc: &mut Document,
    line_no: usize,
    line_snippet: &str,
) -> Result<(), Error> {
    let non_empty_lines: Vec<&str> = raw_lines
        .iter()
        .map(|l| l.trim())
        .filter(|l| !l.is_empty())
        .collect();

    if non_empty_lines.is_empty() || non_empty_lines.len() < 2 {
        return Err(build_invalid_variables_block_directive_err(
            line_no,
            line_snippet,
        ));
    }

    let header_line = non_empty_lines[0];
    let delimiter_line = non_empty_lines[1];

    if !header_line.contains('|') {
        return Err(build_invalid_variables_block_directive_err(
            line_no,
            line_snippet,
        ));
    }

    if parse_table_delimiter_row(delimiter_line).is_none() {
        return Err(build_invalid_variables_block_directive_err(
            line_no,
            line_snippet,
        ));
    }

    let first_idx = raw_lines.iter().position(|l| !l.trim().is_empty()).unwrap();
    let last_idx = raw_lines.iter().rposition(|l| !l.trim().is_empty()).unwrap();
    for line in &raw_lines[first_idx..=last_idx] {
        let trimmed = line.trim();
        if trimmed.is_empty() || !trimmed.contains('|') {
            return Err(build_invalid_variables_block_directive_err(
                line_no,
                line_snippet,
            ));
        }
    }

    let mut variables_map = HashMap::new();
    for line in &non_empty_lines[2..] {
        let mut row = parse_table_row(line);
        if !row.is_empty() {
            let key = row.remove(0);
            let val = if row.is_empty() {
                String::new()
            } else {
                row.remove(0)
            };
            doc.head.insert_variable(key.clone(), val.clone());
            variables_map.insert(key, val);
        }
    }

    doc.head.variables = Some(DocumentElement::table_variables(variables_map));
    let _ = doc.head.variables_mut();
    Ok(())
}

/// Trims leading and trailing empty lines from a slice of code lines.
#[must_use]
pub(crate) fn trim_code_block_lines(lines: &[&str]) -> String {
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

