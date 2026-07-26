use crate::error::DiagnosticError;
use crate::i18n::Locale;
use anyhow::Result;
use pulldown_cmark::{
    CodeBlockKind, Event, HeadingLevel, Options, Parser as MarkdownParser, Tag, TagEnd, html,
};

/// Escapes HTML special characters in code strings.
fn html_escape(input: &str) -> String {
    let mut escaped = String::with_capacity(input.len());
    for ch in input.chars() {
        match ch {
            '&' => escaped.push_str("&amp;"),
            '<' => escaped.push_str("&lt;"),
            '>' => escaped.push_str("&gt;"),
            '"' => escaped.push_str("&quot;"),
            _ => escaped.push(ch),
        }
    }
    escaped
}

/// Frontmatter metadata extracted from Markdown header.
#[derive(Debug, Clone, Copy)]
enum ListKind {
    Unordered,
    Ordered { current: u64 },
}

/// Converts a 1-based number to alphabetic representation (1 -> a, 2 -> b, ..., 26 -> z).
fn to_alpha(mut n: u64) -> String {
    if n == 0 {
        return "a".to_string();
    }
    let mut result = String::new();
    while n > 0 {
        n -= 1;
        let rem = (n % 26) as u8;
        result.insert(0, (b'a' + rem) as char);
        n /= 26;
    }
    result
}

/// Converts a 1-based number to lowercase Roman numerals (1 -> i, 2 -> ii, 3 -> iii...).
fn to_roman(n: u64) -> String {
    let mapping = [
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
    let mut result = String::new();
    for (val, sym) in mapping {
        while num >= val {
            result.push_str(sym);
            num -= val;
        }
    }
    if result.is_empty() {
        "i".to_string()
    } else {
        result
    }
}

/// Formats bullet symbol or ordered number based on list kind and nesting depth.
fn format_bullet(kind: &mut ListKind, depth: usize) -> String {
    match kind {
        ListKind::Ordered { current } => {
            let num = *current;
            *current += 1;
            match depth % 3 {
                0 => format!("{num}."),
                1 => format!("{}.", to_alpha(num)),
                _ => format!("{}.", to_roman(num)),
            }
        }
        ListKind::Unordered => "&bull;".to_string(),
    }
}

#[derive(Debug, Default)]
pub struct Frontmatter {
    pub title: String,
    pub subtitle: String,
    pub company: String,
    pub contact: String,
    pub agent: String,
    pub date: String,
    pub version: String,
    pub language: String,
}

/// Finds the character indices for frontmatter block delimiters `---`.
fn find_frontmatter_bounds(md_content: &str) -> Option<(usize, usize, usize, usize)> {
    for (start_idx, _) in md_content.match_indices("---") {
        if start_idx == 0 || md_content[..start_idx].ends_with('\n') {
            let prefix = md_content[..start_idx].trim();
            let is_valid_prefix =
                prefix.is_empty() || (prefix.starts_with("<!--") && prefix.ends_with("-->"));

            if is_valid_prefix {
                let after_first = start_idx + 3;
                let rest = &md_content[after_first..];
                let content_start = rest
                    .strip_prefix("\r\n")
                    .or_else(|| rest.strip_prefix("\n"))
                    .map(|s| md_content.len() - s.len())
                    .unwrap_or(after_first);

                if let Some((close_rel_idx, _)) = md_content[content_start..]
                    .match_indices("---")
                    .find(|(idx, _)| {
                        let abs_idx = content_start + idx;
                        (abs_idx == 0 || md_content[..abs_idx].ends_with('\n'))
                            && (md_content[abs_idx + 3..].starts_with("\r\n")
                                || md_content[abs_idx + 3..].starts_with('\n')
                                || md_content[abs_idx + 3..].is_empty())
                    })
                {
                    let close_idx = content_start + close_rel_idx;
                    let after_close = close_idx + 3;
                    let body_start = md_content[after_close..]
                        .strip_prefix("\r\n")
                        .or_else(|| md_content[after_close..].strip_prefix("\n"))
                        .map(|s| md_content.len() - s.len())
                        .unwrap_or(after_close);

                    return Some((start_idx, content_start, close_idx, body_start));
                }
            }
        }
    }
    None
}

/// Parses YAML-style frontmatter delimited by `---`.
pub fn parse_frontmatter(md_content: &str) -> (Frontmatter, &str) {
    let mut fm = Frontmatter::default();

    if let Some((_, content_start, close_idx, body_start)) = find_frontmatter_bounds(md_content) {
        let frontmatter_text = &md_content[content_start..close_idx];

        for line in frontmatter_text.lines() {
            if let Some((key, val)) = line.split_once(':') {
                let key = key.trim();
                let val = val.trim().trim_matches('"').trim_matches('\'');
                match key {
                    "title" => fm.title = val.to_string(),
                    "subtitle" => fm.subtitle = val.to_string(),
                    "company" => fm.company = val.to_string(),
                    "contact" => fm.contact = val.to_string(),
                    "agent" => fm.agent = val.to_string(),
                    "date" => fm.date = val.to_string(),
                    "version" => fm.version = val.to_string(),
                    "language" | "lang" => fm.language = val.to_string(),
                    _ => {}
                }
            }
        }

        return (fm, &md_content[body_start..]);
    }

    (fm, md_content)
}

/// Validates frontmatter metadata for required fields, returning a compiler-style diagnostic error if invalid.
///
/// Ensures the `company` field is present and non-empty.
///
/// # Errors
///
/// Returns an error formatted like a Rust compiler diagnostic if `company` is missing or empty.
pub fn validate_frontmatter(
    frontmatter: &Frontmatter,
    md_content: &str,
    file_name: Option<&str>,
) -> Result<()> {
    if !frontmatter.company.trim().is_empty() {
        return Ok(());
    }

    let file_path = file_name.unwrap_or("input.md");

    if let Some((start_idx, content_start, close_idx, _)) = find_frontmatter_bounds(md_content) {
        let frontmatter_text = &md_content[content_start..close_idx];
        let mut company_line_info = None;

        for (idx, line) in frontmatter_text.lines().enumerate() {
            if matches!(line.split_once(':'), Some((key, _)) if key.trim() == "company") {
                let line_no = md_content[..content_start].lines().count() + idx + 1;
                company_line_info = Some((line_no, line.to_string()));
                break;
            }
        }

        if let Some((line_no, line_content)) = company_line_info {
            Err(DiagnosticError::empty_frontmatter_field(
                file_path,
                line_no,
                &line_content,
            ))
        } else {
            let start_line = (md_content[..start_idx].lines().count() + 1).max(1);
            Err(DiagnosticError::missing_frontmatter_field(
                file_path, start_line,
            ))
        }
    } else {
        let first_line = md_content.lines().next().unwrap_or("");
        Err(DiagnosticError::missing_frontmatter_block(
            file_path, first_line,
        ))
    }
}

/// Parses and validates YAML frontmatter from Markdown content.
///
/// # Errors
///
/// Returns a compiler-style diagnostic error if required fields like `company` are missing or empty.
pub fn parse_and_validate_frontmatter<'a>(
    md_content: &'a str,
    file_name: Option<&str>,
) -> Result<(Frontmatter, &'a str)> {
    let (fm, body) = parse_frontmatter(md_content);
    validate_frontmatter(&fm, md_content, file_name)?;
    Ok((fm, body))
}

/// Parses callout metadata (CSS class, inner text, callout label) from raw blockquote inner string.
fn parse_callout<'a>(inner: &'a str, locale: &'a Locale) -> (&'static str, &'a str, &'a str) {
    let prefixes: &[(&str, &'static str, &str)] = &[
        ("!!! ", "note note-caution", locale.get("callout_caution")),
        ("!!!", "note note-caution", locale.get("callout_caution")),
        ("!! ", "note note-warning", locale.get("callout_warning")),
        ("!!", "note note-warning", locale.get("callout_warning")),
        ("! ", "note note-important", locale.get("callout_important")),
        ("!", "note note-important", locale.get("callout_important")),
        ("? ", "note note-tip", locale.get("callout_tip")),
        ("?", "note note-tip", locale.get("callout_tip")),
    ];

    for &(prefix, css_class, label) in prefixes {
        if let Some(stripped) = inner.strip_prefix(prefix) {
            return (css_class, stripped, label);
        }
    }

    ("note", inner, locale.get("callout_note"))
}

/// Converts Markdown body into interactive HTML following doc2flow structure using default English locale.
/// Filters out HTML comments (both single-line and multiline) from a vector of Markdown events.
fn filter_comment_events(events: Vec<Event>) -> Vec<Event> {
    let mut filtered = Vec::with_capacity(events.len());
    let mut in_comment = false;

    for ev in events {
        match &ev {
            Event::Html(text) | Event::InlineHtml(text) => {
                let s = text.as_ref();
                if in_comment {
                    if s.contains("-->") {
                        in_comment = false;
                    }
                    continue;
                } else if let Some(start_idx) = s.find("<!--") {
                    if let Some(end_idx) = s[start_idx..].find("-->") {
                        let _ = end_idx;
                        // Single-line or self-contained comment in this event
                        continue;
                    } else {
                        // Start of multiline comment
                        in_comment = true;
                        continue;
                    }
                }
            }
            _ => {
                if in_comment {
                    continue;
                }
            }
        }
        filtered.push(ev);
    }

    filtered
}

/// Strips enclosing `<p>` and `</p>` HTML tags if present.
#[inline]
fn strip_paragraph_tags(input: &str) -> &str {
    input
        .strip_prefix("<p>")
        .and_then(|s| s.strip_suffix("</p>"))
        .unwrap_or(input)
}

/// Converts Markdown body into interactive HTML following doc2flow structure using default English locale.
pub fn convert_markdown_to_html(markdown_body: &str) -> Result<String> {
    convert_markdown_to_html_with_locale(markdown_body, &Locale::default())
}

/// Converts Markdown body into interactive HTML following doc2flow structure using specified locale.
pub fn convert_markdown_to_html_with_locale(
    markdown_body: &str,
    locale: &Locale,
) -> Result<String> {
    let mut options = Options::empty();
    options.insert(Options::ENABLE_TASKLISTS);
    options.insert(Options::ENABLE_STRIKETHROUGH);
    options.insert(Options::ENABLE_TABLES);

    let parser = MarkdownParser::new_ext(markdown_body, options);
    let events: Vec<Event> = filter_comment_events(parser.collect());

    let mut out = String::with_capacity(markdown_body.len() * 2);
    let mut section_count = 0usize;
    let mut global_cb_count = 0usize;
    let mut global_txt_count = 0usize;
    let mut global_item_count = 0usize;
    let mut in_section = false;
    let mut list_stack: Vec<ListKind> = Vec::new();

    let mut idx = 0;
    while idx < events.len() {
        match &events[idx] {
            // Level 2 Headings (## Section)
            Event::Start(Tag::Heading {
                level: HeadingLevel::H2,
                ..
            }) => {
                if in_section {
                    out.push_str("</div></div>\n\n");
                }
                section_count += 1;
                in_section = true;

                // Collect heading content until End(Heading)
                let mut heading_events = Vec::new();
                idx += 1;
                while idx < events.len() {
                    if matches!(events[idx], Event::End(TagEnd::Heading(HeadingLevel::H2))) {
                        break;
                    }
                    heading_events.push(events[idx].clone());
                    idx += 1;
                }

                let mut heading_html = String::new();
                html::push_html(&mut heading_html, heading_events.into_iter());
                let heading_text = heading_html.trim();

                out.push_str(&format!("<!-- S{section_count} -->\n"));
                out.push_str(&format!(
                    "<div class=\"section\" id=\"s{section_count}\">\n"
                ));
                out.push_str(&format!(
                    "<div class=\"sh\" onclick=\"toggleSection('s{section_count}')\"><span>{heading_text}</span>\n"
                ));
                out.push_str(&format!(
                    "<div style=\"display:flex;align-items:center;gap:8px\"><span class=\"sbadge\" id=\"badge-s{section_count}\"></span><span class=\"stog\" id=\"tog-s{section_count}\">&#9660;</span></div></div>\n"
                ));
                out.push_str(&format!(
                    "<div class=\"sb\" id=\"body-s{section_count}\">\n"
                ));
            }

            // Level 3 Headings (### Subheading)
            Event::Start(Tag::Heading {
                level: HeadingLevel::H3,
                ..
            }) => {
                let mut sub_events = Vec::new();
                idx += 1;
                while idx < events.len() {
                    if matches!(events[idx], Event::End(TagEnd::Heading(HeadingLevel::H3))) {
                        break;
                    }
                    sub_events.push(events[idx].clone());
                    idx += 1;
                }

                let mut sub_html = String::new();
                html::push_html(&mut sub_html, sub_events.into_iter());
                out.push_str(&format!("<div class=\"subh\">{}</div>\n", sub_html.trim()));
            }

            // Blockquotes (> Note, >? Tip, >! Important, >!! Warning, >!!! Caution)
            Event::Start(Tag::BlockQuote(_)) => {
                let mut bq_events = Vec::new();
                idx += 1;
                let mut depth = 1;
                while idx < events.len() {
                    match &events[idx] {
                        Event::Start(Tag::BlockQuote(_)) => depth += 1,
                        Event::End(TagEnd::BlockQuote(_)) => {
                            depth -= 1;
                            if depth == 0 {
                                break;
                            }
                        }
                        _ => {}
                    }
                    bq_events.push(events[idx].clone());
                    idx += 1;
                }

                let mut bq_html = String::new();
                html::push_html(&mut bq_html, bq_events.into_iter());
                let trimmed = bq_html.trim();

                let inner = strip_paragraph_tags(trimmed);
                let (note_cls, note_content, callout_label) = parse_callout(inner, locale);

                let escaped_label = html_escape(callout_label);
                out.push_str(&format!(
                    "<div class=\"{}\" data-label=\"{}\">{}</div>\n",
                    note_cls,
                    escaped_label,
                    note_content.trim()
                ));
            }

            // Code Blocks (e.g. ```ini ... ```)
            Event::Start(Tag::CodeBlock(kind)) => {
                let lang_opt = match kind {
                    CodeBlockKind::Fenced(lang) if !lang.is_empty() => Some(lang.to_string()),
                    _ => None,
                };

                let mut code_events = Vec::new();
                idx += 1;
                let mut depth = 1;
                while idx < events.len() {
                    match &events[idx] {
                        Event::Start(Tag::CodeBlock(_)) => depth += 1,
                        Event::End(TagEnd::CodeBlock) => {
                            depth -= 1;
                            if depth == 0 {
                                break;
                            }
                        }
                        _ => {}
                    }
                    code_events.push(events[idx].clone());
                    idx += 1;
                }

                let mut code_text = String::new();
                for ev in code_events {
                    if let Event::Text(text) = ev {
                        code_text.push_str(&text);
                    }
                }

                let escaped_code = html_escape(&code_text);
                let lang_span = if let Some(ref lang) = lang_opt {
                    format!("<span class=\"code-lang\">{}</span>", html_escape(lang))
                } else {
                    String::new()
                };
                let lang_cls = if let Some(ref lang) = lang_opt {
                    format!(" language-{}", html_escape(lang))
                } else {
                    String::new()
                };

                let copy_icon = r#"<svg aria-hidden="true" class="svg-icon iconCopy" width="14" height="15" viewBox="0 0 17 18"><path fill="currentColor" d="M5 6c0-1.09.91-2 2-2h4.5L15 7.5V15c0 1.09-.91 2-2 2H7c-1.09 0-2-.91-2-2zm6-1.25V8h3.25z"/><path fill="currentColor" d="M10 1a2 2 0 0 1 2 2H6a2 2 0 0 0-2 2v9a2 2 0 0 1-2-2V4a3 3 0 0 1 3-3z" opacity=".4"/></svg>"#;

                let copy_label = html_escape(locale.get("copy_code"));
                out.push_str(&format!(
                    "<div class=\"code-block-wrap\"><div class=\"code-header\">{}<button class=\"copy-btn\" onclick=\"copyCode(this)\" title=\"{}\" aria-label=\"{}\">{}</button></div><pre class=\"code-block{}\"><code>{}</code></pre></div>\n",
                    lang_span, copy_label, copy_label, copy_icon, lang_cls, escaped_code
                ));
            }

            // Task List Items (- [ ] or - [x]) or Simple List Items (-)
            Event::Start(Tag::Item) => {
                let is_task = matches!(events.get(idx + 1), Some(Event::TaskListMarker(_)));
                let is_checked = if is_task {
                    if let Some(Event::TaskListMarker(checked)) = events.get(idx + 1) {
                        *checked
                    } else {
                        false
                    }
                } else {
                    false
                };

                if is_task {
                    idx += 2;
                } else {
                    idx += 1;
                }

                let mut item_events = Vec::new();
                let mut depth = 1;
                while idx < events.len() {
                    match &events[idx] {
                        Event::Start(Tag::Item) => depth += 1,
                        Event::End(TagEnd::Item) => {
                            depth -= 1;
                            if depth == 0 {
                                break;
                            }
                        }
                        Event::Start(Tag::List(_)) => {
                            // Sub-list encountered within list item, stop collecting item heading content
                            break;
                        }
                        _ => {}
                    }
                    item_events.push(events[idx].clone());
                    idx += 1;
                }

                let mut label_html = String::new();
                html::push_html(&mut label_html, item_events.into_iter());
                let trimmed = label_html.trim();
                let clean_label = strip_paragraph_tags(trimmed);

                let list_depth = if list_stack.is_empty() { 0 } else { list_stack.len() - 1 };
                let indent_style = if list_depth > 0 {
                    format!(" style=\"--indent: {list_depth};\"")
                } else {
                    String::new()
                };

                let sec_num = if section_count == 0 { 1 } else { section_count };

                if is_task {
                    global_cb_count += 1;
                    let checked_attr = if is_checked { " checked" } else { "" };
                    let checked_cls = if is_checked { " checked" } else { "" };

                    out.push_str(&format!(
                        "<div class=\"check-item{}\" id=\"wrap-cb_s{sec_num}_{global_cb_count}\"{indent_style}>\n",
                        checked_cls
                    ));
                    out.push_str(&format!(
                        "  <input type=\"checkbox\" id=\"cb_s{sec_num}_{global_cb_count}\"{checked_attr}>\n"
                    ));
                    out.push_str(&format!(
                        "  <label class=\"check-label\" for=\"cb_s{sec_num}_{global_cb_count}\">{}</label>\n",
                        clean_label.trim()
                    ));
                    out.push_str("</div>\n");
                } else {
                    let bullet = match list_stack.last_mut() {
                        Some(kind) => format_bullet(kind, list_depth),
                        None => "&bull;".to_string(),
                    };

                    global_item_count += 1;
                    out.push_str(&format!(
                        "<div class=\"check-item simple-item\" id=\"item_s{sec_num}_{global_item_count}\"{indent_style}>\n"
                    ));
                    out.push_str(&format!(
                        "  <span class=\"list-bullet\">{bullet}</span>\n"
                    ));
                    out.push_str(&format!(
                        "  <span class=\"check-label\">{}</span>\n",
                        clean_label.trim()
                    ));
                    out.push_str("</div>\n");
                }

                if idx < events.len() && matches!(events[idx], Event::End(TagEnd::Item)) {
                    idx += 1;
                }
                continue;
            }

            Event::End(TagEnd::Item) => {
                // Suppress standard </li> tags
            }

            // Standalone Text Paragraphs
            Event::Start(Tag::Paragraph) => {
                let mut para_events = Vec::new();
                idx += 1;
                let mut depth = 1;
                while idx < events.len() {
                    match &events[idx] {
                        Event::Start(Tag::Paragraph) => depth += 1,
                        Event::End(TagEnd::Paragraph) => {
                            depth -= 1;
                            if depth == 0 {
                                break;
                            }
                        }
                        _ => {}
                    }
                    para_events.push(events[idx].clone());
                    idx += 1;
                }

                let mut para_html = String::new();
                html::push_html(&mut para_html, para_events.into_iter());
                let trimmed = para_html.trim();

                let clean_content = strip_paragraph_tags(trimmed).trim();

                if !clean_content.is_empty() {
                    let is_image_block = clean_content.starts_with("<img");
                    if is_image_block {
                        out.push_str(&format!(
                            "<div class=\"img-item\">\n  {}\n</div>\n",
                            clean_content
                        ));
                    } else {
                        global_txt_count += 1;
                        let sec_num = if section_count == 0 { 1 } else { section_count };
                        let list_depth = if list_stack.is_empty() { 0 } else { list_stack.len() - 1 };
                        let indent_style = if list_depth > 0 {
                            format!(" style=\"--indent: {list_depth};\"")
                        } else {
                            String::new()
                        };
                        out.push_str(&format!(
                            "<div class=\"check-item text-item\" id=\"txt_s{sec_num}_{global_txt_count}\"{indent_style}>\n"
                        ));
                        out.push_str(&format!(
                            "  <span class=\"text-content\">{}</span>\n",
                            trimmed
                        ));
                        out.push_str("</div>\n");
                    }
                }
            }

            // Track lists (ordered vs unordered)
            Event::Start(Tag::List(first_item_number)) => {
                match first_item_number {
                    Some(start) => list_stack.push(ListKind::Ordered { current: *start }),
                    None => list_stack.push(ListKind::Unordered),
                }
            }
            Event::End(TagEnd::List(_)) => {
                list_stack.pop();
            }

            // Fallback for standard events
            ev => {
                let mut temp = String::new();
                html::push_html(&mut temp, std::iter::once(ev.clone()));
                out.push_str(&temp);
            }
        }
        idx += 1;
    }

    if in_section {
        out.push_str("</div></div>\n");
    }

    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_html_escape() {
        assert_eq!(
            html_escape("a & b < c > d \" e"),
            "a &amp; b &lt; c &gt; d &quot; e"
        );
        assert_eq!(html_escape("plain text"), "plain text");
        assert_eq!(html_escape(""), "");
    }

    #[test]
    fn test_parse_frontmatter_split_once() {
        let input = "---\ntitle: \"My Title\"\nlanguage: de\n---\nBody text";
        let (fm, body) = parse_frontmatter(input);
        assert_eq!(fm.title, "My Title");
        assert_eq!(fm.language, "de");
        assert_eq!(body, "Body text");
    }

    #[test]
    fn test_parse_frontmatter_with_leading_comments() {
        let input = "<!-- Leading comment -->\n\n---\ntitle: \"Header Title\"\nlanguage: en\n---\nBody content";
        let (fm, body) = parse_frontmatter(input);
        assert_eq!(fm.title, "Header Title");
        assert_eq!(fm.language, "en");
        assert_eq!(body, "Body content");
    }

    #[test]
    fn test_parse_callout() {
        let locale = Locale::default();
        assert_eq!(
            parse_callout("!!! Danger zone", &locale),
            ("note note-caution", "Danger zone", "Caution")
        );
        assert_eq!(
            parse_callout("!! Be careful", &locale),
            ("note note-warning", "Be careful", "Warning")
        );
        assert_eq!(
            parse_callout("! Read this", &locale),
            ("note note-important", "Read this", "Important")
        );
        assert_eq!(
            parse_callout("? Pro tip", &locale),
            ("note note-tip", "Pro tip", "Tip")
        );
        assert_eq!(
            parse_callout("Just text", &locale),
            ("note", "Just text", "Note")
        );
    }

    #[test]
    fn test_text_paragraph_conversion() {
        let input = "## Section 1\n\nThis is a standard text paragraph.\n";
        let html = convert_markdown_to_html(input).expect("conversion failed");
        assert!(html.contains("<div class=\"check-item text-item\" id=\"txt_s1_1\">"));
        assert!(
            html.contains("<span class=\"text-content\">This is a standard text paragraph.</span>")
        );
    }

    #[test]
    fn test_ordered_list_conversion() {
        let input = "## Section 1\n\n1. First step\n2. Second step\n3. Third step\n";
        let html = convert_markdown_to_html(input).expect("conversion failed");

        assert!(html.contains("<div class=\"check-item simple-item\" id=\"item_s1_1\">"));
        assert!(html.contains("<span class=\"list-bullet\">1.</span>"));
        assert!(html.contains("<span class=\"check-label\">First step</span>"));
        assert!(html.contains("<span class=\"list-bullet\">2.</span>"));
        assert!(html.contains("<span class=\"check-label\">Second step</span>"));
        assert!(html.contains("<span class=\"list-bullet\">3.</span>"));
        assert!(html.contains("<span class=\"check-label\">Third step</span>"));
    }

    #[test]
    fn test_horizontal_rule_conversion() {
        let input = "## Section 1\n\nText before divider\n\n---\n\nText after divider\n";
        let html = convert_markdown_to_html(input).expect("conversion failed");
        assert!(html.contains("<hr />") || html.contains("<hr>"));
    }

    #[test]
    fn test_html_comments_ignored() {
        let input = "## Section 1\n\n<!-- Secret internal comment -->\n\nThis is visible content.\n<!-- Another hidden comment -->\n";
        let html = convert_markdown_to_html(input).expect("conversion failed");
        assert!(!html.contains("Secret internal comment"));
        assert!(!html.contains("Another hidden comment"));
        assert!(html.contains("This is visible content."));
        // Ensure comments don't create dummy text-items
        assert_eq!(html.matches("class=\"check-item text-item\"").count(), 1);
    }

    #[test]
    fn test_strip_paragraph_tags() {
        assert_eq!(strip_paragraph_tags("<p>Hello</p>"), "Hello");
        assert_eq!(strip_paragraph_tags("<p>World"), "<p>World");
        assert_eq!(strip_paragraph_tags("Plain text</p>"), "Plain text</p>");
        assert_eq!(strip_paragraph_tags("No tags"), "No tags");
        assert_eq!(strip_paragraph_tags("<p></p>"), "");
    }

    #[test]
    fn test_nested_lists_conversion() {
        let input = r#"## Section 1

- Top Task
  1. Sub step A
  2. Sub step B
- Next Task
  - [ ] Sub-task 1
     - Deep detail X
     - Deep detail Y
"#;
        let html = convert_markdown_to_html(input).expect("conversion failed");

        // Top level unordered item: depth 0 (no --indent style)
        assert!(html.contains(r#"<div class="check-item simple-item" id="item_s1_1">"#));
        assert!(html.contains(r#"<span class="list-bullet">&bull;</span>"#));
        assert!(html.contains(r#"<span class="check-label">Top Task</span>"#));

        // Sub-steps A & B: ordered at depth 1 (--indent: 1)
        assert!(html.contains(r#"<div class="check-item simple-item" id="item_s1_2" style="--indent: 1;">"#));
        assert!(html.contains(r#"<span class="list-bullet">a.</span>"#));
        assert!(html.contains(r#"<span class="check-label">Sub step A</span>"#));

        assert!(html.contains(r#"<div class="check-item simple-item" id="item_s1_3" style="--indent: 1;">"#));
        assert!(html.contains(r#"<span class="list-bullet">b.</span>"#));
        assert!(html.contains(r#"<span class="check-label">Sub step B</span>"#));

        // Sub-task 1: task checkbox at depth 1 (--indent: 1)
        assert!(html.contains(r#"<div class="check-item" id="wrap-cb_s1_1" style="--indent: 1;">"#));
        assert!(html.contains(r#"<input type="checkbox" id="cb_s1_1">"#));
        assert!(html.contains(r#"<label class="check-label" for="cb_s1_1">Sub-task 1</label>"#));

        // Deep details X & Y: unordered at depth 2 (--indent: 2) with bullet (&bull;)
        assert!(html.contains(r#"<div class="check-item simple-item" id="item_s1_5" style="--indent: 2;">"#));
        assert!(html.contains(r#"<span class="list-bullet">&bull;</span>"#));
        assert!(html.contains(r#"<span class="check-label">Deep detail X</span>"#));
    }

    #[test]
    fn test_multiline_html_comments_ignored() {
        let input = "## Section 1\n\n<!--\nMultline comment block\nLine 2\n-->\n\nThis is visible content.\n";
        let html = convert_markdown_to_html(input).expect("conversion failed");
        assert!(!html.contains("Multline comment block"));
        assert!(!html.contains("Line 2"));
        assert!(html.contains("This is visible content."));
        assert_eq!(html.matches("class=\"check-item text-item\"").count(), 1);
    }

    #[test]
    fn test_validate_frontmatter_valid_company() {
        let input = "---\ntitle: \"Guide\"\ncompany: \"Acme Corp\"\n---\n## Section 1";
        let (fm, body) = parse_and_validate_frontmatter(input, Some("guide.md")).unwrap();
        assert_eq!(fm.company, "Acme Corp");
        assert_eq!(body, "## Section 1");
    }

    #[test]
    fn test_validate_frontmatter_missing_company_field() {
        let input = "---\ntitle: \"Guide\"\ndate: \"2026-07-25\"\n---\n## Section 1";
        let (fm, _) = parse_frontmatter(input);
        let err = validate_frontmatter(&fm, input, Some("guide.md")).unwrap_err();
        let err_str = err.to_string();

        assert!(err_str.contains("error: missing required frontmatter field 'company'"));
        assert!(err_str.contains("--> guide.md:1:1"));
        assert!(err_str.contains("1 | ---"));
        assert!(
            err_str
                .contains("^^^ frontmatter block defined here is missing required field 'company'")
        );
        assert!(err_str.contains("= help: add 'company: \"Company Name\"'"));
    }

    #[test]
    fn test_validate_frontmatter_empty_company_field() {
        let input = "---\ntitle: \"Guide\"\ncompany: \"\"\ndate: \"2026-07-25\"\n---\n## Section 1";
        let (fm, _) = parse_frontmatter(input);
        let err = validate_frontmatter(&fm, input, Some("guide.md")).unwrap_err();
        let err_str = err.to_string();

        assert!(err_str.contains("error: required frontmatter field 'company' cannot be empty"));
        assert!(err_str.contains("--> guide.md:3:1"));
        assert!(err_str.contains("3 | company: \"\""));
        assert!(err_str.contains("^^^^^^^^^^^ 'company' field value cannot be empty"));
        assert!(err_str.contains("= help: provide a valid company name"));
    }

    #[test]
    fn test_validate_frontmatter_missing_block() {
        let input = "# No Frontmatter Document\n\nContent paragraph.";
        let fm = Frontmatter::default();
        let err = validate_frontmatter(&fm, input, Some("doc.md")).unwrap_err();
        let err_str = err.to_string();

        assert!(
            err_str.contains("error: missing YAML frontmatter block with required field 'company'")
        );
        assert!(err_str.contains("--> doc.md:1:1"));
        assert!(err_str.contains("1 | # No Frontmatter Document"));
        assert!(err_str.contains("^ missing frontmatter section '---'"));
        assert!(err_str.contains("= help: add YAML frontmatter"));
    }
}
