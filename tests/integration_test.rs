use doc2flow::converter::{convert_markdown_to_html, parse_frontmatter};

#[test]
fn test_callout_variants_conversion() {
    let input = r#"## Section 1

> Standard Note text

>? Tip text

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

1. Ordered step 1
2. Ordered step 2

## Section 2

- [ ] Checkbox task
- Simple item inside mixed section
"#;

    let (_frontmatter, body) = parse_frontmatter(input);
    let html = convert_markdown_to_html(body).expect("conversion failed");

    // Simple list items check
    assert!(html.contains("<div class=\"check-item simple-item\" id=\"item_s1_1\">"));
    assert!(html.contains("<span class=\"list-bullet\">&bull;</span>"));
    assert!(html.contains("<span class=\"check-label\">Simple item 1</span>"));

    // Ordered list items check
    assert!(html.contains("<span class=\"list-bullet\">1.</span>"));
    assert!(html.contains("<span class=\"check-label\">Ordered step 1</span>"));
    assert!(html.contains("<span class=\"list-bullet\">2.</span>"));
    assert!(html.contains("<span class=\"check-label\">Ordered step 2</span>"));

    // Checkbox item check
    assert!(html.contains("<input type=\"checkbox\" id=\"cb_s2_1\">"));
    assert!(html.contains("<label class=\"check-label\" for=\"cb_s2_1\">Checkbox task</label>"));
}

#[test]
fn test_frontmatter_language_parsing() {
    let input = r#"---
title: "Test"
company: "Test Corp"
language: "de"
---
## Section 1
"#;

    let (fm, _body) = parse_frontmatter(input);
    assert_eq!(fm.language, "de");

    let locale = doc2flow::i18n::Locale::from_lang_code(&fm.language);
    assert_eq!(locale.lang_code, "de");
    assert_eq!(locale.get("company"), "Firma");
    assert_eq!(
        locale.get("reset_all"),
        "Zurücksetzen"
    );
}

#[test]
fn test_german_locale_conversion() {
    let input = r#"## Section 1

> Standard Hinweis

>? Tipp Text

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
company: "Test Corp"
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
    assert!(final_html.contains("id=\"f_info_contact\""));
    assert!(final_html.contains("id=\"f_info_agent\""));
    assert!(final_html.contains("id=\"f_info_date\""));
    assert!(final_html.contains("table-layout: fixed"));
    assert!(
        final_html
            .contains(".info-table th:nth-child(4), .info-table td:nth-child(4) { width: 15%; }")
    );
    assert!(final_html.contains("id=\"finish-box\""));
    assert!(final_html.contains("id=\"finish-icon\""));
    assert!(final_html.contains("id=\"finish-title\""));
    assert!(final_html.contains("id=\"btn-pdf\""));
    assert!(final_html.contains("id=\"btn-save\""));
    assert!(final_html.contains("Stand sichern"));
    assert!(final_html.contains("<div class=\"logo-wrap\">"));
}

#[test]
fn test_showcase_en_fixture_conversion() {
    let md_content = std::fs::read_to_string("tests/showcase_en.md")
        .expect("Failed to read tests/showcase_en.md");
    let (fm, body) = doc2flow::converter::parse_frontmatter(&md_content);
    let locale = doc2flow::i18n::Locale::from_lang_code(&fm.language);
    let html_body = doc2flow::converter::convert_markdown_to_html_with_locale(body, &locale)
        .expect("conversion failed");
    let d2f_id = doc2flow::id::generate_d2f_id(&fm).expect("id gen failed");
    let rendered =
        doc2flow::template::render(&fm, &locale, &html_body, &d2f_id).expect("rendering failed");
    let html =
        doc2flow::image::embed_images_as_base64(&rendered, Some(std::path::Path::new("tests")));

    assert!(html.contains("Doc2Flow English Showcase"));
    assert!(html.contains("<div class=\"check-item text-item\" id=\"txt_s1_1\">"));
    assert!(html.contains("data:image/jpeg;base64,"));
    assert!(html.contains("<div class=\"note\" data-label=\"Note\">"));
    assert!(html.contains("<div class=\"note note-tip\" data-label=\"Tip\">"));
    assert!(html.contains("<input type=\"checkbox\" id=\"cb_s2_1\">"));
    assert!(!html.contains("Test comment: This comment must not appear"));
    assert!(html.contains("<div class=\"logo-wrap\">"));
}

#[test]
fn test_showcase_de_fixture_conversion() {
    let md_content = std::fs::read_to_string("tests/showcase_de.md")
        .expect("Failed to read tests/showcase_de.md");
    let (fm, body) = doc2flow::converter::parse_frontmatter(&md_content);
    let locale = doc2flow::i18n::Locale::from_lang_code(&fm.language);
    let html_body = doc2flow::converter::convert_markdown_to_html_with_locale(body, &locale)
        .expect("conversion failed");
    let d2f_id = doc2flow::id::generate_d2f_id(&fm).expect("id gen failed");
    let rendered =
        doc2flow::template::render(&fm, &locale, &html_body, &d2f_id).expect("rendering failed");
    let html =
        doc2flow::image::embed_images_as_base64(&rendered, Some(std::path::Path::new("tests")));

    assert!(html.contains("Doc2Flow Deutscher Showcase"));
    assert!(html.contains("<div class=\"check-item text-item\" id=\"txt_s1_1\">"));
    assert!(html.contains("data:image/jpeg;base64,"));
    assert!(html.contains("<div class=\"note\" data-label=\"Hinweis\">"));
    assert!(html.contains("<div class=\"note note-tip\" data-label=\"Tipp\">"));
    assert!(html.contains("<input type=\"checkbox\" id=\"cb_s2_1\">"));
    assert!(!html.contains("Test-Kommentar: Dieser Hinweis darf nicht"));
    assert!(html.contains("<div class=\"logo-wrap\">"));
}

#[test]
fn test_template_generator_conversion() {
    let template_md = doc2flow::template::generate_template_markdown();
    let (fm, body) = doc2flow::converter::parse_frontmatter(template_md);
    let locale = doc2flow::i18n::Locale::from_lang_code(&fm.language);
    let html_body = doc2flow::converter::convert_markdown_to_html_with_locale(body, &locale)
        .expect("conversion failed");
    let d2f_id = doc2flow::id::generate_d2f_id(&fm).expect("id gen failed");
    let html =
        doc2flow::template::render(&fm, &locale, &html_body, &d2f_id).expect("rendering failed");

    assert!(html.contains("Doc2Flow Standard Operating Procedure"));
    assert!(!html.contains("DOC2FLOW (D2F) - TEMPLATE & USAGE GUIDE"));
    assert!(html.contains("Section 1: Initial System Verification"));
    assert!(html.contains("Prerequisites Checklist"));
    assert!(html.contains("Configuration &amp; Service Deployment"));
    assert!(html.contains("<div class=\"logo-wrap\">"));
}

#[test]
fn test_frontmatter_company_validation_success() {
    let input = r#"---
title: "Valid Spec"
company: "ACME Corp"
date: "2026-07-25"
---
## Section 1
"#;

    let (fm, _body) = doc2flow::converter::parse_and_validate_frontmatter(input, Some("test.md"))
        .expect("validation failed for valid company");
    assert_eq!(fm.company, "ACME Corp");
}

#[test]
fn test_frontmatter_company_validation_missing_error_feedback() {
    let input = r#"---
title: "Missing Company Spec"
date: "2026-07-25"
---
## Section 1
"#;

    let err =
        doc2flow::converter::parse_and_validate_frontmatter(input, Some("invalid.md")).unwrap_err();
    let err_msg = err.to_string();

    assert!(err_msg.contains("error: missing required frontmatter field 'company'"));
    assert!(err_msg.contains("--> invalid.md:1:1"));
    assert!(err_msg.contains("1 | ---"));
    assert!(
        err_msg.contains("^^^ frontmatter block defined here is missing required field 'company'")
    );
    assert!(err_msg.contains("= help: add 'company: \"Company Name\"'"));
}

#[test]
fn test_frontmatter_company_validation_empty_error_feedback() {
    let input = r#"---
title: "Empty Company Spec"
company: ""
date: "2026-07-25"
---
## Section 1
"#;

    let err =
        doc2flow::converter::parse_and_validate_frontmatter(input, Some("empty.md")).unwrap_err();
    let err_msg = err.to_string();

    assert!(err_msg.contains("error: required frontmatter field 'company' cannot be empty"));
    assert!(err_msg.contains("--> empty.md:3:1"));
    assert!(err_msg.contains("3 | company: \"\""));
    assert!(err_msg.contains("^^^^^^^^^^^ 'company' field value cannot be empty"));
    assert!(err_msg.contains("= help: provide a valid company name"));
}
