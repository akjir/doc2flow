use doc2flow::converter::{convert_markdown_to_html, parse_frontmatter};

#[test]
fn test_frontmatter_and_blockquote_conversion() {
    let input = r#"---
title: "Test Guide"
customer: "Acme Corp"
---

## Section 1

> i Note text
"#;

    let (frontmatter, body) = parse_frontmatter(input);
    assert_eq!(frontmatter.title, "Test Guide");
    assert_eq!(frontmatter.customer, "Acme Corp");

    let html = convert_markdown_to_html(body).expect("conversion failed");
    assert!(html.contains("<div class=\"section\" id=\"s1\">"));
    assert!(html.contains("<div class=\"note\">&#x24D8; Note text</div>"));
}
