mod common;

use std::cell::RefCell;
use std::hint::black_box;

use dudect_bencher::rand::RngExt;
use dudect_bencher::{BenchRng, CtRunner, ctbench_main};

use common::*;
use ml_dsa::ct::integer as arith;
use ml_dsa::ct::poly::{self, N, Poly, Q};

const M: i32 = (Q - 1) / 2;
const RMAX: i32 = i32::MAX - (1 << 22);
const LIMIT: i32 = (1 << 19) - 196;

// ---------- generators ----------
fn encode(p: &Poly, d: usize) -> Vec<u8> {
    let mut b = vec![0u8; d << 5];
    p.to_bytes(&mut b, d);
    b
}

// ---------- harness ----------
fn edges_vs_random<S>(
    runner: &mut CtRunner,
    rng: &mut BenchRng,
    consts: &[i32],
    lo: i32,
    hi: i32,
    build: impl Fn(Poly) -> S,
    op: impl Fn(&S),
) {
    let edges = edge_set(rng, consts, lo, hi);
    run_state(
        runner,
        rng,
        |r| build(pick(&edges, r)),
        |r| build(poly_random(r, lo, hi)),
        op,
    );
}

// ---------- arithmetic ----------
fn fwd_ntt(runner: &mut CtRunner, rng: &mut BenchRng) {
    edges_vs_random(runner, rng, &[-M, M, 0], -M, M, RefCell::new, |s| {
        s.borrow_mut().ntt()
    });
}

fn inv_ntt(runner: &mut CtRunner, rng: &mut BenchRng) {
    edges_vs_random(runner, rng, &[-M, M, 0], -M, M, RefCell::new, |s| {
        s.borrow_mut().inv_ntt()
    });
}

fn reduce(runner: &mut CtRunner, rng: &mut BenchRng) {
    edges_vs_random(
        runner,
        rng,
        &[-RMAX, RMAX, 0],
        -RMAX,
        RMAX,
        RefCell::new,
        |s| s.borrow_mut().reduce(),
    );
}

fn cond_add(runner: &mut CtRunner, rng: &mut BenchRng) {
    edges_vs_random(
        runner,
        rng,
        &[-(Q - 1), Q - 1, 0, -1],
        -(Q - 1),
        Q - 1,
        RefCell::new,
        |s| s.borrow_mut().cond_add(),
    );
}

fn mul_acc(runner: &mut CtRunner, rng: &mut BenchRng) {
    let edges = edge_set(rng, &[-M, M, 0], -M, M);
    run_state(
        runner,
        rng,
        |r| {
            (
                RefCell::new(poly_random(r, -M, M)),
                pick(&edges, r),
                pick(&edges, r),
            )
        },
        |r| {
            (
                RefCell::new(poly_random(r, -M, M)),
                poly_random(r, -M, M),
                poly_random(r, -M, M),
            )
        },
        |(acc, l, rr)| acc.borrow_mut().mul_acc(l, rr),
    );
}

fn pow2_round(runner: &mut CtRunner, rng: &mut BenchRng) {
    edges_vs_random(
        runner,
        rng,
        &[0, Q - 1, 0xfff, 0x1000],
        0,
        Q - 1,
        |p| (p, RefCell::new(zero()), RefCell::new(zero())),
        |(p, hi, lo)| p.pow2_round(&mut hi.borrow_mut(), &mut lo.borrow_mut()),
    );
}

fn decompose_case(runner: &mut CtRunner, rng: &mut BenchRng, gamma: usize) {
    edges_vs_random(
        runner,
        rng,
        &[0, Q - 1, (Q - 1) / 2],
        0,
        Q - 1,
        |p| (p, RefCell::new(zero()), RefCell::new(zero())),
        |(p, hi, lo)| p.decompose(&mut hi.borrow_mut(), &mut lo.borrow_mut(), gamma),
    );
}
fn decompose_32(r: &mut CtRunner, g: &mut BenchRng) {
    decompose_case(r, g, arith::GAMMA2_32)
}
fn decompose_88(r: &mut CtRunner, g: &mut BenchRng) {
    decompose_case(r, g, arith::GAMMA2_88)
}

fn make_hint_case(runner: &mut CtRunner, rng: &mut BenchRng, gamma: usize, nhi: i32) {
    let g = gamma as i32;
    let lo_edges = edge_set(
        rng,
        &[-2 * g, 2 * g, 0, g, g + 1, -g, -g - 1],
        -2 * g,
        2 * g,
    );
    let hi_edges = edge_set(rng, &[0, nhi - 1], 0, nhi - 1);
    run_state(
        runner,
        rng,
        |r| (RefCell::new(zero()), pick(&lo_edges, r), pick(&hi_edges, r)),
        |r| {
            (
                RefCell::new(zero()),
                poly_random(r, -2 * g, 2 * g),
                poly_random(r, 0, nhi - 1),
            )
        },
        |(out, lo, hi)| {
            black_box(out.borrow_mut().make_hint(lo, hi, g));
        },
    );
}
fn make_hint_32(r: &mut CtRunner, g: &mut BenchRng) {
    make_hint_case(r, g, arith::GAMMA2_32, 16)
}
fn make_hint_88(r: &mut CtRunner, g: &mut BenchRng) {
    make_hint_case(r, g, arith::GAMMA2_88, 44)
}

// ---------- norm check ----------
fn norm_pass(runner: &mut CtRunner, rng: &mut BenchRng) {
    edges_vs_random(
        runner,
        rng,
        &[0, LIMIT - 1, -(LIMIT - 1)],
        -(LIMIT - 1),
        LIMIT - 1,
        |p| p,
        |p| {
            black_box(p.norm_within_limit(LIMIT));
        },
    );
}

/// Expected to FAIL: norm_within_limit returns at the first violating coefficient.
/// Left: violation at index 0. Right: violation at a random index.
fn norm_violation_position(runner: &mut CtRunner, rng: &mut BenchRng) {
    let make = |r: &mut BenchRng, k: usize| {
        let mut c = random_coeffs(r, -(LIMIT - 1), LIMIT - 1);
        c[k] = LIMIT;
        Poly::from_coeffs(c)
    };
    run_state(
        runner,
        rng,
        |r| make(r, 0),
        |r| {
            let k = r.random_range(0..N);
            make(r, k)
        },
        |p| {
            black_box(p.norm_within_limit(LIMIT));
        },
    );
}

// ---------- unpacking ----------
fn unpack_raw(runner: &mut CtRunner, rng: &mut BenchRng, d: usize) {
    let len = d << 5;
    let edges = vec![vec![0x00u8; len], vec![0xffu8; len], raw_bytes(rng, len)];
    run_state(
        runner,
        rng,
        |r| edges[r.random_range(0..edges.len())].clone(),
        |r| raw_bytes(r, len),
        |b| {
            black_box(poly::from_bytes(b, d));
        },
    );
}
fn from_bytes_20(r: &mut CtRunner, g: &mut BenchRng) {
    unpack_raw(r, g, 20)
}
fn from_bytes_18(r: &mut CtRunner, g: &mut BenchRng) {
    unpack_raw(r, g, 18)
}
fn from_bytes_13(r: &mut CtRunner, g: &mut BenchRng) {
    unpack_raw(r, g, 13)
}

fn unpack_eta(runner: &mut CtRunner, rng: &mut BenchRng, eta: i32, eta_bits: usize) {
    let edges: Vec<Vec<u8>> = edge_set(rng, &[-eta, eta, 0], -eta, eta)
        .iter()
        .map(|p| encode(p, eta_bits))
        .collect();
    run_state(
        runner,
        rng,
        |r| edges[r.random_range(0..edges.len())].clone(),
        |r| encode(&poly_random(r, -eta, eta), eta_bits),
        |b| {
            let _ = black_box(poly::from_bytes_eta(b, eta_bits));
        },
    );
}
fn from_bytes_eta3(r: &mut CtRunner, g: &mut BenchRng) {
    unpack_eta(r, g, 2, 3)
}
fn from_bytes_eta4(r: &mut CtRunner, g: &mut BenchRng) {
    unpack_eta(r, g, 4, 4)
}

// ---------- packing (keygen's skEncode) ----------
fn pack(runner: &mut CtRunner, rng: &mut BenchRng, d: usize, lo: i32, hi: i32) {
    edges_vs_random(
        runner,
        rng,
        &[lo, hi, (lo + hi) / 2],
        lo,
        hi,
        |p| (p, RefCell::new(vec![0u8; d << 5])),
        |(p, buf)| p.to_bytes(&mut buf.borrow_mut(), d),
    );
}
fn to_bytes_13(r: &mut CtRunner, g: &mut BenchRng) {
    pack(r, g, 13, -4095, 4096)
} // t0
fn to_bytes_eta3(r: &mut CtRunner, g: &mut BenchRng) {
    pack(r, g, 3, -2, 2)
}
fn to_bytes_eta4(r: &mut CtRunner, g: &mut BenchRng) {
    pack(r, g, 4, -4, 4)
}

// ---------- control ----------
fn null_control(runner: &mut CtRunner, rng: &mut BenchRng) {
    run_state(
        runner,
        rng,
        |r| RefCell::new(poly_random(r, -M, M)),
        |r| RefCell::new(poly_random(r, -M, M)),
        |s| s.borrow_mut().ntt(),
    );
}

ctbench_main!(
    fwd_ntt,
    inv_ntt,
    reduce,
    cond_add,
    mul_acc,
    pow2_round,
    decompose_32,
    decompose_88,
    make_hint_32,
    make_hint_88,
    norm_pass,
    norm_violation_position,
    from_bytes_20,
    from_bytes_18,
    from_bytes_13,
    from_bytes_eta3,
    from_bytes_eta4,
    to_bytes_13,
    to_bytes_eta3,
    to_bytes_eta4,
    null_control
);
