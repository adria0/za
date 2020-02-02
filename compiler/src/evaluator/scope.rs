#![allow(dead_code)]

use itertools::sorted;
use std::cell::RefCell;
use std::collections::HashMap;
use std::fmt::{Debug, Formatter};

use za_parser::ast::{Attributes, StatementP};

use super::algebra;
use super::error::*;
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

#[derive(Clone)]
pub struct Scope<'a> {
    start: bool,
    prev: Option<&'a Self>,
    pos: String,

    pub return_value: RefCell<Option<ReturnValue>>,
    pub vars: RefCell<HashMap<String, ScopeValue>>,
}

#[derive(Debug)]
pub struct ScopeValueGuard<'a, 'b> {
    guard: std::cell::Ref<'a, HashMap<String, ScopeValue>>,
    key: &'b str,
}
impl<'a, 'b> std::ops::Deref for ScopeValueGuard<'a, 'b> {
    type Target = ScopeValue;

    fn deref(&self) -> &Self::Target {
        &self.guard.get(self.key).unwrap()
    }
}

#[derive(Debug)]
pub struct ScopeValueGuardMut<'a, 'b> {
    guard: std::cell::RefMut<'a, HashMap<String, ScopeValue>>,
    key: &'b str,
}
impl<'a, 'b> std::ops::Deref for ScopeValueGuardMut<'a, 'b> {
    type Target = ScopeValue;

    fn deref(&self) -> &Self::Target {
        &self.guard.get(self.key).unwrap()
    }
}
impl<'a, 'b> std::ops::DerefMut for ScopeValueGuardMut<'a, 'b> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.guard.get_mut(self.key).unwrap()
    }
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

    pub fn root(&self) -> &Scope {
        let mut it = self;
        while let Some(prev) = it.prev {
            it = prev;
        }
        it
    }

    pub fn start(&self) -> &Scope {
        let mut it = self;
        while !it.start {
            it = it.prev.unwrap();
        }
        it
    }

    pub fn insert(&self, k: String, v: ScopeValue) -> Result<()> {
        if self.vars.borrow().contains_key(&k) {
            Err(Error::AlreadyExists(k))
        } else {
            self.vars.borrow_mut().insert(k, v);
            Ok(())
        }
    }

    pub fn get(&self, key: &'a str) -> Option<ScopeValueGuard> {
        let mut it = self;
        loop {
            if it.vars.borrow().contains_key(key) {
                return Some(ScopeValueGuard {
                    guard: it.vars.borrow(),
                    key,
                });
            } else if it.prev.is_none() || it.start {
                return None;
            }
            it = it.prev.unwrap();
        }
    }

    pub fn get_mut(&self, key: &'a str) -> Option<ScopeValueGuardMut> {
        let mut it = self;
        loop {
            if it.vars.borrow().contains_key(key) {
                return Some(ScopeValueGuardMut {
                    guard: it.vars.borrow_mut(),
                    key,
                });
            } else if it.prev.is_none() || it.start {
                return None;
            }
            it = it.prev.unwrap();
        }
    }

    pub fn contains_key(&self, key: &'a str) -> bool {
        let mut it = self;
        loop {
            if it.vars.borrow().contains_key(key) {
                return true;
            } else if it.prev.is_none() || it.start {
                return false;
            }
            it = it.prev.unwrap();
        }
    }

    pub fn update(&self, key: &'a str, v: ScopeValue) -> Result<()> {
        let mut scope_value = self
            .get_mut(key)
            .ok_or_else(|| Error::NotFound(key.to_string()))?;
        *scope_value = v;
        Ok(())
    }

    pub fn set_return(&self, v: ReturnValue) {
        *self.start().return_value.borrow_mut() = Some(v);
    }

    pub fn take_return(&self) -> Option<ReturnValue> {
        self.start().return_value.borrow_mut().take()
    }

    pub fn has_return(&self) -> bool {
        self.start().return_value.borrow().is_some()
    }
}

impl<'a> Debug for Scope<'a> {
    fn fmt(&self, fmt: &mut Formatter) -> std::result::Result<(), std::fmt::Error> {
        writeln!(fmt, "--------------------------------------------")?;
        writeln!(fmt, "{}", self.pos)?;
        writeln!(fmt, "  start: {}", self.start)?;
        writeln!(fmt, "  return_value: {:?}", self.return_value.borrow())?;
        let vars = &*self.vars.borrow();
        for k in sorted(vars.keys()) {
            let v = vars.get(k).unwrap();
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

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_scope_basic() -> Result<()> {
        let sc = Scope::new(true, None, "sc1".to_string());
        sc.insert("k1".to_string(), ScopeValue::Bool(true))?;

        assert_eq!("Bool(true)", format!("{:?}", &*sc.get("k1").unwrap()));
        *sc.get_mut("k1").unwrap() = ScopeValue::Bool(false);
        assert_eq!("Bool(false)", format!("{:?}", &*sc.get("k1").unwrap()));

        sc.update("k1", ScopeValue::Bool(true))?;
        assert_eq!("Bool(true)", format!("{:?}", &*sc.get("k1").unwrap()));

        Ok(())
    }

    #[test]
    fn test_no_duplicated_key() -> Result<()> {
        let sc = Scope::new(true, None, "sc1".to_string());

        sc.insert("k1".to_string(), ScopeValue::Bool(true))?;
        assert_eq!(
            true,
            sc.insert("k1".to_string(), ScopeValue::Bool(false))
                .is_err()
        );

        Ok(())
    }

    #[test]
    fn test_shadowing_allowed() -> Result<()> {
        let sc1 = Scope::new(true, None, "sc1".to_string());
        sc1.insert("k1".to_string(), ScopeValue::Bool(true))?;
        let sc2 = Scope::new(false, Some(&sc1), "sc2".to_string());
        sc2.insert("k1".to_string(), ScopeValue::Bool(true))?;
        Ok(())
    }

    #[test]
    fn test_find_deeper_var_in_same_start() -> Result<()> {
        let sc1 = Scope::new(true, None, "sc1".to_string());
        sc1.insert("k1".to_string(), ScopeValue::Bool(true))?;
        let sc2 = Scope::new(false, Some(&sc1), "sc2".to_string());
        sc2.insert("k2".to_string(), ScopeValue::Bool(true))?;

        assert_eq!(true, sc2.contains_key("k1"));

        Ok(())
    }

    #[test]
    fn test_find_deeper_var_in_another_start() -> Result<()> {
        let sc1 = Scope::new(true, None, "sc1".to_string());
        sc1.insert("k1".to_string(), ScopeValue::Bool(true))?;
        let sc2 = Scope::new(true, Some(&sc1), "sc2".to_string());
        sc2.insert("k2".to_string(), ScopeValue::Bool(true))?;

        assert_eq!(false, sc2.contains_key("k1"));
        assert_eq!(true, sc2.root().contains_key("k1"));

        Ok(())
    }

    #[test]
    fn test_returns() -> Result<()> {
        let sc1 = Scope::new(true, None, "sc1".to_string());
        sc1.insert("k1".to_string(), ScopeValue::Bool(true))?;
        let sc2 = Scope::new(false, Some(&sc1), "sc2".to_string());
        sc2.insert("k2".to_string(), ScopeValue::Bool(true))?;

        assert_eq!(false, sc1.has_return());
        assert_eq!(false, sc2.has_return());

        sc2.set_return(ReturnValue::Bool(true));
        assert_eq!(true, sc1.has_return());
        assert_eq!(true, sc2.has_return());

        let ret = sc2.take_return();
        assert_eq!(false, sc1.has_return());
        assert_eq!(false, sc2.has_return());

        assert_eq!("Some(Bool(true))", format!("{:?}", ret));

        Ok(())
    }
}
