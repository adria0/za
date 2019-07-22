use std::io;

use crate::storage;
use crate::evaluator;

#[derive(Debug)]
pub enum Error {
    Storage(storage::Error),
    Evaluator(evaluator::Error),
    Io(io::Error),
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
impl From<io::Error> for Error {
    fn from(err : io::Error) -> Self {
        Error::Io(err)
    }
}


pub type Result<T> = std::result::Result<T, Error>;
