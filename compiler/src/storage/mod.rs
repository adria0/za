mod error;
mod factory;
mod ram;
mod rocks;
mod types;
mod cuda;

pub use self::error::{Error, Result};
pub use self::ram::{Ram, RamConstraints, RamSignals};
pub use self::rocks::{RockConstraints, Rocks, RocksSignals};
pub use self::types::{Constraints, Signal, Signals,StorageFactory};
pub use self::cuda::generate_cuda;