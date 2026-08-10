//! Core Doc2Flow engine and document pipeline.

pub mod dev_helper;
pub mod document;
pub mod error;
pub mod parser;
pub mod parsing;

pub use dev_helper::document_to_json;
pub use document::{Document, DocumentElement, ShoutoutElementKind, TableAlignment};
pub use error::{DiagnosticError, Error, Result, build_caret_annotation};
pub use parsing::arguments::{Args, help_message, parse_args};


