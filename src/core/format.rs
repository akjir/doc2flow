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

/// Finds the index of a matching closing delimiter, skipping inner code spans.
fn find_matching_delimiter(slice: &str, delimiter: &str) -> Option<usize> {
    let mut idx = 0;
    let bytes = slice.as_bytes();
    let delim_bytes = delimiter.as_bytes();

    while idx < bytes.len() {
        if bytes[idx..].starts_with(delim_bytes) {
            return Some(idx);
        }

        if let Some(next_idx) = skip_code_span(slice, idx) {
            idx = next_idx;
            continue;
        }

        idx += 1;
    }

    None
}

/// Finds the index of a matching single asterisk delimiter, skipping code spans.
fn find_matching_single_asterisk(slice: &str) -> Option<usize> {
    let mut idx = 0;
    let bytes = slice.as_bytes();

    while idx < bytes.len() {
        if let Some(next_idx) = skip_code_span(slice, idx) {
            idx = next_idx;
            continue;
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

/// Finds the index of a matching single underscore delimiter with boundary checking.
fn find_matching_single_underscore(slice: &str) -> Option<usize> {
    let mut idx = 0;
    let bytes = slice.as_bytes();

    while idx < bytes.len() {
        if let Some(next_idx) = skip_code_span(slice, idx) {
            idx = next_idx;
            continue;
        }

        if bytes[idx] == b'_' {
            let is_double = (idx + 1 < bytes.len() && bytes[idx + 1] == b'_')
                || (idx > 0 && bytes[idx - 1] == b'_');
            if !is_double && idx > 0 && !bytes[idx - 1].is_ascii_whitespace() {
                let is_word_char_after = idx + 1 < bytes.len()
                    && slice[idx + 1..]
                        .chars()
                        .next()
                        .is_some_and(|c| c.is_alphanumeric());
                if !is_word_char_after {
                    return Some(idx);
                }
            }
        }

        idx += 1;
    }

    None
}

/// Finds the index of a matching underscore delimiter with boundary checking.
fn find_matching_underscore_delimiter(slice: &str, delimiter: &str) -> Option<usize> {
    let mut idx = 0;
    let bytes = slice.as_bytes();
    let delim_bytes = delimiter.as_bytes();
    let delim_len = delim_bytes.len();

    while idx < bytes.len() {
        if let Some(next_idx) = skip_code_span(slice, idx) {
            idx = next_idx;
            continue;
        }

        if bytes[idx..].starts_with(delim_bytes) && idx > 0 && !bytes[idx - 1].is_ascii_whitespace()
        {
            let is_word_char_after = idx + delim_len < bytes.len()
                && slice[idx + delim_len..]
                    .chars()
                    .next()
                    .is_some_and(|c| c.is_alphanumeric());
            if !is_word_char_after {
                return Some(idx);
            }
        }

        idx += 1;
    }

    None
}

/// Finds the end of link text, start of URL, and end of URL in a slice starting with `[`.
fn find_matching_link(slice: &str) -> Option<(usize, usize, usize)> {
    let bytes = slice.as_bytes();
    if bytes.first() != Some(&b'[') {
        return None;
    }

    let mut idx = 1;
    let mut bracket_depth = 1usize;

    while idx < bytes.len() {
        if let Some(next_idx) = skip_code_span(slice, idx) {
            idx = next_idx;
            continue;
        }

        if bytes[idx] == b'[' {
            bracket_depth += 1;
        } else if bytes[idx] == b']' {
            bracket_depth -= 1;
            if bracket_depth == 0 {
                if idx + 1 < bytes.len() && bytes[idx + 1] == b'(' {
                    let text_end = idx;
                    let url_start = idx + 2;
                    let mut url_idx = url_start;
                    let mut paren_depth = 1usize;

                    while url_idx < bytes.len() {
                        if bytes[url_idx] == b'(' {
                            paren_depth += 1;
                        } else if bytes[url_idx] == b')' {
                            paren_depth -= 1;
                            if paren_depth == 0 {
                                let url_end = url_idx;
                                return Some((text_end, url_start, url_end));
                            }
                        }
                        url_idx += 1;
                    }
                    return None;
                }
                return None;
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

        // 1. Inline code span
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

        // 2. Strikethrough
        if remaining.starts_with("~~") {
            let after_open = idx + 2;
            if let Some(close_pos) =
                find_matching_delimiter(&input[after_open..], "~~").filter(|&p| p > 0)
            {
                let inner = &input[after_open..after_open + close_pos];
                out.push_str("<s>");
                format_inline_into(out, inner);
                out.push_str("</s>");
                idx = after_open + close_pos + 2;
                continue;
            }
            out.push_str("~~");
            idx += 2;
            continue;
        }

        // 3. Bold + Italic
        if remaining.starts_with("***") {
            let after_open = idx + 3;
            if let Some(close_pos) =
                find_matching_delimiter(&input[after_open..], "***").filter(|&p| p > 0)
            {
                let inner = &input[after_open..after_open + close_pos];
                out.push_str("<strong><em>");
                format_inline_into(out, inner);
                out.push_str("</em></strong>");
                idx = after_open + close_pos + 3;
                continue;
            }
        }

        // 4. Bold
        if remaining.starts_with("**") {
            let after_open = idx + 2;
            if let Some(close_pos) =
                find_matching_delimiter(&input[after_open..], "**").filter(|&p| p > 0)
            {
                let inner = &input[after_open..after_open + close_pos];
                out.push_str("<strong>");
                format_inline_into(out, inner);
                out.push_str("</strong>");
                idx = after_open + close_pos + 2;
                continue;
            }
            out.push_str("**");
            idx += 2;
            continue;
        }

        // 5. Bold + Italic with underscores
        if remaining.starts_with("___") {
            let is_word_char_before = idx > 0
                && input[..idx]
                    .chars()
                    .next_back()
                    .is_some_and(|c| c.is_alphanumeric());
            if !is_word_char_before {
                let after_open = idx + 3;
                if let Some(close_pos) =
                    find_matching_underscore_delimiter(&input[after_open..], "___")
                        .filter(|&p| p > 0)
                {
                    let inner = &input[after_open..after_open + close_pos];
                    out.push_str("<strong><em>");
                    format_inline_into(out, inner);
                    out.push_str("</em></strong>");
                    idx = after_open + close_pos + 3;
                    continue;
                }
            }
        }

        // 6. Bold with underscores
        if remaining.starts_with("__") {
            let is_word_char_before = idx > 0
                && input[..idx]
                    .chars()
                    .next_back()
                    .is_some_and(|c| c.is_alphanumeric());
            if !is_word_char_before {
                let after_open = idx + 2;
                if let Some(close_pos) =
                    find_matching_underscore_delimiter(&input[after_open..], "__")
                        .filter(|&p| p > 0)
                {
                    let inner = &input[after_open..after_open + close_pos];
                    out.push_str("<strong>");
                    format_inline_into(out, inner);
                    out.push_str("</strong>");
                    idx = after_open + close_pos + 2;
                    continue;
                }
            }
            out.push_str("__");
            idx += 2;
            continue;
        }

        // 7. Italic
        if bytes[idx] == b'*' {
            let after_open = idx + 1;
            let not_leading_space =
                after_open < input.len() && !bytes[after_open].is_ascii_whitespace();
            let close_opt = if not_leading_space {
                find_matching_single_asterisk(&input[after_open..]).filter(|&p| p > 0)
            } else {
                None
            };
            if let Some(close_pos) = close_opt {
                let inner = &input[after_open..after_open + close_pos];
                out.push_str("<em>");
                format_inline_into(out, inner);
                out.push_str("</em>");
                idx = after_open + close_pos + 1;
                continue;
            }
            out.push('*');
            idx += 1;
            continue;
        }

        // 8. Italic with underscore
        if bytes[idx] == b'_' {
            let is_word_char_before = idx > 0
                && input[..idx]
                    .chars()
                    .next_back()
                    .is_some_and(|c| c.is_alphanumeric());
            let after_open = idx + 1;
            let not_leading_space =
                after_open < input.len() && !bytes[after_open].is_ascii_whitespace();
            let close_opt = if !is_word_char_before && not_leading_space {
                find_matching_single_underscore(&input[after_open..]).filter(|&p| p > 0)
            } else {
                None
            };
            if let Some(close_pos) = close_opt {
                let inner = &input[after_open..after_open + close_pos];
                out.push_str("<em>");
                format_inline_into(out, inner);
                out.push_str("</em>");
                idx = after_open + close_pos + 1;
                continue;
            }
            out.push('_');
            idx += 1;
            continue;
        }

        // 9. Inline link
        if bytes[idx] == b'[' {
            if let Some((text_end, url_start, url_end)) = find_matching_link(remaining) {
                let text = &remaining[1..text_end];
                let url = &remaining[url_start..url_end];
                out.push_str("<a href=\"");
                escape_html_into(out, url.trim());
                out.push_str("\">");
                format_inline_into(out, text);
                out.push_str("</a>");
                idx += url_end + 1;
                continue;
            }
            out.push('[');
            idx += 1;
            continue;
        }

        // 10. HTML Entities and regular characters
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
                        !matches!(
                            b,
                            b'`' | b'~' | b'*' | b'_' | b'[' | b'&' | b'<' | b'>' | b'"' | b'\''
                        )
                    })
                    .count();
                if plain_len > 0 {
                    out.push_str(&input[idx..idx + plain_len]);
                    idx += plain_len;
                    continue;
                }
                if let Some(ch) = input[idx..].chars().next() {
                    out.push(ch);
                    idx += ch.len_utf8();
                }
                continue;
            }
        }
        idx += 1;
    }
}

/// Appends leading whitespace indentation to a buffer based on indent level.
pub fn push_indent(out: &mut String, indent: usize) {
    for _ in 0..indent {
        out.push_str("  ");
    }
}

/// Skips an inline code span starting at the given index if present.
fn skip_code_span(slice: &str, idx: usize) -> Option<usize> {
    let bytes = slice.as_bytes();
    if bytes.get(idx) != Some(&b'`') {
        return None;
    }
    let count = bytes[idx..].iter().take_while(|&&b| b == b'`').count();
    let after_open = idx + count;
    find_matching_backticks(&slice[after_open..], count)
        .map(|close_pos| after_open + close_pos + count)
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
    fn test_format_inline_bold_and_italic() {
        let mut out = String::new();
        format_inline_into(&mut out, "This is **bold**, *italic*, and ***both***.");
        assert_eq!(
            out,
            "This is <strong>bold</strong>, <em>italic</em>, and <strong><em>both</em></strong>."
        );
    }

    #[test]
    fn test_format_inline_code() {
        let mut out = String::new();
        format_inline_into(&mut out, "Use `let x = 1;` here");
        assert_eq!(out, "Use <code>let x = 1;</code> here");
    }

    #[test]
    fn test_format_inline_code_multi_backticks() {
        let mut out = String::new();
        format_inline_into(&mut out, "Use ``let `x` = 1;`` here");
        assert_eq!(out, "Use <code>let `x` = 1;</code> here");
    }

    #[test]
    fn test_format_inline_delimiters_inside_code() {
        let mut out = String::new();
        format_inline_into(&mut out, "Run `*not italic*` and `**not bold**`");
        assert_eq!(
            out,
            "Run <code>*not italic*</code> and <code>**not bold**</code>"
        );
    }

    #[test]
    fn test_format_inline_links() {
        let mut out = String::new();
        format_inline_into(&mut out, "Visit [Doc2Flow](https://doc2flow.dev) today.");
        assert_eq!(
            out,
            "Visit <a href=\"https://doc2flow.dev\">Doc2Flow</a> today."
        );
    }

    #[test]
    fn test_format_inline_links_nested_formatting() {
        let mut out = String::new();
        format_inline_into(
            &mut out,
            "[**Bold** & *Italic* and `code`](https://example.com)",
        );
        assert_eq!(
            out,
            "<a href=\"https://example.com\"><strong>Bold</strong> &amp; <em>Italic</em> and <code>code</code></a>"
        );
    }

    #[test]
    fn test_format_inline_links_inside_bold_and_italic() {
        let mut out = String::new();
        format_inline_into(
            &mut out,
            "**[Bold Link](https://example.com)** and *[Italic Link](https://example.com)*",
        );
        assert_eq!(
            out,
            "<strong><a href=\"https://example.com\">Bold Link</a></strong> and <em><a href=\"https://example.com\">Italic Link</a></em>"
        );
    }

    #[test]
    fn test_format_inline_links_parentheses_in_url() {
        let mut out = String::new();
        format_inline_into(
            &mut out,
            "[Rust](https://en.wikipedia.org/wiki/Rust_(programming_language))",
        );
        assert_eq!(
            out,
            "<a href=\"https://en.wikipedia.org/wiki/Rust_(programming_language)\">Rust</a>"
        );
    }

    #[test]
    fn test_format_inline_links_escapes_url_and_text() {
        let mut out = String::new();
        format_inline_into(
            &mut out,
            "[Search <\"Doc & Flow\">](https://example.com/search?q=\"test\"&lang=en)",
        );
        assert_eq!(
            out,
            "<a href=\"https://example.com/search?q=&quot;test&quot;&amp;lang=en\">Search &lt;&quot;Doc &amp; Flow&quot;&gt;</a>"
        );
    }

    #[test]
    fn test_format_inline_links_code_brackets_inside_text() {
        let mut out = String::new();
        format_inline_into(&mut out, "[Index `arr[0]`](https://example.com)");
        assert_eq!(
            out,
            "<a href=\"https://example.com\">Index <code>arr[0]</code></a>"
        );
    }

    #[test]
    fn test_format_inline_links_unclosed_or_invalid() {
        let mut out = String::new();
        format_inline_into(
            &mut out,
            "[Unclosed text and [not link] text and [link] (spaced)",
        );
        assert_eq!(
            out,
            "[Unclosed text and [not link] text and [link] (spaced)"
        );
    }

    #[test]
    fn test_format_inline_nested() {
        let mut out = String::new();
        format_inline_into(
            &mut out,
            "Formatted ~~**bold strikethrough**~~ with `code`.",
        );
        assert_eq!(
            out,
            "Formatted <s><strong>bold strikethrough</strong></s> with <code>code</code>."
        );
    }

    #[test]
    fn test_format_inline_preserves_intra_word_underscores() {
        let mut out = String::new();
        format_inline_into(
            &mut out,
            "Host {{SERVER_NAME}}:{{PORT}} with key {{API_KEY}}.",
        );
        assert_eq!(out, "Host {{SERVER_NAME}}:{{PORT}} with key {{API_KEY}}.");
    }

    #[test]
    fn test_format_inline_strikethrough() {
        let mut out = String::new();
        format_inline_into(&mut out, "This is ~~strike~~ text.");
        assert_eq!(out, "This is <s>strike</s> text.");
    }

    #[test]
    fn test_format_inline_unclosed_backtick() {
        let mut out = String::new();
        format_inline_into(&mut out, "Unclosed `backtick and & symbols");
        assert_eq!(out, "Unclosed `backtick and &amp; symbols");
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
    fn test_format_inline_utf8() {
        let mut out = String::new();
        format_inline_into(&mut out, "Grüße **überall** & schön 🚀");
        assert_eq!(out, "Grüße <strong>überall</strong> &amp; schön 🚀");
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
