mod factory;
mod error;
mod ram;
mod rocks;
mod types;


pub use self::ram::{Ram,RamSignals,RamConstraints};
pub use self::rocks::{Rocks,RocksSignals,RockConstraints};
pub use self::factory::StorageFactory;
pub use self::error::{Error,Result};
pub use self::types::{Constraints, Signal, Signals};