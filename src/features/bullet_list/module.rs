//! Bullet list vertical slice feature module.

use crate::core::document::DocumentElement;
use crate::core::feature::Feature;

/// Embedded bullet list CSS stylesheet.
pub const CSS: &str = include_str!("bullet_list.css");

/// Bullet list feature renderer handling bullet and unordered list items.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct BulletListFeature;

impl BulletListFeature {
    /// Creates a new bullet list feature instance.
    ///
    /// # Examples
    ///
    /// ```
    /// use doc2flow::features::bullet_list::BulletListFeature;
    ///
    /// let feature = BulletListFeature::new();
    /// ```
    pub const fn new() -> Self {
        Self
    }
}

impl Feature for BulletListFeature {
    /// Converts a bullet list item document element into an HTML string representation.
    ///
    /// # Examples
    ///
    /// ```
    /// use doc2flow::core::document::DocumentElement;
    /// use doc2flow::core::feature::Feature;
    /// use doc2flow::features::bullet_list::BulletListFeature;
    ///
    /// let feature = BulletListFeature::new();
    /// let elem = DocumentElement::bullet_list_item(DocumentElement::text("List item"));
    /// let html = feature.to_html(&elem, "", 1);
    /// assert!(html.contains("class=\"item item-bullet\""));
    /// ```
    fn to_html(&self, element: &DocumentElement, _content: &str, indent: usize) -> String {
        match element {
            DocumentElement::BulletListItem { content, .. } => {
                let spaces = indent * 2;
                let inner_spaces = (indent + 1) * 2;
                let content_len = match content.as_ref() {
                    DocumentElement::Text(text) => text.len() * 2,
                    _ => 0,
                };

                let mut out = String::with_capacity(
                    content_len + _content.len() + spaces * 2 + inner_spaces * 2 + 128,
                );
                push_indent(&mut out, indent);
                out.push_str("<div class=\"item item-bullet\">\n");

                push_indent(&mut out, indent + 1);
                out.push_str("<span class=\"bullet-marker\">&bull;</span>\n");

                push_indent(&mut out, indent + 1);
                out.push_str("<span class=\"bullet-content\">\n");

                match content.as_ref() {
                    DocumentElement::Text(text) => {
                        for line in text.lines() {
                            push_indent(&mut out, indent + 2);
                            format_inline_into(&mut out, line);
                            out.push('\n');
                        }
                    }
                    DocumentElement::Image { alt, url } => {
                        push_indent(&mut out, indent + 2);
                        out.push_str("<img src=\"");
                        escape_html_into(&mut out, url);
                        out.push_str("\" alt=\"");
                        escape_html_into(&mut out, alt);
                        out.push_str("\" />\n");
                    }
                    _ => {}
                }

                push_indent(&mut out, indent + 1);
                out.push_str("</span>\n");

                push_indent(&mut out, indent);
                out.push_str("</div>\n");
                out.push_str(_content);

                out
            }
            _ => String::new(),
        }
    }

    /// Returns the embedded CSS stylesheet for the bullet list feature.
    fn css(&self) -> Option<&'static str> {
        Some(CSS)
    }
}

/// Escapes special HTML characters in a string into the destination buffer.
fn escape_html_into(out: &mut String, s: &str) {
    let mut last_idx = 0;
    let bytes = s.as_bytes();

    for (i, &b) in bytes.iter().enumerate() {
        let escape = match b {
            b'&' => "&amp;",
            b'<' => "&lt;",
            b'>' => "&gt;",
            b'"' => "&quot;",
            b'\'' => "&#39;",
            _ => continue,
        };

        if i > last_idx {
            out.push_str(&s[last_idx..i]);
        }
        out.push_str(escape);
        last_idx = i + 1;
    }

    if last_idx < s.len() {
        out.push_str(&s[last_idx..]);
    }
}

/// Finds the index of a matching sequence of backticks in a slice.
fn find_matching_backticks(slice: &str, count: usize) -> Option<usize> {
    let mut idx = 0;
    let bytes = slice.as_bytes();

    while idx < slice.len() {
        if bytes[idx] == b'`' {
            let current_count = bytes[idx..].iter().take_while(|&&b| b == b'`').count();
            if current_count == count {
                return Some(idx);
            }
            idx += current_count;
        } else {
            idx += 1;
        }
    }

    None
}

/// Finds the index of a matching closing delimiter, skipping any inner code spans.
fn find_matching_delimiter(slice: &str, delimiter: &str) -> Option<usize> {
    let mut idx = 0;
    let bytes = slice.as_bytes();

    while idx < slice.len() {
        if slice[idx..].starts_with(delimiter) {
            return Some(idx);
        }

        if bytes[idx] == b'`' {
            let count = bytes[idx..].iter().take_while(|&&b| b == b'`').count();
            let after_open = idx + count;
            if let Some(close_pos) = find_matching_backticks(&slice[after_open..], count) {
                idx = after_open + close_pos + count;
                continue;
            }
        }

        idx += 1;
    }

    None
}

/// Finds the index of a matching single asterisk delimiter, skipping code spans and double asterisks.
fn find_matching_single_asterisk(slice: &str) -> Option<usize> {
    let mut idx = 0;
    let bytes = slice.as_bytes();

    while idx < slice.len() {
        if bytes[idx] == b'`' {
            let count = bytes[idx..].iter().take_while(|&&b| b == b'`').count();
            let after_open = idx + count;
            if let Some(close_pos) = find_matching_backticks(&slice[after_open..], count) {
                idx = after_open + close_pos + count;
                continue;
            }
        }

        if bytes[idx] == b'*' {
            let is_double = (idx + 1 < bytes.len() && bytes[idx + 1] == b'*')
                || (idx > 0 && bytes[idx - 1] == b'*');
            if !is_double && idx > 0 && !bytes[idx - 1].is_ascii_whitespace() {
                return Some(idx);
            }
        }

        idx += 1;
    }

    None
}

/// Finds the index of a matching single underscore delimiter with word boundary checking.
fn find_matching_single_underscore(slice: &str) -> Option<usize> {
    let mut idx = 0;
    let bytes = slice.as_bytes();

    while idx < slice.len() {
        if bytes[idx] == b'`' {
            let count = bytes[idx..].iter().take_while(|&&b| b == b'`').count();
            let after_open = idx + count;
            if let Some(close_pos) = find_matching_backticks(&slice[after_open..], count) {
                idx = after_open + close_pos + count;
                continue;
            }
        }

        if bytes[idx] == b'_' {
            let is_double = (idx + 1 < bytes.len() && bytes[idx + 1] == b'_')
                || (idx > 0 && bytes[idx - 1] == b'_');
            if !is_double && idx > 0 && !bytes[idx - 1].is_ascii_whitespace() {
                let is_word_char_after = idx + 1 < bytes.len()
                    && slice[idx + 1..].chars().next().is_some_and(|c| c.is_alphanumeric());
                if !is_word_char_after {
                    return Some(idx);
                }
            }
        }

        idx += 1;
    }

    None
}

/// Finds the index of a matching underscore delimiter with word boundary checking.
fn find_matching_underscore_delimiter(slice: &str, delimiter: &str) -> Option<usize> {
    let mut idx = 0;
    let bytes = slice.as_bytes();
    let delim_len = delimiter.len();

    while idx < slice.len() {
        if bytes[idx] == b'`' {
            let count = bytes[idx..].iter().take_while(|&&b| b == b'`').count();
            let after_open = idx + count;
            if let Some(close_pos) = find_matching_backticks(&slice[after_open..], count) {
                idx = after_open + close_pos + count;
                continue;
            }
        }

        if slice[idx..].starts_with(delimiter) && idx > 0 && !bytes[idx - 1].is_ascii_whitespace() {
            let is_word_char_after = idx + delim_len < bytes.len()
                && slice[idx + delim_len..].chars().next().is_some_and(|c| c.is_alphanumeric());
            if !is_word_char_after {
                return Some(idx);
            }
        }

        idx += 1;
    }

    None
}

/// Formats inline markdown elements into an HTML output buffer.
fn format_inline_into(out: &mut String, input: &str) {
    let mut idx = 0;
    let bytes = input.as_bytes();

    while idx < input.len() {
        let remaining = &input[idx..];

        // 1. Inline code span: `code`
        if bytes[idx] == b'`' {
            let count = bytes[idx..].iter().take_while(|&&b| b == b'`').count();
            let after_open = idx + count;
            if let Some(close_pos) = find_matching_backticks(&input[after_open..], count) {
                let code_content = &input[after_open..after_open + close_pos];
                out.push_str("<code>");
                escape_html_into(out, code_content);
                out.push_str("</code>");
                idx = after_open + close_pos + count;
                continue;
            } else {
                escape_html_into(out, &input[idx..after_open]);
                idx = after_open;
                continue;
            }
        }

        // 2. Strikethrough: ~~text~~
        if remaining.starts_with("~~") {
            let after_open = idx + 2;
            if let Some(close_pos) = find_matching_delimiter(&input[after_open..], "~~") {
                if close_pos > 0 {
                    let inner = &input[after_open..after_open + close_pos];
                    out.push_str("<s>");
                    format_inline_into(out, inner);
                    out.push_str("</s>");
                    idx = after_open + close_pos + 2;
                    continue;
                }
            }
            out.push_str("~~");
            idx += 2;
            continue;
        }

        // 3. Bold + Italic: ***text***
        if remaining.starts_with("***") {
            let after_open = idx + 3;
            if let Some(close_pos) = find_matching_delimiter(&input[after_open..], "***") {
                if close_pos > 0 {
                    let inner = &input[after_open..after_open + close_pos];
                    out.push_str("<strong><em>");
                    format_inline_into(out, inner);
                    out.push_str("</em></strong>");
                    idx = after_open + close_pos + 3;
                    continue;
                }
            }
        }

        // 4. Bold: **text**
        if remaining.starts_with("**") {
            let after_open = idx + 2;
            if let Some(close_pos) = find_matching_delimiter(&input[after_open..], "**") {
                if close_pos > 0 {
                    let inner = &input[after_open..after_open + close_pos];
                    out.push_str("<strong>");
                    format_inline_into(out, inner);
                    out.push_str("</strong>");
                    idx = after_open + close_pos + 2;
                    continue;
                }
            }
            out.push_str("**");
            idx += 2;
            continue;
        }

        // 5. Bold + Italic with underscores: ___text___
        if remaining.starts_with("___") {
            let is_word_char_before = idx > 0
                && input[..idx].chars().next_back().is_some_and(|c| c.is_alphanumeric());
            if !is_word_char_before {
                let after_open = idx + 3;
                if let Some(close_pos) =
                    find_matching_underscore_delimiter(&input[after_open..], "___")
                {
                    if close_pos > 0 {
                        let inner = &input[after_open..after_open + close_pos];
                        out.push_str("<strong><em>");
                        format_inline_into(out, inner);
                        out.push_str("</em></strong>");
                        idx = after_open + close_pos + 3;
                        continue;
                    }
                }
            }
        }

        // 6. Bold with underscores: __text__
        if remaining.starts_with("__") {
            let is_word_char_before = idx > 0
                && input[..idx].chars().next_back().is_some_and(|c| c.is_alphanumeric());
            if !is_word_char_before {
                let after_open = idx + 2;
                if let Some(close_pos) =
                    find_matching_underscore_delimiter(&input[after_open..], "__")
                {
                    if close_pos > 0 {
                        let inner = &input[after_open..after_open + close_pos];
                        out.push_str("<strong>");
                        format_inline_into(out, inner);
                        out.push_str("</strong>");
                        idx = after_open + close_pos + 2;
                        continue;
                    }
                }
            }
            out.push_str("__");
            idx += 2;
            continue;
        }

        // 7. Italic: *text*
        if bytes[idx] == b'*' {
            let after_open = idx + 1;
            let not_leading_space =
                after_open < input.len() && !bytes[after_open].is_ascii_whitespace();
            if not_leading_space {
                if let Some(close_pos) = find_matching_single_asterisk(&input[after_open..]) {
                    if close_pos > 0 {
                        let inner = &input[after_open..after_open + close_pos];
                        out.push_str("<em>");
                        format_inline_into(out, inner);
                        out.push_str("</em>");
                        idx = after_open + close_pos + 1;
                        continue;
                    }
                }
            }
            out.push('*');
            idx += 1;
            continue;
        }

        // 8. Italic with underscore: _text_
        if bytes[idx] == b'_' {
            let is_word_char_before = idx > 0
                && input[..idx].chars().next_back().is_some_and(|c| c.is_alphanumeric());
            let after_open = idx + 1;
            let not_leading_space =
                after_open < input.len() && !bytes[after_open].is_ascii_whitespace();
            if !is_word_char_before && not_leading_space {
                if let Some(close_pos) = find_matching_single_underscore(&input[after_open..]) {
                    if close_pos > 0 {
                        let inner = &input[after_open..after_open + close_pos];
                        out.push_str("<em>");
                        format_inline_into(out, inner);
                        out.push_str("</em>");
                        idx = after_open + close_pos + 1;
                        continue;
                    }
                }
            }
            out.push('_');
            idx += 1;
            continue;
        }

        // 9. HTML Entities and regular characters
        match bytes[idx] {
            b'&' => out.push_str("&amp;"),
            b'<' => out.push_str("&lt;"),
            b'>' => out.push_str("&gt;"),
            b'"' => out.push_str("&quot;"),
            b'\'' => out.push_str("&#39;"),
            _ => {
                let plain_len = bytes[idx..]
                    .iter()
                    .take_while(|&&b| {
                        !matches!(b, b'`' | b'~' | b'*' | b'_' | b'&' | b'<' | b'>' | b'"' | b'\'')
                    })
                    .count();
                if plain_len > 0 {
                    out.push_str(&input[idx..idx + plain_len]);
                    idx += plain_len;
                    continue;
                }
                out.push(input[idx..].chars().next().unwrap_or(' '));
                idx += input[idx..].chars().next().map_or(1, |c| c.len_utf8());
                continue;
            }
        }
        idx += 1;
    }
}

/// Appends leading whitespace indentation to a buffer based on the specified indent level.
fn push_indent(out: &mut String, indent: usize) {
    for _ in 0..indent {
        out.push_str("  ");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bullet_list_empty_for_unsupported_elements() {
        let feature = BulletListFeature::new();
        let text = DocumentElement::text("Regular text");
        assert_eq!(feature.to_html(&text, "", 0), "");
    }

    #[test]
    fn test_bullet_list_feature_constructor_new() {
        let feature = BulletListFeature::new();
        assert_eq!(feature, BulletListFeature);
    }

    #[test]
    fn test_bullet_list_feature_css() {
        let feature = BulletListFeature::new();
        let css = feature.css().expect("bullet list css should exist");
        assert!(css.contains("--bullet-marker-color:"));
        assert!(css.contains(".bullet-marker"));
        assert!(css.contains(".bullet-content"));
    }

    #[test]
    fn test_bullet_list_renders_image_content() {
        let feature = BulletListFeature::new();
        let element = DocumentElement::bullet_list_item(
            DocumentElement::image("Alt text", "path/to/image.png"),
        );
        let html = feature.to_html(&element, "", 1);
        let expected = concat!(
            "  <div class=\"item item-bullet\">\n",
            "    <span class=\"bullet-marker\">&bull;</span>\n",
            "    <span class=\"bullet-content\">\n",
            "      <img src=\"path/to/image.png\" alt=\"Alt text\" />\n",
            "    </span>\n",
            "  </div>\n"
        );
        assert_eq!(html, expected);
    }

    #[test]
    fn test_bullet_list_renders_inline_formatting() {
        let feature = BulletListFeature::new();
        let element = DocumentElement::bullet_list_item(
            DocumentElement::text("Item with **bold**, *italic*, ~~strike~~, and `code` span"),
        );
        let html = feature.to_html(&element, "", 0);
        let expected = concat!(
            "<div class=\"item item-bullet\">\n",
            "  <span class=\"bullet-marker\">&bull;</span>\n",
            "  <span class=\"bullet-content\">\n",
            "    Item with <strong>bold</strong>, <em>italic</em>, <s>strike</s>, and <code>code</code> span\n",
            "  </span>\n",
            "</div>\n"
        );
        assert_eq!(html, expected);
    }

    #[test]
    fn test_bullet_list_renders_with_indent() {
        let feature = BulletListFeature::new();
        let element = DocumentElement::bullet_list_item(DocumentElement::text("Nested level 2 item"));
        let html = feature.to_html(&element, "", 2);
        let expected = concat!(
            "    <div class=\"item item-bullet\">\n",
            "      <span class=\"bullet-marker\">&bull;</span>\n",
            "      <span class=\"bullet-content\">\n",
            "        Nested level 2 item\n",
            "      </span>\n",
            "    </div>\n"
        );
        assert_eq!(html, expected);
    }

    #[test]
    fn test_bullet_list_renders_with_children() {
        let feature = BulletListFeature::new();
        let element = DocumentElement::bullet_list_item(DocumentElement::text("Parent item"));
        let child_html = "    <div class=\"item item-bullet\">\n      <span class=\"bullet-marker\">&bull;</span>\n      <span class=\"bullet-content\">\n        Child item\n      </span>\n    </div>\n";
        let html = feature.to_html(&element, child_html, 1);
        let expected_prefix = "  <div class=\"item item-bullet\">\n    <span class=\"bullet-marker\">&bull;</span>\n    <span class=\"bullet-content\">\n      Parent item\n    </span>\n  </div>\n";
        assert_eq!(html, format!("{expected_prefix}{child_html}"));
    }
}
