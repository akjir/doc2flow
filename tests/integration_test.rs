use doc2flow::core::arguments::parse_args;
use doc2flow::core::builder::build;
use doc2flow::core::document::{DocumentElement, ShoutoutElementKind};
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
            "        <div class=\"item text-item\">\n          <span class=\"text-content\">\n            Content paragraph\n          </span>\n        </div>"
        )
    );
    assert!(content.contains("<title>Pipeline Test</title>"));
    assert!(content.contains("<meta name=\"features\" content=\"core, task\">"));
    assert!(content.contains("window.d2f.sections"));
    assert!(!content.contains("{{FEATURES}}"));
    assert!(!content.contains("{{TITLE}}"));
}

#[test]
fn test_core_horizontal_rule_pipeline_integration() {
    let input = "---\ntitle: \"HR Pipeline Test\"\n---\n# Main Heading\n\nParagraph before\n\n---\n\n----\n\nParagraph after";
    let document = doc2flow::core::parse_d2f_markdown(input).expect("parse failed");
    let content = doc2flow::core::builder::build(&document);
    assert!(content.contains("<title>HR Pipeline Test</title>"));
    assert!(content.contains("<meta name=\"features\" content=\"core\">"));
    assert!(content.contains("<h1 class=\"section-header section-header-h1\" role=\"button\" tabindex=\"0\" aria-expanded=\"true\">"));
    assert!(content.contains("<span class=\"section-title\">Main Heading</span>"));
    assert!(
        content.contains(
            "        <div class=\"item text-item\">\n          <span class=\"text-content\">\n            Paragraph before\n          </span>\n        </div>"
        )
    );
    assert!(content.contains("        <hr />"));
    assert!(
        content
            .contains("        <div class=\"item text-item\">\n          <span class=\"text-content\">\n            Paragraph after\n          </span>\n        </div>")
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
    assert!(content.contains("<h2 class=\"section-header\" role=\"button\" tabindex=\"0\" aria-expanded=\"true\">"));
    assert!(content.contains("<span class=\"section-title\">Container H2</span>"));

    assert!(content.contains("<section class=\"section\" data-level=\"3\">"));
    assert!(content.contains("<h3 class=\"section-subheading\">Subheading H3</h3>"));

    assert!(content.contains("window.d2f.sections"));
    assert!(content.contains("toggleSection"));
    assert!(content.contains("setSectionCollapseState"));
}

#[test]
fn test_bullet_pipeline_integration() {
    let input = "---\ntitle: \"Bullet Pipeline Test\"\n---\n# List Section\n\n- Root bullet item\n  - Nested bullet **item** with `code`";
    let document = doc2flow::core::parse_d2f_markdown(input).expect("parse failed");
    let content = doc2flow::core::builder::build(&document);
    assert!(content.contains("<title>Bullet Pipeline Test</title>"));
    assert!(content.contains("<meta name=\"features\" content=\"core, bullet\">"));
    assert!(content.contains("--bullet-marker-color:"));
    assert!(content.contains(".bullet-marker"));
    assert!(content.contains("        <div class=\"item bullet-item\">\n          <span class=\"bullet-marker\">&bull;</span>\n          <span class=\"bullet-content\">\n            Root bullet item\n          </span>\n        </div>"));
    assert!(content.contains("        <div class=\"item bullet-item\" style=\"--indent: 1;\">\n          <span class=\"bullet-marker\">&bull;</span>\n          <span class=\"bullet-content\">\n            Nested bullet <strong>item</strong> with <code>code</code>\n          </span>\n        </div>"));
}

#[test]
fn test_ordered_pipeline_integration() {
    let input = "---\ntitle: \"Ordered Pipeline Test\"\n---\n# Ordered Section\n\n1. First ordered step\n2. Second **ordered** step\n  1. Sub-step alpha\n    1. Sub-sub-step roman";
    let document = doc2flow::core::parse_d2f_markdown(input).expect("parse failed");
    let content = doc2flow::core::builder::build(&document);
    assert!(content.contains("<title>Ordered Pipeline Test</title>"));
    assert!(content.contains("<meta name=\"features\" content=\"core, ordered\">"));
    assert!(content.contains("--order-marker-color:"));
    assert!(content.contains(".order-marker"));
    assert!(content.contains(".order-content"));
    assert!(content.contains("        <div class=\"item order-item\">\n          <span class=\"order-marker\">1.</span>\n          <span class=\"order-content\">\n            First ordered step\n          </span>\n        </div>"));
    assert!(content.contains("        <div class=\"item order-item\">\n          <span class=\"order-marker\">2.</span>\n          <span class=\"order-content\">\n            Second <strong>ordered</strong> step\n          </span>\n        </div>"));
    assert!(content.contains("        <div class=\"item order-item\" style=\"--indent: 1;\">\n          <span class=\"order-marker\">1.</span>\n          <span class=\"order-content\">\n            Sub-step alpha\n          </span>\n        </div>"));
    assert!(content.contains("        <div class=\"item order-item\" style=\"--indent: 2;\">\n          <span class=\"order-marker\">1.</span>\n          <span class=\"order-content\">\n            Sub-sub-step roman\n          </span>\n        </div>"));
}

#[test]
fn test_task_pipeline_integration() {
    let input = "---\ntitle: \"Checkbox Pipeline Test\"\n---\n# Checkbox Section\n\n- [ ] Pending task\n- [x] Done **task** with `code`\n  - [ ] Sub-task";
    let document = doc2flow::core::parse_d2f_markdown(input).expect("parse failed");
    let content = doc2flow::core::builder::build(&document);
    assert!(content.contains("<title>Checkbox Pipeline Test</title>"));
    assert!(content.contains("<meta name=\"features\" content=\"core, task\">"));
    assert!(content.contains(".check-marker"));
    assert!(content.contains(".check-box"));
    assert!(content.contains(".check-content"));
    assert!(content.contains(".check-item.checked"));
    assert!(content.contains("        <div class=\"item check-item\">\n          <span class=\"check-marker\">\n            <input type=\"checkbox\" class=\"check-box\" />\n          </span>\n          <span class=\"check-content\">\n            Pending task\n          </span>\n        </div>"));
    assert!(content.contains("        <div class=\"item check-item checked\">\n          <span class=\"check-marker\">\n            <input type=\"checkbox\" class=\"check-box\" checked />\n          </span>\n          <span class=\"check-content\">\n            Done <strong>task</strong> with <code>code</code>\n          </span>\n        </div>"));
    assert!(content.contains("        <div class=\"item check-item\" style=\"--indent: 1;\">\n          <span class=\"check-marker\">\n            <input type=\"checkbox\" class=\"check-box\" />\n          </span>\n          <span class=\"check-content\">\n            Sub-task\n          </span>\n        </div>"));
}

#[test]
fn test_table_pipeline_integration() {
    let input = "---\ntitle: \"Table Pipeline Test\"\n---\n# Table Section\n\n| Item | Qty | Status |\n| :--- | :---: | ---: |\n| **Widget A** | 42 | In Stock |\n| `Widget B` | 0 | *Out of Stock* |";
    let document = doc2flow::core::parse_d2f_markdown(input).expect("parse failed");
    let content = doc2flow::core::builder::build(&document);
    assert!(content.contains("<title>Table Pipeline Test</title>"));
    assert!(content.contains("<meta name=\"features\" content=\"core, table\">"));
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
    assert!(content.contains("<meta name=\"features\" content=\"core, code\">"));
    assert!(content.contains("class=\"code-table-wrap\""));
    assert!(content.contains("class=\"code-table-default\""));
    assert!(content.contains("<td>AUTH_TOKEN</td>"));
    assert!(content.contains("<td>PORT</td>"));
    assert!(content.contains("<td>TARGET_HOST</td>"));
    assert!(content.contains("<input type=\"text\" class=\"code-table-input\" value=\"\">"));
    assert!(content.contains("curl -H &quot;Authorization: Bearer {{AUTH_TOKEN}}&quot; https://{{TARGET_HOST}}:{{PORT}}/api"));
}

#[test]
fn test_code_variable_pipeline_with_variables_directive() {
    let input = "---\ntitle: \"Code Vars Combined Test\"\n---\n:::variables\n| Variable | Value |\n| --- | --- |\n| TARGET_HOST | 192.168.1.50 |\n| PORT | 8080 |\n:::\n# Code Section\n\n```bash\ncurl -H \"Authorization: Bearer {{AUTH_TOKEN}}\" https://{{TARGET_HOST}}:{{PORT}}/api\n```";
    let document = doc2flow::core::parse_d2f_markdown(input).expect("parse failed");
    let content = doc2flow::core::builder::build(&document);
    assert!(content.contains("<title>Code Vars Combined Test</title>"));
    assert!(content.contains("<meta name=\"features\" content=\"core, code\">"));
    assert!(content.contains("<td>TARGET_HOST</td>\n            <td><input type=\"text\" class=\"code-table-input\" value=\"192.168.1.50\"></td>"));
    assert!(content.contains("<td>PORT</td>\n            <td><input type=\"text\" class=\"code-table-input\" value=\"8080\"></td>"));
    assert!(content.contains("<td>AUTH_TOKEN</td>\n            <td><input type=\"text\" class=\"code-table-input\" value=\"\"></td>"));
}

#[test]
fn test_image_and_link_pipeline_integration() {
    let input = "---\ntitle: \"Image and Link Test\"\n---\n# Media Section\n\n![Sample Image](images/example1.jpg)\n\nCheck the [Documentation](https://example.com/docs) for details.";
    let document = doc2flow::core::parse_d2f_markdown(input).expect("parse failed");
    let content = doc2flow::core::builder::build(&document);
    assert!(content.contains("<title>Image and Link Test</title>"));
    assert!(content.contains("<meta name=\"features\" content=\"core, image\">"));
    assert!(content.contains("<div class=\"image-item\">\n          <img src=\"images/example1.jpg\" alt=\"Sample Image\" />\n        </div>"));
    assert!(content.contains("<a href=\"https://example.com/docs\">Documentation</a>"));
}
