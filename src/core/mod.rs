//! Core Doc2Flow engine and document pipeline.

pub mod arguments;
pub mod builder;
pub mod constants;
pub mod document;
pub mod error;
pub mod feature;
pub mod format;
pub mod language;
pub mod markdown;
pub mod utils;

pub use arguments::{Args, help_message, parse_args};
pub use builder::{build, render_element};
pub use constants::{
    APP_NAME, APP_VERSION, CLI_ALIAS, CLI_BANNER, LICENSE_TERMS, LICENSE_URL, REPOSITORY_URL,
};
pub use document::{Document, DocumentElement, ShoutoutElementKind, TableAlignment};
pub use error::{DiagnosticError, Error, Result, build_caret_annotation};
pub use feature::{DocumentFeature, Feature, to_features_string};
pub use format::{escape_html_into, format_inline_into, push_indent};
pub use language::get_language_json;
pub use markdown::parse_d2f_markdown;
