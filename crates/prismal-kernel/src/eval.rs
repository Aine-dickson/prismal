//! Values, statuses and evaluation of compiled expressions (MK sections 2, 5, 10).

use prismal_ir::{BinOp, Dist, Func};
use prismal_ir::Func as F;
use std::fmt;
use std::rc::Rc;

/// A fixed-size coordinate array for points and vectors (spaces of dimension 1 to 3).
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Arr {
    pub v: [f64; 3],
    pub n: u8,
}

impl Arr {
    pub fn zeros(n: usize) -> Arr {
        Arr { v: [0.0; 3], n: n as u8 }
    }
    pub fn from_slice(s: &[f64]) -> Arr {
        let mut a = Arr::zeros(s.len());
        a.v[..s.len()].copy_from_slice(s);
        a
    }
    pub fn as_slice(&self) -> &[f64] {
        &self.v[..self.n as usize]
    }
    fn zip(&self, o: &Arr, f: impl Fn(f64, f64) -> f64) -> Arr {
        let mut r = *self;
        for i in 0..self.n as usize {
            r.v[i] = f(self.v[i], o.v[i]);
        }
        r
    }
    fn map(&self, mut f: impl FnMut(f64) -> f64) -> Arr {
        let mut r = *self;
        for i in 0..self.n as usize {
            r.v[i] = f(self.v[i]);
        }
        r
    }
    pub fn norm(&self) -> f64 {
        self.as_slice().iter().map(|x| x * x).sum::<f64>().sqrt()
    }
}

/// A value (MK-2.6: immutable). Quantities are magnitudes in coherent SI units (MK-3.7);
/// instants are seconds.
#[derive(Clone, Debug)]
pub enum Value {
    Bool(bool),
    Num(f64),
    Vec(Arr),
    Point(Arr),
    Tuple(Rc<Vec<Value>>),
    Case(u32),
    /// A function value: its body is evaluated on the bindings current at application (MK-10.7).
    Func(Rc<CExpr>),
}

impl PartialEq for Value {
    /// Bit-for-bit equality, used for replay checks (RC-14.5).
    fn eq(&self, o: &Value) -> bool {
        use Value::*;
        match (self, o) {
            (Bool(a), Bool(b)) => a == b,
            (Num(a), Num(b)) => a.to_bits() == b.to_bits(),
            (Vec(a), Vec(b)) | (Point(a), Point(b)) => {
                a.n == b.n && a.as_slice().iter().zip(b.as_slice()).all(|(x, y)| x.to_bits() == y.to_bits())
            }
            (Tuple(a), Tuple(b)) => a == b,
            (Case(a), Case(b)) => a == b,
            (Func(a), Func(b)) => Rc::ptr_eq(a, b),
            _ => false,
        }
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::Bool(b) => write!(f, "{b}"),
            Value::Num(x) => write!(f, "{x}"),
            Value::Vec(a) | Value::Point(a) => write!(f, "{:?}", a.as_slice()),
            Value::Tuple(t) => {
                write!(f, "(")?;
                for (i, v) in t.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{v}")?;
                }
                write!(f, ")")
            }
            Value::Case(c) => write!(f, "case {c}"),
            Value::Func(_) => write!(f, "<function>"),
        }
    }
}

impl Value {
    pub fn num(&self) -> f64 {
        match self {
            Value::Num(x) => *x,
            other => panic!("expected a number, found {other}"),
        }
    }
    pub fn boolean(&self) -> bool {
        match self {
            Value::Bool(b) => *b,
            other => panic!("expected a Boolean, found {other}"),
        }
    }
    pub fn arr(&self) -> Arr {
        match self {
            Value::Vec(a) | Value::Point(a) => *a,
            other => panic!("expected a vector or point, found {other}"),
        }
    }
    /// Number of real components (1 for a number).
    pub fn width(&self) -> usize {
        match self {
            Value::Num(_) => 1,
            Value::Vec(a) | Value::Point(a) => a.n as usize,
            _ => 0,
        }
    }
    /// Reads the real components into `out`.
    pub fn write_to(&self, out: &mut [f64]) {
        match self {
            Value::Num(x) => out[0] = *x,
            Value::Vec(a) | Value::Point(a) => out.copy_from_slice(a.as_slice()),
            _ => panic!("not a real-valued state"),
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum StatusKind {
    Unknown,
    Unavailable,
    Pending,
    Invalid,
}

/// A status (MK section 5): not a value, with its provenance (MK-5.4).
#[derive(Clone, Debug, PartialEq)]
pub struct Status {
    pub kind: StatusKind,
    pub cause: String,
}

impl Status {
    pub fn invalid(cause: impl Into<String>) -> Status {
        Status { kind: StatusKind::Invalid, cause: cause.into() }
    }
}

pub type EvalResult = Result<Value, Status>;

/// A compiled expression: identities resolved to binding indices, literals in SI units,
/// tuples resolved to vectors where the checker typed them so (D-032).
#[derive(Clone, Debug)]
pub enum CExpr {
    Const(Value),
    Load(usize),
    Arg(usize),
    Time,
    Time0,
    Elapsed,
    /// White noise number `k` of the model (D-090).
    Noise(usize),
    Der(usize),
    Neg(Box<CExpr>),
    Not(Box<CExpr>),
    Norm(Box<CExpr>),
    Bin(BinOp, Box<CExpr>, Box<CExpr>),
    Powi(Box<CExpr>, f64),
    Call(Func, Vec<CExpr>),
    Apply(Box<CExpr>, Vec<CExpr>),
    If(Box<CExpr>, Box<CExpr>, Box<CExpr>),
    MakeVec(Vec<CExpr>),
    MakeTuple(Vec<CExpr>),
    Comp(Box<CExpr>, usize),
    Lambda(Rc<CExpr>),
    Otherwise(Box<CExpr>, Box<CExpr>),
    /// The value of the arm of the scrutinee's case, arms in case order (D-049).
    Match(Box<CExpr>, Vec<CExpr>),
    /// The payload of the occurrence of event `i` being handled (D-050).
    Payload(usize),
    /// The smallest or largest value among the terms whose condition holds; no value when
    /// none holds (D-057).
    Extreme(Func, Vec<(CExpr, CExpr)>),
    /// The item whose number, from 1, the first expression gives; only it is evaluated (D-058).
    Pick(Box<CExpr>, Vec<CExpr>),
    /// A draw (MK-10.4, D-083).
    Draw(Box<CDraw>),
}

/// A compiled draw: its distribution and arguments, and its stream (RC-13.5): the event that
/// draws (`event`, an index into the model's events, and `stream`, the number of its
/// identity) and the draw site's number in that event.
#[derive(Clone, Debug)]
pub struct CDraw {
    pub dist: Dist,
    pub args: Vec<CExpr>,
    pub event: usize,
    pub stream: u64,
    pub site: u32,
}

/// What draws read during event handling: the run's seed and each event's due count
/// (RC-13.5).
#[derive(Clone, Copy, Debug)]
pub struct Draws<'a> {
    pub seed: u64,
    pub due: &'a [u64],
}

/// The environment of an evaluation.
pub struct Ctx<'a> {
    /// Current values of every binding, by index (derived bindings already computed).
    pub vals: &'a [Value],
    /// Combined flows by binding index, where available (MK-11.5).
    pub der: Option<&'a [Value]>,
    pub t: f64,
    pub t0: f64,
    pub args: &'a [Value],
    /// The payloads of the occurrences being handled, by event index (D-050); empty outside
    /// event handling.
    pub payloads: &'a [Option<Value>],
    /// The random streams, during event handling only (D-083).
    pub draws: Option<Draws<'a>>,
    /// The values of the white noises during a stochastic step, by noise site (D-090); empty
    /// elsewhere, where every noise reads 0 and a flow gives its drift.
    pub noise: &'a [f64],
}

fn finite(x: f64, what: &str) -> EvalResult {
    if x.is_finite() {
        Ok(Value::Num(x))
    } else {
        Err(Status::invalid(format!("{what} produced a non-finite result")))
    }
}

fn finite_arr(a: Arr, point: bool, what: &str) -> EvalResult {
    if a.as_slice().iter().all(|x| x.is_finite()) {
        Ok(if point { Value::Point(a) } else { Value::Vec(a) })
    } else {
        Err(Status::invalid(format!("{what} produced a non-finite result")))
    }
}

impl CExpr {
    pub fn eval(&self, c: &Ctx) -> EvalResult {
        use Value::*;
        match self {
            CExpr::Const(v) => Ok(v.clone()),
            CExpr::Load(i) => Ok(c.vals[*i].clone()),
            CExpr::Arg(i) => Ok(c.args[*i].clone()),
            CExpr::Time => Ok(Num(c.t)),
            CExpr::Time0 => Ok(Num(c.t0)),
            CExpr::Elapsed => Ok(Num(c.t - c.t0)),
            CExpr::Noise(k) => Ok(Num(c.noise.get(*k).copied().unwrap_or(0.0))),
            CExpr::Der(i) => match c.der {
                Some(d) => Ok(d[*i].clone()),
                None => Err(Status { kind: StatusKind::Unavailable, cause: "derivative not available here".into() }),
            },
            CExpr::Neg(e) => match e.eval(c)? {
                Num(x) => Ok(Num(-x)),
                Vec(a) => Ok(Vec(a.map(|x| -x))),
                v => Err(Status::invalid(format!("cannot negate {v}"))),
            },
            CExpr::Not(e) => Ok(Bool(!e.eval(c)?.boolean())),
            CExpr::Norm(e) => match e.eval(c)? {
                Num(x) => Ok(Num(x.abs())),
                Vec(a) => Ok(Num(a.norm())),
                v => Err(Status::invalid(format!("no norm of {v}"))),
            },
            CExpr::Bin(op, l, r) => {
                // Short-circuit logic keeps statuses from unused branches out (MK-5.2).
                match op {
                    BinOp::And => {
                        return Ok(Bool(l.eval(c)?.boolean() && r.eval(c)?.boolean()));
                    }
                    BinOp::Or => {
                        return Ok(Bool(l.eval(c)?.boolean() || r.eval(c)?.boolean()));
                    }
                    _ => {}
                }
                let a = l.eval(c)?;
                let b = r.eval(c)?;
                binary(*op, a, b)
            }
            CExpr::Powi(e, p) => finite(e.eval(c)?.num().powf(*p), "power"),
            CExpr::Call(f, args) => {
                let x = args[0].eval(c)?.num();
                let y = if args.len() > 1 { args[1].eval(c)?.num() } else { 0.0 };
                let r = match f {
                    F::Sin => x.sin(),
                    F::Cos => x.cos(),
                    F::Tan => x.tan(),
                    F::Sqrt => {
                        if x < 0.0 {
                            return Err(Status::invalid("sqrt of a negative number"));
                        }
                        x.sqrt()
                    }
                    F::Exp => x.exp(),
                    F::Log => {
                        if x <= 0.0 {
                            return Err(Status::invalid("log of a non-positive number"));
                        }
                        x.ln()
                    }
                    F::Abs => x.abs(),
                    F::Min => x.min(y),
                    F::Max => x.max(y),
                    F::Atan2 => x.atan2(y),
                };
                finite(r, "function")
            }
            CExpr::Apply(f, args) => match f.eval(c)? {
                Func(body) => {
                    let vals: Result<std::vec::Vec<Value>, Status> = args.iter().map(|a| a.eval(c)).collect();
                    let vals = vals?;
                    let inner = Ctx { vals: c.vals, der: c.der, t: c.t, t0: c.t0, args: &vals, payloads: c.payloads, draws: c.draws, noise: c.noise };
                    body.eval(&inner)
                }
                v => Err(Status::invalid(format!("{v} is not a function"))),
            },
            CExpr::If(k, a, b) => {
                if k.eval(c)?.boolean() {
                    a.eval(c)
                } else {
                    b.eval(c)
                }
            }
            CExpr::MakeVec(items) => {
                let mut a = Arr::zeros(items.len());
                for (i, e) in items.iter().enumerate() {
                    a.v[i] = e.eval(c)?.num();
                }
                Ok(Vec(a))
            }
            CExpr::MakeTuple(items) => {
                let vals: Result<std::vec::Vec<Value>, Status> = items.iter().map(|a| a.eval(c)).collect();
                Ok(Tuple(Rc::new(vals?)))
            }
            CExpr::Comp(e, axis) => match e.eval(c)? {
                Vec(a) | Point(a) => Ok(Num(a.v[*axis])),
                Tuple(t) => Ok(t[*axis].clone()),
                v => Err(Status::invalid(format!("no component {axis} of {v}"))),
            },
            CExpr::Lambda(body) => Ok(Func(body.clone())),
            CExpr::Otherwise(e, d) => match e.eval(c) {
                Ok(v) => Ok(v),
                Err(_) => d.eval(c),
            },
            CExpr::Pick(k, items) => {
                let k = k.eval(c)?.num();
                match items.get((k as usize).wrapping_sub(1)) {
                    Some(x) if k.fract() == 0.0 => x.eval(c),
                    _ => Err(Status::invalid(format!("no member number {k}"))),
                }
            }
            CExpr::Extreme(f, terms) => {
                let mut best: Option<f64> = None;
                for (w, v) in terms {
                    if w.eval(c)?.boolean() {
                        let x = v.eval(c)?.num();
                        best = Some(match (best, f) {
                            (None, _) => x,
                            (Some(b), F::Min) => b.min(x),
                            (Some(b), _) => b.max(x),
                        });
                    }
                }
                best.map(Num).ok_or_else(|| Status::invalid(format!("`{}` over no member has no value", if *f == F::Min { "min" } else { "max" })))
            }
            CExpr::Draw(d) => draw(d, c),
            CExpr::Payload(i) => c.payloads.get(*i).cloned().flatten().ok_or_else(|| Status::invalid("no payload for this occurrence".to_string())),
            CExpr::Match(e, arms) => match e.eval(c)? {
                Case(i) => arms.get(i as usize).ok_or_else(|| Status::invalid(format!("no arm for case {i}")))?.eval(c),
                v => Err(Status::invalid(format!("{v} is not a case"))),
            },
        }
    }
}

fn binary(op: BinOp, a: Value, b: Value) -> EvalResult {
    use Value::*;
    let what = "arithmetic";
    match (op, a, b) {
        (BinOp::Add, Num(x), Num(y)) => finite(x + y, what),
        (BinOp::Sub, Num(x), Num(y)) => finite(x - y, what),
        (BinOp::Mul, Num(x), Num(y)) => finite(x * y, what),
        (BinOp::Div, Num(x), Num(y)) => {
            if y == 0.0 {
                Err(Status::invalid("division by zero"))
            } else {
                finite(x / y, what)
            }
        }
        (BinOp::Pow, Num(x), Num(y)) => finite(x.powf(y), "power"),
        (BinOp::Add, Vec(a), Vec(b)) => finite_arr(a.zip(&b, |x, y| x + y), false, what),
        (BinOp::Sub, Vec(a), Vec(b)) => finite_arr(a.zip(&b, |x, y| x - y), false, what),
        (BinOp::Add, Point(a), Vec(b)) | (BinOp::Add, Vec(b), Point(a)) => finite_arr(a.zip(&b, |x, y| x + y), true, what),
        (BinOp::Sub, Point(a), Vec(b)) => finite_arr(a.zip(&b, |x, y| x - y), true, what),
        (BinOp::Sub, Point(a), Point(b)) => finite_arr(a.zip(&b, |x, y| x - y), false, what),
        (BinOp::Mul, Num(k), Vec(a)) | (BinOp::Mul, Vec(a), Num(k)) => finite_arr(a.map(|x| k * x), false, what),
        (BinOp::Div, Vec(a), Num(k)) => {
            if k == 0.0 {
                Err(Status::invalid("division by zero"))
            } else {
                finite_arr(a.map(|x| x / k), false, what)
            }
        }
        (BinOp::Lt, Num(x), Num(y)) => Ok(Bool(x < y)),
        (BinOp::Le, Num(x), Num(y)) => Ok(Bool(x <= y)),
        (BinOp::Gt, Num(x), Num(y)) => Ok(Bool(x > y)),
        (BinOp::Ge, Num(x), Num(y)) => Ok(Bool(x >= y)),
        (BinOp::Eq, a, b) => Ok(Bool(value_eq(&a, &b))),
        (BinOp::Ne, a, b) => Ok(Bool(!value_eq(&a, &b))),
        (op, a, b) => Err(Status::invalid(format!("cannot apply {op:?} to {a} and {b}"))),
    }
}

/// Numeric equality (not bitwise): `-0.0 == 0.0`.
fn value_eq(a: &Value, b: &Value) -> bool {
    use Value::*;
    match (a, b) {
        (Num(x), Num(y)) => x == y,
        (Vec(x), Vec(y)) | (Point(x), Point(y)) => x.as_slice() == y.as_slice(),
        (Bool(x), Bool(y)) => x == y,
        (Case(x), Case(y)) => x == y,
        (Tuple(x), Tuple(y)) => x.len() == y.len() && x.iter().zip(y.iter()).all(|(p, q)| value_eq(p, q)),
        _ => false,
    }
}

/// The distance used by tolerances: absolute difference of numbers, norm of vector differences.
pub fn distance(a: &Value, b: &Value) -> Option<f64> {
    match (a, b) {
        (Value::Num(x), Value::Num(y)) => Some((x - y).abs()),
        (Value::Vec(x), Value::Vec(y)) | (Value::Point(x), Value::Point(y)) => {
            Some(x.zip(y, |p, q| p - q).norm())
        }
        _ => None,
    }
}

/// Samples a draw from its stream (RC-13.5 to RC-13.7).
fn draw(d: &CDraw, c: &Ctx) -> EvalResult {
    use crate::random::{invalid_args, Uniforms};
    let Some(dr) = c.draws else {
        return Err(Status::invalid("a draw outside the handling of an event"));
    };
    let mut u = Uniforms::new(dr.seed, d.stream, d.site, dr.due[d.event]);
    if d.dist == Dist::Choice {
        let i = ((u.next() * d.args.len() as f64) as usize).min(d.args.len() - 1);
        return d.args[i].eval(c);
    }
    let vals = d.args.iter().map(|a| a.eval(c)).collect::<Result<std::vec::Vec<Value>, Status>>()?;
    if let (Dist::Normal, Value::Vec(m) | Value::Point(m)) = (d.dist, &vals[0]) {
        let s = vals[1].num();
        if let Some(why) = invalid_args(d.dist, &[0.0, s]) {
            return Err(Status::invalid(why));
        }
        let a = m.map(|x| x + s * u.normal());
        return Ok(if matches!(vals[0], Value::Point(_)) { Value::Point(a) } else { Value::Vec(a) });
    }
    let x: std::vec::Vec<f64> = vals.iter().map(|v| v.num()).collect();
    if let Some(why) = invalid_args(d.dist, &x) {
        return Err(Status::invalid(why));
    }
    let r = match d.dist {
        Dist::Uniform => x[0] + (x[1] - x[0]) * u.next(),
        Dist::Normal => x[0] + x[1] * u.normal(),
        Dist::Exponential => u.exp1() / x[0],
        Dist::Bernoulli => return Ok(Value::Bool(u.next() < x[0])),
        Dist::Integer => (x[0] + (u.next() * (x[1] - x[0] + 1.0)).floor()).min(x[1]),
        Dist::Poisson => u.poisson(x[0]),
        Dist::Binomial => u.binomial(x[0], x[1]),
        Dist::Choice => unreachable!(),
    };
    finite(r, "draw")
}
