use std::fmt;
use std::ops::{Add, Mul, Neg};

use super::types::*;
use super::traits::AlgZero;
use super::lc::Substitute;

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
        let f = |v:&LC| if v.0.len() > 0 { format!("{:?}",v) } else { " ".to_string() };
        write!(fmt, "[{}]*[{}]+[{}]", f(&self.a), f(&self.b), f(&self.c))
    }
}

impl Substitute for QEQ {
    fn substitute(&self, signal: &str, equivalenc: &LC) -> Self {
        QEQ {
            a: self.a.substitute(signal, equivalenc),
            b: self.b.substitute(signal, equivalenc),
            c: self.c.substitute(signal, equivalenc),
        }
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
    use num_bigint::ToBigUint;

    fn u32_to_fs(n: u32) -> FS {
        FS::from(n.to_biguint().unwrap())
    }

    #[test]
    fn test_qeq_fs_add_mul() {
        let one = &FS::one();
        let two = &(one + one);
        let lc1s1 = &LC::from_signal("s1", FS::one());
        let lc1s2 = &LC::from_signal("s2", FS::one());
        let lc1s1lc1s2one = &(lc1s1 * lc1s2) + one;
        assert_eq!("[1s1]*[1s2]+[1one]", format!("{:?}", lc1s1lc1s2one));
        assert_eq!("[2s1]*[1s2]+[2one]", format!("{:?}", &lc1s1lc1s2one * two));
    }

    #[test]
    fn test_qeq_neg() {
        let lc1s1 = &LC::from_signal("s1", FS::one());
        let qeq = &(&(&(lc1s1 + lc1s1) * lc1s1) + lc1s1);
        let neq_qeq = &-qeq;
        assert_eq!("[2s1]*[1s1]+[1s1]", format!("{:?}", -neq_qeq));
    }


    #[test]
    fn test_qeq_substitute() {
        let lc2s2 = LC::from_signal("s2", u32_to_fs(2));
        let lc2s3 = LC::from_signal("s2", u32_to_fs(3));
        let lc2s4 = LC::from_signal("s2", u32_to_fs(4));
        let qeq = &(&lc2s2 * &lc2s3) + &lc2s4;
        let lc3s3 = LC::from_signal("s3", u32_to_fs(3));
        let qeq_subst = qeq.substitute("s2", &lc3s3);

        assert_eq!("[6s3]*[9s3]+[12s3]", format!("{:?}", qeq_subst));
    }

}
