//! Legacy Doc2Flow architecture modules.
#![allow(clippy::all)]

pub mod core;
pub mod features;
#[allow(clippy::module_inception)]
pub mod legacy;
pub mod utils;

pub use core::*;
pub use legacy::run;
pub use utils::*;
