//! Task list, checklist progress tracking, and sign-off finish box feature slice.

use crate::core::feature::{DocumentContext, Feature};

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
}
