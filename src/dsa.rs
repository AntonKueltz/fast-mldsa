use shake::{ExtendableOutput, Shake256, Update, XofReader};
use zeroize::Zeroize;

use crate::matrix::Matrix;
use crate::signing_key::SigningKey;
use crate::vector::expand_s;
use crate::verification_key::VerificationKey;

mod sealed {
    pub trait Sealed {}
}

pub struct MlDsa44 {}

pub trait Dsa: sealed::Sealed {
    const K: usize;
    const L: usize;
    const ETA: usize;

    type VerKey;
    type VeryKeyBytes: AsRef<[u8]> + AsMut<[u8]>;

    type SigKey;
    type SigKeyBytes: AsRef<[u8]> + AsMut<[u8]> + Zeroize;

    fn key_gen() -> (Self::VerKey, Self::SigKey);
}

impl sealed::Sealed for MlDsa44 {}

impl Dsa for MlDsa44 {
    const K: usize = 4;
    const L: usize = 4;
    const ETA: usize = 2;

    type VerKey = VerificationKey<{ Self::K }, { Self::L }>;
    type VeryKeyBytes = [u8; 1312];

    type SigKey = SigningKey<{ Self::K }, { Self::L }>;
    type SigKeyBytes = [u8; 2560];

    fn key_gen() -> (Self::VerKey, Self::SigKey) {
        key_gen_random::<Self, { Self::K }, { Self::L }>()
    }
}

fn key_gen_random<P: Dsa, const K: usize, const L: usize>()
-> (VerificationKey<K, L>, SigningKey<K, L>) {
    let mut seed = [0u8; 32];
    getrandom::fill(&mut seed).expect("Rng failure.");
    key_gen_internal::<P, K, L>(&seed)
}

fn key_gen_internal<P: Dsa, const K: usize, const L: usize>(
    seed: &[u8; 32],
) -> (VerificationKey<K, L>, SigningKey<K, L>) {
    let mut buf = [0u8; 128];
    h(&mut buf, seed, K, L);

    let (b1, rest) = buf.split_at(32);
    let (b2, b3) = rest.split_at(64);

    let rho: [u8; 32] = b1.try_into().unwrap();
    let rho_: [u8; 64] = b2.try_into().unwrap();
    let k: [u8; 32] = b3.try_into().unwrap();
    buf.zeroize();

    let a = Matrix::<K, L>::expand_a(&rho);
    let (s1, s2) = expand_s::<K, L>(&rho_, P::ETA);
    let mut s1_ = s1.clone();
    s1_.ntt();

    let mut t = a.mul(&s1_);
    t.reduce();
    t.inv_ntt();
    t.add_acc(&s2);
    t.cond_add();
    let (t1, t0) = t.pow2_round();

    let pk = VerificationKey::new(t1, rho, a);
    let tr = pk.hash();
    let sk = SigningKey::new(rho, k, tr, s1, s2, t0);

    (pk, sk)
}

fn h(out: &mut [u8], seed: &[u8; 32], k: usize, l: usize) {
    let mut shaker = Shake256::default();

    shaker.update(seed);
    shaker.update(&[k as u8]);
    shaker.update(&[l as u8]);

    shaker.finalize_xof().read(out);
}

#[cfg(test)]
#[path = "unit_tests/key_gen_test.rs"]
mod key_gen_test;
