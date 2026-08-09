//! Development helper for AST inspection and JSON formatting.

use crate::exp::document::{Document, DocumentElement, DocumentElementKind};
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
    match elem.kind {
        DocumentElementKind::Text => {
            let _ = write!(out, "{child_indent}\"kind\": \"text\",\n");
        }
        DocumentElementKind::Shoutout(shoutout) => {
            let _ = write!(out, "{child_indent}\"kind\": \"shoutout\",\n");
            let _ = write!(
                out,
                "{child_indent}\"subkind\": \"{}\",\n",
                shoutout.kind.as_str()
            );
        }
        DocumentElementKind::Unknown => {
            let _ = write!(out, "{child_indent}\"kind\": \"unknown\",\n");
        }
    }

    let _ = write!(out, "{child_indent}\"content\": \"");
    escape_json_string(out, &elem.content);
    let _ = write!(out, "\",\n");

    let _ = write!(out, "{child_indent}\"children\": [");
    if elem.children.is_empty() {
        out.push_str("]\n");
    } else {
        out.push('\n');
        for (j, child) in elem.children.iter().enumerate() {
            format_element(out, child, indent_level + 2);
            if j + 1 < elem.children.len() {
                out.push(',');
            }
            out.push('\n');
        }
        let _ = write!(out, "{child_indent}]\n");
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
        let mut text_elem =
            DocumentElement::new(DocumentElementKind::Text, "Hello \"world\"\nNew line");
        let child_elem = DocumentElement::new(DocumentElementKind::Unknown, "Child");
        text_elem.push_child(child_elem);
        doc.push_body(text_elem);

        let json = document_to_json(&doc);
        assert!(json.contains("\"parameters\": {"));
        assert!(json.contains("\"title\": \"Test\""));
        assert!(json.contains("\"header\": ["));
        assert!(json.contains("\"body\": ["));
        assert!(json.contains("\"kind\": \"text\""));
        assert!(json.contains("\"content\": \"Hello \\\"world\\\"\\nNew line\""));
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
}
