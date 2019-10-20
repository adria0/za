mod error;
mod ethereum;
mod format;
mod prover;

pub mod helper;
pub use error::{Error, Result};
pub use format::flatten_json;

pub use prover::{bellman_verbose, generate_verified_proof, setup};