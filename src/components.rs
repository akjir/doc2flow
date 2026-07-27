//! Reusable HTML UI components and building blocks for Doc2Flow.

use std::fmt::Write;

/// Embedded SVG comment icon for interactive elements.
pub const COMMENT_ICON_SVG: &str = r#"<span class="item-comment-icon"><svg width="15" height="15" viewBox="0 0 32 32" aria-hidden="true"><g fill="currentColor" transform="translate(-204, -255)"><path d="M228,267 C226.896,267 226,267.896 226,269 C226,270.104 226.896,271 228,271 C229.104,271 230,270.104 230,269 C230,267.896 229.104,267 228,267 L228,267 Z M220,281 C218.832,281 217.704,280.864 216.62,280.633 L211.912,283.463 L211.975,278.824 C208.366,276.654 206,273.066 206,269 C206,262.373 212.268,257 220,257 C227.732,257 234,262.373 234,269 C234,275.628 227.732,281 220,281 L220,281 Z M220,255 C211.164,255 204,261.269 204,269 C204,273.419 206.345,277.354 210,279.919 L210,287 L217.009,282.747 C217.979,282.907 218.977,283 220,283 C228.836,283 236,276.732 236,269 C236,261.269 228.836,255 220,255 L220,255 Z M212,267 C210.896,267 210,267.896 210,269 C210,270.104 210.896,271 212,271 C213.104,271 214,270.104 214,269 C214,267.896 213.104,267 212,267 L212,267 Z M220,267 C218.896,267 218,267.896 218,269 C218,270.104 218.896,271 220,271 C221.104,271 222,270.104 222,269 C222,267.896 221.104,267 220,267 L220,267 Z"/></g></svg></span>"#;

/// Code copy SVG icon button element.
pub const COPY_ICON_SVG: &str = r#"<svg aria-hidden="true" class="svg-icon iconCopy" width="14" height="15" viewBox="0 0 17 18"><path fill="currentColor" d="M5 6c0-1.09.91-2 2-2h4.5L15 7.5V15c0 1.09-.91 2-2 2H7c-1.09 0-2-.91-2-2zm6-1.25V8h3.25z"/><path fill="currentColor" d="M10 1a2 2 0 0 1 2 2H6a2 2 0 0 0-2 2v9a2 2 0 0 1-2-2V4a3 3 0 0 1 3-3z" opacity=".4"/></svg>"#;

/// Default embedded SVG header logo.
pub const DEFAULT_LOGO_SVG: &str = include_str!("../images/logo.svg");

/// Renders a section header component with badges and toggle buttons directly into the output buffer.
#[inline]
pub fn render_section_header(
    out: &mut impl Write,
    section_count: usize,
    heading_text: &str,
    is_h1: bool,
    is_empty: bool,
) {
    let h1_class = if is_h1 { " sh-h1" } else { "" };
    let empty_class = if is_empty { " no-toggle" } else { "" };
    let _ = write!(
        out,
        "<!-- S{section_count} -->\n<div class=\"section\" id=\"s{section_count}\">\n<div class=\"sh{h1_class}{empty_class}\"><span>{heading_text}</span>\n<div style=\"display:flex;align-items:center;gap:8px\"><span class=\"sbadge\" id=\"badge-s{section_count}\"></span><span class=\"stog\" id=\"tog-s{section_count}\">&#9660;</span></div></div>\n<div class=\"sb\" id=\"body-s{section_count}\">\n"
    );
}

/// Renders section body and container closing tags directly into the output buffer.
#[inline]
pub fn render_section_close(out: &mut impl Write) {
    let _ = out.write_str("</div></div>\n\n");
}

/// Renders a subheading component (H3-H6) directly into the output buffer.
#[inline]
pub fn render_subheading(out: &mut impl Write, sub_html: &str) {
    let _ = writeln!(out, "<div class=\"subh\">{}</div>", sub_html.trim());
}

/// Renders an alert or callout box component directly into the output buffer.
#[inline]
pub fn render_callout(
    out: &mut impl Write,
    note_cls: &str,
    escaped_label: &str,
    note_content: &str,
) {
    let _ = writeln!(
        out,
        "<div class=\"{note_cls}\" data-label=\"{escaped_label}\">{}</div>",
        note_content.trim()
    );
}

/// Renders a code block component with header, copy button, and syntax language badge.
#[inline]
pub fn render_code_block(
    out: &mut impl Write,
    lang_opt: Option<&str>,
    escaped_code: &str,
    copy_label: &str,
) {
    let _ = out.write_str("<div class=\"code-block-wrap\"><div class=\"code-header\">");
    if let Some(lang) = lang_opt {
        let _ = write!(out, "<span class=\"code-lang\">{lang}</span>");
    }
    let _ = write!(
        out,
        "<button class=\"copy-btn\" onclick=\"copyCode(this)\" title=\"{copy_label}\" aria-label=\"{copy_label}\">{COPY_ICON_SVG}</button></div><pre class=\"code-block"
    );
    if let Some(lang) = lang_opt {
        let _ = write!(out, " language-{lang}");
    }
    let _ = writeln!(out, "\"><code>{escaped_code}</code></pre></div>");
}

/// Renders a task list checkbox item directly into the output buffer.
#[inline]
pub fn render_task_item(
    out: &mut impl Write,
    sec_num: usize,
    cb_count: usize,
    is_checked: bool,
    clean_label: &str,
    indent_depth: usize,
) {
    let checked_cls = if is_checked { " checked" } else { "" };
    let checked_attr = if is_checked { " checked" } else { "" };
    let label_text = clean_label.trim();

    if indent_depth > 0 {
        let _ = write!(
            out,
            "<div class=\"check-item{checked_cls}\" id=\"wrap-cb_s{sec_num}_{cb_count}\" style=\"--indent: {indent_depth};\">\n  <input type=\"checkbox\" id=\"cb_s{sec_num}_{cb_count}\"{checked_attr}>\n  <label class=\"check-label\" for=\"cb_s{sec_num}_{cb_count}\">{label_text}</label>\n  {COMMENT_ICON_SVG}\n</div>\n"
        );
    } else {
        let _ = write!(
            out,
            "<div class=\"check-item{checked_cls}\" id=\"wrap-cb_s{sec_num}_{cb_count}\">\n  <input type=\"checkbox\" id=\"cb_s{sec_num}_{cb_count}\"{checked_attr}>\n  <label class=\"check-label\" for=\"cb_s{sec_num}_{cb_count}\">{label_text}</label>\n  {COMMENT_ICON_SVG}\n</div>\n"
        );
    }
}

/// Renders a simple list item component directly into the output buffer.
#[inline]
pub fn render_list_item(
    out: &mut impl Write,
    sec_num: usize,
    item_count: usize,
    bullet: &str,
    clean_label: &str,
    indent_depth: usize,
) {
    let label_text = clean_label.trim();

    if indent_depth > 0 {
        let _ = write!(
            out,
            "<div class=\"check-item simple-item\" id=\"item_s{sec_num}_{item_count}\" style=\"--indent: {indent_depth};\">\n  <span class=\"list-bullet\">{bullet}</span>\n  <span class=\"check-label\">{label_text}</span>\n  {COMMENT_ICON_SVG}\n</div>\n"
        );
    } else {
        let _ = write!(
            out,
            "<div class=\"check-item simple-item\" id=\"item_s{sec_num}_{item_count}\">\n  <span class=\"list-bullet\">{bullet}</span>\n  <span class=\"check-label\">{label_text}</span>\n  {COMMENT_ICON_SVG}\n</div>\n"
        );
    }
}

/// Renders a standalone text paragraph item component directly into the output buffer.
#[inline]
pub fn render_text_item(
    out: &mut impl Write,
    sec_num: usize,
    txt_count: usize,
    content_html: &str,
    indent_depth: usize,
) {
    if indent_depth > 0 {
        let _ = write!(
            out,
            "<div class=\"check-item text-item\" id=\"txt_s{sec_num}_{txt_count}\" style=\"--indent: {indent_depth};\">\n  <span class=\"text-content\">{content_html}</span>\n  {COMMENT_ICON_SVG}\n</div>\n"
        );
    } else {
        let _ = write!(
            out,
            "<div class=\"check-item text-item\" id=\"txt_s{sec_num}_{txt_count}\">\n  <span class=\"text-content\">{content_html}</span>\n  {COMMENT_ICON_SVG}\n</div>\n"
        );
    }
}

/// Renders an image container block directly into the output buffer.
#[inline]
pub fn render_image_item(out: &mut impl Write, clean_content: &str) {
    let _ = write!(out, "<div class=\"img-item\">\n  {clean_content}\n</div>\n");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_render_section_header_and_close() {
        let mut buf = String::new();
        render_section_header(&mut buf, 1, "Section Title", true, false);
        assert!(buf.contains("<!-- S1 -->"));
        assert!(buf.contains("<div class=\"section\" id=\"s1\">"));
        assert!(buf.contains("class=\"sh sh-h1\""));
        assert!(buf.contains("Section Title"));

        render_section_close(&mut buf);
        assert!(buf.contains("</div></div>\n\n"));
    }

    #[test]
    fn test_render_subheading() {
        let mut buf = String::new();
        render_subheading(&mut buf, "Subheading Text");
        assert_eq!(buf, "<div class=\"subh\">Subheading Text</div>\n");
    }

    #[test]
    fn test_render_callout() {
        let mut buf = String::new();
        render_callout(&mut buf, "note note-tip", "Tip", "Callout content");
        assert_eq!(
            buf,
            "<div class=\"note note-tip\" data-label=\"Tip\">Callout content</div>\n"
        );
    }

    #[test]
    fn test_render_code_block() {
        let mut buf = String::new();
        render_code_block(&mut buf, Some("rust"), "fn main() {}", "Copy");
        assert!(buf.contains("<span class=\"code-lang\">rust</span>"));
        assert!(buf.contains("language-rust"));
        assert!(buf.contains("<code>fn main() {}</code>"));
    }

    #[test]
    fn test_render_task_item() {
        let mut buf = String::new();
        render_task_item(&mut buf, 1, 2, true, "Check task", 0);
        assert!(buf.contains("<div class=\"check-item checked\" id=\"wrap-cb_s1_2\">"));
        assert!(buf.contains("<input type=\"checkbox\" id=\"cb_s1_2\" checked>"));
        assert!(buf.contains("<label class=\"check-label\" for=\"cb_s1_2\">Check task</label>"));
    }

    #[test]
    fn test_render_list_item() {
        let mut buf = String::new();
        render_list_item(&mut buf, 2, 1, "&bull;", "Bullet item", 1);
        assert!(buf.contains("id=\"item_s2_1\" style=\"--indent: 1;\""));
        assert!(buf.contains("<span class=\"list-bullet\">&bull;</span>"));
    }

    #[test]
    fn test_render_text_item() {
        let mut buf = String::new();
        render_text_item(&mut buf, 1, 3, "Text line", 0);
        assert!(buf.contains("id=\"txt_s1_3\""));
        assert!(buf.contains("<span class=\"text-content\">Text line</span>"));
    }

    #[test]
    fn test_render_image_item() {
        let mut buf = String::new();
        render_image_item(&mut buf, "<img src=\"foo.png\">");
        assert_eq!(buf, "<div class=\"img-item\">\n  <img src=\"foo.png\">\n</div>\n");
    }
}
