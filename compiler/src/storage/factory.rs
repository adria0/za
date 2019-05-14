use super::types::{Signals,Constraints};
use super::error::Result;

pub trait StorageFactory<S:Signals,C:Constraints> {
    fn new_signals(&mut self) -> Result<S>;
    fn new_constraints(&mut self) -> Result<C>;
}
