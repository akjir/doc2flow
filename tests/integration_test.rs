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

#[test]
fn test_code_block_conversion_with_and_without_language() {
    let input = r#"## Section 1

```ini
[HKEY_LOCAL_MACHINE\SOFTWARE\Test]
"Setting"=dword:00000001
```

```
plain text block
```
"#;

    let (_frontmatter, body) = parse_frontmatter(input);
    let html = convert_markdown_to_html(body).expect("conversion failed");

    // Code block with language tag and copy button
    assert!(html.contains("<span class=\"code-lang\">ini</span>"));
    assert!(html.contains("<button class=\"copy-btn\" onclick=\"copyCode(this)\""));
    assert!(html.contains("class=\"svg-icon iconCopy\""));
    assert!(html.contains("<pre class=\"code-block language-ini\"><code>[HKEY_LOCAL_MACHINE\\SOFTWARE\\Test]\n&quot;Setting&quot;=dword:00000001\n</code></pre>"));

    // Code block without language tag (still has copy button)
    assert!(html.contains("<pre class=\"code-block\"><code>plain text block\n</code></pre>"));
    assert!(!html.contains("<span class=\"code-lang\">plain"));
}
