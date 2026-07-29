use crate::error::{DiagnosticError, Result};
use crate::i18n::Locale;
use crate::template;
use pulldown_cmark::{
    CodeBlockKind, Event, HeadingLevel, Options, Parser as MarkdownParser, Tag, TagEnd, html,
};
use std::borrow::Cow;

/// Escapes HTML special characters in code strings. Returns Cow::Borrowed if no escaping is needed.
fn html_escape(input: &str) -> Cow<'_, str> {
    let bytes = input.as_bytes();
    let first_pos = match bytes
        .iter()
        .position(|&b| matches!(b, b'&' | b'<' | b'>' | b'"'))
    {
        Some(pos) => pos,
        None => return Cow::Borrowed(input),
    };

    let mut escaped = String::with_capacity(input.len() + 8);
    escaped.push_str(&input[..first_pos]);

    for ch in input[first_pos..].chars() {
        match ch {
            '&' => escaped.push_str("&amp;"),
            '<' => escaped.push_str("&lt;"),
            '>' => escaped.push_str("&gt;"),
            '"' => escaped.push_str("&quot;"),
            _ => escaped.push(ch),
        }
    }
    Cow::Owned(escaped)
}

#[derive(Debug, Clone, Copy)]
enum ListKind {
    Unordered,
    Ordered { current: u64 },
}

/// Converts a 1-based number to alphabetic representation (1 -> a, 2 -> b, ..., 26 -> z).
fn to_alpha(mut n: u64) -> Cow<'static, str> {
    if n == 0 {
        return Cow::Borrowed("a");
    }
    if n <= 26 {
        const ALPHAS: &[&str; 26] = &[
            "a", "b", "c", "d", "e", "f", "g", "h", "i", "j", "k", "l", "m", "n", "o", "p", "q",
            "r", "s", "t", "u", "v", "w", "x", "y", "z",
        ];
        return Cow::Borrowed(ALPHAS[(n - 1) as usize]);
    }
    let mut buf = Vec::with_capacity(8);
    while n > 0 {
        n -= 1;
        let rem = (n % 26) as u8;
        buf.push(b'a' + rem);
        n /= 26;
    }
    buf.reverse();
    Cow::Owned(String::from_utf8(buf).expect("valid ASCII bytes"))
}

/// Converts a 1-based number to lowercase Roman numerals (1 -> i, 2 -> ii, 3 -> iii...).
fn to_roman(n: u64) -> Cow<'static, str> {
    if (1..=10).contains(&n) {
        const ROMANS: &[&str; 10] = &["i", "ii", "iii", "iv", "v", "vi", "vii", "viii", "ix", "x"];
        return Cow::Borrowed(ROMANS[(n - 1) as usize]);
    }
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
        Cow::Borrowed("i")
    } else {
        Cow::Owned(result)
    }
}

/// Formats bullet symbol or ordered number based on list kind and nesting depth.
fn format_bullet(kind: &mut ListKind, depth: usize) -> Cow<'static, str> {
    match kind {
        ListKind::Ordered { current } => {
            let num = *current;
            *current += 1;
            match depth % 3 {
                0 => Cow::Owned(format!("{num}.")),
                1 => Cow::Owned(format!("{}.", to_alpha(num))),
                _ => Cow::Owned(format!("{}.", to_roman(num))),
            }
        }
        ListKind::Unordered => Cow::Borrowed("&bull;"),
    }
}

/// Frontmatter metadata extracted from Markdown header.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Frontmatter {
    pub title: Option<String>,
    pub subtitle: Option<String>,
    pub company: String,
    pub contact: Option<String>,
    pub agent: Option<String>,
    pub date: Option<String>,
    pub version: Option<String>,
    pub language: Option<String>,
    pub logo: Option<String>,
    pub number_sections: bool,
}

impl Frontmatter {
    /// Creates a new `Frontmatter` instance with the required `company` field and default optional values.
    pub fn new(company: impl Into<String>) -> Self {
        Self {
            company: company.into(),
            title: None,
            subtitle: None,
            contact: None,
            agent: None,
            date: None,
            version: None,
            language: None,
            logo: None,
            number_sections: true,
        }
    }
}

/// Helper struct holding byte ranges and line info for frontmatter block.
#[derive(Debug)]
struct FrontmatterBounds<'a> {
    frontmatter_text: &'a str,
    body_text: &'a str,
    start_line_no: usize,
}

/// Finds frontmatter bounds robustly using line-based iteration.
fn find_frontmatter_bounds(md_content: &str) -> Option<FrontmatterBounds<'_>> {
    let mut line_no = 1;
    let mut in_leading_comment = false;

    let mut start_line = 1;
    let mut content_start_offset = None;

    for line in md_content.lines() {
        let trimmed = line.trim();
        if in_leading_comment {
            if trimmed.contains("-->") {
                in_leading_comment = false;
            }
            line_no += 1;
            continue;
        }

        if trimmed.starts_with("<!--") {
            if !trimmed.contains("-->") {
                in_leading_comment = true;
            }
            line_no += 1;
            continue;
        }

        if trimmed.is_empty() {
            line_no += 1;
            continue;
        }

        if trimmed == "---" {
            let line_offset = line.as_ptr() as usize - md_content.as_ptr() as usize;
            let after_first = line_offset + line.len();
            let content_start = md_content[after_first..]
                .strip_prefix("\r\n")
                .or_else(|| md_content[after_first..].strip_prefix("\n"))
                .map(|s| md_content.len() - s.len())
                .unwrap_or(after_first);

            content_start_offset = Some(content_start);
            start_line = line_no;
            break;
        } else {
            return None;
        }
    }

    let content_start = content_start_offset?;
    let rest = &md_content[content_start..];
    let mut close_start_offset = None;
    let mut body_start_offset = None;

    for line in rest.lines() {
        let trimmed = line.trim();
        if trimmed == "---" {
            let line_offset = line.as_ptr() as usize - md_content.as_ptr() as usize;
            close_start_offset = Some(line_offset);

            let after_close = line_offset + line.len();
            let body_start = md_content[after_close..]
                .strip_prefix("\r\n")
                .or_else(|| md_content[after_close..].strip_prefix("\n"))
                .map(|s| md_content.len() - s.len())
                .unwrap_or(after_close);

            body_start_offset = Some(body_start);
            break;
        }
    }

    let close_offset = close_start_offset?;
    let body_offset = body_start_offset?;

    let frontmatter_text = &md_content[content_start..close_offset];
    let body_text = &md_content[body_offset..];

    Some(FrontmatterBounds {
        frontmatter_text,
        body_text,
        start_line_no: start_line,
    })
}

/// Trims surrounding quotes only if enclosed by identical matching single or double quotes.
fn trim_matching_quotes(input: &str) -> &str {
    let s = input.trim();
    if (s.starts_with('"') && s.ends_with('"') && s.len() >= 2)
        || (s.starts_with('\'') && s.ends_with('\'') && s.len() >= 2)
    {
        &s[1..s.len() - 1]
    } else {
        s
    }
}

/// Parses YAML-style frontmatter delimited by `---`.
pub fn parse_frontmatter(md_content: &str) -> (Frontmatter, &str) {
    if let Some(bounds) = find_frontmatter_bounds(md_content) {
        let mut fm = Frontmatter::new("");

        for line in bounds.frontmatter_text.lines() {
            if let Some((key, val)) = line.split_once(':') {
                let key = key.trim();
                let val_trimmed = trim_matching_quotes(val);

                if val_trimmed.is_empty() && key != "number_sections" {
                    continue;
                }

                match key {
                    "title" => fm.title = Some(val_trimmed.to_string()),
                    "subtitle" => fm.subtitle = Some(val_trimmed.to_string()),
                    "company" => fm.company = val_trimmed.to_string(),
                    "contact" => fm.contact = Some(val_trimmed.to_string()),
                    "agent" => fm.agent = Some(val_trimmed.to_string()),
                    "date" => fm.date = Some(val_trimmed.to_string()),
                    "version" => fm.version = Some(val_trimmed.to_string()),
                    "language" | "lang" => fm.language = Some(val_trimmed.to_string()),
                    "logo" => fm.logo = Some(val_trimmed.to_string()),
                    "number_sections" => {
                        fm.number_sections = val_trimmed.eq_ignore_ascii_case("true");
                    }
                    _ => {}
                }
            }
        }

        return (fm, bounds.body_text);
    }

    (Frontmatter::new(""), md_content)
}

/// Validates frontmatter metadata for required fields, returning a compiler-style diagnostic error if invalid.
pub fn validate_frontmatter(
    frontmatter: &Frontmatter,
    md_content: &str,
    file_name: Option<&str>,
) -> Result<()> {
    if !frontmatter.company.trim().is_empty() {
        return Ok(());
    }

    let file_path = file_name.unwrap_or("input.md");

    if let Some(bounds) = find_frontmatter_bounds(md_content) {
        let mut company_line_info = None;

        for (idx, line) in bounds.frontmatter_text.lines().enumerate() {
            if matches!(line.split_once(':'), Some((key, _)) if key.trim() == "company") {
                let line_no = bounds.start_line_no + idx + 1;
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
            Err(DiagnosticError::missing_frontmatter_field(
                file_path,
                bounds.start_line_no,
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

/// Iterator adapter filtering out HTML comment blocks from a stream of Markdown events.
struct CommentFilter<I> {
    iter: I,
    in_comment: bool,
}

impl<I> CommentFilter<I> {
    fn new(iter: I) -> Self {
        Self {
            iter,
            in_comment: false,
        }
    }
}

impl<'a, I> Iterator for CommentFilter<I>
where
    I: Iterator<Item = Event<'a>>,
{
    type Item = Event<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(ev) = self.iter.next() {
            match &ev {
                Event::Html(text) | Event::InlineHtml(text) => {
                    let s = text.as_ref();
                    if self.in_comment {
                        if s.contains("-->") {
                            self.in_comment = false;
                        }
                        continue;
                    } else if let Some(start_idx) = s.find("<!--") {
                        if s[start_idx..].contains("-->") {
                            continue;
                        } else {
                            self.in_comment = true;
                            continue;
                        }
                    }
                }
                _ => {
                    if self.in_comment {
                        continue;
                    }
                }
            }
            return Some(ev);
        }
        None
    }
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
    convert_markdown_to_html_with_options(markdown_body, locale, false)
}

/// Converts Markdown body into interactive HTML with specified locale and options (e.g. section numbering).
pub fn convert_markdown_to_html_with_options(
    markdown_body: &str,
    locale: &Locale,
    number_sections: bool,
) -> Result<String> {
    let mut options = Options::empty();
    options.insert(Options::ENABLE_TASKLISTS);
    options.insert(Options::ENABLE_STRIKETHROUGH);
    options.insert(Options::ENABLE_TABLES);

    let parser = MarkdownParser::new_ext(markdown_body, options);
    let events: Vec<Event> = CommentFilter::new(parser).collect();

    let mut out = String::with_capacity(markdown_body.len() * 2);
    let mut section_count = 0usize;
    let mut h1_counter = 0u32;
    let mut h2_counter = 0u32;
    let mut global_cb_count = 0usize;
    let mut global_txt_count = 0usize;
    let mut global_item_count = 0usize;
    let mut in_section = false;
    let mut list_stack: Vec<ListKind> = Vec::new();

    let mut idx = 0;
    while idx < events.len() {
        match &events[idx] {
            // Level 1 (# Section) and Level 2 (## Section) Headings
            Event::Start(Tag::Heading {
                level: level @ (HeadingLevel::H1 | HeadingLevel::H2),
                ..
            }) => {
                let target_level = *level;
                if in_section {
                    template::render_section_close(&mut out);
                }
                section_count += 1;
                in_section = true;

                let start_idx = idx + 1;
                idx += 1;
                while idx < events.len() {
                    if matches!(events[idx], Event::End(TagEnd::Heading(l)) if l == target_level) {
                        break;
                    }
                    idx += 1;
                }

                let mut heading_html = String::new();
                html::push_html(&mut heading_html, events[start_idx..idx].iter().cloned());
                let heading_text = heading_html.trim();

                let is_empty = is_section_empty(&events[idx + 1..]);
                let is_h1 = target_level == HeadingLevel::H1;

                let final_heading_text;
                let formatted_heading;
                if number_sections {
                    if is_h1 {
                        h1_counter += 1;
                        h2_counter = 0;
                        formatted_heading = format!("{h1_counter}. {heading_text}");
                    } else {
                        h2_counter += 1;
                        formatted_heading = format!("{h1_counter}.{h2_counter} {heading_text}");
                    }
                    final_heading_text = formatted_heading.as_str();
                } else {
                    final_heading_text = heading_text;
                }

                template::render_section_header(
                    &mut out,
                    section_count,
                    final_heading_text,
                    is_h1,
                    is_empty,
                );
            }

            // Level 3-6 Headings (###, ####, #####, ###### Subheadings)
            Event::Start(Tag::Heading {
                level: level @ (HeadingLevel::H3
                | HeadingLevel::H4
                | HeadingLevel::H5
                | HeadingLevel::H6),
                ..
            }) => {
                let target_level = *level;
                let start_idx = idx + 1;
                idx += 1;
                while idx < events.len() {
                    if matches!(events[idx], Event::End(TagEnd::Heading(l)) if l == target_level) {
                        break;
                    }
                    idx += 1;
                }

                let mut sub_html = String::new();
                html::push_html(&mut sub_html, events[start_idx..idx].iter().cloned());
                template::render_subheading(&mut out, &sub_html);
            }

            // Blockquotes (> Note, >? Tip, >! Important, >!! Warning, >!!! Caution)
            Event::Start(Tag::BlockQuote(_)) => {
                let start_idx = idx + 1;
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
                    idx += 1;
                }

                let mut bq_html = String::new();
                html::push_html(&mut bq_html, events[start_idx..idx].iter().cloned());
                let trimmed = bq_html.trim();

                let inner = strip_paragraph_tags(trimmed);
                let (note_cls, note_content, callout_label) = parse_callout(inner, locale);

                let escaped_label = html_escape(callout_label);
                template::render_callout(&mut out, note_cls, &escaped_label, note_content);
            }

            // Code Blocks (e.g. ```ini ... ```)
            Event::Start(Tag::CodeBlock(kind)) => {
                let lang_opt = match kind {
                    CodeBlockKind::Fenced(lang) if !lang.is_empty() => Some(lang.to_string()),
                    _ => None,
                };

                let start_idx = idx + 1;
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
                    idx += 1;
                }

                let mut code_text = String::new();
                for ev in &events[start_idx..idx] {
                    if let Event::Text(text) = ev {
                        code_text.push_str(text);
                    }
                }

                let escaped_code = html_escape(&code_text);
                let copy_label = html_escape(locale.get("copy_code"));

                let escaped_lang_opt = lang_opt.as_ref().map(|l| html_escape(l));
                let lang_ref = escaped_lang_opt.as_deref();

                template::render_code_block(&mut out, lang_ref, &escaped_code, &copy_label);
            }

            // Task List Items (- [ ] or - [x]) or Simple List Items (-)
            Event::Start(Tag::Item) => {
                let (is_task, is_checked) = match (events.get(idx + 1), events.get(idx + 2)) {
                    (Some(Event::TaskListMarker(checked)), _) => (true, *checked),
                    (Some(Event::Start(Tag::Paragraph)), Some(Event::TaskListMarker(checked))) => {
                        (true, *checked)
                    }
                    _ => (false, false),
                };

                let start_idx = idx + 1;
                idx = start_idx;
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
                            break;
                        }
                        _ => {}
                    }
                    idx += 1;
                }

                let mut label_html = String::new();
                if is_task {
                    html::push_html(
                        &mut label_html,
                        events[start_idx..idx]
                            .iter()
                            .filter(|ev| !matches!(ev, Event::TaskListMarker(_)))
                            .cloned(),
                    );
                } else {
                    html::push_html(&mut label_html, events[start_idx..idx].iter().cloned());
                }
                let trimmed = label_html.trim();
                let clean_label = strip_paragraph_tags(trimmed);

                let list_depth = if list_stack.is_empty() { 0 } else { list_stack.len() - 1 };
                let sec_num = if section_count == 0 { 1 } else { section_count };

                if is_task {
                    global_cb_count += 1;
                    template::render_task_item(
                        &mut out,
                        sec_num,
                        global_cb_count,
                        is_checked,
                        clean_label,
                        list_depth,
                    );
                } else {
                    let bullet = match list_stack.last_mut() {
                        Some(kind) => format_bullet(kind, list_depth),
                        None => Cow::Borrowed("&bull;"),
                    };

                    global_item_count += 1;
                    template::render_list_item(
                        &mut out,
                        sec_num,
                        global_item_count,
                        &bullet,
                        clean_label,
                        list_depth,
                    );
                }

                if idx < events.len() && matches!(events[idx], Event::End(TagEnd::Item)) {
                    idx += 1;
                }
                continue;
            }

            Event::End(TagEnd::Item) => {}

            // Standalone Text Paragraphs
            Event::Start(Tag::Paragraph) => {
                let start_idx = idx + 1;
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
                    idx += 1;
                }

                let mut para_html = String::new();
                html::push_html(&mut para_html, events[start_idx..idx].iter().cloned());
                let trimmed = para_html.trim();

                let clean_content = strip_paragraph_tags(trimmed).trim();

                if !clean_content.is_empty() {
                    let is_image_block = clean_content.starts_with("<img");
                    if is_image_block {
                        template::render_image_item(&mut out, clean_content);
                    } else {
                        global_txt_count += 1;
                        let sec_num = if section_count == 0 { 1 } else { section_count };
                        let list_depth = if list_stack.is_empty() { 0 } else { list_stack.len() - 1 };
                        template::render_text_item(
                            &mut out,
                            sec_num,
                            global_txt_count,
                            trimmed,
                            list_depth,
                        );
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
        template::render_section_close(&mut out);
    }

    Ok(out)
}

fn is_section_empty(events: &[Event]) -> bool {
    for event in events {
        match event {
            Event::Start(Tag::Heading {
                level: HeadingLevel::H1 | HeadingLevel::H2,
                ..
            }) => break,
            Event::Start(_) => return false,
            Event::Text(text) if !text.trim().is_empty() => return false,
            Event::Code(_) | Event::Html(_) | Event::Rule => return false,
            _ => {}
        }
    }
    true
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
        let input = "---\ntitle: \"My Title\"\nlanguage: de\nlogo: \"custom_logo.svg\"\n---\nBody text";
        let (fm, body) = parse_frontmatter(input);
        assert_eq!(fm.title.as_deref(), Some("My Title"));
        assert_eq!(fm.language.as_deref(), Some("de"));
        assert_eq!(fm.logo.as_deref(), Some("custom_logo.svg"));
        assert_eq!(body, "Body text");
    }

    #[test]
    fn test_parse_frontmatter_with_leading_comments() {
        let input = "<!-- Leading comment -->\n\n---\ntitle: \"Header Title\"\nlanguage: en\n---\nBody content";
        let (fm, body) = parse_frontmatter(input);
        assert_eq!(fm.title.as_deref(), Some("Header Title"));
        assert_eq!(fm.language.as_deref(), Some("en"));
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
        assert!(html.contains("<span class=\"item-comment-icon\">"));
    }

    #[test]
    fn test_item_comment_icon_presence() {
        let input = "## Section 1\n\n- [ ] Task item\n- Simple list item\n\nParagraph text\n";
        let html = convert_markdown_to_html(input).expect("conversion failed");
        assert_eq!(html.matches("class=\"item-comment-icon\"").count(), 3);
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

        assert!(html.contains(r#"<div class="check-item simple-item" id="item_s1_1">"#));
        assert!(html.contains(r#"<span class="list-bullet">&bull;</span>"#));
        assert!(html.contains(r#"<span class="check-label">Top Task</span>"#));

        assert!(html.contains(r#"<div class="check-item simple-item" id="item_s1_2" style="--indent: 1;">"#));
        assert!(html.contains(r#"<span class="list-bullet">a.</span>"#));
        assert!(html.contains(r#"<span class="check-label">Sub step A</span>"#));

        assert!(html.contains(r#"<div class="check-item simple-item" id="item_s1_3" style="--indent: 1;">"#));
        assert!(html.contains(r#"<span class="list-bullet">b.</span>"#));
        assert!(html.contains(r#"<span class="check-label">Sub step B</span>"#));

        assert!(html.contains(r#"<div class="check-item" id="wrap-cb_s1_1" style="--indent: 1;">"#));
        assert!(html.contains(r#"<input type="checkbox" id="cb_s1_1">"#));
        assert!(html.contains(r#"<label class="check-label" for="cb_s1_1">Sub-task 1</label>"#));

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
        let fm = Frontmatter::new("");
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

    #[test]
    fn test_level_1_heading_conversion() {
        let input = "# Top Level Header\n\n- [ ] Task in H1\n\n## Sub Section\n\n- [x] Task in H2";
        let html = convert_markdown_to_html(input).expect("conversion failed");

        assert!(html.contains(r#"<!-- S1 -->"#));
        assert!(html.contains(r#"<section class="section" id="s1">"#));
        assert!(html.contains(r#"<h2 class="sh sh-h1" role="button" tabindex="0" aria-expanded="true"><span>Top Level Header</span>"#));
        assert!(html.contains(r#"badge-s1"#));
        assert!(html.contains(r#"tog-s1"#));
        assert!(html.contains(r#"id="wrap-cb_s1_1""#));

        assert!(html.contains(r#"<!-- S2 -->"#));
        assert!(html.contains(r#"<section class="section" id="s2">"#));
        assert!(html.contains(r#"<h2 class="sh" role="button" tabindex="0" aria-expanded="true"><span>Sub Section</span>"#));
    }

    #[test]
    fn test_empty_heading_conversion() {
        let input = "# Empty H1 Header\n\n## Empty H2 Header\n\n## Non Empty H2\n\nSome paragraph content";
        let html = convert_markdown_to_html(input).expect("conversion failed");

        assert!(html.contains(r#"<h2 class="sh sh-h1 no-toggle"><span>Empty H1 Header</span>"#));
        assert!(html.contains(r#"<h2 class="sh no-toggle"><span>Empty H2 Header</span>"#));
        assert!(html.contains(r#"<h2 class="sh" role="button" tabindex="0" aria-expanded="true"><span>Non Empty H2</span>"#));
    }

    #[test]
    fn test_no_inline_onclick_on_section_headers() {
        let input = "# H1 Heading\n\nSome content\n\n## H2 Heading\n\nMore content";
        let html = convert_markdown_to_html(input).expect("conversion failed");

        assert!(!html.contains("onclick="));
    }

    #[test]
    fn test_h4_to_h6_treated_as_subheading() {
        let input = "## Section 1\n\n### Sub 3\n\n#### Sub 4\n\n##### Sub 5\n\n###### Sub 6";
        let html = convert_markdown_to_html(input).expect("conversion failed");

        assert!(html.contains(r#"<h3 class="subh">Sub 3</h3>"#));
        assert!(html.contains(r#"<h3 class="subh">Sub 4</h3>"#));
        assert!(html.contains(r#"<h3 class="subh">Sub 5</h3>"#));
        assert!(html.contains(r#"<h3 class="subh">Sub 6</h3>"#));
    }

    #[test]
    fn test_frontmatter_windows_crlf_line_endings() {
        let input = "---\r\ntitle: \"CRLF Title\"\r\ncompany: \"CRLF Corp\"\r\nlanguage: de\r\n---\r\n## Section 1\r\nBody line";
        let (fm, body) = parse_frontmatter(input);
        assert_eq!(fm.title.as_deref(), Some("CRLF Title"));
        assert_eq!(fm.company, "CRLF Corp");
        assert_eq!(fm.language.as_deref(), Some("de"));
        assert_eq!(body, "## Section 1\r\nBody line");
    }

    #[test]
    fn test_frontmatter_quoted_and_unquoted_fields() {
        let input = "---\ntitle: 'Single Quoted'\nsubtitle: \"Double Quoted\"\ncompany:   Unquoted Spaced   \ncontact: 'person@example.com'\nagent: \"Agent 007\"\ndate: 2026-07-26\nversion: '1.2.3'\nlang: en\n---\nBody";
        let (fm, _) = parse_frontmatter(input);
        assert_eq!(fm.title.as_deref(), Some("Single Quoted"));
        assert_eq!(fm.subtitle.as_deref(), Some("Double Quoted"));
        assert_eq!(fm.company, "Unquoted Spaced");
        assert_eq!(fm.contact.as_deref(), Some("person@example.com"));
        assert_eq!(fm.agent.as_deref(), Some("Agent 007"));
        assert_eq!(fm.date.as_deref(), Some("2026-07-26"));
        assert_eq!(fm.version.as_deref(), Some("1.2.3"));
        assert_eq!(fm.language.as_deref(), Some("en"));
    }

    #[test]
    fn test_trim_matching_quotes() {
        assert_eq!(trim_matching_quotes("\"quoted\""), "quoted");
        assert_eq!(trim_matching_quotes("'single'"), "single");
        assert_eq!(trim_matching_quotes("\"mismatched'"), "\"mismatched'");
        assert_eq!(trim_matching_quotes("plain"), "plain");
    }

    #[test]
    fn test_to_alpha_and_to_roman() {
        assert_eq!(to_alpha(0), "a");
        assert_eq!(to_alpha(1), "a");
        assert_eq!(to_alpha(2), "b");
        assert_eq!(to_alpha(26), "z");
        assert_eq!(to_alpha(27), "aa");

        assert_eq!(to_roman(0), "i");
        assert_eq!(to_roman(1), "i");
        assert_eq!(to_roman(2), "ii");
        assert_eq!(to_roman(4), "iv");
        assert_eq!(to_roman(9), "ix");
        assert_eq!(to_roman(14), "xiv");
        assert_eq!(to_roman(40), "xl");
        assert_eq!(to_roman(90), "xc");
        assert_eq!(to_roman(400), "cd");
        assert_eq!(to_roman(900), "cm");
        assert_eq!(to_roman(1984), "mcmlxxxiv");
    }

    #[test]
    fn test_callout_parsing_variants_and_formatting() {
        let locale = Locale::default();
        let (cls, text, lbl) = parse_callout("!Important message", &locale);
        assert_eq!(cls, "note note-important");
        assert_eq!(text, "Important message");
        assert_eq!(lbl, "Important");

        let (cls, text, lbl) = parse_callout("?Tip without space", &locale);
        assert_eq!(cls, "note note-tip");
        assert_eq!(text, "Tip without space");
        assert_eq!(lbl, "Tip");

        let (cls, text, lbl) = parse_callout("!!Warning without space", &locale);
        assert_eq!(cls, "note note-warning");
        assert_eq!(text, "Warning without space");
        assert_eq!(lbl, "Warning");

        let (cls, text, lbl) = parse_callout("!!!Caution without space", &locale);
        assert_eq!(cls, "note note-caution");
        assert_eq!(text, "Caution without space");
        assert_eq!(lbl, "Caution");
    }

    #[test]
    fn test_deeply_nested_lists_up_to_level_4() {
        let input = r#"## Section 1

1. Level 1 Item 1
   1. Level 2 Item 1
      1. Level 3 Item 1
         1. Level 4 Item 1
"#;
        let html = convert_markdown_to_html(input).expect("conversion failed");

        assert!(html.contains(r#"<span class="list-bullet">1.</span>"#));
        assert!(html.contains(r#"<span class="list-bullet">a.</span>"#));
        assert!(html.contains(r#"<span class="list-bullet">i.</span>"#));
        assert!(html.contains(r#"<span class="list-bullet">1.</span>"#));
    }

    #[test]
    fn test_html_escaping_in_code_and_headings() {
        let input = "## Header `<script>alert(1)</script>` & More\n\n```html\n<div id=\"app\">&amp;</div>\n```";
        let html = convert_markdown_to_html(input).expect("conversion failed");

        assert!(html.contains("&lt;script&gt;alert(1)&lt;/script&gt;"));
        assert!(html.contains("&amp; More"));
        assert!(html.contains("&lt;div id=&quot;app&quot;&gt;&amp;amp;&lt;/div&gt;"));
    }

    #[test]
    fn test_loose_task_list_conversion() {
        let input = "## Section 1\n\n- [ ] Task 1\n\n- [x] Task 2\n\n- [ ] Task 3\n";
        let html = convert_markdown_to_html(input).expect("conversion failed");

        assert!(html.contains(r#"<div class="check-item" id="wrap-cb_s1_1">"#));
        assert!(html.contains(r#"<input type="checkbox" id="cb_s1_1">"#));
        assert!(html.contains(r#"<label class="check-label" for="cb_s1_1">Task 1</label>"#));

        assert!(html.contains(r#"<div class="check-item checked" id="wrap-cb_s1_2">"#));
        assert!(html.contains(r#"<input type="checkbox" id="cb_s1_2" checked=""#) || html.contains(r#"<input type="checkbox" id="cb_s1_2" checked>"#));
        assert!(html.contains(r#"<label class="check-label" for="cb_s1_2">Task 2</label>"#));

        assert!(html.contains(r#"<div class="check-item" id="wrap-cb_s1_3">"#));
        assert!(html.contains(r#"<input type="checkbox" id="cb_s1_3">"#));
        assert!(html.contains(r#"<label class="check-label" for="cb_s1_3">Task 3</label>"#));

        assert!(!html.contains("simple-item"));
        assert!(!html.contains("<p>Task"));
        assert!(!html.contains("Task 1</p>"));
    }

    #[test]
    fn test_number_sections_frontmatter_parsing() {
        let input1 = "---\nnumber_sections: true\n---";
        let (fm1, _) = parse_frontmatter(input1);
        assert!(fm1.number_sections);

        let input2 = "---\nnumber_sections: True\n---";
        let (fm2, _) = parse_frontmatter(input2);
        assert!(fm2.number_sections);

        let input3 = "---\nnumber_sections: false\n---";
        let (fm3, _) = parse_frontmatter(input3);
        assert!(!fm3.number_sections);

        let input4 = "---\ntitle: \"Default Test\"\n---";
        let (fm4, _) = parse_frontmatter(input4);
        assert!(fm4.number_sections);
    }

    #[test]
    fn test_section_numbering_conversion() {
        let input = r#"# First H1
- [ ] Task 1.1

## First Sub H2
- [ ] Task 1.1.1

## Second Sub H2
- [ ] Task 1.2.1

# Second H1
- [ ] Task 2.1

## Third Sub H2
- [ ] Task 2.1.1
"#;
        let locale = Locale::default();
        let html_enabled = convert_markdown_to_html_with_options(input, &locale, true)
            .expect("conversion failed");

        assert!(html_enabled.contains("<span>1. First H1</span>"));
        assert!(html_enabled.contains("<span>1.1 First Sub H2</span>"));
        assert!(html_enabled.contains("<span>1.2 Second Sub H2</span>"));
        assert!(html_enabled.contains("<span>2. Second H1</span>"));
        assert!(html_enabled.contains("<span>2.1 Third Sub H2</span>"));

        let html_disabled = convert_markdown_to_html_with_options(input, &locale, false)
            .expect("conversion failed");

        assert!(html_disabled.contains("<span>First H1</span>"));
        assert!(html_disabled.contains("<span>First Sub H2</span>"));
        assert!(html_disabled.contains("<span>Second Sub H2</span>"));
        assert!(html_disabled.contains("<span>Second H1</span>"));
        assert!(html_disabled.contains("<span>Third Sub H2</span>"));
    }
}
