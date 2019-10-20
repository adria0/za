use crate::algebra;

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
    Io(String, String),
    Unexpected(String),
}

pub type Result<T> = std::result::Result<T, Error>;
