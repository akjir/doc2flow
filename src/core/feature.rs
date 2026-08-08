//! Feature trait and document context for vertical slice feature detection.

use std::collections::HashMap;

/// Document context provided to features during detection and rendering.
#[derive(Debug, Clone, Copy)]
pub struct DocumentContext<'a> {
    /// Frontmatter key-value pairs extracted from the markdown header.
    pub frontmatter: &'a HashMap<String, String>,
    /// Raw unparsed markdown content of the document.
    pub raw_markdown: &'a str,
}

impl<'a> DocumentContext<'a> {
    /// Creates a new document context.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::collections::HashMap;
    /// use doc2flow::core::feature::DocumentContext;
    ///
    /// let fm = HashMap::new();
    /// let ctx = DocumentContext::new(&fm, "# Heading");
    /// assert_eq!(ctx.raw_markdown, "# Heading");
    /// ```
    pub const fn new(frontmatter: &'a HashMap<String, String>, raw_markdown: &'a str) -> Self {
        Self {
            frontmatter,
            raw_markdown,
        }
    }
}

/// Feature trait defining detection and asset provision for vertical slices.
pub trait Feature {
    /// Unique identifier for the feature (e.g., "copy_code").
    fn name(&self) -> &'static str;

    /// Checks whether the feature should be enabled for the document context.
    fn is_enabled(&self, ctx: &DocumentContext) -> bool;

    /// Returns optional JavaScript code for this feature, defaulting to `None`.
    fn javascript(&self) -> Option<&'static str> {
        None
    }

    /// Returns optional CSS stylesheet rules for this feature, defaulting to `None`.
    fn css(&self) -> Option<&'static str> {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct DefaultFeature;

    impl Feature for DefaultFeature {
        fn name(&self) -> &'static str {
            "default_feat"
        }

        fn is_enabled(&self, ctx: &DocumentContext) -> bool {
            ctx.frontmatter.contains_key("enable_default")
        }
    }

    struct CustomFeature;

    impl Feature for CustomFeature {
        fn name(&self) -> &'static str {
            "custom_feat"
        }

        fn is_enabled(&self, ctx: &DocumentContext) -> bool {
            ctx.raw_markdown.contains("ENABLE_CUSTOM")
        }

        fn javascript(&self) -> Option<&'static str> {
            Some("console.log('custom');")
        }

        fn css(&self) -> Option<&'static str> {
            Some(".custom { display: block; }")
        }
    }

    #[test]
    fn test_document_context_new_and_equality() {
        let mut fm = HashMap::new();
        fm.insert("title".to_string(), "Doc Title".to_string());
        let md = "# Heading\nContent";

        let ctx = DocumentContext::new(&fm, md);
        assert_eq!(ctx.raw_markdown, md);
        assert_eq!(
            ctx.frontmatter.get("title").map(|s| s.as_str()),
            Some("Doc Title")
        );
    }

    #[test]
    fn test_document_context_debug_clone_copy() {
        let fm = HashMap::new();
        let md = "Sample Markdown";
        let ctx = DocumentContext::new(&fm, md);

        let debug_str = format!("{:?}", ctx);
        assert!(debug_str.contains("DocumentContext"));
        assert!(debug_str.contains("Sample Markdown"));

        let copied = ctx;
        let cloned = ctx.clone();

        assert_eq!(copied.raw_markdown, ctx.raw_markdown);
        assert_eq!(cloned.raw_markdown, ctx.raw_markdown);
        assert_eq!(copied.frontmatter, ctx.frontmatter);
        assert_eq!(cloned.frontmatter, ctx.frontmatter);
    }

    #[test]
    fn test_default_feature_methods() {
        let feature = DefaultFeature;
        assert_eq!(feature.name(), "default_feat");
        assert_eq!(feature.javascript(), None);
        assert_eq!(feature.css(), None);

        let mut fm = HashMap::new();
        let ctx_disabled = DocumentContext::new(&fm, "markdown");
        assert!(!feature.is_enabled(&ctx_disabled));

        fm.insert("enable_default".to_string(), "true".to_string());
        let ctx_enabled = DocumentContext::new(&fm, "markdown");
        assert!(feature.is_enabled(&ctx_enabled));
    }

    #[test]
    fn test_custom_feature_override_methods() {
        let feature = CustomFeature;
        assert_eq!(feature.name(), "custom_feat");
        assert_eq!(feature.javascript(), Some("console.log('custom');"));
        assert_eq!(feature.css(), Some(".custom { display: block; }"));

        let fm = HashMap::new();
        let ctx_disabled = DocumentContext::new(&fm, "regular content");
        assert!(!feature.is_enabled(&ctx_disabled));

        let ctx_enabled = DocumentContext::new(&fm, "has ENABLE_CUSTOM token");
        assert!(feature.is_enabled(&ctx_enabled));
    }
}

