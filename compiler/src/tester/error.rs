use std::io;
use crate::algebra;
use crate::evaluator;

#[derive(Debug)]
pub enum Error {
    Evaluator(evaluator::Error),
    Io(io::Error),
    Algebra(algebra::Error),
}

impl From<evaluator::Error> for Error {
    fn from(err: evaluator::Error) -> Self {
        Error::Evaluator(err)
    }
}
impl From<io::Error> for Error {
    fn from(err: io::Error) -> Self {
        Error::Io(err)
    }
}
impl From<algebra::Error> for Error {
    fn from(err: algebra::Error) -> Self {
        Error::Algebra(err)
    }
}

pub type Result<T> = std::result::Result<T, Error>;
