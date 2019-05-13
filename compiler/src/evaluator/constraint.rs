use super::algebra::QEQ;

pub trait Constraints {
    fn len(&self) -> usize;
    fn get(&self, i : usize) -> QEQ;
    fn push(&mut self, qeq : QEQ) -> usize;
}