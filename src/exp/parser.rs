//! Experimental document parser.

use crate::utils::error::Result;

/// Parses Markdown content using the experimental building pipeline.
///
/// # Errors
///
/// Returns an error if parsing fails.
pub fn parse(md_content: &str) -> Result<String> {
    let _ = md_content;
    Ok(String::from("hello world"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_returns_hello_world() {
        let result = parse("# Test").unwrap();
        assert_eq!(result, "hello world");
    }
}
