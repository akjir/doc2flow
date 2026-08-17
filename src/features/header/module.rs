//! Document top header banner vertical slice feature module.

use crate::core::document::{DocumentElement, DocumentElementId, DocumentParameters};
use crate::core::feature::FeatureModule;
use crate::core::format::{escape_html_into, push_indent};
use crate::core::renderer::{DocumentElementRenderer, HtmlRenderer};

/// Embedded header CSS stylesheet.
pub const CSS: &str = include_str!("header.css");

/// Embedded default application logo SVG markup.
pub const DEFAULT_LOGO_SVG: &str = include_str!("../../../resources/images/logo.svg");

/// Supported document element identifiers for header elements.
const HEADER_SUPPORTED: [DocumentElementId; 1] = [DocumentElementId::Header];

/// Returns true if the provided string is a raw SVG markup payload rather than an image path.
#[must_use]
fn is_svg_payload(src: &str) -> bool {
    let trimmed = src.trim_start();
    trimmed.starts_with("<svg")
        || ((trimmed.starts_with("<?xml")
            || trimmed.starts_with("<!DOCTYPE")
            || trimmed.starts_with("<!--"))
            && trimmed.contains("<svg"))
}

/// Header feature renderer handling top document banner card layout.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct HeaderFeature;

impl HeaderFeature {
    /// Creates a new header feature instance.
    ///
    /// # Examples
    ///
    /// ```
    /// use doc2flow::features::HeaderFeature;
    ///
    /// let feature = HeaderFeature::new();
    /// ```
    #[must_use]
    pub fn new() -> Self {
        Self
    }
}

impl DocumentElementRenderer for HeaderFeature {
    fn supported(&self) -> &[DocumentElementId] {
        &HEADER_SUPPORTED
    }

    fn render_element(
        &self,
        element: &DocumentElement,
        indent: usize,
        _depth: usize,
        parameters: &DocumentParameters,
        out: &mut String,
        _renderer: &HtmlRenderer,
    ) {
        if let DocumentElement::Header {
            title,
            subtitle,
            logo,
        } = element
        {
            push_indent(out, indent);
            out.push_str("<section class=\"section header-container\" id=\"header\">\n");

            push_indent(out, indent + 1);
            out.push_str("<div class=\"header-top\">\n");

            push_indent(out, indent + 2);
            out.push_str("<div class=\"header-titles\">\n");

            push_indent(out, indent + 3);
            out.push_str("<h1 class=\"header-title\">");
            escape_html_into(out, title);
            out.push_str("</h1>\n");

            if let Some(sub) = subtitle.as_deref().filter(|s| !s.trim().is_empty()) {
                push_indent(out, indent + 3);
                out.push_str("<div class=\"header-sub\">");
                escape_html_into(out, sub);
                out.push_str("</div>\n");
            }

            push_indent(out, indent + 2);
            out.push_str("</div>\n");

            push_indent(out, indent + 2);
            out.push_str("<div class=\"header-logo\">\n");

            let logo_val = logo
                .as_deref()
                .filter(|s| !s.trim().is_empty())
                .or_else(|| Some(parameters.logo.as_str()).filter(|s| !s.trim().is_empty()));

            push_indent(out, indent + 3);
            if let Some(logo_src) = logo_val {
                if is_svg_payload(logo_src) {
                    out.push_str(logo_src);
                } else {
                    out.push_str("<img src=\"");
                    escape_html_into(out, logo_src);
                    out.push_str("\" alt=\"");
                    escape_html_into(out, title);
                    out.push_str("\" />");
                }
            } else {
                out.push_str(DEFAULT_LOGO_SVG);
            }
            out.push('\n');

            push_indent(out, indent + 2);
            out.push_str("</div>\n");

            push_indent(out, indent + 1);
            out.push_str("</div>\n");

            push_indent(out, indent);
            out.push_str("</section>\n");
        }
    }
}

impl FeatureModule for HeaderFeature {
    fn name(&self) -> &'static str {
        "header"
    }

    fn css(&self) -> Option<&'static str> {
        Some(CSS)
    }

    fn javascript(&self) -> &[&'static str] {
        &[]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_header_feature_constructor_new() {
        let feature = HeaderFeature::new();
        assert_eq!(feature, HeaderFeature);
    }

    #[test]
    fn test_header_feature_metadata_and_css() {
        let feature = HeaderFeature::new();
        assert_eq!(feature.name(), "header");
        assert_eq!(feature.javascript(), &[] as &[&str]);
        let css = feature.css().expect("header CSS must exist");
        assert!(css.contains(".header-container"));
        assert!(css.contains(".header-top"));
        assert!(css.contains(".header-titles"));
        assert!(css.contains(".header-title"));
        assert!(css.contains(".header-sub"));
        assert!(css.contains(".header-logo"));
    }

    #[test]
    fn test_header_feature_renders_title_only() {
        let feature = HeaderFeature::new();
        let element = DocumentElement::header("System Guide", None::<String>, None::<String>);
        let mut out = String::new();
        let renderer = HtmlRenderer::default_renderer();
        feature.render_element(
            &element,
            1,
            0,
            &DocumentParameters::default(),
            &mut out,
            &renderer,
        );

        assert!(out.contains("<section class=\"section header-container\" id=\"header\">"));
        assert!(out.contains("<h1 class=\"header-title\">System Guide</h1>"));
        assert!(!out.contains("header-sub"));
        assert!(out.contains("<svg"));
    }

    #[test]
    fn test_header_feature_renders_title_and_subtitle() {
        let feature = HeaderFeature::new();
        let element = DocumentElement::header(
            "System Guide",
            Some("Maintenance SOP <v1.0>"),
            None::<String>,
        );
        let mut out = String::new();
        let renderer = HtmlRenderer::default_renderer();
        feature.render_element(
            &element,
            2,
            0,
            &DocumentParameters::default(),
            &mut out,
            &renderer,
        );

        assert!(out.contains("    <section class=\"section header-container\" id=\"header\">"));
        assert!(out.contains("<h1 class=\"header-title\">System Guide</h1>"));
        assert!(out.contains("<div class=\"header-sub\">Maintenance SOP &lt;v1.0&gt;</div>"));
        assert!(out.contains("<svg"));
    }

    #[test]
    fn test_header_feature_renders_custom_svg_logo() {
        let feature = HeaderFeature::new();
        let element = DocumentElement::header(
            "Custom Logo",
            None::<String>,
            Some("<svg id=\"custom\"></svg>"),
        );
        let mut out = String::new();
        let renderer = HtmlRenderer::default_renderer();
        feature.render_element(
            &element,
            1,
            0,
            &DocumentParameters::default(),
            &mut out,
            &renderer,
        );

        assert!(out.contains("<svg id=\"custom\"></svg>"));
        assert!(!out.contains(DEFAULT_LOGO_SVG));
    }

    #[test]
    fn test_header_feature_renders_custom_image_logo() {
        let feature = HeaderFeature::new();
        let element =
            DocumentElement::header("Image Logo", None::<String>, Some("assets/brand.png"));
        let mut out = String::new();
        let renderer = HtmlRenderer::default_renderer();
        feature.render_element(
            &element,
            1,
            0,
            &DocumentParameters::default(),
            &mut out,
            &renderer,
        );

        assert!(out.contains("<img src=\"assets/brand.png\" alt=\"Image Logo\" />"));
        assert!(!out.contains(DEFAULT_LOGO_SVG));
    }

    #[test]
    fn test_header_feature_falls_back_to_parameter_logo() {
        let feature = HeaderFeature::new();
        let element = DocumentElement::header("Param Logo", None::<String>, None::<String>);
        let mut params = DocumentParameters::default();
        params.logo = "images/param_logo.svg".into();
        let mut out = String::new();
        let renderer = HtmlRenderer::default_renderer();
        feature.render_element(&element, 1, 0, &params, &mut out, &renderer);

        assert!(out.contains("<img src=\"images/param_logo.svg\" alt=\"Param Logo\" />"));
    }

    #[test]
    fn test_header_feature_handles_svg_with_xml_prolog() {
        let feature = HeaderFeature::new();
        let svg_with_prolog =
            "<?xml version=\"1.0\" encoding=\"utf-8\"?><svg id=\"prolog\"><circle r=\"10\"/></svg>";
        let element =
            DocumentElement::header("Prolog Logo", None::<String>, Some(svg_with_prolog));
        let mut out = String::new();
        let renderer = HtmlRenderer::default_renderer();
        feature.render_element(
            &element,
            1,
            0,
            &DocumentParameters::default(),
            &mut out,
            &renderer,
        );

        assert!(out.contains(svg_with_prolog));
        assert!(!out.contains("<img"));
        assert!(!out.contains(DEFAULT_LOGO_SVG));
    }

    #[test]
    fn test_header_feature_parameter_logo_whitespace_fallback() {
        let feature = HeaderFeature::new();
        let element = DocumentElement::header("Blank Logo", None::<String>, None::<String>);
        let mut params = DocumentParameters::default();
        params.logo = "   \t\n  ".into();
        let mut out = String::new();
        let renderer = HtmlRenderer::default_renderer();
        feature.render_element(&element, 1, 0, &params, &mut out, &renderer);

        assert!(out.contains(DEFAULT_LOGO_SVG));
        assert!(!out.contains("<img"));
    }

    #[test]
    fn test_header_feature_empty_for_unsupported_elements() {
        let feature = HeaderFeature::new();
        let text = DocumentElement::text("Regular text");
        let mut out = String::new();
        let renderer = HtmlRenderer::default_renderer();
        feature.render_element(
            &text,
            0,
            0,
            &DocumentParameters::default(),
            &mut out,
            &renderer,
        );
        assert!(out.is_empty());
    }
}
