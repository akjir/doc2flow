//! AST serialization and JSON formatting for documents.

use std::fmt::Write;

use crate::core::document::{Document, DocumentElement};

/// Serializes a document model to a formatted JSON string.
///
/// # Examples
///
/// ```
/// use doc2flow::core::document::Document;
/// use doc2flow::core::document_json::document_to_json;
///
/// let doc = Document::new();
/// let json = document_to_json(&doc);
/// assert!(json.contains("\"parameters\":"));
/// ```
pub fn document_to_json(doc: &Document) -> String {
    let mut out = String::with_capacity(1024);
    out.push_str("{\n  \"parameters\": {\n");

    let mut sorted_keys: Vec<_> = doc.parameters.keys().collect();
    sorted_keys.sort();
    for (i, key) in sorted_keys.iter().enumerate() {
        let val = &doc.parameters[*key];
        out.push_str("    \"");
        escape_json_string(&mut out, key);
        out.push_str("\": \"");
        escape_json_string(&mut out, val);
        out.push('"');
        if i + 1 < sorted_keys.len() {
            out.push(',');
        }
        out.push('\n');
    }
    out.push_str("  },\n  \"header\": [\n");

    for (i, elem) in doc.header.iter().enumerate() {
        format_element(&mut out, elem, 2);
        if i + 1 < doc.header.len() {
            out.push(',');
        }
        out.push('\n');
    }

    out.push_str("  ],\n  \"body\": [\n");

    for (i, elem) in doc.body.iter().enumerate() {
        format_element(&mut out, elem, 2);
        if i + 1 < doc.body.len() {
            out.push(',');
        }
        out.push('\n');
    }

    out.push_str("  ]\n}");
    out
}

/// Escapes special characters in a string for JSON output.
fn escape_json_string(out: &mut String, s: &str) {
    for ch in s.chars() {
        match ch {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if c.is_control() => {
                let _ = write!(out, "\\u{:04x}", c as u32);
            }
            c => out.push(c),
        }
    }
}

/// Formats a single document element and its children with indentation.
fn format_element(out: &mut String, elem: &DocumentElement, indent_level: usize) {
    write_indent(out, indent_level);
    format_element_inline(out, elem, indent_level);
}

/// Formats a single document element starting with the opening brace.
fn format_element_inline(out: &mut String, elem: &DocumentElement, indent_level: usize) {
    out.push_str("{\n");
    match elem {
        DocumentElement::BlockDirective { children, name } => {
            write_indent(out, indent_level + 1);
            out.push_str("\"kind\": \"block_directive\",\n");
            write_indent(out, indent_level + 1);
            out.push_str("\"name\": \"");
            escape_json_string(out, name);
            out.push_str("\",\n");
            write_indent(out, indent_level + 1);
            out.push_str("\"children\": [");
            if children.is_empty() {
                out.push_str("]\n");
            } else {
                out.push('\n');
                for (j, child) in children.iter().enumerate() {
                    format_element(out, child, indent_level + 2);
                    if j + 1 < children.len() {
                        out.push(',');
                    }
                    out.push('\n');
                }
                write_indent(out, indent_level + 1);
                out.push_str("]\n");
            }
        }
        DocumentElement::BulletListItem { depth, content } => {
            write_indent(out, indent_level + 1);
            out.push_str("\"kind\": \"bullet_list_item\",\n");
            write_indent(out, indent_level + 1);
            let _ = write!(out, "\"depth\": {depth},\n");
            write_indent(out, indent_level + 1);
            out.push_str("\"content\": ");
            format_element_inline(out, content, indent_level + 1);
            out.push('\n');
        }
        DocumentElement::CheckBoxItem {
            depth,
            checked,
            content,
        } => {
            write_indent(out, indent_level + 1);
            out.push_str("\"kind\": \"check_box_item\",\n");
            write_indent(out, indent_level + 1);
            let _ = write!(out, "\"depth\": {depth},\n");
            write_indent(out, indent_level + 1);
            let _ = write!(out, "\"checked\": {checked},\n");
            write_indent(out, indent_level + 1);
            out.push_str("\"content\": ");
            format_element_inline(out, content, indent_level + 1);
            out.push('\n');
        }
        DocumentElement::CodeBlock { language, content } => {
            write_indent(out, indent_level + 1);
            out.push_str("\"kind\": \"code_block\",\n");
            write_indent(out, indent_level + 1);
            if let Some(lang) = language {
                out.push_str("\"language\": \"");
                escape_json_string(out, lang);
                out.push_str("\",\n");
            } else {
                out.push_str("\"language\": null,\n");
            }
            write_indent(out, indent_level + 1);
            out.push_str("\"content\": \"");
            escape_json_string(out, content);
            out.push_str("\"\n");
        }
        DocumentElement::Image { alt, url } => {
            write_indent(out, indent_level + 1);
            out.push_str("\"kind\": \"image\",\n");
            write_indent(out, indent_level + 1);
            out.push_str("\"alt\": \"");
            escape_json_string(out, alt);
            out.push_str("\",\n");
            write_indent(out, indent_level + 1);
            out.push_str("\"url\": \"");
            escape_json_string(out, url);
            out.push_str("\"\n");
        }
        DocumentElement::OrderedListItem {
            depth,
            position,
            content,
        } => {
            write_indent(out, indent_level + 1);
            out.push_str("\"kind\": \"ordered_list_item\",\n");
            write_indent(out, indent_level + 1);
            let _ = write!(out, "\"depth\": {depth},\n");
            write_indent(out, indent_level + 1);
            let _ = write!(out, "\"position\": {position},\n");
            write_indent(out, indent_level + 1);
            out.push_str("\"content\": ");
            format_element_inline(out, content, indent_level + 1);
            out.push('\n');
        }
        DocumentElement::Section {
            level,
            title,
            children,
        } => {
            write_indent(out, indent_level + 1);
            out.push_str("\"kind\": \"section\",\n");
            write_indent(out, indent_level + 1);
            let _ = write!(out, "\"level\": {level},\n");
            write_indent(out, indent_level + 1);
            out.push_str("\"title\": \"");
            escape_json_string(out, title);
            out.push_str("\",\n");
            write_indent(out, indent_level + 1);
            out.push_str("\"children\": [");
            if children.is_empty() {
                out.push_str("]\n");
            } else {
                out.push('\n');
                for (j, child) in children.iter().enumerate() {
                    format_element(out, child, indent_level + 2);
                    if j + 1 < children.len() {
                        out.push(',');
                    }
                    out.push('\n');
                }
                write_indent(out, indent_level + 1);
                out.push_str("]\n");
            }
        }
        DocumentElement::Shoutout { kind, content } => {
            write_indent(out, indent_level + 1);
            out.push_str("\"kind\": \"shoutout\",\n");
            write_indent(out, indent_level + 1);
            out.push_str("\"subkind\": \"");
            out.push_str(kind.as_str());
            out.push_str("\",\n");
            write_indent(out, indent_level + 1);
            out.push_str("\"content\": \"");
            escape_json_string(out, content);
            out.push_str("\"\n");
        }
        DocumentElement::Table { alignments, rows } => {
            write_indent(out, indent_level + 1);
            out.push_str("\"kind\": \"table\",\n");
            write_indent(out, indent_level + 1);
            out.push_str("\"alignments\": [");
            for (k, align) in alignments.iter().enumerate() {
                out.push('"');
                out.push_str(align.as_str());
                out.push('"');
                if k + 1 < alignments.len() {
                    out.push_str(", ");
                }
            }
            out.push_str("],\n");
            write_indent(out, indent_level + 1);
            out.push_str("\"rows\": [");
            if rows.is_empty() {
                out.push_str("]\n");
            } else {
                out.push('\n');
                for (r_idx, row) in rows.iter().enumerate() {
                    write_indent(out, indent_level + 1);
                    out.push_str("  [");
                    for (c_idx, cell) in row.iter().enumerate() {
                        out.push('"');
                        escape_json_string(out, cell);
                        out.push('"');
                        if c_idx + 1 < row.len() {
                            out.push_str(", ");
                        }
                    }
                    out.push(']');
                    if r_idx + 1 < rows.len() {
                        out.push(',');
                    }
                    out.push('\n');
                }
                write_indent(out, indent_level + 1);
                out.push_str("]\n");
            }
        }
        DocumentElement::Text(content) => {
            write_indent(out, indent_level + 1);
            out.push_str("\"kind\": \"text\",\n");
            write_indent(out, indent_level + 1);
            out.push_str("\"content\": \"");
            escape_json_string(out, content);
            out.push_str("\"\n");
        }
        DocumentElement::Unknown(content) => {
            write_indent(out, indent_level + 1);
            out.push_str("\"kind\": \"unknown\",\n");
            write_indent(out, indent_level + 1);
            out.push_str("\"content\": \"");
            escape_json_string(out, content);
            out.push_str("\"\n");
        }
    }

    write_indent(out, indent_level);
    out.push('}');
}

/// Writes indentation spaces directly to the buffer.
fn write_indent(out: &mut String, level: usize) {
    for _ in 0..level {
        out.push_str("  ");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::document::{ShoutoutElementKind, TableAlignment};

    #[test]
    fn test_document_to_json_empty() {
        let doc = Document::new();
        let json = document_to_json(&doc);
        assert_eq!(
            json,
            "{\n  \"parameters\": {\n  },\n  \"header\": [\n  ],\n  \"body\": [\n  ]\n}"
        );
    }

    #[test]
    fn test_document_to_json_with_elements() {
        let mut doc = Document::new();
        doc.insert_parameter("title", "Test Title");
        doc.insert_parameter("author", "Tester");

        let text_elem = DocumentElement::text("Hello \"world\"\nNew line");
        let section_elem = DocumentElement::section(
            1,
            "My Section",
            vec![DocumentElement::unknown("Child")],
        );

        doc.push_header(text_elem);
        doc.push_body(section_elem);

        let json = document_to_json(&doc);
        assert!(json.contains("\"title\": \"Test Title\""));
        assert!(json.contains("\"author\": \"Tester\""));
        assert!(json.contains("\"kind\": \"text\""));
        assert!(json.contains("\"content\": \"Hello \\\"world\\\"\\nNew line\""));
        assert!(json.contains("\"kind\": \"section\""));
        assert!(json.contains("\"level\": 1"));
        assert!(json.contains("\"title\": \"My Section\""));
        assert!(json.contains("\"kind\": \"unknown\""));
        assert!(json.contains("\"content\": \"Child\""));
    }

    #[test]
    fn test_document_to_json_with_shoutouts() {
        let mut doc = Document::new();
        doc.push_body(DocumentElement::shoutout(
            ShoutoutElementKind::Important,
            "Check this out",
        ));
        doc.push_body(DocumentElement::shoutout(
            ShoutoutElementKind::Caution,
            "Watch out",
        ));

        let json = document_to_json(&doc);
        assert!(json.contains("\"kind\": \"shoutout\""));
        assert!(json.contains("\"subkind\": \"important\""));
        assert!(json.contains("\"content\": \"Check this out\""));
        assert!(json.contains("\"subkind\": \"caution\""));
        assert!(json.contains("\"content\": \"Watch out\""));
    }

    #[test]
    fn test_document_to_json_with_bullet_list_items() {
        let mut doc = Document::new();
        doc.push_body(DocumentElement::bullet_list_item(
            0,
            DocumentElement::text("Root item"),
        ));
        doc.push_body(DocumentElement::bullet_list_item(
            1,
            DocumentElement::text("Child item"),
        ));

        let json = document_to_json(&doc);
        assert!(json.contains("\"kind\": \"bullet_list_item\""));
        assert!(json.contains("\"depth\": 0"));
        assert!(json.contains("\"content\": {"));
        assert!(json.contains("\"kind\": \"text\""));
        assert!(json.contains("\"content\": \"Root item\""));
        assert!(json.contains("\"depth\": 1"));
        assert!(json.contains("\"content\": \"Child item\""));
    }

    #[test]
    fn test_document_to_json_with_check_box_items() {
        let mut doc = Document::new();
        doc.push_body(DocumentElement::check_box_item(
            0,
            false,
            DocumentElement::text("Todo item"),
        ));
        doc.push_body(DocumentElement::check_box_item(
            2,
            true,
            DocumentElement::text("Done item"),
        ));

        let json = document_to_json(&doc);
        assert!(json.contains("\"kind\": \"check_box_item\""));
        assert!(json.contains("\"depth\": 0"));
        assert!(json.contains("\"checked\": false"));
        assert!(json.contains("\"content\": {"));
        assert!(json.contains("\"kind\": \"text\""));
        assert!(json.contains("\"content\": \"Todo item\""));
        assert!(json.contains("\"depth\": 2"));
        assert!(json.contains("\"checked\": true"));
        assert!(json.contains("\"content\": \"Done item\""));
    }

    #[test]
    fn test_document_to_json_with_code_blocks() {
        let mut doc = Document::new();
        doc.push_body(DocumentElement::code_block(
            Some("bash"),
            "echo \"Hello world\"",
        ));
        doc.push_body(DocumentElement::code_block(
            None::<String>,
            "plain text block",
        ));

        let json = document_to_json(&doc);
        assert!(json.contains("\"kind\": \"code_block\""));
        assert!(json.contains("\"language\": \"bash\""));
        assert!(json.contains("\"content\": \"echo \\\"Hello world\\\"\""));
        assert!(json.contains("\"language\": null"));
        assert!(json.contains("\"content\": \"plain text block\""));
    }

    #[test]
    fn test_document_to_json_with_images() {
        let mut doc = Document::new();
        doc.push_body(DocumentElement::image(
            "System Architecture",
            "assets/diagram.png",
        ));
        doc.push_body(DocumentElement::image(
            "Logo with \"quotes\"",
            "https://example.com/logo.svg",
        ));

        let json = document_to_json(&doc);
        assert!(json.contains("\"kind\": \"image\""));
        assert!(json.contains("\"alt\": \"System Architecture\""));
        assert!(json.contains("\"url\": \"assets/diagram.png\""));
        assert!(json.contains("\"alt\": \"Logo with \\\"quotes\\\"\""));
        assert!(json.contains("\"url\": \"https://example.com/logo.svg\""));
    }

    #[test]
    fn test_document_to_json_with_ordered_list_items() {
        let mut doc = Document::new();
        doc.push_body(DocumentElement::ordered_list_item(
            0,
            1,
            DocumentElement::text("First item"),
        ));
        doc.push_body(DocumentElement::ordered_list_item(
            2,
            3,
            DocumentElement::text("Nested item"),
        ));

        let json = document_to_json(&doc);
        assert!(json.contains("\"kind\": \"ordered_list_item\""));
        assert!(json.contains("\"depth\": 0"));
        assert!(json.contains("\"position\": 1"));
        assert!(json.contains("\"content\": {"));
        assert!(json.contains("\"kind\": \"text\""));
        assert!(json.contains("\"content\": \"First item\""));
        assert!(json.contains("\"depth\": 2"));
        assert!(json.contains("\"position\": 3"));
        assert!(json.contains("\"content\": \"Nested item\""));
    }

    #[test]
    fn test_document_to_json_with_table() {
        let mut doc = Document::new();
        doc.push_body(DocumentElement::table(
            vec![
                TableAlignment::Left,
                TableAlignment::Center,
                TableAlignment::Right,
            ],
            vec![
                vec!["Col 1".into(), "Col 2".into(), "Col 3".into()],
                vec!["Val 1".into(), "Val \"2\"".into(), "Val 3".into()],
            ],
        ));

        let json = document_to_json(&doc);
        assert!(json.contains("\"kind\": \"table\""));
        assert!(json.contains("\"alignments\": [\"left\", \"center\", \"right\"]"));
        assert!(json.contains("\"rows\": ["));
        assert!(json.contains("[\"Col 1\", \"Col 2\", \"Col 3\"]"));
        assert!(json.contains("[\"Val 1\", \"Val \\\"2\\\"\", \"Val 3\"]"));
    }

    #[test]
    fn test_document_to_json_with_block_directive() {
        let mut doc = Document::new();
        let table = DocumentElement::table(
            vec![],
            vec![vec!["TARGET_HOST".into(), "192.168.1.100".into()]],
        );
        let text1 = DocumentElement::text("First text");
        let block = DocumentElement::block_directive("variables", vec![table, text1]);
        doc.push_body(block);

        let json = document_to_json(&doc);
        assert!(json.contains("\"kind\": \"block_directive\""));
        assert!(json.contains("\"name\": \"variables\""));
        assert!(json.contains("\"children\": ["));
        assert!(json.contains("\"kind\": \"table\""));
        assert!(json.contains("\"kind\": \"text\""));
        assert!(json.contains("\"content\": \"First text\""));
    }

    #[test]
    fn test_escape_json_string_special_characters() {
        let mut out = String::new();
        escape_json_string(&mut out, "Tab\tNewline\nCarriage\rSlash\\Quote\"Control\x07");
        assert_eq!(out, "Tab\\tNewline\\nCarriage\\rSlash\\\\Quote\\\"Control\\u0007");
    }

    #[test]
    fn test_format_empty_containers() {
        let mut doc = Document::new();
        doc.push_body(DocumentElement::block_directive("empty_block", vec![]));
        doc.push_body(DocumentElement::section(1, "Empty Section", vec![]));
        doc.push_body(DocumentElement::table(vec![], vec![]));

        let json = document_to_json(&doc);
        assert!(json.contains("\"children\": []"));
        assert!(json.contains("\"rows\": []"));
    }
}
