use rocksdb::{Direction, IteratorMode, DB};
use circom2_parser::ast::SignalType;

use crate::algebra;
use crate::algebra::{QEQ,SignalId};
use crate::evaluator::{SignalName,Signal,Signals};
use crate::evaluator::Constraints;
use super::StorageFactory;
/*
use super::Result;
pub struct Options {

}

pub struct Rocks {
    opt : Options,
    db : DB,
}

impl StorageFactory<RocksSignals,RockConstraints> for Rocks {
    fn new_signals(&self) -> RocksSignals {
        RocksSignals::default()
    }
    fn new_constraints(&self) -> RockConstraints {
        RockConstraints::default()
    }
}

impl Rocks {
    pub fn new(path: &str, opt : Options ) -> Result<Rocks> {
        Ok(DB::open_default(path).map(|x| Rocks{ opt, db: x })?)
    }
}

struct RocksSignals<'a> {
    rocks : &'a Rocks,
}

struct RockConstraints<'a> {
    rocks : &'a Rocks,    
}

impl<'a> Signals for RocksSignals<'a> {
    fn len(&self) -> usize {
        unimplemented!();
    }
    fn get_by_id(&self, id : SignalId) -> Option<&Signal> {
        unimplemented!();
    }
    fn get_by_id_mut(&mut self, id : SignalId) -> Option<&mut Signal> {
        unimplemented!();
    }
    fn get_by_name(&self, full_name : &str) -> Option<&Signal> {
        unimplemented!();
    }
    fn get_by_name_mut(&mut self, full_name : &str) -> Option<&mut Signal> {
        unimplemented!();
    }
    fn insert(&mut self, full_name: String, xtype: SignalType, value : Option<algebra::Value>) -> SignalId {
        unimplemented!();
    }
    fn to_string(&self, id : SignalId) -> String {
        unimplemented!();
    }
}

impl<'a> RockConstraints<'a>{
    /// increment an u64 counter
    fn inc_u64(&self, key : &[u8]) -> Result<u64> {
         let value = 1+self.get_u64(&key)?.unwrap_or(0);
         self.set_u64(&key,value)?;
         Ok(value)       
    }

    /// get an u64 counter
    fn get_u64(&self, key : &[u8]) -> Result<Option<u64>> {
        Ok(self
            .db
            .get(&key)
            .map(|bytes| bytes.map(|v| u64_from_slice(&*v)))?)
    }

    /// set an u64 counter
    fn set_u64(&self, key: &[u8], n: u64) -> Result<()> {
        self.db.put(&key, &u64_to_le(n))?;
        Ok(())
    }
}

impl<'a> Constraints for RockConstraints<'a> {
    fn len(&self) -> usize {
        unimplemented!();
    }
    fn get(&self, i : usize) -> QEQ {
        unimplemented!();
    }
    fn push(&mut self, qeq : QEQ) -> usize {
        unimplemented!();
    }

}
*/