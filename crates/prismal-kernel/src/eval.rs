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
    /// A field's value in each cell of its grid (MK-9.4, D-091).
    Field(Rc<Vec<f64>>),
    /// A derived binding with no value, and why (MK-5.2, D-106): reading it gives the status.
    Absent(Rc<Status>),
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
            (Field(a), Field(b)) => a.len() == b.len() && a.iter().zip(b.iter()).all(|(x, y)| x.to_bits() == y.to_bits()),
            (Absent(a), Absent(b)) => a == b,
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
            Value::Field(d) => write!(f, "<field of {} cells>", d.len()),
            Value::Absent(s) => write!(f, "<not available: {}>", s.cause),
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
            Value::Field(d) => d.len(),
            _ => 0,
        }
    }
    /// Reads the real components into `out`.
    pub fn write_to(&self, out: &mut [f64]) {
        match self {
            Value::Num(x) => out[0] = *x,
            Value::Vec(a) | Value::Point(a) => out.copy_from_slice(a.as_slice()),
            Value::Field(d) => out.copy_from_slice(d),
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
    /// With values (D-106): the value whose term is the smallest.
    Extreme(Func, Vec<(CExpr, CExpr)>, Vec<CExpr>),
    /// The item whose number, from 1, the first expression gives; only it is evaluated (D-058).
    Pick(Box<CExpr>, Vec<CExpr>),
    /// A draw (MK-10.4, D-083).
    Draw(Box<CDraw>),
    /// The value of field `i` in the cell of a pointwise evaluation (MK-9.7, D-091).
    FieldAt(usize),
    /// An operation on a field (MK-9.7, MK-9.8, D-091).
    Field(Box<CFieldOp>),
    /// A component read in a declared frame (MK-4.6, D-105).
    InFrame(Box<CInFrame>),
}

/// `e.in(F).k` compiled (D-105): the value read (a point, measured from the frame's origin,
/// or a vector), the frame's origin and axis expressions, the component and the dimension of
/// the space.
#[derive(Clone, Debug)]
pub struct CInFrame {
    pub value: CExpr,
    pub point: bool,
    pub at: CExpr,
    pub x: CExpr,
    pub y: Option<CExpr>,
    pub axis: usize,
    pub n: usize,
}

impl CInFrame {
    /// The frame's origin and unit axes on a state (D-105).
    pub fn axes(&self, c: &Ctx) -> Result<([f64; 3], [[f64; 3]; 3]), Status> {
        let y = match &self.y {
            Some(y) => Some(y.eval(c)?),
            None => None,
        };
        frame_axes(self.at.eval(c)?, self.x.eval(c)?, y, self.n)
    }
}

/// A frame's origin and unit axes from the values of its expressions (D-105): the first axis
/// along `x`; in a plane the second the first turned a quarter turn; in three dimensions the
/// second `y` made square to the first and the third their cross product.
pub fn frame_axes(at: Value, x: Value, y: Option<Value>, n: usize) -> Result<([f64; 3], [[f64; 3]; 3]), Status> {
    {
        let three = |v: Value| -> Result<[f64; 3], Status> {
            match v {
                Value::Vec(a) | Value::Point(a) => Ok(a.v),
                Value::Tuple(t) => {
                    let mut out = [0.0; 3];
                    for (k, x) in t.iter().take(3).enumerate() {
                        let Value::Num(x) = x else { return Err(Status::invalid("a frame's axis has components that are not numbers")) };
                        out[k] = *x;
                    }
                    Ok(out)
                }
                other => Err(Status::invalid(format!("a frame's axis is a vector, not {other}"))),
            }
        };
        let dot = |a: &[f64; 3], b: &[f64; 3]| a[0] * b[0] + a[1] * b[1] + a[2] * b[2];
        let unit = |a: [f64; 3]| -> Result<[f64; 3], Status> {
            let n = dot(&a, &a).sqrt();
            if !(n > 0.0) || !n.is_finite() {
                return Err(Status::invalid("a frame's axis has no direction"));
            }
            Ok([a[0] / n, a[1] / n, a[2] / n])
        };
        let at = three(at)?;
        let e1 = unit(three(x)?)?;
        let axes = if n < 3 {
            [e1, [-e1[1], e1[0], 0.0], [0.0, 0.0, 1.0]]
        } else {
            let y = match y {
                Some(y) => three(y)?,
                None => return Err(Status::invalid("a frame of three dimensions needs `y`")),
            };
            let d = dot(&y, &e1);
            let e2 = unit([y[0] - d * e1[0], y[1] - d * e1[1], y[2] - d * e1[2]])?;
            let e3 = [e1[1] * e2[2] - e1[2] * e2[1], e1[2] * e2[0] - e1[0] * e2[2], e1[0] * e2[1] - e1[1] * e2[0]];
            [e1, e2, e3]
        };
        Ok((at, axes))
    }
}

/// One axis of a grid: `n` equal cells of width `h` from `lo`, cell `i` centred at
/// `lo + (i + 1/2) h` (MK-9.5, D-091).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct GridAxis {
    pub lo: f64,
    pub h: f64,
    pub n: usize,
}

impl GridAxis {
    pub fn hi(&self) -> f64 {
        self.lo + self.h * self.n as f64
    }
    pub fn center(&self, i: usize) -> f64 {
        self.lo + (i as f64 + 0.5) * self.h
    }
}

/// A grid compiled (MK-9.5, D-091): one axis over an interval of a quantity of dimension
/// `dim`, or two over a rectangle of the model's plane (`dim` length). Cells are numbered
/// along `x` first: cell `(i, j)` is `i + nx j`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CGrid {
    pub x: GridAxis,
    pub y: Option<GridAxis>,
    pub dim: prismal_ir::Dim,
}

impl CGrid {
    pub fn cells(&self) -> usize {
        self.x.n * self.y.map_or(1, |a| a.n)
    }
    /// The coordinate of cell `k`: a number on a line, a point in the plane.
    pub fn coord(&self, k: usize) -> Value {
        match self.y {
            None => Value::Num(self.x.center(k)),
            Some(y) => Value::Point(Arr::from_slice(&[self.x.center(k % self.x.n), y.center(k / self.x.n)])),
        }
    }
}

/// A field's condition on one edge (MK-9.6, D-091).
#[derive(Clone, Debug)]
pub enum CEdge {
    /// Held at a value.
    Fixed(CExpr),
    /// No flux across the edge: the cell beyond mirrors the cell inside.
    Insulated,
    /// The grid wraps around.
    Periodic,
}

#[derive(Clone, Debug)]
pub enum FieldOp {
    Laplacian,
    Gradient,
    /// The field read at a coordinate: linear (bilinear in the plane) between cell centres
    /// and the values on the edges.
    Sample(CExpr),
    Integral,
    Mean,
    Min,
    Max,
}

/// An operation on field `binding` over `grid`, with its edges: left, right, bottom, top.
#[derive(Clone, Debug)]
pub struct CFieldOp {
    pub op: FieldOp,
    pub binding: usize,
    pub grid: CGrid,
    pub edges: [CEdge; 4],
}

/// The cells of a field as a lookup that reaches one step beyond each edge (D-091): index
/// `-1` and `n` on an axis are the edges themselves for readings, and ghost cells for
/// differences.
struct Cells<'a> {
    op: &'a CFieldOp,
    u: &'a [f64],
    c: &'a Ctx<'a>,
    nx: isize,
    ny: isize,
}

impl Cells<'_> {
    /// The value of cell `(i, j)`; beyond an edge, the edge's value (`ghost` false) or the
    /// ghost cell (`ghost` true): `2 g - inside` for an edge held at `g`, the inside cell
    /// for an insulated edge, the far cell for a periodic grid.
    fn at(&self, i: isize, j: isize, ghost: bool) -> Result<f64, Status> {
        let (nx, ny) = (self.nx, self.ny);
        if i < 0 || i >= nx {
            let side = if i < 0 { 0 } else { 1 };
            let (inside, far) = if i < 0 { (0, nx - 1) } else { (nx - 1, 0) };
            let a = self.at(inside, j, ghost)?;
            return Ok(match &self.op.edges[side] {
                CEdge::Fixed(g) if ghost => 2.0 * g.eval(self.c)?.num() - a,
                CEdge::Fixed(g) => g.eval(self.c)?.num(),
                CEdge::Insulated => a,
                CEdge::Periodic if ghost => self.at(far, j, ghost)?,
                CEdge::Periodic => 0.5 * (a + self.at(far, j, ghost)?),
            });
        }
        if j < 0 || j >= ny {
            let side = if j < 0 { 2 } else { 3 };
            let (inside, far) = if j < 0 { (0, ny - 1) } else { (ny - 1, 0) };
            let a = self.at(i, inside, ghost)?;
            return Ok(match &self.op.edges[side] {
                CEdge::Fixed(g) if ghost => 2.0 * g.eval(self.c)?.num() - a,
                CEdge::Fixed(g) => g.eval(self.c)?.num(),
                CEdge::Insulated => a,
                CEdge::Periodic if ghost => self.at(i, far, ghost)?,
                CEdge::Periodic => 0.5 * (a + self.at(i, far, ghost)?),
            });
        }
        Ok(self.u[(i + nx * j) as usize])
    }
}

/// Where coordinate `x` falls on an axis: the two indices around it (`-1` and `n` stand for
/// the edges, half a cell beyond the outer centres) and the weight of the second.
fn bracket(a: &GridAxis, x: f64) -> (isize, isize, f64) {
    let n = a.n as isize;
    let s = (x - a.lo) / a.h - 0.5;
    let pos = |i: isize| if i < 0 { -0.5 } else if i >= n { n as f64 - 0.5 } else { i as f64 };
    let i0 = (s.floor() as isize).clamp(-1, n - 1);
    let i0 = if s < 0.0 { -1 } else { i0 };
    let i1 = if s < 0.0 { 0 } else if i0 == n - 1 { n } else { i0 + 1 };
    let (p0, p1) = (pos(i0), pos(i1));
    let w = if p1 > p0 { ((s - p0) / (p1 - p0)).clamp(0.0, 1.0) } else { 0.0 };
    (i0, i1, w)
}

impl CFieldOp {
    fn cells<'a>(&'a self, u: &'a [f64], c: &'a Ctx<'a>) -> Cells<'a> {
        Cells { op: self, u, c, nx: self.grid.x.n as isize, ny: self.grid.y.map_or(1, |a| a.n) as isize }
    }

    fn eval(&self, c: &Ctx) -> EvalResult {
        let u: &[f64] = match &c.vals[self.binding] {
            Value::Field(d) => d,
            v => return Err(Status::invalid(format!("{v} is not a field"))),
        };
        let g = &self.grid;
        let cells = self.cells(u, c);
        let at = || c.point.ok_or_else(|| Status::invalid("a field's neighbours are read only where the field is: in its flow or its definition"));
        let ij = |k: usize| ((k % g.x.n) as isize, (k / g.x.n) as isize);
        let num = |x: f64, what: &str| finite(x, what);
        match &self.op {
            FieldOp::Laplacian => {
                let k = at()?;
                let (i, j) = ij(k);
                // Cells inside the grid read their neighbours directly.
                let nx = g.x.n;
                let inner_x = i > 0 && (i as usize) + 1 < nx;
                let inner_y = g.y.is_none_or(|y| j > 0 && (j as usize) + 1 < y.n);
                if inner_x && inner_y {
                    let mut lap = (u[k - 1] - 2.0 * u[k] + u[k + 1]) / (g.x.h * g.x.h);
                    if let Some(y) = g.y {
                        lap += (u[k - nx] - 2.0 * u[k] + u[k + nx]) / (y.h * y.h);
                    }
                    return num(lap, "laplacian");
                }
                let u0 = cells.at(i, j, true)?;
                let mut lap = (cells.at(i - 1, j, true)? - 2.0 * u0 + cells.at(i + 1, j, true)?) / (g.x.h * g.x.h);
                if let Some(y) = g.y {
                    lap += (cells.at(i, j - 1, true)? - 2.0 * u0 + cells.at(i, j + 1, true)?) / (y.h * y.h);
                }
                num(lap, "laplacian")
            }
            FieldOp::Gradient => {
                let (i, j) = ij(at()?);
                let gx = (cells.at(i + 1, j, true)? - cells.at(i - 1, j, true)?) / (2.0 * g.x.h);
                match g.y {
                    None => num(gx, "gradient"),
                    Some(y) => {
                        let gy = (cells.at(i, j + 1, true)? - cells.at(i, j - 1, true)?) / (2.0 * y.h);
                        finite_arr(Arr::from_slice(&[gx, gy]), false, "gradient")
                    }
                }
            }
            FieldOp::Sample(p) => {
                let (px, py) = match (p.eval(c)?, g.y) {
                    (Value::Num(x), None) => (x, 0.0),
                    (Value::Point(a), Some(_)) => (a.v[0], a.v[1]),
                    (v, _) => return Err(Status::invalid(format!("a field is read at a coordinate, not {v}"))),
                };
                let inside = |a: &GridAxis, x: f64| x >= a.lo - 1e-9 * a.h && x <= a.hi() + 1e-9 * a.h;
                if !inside(&g.x, px) || g.y.is_some_and(|y| !inside(&y, py)) {
                    return Err(Status { kind: StatusKind::Unavailable, cause: "the point is outside the grid".into() });
                }
                let (i0, i1, wx) = bracket(&g.x, px);
                let v = match g.y {
                    None => (1.0 - wx) * cells.at(i0, 0, false)? + wx * cells.at(i1, 0, false)?,
                    Some(y) => {
                        let (j0, j1, wy) = bracket(&y, py);
                        let row = |j: isize| -> Result<f64, Status> { Ok((1.0 - wx) * cells.at(i0, j, false)? + wx * cells.at(i1, j, false)?) };
                        (1.0 - wy) * row(j0)? + wy * row(j1)?
                    }
                };
                num(v, "field value")
            }
            FieldOp::Integral => num(g.x.h * g.y.map_or(1.0, |y| y.h) * u.iter().sum::<f64>(), "integral"),
            FieldOp::Mean => num(u.iter().sum::<f64>() / u.len() as f64, "mean"),
            FieldOp::Min => num(u.iter().copied().fold(f64::INFINITY, f64::min), "min"),
            FieldOp::Max => num(u.iter().copied().fold(f64::NEG_INFINITY, f64::max), "max"),
        }
    }
}

impl CExpr {
    /// The points through which a field on a line is drawn: its edge values and cell
    /// centres, in order (D-091). `self` is a `Field` operation on the field.
    pub fn field_profile(&self, c: &Ctx) -> Result<Vec<[f64; 2]>, Status> {
        let (op, u) = self.field_cells(c)?;
        let g = &op.grid;
        let cells = op.cells(&u, c);
        let n = g.x.n as isize;
        let mut out = vec![[g.x.lo, cells.at(-1, 0, false)?]];
        out.extend(u.iter().enumerate().map(|(k, v)| [g.x.center(k), *v]));
        out.push([g.x.hi(), cells.at(n, 0, false)?]);
        Ok(out)
    }

    /// The operation on a field and the field's cells (D-091).
    pub fn field_cells(&self, c: &Ctx) -> Result<(&CFieldOp, Rc<Vec<f64>>), Status> {
        let CExpr::Field(op) = self else { return Err(Status::invalid("not a field")) };
        match &c.vals[op.binding] {
            Value::Field(d) => Ok((op, d.clone())),
            v => Err(Status::invalid(format!("{v} is not a field"))),
        }
    }
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
    /// The cell of a pointwise evaluation of a field's flow or definition (D-091).
    pub point: Option<usize>,
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
            // A derived binding with no value gives its status (MK-5.2, D-106).
            CExpr::Load(i) => match &c.vals[*i] {
                Value::Absent(s) => Err((**s).clone()),
                v => Ok(v.clone()),
            },
            CExpr::Arg(i) => Ok(c.args[*i].clone()),
            CExpr::Time => Ok(Num(c.t)),
            CExpr::Time0 => Ok(Num(c.t0)),
            CExpr::Elapsed => Ok(Num(c.t - c.t0)),
            CExpr::Noise(k) => Ok(Num(c.noise.get(*k).copied().unwrap_or(0.0))),
            CExpr::FieldAt(i) => match (&c.vals[*i], c.point) {
                (Value::Field(d), Some(k)) => Ok(Num(d[k])),
                (v, _) => Err(Status::invalid(format!("{v} read outside a pointwise evaluation"))),
            },
            CExpr::Field(op) => op.eval(c),
            CExpr::Der(i) => match c.der {
                // A field's rate in the cell of a pointwise evaluation (D-091).
                Some(d) => match (&d[*i], c.point) {
                    (Field(v), Some(k)) => Ok(Num(v[k])),
                    (v, _) => Ok(v.clone()),
                },
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
            CExpr::Call(F::Dot, args) => {
                let (a, b) = (args[0].eval(c)?, args[1].eval(c)?);
                let (mut x, mut y) = ([0.0; 3], [0.0; 3]);
                let n = a.width();
                a.write_to(&mut x[..n]);
                b.write_to(&mut y[..n]);
                finite(x[..n].iter().zip(&y[..n]).map(|(p, q)| p * q).sum(), "dot product")
            }
            CExpr::Call(F::Cross, args) => {
                let (a, b) = (args[0].eval(c)?, args[1].eval(c)?);
                let (mut x, mut y) = ([0.0; 3], [0.0; 3]);
                a.write_to(&mut x);
                b.write_to(&mut y);
                let z = [x[1] * y[2] - x[2] * y[1], x[2] * y[0] - x[0] * y[2], x[0] * y[1] - x[1] * y[0]];
                if z.iter().any(|v| !v.is_finite()) {
                    return Err(Status::invalid("cross product is not finite"));
                }
                Ok(Value::Vec(Arr::from_slice(&z)))
            }
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
                    F::Floor => x.floor(),
                    F::Ceil => x.ceil(),
                    F::Round => x.round(),
                    F::Mod => {
                        if y == 0.0 {
                            return Err(Status::invalid("mod by zero"));
                        }
                        x - y * (x / y).floor()
                    }
                    F::Dot | F::Cross => unreachable!(),
                    F::Laplacian | F::Gradient | F::Integral | F::Mean => return Err(Status::invalid("a field operation compiled as a call")),
                };
                finite(r, "function")
            }
            CExpr::Apply(f, args) => match f.eval(c)? {
                Func(body) => {
                    let vals: Result<std::vec::Vec<Value>, Status> = args.iter().map(|a| a.eval(c)).collect();
                    let vals = vals?;
                    let inner = Ctx { vals: c.vals, der: c.der, t: c.t, t0: c.t0, args: &vals, payloads: c.payloads, draws: c.draws, noise: c.noise, point: c.point };
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
            CExpr::InFrame(f) => {
                let (at, axes) = f.axes(c)?;
                let v = match f.value.eval(c)? {
                    Vec(a) | Point(a) => a.v,
                    v => return Err(Status::invalid(format!("no component in a frame of {v}"))),
                };
                let d: [f64; 3] = if f.point { std::array::from_fn(|i| v[i] - at[i]) } else { v };
                let e = axes[f.axis];
                Ok(Num(d[0] * e[0] + d[1] * e[1] + d[2] * e[2]))
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
            // `first` (D-106): the value at the smallest key; the first member wins a tie.
            CExpr::Extreme(_, terms, values) if !values.is_empty() => {
                let mut best: Option<(f64, usize)> = None;
                for (k, (w, v)) in terms.iter().enumerate() {
                    if w.eval(c)?.boolean() {
                        let x = v.eval(c)?.num();
                        if best.is_none_or(|(b, _)| x < b) {
                            best = Some((x, k));
                        }
                    }
                }
                match best {
                    Some((_, k)) => values[k].eval(c),
                    None => Err(Status::invalid("`first` over no member has no value")),
                }
            }
            CExpr::Extreme(f, terms, _) => {
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

impl CExpr {
    /// The value and, per component, the size of the terms it is computed from: `a - b` with
    /// `a` near `b` has a value near 0 and the size `|a| + |b|`, the scale its rounding is
    /// measured against (RC-5.6). Sizes multiply through products, quotients and dot
    /// products, and follow components and vectors; other forms have the size of their value.
    pub fn eval_sized(&self, c: &Ctx, width: usize) -> Result<(Value, [f64; 3]), Status> {
        let plain = |v: Value| {
            let mut x = [0.0; 3];
            let w = v.width();
            if w <= 3 && !matches!(v, Value::Field(_)) && w > 0 {
                v.write_to(&mut x[..w]);
            }
            (v, x.map(f64::abs))
        };
        let size = |v: &Value| v.width().min(3);
        Ok(match self {
            CExpr::Bin(op @ (BinOp::Add | BinOp::Sub), l, r) => {
                let ((a, sa), (b, sb)) = (l.eval_sized(c, width)?, r.eval_sized(c, width)?);
                (binary(*op, a, b)?, [sa[0] + sb[0], sa[1] + sb[1], sa[2] + sb[2]])
            }
            CExpr::Bin(op @ (BinOp::Mul | BinOp::Div), l, r) => {
                let ((a, sa), (b, sb)) = (l.eval_sized(c, width)?, r.eval_sized(c, width)?);
                let (na, nb) = (size(&a), size(&b));
                // A product's size is the product of its factors' sizes; a quotient's, its
                // numerator's over the divisor.
                let divisor = match &b {
                    Value::Num(x) if *x != 0.0 => x.abs(),
                    _ => 1.0,
                };
                let v = binary(*op, a, b)?;
                let mut s = [0.0; 3];
                for (k, x) in s.iter_mut().enumerate() {
                    let p = if na == 1 { sa[0] } else { sa[k] };
                    let q = if nb == 1 { sb[0] } else { sb[k] };
                    *x = if *op == BinOp::Mul { p * q } else { p / divisor };
                }
                (v, s)
            }
            CExpr::Call(F::Dot, args) => {
                let ((a, sa), (_, sb)) = (args[0].eval_sized(c, 3)?, args[1].eval_sized(c, 3)?);
                let n = a.width().min(3);
                (self.eval(c)?, [(0..n).map(|k| sa[k] * sb[k]).sum(), 0.0, 0.0])
            }
            CExpr::Call(F::Cross, args) => {
                let ((_, a), (_, b)) = (args[0].eval_sized(c, 3)?, args[1].eval_sized(c, 3)?);
                (self.eval(c)?, [a[1] * b[2] + a[2] * b[1], a[2] * b[0] + a[0] * b[2], a[0] * b[1] + a[1] * b[0]])
            }
            CExpr::Comp(e, k) => {
                let (_, s) = e.eval_sized(c, 3)?;
                (self.eval(c)?, [s[(*k).min(2)], 0.0, 0.0])
            }
            CExpr::MakeVec(items) => {
                let mut s = [0.0; 3];
                for (k, e) in items.iter().take(3).enumerate() {
                    s[k] = e.eval_sized(c, 1)?.1[0];
                }
                (self.eval(c)?, s)
            }
            CExpr::Neg(e) => {
                let (_, s) = e.eval_sized(c, width)?;
                (self.eval(c)?, s)
            }
            _ => plain(self.eval(c)?),
        })
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

/// The value of a pointwise expression over the cells of a grid (D-091): the same in every
/// cell, or one per cell.
enum Lanes {
    One(f64),
    Each(std::vec::Vec<f64>),
}

/// A condition in every cell of a grid: the same in all, or one per cell.
enum Conds {
    One(bool),
    Each(std::vec::Vec<bool>),
}

impl Conds {
    fn zip(self, o: Conds, f: impl Fn(bool, bool) -> bool, n: usize) -> Conds {
        match (self, o) {
            (Conds::One(a), Conds::One(b)) => Conds::One(f(a, b)),
            (a, b) => {
                let at = |c: &Conds, k: usize| match c {
                    Conds::One(v) => *v,
                    Conds::Each(v) => v[k],
                };
                Conds::Each((0..n).map(|k| f(at(&a, k), at(&b, k))).collect())
            }
        }
    }
}

impl CExpr {
    /// A Boolean expression in every cell, where its form allows: comparisons of numbers,
    /// `and`, `or`, `not` and constants; `None` for other forms.
    fn cond_lanes(&self, c: &Ctx, g: &CGrid) -> Result<Option<Conds>, Status> {
        let n = g.cells();
        Ok(Some(match self {
            CExpr::Const(Value::Bool(b)) => Conds::One(*b),
            CExpr::Load(i) => match &c.vals[*i] {
                Value::Bool(b) => Conds::One(*b),
                _ => return Ok(None),
            },
            CExpr::Not(e) => match e.cond_lanes(c, g)? {
                Some(Conds::One(b)) => Conds::One(!b),
                Some(Conds::Each(v)) => Conds::Each(v.into_iter().map(|b| !b).collect()),
                None => return Ok(None),
            },
            CExpr::Bin(op @ (BinOp::And | BinOp::Or), l, r) => {
                let (Some(a), Some(b)) = (l.cond_lanes(c, g)?, r.cond_lanes(c, g)?) else { return Ok(None) };
                if *op == BinOp::And { a.zip(b, |x, y| x && y, n) } else { a.zip(b, |x, y| x || y, n) }
            }
            CExpr::Bin(op @ (BinOp::Lt | BinOp::Le | BinOp::Gt | BinOp::Ge), l, r) => {
                let (Some(a), Some(b)) = (l.lanes(c, g)?, r.lanes(c, g)?) else { return Ok(None) };
                let f = |x: f64, y: f64| match op {
                    BinOp::Lt => x < y,
                    BinOp::Le => x <= y,
                    BinOp::Gt => x > y,
                    _ => x >= y,
                };
                match (a, b) {
                    (Lanes::One(x), Lanes::One(y)) => Conds::One(f(x, y)),
                    (a, b) => {
                        let at = |l: &Lanes, k: usize| match l {
                            Lanes::One(v) => *v,
                            Lanes::Each(v) => v[k],
                        };
                        Conds::Each((0..n).map(|k| f(at(&a, k), at(&b, k))).collect())
                    }
                }
            }
            _ => return Ok(None),
        }))
    }
}

impl Lanes {
    fn map(self, f: impl Fn(f64) -> f64) -> Lanes {
        match self {
            Lanes::One(x) => Lanes::One(f(x)),
            Lanes::Each(mut v) => {
                v.iter_mut().for_each(|x| *x = f(*x));
                Lanes::Each(v)
            }
        }
    }
    fn zip(self, o: Lanes, f: impl Fn(f64, f64) -> f64) -> Lanes {
        match (self, o) {
            (Lanes::One(a), Lanes::One(b)) => Lanes::One(f(a, b)),
            (Lanes::One(a), Lanes::Each(mut v)) => {
                v.iter_mut().for_each(|x| *x = f(a, *x));
                Lanes::Each(v)
            }
            (Lanes::Each(mut v), Lanes::One(b)) => {
                v.iter_mut().for_each(|x| *x = f(*x, b));
                Lanes::Each(v)
            }
            (Lanes::Each(mut v), Lanes::Each(w)) => {
                v.iter_mut().zip(&w).for_each(|(x, y)| *x = f(*x, *y));
                Lanes::Each(v)
            }
        }
    }
}

impl CExpr {
    /// A pointwise expression evaluated in every cell of `grid` at once (D-091), operator by
    /// operator over arrays: the same values as evaluating it cell by cell, computed without
    /// interpreting the expression once per cell. `None` for forms it does not take (and for
    /// results that are not finite), which are then evaluated cell by cell, so statuses keep
    /// their messages.
    pub fn eval_cells(&self, c: &Ctx, g: &CGrid) -> Option<std::vec::Vec<f64>> {
        let n = g.cells();
        let out = match self.lanes(c, g).ok()?? {
            Lanes::One(x) => vec![x; n],
            Lanes::Each(v) => v,
        };
        out.iter().all(|x| x.is_finite()).then_some(out)
    }

    fn lanes(&self, c: &Ctx, g: &CGrid) -> Result<Option<Lanes>, Status> {
        use Value::*;
        let n = g.cells();
        Ok(Some(match self {
            CExpr::Const(Num(x)) => Lanes::One(*x),
            CExpr::Load(i) => match &c.vals[*i] {
                Num(x) => Lanes::One(*x),
                _ => return Ok(None),
            },
            CExpr::Time => Lanes::One(c.t),
            CExpr::Time0 => Lanes::One(c.t0),
            CExpr::Elapsed => Lanes::One(c.t - c.t0),
            CExpr::FieldAt(i) => match &c.vals[*i] {
                Field(d) if d.len() == n => Lanes::Each(d.to_vec()),
                _ => return Ok(None),
            },
            // The coordinate of each cell, and its components in the plane.
            CExpr::Arg(0) if g.y.is_none() => Lanes::Each((0..n).map(|k| g.x.center(k)).collect()),
            CExpr::Comp(e, axis) if matches!(**e, CExpr::Arg(0)) && g.y.is_some() && *axis < 2 => {
                let y = g.y.unwrap();
                Lanes::Each((0..n).map(|k| if *axis == 0 { g.x.center(k % g.x.n) } else { y.center(k / g.x.n) }).collect())
            }
            CExpr::Neg(e) => match e.lanes(c, g)? {
                Some(l) => l.map(|x| -x),
                None => return Ok(None),
            },
            CExpr::Powi(e, p) => match e.lanes(c, g)? {
                Some(l) => l.map(|x| x.powf(*p)),
                None => return Ok(None),
            },
            CExpr::Bin(op @ (BinOp::Add | BinOp::Sub | BinOp::Mul | BinOp::Div | BinOp::Pow), l, r) => {
                let (Some(a), Some(b)) = (l.lanes(c, g)?, r.lanes(c, g)?) else { return Ok(None) };
                match op {
                    BinOp::Add => a.zip(b, |x, y| x + y),
                    BinOp::Sub => a.zip(b, |x, y| x - y),
                    BinOp::Mul => a.zip(b, |x, y| x * y),
                    // Division by zero gives a value that is not finite: evaluated cell by cell.
                    BinOp::Div => a.zip(b, |x, y| x / y),
                    _ => a.zip(b, f64::powf),
                }
            }
            // A choice per cell (D-091): the condition in every cell, then each cell takes its
            // branch. A branch is computed in every cell, but read only where chosen; one that
            // cannot be computed sends the whole expression back to cell by cell evaluation,
            // which evaluates only the branch each cell takes.
            CExpr::If(k, a, b) => match k.cond_lanes(c, g)? {
                None => return Ok(None),
                Some(Conds::One(true)) => return a.lanes(c, g),
                Some(Conds::One(false)) => return b.lanes(c, g),
                Some(Conds::Each(which)) => {
                    let (Ok(Some(x)), Ok(Some(y))) = (a.lanes(c, g), b.lanes(c, g)) else { return Ok(None) };
                    let at = |l: &Lanes, k: usize| match l {
                        Lanes::One(v) => *v,
                        Lanes::Each(v) => v[k],
                    };
                    Lanes::Each(which.iter().enumerate().map(|(k, w)| if *w { at(&x, k) } else { at(&y, k) }).collect())
                }
            },
            CExpr::Call(f @ (F::Min | F::Max | F::Atan2), args) if args.len() == 2 => {
                let (Some(a), Some(b)) = (args[0].lanes(c, g)?, args[1].lanes(c, g)?) else { return Ok(None) };
                match f {
                    F::Min => a.zip(b, f64::min),
                    F::Max => a.zip(b, f64::max),
                    _ => a.zip(b, f64::atan2),
                }
            }
            CExpr::Call(f, args) if args.len() == 1 && !matches!(f, F::Laplacian | F::Gradient | F::Integral | F::Mean) => {
                let Some(a) = args[0].lanes(c, g)? else { return Ok(None) };
                match f {
                    F::Sin => a.map(f64::sin),
                    F::Cos => a.map(f64::cos),
                    F::Tan => a.map(f64::tan),
                    F::Sqrt => a.map(f64::sqrt),
                    F::Exp => a.map(f64::exp),
                    F::Log => a.map(f64::ln),
                    F::Abs => a.map(f64::abs),
                    F::Floor => a.map(f64::floor),
                    F::Ceil => a.map(f64::ceil),
                    F::Round => a.map(f64::round),
                    _ => return Ok(None),
                }
            }
            CExpr::Field(op) if op.grid == *g || !matches!(op.op, FieldOp::Laplacian | FieldOp::Gradient) => match &op.op {
                FieldOp::Laplacian => {
                    let mut v = vec![0.0; n];
                    for (k, o) in v.iter_mut().enumerate() {
                        let inner = Ctx { vals: c.vals, der: c.der, t: c.t, t0: c.t0, args: c.args, payloads: c.payloads, draws: c.draws, noise: c.noise, point: Some(k) };
                        *o = op.eval(&inner)?.num();
                    }
                    Lanes::Each(v)
                }
                FieldOp::Gradient if g.y.is_none() => {
                    let mut v = vec![0.0; n];
                    for (k, o) in v.iter_mut().enumerate() {
                        let inner = Ctx { vals: c.vals, der: c.der, t: c.t, t0: c.t0, args: c.args, payloads: c.payloads, draws: c.draws, noise: c.noise, point: Some(k) };
                        *o = op.eval(&inner)?.num();
                    }
                    Lanes::Each(v)
                }
                FieldOp::Integral | FieldOp::Mean | FieldOp::Min | FieldOp::Max => Lanes::One(op.eval(c)?.num()),
                _ => return Ok(None),
            },
            _ => return Ok(None),
        }))
    }
}
