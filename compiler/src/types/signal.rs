use std::rc::Rc;

use za_parser::ast::SignalType;
use std::collections::HashMap;
use std::fmt::{Debug, Formatter};

use crate::algebra;
use crate::algebra::SignalId;

#[derive(Clone)]
pub struct SignalName(pub Rc<String>); // see E0210

impl SignalName {
    pub fn new(s: String) -> Self {
        SignalName(Rc::new(s))
    }
}

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
        write!(fmt, "{}", self.0)
    }
}

impl std::string::ToString for SignalName {
    fn to_string(&self) -> String {
        self.0.to_string()
    }
}

#[derive(Clone, Debug)]
pub struct Signal {
    pub id: SignalId,
    pub xtype: SignalType,
    pub full_name: SignalName,
    pub value: Option<algebra::Value>,
}

impl Signal {
    pub fn is_main_public_input(&self) -> bool {
        let component_len = self.full_name.0.chars().filter(|ch| *ch == '.').count();
        component_len == 1
            && (self.xtype == SignalType::Output || self.xtype == SignalType::PublicInput)
    }
    pub fn is_main_input(&self) -> bool {
        let component_len = self.full_name.0.chars().filter(|ch| *ch == '.').count();
        component_len == 1
            && (self.xtype == SignalType::Output
                || self.xtype == SignalType::PublicInput
                || self.xtype == SignalType::PrivateInput)
    }
}

pub struct Signals {
    names: HashMap<SignalName, SignalId>,
    ids: Vec<Rc<Signal>>,
}

impl Default for Signals {
    fn default() -> Self {
        let ids = Vec::new();
        let names = HashMap::new();
        let mut signals = Self { names, ids };
        // FIX
        signals
            .insert("one".to_string(), SignalType::PublicInput, None);

        signals
    }
}

impl Signals  {
    pub fn is_empty(&self) -> bool {
        self.ids.is_empty()
    }

    pub fn len(&self) -> usize {
        self.ids.len()
    }

    pub fn get_by_id(&self, id: SignalId) -> Option<Rc<Signal>> {
        if (id as usize) < self.ids.len() {
            Some(self.ids[id as usize].clone())
        } else {
            None
        }
    }

    pub fn update(&mut self, id: SignalId, value: algebra::Value) {
        let signal = &mut self.ids[id as usize];
        if let Some(signal) = Rc::get_mut(signal) {
            signal.value = Some(value);
        } else {
            (*Rc::make_mut(signal)).value = Some(value);
        }
    }

    pub fn get_by_name(&self, full_name: &str) -> Option<Rc<Signal>> {
        self
            .names
            .get(full_name)
            .map(|id| self.ids[*id as usize].clone())
    }

    pub fn insert(
        &mut self,
        full_name: String,
        xtype: SignalType,
        value: Option<algebra::Value>,
    ) -> SignalId {
        let id = self.ids.len() as SignalId;
        let full_name_rc = SignalName::new(full_name);

        let signal = Signal {
            id,
            xtype,
            full_name: full_name_rc.clone(),
            value
        };

        self.ids.push(Rc::new(signal));
        self.names.insert(full_name_rc, id);

        id
    }

    pub fn main_public_input_names(&self) -> Vec<String> {

        let mut inputs = Vec::new();
        for i in 1..self.len() {
            let signal = self.get_by_id(i).unwrap();
            if signal.is_main_public_input() {
                inputs.push(signal.full_name.to_string());
            }
        }
        inputs
    }

    pub fn main_input_ids(&self) -> Vec<SignalId> {
        let mut inputs = Vec::new();
        for i in 1..self.len() {
            let signal = self.get_by_id(i).unwrap();
            if signal.is_main_input() {
                inputs.push(i);
            }
        }
        inputs
    }

    pub fn to_string(&self, id: SignalId) -> String {
        let s = &self.ids[id as usize];
        format!("{:?}:{:?}:{:?}", s.full_name, s.xtype, s.value)
    }

    pub fn format(&self, a: &algebra::Value) -> String {
        let sname = |id| self.get_by_id(id).map_or("unwnown".to_string(), |s| s.full_name.to_string());
        match a {
            algebra::Value::FieldScalar(fe) => fe.to_string(),
            algebra::Value::LinearCombination(lc) => lc.format(sname),
            algebra::Value::QuadraticEquation(qeq) => qeq.format(sname),
        }
    }
}

impl Debug for Signals {
    fn fmt(&self, fmt: &mut Formatter) -> std::result::Result<(), std::fmt::Error> {
        writeln!(fmt, "signals --------------------------------------------")?;
        for id in self.names.values() {
            writeln!(fmt, "{}", self.to_string(*id))?;
        }
        Ok(())
    }
}

