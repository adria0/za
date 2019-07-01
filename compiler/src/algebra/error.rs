#[derive(Debug, Clone)]
pub enum Error {
    InvalidOperation(String),
    InvalidFormat(String),
}

pub type Result<T> = std::result::Result<T, Error>;
