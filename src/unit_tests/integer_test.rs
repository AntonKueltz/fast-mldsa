use crate::integer::*;

use shake::{ExtendableOutput, Shake128, Update, XofReader};

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

#[test]
#[ignore]
fn test_field_ops_accumulated() {
    let expected = [
        0xf9, 0x30, 0x66, 0x34, 0x17, 0x27, 0x81, 0x56, 0xab, 0x05, 0xd9, 0x40, 0x29, 0x4a, 0x77,
        0x21, 0x0a, 0x80, 0x9c, 0x92, 0x4d, 0x8a, 0xb6, 0x3e, 0xc7, 0x2f, 0x45, 0x26, 0x24, 0x76,
        0x02, 0xc7,
    ];
    let mut shaker = Shake128::default();

    for r in 0..8380417 {
        let (hi88, lo88) = decompose(r, 95232);
        let (hi32, lo32) = decompose(r, 261888);
        let (r1, r0) = pow2_round(r);
        let rmodq = if r > (Q as i32 - 1) / 2 {
            r - Q as i32
        } else {
            r
        };

        let vals = [
            rmodq,
            infinity_norm(rmodq),
            r1,
            cond_add_q(reduce_q(r0)),
            hi88,
            use_hint(r, 1, 95232),
            lo88,
            infinity_norm(lo88),
            hi32,
            use_hint(r, 1, 261888),
            lo32,
            infinity_norm(lo32),
        ];

        for v in vals {
            shaker.update(v.to_string().as_bytes());
            shaker.update(b"\n");
        }
    }

    let mut actual = [0u8; 32];
    shaker.finalize_xof().read(&mut actual);

    assert_eq!(actual, expected);
}
