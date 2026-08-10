//! Development helper for AST inspection and JSON formatting.

use crate::exp::document::Document;
use crate::exp::document::DocumentElement;
use std::fmt::Write;

/// Serializes a document model to a formatted JSON string for development inspection.
pub fn document_to_json(doc: &Document) -> String {
    let mut out = String::with_capacity(512);
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

/// Formats a single document element and its children with indentation.
fn format_element(out: &mut String, elem: &DocumentElement, indent_level: usize) {
    let indent = "  ".repeat(indent_level);
    let child_indent = "  ".repeat(indent_level + 1);

    let _ = write!(out, "{indent}{{\n");
    match elem {
        DocumentElement::BlockDirective { children, name } => {
            let _ = write!(out, "{child_indent}\"kind\": \"block_directive\",\n");
            let _ = write!(out, "{child_indent}\"name\": \"");
            escape_json_string(out, name);
            let _ = write!(out, "\",\n");
            let _ = write!(out, "{child_indent}\"children\": [");
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
                let _ = write!(out, "{child_indent}]\n");
            }
        }
        DocumentElement::BulletListItem { depth, content } => {
            let _ = write!(out, "{child_indent}\"kind\": \"bullet_list_item\",\n");
            let _ = write!(out, "{child_indent}\"depth\": {depth},\n");
            let _ = write!(out, "{child_indent}\"content\": \"");
            escape_json_string(out, content);
            let _ = write!(out, "\"\n");
        }
        DocumentElement::CheckBoxItem {
            depth,
            checked,
            content,
        } => {
            let _ = write!(out, "{child_indent}\"kind\": \"check_box_item\",\n");
            let _ = write!(out, "{child_indent}\"depth\": {depth},\n");
            let _ = write!(out, "{child_indent}\"checked\": {checked},\n");
            let _ = write!(out, "{child_indent}\"content\": \"");
            escape_json_string(out, content);
            let _ = write!(out, "\"\n");
        }
        DocumentElement::CodeBlock { language, content } => {
            let _ = write!(out, "{child_indent}\"kind\": \"code_block\",\n");
            if let Some(lang) = language {
                let _ = write!(out, "{child_indent}\"language\": \"");
                escape_json_string(out, lang);
                let _ = write!(out, "\",\n");
            } else {
                let _ = write!(out, "{child_indent}\"language\": null,\n");
            }
            let _ = write!(out, "{child_indent}\"content\": \"");
            escape_json_string(out, content);
            let _ = write!(out, "\"\n");
        }
        DocumentElement::OrderedListItem {
            depth,
            position,
            content,
        } => {
            let _ = write!(out, "{child_indent}\"kind\": \"ordered_list_item\",\n");
            let _ = write!(out, "{child_indent}\"depth\": {depth},\n");
            let _ = write!(out, "{child_indent}\"position\": {position},\n");
            let _ = write!(out, "{child_indent}\"content\": \"");
            escape_json_string(out, content);
            let _ = write!(out, "\"\n");
        }
        DocumentElement::Section { title, children } => {
            let _ = write!(out, "{child_indent}\"kind\": \"section\",\n");
            let _ = write!(out, "{child_indent}\"title\": \"");
            escape_json_string(out, title);
            let _ = write!(out, "\",\n");
            let _ = write!(out, "{child_indent}\"children\": [");
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
                let _ = write!(out, "{child_indent}]\n");
            }
        }
        DocumentElement::Shoutout { kind, content } => {
            let _ = write!(out, "{child_indent}\"kind\": \"shoutout\",\n");
            let _ = write!(
                out,
                "{child_indent}\"subkind\": \"{}\",\n",
                kind.as_str()
            );
            let _ = write!(out, "{child_indent}\"content\": \"");
            escape_json_string(out, content);
            let _ = write!(out, "\"\n");
        }
        DocumentElement::Table { alignments, rows } => {
            let _ = write!(out, "{child_indent}\"kind\": \"table\",\n");
            let _ = write!(out, "{child_indent}\"alignments\": [");
            for (k, align) in alignments.iter().enumerate() {
                let _ = write!(out, "\"{}\"", align.as_str());
                if k + 1 < alignments.len() {
                    out.push_str(", ");
                }
            }
            out.push_str("],\n");
            let _ = write!(out, "{child_indent}\"rows\": [");
            if rows.is_empty() {
                out.push_str("]\n");
            } else {
                out.push('\n');
                for (r_idx, row) in rows.iter().enumerate() {
                    let _ = write!(out, "{child_indent}  [");
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
                let _ = write!(out, "{child_indent}]\n");
            }
        }
        DocumentElement::Text(content) => {
            let _ = write!(out, "{child_indent}\"kind\": \"text\",\n");
            let _ = write!(out, "{child_indent}\"content\": \"");
            escape_json_string(out, content);
            let _ = write!(out, "\"\n");
        }
        DocumentElement::Unknown(content) => {
            let _ = write!(out, "{child_indent}\"kind\": \"unknown\",\n");
            let _ = write!(out, "{child_indent}\"content\": \"");
            escape_json_string(out, content);
            let _ = write!(out, "\"\n");
        }
    }

    let _ = write!(out, "{indent}}}");
}

/// Escapes special JSON characters into the target output buffer.
fn escape_json_string(out: &mut String, input: &str) {
    for ch in input.chars() {
        match ch {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            '\x08' => out.push_str("\\b"),
            '\x0C' => out.push_str("\\f"),
            c if (c as u32) < 0x20 => {
                let _ = write!(out, "\\u{:04x}", c as u32);
            }
            c => out.push(c),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
        doc.insert_parameter("title", "Test");
        let text_elem = DocumentElement::text("Hello \"world\"\nNew line");
        let section_elem = DocumentElement::section(
            "Section Title",
            vec![DocumentElement::unknown("Child")],
        );
        doc.push_body(text_elem);
        doc.push_body(section_elem);

        let json = document_to_json(&doc);
        assert!(json.contains("\"parameters\": {"));
        assert!(json.contains("\"title\": \"Test\""));
        assert!(json.contains("\"header\": ["));
        assert!(json.contains("\"body\": ["));
        assert!(json.contains("\"kind\": \"text\""));
        assert!(json.contains("\"content\": \"Hello \\\"world\\\"\\nNew line\""));
        assert!(json.contains("\"kind\": \"section\""));
        assert!(json.contains("\"title\": \"Section Title\""));
        assert!(json.contains("\"children\": ["));
        assert!(json.contains("\"kind\": \"unknown\""));
        assert!(json.contains("\"content\": \"Child\""));
    }

    #[test]
    fn test_document_to_json_with_shoutouts() {
        use crate::exp::document::ShoutoutElementKind;

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
        doc.push_body(DocumentElement::bullet_list_item(0, "Root item"));
        doc.push_body(DocumentElement::bullet_list_item(1, "Child item"));

        let json = document_to_json(&doc);
        assert!(json.contains("\"kind\": \"bullet_list_item\""));
        assert!(json.contains("\"depth\": 0"));
        assert!(json.contains("\"content\": \"Root item\""));
        assert!(json.contains("\"depth\": 1"));
        assert!(json.contains("\"content\": \"Child item\""));
    }

    #[test]
    fn test_document_to_json_with_check_box_items() {
        let mut doc = Document::new();
        doc.push_body(DocumentElement::check_box_item(0, false, "Todo item"));
        doc.push_body(DocumentElement::check_box_item(2, true, "Done item"));

        let json = document_to_json(&doc);
        assert!(json.contains("\"kind\": \"check_box_item\""));
        assert!(json.contains("\"depth\": 0"));
        assert!(json.contains("\"checked\": false"));
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
    fn test_document_to_json_with_ordered_list_items() {
        let mut doc = Document::new();
        doc.push_body(DocumentElement::ordered_list_item(0, 1, "First item"));
        doc.push_body(DocumentElement::ordered_list_item(2, 3, "Nested item"));

        let json = document_to_json(&doc);
        assert!(json.contains("\"kind\": \"ordered_list_item\""));
        assert!(json.contains("\"depth\": 0"));
        assert!(json.contains("\"position\": 1"));
        assert!(json.contains("\"content\": \"First item\""));
        assert!(json.contains("\"depth\": 2"));
        assert!(json.contains("\"position\": 3"));
        assert!(json.contains("\"content\": \"Nested item\""));
    }

    #[test]
    fn test_document_to_json_with_table() {
        use crate::exp::document::TableAlignment;

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
}


