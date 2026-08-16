//! Core Doc2Flow engine and document pipeline.

pub mod arguments;
pub mod builder;
pub mod constants;
pub mod document;
pub mod error;
pub mod feature;
pub mod format;
pub mod io;
pub mod language;
pub mod markdown;
pub mod renderer;

pub use arguments::{Args, help_message, parse_args};
pub use builder::{assemble_assets, build};
pub use constants::{
    APP_NAME, APP_VERSION, CLI_ALIAS, CLI_BANNER, LICENSE_TERMS, LICENSE_URL, REPOSITORY_URL,
};
pub use document::{
    Document, DocumentElement, DocumentElementId, DocumentHeader, DocumentParameters,
    ShoutoutElementKind, TableAlignment,
};
pub use error::{build_caret_annotation, CliError, DiagnosticError, Error, Result};
pub use feature::FeatureModule;
pub use format::{append_indented, escape_html_into, format_inline_into, push_indent};
pub use language::get_language_json;
pub use markdown::parse_d2f_markdown;
pub use renderer::{DocumentElementRenderer, HtmlRenderer, render_element, render_element_into};
