use circom2_parser::ast::SignalType;
use circom2_compiler::algebra;
use circom2_compiler::algebra::{SignalId, QEQ};
use circom2_compiler::storage;
use circom2_compiler::storage::{StorageFactory,Constraints, Signal, SignalName, Signals};

use rocksdb::{DB};

use std::rc::Rc;

use serde_cbor::{from_slice, to_vec};
use std::path::PathBuf;

#[derive(Debug)]
pub enum Error {
    NotFound(String),
    RocksDb(rocksdb::Error),
    Cbor(serde_cbor::error::Error),
}

impl From<rocksdb::Error> for Error {
    fn from(err: rocksdb::Error) -> Self {
        Error::RocksDb(err)
    }
}

impl From<serde_cbor::error::Error> for Error {
    fn from(err: serde_cbor::error::Error) -> Self {
        Error::Cbor(err)
    }
}

pub type Result<T> = std::result::Result<T, Error>;

#[macro_export]
macro_rules! map_err {
    ( $x:block ) => {
        ({|| Ok($x) })()
            .map_err(|err:Error| storage::Error::Inner(format!("{:?})",err)))
    };
}

#[derive(Debug, Serialize, Deserialize)]
struct SignalEntry {
    pub id: u64,
    pub xtype: SignalType,
    pub full_name: String,
    pub value: Option<algebra::Value>,
}

pub struct Rocks {
    base_path: String,
    count: usize,
}

impl Rocks {
    pub fn new(base_path: String) -> Rocks {
        Rocks {
            base_path,
            count: 0,
        }
    }
}

impl StorageFactory<RocksSignals, RockConstraints> for Rocks {
    fn new_signals(&mut self) -> storage::Result<RocksSignals> {
        let mut full_path = PathBuf::new();
        full_path.push(&self.base_path);
        full_path.push(format!("_signals_{}", self.count));
        self.count += 1;
        RocksSignals::new(full_path.as_os_str().to_str().unwrap())
    }
    fn new_constraints(&mut self) -> storage::Result<RockConstraints> {
        let mut full_path = PathBuf::new();
        full_path.push(&self.base_path);
        full_path.push(format!("_constraints_{}", self.count));
        self.count += 1;
        RockConstraints::new(full_path.as_os_str().to_str().unwrap())
    }
}

pub struct RocksSignals {
    db: DB,
}

impl RocksSignals {
    pub fn new(path: &str) -> storage::Result<Self> {
        (||{
            Ok(DB::open_default(path).map(|x| RocksSignals { db: x })?)
        })().map_err(|err:Error| storage::Error::Inner(format!("{:?})",err)))
    }
}

pub struct RockConstraints {
    db: DB,
}

impl RockConstraints {
    pub fn new(path: &str) -> storage::Result<Self> {
        map_err!{{
            DB::open_default(path).map(|x| RockConstraints { db: x })?
        }}
    }               
}

impl RocksSignals {
    fn load(&self, id: SignalId) -> Result<Option<(Vec<u8>, SignalEntry)>> {
        let index_bytes = u64_to_le(id as u64);

        let mut key: Vec<u8> = vec![1];
        key.extend_from_slice(&index_bytes);

        match self.db.get(&key)? {
            None => Ok(None),
            Some(v) => Ok(Some((key, from_slice::<SignalEntry>(&v)?))),
        }
    }
    fn _get_by_id(&self, id: SignalId) -> Result<Option<Rc<Signal>>> {
        if let Some((_, entry)) = self.load(id)? {
            Ok(Some(Rc::new(Signal {
                id: entry.id as usize,
                xtype: entry.xtype,
                full_name: SignalName::new(entry.full_name),
                value: entry.value,
            })))
        } else {
            Ok(None)
        }
    }
}

impl<'a> Signals for RocksSignals {
    fn is_empty(&self) -> storage::Result<bool> {
        Ok(self.len()? == 0)
    }
    fn len(&self) -> storage::Result<usize> {
        map_err!{{
            get_u64(&self.db, &[0])?.unwrap_or(0) as usize
        }}
    }
    fn insert(
        &mut self,
        full_name: String,
        xtype: SignalType,
        value: Option<algebra::Value>,
    ) -> storage::Result<SignalId> {
        map_err!{{
            let index = inc_u64(&mut self.db, &[0])? - 1;
            let index_bytes = u64_to_le(index as u64);

            let entry = SignalEntry {
                id: index,
                xtype,
                full_name,
                value,
            };

            let mut key: Vec<u8> = vec![1];
            key.extend_from_slice(&index_bytes);
            self.db
                .put(&key.to_owned(), to_vec(&entry).unwrap().as_slice())?;

            let mut key: Vec<u8> = vec![2];
            key.extend_from_slice(entry.full_name.as_bytes());
            self.db.put(&key.to_owned(), &index_bytes)?;

            index as usize
        }}
    }

    fn update(&mut self, id: SignalId, value: algebra::Value) -> storage::Result<()> {
        map_err!{{
            if let Some((index, mut entry)) = self.load(id)? {
                entry.value = Some(value);
                self.db
                    .put(&index.to_owned(), to_vec(&entry).unwrap().as_slice())?;
                Ok(())
            } else {
                Err(Error::NotFound(format!("signal {}", id)))
            }?
        }}
    }

    fn get_by_id(&self, id: SignalId) -> storage::Result<Option<Rc<Signal>>> {
        map_err!{{
            self._get_by_id(id)?
        }}
    }

    fn get_by_name(&self, full_name: &str) -> storage::Result<Option<Rc<Signal>>> {
        map_err!{{
            let mut key: Vec<u8> = vec![2];
            key.extend_from_slice(full_name.as_bytes());
            match self.db.get(&key)? {
                None => Ok(None),
                Some(v) => self._get_by_id(u64_from_slice(&v) as usize),
            }?
        }}
    }

    fn to_string(&self, id: SignalId) -> storage::Result<String> {
        map_err!{{
            let (_, s) = self.load(id)?.unwrap();
            format!("{:?}:{:?}:{:?}", s.full_name, s.xtype, s.value)
        }}
    }
}

impl<'a> Constraints for RockConstraints {
    fn is_empty(&self) -> storage::Result<bool> {
        Ok(self.len()? == 0)
    }
    fn len(&self) -> storage::Result<usize> {
        map_err!{{
            get_u64(&self.db, &[0])?.unwrap_or(0) as usize
        }}
    }
    fn get(&self, i: usize) -> storage::Result<QEQ> {
        map_err!{{
            let mut key: Vec<u8> = vec![1];
            key.extend_from_slice(&u64_to_le(i as u64));
            match self.db.get(&key)? {
                None => Err(Error::NotFound(format!("Constraint at index {}", i))),
                Some(v) => Ok(from_slice::<QEQ>(&v)?),
            }?
        }}
    }
    fn get_debug(&self, _i: usize) -> Option<String> {
        None
    }
    fn push(&mut self, qeq: QEQ, _debug: Option<String>) -> storage::Result<usize> {
        map_err!{{
            let index = inc_u64(&mut self.db, &[0])? - 1;
            let mut key: Vec<u8> = vec![1];
            key.extend_from_slice(&u64_to_le(index as u64));
            self.db
                .put(&key.to_owned(), to_vec(&qeq).unwrap().as_slice())?;
            index as usize
        }}
    }
}

/// increment an u64 counter
fn inc_u64(db: &mut DB, key: &[u8]) -> Result<u64> {
    let value = 1 + get_u64(db, &key)?.unwrap_or(0);
    set_u64(db, &key, value)?;
    Ok(value)
}

/// get an u64 counter
fn get_u64(db: &DB, key: &[u8]) -> Result<Option<u64>> {
    Ok(db
        .get(&key)
        .map(|bytes| bytes.map(|v| u64_from_slice(&*v)))?)
}

/// set an u64 counter
fn set_u64(db: &mut DB, key: &[u8], n: u64) -> Result<()> {
    db.put(&key, &u64_to_le(n))?;
    Ok(())
}

/// get u64 as litte endian
fn u64_to_le(v: u64) -> [u8; 8] {
    [
        ((v >> 56) & 0xff) as u8,
        ((v >> 48) & 0xff) as u8,
        ((v >> 40) & 0xff) as u8,
        ((v >> 32) & 0xff) as u8,
        ((v >> 24) & 0xff) as u8,
        ((v >> 16) & 0xff) as u8,
        ((v >> 8) & 0xff) as u8,
        ((v) & 0xff) as u8,
    ]
}

/// get u64 from litte endian
fn le_to_u64(v: [u8; 8]) -> u64 {
    u64::from(v[7])
        + (u64::from(v[6]) << 8)
        + (u64::from(v[5]) << 16)
        + (u64::from(v[4]) << 24)
        + (u64::from(v[3]) << 32)
        + (u64::from(v[2]) << 40)
        + (u64::from(v[1]) << 48)
        + (u64::from(v[0]) << 56)
}

/// get u64 from litte endian slice
fn u64_from_slice(v: &[u8]) -> u64 {
    let mut le = [0; 8];
    le[..].copy_from_slice(v);
    le_to_u64(le)
}

#[cfg(test)]
mod test {

    use super::*;
    use circom2_compiler::algebra::{Value, FS};

    use super::SignalType;
    use rand::distributions::Alphanumeric;
    use rand::{thread_rng, Rng};
    use std::iter;

    fn init() -> Rocks {
        let mut rng = thread_rng();
        let chars: String = iter::repeat(())
            .map(|()| rng.sample(Alphanumeric))
            .take(7)
            .collect();

        let mut tmpfile = std::env::temp_dir();
        tmpfile.push(chars);

        let tmpfile = tmpfile.as_os_str().to_str().expect("bad OS filename");

        Rocks::new(tmpfile.to_string())
    }

    #[test]
    fn test_rocks_signals() -> storage::Result<()> {
        let one = FS::one();
        let two = &one + &one;
        let three = &one + &two;

        let mut rocks = init();
        let mut signals = rocks.new_signals()?;
        assert_eq!(0, signals.len()?);

        signals.insert(
            "s1".to_string(),
            SignalType::Internal,
            Some(Value::from(one)),
        )?;
        signals.insert(
            "s2".to_string(),
            SignalType::Internal,
            Some(Value::from(two)),
        )?;
        signals.insert("s3".to_string(), SignalType::Internal, None)?;
        assert_eq!(3, signals.len()?);

        let s1 = &*signals.get_by_name("s1")?.unwrap();
        assert_eq!("Some(1)", format!("{:?}", s1.value));

        let s2 = &*signals.get_by_name("s2")?.unwrap();
        assert_eq!("Some(2)", format!("{:?}", s2.value));

        let s3 = &*signals.get_by_name("s3")?.unwrap();
        assert_eq!("s3", s3.full_name.to_string());
        assert_eq!(true, s3.value.is_none());

        signals.update(s3.id, Value::from(three))?;
        assert_eq!(3, signals.len()?);

        let s3 = &*signals.get_by_name("s3")?.unwrap();
        assert_eq!("Some(3)", format!("{:?}", s3.value));

        Ok(())
    }

    #[test]
    fn test_rocks_constraints() -> storage::Result<()> {
        let one = QEQ::from(&FS::one());
        let two = QEQ::from(&(&FS::one() + &FS::one()));

        let mut rocks = init();
        let mut constraints = rocks.new_constraints()?;
        assert_eq!(0, constraints.len()?);

        let c1 = constraints.push(one, None)?;
        let c2 = constraints.push(two, None)?;

        assert_eq!(2, constraints.len()?);
        assert_eq!("[ ]*[ ]+[1s0]", format!("{:?}", constraints.get(c1)?));
        assert_eq!("[ ]*[ ]+[2s0]", format!("{:?}", constraints.get(c2)?));

        Ok(())
    }

}
