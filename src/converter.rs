use anyhow::Result;
use pulldown_cmark::{
    CodeBlockKind, Event, HeadingLevel, Options, Parser as MarkdownParser, Tag, TagEnd, html,
};

/// Escapes HTML special characters in code strings.
fn html_escape(input: &str) -> String {
    input
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

/// Frontmatter metadata extracted from Markdown header.
#[derive(Debug, Default)]
pub struct Frontmatter {
    pub title: String,
    pub subtitle: String,
    pub customer: String,
    pub employee: String,
    pub technician: String,
    pub date: String,
}

/// Parses YAML-style frontmatter delimited by `---`.
pub fn parse_frontmatter(md_content: &str) -> (Frontmatter, &str) {
    let mut fm = Frontmatter::default();

    let search_str = if md_content.starts_with("---\r\n") {
        "---\r\n"
    } else if md_content.starts_with("---\n") {
        "---\n"
    } else {
        return (fm, md_content);
    };

    let start_len = search_str.len();

    if let Some(end_idx) = md_content[start_len..].find(search_str) {
        let frontmatter_text = &md_content[start_len..start_len + end_idx];
        let body_start = start_len + end_idx + start_len;

        for line in frontmatter_text.lines() {
            let parts: Vec<&str> = line.splitn(2, ':').collect();
            if parts.len() == 2 {
                let key = parts[0].trim();
                let val = parts[1].trim().trim_matches('"');
                match key {
                    "title" => fm.title = val.to_string(),
                    "subtitle" => fm.subtitle = val.to_string(),
                    "customer" => fm.customer = val.to_string(),
                    "employee" => fm.employee = val.to_string(),
                    "technician" => fm.technician = val.to_string(),
                    "date" => fm.date = val.to_string(),
                    _ => {}
                }
            }
        }

        (fm, &md_content[body_start..])
    } else {
        (fm, md_content)
    }
}

/// Converts Markdown body into interactive HTML following doc2flow structure.
pub fn convert_markdown_to_html(markdown_body: &str) -> Result<String> {
    let mut options = Options::empty();
    options.insert(Options::ENABLE_TASKLISTS);
    options.insert(Options::ENABLE_STRIKETHROUGH);
    options.insert(Options::ENABLE_TABLES);

    let parser = MarkdownParser::new_ext(markdown_body, options);
    let events: Vec<Event> = parser.collect();

    let mut out = String::new();
    let mut section_count = 0usize;
    let mut global_cb_count = 0usize;
    let mut in_section = false;

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

            // Blockquotes (> i Note text)
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
                let inner = if trimmed.starts_with("<p>") && trimmed.ends_with("</p>") {
                    &trimmed[3..trimmed.len() - 4]
                } else {
                    trimmed
                };

                let note_content = if let Some(stripped) = inner.strip_prefix("i ") {
                    format!("&#x24D8; {}", stripped)
                } else if let Some(stripped) = inner.strip_prefix("info ") {
                    format!("&#x24D8; {}", stripped)
                } else {
                    inner.to_string()
                };

                out.push_str(&format!(
                    "<div class=\"note\">{}</div>\n",
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

                out.push_str(&format!(
                    "<div class=\"code-block-wrap\"><div class=\"code-header\">{}<button class=\"copy-btn\" onclick=\"copyCode(this)\" title=\"Copy code\" aria-label=\"Copy code\">{}</button></div><pre class=\"code-block{}\"><code>{}</code></pre></div>\n",
                    lang_span, copy_icon, lang_cls, escaped_code
                ));
            }

            // Task List Items (- [ ] or - [x])
            Event::Start(Tag::Item) => {
                // Check if the next event is TaskListMarker
                if let Some(Event::TaskListMarker(checked)) = events.get(idx + 1) {
                    let is_checked = *checked;
                    global_cb_count += 1;

                    // Advance past Start(Item) and TaskListMarker
                    idx += 2;

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
                            _ => {}
                        }
                        item_events.push(events[idx].clone());
                        idx += 1;
                    }

                    let mut label_html = String::new();
                    html::push_html(&mut label_html, item_events.into_iter());
                    let trimmed = label_html.trim();
                    let clean_label = if trimmed.starts_with("<p>") && trimmed.ends_with("</p>") {
                        &trimmed[3..trimmed.len() - 4]
                    } else {
                        trimmed
                    };

                    let sec_num = if section_count == 0 { 1 } else { section_count };
                    let checked_attr = if is_checked { " checked" } else { "" };
                    let checked_cls = if is_checked { " checked" } else { "" };

                    out.push_str(&format!(
                            "<div class=\"check-item{}\" id=\"wrap-cb_s{sec_num}_{global_cb_count}\">\n",
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

                    idx += 1;
                    continue;
                }

                // Standard item (non-tasklist) fallback
                out.push_str("<li>");
            }

            // Suppress <ul> and </ul> wrappers around tasklists if they only contain task items
            Event::Start(Tag::List(_)) => {
                // Do not output <ul> for task lists
            }
            Event::End(TagEnd::List(_)) => {
                // Do not output </ul> for task lists
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
