//! Document build and rendering module.

use crate::core::dev_helper::document_to_json;
use crate::core::document::Document;

/// Builds output content from a structured [`Document`].
///
/// Returns a formatted string implementing `AsRef<[u8]>`.
///
/// # Examples
///
/// ```
/// use doc2flow::core::build::builder::build;
/// use doc2flow::core::document::Document;
///
/// let doc = Document::new();
/// let content = build(&doc);
/// assert!(!content.is_empty());
/// ```
pub fn build(document: &Document) -> String {
    document_to_json(document)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_builder_build_as_ref_u8() {
        let doc = Document::new();
        let content = build(&doc);
        assert!(!content.as_bytes().is_empty());
    }
}
