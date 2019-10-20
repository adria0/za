mod fs;
mod lc;
mod qeq;
mod error;
mod value;

pub trait AlgZero {
    fn zero() -> Self;
    fn is_zero(&self) -> bool;
}

pub use self::error::{Error, Result};
pub use self::fs::FS;
pub use self::lc::{SignalId, LC, SIGNAL_ONE};
pub use self::qeq::QEQ;
pub use self::value::{eval_infix, eval_prefix, Value};
