use shake::{ExtendableOutput, Shake256, Update, XofReader};

use crate::matrix::Matrix;
use crate::vector::Vector;

pub struct VerificationKey<const K: usize, const L: usize> {
    t1: Vector<K>,
    rho: [u8; 32],
    a: Matrix<K, L>,
}

impl<const K: usize, const L: usize> VerificationKey<K, L> {
    pub(crate) fn new(t1: Vector<K>, rho: [u8; 32], a: Matrix<K, L>) -> Self {
        Self { t1, rho, a }
    }

    pub(crate) fn hash(&self) -> [u8; 64] {
        let mut shaker = Shake256::default();
        shaker.update(&self.rho);
        for i in 0..K {
            let mut buf = [0u8; 320];
            self.t1.entries[i].to_bytes(&mut buf, 10);
            shaker.update(&buf);
        }
        let mut xof = shaker.finalize_xof();

        let mut out = [0u8; 64];
        xof.read(&mut out);
        out
    }

    pub fn to_bytes(&self, buf: &mut [u8]) {
        buf[..32].copy_from_slice(self.rho.as_slice());

        let (mut start, mut end) = (32, 352);
        for i in 0..K {
            self.t1.entries[i].to_bytes(&mut buf[start..end], 10);
            start = end;
            end += 320;
        }
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Self, &'static str> {
        if bytes.len() != 32 + K * 320 {
            return Err("Invalid length for byte encoding.");
        }

        let (rho, t1_bytes) = bytes.split_at(32);
        let t1 = Vector::<K>::from_bytes(t1_bytes, 10);
        let rho: [u8; 32] = rho.try_into().unwrap();
        let a = Matrix::<K, L>::expand_a(&rho);

        return Ok(Self { t1, rho, a });
    }

    pub(crate) fn t1(&self) -> &Vector<K> {
        &self.t1
    }

    pub(crate) fn a(&self) -> &Matrix<K, L> {
        &self.a
    }
}
