//! Expressions (04-ir section 6). Each keeps its symbolic structure (MK-10.6).

use crate::units::Unit;
use crate::{Id, Type};
use serde::{Deserialize, Serialize};
use std::ops;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BinOp {
    Add,
    Sub,
    Mul,
    Div,
    Pow,
    And,
    Or,
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Builtin {
    T,
    T0,
    Elapsed,
    /// The number of the member being declared, from 1, in the overrides of a collection
    /// (D-055).
    Index,
    /// White noise in a flow, `der(x) = a + b * noise` (MK-14.16, D-090): each occurrence an
    /// independent standard white noise, of dimension `T^(-1/2)`.
    Noise,
}

/// Aggregates over the members of a collection (MK-8.3, D-055).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Agg {
    Sum,
    Min,
    Max,
    Any,
    All,
    Count,
}

impl Agg {
    pub fn name(self) -> &'static str {
        match self {
            Agg::Sum => "sum",
            Agg::Min => "min",
            Agg::Max => "max",
            Agg::Any => "any",
            Agg::All => "all",
            Agg::Count => "count",
        }
    }
}

/// Named mathematical constants, kept by name so that formulas typeset them (D-034, D-039).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Constant {
    Pi,
}

impl Constant {
    pub fn value(self) -> f64 {
        match self {
            Constant::Pi => std::f64::consts::PI,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Func {
    Sin,
    Cos,
    Tan,
    Sqrt,
    Exp,
    Log,
    Abs,
    Min,
    Max,
    Atan2,
    /// Of a field, at the cell of a pointwise expression (MK-9.7, D-091).
    Laplacian,
    Gradient,
    /// Of a whole field: `h` times the sum of its cells, and their mean (MK-9.8, D-091).
    Integral,
    Mean,
}

/// Distributions a `draw` samples (D-083, MK-10.4).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Dist {
    /// `uniform(a, b)`: a quantity in `[a, b)`.
    Uniform,
    /// `normal(m, s)`: mean `m` (a quantity or a vector, by component), deviation `s`.
    Normal,
    /// `exponential(r)`: a waiting time at rate `r`, of the inverse dimension.
    Exponential,
    /// `bernoulli(p)`: true with probability `p`.
    Bernoulli,
    /// `integer(a, b)`: an integer from `a` to `b`, both included, equally likely.
    Integer,
    /// `poisson(m)`: a count with mean `m`.
    Poisson,
    /// `binomial(n, p)`: the number of successes in `n` trials of probability `p`.
    Binomial,
    /// `choice(x1, ..., xk)`: one of the arguments, equally likely.
    Choice,
}

impl Dist {
    pub fn name(self) -> &'static str {
        match self {
            Dist::Uniform => "uniform",
            Dist::Normal => "normal",
            Dist::Exponential => "exponential",
            Dist::Bernoulli => "bernoulli",
            Dist::Integer => "integer",
            Dist::Poisson => "poisson",
            Dist::Binomial => "binomial",
            Dist::Choice => "choice",
        }
    }
    pub fn from_name(s: &str) -> Option<Dist> {
        Dist::ALL.into_iter().find(|d| d.name() == s)
    }
    pub const ALL: [Dist; 8] = [Dist::Uniform, Dist::Normal, Dist::Exponential, Dist::Bernoulli, Dist::Integer, Dist::Poisson, Dist::Binomial, Dist::Choice];
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Lambda {
    pub params: Vec<Type>,
    pub body: Box<Expr>,
    /// Parameter names as written, for display only (`f(x) = a x^2`, D-034).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub names: Vec<String>,
}

/// An expression. Serialized with one distinguishing field per form.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Expr {
    Num {
        num: f64,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        unit: Option<Unit>,
    },
    Bool {
        bool: bool,
    },
    Case {
        case: String,
    },
    Ref {
        r#ref: Id,
    },
    Param {
        param: usize,
    },
    Builtin {
        builtin: Builtin,
    },
    Const {
        r#const: Constant,
    },
    Origin {
        origin: Id,
    },
    Der {
        der: Id,
    },
    Neg {
        neg: Box<Expr>,
    },
    Not {
        not: Box<Expr>,
    },
    Norm {
        norm: Box<Expr>,
    },
    Bin {
        bin: BinOp,
        l: Box<Expr>,
        r: Box<Expr>,
    },
    Call {
        call: Func,
        args: Vec<Expr>,
    },
    Apply {
        apply: Box<Expr>,
        args: Vec<Expr>,
    },
    /// A value drawn from a distribution, only in an event's condition and operations
    /// (MK-10.4, D-083).
    Draw {
        draw: Dist,
        args: Vec<Expr>,
    },
    If {
        r#if: Box<Expr>,
        then: Box<Expr>,
        r#else: Box<Expr>,
    },
    Tuple {
        tuple: Vec<Expr>,
    },
    Comp {
        comp: Box<Expr>,
        axis: usize,
    },
    Lambda {
        lambda: Lambda,
    },
    Otherwise {
        otherwise: Box<Expr>,
        default: Box<Expr>,
    },
    /// A declared function as a value (D-048); applied with `apply`.
    Fn {
        r#fn: Id,
    },
    /// The payload of the occurrence being handled, in the enabling condition and handler of
    /// the event with this identity (MK-15.1, D-050).
    Payload {
        payload: Id,
    },
    /// A choice by the case of an enumeration, one arm per case (MK-10.2, D-049).
    Match {
        r#match: Box<Expr>,
        arms: Vec<Arm>,
    },
    // Objects and collections (D-055). These forms select members and read their bindings;
    // elaboration (`crate::elaborate`) replaces them with references before checking.
    /// A binding (`field`, the identity of its declaration in the object type) of the
    /// member `of`: `ball.pos`, `row[2].pos`, `b.pos`.
    Field {
        field: Id,
        of: Box<Expr>,
    },
    /// A contained object: the part with this identity, of single membership.
    Part {
        part: Id,
    },
    /// A member of a collection by its number, from 1: `row[2]`.
    Item {
        item: Id,
        index: Box<Expr>,
    },
    /// An aggregate over the members of the collection `over`, each named `var` in `body`
    /// and `filter`; `count` has no body.
    Aggregate {
        aggregate: Agg,
        var: String,
        over: Id,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        body: Option<Box<Expr>>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        filter: Option<Box<Expr>>,
    },
    /// The smallest (`min`) or largest (`max`) of the values whose condition holds, and no
    /// value when none holds: `min` and `max` over a collection whose membership changes,
    /// each term guarded by its member being alive. Made by elaboration only (D-057).
    Extreme {
        extreme: Func,
        terms: Vec<Guarded>,
    },
    /// The member at an endpoint of a relation instance: `s.a` (`of` the relation), or `a`
    /// inside the relation type (`of` absent) (D-058).
    End {
        end: Id,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        of: Option<Box<Expr>>,
    },
    /// The endpoint of the relation instance `rel` that is not the member `other`: `s.other(o)`
    /// (D-064). When both endpoints are `other`, it is `other`.
    Other {
        other: Box<Expr>,
        rel: Box<Expr>,
    },
    /// Whether the member `has` is an endpoint of the relation instance `rel`: `s.has(o)`
    /// (D-064).
    Has {
        has: Box<Expr>,
        rel: Box<Expr>,
    },
    /// The `pick`-th of `from`, counted from 1, evaluating only that one: a binding of the
    /// member at an endpoint, chosen during the run. Made by elaboration only (D-058).
    Pick {
        pick: Box<Expr>,
        from: Vec<Expr>,
    },
    /// The member a loop or an aggregate is at: `b` in `for b in row`. After `Aggregate`,
    /// which also has a `var` field: untagged forms are read in declaration order.
    Var {
        var: String,
    },
}

/// A term of an `extreme`: its value counts while `when` holds (D-057).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Guarded {
    pub when: Expr,
    pub value: Expr,
}

/// One arm of a `match`: the value when the scrutinee is `case`.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Arm {
    pub case: String,
    pub value: Expr,
}

impl Expr {
    pub fn bin(op: BinOp, l: Expr, r: Expr) -> Expr {
        Expr::Bin { bin: op, l: Box::new(l), r: Box::new(r) }
    }

    /// True for a bare numeric literal other than zero (MK-3.8).
    pub fn is_bare_nonzero(&self) -> bool {
        matches!(self, Expr::Num { num, unit: None } if *num != 0.0)
    }

    /// True for the bare literal `0`.
    pub fn is_bare_zero(&self) -> bool {
        matches!(self, Expr::Num { num, unit: None } if *num == 0.0)
    }

    /// Visits every sub-expression, including this one, in pre-order.
    pub fn walk<'a>(&'a self, f: &mut dyn FnMut(&'a Expr)) {
        f(self);
        match self {
            Expr::Neg { neg: e } | Expr::Not { not: e } | Expr::Norm { norm: e } => e.walk(f),
            Expr::Comp { comp: e, .. } => e.walk(f),
            Expr::Bin { l, r, .. } => {
                l.walk(f);
                r.walk(f);
            }
            Expr::Call { args, .. } | Expr::Draw { args, .. } | Expr::Tuple { tuple: args } => args.iter().for_each(|a| a.walk(f)),
            Expr::Apply { apply, args } => {
                apply.walk(f);
                args.iter().for_each(|a| a.walk(f));
            }
            Expr::If { r#if, then, r#else } => {
                r#if.walk(f);
                then.walk(f);
                r#else.walk(f);
            }
            Expr::Lambda { lambda } => lambda.body.walk(f),
            Expr::Otherwise { otherwise, default } => {
                otherwise.walk(f);
                default.walk(f);
            }
            Expr::Match { r#match, arms } => {
                r#match.walk(f);
                arms.iter().for_each(|a| a.value.walk(f));
            }
            Expr::Field { of, .. } => of.walk(f),
            Expr::Item { index, .. } => index.walk(f),
            Expr::Aggregate { body, filter, .. } => {
                body.iter().for_each(|b| b.walk(f));
                filter.iter().for_each(|c| c.walk(f));
            }
            Expr::End { of, .. } => of.iter().for_each(|o| o.walk(f)),
            Expr::Other { other: m, rel } | Expr::Has { has: m, rel } => {
                m.walk(f);
                rel.walk(f);
            }
            Expr::Pick { pick, from } => {
                pick.walk(f);
                from.iter().for_each(|x| x.walk(f));
            }
            Expr::Extreme { terms, .. } => terms.iter().for_each(|g| {
                g.when.walk(f);
                g.value.walk(f);
            }),
            _ => {}
        }
    }

    /// Identities of the bindings this expression reads directly (not through derived bindings).
    pub fn refs(&self) -> Vec<Id> {
        let mut out = Vec::new();
        self.walk(&mut |e| {
            if let Expr::Ref { r#ref } = e {
                if !out.contains(r#ref) {
                    out.push(r#ref.clone());
                }
            }
        });
        out
    }
}

macro_rules! binop {
    ($tr:ident, $m:ident, $op:expr) => {
        impl ops::$tr for Expr {
            type Output = Expr;
            fn $m(self, r: Expr) -> Expr {
                Expr::bin($op, self, r)
            }
        }
    };
}
binop!(Add, add, BinOp::Add);
binop!(Sub, sub, BinOp::Sub);
binop!(Mul, mul, BinOp::Mul);
binop!(Div, div, BinOp::Div);

impl ops::Neg for Expr {
    type Output = Expr;
    fn neg(self) -> Expr {
        Expr::Neg { neg: Box::new(self) }
    }
}
