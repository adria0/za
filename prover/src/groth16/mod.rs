mod format;
mod error;
mod ethereum;
mod prover;

pub use format::flatten_json;
pub use error::{Error,Result};
pub use prover::{generate_verified_proof,setup};
