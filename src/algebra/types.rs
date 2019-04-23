use num_bigint::BigUint;

#[derive(Clone)]
pub struct FS(pub BigUint);

#[derive(Clone)]
pub struct LC(pub Vec<(String, FS)>);

#[derive(Clone)]
pub struct QEQ {
    pub a: LC,
    pub b: LC,
    pub c: LC,
}

