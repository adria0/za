mod error;
mod factory;
mod ram;
mod types;
mod utils;

pub use self::error::{Error, Result};
pub use self::ram::{Ram, RamConstraints, RamSignals};
pub use self::types::{Constraints, Signal, Signals,StorageFactory,SignalName};
pub use self::utils::{count_public_inputs,is_public_input};