use std::hint::black_box;

use dudect_bencher::rand::RngExt;
use dudect_bencher::{BenchRng, Class, CtRunner, ctbench_main};

use ml_dsa::ct::integer as arith;

const N: usize = 20_000;
const BATCH: usize = 512;

const Q: i32 = arith::Q;
const HALF_Q: i32 = (Q - 1) / 2;
const MONT_MAX: i64 = (Q as i64) << 31;
const REDUCE_MAX: i32 = i32::MAX - (1 << 22);

const MONT_EDGES: &[i64] = &[0, MONT_MAX, -MONT_MAX];
const REDUCE_EDGES: &[i32] = &[0, -1, REDUCE_MAX, -REDUCE_MAX];
const SIGNED_Q_EDGES: &[i32] = &[0, -1, 1, Q - 1, -(Q - 1)];
const UNSIGNED_Q_EDGES: &[i32] = &[0, 1, 0xfff, 0x1000, HALF_Q, Q - 1];

fn signed_q(r: &mut BenchRng) -> i32 {
    r.random_range(-(Q - 1)..=(Q - 1))
}
fn unsigned_q(r: &mut BenchRng) -> i32 {
    r.random_range(0..Q)
}

// ---------- harness ----------
type Fill<T> = Box<dyn FnMut(&mut BenchRng, &mut Vec<T>)>;

fn constant_batches<T: Copy + 'static>(edges: &'static [T]) -> Fill<T> {
    Box::new(move |rng, out| {
        let v = edges[rng.random_range(0..edges.len())];
        out.resize(out.len() + BATCH, v);
    })
}

fn random_batches<T: 'static>(gen_one: impl Fn(&mut BenchRng) -> T + 'static) -> Fill<T> {
    Box::new(move |rng, out| out.extend((0..BATCH).map(|_| gen_one(&mut *rng))))
}

fn run<T: Copy, R>(
    runner: &mut CtRunner,
    rng: &mut BenchRng,
    mut left: Fill<T>,
    mut right: Fill<T>,
    op: impl Fn(T) -> R,
) {
    let mut data = Vec::with_capacity(N * BATCH);
    let mut classes = Vec::with_capacity(N);

    for _ in 0..N {
        if rng.random::<bool>() {
            left(rng, &mut data);
            classes.push(Class::Left);
        } else {
            right(rng, &mut data);
            classes.push(Class::Right);
        }
    }

    for (batch, class) in data.chunks_exact(BATCH).zip(classes) {
        runner.run_one(class, || {
            for &x in batch {
                black_box(op(x));
            }
        });
    }
}

// ---------- benches: fixed edge values vs random ----------
fn mont_q(runner: &mut CtRunner, rng: &mut BenchRng) {
    run(
        runner,
        rng,
        constant_batches(MONT_EDGES),
        random_batches(|r| r.random_range(-MONT_MAX..=MONT_MAX)),
        arith::mont_q,
    );
}

fn reduce_q(runner: &mut CtRunner, rng: &mut BenchRng) {
    run(
        runner,
        rng,
        constant_batches(REDUCE_EDGES),
        random_batches(|r| r.random_range(-REDUCE_MAX..=REDUCE_MAX)),
        arith::reduce_q,
    );
}

fn cond_add_q(runner: &mut CtRunner, rng: &mut BenchRng) {
    run(
        runner,
        rng,
        constant_batches(SIGNED_Q_EDGES),
        random_batches(signed_q),
        arith::cond_add_q,
    );
}

fn infinity_norm(runner: &mut CtRunner, rng: &mut BenchRng) {
    run(
        runner,
        rng,
        constant_batches(SIGNED_Q_EDGES),
        random_batches(signed_q),
        arith::infinity_norm,
    );
}

fn pow2_round(runner: &mut CtRunner, rng: &mut BenchRng) {
    run(
        runner,
        rng,
        constant_batches(UNSIGNED_Q_EDGES),
        random_batches(unsigned_q),
        arith::pow2_round,
    );
}

fn decompose_32(runner: &mut CtRunner, rng: &mut BenchRng) {
    run(
        runner,
        rng,
        constant_batches(UNSIGNED_Q_EDGES),
        random_batches(unsigned_q),
        |a| arith::decompose(a, arith::GAMMA2_32),
    );
}

fn decompose_88(runner: &mut CtRunner, rng: &mut BenchRng) {
    run(
        runner,
        rng,
        constant_batches(UNSIGNED_Q_EDGES),
        random_batches(unsigned_q),
        |a| arith::decompose(a, arith::GAMMA2_88),
    );
}

// ---------- controls ----------
fn null_control(runner: &mut CtRunner, rng: &mut BenchRng) {
    run(
        runner,
        rng,
        random_batches(signed_q),
        random_batches(signed_q),
        arith::cond_add_q,
    );
}

fn use_hint_positive_control(runner: &mut CtRunner, rng: &mut BenchRng) {
    run(
        runner,
        rng,
        constant_batches(&[(HALF_Q, 0i32)]),
        random_batches(|r| (r.random_range(0..Q), r.random_range(0..=1))),
        |(a, h)| arith::use_hint(a, h, arith::GAMMA2_88),
    );
}

ctbench_main!(
    mont_q,
    reduce_q,
    cond_add_q,
    infinity_norm,
    pow2_round,
    decompose_32,
    decompose_88,
    null_control,
    use_hint_positive_control
);
