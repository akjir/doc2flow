//! Document and CLI argument parsing modules.

pub mod arguments;
pub mod markdown;
pub mod parser;

pub use arguments::{Args, help_message, parse_args};
pub use markdown::parse_d2f_markdown;
pub use parser::parse;
