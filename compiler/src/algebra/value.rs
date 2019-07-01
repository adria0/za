use std::fmt;

use circom2_parser::ast;
use num_bigint::{BigInt, BigUint};
use num_traits::identities::Zero;

use super::error::{Error, Result};
use super::types::*;

#[derive(Clone, Serialize, Deserialize)]
pub enum Value {
    FieldScalar(FS),
    LinearCombination(LC),
    QuadraticEquation(QEQ),
}

impl Value {
    pub fn from_signal(signal: SignalId) -> Self {
        Value::LinearCombination(LC::from_signal(signal, FS::one()))
    }
    pub fn into_qeq(self) -> QEQ {
        use Value::*;
        match self {
            FieldScalar(a) => QEQ::from(&a),
            LinearCombination(a) => QEQ::from(&a),
            QuadraticEquation(a) => a,
        }
    }
    pub fn try_to_signal(&self) -> Option<SignalId> {
        if let Value::LinearCombination(lc) = self {
            if lc.0.len() == 1 && lc.0[0].1.is_one() {
                return Some(lc.0[0].0);
            }
        }
        None
    }
    pub fn try_into_fs(self) -> Option<FS> {
        if let Value::FieldScalar(fs) = self {
            Some(fs)
        } else {
            None
        }
    }
}

impl Default for Value {
    fn default() -> Self {
        Value::FieldScalar(FS::from(BigUint::zero()))
    }
}

impl From<&BigInt> for Value {
    fn from(n: &BigInt) -> Self {
        Value::FieldScalar(FS::from(n))
    }
}

impl From<&BigUint> for Value {
    fn from(n: &BigUint) -> Self {
        Value::FieldScalar(FS::from(n))
    }
}

impl From<u64> for Value {
    fn from(n: u64) -> Self {
        Value::FieldScalar(FS::from(n))
    }
}

impl From<FS> for Value {
    fn from(fs: FS) -> Self {
        Value::FieldScalar(fs)
    }
}

impl From<&FS> for Value {
    fn from(fs: &FS) -> Self {
        Value::FieldScalar(fs.clone())
    }
}

impl From<LC> for Value {
    fn from(lc: LC) -> Self {
        Value::LinearCombination(lc)
    }
}

impl From<QEQ> for Value {
    fn from(qeq: QEQ) -> Self {
        Value::QuadraticEquation(qeq)
    }
}

impl fmt::Debug for Value {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> std::result::Result<(), fmt::Error> {
        use Value::*;
        match &self {
            FieldScalar(a) => write!(fmt, "{:?}", a)?,
            LinearCombination(a) => write!(fmt, "{:?}", a)?,
            QuadraticEquation(a) => write!(fmt, "{:?}", a)?,
        }
        Ok(())
    }
}

pub fn eval_infix(lhv: &Value, op: ast::Opcode, rhv: &Value) -> Result<Value> {
    use ast::Opcode::*;
    use Value::*;
    match (op, lhv, rhv) {
        // add
        (Add, FieldScalar(lhv), FieldScalar(rhv)) => Ok(FieldScalar(lhv + rhv)),
        (Add, LinearCombination(lhv), LinearCombination(rhv)) => Ok(LinearCombination(lhv + rhv)),

        (Add, FieldScalar(lhv), LinearCombination(rhv)) => Ok(LinearCombination(rhv + lhv)),
        (Add, LinearCombination(lhv), FieldScalar(rhv)) => Ok(LinearCombination(lhv + rhv)),

        (Add, FieldScalar(lhv), QuadraticEquation(rhv)) => Ok(QuadraticEquation(rhv + lhv)),
        (Add, QuadraticEquation(lhv), FieldScalar(rhv)) => Ok(QuadraticEquation(lhv + rhv)),

        (Add, LinearCombination(lhv), QuadraticEquation(rhv)) => Ok(QuadraticEquation(rhv + lhv)),
        (Add, QuadraticEquation(lhv), LinearCombination(rhv)) => Ok(QuadraticEquation(lhv + rhv)),

        // sub
        (Sub, FieldScalar(lhv), FieldScalar(rhv)) => Ok(FieldScalar(lhv + &-rhv)),
        (Sub, LinearCombination(lhv), LinearCombination(rhv)) => Ok(LinearCombination(lhv + &-rhv)),

        (Sub, FieldScalar(lhv), LinearCombination(rhv)) => Ok(LinearCombination(&-rhv + lhv)),
        (Sub, LinearCombination(lhv), FieldScalar(rhv)) => Ok(LinearCombination(lhv + &-rhv)),

        (Sub, FieldScalar(lhv), QuadraticEquation(rhv)) => Ok(QuadraticEquation(&-rhv + lhv)),
        (Sub, QuadraticEquation(lhv), FieldScalar(rhv)) => Ok(QuadraticEquation(lhv + &-rhv)),

        (Sub, LinearCombination(lhv), QuadraticEquation(rhv)) => Ok(QuadraticEquation(&-rhv + lhv)),
        (Sub, QuadraticEquation(lhv), LinearCombination(rhv)) => Ok(QuadraticEquation(lhv + &-rhv)),

        // mul
        (Mul, FieldScalar(lhv), FieldScalar(rhv)) => Ok(FieldScalar(lhv * rhv)),
        (Mul, LinearCombination(lhv), LinearCombination(rhv)) => Ok(QuadraticEquation(lhv * rhv)),

        (Mul, LinearCombination(lhv), FieldScalar(rhv)) => Ok(LinearCombination(lhv * rhv)),
        (Mul, FieldScalar(lhv), LinearCombination(rhv)) => Ok(LinearCombination(rhv * lhv)),

        (Mul, QuadraticEquation(lhv), FieldScalar(rhv)) => Ok(QuadraticEquation(lhv * rhv)),
        (Mul, FieldScalar(lhv), QuadraticEquation(rhv)) => Ok(QuadraticEquation(rhv * lhv)),

        // div
        (Div, FieldScalar(lhv), FieldScalar(rhv)) => Ok(FieldScalar((lhv / rhv)?)),

        // intdiv
        (IntDiv, FieldScalar(lhv), FieldScalar(rhv)) => Ok(FieldScalar(lhv.intdiv(rhv))),

        // mod
        (Mod, FieldScalar(lhv), FieldScalar(rhv)) => Ok(FieldScalar((lhv % rhv)?)),

        // <<
        (ShiftL, FieldScalar(lhv), FieldScalar(rhv)) => Ok(FieldScalar((lhv << rhv)?)),

        // >>
        (ShiftR, FieldScalar(lhv), FieldScalar(rhv)) => Ok(FieldScalar((lhv >> rhv)?)),

        // and
        (BitAnd, FieldScalar(lhv), FieldScalar(rhv)) => Ok(FieldScalar(lhv & rhv)),

        // or
        (BitOr, FieldScalar(lhv), FieldScalar(rhv)) => Ok(FieldScalar(lhv | rhv)),

        // xor
        (BitXor, FieldScalar(lhv), FieldScalar(rhv)) => Ok(FieldScalar(lhv ^ rhv)),

        // powmod
        (Pow, FieldScalar(lhv), FieldScalar(rhv)) => Ok(FieldScalar(lhv.pow(rhv))),

        _ => Err(Error::InvalidOperation(format!(
            "Cannot apply operator {:?} on {:?} over {:?}",
            op, lhv, rhv
        ))),
    }
}

pub fn eval_prefix(op: ast::Opcode, rhv: &Value) -> Result<Value> {
    use ast::Opcode::Sub;
    use Value::*;
    match (op, rhv) {
        // negate
        (Sub, FieldScalar(rhv)) => Ok(FieldScalar(-rhv)),
        (Sub, LinearCombination(rhv)) => Ok(LinearCombination(-rhv)),
        (Sub, QuadraticEquation(rhv)) => Ok(QuadraticEquation(-rhv)),

        _ => Err(Error::InvalidOperation(format!(
            "Cannot apply operator {:?} on {:?}",
            op, rhv
        ))),
    }
}
