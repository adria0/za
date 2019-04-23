use num_bigint::{BigInt,BigUint};

pub const SIGNAL_ONE: &str = "one";

lazy_static! {
    pub static ref FIELD_UINT: BigUint = BigUint::parse_bytes(
        b"21888242871839275222246405745257275088548364400416034343698204186575808495617",
        10
    )
    .unwrap();
    pub static ref FIELD_UINT_NEG: BigUint = BigUint::parse_bytes(
        b"10944121435919637611123202872628637544274182200208017171849102093287904247808",
        10
    )
    .unwrap();
    pub static ref FIELD_INT: BigInt = BigInt::parse_bytes(
        b"21888242871839275222246405745257275088548364400416034343698204186575808495617",
        10
    )
    .unwrap();
    pub static ref ONE: BigUint = BigUint::parse_bytes(b"1", 10).unwrap();
}
