//! HTML rendering and template substitution module for Doc2Flow documents.

use crate::converter::Frontmatter;
use crate::i18n::Locale;
use anyhow::{Context, Result};
use std::collections::HashMap;

/// Performs single-pass template placeholder substitution using standard library tools.
///
/// Replaces placeholders formatted as `{{KEY}}` in the template string with corresponding
/// values provided in the `vars` map. Unknown placeholders are left untouched in the output.
///
/// # Examples
///
/// ```
/// use std::collections::HashMap;
/// use doc2flow::template::substitute_template;
///
/// let mut vars = HashMap::new();
/// vars.insert("NAME", "World");
/// let result = substitute_template("Hello {{NAME}}!", &vars);
/// assert_eq!(result, "Hello World!");
/// ```
pub fn substitute_template(template: &str, vars: &HashMap<&str, &str>) -> String {
    let total_values_len: usize = vars.values().map(|v| v.len()).sum();
    let mut result = String::with_capacity(template.len() + total_values_len);

    let mut cursor = 0;
    while let Some(start) = template[cursor..].find("{{") {
        let abs_start = cursor + start;
        result.push_str(&template[cursor..abs_start]);

        if let Some(end) = template[abs_start + 2..].find("}}") {
            let abs_end = abs_start + 2 + end;
            let key = &template[abs_start + 2..abs_end];

            if let Some(val) = vars.get(key) {
                result.push_str(val);
            } else {
                result.push_str(&template[abs_start..abs_end + 2]);
            }
            cursor = abs_end + 2;
        } else {
            cursor = abs_start;
            break;
        }
    }
    result.push_str(&template[cursor..]);
    result
}

/// Renders a complete self-contained HTML document.
///
/// Combines frontmatter metadata, locale configuration, rendered markdown content,
/// embedded CSS/JS resources, and base HTML template into a single HTML string.
///
/// # Errors
///
/// Returns an error if the locale cannot be serialized to JSON.
pub fn render(
    frontmatter: &Frontmatter,
    locale: &Locale,
    html_content: &str,
    doc_id: &str,
) -> Result<String> {
    let base_html = include_str!("../templates/base.html");
    let style_css = include_str!("../templates/style.css");
    let script_js = include_str!("../templates/script.js");

    let i18n_json = serde_json::to_string(locale)
        .context("Failed to serialize locale to JSON for template rendering")?;

    let mut vars = HashMap::with_capacity(24);
    vars.insert("LANG_CODE", locale.lang_code.as_str());
    vars.insert("TITLE", frontmatter.title.as_str());
    vars.insert("SUBTITLE", frontmatter.subtitle.as_str());
    vars.insert("CUSTOMER", frontmatter.customer.as_str());
    vars.insert("EMPLOYEE", frontmatter.employee.as_str());
    vars.insert("TECHNICIAN", frontmatter.technician.as_str());
    vars.insert("DATE", frontmatter.date.as_str());
    vars.insert("L_CUSTOMER", locale.customer.as_str());
    vars.insert("L_EMPLOYEE", locale.employee.as_str());
    vars.insert("L_TECHNICIAN", locale.technician.as_str());
    vars.insert("L_DATE", locale.date.as_str());
    vars.insert("L_SETUP_COMPLETED", locale.setup_completed.as_str());
    vars.insert("L_NAME_PLACEHOLDER", locale.name_placeholder.as_str());
    vars.insert(
        "L_SIGNATURE_TECHNICIAN",
        locale.signature_technician.as_str(),
    );
    vars.insert("L_DATE_PLACEHOLDER", locale.date_placeholder.as_str());
    vars.insert("L_SIGNATURE_DATE", locale.signature_date.as_str());
    vars.insert("L_EXPORT_PDF", locale.export_pdf.as_str());
    vars.insert("L_RESET_ALL", locale.reset_all.as_str());
    vars.insert("L_LOADING", locale.loading.as_str());
    vars.insert("I18N_JSON", i18n_json.as_str());
    vars.insert("CSS", style_css);
    vars.insert("JS", script_js);
    vars.insert("CONTENT", html_content);
    vars.insert("DOC_ID", doc_id);

    Ok(substitute_template(base_html, &vars))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_substitute_template_basic() {
        let mut vars = HashMap::new();
        vars.insert("TITLE", "Test Title");
        vars.insert("AUTHOR", "Alice");

        let tmpl = "<h1>{{TITLE}}</h1><p>By {{AUTHOR}}</p>";
        let res = substitute_template(tmpl, &vars);
        assert_eq!(res, "<h1>Test Title</h1><p>By Alice</p>");
    }

    #[test]
    fn test_substitute_template_unknown_and_unclosed() {
        let mut vars = HashMap::new();
        vars.insert("KNOWN", "Value");

        let tmpl = "{{KNOWN}} - {{UNKNOWN}} - {{UNCLOSED";
        let res = substitute_template(tmpl, &vars);
        assert_eq!(res, "Value - {{UNKNOWN}} - {{UNCLOSED");
    }

    #[test]
    fn test_substitute_template_no_placeholders() {
        let vars = HashMap::new();
        let tmpl = "Plain string with no tags.";
        let res = substitute_template(tmpl, &vars);
        assert_eq!(res, tmpl);
    }

    #[test]
    fn test_render_full_document() {
        let fm = Frontmatter {
            title: "Doc Title".into(),
            language: "de".into(),
            ..Frontmatter::default()
        };

        let locale = Locale::german();
        let body = "<p>Body Content</p>";
        let doc_id = "test_id_99";

        let html = render(&fm, &locale, body, doc_id).expect("Render failed");
        assert!(html.contains("lang=\"de\""));
        assert!(html.contains("Doc Title"));
        assert!(html.contains("<p>Body Content</p>"));
        assert!(html.contains("test_id_99"));
        assert!(!html.contains("{{TITLE}}"));
        assert!(!html.contains("{{CONTENT}}"));
    }
}
