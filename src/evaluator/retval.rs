use num_traits::cast::ToPrimitive;

use super::algebra;
use super::algebra::{FS,SignalId};
use super::error::*;
use super::signal::*;
use super::types::*;

#[derive(Debug, Clone)]
pub enum ReturnValue {
    Bool(bool),
    Algebra(algebra::Value),
    List(List),
}

impl ReturnValue {
    pub fn from_signal_name(full_name: &str, signals : &Signals ) -> Result<ReturnValue> {
        match signals.get_by_name(full_name) {
            Some(Signal{value:Some(algebra::Value::FieldScalar(fs)), ..})
                => Ok(ReturnValue::Algebra(algebra::Value::from(fs))),
            Some(Signal{value:Some(_),id,..}) | Some(Signal{value:None,id,..})
                => Ok(ReturnValue::Algebra(algebra::Value::from_signal(*id))),
            None
                => Err(Error::NotFound(format!("Signal {:?}",full_name)))
        }
    }
    pub fn from_signal_id(id: SignalId) -> Result<ReturnValue> {
        Ok(ReturnValue::Algebra(algebra::Value::from_signal(id)))
    }
    pub fn into_algebra(self) -> Result<algebra::Value> {
        match self {
            ReturnValue::Algebra(a) => Ok(a),
            _ => Err(Error::InvalidType(format!("Cannot convert to algebraic value {:?}",self)))
        }
    }
    pub fn to_signal(&self) -> Result<SignalId> {
        if let ReturnValue::Algebra(a) = self {
            if let Some(signal) = a.try_to_signal() {
                return Ok(signal)
            }
        }
        Err(Error::InvalidType(format!("Cannot convert to signal {:?}",self)))
    }
    pub fn into_bool(self) -> Result<bool> {
        match self {
            ReturnValue::Bool(b) => Ok(b),
            _ => Err(Error::InvalidType(format!("Cannot convert to boolean value {:?}",self)))
        }
    }
    pub fn into_fs(self) -> Result<FS> {
        match self {
            ReturnValue::Algebra(algebra::Value::FieldScalar(fs)) => Ok(fs),
            _ => Err(Error::InvalidType(format!("Cannot convert to scalar value {:?}",self)))
        }
    }
    pub fn into_u64(self) -> Result<u64> {
        let fs = self.into_fs()?;
        if let Some(n) = fs.0.to_u64() {
            Ok(n)
        } else {
            Err(Error::CannotConvertToU64(fs))
        }
    }
}