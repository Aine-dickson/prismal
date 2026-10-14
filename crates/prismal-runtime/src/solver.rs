//! Solvers (RC section 6): classical RK4 with cubic Hermite dense output, and adaptive
//! Dormand-Prince 5(4) with its fourth-order dense output (Hairer, Norsett and Wanner).

use prismal_kernel::Status;

/// The right-hand side of the ODE between two event instants.
pub trait Rhs {
    fn f(&mut self, t: f64, y: &[f64], dy: &mut [f64]) -> Result<(), Status>;
}

/// Dense output over one accepted step (RC-6.2).
#[derive(Clone, Debug)]
pub enum Dense {
    Hermite { y0: Vec<f64>, y1: Vec<f64>, f0: Vec<f64>, f1: Vec<f64> },
    Dopri { r: [Vec<f64>; 5] },
    /// An Euler-Maruyama step (RC-13.9): the straight line between its two states, the
    /// mean of the Brownian bridge.
    Linear { y0: std::rc::Rc<Vec<f64>>, y1: std::rc::Rc<Vec<f64>> },
}

/// One accepted step. `t1` may be truncated at an event (RC-7.6); the interpolant stays the
/// one built for the full step of length `h_full`.
#[derive(Clone, Debug)]
pub struct Step {
    pub t0: f64,
    pub t1: f64,
    pub dense: Dense,
    h_full: f64,
}

impl Step {
    /// The interpolated state at `t` in `[t0, t1]` of the original step.
    pub fn eval(&self, t: f64, out: &mut [f64]) {
        let h = self.dense_h();
        let th = if h > 0.0 { (t - self.t0) / h } else { 0.0 };
        match &self.dense {
            Dense::Hermite { y0, y1, f0, f1 } => {
                let (t2, t3) = (th * th, th * th * th);
                let h00 = 2.0 * t3 - 3.0 * t2 + 1.0;
                let h10 = t3 - 2.0 * t2 + th;
                let h01 = -2.0 * t3 + 3.0 * t2;
                let h11 = t3 - t2;
                for i in 0..out.len() {
                    out[i] = h00 * y0[i] + h10 * h * f0[i] + h01 * y1[i] + h11 * h * f1[i];
                }
            }
            Dense::Dopri { r } => {
                let th1 = 1.0 - th;
                for i in 0..out.len() {
                    out[i] = r[0][i] + th * (r[1][i] + th1 * (r[2][i] + th * (r[3][i] + th1 * r[4][i])));
                }
            }
            Dense::Linear { y0, y1 } => {
                for i in 0..out.len() {
                    out[i] = y0[i] + th * (y1[i] - y0[i]);
                }
            }
        }
    }

    fn dense_h(&self) -> f64 {
        self.h_full
    }

    pub fn new(t0: f64, h: f64, dense: Dense) -> Step {
        Step { t0, t1: t0 + h, dense, h_full: h }
    }
}

pub struct Rk4;

impl Rk4 {
    /// One RK4 step of size `h` from `(t, y)`. Returns the new state and its dense output.
    pub fn step(rhs: &mut dyn Rhs, t: f64, y: &[f64], h: f64) -> Result<(Vec<f64>, Step), Status> {
        let n = y.len();
        let mut k1 = vec![0.0; n];
        let mut k2 = vec![0.0; n];
        let mut k3 = vec![0.0; n];
        let mut k4 = vec![0.0; n];
        let mut tmp = vec![0.0; n];
        rhs.f(t, y, &mut k1)?;
        for i in 0..n {
            tmp[i] = y[i] + 0.5 * h * k1[i];
        }
        rhs.f(t + 0.5 * h, &tmp, &mut k2)?;
        for i in 0..n {
            tmp[i] = y[i] + 0.5 * h * k2[i];
        }
        rhs.f(t + 0.5 * h, &tmp, &mut k3)?;
        for i in 0..n {
            tmp[i] = y[i] + h * k3[i];
        }
        rhs.f(t + h, &tmp, &mut k4)?;
        let mut y1 = vec![0.0; n];
        for i in 0..n {
            y1[i] = y[i] + h / 6.0 * (k1[i] + 2.0 * k2[i] + 2.0 * k3[i] + k4[i]);
        }
        let mut f1 = vec![0.0; n];
        rhs.f(t + h, &y1, &mut f1)?;
        let dense = Dense::Hermite { y0: y.to_vec(), y1: y1.clone(), f0: k1, f1 };
        Ok((y1, Step::new(t, h, dense)))
    }
}

// Dormand-Prince 5(4) coefficients.
const C2: f64 = 1.0 / 5.0;
const C3: f64 = 3.0 / 10.0;
const C4: f64 = 4.0 / 5.0;
const C5: f64 = 8.0 / 9.0;
const A21: f64 = 1.0 / 5.0;
const A31: f64 = 3.0 / 40.0;
const A32: f64 = 9.0 / 40.0;
const A41: f64 = 44.0 / 45.0;
const A42: f64 = -56.0 / 15.0;
const A43: f64 = 32.0 / 9.0;
const A51: f64 = 19372.0 / 6561.0;
const A52: f64 = -25360.0 / 2187.0;
const A53: f64 = 64448.0 / 6561.0;
const A54: f64 = -212.0 / 729.0;
const A61: f64 = 9017.0 / 3168.0;
const A62: f64 = -355.0 / 33.0;
const A63: f64 = 46732.0 / 5247.0;
const A64: f64 = 49.0 / 176.0;
const A65: f64 = -5103.0 / 18656.0;
const A71: f64 = 35.0 / 384.0;
const A73: f64 = 500.0 / 1113.0;
const A74: f64 = 125.0 / 192.0;
const A75: f64 = -2187.0 / 6784.0;
const A76: f64 = 11.0 / 84.0;
const E1: f64 = 71.0 / 57600.0;
const E3: f64 = -71.0 / 16695.0;
const E4: f64 = 71.0 / 1920.0;
const E5: f64 = -17253.0 / 339200.0;
const E6: f64 = 22.0 / 525.0;
const E7: f64 = -1.0 / 40.0;
const D1: f64 = -12715105075.0 / 11282082432.0;
const D3: f64 = 87487479700.0 / 32700410799.0;
const D4: f64 = -10690763975.0 / 1880347072.0;
const D5: f64 = 701980252875.0 / 199316789632.0;
const D6: f64 = -1453857185.0 / 822651844.0;
const D7: f64 = 69997945.0 / 29380423.0;

pub struct Dopri5 {
    pub rtol: f64,
    pub atol: f64,
    pub h_max: f64,
}

pub enum Attempt {
    Accepted { y1: Vec<f64>, step: Step, h_next: f64 },
    Rejected { h_next: f64 },
}

impl Dopri5 {
    fn norm(&self, y0: &[f64], y1: &[f64], e: &[f64]) -> f64 {
        let n = e.len().max(1) as f64;
        let s: f64 = (0..e.len())
            .map(|i| {
                let sk = self.atol + self.rtol * y0[i].abs().max(y1[i].abs());
                (e[i] / sk).powi(2)
            })
            .sum();
        (s / n).sqrt()
    }

    /// Initial step size from the state alone (Hairer's algorithm), so that the step after an
    /// event depends only on the post-event state (RC-8.6).
    ///
    /// Also returns the step limit below which the result depends on `h_limit`: with any
    /// limit above it, the same step is chosen (snapshots rely on this, RC-14.2).
    pub fn initial_step(&self, rhs: &mut dyn Rhs, t: f64, y: &[f64], h_limit: f64) -> Result<(f64, f64), Status> {
        let n = y.len();
        if n == 0 {
            return Ok((h_limit.min(self.h_max), self.h_max));
        }
        let mut f0 = vec![0.0; n];
        rhs.f(t, y, &mut f0)?;
        let sk: Vec<f64> = y.iter().map(|v| self.atol + self.rtol * v.abs()).collect();
        let rms = |v: &[f64]| ((0..n).map(|i| (v[i] / sk[i]).powi(2)).sum::<f64>() / n as f64).sqrt();
        let d0 = rms(y);
        let d1 = rms(&f0);
        let mut h0 = if d0 < 1e-5 || d1 < 1e-5 { 1e-6 } else { 0.01 * d0 / d1 };
        h0 = h0.min(self.h_max);
        let free0 = h0;
        h0 = h0.min(h_limit);
        let y1: Vec<f64> = (0..n).map(|i| y[i] + h0 * f0[i]).collect();
        let mut f1 = vec![0.0; n];
        rhs.f(t + h0, &y1, &mut f1)?;
        let diff: Vec<f64> = (0..n).map(|i| f1[i] - f0[i]).collect();
        let d2 = rms(&diff) / h0;
        let m = d1.max(d2);
        let h1 = if m <= 1e-15 { (h0 * 1e-3).max(1e-6) } else { (0.01 / m).powf(0.2) };
        let free = (100.0 * h0).min(h1).min(self.h_max);
        Ok((free.min(h_limit), free.max(free0)))
    }

    /// Attempts one step of size `h`.
    pub fn attempt(&self, rhs: &mut dyn Rhs, t: f64, y: &[f64], h: f64) -> Result<Attempt, Status> {
        let n = y.len();
        let mut k = [(); 7].map(|_| vec![0.0; n]);
        let mut tmp = vec![0.0; n];
        rhs.f(t, y, &mut k[0])?;
        for i in 0..n {
            tmp[i] = y[i] + h * A21 * k[0][i];
        }
        rhs.f(t + C2 * h, &tmp, &mut k[1])?;
        for i in 0..n {
            tmp[i] = y[i] + h * (A31 * k[0][i] + A32 * k[1][i]);
        }
        rhs.f(t + C3 * h, &tmp, &mut k[2])?;
        for i in 0..n {
            tmp[i] = y[i] + h * (A41 * k[0][i] + A42 * k[1][i] + A43 * k[2][i]);
        }
        rhs.f(t + C4 * h, &tmp, &mut k[3])?;
        for i in 0..n {
            tmp[i] = y[i] + h * (A51 * k[0][i] + A52 * k[1][i] + A53 * k[2][i] + A54 * k[3][i]);
        }
        rhs.f(t + C5 * h, &tmp, &mut k[4])?;
        for i in 0..n {
            tmp[i] = y[i] + h * (A61 * k[0][i] + A62 * k[1][i] + A63 * k[2][i] + A64 * k[3][i] + A65 * k[4][i]);
        }
        rhs.f(t + h, &tmp, &mut k[5])?;
        let mut y1 = vec![0.0; n];
        for i in 0..n {
            y1[i] = y[i] + h * (A71 * k[0][i] + A73 * k[2][i] + A74 * k[3][i] + A75 * k[4][i] + A76 * k[5][i]);
        }
        rhs.f(t + h, &y1, &mut k[6])?;
        let e: Vec<f64> = (0..n)
            .map(|i| h * (E1 * k[0][i] + E3 * k[2][i] + E4 * k[3][i] + E5 * k[4][i] + E6 * k[5][i] + E7 * k[6][i]))
            .collect();
        let err = self.norm(y, &y1, &e);
        if err <= 1.0 {
            let fac = if err == 0.0 { 10.0 } else { (0.9 * err.powf(-0.2)).clamp(0.2, 10.0) };
            let r0 = y.to_vec();
            let r1: Vec<f64> = (0..n).map(|i| y1[i] - y[i]).collect();
            let r2: Vec<f64> = (0..n).map(|i| h * k[0][i] - r1[i]).collect();
            let r3: Vec<f64> = (0..n).map(|i| r1[i] - h * k[6][i] - r2[i]).collect();
            let r4: Vec<f64> = (0..n)
                .map(|i| h * (D1 * k[0][i] + D3 * k[2][i] + D4 * k[3][i] + D5 * k[4][i] + D6 * k[5][i] + D7 * k[6][i]))
                .collect();
            let step = Step::new(t, h, Dense::Dopri { r: [r0, r1, r2, r3, r4] });
            Ok(Attempt::Accepted { y1, step, h_next: (h * fac).min(self.h_max) })
        } else {
            let fac = (0.9 * err.powf(-0.2)).clamp(0.2, 1.0);
            Ok(Attempt::Rejected { h_next: h * fac })
        }
    }
}
