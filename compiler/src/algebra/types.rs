use num_bigint::BigUint;

pub type SignalId = usize; 

#[derive(Clone)]
pub struct FS(pub BigUint);

#[derive(Clone)]
pub struct LC(pub Vec<(SignalId, FS)>);

#[derive(Clone)]
pub struct QEQ {
    pub a: LC,
    pub b: LC,
    pub c: LC,
}

