use super::*;
use crate::core::document::*;

/// Active list item held on the hierarchical list tracking stack.
#[derive(Debug)]
pub(crate) struct ActiveListItem {
    /// Element data model payload.
    element: DocumentElement,
    /// Indentation spaces offset.
    indent_spaces: usize,
    /// Last child ordered list numerical position.
    last_child_ordered_position: Option<usize>,
}

/// Intermediate list item classification before AST node construction.
#[derive(Debug)]
pub(crate) enum ListItemKind {
    /// Bullet list item with raw inner text/image slice.
    Bullet(String),
    /// Checkbox task item with checked state and raw inner slice.
    CheckBox(bool, String),
    /// Ordered list item with raw inner text/image slice.
    Ordered(String),
}

/// Hierarchical list parsing state tracking active parent list items and sequential numbering.
#[derive(Debug, Default)]
pub(crate) struct ListState {
    /// Next sequential position counter for root-level ordered lists.
    pub(crate) root_ordered_position: Option<usize>,
    /// Stack of currently open parent list items.
    pub(crate) stack: Vec<ActiveListItem>,
}

impl ListState {
    /// Creates a new empty list parsing state.
    #[must_use]
    pub(crate) fn new() -> Self {
        Self::default()
    }

    /// Flushes all active list items on the stack into their parent elements or target container.
    pub(crate) fn flush(
        &mut self,
        doc: &mut Document,
        section_stack: &mut [DocumentElement],
    ) {
        while let Some(popped) = self.stack.pop() {
            if let Some(parent) = self.stack.last_mut() {
                let _ = parent.element.push_child(popped.element);
            } else {
                push_element(doc, section_stack, popped.element);
            }
        }
    }

    /// Processes an incoming list item line into the active list hierarchy.
    pub(crate) fn process_item(
        &mut self,
        doc: &mut Document,
        section_stack: &mut [DocumentElement],
        indent_spaces: usize,
        kind: ListItemKind,
    ) {
        while let Some(top) = self.stack.last() {
            if top.indent_spaces >= indent_spaces {
                let popped = self.stack.pop().unwrap();
                if let Some(parent) = self.stack.last_mut() {
                    let _ = parent.element.push_child(popped.element);
                } else {
                    push_element(doc, section_stack, popped.element);
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

/// Parses a bullet list item into its leading spaces and inner content.
#[must_use]
pub(crate) fn parse_bullet_list_item(line: &str) -> Option<(usize, &str)> {
    let leading_spaces = line.chars().take_while(|&c| c == ' ').count();
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
#[must_use]
pub(crate) fn parse_check_box_item(line: &str) -> Option<(usize, bool, &str)> {
    let leading_spaces = line.chars().take_while(|&c| c == ' ').count();
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

/// Parses an ordered list item candidate into its leading spaces, leading number, and inner content.
#[must_use]
pub(crate) fn parse_ordered_list_candidate(line: &str) -> Option<(usize, u64, &str)> {
    let leading_spaces = line.chars().take_while(|&c| c == ' ').count();
    let rest = &line[leading_spaces..];

    let digits_len = rest.chars().take_while(|c| c.is_ascii_digit()).count();
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

