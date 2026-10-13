//! Headless evaluation of run cases and their expectations (PK-4.1, PK-4.2).
//!
//! A case whose presentation has a timeline is played as a lesson (interactive medium, with
//! the case's learner script); its beat expectations read the playback, and its other
//! expectations read the run current at the end of the lesson. Any other case is one run.

use crate::data::{observe, Data};
use crate::timeline::{play, Input, Medium, Playback};
use crate::{config, constant, constant_as, flat, number, Program, LESSON_HORIZON};
use prismal_ir::present::{Check, Item, Operand, Outcome, RunCase, Subject, Tolerance};
use prismal_ir::Id;
use prismal_kernel::CModel;
use prismal_runtime::{run, Category, Run, RunStatus};

#[derive(Clone, Debug, PartialEq)]
pub struct ExpectResult {
    pub id: Id,
    pub pass: bool,
    pub message: String,
}

pub struct CaseReport {
    pub case: Id,
    pub results: Vec<ExpectResult>,
    /// The run the expectations read (for a lesson, the run current at its end).
    pub run: Option<Run>,
    pub playback: Option<Playback>,
}

impl CaseReport {
    pub fn failures(&self) -> Vec<&ExpectResult> {
        self.results.iter().filter(|r| !r.pass).collect()
    }
}

enum Got {
    Values(Vec<f64>),
    Data(Data),
}

struct Cx<'a, 'b> {
    cm: &'a CModel,
    run: &'b Run,
    playback: Option<&'b Playback>,
    pres: Option<&'a prismal_ir::present::Presentation>,
}

impl Cx<'_, '_> {
    /// The type of a subject that is an expression, so that an expected enumeration case
    /// takes its enumeration from it (D-049).
    fn subject_type(&self, s: &Subject) -> Option<prismal_ir::Type> {
        let e = match s {
            Subject::On { expr, .. } => expr,
            Subject::Observation { observation, .. } => {
                let o = self.pres?.observations.iter().find(|o| &o.id == observation)?;
                match &o.source {
                    prismal_ir::present::Source::Expr { expr } => expr,
                    _ => return None,
                }
            }
            _ => return None,
        };
        prismal_kernel::compile_expr(self.cm, e, None).ok().map(|x| x.1)
    }

    fn subject(&self, s: &Subject) -> Result<Got, String> {
        match s {
            Subject::Observation { observation, index } => {
                let o = self.pres.and_then(|p| p.observations.iter().find(|o| &o.id == observation)).ok_or(format!("unknown observation `{observation}`"))?;
                let data = observe(self.cm, self.run, o, &self.run.config.log)?;
                match (data, index) {
                    (Data::Value(v), None) => Ok(Got::Values(flat(&v))),
                    (Data::Series(s), Some(k)) => s.get(k - 1).map(|x| Got::Values(flat(&x.1))).ok_or(format!("`{}` has {} values, not {k}", o.name, s.len())),
                    (Data::Series(s), None) if s.len() == 1 => Ok(Got::Values(flat(&s[0].1))),
                    (Data::Series(s), None) => Err(format!("`{}` is a series of {} values; index it", o.name, s.len())),
                    (Data::Value(_), Some(_)) => Err(format!("`{}` is a single value", o.name)),
                    (d, None) => Ok(Got::Data(d)),
                    (_, Some(_)) => Err(format!("`{}` cannot be indexed", o.name)),
                }
            }
            Subject::On { expr, event, microstep } => {
                let name = self.cm.ir.events.iter().find(|e| &e.id == event).map(|e| e.name.clone()).ok_or("unknown event")?;
                let vals = self.run.on(&name, expr, *microstep);
                vals.first().map(|v| Got::Values(flat(v))).ok_or(format!("`{name}` did not occur"))
            }
            // D-084: the mean or the sample deviation of an observation's values.
            Subject::Statistic { statistic, observation } => {
                let o = self.pres.and_then(|p| p.observations.iter().find(|o| &o.id == observation)).ok_or(format!("unknown observation `{observation}`"))?;
                let xs: Vec<f64> = match observe(self.cm, self.run, o, &self.run.config.log)? {
                    Data::Series(s) => s.iter().map(|(_, v)| match v {
                        prismal_kernel::Value::Num(x) => Ok(*x),
                        prismal_kernel::Value::Bool(b) => Ok(if *b { 1.0 } else { 0.0 }),
                        other => Err(format!("`{}` has a value {other} that is not a number", o.name)),
                    }).collect::<Result<_, _>>()?,
                    Data::Value(prismal_kernel::Value::Num(x)) => vec![x],
                    _ => return Err(format!("`{}` has no numbers to average", o.name)),
                };
                let n = xs.len() as f64;
                let mean = xs.iter().sum::<f64>() / n;
                Ok(Got::Values(vec![match statistic {
                    prismal_ir::present::Statistic::Mean if n >= 1.0 => mean,
                    prismal_ir::present::Statistic::Deviation if n >= 2.0 => (xs.iter().map(|x| (x - mean) * (x - mean)).sum::<f64>() / (n - 1.0)).sqrt(),
                    _ => return Err(format!("`{}` has too few values ({n})", o.name)),
                }]))
            }
            Subject::BeatStart { beat } | Subject::BeatEnd { beat } => {
                let pb = self.playback.ok_or("beat times need a timeline")?;
                let b = pb.beats.iter().find(|b| &b.id == beat).ok_or(format!("beat `{beat}` was not played"))?;
                Ok(Got::Values(vec![if matches!(s, Subject::BeatStart { .. }) { b.start } else { b.end }]))
            }
        }
    }

    fn check(&self, c: &Check) -> Result<(), String> {
        match c {
            Check::Outcome { outcome } => match (&self.run.status, outcome) {
                (RunStatus::NotStarted(d), Outcome::ConfigurationRejected) if d.category == Category::Configuration => Ok(()),
                (RunStatus::NotStarted(d), Outcome::InitializationFails) if d.category != Category::Configuration => Ok(()),
                (s, _) => Err(format!("run status {s:?}")),
            },
            Check::Within { subject, lo, hi, lo_closed, hi_closed } => {
                let Got::Values(v) = self.subject(subject)? else { return Err("not a value".into()) };
                let (a, b) = (number(self.cm, lo)?, number(self.cm, hi)?);
                let x = *v.first().ok_or("no value")?;
                let ok = (if *lo_closed { x >= a } else { x > a }) && (if *hi_closed { x <= b } else { x < b });
                if ok {
                    Ok(())
                } else {
                    Err(format!("{x:e} is outside the interval [{a:e}, {b:e}]"))
                }
            }
            Check::Equal { subject, expected, tolerance } => {
                // A series compared with a list is compared as a whole: `drops == []`.
                if let (Subject::Observation { observation, index: None }, Operand::List { items }) = (subject, expected) {
                    let o = self.pres.and_then(|p| p.observations.iter().find(|o| &o.id == observation)).ok_or(format!("unknown observation `{observation}`"))?;
                    if let Data::Series(series) = observe(self.cm, self.run, o, &self.run.config.log)? {
                        return series_list(self.cm, &series, items, tolerance);
                    }
                }
                let got = self.subject(subject)?;
                match (got, expected) {
                    (Got::Data(d), Operand::List { items }) => list(&d, items),
                    (Got::Data(_), _) => Err("a log is compared with a list".into()),
                    (Got::Values(g), Operand::List { .. }) => Err(format!("{g:?} is not a list")),
                    (Got::Values(g), Operand::Value { expr }) => {
                        let ty = self.subject_type(subject).filter(|t| matches!(t, prismal_ir::Type::Enum { .. }));
                        let mut w = flat(&constant_as(self.cm, expr, ty.as_ref())?);
                        if w == [0.0] && g.len() > 1 {
                            w = vec![0.0; g.len()]; // `0` is the zero vector (D-030)
                        }
                        compare(self.cm, &g, &w, tolerance)
                    }
                    (Got::Values(g), Operand::Subject { subject }) => match self.subject(subject)? {
                        Got::Values(w) => compare(self.cm, &g, &w, tolerance),
                        Got::Data(_) => Err("a value is compared with a log".into()),
                    },
                }
            }
        }
    }
}

/// A series of values against a list of expected values, element by element.
fn series_list(cm: &CModel, series: &[(f64, prismal_kernel::Value)], items: &[Item], tol: &Tolerance) -> Result<(), String> {
    if series.len() != items.len() {
        return Err(format!("{} values, expected {}", series.len(), items.len()));
    }
    for ((_, v), item) in series.iter().zip(items) {
        let Item::Value { expr } = item else { return Err("a series is compared with a list of values".into()) };
        compare(cm, &flat(v), &flat(&constant(cm, expr)?), tol)?;
    }
    Ok(())
}

fn list(d: &Data, items: &[Item]) -> Result<(), String> {
    match d {
        Data::Events(evs) => {
            let want: Vec<&Id> = items
                .iter()
                .map(|i| match i {
                    Item::Event { event } => Ok(event),
                    Item::Value { .. } => Err("an event log is compared with a list of events".to_string()),
                })
                .collect::<Result<_, _>>()?;
            let got: Vec<&Id> = evs.iter().map(|e| &e.event).collect();
            if got == want {
                Ok(())
            } else {
                Err(format!("events {:?}", evs.iter().map(|e| e.name.as_str()).collect::<Vec<_>>()))
            }
        }
        Data::Diagnostics(ds) if items.is_empty() => {
            if ds.is_empty() {
                Ok(())
            } else {
                Err(format!("{} diagnostics, first: {}", ds.len(), ds[0].message))
            }
        }
        Data::Interventions(is) if items.is_empty() => {
            if is.is_empty() {
                Ok(())
            } else {
                Err(format!("{} interventions", is.len()))
            }
        }
        _ => Err("only an empty list is compared with diagnostics or interventions".into()),
    }
}

fn compare(cm: &CModel, g: &[f64], w: &[f64], tol: &Tolerance) -> Result<(), String> {
    if g.len() != w.len() {
        return Err(format!("{g:?} and {w:?} differ in shape"));
    }
    for (a, b) in g.iter().zip(w) {
        let ok = match tol {
            Tolerance::Exact => a.to_bits() == b.to_bits() || (*a == 0.0 && *b == 0.0),
            Tolerance::Abs { tol } => (a - b).abs() <= number(cm, tol)?,
            Tolerance::Rel { tol } => ((a - b) / b).abs() <= number(cm, tol)?,
        };
        if !ok {
            return Err(format!("got {a:.15e}, expected {b:.15e} (difference {:e})", (a - b).abs()));
        }
    }
    Ok(())
}

/// Runs a case and evaluates its expectations.
pub fn run_case(prog: &Program, case: &RunCase) -> Result<CaseReport, String> {
    // A run that fills a collection declared without a limit runs again with room for twice
    // as many members (D-066); lessons grow in `play`.
    let mut grown: Option<Program> = None;
    loop {
        let p = grown.as_ref().unwrap_or(prog);
        let report = run_case_once(p, case)?;
        match report.run.as_ref().and_then(|r| p.overflow(r)).and_then(|part| p.grown(&part)) {
            Some(next) if report.playback.is_none() => grown = Some(next),
            _ => return Ok(report),
        }
    }
}

fn run_case_once(prog: &Program, case: &RunCase) -> Result<CaseReport, String> {
    let cm = prog.model(&case.model);
    let pres = case.presentation.as_ref().map(|p| prog.presentation(p));
    let lesson = pres.is_some_and(|p| p.timeline.is_some());
    let cfg = config(cm, case, if lesson { LESSON_HORIZON } else { 0.0 })?;
    let (the_run, playback) = if lesson {
        let inputs = case.learner.iter().map(|s| Ok(Input { at: number(cm, &s.at)?, input: s.input.clone() })).collect::<Result<Vec<_>, String>>()?;
        let pb = play(prog, &pres.unwrap().id, cfg, Medium::Interactive, inputs).map_err(|d| d.iter().map(|x| x.to_string()).collect::<Vec<_>>().join("; "))?;
        (pb.runs[pb.current].run.clone(), Some(pb))
    } else {
        (run(cm, cfg), None)
    };
    let cx = Cx { cm, run: &the_run, playback: playback.as_ref(), pres };
    let results = case
        .expectations
        .iter()
        .map(|e| match cx.check(&e.check) {
            Ok(()) => ExpectResult { id: e.id.clone(), pass: true, message: String::new() },
            Err(m) => ExpectResult { id: e.id.clone(), pass: false, message: m },
        })
        .collect();
    Ok(CaseReport { case: case.id.clone(), results, run: Some(the_run), playback })
}
