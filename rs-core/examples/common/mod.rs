#![allow(dead_code)]

use core::array::from_fn;

use dudect_bencher::rand::{Rng, RngExt};
use dudect_bencher::{BenchRng, Class, CtRunner};

use ml_dsa::ct::poly::{N, Poly};
use ml_dsa::ct::vector::V;

pub const SAMPLES: usize = 5_000;

pub fn zero() -> Poly {
    Poly::from_coeffs([0; N])
}

pub fn random_coeffs(r: &mut BenchRng, lo: i32, hi: i32) -> [i32; N] {
    let mut c = [0i32; N];
    for x in c.iter_mut() {
        *x = r.random_range(lo..=hi);
    }
    c
}

pub fn poly_random(r: &mut BenchRng, lo: i32, hi: i32) -> Poly {
    Poly::from_coeffs(random_coeffs(r, lo, hi))
}

pub fn edge_set(rng: &mut BenchRng, consts: &[i32], lo: i32, hi: i32) -> Vec<Poly> {
    let mut v: Vec<Poly> = consts.iter().map(|&c| Poly::from_coeffs([c; N])).collect();
    v.push(poly_random(rng, lo, hi));
    v
}

pub fn pick(edges: &[Poly], r: &mut BenchRng) -> Poly {
    edges[r.random_range(0..edges.len())].clone()
}

pub fn raw_bytes(r: &mut BenchRng, len: usize) -> Vec<u8> {
    let mut b = vec![0u8; len];
    r.fill_bytes(&mut b);
    b
}

pub fn vec_const<const D: usize>(c: i32) -> V<D> {
    V::from_polys(from_fn(|_| Poly::from_coeffs([c; N])))
}

pub fn vec_random<const D: usize>(r: &mut BenchRng, lo: i32, hi: i32) -> V<D> {
    V::from_polys(from_fn(|_| poly_random(r, lo, hi)))
}

pub fn vec_edges<const D: usize>(
    rng: &mut BenchRng,
    consts: &[i32],
    lo: i32,
    hi: i32,
) -> Vec<V<D>> {
    let mut v: Vec<V<D>> = consts.iter().map(|&c| vec_const::<D>(c)).collect();
    v.push(vec_random::<D>(rng, lo, hi));
    v
}

pub fn pick_v<const D: usize>(edges: &[V<D>], r: &mut BenchRng) -> V<D> {
    edges[r.random_range(0..edges.len())].clone()
}

pub fn run_state_n<S>(
    samples: usize,
    runner: &mut CtRunner,
    rng: &mut BenchRng,
    mut left: impl FnMut(&mut BenchRng) -> S,
    mut right: impl FnMut(&mut BenchRng) -> S,
    op: impl Fn(&S),
) {
    let mut states = Vec::with_capacity(samples);
    let mut classes = Vec::with_capacity(samples);

    for _ in 0..samples {
        if rng.random::<bool>() {
            states.push(left(rng));
            classes.push(Class::Left);
        } else {
            states.push(right(rng));
            classes.push(Class::Right);
        }
    }

    for (s, class) in states.iter().zip(classes) {
        runner.run_one(class, || op(s));
    }
}

pub fn run_state<S>(
    runner: &mut CtRunner,
    rng: &mut BenchRng,
    left: impl FnMut(&mut BenchRng) -> S,
    right: impl FnMut(&mut BenchRng) -> S,
    op: impl Fn(&S),
) {
    run_state_n(SAMPLES, runner, rng, left, right, op)
}
