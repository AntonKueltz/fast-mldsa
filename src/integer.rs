const Q: i64 = 8380417;
const QINV: i64 = 58728449;

const BIT_21: i32 = 1 << 21;
const BIT_22: i32 = 1 << 22;
const BIT_23: i32 = 1 << 23;

const GAMMA2_32: usize = 261888;
const GAMMA2_88: usize = 95232;

pub(crate) const fn mont_q(n: i64) -> i32 {
    let m = ((n as i32) as i64 * QINV) as i32;
    let t = n - (m as i64) * Q;
    (t >> 32) as i32
}

pub(crate) fn reduce_q(n: i32) -> i32 {
    let t = (n + BIT_22) >> 23;
    n - t * (Q as i32)
}

pub(crate) fn cond_add_q(n: i32) -> i32 {
    n + ((n >> 31) & (Q as i32))
}

pub(crate) fn infinity_norm(n: i32) -> i32 {
    let t = n >> 31;
    n - (t & (n << 1))
}

pub(crate) fn pow2_round(n: i32) -> (i32, i32) {
    let hi = (n + 0xfff) >> 13;
    let lo = n - (hi << 13);
    (hi, lo)
}

pub(crate) fn decompose(a: i32, gamma: usize) -> (i32, i32) {
    let mut a_hi = (a + 127) >> 7;

    if gamma == GAMMA2_32 {
        a_hi = ((a_hi * 1025 + BIT_21) >> 22) & 0xf;
    } else if gamma == GAMMA2_88 {
        a_hi = (a_hi * 11275 + BIT_23) >> 24;
        a_hi ^= ((43 - a_hi) >> 31) & a_hi;
    } else {
        unreachable!()
    }

    let mut a_lo = a - a_hi * (gamma << 1) as i32;
    a_lo -= ((((Q as i32 - 1) >> 1) - a_lo) >> 31) & Q as i32;

    (a_hi, a_lo)
}

pub(crate) fn use_hint(a: i32, hint: i32, gamma: usize) -> i32 {
    let (a1, a0) = decompose(a, gamma);

    if hint == 0 {
        return a1;
    }

    if gamma == GAMMA2_32 {
        if a0 > 0 {
            (a1 + 1) & 0xf
        } else {
            (a1 - 1) & 0xf
        }
    } else if gamma == GAMMA2_88 {
        if a0 > 0 {
            if a1 == 43 { 0 } else { a1 + 1 }
        } else {
            if a1 == 0 { 43 } else { a1 - 1 }
        }
    } else {
        unreachable!()
    }
}

#[cfg(test)]
#[path = "unit_tests/integer_test.rs"]
mod integer_test;

#[cfg(feature = "ct-testing")]
pub mod ct_exports {
    pub const Q: i32 = super::Q as i32;
    pub const GAMMA2_32: usize = super::GAMMA2_32;
    pub const GAMMA2_88: usize = super::GAMMA2_88;

    #[inline(never)]
    pub fn mont_q(n: i64) -> i32 {
        super::mont_q(n)
    }
    #[inline(never)]
    pub fn reduce_q(n: i32) -> i32 {
        super::reduce_q(n)
    }
    #[inline(never)]
    pub fn cond_add_q(n: i32) -> i32 {
        super::cond_add_q(n)
    }
    #[inline(never)]
    pub fn infinity_norm(n: i32) -> i32 {
        super::infinity_norm(n)
    }
    #[inline(never)]
    pub fn pow2_round(n: i32) -> (i32, i32) {
        super::pow2_round(n)
    }
    #[inline(never)]
    pub fn decompose(a: i32, gamma: usize) -> (i32, i32) {
        super::decompose(a, gamma)
    }
    #[inline(never)]
    pub fn use_hint(a: i32, hint: i32, gamma: usize) -> i32 {
        super::use_hint(a, hint, gamma)
    }
}
