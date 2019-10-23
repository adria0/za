#[derive(Debug)]
pub enum Error {
    Io(std::io::Error),
    Synthesis(bellman::SynthesisError),
    Cbor(serde_cbor::error::Error),
    Bincode(bincode::Error),
    Algebra(za_compiler::algebra::Error),
    Evaluator(za_compiler::evaluator::Error),
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

impl From<bincode::Error> for Error {
    fn from(err: bincode::Error) -> Self {
        Error::Bincode(err)
    }
}

impl From<za_compiler::algebra::Error> for Error {
    fn from(err: za_compiler::algebra::Error) -> Self {
        Error::Algebra(err)
    }
}

impl From<serde_json::error::Error> for Error {
    fn from(err: serde_json::error::Error) -> Self {
        Error::Json(err)
    }
}

impl From<za_compiler::evaluator::Error> for Error {
    fn from(err: za_compiler::evaluator::Error) -> Self {
        Error::Evaluator(err)
    }
}

pub type Result<T> = std::result::Result<T, Error>;
