//! Ordered list item vertical slice feature module.

use std::fmt::Write as _;

use crate::core::document::DocumentElement;
use crate::core::feature::Feature;

/// Embedded ordered list item CSS stylesheet.
pub const CSS: &str = include_str!("ordered_list_item.css");

/// Ordered list item feature renderer handling numbered and ordered list items.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct OrderedListItemFeature;

impl OrderedListItemFeature {
    /// Creates a new ordered list item feature instance.
    ///
    /// # Examples
    ///
    /// ```
    /// use doc2flow::features::ordered_list_item::OrderedListItemFeature;
    ///
    /// let feature = OrderedListItemFeature::new();
    /// ```
    pub const fn new() -> Self {
        Self
    }
}

impl Feature for OrderedListItemFeature {
    /// Converts an ordered list item document element into an HTML string representation.
    ///
    /// # Examples
    ///
    /// ```
    /// use doc2flow::core::document::DocumentElement;
    /// use doc2flow::core::feature::Feature;
    /// use doc2flow::features::ordered_list_item::OrderedListItemFeature;
    ///
    /// let feature = OrderedListItemFeature::new();
    /// let elem = DocumentElement::ordered_list_item(0, 1, DocumentElement::text("First item"));
    /// let html = feature.to_html(&elem, "", 1);
    /// assert!(html.contains("class=\"item item-order\""));
    /// ```
    fn to_html(&self, element: &DocumentElement, _content: &str, indent: usize) -> String {
        match element {
            DocumentElement::OrderedListItem {
                content,
                depth,
                position,
            } => {
                let spaces = indent * 2;
                let inner_spaces = (indent + 1) * 2;
                let content_len = match content.as_ref() {
                    DocumentElement::Text(text) => text.len() * 2,
                    _ => _content.len(),
                };

                let mut out =
                    String::with_capacity(content_len + spaces * 2 + inner_spaces * 2 + 128);
                push_indent(&mut out, indent);

                if *depth > 0 {
                    out.push_str("<div class=\"item item-order\" style=\"--indent: ");
                    let _ = write!(out, "{depth}");
                    out.push_str(";\">\n");
                } else {
                    out.push_str("<div class=\"item item-order\">\n");
                }

                push_indent(&mut out, indent + 1);
                out.push_str("<span class=\"order-marker\">");
                format_marker_into(&mut out, *depth, *position);
                out.push_str("</span>\n");

                push_indent(&mut out, indent + 1);
                out.push_str("<span class=\"order-content\">\n");

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
                    _ => {
                        if !_content.is_empty() {
                            for line in _content.lines() {
                                push_indent(&mut out, indent + 2);
                                out.push_str(line.trim());
                                out.push('\n');
                            }
                        }
                    }
                }

                push_indent(&mut out, indent + 1);
                out.push_str("</span>\n");

                push_indent(&mut out, indent);
                out.push_str("</div>\n");

                out
            }
            _ => String::new(),
        }
    }

    /// Returns the embedded CSS stylesheet for the ordered list item feature.
    fn css(&self) -> Option<&'static str> {
        Some(CSS)
    }
}

/// Formats the ordered list item marker based on list depth and position.
fn format_marker_into(out: &mut String, depth: usize, position: usize) {
    match depth % 3 {
        0 => {
            let _ = write!(out, "{position}.");
        }
        1 => {
            format_alpha_into(out, position);
            out.push('.');
        }
        _ => {
            format_roman_into(out, position);
            out.push('.');
        }
    }
}

/// Formats a 1-based number to alphabetic representation into a destination buffer.
fn format_alpha_into(out: &mut String, mut n: usize) {
    if n == 0 {
        out.push('a');
        return;
    }
    if n <= 26 {
        const ALPHAS: &[u8; 26] = b"abcdefghijklmnopqrstuvwxyz";
        out.push(ALPHAS[n - 1] as char);
        return;
    }
    let mut buf = [0u8; 16];
    let mut pos = 16;
    while n > 0 {
        n -= 1;
        pos -= 1;
        buf[pos] = b'a' + (n % 26) as u8;
        n /= 26;
    }
    if let Ok(s) = std::str::from_utf8(&buf[pos..]) {
        out.push_str(s);
    }
}

/// Formats a 1-based number to lowercase Roman numerals into a destination buffer.
fn format_roman_into(out: &mut String, n: usize) {
    if n == 0 {
        out.push('i');
        return;
    }
    if (1..=10).contains(&n) {
        const ROMANS: &[&str; 10] = &["i", "ii", "iii", "iv", "v", "vi", "vii", "viii", "ix", "x"];
        out.push_str(ROMANS[n - 1]);
        return;
    }
    const ROMAN_MAPPING: [(usize, &str); 13] = [
        (1000, "m"),
        (900, "cm"),
        (500, "d"),
        (400, "cd"),
        (100, "c"),
        (90, "xc"),
        (50, "l"),
        (40, "xl"),
        (10, "x"),
        (9, "ix"),
        (5, "v"),
        (4, "iv"),
        (1, "i"),
    ];
    let mut num = n;
    for (val, sym) in ROMAN_MAPPING {
        while num >= val {
            out.push_str(sym);
            num -= val;
        }
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
    fn test_ordered_list_item_empty_for_unsupported_elements() {
        let feature = OrderedListItemFeature::new();
        let text = DocumentElement::text("Regular text");
        assert_eq!(feature.to_html(&text, "", 0), "");
    }

    #[test]
    fn test_ordered_list_item_feature_constructor_new() {
        let feature = OrderedListItemFeature::new();
        assert_eq!(feature, OrderedListItemFeature);
    }

    #[test]
    fn test_ordered_list_item_feature_css() {
        let feature = OrderedListItemFeature::new();
        let css = feature.css().expect("ordered list item css should exist");
        assert!(css.contains("--order-marker-color:"));
        assert!(css.contains(".order-marker"));
        assert!(css.contains(".order-content"));
    }

    #[test]
    fn test_ordered_list_item_renders_root_depth() {
        let feature = OrderedListItemFeature::new();
        let element =
            DocumentElement::ordered_list_item(0, 1, DocumentElement::text("First numbered item"));
        let html = feature.to_html(&element, "", 1);
        let expected = concat!(
            "  <div class=\"item item-order\">\n",
            "    <span class=\"order-marker\">1.</span>\n",
            "    <span class=\"order-content\">\n",
            "      First numbered item\n",
            "    </span>\n",
            "  </div>\n"
        );
        assert_eq!(html, expected);
    }

    #[test]
    fn test_ordered_list_item_renders_nested_depth_1_alpha() {
        let feature = OrderedListItemFeature::new();
        let element =
            DocumentElement::ordered_list_item(1, 2, DocumentElement::text("Nested sub-item"));
        let html = feature.to_html(&element, "", 2);
        let expected = concat!(
            "    <div class=\"item item-order\" style=\"--indent: 1;\">\n",
            "      <span class=\"order-marker\">b.</span>\n",
            "      <span class=\"order-content\">\n",
            "        Nested sub-item\n",
            "      </span>\n",
            "    </div>\n"
        );
        assert_eq!(html, expected);
    }

    #[test]
    fn test_ordered_list_item_renders_nested_depth_2_roman() {
        let feature = OrderedListItemFeature::new();
        let element =
            DocumentElement::ordered_list_item(2, 3, DocumentElement::text("Sub-sub-item"));
        let html = feature.to_html(&element, "", 0);
        let expected = concat!(
            "<div class=\"item item-order\" style=\"--indent: 2;\">\n",
            "  <span class=\"order-marker\">iii.</span>\n",
            "  <span class=\"order-content\">\n",
            "    Sub-sub-item\n",
            "  </span>\n",
            "</div>\n"
        );
        assert_eq!(html, expected);
    }

    #[test]
    fn test_ordered_list_item_renders_nested_depth_3_numeric() {
        let feature = OrderedListItemFeature::new();
        let element =
            DocumentElement::ordered_list_item(3, 4, DocumentElement::text("Deep numeric item"));
        let html = feature.to_html(&element, "", 1);
        let expected = concat!(
            "  <div class=\"item item-order\" style=\"--indent: 3;\">\n",
            "    <span class=\"order-marker\">4.</span>\n",
            "    <span class=\"order-content\">\n",
            "      Deep numeric item\n",
            "    </span>\n",
            "  </div>\n"
        );
        assert_eq!(html, expected);
    }

    #[test]
    fn test_ordered_list_item_renders_image_content() {
        let feature = OrderedListItemFeature::new();
        let element = DocumentElement::ordered_list_item(
            1,
            1,
            DocumentElement::image("Diagram preview", "images/diagram.png"),
        );
        let html = feature.to_html(&element, "", 1);
        let expected = concat!(
            "  <div class=\"item item-order\" style=\"--indent: 1;\">\n",
            "    <span class=\"order-marker\">a.</span>\n",
            "    <span class=\"order-content\">\n",
            "      <img src=\"images/diagram.png\" alt=\"Diagram preview\" />\n",
            "    </span>\n",
            "  </div>\n"
        );
        assert_eq!(html, expected);
    }

    #[test]
    fn test_ordered_list_item_renders_inline_formatting() {
        let feature = OrderedListItemFeature::new();
        let element = DocumentElement::ordered_list_item(
            0,
            2,
            DocumentElement::text(
                "Step with **bold**, *italic*, ~~strike~~, `code`, and <special> & characters",
            ),
        );
        let html = feature.to_html(&element, "", 0);
        let expected = concat!(
            "<div class=\"item item-order\">\n",
            "  <span class=\"order-marker\">2.</span>\n",
            "  <span class=\"order-content\">\n",
            "    Step with <strong>bold</strong>, <em>italic</em>, <s>strike</s>, <code>code</code>, and &lt;special&gt; &amp; characters\n",
            "  </span>\n",
            "</div>\n"
        );
        assert_eq!(html, expected);
    }

    #[test]
    fn test_format_alpha_and_roman_boundary_cases() {
        let mut buf = String::new();

        format_alpha_into(&mut buf, 0);
        assert_eq!(buf, "a");
        buf.clear();

        format_alpha_into(&mut buf, 1);
        assert_eq!(buf, "a");
        buf.clear();

        format_alpha_into(&mut buf, 26);
        assert_eq!(buf, "z");
        buf.clear();

        format_alpha_into(&mut buf, 27);
        assert_eq!(buf, "aa");
        buf.clear();

        format_alpha_into(&mut buf, 52);
        assert_eq!(buf, "az");
        buf.clear();

        format_alpha_into(&mut buf, 53);
        assert_eq!(buf, "ba");
        buf.clear();

        format_alpha_into(&mut buf, 702);
        assert_eq!(buf, "zz");
        buf.clear();

        format_alpha_into(&mut buf, 703);
        assert_eq!(buf, "aaa");
        buf.clear();

        format_roman_into(&mut buf, 0);
        assert_eq!(buf, "i");
        buf.clear();

        format_roman_into(&mut buf, 1);
        assert_eq!(buf, "i");
        buf.clear();

        format_roman_into(&mut buf, 4);
        assert_eq!(buf, "iv");
        buf.clear();

        format_roman_into(&mut buf, 9);
        assert_eq!(buf, "ix");
        buf.clear();

        format_roman_into(&mut buf, 10);
        assert_eq!(buf, "x");
        buf.clear();

        format_roman_into(&mut buf, 1984);
        assert_eq!(buf, "mcmlxxxiv");
        buf.clear();

        format_roman_into(&mut buf, 3999);
        assert_eq!(buf, "mmmcmxcix");
        buf.clear();
    }
}
