#[derive(Debug)]
pub enum Error {
    Io(std::io::Error),
    Synthesis(bellman::SynthesisError),
    Storage(circom2_compiler::storage::Error),
    Cbor(serde_cbor::error::Error),
    Algebra(circom2_compiler::algebra::Error),
    Unexpected(String),
    Json(serde_json::error::Error),
}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Error::Io(err)
    }
}

impl From<bellman::SynthesisError> for Error {
    fn from(err: bellman::SynthesisError) -> Self {
        Error::Synthesis(err)
    }
}

impl From<circom2_compiler::storage::Error> for Error {
    fn from(err: circom2_compiler::storage::Error) -> Self {
        Error::Storage(err)
    }
}

impl From<serde_cbor::error::Error> for Error {
    fn from(err: serde_cbor::error::Error) -> Self {
        Error::Cbor(err)
    }
}

impl From<circom2_compiler::algebra::Error> for Error {
    fn from(err: circom2_compiler::algebra::Error) -> Self {
        Error::Algebra(err)
    }
}

impl From<serde_json::error::Error> for Error {
    fn from(err: serde_json::error::Error) -> Self {
        Error::Json(err)
    }
}

pub type Result<T> = std::result::Result<T, Error>;

