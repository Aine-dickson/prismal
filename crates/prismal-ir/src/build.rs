//! A Rust API for building IR (D-002: kernel-first; the text syntax lowers to the same IR).
//!
//! Identities follow declaration paths (D-036): `Model.name` for bindings,
//! `Model.event.name` for events, and so on.

use crate::expr::{BinOp, Builtin, Constant, Expr, Func, Lambda};
use crate::units::Unit;
use crate::*;

// ---------------------------------------------------------------- expressions

/// A literal with a unit: `num(9.81, "m/s^2")`. An empty unit gives a bare literal.
pub fn num(v: f64, unit: &str) -> Expr {
    if unit.is_empty() {
        Expr::Num { num: v, unit: None }
    } else {
        Expr::Num { num: v, unit: Some(Unit::parse(unit).expect("valid unit")) }
    }
}
/// A bare literal (dimensionless, or any dimension for `0`, MK-3.8).
pub fn lit(v: f64) -> Expr {
    Expr::Num { num: v, unit: None }
}
pub fn boolean(b: bool) -> Expr {
    Expr::Bool { bool: b }
}
pub fn r(id: &str) -> Expr {
    Expr::Ref { r#ref: id.to_string() }
}
pub fn der(id: &str) -> Expr {
    Expr::Der { der: id.to_string() }
}
pub fn tuple(items: Vec<Expr>) -> Expr {
    Expr::Tuple { tuple: items }
}
pub fn comp(e: Expr, axis: usize) -> Expr {
    Expr::Comp { comp: Box::new(e), axis }
}
pub fn norm(e: Expr) -> Expr {
    Expr::Norm { norm: Box::new(e) }
}
pub fn call(f: Func, args: Vec<Expr>) -> Expr {
    Expr::Call { call: f, args }
}
pub fn sin(e: Expr) -> Expr {
    call(Func::Sin, vec![e])
}
pub fn cos(e: Expr) -> Expr {
    call(Func::Cos, vec![e])
}
pub fn sqrt(e: Expr) -> Expr {
    call(Func::Sqrt, vec![e])
}
pub fn pow(a: Expr, b: Expr) -> Expr {
    Expr::bin(BinOp::Pow, a, b)
}
pub fn ite(c: Expr, a: Expr, b: Expr) -> Expr {
    Expr::If { r#if: Box::new(c), then: Box::new(a), r#else: Box::new(b) }
}
pub fn not(e: Expr) -> Expr {
    Expr::Not { not: Box::new(e) }
}
pub fn and(a: Expr, b: Expr) -> Expr {
    Expr::bin(BinOp::And, a, b)
}
pub fn or(a: Expr, b: Expr) -> Expr {
    Expr::bin(BinOp::Or, a, b)
}
pub fn eq(a: Expr, b: Expr) -> Expr {
    Expr::bin(BinOp::Eq, a, b)
}
pub fn lt(a: Expr, b: Expr) -> Expr {
    Expr::bin(BinOp::Lt, a, b)
}
pub fn le(a: Expr, b: Expr) -> Expr {
    Expr::bin(BinOp::Le, a, b)
}
pub fn gt(a: Expr, b: Expr) -> Expr {
    Expr::bin(BinOp::Gt, a, b)
}
pub fn ge(a: Expr, b: Expr) -> Expr {
    Expr::bin(BinOp::Ge, a, b)
}
pub fn t() -> Expr {
    Expr::Builtin { builtin: Builtin::T }
}
pub fn t0() -> Expr {
    Expr::Builtin { builtin: Builtin::T0 }
}
pub fn elapsed() -> Expr {
    Expr::Builtin { builtin: Builtin::Elapsed }
}
/// The constant π, kept by name (D-039).
pub fn pi() -> Expr {
    Expr::Const { r#const: Constant::Pi }
}
pub fn origin(space: &str) -> Expr {
    Expr::Origin { origin: space.to_string() }
}
pub fn param(i: usize) -> Expr {
    Expr::Param { param: i }
}
pub fn lambda(params: Vec<Type>, body: Expr) -> Expr {
    Expr::Lambda { lambda: Lambda { params, body: Box::new(body), names: vec![] } }
}
/// A lambda that keeps its parameter names for display (D-034).
pub fn lambda_named(names: &[&str], params: Vec<Type>, body: Expr) -> Expr {
    Expr::Lambda { lambda: Lambda { params, body: Box::new(body), names: names.iter().map(|n| n.to_string()).collect() } }
}
pub fn apply(f: Expr, args: Vec<Expr>) -> Expr {
    Expr::Apply { apply: Box::new(f), args }
}
/// `x in [lo, hi)` and similar: the interval lowered to comparisons (D-028).
pub fn interval(x: Expr, lo: Expr, lo_closed: bool, hi: Expr, hi_closed: bool) -> Expr {
    let a = if lo_closed { le(lo, x.clone()) } else { lt(lo, x.clone()) };
    let b = if hi_closed { le(x, hi) } else { lt(x, hi) };
    and(a, b)
}

pub fn set(binding: &str, value: Expr) -> Op {
    Op::Set { target: Target::of(binding), value }
}
pub fn set_comp(binding: &str, component: usize, value: Expr) -> Op {
    Op::Set { target: Target { component: Some(component), ..Target::of(binding) }, value }
}

pub fn space(name: &str, dimension: u32) -> Space {
    let axes = ["x", "y", "z"].iter().take(dimension as usize).map(|s| s.to_string()).collect();
    Space { id: name.into(), name: name.into(), dimension, axes, notes: vec![] }
}

// ---------------------------------------------------------------- models

pub struct ModelBuilder {
    pub model: Model,
    flow_count: usize,
}

impl ModelBuilder {
    pub fn new(name: &str) -> ModelBuilder {
        ModelBuilder {
            model: Model {
                id: name.into(),
                name: name.into(),
                default_space: None,
                bindings: vec![],
                processes: vec![],
                flows: vec![],
                events: vec![],
                equations: vec![],
                objects: vec![],
                parts: vec![],
                ends: vec![],
                undirected: false,
                constraints: vec![],
                enums: vec![],
                grids: vec![],
                functions: vec![],
                notes: vec![],
            },
            flow_count: 0,
        }
    }

    /// Continues building an existing model (for one-line variants of a program).
    pub fn from_model(model: Model) -> ModelBuilder {
        let flow_count = model.flows.len();
        ModelBuilder { model, flow_count }
    }

    pub fn in_space(mut self, space: &str) -> Self {
        self.model.default_space = Some(space.into());
        self
    }

    fn bid(&self, name: &str) -> Id {
        format!("{}.{}", self.model.name, name)
    }

    fn add_binding(&mut self, name: &str, role: Role, ty: Type, init: Option<Expr>, def: Option<Expr>) -> Id {
        let id = self.bid(name);
        assert!(self.model.binding(&id).is_none(), "duplicate binding {id}");
        self.model.bindings.push(Binding {
            id: id.clone(),
            name: name.into(),
            role,
            ty,
            init,
            def,
            intervenable: None,
            private: false,
            display: Display::default(),
            notes: vec![],
            when: None,
            combine: None,
            boundary: None,
        });
        id
    }

    pub fn constant(&mut self, name: &str, ty: Type, value: Expr) -> Id {
        self.add_binding(name, Role::Constant, ty, Some(value), None)
    }
    pub fn param(&mut self, name: &str, ty: Type, default: Expr) -> Id {
        self.add_binding(name, Role::Parameter, ty, Some(default), None)
    }
    pub fn input(&mut self, name: &str, ty: Type) -> Id {
        self.add_binding(name, Role::Input, ty, None, None)
    }
    pub fn state(&mut self, name: &str, ty: Type, init: Expr) -> Id {
        self.add_binding(name, Role::Continuous, ty, Some(init), None)
    }
    pub fn discrete(&mut self, name: &str, ty: Type, init: Expr) -> Id {
        self.add_binding(name, Role::Discrete, ty, Some(init), None)
    }
    pub fn derived(&mut self, name: &str, ty: Type, def: Expr) -> Id {
        self.add_binding(name, Role::Derived, ty, None, Some(def))
    }

    /// Sets display metadata of a binding.
    pub fn symbol(&mut self, id: &str, symbol: &str) {
        let b = self.model.bindings.iter_mut().find(|b| b.id == id).expect("binding");
        b.display.symbol = Some(symbol.into());
    }

    /// Sets the display unit of a binding (MK-3.12).
    pub fn display_unit(&mut self, id: &str, unit: &str) {
        let b = self.model.bindings.iter_mut().find(|b| b.id == id).expect("binding");
        b.display.unit = Some(unit.into());
    }

    /// A constraint; `attached_to` records a `where` or `in` written on a parameter.
    pub fn constraint(&mut self, name: &str, cond: Expr, tol: Option<Expr>, policy: Policy, attached_to: Option<&str>) -> Id {
        let id = format!("{}.constraint.{}", self.model.name, name);
        self.model.constraints.push(Constraint {
            id: id.clone(),
            name: name.into(),
            cond,
            tol,
            policy,
            attached_to: attached_to.map(|s| s.to_string()),
        });
        id
    }
    /// `param x ... where cond` or `in I`: a `reject` constraint attached to the parameter (MK-12.4).
    pub fn range(&mut self, binding: &str, cond: Expr) -> Id {
        let name = format!("{}_range", binding.rsplit('.').next().unwrap_or(binding));
        self.constraint(&name, cond, None, Policy::Reject, Some(binding))
    }

    pub fn process(&mut self, name: &str) -> Id {
        let id = format!("{}.process.{}", self.model.name, name);
        self.model.processes.push(Process { id: id.clone(), name: name.into(), kind: ProcessKind::Continuous, notes: vec![] });
        id
    }

    fn add_flow(&mut self, target: &str, kind: FlowKind, expr: Expr, process: Option<&str>) -> Id {
        self.flow_count += 1;
        let id = format!("{}.flow.{}", self.model.name, self.flow_count);
        self.model.flows.push(Flow { id: id.clone(), target: target.into(), kind, expr, process: process.map(|s| s.into()), member: None, each: None });
        id
    }
    pub fn flow(&mut self, target: &str, expr: Expr) -> Id {
        self.add_flow(target, FlowKind::Define, expr, None)
    }
    pub fn contribute(&mut self, target: &str, expr: Expr) -> Id {
        self.add_flow(target, FlowKind::Contribute, expr, None)
    }
    pub fn contribute_in(&mut self, process: &str, target: &str, expr: Expr) -> Id {
        self.add_flow(target, FlowKind::Contribute, expr, Some(process))
    }

    pub fn event(&mut self, name: &str, trigger: Trigger, handler: Vec<Op>) -> Id {
        let id = format!("{}.event.{}", self.model.name, name);
        self.model.events.push(Event {
            id: id.clone(),
            name: name.into(),
            trigger,
            enable: None,
            handler,
            zeno: None,
            process: None,
            payload: None,
            each: None,
            notes: vec![],
        });
        id
    }
    pub fn zeno(&mut self, event: &str, policy: ZenoPolicy) {
        let e = self.model.events.iter_mut().find(|e| e.id == event).expect("event");
        e.zeno = Some(Zeno { policy, eps: None, n: None, window: None });
    }

    pub fn equation(&mut self, name: &str, lhs: Expr, rhs: Expr, check_tol: Option<Expr>) -> Id {
        let id = format!("{}.equation.{}", self.model.name, name);
        let role = if check_tol.is_some() { EquationRole::Check } else { EquationRole::Display };
        self.model.equations.push(Equation { id: id.clone(), name: name.into(), lhs, rhs, role, tol: check_tol });
        id
    }

    pub fn finish(self) -> Model {
        self.model
    }
}

pub fn falling(guard: Expr) -> Trigger {
    Trigger::Falling { guard }
}
pub fn rising(guard: Expr) -> Trigger {
    Trigger::Rising { guard }
}
