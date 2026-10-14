//! Enforced constraints (MK-12.6 to MK-12.9, D-093).
//!
//! A constraint with the policy `enforce` is an equation `a == b` the runtime keeps true. When
//! it reads only state (a rod `|bob - pivot| == L`), it is differentiated with respect to time,
//! each state replaced by its flow, until the result reads an unknown (the rod's tension, read
//! by the bob's flow): that rate of the constraint is solved with the other equations, and the
//! lower rates, which read state only, are what the runtime projects the state onto. A
//! constraint whose rates vanish before one reads an unknown (a point held on a circle in a
//! model without flows) is enforced by projection alone.

use crate::{BinOp, Builtin, Equation, EquationRole, Expr, FlowKind, Func, Model, Policy, Role};
use std::collections::BTreeSet;

/// The most times a constraint is differentiated: a position held by a force needs two.
pub const MAX_RATES: usize = 3;

/// An enforced constraint, differentiated.
#[derive(Clone, Debug, PartialEq)]
pub struct Rates {
    /// The constraint and its rates that read no unknown, from the constraint itself: the
    /// state is projected onto all of them.
    pub levels: Vec<(Expr, Expr)>,
    /// The first rate that reads an unknown, solved with the model's equations; `None` when a
    /// rate vanishes first (projection alone).
    pub solved: Option<(Expr, Expr)>,
}

/// The identity of the solved rate of constraint `id`, as an equation of the elaborated model.
pub fn equation_id(id: &str) -> String {
    format!("{id}@solved")
}

/// The constraint `cond` (an equation `a == b`) and its rates; an error says why it cannot be
/// enforced.
pub fn rates(m: &Model, cond: &Expr) -> Result<Rates, String> {
    // An inequality is differentiated as its boundary, `a == b` (D-098).
    let Expr::Bin { bin: BinOp::Eq | BinOp::Ge | BinOp::Le, l, r } = cond else {
        return Err("an enforced constraint is an equation `a == b` or an inequality `a >= b`, `a <= b`".into());
    };
    let mut cur = ((**l).clone(), (**r).clone());
    let mut levels = vec![];
    for _ in 0..=MAX_RATES {
        if reads_unknown(m, &cur.0) || reads_unknown(m, &cur.1) {
            return Ok(Rates { levels, solved: Some(cur) });
        }
        levels.push(cur.clone());
        if levels.len() > MAX_RATES {
            break;
        }
        let (dl, dr) = (ddt(m, &cur.0)?, ddt(m, &cur.1)?);
        if dl.is_none() && dr.is_none() {
            return Ok(Rates { levels, solved: None });
        }
        cur = (dl.unwrap_or_else(zero), dr.unwrap_or_else(zero));
    }
    Err(format!("neither it nor its first {MAX_RATES} rates read an unknown; declare what holds it (a force) as an `unknown` and read it in the flows"))
}

/// Adds, for each enforced constraint whose rates reach an unknown, that rate as an equation
/// solved with the others (MK-12.7). Constraints that cannot be enforced are left as they are:
/// the checker reports them.
pub fn elaborate(mut m: Model) -> Model {
    // Each flowing state's rate as a hidden derived binding of the rate's type, so that a
    // flow read inside a rate keeps the expected type its tuples need (D-032).
    let mut rates_of = vec![];
    for b in m.bindings.iter().filter(|b| b.role == Role::Continuous) {
        let Some(ty) = rate_type(&b.ty) else { continue };
        let flows: Vec<&crate::Flow> = m.flows.iter().filter(|f| f.target == b.id).collect();
        let def = match flows.iter().find(|f| f.kind == FlowKind::Define) {
            Some(f) => Some(f.expr.clone()),
            None => flows.iter().fold(None, |acc, f| add(acc, Some(f.expr.clone()))),
        };
        let Some(def) = def else { continue };
        let mut noisy = false;
        def.walk(&mut |x| noisy |= matches!(x, Expr::Builtin { builtin: Builtin::Noise }));
        if noisy {
            continue;
        }
        rates_of.push(crate::Binding {
            id: flow_id(&b.id),
            name: format!("der({})", b.name),
            role: Role::Derived,
            ty,
            init: None,
            def: Some(def),
            intervenable: None,
            private: true,
            display: Default::default(),
            notes: vec![],
            when: None,
            combine: None,
            boundary: None,
        });
    }
    m.bindings.extend(rates_of);
    let mut extra = vec![];
    for k in m.constraints.iter().filter(|k| k.policy == Policy::Enforce) {
        if let Ok(Rates { levels, solved: Some((lhs, rhs)) }) = rates(&m, &k.cond) {
            let name = match levels.len() {
                0 => k.name.clone(),
                1 => format!("rate of {}", k.name),
                n => format!("rate {n} of {}", k.name),
            };
            extra.push(Equation { id: equation_id(&k.id), name, lhs, rhs, role: EquationRole::Display, tol: None });
        }
    }
    m.equations.extend(extra);
    // D-098: an enforced inequality is in contact or not, a hidden discrete state switched by
    // hidden events: it comes into contact when its gap closes, and leaves contact when the
    // force that holds it would pull.
    let mut bindings = vec![];
    let mut events = vec![];
    for k in m.constraints.iter().filter(|k| k.policy == Policy::Enforce) {
        let (Some((l, r)), Some(force)) = (inequality(&k.cond), &k.force) else { continue };
        let gap = bin(BinOp::Sub, l.clone(), r.clone());
        let contact = contact_id(&k.id);
        let at = |x: &str| Expr::Ref { r#ref: x.to_string() };
        bindings.push(crate::Binding {
            id: contact.clone(),
            name: format!("contact of {}", k.name),
            role: Role::Discrete,
            ty: crate::Type::Boolean,
            // In contact from the start when the gap is closed or closing past the boundary.
            init: Some(bin(BinOp::Le, gap.clone(), zero())),
            def: None,
            intervenable: None,
            private: true,
            display: Default::default(),
            notes: vec![],
            when: None,
            combine: None,
            boundary: None,
        });
        let set = |v: bool| crate::Op::Set { target: crate::Target { binding: contact.clone(), component: None, member: None }, value: Expr::Bool { bool: v } };
        let pulls = bin(BinOp::Lt, at(force), zero());
        let event = |id: &str, name: &str, trigger: crate::Trigger, enable: Expr, value: bool| crate::Event {
            id: format!("{}@{id}", k.id),
            name: format!("{name} {}", k.name),
            trigger,
            enable: Some(enable),
            handler: vec![set(value)],
            zeno: None,
            process: None,
            payload: None,
            each: None,
            notes: vec![],
        };
        let not = |e: Expr| Expr::Not { not: Box::new(e) };
        events.push(event("touch", "touch of", crate::Trigger::Falling { guard: gap }, not(at(&contact)), true));
        events.push(event("release", "release of", crate::Trigger::Falling { guard: at(force) }, at(&contact), false));
        // A contact that pulls as soon as it is made, or from the start, ends at once.
        events.push(event("settle", "settling of", crate::Trigger::On { event: format!("{}@touch", k.id) }, pulls.clone(), false));
        events.push(event("start", "start of", crate::Trigger::Start, bin(BinOp::And, at(&contact), pulls), false));
    }
    m.bindings.extend(bindings);
    m.events.extend(events);
    m
}

/// An inequality `a >= b` or `a <= b` as the two sides of its gap, `a - b >= 0` or
/// `b - a >= 0`.
pub fn inequality(cond: &Expr) -> Option<(Expr, Expr)> {
    match cond {
        Expr::Bin { bin: BinOp::Ge, l, r } => Some(((**l).clone(), (**r).clone())),
        Expr::Bin { bin: BinOp::Le, l, r } => Some(((**r).clone(), (**l).clone())),
        _ => None,
    }
}

/// The identity of the hidden discrete state that says whether inequality `id` is in contact.
pub fn contact_id(id: &str) -> String {
    format!("{id}@contact")
}

/// The identity of the hidden derived binding that holds the rate of state `id`.
pub fn flow_id(id: &str) -> String {
    format!("{id}@flow")
}

/// The type of the rate of a value of type `t`: a quantity or vector per time, a point's
/// velocity; `None` for other types.
pub(crate) fn rate_type(t: &crate::Type) -> Option<crate::Type> {
    let per_t = crate::Dim::time().inv();
    Some(match t {
        crate::Type::Quantity { dim } => crate::Type::Quantity { dim: dim.mul(&per_t) },
        crate::Type::Vector { space, dim } => crate::Type::Vector { space: space.clone(), dim: dim.mul(&per_t) },
        crate::Type::Point { space } => crate::Type::Vector { space: space.clone(), dim: crate::Dim::length().mul(&per_t) },
        _ => return None,
    })
}

/// Whether `e` reads an unknown, directly or through derived values.
pub fn reads_unknown(m: &Model, e: &Expr) -> bool {
    let mut seen = BTreeSet::new();
    let mut stack = e.refs();
    while let Some(id) = stack.pop() {
        if !seen.insert(id.clone()) {
            continue;
        }
        let Some(b) = m.binding(&id) else { continue };
        match b.role {
            Role::Unknown => return true,
            Role::Derived => stack.extend(b.def.iter().flat_map(|d| d.refs())),
            _ => {}
        }
    }
    false
}

/// The bindings `e` reads that hold values (not derived), through derived values.
pub fn stored_reads(m: &Model, e: &Expr) -> BTreeSet<String> {
    let mut seen = BTreeSet::new();
    let mut out = BTreeSet::new();
    let mut stack = e.refs();
    while let Some(id) = stack.pop() {
        if !seen.insert(id.clone()) {
            continue;
        }
        let Some(b) = m.binding(&id) else { continue };
        if b.role == Role::Derived {
            stack.extend(b.def.iter().flat_map(|d| d.refs()));
        } else {
            out.insert(id);
        }
    }
    out
}

fn zero() -> Expr {
    Expr::Num { num: 0.0, unit: None }
}

fn num(x: f64) -> Expr {
    Expr::Num { num: x, unit: None }
}

fn bin(op: BinOp, l: Expr, r: Expr) -> Expr {
    Expr::Bin { bin: op, l: Box::new(l), r: Box::new(r) }
}

fn call(f: Func, args: Vec<Expr>) -> Expr {
    Expr::Call { call: f, args }
}

fn neg(e: Expr) -> Expr {
    Expr::Neg { neg: Box::new(e) }
}

fn add(a: Option<Expr>, b: Option<Expr>) -> Option<Expr> {
    match (a, b) {
        (Some(a), Some(b)) => Some(bin(BinOp::Add, a, b)),
        (a, None) => a,
        (None, b) => b,
    }
}

fn sub(a: Option<Expr>, b: Option<Expr>) -> Option<Expr> {
    match (a, b) {
        (Some(a), Some(b)) => Some(bin(BinOp::Sub, a, b)),
        (a, None) => a,
        (None, Some(b)) => Some(neg(b)),
    }
}

/// `a * d`, with `d` the rate of something (none when it is constant).
fn times(a: Expr, d: Option<Expr>) -> Option<Expr> {
    d.map(|d| bin(BinOp::Mul, a, d))
}

/// The rate of `e` along the model's flows: each continuous state replaced by its flow,
/// parameters, constants, inputs and discrete state constant between transitions, derived
/// values through their definitions. `None` is a rate of zero.
pub fn ddt(m: &Model, e: &Expr) -> Result<Option<Expr>, String> {
    let d = |x: &Expr| ddt(m, x);
    Ok(match e {
        Expr::Num { .. } | Expr::Bool { .. } | Expr::Case { .. } | Expr::Const { .. } | Expr::Origin { .. } => None,
        Expr::Builtin { builtin: Builtin::T | Builtin::Elapsed } => Some(num(1.0)),
        Expr::Builtin { builtin: Builtin::T0 | Builtin::Index } => None,
        Expr::Builtin { builtin: Builtin::Noise } => return Err("white noise has no rate".into()),
        Expr::Ref { r#ref } => {
            let Some(b) = m.binding(r#ref) else { return Err(format!("`{}` is not a binding of the model", r#ref)) };
            match b.role {
                Role::Constant | Role::Parameter | Role::Input | Role::Discrete => None,
                Role::Continuous => {
                    if matches!(b.ty, crate::Type::Field { .. }) {
                        return Err(format!("`{}` is a field; constraints on fields are a later extension", b.name));
                    }
                    if m.binding(&flow_id(r#ref)).is_some() {
                        return Ok(Some(Expr::Ref { r#ref: flow_id(r#ref) }));
                    }
                    let flows: Vec<&crate::Flow> = m.flows.iter().filter(|f| &f.target == r#ref).collect();
                    match flows.iter().find(|f| f.kind == FlowKind::Define) {
                        Some(f) => Some(f.expr.clone()),
                        None => flows.iter().fold(None, |acc, f| add(acc, Some(f.expr.clone()))),
                    }
                }
                Role::Derived => {
                    if b.when.is_some() {
                        return Err(format!("`{}` has a value only while a condition holds, so it has no rate", b.name));
                    }
                    match &b.def {
                        Some(def) => d(def)?,
                        None => None,
                    }
                }
                Role::Unknown => return Err(format!("the rate of the unknown `{}` is not known", b.name)),
            }
        }
        Expr::Neg { neg: x } => d(x)?.map(neg),
        Expr::Norm { norm: v } => d(v)?.map(|dv| bin(BinOp::Div, call(Func::Dot, vec![(**v).clone(), dv]), e.clone())),
        Expr::Bin { bin: op, l, r } => {
            let (a, b) = (&**l, &**r);
            match op {
                BinOp::Add => add(d(a)?, d(b)?),
                BinOp::Sub => sub(d(a)?, d(b)?),
                BinOp::Mul => add(d(a)?.map(|da| bin(BinOp::Mul, da, b.clone())), times(a.clone(), d(b)?)),
                // (a / b)' = a' / b - a b' / b²
                BinOp::Div => sub(
                    d(a)?.map(|da| bin(BinOp::Div, da, b.clone())),
                    d(b)?.map(|db| bin(BinOp::Div, bin(BinOp::Mul, a.clone(), db), bin(BinOp::Mul, b.clone(), b.clone()))),
                ),
                BinOp::Pow => {
                    let Expr::Num { num: n, unit: None } = b else {
                        return Err("a power's exponent is a number to differentiate it".into());
                    };
                    let Some(da) = d(a)? else { return Ok(None) };
                    let base = match *n - 1.0 {
                        k if k == 0.0 => None,
                        k if k == 1.0 => Some(a.clone()),
                        k => Some(bin(BinOp::Pow, a.clone(), num(k))),
                    };
                    let scaled = match base {
                        Some(p) => bin(BinOp::Mul, num(*n), p),
                        None => num(*n),
                    };
                    Some(bin(BinOp::Mul, scaled, da))
                }
                _ => return Err("a comparison or a logical operator has no rate".into()),
            }
        }
        Expr::Call { call: f, args } => {
            let a = &args[0];
            match f {
                Func::Sin => times(call(Func::Cos, vec![a.clone()]), d(a)?),
                Func::Cos => times(neg(call(Func::Sin, vec![a.clone()])), d(a)?),
                Func::Tan => d(a)?.map(|da| bin(BinOp::Div, da, bin(BinOp::Mul, call(Func::Cos, vec![a.clone()]), call(Func::Cos, vec![a.clone()])))),
                Func::Exp => times(e.clone(), d(a)?),
                Func::Log => d(a)?.map(|da| bin(BinOp::Div, da, a.clone())),
                Func::Sqrt => d(a)?.map(|da| bin(BinOp::Div, da, bin(BinOp::Mul, num(2.0), e.clone()))),
                Func::Abs => d(a)?.map(|da| bin(BinOp::Div, bin(BinOp::Mul, a.clone(), da), e.clone())),
                // atan2(y, x)' = (x y' - y x') / (x² + y²)
                Func::Atan2 => {
                    let (y, x) = (a, &args[1]);
                    sub(times(x.clone(), d(y)?), times(y.clone(), d(x)?)).map(|n| bin(BinOp::Div, n, bin(BinOp::Add, bin(BinOp::Mul, x.clone(), x.clone()), bin(BinOp::Mul, y.clone(), y.clone()))))
                }
                Func::Dot => {
                    let b = &args[1];
                    add(d(a)?.map(|da| call(Func::Dot, vec![da, b.clone()])), d(b)?.map(|db| call(Func::Dot, vec![a.clone(), db])))
                }
                // (a × b)' = a' × b + a × b'
                Func::Cross => {
                    let b = &args[1];
                    add(d(a)?.map(|da| call(Func::Cross, vec![da, b.clone()])), d(b)?.map(|db| call(Func::Cross, vec![a.clone(), db])))
                }
                Func::Min | Func::Max => return Err("`min` and `max` have no rate where their arguments meet".into()),
                Func::Laplacian | Func::Gradient | Func::Integral | Func::Mean => return Err("constraints on fields are a later extension".into()),
            }
        }
        Expr::Tuple { tuple } => {
            let ds: Vec<Option<Expr>> = tuple.iter().map(d).collect::<Result<_, _>>()?;
            if ds.iter().all(Option::is_none) {
                None
            } else {
                Some(Expr::Tuple { tuple: ds.into_iter().map(|x| x.unwrap_or_else(zero)).collect() })
            }
        }
        Expr::Comp { comp, axis } => d(comp)?.map(|x| Expr::Comp { comp: Box::new(x), axis: *axis }),
        Expr::Der { .. } => return Err("a rate inside an enforced constraint; write the constraint on the state itself".into()),
        Expr::If { .. } | Expr::Match { .. } | Expr::Otherwise { .. } => return Err("a choice between values (`if`, `match`, `otherwise`) has no rate where it switches".into()),
        _ => return Err("this form of expression cannot be differentiated; write the constraint with arithmetic and the functions `sqrt`, `sin`, `cos`, `tan`, `exp`, `log`, `abs`, `atan2`, `dot`".into()),
    })
}
