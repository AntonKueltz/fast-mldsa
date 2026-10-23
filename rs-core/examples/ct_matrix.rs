mod common;

use std::cell::RefCell;
use std::hint::black_box;

use dudect_bencher::rand::Rng;
use dudect_bencher::{BenchRng, CtRunner, ctbench_main};

use common::*;
use ml_dsa::ct::matrix::Mat;
use ml_dsa::ct::vector::V;

const K: usize = 6;
const L: usize = 5;
const SAMPLES_MAT: usize = 1_000;

fn random_rho(r: &mut BenchRng) -> [u8; 32] {
    let mut b = [0u8; 32];
    r.fill_bytes(&mut b);
    b
}

fn pipeline(a: &Mat<K, L>, y: &RefCell<V<L>>) {
    let mut y = y.borrow_mut();
    y.ntt();
    let mut w = a.mul_vec(&y);
    w.reduce();
    w.inv_ntt();
    w.cond_add();
    black_box(w);
}

fn mul_vec_case(runner: &mut CtRunner, rng: &mut BenchRng, lo: i32, hi: i32) {
    let a = Mat::<K, L>::expand_a(&random_rho(rng));
    let edges = vec_edges::<L>(rng, &[lo, hi, 0], lo, hi);
    run_state_n(
        SAMPLES_MAT,
        runner,
        rng,
        |r| RefCell::new(pick_v(&edges, r)),
        |r| RefCell::new(vec_random::<L>(r, lo, hi)),
        |y| pipeline(&a, y),
    );
}

fn mul_vec_y(r: &mut CtRunner, g: &mut BenchRng) {
    mul_vec_case(r, g, -(1 << 19) + 1, 1 << 19)
}
fn mul_vec_s1(r: &mut CtRunner, g: &mut BenchRng) {
    mul_vec_case(r, g, -4, 4)
}

fn null_control(runner: &mut CtRunner, rng: &mut BenchRng) {
    let a = Mat::<K, L>::expand_a(&random_rho(rng));
    run_state_n(
        SAMPLES_MAT,
        runner,
        rng,
        |r| RefCell::new(vec_random::<L>(r, -4, 4)),
        |r| RefCell::new(vec_random::<L>(r, -4, 4)),
        |y| pipeline(&a, y),
    );
}

ctbench_main!(mul_vec_y, mul_vec_s1, null_control);
