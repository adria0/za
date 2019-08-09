mod format;
mod error;
mod ethereum;
mod prover;
mod ram;

pub use error::{Error,Result};
pub use prover::{generate_verified_proof,setup,bellman_verbose};
pub use ram::{prove_ram,setup_ram, verify_ram, VerifierType};
pub use format::flatten_json;