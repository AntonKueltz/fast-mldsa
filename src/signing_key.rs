use crate::matrix::Matrix;
use crate::vector::Vector;

pub struct SigningKey<const K: usize, const L: usize> {
    rho: [u8; 32],
    k: [u8; 32],
    tr: [u8; 64],
    s1: Vector<L>,
    s2: Vector<K>,
    t0: Vector<K>,
    a: Matrix<K, L>,
}

impl<const K: usize, const L: usize> SigningKey<K, L> {
    pub(crate) fn new(
        rho: [u8; 32],
        k: [u8; 32],
        tr: [u8; 64],
        s1: Vector<L>,
        s2: Vector<K>,
        t0: Vector<K>,
        a: Matrix<K, L>,
    ) -> Self {
        Self {
            rho,
            k,
            tr,
            s1,
            s2,
            t0,
            a,
        }
    }

    pub(crate) fn from_bytes(bytes: &[u8], eta: usize) -> Result<Self, &'static str> {
        const D: usize = 13;
        let eta_bits = if eta == 2 { 3 } else { 4 };
        let s_size = eta_bits << 5;

        let (rho, rest) = bytes.split_at(32);
        let (k, rest) = rest.split_at(32);
        let (tr, rest) = rest.split_at(64);
        let (s1_bytes, rest) = rest.split_at(L * s_size);
        let (s2_bytes, rest) = rest.split_at(K * s_size);

        if rest.len() != (K << 5) * D {
            return Err("Invalid length for byte encoding.");
        }

        let rho: [u8; 32] = rho.try_into().unwrap();
        let k: [u8; 32] = k.try_into().unwrap();
        let tr: [u8; 64] = tr.try_into().unwrap();
        let s1 = Vector::<L>::from_bytes_eta(s1_bytes, eta_bits)?;
        let s2 = Vector::<K>::from_bytes_eta(s2_bytes, eta_bits)?;
        let t0 = Vector::<K>::from_bytes(rest, D);
        let a = Matrix::<K, L>::expand_a(&rho);

        Ok(Self {
            rho,
            k,
            tr,
            s1,
            s2,
            t0,
            a,
        })
    }

    pub(crate) fn to_bytes(&self, buf: &mut [u8], eta_d: usize) {
        buf[..32].copy_from_slice(self.rho.as_slice());
        buf[32..64].copy_from_slice(self.k.as_slice());
        buf[64..128].copy_from_slice(self.tr.as_slice());

        let inc: usize = eta_d * 32;
        let mut start = 128;
        let mut end = start + inc;

        for i in 0..L {
            self.s1.entries[i].to_bytes(&mut buf[start..end], eta_d);
            start = end;
            end += inc;
        }

        for i in 0..K {
            self.s2.entries[i].to_bytes(&mut buf[start..end], eta_d);
            start = end;
            end += inc;
        }

        end += 416 - inc;
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
