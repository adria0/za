use num_bigint::BigUint;

pub type SignalId = usize;

#[derive(Clone, Serialize, Deserialize)]
pub struct FS(pub BigUint);

#[derive(Clone, Serialize, Deserialize)]
pub struct LC(pub Vec<(SignalId, FS)>);

#[derive(Clone, Serialize, Deserialize)]
pub struct QEQ {
    pub a: LC,
    pub b: LC,
    pub c: LC,
}
