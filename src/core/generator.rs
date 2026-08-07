//! HTML Assembler and template engine integrating vertical slices and core assets.

use crate::core::feature::{DocumentContext, Feature};
use crate::error::{Doc2FlowError, Result};
use crate::locales::{Locale, validate_locale_coverage};
use std::collections::HashMap;

/// Embedded core CSS styles for layout and components.
pub static STYLE_CORE: &str = include_str!("css/core.css");

/// Embedded core JavaScript bundle for client runtime.
pub static SCRIPT_CORE: &str = include_str!("../../web/dist/script-core.js");

/// Assembles active CSS feature styles into the output buffer based on document context.
pub fn assemble_styles(ctx: &DocumentContext, features: &[Box<dyn Feature>], out: &mut String) {
    out.push_str(STYLE_CORE);
    out.push('\n');

    for feature in features {
        if feature.is_enabled(ctx) {
            if let Some(css) = feature.css() {
                out.push_str(css);
                out.push('\n');
            }
        }
    }
}

/// Assembles active JavaScript feature bundles into the output buffer based on document context.
pub fn assemble_scripts(ctx: &DocumentContext, features: &[Box<dyn Feature>], out: &mut String) {
    out.push_str(SCRIPT_CORE);
    out.push('\n');

    for feature in features {
        if feature.is_enabled(ctx) {
            if let Some(js) = feature.javascript() {
                out.push_str(js);
                out.push('\n');
            }
        }
    }
}

/// Assembles the complete self-contained HTML document with enabled features.
///
/// Iterates over all registered features. When `feature.is_enabled(ctx)` is true,
/// appends only the CSS and JS of that feature to the final HTML document.
///
/// # Errors
///
/// Returns an error if the locale configuration cannot be serialized.
pub fn assemble_html(
    ctx: &DocumentContext,
    features: &[Box<dyn Feature>],
    locale: &Locale,
    html_content: &str,
    doc_id: &str,
    logo_html: Option<&str>,
) -> Result<String> {
    let base_html = include_str!("../../templates/base.html");

    let mut style_css = String::with_capacity(32768);
    assemble_styles(ctx, features, &mut style_css);

    let mut script_js = String::with_capacity(32768);
    assemble_scripts(ctx, features, &mut script_js);

    validate_locale_coverage(base_html, locale);

    let i18n_json =
        serde_json::to_string(&locale.entries).map_err(|e| Doc2FlowError::Json(e.to_string()))?;

    let logo = logo_html
        .filter(|s| !s.is_empty())
        .unwrap_or(crate::template::DEFAULT_LOGO_SVG);

    let app_version = crate::template::APP_VERSION;
    let app_version_raw = app_version.strip_prefix('v').unwrap_or(app_version);
    let created_at = crate::template::format_iso8601_utc(std::time::SystemTime::now());

    let mut active_features_str = String::with_capacity(64);
    active_features_str.push_str("core");
    for feature in features {
        if feature.is_enabled(ctx) {
            active_features_str.push_str(", ");
            active_features_str.push_str(feature.name());
        }
    }

    let title = ctx
        .frontmatter
        .get("title")
        .map(String::as_str)
        .unwrap_or("");
    let subtitle = ctx
        .frontmatter
        .get("subtitle")
        .map(String::as_str)
        .unwrap_or("");
    let date = ctx
        .frontmatter
        .get("date")
        .map(String::as_str)
        .unwrap_or("");

    let mut vars = HashMap::with_capacity(24);
    vars.insert("APP_VERSION", app_version);
    vars.insert("APP_VERSION_RAW", app_version_raw);
    vars.insert("REPOSITORY_URL", crate::template::REPOSITORY_URL);
    vars.insert("LICENSE_TERMS", crate::template::LICENSE_TERMS);
    vars.insert("LICENSE_URL", crate::template::LICENSE_URL);
    vars.insert("CREATED_AT", created_at.as_str());
    vars.insert("LANG_CODE", locale.lang_code.as_str());
    vars.insert("TITLE", title);
    vars.insert("SUBTITLE", subtitle);
    vars.insert("DATE", date);
    vars.insert("I18N_JSON", i18n_json.as_str());
    vars.insert("CSS", style_css.as_str());
    vars.insert("JS", script_js.as_str());
    vars.insert("LIGHTBOX_HTML", "");
    vars.insert("PROGRESS_BAR_HTML", "");
    vars.insert("FINISH_BOX_HTML", "");
    vars.insert("CONTENT", html_content);
    vars.insert("DOC_ID", doc_id);
    vars.insert("LOGO", logo);
    vars.insert("FEATURES", active_features_str.as_str());
    vars.insert("features", active_features_str.as_str());

    Ok(crate::template::substitute_template(
        base_html,
        &vars,
        Some(locale),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::features::code::CodeFeature;
    use std::collections::HashMap;

    #[test]
    fn test_assemble_feature_styles_and_scripts() {
        let feature_list: Vec<Box<dyn Feature>> = vec![Box::new(CodeFeature::new())];

        // 1. Context with code blocks: feature is enabled
        let fm = HashMap::new();
        let ctx_with_code = DocumentContext::new(&fm, "```rust\nfn main() {}\n```");

        let mut styles = String::new();
        assemble_styles(&ctx_with_code, &feature_list, &mut styles);
        assert!(styles.contains(".code-block"));
        assert!(styles.contains(".copy-btn"));

        let mut scripts = String::new();
        assemble_scripts(&ctx_with_code, &feature_list, &mut scripts);
        assert!(scripts.contains("copyCode") || scripts.contains("copy"));

        // 2. Context without code blocks: feature is disabled
        let ctx_disabled = DocumentContext::new(&fm, "# Plain documentation\nNo code blocks.");

        let mut styles_disabled = String::new();
        assemble_styles(&ctx_disabled, &feature_list, &mut styles_disabled);
        assert!(!styles_disabled.contains(".copy-btn"));
    }

    #[test]
    fn test_assemble_html_features_meta_tag() {
        let feature_list: Vec<Box<dyn Feature>> = vec![Box::new(CodeFeature::new())];
        let locale = Locale::default();
        let fm = HashMap::new();

        // 1. When code feature is active: meta features must contain "core, code"
        let ctx_with_code = DocumentContext::new(&fm, "```rust\nfn main() {}\n```");
        let html_with_code = assemble_html(
            &ctx_with_code,
            &feature_list,
            &locale,
            "<p>Content</p>",
            "doc_123",
            None,
        )
        .expect("assemble_html must succeed");

        assert!(
            html_with_code.contains(r#"<meta name="features" content="core, code">"#),
            "HTML meta features tag must contain 'core, code' when code block is present"
        );

        // 2. When code feature is not active: meta features must contain "core"
        let ctx_without_code = DocumentContext::new(&fm, "# Heading\nPlain text content.");
        let html_without_code = assemble_html(
            &ctx_without_code,
            &feature_list,
            &locale,
            "<p>Content</p>",
            "doc_123",
            None,
        )
        .expect("assemble_html must succeed");

        assert!(
            html_without_code.contains(r#"<meta name="features" content="core">"#),
            "HTML meta features tag must contain 'core' when no features are active"
        );
    }
}
