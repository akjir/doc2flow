//! Document model and hierarchical element definitions.

use std::collections::HashMap;
use std::fmt::{self, Display, Formatter};

use crate::core::error::{Error, Result};

/// Represents a parsed document tree.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct Document {
    /// Main body document elements.
    pub body: Vec<DocumentElement>,
    /// Header document elements and configuration.
    pub head: DocumentHeader,
    /// Frontmatter and document configuration parameters.
    pub parameters: DocumentParameters,
}

impl Document {
    /// Creates an empty document.
    #[must_use]
    pub fn new() -> Self {
        Self {
            body: Vec::new(),
            head: DocumentHeader::new(),
            parameters: DocumentParameters::default(),
        }
    }

    /// Creates an empty document with pre-allocated capacities.
    #[must_use]
    pub fn with_capacity(body_capacity: usize) -> Self {
        Self {
            body: Vec::with_capacity(body_capacity),
            head: DocumentHeader::new(),
            parameters: DocumentParameters::default(),
        }
    }

    /// Appends a body element.
    pub fn push_body(&mut self, element: DocumentElement) {
        self.body.push(element);
    }
}

/// Category or syntactic kind of a markdown directive.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DirectiveKind {
    /// Inline text directive within prose (e.g. `:name[label]{attributes}`).
    Text,
    /// Standalone leaf block directive (e.g. `::name[label]{attributes}`).
    Leaf,
    /// Container block directive enclosed by colons (e.g. `:::name[label]{attributes}\n...\n:::`).
    Block,
}

/// Hierarchical document element representation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DocumentElement {
    /// Generic directive representation across inline, leaf, and block forms.
    Directive {
        /// Directive attributes string (e.g. `{url=...}`).
        attributes: String,
        /// Inner text content (only present for block directives).
        content: String,
        /// Category or syntactic kind of the directive.
        kind: DirectiveKind,
        /// Directive label string (e.g. `[label]`).
        label: String,
        /// Directive name identifier (e.g. `variables`).
        name: String,
    },
    /// Bullet list item element with child content.
    BulletListItem {
        /// Nested child list items.
        children: Vec<DocumentElement>,
        /// Child list item text content.
        content: String,
    },
    /// Checkbox list item element with checked status and child content.
    CheckBoxItem {
        /// Indicates whether the checkbox is checked.
        checked: bool,
        /// Nested child list items.
        children: Vec<DocumentElement>,
        /// Child list item text content.
        content: String,
    },
    /// Fenced code block with optional language specifier and content.
    CodeBlock {
        /// Code block text content.
        content: String,
        /// Optional programming or markup language identifier (info string).
        language: Option<String>,
    },
    /// Top-level document header card containing title, subtitle, and logo.
    Header {
        /// Custom logo image path, Base64 URI, or inline SVG markup.
        logo: Option<String>,
        /// Optional subtitle or secondary description text.
        subtitle: Option<String>,
        /// Main document title.
        title: String,
    },
    /// Horizontal divider or thematic break rule line.
    HorizontalRule,
    /// Image element with alt text and source target URL.
    Image {
        /// Alternative descriptive text for the image.
        alt: String,
        /// Source URL or file path location of the image.
        url: String,
    },
    /// Input form field element with text content.
    Input {
        /// Input element text content.
        text: String,
    },
    /// Ordered list item element with sequential position and child content.
    OrderedListItem {
        /// Nested child list items.
        children: Vec<DocumentElement>,
        /// Child list item text content.
        content: String,
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
    /// Dynamic variables key-value mapping extracted from a table variables block directive.
    TableVariables {
        /// Key-value variable mapping.
        variables: HashMap<String, String>,
    },
    /// Standard plain text paragraph or line.
    Text(String),
    /// Unrecognized or fallback content.
    Unknown(String),
}

impl DocumentElement {
    /// Creates a new directive document element.
    #[must_use]
    pub fn directive(
        kind: DirectiveKind,
        name: impl Into<String>,
        label: impl Into<String>,
        attributes: impl Into<String>,
        content: impl Into<String>,
    ) -> Self {
        Self::Directive {
            attributes: attributes.into(),
            content: content.into(),
            kind,
            label: label.into(),
            name: name.into(),
        }
    }

    /// Creates a new bullet list item document element with child content.
    #[must_use]
    pub fn bullet_list_item(content: impl Into<String>) -> Self {
        Self::BulletListItem {
            children: Vec::new(),
            content: content.into(),
        }
    }

    /// Creates a new checkbox list item document element with checked state and child content.
    #[must_use]
    pub fn check_box_item(checked: bool, content: impl Into<String>) -> Self {
        Self::CheckBoxItem {
            checked,
            children: Vec::new(),
            content: content.into(),
        }
    }

    /// Creates a new code block document element.
    #[must_use]
    pub fn code_block(language: Option<impl Into<String>>, content: impl Into<String>) -> Self {
        Self::CodeBlock {
            content: content.into(),
            language: language.map(Into::into),
        }
    }

    /// Creates a new header document element with title, optional subtitle, and optional logo.
    #[must_use]
    pub fn header(
        title: impl Into<String>,
        subtitle: Option<impl Into<String>>,
        logo: Option<impl Into<String>>,
    ) -> Self {
        Self::Header {
            logo: logo.map(Into::into),
            subtitle: subtitle.map(Into::into),
            title: title.into(),
        }
    }

    /// Creates a new horizontal rule document element.
    #[must_use]
    pub const fn horizontal_rule() -> Self {
        Self::HorizontalRule
    }

    /// Creates a new image document element with alt text and target URL.
    #[must_use]
    pub fn image(alt: impl Into<String>, url: impl Into<String>) -> Self {
        Self::Image {
            alt: alt.into(),
            url: url.into(),
        }
    }

    /// Creates a new input document element.
    #[must_use]
    pub fn input(text: impl Into<String>) -> Self {
        Self::Input { text: text.into() }
    }

    /// Returns true if this element is a list item variant.
    #[must_use]
    pub const fn is_list_item(&self) -> bool {
        matches!(
            self,
            Self::BulletListItem { .. } | Self::CheckBoxItem { .. } | Self::OrderedListItem { .. }
        )
    }

    /// Creates a new ordered list item document element with position and child content.
    #[must_use]
    pub fn ordered_list_item(position: usize, content: impl Into<String>) -> Self {
        Self::OrderedListItem {
            children: Vec::new(),
            content: content.into(),
            position,
        }
    }

    /// Creates a new section document element with level, title, and children.
    #[must_use]
    pub fn section(level: usize, title: impl Into<String>, children: Vec<DocumentElement>) -> Self {
        Self::Section {
            children,
            level,
            title: title.into(),
        }
    }

    /// Creates a new shoutout document element.
    #[must_use]
    pub fn shoutout(kind: ShoutoutElementKind, content: impl Into<String>) -> Self {
        Self::Shoutout {
            content: content.into(),
            kind,
        }
    }

    /// Creates a new table document element with column alignments and row matrix.
    #[must_use]
    pub fn table(alignments: Vec<TableAlignment>, rows: Vec<Vec<String>>) -> Self {
        Self::Table { alignments, rows }
    }

    /// Creates a new table variables document element with key-value pairs.
    #[must_use]
    pub fn table_variables(variables: HashMap<String, String>) -> Self {
        Self::TableVariables { variables }
    }

    /// Creates a new plain text document element.
    #[must_use]
    pub fn text(content: impl Into<String>) -> Self {
        Self::Text(content.into())
    }

    /// Creates a new unknown fallback document element.
    #[must_use]
    pub fn unknown(content: impl Into<String>) -> Self {
        Self::Unknown(content.into())
    }

    /// Returns the corresponding [`DocumentElementId`] for this element.
    #[must_use]
    pub const fn element_id(&self) -> DocumentElementId {
        match self {
            Self::Directive { .. } => DocumentElementId::Directive,
            Self::BulletListItem { .. } => DocumentElementId::BulletListItem,
            Self::CheckBoxItem { .. } => DocumentElementId::CheckBoxItem,
            Self::CodeBlock { .. } => DocumentElementId::CodeBlock,
            Self::Header { .. } => DocumentElementId::Header,
            Self::HorizontalRule => DocumentElementId::HorizontalRule,
            Self::Image { .. } => DocumentElementId::Image,
            Self::Input { .. } => DocumentElementId::Input,
            Self::OrderedListItem { .. } => DocumentElementId::OrderedListItem,
            Self::Section { .. } => DocumentElementId::Section,
            Self::Shoutout { .. } => DocumentElementId::Shoutout,
            Self::Table { .. } => DocumentElementId::Table,
            Self::TableVariables { .. } => DocumentElementId::TableVariables,
            Self::Text(_) => DocumentElementId::Text,
            Self::Unknown(_) => DocumentElementId::Unknown,
        }
    }

    /// Appends a child element to this container element.
    ///
    /// # Errors
    ///
    /// Returns [`Error`] if adding to a non-container or if nesting rules are violated.
    pub fn push_child(&mut self, child: Self) -> Result<()> {
        match self {
            Self::Section {
                level: parent_level,
                children,
                ..
            } => {
                if let Self::Section {
                    level: child_level, ..
                } = &child
                {
                    // Strict nesting rule: In Doc2Flow, H1 and H2 sections can strictly accept only H3 section children.
                    // This is an intentional, Doc2Flow-specific requirement driven by the final HTML rendering layout
                    // and collapsible section structure.
                    if (*parent_level == 1 || *parent_level == 2) && *child_level == 3 {
                        children.push(child);
                        Ok(())
                    } else {
                        Err(Error::Message(format!(
                            "cannot add h{child_level} section to h{parent_level} section"
                        )))
                    }
                } else {
                    children.push(child);
                    Ok(())
                }
            }
            Self::BulletListItem { children, .. }
            | Self::CheckBoxItem { children, .. }
            | Self::OrderedListItem { children, .. } => {
                if child.is_list_item() {
                    children.push(child);
                    Ok(())
                } else {
                    Err(Error::Message(
                        "cannot add non-list-item child to list item".into(),
                    ))
                }
            }
            _ => Err(Error::Message(
                "cannot add child to non-container element".into(),
            )),
        }
    }
}

/// Discriminant identifier representing each variant of [`DocumentElement`].
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
#[repr(u8)]
pub enum DocumentElementId {
    /// Identifier for generic directives.
    Directive = 0,
    /// Identifier for bullet list items.
    BulletListItem = 1,
    /// Identifier for checkbox list items.
    CheckBoxItem = 2,
    /// Identifier for code blocks.
    CodeBlock = 3,
    /// Identifier for header card banner.
    Header = 4,
    /// Identifier for horizontal rules.
    HorizontalRule = 5,
    /// Identifier for images.
    Image = 6,
    /// Identifier for input elements.
    Input = 7,
    /// Identifier for ordered list items.
    OrderedListItem = 8,
    /// Identifier for sections.
    Section = 9,
    /// Identifier for shoutouts.
    Shoutout = 10,
    /// Identifier for tables.
    Table = 11,
    /// Identifier for table variables mapping.
    TableVariables = 12,
    /// Identifier for plain text lines.
    Text = 13,
    /// Identifier for unrecognized fallback content.
    Unknown = 14,
}

impl DocumentElementId {
    /// Total number of distinct document element identifiers.
    pub const COUNT: usize = 15;
}

/// Represents document header elements and configuration.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct DocumentHeader {
    /// Header banner document element.
    pub header: Option<DocumentElement>,
    /// Dynamic variables table element.
    pub variables: Option<DocumentElement>,
}

impl DocumentHeader {
    /// Creates a new empty document header.
    #[must_use]
    pub fn new() -> Self {
        Self {
            header: None,
            variables: None,
        }
    }

    /// Returns a mutable reference to the underlying variables map, initializing it lazily if empty.
    pub fn variables_mut(&mut self) -> &mut HashMap<String, String> {
        let element = self
            .variables
            .get_or_insert_with(|| DocumentElement::table_variables(HashMap::new()));

        let DocumentElement::TableVariables { variables } = element else {
            unreachable!("DocumentHeader variables slot contains invalid AST element");
        };

        variables
    }

    /// Inserts or updates a variable in the variables table.
    pub fn insert_variable(&mut self, key: impl Into<String>, value: impl Into<String>) {
        self.variables_mut().insert(key.into(), value.into());
    }

    /// Ensures a variable is present in the variables table, setting it to a fallback value if not already present.
    pub fn ensure_variable(&mut self, key: &str, fallback_value: &str) {
        let vars = self.variables_mut();

        // P-MIN-ALLOC: Avoid Entry API (vars.entry(key.to_string()))
        // to prevent unconditional heap allocations when key already exists.
        if !vars.contains_key(key) {
            vars.insert(key.to_string(), fallback_value.to_string());
        }
    }
}

/// Document metadata and configuration options extracted from frontmatter.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DocumentParameters {
    /// Protocol or document date string.
    pub date: String,
    /// Whether the top header banner is enabled.
    pub header: bool,
    /// Language code for static UI translation (e.g. `en`, `de`).
    pub language: String,
    /// Path or URL to a custom logo image.
    pub logo: String,
    /// Whether automatic section numbering for headings is enabled.
    pub numbered_sections: bool,
    /// Subtitle or secondary description text.
    pub subtitle: String,
    /// Main document title.
    pub title: String,
    /// Dynamic document variables and custom frontmatter parameters.
    pub variables: HashMap<String, String>,
    /// Document version string.
    pub version: String,
}

impl DocumentParameters {
    /// Creates a new document parameters instance with default settings.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Creates document parameters by consuming a frontmatter key-value map.
    #[must_use]
    pub fn from_map(map: HashMap<String, String>) -> Self {
        Self::from(map)
    }

    /// Returns a reference to a variable value if present.
    #[must_use]
    pub fn get_variable(&self, key: &str) -> Option<&str> {
        self.variables.get(key).map(String::as_str)
    }

    /// Removes a variable by key, returning the previous value if present.
    pub fn remove_variable(&mut self, key: &str) -> Option<String> {
        self.variables.remove(key)
    }

    /// Inserts or updates a variable in the dynamic variables map.
    pub fn set_variable(&mut self, key: impl Into<String>, value: impl Into<String>) {
        self.variables.insert(key.into(), value.into());
    }
}

impl Default for DocumentParameters {
    fn default() -> Self {
        Self {
            date: String::new(),
            header: true,
            language: "en".to_string(),
            logo: String::new(),
            numbered_sections: false,
            subtitle: String::new(),
            title: String::new(),
            variables: HashMap::new(),
            version: String::new(),
        }
    }
}

impl From<HashMap<String, String>> for DocumentParameters {
    fn from(mut map: HashMap<String, String>) -> Self {
        let title = map.remove("title").unwrap_or_default();
        let subtitle = map.remove("subtitle").unwrap_or_default();
        let date = map.remove("date").unwrap_or_default();
        let version = map.remove("version").unwrap_or_default();
        let language = map.remove("language").unwrap_or_else(|| "en".to_string());
        let logo = map.remove("logo").unwrap_or_default();
        let header = match map.remove("header") {
            Some(val) => {
                const TRUTHY_VALUES: &[&str] = &["true", "yes", "y", "1"];
                let trimmed = val.trim();
                TRUTHY_VALUES
                    .iter()
                    .any(|&truthy| truthy.eq_ignore_ascii_case(trimmed))
            }
            None => true,
        };
        let numbered_sections = match map.remove("numbered_sections") {
            Some(val) => {
                const TRUTHY_VALUES: &[&str] = &["true", "yes", "y", "1"];
                let trimmed = val.trim();
                TRUTHY_VALUES
                    .iter()
                    .any(|&truthy| truthy.eq_ignore_ascii_case(trimmed))
            }
            None => false,
        };

        Self {
            date,
            header,
            language,
            logo,
            numbered_sections,
            subtitle,
            title,
            variables: map,
            version,
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
    #[must_use]
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
    #[must_use]
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
    fn test_directive_element_creation_and_push_child() {
        let mut directive = DocumentElement::directive(
            DirectiveKind::Block,
            "variables",
            "My Label",
            "attr=1",
            "Body content",
        );

        assert_eq!(
            directive,
            DocumentElement::Directive {
                attributes: "attr=1".into(),
                content: "Body content".into(),
                kind: DirectiveKind::Block,
                label: "My Label".into(),
                name: "variables".into(),
            }
        );

        let child = DocumentElement::bullet_list_item("List item");
        assert!(directive.push_child(child).is_err());
    }

    #[test]
    fn test_bullet_list_item_creation() {
        let elem = DocumentElement::bullet_list_item("Nested item");
        assert_eq!(
            elem,
            DocumentElement::BulletListItem {
                children: Vec::new(),
                content: "Nested item".into(),
            }
        );
    }

    #[test]
    fn test_check_box_item_creation() {
        let unchecked = DocumentElement::check_box_item(false, "Pending task");
        assert_eq!(
            unchecked,
            DocumentElement::CheckBoxItem {
                checked: false,
                children: Vec::new(),
                content: "Pending task".into(),
            }
        );

        let checked = DocumentElement::check_box_item(true, "Completed subtask");
        assert_eq!(
            checked,
            DocumentElement::CheckBoxItem {
                checked: true,
                children: Vec::new(),
                content: "Completed subtask".into(),
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
        assert!(section.push_child(child.clone()).is_ok());
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
    fn test_document_parameters_boolean_parsing() {
        let truthy_cases = [
            "true", "True", "TRUE", "yes", "Yes", "YES", "y", "Y", "1", " true ", " yes ", " 1 ",
        ];
        for val in truthy_cases {
            let mut map = HashMap::new();
            map.insert("numbered_sections".into(), val.into());
            let params = DocumentParameters::from(map);
            assert!(
                params.numbered_sections,
                "Expected '{val}' to evaluate to true"
            );
        }

        let falsy_cases = [
            "false", "False", "FALSE", "no", "No", "0", "n", "N", "random", "", "off",
        ];
        for val in falsy_cases {
            let mut map = HashMap::new();
            map.insert("numbered_sections".into(), val.into());
            let params = DocumentParameters::from(map);
            assert!(
                !params.numbered_sections,
                "Expected '{val}' to evaluate to false"
            );
        }

        let empty_params = DocumentParameters::from(HashMap::new());
        assert!(!empty_params.numbered_sections);
    }

    #[test]
    fn test_document_parameters_operations() {
        let params = DocumentParameters::new();
        assert!(!params.numbered_sections);
        assert_eq!(params.title, "");
        assert_eq!(params.subtitle, "");
        assert_eq!(params.date, "");
        assert_eq!(params.version, "");
        assert_eq!(params.language, "en");
        assert_eq!(params.logo, "");
        assert!(params.header);
        assert!(params.variables.is_empty());

        let mut map = HashMap::new();
        map.insert("title".into(), "Spec Title".into());
        map.insert("subtitle".into(), "Spec Subtitle".into());
        map.insert("date".into(), "2026-08-15".into());
        map.insert("version".into(), "1.2.3".into());
        map.insert("language".into(), "de".into());
        map.insert("logo".into(), "img/logo.svg".into());
        map.insert("header".into(), "false".into());
        map.insert("numbered_sections".into(), "false".into());
        map.insert("custom_key".into(), "custom_val".into());
        map.insert("env".into(), "production".into());

        let mut from_map = DocumentParameters::from_map(map);
        assert_eq!(from_map.title, "Spec Title");
        assert_eq!(from_map.subtitle, "Spec Subtitle");
        assert_eq!(from_map.date, "2026-08-15");
        assert_eq!(from_map.version, "1.2.3");
        assert_eq!(from_map.language, "de");
        assert_eq!(from_map.logo, "img/logo.svg");
        assert!(!from_map.header);
        assert!(!from_map.numbered_sections);

        // Variables map contains unknown frontmatter parameters
        assert_eq!(from_map.get_variable("custom_key"), Some("custom_val"));
        assert_eq!(from_map.get_variable("env"), Some("production"));
        assert_eq!(
            from_map.variables.get("custom_key").map(|s| s.as_str()),
            Some("custom_val")
        );
        assert_eq!(from_map.get_variable("unknown_key"), None);

        // Mutating variables map
        from_map.set_variable("port", "8080");
        assert_eq!(from_map.get_variable("port"), Some("8080"));
        assert_eq!(
            from_map.remove_variable("custom_key"),
            Some("custom_val".into())
        );
        assert_eq!(from_map.get_variable("custom_key"), None);

        // Unrecognized 'lang' key is preserved in variables without overriding language
        let mut lang_map = HashMap::new();
        lang_map.insert("lang".into(), "de".into());
        let lang_params = DocumentParameters::from(lang_map);
        assert_eq!(lang_params.language, "en");
        assert_eq!(lang_params.get_variable("lang"), Some("de"));
    }

    #[test]
    fn test_document_header_creation() {
        let header = DocumentHeader::new();
        assert_eq!(header, DocumentHeader::default());
        assert_eq!(header.header, None);
        assert_eq!(header.variables, None);
    }

    #[test]
    fn test_document_header_lazy_init() {
        let mut header = DocumentHeader::new();
        assert_eq!(header.variables, None);

        // Accessing variables_mut lazily initializes the TableVariables AST node
        let vars = header.variables_mut();
        assert!(vars.is_empty());
        assert!(matches!(
            header.variables,
            Some(DocumentElement::TableVariables { .. })
        ));
    }

    #[test]
    fn test_ensure_variable_no_overwrite() {
        let mut header = DocumentHeader::new();
        header.insert_variable("PORT", "8080");

        // ensure_variable must not overwrite existing value
        header.ensure_variable("PORT", "9090");
        assert_eq!(
            header.variables_mut().get("PORT").map(String::as_str),
            Some("8080")
        );

        // ensure_variable sets fallback when key is not present
        header.ensure_variable("HOST", "127.0.0.1");
        assert_eq!(
            header.variables_mut().get("HOST").map(String::as_str),
            Some("127.0.0.1")
        );
    }

    #[test]
    fn test_document_header_variable_operations() {
        let mut header = DocumentHeader::new();
        assert_eq!(header.variables, None);

        // Ensure variable when None creates TableVariables with fallback
        header.ensure_variable("PORT", "8080");
        let mut expected = HashMap::new();
        expected.insert("PORT".into(), "8080".into());
        assert_eq!(
            header.variables,
            Some(DocumentElement::table_variables(expected.clone()))
        );

        // Ensure variable does not overwrite existing value
        header.ensure_variable("PORT", "9090");
        assert_eq!(
            header.variables,
            Some(DocumentElement::table_variables(expected.clone()))
        );

        // Insert variable overwrites existing value
        header.insert_variable("PORT", "9090");
        expected.insert("PORT".into(), "9090".into());
        assert_eq!(
            header.variables,
            Some(DocumentElement::table_variables(expected.clone()))
        );

        // Ensure another variable appends to existing map
        header.ensure_variable("HOST", "127.0.0.1");
        expected.insert("HOST".into(), "127.0.0.1".into());
        assert_eq!(
            header.variables,
            Some(DocumentElement::table_variables(expected))
        );
    }

    #[test]
    fn test_document_push_body_and_header() {
        let mut doc = Document::with_capacity(4);
        assert_eq!(doc.parameters.title, "");
        assert_eq!(doc.head.variables, None);
        assert!(doc.body.is_empty());

        doc.parameters.title = "My Doc".into();
        doc.push_body(DocumentElement::text("Line 1"));
        doc.head.variables = Some(DocumentElement::table(
            vec![TableAlignment::None],
            vec![vec!["Var".into()]],
        ));
        assert_eq!(doc.parameters.title, "My Doc");
        assert_eq!(doc.body.len(), 1);
        assert!(doc.head.variables.is_some());
        assert_eq!(doc.body[0], DocumentElement::Text("Line 1".into()));
    }

    #[test]
    fn test_horizontal_rule_creation() {
        let elem = DocumentElement::horizontal_rule();
        assert_eq!(elem, DocumentElement::HorizontalRule);

        let mut section = DocumentElement::section(1, "Section With HR", Vec::new());
        assert!(section.push_child(elem).is_ok());
        assert_eq!(
            section,
            DocumentElement::Section {
                children: vec![DocumentElement::HorizontalRule],
                level: 1,
                title: "Section With HR".into(),
            }
        );
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
    fn test_input_element_creation() {
        let elem = DocumentElement::input("server_host");
        assert_eq!(
            elem,
            DocumentElement::Input {
                text: "server_host".into(),
            }
        );
    }

    #[test]
    fn test_list_items_push_child_allowed_and_forbidden() {
        let mut bullet = DocumentElement::bullet_list_item("Parent bullet");
        let child_bullet = DocumentElement::bullet_list_item("Child bullet");
        let child_order = DocumentElement::ordered_list_item(1, "Child order");
        let child_check = DocumentElement::check_box_item(false, "Child check");

        assert!(bullet.push_child(child_bullet.clone()).is_ok());
        assert!(bullet.push_child(child_order.clone()).is_ok());
        assert!(bullet.push_child(child_check.clone()).is_ok());

        assert!(
            bullet
                .push_child(DocumentElement::text("Non-list item"))
                .is_err()
        );
        assert!(
            bullet
                .push_child(DocumentElement::horizontal_rule())
                .is_err()
        );

        let mut order = DocumentElement::ordered_list_item(1, "Parent order");
        assert!(order.push_child(child_bullet.clone()).is_ok());
        assert!(
            order
                .push_child(DocumentElement::text("Non-list item"))
                .is_err()
        );

        let mut check = DocumentElement::check_box_item(false, "Parent check");
        assert!(check.push_child(child_order).is_ok());
        assert!(
            check
                .push_child(DocumentElement::text("Non-list item"))
                .is_err()
        );
    }

    #[test]
    fn test_non_container_push_child_error() {
        let mut text_elem = DocumentElement::text("Plain text");
        let child = DocumentElement::text("Other text");
        assert!(text_elem.push_child(child).is_err());
    }

    #[test]
    fn test_ordered_list_item_creation() {
        let root_item = DocumentElement::ordered_list_item(1, "First item");
        assert_eq!(
            root_item,
            DocumentElement::OrderedListItem {
                children: Vec::new(),
                content: "First item".into(),
                position: 1,
            }
        );

        let nested_item = DocumentElement::ordered_list_item(5, "Deep item");
        assert_eq!(
            nested_item,
            DocumentElement::OrderedListItem {
                children: Vec::new(),
                content: "Deep item".into(),
                position: 5,
            }
        );
    }

    #[test]
    fn test_section_push_child_allowed_permutations() {
        // H3 can be added to H1
        let mut h1 = DocumentElement::section(1, "H1", Vec::new());
        let h3_a = DocumentElement::section(3, "H3 A", Vec::new());
        assert!(h1.push_child(h3_a).is_ok());

        // H3 can be added to H2
        let mut h2 = DocumentElement::section(2, "H2", Vec::new());
        let h3_b = DocumentElement::section(3, "H3 B", Vec::new());
        assert!(h2.push_child(h3_b).is_ok());

        // Non-section elements can be added to H1, H2, and H3
        let mut h3 = DocumentElement::section(3, "H3", Vec::new());
        assert!(h1.push_child(DocumentElement::text("Text in H1")).is_ok());
        assert!(h2.push_child(DocumentElement::text("Text in H2")).is_ok());
        assert!(h3.push_child(DocumentElement::text("Text in H3")).is_ok());
    }

    #[test]
    fn test_section_push_child_forbidden_permutations() {
        // H2 into H2 -> error
        let mut h2_parent = DocumentElement::section(2, "H2 Parent", Vec::new());
        let h2_child = DocumentElement::section(2, "H2 Child", Vec::new());
        assert!(h2_parent.push_child(h2_child).is_err());

        // H2 into H1 -> error
        let mut h1_parent = DocumentElement::section(1, "H1 Parent", Vec::new());
        let h2_child2 = DocumentElement::section(2, "H2 Child", Vec::new());
        assert!(h1_parent.push_child(h2_child2).is_err());

        // H1 into H1 -> error
        let h1_child = DocumentElement::section(1, "H1 Child", Vec::new());
        assert!(h1_parent.push_child(h1_child).is_err());

        // H1 into H2 -> error
        let h1_child2 = DocumentElement::section(1, "H1 Child", Vec::new());
        assert!(h2_parent.push_child(h1_child2).is_err());

        // H3 into H3 -> error
        let mut h3_parent = DocumentElement::section(3, "H3 Parent", Vec::new());
        let h3_child = DocumentElement::section(3, "H3 Child", Vec::new());
        assert!(h3_parent.push_child(h3_child).is_err());

        // H1 into H3 -> error
        let h1_child3 = DocumentElement::section(1, "H1 Child", Vec::new());
        assert!(h3_parent.push_child(h1_child3).is_err());

        // H2 into H3 -> error
        let h2_child3 = DocumentElement::section(2, "H2 Child", Vec::new());
        assert!(h3_parent.push_child(h2_child3).is_err());
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

        assert_eq!(table, DocumentElement::Table { alignments, rows });
    }

    #[test]
    fn test_table_variables_element_creation() {
        let mut map = HashMap::new();
        map.insert("PORT".into(), "8080".into());
        map.insert("ENV".into(), "prod".into());
        let elem = DocumentElement::table_variables(map.clone());
        assert_eq!(elem, DocumentElement::TableVariables { variables: map });
    }

    #[test]
    fn test_header_element_creation() {
        let elem = DocumentElement::header("My Title", Some("My Subtitle"), Some("logo.svg"));
        assert_eq!(
            elem,
            DocumentElement::Header {
                logo: Some("logo.svg".into()),
                subtitle: Some("My Subtitle".into()),
                title: "My Title".into(),
            }
        );

        let elem_no_opt = DocumentElement::header("Title Only", None::<String>, None::<String>);
        assert_eq!(
            elem_no_opt,
            DocumentElement::Header {
                logo: None,
                subtitle: None,
                title: "Title Only".into(),
            }
        );
    }

    #[test]
    fn test_document_element_id_mapping() {
        assert_eq!(DocumentElementId::COUNT, 15);
        assert_eq!(
            DocumentElement::directive(DirectiveKind::Block, "test", "", "", "").element_id(),
            DocumentElementId::Directive
        );
        assert_eq!(
            DocumentElement::bullet_list_item("item").element_id(),
            DocumentElementId::BulletListItem
        );
        assert_eq!(
            DocumentElement::check_box_item(false, "item").element_id(),
            DocumentElementId::CheckBoxItem
        );
        assert_eq!(
            DocumentElement::code_block(None::<String>, "code").element_id(),
            DocumentElementId::CodeBlock
        );
        assert_eq!(
            DocumentElement::header("t", None::<String>, None::<String>).element_id(),
            DocumentElementId::Header
        );
        assert_eq!(
            DocumentElement::horizontal_rule().element_id(),
            DocumentElementId::HorizontalRule
        );
        assert_eq!(
            DocumentElement::image("alt", "url").element_id(),
            DocumentElementId::Image
        );
        assert_eq!(
            DocumentElement::input("text").element_id(),
            DocumentElementId::Input
        );
        assert_eq!(
            DocumentElement::ordered_list_item(1, "item").element_id(),
            DocumentElementId::OrderedListItem
        );
        assert_eq!(
            DocumentElement::section(1, "title", vec![]).element_id(),
            DocumentElementId::Section
        );
        assert_eq!(
            DocumentElement::shoutout(ShoutoutElementKind::Note, "text").element_id(),
            DocumentElementId::Shoutout
        );
        assert_eq!(
            DocumentElement::table(vec![], vec![]).element_id(),
            DocumentElementId::Table
        );
        assert_eq!(
            DocumentElement::table_variables(HashMap::new()).element_id(),
            DocumentElementId::TableVariables
        );
        assert_eq!(
            DocumentElement::text("text").element_id(),
            DocumentElementId::Text
        );
        assert_eq!(
            DocumentElement::unknown("raw").element_id(),
            DocumentElementId::Unknown
        );
    }
}
