use crate::storage;

#[derive(Debug)]
pub enum Error {
    Storage(storage::Error),
}

impl From<storage::Error> for Error {
    fn from(err: storage::Error) -> Self {
        Error::Storage(err)
    }
}

pub type Result<T> = std::result::Result<T, Error>;
