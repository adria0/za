use crate::types::{Signal, Signals};

use super::algebra;
use super::error::{Error, Result};
use super::algebra::{SignalId, FS};

#[derive(Debug, Clone)]
pub enum List {
    Algebra(algebra::Value),
    List(Vec<List>),
}

impl List {
    pub fn new(sizes: &[usize]) -> Self {
        if sizes.is_empty() {
            List::Algebra(algebra::Value::default())
        } else {
            let mut l = Vec::new();
            for _ in 0..sizes[0] {
                l.push(List::new(&sizes[1..]));
            }
            List::List(l)
        }
    }

    pub fn get(&self, indexes: &[usize]) -> Result<&List> {
        if !indexes.is_empty() {
            match self {
                List::Algebra(_) => Err(Error::InvalidSelector(format!(
                    "index at [{}] contains a value",
                    indexes[0]
                ))),
                List::List(v) => {
                    if indexes[0] >= v.len() {
                        Err(Error::InvalidSelector(format!(
                            "index at [{}] too large",
                            indexes[0]
                        )))
                    } else {
                        v[indexes[0]].get(&indexes[1..])
                    }
                }
            }
        } else {
            Ok(self)
        }
    }

    pub fn set(&mut self, value: &algebra::Value, indexes: &[usize]) -> Result<()> {
        match self {
            List::Algebra(_) => Err(Error::InvalidSelector(format!(
                "index at [{}] contains a value",
                indexes[0]
            ))),

            List::List(v) => {
                if indexes.is_empty() || indexes[0] >= v.len() {
                    Err(Error::InvalidSelector(format!("invalid index for {:?}", v)))
                } else if indexes.len() == 1 {
                    v[indexes[0]] = List::Algebra(value.clone());
                    Ok(())
                } else {
                    v[indexes[0]].set(value, &indexes[1..])
                }
            }
        }
    }
}

#[derive(Debug, Clone)]
pub enum ReturnValue {
    Bool(bool),
    Algebra(algebra::Value),
    List(List),
}

impl ReturnValue {
    pub fn from_signal_name(full_name: &str, signals: &Signals) -> Result<ReturnValue> {
        match signals.get_by_name(full_name) {
            Some(rc) => match &*rc {
                Signal {
                    value: Some(algebra::Value::FieldScalar(fs)),
                    ..
                } => Ok(ReturnValue::Algebra(algebra::Value::from(fs))),
                Signal {
                    value: Some(_), id, ..
                }
                | Signal {
                    value: None, id, ..
                } => Ok(ReturnValue::Algebra(algebra::Value::from_signal(*id))),
            },
            None => Err(Error::NotFound(format!("Signal {:?}", full_name))),
        }
    }
    pub fn from_signal_id(id: SignalId) -> Result<ReturnValue> {
        Ok(ReturnValue::Algebra(algebra::Value::from_signal(id)))
    }
    pub fn try_into_algebra(self) -> Result<algebra::Value> {
        match self {
            ReturnValue::Algebra(a) => Ok(a),
            _ => Err(Error::InvalidType(format!(
                "Cannot convert to algebraic value {:?}",
                self
            ))),
        }
    }
    pub fn try_to_signal(&self) -> Result<SignalId> {
        if let ReturnValue::Algebra(a) = self {
            if let Some(signal) = a.try_to_signal() {
                return Ok(signal);
            }
        }
        Err(Error::InvalidType(format!(
            "Cannot convert to signal {:?}",
            self
        )))
    }
    pub fn try_into_bool(self) -> Result<bool> {
        match self {
            ReturnValue::Bool(b) => Ok(b),
            _ => Err(Error::InvalidType(format!(
                "Cannot convert to boolean value {:?}",
                self
            ))),
        }
    }
    pub fn try_into_fs(self) -> Result<FS> {
        match self {
            ReturnValue::Algebra(algebra::Value::FieldScalar(fs)) => Ok(fs),
            _ => Err(Error::InvalidType(format!(
                "Cannot convert to scalar value {:?}",
                self
            ))),
        }
    }
    pub fn try_into_u64(self) -> Result<u64> {
        let fs = self.try_into_fs()?;
        if let Some(n) = fs.try_to_u64() {
            Ok(n)
        } else {
            Err(Error::CannotConvertToU64(fs))
        }
    }
}
