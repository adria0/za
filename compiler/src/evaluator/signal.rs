#![allow(dead_code)]
use std::collections::HashMap;
use std::rc::Rc;
use std::fmt::{Debug, Formatter};
use circom2_parser::ast::SignalType;

use super::algebra;
use super::algebra::SignalId;

#[derive(Clone)]
pub struct SignalName(Rc<String>); // see E0210

impl std::borrow::Borrow<str> for SignalName {
   fn borrow(&self) -> &str {
       &self.0
   }
}
impl std::cmp::PartialEq for SignalName {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0
    }    
}
impl std::cmp::Eq for SignalName {}

impl std::hash::Hash for SignalName {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.0.hash(state);
    }
}

impl Debug for SignalName {
    fn fmt(&self, fmt: &mut Formatter) -> std::result::Result<(), std::fmt::Error> {
        write!(fmt, "{}",self.0)
    }
}

impl std::string::ToString for SignalName {
    fn to_string(&self) -> String {
        self.0.to_string()
    }
}


#[derive(Clone)]
pub struct Signal {
    pub id : SignalId,
    pub xtype : SignalType,
    pub full_name : SignalName,
    pub value : Option<algebra::Value>,         
}

pub struct Signals {   
    names : HashMap<SignalName,SignalId>,
    ids   : Vec<Signal>,
}

impl Default for Signals {
    fn default() -> Self {
        let ids = Vec::new();
        let names = HashMap::new();
        let mut signals = Self { names, ids };
        signals.insert("one".to_string(), SignalType::PublicInput,None);
        signals
    }
}

impl Signals {
    pub fn len(&self) -> usize {
        self.ids.len()
    }
    pub fn get_by_id(&self, id : SignalId) -> Option<&Signal> {
        if (id as usize) < self.ids.len() {
            Some(&self.ids[id as usize])
        } else {
            None
        }
    }
    pub fn get_by_id_mut(&mut self, id : SignalId) -> Option<&mut Signal> {
        if (id as usize) < self.ids.len() {
            Some(&mut self.ids[id as usize])
        } else {
            None
        }
    }

    pub fn get_by_name(&self, full_name : &str) -> Option<&Signal> {
        self.names.get(full_name)
            .map(|id| &self.ids[*id as usize])
    }
    pub fn get_by_name_mut(&mut self, full_name : &str) -> Option<&mut Signal> {
        let id = self.names.get(full_name).cloned();
        id.map(move |id| &mut (self.ids[id as usize]))
    }
    pub fn insert(&mut self, full_name: String, xtype: SignalType, value : Option<algebra::Value>) -> SignalId {
        let id = self.ids.len() as SignalId;
        let full_name_rc = SignalName(Rc::new(full_name));

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
    pub fn to_string(&self, id : SignalId) -> String {
        let s = &self.ids[id as usize];
        format!("{:?}:{:?}:{:?}",s.full_name,s.xtype,s.value)
    }
}

impl Debug for Signals {
    fn fmt(&self, fmt: &mut Formatter) -> std::result::Result<(), std::fmt::Error> {
        writeln!(fmt, "signals --------------------------------------------")?;
        for (_,id) in &self.names {
            writeln!(fmt, "{}",self.to_string(*id))?;
        }
        Ok(())
    }
}
