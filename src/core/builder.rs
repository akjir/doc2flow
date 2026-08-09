//! HTML Builder and asset orchestrator integrating vertical slices and core assets.

pub use crate::components::DEFAULT_LOGO_SVG;
pub use crate::constants::{APP_VERSION, LICENSE_TERMS, LICENSE_URL, REPOSITORY_URL};
use crate::converter::{DocumentFeatures, Frontmatter};
use crate::utils::error::{Doc2FlowError, Result};
use crate::feature::{DocumentContext, Feature};
use crate::features;
use crate::locales::{Locale, validate_locale_coverage};
use std::collections::HashMap;
use std::fmt::Write;

/// Embedded core CSS styles for layout and components.
pub static STYLE_CORE: &str = include_str!("web/dist/core.css");

/// Embedded core JavaScript bundle for client runtime.
pub static SCRIPT_CORE: &str = include_str!("web/dist/core.js");

/// Assembles active CSS feature styles into the output buffer based on document context.
///
/// Iterates over all registered features. When a feature is enabled and provides a stylesheet,
/// its CSS is appended to the output buffer. Features returning `None` are skipped.
pub fn assemble_styles(ctx: &DocumentContext, features: &[Box<dyn Feature>], out: &mut String) {
    out.push_str(STYLE_CORE);
    out.push('\n');

    let enabled = crate::core::feature::resolve_enabled_features(features, ctx);
    for feature in features {
        if enabled.contains(feature.name())
            && let Some(css) = feature.css()
        {
            out.push_str(css);
            out.push('\n');
        }
    }
}

/// Assembles active JavaScript feature bundles into the output buffer based on document context.
///
/// Iterates over all registered features. When a feature is enabled and provides a script bundle,
/// its JS is appended to the output buffer. Features returning `None` are skipped.
pub fn assemble_scripts(ctx: &DocumentContext, features: &[Box<dyn Feature>], out: &mut String) {
    out.push_str(SCRIPT_CORE);
    out.push('\n');

    let enabled = crate::core::feature::resolve_enabled_features(features, ctx);
    for feature in features {
        if enabled.contains(feature.name())
            && let Some(js) = feature.javascript()
        {
            out.push_str(js);
            out.push('\n');
        }
    }
}

/// Assembles active CSS feature styles into the output buffer based on detected document features.
pub fn render_styles(out: &mut String, features: &DocumentFeatures) {
    out.push_str(STYLE_CORE);
    out.push('\n');

    let all_features = features::get_all_features();
    for feat in &all_features {
        if features.is_feature_active(feat.name())
            && let Some(css) = feat.css()
        {
            out.push_str(css);
            out.push('\n');
        }
    }
}

/// Assembles active JavaScript feature bundles into the output buffer based on detected document features.
pub fn render_scripts(out: &mut String, features: &DocumentFeatures) {
    out.push_str(SCRIPT_CORE);
    out.push('\n');

    let all_features = features::get_all_features();
    for feat in &all_features {
        if features.is_feature_active(feat.name())
            && let Some(js) = feat.javascript()
        {
            out.push_str(js);
            out.push('\n');
        }
    }
}

/// Renders the image lightbox modal markup if the document contains images.
pub fn render_lightbox(out: &mut impl Write, features: &DocumentFeatures) {
    features::image::render_lightbox(out, features.has_images);
}

/// Renders the top process progress bar component if the document contains tasks.
pub fn render_progress_bar(out: &mut impl Write, features: &DocumentFeatures, locale: &Locale) {
    let loading = locale.get_ignore_ascii_case("LOADING").unwrap_or("");
    features::tasks::render_progress_bar(out, features.has_tasks, loading);
}

/// Renders the bottom finish box component if the document contains tasks.
pub fn render_finish_box(out: &mut impl Write, features: &DocumentFeatures, locale: &Locale) {
    let setup_completed = locale.get_ignore_ascii_case("SETUP_COMPLETED").unwrap_or("");
    let name_placeholder = locale.get_ignore_ascii_case("NAME_PLACEHOLDER").unwrap_or("");
    let date_placeholder = locale.get_ignore_ascii_case("DATE_PLACEHOLDER").unwrap_or("");
    let signature_date = locale.get_ignore_ascii_case("SIGNATURE_DATE").unwrap_or("");

    features::tasks::render_finish_box(
        out,
        features.has_tasks,
        setup_completed,
        name_placeholder,
        date_placeholder,
        signature_date,
    );
}

/// Formats a [`std::time::SystemTime`] as an ISO 8601 UTC timestamp (`YYYY-MM-DDTHH:MM:SSZ`).
///
/// Implements the zero-allocation Euclidean civil calendar algorithm (Hinnant/Neri-Schneider)
/// converting elapsed seconds since the Unix epoch into UTC year, month, day, hour, minute, and second.
/// If `time` precedes the Unix epoch, it safely falls back to `"1970-01-01T00:00:00Z"`.
///
/// # Examples
///
/// ```
/// use std::time::{Duration, UNIX_EPOCH};
/// use doc2flow::builder::format_iso8601_utc;
///
/// let epoch = UNIX_EPOCH;
/// assert_eq!(format_iso8601_utc(epoch), "1970-01-01T00:00:00Z");
///
/// let timestamp = UNIX_EPOCH + Duration::from_secs(1_700_000_000);
/// assert_eq!(format_iso8601_utc(timestamp), "2023-11-14T22:13:20Z");
/// ```
pub fn format_iso8601_utc(time: std::time::SystemTime) -> String {
    let dur = time
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default();
    let secs = dur.as_secs();

    let sec = secs % 60;
    let mins = secs / 60;
    let min = mins % 60;
    let hours = mins / 60;
    let hour = hours % 24;
    let days = hours / 24;

    let z = days as i64 + 719468;
    let era = (if z >= 0 { z } else { z - 146096 }) / 146097;
    let doe = (z - era * 146097) as u64;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let y = yoe as i64 + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let year = y + if m <= 2 { 1 } else { 0 };

    let mut out = String::with_capacity(20);
    let _ = write!(out, "{year:04}-{m:02}-{d:02}T{hour:02}:{min:02}:{sec:02}Z");
    out
}

/// Returns the pre-populated default starter Markdown template string.
///
/// Contains frontmatter metadata fields, HTML comments with usage instructions,
/// and a showcase document structure.
///
/// # Examples
///
/// ```
/// use doc2flow::builder::generate_template_markdown;
///
/// let template = generate_template_markdown();
/// assert!(template.contains("title:"));
/// assert!(template.contains("## Section 1: Initial System Verification"));
/// ```
pub fn generate_template_markdown() -> String {
    let raw = include_str!("../../resources/templates/template.md");
    raw.replace("{{APP_VERSION}}", APP_VERSION)
}

/// Context parameters required for single-pass HTML template variable substitution.
#[derive(Debug, Clone, Copy)]
pub struct TemplateContext<'a> {
    /// Application version with SemVer 2.0.0 metadata.
    pub app_version: &'a str,
    /// Application version without 'v' prefix.
    pub app_version_raw: &'a str,
    /// Official application repository URL.
    pub repository_url: &'a str,
    /// Application license terms.
    pub license_terms: &'a str,
    /// Official application license URL.
    pub license_url: &'a str,
    /// ISO 8601 UTC document generation timestamp.
    pub created_at: &'a str,
    /// Two-letter ISO 639-1 language code.
    pub lang_code: &'a str,
    /// Document title from frontmatter.
    pub title: &'a str,
    /// Optional document subtitle from frontmatter.
    pub subtitle: &'a str,
    /// Optional document date string.
    pub date: &'a str,
    /// Serialized JSON string of internationalized UI dictionary entries.
    pub i18n_json: &'a str,
    /// Combined core and active feature CSS stylesheets.
    pub css: &'a str,
    /// Combined core and active feature JavaScript client scripts.
    pub js: &'a str,
    /// Rendered image lightbox modal HTML, or empty string.
    pub lightbox_html: &'a str,
    /// Rendered progress bar component HTML, or empty string.
    pub progress_bar_html: &'a str,
    /// Rendered sign-off finish box component HTML, or empty string.
    pub finish_box_html: &'a str,
    /// Complete HTML body content.
    pub content: &'a str,
    /// Deterministic document unique identifier.
    pub doc_id: &'a str,
    /// Rendered SVG or base64 image logo HTML.
    pub logo: &'a str,
    /// Comma-separated list of enabled feature identifiers (e.g., `"core, code, tasks"`).
    pub features: &'a str,
}

/// Populates the template substitution variable map from the provided template context.
pub fn build_template_vars<'a>(ctx: &TemplateContext<'a>) -> HashMap<&'static str, &'a str> {
    let mut vars = HashMap::with_capacity(20);
    vars.insert("APP_VERSION", ctx.app_version);
    vars.insert("APP_VERSION_RAW", ctx.app_version_raw);
    vars.insert("REPOSITORY_URL", ctx.repository_url);
    vars.insert("LICENSE_TERMS", ctx.license_terms);
    vars.insert("LICENSE_URL", ctx.license_url);
    vars.insert("CREATED_AT", ctx.created_at);
    vars.insert("LANG_CODE", ctx.lang_code);
    vars.insert("TITLE", ctx.title);
    vars.insert("SUBTITLE", ctx.subtitle);
    vars.insert("DATE", ctx.date);
    vars.insert("I18N_JSON", ctx.i18n_json);
    vars.insert("CSS", ctx.css);
    vars.insert("JS", ctx.js);
    vars.insert("LIGHTBOX_HTML", ctx.lightbox_html);
    vars.insert("PROGRESS_BAR_HTML", ctx.progress_bar_html);
    vars.insert("FINISH_BOX_HTML", ctx.finish_box_html);
    vars.insert("CONTENT", ctx.content);
    vars.insert("DOC_ID", ctx.doc_id);
    vars.insert("LOGO", ctx.logo);
    vars.insert("FEATURES", ctx.features);
    vars
}

/// Performs single-pass template placeholder substitution using standard library tools.
///
/// Replaces placeholders formatted as `{{KEY}}` in the template string with corresponding
/// values provided in `vars` or `locale`. Unknown placeholders are left untouched in output.
///
/// # Examples
///
/// ```
/// use std::collections::HashMap;
/// use doc2flow::locales::Locale;
/// use doc2flow::builder::substitute_template;
///
/// let mut vars = HashMap::new();
/// vars.insert("NAME", "World");
/// let result = substitute_template("Hello {{NAME}}!", &vars, None);
/// assert_eq!(result, "Hello World!");
/// ```
pub fn substitute_template(
    template: &str,
    vars: &HashMap<&str, &str>,
    locale: Option<&Locale>,
) -> String {
    let total_vars_len: usize = vars.values().map(|v| v.len()).sum();
    let mut result = String::with_capacity(template.len() + total_vars_len);

    let mut cursor = 0;
    while let Some(start) = template[cursor..].find("{{") {
        let abs_start = cursor + start;
        result.push_str(&template[cursor..abs_start]);
        if let Some(end) = template[abs_start + 2..].find("}}") {
            let abs_end = abs_start + 2 + end;
            let key = &template[abs_start + 2..abs_end];

            let val_opt = vars.get(key).copied().or_else(|| {
                key.strip_prefix("L_")
                    .and_then(|key_name| locale.and_then(|loc| loc.get_ignore_ascii_case(key_name)))
            });
            match val_opt {
                Some(val) => result.push_str(val),
                None => result.push_str(&template[abs_start..abs_end + 2]),
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

/// Builds and assembles a complete self-contained HTML document.
///
/// Combines frontmatter metadata, locale configuration, rendered markdown content,
/// embedded CSS/JS resources from core and enabled features, and base HTML template into a single HTML string.
///
/// # Errors
///
/// Returns an error if the locale entries cannot be serialized to JSON.
pub fn render(
    frontmatter: &Frontmatter,
    locale: &Locale,
    html_content: &str,
    doc_id: &str,
    logo_html: Option<&str>,
    features: &DocumentFeatures,
) -> Result<String> {
    let base_html = include_str!("../../resources/templates/base.html");

    let mut style_css = String::with_capacity(STYLE_CORE.len() + 4096);
    render_styles(&mut style_css, features);

    let mut script_js = String::with_capacity(SCRIPT_CORE.len() + 4096);
    render_scripts(&mut script_js, features);

    let mut lightbox_html = String::with_capacity(256);
    render_lightbox(&mut lightbox_html, features);

    let mut progress_bar_html = String::with_capacity(256);
    render_progress_bar(&mut progress_bar_html, features, locale);

    let mut finish_box_html = String::with_capacity(512);
    render_finish_box(&mut finish_box_html, features, locale);

    validate_locale_coverage(base_html, locale);

    let i18n_json =
        serde_json::to_string(&locale.entries).map_err(|e| Doc2FlowError::Json(e.to_string()))?;

    let logo = logo_html
        .filter(|s| !s.is_empty())
        .unwrap_or(DEFAULT_LOGO_SVG);

    let app_version_raw = APP_VERSION.strip_prefix('v').unwrap_or(APP_VERSION);
    let created_at = format_iso8601_utc(std::time::SystemTime::now());

    let features_str = features.to_features_string();

    let mut full_content = String::with_capacity(html_content.len() + 512);
    if features.has_header {
        features::header::render_flex_header(
            &mut full_content,
            frontmatter.title.as_deref().unwrap_or(""),
            frontmatter.subtitle.as_deref(),
            logo,
        );
    }
    full_content.push_str(html_content);

    let template_ctx = TemplateContext {
        app_version: APP_VERSION,
        app_version_raw,
        repository_url: REPOSITORY_URL,
        license_terms: LICENSE_TERMS,
        license_url: LICENSE_URL,
        created_at: created_at.as_str(),
        lang_code: locale.lang_code.as_str(),
        title: frontmatter.title.as_deref().unwrap_or(""),
        subtitle: frontmatter.subtitle.as_deref().unwrap_or(""),
        date: frontmatter.date.as_deref().unwrap_or(""),
        i18n_json: i18n_json.as_str(),
        css: style_css.as_str(),
        js: script_js.as_str(),
        lightbox_html: lightbox_html.as_str(),
        progress_bar_html: progress_bar_html.as_str(),
        finish_box_html: finish_box_html.as_str(),
        content: full_content.as_str(),
        doc_id,
        logo,
        features: features_str.as_str(),
    };

    let vars = build_template_vars(&template_ctx);
    Ok(substitute_template(base_html, &vars, Some(locale)))
}

/// Assembles the complete self-contained HTML document with enabled features.
///
/// Iterates over all registered features. When a feature is enabled, dynamically renders its
/// styles, scripts, and UI components into the final HTML document.
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
    let base_html = include_str!("../../resources/templates/base.html");

    let mut style_css = String::with_capacity(STYLE_CORE.len() + 4096);
    assemble_styles(ctx, features, &mut style_css);

    let mut script_js = String::with_capacity(SCRIPT_CORE.len() + 4096);
    assemble_scripts(ctx, features, &mut script_js);

    let mut has_images = false;
    let mut has_tasks = false;
    let mut has_header = false;
    let mut active_features_str = String::with_capacity(64);
    active_features_str.push_str("core");

    let enabled = crate::core::feature::resolve_enabled_features(features, ctx);
    for feature in features {
        let name = feature.name();
        if enabled.contains(name) {
            active_features_str.push_str(", ");
            active_features_str.push_str(name);
            match name {
                "images" => has_images = true,
                "tasks" => has_tasks = true,
                "header" => has_header = true,
                _ => {}
            }
        }
    }

    let mut lightbox_html = String::with_capacity(256);
    features::image::render_lightbox(&mut lightbox_html, has_images);

    let mut progress_bar_html = String::with_capacity(256);
    let loading = locale.get_ignore_ascii_case("LOADING").unwrap_or("");
    features::tasks::render_progress_bar(&mut progress_bar_html, has_tasks, loading);

    let mut finish_box_html = String::with_capacity(512);
    let setup_completed = locale.get_ignore_ascii_case("SETUP_COMPLETED").unwrap_or("");
    let name_placeholder = locale.get_ignore_ascii_case("NAME_PLACEHOLDER").unwrap_or("");
    let date_placeholder = locale.get_ignore_ascii_case("DATE_PLACEHOLDER").unwrap_or("");
    let signature_date = locale.get_ignore_ascii_case("SIGNATURE_DATE").unwrap_or("");
    features::tasks::render_finish_box(
        &mut finish_box_html,
        has_tasks,
        setup_completed,
        name_placeholder,
        date_placeholder,
        signature_date,
    );

    validate_locale_coverage(base_html, locale);

    let i18n_json =
        serde_json::to_string(&locale.entries).map_err(|e| Doc2FlowError::Json(e.to_string()))?;

    let logo = logo_html
        .filter(|s| !s.is_empty())
        .unwrap_or(DEFAULT_LOGO_SVG);

    let app_version = APP_VERSION;
    let app_version_raw = app_version.strip_prefix('v').unwrap_or(app_version);
    let created_at = format_iso8601_utc(std::time::SystemTime::now());

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

    let mut full_content = String::with_capacity(html_content.len() + 512);
    if has_header {
        features::header::render_flex_header(
            &mut full_content,
            title,
            if subtitle.is_empty() { None } else { Some(subtitle) },
            logo,
        );
    }
    full_content.push_str(html_content);

    let template_ctx = TemplateContext {
        app_version,
        app_version_raw,
        repository_url: REPOSITORY_URL,
        license_terms: LICENSE_TERMS,
        license_url: LICENSE_URL,
        created_at: created_at.as_str(),
        lang_code: locale.lang_code.as_str(),
        title,
        subtitle,
        date,
        i18n_json: i18n_json.as_str(),
        css: style_css.as_str(),
        js: script_js.as_str(),
        lightbox_html: lightbox_html.as_str(),
        progress_bar_html: progress_bar_html.as_str(),
        finish_box_html: finish_box_html.as_str(),
        content: full_content.as_str(),
        doc_id,
        logo,
        features: active_features_str.as_str(),
    };

    let vars = build_template_vars(&template_ctx);
    Ok(substitute_template(base_html, &vars, Some(locale)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::features::code::CodeFeature;
    use crate::features::header::HeaderFeature;
    use crate::features::image::ImageFeature;
    use crate::features::table::TableFeature;
    use crate::features::tasks::TasksFeature;

    #[test]
    fn test_generate_template_markdown_contains_required_sections() {
        let content = generate_template_markdown();
        assert!(content.contains("title:"));
        assert!(content.contains("subtitle:"));
        assert!(content.contains("date:"));
        assert!(content.contains("version:"));
        assert!(content.contains("language:"));
        assert!(content.contains("logo:"));
        assert!(content.contains("## Section 1: Initial System Verification"));
        assert!(content.contains("### Prerequisites Checklist"));
        assert!(content.contains("<!--"));
        assert!(content.contains("-->"));
        assert!(content.contains("> Note:"));
        assert!(content.contains(">? Tip:"));
        assert!(content.contains(">! Important:"));
        assert!(content.contains(">!! Warning:"));
        assert!(content.contains(">!!! Caution:"));
        assert!(content.contains(APP_VERSION));
        assert!(content.contains(REPOSITORY_URL));
        assert!(content.contains(LICENSE_TERMS));
    }

    #[test]
    fn test_substitute_template_basic() {
        let mut vars = HashMap::new();
        vars.insert("TITLE", "Test Title");
        vars.insert("AUTHOR", "Alice");

        let tmpl = "<h1>{{TITLE}}</h1><p>By {{AUTHOR}}</p>";
        let res = substitute_template(tmpl, &vars, None);
        assert_eq!(res, "<h1>Test Title</h1><p>By Alice</p>");
    }

    #[test]
    fn test_substitute_template_unknown_and_unclosed() {
        let mut vars = HashMap::new();
        vars.insert("KNOWN", "Value");

        let tmpl = "{{KNOWN}} - {{UNKNOWN}} - {{UNCLOSED";
        let res = substitute_template(tmpl, &vars, None);
        assert_eq!(res, "Value - {{UNKNOWN}} - {{UNCLOSED");
    }

    #[test]
    fn test_substitute_template_no_placeholders() {
        let vars = HashMap::new();
        let tmpl = "Plain string with no tags.";
        let res = substitute_template(tmpl, &vars, None);
        assert_eq!(res, tmpl);
    }

    #[test]
    fn test_build_template_vars_completeness() {
        let ctx = TemplateContext {
            app_version: "v1.0.0",
            app_version_raw: "1.0.0",
            repository_url: "https://example.com/repo",
            license_terms: "MIT",
            license_url: "https://example.com/license",
            created_at: "2026-08-09T00:00:00Z",
            lang_code: "en",
            title: "My Doc",
            subtitle: "Sub Doc",
            date: "2026-08-09",
            i18n_json: "{}",
            css: "body {}",
            js: "console.log();",
            lightbox_html: "<div>lb</div>",
            progress_bar_html: "<div>pb</div>",
            finish_box_html: "<div>fb</div>",
            content: "<p>Hello</p>",
            doc_id: "doc123",
            logo: "<svg></svg>",
            features: "core, code",
        };

        let vars = build_template_vars(&ctx);
        assert_eq!(vars.get("APP_VERSION"), Some(&"v1.0.0"));
        assert_eq!(vars.get("APP_VERSION_RAW"), Some(&"1.0.0"));
        assert_eq!(vars.get("REPOSITORY_URL"), Some(&"https://example.com/repo"));
        assert_eq!(vars.get("LICENSE_TERMS"), Some(&"MIT"));
        assert_eq!(vars.get("LICENSE_URL"), Some(&"https://example.com/license"));
        assert_eq!(vars.get("CREATED_AT"), Some(&"2026-08-09T00:00:00Z"));
        assert_eq!(vars.get("LANG_CODE"), Some(&"en"));
        assert_eq!(vars.get("TITLE"), Some(&"My Doc"));
        assert_eq!(vars.get("SUBTITLE"), Some(&"Sub Doc"));
        assert_eq!(vars.get("DATE"), Some(&"2026-08-09"));
        assert_eq!(vars.get("I18N_JSON"), Some(&"{}"));
        assert_eq!(vars.get("CSS"), Some(&"body {}"));
        assert_eq!(vars.get("JS"), Some(&"console.log();"));
        assert_eq!(vars.get("LIGHTBOX_HTML"), Some(&"<div>lb</div>"));
        assert_eq!(vars.get("PROGRESS_BAR_HTML"), Some(&"<div>pb</div>"));
        assert_eq!(vars.get("FINISH_BOX_HTML"), Some(&"<div>fb</div>"));
        assert_eq!(vars.get("CONTENT"), Some(&"<p>Hello</p>"));
        assert_eq!(vars.get("DOC_ID"), Some(&"doc123"));
        assert_eq!(vars.get("LOGO"), Some(&"<svg></svg>"));
        assert_eq!(vars.get("FEATURES"), Some(&"core, code"));
        assert_eq!(vars.get("features"), None);
    }

    #[test]
    fn test_document_features_matching_parity() {
        let mut df = DocumentFeatures::default();
        assert!(!df.is_feature_active("code"));
        assert!(!df.is_feature_active("tasks"));
        assert!(!df.is_feature_active("images"));
        assert!(!df.is_feature_active("tables"));
        assert!(!df.is_feature_active("header"));
        assert!(!df.is_feature_active("unknown_feature"));

        df.has_code = true;
        assert!(df.is_feature_active("code"));

        df.has_tasks = true;
        assert!(df.is_feature_active("tasks"));

        df.has_images = true;
        assert!(df.is_feature_active("images"));

        df.has_tables = true;
        assert!(df.is_feature_active("tables"));

        df.has_header = true;
        assert!(df.is_feature_active("header"));
    }

    #[test]
    fn test_render_full_document() {
        let mut fm = Frontmatter::new();
        fm.title = Some("Doc Title".into());
        fm.language = Some("de".into());

        let locale = Locale::from_lang_code("de");
        let body = "<p>Body Content</p>";
        let doc_id = "test_id_99";
        let features = DocumentFeatures::default();

        let html = render(&fm, &locale, body, doc_id, None, &features).expect("Render failed");
        assert!(html.contains("lang=\"de\""));
        assert!(html.contains("Doc Title"));
        assert!(html.contains("<p>Body Content</p>"));
        assert!(html.contains("test_id_99"));
        assert!(!html.contains("{{TITLE}}"));
        assert!(!html.contains("{{CONTENT}}"));
        assert!(!html.contains("{{LOGO}}"));
        assert!(html.contains("<svg"));
    }

    #[test]
    fn test_render_with_custom_logo() {
        let mut fm = Frontmatter::new();
        fm.title = Some("Doc Title".into());
        let locale = Locale::from_lang_code("en");
        let custom_logo = "<img src=\"data:image/png;base64,1234\" alt=\"Logo\">";
        let features = DocumentFeatures::default();

        let html = render(&fm, &locale, "<p>Content</p>", "doc_1", Some(custom_logo), &features)
            .expect("Render failed");

        assert!(html.contains("<img src=\"data:image/png;base64,1234\" alt=\"Logo\">"));
        assert!(!html.contains(DEFAULT_LOGO_SVG));
    }

    #[test]
    fn test_render_dynamic_locale_keys() {
        let json = r#"{
            "lang_code": "fr",
            "custom_dynamic_key": "Bonjour Le Monde",
            "another_new_field": "Valeur Dynamique"
        }"#;
        let locale = Locale::from_json(json);
        let tmpl = "<div>{{L_CUSTOM_DYNAMIC_KEY}}</div><span>{{L_ANOTHER_NEW_FIELD}}</span>";
        let vars = HashMap::new();

        let rendered = substitute_template(tmpl, &vars, Some(&locale));
        assert_eq!(
            rendered,
            "<div>Bonjour Le Monde</div><span>Valeur Dynamique</span>"
        );
    }

    #[test]
    fn test_substitute_template_var_precedence_over_locale() {
        let mut vars = HashMap::new();
        vars.insert("L_AGENT", "Overridden Agent");
        let locale = Locale::from_lang_code("de"); // has "agent": "Bearbeiter"

        let tmpl = "<div>{{L_AGENT}}</div>";
        let res = substitute_template(tmpl, &vars, Some(&locale));
        assert_eq!(res, "<div>Overridden Agent</div>");
    }

    #[test]
    fn test_default_logo_svg_constant() {
        assert!(DEFAULT_LOGO_SVG.contains("<svg"));
        assert!(DEFAULT_LOGO_SVG.contains("</svg>"));
    }

    #[test]
    fn test_render_feature_assembly() {
        let mut features = DocumentFeatures::default();
        features.has_tasks = true;
        features.has_images = false;
        features.has_header = true;

        let mut script_out = String::new();
        render_scripts(&mut script_out, &features);

        assert!(script_out.contains("d2f_state_")); // core script indicator
        assert!(script_out.contains("updateProgress")); // tasks feature indicator
        assert!(!script_out.contains("openLightbox")); // images feature excluded

        let mut style_out = String::new();
        render_styles(&mut style_out, &features);
        assert!(style_out.contains(".header-flex"));
        assert!(!style_out.contains(".lightbox"));
    }

    #[test]
    fn test_render_feature_isolation_full() {
        let mut fm = Frontmatter::new();
        fm.title = Some("Isolated Title".to_string());
        let locale = Locale::from_lang_code("en");

        // Case 1: No images, code, tasks, or header feature
        let mut features_none = DocumentFeatures::default();
        features_none.has_images = false;
        features_none.has_code = false;
        features_none.has_tasks = false;
        features_none.has_header = false;
        let html_no_img = render(&fm, &locale, "<p>No images</p>", "doc_no_img", None, &features_none)
            .expect("Render failed");

        assert!(!html_no_img.contains("<div class=\"lightbox\""));
        assert!(!html_no_img.contains(".lb-x"));
        assert!(!html_no_img.contains("openLightbox"));
        assert!(!html_no_img.contains("closeLightbox"));
        assert!(!html_no_img.contains(".code-block-wrap"));
        assert!(!html_no_img.contains("d2f_code"));
        assert!(!html_no_img.contains("id=\"finish-box\""));
        assert!(!html_no_img.contains("<div class=\"pb-col\">"));
        assert!(!html_no_img.contains("<section class=\"section header-flex\""));

        // Case 2: Images, Code, Tasks & Header feature active
        let mut features_all = DocumentFeatures::default();
        features_all.has_images = true;
        features_all.has_code = true;
        features_all.has_tasks = true;
        features_all.has_header = true;
        let html_img = render(&fm, &locale, "<p>Has image</p>", "doc_img", None, &features_all)
            .expect("Render failed");

        assert!(html_img.contains("<div class=\"lightbox\" id=\"lightbox\">"));
        assert!(html_img.contains(".lb-x"));
        assert!(html_img.contains("openLightbox"));
        assert!(html_img.contains("closeLightbox"));
        assert!(html_img.contains(".code-block-wrap"));
        assert!(html_img.contains("d2f_code"));
        assert!(html_img.contains("id=\"finish-box\""));
        assert!(html_img.contains("<div class=\"pb-col\">"));
        assert!(html_img.contains("<section class=\"section header-flex\" id=\"header-flex\">"));
        assert!(html_img.contains("<h1 class=\"header-flex-title\">Isolated Title</h1>"));
    }

    #[test]
    fn test_render_metadata_injection() {
        let fm = Frontmatter::new();
        let locale = Locale::from_lang_code("en");
        let features = DocumentFeatures::default();
        let html = render(&fm, &locale, "<p>Content</p>", "doc_meta", None, &features).expect("Render failed");
        let app_version_raw = APP_VERSION.strip_prefix('v').unwrap_or(APP_VERSION);

        assert!(html.contains(&format!(
            "<meta name=\"generator\" content=\"Doc2Flow {}\">",
            APP_VERSION
        )));
        assert!(html.contains(&format!(
            "<meta name=\"version\" content=\"{}\">",
            app_version_raw
        )));
        assert!(html.contains(&format!(
            "<meta name=\"repository\" content=\"{}\">",
            REPOSITORY_URL
        )));
        assert!(html.contains(&format!(
            "<meta name=\"license\" content=\"{}\">",
            LICENSE_URL
        )));
        assert!(html.contains("<meta name=\"dcterms.created\" content=\""));
        assert!(html.contains(&format!(
            "<meta name=\"dcterms.source\" content=\"{}\">",
            REPOSITORY_URL
        )));
        assert!(html.contains("<meta name=\"features\" content=\"core\">"));
        assert!(!html.contains("{{APP_VERSION}}"));
        assert!(!html.contains("{{APP_VERSION_RAW}}"));
        assert!(!html.contains("{{REPOSITORY_URL}}"));
        assert!(!html.contains("{{LICENSE_URL}}"));
        assert!(!html.contains("{{CREATED_AT}}"));
        assert!(!html.contains("{{FEATURES}}"));

        let mut custom_features = DocumentFeatures::default();
        custom_features.has_tasks = true;
        custom_features.has_tables = true;
        let html_custom = render(&fm, &locale, "<p>Content</p>", "doc_meta2", None, &custom_features).expect("Render failed");
        assert!(html_custom.contains("<meta name=\"features\" content=\"core, tables, tasks\">"));
    }

    #[test]
    fn test_format_iso8601_utc_epoch_boundary() {
        let epoch = std::time::UNIX_EPOCH;
        let formatted = format_iso8601_utc(epoch);
        assert_eq!(formatted, "1970-01-01T00:00:00Z");
        assert_eq!(formatted.len(), 20);
    }

    #[test]
    fn test_format_iso8601_utc_known_timestamps() {
        use std::time::{Duration, UNIX_EPOCH};

        // Fixed known timestamp: 1700000000 -> 2023-11-14T22:13:20Z
        let t1 = UNIX_EPOCH + Duration::from_secs(1_700_000_000);
        let formatted1 = format_iso8601_utc(t1);
        assert_eq!(formatted1, "2023-11-14T22:13:20Z");
        assert_eq!(formatted1.len(), 20);

        // Leap year leap day: 1582934400 -> 2020-02-29T00:00:00Z
        let t2 = UNIX_EPOCH + Duration::from_secs(1_582_934_400);
        let formatted2 = format_iso8601_utc(t2);
        assert_eq!(formatted2, "2020-02-29T00:00:00Z");
        assert_eq!(formatted2.len(), 20);
    }

    #[test]
    fn test_format_iso8601_utc_sub_epoch_fallback() {
        use std::time::{Duration, UNIX_EPOCH};

        if let Some(sub_epoch) = UNIX_EPOCH.checked_sub(Duration::from_secs(3600)) {
            let formatted = format_iso8601_utc(sub_epoch);
            assert_eq!(formatted, "1970-01-01T00:00:00Z");
            assert_eq!(formatted.len(), 20);
        }
    }

    #[test]
    fn test_format_iso8601_utc_current_time_length() {
        let now = std::time::SystemTime::now();
        let formatted = format_iso8601_utc(now);
        assert!(formatted.ends_with('Z'));
        assert_eq!(formatted.len(), 20);
    }

    #[test]
    fn test_app_version_format() {
        assert!(!APP_VERSION.is_empty());
        assert!(
            APP_VERSION.starts_with('v'),
            "Version string must start with 'v', got: {}",
            APP_VERSION
        );
        assert!(
            APP_VERSION.contains('+'),
            "Version string must contain build metadata separator '+', got: {}",
            APP_VERSION
        );

        let parts: Vec<&str> = APP_VERSION.split('+').collect();
        assert_eq!(
            parts.len(),
            2,
            "Version string must split into version and build metadata"
        );

        let semver_part = parts[0];
        let metadata_part = parts[1];

        assert!(semver_part.starts_with('v'));
        assert!(!metadata_part.is_empty());

        if metadata_part.ends_with(".dev") {
            let meta_clean = metadata_part.strip_suffix(".dev").unwrap();
            assert!(meta_clean.contains('.'));
        } else {
            assert!(metadata_part.contains('.'));
        }
    }

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

    #[test]
    fn test_assembly_pipelines_component_parity() {
        let all_features: Vec<Box<dyn Feature>> = vec![
            Box::new(CodeFeature::new()),
            Box::new(HeaderFeature::new()),
            Box::new(ImageFeature::new()),
            Box::new(TableFeature::new()),
            Box::new(TasksFeature::new()),
        ];
        let locale = Locale::default();

        // Context with all features active
        let mut fm_map = HashMap::new();
        fm_map.insert("header".to_string(), "true".to_string());
        fm_map.insert("title".to_string(), "Title".to_string());
        fm_map.insert("subtitle".to_string(), "Subtitle".to_string());

        let raw_md = "# Title\n- [ ] Task 1\n```rust\nlet x = 1;\n```\n![Alt](img.png)\n| A | B |\n|---|---|";
        let ctx = DocumentContext::new(&fm_map, raw_md);

        let html_assembled = assemble_html(
            &ctx,
            &all_features,
            &locale,
            "<p>Content</p>",
            "doc_parity",
            None,
        )
        .expect("assemble_html must succeed");

        // Verify that conditional components are present in assemble_html output
        assert!(html_assembled.contains("<div class=\"lightbox\" id=\"lightbox\">"));
        assert!(html_assembled.contains("class=\"pb-wrap\""));
        assert!(html_assembled.contains("id=\"finish-box\""));
        assert!(html_assembled.contains("id=\"header-flex\""));

        // Compare with render path
        let mut fm = Frontmatter::new();
        fm.title = Some("Title".to_string());
        fm.subtitle = Some("Subtitle".to_string());
        fm.header = true;

        let mut doc_features = DocumentFeatures::default();
        doc_features.has_code = true;
        doc_features.has_header = true;
        doc_features.has_images = true;
        doc_features.has_tables = true;
        doc_features.has_tasks = true;

        let html_rendered = render(
            &fm,
            &locale,
            "<p>Content</p>",
            "doc_parity",
            None,
            &doc_features,
        )
        .expect("render must succeed");

        assert!(html_rendered.contains("<div class=\"lightbox\" id=\"lightbox\">"));
        assert!(html_rendered.contains("class=\"pb-wrap\""));
        assert!(html_rendered.contains("id=\"finish-box\""));
        assert!(html_rendered.contains("id=\"header-flex\""));
    }

    #[test]
    fn test_metadata_formatting_consistency_between_render_and_assemble() {
        let all_features = features::get_all_features();
        let locale = Locale::default();
        let doc_id = "doc_meta_consistency";

        // Case 1: Minimal / Core only
        let fm_map_empty = HashMap::new();
        let ctx_empty = DocumentContext::new(&fm_map_empty, "Just plain text");
        let fm_empty = Frontmatter::default();
        let df_empty = DocumentFeatures::default();

        let html_asm_empty = assemble_html(
            &ctx_empty,
            &all_features,
            &locale,
            "<p>Plain</p>",
            doc_id,
            None,
        )
        .unwrap();

        let html_rnd_empty = render(
            &fm_empty,
            &locale,
            "<p>Plain</p>",
            doc_id,
            None,
            &df_empty,
        )
        .unwrap();

        let app_version_raw = APP_VERSION.strip_prefix('v').unwrap_or(APP_VERSION);
        let expected_generator = format!("<meta name=\"generator\" content=\"Doc2Flow {APP_VERSION}\">");
        let expected_version = format!("<meta name=\"version\" content=\"{app_version_raw}\">");
        let expected_repo = format!("<meta name=\"repository\" content=\"{REPOSITORY_URL}\">");
        let expected_lic = format!("<meta name=\"license\" content=\"{LICENSE_URL}\">");
        let expected_feat = "<meta name=\"features\" content=\"core\">";

        for html in [&html_asm_empty, &html_rnd_empty] {
            assert!(html.contains(&expected_generator));
            assert!(html.contains(&expected_version));
            assert!(html.contains(&expected_repo));
            assert!(html.contains(&expected_lic));
            assert!(html.contains(expected_feat));
        }

        // Case 2: Code feature active (with transitive fields dependency)
        let ctx_code = DocumentContext::new(&fm_map_empty, "```rust\nfn main() {}\n```");
        let mut df_code = DocumentFeatures::default();
        df_code.has_code = true;
        df_code.has_fields = true;

        let html_asm_code = assemble_html(
            &ctx_code,
            &all_features,
            &locale,
            "<p>Code</p>",
            doc_id,
            None,
        )
        .unwrap();

        let html_rnd_code = render(
            &fm_empty,
            &locale,
            "<p>Code</p>",
            doc_id,
            None,
            &df_code,
        )
        .unwrap();

        let expected_feat_code = "<meta name=\"features\" content=\"core, code, fields\">";
        assert!(html_asm_code.contains(expected_feat_code));
        assert!(html_rnd_code.contains(expected_feat_code));
    }

    #[test]
    fn test_feature_matching_parity_all_flags() {
        let mut df = DocumentFeatures::default();

        // 1. Code
        df.has_code = true;
        let mut s_code = String::new();
        let mut j_code = String::new();
        render_styles(&mut s_code, &df);
        render_scripts(&mut j_code, &df);
        assert!(s_code.contains(".code-block"));
        assert!(j_code.contains("copyCode") || j_code.contains("copy"));
        assert!(!s_code.contains(".check-item"));
        assert!(!s_code.contains(".lightbox"));

        // 2. Tasks
        let mut df_tasks = DocumentFeatures::default();
        df_tasks.has_tasks = true;
        let mut s_tasks = String::new();
        let mut j_tasks = String::new();
        render_styles(&mut s_tasks, &df_tasks);
        render_scripts(&mut j_tasks, &df_tasks);
        assert!(s_tasks.contains(".check-item"));
        assert!(j_tasks.contains("updateProgress") || j_tasks.contains("saveTasks"));
        assert!(!s_tasks.contains(".code-block"));
        assert!(!s_tasks.contains(".lightbox"));

        // 3. Image
        let mut df_img = DocumentFeatures::default();
        df_img.has_images = true;
        let mut s_img = String::new();
        let mut j_img = String::new();
        render_styles(&mut s_img, &df_img);
        render_scripts(&mut j_img, &df_img);
        assert!(s_img.contains(".lightbox"));
        assert!(j_img.contains("openLightbox") || j_img.contains("closeLightbox"));
        assert!(!s_img.contains(".code-block"));
        assert!(!s_img.contains(".check-item"));

        // 4. Table
        let mut df_tbl = DocumentFeatures::default();
        df_tbl.has_tables = true;
        let mut s_tbl = String::new();
        let mut j_tbl = String::new();
        render_styles(&mut s_tbl, &df_tbl);
        render_scripts(&mut j_tbl, &df_tbl);
        assert!(s_tbl.contains(".item-table"));
        assert!(j_tbl.contains("initSectionTables") || j_tbl.contains("item-table"));
        assert!(!s_tbl.contains(".code-block"));
        assert!(!s_tbl.contains(".lightbox"));

        // 5. Header
        let mut df_hdr = DocumentFeatures::default();
        df_hdr.has_header = true;
        let mut s_hdr = String::new();
        let mut j_hdr = String::new();
        render_styles(&mut s_hdr, &df_hdr);
        render_scripts(&mut j_hdr, &df_hdr);
        assert!(s_hdr.contains(".header-flex"));
        assert!(!s_hdr.contains(".code-block"));
        assert!(!s_hdr.contains(".item-table"));

        // 6. Fields
        let mut df_fld = DocumentFeatures::default();
        df_fld.has_fields = true;
        let mut s_fld = String::new();
        let mut j_fld = String::new();
        render_styles(&mut s_fld, &df_fld);
        render_scripts(&mut j_fld, &df_fld);
        assert_eq!(s_fld.trim(), STYLE_CORE.trim());
        assert!(j_fld.contains("saveFields") || j_fld.contains("loadFields"));
    }
}
