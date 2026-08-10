//! Document model and hierarchical element definitions.

use std::collections::HashMap;
use std::fmt::{self, Display, Formatter};

/// Represents a parsed document tree.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Document {
    /// Main body document elements.
    pub body: Vec<DocumentElement>,
    /// Header document elements.
    pub header: Vec<DocumentElement>,
    /// Frontmatter key-value configuration parameters.
    pub parameters: HashMap<String, String>,
}

impl Document {
    /// Creates an empty document.
    pub fn new() -> Self {
        Self {
            body: Vec::new(),
            header: Vec::new(),
            parameters: HashMap::new(),
        }
    }

    /// Creates an empty document with pre-allocated capacities.
    pub fn with_capacity(body_capacity: usize) -> Self {
        Self {
            body: Vec::with_capacity(body_capacity),
            header: Vec::new(),
            parameters: HashMap::new(),
        }
    }

    /// Inserts a frontmatter parameter.
    pub fn insert_parameter(&mut self, key: impl Into<String>, value: impl Into<String>) {
        self.parameters.insert(key.into(), value.into());
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
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DocumentElement {
    /// Block directive container with a specified name and nested child elements.
    BlockDirective {
        /// Nested child elements within this block directive.
        children: Vec<DocumentElement>,
        /// Directive name identifier (e.g. `variables`).
        name: String,
    },
    /// Bullet list item element with indentation depth and child content.
    BulletListItem {
        /// Child document element content.
        content: Box<DocumentElement>,
        /// Nesting depth level based on leading spaces.
        depth: usize,
    },
    /// Checkbox list item element with indentation depth, checked status, and child content.
    CheckBoxItem {
        /// Indicates whether the checkbox is checked.
        checked: bool,
        /// Child document element content.
        content: Box<DocumentElement>,
        /// Nesting depth level based on leading spaces.
        depth: usize,
    },
    /// Fenced code block with optional language specifier and content.
    CodeBlock {
        /// Code block text content.
        content: String,
        /// Optional programming or markup language identifier (info string).
        language: Option<String>,
    },
    /// Image element with alt text and source target URL.
    Image {
        /// Alternative descriptive text for the image.
        alt: String,
        /// Source URL or file path location of the image.
        url: String,
    },
    /// Ordered list item element with indentation depth, sequential position, and child content.
    OrderedListItem {
        /// Child document element content.
        content: Box<DocumentElement>,
        /// Nesting depth level based on leading spaces.
        depth: usize,
        /// 1-based sequential position within the list at this depth.
        position: usize,
    },
    /// Collapsible or structured section containing nested child elements.
    Section {
        /// Nested child elements within this section.
        children: Vec<DocumentElement>,
        /// Heading level of the section (1 for `#`, 2 for `##`, 3 for `###` and higher).
        level: usize,
        /// Section heading or title.
        title: String,
    },
    /// Shoutout or callout container element with a specific severity level.
    Shoutout {
        /// Inner shoutout message content.
        content: String,
        /// Specific category and severity of this shoutout.
        kind: ShoutoutElementKind,
    },
    /// Table element containing column alignments and a 2D matrix of cell contents (rows, columns).
    Table {
        /// Alignment specification for each column.
        alignments: Vec<TableAlignment>,
        /// Rows of the table (header row followed by data rows), where each row is a vector of cell strings.
        rows: Vec<Vec<String>>,
    },
    /// Standard plain text paragraph or line.
    Text(String),
    /// Unrecognized or fallback content.
    Unknown(String),
}

impl DocumentElement {
    /// Creates a new block directive document element with name and children.
    pub fn block_directive(name: impl Into<String>, children: Vec<DocumentElement>) -> Self {
        Self::BlockDirective {
            children,
            name: name.into(),
        }
    }

    /// Creates a new bullet list item document element with depth and child content.
    pub fn bullet_list_item(depth: usize, content: DocumentElement) -> Self {
        Self::BulletListItem {
            content: Box::new(content),
            depth,
        }
    }

    /// Creates a new checkbox list item document element with depth, checked state, and child content.
    pub fn check_box_item(depth: usize, checked: bool, content: DocumentElement) -> Self {
        Self::CheckBoxItem {
            checked,
            content: Box::new(content),
            depth,
        }
    }

    /// Creates a new code block document element.
    pub fn code_block(language: Option<impl Into<String>>, content: impl Into<String>) -> Self {
        Self::CodeBlock {
            content: content.into(),
            language: language.map(Into::into),
        }
    }

    /// Creates a new image document element with alt text and target URL.
    pub fn image(alt: impl Into<String>, url: impl Into<String>) -> Self {
        Self::Image {
            alt: alt.into(),
            url: url.into(),
        }
    }

    /// Creates a new ordered list item document element with depth, position, and child content.
    pub fn ordered_list_item(depth: usize, position: usize, content: DocumentElement) -> Self {
        Self::OrderedListItem {
            content: Box::new(content),
            depth,
            position,
        }
    }

    /// Creates a new section document element with level, title, and children.
    pub fn section(
        level: usize,
        title: impl Into<String>,
        children: Vec<DocumentElement>,
    ) -> Self {
        Self::Section {
            children,
            level,
            title: title.into(),
        }
    }

    /// Creates a new shoutout document element.
    pub fn shoutout(kind: ShoutoutElementKind, content: impl Into<String>) -> Self {
        Self::Shoutout {
            content: content.into(),
            kind,
        }
    }

    /// Creates a new table document element with column alignments and row matrix.
    pub fn table(alignments: Vec<TableAlignment>, rows: Vec<Vec<String>>) -> Self {
        Self::Table { alignments, rows }
    }

    /// Creates a new plain text document element.
    pub fn text(content: impl Into<String>) -> Self {
        Self::Text(content.into())
    }

    /// Creates a new unknown fallback document element.
    pub fn unknown(content: impl Into<String>) -> Self {
        Self::Unknown(content.into())
    }

    /// Appends a child element if this element is a container (section or block directive).
    pub fn push_child(&mut self, child: Self) {
        match self {
            Self::BlockDirective { children, .. } | Self::Section { children, .. } => {
                children.push(child);
            }
            _ => {}
        }
    }
}

/// Classification of a shoutout element kind.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
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

impl Display for ShoutoutElementKind {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

/// Text alignment specification for a table column.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum TableAlignment {
    /// Center-aligned column text (`:---:`).
    Center,
    /// Left-aligned column text (`:---`).
    Left,
    /// Default or unspecified alignment (`---`).
    #[default]
    None,
    /// Right-aligned column text (`---:`).
    Right,
}

impl TableAlignment {
    /// Returns the static lowercase string identifier.
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Center => "center",
            Self::Left => "left",
            Self::None => "none",
            Self::Right => "right",
        }
    }
}

impl Display for TableAlignment {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_block_directive_element_creation_and_push_child() {
        let child1 = DocumentElement::text("Inside block");
        let mut directive = DocumentElement::block_directive("variables", vec![child1.clone()]);

        assert_eq!(
            directive,
            DocumentElement::BlockDirective {
                children: vec![child1.clone()],
                name: "variables".into(),
            }
        );

        let child2 = DocumentElement::bullet_list_item(0, DocumentElement::text("List item"));
        directive.push_child(child2.clone());

        assert_eq!(
            directive,
            DocumentElement::BlockDirective {
                children: vec![child1, child2],
                name: "variables".into(),
            }
        );
    }

    #[test]
    fn test_bullet_list_item_creation() {
        let elem = DocumentElement::bullet_list_item(2, DocumentElement::text("Nested item"));
        assert_eq!(
            elem,
            DocumentElement::BulletListItem {
                content: Box::new(DocumentElement::text("Nested item")),
                depth: 2,
            }
        );
    }

    #[test]
    fn test_check_box_item_creation() {
        let unchecked = DocumentElement::check_box_item(0, false, DocumentElement::text("Pending task"));
        assert_eq!(
            unchecked,
            DocumentElement::CheckBoxItem {
                checked: false,
                content: Box::new(DocumentElement::text("Pending task")),
                depth: 0,
            }
        );

        let checked = DocumentElement::check_box_item(2, true, DocumentElement::text("Completed subtask"));
        assert_eq!(
            checked,
            DocumentElement::CheckBoxItem {
                checked: true,
                content: Box::new(DocumentElement::text("Completed subtask")),
                depth: 2,
            }
        );
    }

    #[test]
    fn test_code_block_creation() {
        let elem_with_lang = DocumentElement::code_block(Some("bash"), "echo 'hi'");
        assert_eq!(
            elem_with_lang,
            DocumentElement::CodeBlock {
                content: "echo 'hi'".into(),
                language: Some("bash".into()),
            }
        );

        let elem_without_lang = DocumentElement::code_block(None::<String>, "plain code");
        assert_eq!(
            elem_without_lang,
            DocumentElement::CodeBlock {
                content: "plain code".into(),
                language: None,
            }
        );
    }

    #[test]
    fn test_document_element_creation() {
        let elem = DocumentElement::text("Hello world");
        assert_eq!(elem, DocumentElement::Text("Hello world".into()));

        let mut section = DocumentElement::section(1, "Heading", Vec::new());
        let child = DocumentElement::unknown("Child node");
        section.push_child(child.clone());
        assert_eq!(
            section,
            DocumentElement::Section {
                children: vec![child],
                level: 1,
                title: "Heading".into(),
            }
        );
    }

    #[test]
    fn test_document_push_body_and_header() {
        let mut doc = Document::with_capacity(4);
        assert!(doc.parameters.is_empty());
        assert!(doc.header.is_empty());
        assert!(doc.body.is_empty());

        doc.insert_parameter("title", "My Doc");
        doc.push_body(DocumentElement::text("Line 1"));
        doc.push_header(DocumentElement::text("Header Line"));
        assert_eq!(doc.parameters.get("title").map(|s| s.as_str()), Some("My Doc"));
        assert_eq!(doc.body.len(), 1);
        assert_eq!(doc.header.len(), 1);
        assert_eq!(doc.body[0], DocumentElement::Text("Line 1".into()));
        assert_eq!(doc.header[0], DocumentElement::Text("Header Line".into()));
    }

    #[test]
    fn test_image_creation() {
        let elem = DocumentElement::image("Architecture Diagram", "images/arch.png");
        assert_eq!(
            elem,
            DocumentElement::Image {
                alt: "Architecture Diagram".into(),
                url: "images/arch.png".into(),
            }
        );
    }

    #[test]
    fn test_ordered_list_item_creation() {
        let root_item = DocumentElement::ordered_list_item(0, 1, DocumentElement::text("First item"));
        assert_eq!(
            root_item,
            DocumentElement::OrderedListItem {
                content: Box::new(DocumentElement::text("First item")),
                depth: 0,
                position: 1,
            }
        );

        let nested_item = DocumentElement::ordered_list_item(2, 5, DocumentElement::text("Deep item"));
        assert_eq!(
            nested_item,
            DocumentElement::OrderedListItem {
                content: Box::new(DocumentElement::text("Deep item")),
                depth: 2,
                position: 5,
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
            assert_eq!(format!("{kind}"), expected_str);
            let elem = DocumentElement::shoutout(kind, format!("{expected_str} content"));
            assert_eq!(
                elem,
                DocumentElement::Shoutout {
                    content: format!("{expected_str} content"),
                    kind,
                }
            );
        }
    }

    #[test]
    fn test_table_alignment_as_str() {
        assert_eq!(TableAlignment::Center.as_str(), "center");
        assert_eq!(TableAlignment::Left.as_str(), "left");
        assert_eq!(TableAlignment::None.as_str(), "none");
        assert_eq!(TableAlignment::Right.as_str(), "right");
        assert_eq!(format!("{}", TableAlignment::Center), "center");
        assert_eq!(TableAlignment::default(), TableAlignment::None);
    }

    #[test]
    fn test_table_element_creation_and_alignments() {
        let alignments = vec![
            TableAlignment::Left,
            TableAlignment::Center,
            TableAlignment::Right,
            TableAlignment::None,
        ];
        let rows = vec![
            vec!["H1".into(), "H2".into(), "H3".into(), "H4".into()],
            vec!["D1".into(), "D2".into(), "D3".into(), "D4".into()],
        ];
        let table = DocumentElement::table(alignments.clone(), rows.clone());

        assert_eq!(
            table,
            DocumentElement::Table {
                alignments,
                rows,
            }
        );
    }
}
