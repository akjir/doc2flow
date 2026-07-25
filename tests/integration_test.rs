use doc2flow::converter::{convert_markdown_to_html, parse_frontmatter};

#[test]
fn test_callout_variants_conversion() {
    let input = r#"## Section 1

> Standard Note text

>+ Tip text

>! Important text

>!! Warning text

>!!! Caution text
"#;

    let html = convert_markdown_to_html(input).expect("conversion failed");
    assert!(html.contains("<div class=\"note\" data-label=\"Note\">Standard Note text</div>"));
    assert!(html.contains("<div class=\"note note-tip\" data-label=\"Tip\">Tip text</div>"));
    assert!(html.contains(
        "<div class=\"note note-important\" data-label=\"Important\">Important text</div>"
    ));
    assert!(
        html.contains("<div class=\"note note-warning\" data-label=\"Warning\">Warning text</div>")
    );
    assert!(
        html.contains("<div class=\"note note-caution\" data-label=\"Caution\">Caution text</div>")
    );
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

#[test]
fn test_simple_and_mixed_list_items_conversion() {
    let input = r#"## Section 1

- Simple item 1
- Simple item 2

## Section 2

- [ ] Checkbox task
- Simple item inside mixed section
"#;

    let (_frontmatter, body) = parse_frontmatter(input);
    let html = convert_markdown_to_html(body).expect("conversion failed");

    // Simple list items check
    assert!(html.contains("<div class=\"check-item simple-item\">"));
    assert!(html.contains("<span class=\"list-bullet\">&bull;</span>"));
    assert!(html.contains("<span class=\"check-label\">Simple item 1</span>"));

    // Checkbox item check
    assert!(html.contains("<input type=\"checkbox\" id=\"cb_s2_1\">"));
    assert!(html.contains("<label class=\"check-label\" for=\"cb_s2_1\">Checkbox task</label>"));
}

#[test]
fn test_frontmatter_language_parsing() {
    let input = r#"---
title: "Test"
language: "de"
---
## Section 1
"#;

    let (fm, _body) = parse_frontmatter(input);
    assert_eq!(fm.language, "de");

    let locale = doc2flow::i18n::Locale::from_lang_code(&fm.language);
    assert_eq!(locale.lang_code, "de");
    assert_eq!(locale.get("customer"), "Kunde");
    assert_eq!(
        locale.get("reset_all"),
        "↺ Alle Kontrollkästchen zurücksetzen"
    );
}

#[test]
fn test_german_locale_conversion() {
    let input = r#"## Section 1

> Standard Hinweis

>+ Tipp Text

>! Wichtig Text

>!! Warnung Text

>!!! Achtung Text

```
test code
```
"#;

    let locale = doc2flow::i18n::Locale::from_lang_code("de");
    let html = doc2flow::converter::convert_markdown_to_html_with_locale(input, &locale)
        .expect("conversion failed");

    assert!(html.contains("<div class=\"note\" data-label=\"Hinweis\">Standard Hinweis</div>"));
    assert!(html.contains("<div class=\"note note-tip\" data-label=\"Tipp\">Tipp Text</div>"));
    assert!(
        html.contains(
            "<div class=\"note note-important\" data-label=\"Wichtig\">Wichtig Text</div>"
        )
    );
    assert!(
        html.contains("<div class=\"note note-warning\" data-label=\"Warnung\">Warnung Text</div>")
    );
    assert!(
        html.contains("<div class=\"note note-caution\" data-label=\"Achtung\">Achtung Text</div>")
    );
    assert!(html.contains("title=\"Code kopieren\""));
    assert!(html.contains("aria-label=\"Code kopieren\""));
}

#[test]
fn test_end_to_end_template_rendering() {
    let input = r#"---
title: "End-to-End Test"
customer: "Test Corp"
language: "de"
---
## Test Section

- [ ] Task 1
"#;

    let (fm, body) = doc2flow::converter::parse_frontmatter(input);
    let locale = doc2flow::i18n::Locale::from_lang_code(&fm.language);
    let html_body = doc2flow::converter::convert_markdown_to_html_with_locale(body, &locale)
        .expect("body conversion failed");

    let final_html = doc2flow::template::render(&fm, &locale, &html_body, "doc_test_123")
        .expect("template rendering failed");

    assert!(final_html.contains("<!DOCTYPE html>"));
    assert!(final_html.contains("<html lang=\"de\">"));
    assert!(final_html.contains("End-to-End Test"));
    assert!(final_html.contains("Test Corp"));
    assert!(final_html.contains("doc_test_123"));
    assert!(final_html.contains("<input type=\"checkbox\" id=\"cb_s1_1\">"));
}
