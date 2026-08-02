use crate::components;
use crate::converter::{DocumentFeatures, Frontmatter};
use crate::error::{Doc2FlowError, Result};
use crate::language::{Locale, validate_locale_coverage};
use std::collections::HashMap;
use std::fmt::Write;

/// Embedded core CSS styles.
pub static STYLE_CORE: &str = include_str!("../styles/core.css");

/// Embedded code block CSS feature styles.
pub static STYLE_CODE: &str = include_str!("../styles/code.css");

/// Embedded task list and checklist CSS feature styles.
pub static STYLE_TASKS: &str = include_str!("../styles/tasks.css");

/// Embedded image lightbox CSS feature styles.
pub static STYLE_IMAGES: &str = include_str!("../styles/images.css");

/// Embedded Table of Contents CSS feature styles.
pub static STYLE_TOC: &str = include_str!("../styles/toc.css");

/// Embedded core JavaScript bundle.
pub static SCRIPT_CORE: &str = include_str!("../web/dist/script-core.js");

/// Embedded code block JavaScript feature bundle.
pub static SCRIPT_CODE: &str = include_str!("../web/dist/script-code.js");

/// Embedded task list and checklist JavaScript feature bundle.
pub static SCRIPT_TASKS: &str = include_str!("../web/dist/script-tasks.js");

/// Embedded image lightbox JavaScript feature bundle.
pub static SCRIPT_IMAGES: &str = include_str!("../web/dist/script-images.js");

/// Embedded Table of Contents JavaScript feature bundle.
pub static SCRIPT_TOC: &str = include_str!("../web/dist/script-toc.js");

/// Assembles active CSS feature styles into the provided output string based on detected document features.
pub fn render_styles(out: &mut String, features: &DocumentFeatures) {
    out.push_str(STYLE_CORE);
    out.push_str("\n");

    if features.has_code {
        out.push_str(STYLE_CODE);
        out.push_str("\n");
    }

    if features.has_tasks {
        out.push_str(STYLE_TASKS);
        out.push_str("\n");
    }

    if features.has_images {
        out.push_str(STYLE_IMAGES);
        out.push_str("\n");
    }

    if features.has_toc {
        out.push_str(STYLE_TOC);
        out.push_str("\n");
    }
}

/// Assembles active JavaScript feature bundles into the provided output string based on detected document features.
pub fn render_scripts(out: &mut String, features: &DocumentFeatures) {
    out.push_str(SCRIPT_CORE);
    out.push_str("\n");

    if features.has_code {
        out.push_str(SCRIPT_CODE);
        out.push_str("\n");
    }

    if features.has_tasks {
        out.push_str(SCRIPT_TASKS);
        out.push_str("\n");
    }

    if features.has_images {
        out.push_str(SCRIPT_IMAGES);
        out.push_str("\n");
    }

    if features.has_toc {
        out.push_str(SCRIPT_TOC);
        out.push_str("\n");
    }
}

/// Renders the image lightbox modal markup if the document contains images.
#[inline]
pub fn render_lightbox(out: &mut impl Write, features: &DocumentFeatures) {
    components::render_lightbox(out, features.has_images);
}

/// Renders the top process progress bar component if the document contains tasks.
#[inline]
pub fn render_progress_bar(out: &mut impl Write, features: &DocumentFeatures, locale: &Locale) {
    let loading = locale.get_ignore_ascii_case("LOADING").unwrap_or("");
    components::render_progress_bar(out, features.has_tasks, loading);
}

/// Renders the bottom finish box component if the document contains tasks.
#[inline]
pub fn render_finish_box(out: &mut impl Write, features: &DocumentFeatures, locale: &Locale) {
    let setup_completed = locale.get_ignore_ascii_case("SETUP_COMPLETED").unwrap_or("");
    let name_placeholder = locale.get_ignore_ascii_case("NAME_PLACEHOLDER").unwrap_or("");
    let agent = locale.get_ignore_ascii_case("AGENT").unwrap_or("");
    let date_placeholder = locale.get_ignore_ascii_case("DATE_PLACEHOLDER").unwrap_or("");
    let signature_date = locale.get_ignore_ascii_case("SIGNATURE_DATE").unwrap_or("");

    components::render_finish_box(
        out,
        features.has_tasks,
        setup_completed,
        name_placeholder,
        agent,
        date_placeholder,
        signature_date,
    );
}


/// Default embedded SVG header logo.
pub const DEFAULT_LOGO_SVG: &str = components::DEFAULT_LOGO_SVG;

/// Embedded SVG comment icon for interactive elements.
pub const COMMENT_ICON_SVG: &str = components::COMMENT_ICON_SVG;

/// Application version with SemVer 2.0.0 build metadata dynamically generated at compile time.
pub const APP_VERSION: &str = env!("D2F_FULL_VERSION");

/// Official application repository URL.
pub const REPOSITORY_URL: &str = "https://github.com/akjir/doc2flow";

/// Application license terms.
pub const LICENSE_TERMS: &str = "GPL-3.0-or-later";

/// Official application license URL.
pub const LICENSE_URL: &str = "https://github.com/akjir/doc2flow/blob/main/LICENSE";

/// Formats a [`std::time::SystemTime`] as an ISO 8601 UTC timestamp (`YYYY-MM-DDTHH:MM:SSZ`).
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

/// Renders a section header component directly into the output buffer.
#[inline]
pub fn render_section_header(
    out: &mut impl Write,
    section_count: usize,
    heading_text: &str,
    is_h1: bool,
    is_empty: bool,
    has_checklist: bool,
    callout_type: Option<&str>,
) {
    components::render_section_header(
        out,
        section_count,
        heading_text,
        is_h1,
        is_empty,
        has_checklist,
        callout_type,
    );
}

/// Renders section container closing tags directly into the output buffer.
#[inline]
pub fn render_section_close(out: &mut impl Write) {
    components::render_section_close(out);
}

/// Renders a subheading component (H3-H6) directly into the output buffer.
#[inline]
pub fn render_subheading(out: &mut impl Write, sub_html: &str) {
    components::render_subheading(out, sub_html);
}

/// Renders an alert or callout box component directly into the output buffer.
#[inline]
pub fn render_callout(
    out: &mut impl Write,
    note_cls: &str,
    escaped_label: &str,
    note_content: &str,
) {
    components::render_callout(out, note_cls, escaped_label, note_content);
}

/// Renders a code block component directly into the output buffer.
#[inline]
pub fn render_code_block(
    out: &mut impl Write,
    lang_opt: Option<&str>,
    escaped_code: &str,
    copy_label: &str,
) {
    components::render_code_block(out, lang_opt, escaped_code, copy_label);
}

/// Renders a task list checkbox item directly into the output buffer.
#[inline]
pub fn render_task_item(
    out: &mut impl Write,
    sec_num: usize,
    cb_count: usize,
    is_checked: bool,
    clean_label: &str,
    indent_depth: usize,
) {
    components::render_task_item(
        out,
        sec_num,
        cb_count,
        is_checked,
        clean_label,
        indent_depth,
    );
}

/// Renders a simple list item component directly into the output buffer.
#[inline]
pub fn render_list_item(
    out: &mut impl Write,
    sec_num: usize,
    item_count: usize,
    bullet: &str,
    clean_label: &str,
    indent_depth: usize,
) {
    components::render_list_item(out, sec_num, item_count, bullet, clean_label, indent_depth);
}

/// Renders a standalone text paragraph item component directly into the output buffer.
#[inline]
pub fn render_text_item(
    out: &mut impl Write,
    sec_num: usize,
    txt_count: usize,
    content_html: &str,
    indent_depth: usize,
) {
    components::render_text_item(out, sec_num, txt_count, content_html, indent_depth);
}

/// Renders an image container block directly into the output buffer.
#[inline]
pub fn render_image_item(out: &mut impl Write, clean_content: &str) {
    components::render_image_item(out, clean_content);
}

/// Returns the pre-populated default starter Markdown template string.
///
/// Contains frontmatter metadata fields, HTML comments with usage instructions,
/// and a showcase document structure.
///
/// # Examples
///
/// ```
/// use doc2flow::template::generate_template_markdown;
///
/// let template = generate_template_markdown();
/// assert!(template.contains("title:"));
/// assert!(template.contains("## Section 1: Initial System Verification"));
/// ```
pub fn generate_template_markdown() -> String {
    let raw = include_str!("../templates/template.md");
    raw.replace("{{APP_VERSION}}", APP_VERSION)
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
/// use doc2flow::language::Locale;
/// use doc2flow::template::substitute_template;
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
    let total_locale_len: usize = locale.map_or(0, |l| l.entries.values().map(String::len).sum());
    let mut result = String::with_capacity(template.len() + total_vars_len + total_locale_len);

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

/// Renders a complete self-contained HTML document.
///
/// Combines frontmatter metadata, locale configuration, rendered markdown content,
/// embedded CSS/JS resources, and base HTML template into a single HTML string.
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
    let base_html = include_str!("../templates/base.html");

    let mut style_css = String::with_capacity(32768);
    render_styles(&mut style_css, features);

    let mut script_js = String::with_capacity(32768);
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

    let mut vars = HashMap::with_capacity(23);
    vars.insert("APP_VERSION", APP_VERSION);
    vars.insert("APP_VERSION_RAW", app_version_raw);
    vars.insert("REPOSITORY_URL", REPOSITORY_URL);
    vars.insert("LICENSE_TERMS", LICENSE_TERMS);
    vars.insert("LICENSE_URL", LICENSE_URL);
    vars.insert("CREATED_AT", created_at.as_str());
    vars.insert("LANG_CODE", locale.lang_code.as_str());
    vars.insert("TITLE", frontmatter.title.as_deref().unwrap_or(""));
    vars.insert("SUBTITLE", frontmatter.subtitle.as_deref().unwrap_or(""));
    vars.insert("COMPANY", frontmatter.company.as_str());
    vars.insert("CONTACT", frontmatter.contact.as_deref().unwrap_or(""));
    vars.insert("AGENT", frontmatter.agent.as_deref().unwrap_or(""));
    vars.insert("DATE", frontmatter.date.as_deref().unwrap_or(""));
    vars.insert("I18N_JSON", i18n_json.as_str());
    vars.insert("CSS", style_css.as_str());
    vars.insert("JS", script_js.as_str());
    vars.insert("LIGHTBOX_HTML", lightbox_html.as_str());
    vars.insert("PROGRESS_BAR_HTML", progress_bar_html.as_str());
    vars.insert("FINISH_BOX_HTML", finish_box_html.as_str());
    vars.insert("CONTENT", html_content);
    vars.insert("DOC_ID", doc_id);
    vars.insert("LOGO", logo);

    Ok(substitute_template(base_html, &vars, Some(locale)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_template_markdown_contains_required_sections() {
        let content = generate_template_markdown();
        assert!(content.contains("title:"));
        assert!(content.contains("subtitle:"));
        assert!(content.contains("company:"));
        assert!(content.contains("contact:"));
        assert!(content.contains("agent:"));
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
    fn test_render_full_document() {
        let mut fm = Frontmatter::new("Test Corp");
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
        assert!(!html.contains("{{L_COMPANY}}"));
        assert!(!html.contains("{{LOGO}}"));
        assert!(html.contains("<svg"));
        assert!(html.contains("Firma"));
    }

    #[test]
    fn test_render_with_custom_logo() {
        let mut fm = Frontmatter::new("Acme");
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
        vars.insert("L_COMPANY", "Overridden Company");
        let locale = Locale::from_lang_code("de"); // has "company": "Firma"

        let tmpl = "<div>{{L_COMPANY}}</div>";
        let res = substitute_template(tmpl, &vars, Some(&locale));
        assert_eq!(res, "<div>Overridden Company</div>");
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
        features.has_toc = true;

        let mut script_out = String::new();
        render_scripts(&mut script_out, &features);

        assert!(script_out.contains("d2f_state_")); // core script indicator
        assert!(script_out.contains("updateProgress")); // tasks feature indicator
        assert!(!script_out.contains("openLightbox")); // images feature excluded

        let mut style_out = String::new();
        render_styles(&mut style_out, &features);
        assert!(!style_out.contains(".lightbox"));
    }

    #[test]
    fn test_render_feature_isolation_full() {
        let fm = Frontmatter::new("Test Corp");
        let locale = Locale::from_lang_code("en");

        // Case 1: No images, code, or tasks feature
        let mut features_none = DocumentFeatures::default();
        features_none.has_images = false;
        features_none.has_code = false;
        features_none.has_tasks = false;
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

        // Case 2: Images, Code & Tasks feature active
        let mut features_all = DocumentFeatures::default();
        features_all.has_images = true;
        features_all.has_code = true;
        features_all.has_tasks = true;
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
    }

    #[test]
    fn test_render_metadata_injection() {
        let fm = Frontmatter::new("Test Corp");
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
        assert!(!html.contains("{{APP_VERSION}}"));
        assert!(!html.contains("{{APP_VERSION_RAW}}"));
        assert!(!html.contains("{{REPOSITORY_URL}}"));
        assert!(!html.contains("{{LICENSE_URL}}"));
        assert!(!html.contains("{{CREATED_AT}}"));
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
}
