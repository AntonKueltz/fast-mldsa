use criterion::{Criterion, criterion_group, criterion_main};
use std::hint::black_box;

use ml_dsa::dsa::{Dsa, MlDsa44, MlDsa65, MlDsa87};

fn bench_dsa(c: &mut Criterion) {
    // 44
    c.bench_function(&format!("keygen/44"), |b| b.iter(|| MlDsa44::key_gen(None)));

    let (pk, sk) = MlDsa44::key_gen(None);
    let msg = [0u8; 64];
    let ctx = [0u8; 32];

    c.bench_function(&format!("sign/44"), |b| {
        b.iter(|| MlDsa44::sign(black_box(&sk), black_box(&msg), black_box(&ctx), false))
    });

    let sig = MlDsa44::sign(&sk, &msg, &ctx, false).unwrap();
    c.bench_function(&format!("verify/44"), |b| {
        b.iter(|| {
            MlDsa44::verify(
                black_box(&pk),
                black_box(&msg),
                black_box(&ctx),
                black_box(&sig),
            )
        })
    });

    // 65
    c.bench_function(&format!("keygen/65"), |b| b.iter(|| MlDsa65::key_gen(None)));

    let (pk, sk) = MlDsa65::key_gen(None);
    let msg = [0u8; 64];
    let ctx = [0u8; 32];

    c.bench_function(&format!("sign/65"), |b| {
        b.iter(|| MlDsa65::sign(black_box(&sk), black_box(&msg), black_box(&ctx), false))
    });

    let sig = MlDsa65::sign(&sk, &msg, &ctx, false).unwrap();
    c.bench_function(&format!("verify/65"), |b| {
        b.iter(|| {
            MlDsa65::verify(
                black_box(&pk),
                black_box(&msg),
                black_box(&ctx),
                black_box(&sig),
            )
        })
    });

    // 87
    c.bench_function(&format!("keygen/87"), |b| b.iter(|| MlDsa87::key_gen(None)));

    let (pk, sk) = MlDsa87::key_gen(None);
    let msg = [0u8; 64];
    let ctx = [0u8; 32];

    c.bench_function(&format!("sign/87"), |b| {
        b.iter(|| MlDsa87::sign(black_box(&sk), black_box(&msg), black_box(&ctx), false))
    });

    let sig = MlDsa87::sign(&sk, &msg, &ctx, false).unwrap();
    c.bench_function(&format!("verify/87"), |b| {
        b.iter(|| {
            MlDsa87::verify(
                black_box(&pk),
                black_box(&msg),
                black_box(&ctx),
                black_box(&sig),
            )
        })
    });
}

criterion_group!(benches, bench_dsa);
criterion_main!(benches);
