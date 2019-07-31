use num_bigint::BigUint;

pub type SignalId = usize;
pub const SIGNAL_ONE: SignalId = 0;

#[derive(Clone, Serialize, Deserialize)]
pub struct FS(pub(super) BigUint);

#[derive(Clone, Serialize, Deserialize)]
pub struct LC(pub Vec<(SignalId, FS)>);

#[derive(Clone, Serialize, Deserialize)]
pub struct QEQ {
    pub a: LC,
    pub b: LC,
    pub c: LC,
}
