//! Document parser module.

use crate::core::document::Document;
use crate::core::error::Result;
use crate::core::parse::markdown::parse_d2f_markdown;

/// Parses Markdown content into a structured [`Document`] model.
///
/// # Examples
///
/// ```
/// use doc2flow::core::parse::parser::parse;
///
/// let doc = parse("---\ntitle: Document\n---\n# Hello").unwrap();
/// assert_eq!(doc.parameters.get("title").map(String::as_str), Some("Document"));
/// ```
///
/// # Errors
///
/// Returns [`Error`](crate::core::error::Error) if frontmatter or document syntax is invalid.
pub fn parse(md_content: &str) -> Result<Document> {
    parse_d2f_markdown(md_content)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parser_parse_basic() {
        let input = "---\ntitle: Sample Doc\n---\n# Heading\nContent";
        let doc = parse(input).expect("parse failed");
        assert_eq!(doc.parameters.get("title").map(String::as_str), Some("Sample Doc"));
    }
}
