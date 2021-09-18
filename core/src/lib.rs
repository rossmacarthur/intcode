pub mod assemble;
mod error;
pub mod fmt;
pub mod run;
mod span;

pub use crate::error::{Error, ErrorSet, Warning};
pub use assemble::Intcode;
