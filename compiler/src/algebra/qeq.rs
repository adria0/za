use std::fmt;
use std::ops::{Add, Mul, Neg};

use super::traits::AlgZero;
use super::types::*;

impl QEQ {
    pub fn format<F>(&self, func: F) -> String
    where
        F: Fn(SignalId) -> String,
    {
        let f = |v: &LC| {
            if !v.0.is_empty() {
                v.format(&func)
            } else {
                " ".to_string()
            }
        };
        format!("[{}]*[{}]+[{}]", f(&self.a), f(&self.b), f(&self.c))
    }
}

impl AlgZero for QEQ {
    fn zero() -> Self {
        QEQ {
            a: LC::zero(),
            b: LC::zero(),
            c: LC::zero(),
        }
    }
    fn is_zero(&self) -> bool {
        (self.a.is_zero() || self.b.is_zero()) && self.c.is_zero()
    }
}

impl fmt::Debug for QEQ {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> std::result::Result<(), fmt::Error> {
        write!(fmt, "{}", self.format(|s| format!("s{}", s)))
    }
}

// &QEQ + &FS -> QEQ
impl<'a> Add<&'a FS> for &'a QEQ {
    type Output = QEQ;

    fn add(self, rhs: &'a FS) -> QEQ {
        QEQ {
            a: self.a.clone(),
            b: self.b.clone(),
            c: &self.c + rhs,
        }
    }
}

// &QEQ * &FS -> QEQ
impl<'a> Mul<&'a FS> for &'a QEQ {
    type Output = QEQ;

    fn mul(self, rhs: &'a FS) -> QEQ {
        QEQ {
            a: &self.a * rhs,
            b: self.b.clone(),
            c: &self.c * rhs,
        }
    }
}

// &QEQ + &LC -> QEQ
impl<'a> Add<&'a LC> for &'a QEQ {
    type Output = QEQ;

    fn add(self, rhs: &'a LC) -> QEQ {
        QEQ {
            a: self.a.clone(),
            b: self.b.clone(),
            c: &self.c + rhs,
        }
    }
}

// -&QEQ -> QEQ
impl<'a> Neg for &'a QEQ {
    type Output = QEQ;

    fn neg(self) -> QEQ {
        QEQ {
            a: -&self.a,
            b: self.b.clone(),
            c: -&self.c,
        }
    }
}

impl<'a> From<&'a FS> for QEQ {
    fn from(fs: &'a FS) -> Self {
        QEQ {
            a: LC::new(),
            b: LC::new(),
            c: LC::from(fs),
        }
    }
}

impl<'a> From<&'a LC> for QEQ {
    fn from(lc: &'a LC) -> Self {
        QEQ {
            a: LC::new(),
            b: LC::new(),
            c: lc.clone(),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_qeq_fs_add_mul() {
        let one = &FS::one();
        let two = &(one + one);
        let s1 = 1 as SignalId;
        let s2 = 2 as SignalId;

        let lc_1s1 = &LC::from_signal(s1, FS::one());
        let lc_1s2 = &LC::from_signal(s2, FS::one());
        let lc_1s1_1s2_one = &(lc_1s1 * lc_1s2) + one;

        assert_eq!("[1s1]*[1s2]+[1s0]", format!("{:?}", lc_1s1_1s2_one));
        assert_eq!("[2s1]*[1s2]+[2s0]", format!("{:?}", &lc_1s1_1s2_one * two));
    }

    #[test]
    fn test_qeq_neg() {
        let s1 = 1 as SignalId;
        let lc_1s1 = &LC::from_signal(s1, FS::one());
        let qeq = &(&(&(lc_1s1 + lc_1s1) * lc_1s1) + lc_1s1);
        let neq_qeq = &-qeq;
        assert_eq!("[2s1]*[1s1]+[1s1]", format!("{:?}", -neq_qeq));
    }

}
