//! Elaboration of contained objects and collections (MK sections 7 and 8, D-055).
//!
//! A model with parts is expanded into a flat model: each member of each part becomes the
//! bindings, processes, functions, flows, events, equations and constraints of its object
//! type, with identities `declaration@path` and names `path.name`, where the path of a member
//! is `ball`, `row[2]` or, nested, `cart.wheels[1]` (MK-7.6). Member expressions (`ball.pos`,
//! `row[2].pos`), aggregates (`sum(b.m for b in row)`), container flows over members
//! (`for b in row { der(b.vel) += ... }`) and representations repeated per member are
//! replaced by what they denote. The result is an ordinary model, checked and run as any
//! other; elaboration only moves and renames what was written.
//!
//! A part holds one object, or `count` members. A collection declared with a `capacity`
//! changes its membership during a run (D-057): it is elaborated to `capacity` members, the
//! `k`-th member made in the run being `drops[k]`, each with a liveness binding. A member not
//! alive has no flows, events, equations or constraints in effect, is left out of aggregates
//! and is not drawn; `create` makes the next member alive and `destroy` ends one. Identities
//! are never reused within a run (MK-7.7).

use std::collections::BTreeMap;
use crate::present::{Action, Arg, Check, Inverse, Item, LearnerInput, Observation, Operand, Presentation, Prop, Rep, RunCase, Schedule, Source, Subject, Tolerance, ViewKind};
use crate::{Agg, BinOp, Binding, Builtin, Constraint, Display, Document, Each, Event, Expr, Flow, Guarded, Id, Model, Op, Policy, Role, Target, Trigger, Type, Zeno, ZenoPolicy};

/// A problem found while elaborating; `element` is the identity of what was written.
#[derive(Clone, Debug, PartialEq)]
pub struct Diag {
    pub code: &'static str,
    pub message: String,
    pub element: Id,
}

/// The identity of the element declared as `id` in the member at `path`.
pub fn flat(id: &str, path: &str) -> Id {
    if path.is_empty() {
        id.to_string()
    } else {
        format!("{id}@{path}")
    }
}

/// The identity of the declaration an elaborated element comes from.
pub fn declaration(id: &str) -> &str {
    id.split('@').next().unwrap_or(id)
}

fn join(a: &str, b: &str) -> String {
    if a.is_empty() {
        b.to_string()
    } else {
        format!("{a}.{b}")
    }
}

/// A member: its path, its object type and, in a collection whose membership changes, the
/// reference to its liveness binding; the container it is declared in (type and path), the
/// key of its collection and its number there (D-058).
#[derive(Clone)]
struct Inst<'a> {
    path: String,
    ty: &'a Model,
    live: Option<Expr>,
    outer_ty: &'a Model,
    outer_path: String,
    coll: String,
    pos: u32,
}

/// A member selected by an expression: one known when elaborating, or one of a collection's
/// members chosen during the run by its number, as the member at a relation's endpoint or a
/// member payload (D-058, D-059).
#[derive(Clone)]
enum Sel<'a> {
    One(Inst<'a>),
    Chosen { members: Vec<Inst<'a>>, index: Expr, coll: String },
}

/// The key of a collection: the container's path and the part.
fn coll_key(path: &str, part: &str) -> String {
    format!("{path}#{part}")
}

/// The liveness binding of the member at `path` of the part `part` (D-057).
pub fn alive(part: &str, path: &str) -> Id {
    flat(&format!("{part}.alive"), path)
}

/// The binding counting the members a collection has made, in the container at `path`.
pub fn created(part: &str, path: &str) -> Id {
    flat(&format!("{part}.created"), path)
}

/// `a and b`, where either may be absent (always true).
fn both(a: Option<Expr>, b: Option<Expr>) -> Option<Expr> {
    match (a, b) {
        (Some(a), Some(b)) => Some(Expr::bin(BinOp::And, a, b)),
        (a, b) => a.or(b),
    }
}

fn num(x: f64) -> Expr {
    Expr::Num { num: x, unit: None }
}

/// Where the identities of an expression resolve: the object type whose declarations it
/// refers to, the member's path (empty for the model itself), loop variables, and the
/// member number in the overrides of a collection.
#[derive(Clone)]
struct Scope<'a> {
    ty: &'a Model,
    path: String,
    vars: Vec<(String, Sel<'a>)>,
    index: Option<u32>,
    /// Whether the scope's member is alive, when it or a container belongs to a collection
    /// whose membership changes (D-057).
    live: Option<Expr>,
    /// In an event repeated per member: its payload's identity as written and the member's.
    payload: Option<(Id, Id)>,
    /// The member whose body this is, for the endpoints of a relation type (D-058).
    me: Option<Inst<'a>>,
}

impl<'a> Scope<'a> {
    fn root(m: &'a Model) -> Scope<'a> {
        Scope { ty: m, path: String::new(), vars: vec![], index: None, live: None, payload: None, me: None }
    }
    /// The scope of the container a member is declared in.
    fn outer(inst: &Inst<'a>) -> Scope<'a> {
        Scope { ty: inst.outer_ty, path: inst.outer_path.clone(), vars: vec![], index: None, live: None, payload: None, me: None }
    }
    /// The scope inside a member: its type and path.
    fn inside(inst: Inst<'a>) -> Scope<'a> {
        Scope { ty: inst.ty, path: inst.path, vars: vec![], index: None, live: None, payload: None, me: None }
    }
    fn with_var(&self, var: &str, inst: Inst<'a>) -> Scope<'a> {
        let mut s = self.clone();
        s.vars.push((var.to_string(), Sel::One(inst)));
        s
    }
    fn local(&self, id: &str) -> Id {
        flat(id, &self.path)
    }
}

struct Cx<'a> {
    root: &'a Model,
    diags: Vec<Diag>,
    element: Id,
    /// Elaborating a presentation, which may read the endpoints of an undirected relation by
    /// role to draw it (D-064).
    drawing: bool,
}

impl<'a> Cx<'a> {
    fn err(&mut self, code: &'static str, message: String) {
        let d = Diag { code, message, element: self.element.clone() };
        if !self.diags.contains(&d) {
            self.diags.push(d);
        }
    }

    fn object(&mut self, id: &str) -> Option<&'a Model> {
        let o = self.root.objects.iter().find(|o| o.id == id);
        if o.is_none() {
            self.err("MK-E26", format!("unknown object type `{id}`"));
        }
        o
    }

    /// The members of the part `id` of the object type in `s`: one for a contained object,
    /// `count` for a collection, `capacity` for a collection whose membership changes.
    fn members(&mut self, s: &Scope<'a>, id: &str) -> Option<(Vec<Inst<'a>>, bool)> {
        // A part path: the contained object, then the rest of the path inside it (D-065).
        if let Some((first, rest)) = id.split_once(crate::PART_PATH) {
            let (ms, many) = self.members(s, first)?;
            if many {
                self.err("MK-E26", format!("`{}` is a collection: a path goes through contained objects only", part_name(s.ty, first)));
                return None;
            }
            return self.members(&Scope::inside(ms.into_iter().next()?), rest);
        }
        let Some(p) = s.ty.part(id) else {
            self.err("MK-E26", format!("unknown part `{id}`"));
            return None;
        };
        let ty = self.object(&p.object)?;
        let coll = coll_key(&s.path, &p.id);
        let inst = |path: String, live: Option<Expr>, pos: u32| Inst { path, ty, live, outer_ty: s.ty, outer_path: s.path.clone(), coll: coll.clone(), pos };
        Some(match (p.count, p.capacity) {
            (_, Some(n)) => (
                (1..=n)
                    .map(|k| {
                        let path = join(&s.path, &format!("{}[{k}]", p.name));
                        inst(path.clone(), Some(Expr::Ref { r#ref: alive(&p.id, &path) }), k)
                    })
                    .collect(),
                true,
            ),
            (None, None) => (vec![inst(join(&s.path, &p.name), None, 1)], false),
            (Some(n), None) => ((1..=n).map(|k| inst(join(&s.path, &format!("{}[{k}]", p.name)), None, k)).collect(), true),
        })
    }

    /// The member at the endpoint `end` of the relation instance `rel` (D-058): its number
    /// is the instance's endpoint binding.
    fn end(&mut self, rel: &Inst<'a>, end: &str) -> Option<Sel<'a>> {
        let Some(e) = rel.ty.ends.iter().find(|x| x.id == end) else {
            self.err("MK-E26", format!("`{}` is not a relation with an endpoint `{}`", rel.ty.name, end.rsplit('.').next().unwrap_or(end)));
            return None;
        };
        let outer = Scope::outer(rel);
        let (members, many) = self.members(&outer, &e.over)?;
        if !many {
            return members.into_iter().next().map(Sel::One);
        }
        let coll = members[0].coll.clone();
        Some(Sel::Chosen { members, index: Expr::Ref { r#ref: flat(end, &rel.path) }, coll })
    }

    /// The key of the collection a part (or part path, D-065) names in scope `s`.
    /// The scope that holds the part a path names, and the part's identity there:
    /// `left/atoms` is `atoms` in the scope of the object `left` (D-065, D-074).
    fn holder(&mut self, s: &Scope<'a>, id: &str) -> Option<(Scope<'a>, Id)> {
        match id.split_once(crate::PART_PATH) {
            Some((first, rest)) => {
                let (ms, many) = self.members(s, first)?;
                if many {
                    self.err("MK-E26", format!("`{}` is a collection: a path goes through contained objects only", part_name(s.ty, first)));
                    return None;
                }
                self.holder(&Scope::inside(ms.into_iter().next()?), rest)
            }
            None => Some((s.clone(), id.to_string())),
        }
    }

    fn coll_of(&mut self, s: &Scope<'a>, id: &str) -> Option<String> {
        match id.split_once(crate::PART_PATH) {
            Some((first, rest)) => {
                let (ms, _) = self.members(s, first)?;
                self.coll_of(&Scope::inside(ms.into_iter().next()?), rest)
            }
            None => Some(coll_key(&s.path, id)),
        }
    }

    /// Every relation set of the model, in the scope that holds it: the parts of the root
    /// and of every member of its parts, nested (D-065).
    fn relation_sets(&mut self, s: &Scope<'a>) -> Vec<(Scope<'a>, &'a crate::Part)> {
        let mut out = vec![];
        for p in &s.ty.parts {
            let Some(ty) = self.root.objects.iter().find(|o| o.id == p.object) else { continue };
            if !ty.ends.is_empty() {
                out.push((s.clone(), p));
                continue;
            }
            if let Some((ms, _)) = self.members(s, &p.id) {
                for m in ms {
                    out.extend(self.relation_sets(&Scope::inside(m)));
                }
            }
        }
        out
    }

    /// The member an expression selects, known now or chosen during the run.
    fn select(&mut self, s: &Scope<'a>, e: &Expr) -> Option<Sel<'a>> {
        match e {
            Expr::End { end, of } => {
                let rel = match of {
                    Some(x) => {
                        let r = self.select(s, x)?;
                        // D-064: outside its body, an undirected relation's endpoints have
                        // no order; a presentation may still draw them by role.
                        if Self::rel_type(&r).undirected && !self.drawing {
                            self.err("MK-E26", format!("the endpoints of the undirected relation `{}` have no order: read them with `s.has(o)` and `s.other(o)`", Self::rel_type(&r).name));
                            return None;
                        }
                        return self.end_of(r, end);
                    }
                    None => match &s.me {
                        Some(m) => m.clone(),
                        None => {
                            self.err("MK-E26", "an endpoint is read in its relation type, or of a relation instance (`s.a`)".into());
                            return None;
                        }
                    },
                };
                self.end(&rel, end)
            }
            // The endpoint that is not `other` (D-064): picked by comparing numbers.
            Expr::Other { other, rel } => {
                let r = self.select(s, rel)?;
                let (a, b) = self.pair(&r)?;
                let o = self.select(s, other)?;
                let ((ko, io), (ka, ia), (_, ib)) = (Self::key(&o), Self::key(&a), Self::key(&b));
                if ko != ka {
                    self.err("MK-E26", format!("`other` is given a member of another collection than the endpoints of `{}`", Self::rel_type(&r).name));
                    return None;
                }
                Some(match a {
                    Sel::One(i) => Sel::One(i),
                    Sel::Chosen { members, coll, .. } => Sel::Chosen { members, index: build_if(Expr::bin(BinOp::Eq, ia.clone(), io), ib, ia), coll },
                })
            }
            Expr::Var { var } => match s.vars.iter().rev().find(|(v, _)| v == var) {
                Some((_, sel)) => Some(sel.clone()),
                None => self.member(s, e).map(Sel::One),
            },
            _ => self.member(s, e).map(Sel::One),
        }
    }

    /// The type of a selected member.
    fn rel_type(sel: &Sel<'a>) -> &'a Model {
        match sel {
            Sel::One(i) => i.ty,
            Sel::Chosen { members, .. } => members[0].ty,
        }
    }

    /// The member at the endpoint `end` of a selected relation; for a relation chosen during
    /// the run, its number is picked among the relations' endpoint bindings (D-059).
    fn end_of(&mut self, rel: Sel<'a>, end: &str) -> Option<Sel<'a>> {
        match rel {
            Sel::One(r) => self.end(&r, end),
            Sel::Chosen { members, index, .. } => match self.end(&members[0], end)? {
                Sel::Chosen { members: ends, coll, .. } => {
                    let from = members.iter().map(|r| Expr::Ref { r#ref: flat(end, &r.path) }).collect();
                    Some(Sel::Chosen { members: ends, index: Expr::Pick { pick: Box::new(index), from }, coll })
                }
                one => Some(one),
            },
        }
    }

    /// Both endpoints of a selected relation with two endpoints in one collection, for
    /// `has` and `other` (D-064).
    fn pair(&mut self, rel: &Sel<'a>) -> Option<(Sel<'a>, Sel<'a>)> {
        let ty = Self::rel_type(rel);
        if ty.ends.len() != 2 || ty.ends[0].over != ty.ends[1].over {
            self.err("MK-E26", format!("`has` and `other` take a relation with two endpoints in one collection; `{}` is not one", ty.name));
            return None;
        }
        let (a, b) = (ty.ends[0].id.clone(), ty.ends[1].id.clone());
        Some((self.end_of(rel.clone(), &a)?, self.end_of(rel.clone(), &b)?))
    }

    /// The collection and the number of a selected member.
    fn key(sel: &Sel<'a>) -> (String, Expr) {
        match sel {
            Sel::One(i) => (i.coll.clone(), num(i.pos as f64)),
            Sel::Chosen { index, coll, .. } => (coll.clone(), index.clone()),
        }
    }

    /// A binding of a selected member: a reference, or a pick among the members.
    fn field_of(&mut self, sel: &Sel<'a>, field: &str) -> Expr {
        let ty = match sel {
            Sel::One(i) => i.ty,
            Sel::Chosen { members, .. } => members[0].ty,
        };
        if ty.binding(field).is_none() && !ty.ends.iter().any(|e| e.id == field) {
            self.err("MK-E26", format!("an object `{}` has no binding `{}`", ty.name, field.rsplit('.').next().unwrap_or(field)));
        }
        match sel {
            Sel::One(i) => Expr::Ref { r#ref: flat(field, &i.path) },
            Sel::Chosen { members, index, .. } => Expr::Pick { pick: Box::new(index.clone()), from: members.iter().map(|m| Expr::Ref { r#ref: flat(field, &m.path) }).collect() },
        }
    }

    /// The member an expression selects: a loop variable, a contained object, or a member of
    /// a collection by a constant number.
    fn member(&mut self, s: &Scope<'a>, e: &Expr) -> Option<Inst<'a>> {
        match e {
            Expr::Var { var } => match s.vars.iter().rev().find(|(v, _)| v == var) {
                Some((_, Sel::One(i))) => Some(i.clone()),
                Some((_, Sel::Chosen { .. })) => {
                    self.err("MK-E26", format!("`{var}` is chosen during the run: here a member known when the program is read is needed"));
                    None
                }
                None => {
                    self.err("MK-E26", format!("`{var}` is not a member here: it is named only inside its `for` or aggregate"));
                    None
                }
            },
            Expr::Part { part } => {
                let (ms, many) = self.members(s, part)?;
                if many {
                    self.err("MK-E26", format!("`{}` is a collection: select a member, `{}[1]`", part_name(s.ty, part), part_name(s.ty, part)));
                    return None;
                }
                ms.into_iter().next()
            }
            Expr::Item { item, index } => {
                let (ms, many) = self.members(s, item)?;
                if !many {
                    self.err("MK-E26", format!("`{}` is one object, not a collection", part_name(s.ty, item)));
                    return None;
                }
                let k = match self.constant_index(s, index) {
                    Some(k) => k,
                    None => {
                        self.err("MK-E26", format!("a member of `{}` is selected by a constant number", part_name(s.ty, item)));
                        return None;
                    }
                };
                if k < 1 || k as usize > ms.len() {
                    self.err("MK-E26", format!("`{}` has members 1 to {}, not {k}", part_name(s.ty, item), ms.len()));
                    return None;
                }
                ms.into_iter().nth(k as usize - 1)
            }
            Expr::End { .. } => {
                self.err("MK-E26", "an endpoint is chosen during the run: here a member known when the program is read is needed".into());
                None
            }
            _ => None,
        }
    }

    fn constant_index(&mut self, s: &Scope<'a>, e: &Expr) -> Option<i64> {
        match e {
            Expr::Num { num, unit: None } if num.fract() == 0.0 => Some(*num as i64),
            Expr::Builtin { builtin: Builtin::Index } => s.index.map(|k| k as i64),
            Expr::Neg { neg } => self.constant_index(s, neg).map(|k| -k),
            Expr::Bin { bin, l, r } => {
                let (a, b) = (self.constant_index(s, l)?, self.constant_index(s, r)?);
                match bin {
                    BinOp::Add => Some(a + b),
                    BinOp::Sub => Some(a - b),
                    BinOp::Mul => Some(a * b),
                    _ => None,
                }
            }
            _ => None,
        }
    }

    fn is_member(e: &Expr) -> bool {
        matches!(e, Expr::Var { .. } | Expr::Part { .. } | Expr::Item { .. } | Expr::End { .. } | Expr::Other { .. })
    }

    /// An expression with members and aggregates replaced, and identities declared in the
    /// scope's object type moved to its member.
    fn expr(&mut self, s: &Scope<'a>, e: &Expr) -> Expr {
        let b = |cx: &mut Self, x: &Expr| Box::new(cx.expr(s, x));
        match e {
            Expr::Ref { r#ref } => Expr::Ref { r#ref: s.local(r#ref) },
            Expr::Der { der } => Expr::Der { der: s.local(der) },
            Expr::Fn { r#fn } => Expr::Fn { r#fn: s.local(r#fn) },
            Expr::Payload { payload } => match &s.payload {
                Some((from, to)) if from == payload => Expr::Payload { payload: to.clone() },
                _ => Expr::Payload { payload: s.local(payload) },
            },
            Expr::Builtin { builtin: Builtin::Index } => match s.index {
                Some(k) => Expr::Num { num: k as f64, unit: None },
                None => e.clone(),
            },
            Expr::Field { field, of } => match self.select(s, of) {
                Some(sel) => self.field_of(&sel, field),
                None => num(0.0),
            },
            Expr::Part { .. } | Expr::Item { .. } | Expr::Var { .. } | Expr::End { .. } | Expr::Other { .. } => {
                self.err("MK-E26", "a member is not a value: read one of its bindings, `b.pos`".into());
                num(0.0)
            }
            // Whether a member is at either endpoint (D-064).
            Expr::Has { has, rel } => {
                let (Some(r), Some(o)) = (self.select(s, rel), self.select(s, has)) else { return Expr::Bool { bool: false } };
                let Some((a, b)) = self.pair(&r) else { return Expr::Bool { bool: false } };
                let ((ko, io), (ka, ia), (_, ib)) = (Self::key(&o), Self::key(&a), Self::key(&b));
                if ko != ka {
                    return Expr::Bool { bool: false };
                }
                Expr::bin(BinOp::Or, Expr::bin(BinOp::Eq, ia, io.clone()), Expr::bin(BinOp::Eq, ib, io))
            }
            Expr::Pick { pick, from } => Expr::Pick { pick: b(self, pick), from: from.iter().map(|x| self.expr(s, x)).collect() },
            // Members compare by identity. Members known now compare to a constant (`o != b`);
            // a member at an endpoint compares by its number in its collection (D-058).
            Expr::Bin { bin: bin @ (BinOp::Eq | BinOp::Ne), l, r } if Self::is_member(l) && Self::is_member(r) => {
                let (Some(a), Some(c)) = (self.select(s, l), self.select(s, r)) else { return Expr::Bool { bool: false } };
                if let (Sel::One(x), Sel::One(y)) = (&a, &c) {
                    return Expr::Bool { bool: (x.path == y.path) == (*bin == BinOp::Eq) };
                }
                let ((ka, ia), (kc, ic)) = (Self::key(&a), Self::key(&c));
                if ka != kc {
                    return Expr::Bool { bool: *bin == BinOp::Ne };
                }
                Expr::bin(*bin, ia, ic)
            }
            Expr::Aggregate { aggregate, var, over, body, filter } => self.aggregate(s, *aggregate, var, over, body.as_deref(), filter.as_deref()),
            Expr::Neg { neg } => Expr::Neg { neg: b(self, neg) },
            Expr::Not { not } => Expr::Not { not: b(self, not) },
            Expr::Norm { norm } => Expr::Norm { norm: b(self, norm) },
            Expr::Bin { bin, l, r } => Expr::Bin { bin: *bin, l: b(self, l), r: b(self, r) },
            Expr::Call { call, args } => Expr::Call { call: *call, args: args.iter().map(|a| self.expr(s, a)).collect() },
            Expr::Draw { draw, args } => Expr::Draw { draw: *draw, args: args.iter().map(|a| self.expr(s, a)).collect() },
            Expr::Apply { apply, args } => Expr::Apply { apply: b(self, apply), args: args.iter().map(|a| self.expr(s, a)).collect() },
            Expr::If { r#if, then, r#else } => Expr::If { r#if: b(self, r#if), then: b(self, then), r#else: b(self, r#else) },
            Expr::Tuple { tuple } => Expr::Tuple { tuple: tuple.iter().map(|a| self.expr(s, a)).collect() },
            Expr::Comp { comp, axis } => Expr::Comp { comp: b(self, comp), axis: *axis },
            Expr::Lambda { lambda } => {
                let mut l = lambda.clone();
                l.body = b(self, &lambda.body);
                Expr::Lambda { lambda: l }
            }
            Expr::Otherwise { otherwise, default } => Expr::Otherwise { otherwise: b(self, otherwise), default: b(self, default) },
            Expr::Match { r#match, arms } => Expr::Match {
                r#match: b(self, r#match),
                arms: arms.iter().map(|a| crate::Arm { case: a.case.clone(), value: self.expr(s, &a.value) }).collect(),
            },
            Expr::Extreme { extreme, terms } => Expr::Extreme {
                extreme: *extreme,
                terms: terms.iter().map(|g| Guarded { when: self.expr(s, &g.when), value: self.expr(s, &g.value) }).collect(),
            },
            Expr::Num { .. } | Expr::Bool { .. } | Expr::Case { .. } | Expr::Param { .. } | Expr::Builtin { .. } | Expr::Const { .. } | Expr::Origin { .. } => e.clone(),
        }
    }

    fn aggregate(&mut self, s: &Scope<'a>, agg: Agg, var: &str, over: &str, body: Option<&Expr>, filter: Option<&Expr>) -> Expr {
        let zero = Expr::Num { num: 0.0, unit: None };
        let Some((members, many)) = self.members(s, over) else { return zero };
        if !many {
            self.err("MK-E26", format!("`{}` is one object; an aggregate goes over a collection", part_name(s.ty, over)));
            return zero;
        }
        let mut terms: Vec<Expr> = vec![];
        let mut guarded: Vec<Guarded> = vec![];
        let changing = members.iter().any(|m| m.live.is_some());
        for inst in members {
            let live = inst.live.clone();
            let inner = s.with_var(var, inst);
            let cond = filter.map(|f| self.expr(&inner, f));
            if matches!(cond, Some(Expr::Bool { bool: false })) {
                continue;
            }
            let written = cond.filter(|c| !matches!(c, Expr::Bool { bool: true }));
            // D-057: `min` and `max` over a collection that changes take the members alive.
            if changing && matches!(agg, Agg::Min | Agg::Max) {
                if written.is_some() {
                    self.err("MK-E26", format!("a filter of `{}` compares members only (`if o != b`)", agg.name()));
                    return zero;
                }
                let value = body.map(|e| self.expr(&inner, e)).unwrap_or_else(|| zero.clone());
                guarded.push(Guarded { when: live.unwrap_or(Expr::Bool { bool: true }), value });
                continue;
            }
            // A member not alive is left out, as one failing the filter (D-057).
            let dynamic = both(live, written);
            let value = match body {
                Some(e) => self.expr(&inner, e),
                None => Expr::Num { num: 1.0, unit: None },
            };
            terms.push(match (agg, dynamic) {
                (_, None) => value,
                (Agg::Sum | Agg::Count, Some(c)) => Expr::If { r#if: Box::new(c), then: Box::new(value), r#else: Box::new(zero.clone()) },
                (Agg::Any, Some(c)) => Expr::bin(BinOp::And, c, value),
                (Agg::All, Some(c)) => Expr::bin(BinOp::Or, Expr::Not { not: Box::new(c) }, value),
                (Agg::Min | Agg::Max, Some(_)) => {
                    self.err("MK-E26", format!("a filter of `{}` compares members only (`if o != b`)", agg.name()));
                    return zero;
                }
            });
        }
        // Terms are combined as a balanced tree, in member order: an expression as deep as the
        // logarithm of the members, and pairwise summation.
        fn fold(mut terms: Vec<Expr>, f: &dyn Fn(Expr, Expr) -> Expr) -> Option<Expr> {
            match terms.len() {
                0 => None,
                1 => terms.pop(),
                n => {
                    let right = terms.split_off(n / 2);
                    Some(f(fold(terms, f)?, fold(right, f)?))
                }
            }
        }
        match agg {
            Agg::Sum | Agg::Count => fold(terms, &|a, b| Expr::bin(BinOp::Add, a, b)).unwrap_or(zero),
            Agg::Any => fold(terms, &|a, b| Expr::bin(BinOp::Or, a, b)).unwrap_or(Expr::Bool { bool: false }),
            Agg::All => fold(terms, &|a, b| Expr::bin(BinOp::And, a, b)).unwrap_or(Expr::Bool { bool: true }),
            Agg::Min | Agg::Max if changing => Expr::Extreme { extreme: if agg == Agg::Min { crate::Func::Min } else { crate::Func::Max }, terms: guarded },
            Agg::Min | Agg::Max => {
                let func = if agg == Agg::Min { crate::Func::Min } else { crate::Func::Max };
                match fold(terms, &|a, b| Expr::Call { call: func, args: vec![a, b] }) {
                    Some(e) => e,
                    None => {
                        self.err("MK-E26", format!("`{}` over no member has no value", agg.name()));
                        zero
                    }
                }
            }
        }
    }

    fn target(&mut self, s: &Scope<'a>, t: &Target) -> Target {
        let binding = match &t.member {
            None => s.local(&t.binding),
            // MK-7.10: a container writes a binding of a member it contains (D-057).
            Some(m) => match self.member(s, m) {
                Some(inst) => {
                    if inst.ty.binding(&t.binding).is_none() {
                        self.err("MK-E26", format!("an object `{}` has no binding `{}`", inst.ty.name, t.binding.rsplit('.').next().unwrap_or(&t.binding)));
                    }
                    flat(&t.binding, &inst.path)
                }
                None => t.binding.clone(),
            },
        };
        Target { binding, component: t.component, member: None }
    }

    /// The targets of an operation on a binding of a member chosen during the run (a member
    /// payload, `set b.vel`, or an endpoint, `set s.a.vel`): one per member of its collection,
    /// each with the condition that it is the one chosen (D-059). `None` for any other target.
    fn chosen_targets(&mut self, s: &Scope<'a>, t: &Target) -> Option<Vec<(Expr, Target)>> {
        let m = t.member.as_ref()?;
        let chosen = match m {
            Expr::End { .. } | Expr::Other { .. } => true,
            Expr::Var { var } => matches!(s.vars.iter().rev().find(|(v, _)| v == var), Some((_, Sel::Chosen { .. }))),
            _ => false,
        };
        if !chosen {
            return None;
        }
        let sel = self.select(s, m)?;
        let (members, index) = match sel {
            Sel::Chosen { members, index, .. } => (members, index),
            Sel::One(i) => return Some(vec![(Expr::Bool { bool: true }, Target { binding: flat(&t.binding, &i.path), component: t.component, member: None })]),
        };
        if members[0].ty.binding(&t.binding).is_none() {
            self.err("MK-E26", format!("an object `{}` has no binding `{}`", members[0].ty.name, t.binding.rsplit('.').next().unwrap_or(&t.binding)));
            return Some(vec![]);
        }
        Some(members.iter().map(|m| (Expr::bin(BinOp::Eq, index.clone(), num(m.pos as f64)), Target { binding: flat(&t.binding, &m.path), component: t.component, member: None })).collect())
    }

    /// The payload `e` supplied to an event that declares `declared`, in scope `s`: a member
    /// payload is given a member and carries its number; several are given as a tuple (D-059).
    fn payload_value(&mut self, s: &Scope<'a>, declared: Option<&crate::Payload>, e: &Expr) -> Expr {
        let Some(p) = declared else { return self.expr(s, e) };
        let items = p.declared();
        match e {
            Expr::Tuple { tuple } if items.len() > 1 && tuple.len() == items.len() => Expr::Tuple { tuple: items.iter().zip(tuple).map(|(i, x)| self.payload_item(s, i, x)).collect() },
            _ if items.len() > 1 => {
                self.err("MK-E26", format!("the payload is {} values: `{}`", items.len(), p.name));
                num(0.0)
            }
            _ => self.payload_item(s, items[0], e),
        }
    }

    fn payload_item(&mut self, s: &Scope<'a>, item: &crate::Payload, e: &Expr) -> Expr {
        let Some(of) = &item.of else { return self.expr(s, e) };
        if !Self::is_member(e) {
            self.err("MK-E26", format!("the payload `{}` is a member of `{}`: give one, `{}[1]` or a member's name", item.name, part_name(s.ty, of), part_name(s.ty, of)));
            return num(0.0);
        }
        let Some(sel) = self.select(s, e) else { return num(0.0) };
        let (coll, index) = Self::key(&sel);
        if coll != coll_key(&s.path, of) {
            self.err("MK-E26", format!("the payload `{}` is a member of `{}`", item.name, part_name(s.ty, of)));
            return num(0.0);
        }
        index
    }

    /// The operations of a handler. The creates and connects of one collection in one handler
    /// take the next free members in order, and the count of members made is set once (D-057).
    fn ops(&mut self, s: &Scope<'a>, ops: &[Op]) -> Vec<Op> {
        let mut out = vec![];
        let mut made: Vec<(Id, u32)> = vec![];
        for o in ops {
            match o {
                Op::Set { target, value } | Op::Contribute { target, value } => {
                    let value = self.expr(s, value);
                    let op = |t: Target| match o {
                        Op::Set { .. } => Op::Set { target: t, value: value.clone() },
                        _ => Op::Contribute { target: t, value: value.clone() },
                    };
                    match self.chosen_targets(s, target) {
                        Some(ts) => out.extend(ts.into_iter().map(|(cond, t)| Op::If { r#if: cond, then: vec![op(t)] })),
                        None => out.push(op(self.target(s, target))),
                    }
                }
                Op::Emit { event, payload } => {
                    let declared = s.ty.events.iter().find(|e| &e.id == event).and_then(|e| e.payload.clone());
                    let payload = payload.as_ref().map(|p| self.payload_value(s, declared.as_ref(), p));
                    out.push(Op::Emit { event: s.local(event), payload })
                }
                Op::If { r#if, then } => out.push(Op::If { r#if: self.expr(s, r#if), then: self.ops(s, then) }),
                Op::Destroy { member: Expr::Ref { .. } } | Op::Make { .. } => out.push(o.clone()),
                Op::Destroy { member } | Op::Disconnect { relation: member } => {
                    let what = if matches!(o, Op::Destroy { .. }) { "destroyed" } else { "disconnected" };
                    let Some(sel) = self.select(s, member) else { continue };
                    let relation = match &sel {
                        Sel::One(i) => !i.ty.ends.is_empty(),
                        Sel::Chosen { members, .. } => !members[0].ty.ends.is_empty(),
                    };
                    if relation != matches!(o, Op::Disconnect { .. }) {
                        self.err("MK-E26", if relation { "a relation is ended with `disconnect`".into() } else { "an object is ended with `destroy`".into() });
                        continue;
                    }
                    self.destroy(&sel, what, &mut out);
                }
                Op::Create { part, overrides } => {
                    // The member is made where its collection is held; its starting values
                    // are read here, in the handler's scope (D-074).
                    let Some((h, part)) = self.holder(s, part) else { continue };
                    let part = &part;
                    let Some(ty) = h.ty.part(part).and_then(|p| self.root.objects.iter().find(|o| o.id == p.object)) else {
                        self.err("MK-E26", format!("unknown part `{part}`"));
                        continue;
                    };
                    if !ty.ends.is_empty() {
                        self.err("MK-E26", format!("`{}` holds relations: they are made with `connect`", part_name(h.ty, part)));
                        continue;
                    }
                    let values = self.starting(s, ty, overrides, "create");
                    self.make(&h, part, values, &mut made, &mut out);
                }
                Op::Connect { part, ends, overrides } => {
                    let Some((h, part)) = self.holder(s, part) else { continue };
                    let part = &part;
                    let Some(ty) = h.ty.part(part).and_then(|p| self.root.objects.iter().find(|o| o.id == p.object)) else {
                        self.err("MK-E26", format!("unknown part `{part}`"));
                        continue;
                    };
                    if ty.ends.is_empty() {
                        self.err("MK-E26", format!("`{}` holds objects: they are made with `create`", part_name(h.ty, part)));
                        continue;
                    }
                    if ends.len() != ty.ends.len() {
                        self.err("MK-E26", format!("`{}` has {} endpoints, `connect` gives {}", ty.name, ty.ends.len(), ends.len()));
                        continue;
                    }
                    let mut values = vec![];
                    for (e, x) in ty.ends.iter().zip(ends) {
                        if let Some(v) = self.end_value(s, &h, e, x) {
                            values.push((e.id.clone(), v));
                        }
                    }
                    values.extend(self.starting(s, ty, overrides, "connect"));
                    self.make(&h, part, values, &mut made, &mut out);
                }
            }
        }
        for (counter, n) in made {
            out.push(Op::Set { target: Target::of(counter.clone()), value: Expr::bin(BinOp::Add, Expr::Ref { r#ref: counter }, num(n as f64)) });
        }
        out
    }

    /// Starting values given by `create` or `connect`: stored bindings of the type only.
    fn starting(&mut self, s: &Scope<'a>, ty: &'a Model, overrides: &[crate::present::Override], op: &str) -> Vec<(Id, Expr)> {
        let mut values = vec![];
        for ov in overrides {
            match ty.binding(&ov.binding) {
                None => self.err("MK-E26", format!("an object `{}` has no binding `{}`", ty.name, ov.binding.rsplit('.').next().unwrap_or(&ov.binding))),
                Some(b) if matches!(b.role, Role::Derived | Role::Input | Role::Constant) => self.err("MK-E26", format!("`{op}` gives starting values to stored bindings; `{}` is not one", b.name)),
                Some(_) => values.push((ov.binding.clone(), self.expr(s, &ov.value))),
            }
        }
        values
    }

    /// The number, in the endpoint's collection, of the member `x` given for the endpoint `e`:
    /// `x` is read in the handler's scope `s`, the endpoint's collection in the scope `h` that
    /// holds the relation set (D-074).
    fn end_value(&mut self, s: &Scope<'a>, h: &Scope<'a>, e: &crate::End, x: &Expr) -> Option<Expr> {
        let sel = self.select(s, x)?;
        let (k, idx) = Self::key(&sel);
        if Some(k) != self.coll_of(h, &e.over) {
            self.err("MK-E26", format!("the endpoint `{}` is a member of `{}`", e.name, part_name(h.ty, &e.over)));
            return None;
        }
        Some(idx)
    }

    /// Makes the next member of the collection `part`, with the values given (D-057, D-058).
    fn make(&mut self, s: &Scope<'a>, part: &Id, values: Vec<(Id, Expr)>, made: &mut Vec<(Id, u32)>, out: &mut Vec<Op>) {
        let p = s.ty.part(part).expect("known part");
        let Some(cap) = p.capacity else {
            self.err("MK-E26", format!("`{}` has a fixed membership: declare it with `max` to make members (`{}: {}[max 20]`)", p.name, p.name, part_name_of(self.root, &p.object)));
            return;
        };
        let start = p.count.unwrap_or(0);
        let counter = created(part, &s.path);
        // Creates of one collection in one handler are counted together, wherever written.
        let j = match made.iter_mut().find(|(id, _)| *id == counter) {
            Some((_, n)) => {
                *n += 1;
                *n - 1
            }
            None => {
                made.push((counter.clone(), 1));
                0
            }
        };
        // The member made is the next one: `created` members exist before it.
        for k in (start + j + 1)..=cap {
            let path = join(&s.path, &format!("{}[{k}]", p.name));
            let then = vec![Op::Make { alive: alive(part, &path), values: values.iter().map(|(b, v)| crate::present::Override { binding: flat(b, &path), value: v.clone() }).collect() }];
            let cond = Expr::bin(BinOp::Eq, Expr::Ref { r#ref: counter.clone() }, num((k - 1 - j) as f64));
            out.push(Op::If { r#if: cond, then });
        }
    }

    /// Ends the selected member and disconnects the relations it is an endpoint of, in the
    /// same transition (MK-8.6, D-058).
    fn destroy(&mut self, sel: &Sel<'a>, what: &str, out: &mut Vec<Op>) {
        let (members, index) = match sel {
            Sel::One(i) => (vec![i.clone()], None),
            Sel::Chosen { members, index, .. } => (members.clone(), Some(index.clone())),
        };
        if members.iter().any(|m| m.live.is_none()) {
            self.err("MK-E26", format!("only a member of a collection declared with `max` can be {what} (`row: Ball[max 20]`)"));
            return;
        }
        for m in &members {
            let op = Op::Destroy { member: m.live.clone().unwrap() };
            match &index {
                None => out.push(op),
                Some(ix) => out.push(Op::If { r#if: Expr::bin(BinOp::Eq, ix.clone(), num(m.pos as f64)), then: vec![op] }),
            }
        }
        let (coll, idx) = Self::key(sel);
        // Relation sets anywhere in the model may have endpoints in the collection (D-065).
        for (outer, p) in self.relation_sets(&Scope::root(self.root)) {
            let Some(rty) = self.root.objects.iter().find(|o| o.id == p.object) else { continue };
            for e in &rty.ends {
                if self.coll_of(&outer, &e.over) != Some(coll.clone()) {
                    continue;
                }
                let Some((rels, _)) = self.members(&outer, &p.id) else { continue };
                for r in rels {
                    let Some(live) = r.live.clone() else {
                        self.err("MK-E26", format!("`{}` holds relations with members that can be {what}: declare it with `max`", p.name));
                        return;
                    };
                    let cond = Expr::bin(BinOp::And, live.clone(), Expr::bin(BinOp::Eq, Expr::Ref { r#ref: flat(&e.id, &r.path) }, idx.clone()));
                    out.push(Op::If { r#if: cond, then: vec![Op::Destroy { member: live }] });
                }
            }
        }
    }

    /// An event in scope `s`, with the identity and name it has there, in effect while `live`.
    fn event(&mut self, s: &Scope<'a>, e: &Event, id: Id, name: String, live: Option<Expr>) -> Event {
        self.element = e.id.clone();
        // D-059: a member payload is a member of its collection chosen by the number the
        // occurrence carries. The event is enabled only for a member that exists and is alive.
        let mut scope = s.clone();
        let mut payload = e.payload.clone();
        let mut chosen: Option<Expr> = None;
        if let Some(p) = &mut payload {
            let several = !p.items.is_empty();
            let items: Vec<&mut crate::Payload> = if several { p.items.iter_mut().collect() } else { vec![p] };
            for (k, item) in items.into_iter().enumerate() {
                let Some(of) = item.of.clone() else { continue };
                let Some((members, many)) = self.members(&scope, &of) else { continue };
                if !many {
                    self.err("MK-E26", format!("`{}` is one object; a member payload names a member of a collection", part_name(scope.ty, &of)));
                    continue;
                }
                let read = Expr::Payload { payload: id.clone() };
                let index = if several { Expr::Comp { comp: Box::new(read), axis: k } } else { read };
                let exists = Expr::Pick { pick: Box::new(index.clone()), from: members.iter().map(|m| m.live.clone().unwrap_or(Expr::Bool { bool: true })).collect() };
                chosen = both(chosen, Some(exists));
                item.members = Some(join(&scope.path, &part_name(scope.ty, &of)));
                let coll = coll_key(&scope.path, &of);
                scope.vars.push((item.name.clone(), Sel::Chosen { members, index, coll }));
            }
        }
        let s = &scope;
        let trigger = match &e.trigger {
            Trigger::Rising { guard } => Trigger::Rising { guard: self.expr(s, guard) },
            Trigger::Falling { guard } => Trigger::Falling { guard: self.expr(s, guard) },
            Trigger::Crossing { guard } => Trigger::Crossing { guard: self.expr(s, guard) },
            Trigger::At { time } => Trigger::At { time: self.expr(s, time) },
            Trigger::Every { period, from } => Trigger::Every { period: self.expr(s, period), from: self.expr(s, from) },
            Trigger::On { event } => Trigger::On { event: s.local(event) },
            Trigger::Input { binding } => Trigger::Input { binding: s.local(binding) },
            Trigger::Level { cond } => Trigger::Level { cond: self.expr(s, cond) },
            Trigger::Start => Trigger::Start,
            Trigger::Request => Trigger::Request,
            Trigger::Random { rate } => Trigger::Random { rate: self.expr(s, rate) },
        };
        // The guard of a member not alive is not evaluated: its values may not exist (D-058).
        let trigger = match (trigger, &live) {
            (Trigger::Rising { guard }, Some(l)) => Trigger::Rising { guard: Self::guarded(l, guard) },
            (Trigger::Falling { guard }, Some(l)) => Trigger::Falling { guard: Self::guarded(l, guard) },
            (Trigger::Crossing { guard }, Some(l)) => Trigger::Crossing { guard: Self::guarded(l, guard) },
            // A member not alive has rate 0 (D-083).
            (Trigger::Random { rate }, Some(l)) => Trigger::Random { rate: Self::guarded(l, rate) },
            (t, _) => t,
        };
        let enable = e.enable.as_ref().map(|c| self.expr(s, c));
        Event {
            id,
            name,
            trigger,
            enable: both(live, both(chosen, enable)),
            handler: self.ops(s, &e.handler),
            zeno: e.zeno.as_ref().map(|z| Zeno {
                policy: match &z.policy {
                    ZenoPolicy::Stop => ZenoPolicy::Stop,
                    ZenoPolicy::Settle { ops } => ZenoPolicy::Settle { ops: self.ops(s, ops) },
                },
                ..z.clone()
            }),
            process: e.process.as_ref().map(|p| s.local(p)),
            payload,
            each: None,
            notes: e.notes.clone(),
        }
    }

    fn guarded(live: &Expr, g: Expr) -> Expr {
        Expr::If { r#if: Box::new(live.clone()), then: Box::new(g), r#else: Box::new(num(0.0)) }
    }

    /// The events of `e` in scope `s`: one, or one per member of its loop (D-057), named
    /// `leave[k]` after the member's number.
    fn events(&mut self, s: &Scope<'a>, e: &Event, out: &mut Vec<Event>) {
        match &e.each {
            None => {
                let ev = self.event(s, e, s.local(&e.id), join(&s.path, &e.name), s.live.clone());
                out.push(ev);
            }
            Some(Each { var, over }) => {
                self.element = e.id.clone();
                let Some((members, _)) = self.members(s, over) else { return };
                for (k, inst) in members.into_iter().enumerate() {
                    let id = flat(&e.id, &inst.path);
                    let live = both(s.live.clone(), inst.live.clone());
                    let mut inner = s.with_var(var, inst);
                    inner.payload = Some((e.id.clone(), id.clone()));
                    let name = join(&s.path, &format!("{}[{}]", e.name, k + 1));
                    let ev = self.event(&inner, e, id, name, live);
                    out.push(ev);
                }
            }
        }
    }

    /// The flows of `f` in scope `s`: one, or one per member of its loop. A flow writing a
    /// member that is not alive, or written by one, has no effect (D-057).
    fn flows(&mut self, s: &Scope<'a>, f: &Flow, out: &mut Vec<Flow>) {
        self.element = f.id.clone();
        let one = |cx: &mut Self, scope: &Scope<'a>, id: Id, out: &mut Vec<Flow>| {
            let (target, live) = match &f.member {
                None => (scope.local(&f.target), scope.live.clone()),
                Some(m) => match cx.member(scope, m) {
                    Some(inst) => {
                        if inst.ty.binding(&f.target).is_none() {
                            cx.err("MK-E26", format!("an object `{}` has no binding `{}`", inst.ty.name, f.target.rsplit('.').next().unwrap_or(&f.target)));
                        }
                        (flat(&f.target, &inst.path), both(scope.live.clone(), inst.live))
                    }
                    None => return,
                },
            };
            let mut expr = cx.expr(scope, &f.expr);
            if let Some(c) = live {
                expr = Expr::If { r#if: Box::new(c), then: Box::new(expr), r#else: Box::new(num(0.0)) };
            }
            out.push(Flow { id, target, kind: f.kind, expr, process: f.process.as_ref().map(|p| s.local(p)), member: None, each: None });
        };
        match &f.each {
            None => one(self, s, s.local(&f.id), out),
            Some(Each { var, over }) => {
                let Some((members, _)) = self.members(s, over) else { return };
                for inst in members {
                    let id = flat(&f.id, &inst.path);
                    let inner = s.with_var(var, inst);
                    one(self, &inner, id, out);
                }
            }
        }
    }

    /// Everything a scope's object type declares, moved to the scope's member, then its parts.
    fn body(&mut self, s: &Scope<'a>, overrides: &[(Id, Expr)], out: &mut Model, depth: usize) {
        let ty = s.ty;
        for b in &ty.bindings {
            self.element = b.id.clone();
            let mut nb = b.clone();
            nb.id = s.local(&b.id);
            nb.name = join(&s.path, &b.name);
            nb.init = b.init.as_ref().map(|e| self.expr(s, e));
            nb.def = b.def.as_ref().map(|e| self.expr(s, e));
            if let Some((_, v)) = overrides.iter().find(|(id, _)| id == &b.id) {
                match b.role {
                    // MK-7.11: a connection makes the input follow an expression of the container.
                    crate::Role::Input => {
                        nb.role = crate::Role::Derived;
                        nb.init = None;
                        nb.def = Some(v.clone());
                    }
                    crate::Role::Derived => self.err("MK-E26", format!("`{}` is derived: it cannot be given a value when the object is declared", b.name)),
                    _ => nb.init = Some(v.clone()),
                }
            }
            // A member not alive has no derived values (D-058).
            if nb.role == Role::Derived {
                nb.when = s.live.clone();
            }
            out.bindings.push(nb);
        }
        // A relation instance's endpoints: the numbers of its members in their collections,
        // starting at the part's overrides; an instance not made yet points at member 1.
        for e in &ty.ends {
            let init = overrides.iter().find(|(id, _)| id == &e.id).map(|(_, v)| v.clone()).unwrap_or_else(|| num(1.0));
            out.bindings.push(Binding {
                id: s.local(&e.id),
                name: join(&s.path, &e.name),
                role: Role::Discrete,
                ty: Type::real(),
                init: Some(init),
                def: None,
                intervenable: None,
                private: true,
                display: Display::default(),
                notes: vec![],
                when: None,
            combine: None,
            });
        }
        for f in &ty.functions {
            self.element = f.id.clone();
            let mut nf = f.clone();
            nf.id = s.local(&f.id);
            nf.name = join(&s.path, &f.name);
            nf.body = self.expr(s, &f.body);
            out.functions.push(nf);
        }
        for p in &ty.processes {
            let mut np = p.clone();
            np.id = s.local(&p.id);
            np.name = join(&s.path, &p.name);
            out.processes.push(np);
        }
        for f in &ty.flows {
            self.flows(s, f, &mut out.flows);
        }
        for e in &ty.events {
            self.events(s, e, &mut out.events);
        }
        for q in &ty.equations {
            self.element = q.id.clone();
            let mut nq = q.clone();
            nq.id = s.local(&q.id);
            nq.name = join(&s.path, &q.name);
            nq.lhs = self.expr(s, &q.lhs);
            nq.rhs = self.expr(s, &q.rhs);
            // A member not alive satisfies its equations (D-057).
            if let Some(c) = &s.live {
                nq.lhs = Expr::If { r#if: Box::new(c.clone()), then: Box::new(nq.lhs), r#else: Box::new(nq.rhs.clone()) };
            }
            nq.tol = q.tol.as_ref().map(|t| self.expr(s, t));
            out.equations.push(nq);
        }
        for c in &ty.constraints {
            self.element = c.id.clone();
            let mut nc = c.clone();
            nc.id = s.local(&c.id);
            nc.name = join(&s.path, &c.name);
            nc.cond = self.expr(s, &c.cond);
            if let Some(l) = &s.live {
                nc.cond = Expr::bin(BinOp::Or, Expr::Not { not: Box::new(l.clone()) }, nc.cond);
            }
            nc.tol = c.tol.as_ref().map(|t| self.expr(s, t));
            nc.attached_to = c.attached_to.as_ref().map(|a| s.local(a));
            out.constraints.push(nc);
        }
        for p in &ty.parts {
            self.element = p.id.clone();
            if depth > 16 {
                self.err("MK-E26", format!("objects contain each other without end, through `{}`", p.name));
                return;
            }
            let Some((members, _)) = self.members(s, &p.id) else { continue };
            if let Some(cap) = p.capacity {
                self.changing(s, p, cap, out);
            }
            for (k, inst) in members.into_iter().enumerate() {
                // Overrides are read in the container's scope, with `index` the member's number.
                let mut outer = s.clone();
                outer.index = p.count.or(p.capacity).map(|_| k as u32 + 1);
                // An endpoint is given a member, whose number it holds (D-058).
                let mut ov: Vec<(Id, Expr)> = vec![];
                for o in &p.overrides {
                    match inst.ty.ends.iter().find(|e| e.id == o.binding) {
                        Some(e) => {
                            if let Some(v) = self.end_value(&outer, &outer, e, &o.value) {
                                ov.push((o.binding.clone(), v));
                            }
                        }
                        None => ov.push((o.binding.clone(), self.expr(&outer, &o.value))),
                    }
                }
                for (id, _) in &ov {
                    if inst.ty.binding(id).is_none() && !inst.ty.ends.iter().any(|e| &e.id == id) {
                        self.err("MK-E26", format!("an object `{}` has no binding `{}`", inst.ty.name, id.rsplit('.').next().unwrap_or(id)));
                    }
                }
                if k == 0 && p.count.unwrap_or(0) > 0 {
                    for e in &inst.ty.ends {
                        if !ov.iter().any(|(id, _)| id == &e.id) {
                            self.err("MK-E26", format!("each starting relation of `{}` names its endpoint `{}`", p.name, e.name));
                        }
                    }
                }
                let live = both(s.live.clone(), inst.live.clone());
                let inner = Scope { ty: inst.ty, path: inst.path.clone(), vars: vec![], index: None, live, payload: None, me: Some(inst.clone()) };
                self.body(&inner, &ov, out, depth + 1);
            }
        }
    }

    /// The bindings of a collection whose membership changes (D-057): each member's
    /// liveness, the count of members made, and a constraint bounding it by the capacity.
    fn changing(&mut self, s: &Scope<'a>, p: &crate::Part, cap: u32, out: &mut Model) {
        let start = p.count.unwrap_or(0);
        if start > cap {
            self.err("MK-E26", format!("`{}` starts with {start} members but holds at most {cap}", p.name));
        }
        let hidden = |id: Id, name: String, ty: Type, init: Expr| Binding {
            id,
            name,
            role: Role::Discrete,
            ty,
            init: Some(init),
            def: None,
            intervenable: None,
            private: true,
            display: Display::default(),
            notes: vec![],
            when: None,
            combine: None,
        };
        let counter = created(&p.id, &s.path);
        out.bindings.push(hidden(counter.clone(), join(&s.path, &format!("{}.created", p.name)), Type::real(), num(start as f64)));
        for k in 1..=cap {
            let path = join(&s.path, &format!("{}[{k}]", p.name));
            out.bindings.push(hidden(alive(&p.id, &path), format!("{path}.alive"), Type::Boolean, Expr::Bool { bool: k <= start }));
        }
        out.constraints.push(Constraint {
            id: flat(&format!("{}.capacity", p.id), &s.path),
            name: join(&s.path, &format!("{}.capacity", p.name)),
            cond: Expr::bin(BinOp::Le, Expr::Ref { r#ref: counter }, num(cap as f64)),
            tol: None,
            policy: Policy::Stop,
            attached_to: None,
        });
    }
}

fn part_name_of(root: &Model, object: &str) -> String {
    root.objects.iter().find(|o| o.id == object).map(|o| o.name.clone()).unwrap_or_else(|| object.rsplit('.').next().unwrap_or(object).to_string())
}

fn part_name(ty: &Model, id: &str) -> String {
    ty.part(id).map(|p| p.name.clone()).unwrap_or_else(|| id.rsplit('.').next().unwrap_or(id).to_string())
}

/// The flat model of `m`: `m` itself when it has no parts.
pub fn model(m: &Model) -> Result<Model, Vec<Diag>> {
    if !m.has_parts() && m.objects.is_empty() {
        return Ok(m.clone());
    }
    let mut cx = Cx { root: m, diags: vec![], element: m.id.clone(), drawing: false };
    let mut out = Model { bindings: vec![], processes: vec![], flows: vec![], events: vec![], equations: vec![], constraints: vec![], functions: vec![], objects: vec![], parts: vec![], ..m.clone() };
    // Enumerations are types: one declaration serves every member (MK-2.2).
    for o in &m.objects {
        out.enums.extend(o.enums.iter().cloned());
        // D-064: an undirected relation joins two members of one collection.
        if o.undirected && (o.ends.len() != 2 || o.ends[0].over != o.ends[1].over) {
            cx.element = o.id.clone();
            cx.err("MK-E26", format!("the undirected relation `{}` has two endpoints in one collection", o.name));
        }
    }
    cx.element = m.id.clone();
    cx.body(&Scope::root(m), &[], &mut out, 0);
    if cx.diags.is_empty() {
        Ok(out)
    } else {
        Err(cx.diags)
    }
}

// ---------------------------------------------------------------- presentations and runs

struct PCx<'a, 'b> {
    cx: &'b mut Cx<'a>,
    root: Scope<'a>,
    /// Representations repeated per member: the identity written and its members'.
    families: Vec<(Id, Vec<Id>)>,
}

impl<'a> PCx<'a, '_> {
    fn e(&mut self, s: &Scope<'a>, e: &Expr) -> Expr {
        self.cx.expr(s, e)
    }

    fn arg(&mut self, s: &Scope<'a>, a: &Arg) -> Arg {
        match a {
            Arg::Expr { expr } => Arg::Expr { expr: self.e(s, expr) },
            Arg::Range { lo, hi } => Arg::Range { lo: self.e(s, lo), hi: self.e(s, hi) },
            Arg::Sampled { expr, every, instrument } => Arg::Sampled { expr: self.e(s, expr), every: self.e(s, every), instrument: instrument.clone() },
            Arg::Ensemble { expr, at, runs, instrument } => Arg::Ensemble { expr: self.e(s, expr), at: self.e(s, at), runs: self.e(s, runs), instrument: instrument.clone() },
            Arg::Scale { scale } => Arg::Scale { scale: crate::present::Scale { quantity: self.e(s, &scale.quantity), px: scale.px } },
            Arg::Text { .. } | Arg::Word { .. } | Arg::Element { .. } | Arg::Texts { .. } => a.clone(),
        }
    }

    fn reps(&mut self, s: &Scope<'a>, reps: &[Rep]) -> Vec<Rep> {
        let mut out = vec![];
        for r in reps {
            self.cx.element = r.id.clone();
            match &r.each {
                None => out.push(self.rep(s, r, None)),
                Some(Each { var, over }) => {
                    let Some((members, _)) = self.cx.members(s, over) else { continue };
                    let mut ids = vec![];
                    for (k, inst) in members.into_iter().enumerate() {
                        // A member's representation is drawn while it is alive (D-057).
                        let live = both(s.live.clone(), inst.live.clone());
                        let mut inner = s.with_var(var, inst);
                        inner.live = live;
                        let nr = self.rep(&inner, r, Some(k + 1));
                        ids.push(nr.id.clone());
                        out.push(nr);
                    }
                    self.families.push((r.id.clone(), ids));
                }
            }
        }
        out
    }

    fn rep(&mut self, s: &Scope<'a>, r: &Rep, k: Option<usize>) -> Rep {
        let suffix = |x: &str| match k {
            Some(k) => format!("{x}[{k}]"),
            None => x.to_string(),
        };
        Rep {
            each: None,
            id: suffix(&r.id),
            name: r.name.as_deref().map(suffix),
            kind: r.kind.clone(),
            sources: r.sources.iter().map(|a| self.arg(s, a)).collect(),
            props: r.props.iter().map(|p| Prop { name: p.name.clone(), value: self.arg(s, &p.value) }).collect(),
            inverse: r.inverse.as_ref().map(|inv| Inverse {
                gesture: inv.gesture.clone(),
                part: inv.part.clone(),
                live: inv.live,
                proposals: inv.proposals.iter().map(|p| crate::present::Proposal { target: self.proposed(s, p), value: self.e(s, &p.value), member: None }).collect(),
            }),
            members: self.reps(s, &r.members),
            click: r.click.as_ref().map(|c| {
                let declared = self.cx.root.events.iter().find(|e| e.id == c.event).and_then(|e| e.payload.clone());
                crate::present::Click { event: c.event.clone(), payload: c.payload.as_ref().map(|p| self.cx.payload_value(s, declared.as_ref(), p)) }
            }),
            when: both(s.live.clone(), r.when.as_ref().map(|w| self.e(s, w))),
        }
    }

    /// The binding a drag proposes: of the model, or of the member a representation repeated
    /// per member draws (D-059). A member chosen during the run, as the one at a relation's
    /// endpoint, is not dragged through the relation: the member's own representation is.
    fn proposed(&mut self, s: &Scope<'a>, p: &crate::present::Proposal) -> Id {
        let Some(m) = &p.member else { return s.local(&p.target) };
        let field = p.target.rsplit('.').next().unwrap_or(&p.target);
        match self.cx.select(s, m) {
            Some(Sel::One(inst)) => {
                if inst.ty.binding(&p.target).is_none() {
                    self.cx.err("MK-E26", format!("an object `{}` has no binding `{field}`", inst.ty.name));
                }
                flat(&p.target, &inst.path)
            }
            Some(Sel::Chosen { .. }) => {
                self.cx.err("MK-E26", format!("a drag proposes a binding of the member it draws; a member at an endpoint is dragged by its own representation (`for b in ... {{ marker(b.{field}) {{ on drag as p {{ propose b.{field} = p }} }} }}`)"));
                p.target.clone()
            }
            None => p.target.clone(),
        }
    }

    fn actions(&mut self, acts: &[Action]) -> Vec<Action> {
        let s = self.root.clone();
        let mut out = vec![];
        for a in acts {
            let ex = |p: &mut Self, e: &Option<Expr>| e.as_ref().map(|x| p.e(&s, x));
            let na = match a {
                Action::Show { view, reps } => Action::Show { view: view.clone(), reps: self.reps(&s, reps) },
                Action::Reveal { view, style, duration, reps } => Action::Reveal { view: view.clone(), style: *style, duration: ex(self, duration), reps: self.reps(&s, reps) },
                Action::Camera { view, center, zoom, duration } => Action::Camera { view: view.clone(), center: ex(self, center), zoom: ex(self, zoom), duration: ex(self, duration) },
                Action::Narrate { text, duration } => Action::Narrate { text: text.clone(), duration: ex(self, duration) },
                Action::Run { rate } => Action::Run { rate: self.e(&s, rate) },
                Action::Seek { time } => Action::Seek { time: self.e(&s, time) },
                Action::Intervene { ops } => Action::Intervene { ops: self.cx.ops(&s, ops) },
                Action::Request { event, payload } => {
                    let declared = s.ty.events.iter().find(|e| &e.id == event).and_then(|e| e.payload.clone());
                    Action::Request { event: event.clone(), payload: payload.as_ref().map(|p| self.cx.payload_value(&s, declared.as_ref(), p)) }
                }
                Action::Wait { duration } => Action::Wait { duration: self.e(&s, duration) },
                Action::Explore { limit, keep, controls, fallback } => Action::Explore { limit: ex(self, limit), keep: keep.clone(), controls: self.reps(&s, controls), fallback: self.actions(fallback) },
                Action::Sequence { actions } => Action::Sequence { actions: self.actions(actions) },
                Action::WaitLearner { limit, fallback } => Action::WaitLearner { limit: ex(self, limit), fallback: self.actions(fallback) },
                // A family of representations is highlighted, hidden, animated, released or
                // bound member by member.
                Action::Highlight { target } | Action::Hide { target, .. } | Action::Animate { target, .. } | Action::Release { target } | Action::Bind { target, .. }
                    if self.families.iter().any(|(id, _)| id == target) =>
                {
                    let ids = self.families.iter().find(|(id, _)| id == target).unwrap().1.clone();
                    for id in ids {
                        out.push(match a {
                            Action::Highlight { .. } => Action::Highlight { target: id },
                            Action::Hide { duration, .. } => Action::Hide { target: id, duration: ex(self, duration) },
                            Action::Animate { property, to, word, duration, ease, .. } => Action::Animate { target: id, property: *property, to: ex(self, to), word: word.clone(), duration: ex(self, duration), ease: *ease },
                            Action::Release { .. } => Action::Release { target: id },
                            Action::Bind { duration, .. } => Action::Bind { target: id, duration: ex(self, duration) },
                            _ => unreachable!(),
                        });
                    }
                    continue;
                }
                Action::Hide { target, duration } => Action::Hide { target: target.clone(), duration: ex(self, duration) },
                Action::Animate { target, property, to, word, duration, ease } => Action::Animate { target: target.clone(), property: *property, to: ex(self, to), word: word.clone(), duration: ex(self, duration), ease: *ease },
                Action::Bind { target, duration } => Action::Bind { target: target.clone(), duration: ex(self, duration) },
                other => other.clone(),
            };
            out.push(na);
        }
        out
    }
}

fn presentation<'a>(cx: &mut Cx<'a>, root: &'a Model, p: &Presentation) -> Presentation {
    let mut pc = PCx { cx, root: Scope::root(root), families: vec![] };
    let s = pc.root.clone();
    let mut out = p.clone();
    out.instruments = p
        .instruments
        .iter()
        .map(|i| {
            pc.cx.element = i.id.clone();
            crate::present::Instrument {
                resolution: i.resolution.as_ref().map(|e| pc.e(&s, e)),
                range: i.range.as_ref().map(|(a, b)| (pc.e(&s, a), pc.e(&s, b))),
                noise: i.noise.as_ref().map(|e| pc.e(&s, e)),
                ..i.clone()
            }
        })
        .collect();
    out.observations = p
        .observations
        .iter()
        .map(|o| {
            pc.cx.element = o.id.clone();
            Observation {
                source: match &o.source {
                    Source::Expr { expr } => Source::Expr { expr: pc.e(&s, expr) },
                    other => other.clone(),
                },
                schedule: match &o.schedule {
                    Schedule::Every { period } => Schedule::Every { period: pc.e(&s, period) },
                    Schedule::At { time, runs } => Schedule::At { time: pc.e(&s, time), runs: runs.as_ref().map(|r| pc.e(&s, r)) },
                    Schedule::Over { from, to, lo_closed, hi_closed } => Schedule::Over { from: pc.e(&s, from), to: to.as_ref().map(|t| pc.e(&s, t)), lo_closed: *lo_closed, hi_closed: *hi_closed },
                    other => other.clone(),
                },
                ..o.clone()
            }
        })
        .collect();
    for v in &mut out.views {
        pc.cx.element = v.id.clone();
        v.kind = match &v.kind {
            ViewKind::Spatial { space, scale, y_up } => ViewKind::Spatial { space: space.clone(), scale: crate::present::Scale { quantity: pc.e(&s, &scale.quantity), px: scale.px }, y_up: *y_up },
            ViewKind::Plot { x, y, follow, units, window } => ViewKind::Plot { x: [pc.e(&s, &x[0]), pc.e(&s, &x[1])], y: [pc.e(&s, &y[0]), pc.e(&s, &y[1])], follow: *follow, units: units.clone(), window: window.as_ref().map(|w| pc.e(&s, w)) },
            ViewKind::Panel => ViewKind::Panel,
        };
        v.representations = pc.reps(&s, &v.representations);
        // A click on a point of the view (D-060): its payload reads the point as `{"param": 0}`.
        if let Some(c) = &v.click {
            let declared = pc.cx.root.events.iter().find(|e| e.id == c.event).and_then(|e| e.payload.clone());
            let payload = c.payload.as_ref().map(|p| pc.cx.payload_value(&s, declared.as_ref(), p));
            v.click = Some(crate::present::Click { event: c.event.clone(), payload });
        }
    }
    if let Some(t) = &mut out.timeline {
        for sc in &mut t.scenes {
            for b in &mut sc.beats {
                pc.cx.element = b.id.clone();
                b.actions = pc.actions(&b.actions);
            }
        }
    }
    out
}

fn run<'a>(cx: &mut Cx<'a>, root: &'a Model, r: &RunCase) -> RunCase {
    let s = Scope::root(root);
    cx.element = r.id.clone();
    let mut out = r.clone();
    for o in &mut out.params {
        o.value = cx.expr(&s, &o.value);
    }
    for i in &mut out.inputs {
        i.value = cx.expr(&s, &i.value);
        i.at = i.at.as_ref().map(|a| cx.expr(&s, a));
    }
    out.end = r.end.as_ref().map(|e| cx.expr(&s, e));
    for l in &mut out.learner {
        l.at = cx.expr(&s, &l.at);
        if let LearnerInput::SetControl { value, .. } = &mut l.input {
            *value = cx.expr(&s, value);
        }
    }
    let subject = |cx: &mut Cx<'a>, sub: &Subject| match sub {
        Subject::On { expr, event, microstep } => Subject::On { expr: cx.expr(&s, expr), event: event.clone(), microstep: *microstep },
        other => other.clone(),
    };
    for x in &mut out.expectations {
        cx.element = x.id.clone();
        x.check = match &x.check {
            Check::Equal { subject: sub, expected, tolerance } => Check::Equal {
                subject: subject(cx, sub),
                expected: match expected {
                    Operand::Value { expr } => Operand::Value { expr: cx.expr(&s, expr) },
                    Operand::Subject { subject: o } => Operand::Subject { subject: subject(cx, o) },
                    Operand::List { items } => Operand::List {
                        items: items
                            .iter()
                            .map(|i| match i {
                                Item::Value { expr } => Item::Value { expr: cx.expr(&s, expr) },
                                other => other.clone(),
                            })
                            .collect(),
                    },
                },
                tolerance: match tolerance {
                    Tolerance::Abs { tol } => Tolerance::Abs { tol: cx.expr(&s, tol) },
                    Tolerance::Rel { tol } => Tolerance::Rel { tol: cx.expr(&s, tol) },
                    Tolerance::Exact => Tolerance::Exact,
                },
            },
            Check::Within { subject: sub, lo, hi, lo_closed, hi_closed } => Check::Within { subject: subject(cx, sub), lo: cx.expr(&s, lo), hi: cx.expr(&s, hi), lo_closed: *lo_closed, hi_closed: *hi_closed },
            other => other.clone(),
        };
    }
    out
}

/// Whether anything in the document needs elaboration.
pub fn needed(d: &Document) -> bool {
    d.models.iter().any(|m| m.has_parts() || !m.objects.is_empty())
}

/// The document with every model elaborated, and the presentations and runs of models with
/// parts elaborated against them. A document without parts is returned unchanged.
pub fn document(d: &Document) -> Result<Document, Vec<Diag>> {
    document_with(d, &BTreeMap::new())
}

/// The working capacity of each collection declared without a limit (D-066), by part
/// identity; a part not listed has [`default_capacity`].
pub type Capacities = BTreeMap<Id, u32>;

/// The capacity a collection without a limit starts with: room for its starting members and
/// as many again, at least 8.
pub fn default_capacity(p: &crate::Part) -> u32 {
    (2 * p.count.unwrap_or(0)).max(8)
}

/// The most members a collection without a limit grows to in one run: beyond it the run
/// stops at the capacity constraint, as for a declared limit.
pub const MAX_CAPACITY: u32 = 1 << 14;

/// The part whose capacity constraint has the identity `constraint` (D-057), when it is a
/// collection declared without a limit in `d` (D-066).
pub fn overflowed(d: &Document, constraint: &str) -> Option<Id> {
    let part = constraint.split('@').next()?.strip_suffix(".capacity")?;
    let unbounded = d.models.iter().flat_map(|m| std::iter::once(m).chain(&m.objects)).flat_map(|m| &m.parts).any(|p| p.id == part && p.unbounded);
    unbounded.then(|| part.to_string())
}

/// [`document`], with the working capacities of collections declared without a limit.
pub fn document_with(d: &Document, caps: &Capacities) -> Result<Document, Vec<Diag>> {
    let sized;
    let d = if caps_needed(d) {
        sized = with_capacities(d, caps);
        &sized
    } else {
        d
    };
    if !needed(d) {
        return Ok(d.clone());
    }
    let mut diags = vec![];
    let mut out = d.clone();
    out.models = vec![];
    for m in &d.models {
        match model(m) {
            Ok(fm) => out.models.push(fm),
            Err(ds) => {
                diags.extend(ds);
                out.models.push(m.clone());
            }
        }
    }
    for (i, p) in d.presentations.iter().enumerate() {
        if let Some(m) = d.model(&p.model).filter(|m| m.has_parts() || !m.objects.is_empty()) {
            let mut cx = Cx { root: m, diags: vec![], element: p.id.clone(), drawing: true };
            out.presentations[i] = presentation(&mut cx, m, p);
            diags.extend(cx.diags);
        }
    }
    for (i, r) in d.runs.iter().enumerate() {
        if let Some(m) = d.model(&r.model).filter(|m| m.has_parts() || !m.objects.is_empty()) {
            let mut cx = Cx { root: m, diags: vec![], element: r.id.clone(), drawing: false };
            out.runs[i] = run(&mut cx, m, r);
            diags.extend(cx.diags);
        }
    }
    if diags.is_empty() {
        Ok(out)
    } else {
        Err(diags)
    }
}

/// `if c then a else b`.
fn build_if(c: Expr, a: Expr, b: Expr) -> Expr {
    Expr::If { r#if: Box::new(c), then: Box::new(a), r#else: Box::new(b) }
}

/// Whether a document declares a collection without a limit.
fn caps_needed(d: &Document) -> bool {
    d.models.iter().flat_map(|m| std::iter::once(m).chain(&m.objects)).flat_map(|m| &m.parts).any(|p| p.unbounded)
}

/// The highest number by which `d` names a member of each collection (`drops[100]`).
fn named_members(d: &Document) -> BTreeMap<String, u32> {
    fn walk(v: &serde_json::Value, named: &mut BTreeMap<String, u32>) {
        match v {
            serde_json::Value::Object(o) => {
                if let (Some(item), Some(k)) = (o.get("item").and_then(|x| x.as_str()), o.get("index").and_then(|x| x.get("num")).and_then(|x| x.as_f64())) {
                    let e = named.entry(item.to_string()).or_default();
                    *e = (*e).max(k.max(0.0) as u32);
                }
                o.values().for_each(|x| walk(x, named));
            }
            serde_json::Value::Array(a) => a.iter().for_each(|x| walk(x, named)),
            _ => {}
        }
    }
    let mut named = BTreeMap::new();
    walk(&serde_json::to_value(d).unwrap_or_default(), &mut named);
    named
}

/// The working capacity a collection declared without a limit starts with in `d`: the
/// default, or room for every member `d` names by number.
pub fn starting_capacity(d: &Document, p: &crate::Part) -> u32 {
    default_capacity(p).max(named_members(d).get(&p.id).copied().unwrap_or(0))
}

/// `d` with each collection declared without a limit given its working capacity (D-066):
/// at least room for every member the document names by number.
fn with_capacities(d: &Document, caps: &Capacities) -> Document {
    let named = named_members(d);
    let mut out = d.clone();
    let size = |parts: &mut Vec<crate::Part>| {
        for p in parts.iter_mut().filter(|p| p.unbounded) {
            let least = named.get(&p.id).copied().unwrap_or(0);
            p.capacity = Some(caps.get(&p.id).copied().unwrap_or_else(|| default_capacity(p)).max(least));
        }
    };
    for m in &mut out.models {
        size(&mut m.parts);
        for o in &mut m.objects {
            size(&mut o.parts);
        }
    }
    out
}
