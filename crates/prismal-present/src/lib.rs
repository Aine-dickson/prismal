//! Prismal presentation kernel (docs/spec/03-presentation-kernel.md).
//!
//! A presentation observes a model's runs, projects them into representations arranged in
//! views, lets a learner act on the model through declared inverses, and directs a lesson
//! run through an explanation timeline. Every presented instant is described by a
//! medium-independent frame description (PK-12.1), which a renderer turns into output.
//!
//! - `check`: static checks of a presentation against its model (PK-E diagnostics).
//! - `data`: observations of a run (PK section 3).
//! - `expect`: headless evaluation of run cases and their expectations (PK section 4).
//! - `frame`: projection and frame descriptions (PK sections 5 to 7, 11, 12).
//! - `interact`: controls, direct manipulation, keyboard operation, undo (PK section 10).
//! - `timeline`: the explanation timeline and its lesson run (PK sections 8, 9).
//! - `text`: text alternatives and formula text (PK-6.5, PK-11.1).

pub mod check;
pub mod data;
pub mod expect;
pub mod frame;
pub mod interact;
pub mod math;
pub mod measure;
pub mod text;
pub mod timeline;

use prismal_ir::present::{Presentation, RunCase};
use prismal_ir::{Document, Expr, Id};
use prismal_kernel::{check_model, compile_expr, CModel, Ctx, Value};
use prismal_runtime::{Config, SolverConfig};

/// A diagnostic about a presentation or a run case.
#[derive(Clone, Debug, PartialEq)]
pub struct PDiag {
    pub code: &'static str,
    pub message: String,
    /// The IR element concerned.
    pub element: Id,
}

impl std::fmt::Display for PDiag {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{} at {}: {}", self.code, self.element, self.message)
    }
}

/// A checked document: its models compiled, its presentations checked.
pub struct Program {
    pub doc: Document,
    pub models: Vec<CModel>,
    /// The document as written, before elaboration, and the working capacities of its
    /// collections declared without a limit (D-066).
    pub source: Document,
    pub capacities: prismal_ir::elaborate::Capacities,
}

impl Program {
    /// Checks every model with the kernel and every presentation against its model.
    pub fn new(doc: Document) -> Result<Program, Vec<PDiag>> {
        Program::sized(doc, Default::default())
    }

    /// [`Program::new`] with working capacities for collections declared without a limit.
    pub fn sized(source: Document, capacities: prismal_ir::elaborate::Capacities) -> Result<Program, Vec<PDiag>> {
        // Objects and collections become a flat model first (D-055).
        let doc = prismal_ir::elaborate::document_with(&source, &capacities).map_err(|ds| ds.into_iter().map(|d| PDiag { code: d.code, message: d.message, element: d.element }).collect::<Vec<_>>())?;
        let mut diags = vec![];
        let mut models = vec![];
        for m in &doc.models {
            match check_model(&doc.spaces, m) {
                Ok(cm) => models.push(cm),
                Err(ds) => diags.extend(ds.into_iter().map(|d| PDiag { code: d.code, message: d.message, element: d.element })),
            }
        }
        if !diags.is_empty() {
            return Err(diags);
        }
        let prog = Program { doc, models, source, capacities };
        for p in &prog.doc.presentations {
            diags.extend(check::check_presentation(prog.model(&p.model), p));
        }
        if diags.is_empty() {
            Ok(prog)
        } else {
            Err(diags)
        }
    }

    /// The collection declared without a limit whose working capacity `run` ran out of
    /// (D-066): the run stopped at its capacity constraint.
    pub fn overflow(&self, run: &prismal_runtime::Run) -> Option<prismal_ir::Id> {
        overflow(&self.source, run)
    }

    /// The program with the working capacity of `part` doubled (D-066); `None` past
    /// [`prismal_ir::elaborate::MAX_CAPACITY`] or when it does not check.
    pub fn grown(&self, part: &str) -> Option<Program> {
        Program::sized(self.source.clone(), grown_capacities(&self.source, &self.capacities, part)?).ok()
    }

    pub fn model(&self, id: &str) -> &CModel {
        self.models.iter().find(|m| m.ir.id == id).unwrap_or_else(|| panic!("unknown model {id}"))
    }

    pub fn presentation(&self, name: &str) -> &Presentation {
        self.doc.presentations.iter().find(|p| p.name == name || p.id == name).unwrap_or_else(|| panic!("unknown presentation {name}"))
    }

    pub fn case(&self, name: &str) -> &RunCase {
        self.doc.runs.iter().find(|r| r.name == name || r.id == name).unwrap_or_else(|| panic!("unknown run {name}"))
    }
}

/// The collection declared without a limit in `source` whose working capacity `run` ran out
/// of (D-066).
pub fn overflow(source: &Document, run: &prismal_runtime::Run) -> Option<prismal_ir::Id> {
    match &run.status {
        prismal_runtime::RunStatus::Stopped(d) if d.category == prismal_runtime::Category::Constraint => prismal_ir::elaborate::overflowed(source, d.element.as_deref()?),
        _ => None,
    }
}

/// `caps` with the working capacity of `part` doubled; `None` past
/// [`prismal_ir::elaborate::MAX_CAPACITY`] or for a part not declared without a limit.
pub fn grown_capacities(source: &Document, caps: &prismal_ir::elaborate::Capacities, part: &str) -> Option<prismal_ir::elaborate::Capacities> {
    let declared = source.models.iter().flat_map(|m| std::iter::once(m).chain(&m.objects)).flat_map(|m| &m.parts).find(|p| p.id == part && p.unbounded)?;
    let now = caps.get(part).copied().unwrap_or_else(|| prismal_ir::elaborate::starting_capacity(source, declared));
    if now >= prismal_ir::elaborate::MAX_CAPACITY {
        return None;
    }
    let mut out = caps.clone();
    out.insert(part.to_string(), (2 * now).min(prismal_ir::elaborate::MAX_CAPACITY));
    Some(out)
}

/// Evaluates an expression that reads no state (a literal, a time such as `t0 + 5 s`).
/// Instants are seconds with `t0 = 0`.
pub fn constant(cm: &CModel, e: &Expr) -> Result<Value, String> {
    constant_as(cm, e, None)
}

/// [`constant`], with the type the expression is expected to have (an enumeration case
/// takes its enumeration from it, D-049).
pub fn constant_as(cm: &CModel, e: &Expr, expected: Option<&prismal_ir::Type>) -> Result<Value, String> {
    let mut reads = false;
    e.walk(&mut |x| reads |= matches!(x, Expr::Ref { .. } | Expr::Der { .. }));
    if reads {
        return Err("the expression reads bindings of the model; it is not a constant".into());
    }
    let (c, _) = compile_expr(cm, e, expected).map_err(|d| d.iter().map(|x| x.to_string()).collect::<Vec<_>>().join("; "))?;
    c.eval(&Ctx { vals: &cm.constant_values(), der: None, t: 0.0, t0: 0.0, args: &[], payloads: &[], draws: None }).map_err(|s| s.cause)
}

/// A constant number (a quantity in coherent SI units, or an instant in seconds).
pub fn number(cm: &CModel, e: &Expr) -> Result<f64, String> {
    match constant(cm, e)? {
        Value::Num(x) => Ok(x),
        v => Err(format!("{v} is not a number")),
    }
}

/// The components of a value, for comparison: numbers, coordinates, Booleans as 0 and 1.
pub fn flat(v: &Value) -> Vec<f64> {
    match v {
        Value::Bool(b) => vec![if *b { 1.0 } else { 0.0 }],
        Value::Num(x) => vec![*x],
        Value::Vec(a) | Value::Point(a) => a.as_slice().to_vec(),
        Value::Tuple(items) => items.iter().flat_map(flat).collect(),
        Value::Case(c) => vec![*c as f64],
        Value::Func(_) => vec![],
    }
}

/// Default simulated span of a lesson run whose case gives no end (PK-9.5).
pub const LESSON_HORIZON: f64 = 60.0;

/// The configuration of a run started again from `t0` with the parameter values `run` has
/// at `t` (D-085): every parameter becomes an override of its current value, and the action
/// log is emptied. What else the learner or the lesson did to the run is not kept.
pub fn restarted(run: &prismal_runtime::Run, t: f64) -> Config {
    let cm = &run.model;
    let vals = run.state_at(t.min(run.end_time()));
    let mut cfg = run.config.clone();
    cfg.log.clear();
    for (i, b) in cm.bindings.iter().enumerate() {
        if b.role != prismal_ir::Role::Parameter {
            continue;
        }
        let e = match (&vals[i], &b.ty) {
            (Value::Num(x), prismal_ir::Type::Quantity { dim }) if dim.is_none() => prismal_ir::build::lit(*x),
            (Value::Num(x), prismal_ir::Type::Quantity { dim }) => Expr::Num { num: *x, unit: Some(prismal_ir::Unit { text: text::unit_text(dim), scale: 1.0, offset: 0.0, dim: *dim }) },
            (Value::Bool(v), _) => Expr::Bool { bool: *v },
            (Value::Case(c), prismal_ir::Type::Enum { cases, .. }) => Expr::Case { case: cases[*c as usize].clone() },
            _ => continue,
        };
        cfg.params.retain(|(id, _)| id != &b.id);
        cfg.params.push((b.id.clone(), e));
    }
    cfg
}

/// The run configuration of a case (RC section 16 defaults, overridden by the case).
pub fn config(cm: &CModel, case: &RunCase, default_end: f64) -> Result<Config, String> {
    let end = match &case.end {
        Some(e) => number(cm, e)?,
        None => default_end,
    };
    let mut cfg = Config::until(end);
    let get = |e: &Option<Expr>| e.as_ref().map(|x| number(cm, x)).transpose();
    let (h, rtol, atol) = (get(&case.config.h)?, get(&case.config.rtol)?, get(&case.config.atol)?);
    cfg.solver = match case.config.solver.as_deref() {
        Some("rk4") => SolverConfig::Rk4 { h },
        _ => SolverConfig::Dopri5 { rtol: rtol.unwrap_or(1e-6), atol: atol.unwrap_or(1e-9), h_max: f64::INFINITY },
    };
    cfg.seed = case.config.seed.unwrap_or(0);
    for o in &case.params {
        cfg = cfg.param(&o.binding, o.value.clone());
    }
    // D-051: an input's value from the start is its starting value; one at an instant is a
    // logged change.
    for i in &case.inputs {
        cfg = match &i.at {
            None => cfg.param(&i.binding, i.value.clone()),
            Some(at) => cfg.at(number(cm, at)?, prismal_runtime::Action::Input(i.binding.clone(), i.value.clone())),
        };
    }
    Ok(cfg)
}
