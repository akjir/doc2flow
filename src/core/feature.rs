//! Document AST feature detection, inspection, and rendering traits.

use std::fmt::{self, Display, Formatter};
use std::ops::{BitAnd, BitAndAssign, BitOr, BitOrAssign, Not};

use crate::core::document::{Document, DocumentElement, DocumentHeader, DocumentParameters};
use crate::core::renderer::HtmlRenderer;

/// Static mapping of feature flags to their canonical display identifiers.
const FEATURE_NAMES: [(DocumentFeature, &str); 8] = [
    (DocumentFeature::BULLET, "bullet"),
    (DocumentFeature::CODE, "code"),
    (DocumentFeature::IMAGE, "image"),
    (DocumentFeature::ORDERED, "ordered"),
    (DocumentFeature::SHOUTOUT, "shoutout"),
    (DocumentFeature::TABLE, "table"),
    (DocumentFeature::TASK, "task"),
    (DocumentFeature::UNKNOWN, "unknown"),
];

/// Feature detection flags for document AST elements represented as a bitmask.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct DocumentFeature {
    bits: u16,
}

impl DocumentFeature {
    /// Mask representing all supported document features combined.
    pub const ALL: Self = Self { bits: (1 << 8) - 1 };
    /// Flag indicating bullet lists are present.
    pub const BULLET: Self = Self { bits: 1 << 0 };
    /// Flag indicating code blocks are present.
    pub const CODE: Self = Self { bits: 1 << 1 };
    /// Flag indicating images are present.
    pub const IMAGE: Self = Self { bits: 1 << 2 };
    /// Empty feature set with no flags enabled.
    pub const NONE: Self = Self { bits: 0 };
    /// Flag indicating ordered lists are present.
    pub const ORDERED: Self = Self { bits: 1 << 3 };
    /// Flag indicating shoutout callouts are present.
    pub const SHOUTOUT: Self = Self { bits: 1 << 4 };
    /// Flag indicating tables are present.
    pub const TABLE: Self = Self { bits: 1 << 5 };
    /// Flag indicating task items are present.
    pub const TASK: Self = Self { bits: 1 << 6 };
    /// Flag indicating unrecognized unknown elements are present.
    pub const UNKNOWN: Self = Self { bits: 1 << 7 };

    /// Creates a new default feature set with no document features enabled.
    ///
    /// # Examples
    ///
    /// ```
    /// use doc2flow::core::feature::DocumentFeature;
    ///
    /// let features = DocumentFeature::new();
    /// assert!(features.is_empty());
    /// ```
    #[must_use]
    pub const fn new() -> Self {
        Self::NONE
    }

    /// Returns `true` if this feature set contains the specified feature flag.
    ///
    /// # Examples
    ///
    /// ```
    /// use doc2flow::core::feature::DocumentFeature;
    ///
    /// let features = DocumentFeature::CODE | DocumentFeature::TABLE;
    /// assert!(features.contains(DocumentFeature::CODE));
    /// assert!(!features.contains(DocumentFeature::BULLET));
    /// ```
    #[must_use]
    pub const fn contains(self, other: Self) -> bool {
        (self.bits & other.bits) == other.bits
    }

    /// Inserts the specified feature flag into this feature set.
    ///
    /// # Examples
    ///
    /// ```
    /// use doc2flow::core::feature::DocumentFeature;
    ///
    /// let mut features = DocumentFeature::default();
    /// features.insert(DocumentFeature::BULLET);
    /// assert!(features.contains(DocumentFeature::BULLET));
    /// ```
    pub fn insert(&mut self, other: Self) {
        self.bits |= other.bits;
    }

    /// Returns `true` if all feature flags are enabled.
    #[must_use]
    pub const fn is_all(self) -> bool {
        (self.bits & Self::ALL.bits) == Self::ALL.bits
    }

    /// Returns `true` if no feature flags are enabled.
    #[must_use]
    pub const fn is_empty(self) -> bool {
        self.bits == 0
    }

    /// Removes the specified feature flag from this feature set.
    ///
    /// # Examples
    ///
    /// ```
    /// use doc2flow::core::feature::DocumentFeature;
    ///
    /// let mut features = DocumentFeature::ALL;
    /// features.remove(DocumentFeature::IMAGE);
    /// assert!(!features.contains(DocumentFeature::IMAGE));
    /// ```
    pub fn remove(&mut self, other: Self) {
        self.bits &= !other.bits;
    }
}

impl BitAnd for DocumentFeature {
    type Output = Self;

    fn bitand(self, rhs: Self) -> Self::Output {
        Self {
            bits: self.bits & rhs.bits,
        }
    }
}

impl BitAndAssign for DocumentFeature {
    fn bitand_assign(&mut self, rhs: Self) {
        self.bits &= rhs.bits;
    }
}

impl BitOr for DocumentFeature {
    type Output = Self;

    fn bitor(self, rhs: Self) -> Self::Output {
        Self {
            bits: self.bits | rhs.bits,
        }
    }
}

impl BitOrAssign for DocumentFeature {
    fn bitor_assign(&mut self, rhs: Self) {
        self.bits |= rhs.bits;
    }
}

impl Display for DocumentFeature {
    /// Formats active features as a comma-separated list.
    ///
    /// # Examples
    ///
    /// ```
    /// use doc2flow::core::feature::DocumentFeature;
    ///
    /// let features = DocumentFeature::default();
    /// assert_eq!(features.to_string(), "core");
    /// ```
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_str("core")?;
        for &(flag, name) in &FEATURE_NAMES {
            if self.contains(flag) {
                f.write_str(", ")?;
                f.write_str(name)?;
            }
        }
        Ok(())
    }
}

impl From<&Document> for DocumentFeature {
    /// Detects active features present within a [`Document`].
    ///
    /// # Examples
    ///
    /// ```
    /// use doc2flow::core::document::{Document, DocumentElement};
    /// use doc2flow::core::feature::DocumentFeature;
    ///
    /// let mut doc = Document::new();
    /// doc.push_body(DocumentElement::code_block(Some("rust"), "fn main() {}"));
    /// let features = DocumentFeature::from(&doc);
    /// assert!(features.contains(DocumentFeature::CODE));
    /// assert!(!features.contains(DocumentFeature::TABLE));
    /// ```
    fn from(doc: &Document) -> Self {
        let mut features = Self::default();
        if let Some(ref variables) = doc.header.variables {
            features.insert(DocumentFeature::CODE);
            scan_element(variables, &mut features);
        }
        scan_elements(&doc.body, &mut features);
        features
    }
}

impl Not for DocumentFeature {
    type Output = Self;

    fn not(self) -> Self::Output {
        Self {
            bits: !self.bits & Self::ALL.bits,
        }
    }
}

/// Trait for document feature renderers converting AST elements to HTML.
pub trait Feature: Send + Sync + fmt::Debug {
    /// Returns optional CSS stylesheet rules for this feature, defaulting to `None`.
    fn css(&self) -> Option<&'static str> {
        None
    }

    /// Returns optional JavaScript client logic files for this feature, defaulting to an empty slice.
    fn javascript(&self) -> &[&'static str] {
        &[]
    }

    /// Intercepts the rendering of a document element into an output buffer.
    ///
    /// Returns `true` if this feature handled rendering the element, or `false`
    /// to delegate to subsequent features or core fallback rendering.
    fn try_render_body(
        &self,
        _element: &DocumentElement,
        _indent: usize,
        _depth: usize,
        _parameters: &DocumentParameters,
        _out: &mut String,
        _renderer: &HtmlRenderer,
    ) -> bool {
        false
    }

    /// Intercepts the rendering of a document header.
    ///
    /// Returns `true` if this feature handled rendering the header, or `false`
    /// to delegate to subsequent features or core fallback rendering.
    fn try_render_header(
        &self,
        _header: &DocumentHeader,
        _parameters: &DocumentParameters,
    ) -> bool {
        false
    }
}

/// Recursively scans an individual document element and its children.
fn scan_element(element: &DocumentElement, features: &mut DocumentFeature) {
    if features.is_all() {
        return;
    }
    match element {
        DocumentElement::BlockDirective { children, .. }
        | DocumentElement::Section { children, .. } => {
            scan_elements(children, features);
        }
        DocumentElement::BulletListItem { children, .. } => {
            features.insert(DocumentFeature::BULLET);
            scan_elements(children, features);
        }
        DocumentElement::CheckBoxItem { children, .. } => {
            features.insert(DocumentFeature::TASK);
            scan_elements(children, features);
        }
        DocumentElement::CodeBlock { .. } => {
            features.insert(DocumentFeature::CODE);
        }
        DocumentElement::Image { .. } => {
            features.insert(DocumentFeature::IMAGE);
        }
        DocumentElement::OrderedListItem { children, .. } => {
            features.insert(DocumentFeature::ORDERED);
            scan_elements(children, features);
        }
        DocumentElement::Shoutout { .. } => {
            features.insert(DocumentFeature::SHOUTOUT);
        }
        DocumentElement::Table { .. } => {
            features.insert(DocumentFeature::TABLE);
        }
        DocumentElement::HorizontalRule | DocumentElement::Text(_) => {}
        DocumentElement::Unknown(_) => {
            features.insert(DocumentFeature::UNKNOWN);
        }
    }
}

/// Recursively scans a slice of document elements and updates active feature flags.
fn scan_elements(elements: &[DocumentElement], features: &mut DocumentFeature) {
    for element in elements {
        if features.is_all() {
            return;
        }
        scan_element(element, features);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::document::{ShoutoutElementKind, TableAlignment};

    #[test]
    fn test_all_features_present() {
        let mut doc = Document::new();
        doc.push_body(DocumentElement::bullet_list_item("Bullet"));
        doc.push_body(DocumentElement::check_box_item(true, "Task"));
        doc.push_body(DocumentElement::code_block(Some("rust"), "fn main() {}"));
        doc.push_body(DocumentElement::image("alt", "image.png"));
        doc.push_body(DocumentElement::ordered_list_item(1, "Ordered"));
        doc.push_body(DocumentElement::section(
            1,
            "Section 1",
            vec![DocumentElement::unknown("Unknown")],
        ));
        doc.push_body(DocumentElement::shoutout(
            ShoutoutElementKind::Note,
            "Note content",
        ));
        doc.push_body(DocumentElement::table(
            vec![TableAlignment::Left],
            vec![vec!["Cell".into()]],
        ));

        let features = DocumentFeature::from(&doc);
        assert!(!features.is_empty());
        assert!(features.is_all());
        assert!(features.contains(DocumentFeature::BULLET));
        assert!(features.contains(DocumentFeature::TASK));
        assert!(features.contains(DocumentFeature::CODE));
        assert!(features.contains(DocumentFeature::IMAGE));
        assert!(features.contains(DocumentFeature::ORDERED));
        assert!(features.contains(DocumentFeature::SHOUTOUT));
        assert!(features.contains(DocumentFeature::TABLE));
        assert!(features.contains(DocumentFeature::UNKNOWN));
    }

    #[test]
    fn test_constructor_new() {
        let features = DocumentFeature::new();
        assert_eq!(features, DocumentFeature::default());
        assert!(features.is_empty());
    }

    #[test]
    fn test_core_elements_only_is_empty() {
        let mut doc = Document::new();
        doc.push_body(DocumentElement::section(
            1,
            "Section Title",
            vec![
                DocumentElement::text("Section body text"),
                DocumentElement::horizontal_rule(),
            ],
        ));
        let features = DocumentFeature::from(&doc);
        assert!(features.is_empty());
        assert!(features.to_string().contains("core"));
    }

    #[test]
    fn test_deeply_nested_recursive_traversal() {
        let checkbox_child = DocumentElement::check_box_item(true, "Deepest item");
        let mut ordered_child = DocumentElement::ordered_list_item(1, "Ordered item");
        ordered_child.push_child(checkbox_child).unwrap();
        let mut bullet_child = DocumentElement::bullet_list_item("Bullet item");
        bullet_child.push_child(ordered_child).unwrap();
        let section = DocumentElement::section(1, "Nested Section", vec![bullet_child]);
        let directive = DocumentElement::block_directive("custom_block", vec![section]);

        let mut doc = Document::new();
        doc.push_body(directive);

        let features = DocumentFeature::from(&doc);
        assert!(!features.is_empty());
        assert!(!features.is_all());
        assert!(features.contains(DocumentFeature::BULLET));
        assert!(features.contains(DocumentFeature::ORDERED));
        assert!(features.contains(DocumentFeature::TASK));
        assert!(!features.contains(DocumentFeature::IMAGE));
        assert!(!features.contains(DocumentFeature::CODE));
        assert!(!features.contains(DocumentFeature::TABLE));
        assert!(!features.contains(DocumentFeature::SHOUTOUT));
        assert!(!features.contains(DocumentFeature::UNKNOWN));
    }

    #[test]
    fn test_display_all() {
        let features = DocumentFeature::ALL;
        let s = features.to_string();
        assert!(s.contains("core"));
        assert!(s.contains("bullet"));
        assert!(s.contains("code"));
        assert!(s.contains("image"));
        assert!(s.contains("ordered"));
        assert!(s.contains("shoutout"));
        assert!(s.contains("table"));
        assert!(s.contains("task"));
        assert!(s.contains("unknown"));
    }

    #[test]
    fn test_display_combinations() {
        let mut features = DocumentFeature::BULLET | DocumentFeature::TABLE;
        let s1 = features.to_string();
        assert!(s1.contains("core"));
        assert!(s1.contains("bullet"));
        assert!(s1.contains("table"));
        assert!(!s1.contains("image"));

        features |= DocumentFeature::IMAGE;
        let s2 = features.to_string();
        assert!(s2.contains("image"));

        features |= DocumentFeature::UNKNOWN;
        let s3 = features.to_string();
        assert!(s3.contains("unknown"));
    }

    #[test]
    fn test_display_default() {
        let features = DocumentFeature::default();
        assert!(features.to_string().contains("core"));
    }

    #[test]
    fn test_display_single() {
        let features = DocumentFeature::CODE;
        let s = features.to_string();
        assert!(s.contains("core"));
        assert!(s.contains("code"));
        assert!(!s.contains("unknown"));

        let unknown_feature = DocumentFeature::UNKNOWN;
        let s_unknown = unknown_feature.to_string();
        assert!(s_unknown.contains("core"));
        assert!(s_unknown.contains("unknown"));
        assert!(!s_unknown.contains("code"));
    }

    #[test]
    fn test_early_exit_short_circuit() {
        let mut doc = Document::new();
        doc.push_body(DocumentElement::bullet_list_item("A"));
        doc.push_body(DocumentElement::check_box_item(true, "B"));
        doc.push_body(DocumentElement::code_block(None::<String>, "C"));
        doc.push_body(DocumentElement::image("D", "d.png"));
        doc.push_body(DocumentElement::ordered_list_item(1, "E"));
        doc.push_body(DocumentElement::shoutout(ShoutoutElementKind::Caution, "G"));
        doc.push_body(DocumentElement::table(vec![], vec![]));
        doc.push_body(DocumentElement::unknown("H"));

        assert!(DocumentFeature::from(&doc).is_all());

        let huge_nested_section = DocumentElement::section(
            1,
            "Huge Section",
            vec![
                DocumentElement::block_directive("b1", vec![]),
                DocumentElement::block_directive("b2", vec![]),
            ],
        );
        doc.push_body(huge_nested_section);

        let features = DocumentFeature::from(&doc);
        assert!(features.is_all());
    }

    #[test]
    fn test_header_variables_activates_code_and_table_features() {
        let mut doc = Document::new();
        doc.header.variables = Some(DocumentElement::table(
            vec![TableAlignment::None, TableAlignment::None],
            vec![
                vec!["Variable".into(), "Value".into()],
                vec!["PORT".into(), "8080".into()],
            ],
        ));
        let features = DocumentFeature::from(&doc);
        assert!(features.contains(DocumentFeature::CODE));
        assert!(features.contains(DocumentFeature::TABLE));
        assert!(!features.contains(DocumentFeature::BULLET));
        assert!(!features.contains(DocumentFeature::TASK));
        assert!(!features.contains(DocumentFeature::IMAGE));
        assert!(!features.contains(DocumentFeature::ORDERED));
        assert!(!features.contains(DocumentFeature::SHOUTOUT));
        assert!(!features.contains(DocumentFeature::UNKNOWN));
    }

    #[test]
    fn test_empty_document_boundary() {
        let doc = Document::new();
        let features = DocumentFeature::from(&doc);
        assert_eq!(features, DocumentFeature::default());
        assert!(features.is_empty());
        assert!(!features.is_all());
        assert!(!features.contains(DocumentFeature::BULLET));
        assert!(!features.contains(DocumentFeature::TASK));
        assert!(!features.contains(DocumentFeature::CODE));
        assert!(!features.contains(DocumentFeature::IMAGE));
        assert!(!features.contains(DocumentFeature::ORDERED));
        assert!(!features.contains(DocumentFeature::SHOUTOUT));
        assert!(!features.contains(DocumentFeature::TABLE));
        assert!(!features.contains(DocumentFeature::UNKNOWN));
    }

    #[test]
    fn test_feature_trait_default_methods() {
        #[derive(Debug)]
        struct MinimalFeature;
        impl Feature for MinimalFeature {}

        let feature = MinimalFeature;
        assert_eq!(feature.css(), None);
        assert!(feature.javascript().is_empty());
        let mut out = String::new();
        let renderer = HtmlRenderer::default_renderer();
        let header = DocumentHeader::default();
        let element = DocumentElement::text("Test");
        let params = DocumentParameters::default();
        assert!(!feature.try_render_header(&header, &params));
        assert!(!feature.try_render_body(&element, 0, 0, &params, &mut out, &renderer));
        assert!(out.is_empty());
    }

    #[test]
    fn test_partial_features_matches() {
        let mut single_feature_doc = Document::new();
        single_feature_doc.push_body(DocumentElement::bullet_list_item("Only bullet"));
        let single_features = DocumentFeature::from(&single_feature_doc);
        assert!(!single_features.is_empty());
        assert!(!single_features.is_all());
        assert!(single_features.contains(DocumentFeature::BULLET));
        assert!(!single_features.contains(DocumentFeature::CODE));
        assert!(!single_features.contains(DocumentFeature::TABLE));
        assert!(!single_features.contains(DocumentFeature::UNKNOWN));

        let mut multi_feature_doc = Document::new();
        multi_feature_doc.push_body(DocumentElement::code_block(
            None::<String>,
            "console.log(1)",
        ));
        multi_feature_doc.push_body(DocumentElement::table(vec![], vec![]));
        let multi_features = DocumentFeature::from(&multi_feature_doc);
        assert!(!multi_features.is_empty());
        assert!(!multi_features.is_all());
        assert!(multi_features.contains(DocumentFeature::CODE));
        assert!(multi_features.contains(DocumentFeature::TABLE));
        assert!(!multi_features.contains(DocumentFeature::IMAGE));
        assert!(!multi_features.contains(DocumentFeature::UNKNOWN));

        let mut six_features_doc = Document::new();
        six_features_doc.push_body(DocumentElement::bullet_list_item("A"));
        six_features_doc.push_body(DocumentElement::check_box_item(false, "B"));
        six_features_doc.push_body(DocumentElement::code_block(None::<String>, "C"));
        six_features_doc.push_body(DocumentElement::ordered_list_item(1, "D"));
        six_features_doc.push_body(DocumentElement::section(1, "E", vec![]));
        six_features_doc.push_body(DocumentElement::shoutout(ShoutoutElementKind::Tip, "F"));
        six_features_doc.push_body(DocumentElement::table(vec![], vec![]));
        let six_features = DocumentFeature::from(&six_features_doc);
        assert!(!six_features.is_empty());
        assert!(!six_features.is_all());
        assert!(!six_features.contains(DocumentFeature::IMAGE));
        assert!(six_features.contains(DocumentFeature::BULLET));
        assert!(six_features.contains(DocumentFeature::TASK));
        assert!(six_features.contains(DocumentFeature::CODE));
        assert!(six_features.contains(DocumentFeature::ORDERED));
        assert!(six_features.contains(DocumentFeature::SHOUTOUT));
        assert!(six_features.contains(DocumentFeature::TABLE));
        assert!(!six_features.contains(DocumentFeature::UNKNOWN));
    }

    #[test]
    fn test_unknown_element_detected() {
        let mut doc = Document::new();
        doc.push_body(DocumentElement::unknown("fallback raw data"));
        let features = DocumentFeature::from(&doc);
        assert!(!features.is_empty());
        assert!(features.contains(DocumentFeature::UNKNOWN));
        let s = features.to_string();
        assert!(s.contains("core"));
        assert!(s.contains("unknown"));
    }
}
