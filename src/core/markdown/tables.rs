use crate::core::document::*;

/// Parses a single cell in a table delimiter row into its `TableAlignment`.
#[must_use]
pub(crate) fn parse_table_alignment(cell: &str) -> Option<TableAlignment> {
    let trimmed = cell.trim();
    if trimmed.is_empty() {
        return None;
    }

    let without_prefix = trimmed.strip_prefix(':');
    let has_prefix_colon = without_prefix.is_some();
    let inner = without_prefix.unwrap_or(trimmed);

    let without_suffix = inner.strip_suffix(':');
    let has_suffix_colon = without_suffix.is_some();
    let inner = without_suffix.unwrap_or(inner);

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
#[must_use]
pub(crate) fn parse_table_delimiter_row(line: &str) -> Option<Vec<TableAlignment>> {
    let trimmed = line.trim();
    if !trimmed.contains('|') && !trimmed.starts_with('-') {
        return None;
    }

    let inner = trimmed.strip_prefix('|').unwrap_or(trimmed);
    let inner = if inner.ends_with('|') && !inner.ends_with("\\|") {
        inner.strip_suffix('|').unwrap_or(inner)
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
#[must_use]
pub(crate) fn parse_table_row(line: &str) -> Vec<String> {
    let trimmed = line.trim();
    let inner = trimmed.strip_prefix('|').unwrap_or(trimmed);
    let inner = if inner.ends_with('|') && !inner.ends_with("\\|") {
        inner.strip_suffix('|').unwrap_or(inner)
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

