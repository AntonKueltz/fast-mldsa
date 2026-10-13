use std::array::from_fn;

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

    pub(crate) fn add_acc(&mut self, other: &Vector<N>) {
        for i in 0..N {
            self.entries[i].add_acc(&other.entries[i]);
        }
    }

    pub(crate) fn pow2_round(&self) -> (Self, Self) {
        let mut hi = Self::new();
        let mut lo = Self::new();

        for i in 0..N {
            self.entries[i].pow2_round(&mut hi.entries[i], &mut lo.entries[i]);
        }

        (hi, lo)
    }

    pub(crate) fn to_bytes(&self, buf: &mut [u8], d: usize) {
        debug_assert_eq!(buf.len(), 320 * N);

        let (mut start, mut end) = (0, 320);
        for i in 0..N {
            self.entries[i].to_bytes(&mut buf[start..end], d);
            start = end;
            end += 320;
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
