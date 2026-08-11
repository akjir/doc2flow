//! Core Doc2Flow engine and document pipeline.

pub mod build;
pub mod dev_helper;
pub mod document;
pub mod error;
pub mod parse;
pub mod utils;

pub use build::builder;
pub use build::builder::build;
pub use dev_helper::document_to_json;
pub use document::{Document, DocumentElement, ShoutoutElementKind, TableAlignment};
pub use error::{DiagnosticError, Error, Result, build_caret_annotation};
pub use parse::arguments::{Args, help_message, parse_args};
pub use parse::markdown::parse_d2f_markdown;
pub use parse::parser;
pub use parse::parser::parse;
