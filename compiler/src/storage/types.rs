#![allow(dead_code)]
use std::rc::Rc;
use std::fmt::{Debug, Formatter};
use circom2_parser::ast::SignalType;
use crate::algebra::QEQ;

use super::error::Result;

use crate::algebra;
use crate::algebra::SignalId;

#[derive(Clone)]
pub struct SignalName(Rc<String>); // see E0210

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

pub trait Signals {
    fn is_empty(&self) -> Result<bool>;
    fn len(&self) -> Result<usize>;
    fn insert(&mut self, full_name: String, xtype: SignalType, value : Option<algebra::Value>) -> Result<SignalId>;
    fn update(&mut self, id : SignalId, value : algebra::Value) -> Result<()>;
    fn get_by_id(&self, id : SignalId) -> Result<Option<Rc<Signal>>>;
    fn get_by_name(&self, full_name : &str) -> Result<Option<Rc<Signal>>>;
    fn to_string(&self, id : SignalId) -> Result<String>;
}

pub trait Constraints {
    fn is_empty(&self) -> Result<bool>;
    fn len(&self) -> Result<usize>;
    fn get(&self, i : usize) -> Result<QEQ>;
    fn push(&mut self, qeq : QEQ) -> Result<usize>;
}
