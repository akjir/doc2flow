//! Document AST feature detection, inspection, and rendering traits.

use std::fmt::{self, Display, Formatter};

use crate::core::builder::HtmlRenderer;
use crate::core::document::{Document, DocumentElement, DocumentParameters};

/// Trait for document feature renderers converting AST elements to HTML.
pub trait Feature: Send + Sync + fmt::Debug {
    /// Intercepts the rendering of a document element into an output buffer.
    ///
    /// Returns `true` if this feature handled rendering the element, or `false`
    /// to delegate to subsequent features or core fallback rendering.
    fn try_render(
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

    /// Returns optional CSS stylesheet rules for this feature, defaulting to `None`.
    fn css(&self) -> Option<&'static str> {
        None
    }

    /// Returns optional JavaScript client logic files for this feature, defaulting to an empty slice.
    fn javascript(&self) -> &[&'static str] {
        &[]
    }
}

/// Feature detection flags for document AST elements.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct DocumentFeature {
    /// Indicates whether bullet lists are present.
    pub bullet: bool,
    /// Indicates whether code blocks are present.
    pub code: bool,
    /// Indicates whether images are present.
    pub image: bool,
    /// Indicates whether ordered lists are present.
    pub ordered: bool,
    /// Indicates whether shoutout callouts are present.
    pub shoutout: bool,
    /// Indicates whether tables are present.
    pub table: bool,
    /// Indicates whether task items are present.
    pub task: bool,
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
            bullet: false,
            code: false,
            image: false,
            ordered: false,
            shoutout: false,
            table: false,
            task: false,
            unknown: false,
        }
    }

    /// Returns `true` if all feature flags are enabled.
    pub const fn is_all(self) -> bool {
        self.bullet
            && self.code
            && self.image
            && self.ordered
            && self.shoutout
            && self.table
            && self.task
            && self.unknown
    }

    /// Returns `true` if no feature flags are enabled.
    pub const fn is_empty(self) -> bool {
        !self.bullet
            && !self.code
            && !self.image
            && !self.ordered
            && !self.shoutout
            && !self.table
            && !self.task
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
    /// features.code = true;
    /// assert_eq!(features.to_features_string(), "core, code");
    ///
    /// features.table = true;
    /// assert_eq!(features.to_features_string(), "core, code, table");
    /// ```
    pub fn to_features_string(&self) -> String {
        let mut out = String::with_capacity(96);
        out.push_str("core");
        if self.bullet {
            out.push_str(", bullet");
        }
        if self.code {
            out.push_str(", code");
        }
        if self.image {
            out.push_str(", image");
        }
        if self.ordered {
            out.push_str(", ordered");
        }
        if self.shoutout {
            out.push_str(", shoutout");
        }
        if self.table {
            out.push_str(", table");
        }
        if self.task {
            out.push_str(", task");
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
    /// assert!(features.code);
    /// assert!(!features.table);
    /// ```
    fn from(doc: &Document) -> Self {
        let mut features = Self::default();
        if let Some(ref variables) = doc.header.variables {
            features.code = true;
            scan_element(variables, &mut features);
        }
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
        DocumentElement::BlockDirective { children, .. }
        | DocumentElement::Section { children, .. } => {
            scan_elements(children, features);
        }
        DocumentElement::BulletListItem { children, .. } => {
            features.bullet = true;
            scan_elements(children, features);
        }
        DocumentElement::CheckBoxItem { children, .. } => {
            features.task = true;
            scan_elements(children, features);
        }
        DocumentElement::CodeBlock { .. } => {
            features.code = true;
        }
        DocumentElement::Image { .. } => {
            features.image = true;
        }
        DocumentElement::OrderedListItem { children, .. } => {
            features.ordered = true;
            scan_elements(children, features);
        }
        DocumentElement::Shoutout { .. } => {
            features.shoutout = true;
        }
        DocumentElement::Table { .. } => {
            features.table = true;
        }
        DocumentElement::HorizontalRule | DocumentElement::Text(_) => {}
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

/// Converts a [`DocumentFeature`] configuration into a formatted feature summary string.
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
        assert!(features.bullet);
        assert!(features.task);
        assert!(features.code);
        assert!(features.image);
        assert!(features.ordered);
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
        assert_eq!(features.to_features_string(), "core");
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
        assert!(features.bullet);
        assert!(features.ordered);
        assert!(features.task);
        assert!(!features.image);
        assert!(!features.code);
        assert!(!features.table);
        assert!(!features.shoutout);
        assert!(!features.unknown);
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
        assert!(features.code);
        assert!(features.table);
        assert!(!features.bullet);
        assert!(!features.task);
        assert!(!features.image);
        assert!(!features.ordered);
        assert!(!features.shoutout);
        assert!(!features.unknown);
    }

    #[test]
    fn test_empty_document_boundary() {
        let doc = Document::new();
        let features = DocumentFeature::from(&doc);
        assert_eq!(features, DocumentFeature::default());
        assert!(features.is_empty());
        assert!(!features.is_all());
        assert!(!features.bullet);
        assert!(!features.task);
        assert!(!features.code);
        assert!(!features.image);
        assert!(!features.ordered);
        assert!(!features.shoutout);
        assert!(!features.table);
        assert!(!features.unknown);
    }

    #[test]
    fn test_partial_features_matches() {
        let mut single_feature_doc = Document::new();
        single_feature_doc.push_body(DocumentElement::bullet_list_item("Only bullet"));
        let single_features = DocumentFeature::from(&single_feature_doc);
        assert!(!single_features.is_empty());
        assert!(!single_features.is_all());
        assert!(single_features.bullet);
        assert!(!single_features.code);
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
        assert!(multi_features.code);
        assert!(multi_features.table);
        assert!(!multi_features.image);
        assert!(!multi_features.unknown);

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
        assert!(!six_features.image);
        assert!(six_features.bullet);
        assert!(six_features.task);
        assert!(six_features.code);
        assert!(six_features.ordered);
        assert!(six_features.shoutout);
        assert!(six_features.table);
        assert!(!six_features.unknown);
    }

    #[test]
    fn test_to_features_string_all() {
        let features = DocumentFeature {
            bullet: true,
            code: true,
            image: true,
            ordered: true,
            shoutout: true,
            table: true,
            task: true,
            unknown: true,
        };
        let expected = "core, bullet, code, image, ordered, shoutout, table, task, unknown";
        assert_eq!(features.to_features_string(), expected);
        assert_eq!(to_features_string(&features), expected);
        assert_eq!(features.to_string(), expected);
    }

    #[test]
    fn test_to_features_string_combinations() {
        let mut features = DocumentFeature {
            bullet: true,
            table: true,
            ..Default::default()
        };
        assert_eq!(features.to_features_string(), "core, bullet, table");

        features.image = true;
        assert_eq!(features.to_features_string(), "core, bullet, image, table");

        features.unknown = true;
        assert_eq!(
            features.to_features_string(),
            "core, bullet, image, table, unknown"
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
        let features = DocumentFeature {
            code: true,
            ..Default::default()
        };
        assert_eq!(features.to_features_string(), "core, code");
        assert_eq!(to_features_string(&features), "core, code");
        assert_eq!(features.to_string(), "core, code");

        let unknown_feature = DocumentFeature {
            unknown: true,
            ..Default::default()
        };
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
        let element = DocumentElement::text("Test");
        let params = DocumentParameters::default();
        assert!(!feature.try_render(&element, 0, 0, &params, &mut out, &renderer));
        assert!(out.is_empty());
    }
}
