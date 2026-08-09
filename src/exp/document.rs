//! Document model and hierarchical element definitions.

use std::collections::HashMap;

/// Classification of a shoutout element kind.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShoutoutElementKind {
    /// Generic informational note box.
    Note,
    /// Proactive recommendation or tip box.
    Tip,
    /// High-priority important notice box.
    Important,
    /// Cautionary warning alert box.
    Warning,
    /// Critical danger or caution alert box.
    Caution,
}

impl ShoutoutElementKind {
    /// Returns the static lowercase string identifier.
    #[inline]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Note => "note",
            Self::Tip => "tip",
            Self::Important => "important",
            Self::Warning => "warning",
            Self::Caution => "caution",
        }
    }
}

/// Metadata and configuration for a shoutout element.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ShoutoutElement {
    /// Specific category of this shoutout.
    pub kind: ShoutoutElementKind,
}

impl ShoutoutElement {
    /// Creates a new shoutout element metadata container.
    #[inline]
    pub const fn new(kind: ShoutoutElementKind) -> Self {
        Self { kind }
    }
}

/// Classification of a document element.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DocumentElementKind {
    /// Standard plain text content.
    Text,
    /// Shoutout or callout container element.
    Shoutout(ShoutoutElement),
    /// Unrecognized or fallback content.
    Unknown,
}

/// Recursive node in the document tree.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DocumentElement {
    /// Type classification of this element.
    pub kind: DocumentElementKind,
    /// Text or raw content payload.
    pub content: String,
    /// Nested child elements.
    pub children: Vec<DocumentElement>,
}

impl DocumentElement {
    /// Creates a new document element with no children.
    pub fn new(kind: DocumentElementKind, content: impl Into<String>) -> Self {
        Self {
            kind,
            content: content.into(),
            children: Vec::new(),
        }
    }

    /// Creates a new shoutout document element.
    pub fn shoutout(kind: ShoutoutElementKind, content: impl Into<String>) -> Self {
        Self::new(
            DocumentElementKind::Shoutout(ShoutoutElement::new(kind)),
            content,
        )
    }

    /// Appends a child element.
    pub fn push_child(&mut self, child: Self) {
        self.children.push(child);
    }
}

/// Represents a parsed document tree.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Document {
    /// Frontmatter key-value configuration parameters.
    pub parameters: HashMap<String, String>,
    /// Header document elements.
    pub header: Vec<DocumentElement>,
    /// Main body document elements.
    pub body: Vec<DocumentElement>,
}

impl Document {
    /// Creates an empty document.
    pub fn new() -> Self {
        Self {
            parameters: HashMap::new(),
            header: Vec::new(),
            body: Vec::new(),
        }
    }

    /// Appends a body element.
    pub fn push_body(&mut self, element: DocumentElement) {
        self.body.push(element);
    }

    /// Appends a header element.
    pub fn push_header(&mut self, element: DocumentElement) {
        self.header.push(element);
    }

    /// Inserts a frontmatter parameter.
    pub fn insert_parameter(&mut self, key: impl Into<String>, value: impl Into<String>) {
        self.parameters.insert(key.into(), value.into());
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_document_element_creation() {
        let mut elem = DocumentElement::new(DocumentElementKind::Text, "Hello world");
        assert_eq!(elem.kind, DocumentElementKind::Text);
        assert_eq!(elem.content, "Hello world");
        assert!(elem.children.is_empty());

        let child = DocumentElement::new(DocumentElementKind::Unknown, "Child node");
        elem.push_child(child);
        assert_eq!(elem.children.len(), 1);
        assert_eq!(elem.children[0].kind, DocumentElementKind::Unknown);
    }

    #[test]
    fn test_document_push_body() {
        let mut doc = Document::new();
        assert!(doc.parameters.is_empty());
        assert!(doc.header.is_empty());
        assert!(doc.body.is_empty());

        doc.insert_parameter("title", "My Doc");
        doc.push_body(DocumentElement::new(DocumentElementKind::Text, "Line 1"));
        assert_eq!(doc.parameters.get("title").map(|s| s.as_str()), Some("My Doc"));
        assert_eq!(doc.body.len(), 1);
        assert_eq!(doc.body[0].content, "Line 1");
    }

    #[test]
    fn test_shoutout_element_creation_and_kinds() {
        let kinds = [
            (ShoutoutElementKind::Note, "note"),
            (ShoutoutElementKind::Tip, "tip"),
            (ShoutoutElementKind::Important, "important"),
            (ShoutoutElementKind::Warning, "warning"),
            (ShoutoutElementKind::Caution, "caution"),
        ];

        for (kind, expected_str) in kinds {
            assert_eq!(kind.as_str(), expected_str);
            let elem = DocumentElement::shoutout(kind, format!("{expected_str} content"));
            assert_eq!(
                elem.kind,
                DocumentElementKind::Shoutout(ShoutoutElement::new(kind))
            );
            assert_eq!(elem.content, format!("{expected_str} content"));
            assert!(elem.children.is_empty());
        }
    }
}
