//! Feature trait and document context for vertical slice feature detection.

use std::collections::HashMap;

/// Document context provided to features during detection and rendering.
pub struct DocumentContext<'a> {
    /// Frontmatter key-value pairs extracted from the markdown header.
    pub frontmatter: &'a HashMap<String, String>,
    /// Raw unparsed markdown content of the document.
    pub raw_markdown: &'a str,
}

impl<'a> DocumentContext<'a> {
    /// Creates a new document context.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::collections::HashMap;
    /// use doc2flow::core::feature::DocumentContext;
    ///
    /// let fm = HashMap::new();
    /// let ctx = DocumentContext::new(&fm, "# Heading");
    /// assert_eq!(ctx.raw_markdown, "# Heading");
    /// ```
    #[inline]
    pub const fn new(frontmatter: &'a HashMap<String, String>, raw_markdown: &'a str) -> Self {
        Self {
            frontmatter,
            raw_markdown,
        }
    }
}

/// Feature trait defining detection and asset provision for vertical slices.
pub trait Feature {
    /// Eindeutiger Bezeichner des Features (z. B. "copy_code")
    fn name(&self) -> &'static str;

    /// Prüft, ob das Feature aktiviert werden soll
    fn is_enabled(&self, ctx: &DocumentContext) -> bool;

    /// Liefert den TypeScript/JavaScript-Code für dieses Feature
    fn javascript(&self) -> Option<&'static str> {
        None
    }

    /// Liefert das CSS für dieses Feature
    fn css(&self) -> Option<&'static str> {
        None
    }
}
