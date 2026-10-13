//! Observations of a run (PK section 3). Observing never changes the run (PK-3.2).

use crate::number;
use crate::measure::meter;
use prismal_ir::present::{Instrument, Observation, Schedule, Source};
use prismal_ir::Id;
use prismal_kernel::{CModel, Value};
use prismal_runtime::{Run, RunDiag, Scheduled};

/// An event occurrence (RC-15.1).
#[derive(Clone, Debug, PartialEq)]
pub struct EventRec {
    pub event: Id,
    pub name: String,
    pub t: f64,
    pub n: u32,
    pub zeno: bool,
    /// The occurrence's payload (D-050).
    pub payload: Option<prismal_kernel::Value>,
}

/// The data an observation produces (PK-3.5).
#[derive(Clone, Debug)]
pub enum Data {
    Value(Value),
    /// Values with the simulation instants they refer to.
    Series(Vec<(f64, Value)>),
    Events(Vec<EventRec>),
    Diagnostics(Vec<RunDiag>),
    Interventions(Vec<Scheduled>),
}

impl Data {
    pub fn len(&self) -> usize {
        match self {
            Data::Value(_) => 1,
            Data::Series(s) => s.len(),
            Data::Events(e) => e.len(),
            Data::Diagnostics(d) => d.len(),
            Data::Interventions(i) => i.len(),
        }
    }
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

/// The event log of a run as records.
pub fn events(cm: &CModel, run: &Run) -> Vec<EventRec> {
    run.log
        .iter()
        .map(|l| EventRec { event: cm.events[l.event].id.clone(), name: l.name.clone(), t: l.t, n: l.n, zeno: l.zeno, payload: l.payload.clone() })
        .collect()
}

fn window(cm: &CModel, run: &Run, schedule: &Schedule) -> Result<Option<(f64, f64, bool, bool)>, String> {
    Ok(match schedule {
        Schedule::Over { from, to, lo_closed, hi_closed } => {
            let a = number(cm, from)?;
            let b = match to {
                Some(e) => number(cm, e)?,
                None => run.end_time(),
            };
            Some((a, b, *lo_closed, *hi_closed))
        }
        _ => None,
    })
}

fn inside(t: f64, w: Option<(f64, f64, bool, bool)>) -> bool {
    match w {
        None => true,
        Some((a, b, lc, hc)) => (if lc { t >= a } else { t > a }) && (if hc { t <= b } else { t < b }),
    }
}

/// The largest number of runs an ensemble takes (D-084).
pub const MAX_RUNS: usize = 10_000;

/// The value of `expr` at instant `t` in each of `n` runs of `run`'s model and
/// configuration, with the seeds `seed`, `seed + 1`, ... from `run`'s own (D-084): the first
/// is `run` itself. Each run ends at `t`, so the actions logged after it do not matter.
pub fn ensemble(run: &Run, expr: &prismal_ir::Expr, t: f64, n: usize) -> Result<Vec<Value>, String> {
    if n == 0 || n > MAX_RUNS {
        return Err(format!("an ensemble takes from 1 to {MAX_RUNS} runs, not {n}"));
    }
    if t < run.config.t0 {
        return Err(format!("instant {t} s is before the start of the run"));
    }
    let c = run.compile(expr);
    let mut out = Vec::with_capacity(n);
    for k in 0..n {
        let mut cfg = run.config.clone();
        cfg.seed = run.config.seed.wrapping_add(k as u64);
        cfg.t_end = t;
        let r = prismal_runtime::run(&run.model, cfg);
        if let prismal_runtime::RunStatus::NotStarted(d) | prismal_runtime::RunStatus::Stopped(d) = &r.status {
            return Err(format!("run with seed {}: {}", r.config.seed, d.message));
        }
        out.push(r.eval_state(&c, &r.state_at(t), t).map_err(|s| s.cause)?);
    }
    Ok(out)
}

/// Observes a run. `interventions` is the committed intervention log of the run's session;
/// `instruments` the presentation's, through which an observation may read (PK-3.7).
pub fn observe(cm: &CModel, run: &Run, obs: &Observation, interventions: &[Scheduled], instruments: &[Instrument]) -> Result<Data, String> {
    let data = observe_exact(cm, run, obs, interventions)?;
    let Source::Expr { expr } = &obs.source else { return Ok(data) };
    let ty = prismal_kernel::compile_expr(cm, expr, None).map_err(|d| format!("{d:?}"))?.1;
    let Some(m) = meter(cm, instruments, obs.instrument.as_ref(), expr, &ty)? else { return Ok(data) };
    let seed = run.config.seed;
    Ok(match (data, &obs.schedule) {
        (_, Schedule::Live) => return Err(format!("instrument `{}` reads at instants: observe `every`, `at`, `on` or `over`, not `live`", m.name)),
        // An ensemble's k-th value comes from the run with the k-th seed (PK-3.8).
        (Data::Series(s), Schedule::At { runs: Some(_), .. }) => {
            Data::Series(s.into_iter().enumerate().filter_map(|(k, (t, v))| m.read(&v, seed.wrapping_add(k as u64), t).map(|r| (t, r))).collect())
        }
        // A reading outside the range is not available: a series leaves it out.
        (Data::Series(s), _) => Data::Series(s.into_iter().filter_map(|(t, v)| m.read(&v, seed, t).map(|r| (t, r))).collect()),
        (Data::Value(v), Schedule::At { time, .. }) => {
            let t = crate::number(cm, time)?;
            Data::Value(m.read(&v, seed, t).ok_or(format!("{v} at {} s is outside the range of instrument `{}`", t - run.config.t0, m.name))?)
        }
        (d, _) => d,
    })
}

fn observe_exact(cm: &CModel, run: &Run, obs: &Observation, interventions: &[Scheduled]) -> Result<Data, String> {
    let w = window(cm, run, &obs.schedule)?;
    Ok(match &obs.source {
        Source::EventLog { event, zeno_only } => Data::Events(
            events(cm, run)
                .into_iter()
                .filter(|e| event.as_ref().is_none_or(|x| *x == e.event) && (!zeno_only || e.zeno) && inside(e.t, w))
                .collect(),
        ),
        Source::Diagnostics { element } => Data::Diagnostics(
            run.diagnostics
                .iter()
                .filter(|d| element.as_ref().is_none_or(|x| d.element.as_ref() == Some(x)) && inside(d.t, w))
                .cloned()
                .collect(),
        ),
        Source::InterventionLog => Data::Interventions(interventions.iter().filter(|s| inside(s.t, w)).cloned().collect()),
        Source::Expr { expr } => {
            let c = run.compile(expr);
            let eval = |vals: &[Value], t: f64| run.eval_state(&c, vals, t).map_err(|s| s.cause);
            match &obs.schedule {
                Schedule::Live => {
                    let t = run.end_time();
                    Data::Value(eval(&run.state_at(t), t)?)
                }
                Schedule::At { time, runs: Some(n) } => {
                    let t = number(cm, time)?;
                    let n = number(cm, n)?;
                    if n.fract() != 0.0 || n < 1.0 {
                        return Err(format!("a number of runs is a whole number from 1, not {n}"));
                    }
                    Data::Series(ensemble(run, expr, t, n as usize)?.into_iter().map(|v| (t, v)).collect())
                }
                Schedule::At { time, runs: None } => {
                    let t = number(cm, time)?;
                    if t > run.end_time() {
                        // RC-12.4: a model without flows or timed events has one instant.
                        let why = if run.model.is_static { " (a static model, with no flows and no events but requests, has a single instant, RC-12.4)" } else { "" };
                        return Err(format!("instant {t} s is after the end of the run{why}"));
                    }
                    Data::Value(eval(&run.state_at(t), t)?)
                }
                Schedule::Every { period } => {
                    let dt = number(cm, period)?;
                    Data::Series(run.every(dt, expr))
                }
                Schedule::On { event, microstep } => {
                    let name = cm.ir.events.iter().find(|e| &e.id == event).map(|e| e.name.clone()).ok_or("unknown event")?;
                    let times = run.times(&name);
                    Data::Series(times.into_iter().zip(run.on(&name, expr, *microstep)).collect())
                }
                Schedule::Over { .. } | Schedule::Run => {
                    let mut out = vec![];
                    for c in run.committed.iter().filter(|c| inside(c.t, w)) {
                        out.push((c.t, eval(&c.vals, c.t)?));
                    }
                    Data::Series(out)
                }
            }
        }
    })
}
