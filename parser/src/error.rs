use super::ast::Meta;

#[derive(Debug)]
pub enum Error {
    ParseError(String, Meta),
}

pub type Result<T> = std::result::Result<T, Error>;
