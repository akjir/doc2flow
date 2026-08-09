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
}
