#![allow(dead_code)]

use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt::{Debug, Formatter};

use circom2_parser::ast::{Attributes, StatementP};

use super::algebra;
use super::error::*;
use super::retval::*;
use super::types::*;

#[derive(Debug, Clone)]
pub enum ScopeValue {
    UndefVar,
    UndefComponent,
    Bool(bool),
    Algebra(algebra::Value),
    Function {
        args: Vec<String>,
        stmt: Box<StatementP>,
        path: String,
    },
    Template {
        attrs: Attributes,
        args: Vec<String>,
        stmt: Box<StatementP>,
        path: String,
    },
    Component {
        template: String,
        path: String,
        args: Vec<ReturnValue>,

        // None => Component already expanded
        // Some(n) => Signals pending for expansion
        pending_inputs: Vec<algebra::SignalId>,
    },
    List(List),
}

impl From<ReturnValue> for ScopeValue {
    fn from(v: ReturnValue) -> Self {
        match v {
            ReturnValue::Bool(v) => ScopeValue::Bool(v),
            ReturnValue::Algebra(v) => ScopeValue::Algebra(v),
            ReturnValue::List(v) => ScopeValue::List(v),
        }
    }
}

impl From<algebra::Value> for ScopeValue {
    fn from(v: algebra::Value) -> Self {
        ScopeValue::Algebra(v)
    }
}

pub struct Scope<'a> {
    start: bool,
    prev: Option<&'a Self>,
    pos: String,

    pub return_value: RefCell<Option<ReturnValue>>,
    pub vars: RefCell<HashMap<String, ScopeValue>>,
}

impl<'a> Scope<'a> {
    pub fn new(start: bool, prev: Option<&'a Scope>, pos: String) -> Self {
        Self {
            start,
            prev,
            pos,
            return_value: RefCell::new(None),
            vars: RefCell::new(HashMap::new()),
        }
    } 
    pub fn deep_clone(&self) -> Self {
        Self {
            start : self.start,
            prev  : self.prev.clone(),
            pos   : self.pos.clone(),
            return_value: self.return_value.clone(),
            vars: self.vars.clone(),
        }    
    }

    pub fn root(&self) -> &Scope {
        let mut this = self;
        while let Some(prev) = this.prev {
            this = prev;
        }
        this
    }

    pub fn insert(&self, k: String, v: ScopeValue) {
        if self.vars.borrow().contains_key(&k) {
            panic!("cannot insert into scope a duplicated key '{}'", k);
        }
        self.vars.borrow_mut().insert(k, v);
    }

    pub fn get<F, R>(&self, key: &'a str, func: F) -> R
    where
        F: FnOnce(Option<&ScopeValue>) -> R,
    {
        if let Some(value) = self.vars.borrow().get(key) {
            func(Some(value))
        } else if !self.start {
            if let Some(prev) = self.prev {
                prev.get(key, func)
            } else {
                func(None)
            }
        } else {
            func(None)
        }
    }

    pub fn get_mut<F, R>(&self, key: &'a str, func: F) -> R
    where
        F: FnOnce(Option<&mut ScopeValue>) -> R,
    {
        if let Some(value) = self.vars.borrow_mut().get_mut(key) {
            func(Some(value))
        } else if !self.start {
            if let Some(prev) = self.prev {
                prev.get_mut(key, func)
            } else {
                func(None)
            }
        } else {
            func(None)
        }
    }

    pub fn contains_key(&self, key: &'a str) -> bool {
        if self.vars.borrow().contains_key(key) {
            true
        } else if !self.start {
            if let Some(prev) = self.prev {
                prev.contains_key(key)
            } else {
                false
            }
        } else {
            false
        }
    }

    pub fn update(&self, key: &'a str, v: ScopeValue) -> Result<()> {
        if self.vars.borrow().contains_key(key) {
            self.vars.borrow_mut().insert(key.to_string(), v);
            Ok(())
        } else if !self.start {
            if let Some(prev) = self.prev {
                prev.update(key, v)
            } else {
                Err(Error::NotFound(key.to_string()))
            }
        } else {
            Err(Error::NotFound(key.to_string()))
        }
    }

    pub fn set_return(&self, v: ReturnValue) {
        if self.start {
            *self.return_value.borrow_mut() = Some(v);
        } else if let Some(prev) = self.prev {
            prev.set_return(v);
        }
    }

    pub fn take_return(&self) -> Option<ReturnValue> {
        if self.start {
            self.return_value.borrow_mut().take()
        } else if let Some(prev) = self.prev {
            prev.take_return()
        } else {
            None
        }
    }

    pub fn has_return(&self) -> bool {
        if self.start {
            self.return_value.borrow().is_some()
        } else if let Some(prev) = self.prev {
            prev.has_return()
        } else {
            false
        }
    }
}

impl<'a> Debug for Scope<'a> {
    fn fmt(&self, fmt: &mut Formatter) -> std::result::Result<(), std::fmt::Error> {
        writeln!(fmt, "--------------------------------------------")?;
        writeln!(fmt, "{}", self.pos)?;
        writeln!(fmt, "  start: {}", self.start)?;
        writeln!(fmt, "  return_value: {:?}", self.return_value.borrow())?;
        for (k, v) in &*self.vars.borrow() {
            if self.prev.is_some() {
                writeln!(fmt, "  {}: {:?}", k, v)?;
            }
        }
        if let Some(prev) = self.prev {
            writeln!(fmt, "{:?}", prev)?;
        }
        Ok(())
    }
}
