//! Task list, checklist progress tracking, and sign-off finish box feature slice.

use crate::core::components::COMMENT_ICON_SVG;
use crate::core::feature::{DocumentContext, Feature};
use std::fmt::Write;

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
    let checked_attr = if is_checked { " checked" } else { "" };
    let label_text = clean_label.trim();

    let _ = write!(
        out,
        "<div class=\"doc-item check-item{checked_attr}\" id=\"wrap-cb_s{sec_num}_{cb_count}\""
    );
    if indent_depth > 0 {
        let _ = write!(out, " style=\"--indent: {indent_depth};\"");
    }
    let _ = write!(
        out,
        ">\n  <input type=\"checkbox\" id=\"cb_s{sec_num}_{cb_count}\"{checked_attr}>\n  <label class=\"check-label\" for=\"cb_s{sec_num}_{cb_count}\">{label_text}</label>\n  {COMMENT_ICON_SVG}\n</div>\n"
    );
}

/// Renders the top process progress bar component if the tasks feature is enabled.
#[inline]
pub fn render_progress_bar(out: &mut impl Write, has_tasks: bool, loading_label: &str) {
    if has_tasks {
        let _ = write!(
            out,
            "<div class=\"pb-col\">\n  <div class=\"pb-wrap\" role=\"progressbar\" aria-valuenow=\"0\" aria-valuemin=\"0\" aria-valuemax=\"100\"><div class=\"pb\" id=\"pb\"></div></div>\n  <div class=\"pt\" id=\"pt\">{loading_label}</div>\n</div>"
        );
    }
}

/// Renders the bottom finish box component if the tasks feature is enabled.
#[inline]
pub fn render_finish_box(
    out: &mut impl Write,
    has_tasks: bool,
    setup_completed_label: &str,
    name_placeholder: &str,
    date_placeholder: &str,
    signature_date_label: &str,
) {
    if has_tasks {
        let _ = write!(
            out,
            "<div class=\"finish\" id=\"finish-box\">\n  <div class=\"big\" id=\"finish-icon\">&#x2714;</div>\n  <h2 id=\"finish-title\">{setup_completed_label}</h2>\n  <div class=\"sigs\">\n    <div><input type=\"text\" class=\"sf persistent-field\" id=\"f_sign_agent\" placeholder=\"{name_placeholder}\" aria-label=\"{name_placeholder}\"><div style=\"margin-top:4px\">{name_placeholder}</div></div>\n    <div><input type=\"text\" class=\"sf persistent-field\" id=\"f_sign_date\" placeholder=\"{date_placeholder}\" aria-label=\"{signature_date_label}\"><div style=\"margin-top:4px\">{signature_date_label}</div></div>\n  </div>\n</div>"
        );
    }
}

/// Unified tasks feature slice providing interactive checklist toggles, progress bar tracking, and sign-off footer.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct TasksFeature;

impl TasksFeature {
    /// Creates a new tasks feature instance.
    ///
    /// # Examples
    ///
    /// ```
    /// use doc2flow::features::tasks::TasksFeature;
    ///
    /// let feature = TasksFeature::new();
    /// ```
    #[inline]
    pub const fn new() -> Self {
        Self
    }
}

impl Feature for TasksFeature {
    /// Returns the unique feature identifier "tasks".
    #[inline]
    fn name(&self) -> &'static str {
        "tasks"
    }

    /// Evaluates if the tasks feature is enabled based on presence of task list markers in markdown.
    #[inline]
    fn is_enabled(&self, ctx: &DocumentContext) -> bool {
        ctx.raw_markdown.contains("[ ]")
            || ctx.raw_markdown.contains("[x]")
            || ctx.raw_markdown.contains("[X]")
    }

    /// Returns embedded TypeScript client script for task checklist progress calculation and state management.
    #[inline]
    fn javascript(&self) -> Option<&'static str> {
        Some(include_str!("tasks.ts"))
    }

    /// Returns embedded CSS styles for task items, progress bar, badges, and sign-off finish box.
    #[inline]
    fn css(&self) -> Option<&'static str> {
        Some(include_str!("tasks.css"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_tasks_feature_activation_logic() {
        let feature = TasksFeature::new();
        let fm = HashMap::new();

        // 1. Markdown with unchecked task items: enabled
        let ctx_with_unchecked = DocumentContext::new(&fm, "- [ ] Verify system configuration");
        assert!(feature.is_enabled(&ctx_with_unchecked));

        // 2. Markdown with checked task items: enabled
        let ctx_with_checked = DocumentContext::new(&fm, "- [x] Initial setup completed");
        assert!(feature.is_enabled(&ctx_with_checked));

        let ctx_with_upper_checked = DocumentContext::new(&fm, "1. [X] Ordered task item");
        assert!(feature.is_enabled(&ctx_with_upper_checked));

        // 3. Markdown without task items: disabled
        let ctx_no_tasks = DocumentContext::new(&fm, "# Heading\n- Regular bullet item\n1. Numbered item");
        assert!(!feature.is_enabled(&ctx_no_tasks));

        let ctx_empty = DocumentContext::new(&fm, "");
        assert!(!feature.is_enabled(&ctx_empty));
    }

    #[test]
    fn test_tasks_assets_embedded() {
        let feature = TasksFeature::new();
        assert_eq!(feature.name(), "tasks");
        let js = feature.javascript().expect("JavaScript must be embedded");
        let css = feature.css().expect("CSS must be embedded");

        assert!(js.contains("updateProgress") || js.contains("saveTasks"));
        assert!(js.contains("styleItem"));
        assert!(css.contains(".check-item"));
        assert!(css.contains(".pb-wrap"));
        assert!(css.contains(".finish"));
        assert!(css.contains(".sbadge"));
    }

    #[test]
    fn test_render_task_item() {
        let mut buf = String::new();
        render_task_item(&mut buf, 1, 2, true, "Check task", 0);
        assert!(buf.contains("<div class=\"doc-item check-item checked\" id=\"wrap-cb_s1_2\">"));
        assert!(buf.contains("<input type=\"checkbox\" id=\"cb_s1_2\" checked>"));
        assert!(buf.contains("<label class=\"check-label\" for=\"cb_s1_2\">Check task</label>"));
    }

    #[test]
    fn test_render_progress_bar() {
        let mut buf_true = String::new();
        render_progress_bar(&mut buf_true, true, "Loading...");
        assert!(buf_true.contains("<div class=\"pb-col\">"));
        assert!(buf_true.contains("<div class=\"pt\" id=\"pt\">Loading...</div>"));

        let mut buf_false = String::new();
        render_progress_bar(&mut buf_false, false, "Loading...");
        assert_eq!(buf_false, "");
    }

    #[test]
    fn test_render_finish_box() {
        let mut buf_true = String::new();
        render_finish_box(
            &mut buf_true,
            true,
            "Completed",
            "Name",
            "MM/DD/YYYY",
            "Date",
        );
        assert!(buf_true.contains("<div class=\"finish\" id=\"finish-box\">"));
        assert!(buf_true.contains("<h2 id=\"finish-title\">Completed</h2>"));

        let mut buf_false = String::new();
        render_finish_box(
            &mut buf_false,
            false,
            "Completed",
            "Name",
            "MM/DD/YYYY",
            "Date",
        );
        assert_eq!(buf_false, "");
    }
}
