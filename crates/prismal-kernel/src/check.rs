//! Static checking and compilation of a model (MK sections 3, 4, 13 to 16, 18).
//!
//! The checker types every expression with expected-type propagation (D-030, D-032),
//! reports the static diagnostics of MK section 18, and produces a compiled model.

use crate::eval::{Arr, CDraw, CEdge, CExpr, CFieldOp, CGrid, FieldOp, Value};
use crate::random::fnv1a;
use crate::model::*;
use prismal_ir::*;
use std::collections::{BTreeSet, HashMap};
use std::rc::Rc;

/// A static diagnostic (MK section 18).
#[derive(Clone, Debug, PartialEq)]
pub struct Diagnostic {
    pub code: &'static str,
    pub message: String,
    /// The element the diagnostic is about.
    pub element: String,
}

impl std::fmt::Display for Diagnostic {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} at {}: {}", self.code, self.element, self.message)
    }
}

/// Type information about the bindings of a model, used while checking expressions.
pub(crate) struct Scope<'a> {
    pub spaces: &'a [Space],
    pub model: &'a Model,
    pub index: &'a HashMap<Id, usize>,
    /// Declared functions compiled so far, with their types (D-048).
    pub funcs: &'a HashMap<Id, (Rc<CExpr>, Type)>,
    /// The model's grids, compiled (D-091).
    pub grids: &'a HashMap<Id, CGrid>,
}

pub(crate) struct Tc<'a, 'b> {
    pub scope: &'b Scope<'a>,
    pub diags: &'b mut Vec<Diagnostic>,
    pub element: String,
    lambda: Vec<Vec<Type>>,
    /// While checking a declared function's body: its name. The body reads only its
    /// parameters, constants and other declared functions (MK-10.3).
    closed: Option<String>,
    /// While checking an event with a payload: its identity, index and payload type (D-050).
    pub payload: Option<(Id, usize, Type)>,
    /// While checking an event's condition and operations: the event's index, the number of
    /// its identity and the next draw site's number (D-083).
    pub draws: Option<(usize, u64, u32)>,
    /// While checking a flow: the number of its first noise in the model and how many it
    /// has so far (D-090).
    pub noise: Option<(usize, usize)>,
    /// While checking a field's initial value, definition or flow: its grid. A field of that
    /// grid is read in the cell evaluated (D-091).
    pub pointwise: Option<Id>,
}

type Typed = Option<(CExpr, Type)>;

fn same_type(a: &Type, b: &Type) -> bool {
    a == b
}

fn is_dimensioned(t: &Type) -> bool {
    match t {
        Type::Quantity { dim } => !dim.is_none(),
        Type::Vector { dim, .. } => !dim.is_none(),
        Type::Point { .. } | Type::Instant { .. } => true,
        _ => false,
    }
}

impl<'a, 'b> Tc<'a, 'b> {
    pub fn new(scope: &'b Scope<'a>, diags: &'b mut Vec<Diagnostic>, element: &str) -> Self {
        Tc { scope, diags, element: element.to_string(), lambda: vec![], closed: None, payload: None, draws: None, noise: None, pointwise: None }
    }

    /// MK-E18: a declared function's body reads something other than its parameters,
    /// constants and declared functions.
    fn open_read(&mut self, what: &str) -> Typed {
        let f = self.closed.clone().unwrap_or_default();
        self.err("MK-E18", format!("declared function `{f}` reads {what}; a function takes what it needs as a parameter (MK-10.3)"))
    }

    fn err(&mut self, code: &'static str, message: String) -> Typed {
        // One expression may be checked several times, as in the members a `create` may
        // make (D-057): a diagnostic is reported once.
        let d = Diagnostic { code, message, element: self.element.clone() };
        if !self.diags.contains(&d) {
            self.diags.push(d);
        }
        None
    }

    /// A value of type `t` that is not a number, for a binding that has no value (D-058).
    fn placeholder(&self, t: &Type) -> Value {
        match t {
            Type::Boolean => Value::Bool(false),
            Type::Vector { space, .. } => Value::Vec(Arr::from_slice(&vec![f64::NAN; self.space_dim(space).unwrap_or(2)])),
            Type::Point { space } => Value::Point(Arr::from_slice(&vec![f64::NAN; self.space_dim(space).unwrap_or(2)])),
            Type::Tuple { items } => Value::Tuple(Rc::new(items.iter().map(|i| self.placeholder(i)).collect())),
            Type::Enum { .. } => Value::Case(0),
            Type::Function { .. } => Value::Func(Rc::new(CExpr::Const(Value::Num(f64::NAN)))),
            _ => Value::Num(f64::NAN),
        }
    }

    fn space_dim(&self, space: &str) -> Option<usize> {
        self.scope.spaces.iter().find(|s| s.id == space).map(|s| s.dimension as usize)
    }

    fn binding(&self, id: &str) -> Option<(usize, &'a Binding)> {
        self.scope.index.get(id).map(|&i| (i, &self.scope.model.bindings[i]))
    }

    /// True if an expression takes its type from its context (a tuple, a bare literal).
    fn needs_ctx(e: &Expr) -> bool {
        match e {
            Expr::Tuple { .. } => true,
            Expr::Num { unit: None, .. } | Expr::Const { .. } | Expr::Case { .. } => true,
            Expr::Neg { neg } => Self::needs_ctx(neg),
            Expr::If { then, r#else, .. } => Self::needs_ctx(then) && Self::needs_ctx(r#else),
            Expr::Match { arms, .. } => arms.iter().all(|a| Self::needs_ctx(&a.value)),
            _ => false,
        }
    }

    /// Checks `e` against a required type and reports a mismatch.
    pub fn expect(&mut self, e: &Expr, want: &Type) -> Option<CExpr> {
        let (c, t) = self.expr(e, Some(want))?;
        if same_type(&t, want) {
            return Some(c);
        }
        self.mismatch(&t, want);
        None
    }

    fn mismatch(&mut self, got: &Type, want: &Type) {
        let (code, what) = match (got, want) {
            (Type::Tuple { .. }, Type::Vector { .. } | Type::Point { .. }) => ("MK-E21", "a tuple with no expected vector type"),
            (Type::Point { .. }, Type::Vector { .. }) | (Type::Vector { .. }, Type::Point { .. }) => ("MK-E04", "point and vector confused"),
            (Type::Vector { space: a, .. }, Type::Vector { space: b, .. }) if a != b => ("MK-E05", "values from different spaces"),
            (Type::Point { space: a }, Type::Point { space: b }) if a != b => ("MK-E05", "values from different spaces"),
            _ => ("MK-E01", "type or dimension mismatch"),
        };
        self.err(code, format!("{what}: found {}, expected {}", show(got), show(want)));
    }

    pub fn expr(&mut self, e: &Expr, exp: Option<&Type>) -> Typed {
        match e {
            Expr::Num { num, unit } => self.num(*num, unit.as_ref(), exp),
            Expr::Bool { bool } => Some((CExpr::Const(Value::Bool(*bool)), Type::Boolean)),
            Expr::Case { case } => match exp {
                Some(t @ Type::Enum { cases, .. }) => match cases.iter().position(|c| c == case) {
                    Some(i) => Some((CExpr::Const(Value::Case(i as u32)), exp.unwrap().clone())),
                    None => self.err("MK-E01", format!("`{case}` is not a case of {}", show(t))),
                },
                _ => self.err("MK-E01", format!("enumeration case `{case}` without an expected enumeration")),
            },
            Expr::Ref { r#ref } => match self.binding(r#ref) {
                Some((_, b)) if self.closed.is_some() && b.role != Role::Constant => {
                    let what = format!("the {} `{}`", role_word(b.role), b.name);
                    self.open_read(&what)
                }
                // D-091: in a pointwise expression a field of its grid is read in the cell.
                Some((i, b)) => match (&b.ty, &self.pointwise) {
                    (Type::Field { grid, of }, Some(p)) if grid == p => Some((CExpr::FieldAt(i), (**of).clone())),
                    (Type::Field { .. }, Some(_)) => self.err("MK-E31", format!("`{}` lies on another grid; a pointwise expression reads fields of its own grid", b.name)),
                    _ => Some((CExpr::Load(i), b.ty.clone())),
                },
                None => self.err("MK-E00", format!("unknown binding `{ref}`", ref = r#ref)),
            },
            Expr::Fn { r#fn } => match self.scope.funcs.get(r#fn) {
                Some((body, ty)) => Some((CExpr::Const(Value::Func(body.clone())), ty.clone())),
                None => match self.scope.model.function(r#fn) {
                    // Declared but not compiled: its own diagnostics are reported with it.
                    Some(_) => None,
                    None => self.err("MK-E00", format!("unknown function `{}`", r#fn)),
                },
            },
            Expr::Match { r#match, arms } => self.match_expr(r#match, arms, exp),
            // `min` or `max` over the members alive (D-057): quantities of one type.
            Expr::Extreme { extreme, terms } => {
                let mut t: Option<Type> = None;
                let mut out = vec![];
                for g in terms {
                    let w = self.expect(&g.when, &Type::Boolean)?;
                    let v = match &t {
                        Some(want) => self.expect(&g.value, want)?,
                        None => {
                            let (v, vt) = self.expr(&g.value, exp)?;
                            t = Some(vt);
                            v
                        }
                    };
                    out.push((w, v));
                }
                match t {
                    Some(t @ Type::Quantity { .. }) => Some((CExpr::Extreme(*extreme, out), t)),
                    Some(other) => self.err("MK-E01", format!("`min` and `max` compare quantities, not {}", show(&other))),
                    None => self.err("MK-E26", "`min` or `max` over no member has no value".into()),
                }
            }
            // A binding of the member chosen by number (D-058): the items have one type.
            Expr::Pick { pick, from } => {
                let k = self.expect(pick, &Type::real())?;
                let mut t: Option<Type> = None;
                let mut items = vec![];
                for x in from {
                    let c = match &t {
                        Some(want) => self.expect(x, want)?,
                        None => {
                            let (c, xt) = self.expr(x, exp)?;
                            t = Some(xt);
                            c
                        }
                    };
                    items.push(c);
                }
                match t {
                    Some(t) => Some((CExpr::Pick(Box::new(k), items), t)),
                    None => self.err("MK-E26", "a choice among no member".into()),
                }
            }
            // Members and aggregates are replaced by elaboration before checking (D-055).
            Expr::Field { .. } | Expr::Part { .. } | Expr::Item { .. } | Expr::Var { .. } | Expr::Aggregate { .. } | Expr::End { .. } | Expr::Other { .. } | Expr::Has { .. } => {
                self.err("MK-E00", "a member or an aggregate in a model that was not elaborated (D-055)".into())
            }
            Expr::Payload { payload } => match &self.payload {
                Some((id, i, t)) if id == payload => Some((CExpr::Payload(*i), t.clone())),
                _ => self.err("MK-E01", format!("the payload of `{}` is read only in that event's condition and handler", payload.rsplit('.').next().unwrap_or(payload))),
            },
            Expr::Builtin { .. } | Expr::Der { .. } if self.closed.is_some() => self.open_read("the simulation's time or a derivative"),
            Expr::Param { param } => match self.lambda.last().and_then(|p| p.get(*param)) {
                Some(t) => Some((CExpr::Arg(*param), t.clone())),
                None => self.err("MK-E00", format!("parameter {param} outside a lambda")),
            },
            Expr::Builtin { builtin } => Some(match builtin {
                Builtin::T => (CExpr::Time, Type::Instant { dim: Dim::time() }),
                Builtin::T0 => (CExpr::Time0, Type::Instant { dim: Dim::time() }),
                Builtin::Elapsed => (CExpr::Elapsed, Type::Quantity { dim: Dim::time() }),
                Builtin::Index => return self.err("MK-E25", "`index` is the number of a member, read only in the overrides of a collection (D-055)".into()),
                // MK-14.16 (D-090): white noise, `T^(-1/2)`, only in flows.
                Builtin::Noise => match &mut self.noise {
                    Some((base, n)) => {
                        *n += 1;
                        (CExpr::Noise(*base + *n - 1), Type::Quantity { dim: Dim::time().pow(Ratio::new(-1, 2)) })
                    }
                    None => return self.err("MK-E29", "`noise` is white noise, written only in a flow: `der(x) = a + b * noise` (MK-14.16)".into()),
                },
            }),
            // A named constant is a bare dimensionless literal (MK-3.8, D-039).
            Expr::Const { r#const } => self.num(r#const.value(), None, exp),
            Expr::Origin { origin } => match self.space_dim(origin) {
                Some(n) => Some((CExpr::Const(Value::Point(Arr::zeros(n))), Type::Point { space: origin.clone() })),
                None => self.err("MK-E00", format!("unknown space `{origin}`")),
            },
            Expr::Der { der } => {
                let (i, b) = match self.binding(der) {
                    Some(x) => x,
                    None => return self.err("MK-E00", format!("unknown binding `{der}`")),
                };
                if b.role != Role::Continuous {
                    return self.err("MK-E06", format!("der({}) of a binding that is not continuous state", b.name));
                }
                let t = derivative_type(&b.ty)?;
                // D-091: in a pointwise expression, `der(T)` of a field of its grid is read in
                // the cell.
                match (&t, &self.pointwise) {
                    (Type::Field { grid, of }, Some(p)) if grid == p => Some((CExpr::Der(i), (**of).clone())),
                    _ => Some((CExpr::Der(i), t)),
                }
            }
            Expr::Neg { neg } => {
                let (c, t) = self.expr(neg, exp)?;
                if matches!(t, Type::Point { .. } | Type::Instant { .. }) {
                    return self.err("MK-E04", "negating a point or instant".into());
                }
                if !matches!(t, Type::Quantity { .. } | Type::Vector { .. }) {
                    return self.err("MK-E01", format!("cannot negate {}", show(&t)));
                }
                Some((CExpr::Neg(Box::new(c)), t))
            }
            Expr::Not { not } => {
                let c = self.expect(not, &Type::Boolean)?;
                Some((CExpr::Not(Box::new(c)), Type::Boolean))
            }
            Expr::Norm { norm } => {
                let (c, t) = self.expr(norm, None)?;
                match t {
                    Type::Quantity { dim } | Type::Vector { dim, .. } => Some((CExpr::Norm(Box::new(c)), Type::Quantity { dim })),
                    other => self.err("MK-E01", format!("no norm of {}", show(&other))),
                }
            }
            Expr::Bin { bin, l, r } => self.binary(*bin, l, r, exp),
            Expr::Call { call, args } => self.call(*call, args),
            Expr::Draw { draw, args } => self.draw(*draw, args, exp),
            // `T(0.5 m)`: a field read at a coordinate (D-091).
            Expr::Apply { apply, args } if self.field_ref(apply).is_some() => {
                let (i, grid, of) = self.field_ref(apply)?;
                if args.len() != 1 {
                    return self.err("MK-E01", "a field is read at one coordinate: `T(0.5 m)`".into());
                }
                let x = self.expect(&args[0], &Type::Quantity { dim: grid.dim })?;
                Some((CExpr::Field(Box::new(self.field_op(i, grid, FieldOp::Sample(x))?)), of))
            }
            Expr::Apply { apply, args } => {
                let (fc, ft) = self.expr(apply, None)?;
                let (params, result) = match ft {
                    Type::Function { params, result } => (params, result),
                    other => return self.err("MK-E01", format!("{} is not a function", show(&other))),
                };
                if params.len() != args.len() {
                    return self.err("MK-E01", format!("expected {} arguments, found {}", params.len(), args.len()));
                }
                let mut cargs = vec![];
                for (a, p) in args.iter().zip(params.iter()) {
                    cargs.push(self.expect(a, p)?);
                }
                Some((CExpr::Apply(Box::new(fc), cargs), *result))
            }
            Expr::If { r#if, then, r#else } => {
                let k = self.expect(r#if, &Type::Boolean)?;
                let (a, b, t) = if exp.is_none() && Self::needs_ctx(then) && !Self::needs_ctx(r#else) {
                    let (b, tb) = self.expr(r#else, None)?;
                    let a = self.expect(then, &tb)?;
                    (a, b, tb)
                } else {
                    let (a, ta) = self.expr(then, exp)?;
                    let b = self.expect(r#else, &ta)?;
                    (a, b, ta)
                };
                Some((CExpr::If(Box::new(k), Box::new(a), Box::new(b)), t))
            }
            Expr::Tuple { tuple } => {
                if let Some(Type::Vector { space, dim }) = exp {
                    if self.space_dim(space) == Some(tuple.len()) {
                        let want = Type::Quantity { dim: *dim };
                        let mut items = vec![];
                        for it in tuple {
                            items.push(self.expect(it, &want)?);
                        }
                        return Some((CExpr::MakeVec(items), exp.unwrap().clone()));
                    }
                }
                let mut items = vec![];
                let mut types = vec![];
                // A tuple expected of a tuple type has items of its item types (D-059).
                let want = match exp {
                    Some(Type::Tuple { items }) if items.len() == tuple.len() => Some(items),
                    _ => None,
                };
                for (k, it) in tuple.iter().enumerate() {
                    let (c, t) = match want {
                        Some(w) => (self.expect(it, &w[k])?, w[k].clone()),
                        None => self.expr(it, None)?,
                    };
                    items.push(c);
                    types.push(t);
                }
                Some((CExpr::MakeTuple(items), Type::Tuple { items: types }))
            }
            Expr::Comp { comp, axis } => {
                let (c, t) = self.expr(comp, None)?;
                let rt = match &t {
                    Type::Vector { space, dim } if Some(*axis) < self.space_dim(space) => Type::Quantity { dim: *dim },
                    Type::Point { space } if Some(*axis) < self.space_dim(space) => Type::Quantity { dim: Dim::length() },
                    Type::Tuple { items } if *axis < items.len() => items[*axis].clone(),
                    other => return self.err("MK-E01", format!("no component {axis} of {}", show(other))),
                };
                Some((CExpr::Comp(Box::new(c), *axis), rt))
            }
            Expr::Lambda { lambda } => {
                self.lambda.push(lambda.params.clone());
                let want = match exp {
                    Some(Type::Function { result, .. }) => Some((**result).clone()),
                    _ => None,
                };
                let body = match &want {
                    Some(w) => self.expect(&lambda.body, w).map(|c| (c, w.clone())),
                    None => self.expr(&lambda.body, None),
                };
                self.lambda.pop();
                let (bc, bt) = body?;
                Some((CExpr::Lambda(Rc::new(bc)), Type::Function { params: lambda.params.clone(), result: Box::new(bt) }))
            }
            Expr::Otherwise { otherwise, default } => {
                let (a, t) = self.expr(otherwise, exp)?;
                let b = self.expect(default, &t)?;
                Some((CExpr::Otherwise(Box::new(a), Box::new(b)), t))
            }
        }
    }

    /// `match e { case => value, ... }` (MK-10.2, D-049): the scrutinee is an enumeration and
    /// every case has exactly one arm (MK-E17); the arms have one type.
    fn match_expr(&mut self, scrutinee: &Expr, arms: &[Arm], exp: Option<&Type>) -> Typed {
        let (sc, st) = self.expr(scrutinee, None)?;
        let cases = match &st {
            Type::Enum { cases, .. } => cases.clone(),
            other => return self.err("MK-E01", format!("`match` needs an enumeration, found {}", show(other))),
        };
        for a in arms {
            if !cases.contains(&a.case) {
                return self.err("MK-E01", format!("`{}` is not a case of {}", a.case, show(&st)));
            }
            if arms.iter().filter(|b| b.case == a.case).count() > 1 {
                return self.err("MK-E17", format!("case `{}` has more than one arm", a.case));
            }
        }
        let missing: Vec<&String> = cases.iter().filter(|c| !arms.iter().any(|a| &&a.case == c)).collect();
        if !missing.is_empty() {
            let list = missing.iter().map(|c| format!("`{c}`")).collect::<Vec<_>>().join(", ");
            return self.err("MK-E17", format!("`match` does not cover {list} (MK-10.2)"));
        }
        // The first arm that fixes a type gives it to the others.
        let lead = arms.iter().position(|a| !Self::needs_ctx(&a.value)).unwrap_or(0);
        let (_, t) = self.expr(&arms[lead].value, exp)?;
        let mut by_case: Vec<Option<CExpr>> = vec![None; cases.len()];
        for a in arms {
            let c = self.expect(&a.value, &t)?;
            by_case[cases.iter().position(|x| x == &a.case).unwrap()] = Some(c);
        }
        // A case left without an arm is a duplicated case, reported with its enumeration.
        let arms = by_case.into_iter().collect::<Option<Vec<_>>>()?;
        Some((CExpr::Match(Box::new(sc), arms), t))
    }

    fn num(&mut self, v: f64, unit: Option<&Unit>, exp: Option<&Type>) -> Typed {
        if let Some(u) = unit {
            let si = v * u.scale + u.offset;
            return Some((CExpr::Const(Value::Num(si)), Type::Quantity { dim: u.dim }));
        }
        if v == 0.0 {
            // MK-3.8 and D-030: `0` adopts the required dimension, or the zero vector.
            return match exp {
                Some(Type::Quantity { dim }) => Some((CExpr::Const(Value::Num(0.0)), Type::Quantity { dim: *dim })),
                Some(t @ Type::Vector { space, .. }) => {
                    let n = self.space_dim(space).unwrap_or(2);
                    Some((CExpr::Const(Value::Vec(Arr::zeros(n))), t.clone()))
                }
                _ => Some((CExpr::Const(Value::Num(0.0)), Type::real())),
            };
        }
        if let Some(t) = exp {
            if is_dimensioned(t) {
                return self.err("MK-E03", format!("bare literal {v} where {} is required", show(t)));
            }
        }
        Some((CExpr::Const(Value::Num(v)), Type::real()))
    }

    /// Checks two operands, the context-free one first, passing its type to the other.
    fn pair(
        &mut self,
        l: &Expr,
        r: &Expr,
        exp_l: Option<&Type>,
        other: impl Fn(&Type) -> Option<Type>,
    ) -> Option<((CExpr, Type), (CExpr, Type))> {
        if Self::needs_ctx(l) && !Self::needs_ctx(r) {
            let rt = self.expr(r, None)?;
            let want = other(&rt.1).or_else(|| exp_l.cloned());
            let lt = self.expr(l, want.as_ref())?;
            Some((lt, rt))
        } else {
            let lt = self.expr(l, if Self::needs_ctx(l) { exp_l } else { exp_l.filter(|t| !matches!(t, Type::Point { .. })) })?;
            let want = other(&lt.1);
            let rt = self.expr(r, want.as_ref())?;
            Some((lt, rt))
        }
    }

    fn binary(&mut self, op: BinOp, l: &Expr, r: &Expr, exp: Option<&Type>) -> Typed {
        use Type::*;
        let b = |op: BinOp, a: CExpr, c: CExpr| CExpr::Bin(op, Box::new(a), Box::new(c));
        match op {
            BinOp::Add | BinOp::Sub => {
                let ((lc, lt), (rc, rt)) = self.pair(l, r, exp, |t| match t {
                    Point { space } => Some(Vector { space: space.clone(), dim: Dim::length() }),
                    Instant { dim } => Some(Quantity { dim: *dim }),
                    t => Some(t.clone()),
                })?;
                let rt_ = match (op, &lt, &rt) {
                    (_, Quantity { dim: a }, Quantity { dim: c }) if a == c => Quantity { dim: *a },
                    (_, Vector { space: s1, dim: a }, Vector { space: s2, dim: c }) if s1 == s2 && a == c => lt.clone(),
                    (_, Vector { space: s1, .. }, Vector { space: s2, .. }) if s1 != s2 => {
                        return self.err("MK-E05", format!("{} and {} are in different spaces", show(&lt), show(&rt)))
                    }
                    (_, Point { space: s1 }, Vector { space: s2, dim }) if s1 == s2 && *dim == Dim::length() => lt.clone(),
                    (BinOp::Add, Vector { space: s2, dim }, Point { space: s1 }) if s1 == s2 && *dim == Dim::length() => rt.clone(),
                    (BinOp::Sub, Point { space: s1 }, Point { space: s2 }) if s1 == s2 => Vector { space: s1.clone(), dim: Dim::length() },
                    (_, Point { space: s1 }, Point { space: s2 } | Vector { space: s2, .. }) if s1 != s2 => {
                        return self.err("MK-E05", format!("{} and {} are in different spaces", show(&lt), show(&rt)))
                    }
                    (BinOp::Add, Point { .. }, Point { .. }) => return self.err("MK-E04", "adding two points".into()),
                    (_, Point { .. }, Vector { .. }) | (_, Vector { .. }, Point { .. }) => {
                        return self.err("MK-E04", format!("point combined with {}, which is not a displacement", show(&rt)))
                    }
                    (_, Instant { dim: a }, Quantity { dim: c }) if a == c => lt.clone(),
                    (BinOp::Sub, Instant { dim: a }, Instant { dim: c }) if a == c => Quantity { dim: *a },
                    (BinOp::Add, Instant { .. }, Instant { .. }) => return self.err("MK-E04", "adding two instants".into()),
                    (_, Tuple { .. }, Vector { .. } | Point { .. }) | (_, Vector { .. } | Point { .. }, Tuple { .. }) => {
                        return self.err("MK-E21", "a tuple combined with a vector without an expected vector type".into())
                    }
                    _ => return self.err("MK-E01", format!("cannot {} {} and {}", if op == BinOp::Add { "add" } else { "subtract" }, show(&lt), show(&rt))),
                };
                Some((b(op, lc, rc), rt_))
            }
            BinOp::Mul | BinOp::Div => {
                // Only a vector-valued side takes an expected type (a tuple scaled by a quantity).
                let vec_exp = |scalar: &Type| -> Option<Type> {
                    match (exp, scalar) {
                        (Some(Vector { space, dim }), Quantity { dim: q }) => Some(Vector {
                            space: space.clone(),
                            dim: if op == BinOp::Mul { dim.div(q) } else { dim.mul(q) },
                        }),
                        _ => None,
                    }
                };
                let (lt_, rt_) = if matches!(l, Expr::Tuple { .. }) || (Self::needs_ctx(l) && !Self::needs_ctx(r) && !matches!(l, Expr::Num { .. })) {
                    let rt = self.expr(r, None)?;
                    let lt = self.expr(l, vec_exp(&rt.1).as_ref())?;
                    (lt, rt)
                } else {
                    let lt = self.expr(l, None)?;
                    let want = if matches!(r, Expr::Tuple { .. } | Expr::If { .. }) && op == BinOp::Mul { vec_exp(&lt.1) } else { None };
                    let rt = self.expr(r, want.as_ref())?;
                    (lt, rt)
                };
                let ((lc, lt), (rc, rt)) = (lt_, rt_);
                let t = match (op, &lt, &rt) {
                    (BinOp::Mul, Quantity { dim: a }, Quantity { dim: c }) => Quantity { dim: a.mul(c) },
                    (BinOp::Div, Quantity { dim: a }, Quantity { dim: c }) => Quantity { dim: a.div(c) },
                    (BinOp::Mul, Quantity { dim: a }, Vector { space, dim }) | (BinOp::Mul, Vector { space, dim }, Quantity { dim: a }) => {
                        Vector { space: space.clone(), dim: a.mul(dim) }
                    }
                    (BinOp::Div, Vector { space, dim }, Quantity { dim: a }) => Vector { space: space.clone(), dim: dim.div(a) },
                    (_, Point { .. }, _) | (_, _, Point { .. }) => return self.err("MK-E04", "scaling a point".into()),
                    (_, Instant { .. }, _) | (_, _, Instant { .. }) => return self.err("MK-E04", "scaling an instant".into()),
                    _ => return self.err("MK-E01", format!("cannot multiply or divide {} and {}", show(&lt), show(&rt))),
                };
                Some((b(op, lc, rc), t))
            }
            BinOp::Pow => {
                let (lc, lt) = self.expr(l, None)?;
                let dim = match &lt {
                    Quantity { dim } => *dim,
                    other => return self.err("MK-E01", format!("cannot raise {} to a power", show(other))),
                };
                if dim.is_none() {
                    let rc = self.expect(r, &Type::real())?;
                    return Some((b(op, lc, rc), Type::real()));
                }
                let p = match const_number(r) {
                    Some(p) => p,
                    None => return self.err("MK-E01", "the exponent of a dimensioned quantity must be a constant".into()),
                };
                let ratio = to_ratio(p);
                let ratio = match ratio {
                    Some(q) => q,
                    None => return self.err("MK-E01", format!("exponent {p} is not a simple fraction")),
                };
                Some((CExpr::Powi(Box::new(lc), p), Quantity { dim: dim.pow(ratio) }))
            }
            BinOp::And | BinOp::Or => {
                let lc = self.expect(l, &Type::Boolean)?;
                let rc = self.expect(r, &Type::Boolean)?;
                Some((b(op, lc, rc), Type::Boolean))
            }
            BinOp::Eq | BinOp::Ne | BinOp::Lt | BinOp::Le | BinOp::Gt | BinOp::Ge => {
                let ((lc, lt), (rc, rt)) = self.pair(l, r, None, |t| Some(t.clone()))?;
                let ordered = !matches!(op, BinOp::Eq | BinOp::Ne);
                let ok = match (&lt, &rt) {
                    (Quantity { dim: a }, Quantity { dim: c }) => a == c,
                    (Instant { dim: a }, Instant { dim: c }) => a == c,
                    (Instant { .. }, Quantity { .. }) | (Quantity { .. }, Instant { .. }) => {
                        return self.err("MK-E04", "comparing an instant with a quantity".into())
                    }
                    (a, c) if !ordered => same_type(a, c),
                    _ => false,
                };
                if !ok {
                    return self.err("MK-E01", format!("cannot compare {} and {}", show(&lt), show(&rt)));
                }
                Some((b(op, lc, rc), Type::Boolean))
            }
        }
    }

    fn call(&mut self, f: Func, args: &[Expr]) -> Typed {
        // D-091: operations on a field take its name.
        let on_field = matches!(f, Func::Laplacian | Func::Gradient | Func::Integral | Func::Mean) || (matches!(f, Func::Min | Func::Max) && args.len() == 1);
        if on_field {
            let Some((i, grid, of)) = args.first().filter(|_| args.len() == 1).and_then(|a| self.field_ref(a)) else {
                return self.err("MK-E01", format!("{} takes the name of a field: `{}(T)`", func_word(f), func_word(f)));
            };
            let Type::Quantity { dim } = of else { return self.err("MK-E01", "fields hold numbers and quantities".into()) };
            let (op, ty) = match f {
                Func::Laplacian => (FieldOp::Laplacian, dim.div(&grid.dim.pow(Ratio::int(2)))),
                Func::Gradient => (FieldOp::Gradient, dim.div(&grid.dim)),
                Func::Integral => (FieldOp::Integral, dim.mul(&grid.dim)),
                Func::Mean => (FieldOp::Mean, dim),
                Func::Min => (FieldOp::Min, dim),
                _ => (FieldOp::Max, dim),
            };
            let local = matches!(op, FieldOp::Laplacian | FieldOp::Gradient);
            let here = self.scope.model.bindings[i].ty.clone();
            if local && !matches!((&here, &self.pointwise), (Type::Field { grid: g, .. }, Some(p)) if g == p) {
                return self.err("MK-E32", format!("`{}` reads a field's neighbours, so it is written where the field is: in a flow or definition of a field of its grid (MK-9.7)", func_word(f)));
            }
            return Some((CExpr::Field(Box::new(self.field_op(i, grid, op)?)), Type::Quantity { dim: ty }));
        }
        let n = if matches!(f, Func::Min | Func::Max | Func::Atan2) { 2 } else { 1 };
        if args.len() != n {
            return self.err("MK-E01", format!("{f:?} takes {n} arguments"));
        }
        let (a, at) = self.expr(&args[0], None)?;
        let dim = match at {
            Type::Quantity { dim } => dim,
            other => return self.err("MK-E01", format!("{f:?} of {}", show(&other))),
        };
        match f {
            Func::Sin | Func::Cos | Func::Tan | Func::Exp | Func::Log => {
                if !dim.is_none() {
                    return self.err("MK-E02", format!("{f:?} of a quantity with dimension {dim}"));
                }
                Some((CExpr::Call(f, vec![a]), Type::real()))
            }
            Func::Sqrt => Some((CExpr::Call(f, vec![a]), Type::Quantity { dim: dim.pow(Ratio::new(1, 2)) })),
            Func::Abs => Some((CExpr::Call(f, vec![a]), Type::Quantity { dim })),
            Func::Min | Func::Max | Func::Atan2 => {
                let bc = self.expect(&args[1], &Type::Quantity { dim })?;
                let rt = if f == Func::Atan2 { Type::real() } else { Type::Quantity { dim } };
                Some((CExpr::Call(f, vec![a, bc]), rt))
            }
            Func::Laplacian | Func::Gradient | Func::Integral | Func::Mean => unreachable!(),
        }
    }

    /// The field an expression names, with its grid and the type of its values (D-091).
    fn field_ref(&self, e: &Expr) -> Option<(usize, CGrid, Type)> {
        let Expr::Ref { r#ref } = e else { return None };
        let (i, b) = self.binding(r#ref)?;
        match &b.ty {
            Type::Field { grid, of } => Some((i, *self.scope.grids.get(grid)?, (**of).clone())),
            _ => None,
        }
    }

    /// An operation on field `i`, with the conditions on its edges (MK-9.6): held at a value,
    /// periodic, or insulated where nothing is said. Edge values read no field.
    fn field_op(&mut self, i: usize, grid: CGrid, op: FieldOp) -> Option<CFieldOp> {
        let b = &self.scope.model.bindings[i];
        let of = match &b.ty {
            Type::Field { of, .. } => (**of).clone(),
            _ => return None,
        };
        let saved = self.pointwise.take();
        let mut edges = [CEdge::Insulated, CEdge::Insulated];
        if let Some(bd) = &b.boundary {
            if bd.periodic {
                edges = [CEdge::Periodic, CEdge::Periodic];
            }
            for f in &bd.fixed {
                let side = if f.edge == "left" { 0 } else { 1 };
                if let Some(v) = self.expect(&f.value, &of) {
                    edges[side] = CEdge::Fixed(v);
                }
            }
        }
        self.pointwise = saved;
        Some(CFieldOp { op, binding: i, grid, edges })
    }

    /// A field's initial value, definition or flow, of type `want` in each cell: a function of
    /// the cell's coordinate (`T(x in rod) = ...`, `der(T(x)) = ...`) or an expression of the
    /// fields of its grid (D-091).
    fn pointwise_expr(&mut self, e: &Expr, grid: &Id, want: &Type) -> Option<CExpr> {
        let g = *self.scope.grids.get(grid)?;
        let saved = self.pointwise.replace(grid.clone());
        let c = match e {
            Expr::Lambda { lambda } if lambda.params.len() == 1 => {
                self.lambda.push(vec![Type::Quantity { dim: g.dim }]);
                let c = self.expect(&lambda.body, want);
                self.lambda.pop();
                c
            }
            _ => self.expect(e, want),
        };
        self.pointwise = saved;
        c
    }
}

fn func_word(f: Func) -> &'static str {
    match f {
        Func::Laplacian => "laplacian",
        Func::Gradient => "gradient",
        Func::Integral => "integral",
        Func::Mean => "mean",
        Func::Min => "min",
        Func::Max => "max",
        _ => "function",
    }
}

impl Tc<'_, '_> {
    /// A draw from a distribution (MK-10.4, D-083): only in an event's condition and
    /// operations (MK-E28). Counts (`integer`, `poisson`, `binomial`) are whole numbers of
    /// type `Real`.
    fn draw(&mut self, d: Dist, args: &[Expr], exp: Option<&Type>) -> Typed {
        let Some((event, stream, _)) = self.draws else {
            return self.err("MK-E28", format!("`{}(...)` draws at random, so it is written only in an event's condition or operations (MK-10.4)", d.name()));
        };
        let arity = match d {
            Dist::Exponential | Dist::Bernoulli | Dist::Poisson => 1,
            Dist::Choice => args.len().max(1),
            _ => 2,
        };
        if args.len() != arity {
            return self.err("MK-E01", format!("`{}` takes {arity} argument{}", d.name(), if arity == 1 { "" } else { "s" }));
        }
        let real = Type::real();
        let (cargs, ty) = match d {
            Dist::Uniform => {
                let (a, at) = self.expr(&args[0], exp)?;
                if !matches!(at, Type::Quantity { .. }) {
                    return self.err("MK-E01", format!("`uniform` draws a quantity, not {}", show(&at)));
                }
                let b = self.expect(&args[1], &at)?;
                (vec![a, b], at)
            }
            Dist::Normal => {
                let (m, mt) = self.expr(&args[0], exp)?;
                let dim = match &mt {
                    Type::Quantity { dim } | Type::Vector { dim, .. } => *dim,
                    Type::Point { .. } => Dim::length(),
                    other => return self.err("MK-E01", format!("`normal` draws a quantity, vector or point, not {}", show(other))),
                };
                let s = self.expect(&args[1], &Type::Quantity { dim })?;
                (vec![m, s], mt)
            }
            Dist::Exponential => {
                let (r, rt) = self.expr(&args[0], None)?;
                let Type::Quantity { dim } = rt else {
                    return self.err("MK-E01", format!("`exponential` takes a rate, not {}", show(&rt)));
                };
                (vec![r], Type::Quantity { dim: dim.pow(Ratio::new(-1, 1)) })
            }
            Dist::Bernoulli => (vec![self.expect(&args[0], &real)?], Type::Boolean),
            Dist::Poisson => (vec![self.expect(&args[0], &real)?], real.clone()),
            Dist::Integer | Dist::Binomial => {
                let a = self.expect(&args[0], &real)?;
                let b = self.expect(&args[1], &real)?;
                (vec![a, b], real.clone())
            }
            Dist::Choice => {
                let (a, at) = self.expr(&args[0], exp)?;
                let mut v = vec![a];
                for x in &args[1..] {
                    v.push(self.expect(x, &at)?);
                }
                (v, at)
            }
        };
        let site = self.draws.as_ref().map_or(0, |d| d.2);
        if let Some(d) = self.draws.as_mut() {
            d.2 += 1;
        }
        Some((CExpr::Draw(Box::new(CDraw { dist: d, args: cargs, event, stream, site })), ty))
    }
}

/// The type of `der(x)` for a continuous state of type `t` (MK-14.5).
fn derivative_type(t: &Type) -> Option<Type> {
    let inv_t = Dim::time().inv();
    Some(match t {
        Type::Quantity { dim } => Type::Quantity { dim: dim.mul(&inv_t) },
        Type::Vector { space, dim } => Type::Vector { space: space.clone(), dim: dim.mul(&inv_t) },
        Type::Point { space } => Type::Vector { space: space.clone(), dim: Dim::length().mul(&inv_t) },
        Type::Field { grid, of } => Type::Field { grid: grid.clone(), of: Box::new(derivative_type(of)?) },
        _ => return None,
    })
}

fn const_number(e: &Expr) -> Option<f64> {
    match e {
        Expr::Num { num, unit: None } => Some(*num),
        Expr::Const { r#const } => Some(r#const.value()),
        Expr::Neg { neg } => const_number(neg).map(|x| -x),
        Expr::Bin { bin: BinOp::Div, l, r } => Some(const_number(l)? / const_number(r)?),
        _ => None,
    }
}

fn to_ratio(p: f64) -> Option<Ratio> {
    for d in 1..=12 {
        let n = p * d as f64;
        if (n - n.round()).abs() < 1e-12 {
            return Some(Ratio::new(n.round() as i32, d));
        }
    }
    None
}

pub fn show(t: &Type) -> String {
    match t {
        Type::Boolean => "Boolean".into(),
        Type::Integer => "Integer".into(),
        Type::Quantity { dim } if dim.is_none() => "Real".into(),
        Type::Quantity { dim } => format!("Quantity<{dim}>"),
        Type::Instant { dim } => format!("Instant<{dim}>"),
        Type::Point { space } => format!("Point<{space}>"),
        Type::Vector { space, dim } => format!("Vector<{space}, {dim}>"),
        Type::Tuple { items } => format!("Tuple<{}>", items.iter().map(show).collect::<Vec<_>>().join(", ")),
        Type::Enum { r#enum, .. } => r#enum.rsplit('.').next().unwrap_or(r#enum).to_string(),
        Type::Function { params, result } => format!("({}) -> {}", params.iter().map(show).collect::<Vec<_>>().join(", "), show(result)),
        Type::Field { grid, of } => format!("Field<{}, {}>", grid.rsplit('.').next().unwrap_or(grid), show(of)),
    }
}

fn role_word(r: Role) -> &'static str {
    match r {
        Role::Constant => "constant",
        Role::Parameter => "parameter",
        Role::Input => "input",
        Role::Discrete => "discrete state",
        Role::Continuous => "state",
        Role::Derived => "derived binding",
    }
}

/// Checks and compiles the declared enumerations and functions of a model (D-048, D-049).
/// Functions are compiled callees first; a function that calls itself, directly or
/// through others, is MK-E23.
fn check_functions(spaces: &[Space], model: &Model, index: &HashMap<Id, usize>, diags: &mut Vec<Diagnostic>) -> HashMap<Id, (Rc<CExpr>, Type)> {
    for e in &model.enums {
        if e.cases.is_empty() {
            diags.push(Diagnostic { code: "MK-E01", message: format!("enumeration `{}` has no cases", e.name), element: e.id.clone() });
        }
        for (i, c) in e.cases.iter().enumerate() {
            if e.cases[..i].contains(c) {
                diags.push(Diagnostic { code: "MK-E22", message: format!("case `{c}` of `{}` declared twice", e.name), element: e.id.clone() });
            }
        }
    }
    let calls = |f: &FunctionDecl| -> Vec<Id> {
        let mut out = vec![];
        f.body.walk(&mut |e| {
            if let Expr::Fn { r#fn } = e {
                if !out.contains(r#fn) {
                    out.push(r#fn.clone());
                }
            }
        });
        out
    };
    // Callees first (depth-first order); a function met again on the current path recurses.
    fn visit(i: usize, fs: &[FunctionDecl], calls: &dyn Fn(&FunctionDecl) -> Vec<Id>, state: &mut [u8], order: &mut Vec<usize>, rec: &mut Vec<usize>) {
        if state[i] == 2 {
            return;
        }
        if state[i] == 1 {
            if !rec.contains(&i) {
                rec.push(i);
            }
            return;
        }
        state[i] = 1;
        for c in calls(&fs[i]) {
            if let Some(j) = fs.iter().position(|f| f.id == c) {
                visit(j, fs, calls, state, order, rec);
            }
        }
        state[i] = 2;
        order.push(i);
    }
    let fs = &model.functions;
    let (mut state, mut order, mut rec) = (vec![0u8; fs.len()], vec![], vec![]);
    for i in 0..fs.len() {
        visit(i, fs, &calls, &mut state, &mut order, &mut rec);
    }
    for &i in &rec {
        diags.push(Diagnostic { code: "MK-E23", message: format!("declared function `{}` calls itself (MK-10.3)", fs[i].name), element: fs[i].id.clone() });
    }
    let mut funcs: HashMap<Id, (Rc<CExpr>, Type)> = HashMap::new();
    for i in order {
        let f = &fs[i];
        if fs[..i].iter().any(|g| g.name == f.name) || model.bindings.iter().any(|b| b.name == f.name) {
            diags.push(Diagnostic { code: "MK-E22", message: format!("`{}` declared twice", f.name), element: f.id.clone() });
        }
        if rec.contains(&i) || calls(f).iter().any(|c| fs.iter().position(|g| &g.id == c).is_some_and(|j| rec.contains(&j))) {
            continue;
        }
        let snapshot = funcs.clone();
        let no_grids = HashMap::new();
        let scope = Scope { spaces, model, index, funcs: &snapshot, grids: &no_grids };
        let mut tc = Tc::new(&scope, diags, &f.id);
        tc.closed = Some(f.name.clone());
        tc.lambda.push(f.params.iter().map(|p| p.ty.clone()).collect());
        if let Some(body) = tc.expect(&f.body, &f.result) {
            funcs.insert(f.id.clone(), (Rc::new(body), f.ty()));
        }
    }
    funcs
}

// ------------------------------------------------------------------ model checking

/// Checks a model and compiles it (MK section 18). Returns every diagnostic found.
pub fn check_model(spaces: &[Space], model: &Model) -> Result<CModel, Vec<Diagnostic>> {
    let mut diags = vec![];
    let mut index = HashMap::new();
    for (i, b) in model.bindings.iter().enumerate() {
        if index.insert(b.id.clone(), i).is_some() {
            diags.push(Diagnostic { code: "MK-E22", message: format!("binding `{}` defined twice", b.name), element: b.id.clone() });
        }
    }
    let funcs = check_functions(spaces, model, &index, &mut diags);
    let grids = check_grids(spaces, model, &index, &funcs, &mut diags);
    let scope = Scope { spaces, model, index: &index, funcs: &funcs, grids: &grids };
    check_boundaries(model, &mut diags);

    // Bindings: initial definitions and definitions.
    let mut cbindings = vec![];
    for b in &model.bindings {
        let mut tc = Tc::new(&scope, &mut diags, &b.id);
        // D-091: a field's initial value and definition are evaluated in each cell.
        let field = match &b.ty {
            Type::Field { grid, of } => match grids.get(grid) {
                Some(g) if matches!(**of, Type::Quantity { .. }) && matches!(b.role, Role::Continuous | Role::Derived) => Some((*g, grid.clone(), (**of).clone())),
                Some(_) => {
                    tc.err("MK-E01", format!("`{}` is a field: state or derived, holding numbers or quantities (MK-9.4)", b.name));
                    None
                }
                None => {
                    tc.err("MK-E00", format!("unknown grid `{grid}`"));
                    None
                }
            },
            _ => None,
        };
        let init = match (&b.init, b.role, &field) {
            (Some(e), Role::Continuous, Some((_, grid, of))) => tc.pointwise_expr(e, grid, of),
            (Some(e), r, _) if r.is_stored() => tc.expect(e, &b.ty),
            (None, Role::Input, _) => None,
            (None, r, _) if r.is_stored() => {
                tc.err("MK-E00", format!("stored binding `{}` has no initial definition (MK-6.7)", b.name));
                None
            }
            _ => None,
        };
        let def = match (&b.def, b.role) {
            (Some(e), Role::Derived) if field.is_some() => {
                let (_, grid, of) = field.as_ref().unwrap();
                tc.pointwise_expr(e, grid, of)
            }
            (Some(e), Role::Derived) => tc.expect(e, &b.ty),
            (None, Role::Derived) => {
                tc.err("MK-E00", format!("derived binding `{}` has no definition", b.name));
                None
            }
            _ => None,
        };
        // MK-10.7 (D-029): a stored function value must be closed.
        if b.role.is_stored() && matches!(b.ty, Type::Function { .. }) {
            if let Some(e) = &b.init {
                if !e.refs().is_empty() {
                    tc.err("MK-E20", format!("stored function `{}` reads bindings", b.name));
                }
            }
        }
        // D-058: a derived binding of a member or relation not alive is not evaluated; it
        // holds a value that is not a number.
        let when = b.when.as_ref().and_then(|w| tc.expect(w, &Type::Boolean)).map(|c| (c, tc.placeholder(&b.ty)));
        cbindings.push(CBinding {
            id: b.id.clone(),
            name: b.name.clone(),
            role: b.role,
            ty: b.ty.clone(),
            init,
            def,
            intervenable: b.is_intervenable(),
            when,
            field: field.map(|f| f.0),
        });
    }

    // Continuous state layout.
    let mut cont = vec![];
    let mut offset = 0;
    for (i, b) in model.bindings.iter().enumerate() {
        if b.role != Role::Continuous {
            continue;
        }
        let len = match &b.ty {
            Type::Quantity { .. } => 1,
            Type::Field { grid, .. } => grids.get(grid).map(|g| g.n).unwrap_or(0),
            Type::Vector { space, .. } | Type::Point { space } => spaces.iter().find(|s| &s.id == space).map(|s| s.dimension as usize).unwrap_or(0),
            _ => {
                diags.push(Diagnostic { code: "MK-E01", message: format!("continuous state `{}` must be a quantity, vector or point", b.name), element: b.id.clone() });
                0
            }
        };
        cont.push(ContState { binding: i, offset, len, point: matches!(b.ty, Type::Point { .. }), flows: FlowSet::None });
        offset += len;
    }

    // Flows (MK section 14). The stream key of each white noise, in order (D-090).
    let mut noise_keys: Vec<u64> = vec![];
    for f in &model.flows {
        let mut tc = Tc::new(&scope, &mut diags, &f.id);
        let bi = match index.get(&f.target) {
            Some(&i) => i,
            None => {
                tc.err("MK-E00", format!("flow target `{}` is unknown", f.target));
                continue;
            }
        };
        let b = &model.bindings[bi];
        if b.role != Role::Continuous {
            tc.err("MK-E06", format!("flow targets `{}`, which is not continuous state", b.name));
            continue;
        }
        let want = match derivative_type(&b.ty) {
            Some(t) => t,
            None => continue,
        };
        tc.noise = Some((noise_keys.len(), 0));
        let compiled = match &want {
            // D-091: a field's flow is evaluated in each cell.
            Type::Field { grid, of } => {
                let c = tc.pointwise_expr(&f.expr, grid, of);
                if tc.noise.is_some_and(|n| n.1 > 0) {
                    tc.err("MK-E29", "`noise` in the flow of a field (independent noise in every cell) is a later extension (MK-14.16)".into());
                }
                c
            }
            _ => tc.expect(&f.expr, &want),
        };
        let n_noise = tc.noise.map(|n| n.1).unwrap_or(0);
        noise_keys.extend((0..n_noise).map(|k| fnv1a(&format!("{}|noise|{k}", f.id))));
        if n_noise > 0 {
            if let Err(m) = noise_linear(&f.expr) {
                diags.push(Diagnostic { code: "MK-E30", message: format!("{m}: a flow is linear in each noise, `der(x) = a + b * noise` with `a` and `b` free of noise (MK-14.16)"), element: f.id.clone() });
            }
        }
        flow_conditions(&scope, &mut diags, f);
        let Some(c) = compiled else { continue };
        let cs = cont.iter_mut().find(|c| c.binding == bi).unwrap();
        cs.flows = match (std::mem::replace(&mut cs.flows, FlowSet::None), f.kind) {
            (FlowSet::None, FlowKind::Define) => FlowSet::Define(c),
            (FlowSet::None, FlowKind::Contribute) => FlowSet::Sum(vec![c]),
            (FlowSet::Sum(mut v), FlowKind::Contribute) => {
                v.push(c);
                FlowSet::Sum(v)
            }
            (old, kind) => {
                let (code, msg) = match (&old, kind) {
                    (FlowSet::Define(_), FlowKind::Define) => ("MK-E22", format!("der({}) defined by two flows", b.name)),
                    _ => ("MK-E10", format!("der({}) is both defined and contributed to", b.name)),
                };
                diags.push(Diagnostic { code, message: msg, element: f.id.clone() });
                old
            }
        };
    }

    // Events (MK sections 15, 16).
    let mut cevents = vec![];
    for e in &model.events {
        let mut tc = Tc::new(&scope, &mut diags, &e.id);
        let trigger = match &e.trigger {
            Trigger::Rising { guard } | Trigger::Falling { guard } | Trigger::Crossing { guard } => {
                let dir = match &e.trigger {
                    Trigger::Rising { .. } => Dir::Rising,
                    Trigger::Falling { .. } => Dir::Falling,
                    _ => Dir::Either,
                };
                match tc.expr(guard, None) {
                    Some((c, Type::Quantity { .. })) => Some(CTrigger::Crossing { dir, guard: c }),
                    Some((_, t)) => {
                        tc.err("MK-E01", format!("a crossing guard must be a real quantity, found {}", show(&t)));
                        None
                    }
                    None => None,
                }
            }
            Trigger::Level { .. } => {
                tc.err("MK-E13", "a level condition used as a continuous trigger; use a crossing (MK-15.4)".into());
                None
            }
            Trigger::At { time } => tc.expect(time, &Type::Instant { dim: Dim::time() }).map(CTrigger::At),
            Trigger::Every { period, from } => {
                let p = tc.expect(period, &Type::Quantity { dim: Dim::time() });
                let f = tc.expect(from, &Type::Instant { dim: Dim::time() });
                match (p, f) {
                    (Some(p), Some(f)) => Some(CTrigger::Every { period: p, from: f }),
                    _ => None,
                }
            }
            Trigger::On { event } => match model.events.iter().position(|x| &x.id == event) {
                Some(i) => Some(CTrigger::On(i)),
                None => {
                    tc.err("MK-E00", format!("unknown event `{event}`"));
                    None
                }
            },
            Trigger::Start => Some(CTrigger::Start),
            Trigger::Request => Some(CTrigger::Request),
            Trigger::Random { rate } => {
                let r = tc.expect(rate, &Type::Quantity { dim: Dim::time().pow(Ratio::new(-1, 1)) });
                let varying = random_rate_varies(&scope, rate);
                r.map(|rate| CTrigger::Random { rate, varying })
            }
            Trigger::Input { binding } => index.get(binding).map(|&i| CTrigger::Input(i)),
        };
        // D-050: an event with a payload gets it from a request, or from the event it
        // follows, which carries a payload of the same type.
        if let Some(p) = &e.payload {
            let source_ok = match &e.trigger {
                Trigger::Request => true,
                Trigger::On { event } => model.events.iter().find(|x| &x.id == event).and_then(|x| x.payload.as_ref()).is_some_and(|q| q.same_kind(p)),
                _ => false,
            };
            if !source_ok {
                tc.err("MK-E24", format!("the payload `{}` of `{}` has no source: it comes from a request (`on request`) or from an event that carries a payload of the same type (`on E`)", p.name, e.name));
            }
        }
        tc.payload = e.payload.as_ref().map(|p| (e.id.clone(), cevents.len(), p.ty.clone()));
        tc.draws = Some((cevents.len(), crate::random::fnv1a(&e.id), 0));
        let enable = e.enable.as_ref().and_then(|x| tc.expect(x, &Type::Boolean));
        if e.each.is_some() {
            tc.err("MK-E00", format!("event `{}` is repeated per member in a model that was not elaborated (D-057)", e.name));
        }
        let handler = check_ops(&mut tc, model, &index, &e.handler, false);
        let zeno = e.zeno.as_ref().map(|z| CZeno {
                settle: match &z.policy {
                    ZenoPolicy::Stop => None,
                    ZenoPolicy::Settle { ops } => Some(check_ops(&mut tc, model, &index, ops, false)),
                },
                eps: z.eps,
                n: z.n,
                window: z.window,
            });
        cevents.push(CEvent { id: e.id.clone(), name: e.name.clone(), trigger, enable, handler, zeno });
    }
    // MK-15.11, MK-15.12 with D-038: self-retriggering crossing events need a Zeno policy.
    for e in &model.events {
        let guard = match &e.trigger {
            Trigger::Rising { guard } | Trigger::Falling { guard } | Trigger::Crossing { guard } => guard,
            _ => continue,
        };
        if e.zeno.is_some() || e.handler.is_empty() || disables_itself(e) {
            continue;
        }
        if self_retriggering(model, &index, guard, &e.handler) {
            diags.push(Diagnostic {
                code: "MK-E16",
                message: format!("event `{}` writes state its guard depends on and declares no Zeno policy", e.name),
                element: e.id.clone(),
            });
        }
    }

    // Equations (MK section 11).
    let mut cequations = vec![];
    for q in &model.equations {
        let mut tc = Tc::new(&scope, &mut diags, &q.id);
        // D-091: an equation that reads a field holds in every cell of its grid.
        let mut field_grid = None;
        for side in [&q.lhs, &q.rhs] {
            side.walk(&mut |x| {
                let id = match x {
                    Expr::Ref { r#ref } => r#ref,
                    Expr::Der { der } => der,
                    _ => return,
                };
                if let Some(Type::Field { grid, .. }) = index.get(id).map(|&i| &model.bindings[i].ty) {
                    field_grid.get_or_insert(grid.clone());
                }
            });
        }
        tc.pointwise = field_grid.clone();
        let Some((l, lt)) = tc.expr(&q.lhs, None) else { continue };
        let Some(r) = tc.expect(&q.rhs, &lt) else { continue };
        let tol = match (&q.role, &q.tol) {
            (EquationRole::Check, Some(t)) => {
                let want = match &lt {
                    Type::Vector { dim, .. } => Type::Quantity { dim: *dim },
                    other => other.clone(),
                };
                tc.expect(t, &want).and_then(|c| const_value(&c))
            }
            _ => None,
        };
        let field = field_grid.and_then(|g| grids.get(&g).copied());
        cequations.push(CEquation { id: q.id.clone(), name: q.name.clone(), lhs: l, rhs: r, check_tol: tol, check: q.role == EquationRole::Check, field });
    }

    // Constraints (MK section 12).
    let mut cconstraints = vec![];
    for k in &model.constraints {
        let mut tc = Tc::new(&scope, &mut diags, &k.id);
        let (cond, parts) = match (&k.tol, &k.cond) {
            (Some(tol), Expr::Bin { bin: BinOp::Eq, l, r }) => {
                let Some((lc, lt)) = tc.expr(l, None) else { continue };
                let Some(rc) = tc.expect(r, &lt) else { continue };
                let want = match &lt {
                    Type::Vector { dim, .. } => Type::Quantity { dim: *dim },
                    other => other.clone(),
                };
                let Some(t) = tc.expect(tol, &want).and_then(|c| const_value(&c)) else { continue };
                (None, Some((lc, rc, t)))
            }
            (Some(_), _) => {
                tc.err("MK-E01", "a tolerance applies only to an equality constraint".into());
                continue;
            }
            (None, c) => (tc.expect(c, &Type::Boolean), None),
        };
        if cond.is_none() && parts.is_none() {
            continue;
        }
        cconstraints.push(CConstraint { id: k.id.clone(), name: k.name.clone(), cond, within: parts, policy: k.policy });
    }

    // Dependency analysis (MK section 13).
    let derived_order = order_derived(model, &index, &mut diags);
    let init_order = order_init(model, &index, &mut diags);

    if !diags.is_empty() {
        return Err(diags);
    }
    let is_static = model.flows.is_empty() && model.events.iter().all(|e| matches!(e.trigger, Trigger::Request));
    Ok(CModel {
        ir: model.clone(),
        spaces: spaces.to_vec(),
        index,
        bindings: cbindings,
        derived_order,
        init_order,
        cont,
        events: cevents,
        equations: cequations,
        constraints: cconstraints,
        functions: funcs,
        is_static,
        noise: noise_keys,
        grids,
    })
}

/// MK-9.5 (D-091): grids, each one axis (an interval of a quantity, cut into equal cells)
/// whose bounds and number of cells are constant numbers.
fn check_grids(spaces: &[Space], model: &Model, index: &HashMap<Id, usize>, funcs: &HashMap<Id, (Rc<CExpr>, Type)>, diags: &mut Vec<Diagnostic>) -> HashMap<Id, CGrid> {
    let mut out = HashMap::new();
    let no_grids = HashMap::new();
    let scope = Scope { spaces, model, index, funcs, grids: &no_grids };
    for g in &model.grids {
        let mut tc = Tc::new(&scope, diags, &g.id);
        if g.axes.len() != 1 {
            tc.err("MK-E01", format!("grid `{}` has {} axes; grids have one axis in this version (MK-9.5)", g.name, g.axes.len()));
            continue;
        }
        let a = &g.axes[0];
        let constant = |tc: &mut Tc, e: &Expr, want: Option<&Type>| -> Option<(f64, Type)> {
            let (c, t) = match want {
                Some(w) => (tc.expect(e, w)?, w.clone()),
                None => tc.expr(e, None)?,
            };
            let vals = vec![Value::Num(f64::NAN); model.bindings.len()];
            let ctx = crate::eval::Ctx { vals: &vals, der: None, t: 0.0, t0: 0.0, args: &[], payloads: &[], draws: None, noise: &[], point: None };
            match c.eval(&ctx) {
                Ok(Value::Num(x)) if x.is_finite() => Some((x, t)),
                _ => {
                    tc.err("MK-E01", format!("the bounds and cells of grid `{}` are numbers, not values of the model", g.name));
                    None
                }
            }
        };
        let Some((lo, lt)) = constant(&mut tc, &a.lo, None) else { continue };
        let Type::Quantity { dim } = lt else {
            tc.err("MK-E01", format!("grid `{}` spans an interval of a quantity", g.name));
            continue;
        };
        let Some((hi, _)) = constant(&mut tc, &a.hi, Some(&Type::Quantity { dim })) else { continue };
        let Some((n, _)) = constant(&mut tc, &a.cells, Some(&Type::real())) else { continue };
        if !(hi > lo) || n.fract() != 0.0 || !(1.0..=1_000_000.0).contains(&n) {
            tc.err("MK-E01", format!("grid `{}` goes from low to high in a whole number of cells, from 1 to 1000000", g.name));
            continue;
        }
        let n = n as usize;
        out.insert(g.id.clone(), CGrid { lo, h: (hi - lo) / n as f64, n, dim });
    }
    out
}

/// MK-9.6 (D-091): boundary conditions name the edges `left` and `right` of a field, once
/// each, or make it periodic, not both.
fn check_boundaries(model: &Model, diags: &mut Vec<Diagnostic>) {
    for b in &model.bindings {
        let Some(bd) = &b.boundary else { continue };
        let mut err = |m: String| diags.push(Diagnostic { code: "MK-E33", message: m, element: b.id.clone() });
        if !matches!(b.ty, Type::Field { .. }) {
            err(format!("`{}` is not a field; boundary conditions belong to fields (MK-9.6)", b.name));
            continue;
        }
        if bd.periodic && !bd.fixed.is_empty() {
            err(format!("the boundary of `{}` is periodic or holds edges at values, not both", b.name));
        }
        for (k, f) in bd.fixed.iter().enumerate() {
            if !["left", "right"].contains(&f.edge.as_str()) {
                err(format!("a field on a grid of one axis has the edges `left` and `right`, not `{}`", f.edge));
            } else if bd.fixed[..k].iter().any(|g| g.edge == f.edge) {
                err(format!("the edge `{}` of `{}` is held twice", f.edge, b.name));
            }
        }
    }
}

fn const_value(c: &CExpr) -> Option<f64> {
    match c {
        CExpr::Const(Value::Num(x)) => Some(*x),
        _ => None,
    }
}

fn check_ops(tc: &mut Tc, model: &Model, index: &HashMap<Id, usize>, ops: &[Op], intervention: bool) -> Vec<COp> {
    let mut out = vec![];
    let mut targets: Vec<(usize, Option<usize>)> = vec![];
    // Bindings contributed to in this handler: contributions combine with each other, and
    // conflict with a `set` of the same binding (MK-16.4, D-073).
    let mut contributed: Vec<usize> = vec![];
    for op in ops {
        match op {
            Op::Set { target, .. } | Op::Contribute { target, .. } if target.member.is_some() => {
                tc.err("MK-E00", format!("a member's binding `{}` in a model that was not elaborated (D-057)", target.binding));
            }
            Op::Create { .. } | Op::Connect { .. } | Op::Disconnect { .. } | Op::Destroy { member: Expr::Var { .. } | Expr::Part { .. } | Expr::Item { .. } | Expr::End { .. } | Expr::Other { .. } } => {
                tc.err("MK-E00", "`create`, `destroy`, `connect` or `disconnect` in a model that was not elaborated (D-057, D-058)".into());
            }
            // After elaboration a destroy names the member's liveness binding (D-057).
            Op::Destroy { member } => match member {
                Expr::Ref { r#ref } if index.get(r#ref).is_some_and(|&i| model.bindings[i].role == Role::Discrete && model.bindings[i].ty == Type::Boolean) => {
                    out.push(COp::Destroy { binding: index[r#ref] });
                }
                _ => {
                    tc.err("MK-E00", "`destroy` names a member of a collection".into());
                }
            },
            // A member being made takes its starting values, parameters included (D-057).
            Op::Make { alive, values } => {
                match index.get(alive) {
                    Some(&i) if model.bindings[i].role == Role::Discrete && model.bindings[i].ty == Type::Boolean => {
                        out.push(COp::Set { binding: i, component: None, value: CExpr::Const(Value::Bool(true)) })
                    }
                    _ => {
                        tc.err("MK-E00", format!("`{alive}` is not the liveness of a member"));
                    }
                }
                for v in values {
                    let Some(&bi) = index.get(&v.binding) else {
                        tc.err("MK-E00", format!("unknown target `{}`", v.binding));
                        continue;
                    };
                    let b = &model.bindings[bi];
                    if !b.role.is_stored() || b.role == Role::Constant || b.role == Role::Input {
                        tc.err("MK-E06", format!("a starting value for `{}`, a {:?} binding", b.name, b.role));
                        continue;
                    }
                    if let Some(c) = tc.expect(&v.value, &b.ty) {
                        out.push(COp::Set { binding: bi, component: None, value: c });
                    }
                }
            }
            // Conditional operations are checked on their own: which of them are performed is
            // known only in the transition, where conflicts are decided (D-057).
            Op::If { r#if, then } => {
                if let Some(c) = tc.expect(r#if, &Type::Boolean) {
                    let inner = check_ops(tc, model, index, then, intervention);
                    out.push(COp::If { cond: c, ops: inner });
                }
            }
            Op::Set { target, value } | Op::Contribute { target, value } => {
                let Some(&bi) = index.get(&target.binding) else {
                    tc.err("MK-E00", format!("unknown target `{}`", target.binding));
                    continue;
                };
                let b = &model.bindings[bi];
                match b.role {
                    Role::Derived | Role::Constant | Role::Input => {
                        tc.err("MK-E06", format!("operation targets `{}`, a {:?} binding", b.name, b.role));
                        continue;
                    }
                    Role::Parameter if !intervention => {
                        tc.err("MK-E08", format!("handler sets the parameter `{}`", b.name));
                        continue;
                    }
                    _ => {}
                }
                if let Op::Contribute { .. } = op {
                    // `contribute x += e` (MK-14.9, D-073): into a binding that declares how
                    // contributions combine, of a type the combination applies to.
                    let Some(comb) = b.combine.clone() else {
                        tc.err("MK-E11", format!("contribution to `{}`, which has no combination: declare one, `{}: ... combine sum`", b.name, b.name));
                        continue;
                    };
                    let fits = match comb.as_str() {
                        "sum" => matches!(b.ty, Type::Quantity { .. } | Type::Vector { .. }),
                        "product" => matches!(&b.ty, Type::Quantity { dim } if dim.is_none()),
                        "min" | "max" => matches!(b.ty, Type::Quantity { .. }),
                        _ => b.ty == Type::Boolean,
                    };
                    if !fits {
                        tc.err("MK-E11", format!("`{}` cannot combine by `{comb}`: {} (sum: numbers, quantities, vectors; product: plain numbers; min, max: numbers and quantities; any, all: Booleans)", b.name, show(&b.ty)));
                        continue;
                    }
                    if target.component.is_some() || targets.iter().any(|(ob, _)| *ob == bi) {
                        tc.err("MK-E19", format!("`{}` is both set and contributed to in one handler", b.name));
                        continue;
                    }
                    if !contributed.contains(&bi) {
                        contributed.push(bi);
                    }
                    if let Some(v) = tc.expect(value, &b.ty) {
                        out.push(COp::Contribute { binding: bi, value: v, combine: comb });
                    }
                    continue;
                }
                if contributed.contains(&bi) {
                    tc.err("MK-E19", format!("`{}` is both set and contributed to in one handler", b.name));
                    continue;
                }
                let want = match (target.component, &b.ty) {
                    (None, t) => t.clone(),
                    (Some(_), Type::Vector { dim, .. }) => Type::Quantity { dim: *dim },
                    (Some(_), Type::Point { .. }) => Type::Quantity { dim: Dim::length() },
                    (Some(_), t) => {
                        tc.err("MK-E01", format!("no component of {}", show(t)));
                        continue;
                    }
                };
                let overlap = targets.iter().any(|(ob, oc)| *ob == bi && (oc.is_none() || target.component.is_none() || *oc == target.component));
                if overlap {
                    tc.err("MK-E19", format!("two operations on `{}` in one handler", b.name));
                    continue;
                }
                targets.push((bi, target.component));
                if let Some(v) = tc.expect(value, &want) {
                    out.push(COp::Set { binding: bi, component: target.component, value: v });
                }
            }
            // An emitted payload has the type the emitted event declares (D-050).
            Op::Emit { event, payload } => match model.events.iter().position(|e| &e.id == event) {
                Some(i) => match (&model.events[i].payload, payload) {
                    (None, None) => out.push(COp::Emit { event: i, payload: None }),
                    (Some(p), Some(v)) => {
                        if let Some(c) = tc.expect(v, &p.ty) {
                            out.push(COp::Emit { event: i, payload: Some(c) });
                        }
                    }
                    (Some(p), None) => {
                        tc.err("MK-E01", format!("`emit {}` needs its payload `{}`", model.events[i].name, p.name));
                    }
                    (None, Some(_)) => {
                        tc.err("MK-E01", format!("`{}` declares no payload", model.events[i].name));
                    }
                },
                None => {
                    tc.err("MK-E00", format!("unknown event `{event}`"));
                }
            },
        }
    }
    out
}

/// Checks operations submitted as an intervention against a compiled model.
pub fn check_intervention(cm: &CModel, ops: &[Op]) -> Result<Vec<COp>, Vec<Diagnostic>> {
    let mut diags = vec![];
    for op in ops {
        if let Op::Set { target, .. } = op {
            match cm.index.get(&target.binding) {
                Some(&i) if !cm.bindings[i].intervenable => diags.push(Diagnostic {
                    code: "RC-11.4",
                    message: format!("`{}` is not intervenable (D-023)", cm.bindings[i].name),
                    element: target.binding.clone(),
                }),
                _ => {}
            }
        }
    }
    if !diags.is_empty() {
        return Err(diags);
    }
    let scope = Scope { spaces: &cm.spaces, model: &cm.ir, index: &cm.index, funcs: &cm.functions, grids: &cm.grids };
    let mut tc = Tc::new(&scope, &mut diags, "intervention");
    let out = check_ops(&mut tc, &cm.ir, &cm.index, ops, true);
    if diags.is_empty() {
        Ok(out)
    } else {
        Err(diags)
    }
}

/// Compiles an expression over a checked model (used by observations and expectations).
pub fn compile_expr(cm: &CModel, e: &Expr, expected: Option<&Type>) -> Result<(CExpr, Type), Vec<Diagnostic>> {
    let mut diags = vec![];
    let scope = Scope { spaces: &cm.spaces, model: &cm.ir, index: &cm.index, funcs: &cm.functions, grids: &cm.grids };
    let mut tc = Tc::new(&scope, &mut diags, "expression");
    let r = match expected {
        Some(t) => tc.expect(e, t).map(|c| (c, t.clone())),
        None => tc.expr(e, None),
    };
    match r {
        Some(x) if diags.is_empty() => Ok(x),
        _ => Err(diags),
    }
}

/// MK-14.12: conditions in flows depend only on discrete state, parameters and constants.
fn flow_conditions(scope: &Scope, diags: &mut Vec<Diagnostic>, f: &Flow) {
    let mut bad = None;
    f.expr.walk(&mut |e| {
        if let Expr::If { r#if, .. } = e {
            let mut uses_time = false;
            r#if.walk(&mut |x| {
                if matches!(x, Expr::Builtin { builtin: Builtin::T | Builtin::Elapsed }) {
                    uses_time = true;
                }
            });
            let reads = closure_derived(scope.model, scope.index, &r#if.refs());
            let cont = reads.iter().find(|i| scope.model.bindings[**i].role == Role::Continuous);
            if uses_time {
                bad = Some("t".to_string());
            } else if let Some(i) = cont {
                bad = Some(scope.model.bindings[*i].name.clone());
            }
        }
    });
    if let Some(name) = bad {
        diags.push(Diagnostic {
            code: "MK-E12",
            message: format!("a flow condition depends on `{name}` (continuous state or time); use an event that sets discrete state"),
            element: f.id.clone(),
        });
    }
}

/// MK-15.16 (D-083, D-086): whether a random event's rate reads continuous state or time,
/// directly or through derived values; such a rate changes between transitions and its
/// clock is integrated along the steps (RC-13.8a).
fn random_rate_varies(scope: &Scope, rate: &Expr) -> bool {
    let mut time = false;
    let mut reads_time = |e: &Expr| {
        e.walk(&mut |x| {
            if matches!(x, Expr::Builtin { builtin: Builtin::T | Builtin::Elapsed }) {
                time = true;
            }
        })
    };
    reads_time(rate);
    let reads = closure_derived(scope.model, scope.index, &rate.refs());
    for &i in &reads {
        if let (Role::Derived, Some(d)) = (scope.model.bindings[i].role, &scope.model.bindings[i].def) {
            reads_time(d);
        }
    }
    time || reads.iter().any(|i| scope.model.bindings[*i].role == Role::Continuous)
}

/// MK-14.16 (D-090): whether `e` is linear in each white noise: noise appears only in sums,
/// differences, negations, products and quotients by factors free of noise, conditional
/// branches and the components of a tuple. Returns whether `e` holds noise.
fn noise_linear(e: &Expr) -> Result<bool, String> {
    let has = |e: &Expr| {
        let mut found = false;
        e.walk(&mut |x| found |= matches!(x, Expr::Builtin { builtin: Builtin::Noise }));
        found
    };
    match e {
        Expr::Builtin { builtin: Builtin::Noise } => Ok(true),
        Expr::Neg { neg } => noise_linear(neg),
        Expr::Bin { bin: BinOp::Add | BinOp::Sub, l, r } => Ok(noise_linear(l)? | noise_linear(r)?),
        Expr::Bin { bin: BinOp::Mul, l, r } => match (noise_linear(l)?, noise_linear(r)?) {
            (true, true) => Err("noise multiplies noise".into()),
            (a, b) => Ok(a || b),
        },
        Expr::Bin { bin: BinOp::Div, l, r } => {
            if has(r) {
                return Err("noise divides".into());
            }
            noise_linear(l)
        }
        Expr::If { r#if, then, r#else } => {
            if has(r#if) {
                return Err("a condition reads noise".into());
            }
            Ok(noise_linear(then)? | noise_linear(r#else)?)
        }
        Expr::Tuple { tuple } => tuple.iter().try_fold(false, |acc, x| Ok(noise_linear(x)? | acc)),
        other if has(other) => Err("noise inside a function, a power or a comparison".into()),
        _ => Ok(false),
    }
}

/// Bindings read by `refs`, following derived definitions (not flows).
fn closure_derived(model: &Model, index: &HashMap<Id, usize>, refs: &[Id]) -> BTreeSet<usize> {
    let mut seen = BTreeSet::new();
    let mut stack: Vec<usize> = refs.iter().filter_map(|r| index.get(r).copied()).collect();
    while let Some(i) = stack.pop() {
        if !seen.insert(i) {
            continue;
        }
        let b = &model.bindings[i];
        if b.role == Role::Derived {
            if let Some(d) = &b.def {
                stack.extend(d.refs().iter().filter_map(|r| index.get(r).copied()));
            }
        }
    }
    seen
}

/// An event whose handler unconditionally destroys the member its enabling condition requires
/// alive (`destroy d` in `for d in drops`, D-057, D-058) never happens again.
fn disables_itself(e: &prismal_ir::Event) -> bool {
    fn conjuncts<'e>(x: &'e Expr, out: &mut Vec<&'e Expr>) {
        match x {
            Expr::Bin { bin: BinOp::And, l, r } => {
                conjuncts(l, out);
                conjuncts(r, out);
            }
            other => out.push(other),
        }
    }
    let Some(en) = &e.enable else { return false };
    let mut cs = vec![];
    conjuncts(en, &mut cs);
    e.handler.iter().any(|op| match op {
        Op::Destroy { member } => cs.contains(&member),
        _ => false,
    })
}

/// D-038: an event is self-retriggering if its handler writes a binding that its guard
/// depends on, through derived bindings and through the flows of continuous state, where
/// the flows are specialized on the discrete values the handler sets to constants.
fn self_retriggering(model: &Model, index: &HashMap<Id, usize>, guard: &Expr, handler: &[Op]) -> bool {
    let mut written = BTreeSet::new();
    let mut consts: HashMap<Id, Expr> = HashMap::new();
    // The bindings the handler may set, with their values; a destroy sets a liveness binding
    // to false and a conditional operation may be performed (D-057).
    fn sets(ops: &[Op], out: &mut Vec<(Id, Expr)>) {
        for op in ops {
            match op {
                Op::Set { target, value } => out.push((target.binding.clone(), value.clone())),
                Op::Destroy { member: Expr::Ref { r#ref } } => out.push((r#ref.clone(), Expr::Bool { bool: false })),
                Op::If { then, .. } => sets(then, out),
                Op::Make { alive, values } => {
                    out.push((alive.clone(), Expr::Bool { bool: true }));
                    out.extend(values.iter().map(|v| (v.binding.clone(), v.value.clone())));
                }
                _ => {}
            }
        }
    }
    let mut all = vec![];
    sets(handler, &mut all);
    for (binding, value) in all {
        if let Some(&i) = index.get(&binding) {
            written.insert(i);
            if model.bindings[i].role == Role::Discrete && matches!(value, Expr::Bool { .. } | Expr::Num { .. } | Expr::Case { .. }) {
                consts.insert(binding, value);
            }
        }
    }
    let mut seen = BTreeSet::new();
    let mut stack: Vec<usize> = guard.refs().iter().filter_map(|r| index.get(r).copied()).collect();
    while let Some(i) = stack.pop() {
        if !seen.insert(i) {
            continue;
        }
        let b = &model.bindings[i];
        match b.role {
            Role::Derived => {
                if let Some(d) = &b.def {
                    stack.extend(d.refs().iter().filter_map(|r| index.get(r).copied()));
                }
            }
            Role::Continuous => {
                for f in model.flows.iter().filter(|f| f.target == b.id) {
                    let s = specialize(&f.expr, &consts);
                    stack.extend(s.refs().iter().filter_map(|r| index.get(r).copied()));
                }
            }
            _ => {}
        }
    }
    seen.intersection(&written).next().is_some()
}

/// Replaces conditionals whose condition is decided by `consts` with the chosen branch.
fn specialize(e: &Expr, consts: &HashMap<Id, Expr>) -> Expr {
    fn decide(c: &Expr, consts: &HashMap<Id, Expr>) -> Option<bool> {
        match c {
            Expr::Ref { r#ref } => match consts.get(r#ref) {
                Some(Expr::Bool { bool }) => Some(*bool),
                _ => None,
            },
            Expr::Not { not } => decide(not, consts).map(|b| !b),
            Expr::Bin { bin: bin @ (BinOp::Eq | BinOp::Ne), l, r } => match (&**l, &**r) {
                (Expr::Ref { r#ref }, v) | (v, Expr::Ref { r#ref }) => consts.get(r#ref).map(|c| (c == v) == (*bin == BinOp::Eq)),
                _ => None,
            },
            _ => None,
        }
    }
    match e {
        Expr::If { r#if, then, r#else } => match decide(r#if, consts) {
            Some(true) => specialize(then, consts),
            Some(false) => specialize(r#else, consts),
            None => Expr::If {
                r#if: r#if.clone(),
                then: Box::new(specialize(then, consts)),
                r#else: Box::new(specialize(r#else, consts)),
            },
        },
        Expr::Bin { bin, l, r } => Expr::bin(*bin, specialize(l, consts), specialize(r, consts)),
        Expr::Neg { neg } => Expr::Neg { neg: Box::new(specialize(neg, consts)) },
        other => other.clone(),
    }
}

fn topo(n: usize, deps: &[Vec<usize>], nodes: &[usize]) -> Result<Vec<usize>, Vec<usize>> {
    // Kahn's algorithm with ties broken by identity order (MK-13.6).
    let in_set: BTreeSet<usize> = nodes.iter().copied().collect();
    let mut indeg = vec![0usize; n];
    for &i in nodes {
        indeg[i] = deps[i].iter().filter(|d| in_set.contains(d)).count();
    }
    let mut ready: BTreeSet<usize> = nodes.iter().copied().filter(|&i| indeg[i] == 0).collect();
    let mut out = vec![];
    while let Some(&i) = ready.iter().next() {
        ready.remove(&i);
        out.push(i);
        for &j in nodes {
            if deps[j].contains(&i) {
                indeg[j] -= 1;
                if indeg[j] == 0 {
                    ready.insert(j);
                }
            }
        }
    }
    if out.len() == nodes.len() {
        Ok(out)
    } else {
        Err(nodes.iter().copied().filter(|i| !out.contains(i)).collect())
    }
}

fn order_derived(model: &Model, index: &HashMap<Id, usize>, diags: &mut Vec<Diagnostic>) -> Vec<usize> {
    let n = model.bindings.len();
    let mut deps = vec![vec![]; n];
    let mut nodes = vec![];
    for (i, b) in model.bindings.iter().enumerate() {
        if b.role == Role::Derived {
            nodes.push(i);
            if let Some(d) = &b.def {
                deps[i] = d.refs().into_iter().chain(b.when.iter().flat_map(|w| w.refs())).filter_map(|r| index.get(&r).copied()).filter(|&j| model.bindings[j].role == Role::Derived).collect();
            }
        }
    }
    match topo(n, &deps, &nodes) {
        Ok(o) => o,
        Err(cycle) => {
            let names: Vec<_> = cycle.iter().map(|&i| model.bindings[i].name.clone()).collect();
            diags.push(Diagnostic { code: "MK-E14", message: format!("algebraic loop among {}", names.join(", ")), element: model.id.clone() });
            vec![]
        }
    }
}

fn order_init(model: &Model, index: &HashMap<Id, usize>, diags: &mut Vec<Diagnostic>) -> Vec<usize> {
    let n = model.bindings.len();
    let mut deps = vec![vec![]; n];
    let mut nodes = vec![];
    for (i, b) in model.bindings.iter().enumerate() {
        let e = if b.role == Role::Derived { b.def.as_ref() } else { b.init.as_ref() };
        nodes.push(i);
        if let Some(e) = e {
            deps[i] = e.refs().into_iter().chain(b.when.iter().flat_map(|w| w.refs())).filter_map(|r| index.get(&r).copied()).collect();
        }
    }
    match topo(n, &deps, &nodes) {
        Ok(o) => o,
        Err(cycle) => {
            let names: Vec<_> = cycle.iter().map(|&i| model.bindings[i].name.clone()).collect();
            diags.push(Diagnostic { code: "MK-E15", message: format!("cycle among initial definitions: {}", names.join(", ")), element: model.id.clone() });
            vec![]
        }
    }
}
