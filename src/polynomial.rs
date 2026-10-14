use shake::{ExtendableOutput, Shake128, Shake256, Update, XofReader};

use crate::integer_field::{cond_add_q, mont_q, reduce_q};

const N: usize = 256;
const Q: i32 = 8380417;

const ZETAS: [i32; N] = [
    0, 25847, -2608894, -518909, 237124, -777960, -876248, 466468, 1826347, 2353451, -359251,
    -2091905, 3119733, -2884855, 3111497, 2680103, 2725464, 1024112, -1079900, 3585928, -549488,
    -1119584, 2619752, -2108549, -2118186, -3859737, -1399561, -3277672, 1757237, -19422, 4010497,
    280005, 2706023, 95776, 3077325, 3530437, -1661693, -3592148, -2537516, 3915439, -3861115,
    -3043716, 3574422, -2867647, 3539968, -300467, 2348700, -539299, -1699267, -1643818, 3505694,
    -3821735, 3507263, -2140649, -1600420, 3699596, 811944, 531354, 954230, 3881043, 3900724,
    -2556880, 2071892, -2797779, -3930395, -1528703, -3677745, -3041255, -1452451, 3475950,
    2176455, -1585221, -1257611, 1939314, -4083598, -1000202, -3190144, -3157330, -3632928, 126922,
    3412210, -983419, 2147896, 2715295, -2967645, -3693493, -411027, -2477047, -671102, -1228525,
    -22981, -1308169, -381987, 1349076, 1852771, -1430430, -3343383, 264944, 508951, 3097992,
    44288, -1100098, 904516, 3958618, -3724342, -8578, 1653064, -3249728, 2389356, -210977, 759969,
    -1316856, 189548, -3553272, 3159746, -1851402, -2409325, -177440, 1315589, 1341330, 1285669,
    -1584928, -812732, -1439742, -3019102, -3881060, -3628969, 3839961, 2091667, 3407706, 2316500,
    3817976, -3342478, 2244091, -2446433, -3562462, 266997, 2434439, -1235728, 3513181, -3520352,
    -3759364, -1197226, -3193378, 900702, 1859098, 909542, 819034, 495491, -1613174, -43260,
    -522500, -655327, -3122442, 2031748, 3207046, -3556995, -525098, -768622, -3595838, 342297,
    286988, -2437823, 4108315, 3437287, -3342277, 1735879, 203044, 2842341, 2691481, -2590150,
    1265009, 4055324, 1247620, 2486353, 1595974, -3767016, 1250494, 2635921, -3548272, -2994039,
    1869119, 1903435, -1050970, -1333058, 1237275, -3318210, -1430225, -451100, 1312455, 3306115,
    -1962642, -1279661, 1917081, -2546312, -1374803, 1500165, 777191, 2235880, 3406031, -542412,
    -2831860, -1671176, -1846953, -2584293, -3724270, 594136, -3776993, -2013608, 2432395, 2454455,
    -164721, 1957272, 3369112, 185531, -1207385, -3183426, 162844, 1616392, 3014001, 810149,
    1652634, -3694233, -1799107, -3038916, 3523897, 3866901, 269760, 2213111, -975884, 1717735,
    472078, -426683, 1723600, -1803090, 1910376, -1667432, -1104333, -260646, -3833893, -2939036,
    -2235985, -420899, -2286327, 183443, -976891, 1612842, -3545687, -554416, 3919660, -48306,
    -1362209, 3937738, 1400424, -846154, 1976782,
];

#[derive(Clone)]
pub(crate) struct Polynomial {
    coeffs: [i32; N],
}

impl Polynomial {
    pub(crate) fn new() -> Self {
        Self { coeffs: [0; N] }
    }

    pub(crate) fn sample_ntt(rho: &[u8; 32], s: usize, r: usize) -> Self {
        let mut p = Self::new();
        let mut j = 0;

        let mut shaker = Shake128::default();
        shaker.update(rho);
        shaker.update(&[s as u8]);
        shaker.update(&[r as u8]);
        let mut xof = shaker.finalize_xof();

        while j < N {
            let mut b = [0u8; 3];
            xof.read(&mut b);

            let z = (((b[2] & 0x7f) as i32) << 16) | ((b[1] as i32) << 8) | (b[0] as i32);
            p.coeffs[j] = z;
            j += (z < Q) as usize;
        }

        p
    }

    pub(crate) fn sample_bounded(rho: &[u8; 64], eta: usize, i: usize) -> Self {
        let mut p = Self::new();
        let mut j = 0;

        let mut shaker = Shake256::default();
        shaker.update(rho);
        shaker.update(&[i as u8, 0u8]);
        let mut xof = shaker.finalize_xof();

        while j < N {
            let mut z = [0u8];
            xof.read(&mut z);

            let (mut z0, mut z1) = ((z[0] & 0xf) as i32, (z[0] >> 4) as i32);

            if eta == 2 {
                if z0 < 15 {
                    z0 = z0 - (205 * z0 >> 10) * 5;
                    p.coeffs[j] = 2 - z0;
                    j += 1
                }
                if z1 < 15 && j < N {
                    z1 = z1 - (205 * z1 >> 10) * 5;
                    p.coeffs[j] = 2 - z1;
                    j += 1
                }
            } else if eta == 4 {
                if z0 < 9 {
                    p.coeffs[j] = 4 - z0;
                    j += 1;
                }
                if z1 < 9 && j < N {
                    p.coeffs[j] = 4 - z1;
                    j += 1;
                }
            }
        }

        p
    }

    pub(crate) fn reduce(&mut self) {
        for i in 0..N {
            self.coeffs[i] = reduce_q(self.coeffs[i]);
        }
    }

    pub(crate) fn cond_add(&mut self) {
        for i in 0..N {
            self.coeffs[i] = cond_add_q(self.coeffs[i]);
        }
    }

    pub(crate) fn ntt(&mut self) {
        let mut m = 0;
        let mut len = 128;

        while len >= 1 {
            let mut start = 0;

            while start < N {
                m += 1;
                let z = ZETAS[m];

                for j in start..(start + len) {
                    let t = mont_q((z as i64) * (self.coeffs[j + len] as i64));
                    self.coeffs[j + len] = self.coeffs[j] - t;
                    self.coeffs[j] += t;
                }

                start += len << 1;
            }

            len >>= 1;
        }
    }

    pub(crate) fn inv_ntt(&mut self) {
        const F: i64 = 41978;
        let mut m = N;
        let mut len = 1;

        while len < N {
            let mut start = 0;

            while start < N {
                m -= 1;
                let z = -ZETAS[m];

                for j in start..(start + len) {
                    let t = self.coeffs[j];
                    self.coeffs[j] = t + self.coeffs[j + len];
                    self.coeffs[j + len] = t - self.coeffs[j + len];
                    self.coeffs[j + len] = mont_q((z as i64) * (self.coeffs[j + len] as i64));
                }

                start += len << 1;
            }

            len <<= 1;
        }

        for j in 0..N {
            self.coeffs[j] = mont_q(F * (self.coeffs[j] as i64))
        }
    }

    pub(crate) fn add_acc(&mut self, other: &Self) {
        for i in 0..N {
            self.coeffs[i] += other.coeffs[i];
        }
    }

    pub(crate) fn mul_acc(&mut self, lhs: &Self, rhs: &Self) {
        for i in 0..N {
            self.coeffs[i] += mont_q((lhs.coeffs[i] as i64) * (rhs.coeffs[i] as i64))
        }
    }

    pub(crate) fn pow2_round(&self, hi: &mut Self, lo: &mut Self) {
        for i in 0..N {
            let a = self.coeffs[i];
            let a_hi = (a + 0xfff) >> 13;

            hi.coeffs[i] = a_hi;
            lo.coeffs[i] = a - (a_hi << 13);
        }
    }

    pub(crate) fn to_bytes(&self, buf: &mut [u8], d: usize) {
        match d {
            13 => self.to_bytes_13(buf),
            10 => self.to_bytes_10(buf),
            4 => self.to_bytes_4(buf),
            3 => self.to_bytes_3(buf),
            _ => unreachable!(),
        }
    }

    fn to_bytes_13(&self, buf: &mut [u8]) {
        debug_assert_eq!(buf.len(), 416);
        let mut bi = 0;
        const B: i32 = 1 << 12;

        for i in (0..N).step_by(8) {
            let w0 = B - self.coeffs[i];
            let w1 = B - self.coeffs[i + 1];
            let w2 = B - self.coeffs[i + 2];
            let w3 = B - self.coeffs[i + 3];
            let w4 = B - self.coeffs[i + 4];
            let w5 = B - self.coeffs[i + 5];
            let w6 = B - self.coeffs[i + 6];
            let w7 = B - self.coeffs[i + 7];

            buf[bi] = w0 as u8;
            buf[bi + 1] = ((w0 >> 8) as u8) | ((w1 << 5) as u8);
            buf[bi + 2] = (w1 >> 3) as u8;
            buf[bi + 3] = ((w1 >> 11) as u8) | ((w2 << 2) as u8);
            buf[bi + 4] = ((w2 >> 6) as u8) | ((w3 << 7) as u8);
            buf[bi + 5] = (w3 >> 1) as u8;
            buf[bi + 6] = ((w3 >> 9) as u8) | ((w4 << 4) as u8);
            buf[bi + 7] = (w4 >> 4) as u8;
            buf[bi + 8] = ((w4 >> 12) as u8) | ((w5 << 1) as u8);
            buf[bi + 9] = ((w5 >> 7) as u8) | ((w6 << 6) as u8);
            buf[bi + 10] = (w6 >> 2) as u8;
            buf[bi + 11] = ((w6 >> 10) as u8) | ((w7 << 3) as u8);
            buf[bi + 12] = (w7 >> 5) as u8;
            bi += 13
        }
    }

    fn to_bytes_10(&self, buf: &mut [u8]) {
        debug_assert_eq!(buf.len(), 320);
        let mut bi = 0;

        for i in (0..N).step_by(4) {
            let w0 = self.coeffs[i];
            let w1 = self.coeffs[i + 1];
            let w2 = self.coeffs[i + 2];
            let w3 = self.coeffs[i + 3];

            buf[bi] = w0 as u8;
            buf[bi + 1] = ((w0 >> 8) as u8) | ((w1 << 2) as u8);
            buf[bi + 2] = ((w1 >> 6) as u8) | ((w2 << 4) as u8);
            buf[bi + 3] = ((w2 >> 4) as u8) | ((w3 << 6) as u8);
            buf[bi + 4] = (w3 >> 2) as u8;
            bi += 5;
        }
    }

    fn to_bytes_4(&self, buf: &mut [u8]) {
        debug_assert_eq!(buf.len(), 128);
        let mut bi = 0;

        for i in (0..N).step_by(2) {
            let w0 = 4 - self.coeffs[i];
            let w1 = 4 - self.coeffs[i + 1];

            buf[bi] = (w0 as u8) | ((w1 << 4) as u8);
            bi += 1;
        }
    }

    fn to_bytes_3(&self, buf: &mut [u8]) {
        debug_assert_eq!(buf.len(), 96);
        let mut bi = 0;

        for i in (0..N).step_by(8) {
            let w0 = 2 - self.coeffs[i];
            let w1 = 2 - self.coeffs[i + 1];
            let w2 = 2 - self.coeffs[i + 2];
            let w3 = 2 - self.coeffs[i + 3];
            let w4 = 2 - self.coeffs[i + 4];
            let w5 = 2 - self.coeffs[i + 5];
            let w6 = 2 - self.coeffs[i + 6];
            let w7 = 2 - self.coeffs[i + 7];

            buf[bi] = (w0 as u8) | ((w1 << 3) as u8) | ((w2 << 6) as u8);
            buf[bi + 1] =
                ((w2 >> 2) as u8) | ((w3 << 1) as u8) | ((w4 << 4) as u8) | ((w5 << 7) as u8);
            buf[bi + 2] = ((w5 >> 1) as u8) | ((w6 << 2) as u8) | ((w7 << 5) as u8);
            bi += 3;
        }
    }
}

#[cfg(test)]
#[path = "unit_tests/polynomial_test.rs"]
mod polynomial_test;
