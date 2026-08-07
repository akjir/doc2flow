//! Core architecture abstractions for Doc2Flow.

pub mod feature;
pub mod generator;
pub mod parser;

pub use feature::{DocumentContext, Feature};
pub use generator::{assemble_html, assemble_scripts, assemble_styles, SCRIPT_CORE, STYLE_CORE};
pub use parser::{
    convert_markdown_to_html, convert_markdown_to_html_with_locale,
    convert_markdown_to_html_with_options, parse_and_validate_frontmatter, parse_frontmatter,
    parse_frontmatter_map, validate_frontmatter, DocumentFeatures, Frontmatter,
};
