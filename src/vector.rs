use std::array::from_fn;

use crate::hash::h;
use crate::polynomial::Polynomial;

#[derive(Clone)]
pub(crate) struct Vector<const N: usize> {
    pub(crate) entries: [Polynomial; N],
}

impl<const N: usize> Vector<N> {
    pub(crate) fn new() -> Self {
        let entries: [Polynomial; N] = from_fn(|_| Polynomial::new());

        Self { entries }
    }

    pub(crate) fn from_bytes(bytes: &[u8], d: usize) -> Self {
        let sz = d << 5;
        debug_assert_eq!(bytes.len(), N * sz);

        let mut r = Self::new();
        let (mut start, mut end) = (0, sz);

        for i in 0..N {
            r.entries[i] = Polynomial::from_bytes(&bytes[start..end], d);
            start = end;
            end += sz;
        }

        r
    }

    pub(crate) fn from_bytes_eta(bytes: &[u8], eta_bits: usize) -> Result<Self, String> {
        let sz = eta_bits << 5;
        debug_assert_eq!(bytes.len(), N * sz);

        let mut r = Self::new();
        let (mut start, mut end) = (0, sz);

        for i in 0..N {
            r.entries[i] = Polynomial::from_bytes_eta(&bytes[start..end], eta_bits)?;
            start = end;
            end += sz;
        }

        Ok(r)
    }

    pub(crate) fn expand_mask(rho: &[u8; 64], mu: usize, gamma: usize, v_size: usize) -> Self {
        let mut y = Self::new();

        for r in 0..N {
            let mut v_buf = [0u8; 640];
            let v = &mut v_buf[..v_size];

            let ctr_bytes = (mu + r).to_le_bytes();
            h(v, &[rho.as_slice(), &ctr_bytes[..2]]);

            y.entries[r] = Polynomial::from_bytes(&v, gamma + 1);
        }

        y
    }

    pub(crate) fn mul_poly(&self, other: &Polynomial) -> Self {
        let mut r = Self::new();

        for i in 0..N {
            r.entries[i].mul_acc(&self.entries[i], other);
        }

        r
    }

    pub(crate) fn from_hint_bytes(bytes: &[u8], omega: usize) -> Option<Self> {
        let mut h = Self::new();
        let mut k: usize = 0;

        for i in 0..N {
            if bytes[omega + i] < k as u8 || bytes[omega + i] > omega as u8 {
                return None;
            }

            for j in k..(bytes[omega + i] as usize) {
                if j > k && bytes[j] <= bytes[j - 1] {
                    return None;
                }

                h.entries[i].coeffs[bytes[j] as usize] = 1;
            }

            k = bytes[omega + i] as usize;
        }

        for j in k..omega {
            if bytes[j] != 0 {
                return None;
            }
        }

        Some(h)
    }

    pub(crate) fn pow2_round(&self) -> (Self, Self) {
        let mut hi = Self::new();
        let mut lo = Self::new();

        for i in 0..N {
            self.entries[i].pow2_round(&mut hi.entries[i], &mut lo.entries[i]);
        }

        (hi, lo)
    }

    pub(crate) fn decompose(&self, gamma: usize) -> (Self, Self) {
        let mut hi = Self::new();
        let mut lo = Self::new();

        for i in 0..N {
            self.entries[i].decompose(&mut hi.entries[i], &mut lo.entries[i], gamma);
        }

        (hi, lo)
    }

    pub(crate) fn reduce(&mut self) {
        for i in 0..N {
            self.entries[i].reduce();
        }
    }

    pub(crate) fn cond_add(&mut self) {
        for i in 0..N {
            self.entries[i].cond_add();
        }
    }

    pub(crate) fn ntt(&mut self) {
        for i in 0..N {
            self.entries[i].ntt();
        }
    }

    pub(crate) fn inv_ntt(&mut self) {
        for i in 0..N {
            self.entries[i].inv_ntt();
        }
    }

    pub(crate) fn norm_within_limit(&self, limit: i32) -> bool {
        for i in 0..N {
            if !self.entries[i].norm_within_limit(limit) {
                return false;
            }
        }

        true
    }

    pub(crate) fn make_hint(&mut self, lo: &Self, hi: &Self, gamma: i32) -> usize {
        let mut ones = 0;

        for i in 0..N {
            ones +=
                Polynomial::make_hint(&mut self.entries[i], &lo.entries[i], &hi.entries[i], gamma);
        }

        ones
    }

    pub(crate) fn use_hint(hint: &Self, v: &Self, gamma: usize) -> Self {
        let mut r = Self::new();

        for i in 0..N {
            r.entries[i].use_hint(&hint.entries[i], &v.entries[i], gamma);
        }

        r
    }

    pub(crate) fn add_acc(&mut self, other: &Vector<N>) {
        for i in 0..N {
            self.entries[i].add_acc(&other.entries[i]);
        }
    }

    pub(crate) fn sub_acc(&mut self, other: &Vector<N>) {
        for i in 0..N {
            self.entries[i].sub_acc(&other.entries[i]);
        }
    }

    pub(crate) fn to_bytes_w1(&self, buf: &mut [u8], gamma: usize, step: usize) {
        let (mut start, mut end) = (0, step);

        for i in 0..N {
            self.entries[i].to_bytes_w1(&mut buf[start..end], gamma);
            start = end;
            end += step;
        }
    }
}

pub(crate) fn expand_s<const K: usize, const L: usize>(
    rho: &[u8; 64],
    eta: usize,
) -> (Vector<L>, Vector<K>) {
    let mut s1 = Vector::<L>::new();
    for r in 0..L {
        s1.entries[r] = Polynomial::sample_bounded(rho, eta, r);
    }

    let mut s2 = Vector::<K>::new();
    for r in 0..K {
        s2.entries[r] = Polynomial::sample_bounded(rho, eta, r + L);
    }

    (s1, s2)
}
