//! Randomness (RC section 13, D-083): counter-based streams and the distributions a `draw`
//! samples.
//!
//! A draw is a pure function of the run's seed, the identity of the event that draws, the
//! draw site's number in that event, the event's due count and the index of the uniform
//! number within the draw (RC-13.5). The generator is Philox4x32-10 (Salmon, Moraes, Dror
//! and Shaw, "Parallel random numbers: as easy as 1, 2, 3", SC 2011).

use prismal_ir::Dist;

/// The draw site of a random event's clock (RC-13.8).
pub const CLOCK_SITE: u32 = u32::MAX;

const M0: u32 = 0xD251_1F53;
const M1: u32 = 0xCD9E_8D57;
const W0: u32 = 0x9E37_79B9;
const W1: u32 = 0xBB67_AE85;

/// Philox4x32 with 10 rounds: four 32-bit words from a counter and a key.
pub fn philox4x32(ctr: [u32; 4], key: [u32; 2]) -> [u32; 4] {
    let (mut c, mut k) = (ctr, key);
    for r in 0..10 {
        if r > 0 {
            k[0] = k[0].wrapping_add(W0);
            k[1] = k[1].wrapping_add(W1);
        }
        let p0 = (M0 as u64) * (c[0] as u64);
        let p1 = (M1 as u64) * (c[2] as u64);
        c = [((p1 >> 32) as u32) ^ c[1] ^ k[0], p1 as u32, ((p0 >> 32) as u32) ^ c[3] ^ k[1], p0 as u32];
    }
    c
}

/// FNV-1a, 64 bits: the stream number of an element identity.
pub fn fnv1a(s: &str) -> u64 {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    for b in s.as_bytes() {
        h ^= *b as u64;
        h = h.wrapping_mul(0x0100_0000_01b3);
    }
    h
}

/// SplitMix64's finalizer: spreads a seed over all 64 bits.
pub fn splitmix64(x: u64) -> u64 {
    let mut z = x.wrapping_add(0x9E37_79B9_7F4A_7C15);
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    z ^ (z >> 31)
}

/// The uniform numbers of one draw: site `site` of the event with stream number `stream`, at
/// its due count `k`, under `seed` (RC-13.5).
pub struct Uniforms {
    key: [u32; 2],
    ctr: [u32; 4],
    block: [u32; 4],
    next: u32,
}

impl Uniforms {
    pub fn new(seed: u64, stream: u64, site: u32, k: u64) -> Uniforms {
        let key = splitmix64(seed) ^ stream;
        Uniforms { key: [key as u32, (key >> 32) as u32], ctr: [k as u32, (k >> 32) as u32, site, 0], block: [0; 4], next: 0 }
    }

    /// The next uniform number in `[0, 1)`, from 53 bits: numbers `2j` and `2j + 1` come from
    /// the words of block `j` (the counter's last word).
    pub fn next(&mut self) -> f64 {
        if self.next.is_multiple_of(2) {
            self.ctr[3] = self.next / 2;
            self.block = philox4x32(self.ctr, self.key);
        }
        let i = (self.next % 2) as usize * 2;
        self.next += 1;
        let bits = ((self.block[i] as u64) << 32 | self.block[i + 1] as u64) >> 11;
        bits as f64 * (1.0 / (1u64 << 53) as f64)
    }

    /// A standard exponential number, by inversion.
    pub fn exp1(&mut self) -> f64 {
        -(1.0 - self.next()).ln()
    }

    /// A standard normal number, by the Box-Muller transform (one of its pair).
    pub fn normal(&mut self) -> f64 {
        let (u1, u2) = (self.next(), self.next());
        (-2.0 * (1.0 - u1).ln()).sqrt() * (2.0 * std::f64::consts::PI * u2).cos()
    }

    /// A Poisson count of mean `m`: by multiplication of uniforms below 30, by Hormann's
    /// transformed rejection with squeeze (PTRS, 1993) from 30.
    pub fn poisson(&mut self, m: f64) -> f64 {
        if m == 0.0 {
            return 0.0;
        }
        if m < 30.0 {
            let limit = (-m).exp();
            let (mut k, mut prod) = (0.0, self.next());
            while prod > limit {
                k += 1.0;
                prod *= self.next();
            }
            return k;
        }
        let smu = m.sqrt();
        let b = 0.931 + 2.53 * smu;
        let a = -0.059 + 0.02483 * b;
        let inv_alpha = 1.1239 + 1.1328 / (b - 3.4);
        let vr = 0.9277 - 3.6224 / (b - 2.0);
        let lm = m.ln();
        loop {
            let u = self.next() - 0.5;
            let v = self.next();
            let us = 0.5 - u.abs();
            let k = ((2.0 * a / us + b) * u + m + 0.43).floor();
            if us >= 0.07 && v <= vr {
                return k;
            }
            if k < 0.0 || (us < 0.013 && v > us) {
                continue;
            }
            if v.ln() + inv_alpha.ln() - (a / (us * us) + b).ln() <= -m + k * lm - ln_factorial(k) {
                return k;
            }
        }
    }

    /// A binomial count of `n` trials of probability `p`: by inversion when `n min(p, 1-p)`
    /// is below 30, by Hormann's BTRS (1993) otherwise.
    pub fn binomial(&mut self, n: f64, p: f64) -> f64 {
        if p > 0.5 {
            return n - self.binomial(n, 1.0 - p);
        }
        if n == 0.0 || p == 0.0 {
            return 0.0;
        }
        let q = 1.0 - p;
        if n * p < 30.0 {
            let s = p / q;
            let a = (n + 1.0) * s;
            'again: loop {
                let mut r = q.powf(n);
                let mut u = self.next();
                let mut x = 0.0;
                while u > r {
                    u -= r;
                    x += 1.0;
                    if x > n {
                        continue 'again;
                    }
                    r *= a / x - s;
                }
                return x;
            }
        }
        let spq = (n * p * q).sqrt();
        let b = 1.15 + 2.53 * spq;
        let a = -0.0873 + 0.0248 * b + 0.01 * p;
        let c = n * p + 0.5;
        let vr = 0.92 - 4.2 / b;
        let alpha = (2.83 + 5.1 / b) * spq;
        let lpq = (p / q).ln();
        let m = ((n + 1.0) * p).floor();
        let h = ln_factorial(m) + ln_factorial(n - m);
        loop {
            let u = self.next() - 0.5;
            let v = self.next();
            let us = 0.5 - u.abs();
            let k = ((2.0 * a / us + b) * u + c).floor();
            if k < 0.0 || k > n {
                continue;
            }
            if us >= 0.07 && v <= vr {
                return k;
            }
            let v = (v * alpha / (a / (us * us) + b)).ln();
            if v <= h - ln_factorial(k) - ln_factorial(n - k) + (k - m) * lpq {
                return k;
            }
        }
    }
}

/// `ln k!` for a whole number `k >= 0`: summed below 20, by Stirling's series from 20.
pub fn ln_factorial(k: f64) -> f64 {
    if k < 20.0 {
        let mut s = 0.0;
        let mut i = 2.0;
        while i <= k {
            s += f64::ln(i);
            i += 1.0;
        }
        return s;
    }
    let k2 = k * k;
    k * k.ln() - k + 0.5 * (2.0 * std::f64::consts::PI * k).ln() + 1.0 / (12.0 * k) - 1.0 / (360.0 * k * k2) + 1.0 / (1260.0 * k * k2 * k2)
}

/// Why the arguments of a draw are not valid, if they are not (D-083).
pub fn invalid_args(d: Dist, args: &[f64]) -> Option<String> {
    let whole = |x: f64| x.fract() == 0.0;
    match d {
        Dist::Uniform if !(args[0] <= args[1]) => Some("`uniform(a, b)` needs a <= b".into()),
        Dist::Normal if !(args[1] >= 0.0) => Some("`normal(m, s)` needs a deviation s >= 0".into()),
        Dist::Exponential if !(args[0] > 0.0) => Some("`exponential(r)` needs a rate r > 0".into()),
        Dist::Bernoulli if !(0.0..=1.0).contains(&args[0]) => Some("`bernoulli(p)` needs p in [0, 1]".into()),
        Dist::Integer if !(whole(args[0]) && whole(args[1]) && args[0] <= args[1]) => Some("`integer(a, b)` needs whole numbers a <= b".into()),
        Dist::Poisson if !(args[0] >= 0.0 && args[0] < 1e9) => Some("`poisson(m)` needs a mean m in [0, 1e9)".into()),
        Dist::Binomial if !(whole(args[0]) && args[0] >= 0.0 && args[0] < 1e15 && (0.0..=1.0).contains(&args[1])) => {
            Some("`binomial(n, p)` needs a whole number n >= 0 and p in [0, 1]".into())
        }
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Known-answer vectors of Random123 (`kat_vectors`, philox4x32 with 10 rounds).
    #[test]
    fn philox_known_answers() {
        assert_eq!(philox4x32([0; 4], [0; 2]), [0x6627e8d5, 0xe169c58d, 0xbc57ac4c, 0x9b00dbd8]);
        assert_eq!(philox4x32([u32::MAX; 4], [u32::MAX; 2]), [0x408f276d, 0x41c83b0e, 0xa20bc7c6, 0x6d5451fd]);
        assert_eq!(
            philox4x32([0x243f6a88, 0x85a308d3, 0x13198a2e, 0x03707344], [0xa4093822, 0x299f31d0]),
            [0xd16cfe09, 0x94fdcceb, 0x5001e420, 0x24126ea1]
        );
    }

    fn mean_var(xs: &[f64]) -> (f64, f64) {
        let n = xs.len() as f64;
        let m = xs.iter().sum::<f64>() / n;
        (m, xs.iter().map(|x| (x - m) * (x - m)).sum::<f64>() / (n - 1.0))
    }

    fn sample(f: impl Fn(&mut Uniforms) -> f64) -> Vec<f64> {
        (0..40_000u64).map(|k| f(&mut Uniforms::new(11, fnv1a("test"), 0, k))).collect()
    }

    /// Means and variances within five standard errors of the exact moments.
    #[test]
    fn moments() {
        let check = |xs: Vec<f64>, mean: f64, var: f64| {
            let (m, v) = mean_var(&xs);
            let n = xs.len() as f64;
            assert!((m - mean).abs() < 5.0 * (var / n).sqrt(), "mean {m}, expected {mean}");
            assert!((v - var).abs() < 0.05 * var, "variance {v}, expected {var}");
        };
        check(sample(|u| u.next()), 0.5, 1.0 / 12.0);
        check(sample(|u| u.exp1()), 1.0, 1.0);
        check(sample(|u| u.normal()), 0.0, 1.0);
        for m in [0.3, 4.0, 29.5, 30.0, 250.0] {
            check(sample(|u| u.poisson(m)), m, m);
        }
        for (n, p) in [(10.0, 0.3), (100.0, 0.25), (1000.0, 0.4), (1000.0, 0.9), (50.0, 0.97)] {
            check(sample(|u| u.binomial(n, p)), n * p, n * p * (1.0 - p));
        }
    }

    #[test]
    fn streams_differ_and_repeat() {
        let a = Uniforms::new(1, fnv1a("e1"), 0, 0).next();
        assert_eq!(a, Uniforms::new(1, fnv1a("e1"), 0, 0).next());
        assert_ne!(a, Uniforms::new(2, fnv1a("e1"), 0, 0).next());
        assert_ne!(a, Uniforms::new(1, fnv1a("e2"), 0, 0).next());
        assert_ne!(a, Uniforms::new(1, fnv1a("e1"), 1, 0).next());
        assert_ne!(a, Uniforms::new(1, fnv1a("e1"), 0, 1).next());
    }

    #[test]
    fn ln_factorial_matches_the_sum_at_the_switch() {
        let exact: f64 = (2..=25).map(|i| (i as f64).ln()).sum();
        assert!((ln_factorial(25.0) - exact).abs() < 1e-11);
    }
}
