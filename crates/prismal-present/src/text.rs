//! Text for representations: numbers, quantities with units, and expressions printed with
//! the model's display symbols (PK-6.5, PK-11.1, D-034).

use prismal_ir::{BinOp, Builtin, Dim, Expr, Func, Type};
use prismal_kernel::{CModel, Value};

/// A number with at most six significant digits, without trailing zeros: `4.2`, `3.60555`.
pub fn fmt_num(x: f64) -> String {
    if x == 0.0 || !x.is_finite() {
        return if x.is_nan() { "NaN".into() } else if x.is_infinite() { if x > 0.0 { "inf".into() } else { "-inf".into() } } else { "0".into() };
    }
    let mag = x.abs().log10().floor() as i32;
    if !(-4..6).contains(&mag) {
        let s = format!("{:.5e}", x);
        let (m, e) = s.split_once('e').unwrap();
        let m = m.trim_end_matches('0').trim_end_matches('.');
        return format!("{m}e{e}");
    }
    let decimals = (5 - mag).max(0) as usize;
    let s = format!("{:.*}", decimals, x);
    let s = if s.contains('.') { s.trim_end_matches('0').trim_end_matches('.').to_string() } else { s };
    if s == "-0" {
        "0".into()
    } else {
        s
    }
}

/// A value with `dp` decimals, in its binding's display unit when it has one (D-078):
/// `35.3 m`, `(2.0 m, 1.5 m)`, `40 %`. Truth values and cases are shown as usual.
pub fn fmt_places(cm: &CModel, idx: Option<usize>, v: &Value, ty: &Type, dp: usize) -> String {
    let fixed = |x: f64| {
        let s = format!("{:.*}", dp, x);
        if s.trim_start_matches('-').chars().all(|c| c == '0' || c == '.') { s.trim_start_matches('-').to_string() } else { s }
    };
    let with = |x: f64, unit: &str| if unit.is_empty() { fixed(x) } else { format!("{} {unit}", fixed(x)) };
    let display = idx.and_then(|i| cm.ir.binding(&cm.bindings[i].id)).and_then(|b| b.display.unit.as_ref()).and_then(|u| prismal_ir::Unit::parse(u).ok());
    match (v, ty, display) {
        (Value::Num(x), _, Some(u)) => with((x - u.offset) / u.scale, &u.text),
        (Value::Num(x), Type::Quantity { dim } | Type::Instant { dim }, None) => with(*x, &unit_text(dim)),
        (Value::Num(x), _, None) => fixed(*x),
        (Value::Vec(a), Type::Vector { dim, .. }, _) => format!("({})", components(a.as_slice()).iter().map(|x| with(*x, &unit_text(dim))).collect::<Vec<_>>().join(", ")),
        (Value::Point(a), _, _) => format!("({})", components(a.as_slice()).iter().map(|x| with(*x, "m")).collect::<Vec<_>>().join(", ")),
        _ => fmt_value(v, ty),
    }
}

/// Powers in units as an author's text shows them to people (D-079): `25 m²`, `9.81 m/s²`,
/// `2 m⁻¹`. Units elsewhere keep `^`, which source text and formula alternatives read.
pub fn superscripts(s: &str) -> String {
    const DIGITS: [char; 10] = ['⁰', '¹', '²', '³', '⁴', '⁵', '⁶', '⁷', '⁸', '⁹'];
    let mut out = String::with_capacity(s.len());
    let mut chars = s.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '^' && chars.peek().is_some_and(|n| n.is_ascii_digit() || *n == '-') {
            while let Some(&n) = chars.peek() {
                match n {
                    '-' => out.push('⁻'),
                    '0'..='9' => out.push(DIGITS[n as usize - '0' as usize]),
                    _ => break,
                }
                chars.next();
            }
        } else {
            out.push(c);
        }
    }
    out
}

/// The components of a vector or point as text shows them: a component smaller than `1e-9`
/// of the largest is rounding left by a computation (a drag mapped from pixels, a rotation)
/// and is shown as 0, never as `4.44089e-16 m`.
pub fn components(a: &[f64]) -> Vec<f64> {
    let big = a.iter().fold(0.0f64, |m, x| m.max(x.abs()));
    a.iter().map(|&x| if x.abs() <= 1e-9 * big { 0.0 } else { x }).collect()
}

/// The coherent SI unit of a dimension: `m`, `m/s`, `m/s^2`, `kg`, `J`, `N`, `1/s`.
pub fn unit_text(d: &Dim) -> String {
    let named = [
        ("L", "m"),
        ("M", "kg"),
        ("T", "s"),
        ("L/T", "m/s"),
        ("L/T^2", "m/s^2"),
        ("M L^2/T^2", "J"),
        ("M L/T^2", "N"),
        ("M L^2/T^3", "W"),
        ("M/T^2", "N/m"),
        ("1/T", "1/s"),
        ("1/L", "1/m"),
        ("M L/T", "kg m/s"),
    ];
    if d.is_none() {
        return String::new();
    }
    for (dim, text) in named {
        if Dim::parse(dim).map(|x| x == *d).unwrap_or(false) {
            return text.into();
        }
    }
    let syms = ["m", "kg", "s", "A", "K", "mol", "cd"];
    let mut parts = vec![];
    for (i, r) in d.0.iter().enumerate() {
        if r.is_zero() {
            continue;
        }
        if r.n == 1 && r.d == 1 {
            parts.push(syms[i].to_string());
        } else if r.d == 1 {
            parts.push(format!("{}^{}", syms[i], r.n));
        } else {
            parts.push(format!("{}^({}/{})", syms[i], r.n, r.d));
        }
    }
    parts.join(" ")
}

fn with_unit(x: f64, unit: &str) -> String {
    if unit.is_empty() {
        fmt_num(x)
    } else {
        format!("{} {unit}", fmt_num(x))
    }
}

/// A binding's value in its display unit when it declares one (MK-3.12): `60 deg`;
/// otherwise as `fmt_value`.
pub fn fmt_binding(cm: &CModel, idx: usize, v: &Value) -> String {
    let b = &cm.bindings[idx];
    let unit = cm.ir.binding(&b.id).and_then(|x| x.display.unit.as_ref()).and_then(|u| prismal_ir::Unit::parse(u).ok());
    match (v, unit) {
        (Value::Num(x), Some(u)) => with_unit((x - u.offset) / u.scale, &u.text),
        _ => fmt_value(v, &b.ty),
    }
}

/// A value of a type, in coherent SI units: `4.2`, `3 m`, `(4 m, -1 m)`, `true`.
/// A payload's value: members by name, `balls[2]`, several separated by commas (D-059).
pub fn fmt_payload(v: &Value, p: &prismal_ir::Payload) -> String {
    match (v, p.items.is_empty()) {
        (Value::Tuple(items), false) => items.iter().zip(&p.items).map(|(x, i)| fmt_payload(x, i)).collect::<Vec<_>>().join(", "),
        (Value::Num(k), true) if p.members.is_some() => format!("{}[{}]", p.members.as_deref().unwrap_or_default(), fmt_num(*k)),
        _ => fmt_value(v, &p.ty),
    }
}

pub fn fmt_value(v: &Value, ty: &Type) -> String {
    match (v, ty) {
        (Value::Bool(b), _) => b.to_string(),
        (Value::Num(x), Type::Quantity { dim } | Type::Instant { dim }) => with_unit(*x, &unit_text(dim)),
        (Value::Num(x), _) => fmt_num(*x),
        (Value::Vec(a), Type::Vector { dim, .. }) => {
            let u = unit_text(dim);
            format!("({})", components(a.as_slice()).iter().map(|x| with_unit(*x, &u)).collect::<Vec<_>>().join(", "))
        }
        (Value::Point(a), _) => format!("({})", components(a.as_slice()).iter().map(|x| with_unit(*x, "m")).collect::<Vec<_>>().join(", ")),
        (Value::Vec(a), _) => format!("({})", components(a.as_slice()).iter().map(|x| fmt_num(*x)).collect::<Vec<_>>().join(", ")),
        (Value::Tuple(items), Type::Tuple { items: tys }) => {
            format!("({})", items.iter().zip(tys).map(|(v, t)| fmt_value(v, t)).collect::<Vec<_>>().join(", "))
        }
        (Value::Tuple(items), _) => format!("({})", items.iter().map(|v| v.to_string()).collect::<Vec<_>>().join(", ")),
        (Value::Case(c), Type::Enum { cases, .. }) => cases.get(*c as usize).cloned().unwrap_or_default(),
        (v, _) => v.to_string(),
    }
}

/// The symbol shown for a binding: its display symbol, or its name (D-034).
pub fn symbol(cm: &CModel, id: &str) -> String {
    match cm.ir.binding(id) {
        Some(b) => b.display.symbol.clone().unwrap_or_else(|| b.name.clone()),
        None => id.rsplit('.').next().unwrap_or(id).to_string(),
    }
}

fn prec(e: &Expr) -> u8 {
    match e {
        Expr::If { .. } | Expr::Otherwise { .. } | Expr::Match { .. } => 0,
        Expr::Bin { bin, .. } => match bin {
            BinOp::Or => 1,
            BinOp::And => 2,
            BinOp::Eq | BinOp::Ne | BinOp::Lt | BinOp::Le | BinOp::Gt | BinOp::Ge => 3,
            BinOp::Add | BinOp::Sub => 4,
            BinOp::Mul | BinOp::Div => 5,
            BinOp::Pow => 7,
        },
        Expr::Neg { .. } | Expr::Not { .. } => 6,
        Expr::Num { num, .. } if *num < 0.0 => 6,
        _ => 8,
    }
}

/// An expression in linear text with display symbols; `params` names lambda parameters.
/// Products are written by juxtaposition, as in typeset mathematics: `0.5 m v^2`.
pub fn print(e: &Expr, cm: &CModel, params: &[String]) -> String {
    let p = |x: &Expr| print(x, cm, params);
    let wrap = |x: &Expr, min: u8| if prec(x) < min { format!("({})", p(x)) } else { p(x) };
    let axes = |space: Option<&str>| -> Vec<String> {
        space
            .and_then(|s| cm.spaces.iter().find(|x| x.id == s))
            .map(|s| s.axes.clone())
            .unwrap_or_else(|| vec!["x".into(), "y".into(), "z".into()])
    };
    match e {
        Expr::Num { num, unit } => match unit {
            Some(u) => format!("{} {}", fmt_num(*num), u.text),
            None => fmt_num(*num),
        },
        Expr::Bool { bool } => bool.to_string(),
        Expr::Case { case } => case.clone(),
        Expr::Ref { r#ref } => symbol(cm, r#ref),
        Expr::Param { param } => params.get(*param).cloned().unwrap_or_else(|| format!("x{}", param + 1)),
        Expr::Builtin { builtin } => match builtin {
            Builtin::T => "t".into(),
            Builtin::T0 => "t0".into(),
            Builtin::Elapsed => "elapsed".into(),
            Builtin::Index => "index".into(),
            Builtin::Noise => "noise".into(),
        },
        // Elaboration replaces members and aggregates before anything is printed (D-055).
        Expr::Field { .. } | Expr::Part { .. } | Expr::Item { .. } | Expr::Var { .. } | Expr::Aggregate { .. } | Expr::Extreme { .. } | Expr::End { .. } | Expr::Other { .. } | Expr::Has { .. } | Expr::Pick { .. } => "…".into(),
        Expr::Const { .. } => "π".into(),
        Expr::Origin { .. } => "origin".into(),
        Expr::Der { der } => format!("der({})", symbol(cm, der)),
        Expr::Neg { neg } => format!("-{}", wrap(neg, 6)),
        Expr::Not { not } => format!("not {}", wrap(not, 6)),
        Expr::Norm { norm } => format!("|{}|", p(norm)),
        Expr::Bin { bin, l, r } => {
            let me = prec(e);
            let (ls, rs) = match bin {
                BinOp::Pow => (wrap(l, 8), wrap(r, 8)),
                BinOp::Sub | BinOp::Div => (wrap(l, me), wrap(r, me + 1)),
                _ => (wrap(l, me), wrap(r, me)),
            };
            let op = match bin {
                BinOp::Add => " + ",
                BinOp::Sub => " - ",
                BinOp::Mul => {
                    // Juxtapose, except between two numbers.
                    if matches!(**r, Expr::Num { .. }) {
                        " * "
                    } else {
                        " "
                    }
                }
                BinOp::Div => " / ",
                BinOp::Pow => "^",
                BinOp::And => " and ",
                BinOp::Or => " or ",
                BinOp::Eq => " == ",
                BinOp::Ne => " != ",
                BinOp::Lt => " < ",
                BinOp::Le => " <= ",
                BinOp::Gt => " > ",
                BinOp::Ge => " >= ",
            };
            format!("{ls}{op}{rs}")
        }
        Expr::Call { call, args } => {
            let name = match call {
                Func::Sin => "sin",
                Func::Cos => "cos",
                Func::Tan => "tan",
                Func::Sqrt => "sqrt",
                Func::Exp => "exp",
                Func::Log => "log",
                Func::Abs => "abs",
                Func::Min => "min",
                Func::Max => "max",
                Func::Atan2 => "atan2",
                Func::Laplacian => "laplacian",
                Func::Gradient => "gradient",
                Func::Integral => "integral",
                Func::Mean => "mean",
            };
            format!("{name}({})", args.iter().map(p).collect::<Vec<_>>().join(", "))
        }
        Expr::Draw { draw, args } => format!("{}({})", draw.name(), args.iter().map(p).collect::<Vec<_>>().join(", ")),
        Expr::Apply { apply, args } => format!("{}({})", wrap(apply, 8), args.iter().map(p).collect::<Vec<_>>().join(", ")),
        Expr::If { r#if, then, r#else } => format!("if {} then {} else {}", p(r#if), p(then), p(r#else)),
        Expr::Tuple { tuple } => format!("({})", tuple.iter().map(p).collect::<Vec<_>>().join(", ")),
        Expr::Comp { comp, axis } => {
            let space = match &**comp {
                Expr::Ref { r#ref } => cm.ir.binding(r#ref).and_then(|b| match &b.ty {
                    Type::Point { space } | Type::Vector { space, .. } => Some(space.clone()),
                    _ => None,
                }),
                _ => cm.ir.default_space.clone(),
            };
            let names = axes(space.as_deref());
            format!("{}.{}", wrap(comp, 8), names.get(*axis).cloned().unwrap_or_else(|| axis.to_string()))
        }
        Expr::Lambda { lambda } => print(&lambda.body, cm, &lambda.names),
        Expr::Otherwise { otherwise, default } => format!("{} otherwise {}", p(otherwise), p(default)),
        Expr::Fn { r#fn } => fn_name(cm, r#fn),
        Expr::Payload { payload } => payload_name(cm, payload),
        Expr::Match { r#match, arms } => {
            format!("match {} {{ {} }}", p(r#match), arms.iter().map(|a| format!("{} => {}", a.case, p(&a.value))).collect::<Vec<_>>().join(", "))
        }
    }
}

/// The name of an event's payload (D-050).
pub fn payload_name(cm: &CModel, event: &str) -> String {
    cm.ir.events.iter().find(|e| e.id == event).and_then(|e| e.payload.as_ref()).map(|p| p.name.clone()).unwrap_or_else(|| "payload".into())
}

/// The name of a declared function (D-048).
pub fn fn_name(cm: &CModel, id: &str) -> String {
    cm.ir.function(id).map(|f| f.name.clone()).unwrap_or_else(|| id.rsplit('.').next().unwrap_or(id).to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn numbers() {
        assert_eq!(fmt_num(4.200000000000001), "4.2");
        assert_eq!(fmt_num(3.605551275463989), "3.60555");
        assert_eq!(fmt_num(-1.0), "-1");
        assert_eq!(fmt_num(1e-9), "1e-9");
        assert_eq!(fmt_num(40.7747196738022), "40.7747");
        assert_eq!(components(&[4.0, 4.440892098500626e-16]), [4.0, 0.0]);
        assert_eq!(components(&[1e-12, 2e-12]), [1e-12, 2e-12]);
        assert_eq!(unit_text(&Dim::parse("L/T").unwrap()), "m/s");
        assert_eq!(superscripts("25.00 m^2"), "25.00 m²");
        assert_eq!(superscripts("(1 m/s^2, 2 m^-10)"), "(1 m/s², 2 m⁻¹⁰)");
        assert_eq!(superscripts("a^b 1e-5"), "a^b 1e-5");
    }
}
