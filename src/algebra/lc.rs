use std::fmt;
use std::iter;
use std::ops::{Add, Mul, Neg};

use super::traits::AlgZero;
use super::constants::*;
use super::types::*;

impl LC {
    pub fn new() -> Self {
        LC(vec![])
    }
    pub fn from_signal(signal: &str, fs: FS) -> Self {
        LC(vec![(signal.to_string(), fs)])
    }
    pub fn get(&self, signal: &str) -> Option<&FS> {
        if let Some(p) = self.0.iter().position(|x| x.0 == signal) {
            Some(&self.0[p].1)
        } else {
            None
        }
    }
    pub fn set<F>(&mut self, signal: &str, func: F)
    where
        F: FnOnce(Option<&FS>) -> FS,
    {
        if let Some(p) = self.0.iter().position(|x| x.0 == signal) {
            self.0[p].1 = func(Some(&self.0[p].1));
        } else {
            self.0.push((signal.to_string(), func(None)));
        }
    }
    pub fn rm(&mut self, signal: &str) {
        self.0.retain(|(s, _)| s != signal);
    }
}

impl Default for LC {
    fn default() -> Self {
        LC::new()
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

pub trait Substitute {
    fn substitute(&self, signal: &str, equivalenc: &LC) -> Self;
}

impl Substitute for LC {
    fn substitute(&self, signal: &str, equivalenc: &LC) -> Self {
        let mut res = self.clone();
        if let Some(coef) = self.get(signal) {
            for (eq_signal, eq_value) in &equivalenc.0 {
                if signal != eq_signal {
                    let mut v = coef * eq_value;
                    if let Some(res_value) = res.get(&eq_signal) {
                        v = &v * res_value;
                    }
                    if v.is_zero() {
                        res.rm(&eq_signal);
                    } else {
                        res.set(&eq_signal, |_| v);
                    }
                }
            }
            res.rm(&signal);
        }
        res
    }
}

impl<'a> From<&'a FS> for LC {
    fn from(fs: &'a FS) -> Self {
        LC(vec![(SIGNAL_ONE.to_string(), fs.clone())])
    }
}

impl fmt::Debug for LC {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> std::result::Result<(), fmt::Error> {
        
        let s = if let Some((head,tail)) = self.0.split_first() {
            let head = format!("{}{}",head.1.format(false),head.0);
            let tail = tail.iter().map(|(s,v)|  format!("{}{}",v.format(true),s));
            iter::once(head).chain(tail).collect::<Vec<_>>().join("")
        } else {
            "0".to_string()
        };

        write!(fmt, "{}", s)
    }
}

// -&LC -> LC
impl<'a> Neg for &'a LC {
    type Output = LC;

    fn neg(self) -> LC {
        LC(self.0.iter().map(|(s, v)| (s.clone(), -v)).collect())
    }
}

// &LC + &FS -> LC
impl<'a> Add<&'a FS> for &'a LC {
    type Output = LC;

    fn add(self, rhs: &'a FS) -> LC {
        let mut v = self.0.clone();

        if let Some(i) = v.iter().position(|(s, _)| s == SIGNAL_ONE) {
            v[i].1 += rhs;
        } else {
            v.push((SIGNAL_ONE.to_string(), rhs.clone()))
        }

        LC(v)
    }
}

// &LC * &FS -> LC
impl<'a> Mul<&'a FS> for &'a LC {
    type Output = LC;

    fn mul(self, rhs: &'a FS) -> LC {
        LC(self.0.iter().map(|(s, e)| (s.clone(), e * rhs)).collect())
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
                v.push((signal.clone(), e.clone()));
            }
        }
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
    use num_bigint::ToBigUint;

    fn u32_to_fs(n: u32) -> FS {
        FS::from(n.to_biguint().unwrap())
    }

    #[test]
    fn test_lc_set_get_rm() {
        let mut lc = LC::zero();

        assert_eq!("0", format!("{:?}", lc));
        assert!(lc.get("s1").is_none());

        lc.set("s1", |_| u32_to_fs(2));
        assert_eq!("2s1", format!("{:?}", lc));

        lc.set("s1", |_| u32_to_fs(3));
        assert_eq!("3s1", format!("{:?}", lc));

        lc.set("s2", |_| u32_to_fs(2));
        assert_eq!("3s1+2s2", format!("{:?}", lc));

        assert_eq!("3", format!("{:?}", lc.get("s1").unwrap()));
        assert_eq!("2", format!("{:?}", lc.get("s2").unwrap()));

        lc.rm("s1");
        assert_eq!("2s2", format!("{:?}", lc));

        lc.rm("s2");
        assert_eq!("0", format!("{:?}", lc));
    }

    #[test]
    fn test_lc_fs_add_mul() {
        let one = &FS::one();
        let two = &(one + one);

        let lc1s1 = &LC::from_signal("s1", FS::one());
        assert_eq!("1s1+2one", format!("{:?}", &(lc1s1 + one) + one));

        let v1s12one = &(lc1s1 + two);
        assert_eq!("2s1+4one", format!("{:?}", v1s12one * two));
    }

    #[test]
    fn test_lc_neg() {
        let lc1s1 = &LC::from_signal("s1", FS::one());
        let lc1s2 = &LC::from_signal("s2", FS::one());
        let nlc1s1lc1s2 = &(&(-lc1s1) + lc1s2);
        assert_eq!("-1s1+1s2", format!("{:?}", nlc1s1lc1s2));
        let neg_nlc1s1lc1s2 = &-nlc1s1lc1s2;
        assert_eq!("1s1-1s2", format!("{:?}", neg_nlc1s1lc1s2));
    }

    #[test]
    fn test_lc_lc_add_mul() {
        let lc1s1 = &LC::from_signal("s1", FS::one());
        let lc1s2 = &LC::from_signal("s2", FS::one());
        assert_eq!("1s1", format!("{:?}", lc1s1));
        assert_eq!("2s1", format!("{:?}", lc1s1 + lc1s1));
        let lc2s1lc1s2 = &(lc1s1 + lc1s1) + lc1s2;

        assert_eq!("2s1+1s2", format!("{:?}", &lc2s1lc1s2));
        assert_eq!("[2s1+1s2]*[1s2]+[ ]", format!("{:?}", &lc2s1lc1s2 * lc1s2));
    }

    #[test]
    fn test_le_substitute() {
        let lc1s1lc2s2 = &LC::from_signal("s1", u32_to_fs(1)) + &LC::from_signal("s2", u32_to_fs(2));
        let lc3s3 = LC::from_signal("s3", u32_to_fs(3));
        let lc1s1lc6s3 = lc1s1lc2s2.substitute("s2", &lc3s3);
        assert_eq!("1s1+6s3", format!("{:?}", lc1s1lc6s3));
    }


}
