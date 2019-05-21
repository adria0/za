extern crate serde;
extern crate serde_derive;

#[macro_use]
extern crate lalrpop_util;

lalrpop_mod!(pub lang); // synthesized by LALRPOP

pub mod ast;
pub mod display;
mod error;
mod parse;

pub use self::error::{Error, Result};
pub use self::parse::parse;
