use num_bigint::{BigInt, BigUint};
use num_traits;
use num_traits::cast::FromPrimitive;
use num_traits::cast::ToPrimitive;
use num_traits::identities::{One, Zero};
use std::cmp::Ordering;
use std::fmt;
use std::ops::{Add, AddAssign, BitAnd, BitOr, BitXor, Div, Mul, Neg, Rem, Shl, Shr};

use super::error::{Error, Result};
use super::traits::AlgZero;
use super::types::*;

lazy_static! {
    pub static ref BABYJUB_FIELD_UINT: BigUint = BigUint::parse_bytes(
        b"21888242871839275222246405745257275088548364400416034343698204186575808495617",
        10
    )
    .unwrap();
    pub static ref BABYJUB_FIELD_UINT_NEG: BigUint = BigUint::parse_bytes(
        b"10944121435919637611123202872628637544274182200208017171849102093287904247808",
        10
    )
    .unwrap();
    pub static ref BABYJUB_FIELD_INT: BigInt = BigInt::parse_bytes(
        b"21888242871839275222246405745257275088548364400416034343698204186575808495617",
        10
    )
    .unwrap();
    pub static ref ONE: BigUint = BigUint::parse_bytes(b"1", 10).unwrap();
    pub static ref ZERO: BigUint = BigUint::parse_bytes(b"0", 10).unwrap();
}


// Field Scalar  ------------------------------------------------

impl FS {
    fn field() -> &'static BigUint {
        &BABYJUB_FIELD_UINT as &BigUint
    }
    fn field_int() -> &'static BigInt {
        &BABYJUB_FIELD_INT as &BigInt
    }
    pub fn parse(expr: &str) -> Result<Self> {
        if expr.starts_with("0x") {
            BigUint::parse_bytes(&expr.as_bytes()[2..],16)
                .map_or_else(
                || Err(Error::InvalidFormat(format!("{} is not hexadecimal",expr))),
                |v| Ok(FS(v)),
                )
        } else {
            BigUint::parse_bytes(expr.as_bytes(),10)
                .map_or_else(
                || Err(Error::InvalidFormat(format!("{} is not decimal",expr))),
                |v| Ok(FS(v)),
                )
        }
    }

    pub fn zero() -> Self {
        FS(ZERO.clone())
    }
    pub fn one() -> Self {
        FS(ONE.clone())
    }
    pub fn is_one(&self) -> bool {
        self.0.cmp(&ONE) == Ordering::Equal
    }
    pub fn is_neg(&self) -> bool {
        self.0.cmp(&BABYJUB_FIELD_UINT_NEG as &BigUint) == Ordering::Greater
    }
    pub fn format(&self, plus_sign_at_start: bool) -> String {
        if self.is_neg() {
            format!("-{}", (-self).0.to_str_radix(10))
        } else if plus_sign_at_start {
            format!("+{}", self.0.to_str_radix(10))
        } else {
            self.0.to_str_radix(10)
        }
    }
    pub fn shl(&self, rhs: &FS) -> Result<FS> {
        if let Some(self_u64) = self.0.to_u64() {
            if let Some(rhs_u64) = rhs.0.to_u64() {
                let v = BigUint::from_u64(self_u64 << rhs_u64).unwrap();
                return Ok(FS(v));
            }
        }
        Err(Error::InvalidOperation(
            "Only can shl on 64 bit values".to_string(),
        ))
    }
    pub fn shr(&self, rhs: &FS) -> Result<FS> {
        if let Some(self_u64) = self.0.to_u64() {
            if let Some(rhs_u64) = rhs.0.to_u64() {
                let v = BigUint::from_u64(self_u64 >> rhs_u64).unwrap();
                return Ok(FS(v));
            }
        }
        Err(Error::InvalidOperation(
            "Only can shr on 64 bit values".to_string(),
        ))
    }
    pub fn pow(&self, rhs: &FS) -> FS {
        FS::from(self.0.modpow(&rhs.0, FS::field()))
    }

    pub fn intdiv(&self, rhs: &FS) -> FS {
        FS::from(&self.0 / &rhs.0)
    }
}

impl PartialEq for FS {
    fn eq(&self, other: &FS) -> bool {
        self.0.eq(&other.0)
    }
}

impl PartialOrd for FS {
    fn partial_cmp(&self, other: &FS) -> Option<Ordering> {
        self.0.partial_cmp(&other.0)
    }
}

impl Eq for FS {}

impl Ord for FS {
    fn cmp(&self, other: &FS) -> Ordering {
        self.0.cmp(&other.0)
    }
}

impl From<&BigUint> for FS {
    fn from(n: &BigUint) -> Self {
        FS(n % FS::field())
    }
}

impl From<BigUint> for FS {
    fn from(n: BigUint) -> Self {
        FS::from(&n)
    }
}

impl From<u64> for FS {
    fn from(n: u64) -> Self {
        FS::from(BigUint::from_u64(n).unwrap())
    }
}


impl From<BigInt> for FS {
    fn from(n: BigInt) -> Self {
        FS::from(&n)
    }
}

impl From<&BigInt> for FS {
    fn from(n: &BigInt) -> Self {
        let v = n % (&BABYJUB_FIELD_INT as &BigInt);
        FS(v.to_biguint().unwrap())
    }
}

impl AlgZero for FS {
    fn zero() -> Self {
        FS(num_traits::Zero::zero())
    }
    fn is_zero(&self) -> bool {
        <BigUint as num_traits::Zero>::is_zero(&self.0)
    }
}

impl fmt::Debug for FS {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> std::result::Result<(), fmt::Error> {
        write!(fmt, "{}", self.format(false))
    }
}

// -&FS -> FS
impl<'a> Neg for &'a FS {
    type Output = FS;

    fn neg(self) -> FS {
        FS::from(FS::field() - &self.0)
    }
}

// &FS + &FS -> FS
impl<'a> Add<&'a FS> for &'a FS {
    type Output = FS;

    fn add(self, rhs: &'a FS) -> FS {
        FS::from(&(self.0) + &rhs.0)
    }
}

// &FS * &FS -> FS
impl<'a> Mul<&'a FS> for &'a FS {
    type Output = FS;

    fn mul(self, rhs: &'a FS) -> FS {
        FS::from(&(self.0) * &rhs.0)
    }
}

// &FS / &FS -> FS
impl<'a> Div<&'a FS> for &'a FS {
    type Output = Result<FS>;

    #[allow(clippy::suspicious_arithmetic_impl)]
    fn div(self, rhs: &'a FS) -> Result<FS> {
        let GcdResult { gcd, c1: c, .. } = extended_gcd(
            BigInt::from_biguint(num_bigint::Sign::Plus, rhs.0.clone()),
            FS::field_int().clone(),
        );

        if gcd == BigInt::one() {
            let rhs_inv = normalize(&c, FS::field_int()).to_biguint().unwrap();
            Ok(FS::from(&self.0 * rhs_inv))
        } else {
            Err(Error::InvalidOperation(format!(
                "Cannot find inv gcd={}",
                gcd.to_str_radix(10)
            )))
        }
    }
}

// FS += &FSs
impl<'a> AddAssign<&'a FS> for FS {
    // addLCNum
    fn add_assign(&mut self, rhs: &'a FS) {
        *self = (self as &Self) + rhs;
    }
}

// &FS % &FS
impl<'a> Rem<&'a FS> for &'a FS {
    type Output = Result<FS>;
    fn rem(self, rhs: &'a FS) -> Result<FS> {
        if !rhs.0.is_zero() {
            Ok(FS(&self.0 % &rhs.0))
        } else {
            Err(Error::InvalidOperation("Divison by zero".to_string()))
        }
    }
}

// &FS << &FS
impl<'a> Shl<&'a FS> for &'a FS {
    type Output = Result<FS>;
    fn shl(self, rhs: &'a FS) -> Result<FS> {
        if let Some(rhs_usize) = rhs.0.to_usize() {
            return Ok(FS::from(&self.0 << rhs_usize));
        } else {
            Err(Error::InvalidOperation(
                "Only can shl on 64 bit values".to_string(),
            ))
        }
    }
}

// &FS >> &FS
impl<'a> Shr<&'a FS> for &'a FS {
    type Output = Result<FS>;
    fn shr(self, rhs: &'a FS) -> Result<FS> {
        if let Some(rhs_usize) = rhs.0.to_usize() {
            return Ok(FS::from(&self.0 >> rhs_usize));
        } else {
            Err(Error::InvalidOperation(
                "Only can shr on 64 bit values".to_string(),
            ))
        }
    }
}

// &FS & &FS
impl<'a> BitAnd<&'a FS> for &'a FS {
    type Output = FS;
    fn bitand(self, rhs: &'a FS) -> FS {
        FS(&self.0 & &rhs.0)
    }
}

// &FS | &FS
impl<'a> BitOr<&'a FS> for &'a FS {
    type Output = FS;
    fn bitor(self, rhs: &'a FS) -> FS {
        FS::from(&self.0 | &rhs.0)
    }
}

// &FS ^ &FS
impl<'a> BitXor<&'a FS> for &'a FS {
    type Output = FS;
    fn bitxor(self, rhs: &'a FS) -> FS {
        FS::from(&self.0 ^ &rhs.0)
    }
}

// helpers --------------------------------------------------------------------

pub struct GcdResult {
    /// Greatest common divisor.
    pub gcd: BigInt,
    /// Coefficients such that: gcd(a, b) = c1*a + c2*b
    pub c1: BigInt,
    pub c2: BigInt,
}

/// Taken from unknown source, re-check
/// Calculate greatest common divisor and the corresponding coefficients.
#[allow(clippy::many_single_char_names)]
pub fn extended_gcd(a: BigInt, b: BigInt) -> GcdResult {
    // Euclid's extended algorithm
    let (mut s, mut old_s) = (BigInt::zero(), BigInt::one());
    let (mut t, mut old_t) = (BigInt::one(), BigInt::zero());
    let (mut r, mut old_r) = (b, a);

    while r != BigInt::zero() {
        let quotient = &old_r / &r;
        old_r -= &quotient * &r;
        std::mem::swap(&mut old_r, &mut r);
        old_s -= &quotient * &s;
        std::mem::swap(&mut old_s, &mut s);
        old_t -= quotient * &t;
        std::mem::swap(&mut old_t, &mut t);
    }

    let _quotients = (t, s); // == (a, b) / gcd

    GcdResult {
        gcd: old_r,
        c1: old_s,
        c2: old_t,
    }
}

/// Find the standard representation of a (mod n).
pub fn normalize(a: &BigInt, n: &BigInt) -> BigInt {
    let a = a % n;
    match a.cmp(&BigInt::zero()) {
        Ordering::Less => a + n,
        _ => a,
    }
}

// test --------------------------------------------------------------------

#[cfg(test)]
mod test {
    use super::super::Result;
    use super::*;
    use num_bigint::ToBigUint;

    fn u32_to_fs(n: u32) -> FS {
        FS::from(n.to_biguint().unwrap())
    }

    #[test]
    fn test_fs_fs_add_mul() {
        let one = FS::one();
        let two = &one + &one;
        let three = &(&one + &one) + &one;
        let six = &three * &two;

        assert_eq!("1", format!("{:?}", one));
        assert_eq!("2", format!("{:?}", two));
        assert_eq!("3", format!("{:?}", three));
        assert_eq!("6", format!("{:?}", six));
    }

    #[test]
    fn test_fs_neg() {
        let one = &FS::one();
        let minus_one = &-one;
        assert_eq!("-1", format!("{:?}", minus_one));
        let minus_two = &(minus_one + minus_one);
        assert_eq!("2", format!("{:?}", -minus_two));
    }

    #[test]
    fn test_fs_addassig() {
        let one = &FS::one();
        let mut three = one + one;
        three += one;

        assert_eq!("3", format!("{:?}", three));
    }

    #[test]
    fn test_fs_mod() -> Result<()> {
        let one = &u32_to_fs(1012) % &u32_to_fs(1000);
        assert_eq!("12", format!("{:?}", one?));
        Ok(())
    }

    #[test]
    fn test_fs_shl() -> Result<()> {
        let forty = &u32_to_fs(10) << &u32_to_fs(2);
        assert_eq!("40", format!("{:?}", forty?));

        Ok(())
    }

    #[test]
    fn test_fs_shr() -> Result<()> {
        let twenty = &u32_to_fs(40) >> &u32_to_fs(1);
        assert_eq!("20", format!("{:?}", twenty?));

        Ok(())
    }

    #[test]
    fn test_div() -> Result<()> {
        let div = &u32_to_fs(1) / &u32_to_fs(2);
        let mul = &u32_to_fs(6) * &div?;
        assert_eq!("3", format!("{:?}", mul));

        Ok(())
    }

}
