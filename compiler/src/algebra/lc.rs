use std::fmt;
use std::iter;
use std::ops::{Add, Mul, Neg};

use super::traits::AlgZero;
use super::types::*;

impl LC {
    pub fn new() -> Self {
        LC(vec![])
    }
    pub fn from_signal(signal: SignalId, fs: FS) -> Self {
        LC(vec![(signal, fs)])
    }
    pub fn get(&self, signal: SignalId) -> Option<&FS> {
        if let Some(p) = self.0.iter().position(|x| x.0 == signal) {
            Some(&self.0[p].1)
        } else {
            None
        }
    }
    pub fn set<F>(&mut self, signal: SignalId, func: F)
    where
        F: FnOnce(Option<&FS>) -> FS,
    {
        if let Some(p) = self.0.iter().position(|x| x.0 == signal) {
            self.0[p].1 = func(Some(&self.0[p].1));
        } else {
            self.0.push((signal, func(None)));
        }
    }
    pub fn rm(&mut self, signal: SignalId) {
        self.0.retain(|(s, _)| *s != signal);
    }
    pub fn format<F>(&self, func: F) -> String
    where
        F: Fn(SignalId) -> String,
    {
        if let Some((head, tail)) = self.0.split_first() {
            let head = format!("{}{}", head.1.format(false), func(head.0));
            let tail = tail
                .iter()
                .map(|(s, v)| format!("{}{}", v.format(true), func(*s)));
            iter::once(head).chain(tail).collect::<Vec<_>>().join("")
        } else {
            "0".to_string()
        }
    }
}

impl Default for LC {
    fn default() -> Self {
        LC::new()
    }
}

impl fmt::Display for LC {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.format(|s| format!("s{}", s)))
    }
}

impl fmt::Debug for LC {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> std::result::Result<(), fmt::Error> {
        write!(fmt, "{}", self.to_string())
    }
}

impl AlgZero for LC {
    fn zero() -> Self {
        LC(vec![])
    }
    fn is_zero(&self) -> bool {
        !self.0.iter().any(|(_, e)| !e.is_zero())
    }
}

impl<'a> From<&'a FS> for LC {
    fn from(fs: &'a FS) -> Self {
        LC(vec![(SIGNAL_ONE, fs.clone())])
    }
}

// -&LC -> LC
impl<'a> Neg for &'a LC {
    type Output = LC;

    fn neg(self) -> LC {
        LC(self.0.iter().map(|(s, v)| (*s, -v)).collect())
    }
}

// &LC + &FS -> LC
impl<'a> Add<&'a FS> for &'a LC {
    type Output = LC;

    fn add(self, rhs: &'a FS) -> LC {
        let mut v = self.0.clone();

        if let Some(i) = v.iter().position(|(s, _)| *s == SIGNAL_ONE) {
            v[i].1 += rhs;
        } else {
            v.push((SIGNAL_ONE, rhs.clone()))
        }
        v.retain(|v| !v.1.is_zero());
        LC(v)
    }
}

// &LC * &FS -> LC
impl<'a> Mul<&'a FS> for &'a LC {
    type Output = LC;

    fn mul(self, rhs: &'a FS) -> LC {
        if rhs.is_zero() {
            LC::zero()
        } else {
            LC(self.0.iter().map(|(s, e)| (*s, e * rhs)).collect())
        }
    }
}

// &LC + &LC -> LC
impl<'a> Add<&'a LC> for &'a LC {
    type Output = LC;

    fn add(self, rhs: &'a LC) -> LC {
        let mut v = self.0.clone();
        for (signal, e) in &rhs.0 {
            if let Some(i) = v.iter().position(|(s, _)| s == signal) {
                v[i].1 += e;
            } else {
                v.push((*signal, e.clone()));
            }
        }
        v.retain(|v| !v.1.is_zero());
        LC(v)
    }
}

// &LC * &LC -> QEQ
impl<'a> Mul<&'a LC> for &'a LC {
    type Output = QEQ;

    fn mul(self, rhs: &'a LC) -> QEQ {
        QEQ {
            a: self.clone(),
            b: rhs.clone(),
            c: LC::new(),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_lc_set_get_rm() {
        let mut lc = LC::zero();
        let s1 = 1 as SignalId;
        let s2 = 2 as SignalId;

        assert_eq!("0", lc.to_string());
        assert!(lc.get(s1).is_none());

        lc.set(s1, |_| FS::from(2));
        assert_eq!("2s1", lc.to_string());

        lc.set(s1, |_| FS::from(3));
        assert_eq!("3s1", lc.to_string());

        lc.set(s2, |_| FS::from(2));
        assert_eq!("3s1+2s2", lc.to_string());

        assert_eq!("3", lc.get(s1).unwrap().to_string());
        assert_eq!("2", lc.get(s2).unwrap().to_string());

        lc.rm(s1);
        assert_eq!("2s2", lc.to_string());

        lc.rm(s2);
        assert_eq!("0", lc.to_string());
    }

    #[test]
    fn test_lc_fs_add_mul() {
        let one = &FS::one();
        let two = &(one + one);
        let s1 = 1 as SignalId;

        let lc_1s1 = &LC::from_signal(s1, FS::one());
        assert_eq!("1s1+2s0", (&(lc_1s1 + one) + one).to_string());

        let lc_1s1_4one = &(lc_1s1 + two);
        assert_eq!("2s1+4s0", (lc_1s1_4one * two).to_string());
    }

    #[test]
    fn test_lc_neg() {
        let s1 = 1 as SignalId;
        let s2 = 2 as SignalId;
        let lc_1s1 = &LC::from_signal(s1, FS::one());
        let lc_1s2 = &LC::from_signal(s2, FS::one());

        let lc_n1s1_1s2 = &(&(-lc_1s1) + lc_1s2);
        assert_eq!("-1s1+1s2", lc_n1s1_1s2.to_string());
        let lc_1s1_n1s2 = &-lc_n1s1_1s2;
        assert_eq!("1s1-1s2", lc_1s1_n1s2.to_string());

        let lc_zero = lc_n1s1_1s2 + lc_1s1_n1s2;
        assert_eq!("0", lc_zero.to_string());
    }

    #[test]
    fn test_lc_lc_add_mul() {
        let s1 = 1 as SignalId;
        let s2 = 2 as SignalId;
        let lc_1s1 = &LC::from_signal(s1, FS::one());
        let lc_1s2 = &LC::from_signal(s2, FS::one());

        assert_eq!("1s1", lc_1s1.to_string());
        assert_eq!("2s1", (lc_1s1 + lc_1s1).to_string());
        let lc_2s1_1s2 = &(lc_1s1 + lc_1s1) + lc_1s2;

        assert_eq!("2s1+1s2", lc_2s1_1s2.to_string());
        assert_eq!("[2s1+1s2]*[1s2]+[ ]", (&lc_2s1_1s2 * lc_1s2).to_string());
    }
}
