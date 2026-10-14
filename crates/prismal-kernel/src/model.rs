//! A checked, compiled model and the evaluation services the runtime uses.

use crate::eval::{Arr, CExpr, Ctx, Status, Value};
use prismal_ir::{Id, Model, Policy, Role, Space, Type};
use std::collections::HashMap;

#[derive(Clone, Debug)]
pub struct CBinding {
    pub id: Id,
    pub name: String,
    pub role: Role,
    pub ty: Type,
    pub init: Option<CExpr>,
    pub def: Option<CExpr>,
    pub intervenable: bool,
    /// A derived binding evaluated only while the condition holds, else the value (D-058).
    pub when: Option<(CExpr, Value)>,
    /// The grid of a field (D-091): its initial value or definition is evaluated in each
    /// cell, with the cell's centre as the argument.
    pub field: Option<crate::eval::CGrid>,
}

/// Evaluates a pointwise expression in every cell of `grid`: a field's value (D-091).
pub fn per_cell(e: &CExpr, grid: &crate::eval::CGrid, c: &Ctx) -> Result<Value, Status> {
    if let Some(v) = e.eval_cells(c, grid) {
        return Ok(Value::Field(std::rc::Rc::new(v)));
    }
    let mut out = Vec::with_capacity(grid.cells());
    for k in 0..grid.cells() {
        let args = [grid.coord(k)];
        let inner = Ctx { vals: c.vals, der: c.der, t: c.t, t0: c.t0, args: &args, payloads: c.payloads, draws: c.draws, noise: c.noise, point: Some(k) };
        out.push(e.eval(&inner)?.num());
    }
    Ok(Value::Field(std::rc::Rc::new(out)))
}

/// How a continuous state's rate is formed (MK-14.7 to MK-14.11).
#[derive(Clone, Debug)]
pub enum FlowSet {
    /// No flow: the default sum of zero contributions (MK-14.11).
    None,
    Define(CExpr),
    /// Contributions, combined by sum in identity order (MK-14.10).
    Sum(Vec<CExpr>),
}

#[derive(Clone, Debug)]
pub struct ContState {
    pub binding: usize,
    pub offset: usize,
    pub len: usize,
    pub point: bool,
    pub flows: FlowSet,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Dir {
    Rising,
    Falling,
    Either,
}

#[derive(Clone, Debug)]
pub enum CTrigger {
    Crossing { dir: Dir, guard: CExpr },
    At(CExpr),
    Every { period: CExpr, from: CExpr },
    On(usize),
    Start,
    Request,
    Input(usize),
    /// The jumps of a Poisson process at this rate (D-083). `varying`: the rate reads
    /// continuous state or time, so it changes between transitions (MK-15.16).
    Random { rate: CExpr, varying: bool },
}

#[derive(Clone, Debug)]
pub enum COp {
    Set { binding: usize, component: Option<usize>, value: CExpr },
    /// A field set in each cell of its grid (D-091).
    SetField { binding: usize, grid: crate::eval::CGrid, value: CExpr },
    /// `contribute x += e`: combined with the other contributions of the transition and the
    /// value before it, by `combine` (MK-14.9, D-073).
    Contribute { binding: usize, value: CExpr, combine: String },
    /// `emit E` or `emit E(value)`, the value becoming the payload of `E`'s followers (D-050).
    Emit { event: usize, payload: Option<CExpr> },
    /// `destroy`: the liveness binding of a member becomes false; two in one transition do
    /// not conflict (MK-16.5, D-057).
    Destroy { binding: usize },
    /// Operations performed when `cond` holds before the transition (D-057).
    If { cond: CExpr, ops: Vec<COp> },
}

#[derive(Clone, Debug)]
pub struct CZeno {
    /// `None` is `stop`; `Some(ops)` is `settle { ops }`.
    pub settle: Option<Vec<COp>>,
    pub eps: Option<f64>,
    pub n: Option<u32>,
    pub window: Option<f64>,
}

#[derive(Clone, Debug)]
pub struct CEvent {
    pub id: Id,
    pub name: String,
    /// `None` only for invalid triggers, which never reach a compiled model.
    pub trigger: Option<CTrigger>,
    pub enable: Option<CExpr>,
    pub handler: Vec<COp>,
    pub zeno: Option<CZeno>,
}

#[derive(Clone, Debug)]
pub struct CEquation {
    pub id: Id,
    pub name: String,
    pub lhs: CExpr,
    pub rhs: CExpr,
    pub check: bool,
    pub check_tol: Option<f64>,
    /// An equation over a field holds in each cell of this grid (D-091).
    pub field: Option<crate::eval::CGrid>,
    /// Components of the equation: 1, or the axes of its vectors' space (D-092).
    pub width: usize,
}

/// The unknowns of a model and the equations that determine them (MK-11.6, D-092).
#[derive(Clone, Debug)]
pub struct CSolve {
    pub unknowns: Vec<usize>,
    /// The starting guess of each unknown; none is 0.
    pub guess: Vec<Option<CExpr>>,
    /// Derived bindings computed inside the solve, in order.
    pub inner: Vec<usize>,
    /// Indices into the model's equations.
    pub equations: Vec<usize>,
}

/// The largest number of Newton iterations of a solve (RC-5.6).
pub const SOLVE_ITERATIONS: usize = 60;

#[derive(Clone, Debug)]
pub struct CConstraint {
    pub id: Id,
    pub name: String,
    pub cond: Option<CExpr>,
    /// An equality with tolerance: `|lhs - rhs| <= tol`.
    pub within: Option<(CExpr, CExpr, f64)>,
    pub policy: Policy,
    /// An enforced constraint (D-093) and its rates that read no unknown, from the
    /// constraint itself: the state is projected onto each in turn (MK-12.8).
    pub levels: Vec<CLevel>,
}

/// One rate of an enforced constraint, `lhs == rhs`, and the bindings the projection onto it
/// moves: those it reads that the lower rates do not (velocities, after positions).
#[derive(Clone, Debug)]
pub struct CLevel {
    pub lhs: CExpr,
    pub rhs: CExpr,
    pub width: usize,
    /// The state it moves.
    pub vars: Vec<usize>,
    /// The parameters it reads, moved only by a change that sets them.
    pub params: Vec<usize>,
    /// Derived bindings it reads, in dependency order.
    pub derived: Vec<usize>,
}

/// The largest number of Gauss-Newton iterations of a projection (RC-5.10).
pub const PROJECT_ITERATIONS: usize = 30;

/// A checked model, ready to run.
#[derive(Clone, Debug)]
pub struct CModel {
    pub ir: Model,
    pub spaces: Vec<Space>,
    pub index: HashMap<Id, usize>,
    pub bindings: Vec<CBinding>,
    pub derived_order: Vec<usize>,
    /// The part of `derived_order` the flows read, directly or through other derived values
    /// and the solve (D-095): what a solver's evaluation of the flows needs.
    pub flow_order: Vec<usize>,
    pub init_order: Vec<usize>,
    pub cont: Vec<ContState>,
    pub events: Vec<CEvent>,
    pub equations: Vec<CEquation>,
    pub constraints: Vec<CConstraint>,
    /// Declared functions, compiled, with their types (D-048).
    pub functions: std::collections::HashMap<Id, (std::rc::Rc<crate::eval::CExpr>, Type)>,
    /// MK-14.15: no flows and no events other than requests.
    pub is_static: bool,
    /// The stream key of each white noise in the flows, by noise number (D-090); a model
    /// with noise is integrated by Euler-Maruyama (RC-13.9).
    pub noise: Vec<u64>,
    /// The model's grids by identity (MK-9.5, D-091).
    pub grids: HashMap<Id, crate::eval::CGrid>,
    /// The unknowns and their equations (D-092).
    pub solve: Option<CSolve>,
}

impl CModel {
    pub fn n_y(&self) -> usize {
        self.cont.iter().map(|c| c.len).sum()
    }

    pub fn idx(&self, id: &str) -> usize {
        *self.index.get(id).unwrap_or_else(|| panic!("unknown binding {id}"))
    }

    /// Values for evaluating an expression that reads no state: the constants, which a
    /// declared function may read (MK-10.3), and `NaN` for every other binding.
    pub fn constant_values(&self) -> Vec<Value> {
        let mut vals = vec![Value::Num(f64::NAN); self.bindings.len()];
        for &i in self.init_order.iter().filter(|&&i| i != crate::check::SOLVE) {
            let b = &self.bindings[i];
            if b.role != Role::Constant {
                continue;
            }
            if let Some(e) = &b.init {
                if let Ok(v) = e.eval(&Ctx { vals: &vals, der: None, t: 0.0, t0: 0.0, args: &[], payloads: &[], draws: None, noise: &[], point: None }) {
                    vals[i] = v;
                }
            }
        }
        vals
    }

    pub fn event_idx(&self, id: &str) -> usize {
        self.events.iter().position(|e| e.id == id).unwrap_or_else(|| panic!("unknown event {id}"))
    }

    /// Initial values (RC-5.1): parameters with overrides, then initial definitions and
    /// derived bindings in dependency order.
    pub fn initial(&self, overrides: &[(usize, Value)], t0: f64) -> Result<Vec<Value>, Status> {
        let mut vals = vec![Value::Num(f64::NAN); self.bindings.len()];
        for &i in &self.init_order {
            if i == crate::check::SOLVE {
                self.solve_unknowns(&mut vals, t0, t0).map_err(|s| Status { kind: s.kind, cause: format!("initializing: {}", s.cause) })?;
                continue;
            }
            let b = &self.bindings[i];
            if let Some((_, v)) = overrides.iter().find(|(j, _)| *j == i) {
                vals[i] = v.clone();
                continue;
            }
            if let Some((w, none)) = &b.when {
                let c = Ctx { vals: &vals, der: None, t: t0, t0, args: &[], payloads: &[], draws: None, noise: &[], point: None };
                if !w.eval(&c)?.boolean() {
                    vals[i] = none.clone();
                    continue;
                }
            }
            let e = match b.role {
                Role::Derived => b.def.as_ref(),
                // D-051: an input starts at the value the run configuration supplies, else at
                // its declared default; without either the environment has not supplied it.
                Role::Input if b.init.is_none() => {
                    return Err(Status::invalid(format!("input `{}` has no value at the start: supply one in the run configuration, or declare a default", b.name)))
                }
                _ => b.init.as_ref(),
            };
            let v = {
                let c = Ctx { vals: &vals, der: None, t: t0, t0, args: &[], payloads: &[], draws: None, noise: &[], point: None };
                let e = e.expect("checked");
                match &b.field {
                    Some(g) => per_cell(e, g, &c),
                    None => e.eval(&c),
                }
                .map_err(|s| Status { kind: s.kind, cause: format!("initializing `{}`: {}", b.name, s.cause) })?
            };
            vals[i] = v;
        }
        // RC-5.10 (D-093): the starting state satisfies the enforced constraints.
        if self.project(&mut vals, t0, t0, &[]).map_err(|s| Status { kind: s.kind, cause: format!("initializing: {}", s.cause) })? {
            self.update_derived(&mut vals, t0, t0)?;
        }
        Ok(vals)
    }

    /// RC-5.10 (D-093): moves the values onto the enforced constraints, the constraints
    /// themselves first and then each rate in turn, by the smallest change of the bindings each
    /// moves (Gauss-Newton with the minimum-norm step, each binding measured against its own
    /// size). `set` are the bindings the change being committed sets: where a rate reads some
    /// of them, only those move (a dragged bob, not the bob it hangs from); parameters move
    /// only then. Returns whether anything moved; the derived values are then stale, and the
    /// caller computes them again.
    pub fn project(&self, vals: &mut [Value], t: f64, t0: f64, set: &[usize]) -> Result<bool, Status> {
        let depth = self.constraints.iter().map(|k| k.levels.len()).max().unwrap_or(0);
        let mut moved = false;
        for p in 0..depth {
            let rows: Vec<(&CConstraint, &CLevel)> = self.constraints.iter().filter_map(|k| k.levels.get(p).map(|l| (k, l))).collect();
            let mut vars: Vec<usize> = rows.iter().flat_map(|(_, l)| l.vars.iter().chain(&l.params).copied()).filter(|i| set.contains(i)).collect();
            if vars.is_empty() {
                vars = rows.iter().flat_map(|(_, l)| l.vars.iter().copied()).collect();
            }
            vars.sort();
            vars.dedup();
            let mut derived: Vec<usize> = rows.iter().flat_map(|(_, l)| l.derived.iter().copied()).collect();
            derived.sort_by_key(|i| self.derived_order.iter().position(|j| j == i));
            derived.dedup();
            // The components moved, as (binding, component).
            let comps: Vec<(usize, usize)> = vars.iter().flat_map(|&b| (0..vals[b].width()).map(move |c| (b, c))).collect();
            let read = |vals: &[Value]| -> Vec<f64> {
                comps
                    .iter()
                    .map(|&(b, c)| match &vals[b] {
                        Value::Num(x) => *x,
                        v => v.arr().v[c],
                    })
                    .collect()
            };
            let write = |vals: &mut [Value], x: &[f64]| {
                for (&(b, c), v) in comps.iter().zip(x) {
                    vals[b] = match &vals[b] {
                        Value::Num(_) => Value::Num(*v),
                        Value::Point(a) => {
                            let mut a = *a;
                            a.v[c] = *v;
                            Value::Point(a)
                        }
                        Value::Vec(a) => {
                            let mut a = *a;
                            a.v[c] = *v;
                            Value::Vec(a)
                        }
                        other => other.clone(),
                    };
                }
            };
            let residual = |vals: &mut [Value]| -> Result<(Vec<f64>, Vec<f64>), Status> {
                for &i in &derived {
                    let c = Ctx { vals, der: None, t, t0, args: &[], payloads: &[], draws: None, noise: &[], point: None };
                    let v = self.bindings[i].def.as_ref().expect("checked").eval(&c)?;
                    vals[i] = v;
                }
                let c = Ctx { vals, der: None, t, t0, args: &[], payloads: &[], draws: None, noise: &[], point: None };
                let (mut r, mut size) = (vec![], vec![]);
                for (_, l) in &rows {
                    let ((a, sa), (b, sb)) = (l.lhs.eval_sized(&c, l.width)?, l.rhs.eval_sized(&c, l.width)?);
                    let (mut x, mut y) = ([0.0; 3], [0.0; 3]);
                    a.write_to(&mut x[..l.width]);
                    b.write_to(&mut y[..l.width]);
                    for j in 0..l.width {
                        r.push(x[j] - y[j]);
                        size.push(sa[j] + sb[j]);
                    }
                }
                Ok((r, size))
            };
            let names = || rows.iter().map(|(k, _)| format!("`{}`", k.name)).collect::<Vec<_>>().join(", ");
            let fail = |why: &str| Status::invalid(format!("the constraints {} cannot be enforced here: {why} (RC-5.10)", names()));
            // The scale of each moved component: the size of its binding's value, or 1 in
            // coherent SI units.
            let scale: Vec<f64> = comps
                .iter()
                .map(|&(b, _)| {
                    let mut x = [0.0; 3];
                    let w = vals[b].width();
                    vals[b].write_to(&mut x[..w]);
                    let n = x[..w].iter().map(|v| v * v).sum::<f64>().sqrt();
                    if n > 0.0 { n } else { 1.0 }
                })
                .collect();
            let mut x = read(vals);
            let (mut r, mut size) = residual(vals)?;
            if comps.is_empty() {
                // Only parameters, none of them changed: the values must already hold.
                if r.iter().zip(&size).all(|(e, s)| e.abs() <= 1e-9 * s) {
                    continue;
                }
                return Err(fail("they read only parameters, which move only when a change sets them; give values that satisfy them"));
            }
            let mut settled = false;
            for _ in 0..PROJECT_ITERATIONS {
                if r.iter().zip(&size).all(|(e, s)| e.abs() <= 1e-15 * s || *e == 0.0) {
                    settled = true;
                    break;
                }
                // The Jacobian by central differences, column by column.
                let (m, n) = (r.len(), x.len());
                let mut jac = vec![vec![0.0; n]; m];
                for j in 0..n {
                    let h = 1e-6 * (x[j].abs() + scale[j]);
                    let mut xp = x.clone();
                    xp[j] += h;
                    write(vals, &xp);
                    let (rp, _) = residual(vals)?;
                    xp[j] -= 2.0 * h;
                    write(vals, &xp);
                    let (rm, _) = residual(vals)?;
                    for i in 0..m {
                        jac[i][j] = (rp[i] - rm[i]) / (2.0 * h);
                    }
                }
                // The smallest change of the scaled components: with Js = J S,
                // dx = -S Js^T (Js Js^T)^-1 r.
                let js: Vec<Vec<f64>> = jac.iter().map(|row| row.iter().zip(&scale).map(|(a, s)| a * s).collect()).collect();
                let mut normal = vec![vec![0.0; m]; m];
                for a in 0..m {
                    for b in 0..m {
                        normal[a][b] = js[a].iter().zip(&js[b]).map(|(p, q)| p * q).sum();
                    }
                }
                let Some(z) = gauss(normal, r.clone()) else {
                    write(vals, &x);
                    return Err(fail("they are dependent, or do not depend on the values they move"));
                };
                let dx: Vec<f64> = (0..n).map(|j| -scale[j] * (0..m).map(|i| js[i][j] * z[i]).sum::<f64>()).collect();
                for (a, d) in x.iter_mut().zip(&dx) {
                    *a += d;
                }
                write(vals, &x);
                moved = true;
                (r, size) = residual(vals)?;
                if dx.iter().zip(&x).zip(&scale).all(|((d, a), s)| d.abs() <= 1e-15 * (a.abs() + s)) {
                    settled = true;
                    break;
                }
            }
            if !settled && !r.iter().zip(&size).all(|(e, s)| e.abs() <= 1e-9 * s) {
                return Err(fail("the projection does not converge"));
            }
        }
        Ok(moved)
    }

    /// Recomputes every derived binding from the stored values (MK-6.4).
    pub fn update_derived(&self, vals: &mut [Value], t: f64, t0: f64) -> Result<(), Status> {
        self.update_in(&self.derived_order, vals, t, t0)
    }

    /// Recomputes the derived bindings the flows read (D-095), for a solver's evaluation of
    /// the flows; the others keep their earlier values.
    pub fn update_for_flows(&self, vals: &mut [Value], t: f64, t0: f64) -> Result<(), Status> {
        self.update_in(&self.flow_order, vals, t, t0)
    }

    fn update_in(&self, order: &[usize], vals: &mut [Value], t: f64, t0: f64) -> Result<(), Status> {
        for &i in order {
            if i == crate::check::SOLVE {
                self.solve_unknowns(vals, t, t0)?;
                continue;
            }
            let v = {
                let c = Ctx { vals, der: None, t, t0, args: &[], payloads: &[], draws: None, noise: &[], point: None };
                let b = &self.bindings[i];
                match (&b.when, &b.field) {
                    (Some((w, none)), _) if !w.eval(&c)?.boolean() => none.clone(),
                    (_, Some(g)) => per_cell(b.def.as_ref().expect("checked"), g, &c)?,
                    _ => b.def.as_ref().expect("checked").eval(&c)?,
                }
            };
            vals[i] = v;
        }
        Ok(())
    }

    /// RC-5.6 (D-092): the unknowns at one instant, from their equations, by Newton's
    /// method started at the guesses every time, with a Jacobian of finite differences and
    /// a backtracking line search. The result depends only on the other values at that
    /// instant, never on earlier solves, so runs stay reproducible.
    pub fn solve_unknowns(&self, vals: &mut [Value], t: f64, t0: f64) -> Result<(), Status> {
        let Some(s) = &self.solve else { return Ok(()) };
        let n = s.unknowns.len();
        let names = || s.unknowns.iter().map(|&i| format!("`{}`", self.bindings[i].name)).collect::<Vec<_>>().join(", ");
        let mut u = vec![0.0; n];
        for (k, g) in s.guess.iter().enumerate() {
            if let Some(g) = g {
                let c = Ctx { vals, der: None, t, t0, args: &[], payloads: &[], draws: None, noise: &[], point: None };
                u[k] = g.eval(&c)?.num();
            }
        }
        // The scale of each unknown: its guess, or 1 in coherent SI units.
        let scale: Vec<f64> = u.iter().map(|x| if *x != 0.0 { x.abs() } else { 1.0 }).collect();
        // Residuals, and the size of the sides each is measured against.
        let residuals = |vals: &mut [Value], u: &[f64]| -> Result<(Vec<f64>, Vec<f64>), Status> {
            for (&i, x) in s.unknowns.iter().zip(u) {
                vals[i] = Value::Num(*x);
            }
            for &i in &s.inner {
                let c = Ctx { vals, der: None, t, t0, args: &[], payloads: &[], draws: None, noise: &[], point: None };
                let v = self.bindings[i].def.as_ref().expect("checked").eval(&c)?;
                vals[i] = v;
            }
            let c = Ctx { vals, der: None, t, t0, args: &[], payloads: &[], draws: None, noise: &[], point: None };
            let (mut r, mut size) = (vec![], vec![]);
            for &k in &s.equations {
                let q = &self.equations[k];
                let ((a, sa), (b, sb)) = (q.lhs.eval_sized(&c, q.width)?, q.rhs.eval_sized(&c, q.width)?);
                let (mut x, mut y) = ([0.0; 3], [0.0; 3]);
                a.write_to(&mut x[..q.width]);
                b.write_to(&mut y[..q.width]);
                for j in 0..q.width {
                    r.push(x[j] - y[j]);
                    size.push(sa[j] + sb[j]);
                }
            }
            Ok((r, size))
        };
        // The size of the residuals: their Euclidean norm, scaled by the largest so that
        // squares of residuals near 1e156 (a diode guessed far above its knee) do not overflow.
        let merit = |r: &[f64]| {
            let m = r.iter().fold(0.0f64, |m, x| m.max(x.abs()));
            if m == 0.0 || !m.is_finite() { m } else { m * r.iter().map(|x| (x / m) * (x / m)).sum::<f64>().sqrt() }
        };
        let within = |r: &[f64], size: &[f64], tol: f64| r.iter().zip(size).all(|(x, s)| x.abs() <= tol * s || *x == 0.0);
        let (mut r, mut size) = residuals(vals, &u).map_err(|e| Status {
            cause: format!("the equations for {} cannot be evaluated at the guesses: {} (MK-11.8); give guesses nearer the solution", names(), e.cause),
            ..e
        })?;
        for _ in 0..SOLVE_ITERATIONS {
            if within(&r, &size, 1e-12) {
                // The values in place may be a rejected trial: evaluate at the solution again.
                residuals(vals, &u)?;
                return Ok(());
            }
            // The Jacobian by forward differences, column by column.
            let mut jac = vec![vec![0.0; n]; n];
            for j in 0..n {
                let h = 1e-7 * (u[j].abs() + scale[j]);
                let mut v = u.clone();
                v[j] += h;
                let (rj, _) = residuals(vals, &v)?;
                for i in 0..n {
                    jac[i][j] = (rj[i] - r[i]) / h;
                }
            }
            let Some(du) = gauss(jac, r.iter().map(|x| -x).collect()) else {
                return Err(Status::invalid(format!("the equations do not determine {} here: they are dependent (MK-11.8)", names())));
            };
            // A residual is computed no more precisely than its terms: in `V0 - Vd == I * R`
            // with `Vd` near `V0` the left side is a difference of two close numbers and its
            // rounding can exceed both sides. The Newton correction measures the distance to
            // the solution instead: when it is at the rounding level of every unknown (relative
            // to its value, or to its scale near zero), the corrected values are the solution.
            let settled = du.iter().zip(&u).zip(&scale).all(|((d, a), s)| d.abs() <= 1e-12 * a.abs() + 1e-15 * s);
            if settled && r.iter().all(|x| x.is_finite()) {
                let v: Vec<f64> = u.iter().zip(&du).map(|(a, d)| a + d).collect();
                if residuals(vals, &v).is_ok_and(|(rv, _)| rv.iter().all(|x| x.is_finite())) {
                    return Ok(());
                }
            }
            // Backtracking: the longest step, halved up to ten times, that lowers the residual.
            let m0 = merit(&r);
            let mut lambda = 1.0;
            let mut next = None;
            for _ in 0..=10 {
                let v: Vec<f64> = u.iter().zip(&du).map(|(a, d)| a + lambda * d).collect();
                if let Ok((rv, sv)) = residuals(vals, &v) {
                    if merit(&rv) < m0 || lambda < 1e-3 {
                        next = Some((v, rv, sv));
                        break;
                    }
                }
                lambda *= 0.5;
            }
            let Some((mut v, mut rv, mut sv)) = next else { break };
            // On the far side of an exponential a full step moves by a fixed amount (about
            // `Vt` for a diode above its knee) and the solve would take hundreds of steps:
            // when the full step lowers the residual, longer steps are tried while they lower it
            // further.
            if lambda == 1.0 {
                for _ in 0..30 {
                    lambda *= 2.0;
                    let w: Vec<f64> = u.iter().zip(&du).map(|(a, d)| a + lambda * d).collect();
                    match residuals(vals, &w) {
                        Ok((rw, sw)) if merit(&rw) < merit(&rv) => (v, rv, sv) = (w, rw, sw),
                        _ => break,
                    }
                }
            }
            let small = v.iter().zip(&u).zip(&scale).all(|((a, b), s)| (a - b).abs() <= 1e-14 * (b.abs() + s));
            (u, r, size) = (v, rv, sv);
            if small {
                break;
            }
        }
        // The last values tried may be a rejected step: evaluate at the best values again.
        let (r, size) = residuals(vals, &u)?;
        if within(&r, &size, 1e-9) {
            // Converged to the precision the arithmetic allows; the values are in place.
            return Ok(());
        }
        Err(Status::invalid(format!("no solution of the equations for {} found from the guesses (MK-11.8); give guesses nearer the solution", names())))
    }

    /// Writes the continuous state vector `y` into the binding values.
    pub fn load_y(&self, vals: &mut [Value], y: &[f64]) {
        for c in &self.cont {
            let s = &y[c.offset..c.offset + c.len];
            vals[c.binding] = match &self.bindings[c.binding].ty {
                Type::Quantity { .. } => Value::Num(s[0]),
                Type::Field { .. } => Value::Field(std::rc::Rc::new(s.to_vec())),
                Type::Point { .. } => Value::Point(Arr::from_slice(s)),
                _ => Value::Vec(Arr::from_slice(s)),
            };
        }
    }

    /// Reads the continuous state vector `y` from the binding values.
    pub fn store_y(&self, vals: &[Value], y: &mut [f64]) {
        for c in &self.cont {
            vals[c.binding].write_to(&mut y[c.offset..c.offset + c.len]);
        }
    }

    /// The combined flow of every continuous state (MK-14.5), written into `dy`.
    /// `vals` must hold current stored values and derived bindings.
    pub fn rhs(&self, vals: &[Value], t: f64, t0: f64, dy: &mut [f64]) -> Result<(), Status> {
        self.rhs_noisy(vals, t, t0, &[], dy)
    }

    /// The flows with the white noises at the values `noise` (D-090): during an
    /// Euler-Maruyama step `ΔW / h` per noise, elsewhere none (each noise 0, the drift).
    pub fn rhs_noisy(&self, vals: &[Value], t: f64, t0: f64, noise: &[f64], dy: &mut [f64]) -> Result<(), Status> {
        let c = Ctx { vals, der: None, t, t0, args: &[], payloads: &[], draws: None, noise, point: None };
        for s in &self.cont {
            let out = &mut dy[s.offset..s.offset + s.len];
            // A field's flows are evaluated in each cell (D-091).
            if let Some(g) = &self.bindings[s.binding].field {
                out.iter_mut().for_each(|x| *x = 0.0);
                let es: Vec<&CExpr> = match &s.flows {
                    FlowSet::None => vec![],
                    FlowSet::Define(e) => vec![e],
                    FlowSet::Sum(es) => es.iter().collect(),
                };
                for e in es {
                    let c = Ctx { vals, der: None, t, t0, args: &[], payloads: &[], draws: None, noise, point: None };
                    if let Some(v) = e.eval_cells(&c, g) {
                        out.iter_mut().zip(&v).for_each(|(o, x)| *o += x);
                        continue;
                    }
                    for (k, o) in out.iter_mut().enumerate() {
                        let args = [g.coord(k)];
                        let inner = Ctx { vals, der: None, t, t0, args: &args, payloads: &[], draws: None, noise, point: Some(k) };
                        *o += e.eval(&inner)?.num();
                    }
                }
                continue;
            }
            match &s.flows {
                crate::model::FlowSet::None => out.iter_mut().for_each(|x| *x = 0.0),
                FlowSet::Define(e) => e.eval(&c)?.write_to(out),
                FlowSet::Sum(es) => {
                    out.iter_mut().for_each(|x| *x = 0.0);
                    let mut tmp = [0.0; 3];
                    for e in es {
                        let v = e.eval(&c)?;
                        v.write_to(&mut tmp[..s.len]);
                        for k in 0..s.len {
                            out[k] += tmp[k];
                        }
                    }
                }
            }
        }
        Ok(())
    }

    /// Combined flows by binding index, for expressions that read `der(x)` (MK-11.5).
    pub fn der_values(&self, vals: &[Value], t: f64, t0: f64) -> Result<Vec<Value>, Status> {
        let mut dy = vec![0.0; self.n_y()];
        self.rhs(vals, t, t0, &mut dy)?;
        let mut out = vec![Value::Num(0.0); self.bindings.len()];
        for c in &self.cont {
            let s = &dy[c.offset..c.offset + c.len];
            out[c.binding] = if self.bindings[c.binding].field.is_some() {
                Value::Field(std::rc::Rc::new(s.to_vec()))
            } else if c.len == 1 && matches!(self.bindings[c.binding].ty, Type::Quantity { .. }) {
                Value::Num(s[0])
            } else {
                Value::Vec(Arr::from_slice(s))
            };
        }
        Ok(out)
    }
}

/// Solves `a x = b` by Gaussian elimination with partial pivoting; `None` when `a` is
/// singular to working precision. Rows are equations and columns unknowns, each in its own
/// unit (a diode's row in amperes per volt can hold 1e23 beside a resistor's 1000), so each
/// row and then each column is first scaled to a largest entry of 1: singularity is judged
/// in the scaled matrix, where units play no part.
fn gauss(mut a: Vec<Vec<f64>>, mut b: Vec<f64>) -> Option<Vec<f64>> {
    let n = b.len();
    for (row, y) in a.iter_mut().zip(b.iter_mut()) {
        let m = row.iter().fold(0.0f64, |m, x| m.max(x.abs()));
        if !(m > 0.0 && m.is_finite()) {
            return None;
        }
        row.iter_mut().for_each(|x| *x /= m);
        *y /= m;
    }
    let cols: Vec<f64> = (0..n).map(|j| a.iter().fold(0.0f64, |m, row| m.max(row[j].abs()))).collect();
    if cols.iter().any(|m| !(*m > 0.0)) {
        return None;
    }
    for row in a.iter_mut() {
        row.iter_mut().zip(&cols).for_each(|(x, m)| *x /= m);
    }
    for c in 0..n {
        let p = (c..n).max_by(|&i, &j| a[i][c].abs().total_cmp(&a[j][c].abs()))?;
        if !(a[p][c].abs() > 1e-13) {
            return None;
        }
        a.swap(c, p);
        b.swap(c, p);
        for r in c + 1..n {
            let f = a[r][c] / a[c][c];
            if f != 0.0 {
                for k in c..n {
                    a[r][k] -= f * a[c][k];
                }
                b[r] -= f * b[c];
            }
        }
    }
    let mut x = vec![0.0; n];
    for r in (0..n).rev() {
        let s: f64 = (r + 1..n).map(|k| a[r][k] * x[k]).sum();
        x[r] = (b[r] - s) / a[r][r];
    }
    x.iter_mut().zip(&cols).for_each(|(x, m)| *x /= m);
    x.iter().all(|v| v.is_finite()).then_some(x)
}
