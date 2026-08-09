//! doc2flow library interface.

pub mod core;
pub mod features;
#[allow(special_module_name)]
#[path = "lib/mod.rs"]
pub mod lib;

pub use core::*;
pub use lib::*;
