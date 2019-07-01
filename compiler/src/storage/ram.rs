use circom2_parser::ast::SignalType;
use std::collections::HashMap;
use std::fmt::{Debug, Formatter};
use std::rc::Rc;

use super::types::*;
use crate::algebra;
use crate::algebra::{SignalId, QEQ};

use super::error::Result;
use super::StorageFactory;

pub struct Ram {}
impl Default for Ram {
    fn default() -> Self {
        Ram {}
    }
}

impl StorageFactory<RamSignals, RamConstraints> for Ram {
    fn new_signals(&mut self) -> Result<RamSignals> {
        Ok(RamSignals::default())
    }
    fn new_constraints(&mut self) -> Result<RamConstraints> {
        Ok(RamConstraints::default())
    }
}

pub struct RamSignals {
    names: HashMap<SignalName, SignalId>,
    ids: Vec<Rc<Signal>>,
}

impl RamSignals {
}

impl Default for RamSignals {
    fn default() -> Self {
        let ids = Vec::new();
        let names = HashMap::new();
        let mut signals = Self { names, ids };
        // FIX
        signals
            .insert("one".to_string(), SignalType::PublicInput, None)
            .unwrap();
        signals
    }
}

impl Signals for RamSignals {
    fn is_empty(&self) -> Result<bool> {
        Ok(self.len()? == 0)
    }

    fn len(&self) -> Result<usize> {
        Ok(self.ids.len())
    }

    fn get_by_id(&self, id: SignalId) -> Result<Option<Rc<Signal>>> {
        if (id as usize) < self.ids.len() {
            Ok(Some(self.ids[id as usize].clone()))
        } else {
            Ok(None)
        }
    }

    fn update(&mut self, id: SignalId, value: algebra::Value) -> Result<()> {
        let signal = &mut self.ids[id as usize];
        if let Some(signal) =  Rc::get_mut(signal) {
            signal.value = Some(value);
        } else {
            (*Rc::make_mut(signal)).value = Some(value);
        }
        Ok(())
    }

    fn get_by_name(&self, full_name: &str) -> Result<Option<Rc<Signal>>> {
        Ok(self
            .names
            .get(full_name)
            .map(|id| self.ids[*id as usize].clone()))
    }

    fn insert(
        &mut self,
        full_name: String,
        xtype: SignalType,
        value: Option<algebra::Value>,
    ) -> Result<SignalId> {
        let id = self.ids.len() as SignalId;
        let full_name_rc = SignalName::new(full_name);

        let signal = Signal {
            id,
            xtype,
            full_name: full_name_rc.clone(),
            value,
        };

        self.ids.push(Rc::new(signal));
        self.names.insert(full_name_rc, id);

        Ok(id)
    }
    fn to_string(&self, id: SignalId) -> Result<String> {
        let s = &self.ids[id as usize];
        Ok(format!("{:?}:{:?}:{:?}", s.full_name, s.xtype, s.value))
    }
}

impl Debug for RamSignals {
    fn fmt(&self, fmt: &mut Formatter) -> std::result::Result<(), std::fmt::Error> {
        writeln!(fmt, "signals --------------------------------------------")?;
        for id in self.names.values() {
            writeln!(fmt, "{}", self.to_string(*id).unwrap())?;
        }
        Ok(())
    }
}

pub struct RamConstraints(Vec<(QEQ,Option<String>)>);

impl Default for RamConstraints {
    fn default() -> Self {
        RamConstraints(Vec::new())
    }
}

impl Constraints for RamConstraints {
    fn is_empty(&self) -> Result<bool> {
        Ok(self.len()? == 0)
    }
    fn len(&self) -> Result<usize> {
        Ok(self.0.len())
    }
    fn get(&self, i: usize) -> Result<QEQ> {
        Ok(self.0[i].0.clone())
    }
    fn get_debug(&self, i: usize) -> Option<String> {
        self.0[i].1.clone()
    }
    fn push(&mut self, qeq: QEQ, debug: Option<String>) -> Result<usize> {
        self.0.push((qeq,debug));
        Ok(self.0.len() - 1)
    }
}
