use std::collections::HashMap;
use std::fmt::{Debug, Formatter};
use circom2_parser::ast::SignalType;
use crate::algebra;
use crate::algebra::{QEQ,SignalId};
use crate::evaluator::{SignalName,Signal,Signals};
use crate::evaluator::Constraints;
use super::StorageFactory;

pub struct Ram {}
impl Default for Ram {
    fn default() -> Self {
        Ram{}
    }
}

impl StorageFactory<RamSignals,RamConstraints> for Ram {
    fn new_signals(&self) -> RamSignals {
        RamSignals::default()
    }
    fn new_constraints(&self) -> RamConstraints {
        RamConstraints::default()
    }
}

pub struct RamSignals {   
    names : HashMap<SignalName,SignalId>,
    ids   : Vec<Signal>,
}

impl Default for RamSignals {
    fn default() -> Self {
        let ids = Vec::new();
        let names = HashMap::new();
        let mut signals = Self { names, ids };
        signals.insert("one".to_string(), SignalType::PublicInput,None);
        signals
    }
}

impl Signals for RamSignals {
    fn len(&self) -> usize {
        self.ids.len()
    }
    fn get_by_id(&self, id : SignalId) -> Option<&Signal> {
        if (id as usize) < self.ids.len() {
            Some(&self.ids[id as usize])
        } else {
            None
        }
    }
    fn get_by_id_mut(&mut self, id : SignalId) -> Option<&mut Signal> {
        if (id as usize) < self.ids.len() {
            Some(&mut self.ids[id as usize])
        } else {
            None
        }
    }

    fn get_by_name(&self, full_name : &str) -> Option<&Signal> {
        self.names.get(full_name)
            .map(|id| &self.ids[*id as usize])
    }
    fn get_by_name_mut(&mut self, full_name : &str) -> Option<&mut Signal> {
        let id = self.names.get(full_name).cloned();
        id.map(move |id| &mut (self.ids[id as usize]))
    }
    fn insert(&mut self, full_name: String, xtype: SignalType, value : Option<algebra::Value>) -> SignalId {
        let id = self.ids.len() as SignalId;
        let full_name_rc = SignalName::new(full_name);

        let signal = Signal {
            id,
            xtype,
            full_name : full_name_rc.clone(),
            value : value,
        };

        self.ids.push(signal);
        self.names.insert(full_name_rc, id);

        id
    }
    fn to_string(&self, id : SignalId) -> String {
        let s = &self.ids[id as usize];
        format!("{:?}:{:?}:{:?}",s.full_name,s.xtype,s.value)
    }
}

impl Debug for RamSignals {
    fn fmt(&self, fmt: &mut Formatter) -> std::result::Result<(), std::fmt::Error> {
        writeln!(fmt, "signals --------------------------------------------")?;
        for (_,id) in &self.names {
            writeln!(fmt, "{}",self.to_string(*id))?;
        }
        Ok(())
    }
}

pub struct RamConstraints(Vec<QEQ>);
impl Default for RamConstraints {
    fn default() -> Self {
        RamConstraints(Vec::new())
    } 
}
impl Constraints for RamConstraints {
    fn len(&self) -> usize {
        self.0.len()
    }
    fn get(&self, i : usize) -> QEQ {
        self.0[i].clone()
    }
    fn push(&mut self, qeq : QEQ) -> usize {
        self.0.push(qeq);
        self.0.len() - 1
    }   
}