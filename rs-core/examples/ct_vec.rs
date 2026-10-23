mod common;

use core::array::from_fn;
use std::cell::RefCell;
use std::hint::black_box;

use dudect_bencher::rand::{Rng, RngExt};
use dudect_bencher::{BenchRng, CtRunner, ctbench_main};

use common::*;
use ml_dsa::ct::poly::{N, Poly, Q};
use ml_dsa::ct::vector::{self, V};

const SAMPLES_VEC: usize = 2_000;
const M: i32 = (Q - 1) / 2;
const LIMIT: i32 = (1 << 19) - 196;
const ETA: i32 = 4;

// ---------- expand_mask: secret rho'' -> SHAKE256 -> unpack ----------
fn random_rho(r: &mut BenchRng) -> [u8; 64] {
    let mut b = [0u8; 64];
    r.fill_bytes(&mut b);
    b
}

fn expand_mask_case<const D: usize>(
    runner: &mut CtRunner,
    rng: &mut BenchRng,
    gamma: usize,
    v_size: usize,
) {
    let edges = [[0u8; 64], [0xffu8; 64], random_rho(rng)];
    run_state_n(
        SAMPLES_VEC,
        runner,
        rng,
        |r| {
            (
                edges[r.random_range(0..edges.len())],
                D * r.random_range(0..16),
            )
        },
        |r| (random_rho(r), D * r.random_range(0..16)),
        |(rho, mu)| {
            black_box(vector::expand_mask::<D>(rho, *mu, gamma, v_size));
        },
    );
}
fn expand_mask_65(r: &mut CtRunner, g: &mut BenchRng) {
    expand_mask_case::<5>(r, g, 19, 640)
}
fn expand_mask_44(r: &mut CtRunner, g: &mut BenchRng) {
    expand_mask_case::<4>(r, g, 17, 576)
}

// ---------- norm check ----------
fn vec_norm_pass(runner: &mut CtRunner, rng: &mut BenchRng) {
    let edges = vec_edges::<5>(rng, &[0, LIMIT - 1, -(LIMIT - 1)], -(LIMIT - 1), LIMIT - 1);
    run_state_n(
        SAMPLES_VEC,
        runner,
        rng,
        |r| pick_v(&edges, r),
        |r| vec_random::<5>(r, -(LIMIT - 1), LIMIT - 1),
        |v| {
            black_box(v.norm_within_limit(LIMIT));
        },
    );
}

fn vec_with_violation<const D: usize>(r: &mut BenchRng, p: usize, k: usize) -> V<D> {
    V::from_polys(from_fn(|i| {
        let mut c = random_coeffs(r, -(LIMIT - 1), LIMIT - 1);
        if i == p {
            c[k] = LIMIT;
        }
        Poly::from_coeffs(c)
    }))
}

fn vec_norm_violation_position(runner: &mut CtRunner, rng: &mut BenchRng) {
    run_state_n(
        SAMPLES_VEC,
        runner,
        rng,
        |r| vec_with_violation::<5>(r, 0, 0),
        |r| {
            let (p, k) = (r.random_range(0..5), r.random_range(0..N));
            vec_with_violation::<5>(r, p, k)
        },
        |v| {
            black_box(v.norm_within_limit(LIMIT));
        },
    );
}

// ---------- c*s1 pipeline: z = y + inv_ntt(ntt(s) o c_hat) ----------
type CsState = (RefCell<V<5>>, Poly, RefCell<V<5>>);

fn cs_state(s: V<5>, r: &mut BenchRng) -> CsState {
    (
        RefCell::new(s),
        poly_random(r, -M, M),
        RefCell::new(vec_random::<5>(r, -(1 << 19) + 1, 1 << 19)),
    )
}

fn cs_op((s, c, y): &CsState) {
    let mut s = s.borrow_mut();
    s.ntt();
    let mut cs = s.mul_poly(c);
    cs.inv_ntt();
    cs.reduce();
    y.borrow_mut().add_acc(&cs);
}

fn cs_pipeline(runner: &mut CtRunner, rng: &mut BenchRng) {
    let edges = vec_edges::<5>(rng, &[-ETA, ETA, 0], -ETA, ETA);
    run_state_n(
        SAMPLES_VEC,
        runner,
        rng,
        |r| cs_state(pick_v(&edges, r), r),
        |r| cs_state(vec_random::<5>(r, -ETA, ETA), r),
        cs_op,
    );
}

fn null_control(runner: &mut CtRunner, rng: &mut BenchRng) {
    run_state_n(
        SAMPLES_VEC,
        runner,
        rng,
        |r| cs_state(vec_random::<5>(r, -ETA, ETA), r),
        |r| cs_state(vec_random::<5>(r, -ETA, ETA), r),
        cs_op,
    );
}

ctbench_main!(
    expand_mask_65,
    expand_mask_44,
    vec_norm_pass,
    vec_norm_violation_position,
    cs_pipeline,
    null_control
);
