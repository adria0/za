use crate::algebra::QEQ;

pub struct Constraints(Vec<(QEQ, Option<String>)>);

impl Default for Constraints {
    fn default() -> Self {
        Constraints(Vec::new())
    }
}

impl Constraints  {
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
    pub fn len(&self) -> usize {
        self.0.len()
    }
    pub fn get(&self, i: usize) -> QEQ {
        self.0[i].0.clone()
    }
    pub fn get_debug(&self, i: usize) -> Option<String> {
        self.0[i].1.clone()
    }
    pub fn push(&mut self, qeq: QEQ, debug: Option<String>) -> usize {
        self.0.push((qeq, debug));
        self.0.len() - 1
    }
}
