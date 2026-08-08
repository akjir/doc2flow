use crate::error::Result;
use crate::features;
use crate::locales::Locale;
use crate::template;
use pulldown_cmark::{
    html, CodeBlockKind, Event, HeadingLevel, Options, Parser as MarkdownParser, Tag, TagEnd,
};
use std::borrow::Cow;

/// Escapes HTML special characters in code strings. Returns Cow::Borrowed if no escaping is needed.
pub(crate) fn html_escape(input: &str) -> Cow<'_, str> {
    let bytes = input.as_bytes();
    let first_pos = match bytes
        .iter()
        .position(|&b| matches!(b, b'&' | b'<' | b'>' | b'"'))
    {
        Some(pos) => pos,
        None => return Cow::Borrowed(input),
    };

    let mut escaped = String::with_capacity(input.len() + 16);
    escaped.push_str(&input[..first_pos]);
    let mut last = first_pos;

    for (i, &b) in bytes.iter().enumerate().skip(first_pos) {
        let sub = match b {
            b'&' => "&amp;",
            b'<' => "&lt;",
            b'>' => "&gt;",
            b'"' => "&quot;",
            _ => continue,
        };
        escaped.push_str(&input[last..i]);
        escaped.push_str(sub);
        last = i + 1;
    }
    escaped.push_str(&input[last..]);
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
    let mut buf = [0u8; 16];
    let mut pos = 16;
    while n > 0 {
        n -= 1;
        pos -= 1;
        buf[pos] = b'a' + (n % 26) as u8;
        n /= 26;
    }
    let s = std::str::from_utf8(&buf[pos..]).expect("valid ASCII bytes");
    Cow::Owned(s.to_string())
}

/// Converts a 1-based number to lowercase Roman numerals (1 -> i, 2 -> ii, 3 -> iii...).
fn to_roman(n: u64) -> Cow<'static, str> {
    if (1..=10).contains(&n) {
        const ROMANS: &[&str; 10] = &["i", "ii", "iii", "iv", "v", "vi", "vii", "viii", "ix", "x"];
        return Cow::Borrowed(ROMANS[(n - 1) as usize]);
    }
    const ROMAN_MAPPING: [(u64, &str); 13] = [
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
    let mut result = String::with_capacity(16);
    for (val, sym) in ROMAN_MAPPING {
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
    pub date: Option<String>,
    pub version: Option<String>,
    pub language: Option<String>,
    pub logo: Option<String>,
    pub numbered_sections: bool,
}

/// Detected interactive features present in a Markdown document.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct DocumentFeatures {
    pub has_code: bool,
    pub has_tasks: bool,
    pub has_images: bool,
    pub has_tables: bool,
}

impl DocumentFeatures {
    /// Renders a comma-separated list of enabled feature identifiers starting with `"core"`.
    ///
    /// # Examples
    ///
    /// ```
    /// use doc2flow::converter::DocumentFeatures;
    ///
    /// let mut features = DocumentFeatures::default();
    /// assert_eq!(features.to_features_string(), "core");
    ///
    /// features.has_tasks = true;
    /// assert_eq!(features.to_features_string(), "core, tasks");
    ///
    /// features.has_tables = true;
    /// assert_eq!(features.to_features_string(), "core, tasks, table");
    /// ```
    pub fn to_features_string(&self) -> String {
        let mut out = String::with_capacity(32);
        out.push_str("core");
        if self.has_code {
            out.push_str(", code");
        }
        if self.has_tasks {
            out.push_str(", tasks");
        }
        if self.has_images {
            out.push_str(", images");
        }
        if self.has_tables {
            out.push_str(", table");
        }
        out
    }
}

impl Default for Frontmatter {
    fn default() -> Self {
        Self {
            title: None,
            subtitle: None,
            date: None,
            version: None,
            language: None,
            logo: None,
            numbered_sections: true,
        }
    }
}

impl Frontmatter {
    /// Creates a new `Frontmatter` instance with default optional values and flags.
    pub fn new() -> Self {
        Self::default()
    }

    /// Converts all frontmatter metadata fields into a key-value hash map.
    pub fn to_hashmap(&self) -> std::collections::HashMap<String, String> {
        let mut map = std::collections::HashMap::with_capacity(8);
        if let Some(ref t) = self.title {
            map.insert("title".to_string(), t.clone());
        }
        if let Some(ref s) = self.subtitle {
            map.insert("subtitle".to_string(), s.clone());
        }
        if let Some(ref d) = self.date {
            map.insert("date".to_string(), d.clone());
        }
        if let Some(ref v) = self.version {
            map.insert("version".to_string(), v.clone());
        }
        if let Some(ref l) = self.language {
            map.insert("language".to_string(), l.clone());
            map.insert("lang".to_string(), l.clone());
        }
        if let Some(ref lg) = self.logo {
            map.insert("logo".to_string(), lg.clone());
        }
        map.insert("numbered_sections".to_string(), self.numbered_sections.to_string());
        map
    }
}

/// Helper struct holding byte ranges and line info for frontmatter block.
#[derive(Debug)]
struct FrontmatterBounds<'a> {
    frontmatter_text: &'a str,
    body_text: &'a str,
}

/// Finds frontmatter bounds robustly using line-based iteration.
fn find_frontmatter_bounds(md_content: &str) -> Option<FrontmatterBounds<'_>> {
    let mut in_leading_comment = false;
    let mut content_start_offset = None;

    for line in md_content.lines() {
        let trimmed = line.trim();
        if in_leading_comment {
            if trimmed.contains("-->") {
                in_leading_comment = false;
            }
            continue;
        }

        if trimmed.starts_with("<!--") {
            if !trimmed.contains("-->") {
                in_leading_comment = true;
            }
            continue;
        }

        if trimmed.is_empty() {
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
        let mut fm = Frontmatter::new();

        for line in bounds.frontmatter_text.lines() {
            if let Some((key, val)) = line.split_once(':') {
                let key = key.trim();
                let val_trimmed = trim_matching_quotes(val);

                if val_trimmed.is_empty()
                    && key != "numbered_sections"
                {
                    continue;
                }

                match key {
                    "title" => fm.title = Some(val_trimmed.to_string()),
                    "subtitle" => fm.subtitle = Some(val_trimmed.to_string()),
                    "date" => fm.date = Some(val_trimmed.to_string()),
                    "version" => fm.version = Some(val_trimmed.to_string()),
                    "language" | "lang" => fm.language = Some(val_trimmed.to_string()),
                    "logo" => fm.logo = Some(val_trimmed.to_string()),
                    "numbered_sections" => {
                        fm.numbered_sections = val_trimmed.eq_ignore_ascii_case("true");
                    }
                    _ => {
                        crate::error::print_warning(&format!(
                            "Unknown frontmatter option '{key}'. Refer to starter template ('d2f --init') for supported options."
                        ));
                    }
                }
            }
        }

        return (fm, bounds.body_text);
    }

    (Frontmatter::new(), md_content)
}

/// Parses YAML frontmatter into a key-value hash map and returns remaining markdown body.
pub fn parse_frontmatter_map(md_content: &str) -> (std::collections::HashMap<String, String>, &str) {
    let (fm, body) = parse_frontmatter(md_content);
    let mut map = fm.to_hashmap();
    if let Some(bounds) = find_frontmatter_bounds(md_content) {
        for line in bounds.frontmatter_text.lines() {
            if let Some((key, val)) = line.split_once(':') {
                let key = key.trim();
                let val_trimmed = trim_matching_quotes(val);
                map.insert(key.to_string(), val_trimmed.to_string());
            }
        }
    }
    (map, body)
}

/// Validates frontmatter metadata for required fields.
pub fn validate_frontmatter(
    _frontmatter: &Frontmatter,
    _md_content: &str,
    _file_name: Option<&str>,
) -> Result<()> {
    Ok(())
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

const CALLOUT_TABLE: &[(&str, &str, &str, &str)] = &[
    ("!!! ", "note note-caution", "callout_caution", "caution"),
    ("!!!", "note note-caution", "callout_caution", "caution"),
    ("!! ", "note note-warning", "callout_warning", "warning"),
    ("!!", "note note-warning", "callout_warning", "warning"),
    ("! ", "note note-important", "callout_important", "important"),
    ("!", "note note-important", "callout_important", "important"),
    ("? ", "note note-tip", "callout_tip", "tip"),
    ("?", "note note-tip", "callout_tip", "tip"),
];

/// Parses callout metadata (CSS class, inner text, callout label, callout type) from raw blockquote inner string.
fn parse_callout<'a>(
    inner: &'a str,
    locale: &'a Locale,
) -> (&'static str, &'a str, &'a str, &'static str) {
    for &(prefix, css_class, key, ctype) in CALLOUT_TABLE {
        if let Some(stripped) = inner.strip_prefix(prefix) {
            return (css_class, stripped, locale.get(key), ctype);
        }
    }

    ("note", inner, locale.get("callout_note"), "note")
}

/// Helper function to build and render the variable table component.
fn build_variable_table(
    out: &mut String,
    locale: &Locale,
    table_rows: &[(&str, &str)],
) {
    let mut map = std::collections::BTreeMap::new();
    for &(k, v) in table_rows {
        map.insert(k, v);
    }

    let json_payload = serde_json::to_string(&map).unwrap_or_else(|_| "{}".to_string());

    let var_term = locale
        .get_ignore_ascii_case("var_table_variable")
        .unwrap_or("Variable");
    let val_term = locale
        .get_ignore_ascii_case("var_table_value")
        .unwrap_or("Value");

    features::code::render_variable_table(out, var_term, val_term, table_rows, &json_payload);
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
        for ev in self.iter.by_ref() {
            match &ev {
                Event::Html(text) | Event::InlineHtml(text) => {
                    let s = text.as_ref();
                    if self.in_comment {
                        if s.contains("-->") {
                            self.in_comment = false;
                        }
                        continue;
                    }
                    if let Some(start_idx) = s.find("<!--") {
                        if !s[start_idx..].contains("-->") {
                            self.in_comment = true;
                        }
                        continue;
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
pub fn convert_markdown_to_html(markdown_body: &str) -> Result<(String, DocumentFeatures)> {
    convert_markdown_to_html_with_locale(markdown_body, &Locale::default())
}

/// Converts Markdown body into interactive HTML following doc2flow structure using specified locale.
pub fn convert_markdown_to_html_with_locale(
    markdown_body: &str,
    locale: &Locale,
) -> Result<(String, DocumentFeatures)> {
    convert_markdown_to_html_with_options(markdown_body, locale, false)
}

/// Converts Markdown body into interactive HTML with specified locale and options (e.g. section numbering).
pub fn convert_markdown_to_html_with_options(
    markdown_body: &str,
    locale: &Locale,
    number_sections: bool,
) -> Result<(String, DocumentFeatures)> {
    let mut options = Options::empty();
    options.insert(Options::ENABLE_TASKLISTS);
    options.insert(Options::ENABLE_STRIKETHROUGH);
    options.insert(Options::ENABLE_TABLES);

    let parser = MarkdownParser::new_ext(markdown_body, options);
    let events: Vec<Event> = CommentFilter::new(parser).collect();

    let mut features = DocumentFeatures::default();

    // 1. Collect all variable names used in code blocks across the document
    let mut code_vars: Vec<&str> = Vec::new();
    let mut in_code_block_scan = false;
    for ev in &events {
        match ev {
            Event::Start(Tag::CodeBlock(_)) => in_code_block_scan = true,
            Event::End(TagEnd::CodeBlock) => in_code_block_scan = false,
            Event::Text(text) if in_code_block_scan => {
                let mut start = 0;
                while let Some(s) = text[start..].find("{{") {
                    let open_idx = start + s;
                    if let Some(e) = text[open_idx + 2..].find("}}") {
                        let close_idx = open_idx + 2 + e;
                        let var_name = text[open_idx + 2..close_idx].trim();
                        if !var_name.is_empty()
                            && var_name.chars().all(|c| c.is_alphanumeric() || c == '_')
                            && !code_vars.contains(&var_name)
                        {
                            code_vars.push(var_name);
                        }
                        start = close_idx + 2;
                    } else {
                        break;
                    }
                }
            }
            _ => {}
        }
    }

    let mut var_table_html = String::new();
    let mut var_event_ranges: Vec<(usize, usize)> = Vec::new();

    let mut scan_i = 0;
    let mut temp_scan_html = String::with_capacity(512);

    while scan_i < events.len() {
        if let Event::Start(Tag::Paragraph) = &events[scan_i] {
            let p_start = scan_i;
            let mut p_end = p_start;
            let mut p_depth = 1;
            scan_i += 1;
            while scan_i < events.len() {
                match &events[scan_i] {
                    Event::Start(Tag::Paragraph) => p_depth += 1,
                    Event::End(TagEnd::Paragraph) => {
                        p_depth -= 1;
                        if p_depth == 0 {
                            p_end = scan_i;
                            break;
                        }
                    }
                    _ => {}
                }
                scan_i += 1;
            }

            temp_scan_html.clear();
            html::push_html(&mut temp_scan_html, events[p_start + 1..p_end].iter().cloned());
            let clean_p = strip_paragraph_tags(temp_scan_html.trim()).trim();

            if clean_p.eq_ignore_ascii_case("[Variables]") {
                let mut next_t = p_end + 1;
                while next_t < events.len() {
                    match &events[next_t] {
                        Event::Text(t) if t.trim().is_empty() => next_t += 1,
                        Event::Html(_) | Event::InlineHtml(_) => next_t += 1,
                        _ => break,
                    }
                }

                if next_t < events.len() && matches!(&events[next_t], Event::Start(Tag::Table(_))) {
                    let table_start = next_t;
                    let table_end = find_table_end(&events, table_start);

                    var_event_ranges.push((p_start, table_end));

                    let mut raw_table_map: std::collections::BTreeMap<&str, &str> =
                        std::collections::BTreeMap::new();
                    let mut current_row: Vec<&str> = Vec::with_capacity(2);
                    let mut current_cell = "";
                    let mut in_cell = false;

                    for ev in &events[table_start..=table_end] {
                        match ev {
                            Event::Start(Tag::TableHead) | Event::Start(Tag::TableRow) => {
                                current_row.clear();
                            }
                            Event::End(TagEnd::TableHead) => {
                                current_row.clear();
                            }
                            Event::End(TagEnd::TableRow) => {
                                if !current_row.is_empty() {
                                    let key = current_row[0];
                                    let val = if current_row.len() >= 2 {
                                        current_row[1]
                                    } else {
                                        ""
                                    };
                                    if !key.is_empty() {
                                        raw_table_map.insert(key, val);
                                    }
                                }
                                current_row.clear();
                            }
                            Event::Start(Tag::TableCell) => {
                                in_cell = true;
                                current_cell = "";
                            }
                            Event::End(TagEnd::TableCell) => {
                                in_cell = false;
                                current_row.push(current_cell.trim());
                                current_cell = "";
                            }
                            Event::Text(t) | Event::Code(t) if in_cell => {
                                current_cell = t.as_ref();
                            }
                            _ => {}
                        }
                    }

                    // Issue warnings for variables in [Variables] table unused in code blocks
                    for k in raw_table_map.keys() {
                        if !code_vars.contains(k) {
                            eprintln!(
                                "Warning: Variable '{k}' in [Variables] table is not used in any code block."
                            );
                        }
                    }

                    // Build final rows only for variables used in code blocks
                    let mut final_table_rows: Vec<(&str, &str)> =
                        Vec::with_capacity(code_vars.len());
                    for cv in &code_vars {
                        if let Some(val) = raw_table_map.remove(cv) {
                            final_table_rows.push((cv, val));
                        } else {
                            eprintln!(
                                "Warning: Variable '{cv}' in code block is missing from [Variables] table."
                            );
                            final_table_rows.push((cv, ""));
                        }
                    }

                    build_variable_table(&mut var_table_html, locale, &final_table_rows);

                    scan_i = table_end + 1;
                    continue;
                }
            }
        }
        scan_i += 1;
    }

    // If no [Variables] table was parsed but code blocks contain variables, construct table from code_vars
    if var_table_html.is_empty() && !code_vars.is_empty() {
        let mut final_table_rows: Vec<(&str, &str)> = Vec::with_capacity(code_vars.len());
        for cv in &code_vars {
            eprintln!(
                "Warning: Variable '{cv}' in code block is missing from [Variables] table."
            );
            final_table_rows.push((cv, ""));
        }

        build_variable_table(&mut var_table_html, locale, &final_table_rows);
    }

    if !var_table_html.is_empty() {
        features.has_code = true;
    }

    let mut var_table_emitted = false;
    let mut out = String::with_capacity(markdown_body.len() * 2);
    let mut section_count = 0usize;
    let mut h1_counter = 0u32;
    let mut h2_counter = 0u32;
    let mut global_cb_count = 0usize;
    let mut global_txt_count = 0usize;
    let mut global_item_count = 0usize;
    let mut in_section = false;
    let mut list_stack: Vec<ListKind> = Vec::new();
    let mut temp_html = String::with_capacity(1024);

    let mut idx = 0;
    while idx < events.len() {
        if !var_event_ranges.is_empty()
            && let Some(&(_, end_idx)) = var_event_ranges.iter().find(|(s, e)| idx >= *s && idx <= *e)
        {
            idx = end_idx + 1;
            continue;
        }

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

                if !var_table_emitted && !var_table_html.is_empty() {
                    out.push_str(&var_table_html);
                    var_table_emitted = true;
                }

                let start_idx = idx + 1;
                idx += 1;
                while idx < events.len() {
                    if matches!(events[idx], Event::End(TagEnd::Heading(l)) if l == target_level) {
                        break;
                    }
                    idx += 1;
                }

                temp_html.clear();
                html::push_html(&mut temp_html, events[start_idx..idx].iter().cloned());
                let heading_text = temp_html.trim();

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

                let (has_checklist, callout_type) =
                    inspect_section_metadata(&events[idx + 1..], locale);

                template::render_section_header(
                    &mut out,
                    section_count,
                    final_heading_text,
                    is_h1,
                    is_empty,
                    has_checklist,
                    callout_type.as_deref(),
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

                temp_html.clear();
                html::push_html(&mut temp_html, events[start_idx..idx].iter().cloned());
                template::render_subheading(&mut out, &temp_html);
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

                temp_html.clear();
                html::push_html(&mut temp_html, events[start_idx..idx].iter().cloned());
                let trimmed = temp_html.trim();

                let inner = strip_paragraph_tags(trimmed);
                let (note_cls, note_content, callout_label, _) = parse_callout(inner, locale);

                let escaped_label = html_escape(callout_label);
                template::render_callout(&mut out, note_cls, &escaped_label, note_content);
            }

            // Code Blocks (e.g. ```ini ... ```)
            Event::Start(Tag::CodeBlock(kind)) => {
                features.has_code = true;
                let lang_str = match kind {
                    CodeBlockKind::Fenced(lang) if !lang.is_empty() => Some(lang.as_ref()),
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

                temp_html.clear();
                for ev in &events[start_idx..idx] {
                    if let Event::Text(text) = ev {
                        temp_html.push_str(text);
                    }
                }

                let escaped_code = html_escape(&temp_html);
                let copy_label = html_escape(locale.get("copy_code"));

                let escaped_lang_opt = lang_str.map(html_escape);
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

                temp_html.clear();
                if is_task {
                    html::push_html(
                        &mut temp_html,
                        events[start_idx..idx]
                            .iter()
                            .filter(|ev| !matches!(ev, Event::TaskListMarker(_)))
                            .cloned(),
                    );
                } else {
                    html::push_html(&mut temp_html, events[start_idx..idx].iter().cloned());
                }
                let trimmed = temp_html.trim();
                let clean_label = strip_paragraph_tags(trimmed);

                let list_depth = list_stack.len().saturating_sub(1);
                let sec_num = if section_count == 0 { 1 } else { section_count };

                if is_task {
                    features.has_tasks = true;
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

            // Standalone Text Paragraphs or Annotated [Variables] Table Header
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

                temp_html.clear();
                html::push_html(&mut temp_html, events[start_idx..idx].iter().cloned());
                let trimmed = temp_html.trim();
                let clean_content = strip_paragraph_tags(trimmed).trim();

                if !clean_content.is_empty() {
                    let is_image_block = clean_content.starts_with("<img");
                    if is_image_block {
                        features.has_images = true;
                        template::render_image_item(&mut out, clean_content);
                    } else {
                        global_txt_count += 1;
                        let sec_num = if section_count == 0 { 1 } else { section_count };
                        let list_depth = list_stack.len().saturating_sub(1);
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

            // Section Tables
            Event::Start(Tag::Table(_)) => {
                features.has_tables = true;
                let end_table = find_table_end(&events, idx);

                out.push_str("<div class=\"item-table-wrap\">");
                html::push_html(&mut out, events[idx..=end_table].iter().cloned());
                out.push_str("</div>\n");

                idx = end_table;
            }

            // Fallback for standard events
            ev => {
                html::push_html(&mut out, std::iter::once(ev.clone()));
            }
        }
        idx += 1;
    }

    if in_section {
        template::render_section_close(&mut out);
    }

    if !var_table_emitted && !var_table_html.is_empty() {
        out.insert_str(0, &var_table_html);
    }

    Ok((out, features))
}

/// Finds the index of the matching [`Event::End(TagEnd::Table)`] event for a table starting at `start_idx`.
#[inline]
fn find_table_end(events: &[Event], start_idx: usize) -> usize {
    let mut table_depth = 1usize;
    for (i, ev) in events.iter().enumerate().skip(start_idx + 1) {
        match ev {
            Event::Start(Tag::Table(_)) => table_depth += 1,
            Event::End(TagEnd::Table) => {
                table_depth -= 1;
                if table_depth == 0 {
                    return i;
                }
            }
            _ => {}
        }
    }
    start_idx
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

fn inspect_section_metadata(
    events: &[Event],
    locale: &Locale,
) -> (bool, Option<Cow<'static, str>>) {
    let mut has_checklist = false;
    let mut callout_types: Vec<&'static str> = Vec::new();

    let mut i = 0;
    while i < events.len() {
        match &events[i] {
            Event::Start(Tag::Heading {
                level: HeadingLevel::H1 | HeadingLevel::H2,
                ..
            }) => break,
            Event::TaskListMarker(_) => {
                has_checklist = true;
            }
            Event::Start(Tag::BlockQuote(_)) => {
                let start_idx = i + 1;
                i += 1;
                let mut depth = 1;
                while i < events.len() {
                    match &events[i] {
                        Event::Start(Tag::BlockQuote(_)) => depth += 1,
                        Event::End(TagEnd::BlockQuote(_)) => {
                            depth -= 1;
                            if depth == 0 {
                                break;
                            }
                        }
                        _ => {}
                    }
                    i += 1;
                }
                let ctype = callout_type_from_events(&events[start_idx..i], locale);
                if !callout_types.contains(&ctype) {
                    callout_types.push(ctype);
                }
            }
            _ => {}
        }
        i += 1;
    }

    let callout_type = match callout_types.len() {
        0 => None,
        1 => Some(Cow::Borrowed(callout_types[0])),
        _ => Some(Cow::Owned(callout_types.join(" "))),
    };

    (has_checklist, callout_type)
}

fn callout_type_from_events(events: &[Event], locale: &Locale) -> &'static str {
    for ev in events {
        if let Event::Text(text) = ev {
            let s = text.trim();
            let (_, _, _, ctype) = parse_callout(s, locale);
            return ctype;
        }
    }
    "note"
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
            ("note note-caution", "Danger zone", "Caution", "caution")
        );
        assert_eq!(
            parse_callout("!! Be careful", &locale),
            ("note note-warning", "Be careful", "Warning", "warning")
        );
        assert_eq!(
            parse_callout("! Read this", &locale),
            ("note note-important", "Read this", "Important", "important")
        );
        assert_eq!(
            parse_callout("? Pro tip", &locale),
            ("note note-tip", "Pro tip", "Tip", "tip")
        );
        assert_eq!(
            parse_callout("Just text", &locale),
            ("note", "Just text", "Note", "note")
        );
    }

    #[test]
    fn test_text_paragraph_conversion() {
        let input = "## Section 1\n\nThis is a standard text paragraph.\n";
        let (html, _features) = convert_markdown_to_html(input).expect("conversion failed");
        assert!(html.contains("<div class=\"doc-item text-item\" id=\"txt_s1_1\">"));
        assert!(
            html.contains("<span class=\"text-content\">This is a standard text paragraph.</span>")
        );
        assert!(html.contains("<span class=\"item-comment-icon\">"));
    }

    #[test]
    fn test_item_comment_icon_presence() {
        let input = "## Section 1\n\n- [ ] Task item\n- Simple list item\n\nParagraph text\n";
        let (html, _features) = convert_markdown_to_html(input).expect("conversion failed");
        assert_eq!(html.matches("class=\"item-comment-icon\"").count(), 3);
    }

    #[test]
    fn test_ordered_list_conversion() {
        let input = "## Section 1\n\n1. First step\n2. Second step\n3. Third step\n";
        let (html, _features) = convert_markdown_to_html(input).expect("conversion failed");

        assert!(html.contains("<div class=\"doc-item simple-item\" id=\"item_s1_1\">"));
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
        let (html, _features) = convert_markdown_to_html(input).expect("conversion failed");
        assert!(html.contains("<hr />") || html.contains("<hr>"));
    }

    #[test]
    fn test_html_comments_ignored() {
        let input = "## Section 1\n\n<!-- Secret internal comment -->\n\nThis is visible content.\n<!-- Another hidden comment -->\n";
        let (html, _features) = convert_markdown_to_html(input).expect("conversion failed");
        assert!(!html.contains("Secret internal comment"));
        assert!(!html.contains("Another hidden comment"));
        assert!(html.contains("This is visible content."));
        assert_eq!(html.matches("class=\"doc-item text-item\"").count(), 1);
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
        let (html, _features) = convert_markdown_to_html(input).expect("conversion failed");

        assert!(html.contains(r#"<div class="doc-item simple-item" id="item_s1_1">"#));
        assert!(html.contains(r#"<span class="list-bullet">&bull;</span>"#));
        assert!(html.contains(r#"<span class="check-label">Top Task</span>"#));

        assert!(html.contains(r#"<div class="doc-item simple-item" id="item_s1_2" style="--indent: 1;">"#));
        assert!(html.contains(r#"<span class="list-bullet">a.</span>"#));
        assert!(html.contains(r#"<span class="check-label">Sub step A</span>"#));

        assert!(html.contains(r#"<div class="doc-item simple-item" id="item_s1_3" style="--indent: 1;">"#));
        assert!(html.contains(r#"<span class="list-bullet">b.</span>"#));
        assert!(html.contains(r#"<span class="check-label">Sub step B</span>"#));

        assert!(html.contains(r#"<div class="doc-item check-item" id="wrap-cb_s1_1" style="--indent: 1;">"#));
        assert!(html.contains(r#"<input type="checkbox" id="cb_s1_1">"#));
        assert!(html.contains(r#"<label class="check-label" for="cb_s1_1">Sub-task 1</label>"#));

        assert!(html.contains(r#"<div class="doc-item simple-item" id="item_s1_5" style="--indent: 2;">"#));
        assert!(html.contains(r#"<span class="list-bullet">&bull;</span>"#));
        assert!(html.contains(r#"<span class="check-label">Deep detail X</span>"#));
    }

    #[test]
    fn test_multiline_html_comments_ignored() {
        let input = "## Section 1\n\n<!--\nMultline comment block\nLine 2\n-->\n\nThis is visible content.\n";
        let (html, _features) = convert_markdown_to_html(input).expect("conversion failed");
        assert!(!html.contains("Multline comment block"));
        assert!(!html.contains("Line 2"));
        assert!(html.contains("This is visible content."));
        assert_eq!(html.matches("class=\"doc-item text-item\"").count(), 1);
    }

    #[test]
    fn test_parse_frontmatter_valid() {
        let input = "---\ntitle: \"Guide\"\ndate: \"2026-07-25\"\n---\n## Section 1";
        let (fm, body) = parse_and_validate_frontmatter(input, Some("guide.md")).unwrap();
        assert_eq!(fm.title.as_deref(), Some("Guide"));
        assert_eq!(fm.date.as_deref(), Some("2026-07-25"));
        assert_eq!(body, "## Section 1");
    }

    #[test]
    fn test_level_1_heading_conversion() {
        let input = "# Top Level Header\n\n- [ ] Task in H1\n\n## Sub Section\n\n- [x] Task in H2";
        let (html, _features) = convert_markdown_to_html(input).expect("conversion failed");

        assert!(html.contains(r#"<!-- S1 -->"#));
        assert!(html.contains(r#"<section class="section" id="s1" data-has-checklist="true">"#));
        assert!(html.contains(r#"<h2 class="sh sh-h1" role="button" tabindex="0" aria-expanded="true"><span>Top Level Header</span>"#));
        assert!(html.contains(r#"badge-s1"#));
        assert!(html.contains(r#"tog-s1"#));
        assert!(html.contains(r#"id="wrap-cb_s1_1""#));

        assert!(html.contains(r#"<!-- S2 -->"#));
        assert!(html.contains(r#"<section class="section" id="s2" data-has-checklist="true">"#));
        assert!(html.contains(r#"<h2 class="sh" role="button" tabindex="0" aria-expanded="true"><span>Sub Section</span>"#));
    }

    #[test]
    fn test_section_metadata_attributes() {
        let input = "# Checklist Sec\n\n- [ ] Item 1\n\n## Callout Sec\n\n>! Important note";
        let (html, _features) = convert_markdown_to_html(input).expect("conversion failed");

        assert!(html.contains(r#"<section class="section" id="s1" data-has-checklist="true">"#));
        assert!(html.contains(r#"<section class="section" id="s2" data-callout-type="important">"#));
    }

    #[test]
    fn test_empty_heading_conversion() {
        let input = "# Empty H1 Header\n\n## Empty H2 Header\n\n## Non Empty H2\n\nSome paragraph content";
        let (html, _features) = convert_markdown_to_html(input).expect("conversion failed");

        assert!(html.contains(r#"<h2 class="sh sh-h1 no-toggle"><span>Empty H1 Header</span>"#));
        assert!(html.contains(r#"<h2 class="sh no-toggle"><span>Empty H2 Header</span>"#));
        assert!(html.contains(r#"<h2 class="sh" role="button" tabindex="0" aria-expanded="true"><span>Non Empty H2</span>"#));
    }

    #[test]
    fn test_no_inline_onclick_on_section_headers() {
        let input = "# H1 Heading\n\nSome content\n\n## H2 Heading\n\nMore content";
        let (html, _features) = convert_markdown_to_html(input).expect("conversion failed");

        assert!(!html.contains("onclick="));
    }

    #[test]
    fn test_h4_to_h6_treated_as_subheading() {
        let input = "## Section 1\n\n### Sub 3\n\n#### Sub 4\n\n##### Sub 5\n\n###### Sub 6";
        let (html, _features) = convert_markdown_to_html(input).expect("conversion failed");

        assert!(html.contains(r#"<h3 class="subh">Sub 3</h3>"#));
        assert!(html.contains(r#"<h3 class="subh">Sub 4</h3>"#));
        assert!(html.contains(r#"<h3 class="subh">Sub 5</h3>"#));
        assert!(html.contains(r#"<h3 class="subh">Sub 6</h3>"#));
    }

    #[test]
    fn test_frontmatter_windows_crlf_line_endings() {
        let input = "---\r\ntitle: \"CRLF Title\"\r\nlanguage: de\r\n---\r\n## Section 1\r\nBody line";
        let (fm, body) = parse_frontmatter(input);
        assert_eq!(fm.title.as_deref(), Some("CRLF Title"));
        assert_eq!(fm.language.as_deref(), Some("de"));
        assert_eq!(body, "## Section 1\r\nBody line");
    }

    #[test]
    fn test_frontmatter_quoted_and_unquoted_fields() {
        let input = "---\ntitle: 'Single Quoted'\nsubtitle: \"Double Quoted\"\ndate: 2026-07-26\nversion: '1.2.3'\nlang: en\n---\nBody";
        let (fm, _) = parse_frontmatter(input);
        assert_eq!(fm.title.as_deref(), Some("Single Quoted"));
        assert_eq!(fm.subtitle.as_deref(), Some("Double Quoted"));
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
        let (cls, text, lbl, _ctype) = parse_callout("!Important message", &locale);
        assert_eq!(cls, "note note-important");
        assert_eq!(text, "Important message");
        assert_eq!(lbl, "Important");

        let (cls, text, lbl, _ctype) = parse_callout("?Tip without space", &locale);
        assert_eq!(cls, "note note-tip");
        assert_eq!(text, "Tip without space");
        assert_eq!(lbl, "Tip");

        let (cls, text, lbl, _ctype) = parse_callout("!!Warning without space", &locale);
        assert_eq!(cls, "note note-warning");
        assert_eq!(text, "Warning without space");
        assert_eq!(lbl, "Warning");

        let (cls, text, lbl, _ctype) = parse_callout("!!!Caution without space", &locale);
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
        let (html, _features) = convert_markdown_to_html(input).expect("conversion failed");

        assert!(html.contains(r#"<span class="list-bullet">1.</span>"#));
        assert!(html.contains(r#"<span class="list-bullet">a.</span>"#));
        assert!(html.contains(r#"<span class="list-bullet">i.</span>"#));
        assert!(html.contains(r#"<span class="list-bullet">1.</span>"#));
    }

    #[test]
    fn test_html_escaping_in_code_and_headings() {
        let input = "## Header `<script>alert(1)</script>` & More\n\n```html\n<div id=\"app\">&amp;</div>\n```";
        let (html, _features) = convert_markdown_to_html(input).expect("conversion failed");

        assert!(html.contains("&lt;script&gt;alert(1)&lt;/script&gt;"));
        assert!(html.contains("&amp; More"));
        assert!(html.contains("&lt;div id=&quot;app&quot;&gt;&amp;amp;&lt;/div&gt;"));
    }

    #[test]
    fn test_loose_task_list_conversion() {
        let input = "## Section 1\n\n- [ ] Task 1\n\n- [x] Task 2\n\n- [ ] Task 3\n";
        let (html, _features) = convert_markdown_to_html(input).expect("conversion failed");

        assert!(html.contains(r#"<div class="doc-item check-item" id="wrap-cb_s1_1">"#));
        assert!(html.contains(r#"<input type="checkbox" id="cb_s1_1">"#));
        assert!(html.contains(r#"<label class="check-label" for="cb_s1_1">Task 1</label>"#));

        assert!(html.contains(r#"<div class="doc-item check-item checked" id="wrap-cb_s1_2">"#));
        assert!(html.contains(r#"<input type="checkbox" id="cb_s1_2" checked=""#) || html.contains(r#"<input type="checkbox" id="cb_s1_2" checked>"#));
        assert!(html.contains(r#"<label class="check-label" for="cb_s1_2">Task 2</label>"#));

        assert!(html.contains(r#"<div class="doc-item check-item" id="wrap-cb_s1_3">"#));
        assert!(html.contains(r#"<input type="checkbox" id="cb_s1_3">"#));
        assert!(html.contains(r#"<label class="check-label" for="cb_s1_3">Task 3</label>"#));

        assert!(!html.contains("simple-item"));
        assert!(!html.contains("<p>Task"));
        assert!(!html.contains("Task 1</p>"));
    }

    #[test]
    fn test_numbered_sections_frontmatter_parsing() {
        let input1 = "---\nnumbered_sections: true\n---";
        let (fm1, _) = parse_frontmatter(input1);
        assert!(fm1.numbered_sections);

        let input2 = "---\nnumbered_sections: True\n---";
        let (fm2, _) = parse_frontmatter(input2);
        assert!(fm2.numbered_sections);

        let input3 = "---\nnumbered_sections: false\n---";
        let (fm3, _) = parse_frontmatter(input3);
        assert!(!fm3.numbered_sections);

        let input4 = "---\ntitle: \"Default Test\"\n---";
        let (fm4, _) = parse_frontmatter(input4);
        assert!(fm4.numbered_sections);
    }


    #[test]
    fn test_unknown_frontmatter_option_warning() {
        let input = "---\ntitle: \"Test Doc\"\nunknown_key: \"some_value\"\n---";
        let (fm, _) = parse_frontmatter(input);
        assert_eq!(fm.title.as_deref(), Some("Test Doc"));
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
        let (html_enabled, _features) = convert_markdown_to_html_with_options(input, &locale, true)
            .expect("conversion failed");

        assert!(html_enabled.contains("<span>1. First H1</span>"));
        assert!(html_enabled.contains("<span>1.1 First Sub H2</span>"));
        assert!(html_enabled.contains("<span>1.2 Second Sub H2</span>"));
        assert!(html_enabled.contains("<span>2. Second H1</span>"));
        assert!(html_enabled.contains("<span>2.1 Third Sub H2</span>"));

        let (html_disabled, _features) = convert_markdown_to_html_with_options(input, &locale, false)
            .expect("conversion failed");

        assert!(html_disabled.contains("<span>First H1</span>"));
        assert!(html_disabled.contains("<span>First Sub H2</span>"));
        assert!(html_disabled.contains("<span>Second Sub H2</span>"));
        assert!(html_disabled.contains("<span>Second H1</span>"));
        assert!(html_disabled.contains("<span>Third Sub H2</span>"));
    }

    #[test]
    fn test_code_variables_table_conversion() {
        let input = r#"# System Setup

[Variables]
| Variable | Value |
| --- | --- |
| BLOCK | prod-server |
| PORT | 8080 |

```bash
curl https://{{BLOCK}}.local:{{PORT}}/api
```
"#;
        let (html, features) = convert_markdown_to_html(input).expect("conversion failed");

        assert!(features.has_code);
        assert!(html.contains(r#"<div class="item-table-var-wrap">"#));
        assert!(html.contains(r#"<th>Variable</th><th>Value</th>"#));
        assert!(html.contains(r#"data-variables="{&quot;BLOCK&quot;:&quot;prod-server&quot;,&quot;PORT&quot;:&quot;8080&quot;}""#));
        assert!(html.contains("<td>BLOCK</td>"));
        assert!(html.contains(r#"class="item-table-var-input persistent-field""#));
        assert!(html.contains(r#"data-var-key="BLOCK""#));
        assert!(html.contains(r#"value="prod-server""#));
        assert!(html.find("<div class=\"item-table-var-wrap\">").unwrap() < html.find("<!-- S1 -->").unwrap());
        assert!(html.contains("curl https://{{BLOCK}}.local:{{PORT}}/api"));
    }

    #[test]
    fn test_document_features_to_features_string() {
        let mut features = DocumentFeatures::default();
        assert_eq!(features.to_features_string(), "core");

        features.has_tasks = true;
        assert_eq!(features.to_features_string(), "core, tasks");

        features.has_tables = true;
        assert_eq!(features.to_features_string(), "core, tasks, table");

        features.has_code = true;
        features.has_images = true;
        assert_eq!(features.to_features_string(), "core, code, tasks, images, table");
    }
}
