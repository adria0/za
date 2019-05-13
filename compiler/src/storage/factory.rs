use crate::evaluator::{Signals,Constraints};

pub trait StorageFactory<S:Signals,C:Constraints> {
    fn new_signals(&self) -> S;
    fn new_constraints(&self) -> C;
}
