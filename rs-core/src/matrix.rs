use core::array::from_fn;

use crate::polynomial::Polynomial;
use crate::vector::Vector;

#[derive(Clone)]
pub(crate) struct Matrix<const K: usize, const L: usize> {
    rows: [Vector<L>; K],
}

impl<const K: usize, const L: usize> Matrix<K, L> {
    pub(crate) fn new() -> Self {
        let rows: [Vector<L>; K] = from_fn(|_| Vector::<L>::new());

        Self { rows }
    }

    pub(crate) fn expand_a(rho: &[u8; 32]) -> Self {
        let mut a = Self::new();

        for r in 0..K {
            for s in 0..L {
                a.rows[r].entries[s] = Polynomial::sample_ntt(rho, s, r);
            }
        }

        a
    }

    pub(crate) fn mul_vec(&self, other: &Vector<L>) -> Vector<K> {
        let mut w = Vector::<K>::new();

        for i in 0..K {
            for j in 0..L {
                w.entries[i].mul_acc(&self.rows[i].entries[j], &other.entries[j]);
            }
        }

        w
    }
}

#[cfg(feature = "ct-testing")]
pub mod ct_exports {
    use super::Matrix;
    use crate::vector::ct_exports::V;

    pub struct Mat<const K: usize, const L: usize>(Matrix<K, L>);

    impl<const K: usize, const L: usize> Mat<K, L> {
        pub fn expand_a(rho: &[u8; 32]) -> Self {
            Self(Matrix::expand_a(rho))
        }

        #[inline(never)]
        pub fn mul_vec(&self, y: &V<L>) -> V<K> {
            V(self.0.mul_vec(&y.0))
        }
    }
}
