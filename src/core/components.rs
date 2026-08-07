//! Reusable HTML UI components and building blocks for Doc2Flow.

use std::fmt::Write;

/// Embedded SVG comment icon for interactive elements.
pub const COMMENT_ICON_SVG: &str = r##"<span class="item-comment-icon"><svg width="15" height="15" viewBox="0 0 32 32" aria-hidden="true"><use href="#icon-comment"/></svg></span>"##;

/// Default embedded SVG header logo.
pub const DEFAULT_LOGO_SVG: &str = include_str!("../../resources/images/logo.svg");

/// Renders a section header component with badges and toggle buttons directly into the output buffer.
#[inline]
pub fn render_section_header(
    out: &mut impl Write,
    section_count: usize,
    heading_text: &str,
    is_h1: bool,
    is_empty: bool,
    has_checklist: bool,
    callout_type: Option<&str>,
) {
    let h1_class = if is_h1 { " sh-h1" } else { "" };
    let (empty_class, a11y_attrs) = if is_empty {
        (" no-toggle", "")
    } else {
        ("", " role=\"button\" tabindex=\"0\" aria-expanded=\"true\"")
    };
    let checklist_attr = if has_checklist {
        " data-has-checklist=\"true\""
    } else {
        ""
    };
    let _ = write!(
        out,
        "<!-- S{section_count} -->\n<section class=\"section\" id=\"s{section_count}\"{checklist_attr}"
    );
    if let Some(ct) = callout_type
        && !ct.is_empty()
    {
        let _ = write!(out, " data-callout-type=\"{ct}\"");
    }
    let _ = write!(
        out,
        ">\n<h2 class=\"sh{h1_class}{empty_class}\"{a11y_attrs}><span>{heading_text}</span>\n<div style=\"display:flex;align-items:center;gap:8px\"><span class=\"sbadge\" id=\"badge-s{section_count}\"></span><span class=\"stog\" id=\"tog-s{section_count}\">&#9660;</span></div></h2>\n<div class=\"sb\" id=\"body-s{section_count}\">\n"
    );
}

/// Renders section body and container closing tags directly into the output buffer.
#[inline]
pub fn render_section_close(out: &mut impl Write) {
    let _ = out.write_str("</div></section>\n\n");
}

/// Renders a subheading component (H3-H6) directly into the output buffer.
#[inline]
pub fn render_subheading(out: &mut impl Write, sub_html: &str) {
    let _ = writeln!(out, "<h3 class=\"subh\">{}</h3>", sub_html.trim());
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

    let _ = write!(
        out,
        "<div class=\"doc-item simple-item\" id=\"item_s{sec_num}_{item_count}\""
    );
    if indent_depth > 0 {
        let _ = write!(out, " style=\"--indent: {indent_depth};\"");
    }
    let _ = write!(
        out,
        ">\n  <span class=\"list-bullet\">{bullet}</span>\n  <span class=\"check-label\">{label_text}</span>\n  {COMMENT_ICON_SVG}\n</div>\n"
    );
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
    let _ = write!(
        out,
        "<div class=\"doc-item text-item\" id=\"txt_s{sec_num}_{txt_count}\""
    );
    if indent_depth > 0 {
        let _ = write!(out, " style=\"--indent: {indent_depth};\"");
    }
    let _ = write!(
        out,
        ">\n  <span class=\"text-content\">{content_html}</span>\n  {COMMENT_ICON_SVG}\n</div>\n"
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_render_section_header_and_close() {
        let mut buf = String::new();
        render_section_header(
            &mut buf,
            1,
            "Section Title",
            true,
            false,
            true,
            Some("note"),
        );
        assert!(buf.contains("<!-- S1 -->"));
        assert!(buf.contains("<section class=\"section\" id=\"s1\" data-has-checklist=\"true\" data-callout-type=\"note\">"));
        assert!(buf.contains("class=\"sh sh-h1\""));
        assert!(buf.contains("role=\"button\""));
        assert!(buf.contains("tabindex=\"0\""));
        assert!(buf.contains("aria-expanded=\"true\""));
        assert!(buf.contains("Section Title"));

        render_section_close(&mut buf);
        assert!(buf.contains("</div></section>\n\n"));
    }

    #[test]
    fn test_render_subheading() {
        let mut buf = String::new();
        render_subheading(&mut buf, "Subheading Text");
        assert_eq!(buf, "<h3 class=\"subh\">Subheading Text</h3>\n");
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
}
