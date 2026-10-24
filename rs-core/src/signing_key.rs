use zeroize::{Zeroize, ZeroizeOnDrop};

use crate::matrix::Matrix;
use crate::vector::Vector;

#[derive(Zeroize, ZeroizeOnDrop)]
pub struct SigningKey<const K: usize, const L: usize> {
    #[zeroize(skip)]
    rho: [u8; 32],
    k: [u8; 32],
    #[zeroize(skip)]
    tr: [u8; 64],
    s1: Vector<L>,
    s2: Vector<K>,
    t0: Vector<K>,
    #[zeroize(skip)]
    a: Matrix<K, L>,
}

impl<const K: usize, const L: usize> SigningKey<K, L> {
    pub(crate) fn new(
        rho: [u8; 32],
        mut k: [u8; 32],
        tr: [u8; 64],
        s1: Vector<L>,
        s2: Vector<K>,
        t0: Vector<K>,
        a: Matrix<K, L>,
    ) -> Self {
        let sk = Self {
            rho,
            k,
            tr,
            s1,
            s2,
            t0,
            a,
        };
        k.zeroize();
        sk
    }

    pub fn from_bytes(bytes: &[u8], eta: usize) -> Result<Self, &'static str> {
        const D: usize = 13;
        let eta_bits = if eta == 2 { 3 } else { 4 };
        let s_size = eta_bits << 5;

        if bytes.len() != 128 + (L + K) * s_size + (D << 5) * K {
            return Err("Invalid length for byte encoding.");
        }

        let (rho, rest) = bytes.split_at(32);
        let (k, rest) = rest.split_at(32);
        let (tr, rest) = rest.split_at(64);
        let (s1_bytes, rest) = rest.split_at(L * s_size);
        let (s2_bytes, t0_bytes) = rest.split_at(K * s_size);

        let rho: [u8; 32] = rho.try_into().unwrap();
        let tr: [u8; 64] = tr.try_into().unwrap();
        let a = Matrix::<K, L>::expand_a(&rho);

        let mut sk = Self {
            rho,
            k: [0u8; 32],
            tr,
            s1: Vector::<L>::new(),
            s2: Vector::<K>::new(),
            t0: Vector::<K>::new(),
            a,
        };

        sk.k.copy_from_slice(k);
        sk.s1 = Vector::<L>::from_bytes_eta(s1_bytes, eta_bits)?;
        sk.s2 = Vector::<K>::from_bytes_eta(s2_bytes, eta_bits)?;
        sk.t0 = Vector::<K>::from_bytes(t0_bytes, D);

        Ok(sk)
    }

    pub fn to_bytes(&self, buf: &mut [u8], eta: usize) {
        let eta_bits = if eta == 2 { 3 } else { 4 };
        let s_size = eta_bits << 5;
        assert_eq!(buf.len(), 128 + (L + K) * s_size + K * 416);

        buf[..32].copy_from_slice(self.rho.as_slice());
        buf[32..64].copy_from_slice(self.k.as_slice());
        buf[64..128].copy_from_slice(self.tr.as_slice());

        let mut start = 128;
        let mut end = start + s_size;

        for i in 0..L {
            self.s1.entries[i].to_bytes(&mut buf[start..end], eta_bits);
            start = end;
            end += s_size;
        }

        for i in 0..K {
            self.s2.entries[i].to_bytes(&mut buf[start..end], eta_bits);
            start = end;
            end += s_size;
        }

        end += 416 - s_size;
        for i in 0..K {
            self.t0.entries[i].to_bytes(&mut buf[start..end], 13);
            start = end;
            end += 416;
        }
    }

    pub(crate) fn k(&self) -> &[u8; 32] {
        &self.k
    }

    pub(crate) fn tr(&self) -> &[u8; 64] {
        &self.tr
    }

    pub(crate) fn s1(&self) -> &Vector<L> {
        &self.s1
    }

    pub(crate) fn s2(&self) -> &Vector<K> {
        &self.s2
    }

    pub(crate) fn t0(&self) -> &Vector<K> {
        &self.t0
    }

    pub(crate) fn a(&self) -> &Matrix<K, L> {
        &self.a
    }
}
