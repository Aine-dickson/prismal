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
}

#[derive(Clone, Debug)]
pub struct CConstraint {
    pub id: Id,
    pub name: String,
    pub cond: Option<CExpr>,
    /// An equality with tolerance: `|lhs - rhs| <= tol`.
    pub within: Option<(CExpr, CExpr, f64)>,
    pub policy: Policy,
}

/// A checked model, ready to run.
#[derive(Clone, Debug)]
pub struct CModel {
    pub ir: Model,
    pub spaces: Vec<Space>,
    pub index: HashMap<Id, usize>,
    pub bindings: Vec<CBinding>,
    pub derived_order: Vec<usize>,
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
        for &i in &self.init_order {
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
        Ok(vals)
    }

    /// Recomputes every derived binding from the stored values (MK-6.4).
    pub fn update_derived(&self, vals: &mut [Value], t: f64, t0: f64) -> Result<(), Status> {
        for &i in &self.derived_order {
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
