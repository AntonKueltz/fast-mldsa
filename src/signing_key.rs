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

    pub(crate) fn to_bytes(&self, buf: &mut [u8]) {
        buf[..32].copy_from_slice(self.rho.as_slice());
        buf[32..64].copy_from_slice(self.k.as_slice());
        buf[64..128].copy_from_slice(self.tr.as_slice());

        let (mut start, mut end) = (128, 224);
        for i in 0..L {
            self.s1.entries[i].to_bytes(&mut buf[start..end], 3);
            start = end;
            end += 96;
        }

        for i in 0..K {
            self.s2.entries[i].to_bytes(&mut buf[start..end], 3);
            start = end;
            end += 96;
        }

        end += 320;
        for i in 0..K {
            self.t0.entries[i].to_bytes(&mut buf[start..end], 13);
            start = end;
            end += 416;
        }
    }
}
