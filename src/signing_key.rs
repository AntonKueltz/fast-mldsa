use crate::vector::Vector;

pub struct SigningKey<const K: usize, const L: usize> {
    rho: [u8; 32],
    k: [u8; 32],
    tr: [u8; 64],
    s1: Vector<L>,
    s2: Vector<K>,
    t0: Vector<K>,
}

impl<const K: usize, const L: usize> SigningKey<K, L> {
    pub(crate) fn new(
        rho: [u8; 32],
        k: [u8; 32],
        tr: [u8; 64],
        s1: Vector<L>,
        s2: Vector<K>,
        t0: Vector<K>,
    ) -> Self {
        Self {
            rho,
            k,
            tr,
            s1,
            s2,
            t0,
        }
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
}
