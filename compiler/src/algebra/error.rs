#[derive(Debug,Clone)]
pub enum Error {
    InvalidOperation(String),
}

pub type Result<T> = std::result::Result<T, Error>;
