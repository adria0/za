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
pub struct ScopeValueGuard<'a,'b> {
    guard : std::cell::Ref<'a,HashMap<String, ScopeValue>>,
    key : &'b str,
}
impl<'a,'b> std::ops::Deref for ScopeValueGuard<'a,'b> {
    type Target = ScopeValue;

    fn deref(&self) -> &Self::Target {
        &self.guard.get(self.key).unwrap()
    }
}

#[derive(Debug)]
pub struct ScopeValueGuardMut<'a,'b> {
    guard : std::cell::RefMut<'a,HashMap<String, ScopeValue>>,
    key : &'b str,
}
impl<'a,'b> std::ops::Deref for ScopeValueGuardMut<'a,'b> {
    type Target = ScopeValue;

    fn deref(&self) -> &Self::Target {
        &self.guard.get(self.key).unwrap()
    }
}
impl<'a,'b> std::ops::DerefMut for ScopeValueGuardMut<'a,'b> {
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
        let mut this = self;
        while let Some(prev) = this.prev {
            this = prev;
        }
        this
    }

    pub fn insert(&self, k: String, v: ScopeValue) -> Result<()> {
        if self.vars.borrow().contains_key(&k) {
            Err(Error::AlreadyExists(k.to_string()))
        } else {
            self.vars.borrow_mut().insert(k, v);
            Ok(())
        }
    }

    pub fn get(&self, key: &'a str) -> Option<ScopeValueGuard> {
        if self.vars.borrow().contains_key(key) {
            Some(ScopeValueGuard { guard: self.vars.borrow(), key })
        } else if !self.start {
            if let Some(prev) = self.prev {
                prev.get(key)
            } else {
                None
            }
        } else {
            None
        }
    }

    pub fn get_mut(&self, key: &'a str) -> Option<ScopeValueGuardMut> {
        if self.vars.borrow().contains_key(key) {
            Some(ScopeValueGuardMut { guard: self.vars.borrow_mut(), key })
        } else if !self.start {
            if let Some(prev) = self.prev {
                prev.get_mut(key)
            } else {
                None
            }
        } else {
            None
        }
    }

    pub fn get_f<F, R>(&self, key: &'a str, func: F) -> R
    where
        F: FnOnce(Option<&ScopeValue>) -> R,
    {
        if let Some(value) = self.vars.borrow().get(key) {
            func(Some(value))
        } else if !self.start {
            if let Some(prev) = self.prev {
                prev.get_f(key, func)
            } else {
                func(None)
            }
        } else {
            func(None)
        }
    }

    pub fn get_mut_f<F, R>(&self, key: &'a str, func: F) -> R
    where
        F: FnOnce(Option<&mut ScopeValue>) -> R,
    {
        if let Some(value) = self.vars.borrow_mut().get_mut(key) {
            func(Some(value))
        } else if !self.start {
            if let Some(prev) = self.prev {
                prev.get_mut_f(key, func)
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
    fn test_scope_basic() -> Result<()>{
        let sc = Scope::new(true, None, "sc1".to_string());
        sc.insert("k1".to_string(), ScopeValue::Bool(true))?;
        
        assert_eq!("Bool(true)",format!("{:?}",&*sc.get("k1").unwrap()));
        *sc.get_mut("k1").unwrap() = ScopeValue::Bool(false);
        assert_eq!("Bool(false)",format!("{:?}",&*sc.get("k1").unwrap()));
        
        sc.update("k1",ScopeValue::Bool(true))?;

        sc.get_mut_f("k1", |v| {
            assert_eq!("Some(Bool(true))",format!("{:?}",v));
            *v.unwrap() = ScopeValue::Bool(false);
        });
        sc.get_f("k1", |v| {
            assert_eq!("Some(Bool(false))",format!("{:?}",v));
        });
        Ok(())
    }

    #[test]
    fn test_no_duplicated_key() -> Result<()> {
        let sc = Scope::new(true, None, "sc1".to_string());

        sc.insert("k1".to_string(), ScopeValue::Bool(true))?;
        assert_eq!(true,sc.insert("k1".to_string(), ScopeValue::Bool(false)).is_err());

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

        assert_eq!("Some(Bool(true))",format!("{:?}",ret));


        Ok(())
    }

}
