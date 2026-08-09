//! Document model and hierarchical element definitions.

use std::collections::HashMap;

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
    /// Inserts a frontmatter parameter.
    pub fn insert_parameter(&mut self, key: impl Into<String>, value: impl Into<String>) {
        self.parameters.insert(key.into(), value.into());
    }

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
}

/// Hierarchical document element representation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum DocumentElement {
    /// Collapsible or structured section containing nested child elements.
    Section {
        /// Section heading or title.
        title: String,
        /// Nested child elements within this section.
        children: Vec<DocumentElement>,
    },
    /// Shoutout or callout container element with a specific severity level.
    Shoutout {
        /// Specific category and severity of this shoutout.
        kind: ShoutoutElementKind,
        /// Inner shoutout message content.
        content: String,
    },
    /// Standard plain text paragraph or line.
    Text(String),
    /// Unrecognized or fallback content.
    Unknown(String),
}

impl DocumentElement {
    /// Appends a child element if this element is a section container.
    pub fn push_child(&mut self, child: Self) {
        if let Self::Section { children, .. } = self {
            children.push(child);
        }
    }

    /// Creates a new section document element with children.
    pub fn section(title: impl Into<String>, children: Vec<DocumentElement>) -> Self {
        Self::Section {
            title: title.into(),
            children,
        }
    }

    /// Creates a new shoutout document element.
    pub fn shoutout(kind: ShoutoutElementKind, content: impl Into<String>) -> Self {
        Self::Shoutout {
            kind,
            content: content.into(),
        }
    }

    /// Creates a new plain text document element.
    pub fn text(content: impl Into<String>) -> Self {
        Self::Text(content.into())
    }

    /// Creates a new unknown fallback document element.
    pub fn unknown(content: impl Into<String>) -> Self {
        Self::Unknown(content.into())
    }
}

/// Classification of a shoutout element kind.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShoutoutElementKind {
    /// Critical danger or caution alert box.
    Caution,
    /// High-priority important notice box.
    Important,
    /// Generic informational note box.
    Note,
    /// Proactive recommendation or tip box.
    Tip,
    /// Cautionary warning alert box.
    Warning,
}

impl ShoutoutElementKind {
    /// Returns the static lowercase string identifier.
    #[inline]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Caution => "caution",
            Self::Important => "important",
            Self::Note => "note",
            Self::Tip => "tip",
            Self::Warning => "warning",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_document_push_body() {
        let mut doc = Document::new();
        assert!(doc.parameters.is_empty());
        assert!(doc.header.is_empty());
        assert!(doc.body.is_empty());

        doc.insert_parameter("title", "My Doc");
        doc.push_body(DocumentElement::text("Line 1"));
        assert_eq!(doc.parameters.get("title").map(|s| s.as_str()), Some("My Doc"));
        assert_eq!(doc.body.len(), 1);
        assert_eq!(doc.body[0], DocumentElement::Text("Line 1".into()));
    }

    #[test]
    fn test_document_element_creation() {
        let elem = DocumentElement::text("Hello world");
        assert_eq!(elem, DocumentElement::Text("Hello world".into()));

        let mut section = DocumentElement::section("Heading", Vec::new());
        let child = DocumentElement::unknown("Child node");
        section.push_child(child.clone());
        assert_eq!(
            section,
            DocumentElement::Section {
                title: "Heading".into(),
                children: vec![child],
            }
        );
    }

    #[test]
    fn test_shoutout_element_creation_and_kinds() {
        let kinds = [
            (ShoutoutElementKind::Caution, "caution"),
            (ShoutoutElementKind::Important, "important"),
            (ShoutoutElementKind::Note, "note"),
            (ShoutoutElementKind::Tip, "tip"),
            (ShoutoutElementKind::Warning, "warning"),
        ];

        for (kind, expected_str) in kinds {
            assert_eq!(kind.as_str(), expected_str);
            let elem = DocumentElement::shoutout(kind, format!("{expected_str} content"));
            assert_eq!(
                elem,
                DocumentElement::Shoutout {
                    kind,
                    content: format!("{expected_str} content"),
                }
            );
        }
    }
}
