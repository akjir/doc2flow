use doc2flow::core::arguments::parse_args;
use doc2flow::core::builder::build;
use doc2flow::core::markdown::parse_d2f_markdown;

#[test]
fn test_cli_core_arguments_defaults() {
    let args = parse_args(["input.md"]).unwrap();
    assert_eq!(args.input, Some(std::path::PathBuf::from("input.md")));
    assert_eq!(args.output, None);
    assert_eq!(args.logo, None);
    assert_eq!(args.init, None);
    assert!(!args.auto_scale);
    assert!(!args.show_help);
    assert!(!args.show_version);
}

#[test]
fn test_callout_variants_pipeline_integration() {
    let input = r#"---
title: "Callouts Test"
---
# Main Section

> Standard Note text

>? Tip text

>! Important text

>!! Warning text

>!!! Caution text
"#;

    let document = parse_d2f_markdown(input).expect("parse failed");
    let content = build(&document);

    assert!(content.contains("<div class=\"shoutout shoutout-note\" data-label=\"Note\">\n          Standard Note text\n        </div>"));
    assert!(content.contains("<div class=\"shoutout shoutout-tip\" data-label=\"Tip\">\n          Tip text\n        </div>"));
    assert!(content.contains("<div class=\"shoutout shoutout-important\" data-label=\"Important\">\n          Important text\n        </div>"));
    assert!(content.contains("<div class=\"shoutout shoutout-warning\" data-label=\"Warning\">\n          Warning text\n        </div>"));
    assert!(content.contains("<div class=\"shoutout shoutout-caution\" data-label=\"Caution\">\n          Caution text\n        </div>"));
}

#[test]
fn test_parser_and_builder_pipeline_integration() {
    let input = "---\ntitle: \"Pipeline Test\"\n---\n# Pipeline Heading\n\nContent paragraph\n\n- [ ] Task item";
    let document = doc2flow::core::parse_d2f_markdown(input).expect("parse failed");
    let content = doc2flow::core::builder::build(&document);
    assert!(!content.is_empty());
    assert!(content.contains("<h1 class=\"section-header section-header-h1\" role=\"button\" tabindex=\"0\" aria-expanded=\"true\">"));
    assert!(content.contains("<span class=\"section-title\">Pipeline Heading</span>"));
    assert!(content.contains("<span class=\"section-toggler\">&#9660;</span>"));
    assert!(
        content.contains(
            "        <div class=\"item text-item item-selectable\">\n          <span class=\"text-content\">\n            Content paragraph\n          </span>\n          <span class=\"item-comment-icon\">"
        )
    );
    assert!(content.contains("<title>Pipeline Test</title>"));
    assert!(content.contains("<meta name=\"features\" content=\"core, header, comment, input, task\">"));
    assert!(content.contains("window.d2f.sections"));
    assert!(content.contains("window.d2f.comments"));
    assert!(!content.contains("{{FEATURES}}"));
    assert!(!content.contains("{{TITLE}}"));
}

#[test]
fn test_header_pipeline_integration() {
    let input = r#"---
title: "Header Feature Integration"
subtitle: "Verifying top banner card"
---
# First Section

Section content
"#;
    let document = doc2flow::core::parse_d2f_markdown(input).expect("parse failed");
    let content = doc2flow::core::builder::build(&document);

    assert!(content.contains("<title>Header Feature Integration</title>"));
    assert!(content.contains("<meta name=\"features\" content=\"core, header, comment, input\">"));
    assert!(content.contains("<section class=\"section header-container\" id=\"header\">"));
    assert!(content.contains("<h1 class=\"header-title\">Header Feature Integration</h1>"));
    assert!(content.contains("<div class=\"header-sub\">Verifying top banner card</div>"));
    assert!(content.contains("--header-bg:"));

    // Verify ordering: header precedes body section
    let header_pos = content.find("header-container").unwrap();
    let body_pos = content.find("First Section").unwrap();
    assert!(header_pos < body_pos);
}

#[test]
fn test_header_pipeline_integration_disabled() {
    let input = r#"---
title: "Header Disabled Integration"
header: false
---
# Main Section

Section content
"#;
    let document = doc2flow::core::parse_d2f_markdown(input).expect("parse failed");
    let content = doc2flow::core::builder::build(&document);

    assert!(content.contains("<title>Header Disabled Integration</title>"));
    assert!(content.contains("<meta name=\"features\" content=\"core, comment, input\">"));
    assert!(!content.contains("header-container"));
    assert!(!content.contains("header-title"));
}

#[test]
fn test_core_horizontal_rule_pipeline_integration() {
    let input = "---\ntitle: \"HR Pipeline Test\"\n---\n# Main Heading\n\nParagraph before\n\n---\n\n----\n\nParagraph after";
    let document = doc2flow::core::parse_d2f_markdown(input).expect("parse failed");
    let content = doc2flow::core::builder::build(&document);
    assert!(content.contains("<title>HR Pipeline Test</title>"));
    assert!(content.contains("<meta name=\"features\" content=\"core, header, comment, input\">"));
    assert!(content.contains("<h1 class=\"section-header section-header-h1\" role=\"button\" tabindex=\"0\" aria-expanded=\"true\">"));
    assert!(content.contains("<span class=\"section-title\">Main Heading</span>"));
    assert!(
        content.contains(
            "        <div class=\"item text-item item-selectable\">\n          <span class=\"text-content\">\n            Paragraph before\n          </span>\n          <span class=\"item-comment-icon\">"
        )
    );
    assert!(content.contains("        <hr />"));
    assert!(
        content
            .contains("        <div class=\"item text-item item-selectable\">\n          <span class=\"text-content\">\n            Paragraph after\n          </span>\n          <span class=\"item-comment-icon\">")
    );
    assert!(content.contains("hr {\n      border: none;"));
}

#[test]
fn test_collapsible_sections_pipeline_integration() {
    let input = "---\ntitle: \"Sections Test\"\n---\n# Container H1\n\nText in H1\n\n## Container H2\n\nText in H2\n\n### Subheading H3\n\nText in H3";
    let document = doc2flow::core::parse_d2f_markdown(input).expect("parse failed");
    let content = doc2flow::core::builder::build(&document);

    assert!(content.contains("<section class=\"section\" data-level=\"1\">"));
    assert!(content.contains("<h1 class=\"section-header section-header-h1\" role=\"button\" tabindex=\"0\" aria-expanded=\"true\">"));
    assert!(content.contains("<span class=\"section-title\">Container H1</span>"));

    assert!(content.contains("<section class=\"section\" data-level=\"2\">"));
    assert!(content.contains(
        "<h2 class=\"section-header\" role=\"button\" tabindex=\"0\" aria-expanded=\"true\">"
    ));
    assert!(content.contains("<span class=\"section-title\">Container H2</span>"));

    assert!(content.contains("<section class=\"section\" data-level=\"3\">"));
    assert!(content.contains("<h3 class=\"section-subheading\">Subheading H3</h3>"));

    assert!(content.contains("window.d2f.sections"));
    assert!(content.contains("toggleSection"));
    assert!(content.contains("setSectionCollapseState"));
}

#[test]
fn test_numbered_sections_pipeline_integration_enabled() {
    let input = "---\ntitle: \"Numbered Sections Test\"\nnumbered_sections: true\n---\n# Container H1\n\nText in H1\n\n## Container H2\n\nText in H2\n\n# Second H1\n\n## Second H2";
    let document = doc2flow::core::parse_d2f_markdown(input).expect("parse failed");
    let content = doc2flow::core::builder::build(&document);

    assert!(content.contains("<span class=\"section-title\">1. Container H1</span>"));
    assert!(content.contains("<span class=\"section-title\">1.1 Container H2</span>"));
    assert!(content.contains("<span class=\"section-title\">2. Second H1</span>"));
    assert!(content.contains("<span class=\"section-title\">2.1 Second H2</span>"));
}

#[test]
fn test_numbered_sections_pipeline_integration_disabled() {
    let input = "---\ntitle: \"Unnumbered Sections Test\"\nnumbered_sections: false\n---\n# Container H1\n\nText in H1\n\n## Container H2\n\nText in H2\n\n# Second H1\n\n## Second H2";
    let document = doc2flow::core::parse_d2f_markdown(input).expect("parse failed");
    let content = doc2flow::core::builder::build(&document);

    assert!(content.contains("<span class=\"section-title\">Container H1</span>"));
    assert!(content.contains("<span class=\"section-title\">Container H2</span>"));
    assert!(content.contains("<span class=\"section-title\">Second H1</span>"));
    assert!(content.contains("<span class=\"section-title\">Second H2</span>"));
    assert!(!content.contains("<span class=\"section-title\">1. Container H1</span>"));
    assert!(!content.contains("<span class=\"section-title\">1.1 Container H2</span>"));
}

#[test]
fn test_bullet_pipeline_integration() {
    let input = "---\ntitle: \"Bullet Pipeline Test\"\n---\n# List Section\n\n- Root bullet item\n  - Nested bullet **item** with `code`";
    let document = doc2flow::core::parse_d2f_markdown(input).expect("parse failed");
    let content = doc2flow::core::builder::build(&document);
    assert!(content.contains("<title>Bullet Pipeline Test</title>"));
    assert!(content.contains("<meta name=\"features\" content=\"core, header, bullet, comment, input\">"));
    assert!(content.contains("--bullet-marker-color:"));
    assert!(content.contains(".bullet-marker"));
    assert!(content.contains("        <div class=\"item bullet-item item-selectable\">\n          <span class=\"bullet-marker\">&bull;</span>\n          <span class=\"bullet-content\">\n            Root bullet item\n          </span>\n          <span class=\"item-comment-icon\">"));
    assert!(content.contains("        <div class=\"item bullet-item item-selectable\" style=\"--indent: 1;\">\n          <span class=\"bullet-marker\">&bull;</span>\n          <span class=\"bullet-content\">\n            Nested bullet <strong>item</strong> with <code>code</code>\n          </span>\n          <span class=\"item-comment-icon\">"));
}

#[test]
fn test_ordered_pipeline_integration() {
    let input = "---\ntitle: \"Ordered Pipeline Test\"\n---\n# Ordered Section\n\n1. First ordered step\n2. Second **ordered** step\n  1. Sub-step alpha\n    1. Sub-sub-step roman";
    let document = doc2flow::core::parse_d2f_markdown(input).expect("parse failed");
    let content = doc2flow::core::builder::build(&document);
    assert!(content.contains("<title>Ordered Pipeline Test</title>"));
    assert!(content.contains("<meta name=\"features\" content=\"core, header, comment, input, ordered\">"));
    assert!(content.contains("--order-marker-color:"));
    assert!(content.contains(".order-marker"));
    assert!(content.contains(".order-content"));
    assert!(content.contains("        <div class=\"item order-item item-selectable\">\n          <span class=\"order-marker\">1.</span>\n          <span class=\"order-content\">\n            First ordered step\n          </span>\n          <span class=\"item-comment-icon\">"));
    assert!(content.contains("        <div class=\"item order-item item-selectable\">\n          <span class=\"order-marker\">2.</span>\n          <span class=\"order-content\">\n            Second <strong>ordered</strong> step\n          </span>\n          <span class=\"item-comment-icon\">"));
    assert!(content.contains("        <div class=\"item order-item item-selectable\" style=\"--indent: 1;\">\n          <span class=\"order-marker\">a.</span>\n          <span class=\"order-content\">\n            Sub-step alpha\n          </span>\n          <span class=\"item-comment-icon\">"));
    assert!(content.contains("        <div class=\"item order-item item-selectable\" style=\"--indent: 2;\">\n          <span class=\"order-marker\">i.</span>\n          <span class=\"order-content\">\n            Sub-sub-step roman\n          </span>\n          <span class=\"item-comment-icon\">"));
}

#[test]
fn test_task_pipeline_integration() {
    let input = "---\ntitle: \"Checkbox Pipeline Test\"\n---\n# Checkbox Section\n\n- [ ] Pending task\n- [x] Done **task** with `code`\n  - [ ] Sub-task";
    let document = doc2flow::core::parse_d2f_markdown(input).expect("parse failed");
    let content = doc2flow::core::builder::build(&document);
    assert!(content.contains("<title>Checkbox Pipeline Test</title>"));
    assert!(content.contains("<meta name=\"features\" content=\"core, header, comment, input, task\">"));
    assert!(content.contains(".check-marker"));
    assert!(content.contains(".check-box"));
    assert!(content.contains(".check-content"));
    assert!(content.contains(".check-item.checked"));
    assert!(content.contains("        <div class=\"item check-item item-selectable\">\n          <span class=\"check-marker\">\n            <input type=\"checkbox\" class=\"check-box\" />\n          </span>\n          <span class=\"check-content\">\n            Pending task\n          </span>\n          <span class=\"item-comment-icon\">"));
    assert!(content.contains("        <div class=\"item check-item item-selectable checked\">\n          <span class=\"check-marker\">\n            <input type=\"checkbox\" class=\"check-box\" checked />\n          </span>\n          <span class=\"check-content\">\n            Done <strong>task</strong> with <code>code</code>\n          </span>\n          <span class=\"item-comment-icon\">"));
    assert!(content.contains("        <div class=\"item check-item item-selectable\" style=\"--indent: 1;\">\n          <span class=\"check-marker\">\n            <input type=\"checkbox\" class=\"check-box\" />\n          </span>\n          <span class=\"check-content\">\n            Sub-task\n          </span>\n          <span class=\"item-comment-icon\">"));
}

#[test]
fn test_table_pipeline_integration() {
    let input = "---\ntitle: \"Table Pipeline Test\"\n---\n# Table Section\n\n| Item | Qty | Status |\n| :--- | :---: | ---: |\n| **Widget A** | 42 | In Stock |\n| `Widget B` | 0 | *Out of Stock* |";
    let document = doc2flow::core::parse_d2f_markdown(input).expect("parse failed");
    let content = doc2flow::core::builder::build(&document);
    assert!(content.contains("<title>Table Pipeline Test</title>"));
    assert!(content.contains("<meta name=\"features\" content=\"core, header, comment, input, table\">"));
    assert!(content.contains("--table-border-color:"));
    assert!(content.contains("--table-header-bg:"));
    assert!(content.contains(".table-wrap"));
    assert!(content.contains(".table-default"));
    assert!(content.contains("window.d2f.table"));
    assert!(content.contains("        <div class=\"table-wrap\">\n          <table class=\"table-default\">\n            <thead>\n              <tr>\n                <th style=\"text-align: left;\">Item</th>\n                <th style=\"text-align: center;\">Qty</th>\n                <th style=\"text-align: right;\">Status</th>\n              </tr>\n            </thead>\n            <tbody>\n              <tr>\n                <td style=\"text-align: left;\"><strong>Widget A</strong></td>\n                <td style=\"text-align: center;\">42</td>\n                <td style=\"text-align: right;\">In Stock</td>\n              </tr>\n              <tr>\n                <td style=\"text-align: left;\"><code>Widget B</code></td>\n                <td style=\"text-align: center;\">0</td>\n                <td style=\"text-align: right;\"><em>Out of Stock</em></td>\n              </tr>\n            </tbody>\n          </table>\n        </div>"));
}

#[test]
fn test_code_variable_pipeline_without_variables_directive() {
    let input = "---\ntitle: \"Code Vars Test\"\n---\n# Code Section\n\n```bash\ncurl -H \"Authorization: Bearer {{AUTH_TOKEN}}\" https://{{TARGET_HOST}}:{{PORT}}/api\n```";
    let document = doc2flow::core::parse_d2f_markdown(input).expect("parse failed");
    let content = doc2flow::core::builder::build(&document);
    assert!(content.contains("<title>Code Vars Test</title>"));
    assert!(content.contains("<meta name=\"features\" content=\"core, header, code, comment, input\">"));
    assert!(content.contains("class=\"code-table-wrap\""));
    assert!(content.contains("class=\"code-table-default\""));
    assert!(content.contains("<td>AUTH_TOKEN</td>"));
    assert!(content.contains("<td>PORT</td>"));
    assert!(content.contains("<td>TARGET_HOST</td>"));
    assert!(content.contains("<input type=\"text\" class=\"code-table-input input-field\" id=\"f_var_AUTH_TOKEN\" data-var-key=\"AUTH_TOKEN\" value=\"\">"));
    assert!(content.contains("curl -H &quot;Authorization: Bearer {{AUTH_TOKEN}}&quot; https://{{TARGET_HOST}}:{{PORT}}/api"));
}

#[test]
fn test_code_variable_pipeline_with_variables_directive() {
    let input = "---\ntitle: \"Code Vars Combined Test\"\n---\n:::variables\n| Variable | Value |\n| --- | --- |\n| TARGET_HOST | 192.168.1.50 |\n| PORT | 8080 |\n:::\n# Code Section\n\n```bash\ncurl -H \"Authorization: Bearer {{AUTH_TOKEN}}\" https://{{TARGET_HOST}}:{{PORT}}/api\n```";
    let document = doc2flow::core::parse_d2f_markdown(input).expect("parse failed");
    let content = doc2flow::core::builder::build(&document);
    assert!(content.contains("<title>Code Vars Combined Test</title>"));
    assert!(content.contains("<meta name=\"features\" content=\"core, header, code, comment, input\">"));
    assert!(content.contains("<td>TARGET_HOST</td>\n            <td><input type=\"text\" class=\"code-table-input input-field\" id=\"f_var_TARGET_HOST\" data-var-key=\"TARGET_HOST\" value=\"192.168.1.50\"></td>"));
    assert!(content.contains("<td>PORT</td>\n            <td><input type=\"text\" class=\"code-table-input input-field\" id=\"f_var_PORT\" data-var-key=\"PORT\" value=\"8080\"></td>"));
    assert!(content.contains("<td>AUTH_TOKEN</td>\n            <td><input type=\"text\" class=\"code-table-input input-field\" id=\"f_var_AUTH_TOKEN\" data-var-key=\"AUTH_TOKEN\" value=\"\"></td>"));
}

#[test]
fn test_image_and_link_pipeline_integration() {
    let input = "---\ntitle: \"Image and Link Test\"\n---\n# Media Section\n\n![Sample Image](images/example1.jpg)\n\nCheck the [Documentation](https://example.com/docs) for details.";
    let document = doc2flow::core::parse_d2f_markdown(input).expect("parse failed");
    let content = doc2flow::core::builder::build(&document);
    assert!(content.contains("<title>Image and Link Test</title>"));
    assert!(content.contains("<meta name=\"features\" content=\"core, header, comment, image, input\">"));
    assert!(content.contains("<div class=\"image-item\">\n          <img src=\"images/example1.jpg\" alt=\"Sample Image\" />\n        </div>"));
    assert!(content.contains("<a href=\"https://example.com/docs\">Documentation</a>"));
}

#[test]
fn test_image_embedding_pipeline_local_and_remote() {
    let temp_dir = std::env::temp_dir().join(format!("d2f_integ_img_local_{}", std::process::id()));
    let _ = doc2flow::core::io::create_dir_all(&temp_dir);

    let pic_path = temp_dir.join("diagram.png");
    doc2flow::core::io::write_file(&pic_path, b"png binary payload").unwrap();

    let input = "---\ntitle: \"Image Embed Test\"\n---\n# Architecture\n\n![Diagram](diagram.png)\n\n![Remote](https://example.com/logo.png)";
    let document = doc2flow::core::parse_d2f_markdown(input).expect("parse failed");
    let rendered = doc2flow::core::builder::build(&document);

    let final_html = doc2flow::features::image::embed_images_as_base64_with_source(
        &rendered,
        Some(input),
        Some("doc.md"),
        Some(&temp_dir),
        false,
    )
    .expect("embedding should succeed");

    assert!(final_html.contains("src=\"data:image/png;base64,"));
    assert!(!final_html.contains("src=\"diagram.png\""));
    assert!(final_html.contains("src=\"https://example.com/logo.png\""));

    let _ = doc2flow::core::io::remove_dir_all(&temp_dir);
}

#[test]
fn test_image_embedding_pipeline_large_image_auto_scale() {
    let temp_dir = std::env::temp_dir().join(format!("d2f_integ_img_scale_{}", std::process::id()));
    let _ = doc2flow::core::io::create_dir_all(&temp_dir);

    let pic_path = temp_dir.join("huge.png");
    let img_buf = image::RgbImage::new(800, 800);
    img_buf
        .save_with_format(&pic_path, image::ImageFormat::Png)
        .unwrap();

    let file_size = doc2flow::core::io::get_file_size(&pic_path).unwrap();
    if file_size <= doc2flow::core::image::MAX_IMAGE_SIZE_BYTES {
        let mut existing = doc2flow::core::io::read_file_bytes(&pic_path).unwrap();
        existing.resize(
            (doc2flow::core::image::MAX_IMAGE_SIZE_BYTES + 40 * 1024) as usize,
            0,
        );
        doc2flow::core::io::write_file(&pic_path, &existing).unwrap();
    }

    let input = "---\ntitle: \"Large Image Scale Test\"\n---\n# Overview\n\n![Huge](huge.png)";
    let document = doc2flow::core::parse_d2f_markdown(input).expect("parse failed");
    let rendered = doc2flow::core::builder::build(&document);

    let final_html = doc2flow::features::image::embed_images_as_base64_with_source(
        &rendered,
        Some(input),
        Some("overview.md"),
        Some(&temp_dir),
        true,
    )
    .expect("auto scaling should succeed");

    assert!(final_html.contains("src=\"data:image/webp;base64,"));
    assert!(temp_dir.join("huge.webp").exists());

    let _ = doc2flow::core::io::remove_dir_all(&temp_dir);
}

#[test]
fn test_image_embedding_pipeline_large_image_diagnostic_error() {
    let temp_dir = std::env::temp_dir().join(format!("d2f_integ_img_err_{}", std::process::id()));
    let _ = doc2flow::core::io::create_dir_all(&temp_dir);

    let pic_path = temp_dir.join("unscaled.png");
    let img_buf = image::RgbImage::new(600, 600);
    img_buf
        .save_with_format(&pic_path, image::ImageFormat::Png)
        .unwrap();

    let mut existing = doc2flow::core::io::read_file_bytes(&pic_path).unwrap();
    existing.resize(
        (doc2flow::core::image::MAX_IMAGE_SIZE_BYTES + 20 * 1024) as usize,
        0,
    );
    doc2flow::core::io::write_file(&pic_path, &existing).unwrap();

    let input =
        "---\ntitle: \"Large Image Err Test\"\n---\n# Overview\n\n![Unscaled](unscaled.png)";
    let document = doc2flow::core::parse_d2f_markdown(input).expect("parse failed");
    let rendered = doc2flow::core::builder::build(&document);

    let err = doc2flow::features::image::embed_images_as_base64_with_source(
        &rendered,
        Some(input),
        Some("doc.md"),
        Some(&temp_dir),
        false,
    )
    .unwrap_err();

    let err_msg = err.to_string();
    assert!(err_msg.contains("error: image 'unscaled.png' exceeds maximum allowed size of 250 KB"));
    assert!(err_msg.contains("--> doc.md:6:13"));
    assert!(err_msg.contains("6 | ![Unscaled](unscaled.png)"));
    assert!(err_msg.contains("^^^^^^^^^^^^ local image size"));
    assert!(err_msg.contains(
        "= help: reduce image resolution or compress 'unscaled.png' below 250 KB before embedding."
    ));

    let _ = doc2flow::core::io::remove_dir_all(&temp_dir);
}

#[test]
fn test_image_embedding_pipeline_non_image_attachment_link() {
    let input = "---\ntitle: \"PDF Test\"\n---\n# Downloads\n\n![Download PDF](https://example.com/guide.pdf)";
    let document = doc2flow::core::parse_d2f_markdown(input).expect("parse failed");
    let rendered = doc2flow::core::builder::build(&document);

    let final_html = doc2flow::features::image::embed_images_as_base64_with_source(
        &rendered,
        Some(input),
        Some("downloads.md"),
        None,
        false,
    )
    .expect("non image conversion should succeed");

    assert!(final_html.contains("<div class=\"item text-item item-selectable\">"));
    assert!(final_html.contains("<span class=\"text-content\"><a href=\"https://example.com/guide.pdf\" target=\"_blank\" rel=\"noopener noreferrer\">Download PDF</a></span>"));
    assert!(!final_html.contains("class=\"image-item\""));
    assert!(!final_html.contains("<img src=\"https://example.com/guide.pdf\""));
}
