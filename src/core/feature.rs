//! Document AST feature detection, inspection, and rendering traits.

use std::fmt::{self, Display, Formatter};

use crate::core::document::{Document, DocumentElement};

/// Trait for document feature renderers converting AST elements to HTML.
pub trait Feature {
    /// Converts a document element into an HTML string representation.
    fn to_html(&self, element: &DocumentElement) -> String;

    /// Returns optional CSS stylesheet rules for this feature, defaulting to `None`.
    fn css(&self) -> Option<&'static str> {
        None
    }
}

/// Feature detection flags for document AST elements.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct DocumentFeature {
    /// Indicates whether bullet list items are present.
    pub bullet_list_item: bool,
    /// Indicates whether checkbox items are present.
    pub check_box_item: bool,
    /// Indicates whether code blocks are present.
    pub code_block: bool,
    /// Indicates whether images are present.
    pub image: bool,
    /// Indicates whether ordered list items are present.
    pub ordered_list_item: bool,
    /// Indicates whether shoutout callouts are present.
    pub shoutout: bool,
    /// Indicates whether tables are present.
    pub table: bool,
    /// Indicates whether unrecognized unknown elements are present.
    pub unknown: bool,
}

impl DocumentFeature {
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
    pub const fn new() -> Self {
        Self {
            bullet_list_item: false,
            check_box_item: false,
            code_block: false,
            image: false,
            ordered_list_item: false,
            shoutout: false,
            table: false,
            unknown: false,
        }
    }

    /// Returns `true` if all feature flags are enabled.
    pub const fn is_all(self) -> bool {
        self.bullet_list_item
            && self.check_box_item
            && self.code_block
            && self.image
            && self.ordered_list_item
            && self.shoutout
            && self.table
            && self.unknown
    }

    /// Returns `true` if no feature flags are enabled.
    pub const fn is_empty(self) -> bool {
        !self.bullet_list_item
            && !self.check_box_item
            && !self.code_block
            && !self.image
            && !self.ordered_list_item
            && !self.shoutout
            && !self.table
            && !self.unknown
    }

    /// Renders a comma-separated list of enabled feature identifiers starting with `"core"`.
    ///
    /// # Examples
    ///
    /// ```
    /// use doc2flow::core::feature::DocumentFeature;
    ///
    /// let mut features = DocumentFeature::default();
    /// assert_eq!(features.to_features_string(), "core");
    ///
    /// features.code_block = true;
    /// assert_eq!(features.to_features_string(), "core, code_block");
    ///
    /// features.table = true;
    /// assert_eq!(features.to_features_string(), "core, code_block, table");
    /// ```
    pub fn to_features_string(&self) -> String {
        let mut out = String::with_capacity(96);
        out.push_str("core");
        if self.bullet_list_item {
            out.push_str(", bullet_list_item");
        }
        if self.check_box_item {
            out.push_str(", check_box_item");
        }
        if self.code_block {
            out.push_str(", code_block");
        }
        if self.image {
            out.push_str(", image");
        }
        if self.ordered_list_item {
            out.push_str(", ordered_list_item");
        }
        if self.shoutout {
            out.push_str(", shoutout");
        }
        if self.table {
            out.push_str(", table");
        }
        if self.unknown {
            out.push_str(", unknown");
        }
        out
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
        f.write_str(&self.to_features_string())
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
    /// assert!(features.code_block);
    /// assert!(!features.table);
    /// ```
    fn from(doc: &Document) -> Self {
        let mut features = Self::default();
        scan_elements(&doc.header, &mut features);
        scan_elements(&doc.body, &mut features);
        features
    }
}

/// Recursively scans an individual document element and its children.
fn scan_element(element: &DocumentElement, features: &mut DocumentFeature) {
    if features.is_all() {
        return;
    }
    match element {
        DocumentElement::BlockDirective { children, .. } | DocumentElement::Section { children, .. } => {
            scan_elements(children, features);
        }
        DocumentElement::BulletListItem { content, .. } => {
            features.bullet_list_item = true;
            scan_element(content, features);
        }
        DocumentElement::CheckBoxItem { content, .. } => {
            features.check_box_item = true;
            scan_element(content, features);
        }
        DocumentElement::CodeBlock { .. } => {
            features.code_block = true;
        }
        DocumentElement::Image { .. } => {
            features.image = true;
        }
        DocumentElement::OrderedListItem { content, .. } => {
            features.ordered_list_item = true;
            scan_element(content, features);
        }
        DocumentElement::Shoutout { .. } => {
            features.shoutout = true;
        }
        DocumentElement::Table { .. } => {
            features.table = true;
        }
        DocumentElement::Text(_) => {}
        DocumentElement::Unknown(_) => {
            features.unknown = true;
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

/// Returns a comma-separated list of enabled feature identifiers starting with `"core"`.
///
/// # Examples
///
/// ```
/// use doc2flow::core::feature::{DocumentFeature, to_features_string};
///
/// let features = DocumentFeature::default();
/// assert_eq!(to_features_string(&features), "core");
/// ```
pub fn to_features_string(features: &DocumentFeature) -> String {
    features.to_features_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::document::{ShoutoutElementKind, TableAlignment};

    #[test]
    fn test_all_features_present() {
        let mut doc = Document::new();
        doc.push_header(DocumentElement::text("Header text"));
        doc.push_body(DocumentElement::bullet_list_item(
            0,
            DocumentElement::text("Bullet"),
        ));
        doc.push_body(DocumentElement::check_box_item(
            0,
            true,
            DocumentElement::text("Task"),
        ));
        doc.push_body(DocumentElement::code_block(Some("rust"), "fn main() {}"));
        doc.push_body(DocumentElement::image("alt", "image.png"));
        doc.push_body(DocumentElement::ordered_list_item(
            0,
            1,
            DocumentElement::text("Ordered"),
        ));
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
        assert!(features.bullet_list_item);
        assert!(features.check_box_item);
        assert!(features.code_block);
        assert!(features.image);
        assert!(features.ordered_list_item);
        assert!(features.shoutout);
        assert!(features.table);
        assert!(features.unknown);
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
        doc.push_header(DocumentElement::text("Header text"));
        doc.push_body(DocumentElement::section(
            1,
            "Section Title",
            vec![DocumentElement::text("Section body text")],
        ));
        let features = DocumentFeature::from(&doc);
        assert!(features.is_empty());
        assert_eq!(features.to_features_string(), "core");
    }

    #[test]
    fn test_deeply_nested_recursive_traversal() {
        let text_child = DocumentElement::text("Deepest item");
        let checkbox_child = DocumentElement::check_box_item(2, true, text_child);
        let ordered_child = DocumentElement::ordered_list_item(1, 1, checkbox_child);
        let bullet_child = DocumentElement::bullet_list_item(0, ordered_child);
        let section = DocumentElement::section(1, "Nested Section", vec![bullet_child]);
        let directive = DocumentElement::block_directive("custom_block", vec![section]);

        let mut doc = Document::new();
        doc.push_body(directive);

        let features = DocumentFeature::from(&doc);
        assert!(!features.is_empty());
        assert!(!features.is_all());
        assert!(features.bullet_list_item);
        assert!(features.ordered_list_item);
        assert!(features.check_box_item);
        assert!(!features.image);
        assert!(!features.code_block);
        assert!(!features.table);
        assert!(!features.shoutout);
        assert!(!features.unknown);
    }

    #[test]
    fn test_early_exit_short_circuit() {
        let mut doc = Document::new();
        doc.push_header(DocumentElement::bullet_list_item(
            0,
            DocumentElement::text("A"),
        ));
        doc.push_header(DocumentElement::check_box_item(
            0,
            true,
            DocumentElement::text("B"),
        ));
        doc.push_header(DocumentElement::code_block(None::<String>, "C"));
        doc.push_header(DocumentElement::image("D", "d.png"));
        doc.push_header(DocumentElement::ordered_list_item(
            0,
            1,
            DocumentElement::text("E"),
        ));
        doc.push_header(DocumentElement::shoutout(ShoutoutElementKind::Caution, "G"));
        doc.push_header(DocumentElement::table(vec![], vec![]));
        doc.push_header(DocumentElement::unknown("H"));

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
    fn test_empty_document_boundary() {
        let doc = Document::new();
        let features = DocumentFeature::from(&doc);
        assert_eq!(features, DocumentFeature::default());
        assert!(features.is_empty());
        assert!(!features.is_all());
        assert!(!features.bullet_list_item);
        assert!(!features.check_box_item);
        assert!(!features.code_block);
        assert!(!features.image);
        assert!(!features.ordered_list_item);
        assert!(!features.shoutout);
        assert!(!features.table);
        assert!(!features.unknown);
    }

    #[test]
    fn test_partial_features_matches() {
        let mut single_feature_doc = Document::new();
        single_feature_doc.push_body(DocumentElement::bullet_list_item(
            0,
            DocumentElement::text("Only bullet"),
        ));
        let single_features = DocumentFeature::from(&single_feature_doc);
        assert!(!single_features.is_empty());
        assert!(!single_features.is_all());
        assert!(single_features.bullet_list_item);
        assert!(!single_features.code_block);
        assert!(!single_features.table);
        assert!(!single_features.unknown);

        let mut multi_feature_doc = Document::new();
        multi_feature_doc.push_body(DocumentElement::code_block(
            None::<String>,
            "console.log(1)",
        ));
        multi_feature_doc.push_body(DocumentElement::table(vec![], vec![]));
        let multi_features = DocumentFeature::from(&multi_feature_doc);
        assert!(!multi_features.is_empty());
        assert!(!multi_features.is_all());
        assert!(multi_features.code_block);
        assert!(multi_features.table);
        assert!(!multi_features.image);
        assert!(!multi_features.unknown);

        let mut six_features_doc = Document::new();
        six_features_doc.push_body(DocumentElement::bullet_list_item(
            0,
            DocumentElement::text("A"),
        ));
        six_features_doc.push_body(DocumentElement::check_box_item(
            0,
            false,
            DocumentElement::text("B"),
        ));
        six_features_doc.push_body(DocumentElement::code_block(None::<String>, "C"));
        six_features_doc.push_body(DocumentElement::ordered_list_item(
            0,
            1,
            DocumentElement::text("D"),
        ));
        six_features_doc.push_body(DocumentElement::section(1, "E", vec![]));
        six_features_doc.push_body(DocumentElement::shoutout(ShoutoutElementKind::Tip, "F"));
        six_features_doc.push_body(DocumentElement::table(vec![], vec![]));
        let six_features = DocumentFeature::from(&six_features_doc);
        assert!(!six_features.is_empty());
        assert!(!six_features.is_all());
        assert!(!six_features.image);
        assert!(six_features.bullet_list_item);
        assert!(six_features.check_box_item);
        assert!(six_features.code_block);
        assert!(six_features.ordered_list_item);
        assert!(six_features.shoutout);
        assert!(six_features.table);
        assert!(!six_features.unknown);
    }

    #[test]
    fn test_to_features_string_all() {
        let features = DocumentFeature {
            bullet_list_item: true,
            check_box_item: true,
            code_block: true,
            image: true,
            ordered_list_item: true,
            shoutout: true,
            table: true,
            unknown: true,
        };
        let expected = "core, bullet_list_item, check_box_item, code_block, image, ordered_list_item, shoutout, table, unknown";
        assert_eq!(features.to_features_string(), expected);
        assert_eq!(to_features_string(&features), expected);
        assert_eq!(features.to_string(), expected);
    }

    #[test]
    fn test_to_features_string_combinations() {
        let mut features = DocumentFeature::default();
        features.bullet_list_item = true;
        features.table = true;
        assert_eq!(features.to_features_string(), "core, bullet_list_item, table");

        features.image = true;
        assert_eq!(features.to_features_string(), "core, bullet_list_item, image, table");

        features.unknown = true;
        assert_eq!(
            features.to_features_string(),
            "core, bullet_list_item, image, table, unknown"
        );
    }

    #[test]
    fn test_to_features_string_default() {
        let features = DocumentFeature::default();
        assert_eq!(features.to_features_string(), "core");
        assert_eq!(to_features_string(&features), "core");
        assert_eq!(features.to_string(), "core");
    }

    #[test]
    fn test_to_features_string_single() {
        let mut features = DocumentFeature::default();
        features.code_block = true;
        assert_eq!(features.to_features_string(), "core, code_block");
        assert_eq!(to_features_string(&features), "core, code_block");
        assert_eq!(features.to_string(), "core, code_block");

        let mut unknown_feature = DocumentFeature::default();
        unknown_feature.unknown = true;
        assert_eq!(unknown_feature.to_features_string(), "core, unknown");
    }

    #[test]
    fn test_unknown_element_detected() {
        let mut doc = Document::new();
        doc.push_body(DocumentElement::unknown("fallback raw data"));
        let features = DocumentFeature::from(&doc);
        assert!(!features.is_empty());
        assert!(features.unknown);
        assert_eq!(features.to_features_string(), "core, unknown");
    }
}
