//! Interaction (PK section 10): controls, direct manipulation through declared inverses,
//! keyboard operation (PK-11.2), undo and redo, over an interactive session of the model.
//!
//! Every model action passes through the runtime's validation (PK-1.2). A rejected action is
//! reported to the presentation (RC-10.5) and not logged; an action the presentation does
//! not offer is refused before it reaches the model (PK-10.4).

use crate::data::{observe, Data};
use crate::frame::{subst, CKind, CRep, Frame, Projector, Tf, ViewCtx};
use crate::text::unit_text;
use crate::{PDiag, Program};
use prismal_ir::present::Presentation;
use prismal_ir::{Expr, Op, Target, Type, Unit};
use prismal_kernel::{CModel, Value};
use prismal_runtime::{Action, Config, Run, Session};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Key {
    Left,
    Right,
    Up,
    Down,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Why {
    /// Refused by the presentation; the model never saw it (PK-10.4, PK-9.8).
    Refused,
    /// Rejected by the runtime's validation (RC-10.5).
    Rejected,
}

/// An action that did not take effect, as reported to the presentation.
#[derive(Clone, Debug, PartialEq)]
pub struct Report {
    pub why: Why,
    pub message: String,
}

/// A preview during a drag (PK-10.5 to PK-10.7).
#[derive(Clone, Debug)]
pub struct Preview {
    pub action: Action,
    pub valid: bool,
    pub message: Option<String>,
}

struct Drag {
    rep: String,
    ctx: ViewCtx,
    crep: CRep,
    /// The groups enclosing a dragged member of a group, outermost first (D-043).
    groups: Vec<CRep>,
    previews: Vec<Preview>,
    last_valid: Option<(Action, Run)>,
    /// Drag mode `live` (PK-10.9, D-071): the last pointer position, to propose again as
    /// the clock advances, and how many proposals were committed during the drag.
    live: bool,
    last_p: Option<[f64; 2]>,
    committed: usize,
}

/// A literal of a value in coherent SI units, typed for `ty` (bare for dimensionless).
pub fn si_literal(v: f64, ty: &Type) -> Expr {
    match ty {
        Type::Quantity { dim } if !dim.is_none() => Expr::Num { num: v, unit: Some(Unit { text: unit_text(dim), scale: 1.0, offset: 0.0, dim: *dim }) },
        _ => Expr::Num { num: v, unit: None },
    }
}

/// An interactive presentation of a model: one session, its views and its controls.
///
/// The session's run is computed to the configuration's end. `t` is the simulation instant
/// on display (RC section 12): frames show it, and the learner's actions take effect at it
/// (RC-11.2); the trajectory after it is recomputed. A static model has one instant.
pub struct Interactive {
    /// The model and presentation, owned so that an instance outlives its program's
    /// document (HI-2.2).
    pub cm: CModel,
    pub pres: Presentation,
    pub projector: Projector,
    pub session: Session,
    /// The simulation instant on display.
    pub t: f64,
    cfg: Config,
    drag: Option<Drag>,
    /// Actions that did not take effect, in order.
    pub reports: Vec<Report>,
    /// Previews of the last drag, kept after it ends for inspection.
    pub last_previews: Vec<Preview>,
    /// The document as written and the working capacities of its collections declared
    /// without a limit, to grow them when the run fills one (D-066).
    source: prismal_ir::Document,
    capacities: prismal_ir::elaborate::Capacities,
}

impl Interactive {
    pub fn new(prog: &Program, presentation: &str, cfg: Config) -> Result<Interactive, Vec<PDiag>> {
        let pres = prog.presentation(presentation).clone();
        let cm = prog.model(&pres.model).clone();
        let projector = Projector::new(&cm, &pres)?;
        let session = Session::new(cm.clone(), cfg.clone());
        let t = cfg.t0;
        let mut i = Interactive { cm, pres, projector, session, t, cfg, drag: None, reports: vec![], last_previews: vec![], source: prog.source.clone(), capacities: prog.capacities.clone() };
        i.grow();
        Ok(i)
    }

    /// When the run has filled a collection declared without a limit (D-066), the program
    /// is elaborated again with room for twice as many members and the session's actions
    /// are committed again, in order, on its new run. Identities of members are kept, so
    /// actions naming members still name them.
    fn grow(&mut self) {
        loop {
            let Some(part) = crate::overflow(&self.source, &self.session.current) else { return };
            let Some(caps) = crate::grown_capacities(&self.source, &self.capacities, &part) else { return };
            let Ok(next) = Program::sized(self.source.clone(), caps) else { return };
            let Some(cm) = next.models.iter().find(|m| m.ir.id == self.cm.ir.id).cloned() else { return };
            let pres = next.presentation(&self.pres.id).clone();
            let Ok(projector) = Projector::new(&cm, &pres) else { return };
            let log = self.session.log().to_vec();
            let mut session = Session::new(cm.clone(), self.cfg.clone());
            for s in log {
                let _ = session.commit(s.t, s.action);
            }
            self.capacities = next.capacities.clone();
            (self.cm, self.pres, self.projector, self.session) = (cm, pres, projector, session);
        }
    }

    /// The end of the current run: the last instant that can be shown.
    pub fn end_time(&self) -> f64 {
        self.session.current.end_time()
    }

    /// Makes `t` the instant on display (`seek`, and `play` step by step), within the run.
    /// Returns the instant shown. Never changes the trajectory (RC-12.2).
    pub fn seek(&mut self, t: f64) -> f64 {
        // A live drag steers while the clock runs (PK-10.9, D-071): the proposal pending at
        // the instant left is committed there, and the pointer proposes again at the new one.
        if self.drag.as_ref().is_some_and(|d| d.live) && t > self.t {
            self.commit_live();
            self.t = t.clamp(self.cfg.t0, self.end_time());
            if let Some(p) = self.drag.as_ref().and_then(|d| d.last_p) {
                self.pointer_move(p);
            }
            return self.t;
        }
        self.t = t.clamp(self.cfg.t0, self.end_time());
        self.t
    }

    /// Commits a live drag's pending proposal at the instant shown, as its own intervention.
    fn commit_live(&mut self) {
        let Some((action, _)) = self.drag.as_mut().and_then(|d| d.last_valid.take()) else { return };
        if self.commit(action).is_ok() {
            if let Some(d) = self.drag.as_mut() {
                d.committed += 1;
            }
        }
    }

    /// Whether a drag in progress is `live`.
    pub fn drag_live(&self) -> bool {
        self.drag.as_ref().is_some_and(|d| d.live)
    }

    /// A new run with the same configuration and an empty log, shown at `t0` (RC section 12).
    pub fn reset(&mut self) {
        self.drag = None;
        self.session = Session::new(self.cm.clone(), self.cfg.clone());
        self.t = self.cfg.t0;
        self.grow();
    }

    /// A new run from `t0` with the parameters as they are at the instant shown (D-085): the
    /// learner's settings stay, everything else starts again. `reset` goes back to the
    /// presentation's own starting values.
    pub fn restart(&mut self) {
        self.drag = None;
        let mut cfg = crate::restarted(&self.session.current, self.t);
        cfg.t_end = self.cfg.t_end;
        self.session = Session::new(self.cm.clone(), cfg);
        self.t = self.cfg.t0;
        self.grow();
    }

    /// A restart with the next seed (D-084, D-085): another outcome of the same model with the
    /// learner's settings; `reset` and `restart` keep the seed.
    pub fn reseed(&mut self) {
        let seed = self.session.current.config.seed.wrapping_add(1);
        self.cfg.seed = seed;
        self.drag = None;
        let mut cfg = crate::restarted(&self.session.current, self.t);
        cfg.t_end = self.cfg.t_end;
        cfg.seed = seed;
        self.session = Session::new(self.cm.clone(), cfg);
        self.t = self.cfg.t0;
        self.grow();
    }

    /// The seed of the run shown (RC-13.1).
    pub fn seed(&self) -> u64 {
        self.cfg.seed
    }

    /// Names of the event occurrences in `(a, b]`, in order (announcements, PK-11.3a).
    pub fn events_between(&self, a: f64, b: f64) -> Vec<String> {
        self.session.current.log.iter().filter(|l| l.t > a && l.t <= b).map(|l| l.name.clone()).collect()
    }

    fn rep(&self, name: &str) -> Option<(ViewCtx, CRep)> {
        self.rep_in(name).map(|(ctx, r, _)| (ctx, r))
    }

    /// A representation by identity or name, searched in groups too, with the groups that
    /// enclose it, outermost first.
    fn rep_in(&self, name: &str) -> Option<(ViewCtx, CRep, Vec<CRep>)> {
        fn find(reps: &[CRep], name: &str, chain: &mut Vec<CRep>) -> Option<CRep> {
            for r in reps {
                if r.rep.id == name || r.rep.name.as_deref() == Some(name) || r.rep.id.ends_with(&format!(".{name}")) {
                    return Some(r.clone());
                }
                if let CKind::Group { members, .. } = &r.kind {
                    chain.push(r.clone());
                    if let Some(m) = find(members, name, chain) {
                        return Some(m);
                    }
                    chain.pop();
                }
            }
            None
        }
        for (_, ctx, reps) in &self.projector.views {
            let mut chain = vec![];
            if let Some(r) = find(reps, name, &mut chain) {
                return Some((ctx.clone(), r, chain));
            }
        }
        None
    }

    /// The transform of the groups enclosing a representation at the instant shown.
    fn groups_tf(&self, groups: &[CRep]) -> Option<Tf> {
        let run = &self.session.current;
        Tf::of_groups(groups, run, &run.state_at(self.t), self.t)
    }

    fn report<T>(&mut self, why: Why, message: String) -> Result<T, Report> {
        let r = Report { why, message };
        self.reports.push(r.clone());
        Err(r)
    }

    fn commit(&mut self, action: Action) -> Result<(), Report> {
        let t = self.t;
        match self.session.commit(t, action) {
            Ok(()) => {
                self.grow();
                Ok(())
            }
            Err(d) => self.report(Why::Rejected, d.message),
        }
    }

    fn current_value(&self, idx: usize) -> Value {
        self.session.current.state_at(self.t)[idx].clone()
    }

    /// Sets a control to a value (a pointer on a slider, a typed number).
    pub fn set_control(&mut self, rep: &str, value: Expr) -> Result<(), Report> {
        let Some((_, r)) = self.rep(rep) else { return self.report(Why::Refused, format!("no control `{rep}`")) };
        let CKind::Control { binding, .. } = &r.kind else { return self.report(Why::Refused, format!("`{rep}` is not a control")) };
        let op = Op::Set { target: Target::of(binding.clone()), value };
        self.commit(Action::Intervene(vec![op]))
    }

    /// Presses a button: requests its event at the instant shown (D-027, RC-11.4a), or
    /// applies its runtime control: `reset`, `undo`, `redo` (D-069), `reseed` (D-084), `restart` (D-085).
    pub fn press(&mut self, rep: &str) -> Result<(), Report> {
        let Some((_, r)) = self.rep(rep) else { return self.report(Why::Refused, format!("no button `{rep}`")) };
        match &r.kind {
            CKind::Button { event, payload, .. } => {
                let e = event.clone();
                self.commit(match payload {
                    Some(v) => Action::RequestWith(e, v.clone()),
                    None => Action::Request(e),
                })
            }
            // A runtime control (D-069).
            CKind::RunButton { control, .. } => {
                match control.as_str() {
                    "reset" => self.reset(),
                    "reseed" => self.reseed(),
                    "restart" => self.restart(),
                    "undo" => self.undo(),
                    _ => self.redo(),
                }
                Ok(())
            }
            _ => self.report(Why::Refused, format!("`{rep}` is not a button")),
        }
    }

    /// Clicks or activates a representation that requests an event (D-059): the request is
    /// made at the instant shown, with the payload the representation gives, typically its
    /// member.
    pub fn click(&mut self, rep: &str) -> Result<(), Report> {
        let Some((_, r)) = self.rep(rep) else { return self.report(Why::Refused, format!("nothing named `{rep}`")) };
        let Some(c) = r.rep.click.clone() else { return self.report(Why::Refused, format!("`{rep}` does nothing when clicked")) };
        let run = &self.session.current;
        if !r.shown(run, &run.state_at(self.t), self.t) {
            return self.report(Why::Refused, format!("`{rep}` is not shown now: its member is not alive"));
        }
        self.commit(match c.payload {
            Some(v) => Action::RequestWith(c.event, v),
            None => Action::Request(c.event),
        })
    }

    /// Clicks a point of a view where no representation takes the click (D-060): the view's
    /// `on click as p request E(p)` requests its event with the point, `p` in view
    /// coordinates as a drag reads them (PK-10.5).
    pub fn click_at(&mut self, view: &str, p: [f64; 2]) -> Result<(), Report> {
        let found = self.projector.views.iter().find(|v| v.0 == view || v.0.ends_with(&format!(".view.{view}"))).map(|v| (v.0.clone(), v.1.clone()));
        let Some((id, ctx)) = found else { return self.report(Why::Refused, format!("no view `{view}`")) };
        let Some(c) = self.projector.clicks.iter().find(|c| c.0 == id).map(|c| c.1.clone()) else {
            return self.report(Why::Refused, format!("`{view}` does nothing when a point of it is clicked"));
        };
        let point = ctx.pointer(p);
        self.commit(match c.payload {
            Some(v) => Action::RequestWith(c.event, subst(&v, &point)),
            None => Action::Request(c.event),
        })
    }

    /// Requests an event declared `on request` at the instant shown, with its payload when it
    /// declares one (D-027, D-050). `event` is the event's identity or name.
    pub fn request(&mut self, event: &str, payload: Option<Expr>) -> Result<(), Report> {
        let found = self.cm.ir.events.iter().find(|e| e.id == event || e.name == event).map(|e| e.id.clone());
        let Some(id) = found else { return self.report(Why::Refused, format!("no event `{event}`")) };
        self.commit(match payload {
            Some(v) => Action::RequestWith(id, v),
            None => Action::Request(id),
        })
    }

    /// The environment supplies a new value of an input at the instant shown (RC-11.6, D-051).
    /// `input` is the binding's identity or name.
    pub fn set_input(&mut self, input: &str, value: Expr) -> Result<(), Report> {
        let found = self.cm.ir.bindings.iter().find(|b| (b.id == input || b.name == input) && b.role == prismal_ir::Role::Input).map(|b| b.id.clone());
        match found {
            Some(id) => self.commit(Action::Input(id, value)),
            None => self.report(Why::Refused, format!("`{input}` is not an input of the model")),
        }
    }

    /// Sets the control of a kind that targets a binding (learner scripts: `set slider a = 2`).
    pub fn set_control_of(&mut self, control: &str, binding: &str, value: Expr) -> Result<(), Report> {
        let found = self.projector.views.iter().flat_map(|v| v.2.iter()).find(|r| matches!(&r.kind, CKind::Control { control: c, binding: b, .. } if c == control && b == binding));
        match found.map(|r| r.rep.id.clone()) {
            Some(id) => self.set_control(&id, value),
            None => self.report(Why::Refused, format!("no {control} for `{binding}` in this presentation (PK-10.4)")),
        }
    }

    /// A key press on a focused control or draggable representation (PK-11.2).
    pub fn key(&mut self, rep: &str, key: Key) -> Result<(), Report> {
        let Some((ctx, r, groups)) = self.rep_in(rep) else { return self.report(Why::Refused, format!("nothing named `{rep}`")) };
        let sign = if matches!(key, Key::Right | Key::Up) { 1.0 } else { -1.0 };
        match &r.kind {
            CKind::Control { idx, min, max, step, .. } => {
                let step = step.or(min.zip(*max).map(|(a, b)| (b - a) / 100.0)).unwrap_or(1.0);
                let Value::Num(v) = self.current_value(*idx) else { return self.report(Why::Refused, "not a numeric control".into()) };
                let ty = self.cm.bindings[*idx].ty.clone();
                self.set_control(rep, si_literal(v + sign * step, &ty))
            }
            _ if r.rep.inverse.is_some() => {
                let Some(at) = self.part_position(&ctx, &r, &groups) else { return self.report(Why::Refused, format!("`{rep}` has no position")) };
                let step = keyboard_step(&ctx, &r);
                let vertical = matches!(key, Key::Up | Key::Down);
                let mut to = at;
                match &ctx {
                    ViewCtx::Spatial { y_up, .. } if vertical => to[1] += if *y_up { -sign } else { sign } * step.1,
                    _ if vertical => to[1] += sign * step.1,
                    _ => to[0] += sign * step.0,
                }
                let part = r.rep.inverse.as_ref().and_then(|i| i.part.clone());
                self.pointer_down(rep, part.as_deref())?;
                self.pointer_move(to);
                self.pointer_up().map(|_| ())
            }
            _ => self.report(Why::Refused, format!("`{rep}` has no keyboard operation")),
        }
    }

    /// The view position of the part a representation is dragged by: a marker's position,
    /// an arrow's head.
    fn part_position(&self, ctx: &ViewCtx, r: &CRep, groups: &[CRep]) -> Option<[f64; 2]> {
        let run = &self.session.current;
        let t = self.t;
        let vals = run.state_at(t);
        // In a group, values are in the group's frame (D-043).
        let tf = self.groups_tf(groups)?;
        let eval = |c| run.eval_state(c, &vals, t).ok().map(|v| tf.apply(v));
        match &r.kind {
            CKind::Marker { pos, .. } => eval(pos).map(|p| ctx.to_view(&crate::flat(&p))),
            CKind::Arrow { vec, from, px_per_unit, .. } => {
                let base = match from {
                    Some(f) => crate::flat(&eval(f)?),
                    None => crate::flat(&tf.apply(prismal_kernel::Value::Point(prismal_kernel::Arr::from_slice(&[0.0, 0.0])))),
                };
                let v = crate::flat(&eval(vec)?);
                let b = ctx.to_view(&base);
                let y_up = matches!(ctx, ViewCtx::Spatial { y_up: true, .. });
                Some([b[0] + v[0] * px_per_unit, b[1] + if y_up { -v[1] } else { v[1] } * px_per_unit])
            }
            _ => None,
        }
    }

    /// Begins a drag on a representation, or one of its parts (PK-10.5).
    pub fn pointer_down(&mut self, rep: &str, part: Option<&str>) -> Result<(), Report> {
        let Some((ctx, r, groups)) = self.rep_in(rep) else { return self.report(Why::Refused, format!("nothing named `{rep}`")) };
        let Some(inv) = &r.rep.inverse else { return self.report(Why::Refused, format!("`{rep}` declares no inverse, so it cannot be dragged (PK-5.6)")) };
        if inv.part.as_deref() != part {
            return self.report(Why::Refused, format!("`{rep}` is dragged by {}", inv.part.as_deref().map(|p| format!("its {p}")).unwrap_or("its body".into())));
        }
        // A representation of a member not alive is not drawn, and cannot be dragged (D-059).
        let run = &self.session.current;
        if !r.shown(run, &run.state_at(self.t), self.t) {
            return self.report(Why::Refused, format!("`{rep}` is not shown now: its member is not alive"));
        }
        let live = inv.live;
        self.drag = Some(Drag { rep: r.rep.id.clone(), ctx, crep: r, groups, previews: vec![], last_valid: None, live, last_p: None, committed: 0 });
        Ok(())
    }

    /// Moves the pointer during a drag: the inverse proposes values, which are validated and
    /// previewed but not committed (PK-10.6, PK-10.7). Returns whether the proposal is valid.
    pub fn pointer_move(&mut self, p: [f64; 2]) -> bool {
        let Some(drag) = self.drag.as_mut() else { return false };
        drag.last_p = Some(p);
        let drag = self.drag.as_ref().unwrap();
        let inv = drag.crep.rep.inverse.as_ref().unwrap();
        // A member of a group is dragged in the group's frame, where it is written (D-043).
        let gesture = if drag.groups.is_empty() {
            drag.ctx.pointer(p)
        } else {
            match self.groups_tf(&drag.groups) {
                Some(tf) => drag.ctx.pointer_in(p, &tf),
                None => return false,
            }
        };
        let ops = inv.proposals.iter().map(|pr| Op::Set { target: Target::of(pr.target.clone()), value: subst(&pr.value, &gesture) }).collect();
        let action = Action::Intervene(ops);
        let t = self.t;
        let mut result = self.session.propose(t, action.clone());
        // A proposal that ends the member dragged is not valid (D-059).
        if let Ok(run) = &result {
            let crep = &self.drag.as_ref().unwrap().crep;
            if !crep.shown(run, &run.state_at(t), t) {
                let message = format!("`{}` would stop being alive", crep.rep.name.as_deref().unwrap_or(&crep.rep.id));
                result = Err(prismal_runtime::RunDiag { category: prismal_runtime::Category::Intervention, message, element: None, t, n: 0 });
            }
        }
        let drag = self.drag.as_mut().unwrap();
        match result {
            Ok(run) => {
                drag.previews.push(Preview { action: action.clone(), valid: true, message: None });
                drag.last_valid = Some((action, run));
                true
            }
            Err(d) => {
                drag.previews.push(Preview { action, valid: false, message: Some(d.message) });
                false
            }
        }
    }

    /// The latest preview of the drag in progress.
    pub fn preview(&self) -> Option<&Preview> {
        self.drag.as_ref().and_then(|d| d.previews.last())
    }

    /// Ends a drag: the last valid proposal is committed as one intervention (PK-10.8).
    /// Returns whether anything was committed.
    pub fn pointer_up(&mut self) -> Result<bool, Report> {
        let Some(drag) = self.drag.take() else { return Ok(false) };
        self.last_previews = drag.previews;
        match drag.last_valid {
            Some((action, _)) => self.commit(action).map(|_| true),
            None => Ok(drag.committed > 0),
        }
    }

    /// Cancels a drag: nothing is committed.
    pub fn cancel(&mut self) {
        if let Some(d) = self.drag.take() {
            self.last_previews = d.previews;
        }
    }

    pub fn undo(&mut self) {
        self.session.undo();
    }

    pub fn redo(&mut self) {
        self.session.redo();
    }

    /// An intervention submitted directly through the interaction path. The presentation
    /// refuses targets it does not offer, and targets the model does not allow (PK-10.4).
    pub fn submit(&mut self, ops: Vec<Op>) -> Result<(), Report> {
        for op in &ops {
            let Op::Set { target, .. } = op else { return self.report(Why::Refused, "only `set` interventions".into()) };
            let Some(&i) = self.cm.index.get(&target.binding) else { return self.report(Why::Refused, format!("unknown binding `{}`", target.binding)) };
            if !self.cm.bindings[i].intervenable {
                return self.report(Why::Refused, format!("`{}` is not intervenable (D-023, RC-11.4)", self.cm.bindings[i].name));
            }
            let offered = self.projector.views.iter().flat_map(|v| v.2.iter()).any(|r| {
                matches!(&r.kind, CKind::Control { binding, .. } if *binding == target.binding)
                    || r.rep.inverse.as_ref().is_some_and(|inv| inv.proposals.iter().any(|p| p.target == target.binding))
            });
            if !offered {
                return self.report(Why::Refused, format!("`{}` is not offered by this presentation (PK-10.4)", self.cm.bindings[i].name));
            }
        }
        self.commit(Action::Intervene(ops))
    }

    /// The start of the session's runs.
    pub fn cfg_t0(&self) -> f64 {
        self.cfg.t0
    }

    /// The frame at another instant of the committed run, without changing the display.
    pub fn frame_at(&self, t: f64) -> Frame {
        let run = &self.session.current;
        let t = t.clamp(self.cfg.t0, run.end_time());
        let vals = run.state_at(t);
        let (views, overlay) = self.projector.frame(&self.cm, run, &vals, t, &[]);
        Frame { time: t, run: "session".into(), t, views, overlay, captions: vec![], announcements: vec![] }
    }

    /// The frame shown now: the committed state, or during a drag the last valid preview,
    /// with the dragged representation marked valid or invalid (PK-10.6).
    pub fn frame(&self) -> Frame {
        let (run, valid) = match &self.drag {
            Some(d) => (d.last_valid.as_ref().map(|x| &x.1).unwrap_or(&self.session.current), d.previews.last().map(|p| p.valid)),
            None => (&self.session.current, None),
        };
        let t = self.t.min(run.end_time());
        let vals = run.state_at(t);
        let (mut views, overlay) = self.projector.frame(&self.cm, run, &vals, t, &[]);
        if let Some(d) = &self.drag {
            for r in views.iter_mut().flat_map(|v| v.reps.iter_mut()).filter(|r| r.id == d.rep) {
                r.valid = valid;
            }
        }
        Frame { time: t, run: "session".into(), t, views, overlay, captions: vec![], announcements: vec![] }
    }

    /// The current data of an observation of the presentation.
    pub fn observe(&self, name: &str) -> Result<Data, String> {
        let o = self.pres.observations.iter().find(|o| o.name == name).ok_or(format!("no observation `{name}`"))?;
        observe(&self.cm, &self.session.current, o, self.session.log())
    }
}

/// The keyboard step of a draggable representation, in view units: its `step` property, or
/// 1/100 of each plot axis span, or 10 px in a spatial view (PK-11.2).
pub fn keyboard_step(ctx: &ViewCtx, r: &CRep) -> (f64, f64) {
    if let Some(prismal_ir::present::Arg::Expr { expr: Expr::Num { num, unit: None } }) = r.rep.prop("step") {
        return (*num, *num);
    }
    match ctx {
        ViewCtx::Plot { x, y, .. } => ((x.1 - x.0) / 100.0, (y.1 - y.0) / 100.0),
        _ => (10.0, 10.0),
    }
}
