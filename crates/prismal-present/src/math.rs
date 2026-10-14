//! Formula typesetting from the IR for every medium (PK-6.5, D-034, D-046).
//!
//! Two stages. [`boxes`] builds a **math box tree** from an expression: rows, identifiers,
//! numbers, operators, fractions, powers, subscripts, roots, fences and cases, with the
//! precedences and display symbols of `text::print`. The tree is medium-independent; a
//! medium with its own math engine (MathML in a browser) renders the tree directly.
//! [`layout`] places the tree: text runs, rules and paths at positions in em units of the
//! formula's font size, so that any renderer that draws text and lines draws the formula
//! (vector and raster images, video frames, documents, native user interfaces).
//!
//! Layout coordinates: `x` to the right, `y` down, the origin on the baseline at the left
//! edge. Widths come from approximate metrics of a serif font; each text run carries the
//! width it was given, so that a renderer can fit its own font to it.

use crate::text::{fmt_num, symbol};
use prismal_ir::{BinOp, Builtin, Expr, Func, Id, Type};
use prismal_kernel::CModel;
use serde::Serialize;

// ---------------------------------------------------------------- box tree

/// How an operator is spaced.
#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum OpKind {
    /// `+`, `−`, `·`: medium space on both sides.
    Binary,
    /// `=`, `<`, `≤`: thick space on both sides.
    Relation,
    /// `,`: a thin space after.
    Punct,
    /// A prefix `−` or `¬`: no space.
    Prefix,
    /// Juxtaposed multiplication: a hair space, nothing drawn.
    Invisible,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(tag = "box", rename_all = "snake_case")]
pub enum MathBox {
    Row { items: Vec<MathBox> },
    /// An identifier; `binding` links a symbol to its binding (PK-6.5). Single letters are
    /// italic, names of more than one letter and units upright.
    Ident {
        text: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        binding: Option<Id>,
        upright: bool,
    },
    Number { text: String },
    Op { text: String, kind: OpKind },
    /// Words inside a formula (`if`, `otherwise`).
    Text { text: String },
    Space { em: f64 },
    Frac { num: Box<MathBox>, den: Box<MathBox> },
    Sup { base: Box<MathBox>, sup: Box<MathBox> },
    Sub { base: Box<MathBox>, sub: Box<MathBox> },
    Sqrt { body: Box<MathBox> },
    /// A body between delimiters that grow with it; `close` may be empty.
    Fenced { open: String, close: String, body: Box<MathBox> },
    /// Rows of a value and its condition, after a brace.
    Cases { rows: Vec<(MathBox, MathBox)> },
}

fn row(items: Vec<MathBox>) -> MathBox {
    MathBox::Row { items }
}

fn ident(s: &str) -> MathBox {
    MathBox::Ident { text: s.into(), binding: None, upright: s.chars().count() > 1 }
}

fn upright(s: &str) -> MathBox {
    MathBox::Ident { text: s.into(), binding: None, upright: true }
}

fn op(s: &str, kind: OpKind) -> MathBox {
    MathBox::Op { text: s.into(), kind }
}

fn parens(body: MathBox) -> MathBox {
    MathBox::Fenced { open: "(".into(), close: ")".into(), body: Box::new(body) }
}

fn list(items: Vec<MathBox>) -> MathBox {
    let mut out = vec![];
    for (i, x) in items.into_iter().enumerate() {
        if i > 0 {
            out.push(op(",", OpKind::Punct));
        }
        out.push(x);
    }
    row(out)
}

fn prec(e: &Expr) -> u8 {
    match e {
        Expr::If { .. } | Expr::Otherwise { .. } | Expr::Match { .. } => 0,
        Expr::Bin { bin, .. } => match bin {
            BinOp::Or => 1,
            BinOp::And => 2,
            BinOp::Eq | BinOp::Ne | BinOp::Lt | BinOp::Le | BinOp::Gt | BinOp::Ge => 3,
            BinOp::Add | BinOp::Sub => 4,
            // A fraction is typeset as one block.
            BinOp::Div => 8,
            BinOp::Mul => 5,
            BinOp::Pow => 7,
        },
        Expr::Neg { .. } | Expr::Not { .. } => 6,
        Expr::Num { num, unit } if *num < 0.0 || unit.is_some() => 6,
        _ => 8,
    }
}

/// A unit as upright symbols: `m/s^2` as m/s².
fn unit_box(u: &str) -> MathBox {
    let mut items = vec![];
    for (i, p) in u.split('/').enumerate() {
        if i > 0 {
            items.push(op("/", OpKind::Prefix));
        }
        items.push(match p.split_once('^') {
            Some((b, e)) => MathBox::Sup { base: Box::new(upright(b)), sup: Box::new(MathBox::Number { text: e.into() }) },
            None => upright(p),
        });
    }
    row(items)
}

/// The box tree of an expression; `params` names lambda parameters.
pub fn boxes(e: &Expr, cm: &CModel, params: &[String]) -> MathBox {
    let p = |x: &Expr| boxes(x, cm, params);
    let wrap = |x: &Expr, min: u8| if prec(x) < min { parens(p(x)) } else { p(x) };
    match e {
        Expr::Num { num, unit } => {
            let n = MathBox::Number { text: fmt_num(*num) };
            match unit {
                Some(u) => row(vec![n, MathBox::Space { em: 0.2 }, unit_box(&u.text)]),
                None => n,
            }
        }
        Expr::Bool { bool } => MathBox::Text { text: bool.to_string() },
        Expr::Case { case } => MathBox::Text { text: case.clone() },
        Expr::Ref { r#ref } => {
            let s = symbol(cm, r#ref);
            MathBox::Ident { upright: s.chars().count() > 1, text: s, binding: Some(r#ref.clone()) }
        }
        Expr::Param { param } => ident(&params.get(*param).cloned().unwrap_or_else(|| format!("x{}", param + 1))),
        Expr::Builtin { builtin } => match builtin {
            Builtin::T => ident("t"),
            Builtin::T0 => MathBox::Sub { base: Box::new(ident("t")), sub: Box::new(MathBox::Number { text: "0".into() }) },
            Builtin::Elapsed => MathBox::Text { text: "elapsed".into() },
            Builtin::Index => ident("i"),
            // White noise is written ξ in physics texts (Langevin's equation).
            Builtin::Noise => ident("ξ"),
        },
        // Elaboration replaces members and aggregates before anything is typeset (D-055).
        Expr::Field { .. } | Expr::Part { .. } | Expr::Item { .. } | Expr::Var { .. } | Expr::Aggregate { .. } | Expr::Extreme { .. } | Expr::End { .. } | Expr::Other { .. } | Expr::Has { .. } | Expr::Pick { .. } => MathBox::Text { text: "…".into() },
        Expr::Const { .. } => ident("π"),
        Expr::Origin { .. } => upright("O"),
        // A field's rate is a partial derivative, ∂T/∂t (D-091).
        Expr::Der { der } => {
            let field = cm.index.get(der).is_some_and(|&i| matches!(cm.bindings[i].ty, prismal_ir::Type::Field { .. }));
            let d = if field { "∂" } else { "d" };
            MathBox::Frac { num: Box::new(row(vec![ident(d), ident(&symbol(cm, der))])), den: Box::new(row(vec![ident(d), ident("t")])) }
        }
        Expr::Neg { neg } => row(vec![op("−", OpKind::Prefix), wrap(neg, 6)]),
        Expr::Not { not } => row(vec![op("¬", OpKind::Prefix), wrap(not, 6)]),
        Expr::Norm { norm } => MathBox::Fenced { open: "|".into(), close: "|".into(), body: Box::new(p(norm)) },
        Expr::Bin { bin: BinOp::Div, l, r } => MathBox::Frac { num: Box::new(p(l)), den: Box::new(p(r)) },
        // A fraction or a power as a base is parenthesized: (h/L)², (a²)³.
        Expr::Bin { bin: BinOp::Pow, l, r } => {
            let base = match **l {
                Expr::Bin { bin: BinOp::Div | BinOp::Pow, .. } => parens(p(l)),
                _ => wrap(l, 8),
            };
            MathBox::Sup { base: Box::new(base), sup: Box::new(p(r)) }
        }
        Expr::Bin { bin, l, r } => {
            let me = prec(e);
            let (ls, rs) = match bin {
                BinOp::Sub => (wrap(l, me), wrap(r, me + 1)),
                _ => (wrap(l, me), wrap(r, me)),
            };
            let o = match bin {
                BinOp::Add => op("+", OpKind::Binary),
                BinOp::Sub => op("−", OpKind::Binary),
                // Juxtapose, except before a number.
                BinOp::Mul if matches!(**r, Expr::Num { .. }) => op("·", OpKind::Binary),
                BinOp::Mul => op("", OpKind::Invisible),
                BinOp::And => op("∧", OpKind::Binary),
                BinOp::Or => op("∨", OpKind::Binary),
                BinOp::Eq => op("=", OpKind::Relation),
                BinOp::Ne => op("≠", OpKind::Relation),
                BinOp::Lt => op("<", OpKind::Relation),
                BinOp::Le => op("≤", OpKind::Relation),
                BinOp::Gt => op(">", OpKind::Relation),
                BinOp::Ge => op("≥", OpKind::Relation),
                BinOp::Div | BinOp::Pow => unreachable!(),
            };
            // A thin space before a named function, v² sin(2θ), and around a name of more than
            // one letter, which would otherwise run into its neighbour: 0.5 mass speed².
            let named = matches!(**r, Expr::Call { call, .. } if !matches!(call, Func::Sqrt | Func::Abs));
            let word = |x: &Expr| -> bool {
                let base = match x {
                    Expr::Bin { bin: BinOp::Pow, l, .. } => &**l,
                    other => other,
                };
                match base {
                    Expr::Ref { r#ref } => symbol(cm, r#ref).chars().count() > 1,
                    Expr::Param { param } => params.get(*param).is_some_and(|n| n.chars().count() > 1),
                    Expr::Apply { .. } => true,
                    _ => false,
                }
            };
            if *bin == BinOp::Mul && (named || word(l) || word(r)) {
                return row(vec![ls, o, MathBox::Space { em: 0.17 }, rs]);
            }
            row(vec![ls, o, rs])
        }
        Expr::Call { call: Func::Sqrt, args } => MathBox::Sqrt { body: Box::new(row(args.iter().map(p).collect())) },
        Expr::Call { call: Func::Abs, args } => MathBox::Fenced { open: "|".into(), close: "|".into(), body: Box::new(list(args.iter().map(p).collect())) },
        Expr::Draw { draw, args } => row(vec![upright(draw.name()), parens(list(args.iter().map(p).collect()))]),
        // Field operators as physics texts write them (D-091): ∇²T, ∇T, ∫T dx.
        // The dot product as texts write it (D-093): a · b.
        Expr::Call { call: Func::Dot, args } => row(vec![wrap(&args[0], 7), op("·", OpKind::Binary), wrap(&args[1], 7)]),
        Expr::Call { call: Func::Cross, args } => row(vec![wrap(&args[0], 7), op("×", OpKind::Binary), wrap(&args[1], 7)]),
        Expr::Call { call: Func::Laplacian, args } => row(vec![upright("∇²"), wrap(&args[0], 8)]),
        Expr::Call { call: Func::Gradient, args } => row(vec![upright("∇"), wrap(&args[0], 8)]),
        Expr::Call { call: Func::Integral, args } => row(vec![upright("∫"), MathBox::Space { em: 0.15 }, wrap(&args[0], 8), MathBox::Space { em: 0.15 }, upright("dx")]),
        Expr::Call { call, args } => {
            let name = match call {
                Func::Sin => "sin",
                Func::Cos => "cos",
                Func::Tan => "tan",
                Func::Exp => "exp",
                Func::Log => "log",
                Func::Min => "min",
                Func::Max => "max",
                Func::Atan2 => "atan2",
                Func::Mean => "mean",
                Func::Floor => "floor",
                Func::Ceil => "ceil",
                Func::Round => "round",
                Func::Mod => "mod",
                Func::Sqrt | Func::Abs | Func::Dot | Func::Cross | Func::Laplacian | Func::Gradient | Func::Integral => unreachable!(),
            };
            row(vec![upright(name), parens(list(args.iter().map(p).collect()))])
        }
        Expr::Apply { apply, args } => row(vec![wrap(apply, 8), parens(list(args.iter().map(p).collect()))]),
        Expr::If { r#if, then, r#else } => MathBox::Cases {
            rows: vec![
                (p(then), row(vec![MathBox::Text { text: "if".into() }, MathBox::Space { em: 0.3 }, p(r#if)])),
                (p(r#else), MathBox::Text { text: "otherwise".into() }),
            ],
        },
        Expr::Tuple { tuple } => parens(list(tuple.iter().map(p).collect())),
        Expr::Comp { comp, axis } => {
            let space = match &**comp {
                Expr::Ref { r#ref } => cm.ir.binding(r#ref).and_then(|b| match &b.ty {
                    Type::Point { space } | Type::Vector { space, .. } => Some(space.clone()),
                    _ => None,
                }),
                _ => cm.ir.default_space.clone(),
            };
            let names = space.and_then(|s| cm.spaces.iter().find(|x| x.id == s)).map(|s| s.axes.clone()).unwrap_or_else(|| vec!["x".into(), "y".into(), "z".into()]);
            let axis = names.get(*axis).cloned().unwrap_or_else(|| axis.to_string());
            MathBox::Sub { base: Box::new(wrap(comp, 8)), sub: Box::new(ident(&axis)) }
        }
        Expr::Lambda { lambda } => boxes(&lambda.body, cm, &lambda.names),
        Expr::Otherwise { otherwise, default } => {
            row(vec![p(otherwise), MathBox::Space { em: 0.3 }, MathBox::Text { text: "otherwise".into() }, MathBox::Space { em: 0.3 }, p(default)])
        }
        Expr::Payload { payload } => {
            let name = crate::text::payload_name(cm, payload);
            if name.chars().count() == 1 {
                ident(&name)
            } else {
                upright(&name)
            }
        }
        // A declared function's name: a single letter in italics, a word upright, as `sin`.
        Expr::Fn { r#fn } => {
            let name = crate::text::fn_name(cm, r#fn);
            if name.chars().count() == 1 {
                ident(&name)
            } else {
                upright(&name)
            }
        }
        // One row per case, as a conditional: `value if s = case` (D-049).
        Expr::Match { r#match, arms } => MathBox::Cases {
            rows: arms
                .iter()
                .map(|a| {
                    let cond = row(vec![p(r#match), op("=", OpKind::Relation), MathBox::Text { text: a.case.clone() }]);
                    (p(&a.value), row(vec![MathBox::Text { text: "if".into() }, MathBox::Space { em: 0.3 }, cond]))
                })
                .collect(),
        },
    }
}

/// A formula `lhs = rhs` (PK-6.5, D-034). `lhs` is the representation's label: a symbol,
/// `f(x)`, or a name such as `R`.
pub fn formula(lhs: &str, rhs: &Expr, cm: &CModel, params: &[String]) -> MathBox {
    let left = match lhs.split_once('(') {
        Some((f, rest)) => row(vec![ident(f), parens(list(rest.trim_end_matches(')').split(',').map(|a| ident(a.trim())).collect()))]),
        None => ident(lhs),
    };
    row(vec![left, op("=", OpKind::Relation), boxes(rhs, cm, params)])
}

/// A model equation `lhs = rhs` (PK-6.5).
pub fn equation(lhs: &Expr, rhs: &Expr, cm: &CModel) -> MathBox {
    row(vec![boxes(lhs, cm, &[]), op("=", OpKind::Relation), boxes(rhs, cm, &[])])
}

// ---------------------------------------------------------------- layout

/// A placed formula (D-046), in em units of its font size.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct MathLayout {
    pub width: f64,
    /// Height above the baseline.
    pub ascent: f64,
    /// Depth below the baseline.
    pub descent: f64,
    pub items: Vec<MathItem>,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(tag = "item", rename_all = "snake_case")]
pub enum MathItem {
    /// A text run with its baseline at `(x, y)`, `size` em high, given `width` em.
    Text {
        text: String,
        x: f64,
        y: f64,
        size: f64,
        width: f64,
        #[serde(skip_serializing_if = "std::ops::Not::not")]
        italic: bool,
        #[serde(skip_serializing_if = "Option::is_none")]
        binding: Option<Id>,
    },
    /// A filled rectangle: a fraction bar, a root's overline.
    Rule { x: f64, y: f64, w: f64, h: f64 },
    /// A stroked open path: a radical sign, a delimiter grown with its body.
    Path { points: Vec<[f64; 2]>, stroke: f64 },
}

/// A laid-out box: extent and items relative to its own left baseline.
struct Laid {
    w: f64,
    asc: f64,
    desc: f64,
    items: Vec<MathItem>,
}

impl Laid {
    fn empty() -> Laid {
        Laid { w: 0.0, asc: 0.0, desc: 0.0, items: vec![] }
    }

    /// Appends `other` at `(dx, dy)` and grows the extent.
    fn put(&mut self, other: Laid, dx: f64, dy: f64) {
        self.asc = self.asc.max(other.asc - dy);
        self.desc = self.desc.max(other.desc + dy);
        self.w = self.w.max(dx + other.w);
        for it in other.items {
            self.items.push(match it {
                MathItem::Text { text, x, y, size, width, italic, binding } => MathItem::Text { text, x: x + dx, y: y + dy, size, width, italic, binding },
                MathItem::Rule { x, y, w, h } => MathItem::Rule { x: x + dx, y: y + dy, w, h },
                MathItem::Path { points, stroke } => MathItem::Path { points: points.into_iter().map(|p| [p[0] + dx, p[1] + dy]).collect(), stroke },
            });
        }
    }
}

/// Approximate advance width of a character, in em, for a serif font.
fn advance(c: char) -> f64 {
    match c {
        '0'..='9' => 0.5,
        'i' | 'j' | 'l' | '.' | ',' | ':' | ';' | '\'' | '|' | '!' => 0.28,
        'f' | 'r' | 't' | 'I' | 'J' | '(' | ')' | '[' | ']' | '/' | ' ' => 0.36,
        'm' | 'w' => 0.75,
        'M' | 'W' => 0.9,
        'a'..='z' => 0.5,
        'A'..='Z' => 0.68,
        'π' | 'ω' | 'θ' | 'α' | 'β' | 'γ' | 'δ' | 'λ' | 'μ' | 'σ' | 'τ' | 'φ' | 'ϕ' | 'ρ' | 'ε' => 0.55,
        '+' | '−' | '=' | '<' | '>' | '≤' | '≥' | '≠' | '∧' | '∨' | '¬' => 0.78,
        '·' => 0.28,
        _ => 0.6,
    }
}

fn text_width(s: &str) -> f64 {
    s.chars().map(advance).sum()
}

const ASC: f64 = 0.72;
const DESC: f64 = 0.22;
/// Height of the math axis (fraction bars, operators) above the baseline.
const AXIS: f64 = 0.25;
const RULE: f64 = 0.05;

/// Room after an italic run for the slant of its last letter (italic correction), so that a
/// superscript or a closing delimiter does not touch it.
const ITALIC_CORRECTION: f64 = 0.06;

fn text(s: &str, size: f64, italic: bool, binding: Option<Id>) -> Laid {
    let w = text_width(s) * size;
    let box_w = if italic { w + ITALIC_CORRECTION * size } else { w };
    Laid { w: box_w, asc: ASC * size, desc: DESC * size, items: vec![MathItem::Text { text: s.into(), x: 0.0, y: 0.0, size, width: w, italic, binding }] }
}

/// A delimiter drawn as a path spanning `top` to `bottom` (y down), `size` em wide at most.
fn fence(c: &str, top: f64, bottom: f64, size: f64, left: bool) -> Laid {
    let h = bottom - top;
    let w = 0.35 * size;
    let stroke = RULE * size * 1.2;
    let x = |k: f64| if left { w * k } else { w * (1.0 - k) };
    let points: Vec<[f64; 2]> = match c {
        "(" | ")" => (0..=16)
            .map(|i| {
                let a = std::f64::consts::PI * (i as f64 / 16.0 - 0.5);
                [x(0.75 - 0.55 * a.cos()), top + h * (0.5 + 0.5 * a.sin())]
            })
            .collect(),
        "{" => {
            let m = top + h / 2.0;
            vec![[x(0.85), top], [x(0.5), top + 0.05 * h], [x(0.5), m - 0.06 * h], [x(0.15), m], [x(0.5), m + 0.06 * h], [x(0.5), bottom - 0.05 * h], [x(0.85), bottom]]
        }
        _ => vec![[w / 2.0, top], [w / 2.0, bottom]],
    };
    Laid { w, asc: -top, desc: bottom, items: vec![MathItem::Path { points, stroke }] }
}

fn lay(b: &MathBox, size: f64) -> Laid {
    let small = (size * 0.7).max(0.5);
    match b {
        MathBox::Row { items } => {
            let mut out = Laid::empty();
            let mut x = 0.0;
            for it in items {
                let l = lay(it, size);
                let w = l.w;
                out.put(l, x, 0.0);
                x += w;
            }
            out.w = x;
            out
        }
        MathBox::Ident { text: s, binding, upright } => text(s, size, !upright, binding.clone()),
        MathBox::Number { text: s } | MathBox::Text { text: s } => text(s, size, false, None),
        MathBox::Space { em } => Laid { w: em * size, ..Laid::empty() },
        MathBox::Op { text: s, kind } => {
            let (before, after) = match kind {
                OpKind::Binary => (0.22, 0.22),
                OpKind::Relation => (0.28, 0.28),
                OpKind::Punct => (0.0, 0.17),
                OpKind::Prefix => (0.0, 0.0),
                OpKind::Invisible => (0.06, 0.0),
            };
            let mut out = Laid { w: before * size, ..Laid::empty() };
            if !s.is_empty() {
                out.put(text(s, size, false, None), before * size, 0.0);
            }
            out.w += after * size;
            out
        }
        MathBox::Frac { num, den } => {
            let part = (size * 0.9).max(0.6);
            let (n, d) = (lay(num, part), lay(den, part));
            let pad = 0.12 * size;
            let w = n.w.max(d.w) + 2.0 * pad;
            let (axis, t, gap) = (AXIS * size, RULE * size, 0.14 * size);
            let mut out = Laid::empty();
            let (nw, dw, nd, da) = (n.w, d.w, n.desc, d.asc);
            out.put(n, (w - nw) / 2.0, -axis - t / 2.0 - gap - nd);
            out.put(d, (w - dw) / 2.0, -axis + t / 2.0 + gap + da);
            out.items.push(MathItem::Rule { x: pad / 2.0, y: -axis - t / 2.0, w: w - pad, h: t });
            out.w = w;
            out
        }
        MathBox::Sup { base, sup } => {
            let b = lay(base, size);
            let s = lay(sup, small);
            let raise = (0.42 * size).max(b.asc - 0.5 * s.asc);
            let bw = b.w;
            let mut out = Laid::empty();
            out.put(b, 0.0, 0.0);
            let sw = s.w;
            out.put(s, bw + 0.04 * size, -raise);
            out.w = bw + 0.04 * size + sw;
            out
        }
        MathBox::Sub { base, sub } => {
            let b = lay(base, size);
            let s = lay(sub, small);
            let lower = (0.2 * size).max(b.desc - 0.2 * s.asc);
            let bw = b.w;
            let sw = s.w;
            let mut out = Laid::empty();
            out.put(b, 0.0, 0.0);
            out.put(s, bw + 0.03 * size, lower);
            out.w = bw + 0.03 * size + sw;
            out
        }
        MathBox::Sqrt { body } => {
            let b = lay(body, size);
            let (t, gap) = (RULE * size, 0.12 * size);
            let top = -(b.asc + gap + t / 2.0);
            let bottom = b.desc + 0.06 * size;
            let sign = 0.6 * size;
            let bw = b.w;
            let mut out = Laid::empty();
            out.put(b, sign, 0.0);
            let mid = top + 0.6 * (bottom - top);
            out.items.push(MathItem::Path {
                points: vec![[0.03 * size, mid], [0.15 * size, mid - 0.06 * size], [0.3 * size, bottom], [0.55 * size, top], [sign + bw + 0.08 * size, top]],
                stroke: t,
            });
            out.asc = out.asc.max(-top + t);
            out.desc = out.desc.max(bottom);
            out.w = sign + bw + 0.08 * size;
            out
        }
        MathBox::Fenced { open, close, body } => {
            let b = lay(body, size);
            let (basc, bdesc) = (b.asc, b.desc);
            let tall = basc + bdesc > 1.25 * size;
            let piece = |c: &str, left: bool| {
                if tall {
                    let ext = 0.08 * size;
                    fence(c, -(basc + ext), bdesc + ext, size, left)
                } else if c == "|" {
                    fence(c, -ASC * size, DESC * size, size * 0.8, left)
                } else {
                    text(c, size, false, None)
                }
            };
            let mut out = Laid::empty();
            let mut x = 0.0;
            if !open.is_empty() {
                let o = piece(open, true);
                x = o.w;
                out.put(o, 0.0, 0.0);
            }
            let bw = b.w;
            out.put(b, x, 0.0);
            x += bw;
            if !close.is_empty() {
                let c = piece(close, false);
                let cw = c.w;
                out.put(c, x, 0.0);
                x += cw;
            }
            out.w = x;
            out
        }
        MathBox::Cases { rows } => {
            let laid: Vec<(Laid, Laid)> = rows.iter().map(|(v, c)| (lay(v, size), lay(c, size))).collect();
            let col = laid.iter().map(|r| r.0.w).fold(0.0, f64::max);
            let gap = 0.25 * size;
            let heights: Vec<f64> = laid.iter().map(|(v, c)| v.asc.max(c.asc) + v.desc.max(c.desc)).collect();
            let total: f64 = heights.iter().sum::<f64>() + gap * (rows.len().saturating_sub(1)) as f64;
            // The rows are centred on the math axis.
            let mut y = -AXIS * size - total / 2.0;
            let mut body = Laid::empty();
            for (v, c) in laid {
                let asc = v.asc.max(c.asc);
                let desc = v.desc.max(c.desc);
                body.put(v, 0.0, y + asc);
                body.put(c, col + 1.0 * size, y + asc);
                y += asc + desc + gap;
            }
            let brace = fence("{", -(body.asc + 0.05 * size), body.desc + 0.05 * size, size, true);
            let bw = brace.w;
            let mut out = Laid::empty();
            out.put(brace, 0.0, 0.0);
            let inner = body.w;
            out.put(body, bw + 0.15 * size, 0.0);
            out.w = bw + 0.15 * size + inner;
            out
        }
    }
}

/// Places a box tree at font size 1 em (D-046).
pub fn layout(b: &MathBox) -> MathLayout {
    let l = lay(b, 1.0);
    let r = |x: f64| (x * 1e4).round() / 1e4;
    let items = l
        .items
        .into_iter()
        .map(|it| match it {
            MathItem::Text { text, x, y, size, width, italic, binding } => MathItem::Text { text, x: r(x), y: r(y), size: r(size), width: r(width), italic, binding },
            MathItem::Rule { x, y, w, h } => MathItem::Rule { x: r(x), y: r(y), w: r(w), h: r(h) },
            MathItem::Path { points, stroke } => MathItem::Path { points: points.into_iter().map(|p| [r(p[0]), r(p[1])]).collect(), stroke: r(stroke) },
        })
        .collect();
    MathLayout { width: r(l.w), ascent: r(l.asc), descent: r(l.desc), items }
}
