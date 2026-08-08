//! Feature trait and document context for vertical slice feature detection.

use std::collections::{HashMap, HashSet};

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

    /// Returns slice of static string identifiers of features that this feature depends on.
    ///
    /// If this feature is enabled, all of its dependencies will also be enabled automatically.
    ///
    /// # Examples
    ///
    /// ```
    /// use doc2flow::core::feature::{DocumentContext, Feature};
    ///
    /// struct CustomFeature;
    /// impl Feature for CustomFeature {
    ///     fn name(&self) -> &'static str { "custom" }
    ///     fn is_enabled(&self, _ctx: &DocumentContext) -> bool { true }
    ///     fn dependencies(&self) -> &'static [&'static str] { &["base_feat"] }
    /// }
    ///
    /// let feat = CustomFeature;
    /// assert_eq!(feat.dependencies(), &["base_feat"]);
    /// ```
    fn dependencies(&self) -> &'static [&'static str] {
        &[]
    }

    /// Returns optional JavaScript code for this feature, defaulting to `None`.
    fn javascript(&self) -> Option<&'static str> {
        None
    }

    /// Returns optional CSS stylesheet rules for this feature, defaulting to `None`.
    fn css(&self) -> Option<&'static str> {
        None
    }
}

/// Resolves all enabled feature names given registered features and a document context.
///
/// Traverses directly enabled features and resolves all transitive dependencies.
///
/// # Examples
///
/// ```
/// use std::collections::HashMap;
/// use doc2flow::core::feature::{DocumentContext, Feature, resolve_enabled_features};
///
/// struct FeatA;
/// impl Feature for FeatA {
///     fn name(&self) -> &'static str { "feat_a" }
///     fn is_enabled(&self, _ctx: &DocumentContext) -> bool { true }
///     fn dependencies(&self) -> &'static [&'static str] { &["feat_b"] }
/// }
///
/// struct FeatB;
/// impl Feature for FeatB {
///     fn name(&self) -> &'static str { "feat_b" }
///     fn is_enabled(&self, _ctx: &DocumentContext) -> bool { false }
/// }
///
/// let fm = HashMap::new();
/// let ctx = DocumentContext::new(&fm, "");
/// let features: Vec<Box<dyn Feature>> = vec![Box::new(FeatA), Box::new(FeatB)];
/// let enabled = resolve_enabled_features(&features, &ctx);
/// assert!(enabled.contains("feat_a"));
/// assert!(enabled.contains("feat_b"));
/// ```
pub fn resolve_enabled_features(
    features: &[Box<dyn Feature>],
    ctx: &DocumentContext,
) -> HashSet<&'static str> {
    let mut enabled_names = HashSet::with_capacity(features.len());
    let mut feature_map: HashMap<&'static str, &Box<dyn Feature>> =
        HashMap::with_capacity(features.len());
    let mut queue: Vec<&'static str> = Vec::with_capacity(features.len());

    // 1. Initial pass: build lookup map and collect directly enabled features
    for feature in features {
        let name = feature.name();
        feature_map.insert(name, feature);
        if feature.is_enabled(ctx) && enabled_names.insert(name) {
            queue.push(name);
        }
    }

    // 2. Transitive dependency resolution pass
    while let Some(current_name) = queue.pop() {
        if let Some(feature) = feature_map.get(current_name) {
            for &dep in feature.dependencies() {
                if enabled_names.insert(dep) {
                    queue.push(dep);
                }
            }
        }
    }

    enabled_names
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

    struct FeatureA;
    impl Feature for FeatureA {
        fn name(&self) -> &'static str {
            "feat_a"
        }

        fn is_enabled(&self, ctx: &DocumentContext) -> bool {
            ctx.frontmatter.contains_key("enable_a")
        }

        fn dependencies(&self) -> &'static [&'static str] {
            &["feat_b"]
        }
    }

    struct FeatureB;
    impl Feature for FeatureB {
        fn name(&self) -> &'static str {
            "feat_b"
        }

        fn is_enabled(&self, _ctx: &DocumentContext) -> bool {
            false
        }

        fn dependencies(&self) -> &'static [&'static str] {
            &["feat_c"]
        }
    }

    struct FeatureC;
    impl Feature for FeatureC {
        fn name(&self) -> &'static str {
            "feat_c"
        }

        fn is_enabled(&self, _ctx: &DocumentContext) -> bool {
            false
        }
    }

    struct CyclicFeatureX;
    impl Feature for CyclicFeatureX {
        fn name(&self) -> &'static str {
            "cyclic_x"
        }

        fn is_enabled(&self, _ctx: &DocumentContext) -> bool {
            true
        }

        fn dependencies(&self) -> &'static [&'static str] {
            &["cyclic_y"]
        }
    }

    struct CyclicFeatureY;
    impl Feature for CyclicFeatureY {
        fn name(&self) -> &'static str {
            "cyclic_y"
        }

        fn is_enabled(&self, _ctx: &DocumentContext) -> bool {
            false
        }

        fn dependencies(&self) -> &'static [&'static str] {
            &["cyclic_x"]
        }
    }

    struct UnregisteredDepFeature;
    impl Feature for UnregisteredDepFeature {
        fn name(&self) -> &'static str {
            "unregistered_parent"
        }

        fn is_enabled(&self, _ctx: &DocumentContext) -> bool {
            true
        }

        fn dependencies(&self) -> &'static [&'static str] {
            &["ghost_feature"]
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
        assert_eq!(feature.dependencies(), &[] as &[&str]);
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
        assert_eq!(feature.dependencies(), &[] as &[&str]);
        assert_eq!(feature.javascript(), Some("console.log('custom');"));
        assert_eq!(feature.css(), Some(".custom { display: block; }"));

        let fm = HashMap::new();
        let ctx_disabled = DocumentContext::new(&fm, "regular content");
        assert!(!feature.is_enabled(&ctx_disabled));

        let ctx_enabled = DocumentContext::new(&fm, "has ENABLE_CUSTOM token");
        assert!(feature.is_enabled(&ctx_enabled));
    }

    #[test]
    fn test_resolve_direct_frontmatter_activation() {
        let features: Vec<Box<dyn Feature>> = vec![
            Box::new(DefaultFeature),
            Box::new(CustomFeature),
        ];

        let mut fm = HashMap::new();
        fm.insert("enable_default".to_string(), "true".to_string());
        let ctx = DocumentContext::new(&fm, "plain text");

        let resolved = resolve_enabled_features(&features, &ctx);
        assert!(resolved.contains("default_feat"));
        assert!(!resolved.contains("custom_feat"));
    }

    #[test]
    fn test_resolve_direct_markdown_activation() {
        let features: Vec<Box<dyn Feature>> = vec![
            Box::new(DefaultFeature),
            Box::new(CustomFeature),
        ];

        let fm = HashMap::new();
        let ctx = DocumentContext::new(&fm, "Some markdown with ENABLE_CUSTOM here");

        let resolved = resolve_enabled_features(&features, &ctx);
        assert!(!resolved.contains("default_feat"));
        assert!(resolved.contains("custom_feat"));
    }

    #[test]
    fn test_resolve_single_level_dependency() {
        let features: Vec<Box<dyn Feature>> = vec![
            Box::new(FeatureA),
            Box::new(FeatureB),
        ];

        let mut fm = HashMap::new();
        fm.insert("enable_a".to_string(), "1".to_string());
        let ctx = DocumentContext::new(&fm, "markdown");

        let resolved = resolve_enabled_features(&features, &ctx);
        assert!(resolved.contains("feat_a"));
        assert!(resolved.contains("feat_b"));
    }

    #[test]
    fn test_resolve_transitive_dependencies() {
        let features: Vec<Box<dyn Feature>> = vec![
            Box::new(FeatureA),
            Box::new(FeatureB),
            Box::new(FeatureC),
        ];

        let mut fm = HashMap::new();
        fm.insert("enable_a".to_string(), "1".to_string());
        let ctx = DocumentContext::new(&fm, "markdown");

        let resolved = resolve_enabled_features(&features, &ctx);
        assert!(resolved.contains("feat_a"));
        assert!(resolved.contains("feat_b"));
        assert!(resolved.contains("feat_c"));
    }

    #[test]
    fn test_resolve_cyclic_dependencies() {
        let features: Vec<Box<dyn Feature>> = vec![
            Box::new(CyclicFeatureX),
            Box::new(CyclicFeatureY),
        ];

        let fm = HashMap::new();
        let ctx = DocumentContext::new(&fm, "");

        let resolved = resolve_enabled_features(&features, &ctx);
        assert_eq!(resolved.len(), 2);
        assert!(resolved.contains("cyclic_x"));
        assert!(resolved.contains("cyclic_y"));
    }

    #[test]
    fn test_resolve_unregistered_dependency() {
        let features: Vec<Box<dyn Feature>> = vec![
            Box::new(UnregisteredDepFeature),
        ];

        let fm = HashMap::new();
        let ctx = DocumentContext::new(&fm, "");

        let resolved = resolve_enabled_features(&features, &ctx);
        assert_eq!(resolved.len(), 2);
        assert!(resolved.contains("unregistered_parent"));
        assert!(resolved.contains("ghost_feature"));
    }
}
