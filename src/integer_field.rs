const Q: i64 = 8380417;
const QINV: i64 = 58728449;

const BIT_22: i32 = 1 << 22;

pub(crate) fn mont_q(n: i64) -> i32 {
    let m = ((n as i32) as i64 * QINV) as i32;
    let t = n - (m as i64) * Q;
    (t >> 32) as i32
}

pub(crate) fn norm_q(n: i32) -> i32 {
    cond_add_q(reduce_q(n))
}

pub(crate) fn reduce_q(n: i32) -> i32 {
    let t = (n + BIT_22) >> 23;
    n - t * (Q as i32)
}

pub(crate) fn cond_add_q(n: i32) -> i32 {
    n + ((n >> 31) & (Q as i32))
}
