//! The explanation timeline and its lesson run (PK sections 8 and 9).
//!
//! Playing a timeline is deterministic (PK-8.7): given the model, the lesson run's
//! configuration, the presentation, the medium and the learner's inputs, it produces the
//! beat timings, the time mapping from presentation time to simulation time, every run the
//! lesson used (the lesson run and the learner's branches, each as a sequence of versions),
//! captions, announcements, and refusals. Frames at any presentation instant are projected
//! from that record.
//!
//! Runs are never changed in place: an intervention or request makes a new version of the
//! run, recomputed from its start with the action in its log (RC-11.1). Segments of the
//! time mapping refer to the version they showed, so earlier frames never change.

use crate::frame::{compile_rep, CRep, Frame, Projector, ViewCtx};
use crate::{number, PDiag, Program};
use prismal_ir::present::{Action as TAction, Animated, LearnerInput, Presentation, RevealStyle};
use prismal_ir::{Id, Op};
use prismal_kernel::{CModel, Value};
use prismal_runtime::{resume, run, Action, Config, Run, Scheduled};
use crate::frame::{each_rep_mut, retain_reps, Shape};
use serde::Serialize;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Medium {
    /// Interactive playback: explore beats open for the learner (D-018).
    Interactive,
    /// Linear media: explore beats use their fallback (PK-9.10).
    Video,
}

/// A learner input at a presentation instant (a learner script, README of the programs).
#[derive(Clone, Debug)]
pub struct Input {
    pub at: f64,
    pub input: LearnerInput,
}

/// One version of a run of the lesson.
#[derive(Clone, Debug)]
pub struct RunVersion {
    /// `lesson`, or `branch n` for the n-th learner branch (RC-12.3).
    pub lineage: String,
    pub config: Config,
    pub run: Run,
}

/// A piece of the time mapping (PK-8.2): from `p0` to `p1` presentation time, run version
/// `run` shows simulation time `s0 + rate (p - p0)`.
#[derive(Clone, Debug, Serialize)]
pub struct Segment {
    pub p0: f64,
    pub p1: f64,
    pub run: usize,
    pub s0: f64,
    pub rate: f64,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct BeatTime {
    pub id: Id,
    pub name: String,
    pub start: f64,
    pub end: f64,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct Caption {
    /// The cue's name, which hosts match voice recordings by (D-053): the name of the beat
    /// that narrates it, and `beat.2`, `beat.3` ... for its later narrations.
    pub cue: String,
    pub text: String,
    pub start: f64,
    pub end: f64,
}

/// An event occurrence announced to assistive technology when playback passes it (PK-11.3).
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct Announcement {
    pub at: f64,
    pub run: usize,
    pub event: String,
    pub t: f64,
}

#[derive(Clone, Debug)]
pub struct Shown {
    pub from: f64,
    pub until: f64,
    pub view: Option<Id>,
    pub rep: CRep,
    /// A reveal: its style, duration and easing (D-042, D-095).
    pub reveal: Option<(RevealStyle, f64, prismal_ir::present::Ease)>,
}

/// A camera move (D-042): from presentation instant `from`, over `duration`.
#[derive(Clone, Debug)]
pub struct CameraCue {
    pub view: Id,
    pub from: f64,
    pub duration: f64,
    pub center: Option<prismal_kernel::CExpr>,
    pub zoom: Option<f64>,
    /// The easing of the move (D-095).
    pub ease: prismal_ir::present::Ease,
}

/// The state of a representation `k` of the way through a reveal (D-042): a path is drawn
/// up to `k`, anything else fades in; a group's members are handled one by one.
fn reveal(r: &mut crate::frame::RepFrame, style: RevealStyle, k: f64) {
    let path = matches!(r.shape, Shape::Polyline { .. } | Shape::Polygon { .. } | Shape::Ellipse { .. } | Shape::Segment { .. } | Shape::Arrow { .. });
    if style == RevealStyle::Draw && path {
        r.drawn = Some(k);
    } else if !matches!(r.shape, Shape::Group { .. }) || style == RevealStyle::Fade {
        r.opacity = Some(k);
    }
}

/// Moves a shape by `d` in view coordinates (D-068); text, formulas and controls are not
/// moved.
fn translate(sh: &mut Shape, d: [f64; 2]) {
    let mv = |q: &mut [f64; 2]| {
        q[0] += d[0];
        q[1] += d[1];
    };
    match sh {
        Shape::Point { at } => mv(at),
        Shape::Arrow { from, to } | Shape::Segment { from, to } => {
            mv(from);
            mv(to);
        }
        Shape::Polyline { points } | Shape::Polygon { points } => points.iter_mut().for_each(mv),
        Shape::Ellipse { center, .. } => mv(center),
        Shape::Group { members } => members.iter_mut().for_each(|m| translate(&mut m.shape, d)),
        _ => {}
    }
}

/// The points of a drawn shape, and whether it is closed: what a morph moves (D-075).
/// Ellipses and arcs are sampled.
fn outline(sh: &Shape) -> Option<(Vec<[f64; 2]>, bool)> {
    Some(match sh {
        Shape::Point { at } => (vec![*at], false),
        Shape::Arrow { from, to } | Shape::Segment { from, to } => (vec![*from, *to], false),
        Shape::Polyline { points } => (points.clone(), false),
        Shape::Polygon { points } => (points.clone(), true),
        Shape::Ellipse { center, radii, rotation, start, sweep, closed } => {
            let n = 72;
            let (s, c) = rotation.sin_cos();
            let at = |a: f64| {
                let (x, y) = (radii[0] * a.cos(), radii[1] * a.sin());
                [center[0] + c * x - s * y, center[1] + s * x + c * y]
            };
            let count = if *closed { n } else { n + 1 };
            ((0..count).map(|i| at(start + sweep * i as f64 / n as f64)).collect(), *closed)
        }
        _ => return None,
    })
}

/// `n` points spaced evenly along a path by length; around a closed one, starting at its
/// first point.
fn resample(pts: &[[f64; 2]], closed: bool, n: usize) -> Vec<[f64; 2]> {
    let mut path = pts.to_vec();
    if closed && !pts.is_empty() {
        path.push(pts[0]);
    }
    let seg: Vec<f64> = path.windows(2).map(|w| (w[1][0] - w[0][0]).hypot(w[1][1] - w[0][1])).collect();
    let total: f64 = seg.iter().sum();
    if path.len() < 2 || total <= 0.0 {
        return vec![pts.first().copied().unwrap_or([0.0, 0.0]); n];
    }
    let step = if closed { total / n as f64 } else { total / (n - 1).max(1) as f64 };
    let (mut out, mut i, mut acc) = (vec![], 0, 0.0);
    for j in 0..n {
        let d = (step * j as f64).min(total);
        while i + 1 < seg.len() && acc + seg[i] < d {
            acc += seg[i];
            i += 1;
        }
        let f = if seg[i] > 0.0 { ((d - acc) / seg[i]).clamp(0.0, 1.0) } else { 0.0 };
        out.push([path[i][0] + (path[i + 1][0] - path[i][0]) * f, path[i][1] + (path[i + 1][1] - path[i][1]) * f]);
    }
    out
}

/// The shape `k` of the way from `a` to the shape `b` (D-075). Shapes of one form move point
/// by point; others are resampled to the same number of points, a closed one turned so
/// that each point travels least, and drawn as a path until the morph ends in `b` itself.
/// Shapes without points (text) switch halfway.
fn morph(a: &Shape, b: &Shape, k: f64) -> Shape {
    if k >= 1.0 {
        return b.clone();
    }
    if let Some(s) = lerp_shape(a, b, k) {
        return s;
    }
    let (Some((pa, ca)), Some((pb, cb))) = (outline(a), outline(b)) else {
        return if k >= 0.5 { b.clone() } else { a.clone() };
    };
    let n = pa.len().max(pb.len()).max(64);
    let ra = resample(&pa, ca, n);
    let mut rb = resample(&pb, cb, n);
    if ca && cb {
        let cost = |s: usize| (0..n).map(|i| (ra[i][0] - rb[(i + s) % n][0]).powi(2) + (ra[i][1] - rb[(i + s) % n][1]).powi(2)).sum::<f64>();
        let best = (0..n).min_by(|x, y| cost(*x).total_cmp(&cost(*y))).unwrap_or(0);
        rb.rotate_left(best);
    }
    let points: Vec<[f64; 2]> = ra.iter().zip(&rb).map(|(x, y)| [x[0] + (y[0] - x[0]) * k, x[1] + (y[1] - x[1]) * k]).collect();
    if ca && cb {
        Shape::Polygon { points }
    } else {
        Shape::Polyline { points }
    }
}

/// Scales a drawn shape by `s` about its center (D-075): an arrow about its base, a group
/// about the center of its members; a point keeps its place.
fn scale_shape(sh: &mut Shape, s: f64) {
    fn anchors(sh: &Shape, out: &mut Vec<[f64; 2]>) {
        match sh {
            Shape::Point { at } => out.push(*at),
            Shape::Arrow { from, to } | Shape::Segment { from, to } => out.extend([*from, *to]),
            Shape::Polyline { points } | Shape::Polygon { points } => out.extend(points.iter().copied()),
            Shape::Ellipse { center, .. } => out.push(*center),
            Shape::Group { members } => members.iter().for_each(|m| anchors(&m.shape, out)),
            _ => {}
        }
    }
    fn about(sh: &mut Shape, c: [f64; 2], s: f64) {
        let f = |q: &mut [f64; 2]| *q = [c[0] + (q[0] - c[0]) * s, c[1] + (q[1] - c[1]) * s];
        match sh {
            Shape::Point { at } => f(at),
            Shape::Arrow { from, to } | Shape::Segment { from, to } => {
                f(from);
                f(to);
            }
            Shape::Polyline { points } | Shape::Polygon { points } => points.iter_mut().for_each(f),
            Shape::Ellipse { center, radii, .. } => {
                f(center);
                radii[0] *= s;
                radii[1] *= s;
            }
            Shape::Group { members } => members.iter_mut().for_each(|m| about(&mut m.shape, c, s)),
            _ => {}
        }
    }
    let c = match sh {
        Shape::Arrow { from, .. } => *from,
        _ => {
            let mut pts = vec![];
            anchors(sh, &mut pts);
            if pts.is_empty() {
                return;
            }
            let (mut lo, mut hi) = (pts[0], pts[0]);
            for q in &pts {
                lo = [lo[0].min(q[0]), lo[1].min(q[1])];
                hi = [hi[0].max(q[0]), hi[1].max(q[1])];
            }
            [(lo[0] + hi[0]) / 2.0, (lo[1] + hi[1]) / 2.0]
        }
    };
    about(sh, c, s);
}

/// The shape `k` of the way from `a` to `b` (a `bind` handing back, D-068), when both have
/// the same form; `None` otherwise.
fn lerp_shape(a: &Shape, b: &Shape, k: f64) -> Option<Shape> {
    let l = |x: f64, y: f64| x + (y - x) * k;
    let lp = |x: &[f64; 2], y: &[f64; 2]| [l(x[0], y[0]), l(x[1], y[1])];
    let pts = |x: &[[f64; 2]], y: &[[f64; 2]]| (x.len() == y.len()).then(|| x.iter().zip(y).map(|(p, q)| lp(p, q)).collect::<Vec<_>>());
    Some(match (a, b) {
        (Shape::Point { at: x }, Shape::Point { at: y }) => Shape::Point { at: lp(x, y) },
        (Shape::Arrow { from: f, to: t }, Shape::Arrow { from: g, to: u }) => Shape::Arrow { from: lp(f, g), to: lp(t, u) },
        (Shape::Segment { from: f, to: t }, Shape::Segment { from: g, to: u }) => Shape::Segment { from: lp(f, g), to: lp(t, u) },
        (Shape::Polyline { points: x }, Shape::Polyline { points: y }) => Shape::Polyline { points: pts(x, y)? },
        (Shape::Polygon { points: x }, Shape::Polygon { points: y }) => Shape::Polygon { points: pts(x, y)? },
        (Shape::Ellipse { center: c, radii: r, rotation: o, start: s, sweep: w, closed }, Shape::Ellipse { center: c2, radii: r2, rotation: o2, start: s2, sweep: w2, .. }) => {
            Shape::Ellipse { center: lp(c, c2), radii: lp(r, r2), rotation: l(*o, *o2), start: l(*s, *s2), sweep: l(*w, *w2), closed: *closed }
        }
        (Shape::Group { members: x }, Shape::Group { members: y }) if x.len() == y.len() => {
            let mut out = y.clone();
            for (m, (p, q)) in out.iter_mut().zip(x.iter().zip(y)) {
                m.shape = lerp_shape(&p.shape, &q.shape, k)?;
            }
            Shape::Group { members: out }
        }
        _ if k <= 0.0 => a.clone(),
        _ if k >= 1.0 => b.clone(),
        _ => return None,
    })
}

/// Smooth start and end of an animation (PK-8.4): `3k² - 2k³`.
pub fn ease(k: f64) -> f64 {
    let k = k.clamp(0.0, 1.0);
    k * k * (3.0 - 2.0 * k)
}

/// A continue point of the lesson (PK-9.2, PK-9.8, D-067): a `wait learner` or an explore
/// beat, from `at` to `end`. A pending one has had no learner input and no limit, so the
/// lesson goes on at once; a player stops at it until the learner continues.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct WaitPoint {
    pub beat: String,
    pub at: f64,
    pub end: f64,
    pub pending: bool,
    pub explore: bool,
}

/// The value an animation drives a property to (D-068, D-075): an opacity, an offset in
/// view coordinates, a palette color, a line style, a size factor, or the representation
/// whose shape is taken.
#[derive(Clone, Debug, PartialEq)]
pub enum AnimValue {
    Opacity(f64),
    Offset([f64; 2]),
    Color(String),
    Line(String),
    Scale(f64),
    Morph(Id),
}

/// An animation of a presentation property (PK-8.4, D-068): from presentation instant
/// `from`, over `duration`, starting from the value the property has then.
#[derive(Clone, Debug)]
pub struct Animation {
    pub target: Id,
    pub from: f64,
    pub duration: f64,
    pub to: AnimValue,
    pub ease: prismal_ir::present::Ease,
}

/// A representation released from its projection (PK-8.5, D-068): from `from` it shows its
/// geometry at simulation instant `t` of run version `run`; a `bind` at `bind` hands it back
/// to its projection over `blend`.
#[derive(Clone, Debug)]
pub struct Released {
    pub target: Id,
    pub from: f64,
    pub run: usize,
    pub t: f64,
    pub bind: f64,
    pub blend: f64,
    /// The easing of the blend back (D-095).
    pub ease: prismal_ir::present::Ease,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct Highlight {
    pub target: Id,
    pub from: f64,
    pub until: f64,
}

/// A learner input the presentation did not act on (PK-9.8).
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct Refusal {
    pub at: f64,
    pub input: String,
    pub reason: String,
}

/// The record of one playback of a timeline.
pub struct Playback {
    pub cm: CModel,
    pub pres: Presentation,
    pub projector: Projector,
    pub medium: Medium,
    pub runs: Vec<RunVersion>,
    pub segments: Vec<Segment>,
    pub beats: Vec<BeatTime>,
    pub captions: Vec<Caption>,
    pub announcements: Vec<Announcement>,
    pub shown: Vec<Shown>,
    pub highlights: Vec<Highlight>,
    /// Representations hidden from a presentation instant on (PK-9.2), fading out over a
    /// duration (0 for at once).
    pub hidden: Vec<(Id, f64, f64, prismal_ir::present::Ease)>,
    pub cameras: Vec<CameraCue>,
    pub waits: Vec<WaitPoint>,
    pub animations: Vec<Animation>,
    pub released: Vec<Released>,
    pub refusals: Vec<Refusal>,
    /// Timeline diagnostics: unsatisfiable waits (PK-9.4), rejected interventions.
    pub diagnostics: Vec<String>,
    /// Elements the medium cannot present and that have no fallback (PK-12.3).
    pub unsupported: Vec<String>,
    /// The run version current at the end of the lesson.
    pub current: usize,
    /// Exact simulation instants at presentation instants where a `wait_until` ended: the
    /// event's located instant, not the mapping's rounded arithmetic (PK-9.3).
    pub pins: Vec<(f64, usize, f64)>,
    pub end: f64,
}

/// Reading time of a narration without a stated duration: 0.4 s per word, at least 2 s.
pub fn reading_time(text: &str) -> f64 {
    (text.split_whitespace().count() as f64 * 0.4).max(2.0)
}

struct Player {
    pb: Playback,
    inputs: Vec<(Input, bool)>,
    branches: usize,
    lesson: usize,
    open: Segment,
}

impl Player {
    fn num(&mut self, e: &prismal_ir::Expr) -> f64 {
        match number(&self.pb.cm, e) {
            Ok(x) => x,
            Err(m) => {
                self.pb.diagnostics.push(m);
                0.0
            }
        }
    }

    fn sim_at(&self, p: f64) -> f64 {
        // A pin belongs to the segment that ends at it; a segment opened at that instant (by a
        // seek, say) carries its own start.
        if let Some(pin) = self.pb.pins.iter().rev().find(|x| x.0 == p && x.1 == self.open.run && self.open.p0 < p) {
            return pin.2;
        }
        let s = self.open.s0 + self.open.rate * (p - self.open.p0);
        s.min(self.pb.runs[self.open.run].run.end_time())
    }

    /// Closes the open segment at `p` and opens a new one.
    fn map(&mut self, p: f64, run: usize, s0: f64, rate: f64) {
        let mut seg = std::mem::replace(&mut self.open, Segment { p0: p, p1: f64::NAN, run, s0, rate });
        seg.p1 = p;
        self.close(seg);
    }

    fn close(&mut self, seg: Segment) {
        if seg.rate > 0.0 && seg.p1 > seg.p0 {
            let r = &self.pb.runs[seg.run].run;
            let s1 = (seg.s0 + seg.rate * (seg.p1 - seg.p0)).min(r.end_time());
            for l in r.log.iter().filter(|l| l.t > seg.s0 && l.t <= s1) {
                self.pb.announcements.push(Announcement { at: seg.p0 + (l.t - seg.s0) / seg.rate, run: seg.run, event: l.name.clone(), t: l.t });
            }
        }
        self.pb.segments.push(seg);
    }

    fn cur(&self) -> usize {
        self.pb.current
    }

    /// Applies an intervention or request to the current run at the displayed instant: a
    /// new version of that run (RC-11.1).
    fn modify(&mut self, p: f64, action: Action) {
        let s = self.sim_at(p);
        let old = &self.pb.runs[self.cur()];
        let mut cfg = old.config.clone();
        cfg.log.push(Scheduled { t: s, action });
        let new = resume(&old.run, cfg.clone());
        if new.rejected.len() > old.run.rejected.len() {
            let d = new.rejected.last().unwrap();
            self.pb.diagnostics.push(format!("at {p} s: rejected: {}", d.message));
            return;
        }
        let before = old.run.log.iter().filter(|l| l.t == s).count();
        for l in new.log.iter().filter(|l| l.t == s).skip(before) {
            self.pb.announcements.push(Announcement { at: p, run: self.pb.runs.len(), event: l.name.clone(), t: l.t });
        }
        let lineage = old.lineage.clone();
        self.pb.runs.push(RunVersion { lineage, config: cfg, run: new });
        self.pb.current = self.pb.runs.len() - 1;
        if lineage_is(&self.pb.runs[self.pb.current], "lesson") {
            self.lesson = self.pb.current;
        }
        let rate = self.open.rate;
        self.map(p, self.pb.current, s, rate);
    }

    /// A new version of the current run from `t0` with the parameters of the instant shown
    /// at `p`, and with the next seed when `reseed` (D-085, D-088).
    fn restart_branch(&mut self, p: f64, reseed: bool) {
        let s = self.sim_at(p);
        let old = &self.pb.runs[self.cur()];
        let mut cfg = crate::restarted(&old.run, s);
        if reseed {
            cfg.seed = cfg.seed.wrapping_add(1);
        }
        let t0 = cfg.t0;
        let run = prismal_runtime::run(&old.run.model, cfg.clone());
        let lineage = old.lineage.clone();
        self.pb.runs.push(RunVersion { lineage, config: cfg, run });
        self.pb.current = self.pb.runs.len() - 1;
        if lineage_is(&self.pb.runs[self.pb.current], "lesson") {
            self.lesson = self.pb.current;
        }
        let rate = self.open.rate;
        self.map(p, self.pb.current, t0, rate);
    }

    fn branch(&mut self, p: f64) -> usize {
        self.branches += 1;
        let s = self.sim_at(p);
        let v = RunVersion { lineage: format!("branch {}", self.branches), ..self.pb.runs[self.cur()].clone() };
        self.pb.runs.push(v);
        self.pb.current = self.pb.runs.len() - 1;
        let rate = self.open.rate;
        self.map(p, self.pb.current, s, rate);
        self.pb.current
    }

    /// Run-directing actions take effect at `p`, before presentation time passes.
    fn direct(&mut self, a: &TAction, p: f64) {
        match a {
            TAction::Hold => {
                let s = self.sim_at(p);
                self.map(p, self.cur(), s, 0.0);
            }
            TAction::Run { rate } => {
                let r = self.num(rate);
                let s = self.sim_at(p);
                self.map(p, self.cur(), s, r);
            }
            TAction::Seek { time } => {
                let s = self.num(time);
                let rate = self.open.rate;
                self.map(p, self.cur(), s, rate);
            }
            TAction::Reset => {
                let t0 = self.pb.runs[self.cur()].config.t0;
                let rate = self.open.rate;
                self.map(p, self.cur(), t0, rate);
            }
            // D-085: a new run version from `t0`, with the parameters of the instant shown.
            TAction::Restart => self.restart_branch(p, false),
            TAction::Branch => {
                self.branch(p);
            }
            TAction::Intervene { ops } => self.modify(p, Action::Intervene(ops.clone())),
            TAction::Request { event, payload } => match payload {
                Some(v) => self.modify(p, Action::RequestWith(event.clone(), v.clone())),
                None => self.modify(p, Action::Request(event.clone())),
            },
            _ => unreachable!("not run-directing"),
        }
    }

    /// Actions that start together at `p` (a beat, or a fallback): run-directing ones first,
    /// in written order (PK-9.2a, D-033), and continue points last (D-067). Returns when the
    /// last one ends.
    fn group(&mut self, acts: &[TAction], p: f64, beat: &str) -> f64 {
        for a in acts.iter().filter(|a| a.is_run_directing()) {
            self.direct(a, p);
        }
        let mut end = p;
        let waits = |a: &&TAction| matches!(a, TAction::WaitLearner { .. });
        for a in acts.iter().filter(|a| !a.is_run_directing() && !waits(a)) {
            end = end.max(self.timed(a, p, beat));
        }
        // A continue point opens when the beat's other actions have ended (D-067).
        let open = end;
        for a in acts.iter().filter(waits) {
            end = end.max(self.timed(a, open, beat));
        }
        end
    }

    fn timed(&mut self, a: &TAction, p: f64, beat: &str) -> f64 {
        match a {
            TAction::Show { view, reps } => {
                let ctx = self.pb.projector.ctx(view.as_deref());
                for r in reps {
                    match compile_rep(&self.pb.cm, &ctx, r, &self.pb.pres.instruments) {
                        Ok(c) => self.pb.shown.push(Shown { from: p, until: f64::INFINITY, view: view.clone(), rep: c, reveal: None }),
                        Err(d) => self.pb.diagnostics.extend(d.into_iter().map(|x| x.to_string())),
                    }
                }
                p
            }
            TAction::Highlight { target } => {
                self.pb.highlights.push(Highlight { target: target.clone(), from: p, until: f64::NAN });
                p
            }
            TAction::Hide { target, duration, ease } => {
                let d = duration.as_ref().map(|d| self.num(d)).unwrap_or(0.0);
                self.pb.hidden.push((target.clone(), p, d, ease.unwrap_or(prismal_ir::present::Ease::Smooth)));
                p + d
            }
            TAction::Reveal { view, style, duration, ease, reps } => {
                let ease = ease.unwrap_or(prismal_ir::present::Ease::Smooth);
                let d = duration.as_ref().map(|d| self.num(d)).unwrap_or(1.0);
                let ctx = self.pb.projector.ctx(view.as_deref());
                for r in reps {
                    match compile_rep(&self.pb.cm, &ctx, r, &self.pb.pres.instruments) {
                        Ok(c) => self.pb.shown.push(Shown { from: p, until: f64::INFINITY, view: view.clone(), rep: c, reveal: Some((*style, d, ease)) }),
                        Err(e) => self.pb.diagnostics.extend(e.into_iter().map(|x| x.to_string())),
                    }
                }
                p + d
            }
            TAction::Camera { view, center, zoom, duration, ease } => {
                let d = duration.as_ref().map(|d| self.num(d)).unwrap_or(1.0);
                let z = zoom.as_ref().map(|z| self.num(z));
                let c = match (center, self.pb.projector.ctx(Some(view))) {
                    (Some(e), ViewCtx::Spatial { space, .. }) => match prismal_kernel::compile_expr(&self.pb.cm, e, Some(&prismal_ir::Type::Point { space })) {
                        Ok((ce, _)) => Some(ce),
                        Err(ds) => {
                            self.pb.diagnostics.extend(ds.iter().map(|x| x.to_string()));
                            None
                        }
                    },
                    // A plot's camera centers on a pair of axis values (D-075).
                    (Some(e), ViewCtx::Plot { dims, .. }) => {
                        let ty = prismal_ir::Type::Tuple { items: vec![prismal_ir::Type::Quantity { dim: dims.0 }, prismal_ir::Type::Quantity { dim: dims.1 }] };
                        match prismal_kernel::compile_expr(&self.pb.cm, e, Some(&ty)) {
                            Ok((ce, _)) => Some(ce),
                            Err(ds) => {
                                self.pb.diagnostics.extend(ds.iter().map(|x| x.to_string()));
                                None
                            }
                        }
                    }
                    _ => None,
                };
                self.pb.cameras.push(CameraCue { view: view.clone(), from: p, duration: d, center: c, zoom: z, ease: ease.unwrap_or(prismal_ir::present::Ease::Smooth) });
                p + d
            }
            TAction::Narrate { text, duration } => {
                let d = match duration {
                    Some(e) => self.num(e),
                    None => reading_time(text),
                };
                let n = self.pb.captions.iter().filter(|c| c.cue == beat || c.cue.strip_prefix(beat).is_some_and(|r| r.starts_with('.'))).count();
                let cue = if n == 0 { beat.to_string() } else { format!("{beat}.{}", n + 1) };
                self.pb.captions.push(Caption { cue, text: text.clone(), start: p, end: p + d });
                p + d
            }
            TAction::Wait { duration } => p + self.num(duration),
            TAction::WaitUntil { event } => {
                let name = self.pb.cm.ir.events.iter().find(|e| &e.id == event).map(|e| e.name.clone()).unwrap_or_default();
                let s = self.sim_at(p);
                let rate = self.open.rate;
                let r = &self.pb.runs[self.cur()].run;
                if rate <= 0.0 {
                    self.pb.diagnostics.push(format!("{beat}: waits for `{name}` while the simulation holds; the wait cannot be satisfied (PK-9.4)"));
                    return p;
                }
                match r.log.iter().find(|l| l.name == name && l.t > s) {
                    Some(l) => {
                        let end = p + (l.t - s) / rate;
                        let (run, t) = (self.open.run, l.t);
                        self.pb.pins.push((end, run, t));
                        end
                    }
                    None => {
                        let end = r.end_time();
                        self.pb.diagnostics.push(format!("{beat}: `{name}` does not occur before the run ends at {end} s (PK-9.4)"));
                        p + (end - s).max(0.0) / rate
                    }
                }
            }
            TAction::Sequence { actions } => {
                let mut q = p;
                for x in actions {
                    if x.is_run_directing() {
                        self.direct(x, q);
                    } else {
                        q = self.timed(x, q, beat);
                    }
                }
                q
            }
            TAction::Explore { limit, keep, controls, fallback } => match self.pb.medium {
                Medium::Video => {
                    if fallback.is_empty() {
                        self.pb.unsupported.push(format!("{beat}: an explore beat without a fallback cannot be shown in linear media (PK-9.10, PK-12.3)"));
                        p
                    } else {
                        self.group(fallback, p, beat)
                    }
                }
                Medium::Interactive => self.explore(p, limit.as_ref(), keep, controls, beat),
            },
            // A continue point (D-067): linear media play the fallback, or omit it.
            TAction::WaitLearner { limit, fallback } => match self.pb.medium {
                Medium::Video => self.group(fallback, p, beat),
                Medium::Interactive => {
                    let limit = limit.as_ref().map(|l| self.num(l));
                    let deadline = limit.map(|l| p + l).unwrap_or(f64::INFINITY);
                    let next = self.inputs.iter().position(|(i, used)| !used && i.at >= p && i.at <= deadline && matches!(i.input, LearnerInput::Continue));
                    let end = match next {
                        Some(k) => {
                            self.inputs[k].1 = true;
                            self.inputs[k].0.at
                        }
                        None if deadline.is_finite() => deadline,
                        None => p,
                    };
                    let pending = next.is_none() && limit.is_none();
                    self.pb.waits.push(WaitPoint { beat: beat.to_string(), at: p, end, pending, explore: false });
                    end
                }
            },
            TAction::Animate { target, property, to, word, duration, ease } => {
                let d = duration.as_ref().map(|d| self.num(d)).unwrap_or(1.0);
                let w = || word.clone().unwrap_or_default();
                let to = match (property, to) {
                    (Animated::Opacity, Some(to)) => AnimValue::Opacity(self.num(to).clamp(0.0, 1.0)),
                    (Animated::Offset, Some(to)) => AnimValue::Offset(self.offset(target, to, p)),
                    (Animated::Scale, Some(to)) => AnimValue::Scale(self.num(to).max(0.0)),
                    (Animated::Color, _) => AnimValue::Color(w()),
                    (Animated::Line, _) => AnimValue::Line(w()),
                    (Animated::Morph, _) => AnimValue::Morph(w()),
                    _ => return p,
                };
                let ease = ease.unwrap_or(prismal_ir::present::Ease::Smooth);
                self.pb.animations.push(Animation { target: target.clone(), from: p, duration: d, to, ease });
                p + d
            }
            TAction::Release { target } => {
                if self.pb.released.iter().any(|r| &r.target == target && r.bind.is_infinite()) {
                    self.pb.diagnostics.push(format!("{beat}: `{target}` is already released (D-068)"));
                    return p;
                }
                let (run, t) = (self.open.run, self.sim_at(p));
                self.pb.released.push(Released { target: target.clone(), from: p, run, t, bind: f64::INFINITY, blend: 0.0, ease: prismal_ir::present::Ease::Smooth });
                p
            }
            TAction::Bind { target, duration, ease } => {
                let d = duration.as_ref().map(|d| self.num(d)).unwrap_or(0.0);
                match self.pb.released.iter_mut().rev().find(|r| &r.target == target && r.bind.is_infinite()) {
                    Some(r) => {
                        r.bind = p;
                        r.blend = d;
                        r.ease = ease.unwrap_or(prismal_ir::present::Ease::Smooth);
                        p + d
                    }
                    None => {
                        self.pb.diagnostics.push(format!("{beat}: `{target}` is not released; `bind` hands back a released representation (D-068)"));
                        p
                    }
                }
            }
            other => {
                self.direct(other, p);
                p
            }
        }
    }

    /// An offset in view coordinates (D-068): the vector `to` of the space of the view the
    /// representation is drawn in, evaluated at the instant shown at `p`.
    fn offset(&mut self, target: &str, to: &prismal_ir::Expr, p: f64) -> [f64; 2] {
        let view = crate::frame::rep_view(&self.pb.pres, target).flatten();
        let ctx = self.pb.projector.ctx(view.as_deref());
        let ViewCtx::Spatial { space, .. } = &ctx else {
            self.pb.diagnostics.push(format!("`{target}` is not drawn in a spatial view; only those take an offset (D-068)"));
            return [0.0, 0.0];
        };
        let ty = prismal_ir::Type::Vector { space: space.clone(), dim: prismal_ir::Dim::length() };
        let s = self.sim_at(p);
        let r = &self.pb.runs[self.open.run].run;
        let v = prismal_kernel::compile_expr(&self.pb.cm, to, Some(&ty)).map_err(|ds| ds.iter().map(|d| d.to_string()).collect::<Vec<_>>().join("; ")).and_then(|(ce, _)| r.eval_state(&ce, &r.state_at(s), s).map_err(|e| e.cause));
        match v {
            Ok(v) => ctx.to_view(&crate::flat(&v)),
            Err(m) => {
                self.pb.diagnostics.push(format!("`{target}`: {m}"));
                [0.0, 0.0]
            }
        }
    }

    /// An explore beat (PK-9.8, PK-9.9, D-025): the lesson run is branched at the displayed
    /// instant; the learner's inputs act on the branch through the declared controls.
    fn explore(&mut self, p: f64, limit: Option<&prismal_ir::Expr>, keep: &[Id], controls: &[prismal_ir::present::Rep], beat: &str) -> f64 {
        let back_to = self.cur();
        let s_branch = self.sim_at(p);
        let rate = self.open.rate;
        self.branch(p);
        let limit = limit.map(|l| self.num(l));
        let deadline = limit.map(|l| p + l).unwrap_or(f64::INFINITY);
        let mut end = None;
        for i in 0..self.inputs.len() {
            let (inp, used) = &self.inputs[i];
            if *used || inp.at < p {
                continue;
            }
            if inp.at > deadline {
                break;
            }
            let inp = inp.clone();
            self.inputs[i].1 = true;
            match &inp.input {
                LearnerInput::Continue => {
                    end = Some(inp.at);
                    break;
                }
                LearnerInput::SetControl { control, binding, value } => {
                    let offered = controls.iter().any(|c| c.kind == *control && matches!(c.sources.first(), Some(prismal_ir::present::Arg::Expr { expr: prismal_ir::Expr::Ref { r#ref } }) if r#ref == binding));
                    if !offered {
                        self.pb.refusals.push(Refusal { at: inp.at, input: describe(&inp.input), reason: format!("{beat} offers no {control} for `{binding}` (PK-9.8)") });
                        continue;
                    }
                    let op = Op::Set { target: prismal_ir::Target::of(binding.clone()), value: value.clone() };
                    self.modify(inp.at, Action::Intervene(vec![op]));
                }
                // A button among the beat's controls requests its event on the branch (D-069).
                LearnerInput::Press { event } => {
                    let offered = controls.iter().find(|c| c.kind == "button" && matches!(c.sources.first(), Some(prismal_ir::present::Arg::Element { element }) if element == event));
                    let payload = offered.and_then(|c| c.props.iter().find(|p| p.name == "payload")).and_then(|p| match &p.value {
                        prismal_ir::present::Arg::Expr { expr } => Some(expr.clone()),
                        _ => None,
                    });
                    if offered.is_none() {
                        let name = self.pb.cm.ir.events.iter().find(|e| &e.id == event).map(|e| e.name.clone()).unwrap_or_else(|| event.clone());
                        self.pb.refusals.push(Refusal { at: inp.at, input: describe(&inp.input), reason: format!("{beat} offers no button for `{name}` (PK-9.8)") });
                        continue;
                    }
                    self.modify(inp.at, match payload {
                        Some(v) => Action::RequestWith(event.clone(), v),
                        None => Action::Request(event.clone()),
                    });
                }
                // D-088: `restart` starts the branch again from `t0` with its parameters as
                // they are; `reseed` does so with the next seed.
                LearnerInput::Control { control } => {
                    let offered = controls.iter().any(|c| c.kind == "button" && matches!(c.sources.first(), Some(prismal_ir::present::Arg::Word { word }) if word == control));
                    if !offered {
                        self.pb.refusals.push(Refusal { at: inp.at, input: describe(&inp.input), reason: format!("{beat} offers no button({control}) (PK-9.8)") });
                        continue;
                    }
                    self.restart_branch(inp.at, control == "reseed");
                }
            }
        }
        let end_input = end;
        let end = match end {
            Some(e) => e,
            None if deadline.is_finite() => deadline,
            None => p,
        };
        self.pb.waits.push(WaitPoint { beat: beat.to_string(), at: p, end, pending: end_input.is_none() && limit.is_none(), explore: true });
        for c in controls {
            match compile_rep(&self.pb.cm, &ViewCtx::Panel, c, &self.pb.pres.instruments) {
                Ok(cr) => self.pb.shown.push(Shown { from: p, until: end, view: None, rep: cr, reveal: None }),
                Err(d) => self.pb.diagnostics.extend(d.into_iter().map(|x| x.to_string())),
            }
        }
        if keep.is_empty() {
            // Return to the lesson run at the instant of the branch (PK-9.9).
            let lesson = if lineage_is(&self.pb.runs[back_to], "lesson") { self.lesson } else { back_to };
            self.pb.current = lesson;
            self.map(end, lesson, s_branch, rate);
        }
        end
    }
}

fn lineage_is(v: &RunVersion, name: &str) -> bool {
    v.lineage == name
}

fn describe(i: &LearnerInput) -> String {
    match i {
        LearnerInput::Continue => "continue".into(),
        LearnerInput::SetControl { control, binding, .. } => format!("set {control} {binding}"),
        LearnerInput::Press { event } => format!("press {event}"),
        LearnerInput::Control { control } => format!("press {control}"),
    }
}

/// Plays a presentation's timeline over a lesson run configured by `base` (PK-9.5).
pub fn play(prog: &Program, presentation: &str, base: Config, medium: Medium, inputs: Vec<Input>) -> Result<Playback, Vec<PDiag>> {
    // A run that fills a collection declared without a limit is played again with room for
    // twice as many members (D-066).
    let mut grown: Option<Program> = None;
    loop {
        let p = grown.as_ref().unwrap_or(prog);
        let pb = play_once(p, presentation, base.clone(), medium, inputs.clone())?;
        match pb.runs.iter().find_map(|v| p.overflow(&v.run)).and_then(|part| p.grown(&part)) {
            Some(next) => grown = Some(next),
            None => return Ok(pb),
        }
    }
}

fn play_once(prog: &Program, presentation: &str, base: Config, medium: Medium, inputs: Vec<Input>) -> Result<Playback, Vec<PDiag>> {
    let pres = prog.presentation(presentation);
    let cm = prog.model(&pres.model);
    let projector = Projector::new(cm, pres)?;
    let Some(tl) = &pres.timeline else {
        return Err(vec![PDiag { code: "PK-E01", message: "the presentation has no timeline".into(), element: pres.id.clone() }]);
    };
    let lesson = run(cm, base.clone());
    let t0 = base.t0;
    let mut inputs: Vec<(Input, bool)> = inputs.into_iter().map(|i| (i, false)).collect();
    inputs.sort_by(|a, b| a.0.at.total_cmp(&b.0.at));
    let pb = Playback {
        cm: cm.clone(),
        pres: pres.clone(),
        projector,
        medium,
        runs: vec![RunVersion { lineage: "lesson".into(), config: base, run: lesson }],
        segments: vec![],
        beats: vec![],
        captions: vec![],
        announcements: vec![],
        shown: vec![],
        highlights: vec![],
        hidden: vec![],
        cameras: vec![],
        waits: vec![],
        animations: vec![],
        released: vec![],
        refusals: vec![],
        diagnostics: vec![],
        unsupported: vec![],
        current: 0,
        pins: vec![],
        end: 0.0,
    };
    // The lesson starts holding at t0 (PK-8.2).
    let mut pl = Player { pb, inputs, branches: 0, lesson: 0, open: Segment { p0: 0.0, p1: f64::NAN, run: 0, s0: t0, rate: 0.0 } };
    let mut p = 0.0;
    for b in tl.scenes.iter().flat_map(|s| &s.beats) {
        let start = p;
        let n_hl = pl.pb.highlights.len();
        p = pl.group(&b.actions, p, &b.name);
        for h in &mut pl.pb.highlights[n_hl..] {
            h.until = p;
        }
        pl.pb.beats.push(BeatTime { id: b.id.clone(), name: b.name.clone(), start, end: p });
    }
    let last = std::mem::replace(&mut pl.open, Segment { p0: p, p1: p, run: 0, s0: 0.0, rate: 0.0 });
    pl.close(Segment { p1: p, ..last });
    // Inputs no explore beat took are refused: outside explore beats the learner has no
    // model actions (PK-9.8).
    for (inp, used) in &pl.inputs {
        if !used {
            let reason = match inp.input {
                LearnerInput::Continue => "no continue point at this instant".into(),
                LearnerInput::SetControl { .. } | LearnerInput::Press { .. } | LearnerInput::Control { .. } => "outside an explore beat the learner has no model actions (PK-9.8, D-025)".to_string(),
            };
            pl.pb.refusals.push(Refusal { at: inp.at, input: describe(&inp.input), reason });
        }
    }
    pl.pb.end = p;
    Ok(pl.pb)
}

impl Playback {
    pub fn beat(&self, name: &str) -> &BeatTime {
        self.beats.iter().find(|b| b.name == name || b.id == name).unwrap_or_else(|| panic!("no beat {name}"))
    }

    /// The last version of a lineage (`lesson`, `branch 1`).
    pub fn lineage(&self, name: &str) -> Option<&RunVersion> {
        self.runs.iter().rev().find(|v| v.lineage == name)
    }

    /// The segment of the time mapping shown at presentation instant `p`.
    pub fn segment_at(&self, p: f64) -> &Segment {
        self.segments.iter().rev().find(|s| s.p0 <= p).unwrap_or(&self.segments[0])
    }

    /// The run version and simulation instant shown at presentation instant `p` (PK-8.2).
    pub fn sim_at(&self, p: f64) -> (usize, f64) {
        let s = self.segment_at(p);
        if let Some(pin) = self.pins.iter().rev().find(|x| x.0 == p && x.1 == s.run && s.p0 < p) {
            return (s.run, pin.2);
        }
        let r = &self.runs[s.run].run;
        (s.run, (s.s0 + s.rate * (p - s.p0)).min(r.end_time()))
    }

    /// The frame description at presentation instant `p`; announcements are those due in
    /// `(p - dt, p]` (PK-12.1).
    pub fn frame(&self, p: f64, dt: f64) -> Frame {
        let (ri, t) = self.sim_at(p);
        let v = &self.runs[ri];
        let vals: Vec<Value> = v.run.state_at(t);
        let extra: Vec<(Option<Id>, CRep)> = self.shown.iter().filter(|s| s.from <= p && p < s.until).map(|s| (s.view.clone(), s.rep.clone())).collect();
        let (mut views, mut overlay) = self.projector.frame(&self.cm, &v.run, &vals, t, &extra);
        // Released representations (D-068) show their geometry at the instant of the release,
        // and blend back to their projection while a `bind` hands them back.
        for r in self.released.iter().filter(|r| r.from <= p && p < r.bind + r.blend) {
            let Some(held) = self.held(r) else { continue };
            let k = if p < r.bind { 0.0 } else { r.ease.apply((p - r.bind) / r.blend) };
            let mut swap = |x: &mut crate::frame::RepFrame| {
                if x.id == r.target {
                    match lerp_shape(&held.shape, &x.shape, k) {
                        Some(sh) => x.shape = sh,
                        None => x.shape = held.shape.clone(),
                    }
                    if k == 0.0 {
                        x.text = held.text.clone();
                    }
                }
            };
            for v in views.iter_mut() {
                each_rep_mut(&mut v.reps, &mut swap);
            }
            each_rep_mut(&mut overlay, &mut swap);
        }
        // The shapes representations morph into (D-075), taken before hidden ones are removed:
        // a morph's target is usually hidden, drawn only through the morph.
        let morph_targets: Vec<(Id, Shape)> = {
            let ids: Vec<&Id> = self.animations.iter().filter_map(|a| if let AnimValue::Morph(id) = &a.to { Some(id) } else { None }).collect();
            let mut found = vec![];
            if !ids.is_empty() {
                let mut look = |r: &mut crate::frame::RepFrame| {
                    if ids.contains(&&r.id) {
                        found.push((r.id.clone(), r.shape.clone()));
                    }
                };
                for v in views.iter_mut() {
                    each_rep_mut(&mut v.reps, &mut look);
                }
                each_rep_mut(&mut overlay, &mut look);
            }
            found
        };
        // Hidden representations: gone after their fade, fading during it.
        // Members of groups are found by the same rules (D-043).
        let gone: Vec<&Id> = self.hidden.iter().filter(|h| h.1 + h.2 <= p).map(|h| &h.0).collect();
        let keep = |r: &crate::frame::RepFrame| !gone.contains(&&r.id);
        for v in views.iter_mut() {
            retain_reps(&mut v.reps, &keep);
        }
        retain_reps(&mut overlay, &keep);
        let mut animate = |r: &mut crate::frame::RepFrame| {
            if let Some(h) = self.hidden.iter().find(|h| h.0 == r.id && h.1 <= p && p < h.1 + h.2) {
                r.opacity = Some(1.0 - h.3.apply((p - h.1) / h.2));
            }
            if let Some(s) = self.shown.iter().find(|s| s.rep.rep.id == r.id && s.from <= p) {
                if let Some((style, d, e)) = s.reveal {
                    let k = if d > 0.0 { e.apply((p - s.from) / d) } else { 1.0 };
                    if k < 1.0 {
                        match (&mut r.shape, style) {
                            // A group drawn: its paths are drawn, its markers fade in.
                            (Shape::Group { members }, RevealStyle::Draw) => each_rep_mut(members, &mut |m| reveal(m, style, k)),
                            _ => reveal(r, style, k),
                        }
                    }
                }
            }
        };
        for v in views.iter_mut() {
            each_rep_mut(&mut v.reps, &mut animate);
        }
        each_rep_mut(&mut overlay, &mut animate);
        // Animated properties (D-068): each animation starts from the value the previous one
        // left; the last value holds after it ends.
        let mut animated = |r: &mut crate::frame::RepFrame| {
            let (mut opacity, mut offset, mut scale) = (1.0, [0.0, 0.0], 1.0);
            let mut any = (false, false, false);
            // A color reached so far, and one being blended to (D-075).
            let (mut color, mut mix): (Option<String>, Option<(String, f64)>) = (r.color.clone(), None);
            let mut line: Option<String> = None;
            let mut morphs: Vec<(&Id, f64)> = vec![];
            for a in self.animations.iter().filter(|a| a.target == r.id && a.from <= p) {
                let k = if a.duration > 0.0 { a.ease.apply((p - a.from) / a.duration) } else { 1.0 };
                let done = a.duration <= 0.0 || p >= a.from + a.duration;
                match &a.to {
                    AnimValue::Opacity(x) => {
                        opacity += (x - opacity) * k;
                        any.0 = true;
                    }
                    AnimValue::Offset(d) => {
                        offset = [offset[0] + (d[0] - offset[0]) * k, offset[1] + (d[1] - offset[1]) * k];
                        any.1 = true;
                    }
                    AnimValue::Scale(x) => {
                        scale += (x - scale) * k;
                        any.2 = true;
                    }
                    AnimValue::Color(c) => {
                        if let Some((prev, _)) = mix.take() {
                            color = Some(prev);
                        }
                        if done {
                            color = Some(c.clone());
                        } else {
                            mix = Some((c.clone(), k));
                        }
                    }
                    // A line style has no values between: it switches halfway.
                    AnimValue::Line(l) => {
                        if done || p >= a.from + a.duration / 2.0 {
                            line = Some(l.clone());
                        }
                    }
                    AnimValue::Morph(id) => morphs.push((id, if done { 1.0 } else { k })),
                }
            }
            for (id, k) in morphs {
                if let Some((_, target)) = morph_targets.iter().find(|(t, _)| t == id) {
                    r.shape = morph(&r.shape, target, k);
                }
            }
            if any.2 {
                scale_shape(&mut r.shape, scale);
                r.scale = Some(scale);
            }
            if any.0 {
                r.opacity = Some(r.opacity.unwrap_or(1.0) * opacity);
            }
            if any.1 {
                translate(&mut r.shape, offset);
            }
            r.color = color;
            if let Some((to, k)) = mix {
                r.color_to = Some(to);
                r.color_mix = Some(k);
            }
            if let Some(l) = line {
                r.line = Some(l);
            }
        };
        if !self.animations.is_empty() {
            for v in views.iter_mut() {
                each_rep_mut(&mut v.reps, &mut animated);
            }
            each_rep_mut(&mut overlay, &mut animated);
        }
        // Cameras (D-042): each move starts from where the previous one left the camera.
        for vf in views.iter_mut() {
            let ctx = self.projector.ctx(Some(&vf.id));
            let mut center: Option<[f64; 2]> = None;
            let mut zoom = 1.0;
            let mut blend = 1.0;
            let mut any = false;
            for c in self.cameras.iter().filter(|c| c.view == vf.id && c.from <= p) {
                any = true;
                let target = c.center.as_ref().and_then(|ce| v.run.eval_state(ce, &vals, t).ok()).map(|pt| ctx.to_view(&crate::flat(&pt))).or(center);
                let tz = c.zoom.unwrap_or(zoom);
                let k = if c.duration > 0.0 { c.ease.apply((p - c.from) / c.duration) } else { 1.0 };
                zoom += (tz - zoom) * k;
                match (center, target) {
                    (Some(a), Some(b)) => {
                        center = Some([a[0] + (b[0] - a[0]) * k, a[1] + (b[1] - a[1]) * k]);
                        blend = 1.0;
                    }
                    (None, Some(b)) => {
                        center = Some(b);
                        blend = k;
                    }
                    _ => {}
                }
            }
            if any {
                vf.camera = Some(crate::frame::Camera { center, zoom, blend });
            }
        }
        for h in self.highlights.iter().filter(|h| h.from <= p && p < h.until) {
            let mut mark = |r: &mut crate::frame::RepFrame| {
                if r.id == h.target {
                    r.highlighted = true;
                }
            };
            for v in views.iter_mut() {
                each_rep_mut(&mut v.reps, &mut mark);
            }
            each_rep_mut(&mut overlay, &mut mark);
        }
        Frame {
            time: p,
            run: v.lineage.clone(),
            t,
            views,
            overlay,
            captions: self.captions.iter().filter(|c| c.start <= p && p < c.end).map(|c| c.text.clone()).collect(),
            announcements: self.announcements.iter().filter(|a| a.at > p - dt && a.at <= p).map(|a| a.event.clone()).collect(),
        }
    }

    /// A released representation as it was at its release (D-068).
    fn held(&self, r: &Released) -> Option<crate::frame::RepFrame> {
        let run = &self.runs[r.run].run;
        let vals = run.state_at(r.t);
        let extra: Vec<(Option<Id>, CRep)> = self.shown.iter().filter(|s| s.from <= r.from && r.from < s.until).map(|s| (s.view.clone(), s.rep.clone())).collect();
        let (views, overlay) = self.projector.frame(&self.cm, run, &vals, r.t, &extra);
        fn find(reps: Vec<crate::frame::RepFrame>, id: &str) -> Option<crate::frame::RepFrame> {
            for x in reps {
                if x.id == id {
                    return Some(x);
                }
                if let Shape::Group { members } = x.shape {
                    if let Some(m) = find(members, id) {
                        return Some(m);
                    }
                }
            }
            None
        }
        views.into_iter().find_map(|v| find(v.reps, &r.target)).or_else(|| find(overlay, &r.target))
    }

    /// Frame descriptions at `fps` frames per presentation second, from 0 to the end.
    pub fn export(&self, fps: f64) -> Vec<Frame> {
        let n = (self.end * fps).floor() as usize;
        (0..=n).map(|k| self.frame(k as f64 / fps, 1.0 / fps)).collect()
    }

    /// The values of an `on(E)` observation at each occurrence playback passed, with the
    /// presentation instant of each.
    pub fn observed(&self, observation: &str) -> Vec<(f64, Value)> {
        let Some(o) = self.pres.observations.iter().find(|o| o.name == observation) else { return vec![] };
        let (prismal_ir::present::Source::Expr { expr }, prismal_ir::present::Schedule::On { event, .. }) = (&o.source, &o.schedule) else { return vec![] };
        let name = self.cm.ir.events.iter().find(|e| &e.id == event).map(|e| e.name.clone()).unwrap_or_default();
        self.announcements
            .iter()
            .filter(|a| a.event == name)
            .map(|a| {
                let r = &self.runs[a.run].run;
                let c = r.compile(expr);
                (a.at, r.eval_state(&c, &r.state_at(a.t), a.t).expect("observation value"))
            })
            .collect()
    }
}
