//! Module for generating the default starter Markdown template for Doc2Flow documents.

/// Returns a pre-populated Markdown template string containing full frontmatter,
/// usage instructions in HTML comments, and a showcase "Hello World" document structure.
///
/// # Examples
///
/// ```
/// use doc2flow::template_generator::generate_template_markdown;
///
/// let template = generate_template_markdown();
/// assert!(template.contains("title:"));
/// assert!(template.contains("## Section 1: Initial System Verification"));
/// ```
pub fn generate_template_markdown() -> &'static str {
    include_str!("../templates/template.md")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_template_markdown_contains_required_sections() {
        let content = generate_template_markdown();
        assert!(content.contains("title:"));
        assert!(content.contains("subtitle:"));
        assert!(content.contains("customer:"));
        assert!(content.contains("employee:"));
        assert!(content.contains("technician:"));
        assert!(content.contains("date:"));
        assert!(content.contains("version:"));
        assert!(content.contains("language:"));
        assert!(content.contains("## Section 1: Initial System Verification"));
        assert!(content.contains("### Prerequisites Checklist"));
        assert!(content.contains("<!--"));
        assert!(content.contains("-->"));
        assert!(content.contains("> Note:"));
        assert!(content.contains(">? Tip:"));
        assert!(content.contains(">! Important:"));
        assert!(content.contains(">!! Warning:"));
        assert!(content.contains(">!!! Caution:"));
    }
}
