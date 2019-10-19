#[derive(Debug)]
pub enum Error {
    Io(std::io::Error),
    Synthesis(bellman::SynthesisError),
    Cbor(serde_cbor::error::Error),
    Algebra(circom2_compiler::algebra::Error),
    Evaluator(circom2_compiler::evaluator::Error),
    BadFormat(String),
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

impl From<circom2_compiler::evaluator::Error> for Error {
    fn from(err: circom2_compiler::evaluator::Error) -> Self {
        Error::Evaluator(err)
    }
}

pub type Result<T> = std::result::Result<T, Error>;
