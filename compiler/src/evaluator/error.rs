use crate::algebra;
use crate::storage;

#[derive(Debug)]
pub enum Error {
    NotFound(String),
    AlreadyExists(String),
    Parse(String),
    InvalidParameter(String),
    InvalidSelector(String),
    BadFunctionReturn(String),
    InvalidTag(String),
    InvalidType(String),
    NotYetImplemented(String),
    Algebra(algebra::Error),
    CannotGenerateConstrain(String),
    CannotTestConstrain(String),
    CannotCheckConstrain(String),
    CannotConvertToU64(algebra::FS),
    Storage(storage::Error),
    Io(String, String),
}

impl From<storage::Error> for Error {
    fn from(err: storage::Error) -> Self {
        Error::Storage(err)
    }
}

pub type Result<T> = std::result::Result<T, Error>;
