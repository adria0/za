#[derive(Debug)]
pub enum Error {
    NotFound(String),
    Inner(String),
}

impl PartialEq for Error {
    fn eq(&self, other: &Error) -> bool {
        format!("{:?}", self) == format!("{:?}", other)
    }
}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Error::Inner(err.to_string())
    }
}

impl From<serde_cbor::error::Error> for Error {
    fn from(err: serde_cbor::error::Error) -> Self {
        Error::Inner(err.to_string())
    }
}

pub type Result<T> = std::result::Result<T, Error>;
