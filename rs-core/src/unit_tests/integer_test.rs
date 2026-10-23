use crate::integer::*;

const N: i32 = 6_010_000;

#[test]
fn test_reduce_q() {
    assert_eq!(reduce_q(N), -2370417);
}

#[test]
fn test_infinity_norm() {
    assert_eq!(infinity_norm(reduce_q(N)), 2370417);
}

#[test]
fn test_pow2_round() {
    let (r1, r0) = pow2_round(N);
    assert_eq!(r1, 734);
    assert_eq!(cond_add_q(reduce_q(r0)), 8377489);
}

#[test]
fn test_decompose() {
    let (hi, lo) = decompose(N, 95232);
    assert_eq!(hi, 32);
    assert_eq!(lo, -84848);
    assert_eq!(infinity_norm(lo), 84848);

    let (hi, lo) = decompose(N, 261888);
    assert_eq!(hi, 11);
    assert_eq!(lo, 248464);
    assert_eq!(infinity_norm(lo), 248464);
}

#[test]
fn test_use_hint() {
    assert_eq!(use_hint(N, 0, 95232), 32);
    assert_eq!(use_hint(N, 1, 95232), 31);

    assert_eq!(use_hint(N, 0, 261888), 11);
    assert_eq!(use_hint(N, 1, 261888), 12);
}
