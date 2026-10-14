//! Runs (RC sections 3 to 11): initialization, continuous evolution, event detection and
//! location, event iteration, Zeno detection, failures, interventions and requests.

use crate::solver::{Attempt, Dense, Dopri5, Rhs, Rk4, Step};
use prismal_ir::{Expr, Id, Op, Policy, Role, Type};
use prismal_kernel::random::{fnv1a, Uniforms, CLOCK_SITE};
use prismal_kernel::{check_intervention, compile_expr, distance, CExpr, CModel, COp, CTrigger, Ctx, Dir, Draws, Status, Value};
use std::collections::VecDeque;

#[derive(Clone, Debug, PartialEq)]
pub enum SolverConfig {
    Dopri5 { rtol: f64, atol: f64, h_max: f64 },
    /// `h` MUST be set for a fixed-step solver (RC section 16); `None` is rejected.
    Rk4 { h: Option<f64> },
}

/// A logged action from outside the model (RC section 11).
#[derive(Clone, Debug, PartialEq)]
pub enum Action {
    /// `set` operations on intervenable bindings (MK-17.2).
    Intervene(Vec<Op>),
    /// A request for an `on request` event (MK-17.2a, D-027).
    Request(Id),
    /// A request that supplies the event's payload (D-050); the expression is evaluated at
    /// the request's instant against the payload's type.
    RequestWith(Id, Expr),
    /// The environment supplies a new value of an input binding (RC-11.6, D-051): the
    /// expression is evaluated at the instant against the input's type, and `on input(x)`
    /// events are due.
    Input(Id, Expr),
}

#[derive(Clone, Debug, PartialEq)]
pub struct Scheduled {
    pub t: f64,
    pub action: Action,
}

/// Run configuration (RC-3.1, RC section 16).
#[derive(Clone, Debug, PartialEq)]
pub struct Config {
    pub t0: f64,
    pub t_end: f64,
    pub solver: SolverConfig,
    pub t_ref: f64,
    pub eps_t: Option<f64>,
    pub n_micro: u32,
    pub eps_zeno: Option<f64>,
    pub n_zeno: u32,
    pub w_zeno: Option<f64>,
    /// Parameter overrides by binding identity (MK-6.8).
    pub params: Vec<(Id, Expr)>,
    /// The input and intervention log (RC-3.1).
    pub log: Vec<Scheduled>,
    /// The seed of every random stream (RC-13.1, D-083).
    pub seed: u64,
    /// The step of the Euler-Maruyama solver of a model with noise in its flows (RC-13.9);
    /// `None`: the fixed step `h` of `rk4` if one is given, else `t_ref / 1000`.
    pub h_noise: Option<f64>,
}

impl Config {
    /// Defaults of RC section 16, running to `t_end` seconds after `t0 = 0`.
    pub fn until(t_end: f64) -> Config {
        Config {
            t0: 0.0,
            t_end,
            solver: SolverConfig::Dopri5 { rtol: 1e-6, atol: 1e-9, h_max: f64::INFINITY },
            t_ref: 1.0,
            eps_t: None,
            n_micro: 100,
            eps_zeno: None,
            n_zeno: 1000,
            w_zeno: None,
            params: vec![],
            log: vec![],
            seed: 0,
            h_noise: None,
        }
    }
    pub fn param(mut self, id: &str, value: Expr) -> Config {
        self.params.push((id.into(), value));
        self
    }
    pub fn solver(mut self, s: SolverConfig) -> Config {
        self.solver = s;
        self
    }
    pub fn seed(mut self, seed: u64) -> Config {
        self.seed = seed;
        self
    }
    pub fn at(mut self, t: f64, action: Action) -> Config {
        self.log.push(Scheduled { t, action });
        self
    }
    fn eps_t(&self) -> f64 {
        self.eps_t.unwrap_or(1e-10 * self.t_ref)
    }
    /// The step of the Euler-Maruyama solver (RC-13.9).
    pub fn noise_step(&self) -> f64 {
        let fixed = match self.solver {
            SolverConfig::Rk4 { h } => h,
            _ => None,
        };
        self.h_noise.or(fixed).unwrap_or(1e-3 * self.t_ref)
    }
}

/// Failure categories (RC-10.1) and reported diagnostics.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Category {
    Configuration,
    Initialization,
    Model,
    Constraint,
    Equation,
    Conflict,
    Cascade,
    Zeno,
    Computational,
    Intervention,
}

#[derive(Clone, Debug, PartialEq)]
pub struct RunDiag {
    pub category: Category,
    pub message: String,
    pub element: Option<Id>,
    pub t: f64,
    pub n: u32,
}

#[derive(Clone, Debug, PartialEq)]
pub enum RunStatus {
    Completed,
    /// Stopped by a failure policy (RC-3.3), with the diagnostic that caused it.
    Stopped(RunDiag),
    /// Initialization or configuration failed; the run did not start (RC-5.2).
    NotStarted(RunDiag),
}

#[derive(Clone, Debug)]
pub struct LogEntry {
    pub event: usize,
    pub name: String,
    pub t: f64,
    pub n: u32,
    /// True when a Zeno policy replaced the handler (RC-15.1).
    pub zeno: bool,
    pub requested: bool,
    /// The occurrence's payload (D-050, RC-15.1).
    pub payload: Option<Value>,
}

/// A committed superdense state (RC-4.2).
#[derive(Clone, Debug)]
pub struct Committed {
    pub t: f64,
    pub n: u32,
    pub vals: Vec<Value>,
    /// Made by a logged action (an intervention or a request) and what it caused, not by the
    /// model's own iteration: an observation `on E` stops before it (PK-3.1).
    pub by_action: bool,
}

/// An accepted step and the committed state whose discrete values hold during it.
#[derive(Clone, Debug)]
pub struct Segment {
    pub step: Step,
    pub base: usize,
}

/// The result of a run: trajectory, event log, diagnostics and status (RC section 15).
#[derive(Clone, Debug)]
pub struct Run {
    pub model: CModel,
    pub config: Config,
    pub committed: Vec<Committed>,
    pub segments: Vec<Segment>,
    pub log: Vec<LogEntry>,
    pub diagnostics: Vec<RunDiag>,
    /// Interventions rejected by validation (RC-10.5); not run failures.
    pub rejected: Vec<RunDiag>,
    pub status: RunStatus,
    /// Snapshots taken during the run (RC section 14.1), in time order: at its start, at
    /// every instant where logged actions apply, and every `SNAPSHOT_STEPS` steps between.
    pub snapshots: Vec<Snapshot>,
}

/// Steps between snapshots taken for resuming (RC-14.3: their instants do not affect
/// results).
pub const SNAPSHOT_STEPS: usize = 8;

/// Above this many continuous state components a run stores its steps compactly (RC-6.2a).
pub const LARGE_STATE: usize = 4096;

/// Everything needed to continue a run exactly from the top of its evolution loop at a
/// committed instant (RC-14.1): the state, the event machinery, the solver's proposed next
/// step, and how much of the trajectory, logs and action list lies before it.
#[derive(Clone, Debug)]
pub struct Snapshot {
    pub t: f64,
    n: u32,
    vals: Vec<Value>,
    refs: Vec<Option<bool>>,
    zeno: Vec<ZenoMonitor>,
    time_next: Vec<Option<f64>>,
    every_k: Vec<u64>,
    payloads: Vec<Option<Value>>,
    due_k: Vec<u64>,
    clocks: Vec<Option<Clock>>,
    h: Option<f64>,
    /// Lengths of the run's records at the snapshot.
    committed: usize,
    segments: usize,
    log: usize,
    diagnostics: usize,
    rejected: usize,
    /// Logged actions, in time order, applied before the snapshot.
    consumed: usize,
    /// No step before the snapshot depended on a stop later than this instant: an action
    /// logged after `theta` (and after `t`) leaves the trajectory before the snapshot as it
    /// is (RC-14.2).
    pub theta: f64,
}

/// The clock of a random event (RC-13.8, the modified next reaction method): the integral
/// of its rate up to `t_last`, the rate since then, and the integral at which it occurs next.
#[derive(Clone, Debug)]
struct Clock {
    rate: f64,
    integral: f64,
    t_last: f64,
    target: f64,
    fired: u64,
}

#[derive(Clone, Debug)]
struct ZenoMonitor {
    last: Option<f64>,
    recent: VecDeque<f64>,
}

struct Engine<'a> {
    cm: &'a CModel,
    cfg: &'a Config,
    vals: Vec<Value>,
    t: f64,
    n: u32,
    refs: Vec<Option<bool>>,
    zeno: Vec<ZenoMonitor>,
    /// Next instant of each time event (RC-7.9); `every_k` counts `every` occurrences.
    time_next: Vec<Option<f64>>,
    every_k: Vec<u64>,
    /// Payloads of the occurrences handled at the current microstep, by event (D-050).
    payloads: Vec<Option<Value>>,
    /// How many times each event has been due (RC-13.5): the counter of its draws.
    due_k: Vec<u64>,
    clocks: Vec<Option<Clock>>,
    run: Run,
    /// The solver's proposed next step (`None` after an event or an action, RC-8.6).
    h: Option<f64>,
    /// Logged actions applied so far, in time order.
    consumed: usize,
    /// The latest stop any step so far depended on (see `Snapshot::theta`).
    theta: f64,
    /// While a logged action applies: the states it commits are marked `by_action`.
    acting: bool,
}

/// The right-hand side over one segment: discrete state and parameters fixed. `noise`: the
/// values of the white noises during an Euler-Maruyama step (D-090).
struct SegmentRhs<'a> {
    cm: &'a CModel,
    scratch: Vec<Value>,
    t0: f64,
    noise: Vec<f64>,
}

impl Rhs for SegmentRhs<'_> {
    fn f(&mut self, t: f64, y: &[f64], dy: &mut [f64]) -> Result<(), Status> {
        self.cm.load_y(&mut self.scratch, y);
        self.cm.update_for_flows(&mut self.scratch, t, self.t0)?;
        self.cm.rhs_noisy(&self.scratch, t, self.t0, &self.noise, dy)
    }
}

fn sign(x: f64) -> Option<bool> {
    if x > 0.0 {
        Some(true)
    } else if x < 0.0 {
        Some(false)
    } else {
        None
    }
}

fn matches_dir(dir: Dir, from: bool, to: bool) -> bool {
    match dir {
        Dir::Rising => !from && to,
        Dir::Falling => from && !to,
        Dir::Either => from != to,
    }
}

/// A conservative bound past the end of a step of size `h` from `t`: a stop beyond it is
/// not reached by the step, whatever rounding the landing test meets.
fn reach(t: f64, h: f64) -> f64 {
    t + h * (1.0 + 1e-9) + 1e-12 * t.abs().max(1.0)
}

/// The smallest instant after `t`.
fn after(t: f64) -> f64 {
    if t == 0.0 {
        f64::from_bits(1)
    } else if t > 0.0 {
        f64::from_bits(t.to_bits() + 1)
    } else {
        f64::from_bits(t.to_bits() - 1)
    }
}

/// The action log in the order a run applies it: by instant, in log order at one instant.
fn sorted_log(log: &[Scheduled]) -> Vec<Scheduled> {
    let mut l = log.to_vec();
    l.sort_by(|a, b| a.t.total_cmp(&b.t));
    l
}

/// Runs a checked model headless with failure policy `stop` (RC-10.4).
pub fn run(cm: &CModel, cfg: Config) -> Run {
    let mut run = Run {
        model: cm.clone(),
        config: cfg.clone(),
        committed: vec![],
        segments: vec![],
        log: vec![],
        diagnostics: vec![],
        rejected: vec![],
        status: RunStatus::Completed,
        snapshots: vec![],
    };
    let fail = |category, message: String| RunDiag { category, message, element: None, t: cfg.t0, n: 0 };
    if let SolverConfig::Rk4 { h: None } = cfg.solver {
        run.status = RunStatus::NotStarted(fail(Category::Configuration, "the fixed-step solver rk4 requires a step h (RC section 16)".into()));
        return run;
    }
    // Parameter overrides are evaluated as constants of the binding's type.
    let mut overrides = vec![];
    for (id, e) in &cfg.params {
        let Some(&i) = cm.index.get(id) else {
            run.status = RunStatus::NotStarted(fail(Category::Configuration, format!("unknown parameter `{id}`")));
            return run;
        };
        let v = compile_expr(cm, e, Some(&cm.bindings[i].ty))
            .map_err(|d| format!("{d:?}"))
            .and_then(|(c, _)| c.eval(&Ctx { vals: &cm.constant_values(), der: None, t: cfg.t0, t0: cfg.t0, args: &[], payloads: &[], draws: None, noise: &[], point: None }).map_err(|s| s.cause));
        match v {
            Ok(v) => overrides.push((i, v)),
            Err(m) => {
                run.status = RunStatus::NotStarted(fail(Category::Configuration, format!("override of `{id}`: {m}")));
                return run;
            }
        }
    }
    let vals = match cm.initial(&overrides, cfg.t0) {
        Ok(v) => v,
        Err(s) => {
            run.status = RunStatus::NotStarted(fail(Category::Initialization, s.cause));
            return run;
        }
    };
    let mut e = Engine {
        cm,
        cfg: &cfg,
        vals,
        t: cfg.t0,
        n: 0,
        refs: vec![None; cm.events.len()],
        zeno: (0..cm.events.len()).map(|_| ZenoMonitor { last: None, recent: VecDeque::new() }).collect(),
        time_next: vec![None; cm.events.len()],
        every_k: vec![0; cm.events.len()],
        payloads: vec![None; cm.events.len()],
        due_k: vec![0; cm.events.len()],
        clocks: cm
            .events
            .iter()
            .map(|ev| match ev.trigger {
                Some(CTrigger::Random { .. }) => {
                    Some(Clock { rate: 0.0, integral: 0.0, t_last: cfg.t0, target: Uniforms::new(cfg.seed, fnv1a(&ev.id), CLOCK_SITE, 0).exp1(), fired: 0 })
                }
                _ => None,
            })
            .collect(),
        run,
        h: None,
        consumed: 0,
        theta: f64::NEG_INFINITY,
        acting: false,
    };
    if let Err(d) = e.schedule_time_events().and_then(|_| e.refresh_clocks()) {
        e.run.status = RunStatus::NotStarted(RunDiag { category: Category::Initialization, ..d });
        return e.run;
    }
    // RC-5.2: constraints on the initial state.
    match e.check_constraints(&e.vals.clone(), true) {
        Ok(reports) => e.run.diagnostics.extend(reports),
        Err(d) => {
            e.run.status = RunStatus::NotStarted(RunDiag { category: Category::Initialization, ..d });
            return e.run;
        }
    }
    e.commit();
    // RC-5.3: `on start` and time events at t0 are due at (t0, 0).
    let mut start: Vec<usize> = cm.events.iter().enumerate().filter(|(_, ev)| matches!(ev.trigger, Some(CTrigger::Start))).map(|(i, _)| i).collect();
    start.extend(e.take_time_events());
    if let Err(d) = e.iterate(start, vec![]).and_then(|_| e.refresh_clocks()) {
        e.run.status = RunStatus::Stopped(d);
        return e.run;
    }
    e.retake_refs();
    if let Err(d) = e.advance() {
        e.run.status = RunStatus::Stopped(d);
    }
    e.run
}

/// Runs `cfg` on the model of `prev`, a run whose configuration differs from `cfg` at most
/// in its action log, continuing from the latest of `prev`'s snapshots that the change of
/// log leaves as it was (RC-14.2, RC-14.3). The result is the run `run` would compute, bit
/// for bit; only the work before the first changed action is saved. Interventions, drag
/// previews, undo and redo in a session resume this way instead of recomputing from `t0`.
pub fn resume(prev: &Run, cfg: Config) -> Run {
    let mut same = prev.config.clone();
    same.log = cfg.log.clone();
    if same != cfg || prev.model.is_static || matches!(prev.status, RunStatus::NotStarted(_)) {
        return run(&prev.model, cfg);
    }
    // The first action at which the two logs, in the order runs apply them, differ.
    let (old, new) = (sorted_log(&prev.config.log), sorted_log(&cfg.log));
    let p = old.iter().zip(&new).take_while(|(a, b)| a == b).count();
    let t_d = old.get(p).into_iter().chain(new.get(p)).map(|s| s.t).fold(f64::INFINITY, f64::min);
    if t_d == f64::INFINITY {
        let mut r = prev.clone();
        r.config = cfg;
        return r;
    }
    let Some(k) = prev.snapshots.iter().rposition(|s| s.t < t_d && s.theta < t_d && s.consumed <= p) else {
        return run(&prev.model, cfg);
    };
    let snap = &prev.snapshots[k];
    let r = Run {
        model: prev.model.clone(),
        config: cfg.clone(),
        committed: prev.committed[..snap.committed].to_vec(),
        segments: prev.segments[..snap.segments].to_vec(),
        log: prev.log[..snap.log].to_vec(),
        diagnostics: prev.diagnostics[..snap.diagnostics].to_vec(),
        rejected: prev.rejected[..snap.rejected].to_vec(),
        status: RunStatus::Completed,
        snapshots: prev.snapshots[..=k].to_vec(),
    };
    let mut e = Engine {
        cm: &prev.model,
        cfg: &cfg,
        vals: snap.vals.clone(),
        t: snap.t,
        n: snap.n,
        refs: snap.refs.clone(),
        zeno: snap.zeno.clone(),
        time_next: snap.time_next.clone(),
        every_k: snap.every_k.clone(),
        payloads: snap.payloads.clone(),
        due_k: snap.due_k.clone(),
        clocks: snap.clocks.clone(),
        run: r,
        h: snap.h,
        consumed: snap.consumed,
        theta: snap.theta,
        acting: false,
    };
    if let Err(d) = e.advance() {
        e.run.status = RunStatus::Stopped(d);
    }
    e.run
}

impl<'a> Engine<'a> {
    /// Evaluates the instants of `at` and `every` triggers (RC-7.9). `at(τ)` is evaluated once,
    /// at initialization; the k-th instant of `every` is `from + k Δ`, never accumulated.
    fn schedule_time_events(&mut self) -> Result<(), RunDiag> {
        for ei in 0..self.cm.events.len() {
            let vals = self.vals.clone();
            let c = self.ctx(&vals);
            match &self.cm.events[ei].trigger {
                Some(CTrigger::At(tau)) => {
                    let t = tau.eval(&c).map_err(|s| self.diag(Category::Model, s.cause, None))?.num();
                    self.time_next[ei] = if t >= self.cfg.t0 { Some(t) } else { None };
                }
                Some(CTrigger::Every { period, from }) => {
                    let d = period.eval(&c).map_err(|s| self.diag(Category::Model, s.cause, None))?.num();
                    let f = from.eval(&c).map_err(|s| self.diag(Category::Model, s.cause, None))?.num();
                    if d <= 0.0 {
                        return Err(self.diag(Category::Model, "an `every` period must be positive".into(), Some(&self.cm.events[ei].id)));
                    }
                    let k = ((self.cfg.t0 - f) / d).ceil().max(0.0) as u64;
                    self.every_k[ei] = k;
                    self.time_next[ei] = Some(f + k as f64 * d);
                }
                _ => {}
            }
        }
        Ok(())
    }

    /// RC-13.8: after a transition, each random event's clock integrates its rate up to now,
    /// takes the rate of the committed state and schedules its next occurrence, never at the
    /// current instant. A clock whose rate varies between transitions (RC-13.8a) is
    /// integrated along the steps instead and has no scheduled instant.
    fn refresh_clocks(&mut self) -> Result<(), RunDiag> {
        for ei in 0..self.cm.events.len() {
            let Some(CTrigger::Random { rate, varying }) = &self.cm.events[ei].trigger else { continue };
            let r = rate.eval(&self.ctx(&self.vals)).map_err(|s| self.diag(Category::Model, s.cause, Some(&self.cm.events[ei].id)))?.num();
            if r < 0.0 {
                return Err(self.negative_rate(ei, r));
            }
            let t = self.t;
            let c = self.clocks[ei].as_mut().unwrap();
            if *varying {
                c.t_last = t;
                c.rate = r;
                self.time_next[ei] = None;
                continue;
            }
            c.integral += c.rate * (t - c.t_last);
            c.t_last = t;
            c.rate = r;
            self.time_next[ei] = if r > 0.0 {
                let next = t + (c.target - c.integral) / r;
                Some(if next > t { next } else { after(t) })
            } else {
                None
            };
        }
        Ok(())
    }

    fn next_time_event(&self) -> Option<f64> {
        self.time_next.iter().flatten().copied().fold(None, |m, t| Some(m.map_or(t, |x: f64| x.min(t))))
    }

    /// Time events due at the current time; advances their schedules.
    fn take_time_events(&mut self) -> Vec<usize> {
        let mut due = vec![];
        for ei in 0..self.cm.events.len() {
            if self.time_next[ei] != Some(self.t) {
                continue;
            }
            due.push(ei);
            match &self.cm.events[ei].trigger {
                Some(CTrigger::Every { period, from }) => {
                    let vals = self.vals.clone();
                    let c = self.ctx(&vals);
                    let (d, f) = (period.eval(&c).map(|v| v.num()).unwrap_or(f64::NAN), from.eval(&c).map(|v| v.num()).unwrap_or(f64::NAN));
                    self.every_k[ei] += 1;
                    self.time_next[ei] = Some(f + self.every_k[ei] as f64 * d);
                }
                Some(CTrigger::Random { .. }) => self.fire_clock(ei),
                _ => self.time_next[ei] = None,
            }
        }
        due
    }

    /// RC-13.8: the integral has reached the target; the next target is one more standard
    /// exponential from the clock's stream.
    fn fire_clock(&mut self, ei: usize) {
        let stream = fnv1a(&self.cm.events[ei].id);
        let (seed, t) = (self.cfg.seed, self.t);
        let c = self.clocks[ei].as_mut().unwrap();
        c.integral = c.target;
        c.t_last = t;
        c.fired += 1;
        c.target += Uniforms::new(seed, stream, CLOCK_SITE, c.fired).exp1();
        self.time_next[ei] = None;
    }

    fn negative_rate(&self, ei: usize, r: f64) -> RunDiag {
        self.diag(Category::Model, format!("the rate of `{}` is negative ({r})", self.cm.events[ei].name), Some(&self.cm.events[ei].id))
    }

    /// Stores an accepted step of the trajectory. A run with more than `LARGE_STATE`
    /// components (fields, D-091) keeps each step as the straight line between its two
    /// states, each state shared with the neighbouring step: one state per step instead of
    /// the solver's interpolant (RC-6.2a). Values at step ends are exact either way.
    fn keep(&mut self, step: Step, base: usize) {
        let n = self.cm.n_y();
        if n <= LARGE_STATE || matches!(step.dense, Dense::Linear { .. }) {
            self.run.segments.push(Segment { step, base });
            return;
        }
        let prev = self.run.segments.last().and_then(|s| match &s.step.dense {
            Dense::Linear { y1, .. } if s.step.t1 == step.t0 => Some(y1.clone()),
            _ => None,
        });
        let at = |t: f64| {
            let mut y = vec![0.0; n];
            step.eval(t, &mut y);
            std::rc::Rc::new(y)
        };
        let y0 = prev.unwrap_or_else(|| at(step.t0));
        let y1 = at(step.t1);
        let mut compact = Step::new(step.t0, step.t1 - step.t0, Dense::Linear { y0, y1 });
        compact.t1 = step.t1;
        self.run.segments.push(Segment { step: compact, base });
    }

    /// The rates of the random events `which` at `t` inside a step (RC-13.8a). A negative
    /// rate is a model failure, reported at the first instant of the step found negative, by
    /// bisection to the time tolerance.
    fn rates_in_step(&self, step: &Step, scratch: &[Value], t: f64, which: &[usize], out: &mut [f64]) -> Result<(), RunDiag> {
        let rates = |t: f64, out: &mut [f64]| -> Result<Option<usize>, RunDiag> {
            let vals = self.state_in_step(step, scratch, t).map_err(|s| self.diag(Category::Model, s.cause, None))?;
            let c = Ctx { vals: &vals, der: None, t, t0: self.cfg.t0, args: &[], payloads: &self.payloads, draws: None, noise: &[], point: None };
            for (k, (o, &ei)) in out.iter_mut().zip(which).enumerate() {
                let Some(CTrigger::Random { rate, .. }) = &self.cm.events[ei].trigger else { unreachable!() };
                *o = rate.eval(&c).map_err(|s| self.diag(Category::Model, s.cause, Some(&self.cm.events[ei].id)))?.num();
                if *o < 0.0 {
                    return Ok(Some(k));
                }
            }
            Ok(None)
        };
        let Some(mut k) = rates(t, out)? else { return Ok(()) };
        let (mut lo, mut hi, mut r) = (self.t, t, out[k]);
        let mut probe = vec![0.0; which.len()];
        while hi - lo > self.cfg.eps_t() {
            let mid = 0.5 * (lo + hi);
            if mid <= lo || mid >= hi {
                break;
            }
            match rates(mid, &mut probe)? {
                Some(j) => {
                    (hi, k, r) = (mid, j, probe[j]);
                }
                None => lo = mid,
            }
        }
        Err(RunDiag { t: hi, ..self.negative_rate(which[k], r) })
    }

    /// Three-point Gauss-Legendre integrals over `[a, b]` of the rates of `which`.
    fn rate_panel(&self, step: &Step, scratch: &[Value], a: f64, b: f64, which: &[usize]) -> Result<Vec<f64>, RunDiag> {
        const X: [f64; 3] = [-0.774_596_669_241_483_4, 0.0, 0.774_596_669_241_483_4];
        const W: [f64; 3] = [5.0 / 9.0, 8.0 / 9.0, 5.0 / 9.0];
        let (m, r) = (0.5 * (a + b), 0.5 * (b - a));
        let mut sum = vec![0.0; which.len()];
        let mut rates = vec![0.0; which.len()];
        for (x, w) in X.iter().zip(W) {
            self.rates_in_step(step, scratch, m + r * x, which, &mut rates)?;
            for (s, q) in sum.iter_mut().zip(&rates) {
                *s += w * r * q;
            }
        }
        Ok(sum)
    }

    /// The integrals over `[a, b]` of the rates of `which` along a step's dense output:
    /// Gauss-Legendre panels halved until the two halves agree with the whole to `1e-10`
    /// (absolute and relative) for every rate (RC-13.8a).
    fn integrate_rates(&self, step: &Step, scratch: &[Value], a: f64, b: f64, which: &[usize]) -> Result<Vec<f64>, RunDiag> {
        if b <= a {
            return Ok(vec![0.0; which.len()]);
        }
        let mut total = vec![0.0; which.len()];
        let mut stack = vec![(a, b, self.rate_panel(step, scratch, a, b, which)?, 0u32)];
        while let Some((a, b, whole, depth)) = stack.pop() {
            let m = 0.5 * (a + b);
            let (l, r) = (self.rate_panel(step, scratch, a, m, which)?, self.rate_panel(step, scratch, m, b, which)?);
            let close = whole.iter().zip(l.iter().zip(&r)).all(|(w, (x, y))| (w - (x + y)).abs() <= 1e-10 * (1.0 + (x + y).abs()));
            if close || depth >= 30 || m <= a || m >= b {
                for (t, (x, y)) in total.iter_mut().zip(l.iter().zip(&r)) {
                    *t += x + y;
                }
            } else {
                stack.push((m, b, r, depth + 1));
                stack.push((a, m, l, depth + 1));
            }
        }
        Ok(total)
    }

    /// RC-13.8a: the first instant in the step at which a varying clock reaches its target,
    /// by bisection to the time tolerance (the far end of the bracket, as RC-7.6), and the
    /// clocks reached there. `total` holds the integrals over the whole step.
    fn locate_clocks(&self, step: &Step, scratch: &[Value], which: &[usize], total: &[f64], t1: f64, eps_t: f64) -> Result<Option<(f64, Vec<usize>)>, RunDiag> {
        let need: Vec<f64> = which.iter().map(|&ei| self.clocks[ei].as_ref().map(|c| c.target - c.integral).unwrap()).collect();
        let reach = |at: &[f64]| at.iter().zip(&need).any(|(i, n)| i >= n);
        if !reach(total) {
            return Ok(None);
        }
        let (mut lo, mut hi) = (self.t, t1);
        let mut at_lo = vec![0.0; which.len()];
        let mut at_hi = total.to_vec();
        while hi - lo > eps_t {
            let mid = 0.5 * (lo + hi);
            if mid <= lo || mid >= hi {
                break;
            }
            let add = self.integrate_rates(step, scratch, lo, mid, which)?;
            let at_mid: Vec<f64> = at_lo.iter().zip(&add).map(|(x, y)| x + y).collect();
            if reach(&at_mid) {
                hi = mid;
                at_hi = at_mid;
            } else {
                lo = mid;
                at_lo = at_mid;
            }
        }
        let reached = which.iter().zip(at_hi.iter().zip(&need)).filter(|(_, (i, n))| i >= n).map(|(&ei, _)| ei).collect();
        Ok(Some((hi, reached)))
    }

    /// Adds to the varying clocks `which` the integrals of their rates up to `t`.
    fn advance_clocks(&mut self, which: &[usize], add: &[f64], t: f64) {
        for (&ei, a) in which.iter().zip(add) {
            let c = self.clocks[ei].as_mut().unwrap();
            c.integral += a;
            c.t_last = t;
        }
    }

    fn diag(&self, category: Category, message: String, element: Option<&str>) -> RunDiag {
        RunDiag { category, message, element: element.map(|s| s.to_string()), t: self.t, n: self.n }
    }

    fn ctx<'v>(&'v self, vals: &'v [Value]) -> Ctx<'v> {
        Ctx { vals, der: None, t: self.t, t0: self.cfg.t0, args: &[], payloads: &self.payloads, draws: Some(Draws { seed: self.cfg.seed, due: &self.due_k }), noise: &[], point: None }
    }

    fn snapshot(&mut self) {
        let r = &self.run;
        let s = Snapshot {
            t: self.t,
            n: self.n,
            vals: self.vals.clone(),
            refs: self.refs.clone(),
            zeno: self.zeno.clone(),
            time_next: self.time_next.clone(),
            every_k: self.every_k.clone(),
            payloads: self.payloads.clone(),
            due_k: self.due_k.clone(),
            clocks: self.clocks.clone(),
            h: self.h,
            committed: r.committed.len(),
            segments: r.segments.len(),
            log: r.log.len(),
            diagnostics: r.diagnostics.len(),
            rejected: r.rejected.len(),
            consumed: self.consumed,
            theta: self.theta,
        };
        self.run.snapshots.push(s);
    }

    fn commit(&mut self) {
        self.run.committed.push(Committed { t: self.t, n: self.n, vals: self.vals.clone(), by_action: self.acting });
    }

    fn guard_value(&self, vals: &[Value], ei: usize) -> Result<Option<f64>, Status> {
        match &self.cm.events[ei].trigger {
            Some(CTrigger::Crossing { guard, .. }) => Ok(Some(guard.eval(&self.ctx(vals))?.num())),
            _ => Ok(None),
        }
    }

    /// RC-5.4, RC-8.4: sign references from the current committed state.
    ///
    /// RC-7.2a (D-056): a located crossing leaves its guard within the location tolerance of
    /// zero, on the far side (RC-7.6). When the guard changes sign again within `4 ε_t` under
    /// the flows of the committed state (a ball whose bounce has just reversed it), its
    /// reference is the sign it is heading to, so that the next crossing is detected even
    /// when the whole excursion lies inside one step.
    fn retake_refs(&mut self) {
        let vals = self.vals.clone();
        let ahead = self.just_ahead(&vals);
        for ei in 0..self.cm.events.len() {
            if let Ok(Some(g)) = self.guard_value(&vals, ei) {
                let heading = ahead.as_ref().and_then(|v| self.guard_value(v, ei).ok().flatten()).and_then(sign);
                let s = match (sign(g), heading) {
                    (Some(now), Some(next)) if now != next => Some(next),
                    (now, _) => now,
                };
                if let Some(s) = s {
                    self.refs[ei] = Some(s);
                }
            }
        }
    }

    /// The state `4 ε_t` after the committed one, by one explicit step of the flows; `None`
    /// for a model without continuous state or when the flows cannot be evaluated.
    fn just_ahead(&self, vals: &[Value]) -> Option<Vec<Value>> {
        let cm = self.cm;
        let n = cm.n_y();
        if n == 0 || cm.is_static {
            return None;
        }
        let dt = 4.0 * self.cfg.eps_t();
        let (mut y, mut dy) = (vec![0.0; n], vec![0.0; n]);
        cm.store_y(vals, &mut y);
        cm.rhs(vals, self.t, self.cfg.t0, &mut dy).ok()?;
        for (a, d) in y.iter_mut().zip(&dy) {
            *a += dt * d;
        }
        let mut out = vals.to_vec();
        cm.load_y(&mut out, &y);
        cm.update_derived(&mut out, self.t + dt, self.cfg.t0).ok()?;
        Some(out)
    }

    /// Checks constraints (MK-12.3). Returns reports, or the failure of a `reject` or `stop`
    /// constraint.
    fn check_constraints(&self, vals: &[Value], _initial: bool) -> Result<Vec<RunDiag>, RunDiag> {
        let mut reports = vec![];
        for k in &self.cm.constraints {
            // An enforced constraint is kept by projection (RC-5.10), not checked. An inequality
            // out of contact holds by itself; one found well past its boundary was crossed
            // without touching, which a holding force of the wrong sign causes (RC-5.11).
            if k.policy == Policy::Enforce {
                if let (Some((on, side)), Some(gap)) = (k.contact, k.levels.first()) {
                    let c = self.ctx(vals);
                    let force = || self.cm.ir.constraints.iter().find(|x| x.id == k.id).and_then(|x| x.force.clone()).map(|f| self.cm.bindings[self.cm.idx(&f)].name.clone()).unwrap_or_default();
                    // A contact whose force comes out negative is released, unless a positive
                    // force would close the gap: then the force is written the wrong way.
                    let pulls = vals[on].boolean() && self.cm.equations.iter().find(|q| q.contact.is_some_and(|(c, _)| c == on)).is_some_and(|q| vals[q.contact.unwrap().1].num() < 0.0);
                    if pulls && self.cm.holds_as_declared(vals, on, side, self.t, self.cfg.t0).map_err(|s| self.diag(Category::Model, s.cause, None))? == Some(false) {
                        let name = force();
                        return Err(self.diag(
                            Category::Constraint,
                            format!("constraint `{}`: a positive `{name}` closes it instead of holding it; write `{name}` in the flows so that it is positive when it holds the constraint (RC-5.11)", k.name),
                            Some(&k.id),
                        ));
                    }
                    if !vals[on].boolean() {
                        if let (Ok((a, sa)), Ok((b, sb))) = (gap.lhs.eval_sized(&c, 1), gap.rhs.eval_sized(&c, 1)) {
                            if side * (a.num() - b.num()) < -(1e-6 * (sa[0] + sb[0]) + 1e-6) {
                                return Err(self.diag(
                                    Category::Constraint,
                                    format!("constraint `{}` is crossed out of contact (RC-5.11): the crossing was not found; give the solver a smaller step", k.name),
                                    Some(&k.id),
                                ));
                            }
                        }
                    }
                }
                continue;
            }
            let c = self.ctx(vals);
            let ok = if let Some((l, r, tol)) = &k.within {
                match (l.eval(&c), r.eval(&c)) {
                    (Ok(a), Ok(b)) => distance(&a, &b).map(|d| d <= *tol).unwrap_or(false),
                    _ => false,
                }
            } else {
                matches!(k.cond.as_ref().unwrap().eval(&c), Ok(Value::Bool(true)))
            };
            if ok {
                continue;
            }
            let values: Vec<String> = self
                .cm
                .ir
                .constraints
                .iter()
                .find(|x| x.id == k.id)
                .map(|x| x.cond.refs())
                .unwrap_or_default()
                .iter()
                .map(|id| format!("{} = {}", self.cm.bindings[self.cm.idx(id)].name, vals[self.cm.idx(id)]))
                .collect();
            let d = self.diag(Category::Constraint, format!("constraint `{}` violated ({})", k.name, values.join(", ")), Some(&k.id));
            match k.policy {
                Policy::Report => reports.push(d),
                Policy::Reject | Policy::Stop => return Err(d),
                Policy::Enforce => unreachable!("skipped above"),
            }
        }
        Ok(reports)
    }

    /// Equation checks (MK-11.2): residuals above the tolerance are reported.
    fn check_equations(&mut self) {
        let der = self.cm.der_values(&self.vals, self.t, self.cfg.t0).ok();
        for q in &self.cm.equations {
            let Some(tol) = q.check_tol else { continue };
            // An equation over a field: the largest residual over its cells (D-091).
            let cells = q.field.map(|g| g.cells()).unwrap_or(1);
            let mut res = Some(0.0f64);
            for k in 0..cells {
                let args = q.field.map(|g| [g.coord(k)]);
                let c = Ctx { vals: &self.vals, der: der.as_deref(), t: self.t, t0: self.cfg.t0, args: args.as_ref().map(|a| &a[..]).unwrap_or(&[]), payloads: &[], draws: None, noise: &[], point: q.field.map(|_| k) };
                let r = match (q.lhs.eval(&c), q.rhs.eval(&c)) {
                    (Ok(a), Ok(b)) => distance(&a, &b),
                    _ => None,
                };
                res = match (res, r) {
                    (Some(a), Some(b)) => Some(a.max(b)),
                    _ => None,
                };
            }
            if res.map(|r| r > tol).unwrap_or(true) {
                let message = match res {
                    Some(r) => format!("check equation `{}` at t = {} s: residual {r:.3e} exceeds {tol:e}", q.name, self.t),
                    None => format!("check equation `{}` at t = {} s: a side has no value", q.name, self.t),
                };
                let d = self.diag(Category::Equation, message, Some(&q.id));
                self.run.diagnostics.push(d);
            }
        }
    }

    /// Locates, by bisection to the time tolerance, the first instant of `step` whose state
    /// breaks a constraint; keeps the step up to the last instant found valid, commits that
    /// state, and returns the failure at the first instant found broken (RC-10.1, RC-10.3).
    fn fail_in_step(&mut self, mut step: Step, base: usize, first: RunDiag) -> RunDiag {
        let state = |me: &Self, t: f64| -> Option<Vec<Value>> {
            let mut y = vec![0.0; me.cm.n_y()];
            step.eval(t, &mut y);
            let mut vals = me.vals.clone();
            me.cm.load_y(&mut vals, &y);
            me.cm.update_derived(&mut vals, t, me.cfg.t0).ok()?;
            Some(vals)
        };
        let (mut a, mut b, mut fail) = (step.t0, step.t1, first);
        while b - a > self.cfg.eps_t() {
            let m = 0.5 * (a + b);
            match state(self, m).map(|v| self.check_constraints(&v, false)) {
                Some(Ok(_)) => a = m,
                Some(Err(d)) => {
                    b = m;
                    fail = d;
                }
                None => b = m,
            }
        }
        if a > step.t0 {
            if let Some(v) = state(self, a) {
                step.t1 = a;
                self.keep(step, base);
                self.t = a;
                self.n = 0;
                self.vals = v;
                self.commit();
            }
        }
        fail.t = b;
        fail
    }

    fn after_step_checks(&mut self) -> Result<(), RunDiag> {
        let reports = self.check_constraints(&self.vals.clone(), false)?;
        self.run.diagnostics.extend(reports);
        self.check_equations();
        Ok(())
    }

    /// Continuous evolution with event detection until `t_end` (RC sections 6, 7).
    fn advance(&mut self) -> Result<(), RunDiag> {
        let cm = self.cm;
        if cm.is_static {
            // RC-12.4: a static model has a single instant; only interventions apply.
            let actions: Vec<Scheduled> = self.cfg.log.clone();
            for s in actions {
                self.acting = true;
                let applied = self.intervene(&s.action);
                self.acting = false;
                applied?;
            }
            return Ok(());
        }
        let eps_t = self.cfg.eps_t();
        let n_y = cm.n_y();
        let mut y = vec![0.0; n_y];
        let mut h: Option<f64> = self.h;
        let mut pending: VecDeque<Scheduled> = sorted_log(&self.cfg.log).into_iter().skip(self.consumed).collect();
        let mut steps = 0usize;
        // RC-13.8a: random events whose rate varies along a step.
        let varying: Vec<usize> = (0..cm.events.len()).filter(|&ei| matches!(cm.events[ei].trigger, Some(CTrigger::Random { varying: true, .. }))).collect();
        loop {
            // RC-14.3: a snapshot at the start, where actions apply, and every few steps.
            let due = pending.front().map(|s| s.t <= self.t).unwrap_or(false);
            let fresh = self.run.snapshots.last().map(|s| s.committed != self.run.committed.len() || s.t != self.t).unwrap_or(true);
            if fresh && (steps == 0 || due || steps.is_multiple_of(SNAPSHOT_STEPS)) {
                self.h = h;
                self.snapshot();
            }
            steps += 1;
            // Logged actions due now apply after the model's own iteration (RC-8.7, RC-11.1).
            let mut acted = false;
            while pending.front().map(|s| s.t <= self.t).unwrap_or(false) {
                if self.run.committed.last().map(|c| c.t != self.t).unwrap_or(true) {
                    self.n = 0;
                    self.commit();
                }
                let s = pending.pop_front().unwrap();
                self.consumed += 1;
                self.acting = true;
                let applied = self.intervene(&s.action);
                self.acting = false;
                applied?;
                acted = true;
            }
            if acted {
                self.refresh_clocks()?;
                self.retake_refs();
                h = None;
            }
            if self.t >= self.cfg.t_end {
                break;
            }
            cm.store_y(&self.vals, &mut y);
            let mut next_stop = pending.front().map(|s| s.t.min(self.cfg.t_end)).unwrap_or(self.cfg.t_end);
            if let Some(te) = self.next_time_event() {
                next_stop = next_stop.min(te);
            }
            let base = self.run.committed.len() - 1;
            let mut rhs = SegmentRhs { cm, scratch: self.vals.clone(), t0: self.cfg.t0, noise: vec![] };
            let span = next_stop - self.t;
            // One accepted step (RC-6.7: never past the next stop).
            let (y1, step, h_next) = if !cm.noise.is_empty() {
                // RC-13.9 (D-090): an Euler-Maruyama step. Each noise takes `ΔW / h` with
                // `ΔW = sqrt(h) N`, `N` standard normal from the noise's stream at the step's
                // start, so the flows give the drift plus the noise's coefficient times `ΔW`.
                let hf = self.cfg.noise_step();
                let land = self.t + hf >= next_stop - 1e-12 * hf;
                let hs = if land { span } else { hf };
                self.theta = self.theta.max(if land { next_stop } else { reach(self.t, hf) });
                let (seed, t) = (self.cfg.seed, self.t);
                rhs.noise = cm.noise.iter().map(|&key| Uniforms::new(seed, key, 0, t.to_bits()).normal() / hs.sqrt()).collect();
                let mut dy = vec![0.0; n_y];
                rhs.f(self.t, &y, &mut dy).map_err(|s| self.diag(Category::Model, s.cause, None))?;
                let y1: Vec<f64> = y.iter().zip(&dy).map(|(a, d)| a + hs * d).collect();
                rhs.noise.clear();
                let step = Step::new(self.t, hs, Dense::Linear { y0: std::rc::Rc::new(y.clone()), y1: std::rc::Rc::new(y1.clone()) });
                (y1, step, None)
            } else {
                match &self.cfg.solver {
                SolverConfig::Rk4 { h: Some(hf) } => {
                    let land = self.t + hf >= next_stop - 1e-12 * hf;
                    let hs = if land { span } else { *hf };
                    self.theta = self.theta.max(if land { next_stop } else { reach(self.t, *hf) });
                    let (y1, step) = Rk4::step(&mut rhs, self.t, &y, hs).map_err(|s| self.diag(Category::Model, s.cause, None))?;
                    (y1, step, None)
                }
                SolverConfig::Rk4 { h: None } => unreachable!(),
                SolverConfig::Dopri5 { rtol, atol, h_max } => {
                    let solver = Dopri5 { rtol: *rtol, atol: *atol, h_max: *h_max };
                    let mut hh = match h {
                        Some(x) => x,
                        None => {
                            let (h0, free) = solver.initial_step(&mut rhs, self.t, &y, span).map_err(|s| self.diag(Category::Model, s.cause, None))?;
                            self.theta = self.theta.max(reach(self.t, free));
                            h0
                        }
                    };
                    loop {
                        let land = self.t + hh >= next_stop - 1e-12 * hh.max(1e-300);
                        let hs = if land { span } else { hh };
                        // A step that lands depends on where the stop is; one that does not,
                        // only on the stop lying beyond it.
                        self.theta = self.theta.max(if land { next_stop } else { reach(self.t, hh) });
                        let h_min = 1e-14 * self.t.abs().max(1.0);
                        if hs < h_min {
                            return Err(self.diag(Category::Computational, format!("step size {hs} below the minimum at t = {}", self.t), None));
                        }
                        match solver.attempt(&mut rhs, self.t, &y, hs) {
                            Ok(Attempt::Accepted { y1, step, h_next }) => break (y1, step, Some(h_next)),
                            Ok(Attempt::Rejected { h_next }) => hh = h_next,
                            // RC-6.8: a status in a flow rejects the step; retry smaller.
                            Err(s) => {
                                hh = hs * 0.25;
                                if hh < h_min {
                                    return Err(self.diag(Category::Model, s.cause, None));
                                }
                            }
                        }
                    }
                }
            }
            };
            h = h_next;
            let t1 = if step.t1 >= next_stop - 1e-12 * (step.t1 - step.t0) { next_stop } else { step.t1 };
            let mut step = step;
            step.t1 = t1;

            // Event detection on the dense output (RC-7.1 to RC-7.7).
            let mut found: Vec<(usize, f64)> = vec![];
            let mut new_refs = self.refs.clone();
            // The state at each of the four sample instants, with its derived values, is
            // computed once and shared by every event's guard.
            let mut samples: [Option<Result<Vec<Value>, Status>>; 4] = Default::default();
            for ei in 0..cm.events.len() {
                let Some(CTrigger::Crossing { dir, .. }) = &cm.events[ei].trigger else { continue };
                let mut r = self.refs[ei];
                let mut prev = self.t;
                for k in 1..=4 {
                    let tk = if k == 4 { t1 } else { self.t + (t1 - self.t) * k as f64 / 4.0 };
                    let vals = samples[k - 1].get_or_insert_with(|| self.state_in_step(&step, &rhs.scratch, tk));
                    let g = vals.as_ref().map_err(|s| s.clone()).and_then(|v| self.guard_in(v, tk, ei)).map_err(|s| self.diag(Category::Model, s.cause, None))?;
                    if let Some(s) = sign(g) {
                        match r {
                            Some(rs) if rs != s => {
                                if matches_dir(*dir, rs, s) {
                                    let th = self.locate(&step, &rhs.scratch, ei, prev, tk, s, eps_t)?;
                                    found.push((ei, th));
                                    break;
                                }
                                r = Some(s);
                            }
                            None => r = Some(s),
                            _ => {}
                        }
                        prev = tk;
                    }
                }
                new_refs[ei] = r;
            }
            let mut clocks_total = vec![];
            if !varying.is_empty() {
                clocks_total = self.integrate_rates(&step, &rhs.scratch, self.t, t1, &varying)?;
                if let Some((th, reached)) = self.locate_clocks(&step, &rhs.scratch, &varying, &clocks_total, t1, eps_t)? {
                    found.extend(reached.into_iter().map(|ei| (ei, th)));
                }
            }

            if !found.is_empty() {
                let te = found.iter().map(|x| x.1).fold(f64::INFINITY, f64::min);
                let due: Vec<usize> = found.iter().filter(|x| x.1 <= te + eps_t).map(|x| x.0).collect();
                if !varying.is_empty() {
                    let add = if te == t1 { clocks_total.clone() } else { self.integrate_rates(&step, &rhs.scratch, self.t, te, &varying)? };
                    self.advance_clocks(&varying, &add, te);
                }
                step.t1 = te;
                let mut ye = vec![0.0; n_y];
                step.eval(te, &mut ye);
                self.keep(step, base);
                self.t = te;
                self.n = 0;
                cm.load_y(&mut self.vals, &ye);
                cm.project(&mut self.vals, self.t, self.cfg.t0, &[]).map_err(|s| self.diag(Category::Model, s.cause, None))?;
                cm.update_derived(&mut self.vals, self.t, self.cfg.t0).map_err(|s| self.diag(Category::Model, s.cause, None))?;
                self.commit();
                self.after_step_checks()?;
                for &ei in &due {
                    if matches!(cm.events[ei].trigger, Some(CTrigger::Random { .. })) {
                        self.fire_clock(ei);
                    }
                }
                self.iterate(due, vec![])?;
                self.refresh_clocks()?;
                self.retake_refs();
                h = None;
                continue;
            }
            // RC-10.1: a step whose end breaks a `reject` or `stop` constraint is not taken
            // whole. The run ends at the last state found valid inside it.
            let mut v1 = self.vals.clone();
            cm.load_y(&mut v1, &y1);
            // RC-5.10: the end of each step is moved back onto the enforced constraints, so
            // the solver's error does not accumulate as drift.
            cm.project(&mut v1, t1, self.cfg.t0, &[]).map_err(|s| self.diag(Category::Model, s.cause, None))?;
            cm.update_derived(&mut v1, t1, self.cfg.t0).map_err(|s| self.diag(Category::Model, s.cause, None))?;
            if let Err(d) = self.check_constraints(&v1, false) {
                return Err(self.fail_in_step(step, base, d));
            }
            self.advance_clocks(&varying, &clocks_total, t1);
            self.keep(step, base);
            self.t = t1;
            self.n = 0;
            self.vals = v1;
            self.refs = new_refs;
            self.after_step_checks()?;
            // RC-6.7, RC-7.9: a step never crosses a time event; it lands on it.
            let timed = self.take_time_events();
            if !timed.is_empty() {
                self.commit();
                self.iterate(timed, vec![])?;
                self.refresh_clocks()?;
                self.retake_refs();
                h = None;
            }
        }
        let last = self.run.committed.last().unwrap();
        if last.t != self.t {
            self.commit();
        }
        Ok(())
    }

    fn guard_at(&self, step: &Step, scratch: &[Value], t: f64, ei: usize) -> Result<f64, Status> {
        self.guard_in(&self.state_in_step(step, scratch, t)?, t, ei)
    }

    /// The state at `t` inside a step, from its dense output, with its derived values.
    fn state_in_step(&self, step: &Step, scratch: &[Value], t: f64) -> Result<Vec<Value>, Status> {
        let mut y = vec![0.0; self.cm.n_y()];
        step.eval(t, &mut y);
        let mut vals = scratch.to_vec();
        self.cm.load_y(&mut vals, &y);
        self.cm.update_derived(&mut vals, t, self.cfg.t0)?;
        Ok(vals)
    }

    /// The guard of crossing event `ei` in a state at `t`.
    fn guard_in(&self, vals: &[Value], t: f64, ei: usize) -> Result<f64, Status> {
        let c = Ctx { vals, der: None, t, t0: self.cfg.t0, args: &[], payloads: &[], draws: None, noise: &[], point: None };
        match &self.cm.events[ei].trigger {
            Some(CTrigger::Crossing { guard, .. }) => Ok(guard.eval(&c)?.num()),
            _ => unreachable!(),
        }
    }

    /// Bisection on the dense output until the bracket is at most `eps_t` wide (RC-7.5).
    /// Returns `t_hi`, the far end of the bracket (RC-7.6).
    fn locate(&self, step: &Step, scratch: &[Value], ei: usize, mut lo: f64, mut hi: f64, new_sign: bool, eps_t: f64) -> Result<f64, RunDiag> {
        while hi - lo > eps_t {
            let mid = 0.5 * (lo + hi);
            if mid <= lo || mid >= hi {
                break;
            }
            let g = self.guard_at(step, scratch, mid, ei).map_err(|s| self.diag(Category::Model, s.cause, None))?;
            if sign(g) == Some(new_sign) {
                hi = mid;
            } else {
                lo = mid;
            }
        }
        Ok(hi)
    }

    fn zeno_detected(&mut self, ei: usize) -> bool {
        let ev = &self.cm.events[ei];
        let Some(z) = &ev.zeno else { return false };
        let eps = z.eps.or(self.cfg.eps_zeno).unwrap_or(1e-6 * self.cfg.t_ref);
        let n = z.n.unwrap_or(self.cfg.n_zeno) as usize;
        let w = z.window.or(self.cfg.w_zeno).unwrap_or(self.cfg.t_ref);
        let m = &mut self.zeno[ei];
        let t = self.t;
        while m.recent.front().map(|&x| x < t - w).unwrap_or(false) {
            m.recent.pop_front();
        }
        let close = m.last.map(|l| t != l && t - l < eps).unwrap_or(false);
        close || m.recent.len() + 1 > n
    }

    fn record_occurrence(&mut self, ei: usize) {
        let m = &mut self.zeno[ei];
        if m.last != Some(self.t) {
            m.last = Some(self.t);
            m.recent.push_back(self.t);
        }
    }

    /// Why a request of event `ei` with `payload` is not enabled: a member it names is not
    /// alive (D-059), or the event's condition does not hold.
    fn not_enabled(&self, ei: usize, payload: Option<&Value>) -> String {
        let name = &self.cm.events[ei].name;
        if let (Some(p), Some(v)) = (&self.cm.ir.events[ei].payload, payload) {
            let values: Vec<Value> = match v {
                Value::Tuple(t) if !p.items.is_empty() => t.iter().cloned().collect(),
                other => vec![other.clone()],
            };
            for (item, v) in p.declared().into_iter().zip(values) {
                let (Some(of), Some(path)) = (&item.of, &item.members) else { continue };
                let member = format!("{path}[{}]", v.num());
                let live = self.cm.index.get(&prismal_ir::elaborate::alive(of, &member));
                if live.is_some_and(|&b| !self.vals[b].boolean()) {
                    return format!("`{name}` is refused: `{member}` is not alive (not made yet, or destroyed)");
                }
            }
        }
        format!("`{name}` is not enabled now: its condition does not hold")
    }

    /// Event iteration at the current event time (RC section 8.1).
    fn iterate(&mut self, due: Vec<usize>, requested: Vec<usize>) -> Result<(), RunDiag> {
        let r = self.iterate_with(due, requested.into_iter().map(|e| (e, None)).collect(), vec![None; self.cm.events.len()]);
        self.payloads = vec![None; self.cm.events.len()];
        r
    }

    /// Event iteration with payloads (D-050): `requested` with the payload each request
    /// supplies, `carried` the payload each event occurred or was emitted with at the
    /// previous microstep, which the events that follow it (`on E`) receive.
    fn iterate_with(&mut self, mut due: Vec<usize>, mut requested: Vec<(usize, Option<Value>)>, mut carried: Vec<Option<Value>>) -> Result<(), RunDiag> {
        loop {
            // The payloads of this microstep's occurrences.
            self.payloads = vec![None; self.cm.events.len()];
            for &ei in &due {
                if let Some(CTrigger::On(src)) = &self.cm.events[ei].trigger {
                    self.payloads[ei] = carried[*src].clone();
                }
            }
            for (ei, p) in &requested {
                self.payloads[*ei] = p.clone();
            }
            // RC-13.5: each event due at this microstep draws with its due count, which
            // advances after it, whether or not the event is enabled.
            let mut counted: Vec<usize> = due.iter().copied().chain(requested.iter().map(|r| r.0)).collect();
            counted.sort();
            counted.dedup();
            let r = self.microstep(&mut due, &mut requested, &mut carried);
            for ei in counted {
                self.due_k[ei] += 1;
            }
            if !r? {
                return Ok(());
            }
        }
    }

    /// One microstep of `iterate_with`: false when no event is enabled; otherwise `due`,
    /// `requested` and `carried` become those of the next microstep.
    fn microstep(&mut self, due: &mut Vec<usize>, requested: &mut Vec<(usize, Option<Value>)>, carried: &mut Vec<Option<Value>>) -> Result<bool, RunDiag> {
        {
            // Step 2: enabling conditions.
            let vals = self.vals.clone();
            due.retain(|&ei| match &self.cm.events[ei].enable {
                Some(c) => matches!(c.eval(&self.ctx(&vals)), Ok(Value::Bool(true))),
                None => true,
            });
            // A requested event obeys its enabling condition as any other (MK-15.5).
            requested.retain(|(ei, _)| match &self.cm.events[*ei].enable {
                Some(c) => matches!(c.eval(&self.ctx(&vals)), Ok(Value::Bool(true))),
                None => true,
            });
            // A finished algorithm no longer occurs (MK-14.17, D-101).
            due.retain(|&ei| !self.finished(ei));
            requested.retain(|(ei, _)| !self.finished(*ei));
            due.sort();
            due.dedup();
            if due.is_empty() && requested.is_empty() {
                return Ok(false);
            }
            if self.n >= self.cfg.n_micro {
                return Err(self.diag(Category::Cascade, format!("more than {} microsteps at t = {}", self.cfg.n_micro, self.t), None));
            }
            // Zeno policies replace handlers at an accumulating occurrence (RC-9.3).
            let mut handled: Vec<(usize, bool, bool)> = vec![];
            let mut ops: Vec<COp> = vec![];
            for &ei in due.iter() {
                let is_crossing = matches!(self.cm.events[ei].trigger, Some(CTrigger::Crossing { .. }));
                if is_crossing && self.zeno_detected(ei) {
                    match self.cm.events[ei].zeno.as_ref().unwrap().settle.clone() {
                        None => {
                            return Err(self.diag(Category::Zeno, format!("event accumulation detected for `{}`", self.cm.events[ei].name), Some(&self.cm.events[ei].id)));
                        }
                        Some(settle) => {
                            ops.extend(settle);
                            handled.push((ei, true, false));
                        }
                    }
                } else {
                    ops.extend(self.work_of(ei)?);
                    handled.push((ei, false, false));
                }
            }
            for (ei, _) in requested.clone().iter() {
                ops.extend(self.work_of(*ei)?);
                handled.push((*ei, false, true));
            }
            let emitted = self.transition(&ops, false)?;
            for &(ei, z, rq) in &handled {
                if matches!(self.cm.events[ei].trigger, Some(CTrigger::Crossing { .. })) {
                    self.record_occurrence(ei);
                }
                let payload = self.payloads[ei].clone();
                self.run.log.push(LogEntry { event: ei, name: self.cm.events[ei].name.clone(), t: self.t, n: self.n, zeno: z, requested: rq, payload });
            }
            // Next microstep: `on(E)` for each E that occurred or was emitted at this one
            // (MK-15.3, MK-15.10, D-041), and crossings caused by jumps (RC-8.5). An event
            // carries its occurrence's payload, or the payload it was emitted with (D-050).
            let occurred: Vec<usize> = handled.iter().map(|h| h.0).collect();
            *carried = vec![None; self.cm.events.len()];
            for &ei in &occurred {
                carried[ei] = self.payloads[ei].clone();
            }
            for (ei, p) in &emitted {
                if p.is_some() {
                    carried[*ei] = p.clone();
                }
            }
            let emitted: Vec<usize> = emitted.into_iter().map(|e| e.0).collect();
            let mut next: Vec<usize> = (0..self.cm.events.len())
                .filter(|&j| matches!(self.cm.events[j].trigger, Some(CTrigger::On(src)) if emitted.contains(&src) || occurred.contains(&src)))
                .collect();
            for ei in 0..self.cm.events.len() {
                if let Some(CTrigger::Crossing { dir, .. }) = &self.cm.events[ei].trigger {
                    let before = self.run.committed[self.run.committed.len() - 2].vals.clone();
                    let (a, b) = match (self.guard_value(&before, ei), self.guard_value(&self.vals.clone(), ei)) {
                        (Ok(Some(a)), Ok(Some(b))) => (a, b),
                        _ => continue,
                    };
                    if let (Some(sa), Some(sb)) = (sign(a), sign(b)) {
                        if sa != sb && matches_dir(*dir, sa, sb) {
                            next.push(ei);
                        }
                    }
                }
            }
            *due = next;
            *requested = vec![];
        }
        Ok(true)
    }

    /// Whether event `ei` is an algorithm that has run to its end (D-101).
    fn finished(&self, ei: usize) -> bool {
        self.cm.events[ei].algorithm.as_ref().is_some_and(|a| self.vals[a.resume].num() as usize >= a.code.len())
    }

    /// The operations of an occurrence of event `ei`: its handler, or for an algorithm the
    /// changes made by running it to its next `step` (D-101).
    fn work_of(&mut self, ei: usize) -> Result<Vec<COp>, RunDiag> {
        if self.cm.events[ei].algorithm.is_none() {
            return Ok(self.cm.events[ei].handler.clone());
        }
        self.algorithm_step(ei)
    }

    /// Runs algorithm `ei` from where it stopped to its next `step` or its end, each
    /// statement reading the values the ones before it wrote (MK-14.17, RC-5.12). The result
    /// is a `set` of each stored value that changed, its resume point included, and the
    /// events it emitted: the transition then treats them as a handler's operations.
    fn algorithm_step(&mut self, ei: usize) -> Result<Vec<COp>, RunDiag> {
        let cm = self.cm;
        let ev = &cm.events[ei];
        let alg = ev.algorithm.as_ref().unwrap();
        let mut work = self.vals.clone();
        let mut emits = vec![];
        let mut pc = work[alg.resume].num() as usize;
        let mut count = 0;
        let fail = |s: &Self, st: Status| s.diag(Category::Model, format!("algorithm `{}`: {}", ev.name, st.cause), Some(&ev.id));
        while pc < alg.code.len() {
            count += 1;
            if count > prismal_kernel::ALGORITHM_INSTRUCTIONS {
                return Err(self.diag(Category::Model, format!("algorithm `{}` ran {} instructions without reaching `step` or its end", ev.name, prismal_kernel::ALGORITHM_INSTRUCTIONS), Some(&ev.id)));
            }
            match &alg.code[pc] {
                prismal_kernel::CInstr::Step => {
                    pc += 1;
                    break;
                }
                prismal_kernel::CInstr::Jump(to) => pc = *to,
                prismal_kernel::CInstr::Unless(c, to) => {
                    let v = c.eval(&self.ctx(&work)).map_err(|s| fail(self, s))?;
                    pc = if v.boolean() { pc + 1 } else { *to };
                }
                prismal_kernel::CInstr::Do(ops) => {
                    let before = work.clone();
                    self.apply_in_order(ops, &before, &mut work, &mut emits).map_err(|s| fail(self, s))?;
                    cm.update_derived(&mut work, self.t, self.cfg.t0).map_err(|s| fail(self, s))?;
                    pc += 1;
                }
            }
        }
        work[alg.resume] = Value::Num(pc as f64);
        let mut out: Vec<COp> = (0..work.len())
            .filter(|&b| !matches!(cm.bindings[b].role, Role::Derived | Role::Unknown | Role::Constant) && work[b] != self.vals[b])
            .map(|b| COp::Set { binding: b, component: None, value: CExpr::Const(work[b].clone()) })
            .collect();
        out.extend(emits);
        Ok(out)
    }

    /// Performs one statement's operations on `work`, all reading `before` (D-101).
    fn apply_in_order(&self, ops: &[COp], before: &[Value], work: &mut [Value], emits: &mut Vec<COp>) -> Result<(), Status> {
        for op in ops {
            match op {
                COp::If { cond, ops } => {
                    if cond.eval(&self.ctx(before))?.boolean() {
                        self.apply_in_order(ops, before, work, emits)?;
                    }
                }
                COp::Set { binding, component, value } => {
                    let v = value.eval(&self.ctx(before))?;
                    match component {
                        None => work[*binding] = v,
                        Some(k) => {
                            let mut a = work[*binding].arr();
                            a.v[*k] = v.num();
                            work[*binding] = match work[*binding] {
                                Value::Point(_) => Value::Point(a),
                                _ => Value::Vec(a),
                            };
                        }
                    }
                }
                COp::SetField { binding, grid, value } => work[*binding] = prismal_kernel::per_cell(value, grid, &self.ctx(before))?,
                COp::Destroy { binding } => work[*binding] = Value::Bool(false),
                COp::Emit { event, payload } => {
                    let p = match payload {
                        Some(c) => Some(CExpr::Const(c.eval(&self.ctx(before))?)),
                        None => None,
                    };
                    emits.push(COp::Emit { event: *event, payload: p });
                }
                COp::Contribute { .. } => return Err(Status::invalid("`contribute` in an algorithm")),
            }
        }
        Ok(())
    }

    /// One transition (MK-16.3): conflicts, proposed state, constraints, commit.
    /// Returns the events emitted, with their payloads. For interventions, a `reject`
    /// violation is returned as an error with category `Intervention`.
    #[allow(clippy::type_complexity)]
    fn transition(&mut self, ops: &[COp], intervention: bool) -> Result<Vec<(usize, Option<Value>)>, RunDiag> {
        let mut targets: Vec<(usize, Option<usize>)> = vec![];
        let mut destroyed: Vec<usize> = vec![];
        let mut proposed = self.vals.clone();
        let mut emitted = vec![];
        // `contribute` operations by binding: combined after all operations are read
        // (MK-14.9, MK-16.4, D-073).
        let mut contributions: Vec<(usize, String, Vec<Value>)> = vec![];
        // Conditional operations are read on the state before the transition (D-057).
        let mut flat: Vec<&COp> = vec![];
        fn performed<'o>(s: &Engine, ops: &'o [COp], out: &mut Vec<&'o COp>) -> Result<(), RunDiag> {
            for op in ops {
                match op {
                    COp::If { cond, ops } => {
                        if cond.eval(&s.ctx(&s.vals)).map_err(|st| s.diag(Category::Model, st.cause, None))?.boolean() {
                            performed(s, ops, out)?;
                        }
                    }
                    other => out.push(other),
                }
            }
            Ok(())
        }
        performed(self, ops, &mut flat)?;
        let conflict = |s: &Self, b: usize| s.diag(Category::Conflict, format!("two operations on `{}` in one transition", s.cm.bindings[b].name), None);
        for op in flat {
            match op {
                COp::If { .. } => unreachable!(),
                // MK-16.5: destroying one member twice is not a conflict.
                COp::Destroy { binding } => {
                    if targets.iter().any(|(b, _)| b == binding) {
                        return Err(conflict(self, *binding));
                    }
                    if !destroyed.contains(binding) {
                        destroyed.push(*binding);
                    }
                    proposed[*binding] = Value::Bool(false);
                }
                COp::Contribute { binding, value, combine } => {
                    if destroyed.contains(binding) || targets.iter().any(|(b, _)| b == binding) {
                        return Err(conflict(self, *binding));
                    }
                    let v = value.eval(&self.ctx(&self.vals)).map_err(|s| self.diag(Category::Model, s.cause, None))?;
                    match contributions.iter_mut().find(|c| c.0 == *binding) {
                        Some(c) => c.2.push(v),
                        None => contributions.push((*binding, combine.clone(), vec![v])),
                    }
                }
                COp::SetField { binding, grid, value } => {
                    if contributions.iter().any(|c| c.0 == *binding) || destroyed.contains(binding) || targets.iter().any(|(b, _)| b == binding) {
                        return Err(conflict(self, *binding));
                    }
                    targets.push((*binding, None));
                    let v = prismal_kernel::per_cell(value, grid, &self.ctx(&self.vals)).map_err(|s| self.diag(Category::Model, s.cause, None))?;
                    proposed[*binding] = v;
                }
                COp::Set { binding, component, value } => {
                    if contributions.iter().any(|c| c.0 == *binding) {
                        return Err(conflict(self, *binding));
                    }
                    if destroyed.contains(binding) || targets.iter().any(|(b, c)| b == binding && (c.is_none() || component.is_none() || c == component)) {
                        return Err(conflict(self, *binding));
                    }
                    targets.push((*binding, *component));
                    let v = value.eval(&self.ctx(&self.vals)).map_err(|s| self.diag(Category::Model, s.cause, None))?;
                    match component {
                        None => proposed[*binding] = v,
                        Some(k) => {
                            let mut a = proposed[*binding].arr();
                            a.v[*k] = v.num();
                            proposed[*binding] = match proposed[*binding] {
                                Value::Point(_) => Value::Point(a),
                                _ => Value::Vec(a),
                            };
                        }
                    }
                }
                COp::Emit { event, payload } => {
                    let p = match payload {
                        Some(c) => Some(c.eval(&self.ctx(&self.vals)).map_err(|s| self.diag(Category::Model, s.cause, None))?),
                        None => None,
                    };
                    emitted.push((*event, p));
                }
            }
        }
        let contributions_set: Vec<usize> = contributions.iter().map(|c| c.0).collect();
        for (b, how, vs) in contributions {
            proposed[b] = combine(&how, &self.vals[b], &vs);
        }
        // MK-16.5: an operation on a member destroyed in the same transition is a conflict. A
        // member's bindings have the identity `declaration@path` of its liveness binding's
        // path, or of a path inside it (D-057).
        for &d in &destroyed {
            let Some((_, path)) = self.cm.bindings[d].id.split_once('@') else { continue };
            let inside = |id: &str| id.split_once('@').is_some_and(|(_, p)| p == path || p.starts_with(&format!("{path}.")));
            if let Some(&(b, _)) = targets.iter().find(|(b, _)| inside(&self.cm.bindings[*b].id)) {
                return Err(self.diag(Category::Conflict, format!("`{}` is set in the transition that destroys its member", self.cm.bindings[b].name), None));
            }
        }
        // RC-5.10: a transition's result is moved onto the enforced constraints (a bob dragged
        // off its circle lands on it).
        let set: Vec<usize> = targets.iter().map(|(b, _)| *b).chain(contributions_set.iter().copied()).collect();
        self.cm.separate(&mut proposed, self.t, self.cfg.t0).map_err(|s| self.diag(Category::Model, s.cause, None))?;
        self.cm.project(&mut proposed, self.t, self.cfg.t0, &set).map_err(|s| self.diag(Category::Model, s.cause, None))?;
        self.cm.update_derived(&mut proposed, self.t, self.cfg.t0).map_err(|s| self.diag(Category::Model, s.cause, None))?;
        match self.check_constraints(&proposed, false) {
            Ok(reports) => self.run.diagnostics.extend(reports),
            Err(d) if intervention => return Err(RunDiag { category: Category::Intervention, ..d }),
            Err(d) => return Err(d),
        }
        self.vals = proposed;
        self.n += 1;
        self.commit();
        Ok(emitted)
    }

    /// Applies one logged action after the model's own iteration at this instant (RC-8.7, RC-11.1).
    fn intervene(&mut self, action: &Action) -> Result<(), RunDiag> {
        match action {
            Action::Intervene(ops) => {
                let cops = match check_intervention(self.cm, ops) {
                    Ok(c) => c,
                    Err(d) => {
                        let msg = d.iter().map(|x| x.to_string()).collect::<Vec<_>>().join("; ");
                        let rd = self.diag(Category::Intervention, msg, None);
                        self.run.rejected.push(rd);
                        return Ok(());
                    }
                };
                match self.transition(&cops, true) {
                    Ok(emitted) => {
                        let due = (0..self.cm.events.len())
                            .filter(|&j| matches!(self.cm.events[j].trigger, Some(CTrigger::On(src)) if emitted.iter().any(|e| e.0 == src)))
                            .collect();
                        let mut carried = vec![None; self.cm.events.len()];
                        for (ei, p) in emitted {
                            carried[ei] = p;
                        }
                        let r = self.iterate_with(due, vec![], carried);
                        self.payloads = vec![None; self.cm.events.len()];
                        r
                    }
                    Err(d) if d.category == Category::Intervention => {
                        self.run.rejected.push(d);
                        Ok(())
                    }
                    Err(d) => Err(d),
                }
            }
            Action::Input(id, e) => {
                let reject = |s: &mut Self, msg: String| {
                    let rd = s.diag(Category::Intervention, msg, Some(id));
                    s.run.rejected.push(rd);
                    Ok(())
                };
                let Some(&i) = self.cm.index.get(id) else { return reject(self, format!("unknown input `{id}`")) };
                if self.cm.bindings[i].role != Role::Input {
                    return reject(self, format!("`{}` is not an input (RC-11.6)", self.cm.bindings[i].name));
                }
                // The environment's value reads nothing of the model but its constants.
                let value = match compile_expr(self.cm, e, Some(&self.cm.bindings[i].ty)) {
                    Ok((c, _)) => c.eval(&Ctx { vals: &self.cm.constant_values(), der: None, t: self.t, t0: self.cfg.t0, args: &[], payloads: &[], draws: None, noise: &[], point: None }),
                    Err(d) => return reject(self, format!("the value of `{}`: {}", self.cm.bindings[i].name, d.iter().map(|x| x.message.clone()).collect::<Vec<_>>().join("; "))),
                };
                let v = match value {
                    Ok(v) => v,
                    Err(s) => return reject(self, format!("the value of `{}`: {}", self.cm.bindings[i].name, s.cause)),
                };
                self.transition(&[COp::Set { binding: i, component: None, value: CExpr::Const(v) }], false)?;
                let due = (0..self.cm.events.len()).filter(|&j| matches!(self.cm.events[j].trigger, Some(CTrigger::Input(b)) if b == i)).collect();
                self.iterate(due, vec![])
            }
            Action::Request(id) | Action::RequestWith(id, _) => {
                let Some(ei) = self.cm.events.iter().position(|e| &e.id == id) else {
                    let rd = self.diag(Category::Intervention, format!("unknown event `{id}`"), None);
                    self.run.rejected.push(rd);
                    return Ok(());
                };
                if !matches!(self.cm.events[ei].trigger, Some(CTrigger::Request)) {
                    let rd = self.diag(Category::Intervention, format!("event `{}` is not requestable (D-027)", self.cm.events[ei].name), Some(id));
                    self.run.rejected.push(rd);
                    return Ok(());
                }
                // D-050: a request supplies the payload the event declares, of its type.
                let declared = self.cm.ir.events[ei].payload.clone();
                let payload = match (action, declared) {
                    (Action::Request(_), None) => None,
                    (Action::RequestWith(_, e), Some(p)) => match compile_expr(self.cm, e, Some(&p.ty)) {
                        Ok((c, _)) => match c.eval(&Ctx { vals: &self.vals, der: None, t: self.t, t0: self.cfg.t0, args: &[], payloads: &[], draws: None, noise: &[], point: None }) {
                            Ok(v) => Some(v),
                            Err(s) => {
                                let rd = self.diag(Category::Intervention, format!("the payload of `{}`: {}", self.cm.events[ei].name, s.cause), Some(id));
                                self.run.rejected.push(rd);
                                return Ok(());
                            }
                        },
                        Err(d) => {
                            let msg = d.iter().map(|x| x.message.clone()).collect::<Vec<_>>().join("; ");
                            let rd = self.diag(Category::Intervention, format!("the payload of `{}`: {msg}", self.cm.events[ei].name), Some(id));
                            self.run.rejected.push(rd);
                            return Ok(());
                        }
                    },
                    (Action::Request(_), Some(p)) => {
                        let rd = self.diag(Category::Intervention, format!("a request of `{}` supplies its payload `{}`", self.cm.events[ei].name, p.name), Some(id));
                        self.run.rejected.push(rd);
                        return Ok(());
                    }
                    (Action::RequestWith(..), None) => {
                        let rd = self.diag(Category::Intervention, format!("`{}` declares no payload", self.cm.events[ei].name), Some(id));
                        self.run.rejected.push(rd);
                        return Ok(());
                    }
                    (Action::Intervene(_) | Action::Input(..), _) => unreachable!(),
                };
                // A request the event's enabling condition refuses is rejected with the reason:
                // a member payload names a member that is not alive, or the condition is false
                // (MK-15.5, D-059).
                if let Some(c) = &self.cm.events[ei].enable {
                    let mut payloads = vec![None; self.cm.events.len()];
                    payloads[ei] = payload.clone();
                    let ctx = Ctx { vals: &self.vals, der: None, t: self.t, t0: self.cfg.t0, args: &[], payloads: &payloads, draws: None, noise: &[], point: None };
                    let why = match c.eval(&ctx) {
                        Ok(Value::Bool(true)) => None,
                        Ok(_) => Some(self.not_enabled(ei, payload.as_ref())),
                        Err(s) => Some(format!("`{}` cannot be requested: {}", self.cm.events[ei].name, s.cause)),
                    };
                    if let Some(why) = why {
                        let rd = self.diag(Category::Intervention, why, Some(id));
                        self.run.rejected.push(rd);
                        return Ok(());
                    }
                }
                let r = self.iterate_with(vec![], vec![(ei, payload)], vec![None; self.cm.events.len()]);
                self.payloads = vec![None; self.cm.events.len()];
                r
            }
        }
    }
}

// ------------------------------------------------------------------ observation (RC section 15)

impl Run {
    /// Compiles an observation expression over this run's model.
    pub fn compile(&self, e: &Expr) -> CExpr {
        compile_expr(&self.model, e, None).unwrap_or_else(|d| panic!("observation does not check: {d:?}")).0
    }

    /// Evaluates a compiled expression on a state of this run at time `t`.
    pub fn eval_state(&self, c: &CExpr, vals: &[Value], t: f64) -> Result<Value, Status> {
        self.eval_on(c, vals, t)
    }

    /// The last simulation instant the run reached.
    pub fn end_time(&self) -> f64 {
        let c = self.committed.last().map(|c| c.t).unwrap_or(self.config.t0);
        let s = self.segments.last().map(|s| s.step.t1).unwrap_or(c);
        c.max(s)
    }

    fn eval_on(&self, c: &CExpr, vals: &[Value], t: f64) -> Result<Value, Status> {
        let der = self.model.der_values(vals, t, self.config.t0).ok();
        c.eval(&Ctx { vals, der: der.as_deref(), t, t0: self.config.t0, args: &[], payloads: &[], draws: None, noise: &[], point: None })
    }

    /// The state at time `t`: the final microstep at an event time (RC-4.3), otherwise the
    /// dense output (RC-15.1).
    pub fn state_at(&self, t: f64) -> Vec<Value> {
        if let Some(c) = self.committed.iter().rev().find(|c| c.t == t) {
            return c.vals.clone();
        }
        // Segments are in time order: the first that ends at or after `t`, found by bisection
        // (a run with noise has a segment per millisecond, D-090), is the first containing it.
        let k = self.segments.partition_point(|s| s.step.t1 < t);
        let seg = match self.segments.get(k) {
            Some(s) if s.step.t0 <= t => s,
            _ => self.segments.iter().find(|s| s.step.t0 <= t && t <= s.step.t1).unwrap_or_else(|| panic!("time {t} is outside the run")),
        };
        let mut y = vec![0.0; self.model.n_y()];
        seg.step.eval(t, &mut y);
        let mut vals = self.committed[seg.base].vals.clone();
        self.model.load_y(&mut vals, &y);
        self.model.update_derived(&mut vals, t, self.config.t0).expect("derived values");
        vals
    }

    /// `expr at(τ)` (PK-3.1).
    pub fn at(&self, t: f64, e: &Expr) -> Value {
        let c = self.compile(e);
        self.eval_on(&c, &self.state_at(t), t).expect("observation value")
    }

    /// `expr on(E)` at the final microstep of each occurrence, or at a given microstep (PK-3.1).
    pub fn on(&self, event: &str, e: &Expr, microstep: Option<u32>) -> Vec<Value> {
        let c = self.compile(e);
        self.log
            .iter()
            .filter(|l| self.model.events[l.event].name == event)
            .map(|l| {
                let state = match microstep {
                    Some(n) => self.committed.iter().find(|s| s.t == l.t && s.n == n).expect("microstep"),
                    // The event's final microstep (PK-3.1): the last state of the cascade it
                    // belongs to, before any logged action applied at the same instant.
                    None => {
                        let at = || self.committed.iter().filter(|s| s.t == l.t);
                        at().filter(|s| s.n >= l.n).take_while(|s| !s.by_action).last().or_else(|| at().last()).expect("event instant")
                    }
                };
                self.eval_on(&c, &state.vals, l.t).expect("observation value")
            })
            .collect()
    }

    /// `expr every(Δ)` from `t0` to the end of the run (PK-3.1).
    pub fn every(&self, dt: f64, e: &Expr) -> Vec<(f64, Value)> {
        let c = self.compile(e);
        let end = self.committed.last().map(|s| s.t).unwrap_or(self.config.t0);
        let mut out = vec![];
        let mut k = 0u64;
        loop {
            let t = self.config.t0 + k as f64 * dt;
            if t > end {
                break;
            }
            out.push((t, self.eval_on(&c, &self.state_at(t), t).expect("observation value")));
            k += 1;
        }
        out
    }

    /// Times of each occurrence of an event.
    pub fn times(&self, event: &str) -> Vec<f64> {
        self.log.iter().filter(|l| l.name == event).map(|l| l.t).collect()
    }

    /// The value of a binding in the final state.
    pub fn final_value(&self, binding: &str) -> Value {
        self.committed.last().unwrap().vals[self.model.idx(binding)].clone()
    }
}

/// The type of an observation expression.
pub fn observation_type(cm: &CModel, e: &Expr) -> Type {
    compile_expr(cm, e, None).expect("checks").1
}

#[cfg(test)]
mod tests {
    use super::*;
    use prismal_ir::build::*;
    use prismal_ir::{Op, Trigger, Type};
    use prismal_kernel::check_model;

    /// `emit(E)` makes `on(E)` due at the next microstep (MK-15.10); an endless cascade is
    /// stopped by the cascade limit (RC-8.3).
    #[test]
    fn cascades_and_cascade_limit() {
        let mut b = ModelBuilder::new("M");
        let x = b.state("x", Type::qty("L"), num(1.0, "m"));
        b.flow(&x, num(-1.0, "m/s"));
        let n = b.discrete("n", Type::real(), lit(0.0));
        b.event("cross", falling(r(&x)), vec![Op::Emit { event: "M.event.cross".into(), payload: None }]);
        b.event("second", Trigger::On { event: "M.event.cross".into() }, vec![set(&n, r(&n) + lit(1.0))]);
        let cm = check_model(&[], &b.finish()).unwrap();
        let out = run(&cm, Config::until(2.0));
        let names: Vec<(&str, u32)> = out.log.iter().map(|l| (l.name.as_str(), l.n)).collect();
        assert_eq!(names, vec![("cross", 1), ("second", 2)]);
        assert_eq!(out.final_value("M.n").num(), 1.0);

        let mut b = ModelBuilder::new("L");
        let x = b.state("x", Type::qty("L"), num(1.0, "m"));
        b.flow(&x, num(-1.0, "m/s"));
        b.event("ping", falling(r(&x)), vec![Op::Emit { event: "L.event.ping".into(), payload: None }]);
        b.event("pong", Trigger::On { event: "L.event.ping".into() }, vec![Op::Emit { event: "L.event.pong".into(), payload: None }]);
        b.event("back", Trigger::On { event: "L.event.pong".into() }, vec![Op::Emit { event: "L.event.ping".into(), payload: None }]);
        let cm = check_model(&[], &b.finish()).unwrap();
        let out = run(&cm, Config::until(2.0));
        match out.status {
            RunStatus::Stopped(d) => assert_eq!(d.category, Category::Cascade),
            s => panic!("expected a cascade failure, got {s:?}"),
        }
    }

    /// `every(Δ)` occurs at `from + kΔ` exactly, without drift (RC-7.9); `at(τ)` once.
    #[test]
    fn time_events() {
        let mut b = ModelBuilder::new("T");
        let x = b.state("x", Type::qty("L"), num(0.0, "m"));
        b.flow(&x, num(1.0, "m/s"));
        let n = b.discrete("n", Type::real(), lit(0.0));
        b.event("tick", Trigger::Every { period: num(0.1, "s"), from: t0() }, vec![set(&n, r(&n) + lit(1.0))]);
        b.event("once", Trigger::At { time: t0() + num(0.25, "s") }, vec![]);
        let cm = check_model(&[], &b.finish()).unwrap();
        let out = run(&cm, Config::until(1.0));
        let ticks: Vec<f64> = out.times("tick");
        assert_eq!(ticks.len(), 11, "t0, 0.1, ..., 1.0");
        for (k, t) in ticks.iter().enumerate() {
            assert_eq!(*t, k as f64 * 0.1, "tick {k}");
        }
        assert_eq!(out.times("once"), vec![0.25]);
        assert_eq!(out.final_value("T.n").num(), 11.0);
        assert!((out.at(0.25, &r(&x)).num() - 0.25).abs() < 1e-15);
    }

    /// `on(E)` is due at the microstep after `E` occurs, without an `emit` (D-041).
    #[test]
    fn on_follows_occurrence() {
        let mut b = ModelBuilder::new("O");
        let x = b.state("x", Type::qty("L"), num(1.0, "m"));
        b.flow(&x, num(-1.0, "m/s"));
        let n = b.discrete("n", Type::real(), lit(0.0));
        b.event("cross", falling(r(&x)), vec![]);
        b.event("after", Trigger::On { event: "O.event.cross".into() }, vec![set(&n, r(&n) + lit(1.0))]);
        let cm = check_model(&[], &b.finish()).unwrap();
        let out = run(&cm, Config::until(2.0));
        let names: Vec<(&str, u32)> = out.log.iter().map(|l| (l.name.as_str(), l.n)).collect();
        assert_eq!(names, vec![("cross", 1), ("after", 2)]);
        assert_eq!(out.final_value("O.n").num(), 1.0);
    }

    /// Two events setting one binding at the same instant are a conflict (MK-16.4, RC-10.2).
    #[test]
    fn same_instant_conflict() {
        let mut b = ModelBuilder::new("C");
        let x = b.state("x", Type::qty("L"), num(1.0, "m"));
        b.flow(&x, num(-1.0, "m/s"));
        let k = b.discrete("k", Type::real(), lit(0.0));
        b.event("a", falling(r(&x)), vec![set(&k, lit(1.0))]);
        b.event("b", falling(r(&x)), vec![set(&k, lit(1.0))]);
        let cm = check_model(&[], &b.finish()).unwrap();
        let out = run(&cm, Config::until(2.0));
        match out.status {
            RunStatus::Stopped(d) => assert_eq!(d.category, Category::Conflict),
            s => panic!("expected a conflict, got {s:?}"),
        }
        assert_eq!(out.committed.last().unwrap().n, 0, "no partial commit (RC-10.1)");
    }
}

/// Contributions combined with the value before the transition (MK-14.9, D-073): `sum`
/// and `product` of numbers (`sum` of vectors by component), `min` and `max` of numbers,
/// `any` and `all` of Booleans. The checker has matched each combination to its type.
fn combine(how: &str, before: &Value, vs: &[Value]) -> Value {
    match (how, before) {
        ("sum", Value::Vec(a)) => {
            let mut a = *a;
            for v in vs {
                let b = v.arr();
                for k in 0..a.n as usize {
                    a.v[k] += b.v[k];
                }
            }
            Value::Vec(a)
        }
        ("any", _) => Value::Bool(before.boolean() || vs.iter().any(|v| v.boolean())),
        ("all", _) => Value::Bool(before.boolean() && vs.iter().all(|v| v.boolean())),
        _ => {
            let f = |x: f64, y: f64| match how {
                "sum" => x + y,
                "product" => x * y,
                "min" => x.min(y),
                _ => x.max(y),
            };
            Value::Num(vs.iter().fold(before.num(), |acc, v| f(acc, v.num())))
        }
    }
}

