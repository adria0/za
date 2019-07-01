use crate::storage;
use crate::evaluator;

#[derive(Debug)]
pub enum Error {
    Storage(storage::Error),
    Evaluator(evaluator::Error),
}

impl From<storage::Error> for Error {
    fn from(err: storage::Error) -> Self {
        Error::Storage(err)
    }
}

impl From<evaluator::Error> for Error {
    fn from(err: evaluator::Error) -> Self {
        Error::Evaluator(err)
    }
}

pub type Result<T> = std::result::Result<T, Error>;
