use crate::libraries::big_num::{U128, U256, U512};

pub trait MulDiv<RHS = Self> {
    type Output;

    fn mul_div_floor(self, num: RHS, denom: RHS) -> Option<Self::Output>;

    fn mul_div_ceil(self, num: RHS, denom: RHS) -> Option<Self::Output>;

    fn to_underflow_u64(self) -> u64;
}

pub trait Upcast256 {
    fn as_u256(self) -> U256;
}
impl Upcast256 for U128 {
    fn as_u256(self) -> U256 {
        U256([self.0[0], self.0[1], 0, 0])
    }
}

pub trait Downcast256 {
    fn as_u128(self) -> U128;
}
impl Downcast256 for U256 {
    fn as_u128(self) -> U128 {
        U128([self.0[0], self.0[1]])
    }
}

pub trait Upcast512 {
    fn as_u512(self) -> U512;
}
impl Upcast512 for U256 {
    fn as_u512(self) -> U512 {
        U512([self.0[0], self.0[1], self.0[2], self.0[3], 0, 0, 0, 0])
    }
}

pub trait Downcast512 {
    fn as_u256(self) -> U256;
}
impl Downcast512 for U512 {
    fn as_u256(self) -> U256 {
        U256([self.0[0], self.0[1], self.0[2], self.0[3]])
    }
}

impl MulDiv for u64 {
    type Output = u64;

    fn mul_div_floor(self, num: Self, denom: Self) -> Option<Self::Output> {
        if denom == 0 {
            return None;
        }
        let r = (U128::from(self) * U128::from(num)) / U128::from(denom);
        if r > U128::from(u64::MAX) {
            None
        } else {
            Some(r.as_u64())
        }
    }

    fn mul_div_ceil(self, num: Self, denom: Self) -> Option<Self::Output> {
        if denom == 0 {
            return None;
        }
        let r = (U128::from(self) * U128::from(num) + U128::from(denom - 1)) / U128::from(denom);
        if r > U128::from(u64::MAX) {
            None
        } else {
            Some(r.as_u64())
        }
    }

    fn to_underflow_u64(self) -> u64 {
        self
    }
}

impl MulDiv for U128 {
    type Output = U128;

    fn mul_div_floor(self, num: Self, denom: Self) -> Option<Self::Output> {
        if denom.is_zero() {
            return None;
        }
        let (prod, overflow) = self.overflowing_mul(num);
        if !overflow {
            return Some(prod / denom);
        }
        let r = ((self.as_u256()) * (num.as_u256())) / (denom.as_u256());
        if r > U128::MAX.as_u256() {
            None
        } else {
            Some(r.as_u128())
        }
    }

    fn mul_div_ceil(self, num: Self, denom: Self) -> Option<Self::Output> {
        if denom.is_zero() {
            return None;
        }
        let (prod, overflow) = self.overflowing_mul(num);
        if !overflow {
            if let Some(numerator) = prod.checked_add(denom - 1) {
                return Some(numerator / denom);
            }
        }
        let r = (self.as_u256() * num.as_u256() + (denom - 1).as_u256()) / denom.as_u256();
        if r > U128::MAX.as_u256() {
            None
        } else {
            Some(r.as_u128())
        }
    }

    fn to_underflow_u64(self) -> u64 {
        if self < U128::from(u64::MAX) {
            self.as_u64()
        } else {
            0
        }
    }
}

impl MulDiv for U256 {
    type Output = U256;

    fn mul_div_floor(self, num: Self, denom: Self) -> Option<Self::Output> {
        if denom.is_zero() {
            return None;
        }
        let (prod, overflow) = self.overflowing_mul(num);
        if !overflow {
            return Some(prod / denom);
        }
        let r = (self.as_u512() * num.as_u512()) / denom.as_u512();
        if r > U256::MAX.as_u512() {
            None
        } else {
            Some(r.as_u256())
        }
    }

    fn mul_div_ceil(self, num: Self, denom: Self) -> Option<Self::Output> {
        if denom.is_zero() {
            return None;
        }
        let (prod, overflow) = self.overflowing_mul(num);
        if !overflow {
            if let Some(numerator) = prod.checked_add(denom - 1) {
                return Some(numerator / denom);
            }
        }
        let r = (self.as_u512() * num.as_u512() + (denom - 1).as_u512()) / denom.as_u512();
        if r > U256::MAX.as_u512() {
            None
        } else {
            Some(r.as_u256())
        }
    }

    fn to_underflow_u64(self) -> u64 {
        if self < U256::from(u64::MAX) {
            self.as_u64()
        } else {
            0
        }
    }
}

#[inline]
pub fn mul_pow2_div_ceil(x: U256, shift: u32, denom: U256) -> Option<U256> {
    if denom.is_zero() {
        return None;
    }
    if x.leading_zeros() >= shift {
        let shifted = x << (shift as usize);
        if let Some(numerator) = shifted.checked_add(denom - U256::from(1u8)) {
            return Some(numerator / denom);
        }
    }
    x.mul_div_ceil(U256::from(1u128) << (shift as usize), denom)
}

#[inline]
pub fn mul_pow2_div_floor(x: U256, shift: u32, denom: U256) -> Option<U256> {
    if denom.is_zero() {
        return None;
    }
    if x.leading_zeros() >= shift {
        let shifted = x << (shift as usize);
        return Some(shifted / denom);
    }
    x.mul_div_floor(U256::from(1u128) << (shift as usize), denom)
}

#[cfg(test)]
mod muldiv_u64_tests {
    use super::*;

    use quickcheck::{quickcheck, Arbitrary, Gen};

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    struct NonZero(u64);

    impl Arbitrary for NonZero {
        fn arbitrary<G: Gen>(g: &mut G) -> Self {
            loop {
                let v = u64::arbitrary(g);
                if v != 0 {
                    return NonZero(v);
                }
            }
        }
    }

    quickcheck! {
        fn scale_floor(val: u64, num: u64, den: NonZero) -> bool {
            let res = val.mul_div_floor(num, den.0);

            let expected = (U128::from(val) * U128::from(num)) / U128::from(den.0);

            if expected > U128::from(u64::MAX) {
                res.is_none()
            } else {
                res == Some(expected.as_u64())
            }
        }
    }

    quickcheck! {
        fn scale_ceil(val: u64, num: u64, den: NonZero) -> bool {
            let res = val.mul_div_ceil(num, den.0);

            let mut expected = (U128::from(val) * U128::from(num)) / U128::from(den.0);
            let expected_rem = (U128::from(val) * U128::from(num)) % U128::from(den.0);

            if expected_rem != U128::default() {
                expected += U128::from(1)
            }

            if expected > U128::from(u64::MAX) {
                res.is_none()
            } else {
                res == Some(expected.as_u64())
            }
        }
    }
}

#[cfg(test)]
mod muldiv_u128_tests {
    use super::*;

    use quickcheck::{quickcheck, Arbitrary, Gen};

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    struct NonZero(U128);

    impl Arbitrary for NonZero {
        fn arbitrary<G: Gen>(g: &mut G) -> Self {
            loop {
                let v = U128::from(u128::arbitrary(g));
                if v != U128::default() {
                    return NonZero(v);
                }
            }
        }
    }

    impl Arbitrary for U128 {
        fn arbitrary<G: Gen>(g: &mut G) -> Self {
            loop {
                let v = U128::from(u128::arbitrary(g));
                if v != U128::default() {
                    return v;
                }
            }
        }
    }

    quickcheck! {
        fn scale_floor(val: U128, num: U128, den: NonZero) -> bool {
            let res = val.mul_div_floor(num, den.0);

            let expected = ((val.as_u256()) * (num.as_u256())) / (den.0.as_u256());

            if expected > U128::MAX.as_u256() {
                res.is_none()
            } else {
                res == Some(expected.as_u128())
            }
        }
    }

    quickcheck! {
        fn scale_ceil(val: U128, num: U128, den: NonZero) -> bool {
            let res = val.mul_div_ceil(num, den.0);

            let mut expected = ((val.as_u256()) * (num.as_u256())) / (den.0.as_u256());
            let expected_rem = ((val.as_u256()) * (num.as_u256())) % (den.0.as_u256());

            if expected_rem != U256::default() {
                expected += U256::from(1)
            }

            if expected > U128::MAX.as_u256() {
                res.is_none()
            } else {
                res == Some(expected.as_u128())
            }
        }
    }
}
