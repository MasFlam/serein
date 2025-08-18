extern crate self as serein;

pub mod error;
pub mod options;
pub mod slash;

pub use serein_macros as macros;

pub use error::{Error, Result};
