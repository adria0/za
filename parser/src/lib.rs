extern crate serde;

#[macro_use]
extern crate serde_derive;

#[macro_use]
extern crate lalrpop_util;

lalrpop_mod!(pub lang); // synthesized by LALRPOP

mod parse;
mod error;
pub mod ast;
pub mod display;

pub use self::parse::parse;
pub use self::error::{Error,Result};
