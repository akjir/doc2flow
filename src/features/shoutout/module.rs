//! Shoutout and callout vertical slice feature module.

use crate::core::document::{
    DocumentElement, DocumentElementId, DocumentParameters, ShoutoutElementKind,
};
use crate::core::feature::FeatureModule;
use crate::core::format::{escape_html_into, format_inline_into, push_indent};
use crate::core::language::localize;
use crate::core::renderer::{DocumentElementRenderer, HtmlRenderer};

/// Embedded shoutout CSS stylesheet.
pub const CSS: &str = include_str!("shoutout.css");

/// Supported document element identifiers for shoutout elements.
const SHOUTOUT_SUPPORTED: [DocumentElementId; 1] = [DocumentElementId::Shoutout];

/// Shoutout feature renderer handling callout and shoutout panel elements.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct ShoutoutFeature;

impl ShoutoutFeature {
    /// Creates a new shoutout feature instance.
    ///
    /// # Examples
    ///
    /// ```
    /// use doc2flow::features::shoutout::ShoutoutFeature;
    ///
    /// let feature = ShoutoutFeature::new();
    /// ```
    #[must_use]
    pub fn new() -> Self {
        Self
    }
}

/// Returns the CSS class string for a given shoutout element kind.
#[must_use]
pub const fn shoutout_class(kind: ShoutoutElementKind) -> &'static str {
    match kind {
        ShoutoutElementKind::Caution => "shoutout shoutout-caution",
        ShoutoutElementKind::Important => "shoutout shoutout-important",
        ShoutoutElementKind::Note => "shoutout shoutout-note",
        ShoutoutElementKind::Tip => "shoutout shoutout-tip",
        ShoutoutElementKind::Warning => "shoutout shoutout-warning",
    }
}

/// Returns the translation dictionary key for a given shoutout element kind.
#[must_use]
pub const fn shoutout_translation_key(kind: ShoutoutElementKind) -> &'static str {
    match kind {
        ShoutoutElementKind::Caution => "callout_caution",
        ShoutoutElementKind::Important => "callout_important",
        ShoutoutElementKind::Note => "callout_note",
        ShoutoutElementKind::Tip => "callout_tip",
        ShoutoutElementKind::Warning => "callout_warning",
    }
}

impl DocumentElementRenderer for ShoutoutFeature {
    fn supported(&self) -> &[DocumentElementId] {
        &SHOUTOUT_SUPPORTED
    }

    fn render_element(
        &self,
        element: &DocumentElement,
        indent: usize,
        _depth: usize,
        _parameters: &DocumentParameters,
        out: &mut String,
        _renderer: &HtmlRenderer,
    ) {
        if let DocumentElement::Shoutout { content, kind } = element {
            push_indent(out, indent);
            let cls = shoutout_class(*kind);
            let key = shoutout_translation_key(*kind);
            let label = localize(key);
            out.push_str("<div class=\"");
            out.push_str(cls);
            out.push_str("\" data-label=\"");
            escape_html_into(out, &label);
            out.push_str("\">\n");

            for line in content.lines() {
                push_indent(out, indent + 1);
                format_inline_into(out, line);
                out.push('\n');
            }

            push_indent(out, indent);
            out.push_str("</div>\n");
        }
    }
}

impl FeatureModule for ShoutoutFeature {
    fn name(&self) -> &'static str {
        "shoutout"
    }

    fn css(&self) -> Option<&'static str> {
        Some(CSS)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shoutout_feature_constructor_new() {
        let feature = ShoutoutFeature::new();
        assert_eq!(feature, ShoutoutFeature);
    }

    #[test]
    fn test_shoutout_feature_metadata() {
        let feature = ShoutoutFeature::new();
        assert_eq!(feature.name(), "shoutout");
        assert!(feature.css().is_some());
        assert_eq!(feature.javascript(), &[] as &[&str]);
        assert_eq!(feature.supported(), &[DocumentElementId::Shoutout]);
    }

    #[test]
    fn test_shoutout_feature_css_tokens_and_classes() {
        let feature = ShoutoutFeature::new();
        let css = feature.css().expect("shoutout css should exist");
        assert!(css.contains("--shoutout-note-bg:"));
        assert!(css.contains("--shoutout-tip-bg:"));
        assert!(css.contains("--shoutout-important-bg:"));
        assert!(css.contains("--shoutout-warning-bg:"));
        assert!(css.contains("--shoutout-caution-bg:"));
        assert!(css.contains(".shoutout"));
        assert!(css.contains(".shoutout-note"));
        assert!(css.contains(".shoutout-tip"));
        assert!(css.contains(".shoutout-important"));
        assert!(css.contains(".shoutout-warning"));
        assert!(css.contains(".shoutout-caution"));
        assert!(css.contains("content: attr(data-label)"));
    }

    #[test]
    fn test_shoutout_classes_for_all_kinds() {
        assert_eq!(
            shoutout_class(ShoutoutElementKind::Caution),
            "shoutout shoutout-caution"
        );
        assert_eq!(
            shoutout_class(ShoutoutElementKind::Important),
            "shoutout shoutout-important"
        );
        assert_eq!(
            shoutout_class(ShoutoutElementKind::Note),
            "shoutout shoutout-note"
        );
        assert_eq!(
            shoutout_class(ShoutoutElementKind::Tip),
            "shoutout shoutout-tip"
        );
        assert_eq!(
            shoutout_class(ShoutoutElementKind::Warning),
            "shoutout shoutout-warning"
        );
    }

    #[test]
    fn test_shoutout_translation_keys_for_all_kinds() {
        assert_eq!(
            shoutout_translation_key(ShoutoutElementKind::Caution),
            "callout_caution"
        );
        assert_eq!(
            shoutout_translation_key(ShoutoutElementKind::Important),
            "callout_important"
        );
        assert_eq!(
            shoutout_translation_key(ShoutoutElementKind::Note),
            "callout_note"
        );
        assert_eq!(
            shoutout_translation_key(ShoutoutElementKind::Tip),
            "callout_tip"
        );
        assert_eq!(
            shoutout_translation_key(ShoutoutElementKind::Warning),
            "callout_warning"
        );
    }

    #[test]
    fn test_shoutout_empty_for_unsupported_elements() {
        let feature = ShoutoutFeature::new();
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

    #[test]
    fn test_shoutout_renders_note_english() {
        let _guard = crate::core::language::TEST_I18N_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        crate::core::language::init("en");
        let feature = ShoutoutFeature::new();
        let element = DocumentElement::shoutout(ShoutoutElementKind::Note, "Standard note message");
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
        let expected = concat!(
            "  <div class=\"shoutout shoutout-note\" data-label=\"Note\">\n",
            "    Standard note message\n",
            "  </div>\n"
        );
        assert_eq!(out, expected);
    }

    #[test]
    fn test_shoutout_renders_tip_german() {
        let _guard = crate::core::language::TEST_I18N_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        crate::core::language::init("de");
        let feature = ShoutoutFeature::new();
        let element = DocumentElement::shoutout(ShoutoutElementKind::Tip, "Ein nützlicher Tipp");
        let mut out = String::new();
        let renderer = HtmlRenderer::default_renderer();
        let params = DocumentParameters {
            language: "de".to_string(),
            ..Default::default()
        };
        feature.render_element(&element, 2, 0, &params, &mut out, &renderer);
        let expected = concat!(
            "    <div class=\"shoutout shoutout-tip\" data-label=\"Tipp\">\n",
            "      Ein nützlicher Tipp\n",
            "    </div>\n"
        );
        assert_eq!(out, expected);
    }

    #[test]
    fn test_shoutout_renders_all_kinds_german() {
        let _guard = crate::core::language::TEST_I18N_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        crate::core::language::init("de");
        let feature = ShoutoutFeature::new();
        let renderer = HtmlRenderer::default_renderer();
        let params = DocumentParameters {
            language: "de".to_string(),
            ..DocumentParameters::default()
        };

        let kinds = [
            (ShoutoutElementKind::Note, "shoutout-note", "Hinweis"),
            (ShoutoutElementKind::Tip, "shoutout-tip", "Tipp"),
            (ShoutoutElementKind::Important, "shoutout-important", "Wichtig"),
            (ShoutoutElementKind::Warning, "shoutout-warning", "Warnung"),
            (ShoutoutElementKind::Caution, "shoutout-caution", "Achtung"),
        ];

        for (kind, expected_cls, expected_label) in kinds {
            let element = DocumentElement::shoutout(kind, "Text");
            let mut out = String::new();
            feature.render_element(&element, 0, 0, &params, &mut out, &renderer);
            let expected = format!(
                "<div class=\"shoutout {expected_cls}\" data-label=\"{expected_label}\">\n  Text\n</div>\n"
            );
            assert_eq!(out, expected);
        }
    }

    #[test]
    fn test_shoutout_renders_all_kinds_english_default() {
        let _guard = crate::core::language::TEST_I18N_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        crate::core::language::init("en");
        let feature = ShoutoutFeature::new();
        let renderer = HtmlRenderer::default_renderer();
        let params = DocumentParameters::default();

        let kinds = [
            (ShoutoutElementKind::Note, "shoutout-note", "Note"),
            (ShoutoutElementKind::Tip, "shoutout-tip", "Tip"),
            (ShoutoutElementKind::Important, "shoutout-important", "Important"),
            (ShoutoutElementKind::Warning, "shoutout-warning", "Warning"),
            (ShoutoutElementKind::Caution, "shoutout-caution", "Caution"),
        ];

        for (kind, expected_cls, expected_label) in kinds {
            let element = DocumentElement::shoutout(kind, "Text");
            let mut out = String::new();
            feature.render_element(&element, 0, 0, &params, &mut out, &renderer);
            let expected = format!(
                "<div class=\"shoutout {expected_cls}\" data-label=\"{expected_label}\">\n  Text\n</div>\n"
            );
            assert_eq!(out, expected);
        }
    }

    #[test]
    fn test_shoutout_renders_unknown_language_placeholder() {
        let _guard = crate::core::language::TEST_I18N_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        crate::core::language::init("fr");
        let feature = ShoutoutFeature::new();
        let renderer = HtmlRenderer::default_renderer();
        let params = DocumentParameters {
            language: "fr".to_string(),
            ..DocumentParameters::default()
        };
        let element = DocumentElement::shoutout(ShoutoutElementKind::Note, "Texte");
        let mut out = String::new();
        feature.render_element(&element, 0, 0, &params, &mut out, &renderer);
        let expected = "<div class=\"shoutout shoutout-note\" data-label=\"{{callout_note}}\">\n  Texte\n</div>\n";
        assert_eq!(out, expected);
    }

    #[test]
    fn test_shoutout_renders_inline_formatting() {
        let _guard = crate::core::language::TEST_I18N_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        crate::core::language::init("en");
        let feature = ShoutoutFeature::new();
        let element = DocumentElement::shoutout(
            ShoutoutElementKind::Important,
            "Notice with **bold**, *italic*, ~~strike~~, `code`, and [link](https://example.com) span",
        );
        let mut out = String::new();
        let renderer = HtmlRenderer::default_renderer();
        feature.render_element(
            &element,
            0,
            0,
            &DocumentParameters::default(),
            &mut out,
            &renderer,
        );
        let expected = concat!(
            "<div class=\"shoutout shoutout-important\" data-label=\"Important\">\n",
            "  Notice with <strong>bold</strong>, <em>italic</em>, <s>strike</s>, <code>code</code>, and <a href=\"https://example.com\">link</a> span\n",
            "</div>\n"
        );
        assert_eq!(out, expected);
    }

    #[test]
    fn test_shoutout_renders_multiline() {
        let _guard = crate::core::language::TEST_I18N_LOCK.lock().unwrap_or_else(|e| e.into_inner());
        crate::core::language::init("en");
        let feature = ShoutoutFeature::new();
        let element = DocumentElement::shoutout(
            ShoutoutElementKind::Warning,
            "Line 1 warning\nLine 2 warning",
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
        let expected = concat!(
            "  <div class=\"shoutout shoutout-warning\" data-label=\"Warning\">\n",
            "    Line 1 warning\n",
            "    Line 2 warning\n",
            "  </div>\n"
        );
        assert_eq!(out, expected);
    }
}
