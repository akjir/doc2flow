//! Experimental building pipeline module.

pub mod dev_helper;
pub mod document;
pub mod error;
pub mod parser;
pub mod parsing;

pub use dev_helper::document_to_json;
pub use document::{Document, DocumentElement, DocumentElementKind, ShoutoutElement, ShoutoutElementKind};
pub use error::{DiagnosticError, Error, Result, build_caret_annotation};
