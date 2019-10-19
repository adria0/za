mod error;
mod ethereum;
mod format;
mod prover;
mod ram;

pub use error::{Error, Result};
pub use format::flatten_json;
pub use prover::{bellman_verbose, generate_verified_proof, setup};
pub use ram::{prove_ram, setup_ram, verify_ram, VerifierType};
