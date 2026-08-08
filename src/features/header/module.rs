//! Document header and flexible banner layout feature slice.

use crate::core::feature::{DocumentContext, Feature};
use std::fmt::Write;

/// Renders a section-style flexible header card containing title, subtitle, and logo.
///
/// # Examples
///
/// ```
/// use doc2flow::features::header::render_flex_header;
///
/// let mut buf = String::new();
/// render_flex_header(&mut buf, "My Title", Some("My Subtitle"), "<svg></svg>");
/// assert!(buf.contains("My Title"));
/// assert!(buf.contains("My Subtitle"));
/// ```
#[inline]
pub fn render_flex_header(
    out: &mut impl Write,
    title: &str,
    subtitle: Option<&str>,
    logo_html: &str,
) {
    let escaped_title = crate::converter::html_escape(title);
    let _ = out.write_str("<section class=\"section header-flex\" id=\"header-flex\">\n  <div class=\"header-flex-top\">\n    <div class=\"header-flex-titles\">\n      <h1 class=\"header-flex-title\">");
    let _ = out.write_str(&escaped_title);
    let _ = out.write_str("</h1>\n");

    if let Some(sub) = subtitle
        && !sub.trim().is_empty()
    {
        let escaped_sub = crate::converter::html_escape(sub);
        let _ = out.write_str("      <div class=\"header-flex-sub\">");
        let _ = out.write_str(&escaped_sub);
        let _ = out.write_str("</div>\n");
    }

    let _ = out.write_str("    </div>\n    <div class=\"header-flex-logo\">\n      ");
    let _ = out.write_str(logo_html);
    let _ = out.write_str("\n    </div>\n  </div>\n</section>\n\n");
}

/// Unified header feature slice providing top-level flexible banner container styling.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct HeaderFeature;

impl HeaderFeature {
    /// Creates a new header feature instance.
    ///
    /// # Examples
    ///
    /// ```
    /// use doc2flow::features::header::HeaderFeature;
    ///
    /// let feature = HeaderFeature::new();
    /// ```
    #[inline]
    pub const fn new() -> Self {
        Self
    }
}

impl Feature for HeaderFeature {
    /// Returns the unique feature identifier "header".
    #[inline]
    fn name(&self) -> &'static str {
        "header"
    }

    /// Evaluates if the header feature is enabled based on frontmatter option `header: "flex"`.
    #[inline]
    fn is_enabled(&self, ctx: &DocumentContext) -> bool {
        ctx.frontmatter.get("header").is_some_and(|val| {
            let trimmed = val.trim().trim_matches('"').trim_matches('\'');
            trimmed.eq_ignore_ascii_case("flex")
        })
    }

    /// Returns optional CSS stylesheet rules for flexible header layout.
    #[inline]
    fn css(&self) -> Option<&'static str> {
        Some(include_str!("header.css"))
    }

    /// Returns embedded JavaScript code for this feature, which is none for static headers.
    #[inline]
    fn javascript(&self) -> Option<&'static str> {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    #[test]
    fn test_header_feature_activation_logic() {
        let feature = HeaderFeature::new();
        let mut fm = HashMap::new();

        // 1. Missing header key: disabled
        let ctx_empty = DocumentContext::new(&fm, "# Heading");
        assert!(!feature.is_enabled(&ctx_empty));

        // 2. Explicit header: "none": disabled
        fm.insert("header".to_string(), "none".to_string());
        let ctx_none = DocumentContext::new(&fm, "# Heading");
        assert!(!feature.is_enabled(&ctx_none));

        // 3. Explicit header: "flex": enabled
        fm.insert("header".to_string(), "flex".to_string());
        let ctx_flex = DocumentContext::new(&fm, "# Heading");
        assert!(feature.is_enabled(&ctx_flex));

        // 4. Quoted header: "\"flex\"": enabled
        fm.insert("header".to_string(), "\"flex\"".to_string());
        let ctx_quoted = DocumentContext::new(&fm, "# Heading");
        assert!(feature.is_enabled(&ctx_quoted));

        // 5. Uppercase header: "FLEX": enabled
        fm.insert("header".to_string(), "FLEX".to_string());
        let ctx_upper = DocumentContext::new(&fm, "# Heading");
        assert!(feature.is_enabled(&ctx_upper));
    }

    #[test]
    fn test_header_assets_embedded() {
        let feature = HeaderFeature::new();
        assert_eq!(feature.name(), "header");
        assert_eq!(feature.javascript(), None);
        let css = feature.css().expect("CSS must be embedded");
        assert!(css.contains(".header-flex"));
        assert!(css.contains(".header-flex-top"));
        assert!(css.contains(".doc-header .header-top"));
    }

    #[test]
    fn test_render_flex_header_output() {
        let mut buf = String::new();
        render_flex_header(
            &mut buf,
            "System Guide",
            Some("Maintenance SOP"),
            "<svg id=\"logo\"></svg>",
        );

        assert!(buf.contains("<section class=\"section header-flex\" id=\"header-flex\">"));
        assert!(buf.contains("<h1 class=\"header-flex-title\">System Guide</h1>"));
        assert!(buf.contains("<div class=\"header-flex-sub\">Maintenance SOP</div>"));
        assert!(buf.contains("<svg id=\"logo\"></svg>"));

        let mut buf_no_sub = String::new();
        render_flex_header(&mut buf_no_sub, "Title Only", None, "<svg></svg>");
        assert!(buf_no_sub.contains("<h1 class=\"header-flex-title\">Title Only</h1>"));
        assert!(!buf_no_sub.contains("header-flex-sub"));
    }
}
