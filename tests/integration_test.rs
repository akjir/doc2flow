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

    let (html, _features) = convert_markdown_to_html(input).expect("conversion failed");
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
    let (html, _features) = convert_markdown_to_html(body).expect("conversion failed");

    // Code block with language tag and copy button
    assert!(html.contains("<span class=\"code-lang\">ini</span>"));
    assert!(html.contains("<button class=\"copy-btn\" onclick=\"window.d2f_code.copy(this)\""));
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
    let (html, _features) = convert_markdown_to_html(body).expect("conversion failed");

    // Simple list items check
    assert!(html.contains("<div class=\"doc-item simple-item\" id=\"item_s1_1\">"));
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
    assert_eq!(fm.language.as_deref(), Some("de"));

    let locale = doc2flow::language::Locale::from_lang_code(fm.language.as_deref().unwrap_or("en"));
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

    let locale = doc2flow::language::Locale::from_lang_code("de");
    let (html, _features) = doc2flow::converter::convert_markdown_to_html_with_locale(input, &locale)
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
    let locale = doc2flow::language::Locale::from_lang_code(fm.language.as_deref().unwrap_or("en"));
    let (html_body, features) = doc2flow::converter::convert_markdown_to_html_with_locale(body, &locale)
        .expect("body conversion failed");

    let final_html = doc2flow::template::render(&fm, &locale, &html_body, "doc_test_123", None, &features)
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
    let md_content = doc2flow::io::read_file_to_string(std::path::Path::new("examples/showcase_en.md"))
        .expect("Failed to read examples/showcase_en.md");
    let (fm, body) = doc2flow::converter::parse_frontmatter(&md_content);
    let locale = doc2flow::language::Locale::from_lang_code(fm.language.as_deref().unwrap_or("en"));
    let (html_body, features) = doc2flow::converter::convert_markdown_to_html_with_options(
        body,
        &locale,
        fm.number_sections,
    )
    .expect("conversion failed");
    let d2f_id = doc2flow::id::generate_d2f_id(&fm).expect("id gen failed");
    let rendered =
        doc2flow::template::render(&fm, &locale, &html_body, &d2f_id, None, &features).expect("rendering failed");
    let html = doc2flow::image::embed_images_as_base64(
        &rendered,
        Some(std::path::Path::new("examples")),
    )
    .expect("image embedding failed");

    assert!(html.contains("Doc2Flow English Showcase"));
    assert!(html.contains("<h2 class=\"sh sh-h1\" role=\"button\" tabindex=\"0\" aria-expanded=\"true\"><span>1. Part 1: System Setup &amp; Preparation</span>"));
    assert!(html.contains("no-toggle"));
    assert!(html.contains("<div class=\"doc-item text-item\" id=\"txt_s1_1\">"));
    assert!(html.contains("<input type=\"checkbox\" id=\"cb_s1_1\" checked>"));
    assert!(html.contains("data:image/jpeg;base64,"));
    assert!(html.contains("<div class=\"note\" data-label=\"Note\">"));
    assert!(html.contains("<div class=\"note note-tip\" data-label=\"Tip\">"));
    assert!(html.contains("<input type=\"checkbox\" id=\"cb_s3_2\">"));
    assert!(!html.contains("Test comment: This comment must not appear"));
    assert!(html.contains("<div class=\"logo-wrap\">"));
}

#[test]
fn test_showcase_de_fixture_conversion() {
    let md_content = doc2flow::io::read_file_to_string(std::path::Path::new("examples/showcase_de.md"))
        .expect("Failed to read examples/showcase_de.md");
    let (fm, body) = doc2flow::converter::parse_frontmatter(&md_content);
    let locale = doc2flow::language::Locale::from_lang_code(fm.language.as_deref().unwrap_or("en"));
    let (html_body, features) = doc2flow::converter::convert_markdown_to_html_with_options(
        body,
        &locale,
        fm.number_sections,
    )
    .expect("conversion failed");
    let d2f_id = doc2flow::id::generate_d2f_id(&fm).expect("id gen failed");
    let rendered =
        doc2flow::template::render(&fm, &locale, &html_body, &d2f_id, None, &features).expect("rendering failed");
    let html = doc2flow::image::embed_images_as_base64(
        &rendered,
        Some(std::path::Path::new("examples")),
    )
    .expect("image embedding failed");

    assert!(html.contains("Doc2Flow Deutscher Showcase"));
    assert!(html.contains("<h2 class=\"sh sh-h1\" role=\"button\" tabindex=\"0\" aria-expanded=\"true\"><span>1. Teil 1: Systemeinrichtung &amp; Vorbereitung</span>"));
    assert!(html.contains("no-toggle"));
    assert!(html.contains("<div class=\"doc-item text-item\" id=\"txt_s1_1\">"));
    assert!(html.contains("<input type=\"checkbox\" id=\"cb_s1_1\" checked>"));
    assert!(html.contains("data:image/jpeg;base64,"));
    assert!(html.contains("<div class=\"note\" data-label=\"Hinweis\">"));
    assert!(html.contains("<div class=\"note note-tip\" data-label=\"Tipp\">"));
    assert!(html.contains("<input type=\"checkbox\" id=\"cb_s3_2\">"));
    assert!(!html.contains("Test-Kommentar: Dieser Hinweis darf nicht"));
    assert!(html.contains("<div class=\"logo-wrap\">"));
}

#[test]
fn test_template_generator_conversion() {
    let template_md = doc2flow::template::generate_template_markdown();
    let (fm, body) = doc2flow::converter::parse_frontmatter(&template_md);
    let locale = doc2flow::language::Locale::from_lang_code(fm.language.as_deref().unwrap_or("en"));
    let (html_body, features) = doc2flow::converter::convert_markdown_to_html_with_options(
        body,
        &locale,
        fm.number_sections,
    )
    .expect("conversion failed");
    let d2f_id = doc2flow::id::generate_d2f_id(&fm).expect("id gen failed");
    let html =
        doc2flow::template::render(&fm, &locale, &html_body, &d2f_id, None, &features).expect("rendering failed");

    assert!(html.contains("Doc2Flow Standard Operating Procedure"));
    assert!(!html.contains("DOC2FLOW (D2F) - TEMPLATE & USAGE GUIDE"));
    assert!(html.contains("1.1 Section 1: Initial System Verification"));
    assert!(html.contains("Prerequisites Checklist"));
    assert!(html.contains("2.1 Section 2: Configuration &amp; Service Deployment"));
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

#[test]
fn test_level_1_heading_integration() {
    let input = r#"---
title: "H1 Test"
company: "Test Corp"
date: "2026-07-26"
---
# Main Section
- [ ] Task 1
- [x] Task 2

## Sub Section
- [ ] Task 3
"#;

    let (fm, body) = doc2flow::converter::parse_and_validate_frontmatter(input, Some("test_h1.md")).unwrap();
    let locale = doc2flow::language::Locale::from_lang_code(fm.language.as_deref().unwrap_or("en"));
    let (html_body, features) = doc2flow::converter::convert_markdown_to_html_with_locale(body, &locale).unwrap();
    let d2f_id = doc2flow::id::generate_d2f_id(&fm).unwrap();
    let rendered = doc2flow::template::render(&fm, &locale, &html_body, &d2f_id, None, &features).unwrap();

    assert!(rendered.contains(r#"<h2 class="sh sh-h1" role="button" tabindex="0" aria-expanded="true"><span>Main Section</span>"#));
    assert!(rendered.contains(r#"badge-s1"#));
    assert!(rendered.contains(r#"<h2 class="sh" role="button" tabindex="0" aria-expanded="true"><span>Sub Section</span>"#));
    assert!(rendered.contains(r#"badge-s2"#));
    assert!(!html_body.contains("onclick="));
}

#[test]
fn test_auto_scale_integration_test() {
    let dir = std::env::temp_dir().join("d2f_integration_auto_scale");
    let _ = doc2flow::io::create_dir_all(&dir);
    let img_path = dir.join("heavy.png");
    let img_buf = image::RgbImage::new(1200, 800);
    img_buf
        .save_with_format(&img_path, image::ImageFormat::Png)
        .unwrap();

    let file_size = doc2flow::io::get_file_size(&img_path).unwrap();
    if file_size <= doc2flow::image::MAX_IMAGE_SIZE_BYTES {
        let mut existing = doc2flow::io::read_file_bytes(&img_path).unwrap();
        existing.resize((doc2flow::image::MAX_IMAGE_SIZE_BYTES + 50 * 1024) as usize, 0);
        doc2flow::io::write_file(&img_path, &existing).unwrap();
    }

    let input = r#"---
title: "Auto Scale Spec"
company: "Test Corp"
date: "2026-07-26"
---
## System Overview

![Heavy Diagram](heavy.png)
"#;

    let file_name = "spec_scale.md";
    let (fm, body) =
        doc2flow::converter::parse_and_validate_frontmatter(input, Some(file_name)).unwrap();
    let locale = doc2flow::language::Locale::from_lang_code(fm.language.as_deref().unwrap_or("en"));
    let (html_body, features) =
        doc2flow::converter::convert_markdown_to_html_with_locale(body, &locale).unwrap();
    let d2f_id = doc2flow::id::generate_d2f_id(&fm).unwrap();
    let rendered = doc2flow::template::render(&fm, &locale, &html_body, &d2f_id, None, &features).unwrap();

    let html = doc2flow::image::embed_images_as_base64_with_source(
        &rendered,
        Some(input),
        Some(file_name),
        Some(&dir),
        true, // auto_scale
    )
    .expect("scaling should succeed");

    let _ = doc2flow::io::remove_dir_all(&dir);

    assert!(html.contains("data:image/webp;base64,"));
}

#[test]
fn test_full_pipeline_multi_language_and_callouts() {
    let input = r#"---
title: "Pipeline Callouts & Multilang"
company: "Global Tech"
language: "de"
version: "2.1.0"
date: "2026-07-26"
---
# Main Section

> Standard Hinweis

>? Tipp Text

>! Wichtig Text

>!! Warnung Text

>!!! Achtung Text

- [ ] Task 1
- [x] Task 2 Completed
"#;

    let (fm, body) = doc2flow::converter::parse_and_validate_frontmatter(input, Some("pipeline.md")).unwrap();
    let locale = doc2flow::language::Locale::from_lang_code(fm.language.as_deref().unwrap_or("en"));
    let (html_body, features) = doc2flow::converter::convert_markdown_to_html_with_locale(body, &locale).unwrap();
    let d2f_id = doc2flow::id::generate_d2f_id(&fm).unwrap();
    let html = doc2flow::template::render(&fm, &locale, &html_body, &d2f_id, None, &features).unwrap();

    assert!(html.contains("<html lang=\"de\">"));
    assert!(html.contains("data-label=\"Hinweis\""));
    assert!(html.contains("data-label=\"Tipp\""));
    assert!(html.contains("data-label=\"Wichtig\""));
    assert!(html.contains("data-label=\"Warnung\""));
    assert!(html.contains("data-label=\"Achtung\""));
    assert!(html.contains("id=\"wrap-cb_s1_1\""));
    assert!(html.contains("id=\"wrap-cb_s1_2\""));
}

#[test]
fn test_non_image_resource_link_wrapper_integration() {
    let input = r#"---
title: "PDF Resource Spec"
company: "Docs Inc"
date: "2026-07-26"
---
## Attachments

![Specification PDF](files/spec.pdf)
"#;

    let (fm, body) = doc2flow::converter::parse_and_validate_frontmatter(input, Some("pdf_spec.md")).unwrap();
    let locale = doc2flow::language::Locale::from_lang_code(fm.language.as_deref().unwrap_or("en"));
    let (html_body, features) = doc2flow::converter::convert_markdown_to_html_with_locale(body, &locale).unwrap();
    let d2f_id = doc2flow::id::generate_d2f_id(&fm).unwrap();
    let rendered = doc2flow::template::render(&fm, &locale, &html_body, &d2f_id, None, &features).unwrap();
    let html = doc2flow::image::embed_images_as_base64(&rendered, None).unwrap();

    assert!(html.contains("<div class=\"doc-item text-item\">"));
    assert!(html.contains("<a href=\"files/spec.pdf\" target=\"_blank\" rel=\"noopener noreferrer\">Specification PDF</a>"));
    assert!(!html.contains("src=\"files/spec.pdf\""));
}

#[test]
fn test_cli_parse_args_error_handling() {
    use doc2flow::utils::parse_args;

    // Unknown option
    let err = parse_args(&["d2f", "--unknown-flag"]).unwrap_err();
    assert!(err.contains("Unrecognized option '--unknown-flag'"));

    // Missing value for -o
    let err_o = parse_args(&["d2f", "-o"]).unwrap_err();
    assert!(err_o.contains("Option '--output' requires a path value"));

    // Empty value for --output=
    let err_empty = parse_args(&["d2f", "--output="]).unwrap_err();
    assert!(err_empty.contains("Option '--output' requires a non-empty path value"));

    // Multiple positional arguments
    let err_pos = parse_args(&["d2f", "doc1.md", "doc2.md"]).unwrap_err();
    assert!(err_pos.contains("Unexpected positional argument 'doc2.md'"));
}

#[test]
fn test_cli_version_output_formatting() {
    use doc2flow::utils::parse_args;

    let args = parse_args(&["d2f", "--version"]).unwrap();
    assert!(args.show_version);
    let full_version = env!("D2F_FULL_VERSION");
    assert!(full_version.starts_with('v'), "Full version must start with 'v': {}", full_version);
    assert!(full_version.contains('+'), "Full version must contain build metadata '+': {}", full_version);
    assert!(full_version.contains(env!("CARGO_PKG_VERSION")));
}

#[test]
fn test_unknown_locale_fallback_to_english() {
    let input = r#"---
title: "Fallback Spec"
company: "Global Inc"
date: "2026-07-26"
language: "fr"
---
## Section 1
- [ ] Task 1
"#;

    let (fm, body) = doc2flow::converter::parse_and_validate_frontmatter(input, Some("fallback.md")).unwrap();
    let locale = doc2flow::language::Locale::from_lang_code(fm.language.as_deref().unwrap_or("en"));
    assert_eq!(locale.lang_code, "en"); // Fallback to English
    let (html_body, features) = doc2flow::converter::convert_markdown_to_html_with_locale(body, &locale).unwrap();
    let d2f_id = doc2flow::id::generate_d2f_id(&fm).unwrap();
    let html = doc2flow::template::render(&fm, &locale, &html_body, &d2f_id, None, &features).unwrap();

    assert!(html.contains("<html lang=\"en\">"));
    assert!(html.contains("Save State"));
}

#[test]
fn test_non_existent_input_file_handling() {
    let non_existent_path = std::path::PathBuf::from("tests/non_existent_file_xyz123.md");
    let err = doc2flow::io::read_file_to_string(&non_existent_path).unwrap_err();

    match err {
        doc2flow::error::Doc2FlowError::Io { path, .. } => {
            assert_eq!(path, Some(non_existent_path));
        }
        _ => panic!("Expected Doc2FlowError::Io variant"),
    }
}

#[test]
fn test_custom_logo_frontmatter_and_cli_precedence_integration() {
    use std::path::Path;

    let temp_dir = std::env::temp_dir().join("d2f_integration_logo");
    let _ = doc2flow::io::create_dir_all(&temp_dir);

    let fm_logo_path = temp_dir.join("fm_logo.svg");
    doc2flow::io::write_file(
        &fm_logo_path,
        "<svg id=\"fm-logo\" width=\"10\"><rect/></svg>",
    )
    .unwrap();

    let cli_logo_path = temp_dir.join("cli_logo.png");
    doc2flow::io::write_file(&cli_logo_path, b"cli png data").unwrap();

    let input = format!(
        r#"---
title: "Custom Logo Spec"
company: "Logo Inc"
date: "2026-07-27"
logo: "{}"
---
## Section 1
- [ ] Check logo
"#,
        fm_logo_path.file_name().unwrap().to_str().unwrap()
    );

    let (fm, body) =
        doc2flow::converter::parse_and_validate_frontmatter(&input, Some("logo_spec.md")).unwrap();
    let locale = doc2flow::language::Locale::from_lang_code(fm.language.as_deref().unwrap_or("en"));
    let (html_body, features) =
        doc2flow::converter::convert_markdown_to_html_with_locale(body, &locale).unwrap();
    let d2f_id = doc2flow::id::generate_d2f_id(&fm).unwrap();

    // 1. Frontmatter logo resolution
    let fm_logo_html = doc2flow::image::load_logo(
        fm.logo.as_deref().map(Path::new),
        Some(&temp_dir),
    );
    assert!(fm_logo_html.contains("id=\"fm-logo\""));

    let rendered_fm =
        doc2flow::template::render(&fm, &locale, &html_body, &d2f_id, Some(&fm_logo_html), &features)
            .unwrap();
    assert!(rendered_fm.contains("id=\"fm-logo\""));

    // 2. CLI logo precedence over frontmatter logo
    let cli_logo_html = doc2flow::image::load_logo(
        Some(&cli_logo_path),
        Some(&temp_dir),
    );
    assert!(cli_logo_html.contains("<img src=\"data:image/png;base64,"));

    let rendered_cli =
        doc2flow::template::render(&fm, &locale, &html_body, &d2f_id, Some(&cli_logo_html), &features)
            .unwrap();
    assert!(rendered_cli.contains("<img src=\"data:image/png;base64,"));
    assert!(!rendered_cli.contains("id=\"fm-logo\""));

    let _ = doc2flow::io::remove_dir_all(&temp_dir);
}

#[test]
fn test_metadata_injection_integration() {
    let input = r#"---
title: "Metadata Spec"
company: "Acme Corp"
version: "1.0.0"
date: "2026-07-27"
---
## Section 1
- [ ] Test metadata
"#;

    let (fm, body) =
        doc2flow::converter::parse_and_validate_frontmatter(input, Some("meta_spec.md")).unwrap();
    let locale = doc2flow::language::Locale::from_lang_code(fm.language.as_deref().unwrap_or("en"));
    let (html_body, features) =
        doc2flow::converter::convert_markdown_to_html_with_locale(body, &locale).unwrap();
    let d2f_id = doc2flow::id::generate_d2f_id(&fm).unwrap();

    let rendered = doc2flow::template::render(&fm, &locale, &html_body, &d2f_id, None, &features).unwrap();

    // Verify meta tags
    let raw_ver = doc2flow::template::APP_VERSION.strip_prefix('v').unwrap_or(doc2flow::template::APP_VERSION);
    assert!(rendered.contains(&format!("<meta name=\"generator\" content=\"Doc2Flow {}\">", doc2flow::template::APP_VERSION)));
    assert!(rendered.contains(&format!("<meta name=\"version\" content=\"{}\">", raw_ver)));
    assert!(rendered.contains("<meta name=\"repository\" content=\"https://github.com/akjir/doc2flow\">"));
    assert!(rendered.contains("<meta name=\"license\" content=\"https://github.com/akjir/doc2flow/blob/main/LICENSE\">"));
    assert!(rendered.contains("<meta name=\"dcterms.created\" content=\""));
    assert!(rendered.contains("<meta name=\"dcterms.source\" content=\"https://github.com/akjir/doc2flow\">"));

    // Verify template.md metadata comments
    let init_tmpl = doc2flow::template::generate_template_markdown();
    assert!(init_tmpl.contains(&format!("DOC2FLOW (D2F) {} - TEMPLATE & USAGE GUIDE", doc2flow::template::APP_VERSION)));
    assert!(init_tmpl.contains("Repository: https://github.com/akjir/doc2flow"));
    assert!(init_tmpl.contains("License: GPL-3.0-or-later"));
}

#[test]
fn test_loose_task_list_integration() {
    let input = r#"---
title: "Loose Task List Test"
company: "Acme Corp"
date: "2026-07-28"
---
## Loose Checklist

- [ ] Item 1

- [x] Item 2

- [ ] Item 3
"#;

    let (fm, body) =
        doc2flow::converter::parse_and_validate_frontmatter(input, Some("loose_task.md")).unwrap();
    let locale = doc2flow::language::Locale::from_lang_code(fm.language.as_deref().unwrap_or("en"));
    let (html_body, _features) =
        doc2flow::converter::convert_markdown_to_html_with_locale(body, &locale).unwrap();

    assert!(html_body.contains(r#"<div class="doc-item check-item" id="wrap-cb_s1_1">"#));
    assert!(html_body.contains(r#"<input type="checkbox" id="cb_s1_1">"#));
    assert!(html_body.contains(r#"<label class="check-label" for="cb_s1_1">Item 1</label>"#));

    assert!(html_body.contains(r#"<div class="doc-item check-item checked" id="wrap-cb_s1_2">"#));
    assert!(html_body.contains(r#"<input type="checkbox" id="cb_s1_2" checked=""#) || html_body.contains(r#"<input type="checkbox" id="cb_s1_2" checked>"#));
    assert!(html_body.contains(r#"<label class="check-label" for="cb_s1_2">Item 2</label>"#));

    assert!(html_body.contains(r#"<div class="doc-item check-item" id="wrap-cb_s1_3">"#));
    assert!(html_body.contains(r#"<input type="checkbox" id="cb_s1_3">"#));
    assert!(html_body.contains(r#"<label class="check-label" for="cb_s1_3">Item 3</label>"#));

    assert!(!html_body.contains("simple-item"));
    assert!(!html_body.contains("<p>Item"));
    assert!(!html_body.contains("Item 1</p>"));
}

#[test]
fn test_section_numbering_integration() {
    let input = r#"---
title: "Section Numbering Test"
company: "Acme Corp"
date: "2026-07-29"
number_sections: true
---
# Main Architecture

## Overview & Scope
- [ ] Task 1

## Key Guidelines
- [ ] Task 2

# Deployment Guide

## Prerequisites
- [x] Done
"#;

    let (fm, body) =
        doc2flow::converter::parse_and_validate_frontmatter(input, Some("numbering.md")).unwrap();
    assert!(fm.number_sections);

    let locale = doc2flow::language::Locale::from_lang_code(fm.language.as_deref().unwrap_or("en"));
    let (html_body, _features) = doc2flow::converter::convert_markdown_to_html_with_options(
        body,
        &locale,
        fm.number_sections,
    )
    .unwrap();

    assert!(html_body.contains("<span>1. Main Architecture</span>"));
    assert!(html_body.contains("<span>1.1 Overview &amp; Scope</span>"));
    assert!(html_body.contains("<span>1.2 Key Guidelines</span>"));
    assert!(html_body.contains("<span>2. Deployment Guide</span>"));
    assert!(html_body.contains("<span>2.1 Prerequisites</span>"));

    // Verify section IDs and classes are preserved
    assert!(html_body.contains(r#"<section class="section" id="s1">"#));
    assert!(html_body.contains(r#"id="badge-s1""#));
    assert!(html_body.contains(r#"id="tog-s1""#));
    assert!(html_body.contains(r#"<section class="section" id="s2" data-has-checklist="true">"#));
    assert!(html_body.contains(r#"<section class="section" id="s3" data-has-checklist="true">"#));
}

#[test]
fn test_search_toolbar_integration() {
    let input = r#"---
title: "Search Toolbar Test"
company: "Test Corp"
date: "2026-07-29"
---
# Main Section

- [ ] Task 1

## Callout Section

>!! Warning note
"#;

    let (fm, body) =
        doc2flow::converter::parse_and_validate_frontmatter(input, Some("test.md")).unwrap();
    let locale = doc2flow::language::Locale::from_lang_code("en");
    let (html_body, features) = doc2flow::converter::convert_markdown_to_html(&body).unwrap();
    let full_doc = doc2flow::template::render(&fm, &locale, &html_body, "doc123", None, &features).unwrap();

    assert!(full_doc.contains(r#"<div class="search-toolbar hidden" id="search-toolbar">"#));
    assert!(full_doc.contains(r#"id="search-toggle-btn""#));
    assert!(full_doc.contains(r#"id="search-input""#));
    assert!(full_doc.contains(r#"id="search-clear-btn""#));
    assert!(full_doc.contains(r#"id="search-counter""#));
    assert!(full_doc.contains(r#"data-has-checklist="true""#));
    assert!(full_doc.contains(r#"data-callout-type="warning""#));
}
