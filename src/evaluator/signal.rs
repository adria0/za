#![allow(dead_code)]
use std::collections::HashMap;

use std::fmt::{Debug, Formatter};
use circom2_parser::ast::SignalType;

use super::algebra;

#[derive(Clone)]
pub struct Signal {
    pub xtype : SignalType,
    pub full_name : String,
    pub equivalence : Option<String>,
    pub value : Option<algebra::Value>,         
}

impl Signal {
    pub fn new(xtype: SignalType, full_name : String) -> Self {
        Self { xtype, full_name, value : None, equivalence : None }
    }
}

impl<'a> Debug for Signal {
    fn fmt(&self, fmt: &mut Formatter) -> std::result::Result<(), std::fmt::Error> {
        if let Some(eq) = &self.equivalence {
            write!(fmt, "{}:{:?}:{:?}:Some({})",self.full_name,self.xtype,self.value,eq)
        } else {
            write!(fmt, "{}:{:?}:{:?}:None",self.full_name,self.xtype,self.value)
        }
    }
}

pub struct Signals(HashMap<String,Signal>);

impl Signals {
    pub fn new() -> Self {
        Self(HashMap::new())
    }
    pub fn len(&self) -> usize {
        self.0.len()
    }
    pub fn get(&self, full_name : &str) -> Option<&Signal> {
        self.0.get(full_name)
    }
    pub fn get_mut(&mut self, full_name : &str) -> Option<&mut Signal> {
        self.0.get_mut(full_name)
    }
    pub fn insert(&mut self, signal : Signal) {
        self.0.insert(signal.full_name.clone(),signal);
    }
    pub fn equivalent<'a> (&'a self, full_name : &'a str) -> &'a str {
        let mut full_name = full_name;
        while let Some(signal) = self.0.get(full_name) {
            if let Some(equivalence) = &signal.equivalence {
                full_name = equivalence;
            } else {
                break;
            }
        }
        full_name
    }
}

impl Debug for Signals {
    fn fmt(&self, fmt: &mut Formatter) -> std::result::Result<(), std::fmt::Error> {
        writeln!(fmt, "signals --------------------------------------------")?;
        for (k,v) in &self.0 {
            writeln!(fmt, "  {}: {:?}",k,v)?;
        }
        Ok(())
    }
}
