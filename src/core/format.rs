//! Inline markdown formatting and HTML escaping engine.

/// Appends multiline text to a buffer, indenting each non-empty line.
pub fn append_indented(out: &mut String, text: &str) {
    for line in text.lines() {
        if !line.is_empty() {
            out.push_str("    ");
            out.push_str(line);
        }
        out.push('\n');
    }
}

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

/// Scans forward skipping code spans until a predicate matches.
fn find_end_delimiter<F>(slice: &str, mut predicate: F) -> Option<usize>
where
    F: FnMut(&str, usize) -> bool,
{
    let mut idx = 0;
    let bytes = slice.as_bytes();

    while idx < bytes.len() {
        if let Some(next_idx) = skip_code_span(slice, idx) {
            idx = next_idx;
            continue;
        }

        if predicate(slice, idx) {
            return Some(idx);
        }

        idx += 1;
    }

    None
}

/// Finds the index of a matching sequence of backticks in a slice.
fn find_matching_backticks(slice: &str, count: usize) -> Option<usize> {
    let mut idx = 0;
    let bytes = slice.as_bytes();

    while idx < bytes.len() {
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
    let delim_bytes = delimiter.as_bytes();
    find_end_delimiter(slice, |s, idx| s.as_bytes()[idx..].starts_with(delim_bytes))
}

/// Finds link text end, URL start, and URL end in a markdown link slice.
fn find_matching_link(slice: &str) -> Option<(usize, usize, usize)> {
    let bytes = slice.as_bytes();
    if bytes.first() != Some(&b'[') {
        return None;
    }
    if !bytes.contains(&b']') || !bytes.contains(&b')') {
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

/// Finds the index of a matching single asterisk delimiter.
fn find_matching_single_asterisk(slice: &str) -> Option<usize> {
    let bytes = slice.as_bytes();
    find_end_delimiter(slice, |_s, idx| {
        if bytes[idx] != b'*' {
            return false;
        }
        let is_double = (idx + 1 < bytes.len() && bytes[idx + 1] == b'*')
            || (idx > 0 && bytes[idx - 1] == b'*');
        !is_double && !is_whitespace_before(slice, idx)
    })
}

/// Finds the index of a matching single underscore delimiter.
fn find_matching_single_underscore(slice: &str) -> Option<usize> {
    let bytes = slice.as_bytes();
    find_end_delimiter(slice, |s, idx| {
        if bytes[idx] != b'_' {
            return false;
        }
        let is_double = (idx + 1 < bytes.len() && bytes[idx + 1] == b'_')
            || (idx > 0 && bytes[idx - 1] == b'_');
        !is_double && !is_whitespace_before(s, idx) && !is_alphanumeric_after(s, idx + 1)
    })
}

/// Finds the index of a matching underscore delimiter with boundary checks.
fn find_matching_underscore_delimiter(slice: &str, delimiter: &str) -> Option<usize> {
    let delim_bytes = delimiter.as_bytes();
    let delim_len = delim_bytes.len();
    find_end_delimiter(slice, |s, idx| {
        s.as_bytes()[idx..].starts_with(delim_bytes)
            && !is_whitespace_before(s, idx)
            && !is_alphanumeric_after(s, idx + delim_len)
    })
}

/// Formats inline markdown elements into an HTML output buffer.
pub fn format_inline_into(out: &mut String, input: &str) {
    let mut idx = 0;
    let bytes = input.as_bytes();

    while idx < input.len() {
        let consumed = match bytes[idx] {
            b'`' => try_parse_code(out, input, idx),
            b'~' => try_parse_strikethrough(out, input, idx),
            b'*' => try_parse_asterisks(out, input, idx),
            b'_' => try_parse_underscores(out, input, idx),
            b'[' => try_parse_link(out, input, idx),
            _ => None,
        };

        if let Some(n) = consumed {
            idx += n;
            continue;
        }

        match bytes[idx] {
            b'&' => {
                out.push_str("&amp;");
                idx += 1;
            }
            b'<' => {
                out.push_str("&lt;");
                idx += 1;
            }
            b'>' => {
                out.push_str("&gt;");
                idx += 1;
            }
            b'"' => {
                out.push_str("&quot;");
                idx += 1;
            }
            b'\'' => {
                out.push_str("&#39;");
                idx += 1;
            }
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
                } else if let Some(ch) = input[idx..].chars().next() {
                    out.push(ch);
                    idx += ch.len_utf8();
                } else {
                    break;
                }
            }
        }
    }
}

/// Returns true if character immediately following offset is alphanumeric.
fn is_alphanumeric_after(slice: &str, idx: usize) -> bool {
    idx < slice.len()
        && slice[idx..]
            .chars()
            .next()
            .is_some_and(|c| c.is_alphanumeric())
}

/// Returns true if character immediately preceding offset is alphanumeric.
fn is_alphanumeric_before(slice: &str, idx: usize) -> bool {
    idx > 0
        && slice[..idx]
            .chars()
            .next_back()
            .is_some_and(|c| c.is_alphanumeric())
}

/// Returns true if byte immediately preceding offset is ASCII whitespace.
fn is_whitespace_before(slice: &str, idx: usize) -> bool {
    idx > 0
        && slice
            .as_bytes()
            .get(idx - 1)
            .is_some_and(|b| b.is_ascii_whitespace())
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

/// Attempts to parse asterisk emphasis tags and writes HTML to output.
fn try_parse_asterisks(out: &mut String, input: &str, idx: usize) -> Option<usize> {
    let remaining = &input[idx..];

    // 1. Bold + Italic (***)
    if remaining.starts_with("***") {
        let after_open = idx + 3;
        if let Some(close_pos) =
            find_matching_delimiter(&input[after_open..], "***").filter(|&p| p > 0)
        {
            let inner = &input[after_open..after_open + close_pos];
            out.push_str("<strong><em>");
            format_inline_into(out, inner);
            out.push_str("</em></strong>");
            return Some(close_pos + 6);
        }
    }

    // 2. Bold (**)
    if remaining.starts_with("**") {
        let after_open = idx + 2;
        if let Some(close_pos) =
            find_matching_delimiter(&input[after_open..], "**").filter(|&p| p > 0)
        {
            let inner = &input[after_open..after_open + close_pos];
            out.push_str("<strong>");
            format_inline_into(out, inner);
            out.push_str("</strong>");
            return Some(close_pos + 4);
        }
    }

    // 3. Italic (*)
    if remaining.starts_with('*') {
        let after_open = idx + 1;
        let not_leading_space =
            after_open < input.len() && !input.as_bytes()[after_open].is_ascii_whitespace();
        if not_leading_space
            && let Some(close_pos) =
                find_matching_single_asterisk(&input[after_open..]).filter(|&p| p > 0)
        {
            let inner = &input[after_open..after_open + close_pos];
            out.push_str("<em>");
            format_inline_into(out, inner);
            out.push_str("</em>");
            return Some(close_pos + 2);
        }
    }

    None
}

/// Attempts to parse an inline code span and writes HTML to output.
fn try_parse_code(out: &mut String, input: &str, idx: usize) -> Option<usize> {
    let bytes = input.as_bytes();
    if bytes.get(idx) != Some(&b'`') {
        return None;
    }
    let count = bytes[idx..].iter().take_while(|&&b| b == b'`').count();
    let after_open = idx + count;
    if let Some(close_pos) = find_matching_backticks(&input[after_open..], count) {
        let code_content = &input[after_open..after_open + close_pos];
        out.push_str("<code>");
        escape_html_into(out, code_content);
        out.push_str("</code>");
        Some(count + close_pos + count)
    } else {
        escape_html_into(out, &input[idx..after_open]);
        Some(count)
    }
}

/// Attempts to parse an inline markdown link and writes HTML to output.
fn try_parse_link(out: &mut String, input: &str, idx: usize) -> Option<usize> {
    let remaining = &input[idx..];
    if let Some((text_end, url_start, url_end)) = find_matching_link(remaining) {
        let text = &remaining[1..text_end];
        let url = &remaining[url_start..url_end];
        out.push_str("<a href=\"");
        escape_html_into(out, url.trim());
        out.push_str("\">");
        format_inline_into(out, text);
        out.push_str("</a>");
        Some(url_end + 1)
    } else {
        None
    }
}

/// Attempts to parse a strikethrough span and writes HTML to output.
fn try_parse_strikethrough(out: &mut String, input: &str, idx: usize) -> Option<usize> {
    let remaining = &input[idx..];
    if !remaining.starts_with("~~") {
        return None;
    }
    let after_open = idx + 2;
    if let Some(close_pos) = find_matching_delimiter(&input[after_open..], "~~").filter(|&p| p > 0)
    {
        let inner = &input[after_open..after_open + close_pos];
        out.push_str("<s>");
        format_inline_into(out, inner);
        out.push_str("</s>");
        Some(close_pos + 4)
    } else {
        None
    }
}

/// Attempts to parse underscore emphasis tags and writes HTML to output.
fn try_parse_underscores(out: &mut String, input: &str, idx: usize) -> Option<usize> {
    if is_alphanumeric_before(input, idx) {
        return None;
    }
    let remaining = &input[idx..];

    // 1. Bold + Italic (___)
    if remaining.starts_with("___") {
        let after_open = idx + 3;
        if let Some(close_pos) =
            find_matching_underscore_delimiter(&input[after_open..], "___").filter(|&p| p > 0)
        {
            let inner = &input[after_open..after_open + close_pos];
            out.push_str("<strong><em>");
            format_inline_into(out, inner);
            out.push_str("</em></strong>");
            return Some(close_pos + 6);
        }
    }

    // 2. Bold (__)
    if remaining.starts_with("__") {
        let after_open = idx + 2;
        if let Some(close_pos) =
            find_matching_underscore_delimiter(&input[after_open..], "__").filter(|&p| p > 0)
        {
            let inner = &input[after_open..after_open + close_pos];
            out.push_str("<strong>");
            format_inline_into(out, inner);
            out.push_str("</strong>");
            return Some(close_pos + 4);
        }
    }

    // 3. Italic (_)
    if remaining.starts_with('_') {
        let after_open = idx + 1;
        let not_leading_space =
            after_open < input.len() && !input.as_bytes()[after_open].is_ascii_whitespace();
        if not_leading_space
            && let Some(close_pos) =
                find_matching_single_underscore(&input[after_open..]).filter(|&p| p > 0)
        {
            let inner = &input[after_open..after_open + close_pos];
            out.push_str("<em>");
            format_inline_into(out, inner);
            out.push_str("</em>");
            return Some(close_pos + 2);
        }
    }

    None
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
    fn test_format_inline_malformed_links() {
        let mut out = String::new();
        format_inline_into(&mut out, "[link without close paren](https://example.com");
        assert_eq!(out, "[link without close paren](https://example.com");

        out.clear();
        format_inline_into(&mut out, "[link without url](");
        assert_eq!(out, "[link without url](");

        out.clear();
        format_inline_into(&mut out, "[empty url]()");
        assert_eq!(out, "<a href=\"\">empty url</a>");
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
    fn test_format_inline_empty_emphasis() {
        let mut out = String::new();
        format_inline_into(&mut out, "****");
        assert_eq!(out, "****");

        out.clear();
        format_inline_into(&mut out, "____");
        assert_eq!(out, "____");

        out.clear();
        format_inline_into(&mut out, "~~~~");
        assert_eq!(out, "~~~~");

        out.clear();
        format_inline_into(&mut out, "******");
        assert_eq!(out, "******");

        out.clear();
        format_inline_into(&mut out, "______");
        assert_eq!(out, "______");

        out.clear();
        format_inline_into(&mut out, "**");
        assert_eq!(out, "**");

        out.clear();
        format_inline_into(&mut out, "__");
        assert_eq!(out, "__");

        out.clear();
        format_inline_into(&mut out, "~~");
        assert_eq!(out, "~~");
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
    fn test_format_inline_utf8_boundaries_adjacent_underscores() {
        let mut out = String::new();
        format_inline_into(&mut out, "ä_ö_ü");
        assert_eq!(out, "ä_ö_ü");

        out.clear();
        format_inline_into(&mut out, " _ü_ ");
        assert_eq!(out, " <em>ü</em> ");

        out.clear();
        format_inline_into(&mut out, " __ä__ ");
        assert_eq!(out, " <strong>ä</strong> ");

        out.clear();
        format_inline_into(&mut out, "🚀_test_🚀");
        assert_eq!(out, "🚀<em>test</em>🚀");

        out.clear();
        format_inline_into(&mut out, "🎉**bold**🎉");
        assert_eq!(out, "🎉<strong>bold</strong>🎉");
    }

    #[test]
    fn test_format_inline_deeply_nested_and_adversarial() {
        let mut out = String::new();
        format_inline_into(&mut out, "**~~*nested*~~**");
        assert_eq!(out, "<strong><s><em>nested</em></s></strong>");

        out.clear();
        let unclosed = "[".repeat(50);
        format_inline_into(&mut out, &unclosed);
        assert_eq!(out, unclosed);

        out.clear();
        let adversarial = "[[[[[[[[[[[[not a link";
        format_inline_into(&mut out, adversarial);
        assert_eq!(out, adversarial);
    }

    #[test]
    fn test_append_indented() {
        let mut out = String::new();
        append_indented(&mut out, "first line\n\nsecond line");
        assert_eq!(out, "    first line\n\n    second line\n");
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
