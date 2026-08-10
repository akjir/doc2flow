//! Document and CLI argument parsing modules.

pub mod arguments;
pub mod markdown;

pub use arguments::{Args, help_message, parse_args};
pub use markdown::parse_d2f_markdown;

