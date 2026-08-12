//! Document AST feature detection and inspection.

use crate::core::document::{Document, DocumentElement};

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
    /// Indicates whether section headings are present.
    pub section: bool,
    /// Indicates whether shoutout callouts are present.
    pub shoutout: bool,
    /// Indicates whether tables are present.
    pub table: bool,
    /// Indicates whether standard text paragraphs are present.
    pub text: bool,
}

impl DocumentFeature {
    /// Returns `true` if all feature flags are enabled.
    pub const fn is_all(self) -> bool {
        self.bullet_list_item
            && self.check_box_item
            && self.code_block
            && self.image
            && self.ordered_list_item
            && self.section
            && self.shoutout
            && self.table
            && self.text
    }

    /// Returns `true` if no feature flags are enabled.
    pub const fn is_empty(self) -> bool {
        !self.bullet_list_item
            && !self.check_box_item
            && !self.code_block
            && !self.image
            && !self.ordered_list_item
            && !self.section
            && !self.shoutout
            && !self.table
            && !self.text
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
    /// doc.push_body(DocumentElement::text("Hello"));
    /// let features = DocumentFeature::from(&doc);
    /// assert!(features.text);
    /// assert!(!features.code_block);
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
        DocumentElement::BlockDirective { children, .. } => {
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
        DocumentElement::Section { children, .. } => {
            features.section = true;
            scan_elements(children, features);
        }
        DocumentElement::Shoutout { .. } => {
            features.shoutout = true;
        }
        DocumentElement::Table { .. } => {
            features.table = true;
        }
        DocumentElement::Text(_) => {
            features.text = true;
        }
        DocumentElement::Unknown(_) => {}
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
        assert!(!features.section);
        assert!(!features.shoutout);
        assert!(!features.table);
        assert!(!features.text);
    }

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
        assert!(features.section);
        assert!(features.shoutout);
        assert!(features.table);
        assert!(features.text);
    }

    #[test]
    fn test_partial_features_matches() {
        let mut single_feature_doc = Document::new();
        single_feature_doc.push_body(DocumentElement::text("Only plain text"));
        let single_features = DocumentFeature::from(&single_feature_doc);
        assert!(!single_features.is_empty());
        assert!(!single_features.is_all());
        assert!(single_features.text);
        assert!(!single_features.code_block);
        assert!(!single_features.table);

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
        assert!(!multi_features.text);
        assert!(!multi_features.image);

        let mut eight_features_doc = Document::new();
        eight_features_doc.push_body(DocumentElement::bullet_list_item(
            0,
            DocumentElement::text("A"),
        ));
        eight_features_doc.push_body(DocumentElement::check_box_item(
            0,
            false,
            DocumentElement::text("B"),
        ));
        eight_features_doc.push_body(DocumentElement::code_block(None::<String>, "C"));
        eight_features_doc.push_body(DocumentElement::ordered_list_item(
            0,
            1,
            DocumentElement::text("D"),
        ));
        eight_features_doc.push_body(DocumentElement::section(1, "E", vec![]));
        eight_features_doc.push_body(DocumentElement::shoutout(ShoutoutElementKind::Tip, "F"));
        eight_features_doc.push_body(DocumentElement::table(vec![], vec![]));
        let eight_features = DocumentFeature::from(&eight_features_doc);
        assert!(!eight_features.is_empty());
        assert!(!eight_features.is_all());
        assert!(!eight_features.image);
        assert!(eight_features.bullet_list_item);
        assert!(eight_features.check_box_item);
        assert!(eight_features.code_block);
        assert!(eight_features.ordered_list_item);
        assert!(eight_features.section);
        assert!(eight_features.shoutout);
        assert!(eight_features.table);
        assert!(eight_features.text);
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
        assert!(features.section);
        assert!(features.bullet_list_item);
        assert!(features.ordered_list_item);
        assert!(features.check_box_item);
        assert!(features.text);
        assert!(!features.image);
        assert!(!features.code_block);
        assert!(!features.table);
        assert!(!features.shoutout);
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
        doc.push_header(DocumentElement::section(1, "F", vec![]));
        doc.push_header(DocumentElement::shoutout(ShoutoutElementKind::Caution, "G"));
        doc.push_header(DocumentElement::table(vec![], vec![]));

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
    fn test_unknown_element_ignored() {
        let mut doc = Document::new();
        doc.push_body(DocumentElement::unknown("fallback raw data"));
        let features = DocumentFeature::from(&doc);
        assert!(features.is_empty());
        assert_eq!(features, DocumentFeature::default());
    }
}
