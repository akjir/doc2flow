//! Core vertical slice feature module.

use crate::core::document::DocumentElement;
use crate::core::feature::Feature;

/// Embedded core CSS styles for layout and components.
pub const CSS: &str = include_str!("core.css");

/// Embedded core JavaScript bundle for client runtime.
pub const JS: &str = include_str!("core.js");

/// Core feature renderer handling text, horizontal rules, and section elements.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct CoreFeature;

impl CoreFeature {
    /// Creates a new core feature instance.
    pub const fn new() -> Self {
        Self
    }
}

impl Feature for CoreFeature {
    /// Converts a document element and inner content into an HTML string representation.
    fn to_html(&self, element: &DocumentElement, content: &str, indent: usize) -> String {
        match element {
            DocumentElement::HorizontalRule => {
                let spaces = indent * 2;
                let mut out = String::with_capacity(spaces + 8);
                push_indent(&mut out, indent);
                out.push_str("<hr />\n");
                out
            }
            DocumentElement::Section { level, title, .. } => {
                let spaces = indent * 2;
                let inner_spaces = (indent + 1) * 2;
                let mut out = String::with_capacity(
                    title.len() + content.len() + spaces * 2 + inner_spaces * 3 + 128,
                );
                push_indent(&mut out, indent);
                out.push_str("<section class=\"section\" data-level=\"");
                out.push_str(&level.to_string());
                out.push_str("\">\n");

                push_indent(&mut out, indent + 1);
                out.push_str("<h");
                out.push_str(&level.to_string());
                out.push('>');
                out.push_str(title);
                out.push_str("</h");
                out.push_str(&level.to_string());
                out.push_str(">\n");

                push_indent(&mut out, indent + 1);
                out.push_str("<div class=\"section-body\">\n");

                out.push_str(content);

                push_indent(&mut out, indent + 1);
                out.push_str("</div>\n");

                push_indent(&mut out, indent);
                out.push_str("</section>\n");
                out
            }
            DocumentElement::Text(text) => {
                let spaces = indent * 2;
                let inner_spaces = (indent + 1) * 2;
                let mut out =
                    String::with_capacity(text.len() * 2 + spaces * 2 + inner_spaces * 2 + 64);
                push_indent(&mut out, indent);
                out.push_str("<div class=\"item item-text\">\n");
                push_indent(&mut out, indent + 1);
                out.push_str("<span class=\"text-content\">\n");
                for line in text.lines() {
                    push_indent(&mut out, indent + 2);
                    format_inline_into(&mut out, line);
                    out.push('\n');
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

    /// Returns the embedded CSS stylesheet for the core feature.
    fn css(&self) -> Option<&'static str> {
        Some(CSS)
    }

    /// Returns the embedded JavaScript client script for the core feature.
    fn javascript(&self) -> Option<&'static str> {
        Some(JS)
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
    fn test_core_feature_css() {
        let feature = CoreFeature::new();
        let css = feature.css().expect("core css should exist");
        assert!(css.contains("--bg-body:"));
        assert!(css.contains("--code-bg:"));
        assert!(css.contains("--code-border:"));
        assert!(css.contains("--code-color:"));
        assert!(css.contains("--item-hover-bg:"));
        assert!(css.contains("--item-done-bg:"));
        assert!(css.contains(".doc-body"));
        assert!(css.contains(".item"));
        assert!(css.contains(".text-content"));
        assert!(css.contains(".txt-default"));
        assert!(css.contains(".txt-code"));
        assert!(css.contains(".txt-strike"));
        assert!(css.contains(".section"));
        assert!(css.contains(".section-header"));
        assert!(css.contains(".section-body"));
        assert!(css.contains(".section-subheading"));
        assert!(css.contains("--section-bg-header:"));
        assert!(css.contains("hr {"));
    }

    #[test]
    fn test_core_feature_javascript() {
        let feature = CoreFeature::new();
        let js = feature.javascript().expect("core javascript should exist");
        assert!(js.contains("window.d2f"));
        assert!(js.contains("core"));
    }

    #[test]
    fn test_core_feature_empty_for_unsupported_elements() {
        let feature = CoreFeature::new();
        let code = DocumentElement::code_block(Some("rust"), "fn main() {}");
        assert_eq!(feature.to_html(&code, "", 0), "");
    }

    #[test]
    fn test_core_feature_escapes_plain_text_html_entities() {
        let feature = CoreFeature::new();
        let elem = DocumentElement::text("5 < 10 & 20 > 15 \"quoted\" 'single'");
        assert_eq!(
            feature.to_html(&elem, "", 0),
            "<div class=\"item item-text\">\n  <span class=\"text-content\">\n    5 &lt; 10 &amp; 20 &gt; 15 &quot;quoted&quot; &#39;single&#39;\n  </span>\n</div>\n"
        );
    }

    #[test]
    fn test_core_feature_inline_code_escapes_html_and_preserves_literals() {
        let feature = CoreFeature::new();
        let elem = DocumentElement::text("Example `<div class=\"box\"> && **not bold**</div>` here.");
        assert_eq!(
            feature.to_html(&elem, "", 0),
            "<div class=\"item item-text\">\n  <span class=\"text-content\">\n    Example <code>&lt;div class=&quot;box&quot;&gt; &amp;&amp; **not bold**&lt;/div&gt;</code> here.\n  </span>\n</div>\n"
        );
    }

    #[test]
    fn test_core_feature_nested_formatting() {
        let feature = CoreFeature::new();
        let elem = DocumentElement::text("Formatted ~~**bold strikethrough**~~ with `code`.");
        assert_eq!(
            feature.to_html(&elem, "", 0),
            "<div class=\"item item-text\">\n  <span class=\"text-content\">\n    Formatted <s><strong>bold strikethrough</strong></s> with <code>code</code>.\n  </span>\n</div>\n"
        );
    }

    #[test]
    fn test_core_feature_preserves_intra_word_underscores() {
        let feature = CoreFeature::new();
        let elem = DocumentElement::text("Host {{SERVER_NAME}}:{{PORT}} with key {{API_KEY}}.");
        assert_eq!(
            feature.to_html(&elem, "", 0),
            "<div class=\"item item-text\">\n  <span class=\"text-content\">\n    Host {{SERVER_NAME}}:{{PORT}} with key {{API_KEY}}.\n  </span>\n</div>\n"
        );
    }

    #[test]
    fn test_core_feature_renders_bold_and_italic_combined() {
        let feature = CoreFeature::new();
        let elem = DocumentElement::text("This is ***bold and italic*** text.");
        assert_eq!(
            feature.to_html(&elem, "", 0),
            "<div class=\"item item-text\">\n  <span class=\"text-content\">\n    This is <strong><em>bold and italic</em></strong> text.\n  </span>\n</div>\n"
        );

        let elem_underscores = DocumentElement::text("This is ___bold and italic___ text.");
        assert_eq!(
            feature.to_html(&elem_underscores, "", 0),
            "<div class=\"item item-text\">\n  <span class=\"text-content\">\n    This is <strong><em>bold and italic</em></strong> text.\n  </span>\n</div>\n"
        );
    }

    #[test]
    fn test_core_feature_renders_bold_asterisks_and_underscores() {
        let feature = CoreFeature::new();
        let elem_asterisk = DocumentElement::text("This is **bold** text.");
        assert_eq!(
            feature.to_html(&elem_asterisk, "", 0),
            "<div class=\"item item-text\">\n  <span class=\"text-content\">\n    This is <strong>bold</strong> text.\n  </span>\n</div>\n"
        );

        let elem_underscore = DocumentElement::text("This is __bold__ text.");
        assert_eq!(
            feature.to_html(&elem_underscore, "", 0),
            "<div class=\"item item-text\">\n  <span class=\"text-content\">\n    This is <strong>bold</strong> text.\n  </span>\n</div>\n"
        );
    }

    #[test]
    fn test_core_feature_renders_horizontal_rule() {
        let feature = CoreFeature::new();
        let element = DocumentElement::horizontal_rule();
        assert_eq!(feature.to_html(&element, "", 0), "<hr />\n");
        assert_eq!(feature.to_html(&element, "", 1), "  <hr />\n");
        assert_eq!(feature.to_html(&element, "", 2), "    <hr />\n");
    }

    #[test]
    fn test_core_feature_renders_inline_code_and_strips_backticks() {
        let feature = CoreFeature::new();
        let elem = DocumentElement::text("Run `cargo test --all` now.");
        assert_eq!(
            feature.to_html(&elem, "", 0),
            "<div class=\"item item-text\">\n  <span class=\"text-content\">\n    Run <code>cargo test --all</code> now.\n  </span>\n</div>\n"
        );
    }

    #[test]
    fn test_core_feature_renders_italic_asterisks_and_underscores() {
        let feature = CoreFeature::new();
        let elem_asterisk = DocumentElement::text("This is *italic* text.");
        assert_eq!(
            feature.to_html(&elem_asterisk, "", 0),
            "<div class=\"item item-text\">\n  <span class=\"text-content\">\n    This is <em>italic</em> text.\n  </span>\n</div>\n"
        );

        let elem_underscore = DocumentElement::text("This is _italic_ text.");
        assert_eq!(
            feature.to_html(&elem_underscore, "", 0),
            "<div class=\"item item-text\">\n  <span class=\"text-content\">\n    This is <em>italic</em> text.\n  </span>\n</div>\n"
        );
    }

    #[test]
    fn test_core_feature_renders_plain_text() {
        let feature = CoreFeature::new();
        let element = DocumentElement::text("Hello, world!");
        assert_eq!(
            feature.to_html(&element, "", 1),
            "  <div class=\"item item-text\">\n    <span class=\"text-content\">\n      Hello, world!\n    </span>\n  </div>\n"
        );
    }

    #[test]
    fn test_core_feature_renders_section_with_content() {
        let feature = CoreFeature::new();
        let section = DocumentElement::section(
            1,
            "Overview",
            vec![DocumentElement::text("Section body content")],
        );
        let child_html =
            "        <div class=\"item item-text\">\n          <span class=\"text-content\">\n            Section body content\n          </span>\n        </div>\n";
        let html = feature.to_html(&section, child_html, 2);
        let expected = concat!(
            "    <section class=\"section\" data-level=\"1\">\n",
            "      <h1>Overview</h1>\n",
            "      <div class=\"section-body\">\n",
            "        <div class=\"item item-text\">\n",
            "          <span class=\"text-content\">\n",
            "            Section body content\n",
            "          </span>\n",
            "        </div>\n",
            "      </div>\n",
            "    </section>\n"
        );
        assert_eq!(html, expected);
    }

    #[test]
    fn test_core_feature_renders_strikethrough() {
        let feature = CoreFeature::new();
        let elem = DocumentElement::text("Replaces ~~legacy procedures~~ with modern.");
        assert_eq!(
            feature.to_html(&elem, "", 0),
            "<div class=\"item item-text\">\n  <span class=\"text-content\">\n    Replaces <s>legacy procedures</s> with modern.\n  </span>\n</div>\n"
        );
    }

    #[test]
    fn test_core_feature_unclosed_delimiters() {
        let feature = CoreFeature::new();
        let elem = DocumentElement::text("Unclosed **bold and ~~strike and `code");
        assert_eq!(
            feature.to_html(&elem, "", 0),
            "<div class=\"item item-text\">\n  <span class=\"text-content\">\n    Unclosed **bold and ~~strike and `code\n  </span>\n</div>\n"
        );
    }
}

