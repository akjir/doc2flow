//! Experimental document parser.

use crate::exp::dev_helper::document_to_json;
use crate::exp::parsing::parse_d2f_markdown;
use crate::utils::error::{Doc2FlowError, Result};

/// Parses Markdown content using the experimental building pipeline.
///
/// # Errors
///
/// Returns an error if parsing fails.
pub fn parse(md_content: &str) -> Result<String> {
    let document =
        parse_d2f_markdown(md_content).map_err(|e| Doc2FlowError::Message(e.to_string()))?;
    let json = document_to_json(&document);
    Ok(json)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_returns_json_ast() {
        let result = parse("---\ntitle: \"Test Doc\"\n---\n# Test\n\nParagraph text").unwrap();
        assert!(result.contains("\"parameters\": {"));
        assert!(result.contains("\"title\": \"Test Doc\""));
        assert!(result.contains("\"header\": ["));
        assert!(result.contains("\"body\": ["));
        assert!(result.contains("\"kind\": \"unknown\""));
        assert!(result.contains("\"content\": \"# Test\""));
        assert!(result.contains("\"kind\": \"text\""));
        assert!(result.contains("\"content\": \"Paragraph text\""));
    }

    #[test]
    fn test_parse_ignores_comments() {
        let result = parse("<!-- Header comment -->\n---\ntitle: \"Test\"\n---\n# Test <!-- inline -->\n<!-- multiline\ncomment -->\nParagraph text").unwrap();
        assert!(!result.contains("Header comment"));
        assert!(!result.contains("multiline"));
        assert!(result.contains("\"title\": \"Test\""));
        assert!(result.contains("\"content\": \"# Test\""));
        assert!(result.contains("\"content\": \"Paragraph text\""));
    }

    #[test]
    fn test_parse_fails_without_frontmatter() {
        let result = parse("# No frontmatter");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_with_shoutouts() {
        let result = parse("---\ntitle: \"Shoutout Doc\"\n---\n>! Important note\n>? Helpful tip").unwrap();
        assert!(result.contains("\"kind\": \"shoutout\""));
        assert!(result.contains("\"subkind\": \"important\""));
        assert!(result.contains("\"content\": \"Important note\""));
        assert!(result.contains("\"subkind\": \"tip\""));
        assert!(result.contains("\"content\": \"Helpful tip\""));
    }

    #[test]
    fn test_parse_with_bullet_list() {
        let result = parse("---\ntitle: \"Bullet List Doc\"\n---\n- Root item\n  - Nested item").unwrap();
        assert!(result.contains("\"kind\": \"bullet_list_item\""));
        assert!(result.contains("\"depth\": 0"));
        assert!(result.contains("\"content\": \"Root item\""));
        assert!(result.contains("\"depth\": 1"));
        assert!(result.contains("\"content\": \"Nested item\""));
    }

    #[test]
    fn test_parse_with_check_box_item() {
        let result = parse("---\ntitle: \"Check Box Doc\"\n---\n- [ ] Pending item\n  - [x] Completed item").unwrap();
        assert!(result.contains("\"kind\": \"check_box_item\""));
        assert!(result.contains("\"depth\": 0"));
        assert!(result.contains("\"checked\": false"));
        assert!(result.contains("\"content\": \"Pending item\""));
        assert!(result.contains("\"depth\": 1"));
        assert!(result.contains("\"checked\": true"));
        assert!(result.contains("\"content\": \"Completed item\""));
    }

    #[test]
    fn test_parse_with_code_block() {
        let result = parse("---\ntitle: \"Code Block Doc\"\n---\n```bash\n\n\necho \"test\"\n\n\n```").unwrap();
        assert!(result.contains("\"kind\": \"code_block\""));
        assert!(result.contains("\"language\": \"bash\""));
        assert!(result.contains("\"content\": \"echo \\\"test\\\"\""));
    }

    #[test]
    fn test_parse_with_ordered_list() {
        let result = parse("---\ntitle: \"Ordered List Doc\"\n---\n1. First item\n  1. Subitem\n2. Second item").unwrap();
        assert!(result.contains("\"kind\": \"ordered_list_item\""));
        assert!(result.contains("\"depth\": 0"));
        assert!(result.contains("\"position\": 1"));
        assert!(result.contains("\"content\": \"First item\""));
        assert!(result.contains("\"depth\": 1"));
        assert!(result.contains("\"position\": 1"));
        assert!(result.contains("\"content\": \"Subitem\""));
        assert!(result.contains("\"position\": 2"));
        assert!(result.contains("\"content\": \"Second item\""));
    }

    #[test]
    fn test_parse_with_table() {
        let result = parse("---\ntitle: \"Table Test\"\n---\n| Header A | Header B |\n| :--- | ---: |\n| Cell 1 | Cell 2 |").unwrap();
        assert!(result.contains("\"kind\": \"table\""));
        assert!(result.contains("\"alignments\": [\"left\", \"right\"]"));
        assert!(result.contains("[\"Header A\", \"Header B\"]"));
        assert!(result.contains("[\"Cell 1\", \"Cell 2\"]"));
    }
}

