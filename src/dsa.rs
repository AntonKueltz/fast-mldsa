use zeroize::Zeroize;

use crate::hash::h;
use crate::matrix::Matrix;
use crate::signer::Signer;
use crate::signing_key::SigningKey;
use crate::vector::expand_s;
use crate::verification_key::VerificationKey;

mod sealed {
    pub trait Sealed {}
}

pub struct MlDsa44 {}
pub struct MlDsa65 {}
pub struct MlDsa87 {}

pub trait Dsa: sealed::Sealed {
    const K: usize;
    const L: usize;
    const ETA: usize;
    const GAMMA1: usize;
    const GAMMA2: usize;
    const LAMBDA: usize;
    const TAU: usize;
    const BETA: usize;
    const OMEGA: usize;

    const V_SZ: usize;
    const W1_SZ: usize;
    const C_SZ: usize;

    type VerKey;
    type VerKeyBytes: AsRef<[u8]> + AsMut<[u8]>;

    type SigKey;
    type SigKeyBytes: AsRef<[u8]> + AsMut<[u8]> + Zeroize;

    type SigBytes: AsRef<[u8]> + AsMut<[u8]> + Zeroize;

    fn key_gen() -> (Self::VerKey, Self::SigKey);

    fn sign(sk: &Self::SigKey, msg: &[u8], ctx: &[u8]) -> Option<Self::SigBytes>;
}

impl sealed::Sealed for MlDsa44 {}

impl Dsa for MlDsa44 {
    const K: usize = 4;
    const L: usize = 4;
    const ETA: usize = 2;
    const GAMMA1: usize = 17;
    const GAMMA2: usize = 95232;
    const LAMBDA: usize = 128;
    const TAU: usize = 39;
    const BETA: usize = 78;
    const OMEGA: usize = 80;

    const V_SZ: usize = 18 << 5;
    const W1_SZ: usize = 192 * 4;
    const C_SZ: usize = 32;

    type VerKey = VerificationKey<{ Self::K }, { Self::L }>;
    type VerKeyBytes = [u8; 1312];

    type SigKey = SigningKey<{ Self::K }, { Self::L }>;
    type SigKeyBytes = [u8; 2560];

    type SigBytes = [u8; 2420];

    fn key_gen() -> (Self::VerKey, Self::SigKey) {
        key_gen_random::<Self, { Self::K }, { Self::L }>()
    }

    fn sign(sk: &Self::SigKey, msg: &[u8], ctx: &[u8]) -> Option<Self::SigBytes> {
        if ctx.len() > 255 {
            return None;
        }

        let mut sig = [0u8; 2420];
        let success = sign_random::<Self, { Self::K }, { Self::L }>(sk, msg, ctx, &mut sig);

        match success {
            true => Some(sig),
            false => None,
        }
    }
}

impl sealed::Sealed for MlDsa65 {}

impl Dsa for MlDsa65 {
    const K: usize = 6;
    const L: usize = 5;
    const ETA: usize = 4;
    const GAMMA1: usize = 19;
    const GAMMA2: usize = 261888;
    const LAMBDA: usize = 192;
    const TAU: usize = 49;
    const BETA: usize = 196;
    const OMEGA: usize = 55;

    const V_SZ: usize = 20 << 5;
    const W1_SZ: usize = 128 * 6;
    const C_SZ: usize = 48;

    type VerKey = VerificationKey<{ Self::K }, { Self::L }>;
    type VerKeyBytes = [u8; 1952];

    type SigKey = SigningKey<{ Self::K }, { Self::L }>;
    type SigKeyBytes = [u8; 4032];

    type SigBytes = [u8; 3309];

    fn key_gen() -> (Self::VerKey, Self::SigKey) {
        key_gen_random::<Self, { Self::K }, { Self::L }>()
    }

    fn sign(sk: &Self::SigKey, msg: &[u8], ctx: &[u8]) -> Option<Self::SigBytes> {
        if ctx.len() > 255 {
            return None;
        }

        let mut sig = [0u8; 3309];
        let success = sign_random::<Self, { Self::K }, { Self::L }>(sk, msg, ctx, &mut sig);

        match success {
            true => Some(sig),
            false => None,
        }
    }
}

impl sealed::Sealed for MlDsa87 {}

impl Dsa for MlDsa87 {
    const K: usize = 8;
    const L: usize = 7;
    const ETA: usize = 2;
    const GAMMA1: usize = 19;
    const GAMMA2: usize = 261888;
    const LAMBDA: usize = 256;
    const TAU: usize = 60;
    const BETA: usize = 120;
    const OMEGA: usize = 75;

    const V_SZ: usize = 20 << 5;
    const W1_SZ: usize = 128 * 8;
    const C_SZ: usize = 64;

    type VerKey = VerificationKey<{ Self::K }, { Self::L }>;
    type VerKeyBytes = [u8; 2592];

    type SigKey = SigningKey<{ Self::K }, { Self::L }>;
    type SigKeyBytes = [u8; 4896];

    type SigBytes = [u8; 4627];

    fn key_gen() -> (Self::VerKey, Self::SigKey) {
        key_gen_random::<Self, { Self::K }, { Self::L }>()
    }

    fn sign(sk: &Self::SigKey, msg: &[u8], ctx: &[u8]) -> Option<Self::SigBytes> {
        if ctx.len() > 255 {
            return None;
        }

        let mut sig = [0u8; 4627];
        let success = sign_random::<Self, { Self::K }, { Self::L }>(sk, msg, ctx, &mut sig);

        match success {
            true => Some(sig),
            false => None,
        }
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
    h(&mut buf, &[seed.as_slice(), &[K as u8, L as u8]]);

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

    let mut t = a.mul_vec(&s1_);
    t.reduce();
    t.inv_ntt();
    t.add_acc(&s2);
    t.cond_add();
    let (t1, t0) = t.pow2_round();

    let pk = VerificationKey::new(t1, rho, a.clone());
    let tr = pk.hash();
    let sk = SigningKey::new(rho, k, tr, s1, s2, t0, a);

    (pk, sk)
}

fn sign_random<P: Dsa, const K: usize, const L: usize>(
    sk: &SigningKey<K, L>,
    msg: &[u8],
    ctx: &[u8],
    sig: &mut [u8],
) -> bool {
    let mut rnd = [0u8; 32];
    if getrandom::fill(&mut rnd).is_ok() {
        sign_internal::<P, K, L>(sk, msg, ctx, &rnd, sig);
        true
    } else {
        false
    }
}

fn sign_internal<P: Dsa, const K: usize, const L: usize>(
    sk: &SigningKey<K, L>,
    msg: &[u8],
    ctx: &[u8],
    rnd: &[u8; 32],
    sig: &mut [u8],
) {
    let signer = Signer::<K, L>::new(sk, msg, ctx, rnd);
    signer.sign::<P>(sig);
}

#[cfg(test)]
#[path = "unit_tests/key_gen_test.rs"]
mod key_gen_test;

#[cfg(test)]
#[path = "unit_tests/sign_test.rs"]
mod sign_test;
