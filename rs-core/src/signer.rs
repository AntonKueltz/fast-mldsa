use zeroize::{Zeroize, ZeroizeOnDrop, Zeroizing};

use crate::dsa::Dsa;
use crate::hash::h;
use crate::polynomial::{N, Polynomial};
use crate::signing_key::SigningKey;
use crate::vector::Vector;

#[derive(Zeroize, ZeroizeOnDrop)]
pub(crate) struct Signer<'a, const K: usize, const L: usize> {
    #[zeroize(skip)]
    sk: &'a SigningKey<K, L>,
    s1_hat: Vector<L>,
    s2_hat: Vector<K>,
    t0_hat: Vector<K>,
    mu: [u8; 64],
    rho_prime: [u8; 64],
}

impl<'a, const K: usize, const L: usize> Signer<'a, K, L> {
    pub(crate) fn new(sk: &'a SigningKey<K, L>, msg: &[u8], ctx: &[u8], rnd: &[u8; 32]) -> Self {
        let mut s1_hat = sk.s1().clone();
        s1_hat.ntt();

        let mut s2_hat = sk.s2().clone();
        s2_hat.ntt();

        let mut t0_hat = sk.t0().clone();
        t0_hat.ntt();

        let mut mu = [0u8; 64];
        h(
            &mut mu,
            &[sk.tr().as_slice(), &[0u8], &[ctx.len() as u8], ctx, msg],
        );

        let mut rho_prime = [0u8; 64];
        h(
            &mut rho_prime,
            &[sk.k().as_slice(), rnd.as_slice(), mu.as_slice()],
        );

        Self {
            sk,
            s1_hat,
            s2_hat,
            t0_hat,
            mu,
            rho_prime,
        }
    }

    pub(crate) fn sign<P: Dsa>(&self, sig: &mut [u8]) {
        let mut kappa = 0;

        loop {
            let y = Zeroizing::new(Vector::<L>::expand_mask(
                &self.rho_prime,
                kappa,
                P::GAMMA1,
                P::V_SZ,
            ));
            kappa += L;
            let mut y_hat = y.clone();
            y_hat.ntt();

            let mut w = Zeroizing::new(self.sk.a().mul_vec(&y_hat));
            w.reduce();
            w.inv_ntt();
            w.cond_add();
            let (w1, w0) = w.decompose(P::GAMMA2);
            let mut w0 = Zeroizing::new(w0);

            let mut c_buf = [0u8; 64];
            let chash = &mut c_buf[..P::C_SZ];
            let mut w1_buf = [0u8; 1024];
            let w1_bytes = &mut w1_buf[..P::W1_SZ];

            w1.to_bytes_w1(w1_bytes, P::GAMMA2, P::W1_SZ / K);
            h(chash, &[self.mu.as_slice(), w1_bytes]);
            let mut c_hat = Polynomial::sample_ball(&chash, P::TAU);
            c_hat.ntt();

            let mut z = Zeroizing::new(self.s1_hat.mul_poly(&c_hat));
            z.inv_ntt();
            z.add_acc(&y);
            z.reduce();
            if !z.norm_within_limit(((1 << P::GAMMA1) - P::BETA) as i32) {
                continue;
            }

            let mut cs2 = Zeroizing::new(self.s2_hat.mul_poly(&c_hat));
            cs2.inv_ntt();
            w0.sub_acc(&cs2);
            w0.reduce();
            if !w0.norm_within_limit((P::GAMMA2 - P::BETA) as i32) {
                continue;
            }

            let mut ct0 = Zeroizing::new(self.t0_hat.mul_poly(&c_hat));
            ct0.inv_ntt();
            ct0.reduce();
            if !ct0.norm_within_limit(P::GAMMA2 as i32) {
                continue;
            }

            let mut hint = cs2;
            w0.add_acc(&ct0);
            let n = hint.make_hint(&w0, &w1, P::GAMMA2 as i32);
            if n > P::OMEGA {
                continue;
            }

            Self::encode::<P>(&z, &hint, chash, sig);
            break;
        }
    }

    fn encode<P: Dsa>(z: &Vector<L>, h: &Vector<K>, chash: &[u8], sig: &mut [u8]) {
        let d = 1 + P::GAMMA1;
        debug_assert_eq!(sig.len(), (P::LAMBDA >> 2) + (L << 5) * d + P::OMEGA + K);

        sig[..P::C_SZ].copy_from_slice(&chash[..P::C_SZ]);

        let step = d << 5;
        let mut start = P::C_SZ;
        let mut end = start + step;

        for i in 0..L {
            z.entries[i].to_bytes(&mut sig[start..end], d);
            start = end;
            end += step;
        }

        let base = start;
        let mut k = 0;

        for i in 0..K {
            for j in 0..N {
                if h.entries[i].coeffs[j] != 0 {
                    sig[base + k] = j as u8;
                    k += 1;
                }
            }
            sig[base + P::OMEGA + i] = k as u8;
        }
    }
}
