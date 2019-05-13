use crate::evaluator::Signals;

pub trait StorageFactory<S:Signals> {
    fn new_signals(&self) -> S;
}
