//! Instruments (PK-3.7, D-087): readings rounded to a resolution, limited to a range and
//! disturbed by noise drawn from the run's seed.

use crate::constant;
use prismal_ir::present::Instrument;
use prismal_ir::{Dim, Expr, Id, Type};
use prismal_kernel::random::{fnv1a, Uniforms};
use prismal_kernel::{Arr, CModel, Value};

/// An instrument compiled for one source: its options as numbers in base units.
#[derive(Clone, Debug)]
pub struct Meter {
    pub name: String,
    resolution: Option<f64>,
    range: Option<(f64, f64)>,
    noise: Option<f64>,
    /// The noise stream: the instrument's identity and the source it reads (RC-13.5a).
    stream: u64,
}

/// The instrument `id` among `instruments`.
pub fn find<'i>(instruments: &'i [Instrument], id: &Id) -> Result<&'i Instrument, String> {
    instruments.iter().find(|i| &i.id == id).ok_or(format!("unknown instrument `{id}`"))
}

impl Meter {
    /// Compiles `inst` for values of type `ty` read from `source`. Each option is a constant
    /// with the dimension of the values: numbers and quantities take every option, vectors
    /// and points (lengths) resolution and noise, per component.
    pub fn new(cm: &CModel, inst: &Instrument, source: &Expr, ty: &Type) -> Result<Meter, String> {
        let (dim, scalar) = match ty {
            Type::Quantity { dim } => (*dim, true),
            Type::Vector { dim, .. } => (*dim, false),
            Type::Point { .. } => (Dim::length(), false),
            _ => return Err(format!("instrument `{}` reads numbers, quantities, vectors and points", inst.name)),
        };
        let option = |what: &str, e: &Expr| -> Result<f64, String> {
            match prismal_kernel::compile_expr(cm, e, None) {
                Ok((_, Type::Quantity { dim: d })) if d == dim => {}
                Ok((_, t)) => return Err(format!("the {what} of instrument `{}` is {t:?}; the values it reads have dimension {dim}", inst.name)),
                Err(ds) => return Err(ds.iter().map(|d| d.message.clone()).collect::<Vec<_>>().join("; ")),
            }
            match constant(cm, e) {
                Ok(Value::Num(x)) => Ok(x),
                Ok(v) => Err(format!("the {what} of instrument `{}` is {v}, not a number", inst.name)),
                Err(m) => Err(format!("the {what} of instrument `{}` is a constant: {m}", inst.name)),
            }
        };
        let resolution = inst.resolution.as_ref().map(|e| option("resolution", e)).transpose()?;
        let noise = inst.noise.as_ref().map(|e| option("noise", e)).transpose()?;
        let range = match &inst.range {
            Some(_) if !scalar => return Err(format!("instrument `{}` has a range; a range limits numbers and quantities, not vectors or points", inst.name)),
            Some((a, b)) => Some((option("range", a)?, option("range", b)?)),
            None => None,
        };
        if resolution.is_some_and(|r| !(r > 0.0)) {
            return Err(format!("the resolution of instrument `{}` is positive", inst.name));
        }
        if noise.is_some_and(|s| !(s >= 0.0)) {
            return Err(format!("the noise of instrument `{}` is a deviation, at least 0", inst.name));
        }
        if range.is_some_and(|(a, b)| !(a < b)) {
            return Err(format!("the range of instrument `{}` goes from low to high", inst.name));
        }
        let stream = fnv1a(&format!("{}|{source:?}", inst.id));
        Ok(Meter { name: inst.name.clone(), resolution, range, noise, stream })
    }

    /// The standard uncertainty of a reading: the noise and the resolution's rounding (a
    /// uniform error over one step) combined, `sqrt(noise^2 + resolution^2 / 12)`.
    pub fn uncertainty(&self) -> f64 {
        let (s, r) = (self.noise.unwrap_or(0.0), self.resolution.unwrap_or(0.0));
        (s * s + r * r / 12.0).sqrt()
    }

    /// The reading of `v` at instant `t` in a run with `seed`, or `None` when `v` lies outside
    /// the range. The noise of a reading depends only on the seed, the instrument, the source
    /// and the instant, so the same reading is shown wherever it is taken (RC-13.5a).
    pub fn read(&self, v: &Value, seed: u64, t: f64) -> Option<Value> {
        if let (Some((lo, hi)), Value::Num(x)) = (self.range, v) {
            if !(*x >= lo && *x <= hi) {
                return None;
            }
        }
        let mut u = Uniforms::new(seed, self.stream, 0, t.to_bits());
        let mut one = |x: f64| {
            let x = match self.noise {
                Some(s) if s > 0.0 => x + s * u.normal(),
                _ => x,
            };
            match self.resolution {
                // A step that divides one (0.1, 0.01, 0.5) is applied by dividing, so that
                // 17 steps of 0.1 read 1.7 exactly, as the step's decimal digits say.
                Some(r) => {
                    let (k, inv) = ((x / r).round(), (1.0 / r).round());
                    if inv >= 1.0 && (inv * r - 1.0).abs() < 1e-12 {
                        k / inv
                    } else {
                        k * r
                    }
                }
                None => x,
            }
        };
        Some(match v {
            Value::Num(x) => Value::Num(one(*x)),
            Value::Vec(a) => Value::Vec(Arr::from_slice(&a.as_slice().iter().map(|x| one(*x)).collect::<Vec<_>>())),
            Value::Point(a) => Value::Point(Arr::from_slice(&a.as_slice().iter().map(|x| one(*x)).collect::<Vec<_>>())),
            other => other.clone(),
        })
    }

    /// Words for text alternatives: "measured with ruler (uncertainty 0.0021)".
    pub fn describe(&self) -> String {
        format!("measured with {} (uncertainty {})", self.name, crate::text::fmt_num(self.uncertainty()))
    }
}

/// The meter for an instrument reference, if any, reading `source` of type `ty`.
pub fn meter(cm: &CModel, instruments: &[Instrument], id: Option<&Id>, source: &Expr, ty: &Type) -> Result<Option<Meter>, String> {
    id.map(|id| Meter::new(cm, find(instruments, id)?, source, ty)).transpose()
}
