//! Inline markdown formatting and HTML escaping engine.

/// Escapes special HTML characters in a string into the destination buffer.
pub fn escape_html_into(out: &mut String, s: &str) {
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
pub fn format_inline_into(out: &mut String, input: &str) {
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
pub fn push_indent(out: &mut String, indent: usize) {
    for _ in 0..indent {
        out.push_str("  ");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_escape_html_entities() {
        let mut out = String::new();
        escape_html_into(&mut out, "5 < 10 & 20 > 15 \"quotes\" 'single'");
        assert_eq!(
            out,
            "5 &lt; 10 &amp; 20 &gt; 15 &quot;quotes&quot; &#39;single&#39;"
        );
    }

    #[test]
    fn test_format_inline_code() {
        let mut out = String::new();
        format_inline_into(&mut out, "Use `let x = 1;` here");
        assert_eq!(out, "Use <code>let x = 1;</code> here");
    }

    #[test]
    fn test_format_inline_bold_and_italic() {
        let mut out = String::new();
        format_inline_into(&mut out, "This is **bold**, *italic*, and ***both***.");
        assert_eq!(
            out,
            "This is <strong>bold</strong>, <em>italic</em>, and <strong><em>both</em></strong>."
        );
    }

    #[test]
    fn test_format_inline_underscores() {
        let mut out = String::new();
        format_inline_into(&mut out, "This is __bold__, _italic_, and ___both___.");
        assert_eq!(
            out,
            "This is <strong>bold</strong>, <em>italic</em>, and <strong><em>both</em></strong>."
        );
    }

    #[test]
    fn test_format_inline_strikethrough() {
        let mut out = String::new();
        format_inline_into(&mut out, "This is ~~strike~~ text.");
        assert_eq!(out, "This is <s>strike</s> text.");
    }

    #[test]
    fn test_format_inline_nested() {
        let mut out = String::new();
        format_inline_into(&mut out, "Formatted ~~**bold strikethrough**~~ with `code`.");
        assert_eq!(
            out,
            "Formatted <s><strong>bold strikethrough</strong></s> with <code>code</code>."
        );
    }

    #[test]
    fn test_format_inline_preserves_intra_word_underscores() {
        let mut out = String::new();
        format_inline_into(&mut out, "Host {{SERVER_NAME}}:{{PORT}} with key {{API_KEY}}.");
        assert_eq!(
            out,
            "Host {{SERVER_NAME}}:{{PORT}} with key {{API_KEY}}."
        );
    }

    #[test]
    fn test_push_indent() {
        let mut out = String::new();
        push_indent(&mut out, 0);
        assert_eq!(out, "");
        push_indent(&mut out, 2);
        assert_eq!(out, "    ");
    }
}
