//! An open presentation (HI section 4): an interactive session or a lesson playback, driven
//! by the host's clock and the learner's inputs, answering with layouts, frame descriptions
//! (PK-12.1) and observations as JSON.
//!
//! A presentation opens in one of two modes:
//! - interactive: an interactive session of the model (PK section 10), for presentations
//!   without a timeline; controls, drags and keys act on the session.
//! - lesson: a playback of the timeline (PK sections 8, 9). The host's clock chooses the
//!   presentation instant; learner inputs are recorded with their instants and the playback
//!   is recomputed from them, which is deterministic (PK-8.7), so frames before an input
//!   never change.

use std::rc::Rc;
use prismal_ir::present::{Action as TAction, LearnerInput, Observation, Schedule, Source};
use prismal_ir::Op;
use prismal_kernel::{compile_expr, CModel};
use prismal_present::data::Data;
use crate::input::{dist, focus_order, hit, tolerance, Map, Target, ViewKind, ViewState, Viewport, PLOT_MARGIN};
use prismal_present::frame::{CKind, CRep, Frame, RepFrame, Shape, ViewCtx};
use prismal_present::interact::{si_literal, Interactive, Key};
use prismal_present::text::{fmt_binding, fmt_num, fmt_payload, fmt_value, print, symbol, unit_text};
use prismal_present::timeline::{play, Input, Medium, Playback};
use prismal_present::{Program, LESSON_HORIZON};
use prismal_runtime::{Action, Config};
use serde_json::{json, Value as Json};

/// Least simulated span of an interactive session of a dynamic model.
pub const SESSION_HORIZON: f64 = LESSON_HORIZON;

enum Mode {
    Closed,
    Interactive(Box<Interactive>),
    Lesson(Box<Lesson>),
}

struct Lesson {
    pres: String,
    medium: Medium,
    inputs: Vec<Input>,
    pb: Playback,
}

impl Lesson {
    fn replay(prog: &Program, pres: &str, medium: Medium, inputs: Vec<Input>) -> Result<Lesson, Json> {
        let p = prog.presentation(pres);
        let horizon = declared_until(prog, p).unwrap_or(LESSON_HORIZON);
        let pb = play(prog, pres, Config::until(horizon), medium, inputs.clone()).map_err(|ds| pdiags(prog, &ds))?;
        Ok(Lesson { pres: pres.into(), medium, inputs, pb })
    }
}

/// A presentation's `until`, seconds after `t0` (D-091).
fn declared_until(prog: &Program, p: &prismal_ir::present::Presentation) -> Option<f64> {
    let e = p.until.as_ref()?;
    prismal_present::number(prog.model(&p.model), e).ok()
}

/// The session horizon of a model with a large state (fields, D-091), unless a presentation
/// says otherwise: each second of such a run is costly to compute and keep.
pub const LARGE_SESSION_HORIZON: f64 = 20.0;

fn pdiags(_prog: &Program, ds: &[prismal_present::PDiag]) -> Json {
    Json::Array(ds.iter().map(|d| json!({ "code": d.code, "message": d.message, "element": d.element })).collect())
}

/// An instance: a checked program and at most one open presentation of it. The program is
/// shared with its document, and the instance keeps it alive (HI-2.2).
pub struct Instance {
    prog: Rc<Program>,
    mode: Mode,
    /// The views of the open presentation, with the learner's zoom and pan (D-047).
    views: Vec<ViewState>,
    /// A drag or a pan in progress, begun by a raw pointer event.
    gesture: Option<Gesture>,
    /// The representation with keyboard focus (PK-11.2).
    focus: Option<String>,
}

#[derive(Clone, Debug)]
enum Gesture {
    /// A drag; `click` holds, while the pointer has not moved past the drag tolerance, the
    /// representation a release would click instead and where the press was (D-059).
    Drag { view: usize, click: Option<(String, [f64; 2], f64)> },
    /// A press on a representation that can only be clicked: a release within `tol` pixels
    /// of `from` clicks it.
    Click { view: usize, rep: String, from: [f64; 2], tol: f64 },
    /// A pan from pixel `from` of the framing `shown`, drawn with `map` when it began;
    /// `before` is the learner's framing to restore on cancel.
    /// `click` holds the tolerance while a release would still click the point pressed
    /// instead (D-060).
    /// `id` is the pointer's, and `at` where it is now, so that a second touch can make the
    /// pan a pinch.
    Pan { view: usize, from: [f64; 2], shown: [f64; 4], map: Map, before: Option<[f64; 4]>, click: Option<f64>, id: Option<u64>, at: [f64; 2] },
    /// Two touches zooming and moving a spatial view (HI-4.5): `p0` where they were when the
    /// second came down, `p` where they are, over the framing `shown` drawn at `scale`
    /// pixels per view unit; `before` is the learner's framing to restore on cancel.
    /// `q0` is the view point under the touches' first midpoint.
    Pinch { view: usize, ids: [Option<u64>; 2], p0: [[f64; 2]; 2], p: [[f64; 2]; 2], q0: [f64; 2], shown: [f64; 4], map: Map, before: Option<[f64; 4]> },
    /// A press on an empty point of a view that requests an event when clicked, and that
    /// does not pan: a release within `tol` pixels of `from` clicks the point (D-060).
    Point { view: usize, from: [f64; 2], tol: f64 },
}

/// A pointer event as the host captured it (HI-4.5): its phase (`down`, `move`, `up`,
/// `cancel`), the view it happened in (identity or name), its position in pixels from the
/// top left of the view as the host drew it, the size the host drew the view at (`None`:
/// as last given, else the view's natural size), the kind of pointer (`mouse`, `touch`,
/// `pen`) and, in a lesson, the presentation instant.
#[derive(Clone, Debug)]
pub struct PointerEvent<'a> {
    pub phase: &'a str,
    pub view: &'a str,
    pub x: f64,
    pub y: f64,
    pub size: Option<[f64; 2]>,
    pub pointer: &'a str,
    /// The pointer's identity, for several pointers at once (two touches pinch).
    pub id: Option<u64>,
    pub time: f64,
}

/// Adds the fields of `extra` to the object `base`.
fn merge(mut base: Json, extra: Json) -> Json {
    if let (Some(b), Json::Object(e)) = (base.as_object_mut(), extra) {
        b.extend(e);
    }
    base
}

fn target_json(t: &Target) -> Json {
    json!({ "rep": t.rep, "part": t.part, "drag": t.drag, "click": t.click })
}

fn unhandled() -> Json {
    json!({ "handled": false, "action": null })
}

impl Instance {
    /// An instance with no presentation open yet.
    pub fn new(prog: Rc<Program>) -> Instance {
        Instance { prog, mode: Mode::Closed, views: vec![], gesture: None, focus: None }
    }

    /// `closed`, `interactive` or `lesson` (HI-4.1).
    pub fn mode(&self) -> &'static str {
        match self.mode {
            Mode::Closed => "closed",
            Mode::Interactive(_) => "interactive",
            Mode::Lesson(_) => "lesson",
        }
    }

    /// The program the instance presents.
    pub fn program(&self) -> &Program {
        &self.prog
    }

    /// Opens a presentation: as a lesson when it has a timeline (in the interactive medium,
    /// or `video` for linear media with fallbacks, PK-9.10), otherwise interactive.
    /// Returns its layout.
    pub fn open(&mut self, presentation: &str, video: bool) -> Result<Json, Json> {
        let pres = self.find_presentation(presentation)?;
        self.mode = if pres.timeline.is_some() {
            let medium = if video { Medium::Video } else { Medium::Interactive };
            Mode::Lesson(Box::new(Lesson::replay(&self.prog, &pres.id, medium, vec![])?))
        } else {
            // A dynamic model runs to the session horizon, or to the end of the longest time
            // axis the presentation plots; a static one has one instant.
            let cm = self.prog.model(&pres.model);
            let base = if cm.n_y() > prismal_runtime::run::LARGE_STATE { LARGE_SESSION_HORIZON } else { SESSION_HORIZON };
            let plotted = match prismal_present::frame::Projector::new(cm, &pres) {
                Ok(pj) => pj
                    .views
                    .iter()
                    .filter_map(|(_, ctx, _)| match ctx {
                        ViewCtx::Plot { x, dims, .. } if dims.0 == prismal_ir::Dim::time() => Some(x.1),
                        _ => None,
                    })
                    .fold(base, f64::max),
                Err(_) => base,
            };
            let horizon = declared_until(&self.prog, &pres).unwrap_or(plotted);
            let i = Interactive::new(&self.prog, &pres.id, Config::until(horizon)).map_err(|ds| pdiags(&self.prog, &ds))?;
            Mode::Interactive(Box::new(i))
        };
        let layout = self.layout();
        self.views = layout["views"].as_array().into_iter().flatten().map(view_state).collect();
        self.gesture = None;
        self.focus = None;
        Ok(layout)
    }

    fn find_presentation(&self, name: &str) -> Result<prismal_ir::present::Presentation, Json> {
        self.prog.doc.presentations.iter().find(|p| p.name == name || p.id == name).cloned().ok_or_else(|| json!([{ "code": "HI-E02", "message": format!("no presentation `{name}`") }]))
    }

    fn cm(&self) -> Option<&CModel> {
        match &self.mode {
            Mode::Closed => None,
            Mode::Interactive(i) => Some(&i.cm),
            Mode::Lesson(l) => Some(&l.pb.cm),
        }
    }

    /// The views of the open presentation with their coordinate systems, the extent of
    /// their content (view coordinates, for the renderer's initial viewport), and for a
    /// lesson its beats, captions and explore windows.
    pub fn layout(&self) -> Json {
        let (projector, pres) = match &self.mode {
            Mode::Closed => return Json::Null,
            Mode::Interactive(i) => (&i.projector, &i.pres),
            Mode::Lesson(l) => (&l.pb.projector, &l.pb.pres),
        };
        let cm = self.cm().unwrap();
        let frames: Vec<Frame> = match &self.mode {
            // The first and last instants: a trace at the end covers the whole path.
            Mode::Interactive(i) => vec![i.frame_at(i.cfg_t0()), i.frame_at(i.end_time())],
            Mode::Lesson(l) => l.pb.export(10.0),
            Mode::Closed => vec![],
        };
        let views: Vec<Json> = projector
            .views
            .iter()
            .map(|(id, ctx, reps)| {
                let name = pres.views.iter().find(|v| &v.id == id).map(|v| v.name.clone()).unwrap_or_default();
                let mut v = json!({ "id": id, "name": name, "kind": ctx.kind() });
                // `title("...")` in a view or panel is its caption (D-076).
                if let Some(t) = reps.iter().find_map(|r| if let CKind::Title { value } = &r.kind { Some(value.clone()) } else { None }) {
                    v["title"] = json!(t);
                }
                match ctx {
                    ViewCtx::Spatial { space, px_per_m, y_up } => {
                        let axes = cm.spaces.iter().find(|s| &s.id == space).map(|s| s.axes.clone()).unwrap_or_default();
                        v["px_per_m"] = json!(px_per_m);
                        v["y_up"] = json!(y_up);
                        v["axes"] = json!(axes);
                        v["extent"] = json!(extent(frames.iter().flat_map(|f| f.views.iter().filter(|x| &x.id == id)).flat_map(|x| x.reps.iter()).map(|r| &r.shape)));
                    }
                    ViewCtx::Plot { x, y, dims, follow, units, window, .. } => {
                        v["x"] = json!([x.0, x.1]);
                        v["y"] = json!([y.0, y.1]);
                        v["follow"] = json!(follow);
                        v["window"] = json!(window);
                        // Each axis in its display unit (D-070): its text, and the value of one
                        // unit in coherent SI units, by which a renderer divides tick values.
                        let shown = |u: &Option<prismal_ir::Unit>, d| u.as_ref().map(|u| (u.text.clone(), u.scale)).unwrap_or_else(|| (unit_text(d), 1.0));
                        let (ux, uy) = (shown(&units[0], &dims.0), shown(&units[1], &dims.1));
                        v["units"] = json!([ux.0, uy.0]);
                        v["unit_scale"] = json!([ux.1, uy.1]);
                        // A time axis is elapsed time where things are drawn against it, not
                        // where a histogram counts durations (D-089).
                        v["x_elapsed"] = json!(dims.0 == prismal_ir::Dim::time() && !reps.iter().any(|r| matches!(r.kind, CKind::Histogram { .. })));
                    }
                    ViewCtx::Panel => {}
                }
                v
            })
            .collect();
        let permits: Vec<&String> = pres.permissions.iter().flat_map(|p| p.allows.iter()).collect();
        let mut out = json!({ "presentation": pres.name, "views": views, "permits": permits });
        if let Some(t) = &pres.title {
            out["title"] = json!(t);
        }
        // The author's page layout (PK-7.4a, D-063), with the views it leaves out after it.
        if let Some(l) = &pres.layout {
            let placed = l.views();
            let rest: Vec<Json> = pres.views.iter().filter(|v| !placed.contains(&&v.id)).map(|v| json!({ "view": v.id })).collect();
            let page = serde_json::to_value(l).unwrap_or(Json::Null);
            out["page"] = if rest.is_empty() { page } else { json!({ "column": std::iter::once(page).chain(rest).collect::<Vec<_>>() }) };
        }
        match &self.mode {
            Mode::Lesson(l) => {
                out["mode"] = json!("lesson");
                out["lesson"] = self.lesson_info(l);
            }
            Mode::Interactive(i) => {
                out["mode"] = json!("interactive");
                out["session"] = self.session_info(i);
            }
            Mode::Closed => {}
        }
        out
    }

    /// The clock of an interactive session: `dynamic` when the model evolves in time, the
    /// instant shown, the run's span, its diagnostics and the number of interventions.
    fn session_info(&self, i: &Interactive) -> Json {
        let run = &i.session.current;
        // An interactive run's failure policy is `pause` (RC-10.3, RC-10.4): the run holds
        // at its last committed state with the diagnostic, until an intervention changes it.
        let (status, failure) = match &run.status {
            prismal_runtime::RunStatus::Completed => ("running", Json::Null),
            prismal_runtime::RunStatus::Stopped(d) => ("paused", json!({ "message": d.message, "category": format!("{:?}", d.category).to_lowercase(), "t": d.t })),
            prismal_runtime::RunStatus::NotStarted(d) => ("failed", json!({ "message": d.message, "category": format!("{:?}", d.category).to_lowercase(), "t": d.t })),
        };
        json!({
            "status": status,
            "failure": failure,
            "dynamic": !i.cm.is_static,
            "t": i.t,
            "t0": run.config.t0,
            "end": i.end_time(),
            "interventions": i.session.log().len(),
            // The seed of the run shown (RC-13.1, D-084).
            "seed": i.seed(),
            "diagnostics": run.diagnostics.iter().map(|d| d.message.clone()).collect::<Vec<_>>(),
        })
    }

    /// Shows simulation instant `t` in an interactive session (play and seek). Returns the
    /// session's clock.
    pub fn seek(&mut self, t: f64) -> Json {
        match &mut self.mode {
            Mode::Interactive(i) => {
                i.seek(t);
            }
            _ => return Json::Null,
        }
        let Mode::Interactive(i) = &self.mode else { unreachable!() };
        self.session_info(i)
    }

    /// A new run of the session, with an empty log, at `t0` (RC section 12).
    pub fn reset(&mut self) -> Json {
        if let Mode::Interactive(i) = &mut self.mode {
            i.reset();
        }
        self.layout()
    }

    /// The session's clock after an action (interventions change the run and its span).
    pub fn session(&self) -> Json {
        match &self.mode {
            Mode::Interactive(i) => self.session_info(i),
            _ => Json::Null,
        }
    }

    fn lesson_info(&self, l: &Lesson) -> Json {
        let pb = &l.pb;
        let explore: Vec<Json> = match l.medium {
            Medium::Video => vec![],
            Medium::Interactive => pb
                .pres
                .timeline
                .iter()
                .flat_map(|t| t.scenes.iter().flat_map(|s| s.beats.iter()))
                .filter(|b| b.actions.iter().any(|a| matches!(a, TAction::Explore { .. })))
                .map(|b| {
                    let bt = pb.beat(&b.id);
                    json!({ "beat": b.name, "start": bt.start, "end": bt.end })
                })
                .collect(),
        };
        let beats: Vec<Json> = pb
            .pres
            .timeline
            .iter()
            .flat_map(|t| t.scenes.iter().flat_map(|s| s.beats.iter().map(move |b| (s, b))))
            .map(|(s, b)| {
                let bt = pb.beat(&b.id);
                json!({ "scene": s.name, "beat": b.name, "start": bt.start, "end": bt.end })
            })
            .collect();
        json!({
            "medium": match l.medium { Medium::Video => "video", Medium::Interactive => "interactive" },
            "end": pb.end,
            "beats": beats,
            "explore": explore,
            "waits": pb.waits,
            "captions": pb.captions,
            "inputs": l.inputs.iter().map(|i| json!({ "at": i.at, "input": i.input })).collect::<Vec<_>>(),
            "refusals": pb.refusals,
            "diagnostics": pb.diagnostics,
            "unsupported": pb.unsupported,
        })
    }

    /// The frame description at presentation instant `p` (a lesson; announcements due in
    /// `(p - dt, p]`), or the session's current frame (interactive). Formulas carry their
    /// MathML typesetting.
    pub fn frame(&self, p: f64, dt: f64) -> Json {
        let frame = match &self.mode {
            Mode::Closed => return Json::Null,
            Mode::Interactive(i) => {
                let mut f = i.frame();
                if dt > 0.0 {
                    f.announcements = i.events_between(i.t - dt, i.t);
                }
                f
            }
            Mode::Lesson(l) => l.pb.frame(p, dt),
        };
        let mut out = serde_json::to_value(&frame).expect("frame description");
        self.decorate(&frame, &mut out);
        out
    }

    /// Adds to a frame what every renderer draws it with (D-047): each view's viewport
    /// (`box`, the view coordinates shown; `size`, its natural drawn size in pixels; for a
    /// plot, `margin`) and the representation with keyboard focus.
    fn decorate(&self, frame: &Frame, out: &mut Json) {
        let grow = self.mode() == "interactive";
        for (vf, vj) in frame.views.iter().zip(out["views"].as_array_mut().into_iter().flatten()) {
            let Some(vs) = self.views.iter().find(|v| v.id == vf.id) else { continue };
            match vs.viewport(vf, grow) {
                Some(Viewport::Spatial { r#box, size, .. }) => {
                    vj["box"] = json!(r#box);
                    vj["size"] = json!(size);
                }
                Some(vp @ Viewport::Plot { x, y }) => {
                    vj["box"] = json!([x.0, y.0, x.1 - x.0, y.1 - y.0]);
                    vj["size"] = json!(vp.natural_size());
                    vj["margin"] = json!(PLOT_MARGIN);
                }
                None => {}
            }
        }
        if let Some(f) = &self.focus {
            if focus_order(frame).iter().any(|r| &r.id == f) {
                out["focus"] = json!(f);
            }
        }
    }

    /// The math box tree of a formula or equation representation (D-046), for a medium
    /// with its own math engine (MathML in a browser).
    pub fn math(&self, rep: &str) -> Option<prismal_present::math::MathBox> {
        let (cm, compiled): (&CModel, Vec<&CRep>) = match &self.mode {
            Mode::Closed => return None,
            Mode::Interactive(i) => (&i.cm, i.projector.views.iter().flat_map(|v| v.2.iter()).collect()),
            Mode::Lesson(l) => (&l.pb.cm, l.pb.projector.views.iter().flat_map(|v| v.2.iter()).chain(l.pb.shown.iter().map(|s| &s.rep)).collect()),
        };
        let c = compiled.into_iter().find(|c| c.rep.id == rep)?;
        match &c.kind {
            CKind::Formula { lhs, rhs, params, .. } => Some(prismal_present::math::formula(lhs, rhs, cm, params)),
            CKind::Equation { lhs, rhs, .. } => Some(prismal_present::math::equation(lhs, rhs, cm)),
            _ => None,
        }
    }

    /// The observations of the open presentation, as text lines per observation.
    pub fn observations(&self) -> Json {
        let mut out = serde_json::Map::new();
        match &self.mode {
            Mode::Closed => {}
            Mode::Interactive(i) => {
                for o in &i.pres.observations {
                    let text = match i.observe(&o.name) {
                        Ok(d) => fmt_data(&i.cm, &i.session.current, o, &d),
                        Err(e) => vec![e],
                    };
                    out.insert(o.name.clone(), json!(text));
                }
            }
            Mode::Lesson(l) => {
                for o in &l.pb.pres.observations {
                    if matches!(o.schedule, Schedule::On { .. }) {
                        let ty = obs_type(&l.pb.cm, o);
                        let lines: Vec<String> = l.pb.observed(&o.name).iter().map(|(p, v)| format!("at {} s: {}", fmt_num(*p), ty.as_ref().map(|t| fmt_value(v, t)).unwrap_or_else(|| v.to_string()))).collect();
                        out.insert(o.name.clone(), json!(lines));
                    }
                }
            }
        }
        Json::Object(out)
    }

    // ------------------------------------------------------------ interactive mode

    fn interactive(&mut self) -> Result<&mut Interactive, Json> {
        match &mut self.mode {
            Mode::Interactive(i) => Ok(i),
            _ => Err(json!("no interactive presentation is open")),
        }
    }

    fn outcome(r: Result<(), prismal_present::interact::Report>) -> Json {
        match r {
            Ok(()) => json!({ "ok": true }),
            Err(rep) => json!({ "ok": false, "why": format!("{:?}", rep.why).to_lowercase(), "message": rep.message }),
        }
    }

    /// Sets a control to a value in coherent SI units (the frame's `value`).
    pub fn set_control(&mut self, rep: &str, value: f64) -> Json {
        let i = match self.interactive() {
            Ok(i) => i,
            Err(e) => return json!({ "ok": false, "message": e }),
        };
        let ty = control_type(&i.cm, i.projector.views.iter().flat_map(|v| v.2.iter()), rep);
        let r = i.set_control(rep, literal(value, &ty));
        Self::outcome(r)
    }

    /// Supplies a new value of an input binding (by name or identity) at the instant shown
    /// (HI-4.3a, D-051): a number in coherent SI units, or 0 and 1 for a Boolean.
    pub fn set_input(&mut self, input: &str, value: f64) -> Json {
        let i = match self.interactive() {
            Ok(i) => i,
            Err(e) => return json!({ "ok": false, "message": e }),
        };
        let ty = i.cm.ir.bindings.iter().find(|b| b.id == input || b.name == input).map(|b| b.ty.clone()).unwrap_or_else(prismal_ir::Type::real);
        let r = i.set_input(input, literal(value, &ty));
        Self::outcome(r)
    }

    /// A key on a focused control or draggable representation: `left`, `right`, `up`, `down`.
    pub fn key(&mut self, rep: &str, key: &str) -> Json {
        let key = match key {
            "left" => Key::Left,
            "right" => Key::Right,
            "up" => Key::Up,
            _ => Key::Down,
        };
        match self.interactive() {
            Ok(i) => Self::outcome(i.key(rep, key)),
            Err(e) => json!({ "ok": false, "message": e }),
        }
    }

    /// Requests an event declared `on request` (by name or identity) at the instant shown,
    /// with its payload (HI-4.3b, D-050, D-059): a number in coherent SI units (0 and 1 for a
    /// Boolean), an array of numbers for a vector or a point, a case by name, a member as `"balls[2]"`, and
    /// several payloads as an array with one entry each.
    pub fn request(&mut self, event: &str, payload: Option<&Json>) -> Json {
        let i = match self.interactive() {
            Ok(i) => i,
            Err(e) => return json!({ "ok": false, "message": e }),
        };
        let declared = i.cm.ir.events.iter().find(|e| e.id == event || e.name == event).and_then(|e| e.payload.clone());
        let value = match (payload, &declared) {
            (Some(v), Some(p)) => match payload_expr(p, v) {
                Ok(e) => Some(e),
                Err(m) => return json!({ "ok": false, "why": "refused", "message": m }),
            },
            (Some(_), None) => return json!({ "ok": false, "why": "refused", "message": format!("`{event}` declares no payload") }),
            (None, _) => None,
        };
        Self::outcome(i.request(event, value))
    }

    /// Clicks or activates a representation that requests an event (D-059).
    pub fn click(&mut self, rep: &str) -> Json {
        match self.interactive() {
            Ok(i) => Self::outcome(i.click(rep)),
            Err(e) => json!({ "ok": false, "message": e }),
        }
    }

    /// Clicks a point of a view, in view coordinates, where no representation takes the
    /// click (D-060).
    pub fn click_at(&mut self, view: &str, x: f64, y: f64) -> Json {
        let id = self.view_index(view).map(|vi| self.views[vi].id.clone()).unwrap_or_else(|| view.to_string());
        match self.interactive() {
            Ok(i) => Self::outcome(i.click_at(&id, [x, y])),
            Err(e) => json!({ "ok": false, "message": e }),
        }
    }

    /// Presses a button: requests its event at the instant shown (D-027).
    pub fn press(&mut self, rep: &str) -> Json {
        match self.interactive() {
            Ok(i) => Self::outcome(i.press(rep)),
            Err(e) => json!({ "ok": false, "message": e }),
        }
    }

    pub fn pointer_down(&mut self, rep: &str, part: Option<&str>) -> Json {
        match self.interactive() {
            Ok(i) => {
                let mut out = Self::outcome(i.pointer_down(rep, part));
                // Drag mode `live` keeps the clock running (PK-10.9, D-071).
                out["live"] = json!(i.drag_live());
                out
            }
            Err(e) => json!({ "ok": false, "message": e }),
        }
    }

    /// Moves the pointer during a drag, in view coordinates. Returns whether the proposal
    /// is valid, with the runtime's message when it is not (PK-10.6).
    pub fn pointer_move(&mut self, x: f64, y: f64) -> Json {
        match self.interactive() {
            Ok(i) => {
                let valid = i.pointer_move([x, y]);
                json!({ "ok": valid, "message": i.preview().and_then(|p| p.message.clone()) })
            }
            Err(e) => json!({ "ok": false, "message": e }),
        }
    }

    pub fn pointer_up(&mut self) -> Json {
        match self.interactive() {
            Ok(i) => match i.pointer_up() {
                Ok(committed) => json!({ "ok": true, "committed": committed }),
                Err(r) => json!({ "ok": false, "message": r.message }),
            },
            Err(e) => json!({ "ok": false, "message": e }),
        }
    }

    pub fn cancel(&mut self) {
        if let Ok(i) = self.interactive() {
            i.cancel();
        }
    }

    pub fn undo(&mut self) {
        if let Ok(i) = self.interactive() {
            i.undo();
        }
    }

    pub fn redo(&mut self) {
        if let Ok(i) = self.interactive() {
            i.redo();
        }
    }

    // ------------------------------------------------------------ raw input (HI-4.5, D-047)

    /// The frame the learner sees: the session's current frame, or the lesson's at `time`.
    fn current_frame(&self, time: f64) -> Option<Frame> {
        match &self.mode {
            Mode::Closed => None,
            Mode::Interactive(i) => Some(i.frame()),
            Mode::Lesson(l) => Some(l.pb.frame(time, 0.0)),
        }
    }

    fn permits(&self, what: &str) -> bool {
        let pres = match &self.mode {
            Mode::Closed => return false,
            Mode::Interactive(i) => &i.pres,
            Mode::Lesson(l) => &l.pb.pres,
        };
        pres.permissions.iter().any(|p| p.allows.iter().any(|a| a == what))
    }

    fn view_index(&self, view: &str) -> Option<usize> {
        self.views.iter().position(|v| v.id == view || v.name == view)
    }

    /// A pointer event the host captured (HI-4.5). The engine finds its target: pressing on
    /// a draggable part starts a drag (sessions only, PK-10.5); pressing elsewhere pans a
    /// spatial view when the presentation permits `pan`. Moving without a gesture reports
    /// what is under the pointer (`hover`), so that the host can show that it can be
    /// grabbed. Answers `handled`, the `action` (`drag`, `pan`, `cancel` or null), the
    /// `target`, and for a drag its outcome as the semantic inputs answer it.
    pub fn pointer(&mut self, e: &PointerEvent) -> Json {
        let out = self.pointer_event(e);
        self.sync_sample();
        out
    }

    /// Function graphs follow the learner's zoom and pan of their plot (D-070): each plot's
    /// projector draws them over the `x` range shown.
    fn sync_sample(&mut self) {
        let plots: Vec<(String, Option<(f64, f64)>)> =
            self.views.iter().filter(|v| matches!(v.kind, ViewKind::Plot { .. })).map(|v| (v.id.clone(), v.user.map(|b| (b[0], b[0] + b[2])))).collect();
        let pj = match &mut self.mode {
            Mode::Closed => return,
            Mode::Interactive(i) => &mut i.projector,
            Mode::Lesson(l) => &mut l.pb.projector,
        };
        for (id, x) in plots {
            pj.set_sample_x(&id, x);
        }
    }

    fn pointer_event(&mut self, e: &PointerEvent) -> Json {
        let Some(vi) = self.view_index(e.view) else { return merge(unhandled(), json!({ "message": format!("no view `{}`", e.view) })) };
        if let Some(size) = e.size {
            self.views[vi].size = Some(size);
        }
        let Some(frame) = self.current_frame(e.time) else { return unhandled() };
        let Some(vf) = frame.views.iter().find(|v| v.id == self.views[vi].id) else { return unhandled() };
        let session = self.mode() == "interactive";
        let Some(vp) = self.views[vi].viewport(vf, session) else { return unhandled() };
        let map = vp.map(self.views[vi].size.unwrap_or(vp.natural_size()));
        let p = [e.x, e.y];
        match e.phase {
            "down" => {
                // A second touch during a pan pinches the view, where zoom is permitted.
                if let Some(Gesture::Pan { view, id, at, before, .. }) = self.gesture.clone() {
                    if view == vi && e.pointer == "touch" && e.id.is_some() && e.id != id && self.permits("zoom") {
                        if let Some(shown) = vp.shown() {
                            let q0 = map.to_view([(at[0] + p[0]) / 2.0, (at[1] + p[1]) / 2.0]);
                            self.gesture = Some(Gesture::Pinch { view, ids: [id, e.id], p0: [at, p], p: [at, p], q0, shown, map, before });
                            return json!({ "handled": true, "action": "pinch" });
                        }
                    }
                }
                if self.gesture.is_some() {
                    return merge(unhandled(), json!({ "message": "a gesture is already in progress" }));
                }
                if session {
                    let tol = tolerance(e.pointer);
                    if let Some(t) = hit(vf, &map, p, tol) {
                        // D-059: a press that does not move past the tolerance is a click.
                        if !t.drag {
                            self.gesture = Some(Gesture::Click { view: vi, rep: t.rep.clone(), from: p, tol });
                            self.focus = Some(t.rep.clone());
                            return json!({ "handled": true, "action": "click", "ok": true, "target": target_json(&t) });
                        }
                        let r = self.pointer_down(&t.rep, t.part.as_deref());
                        if r["ok"] == true {
                            let click = if t.click { Some((t.rep.clone(), p, tol)) } else { None };
                            self.gesture = Some(Gesture::Drag { view: vi, click });
                            self.focus = Some(t.rep.clone());
                        }
                        return merge(r, json!({ "handled": true, "action": "drag", "target": target_json(&t) }));
                    }
                }
                // D-060: on an empty point of a view that requests an event when clicked, a
                // press that does not move past the tolerance clicks the point.
                let click = if session && vf.click.is_some() { Some(tolerance(e.pointer)) } else { None };
                match vp.shown() {
                    Some(shown) if self.permits("pan") => {
                        self.gesture = Some(Gesture::Pan { view: vi, from: p, shown, map, before: self.views[vi].user, click, id: e.id, at: p });
                        json!({ "handled": true, "action": "pan", "target": null })
                    }
                    _ => match click {
                        Some(tol) => {
                            self.gesture = Some(Gesture::Point { view: vi, from: p, tol });
                            json!({ "handled": true, "action": "click", "ok": true, "target": null })
                        }
                        None => unhandled(),
                    },
                }
            }
            "move" => match self.gesture.clone() {
                // Within the tolerance of the press, a clickable representation is not dragged.
                Some(Gesture::Drag { view, click: Some((_, from, tol)) }) if view == vi && dist(p, from) <= tol => json!({ "handled": true, "action": "drag", "ok": true }),
                Some(Gesture::Drag { view, .. }) if view == vi => {
                    self.gesture = Some(Gesture::Drag { view, click: None });
                    let q = map.to_view(p);
                    merge(self.pointer_move(q[0], q[1]), json!({ "handled": true, "action": "drag" }))
                }
                Some(Gesture::Click { view, from, tol, .. }) if view == vi => {
                    if dist(p, from) > tol {
                        self.gesture = None;
                        return json!({ "handled": true, "action": "cancel", "message": "moved: not a click" });
                    }
                    json!({ "handled": true, "action": "click", "ok": true })
                }
                // Within the tolerance of the press, a clickable view is not panned (D-060).
                Some(Gesture::Pinch { view, ids, p0, p: mut at, q0, shown, map, before }) if view == vi => {
                    let Some(k) = ids.iter().position(|i| *i == e.id) else { return unhandled() };
                    at[k] = p;
                    self.gesture = Some(Gesture::Pinch { view, ids, p0, p: at, q0, shown, map, before });
                    // The box grows as the touches close in; the view point under the first
                    // midpoint stays under the midpoint.
                    let (d0, d) = (dist(p0[0], p0[1]), dist(at[0], at[1]));
                    if d0 < 1.0 || d < 1.0 {
                        return json!({ "handled": true, "action": "pinch" });
                    }
                    let f = d0 / d;
                    let m0 = [(p0[0][0] + p0[1][0]) / 2.0, (p0[0][1] + p0[1][1]) / 2.0];
                    let m = [(at[0][0] + at[1][0]) / 2.0, (at[0][1] + at[1][1]) / 2.0];
                    // In view coordinates with the mapping the pinch began with (spatial or plot).
                    let (qm0, qm) = (map.to_view(m0), map.to_view(m));
                    let bx = q0[0] - f * (qm[0] - qm0[0]) - f * (q0[0] - shown[0]);
                    let by = q0[1] - f * (qm[1] - qm0[1]) - f * (q0[1] - shown[1]);
                    self.views[vi].user = Some([bx, by, shown[2] * f, shown[3] * f]);
                    json!({ "handled": true, "action": "pinch", "zoom": 1.0 / f })
                }
                // Moves of another pointer than the one panning are not the pan's.
                Some(Gesture::Pan { id, .. }) if e.id.is_some() && id.is_some() && e.id != id => unhandled(),
                Some(Gesture::Pan { view, from, click: Some(tol), .. }) if view == vi && dist(p, from) <= tol => {
                    if let Some(Gesture::Pan { at, .. }) = self.gesture.as_mut() {
                        *at = p;
                    }
                    json!({ "handled": true, "action": "pan" })
                }
                Some(Gesture::Point { view, from, tol }) if view == vi => {
                    if dist(p, from) > tol {
                        self.gesture = None;
                        return json!({ "handled": true, "action": "cancel", "message": "moved: not a click" });
                    }
                    json!({ "handled": true, "action": "click", "ok": true })
                }
                Some(Gesture::Pan { view, from, shown, map, before, id, .. }) if view == vi => {
                    self.gesture = Some(Gesture::Pan { view, from, shown, map, before, click: None, id, at: p });
                    // The view point under the press stays under the pointer.
                    let (q0, q) = (map.to_view(from), map.to_view(p));
                    self.views[vi].user = Some([shown[0] + q0[0] - q[0], shown[1] + q0[1] - q[1], shown[2], shown[3]]);
                    json!({ "handled": true, "action": "pan" })
                }
                Some(_) => unhandled(),
                None => {
                    let over = if session { hit(vf, &map, p, tolerance(e.pointer)) } else { None };
                    json!({ "handled": false, "action": null, "hover": over.as_ref().map(target_json) })
                }
            },
            "up" => match self.gesture.take() {
                Some(Gesture::Drag { click: Some((rep, ..)), .. }) => {
                    self.cancel();
                    merge(self.click(&rep), json!({ "handled": true, "action": "click", "target": { "rep": rep } }))
                }
                Some(Gesture::Click { rep, .. }) => merge(self.click(&rep), json!({ "handled": true, "action": "click", "target": { "rep": rep } })),
                Some(Gesture::Drag { .. }) => merge(self.pointer_up(), json!({ "handled": true, "action": "drag" })),
                Some(Gesture::Pan { from, click: Some(_), .. } | Gesture::Point { from, .. }) => {
                    let q = map.to_view(from);
                    let id = self.views[vi].id.clone();
                    merge(self.click_at(&id, q[0], q[1]), json!({ "handled": true, "action": "click", "target": null, "at": q }))
                }
                Some(Gesture::Pan { .. }) => json!({ "handled": true, "action": "pan" }),
                // Lifting either touch ends a pinch; the view stays where it was pinched to.
                Some(Gesture::Pinch { .. }) => json!({ "handled": true, "action": "pinch" }),
                None => unhandled(),
            },
            "cancel" => self.cancel_gesture(),
            other => merge(unhandled(), json!({ "message": format!("unknown pointer phase `{other}`") })),
        }
    }

    /// Ends a drag without committing it, or returns a pan to where it began.
    fn cancel_gesture(&mut self) -> Json {
        match self.gesture.take() {
            Some(Gesture::Drag { .. }) => self.cancel(),
            Some(Gesture::Click { .. } | Gesture::Point { .. }) => {}
            Some(Gesture::Pan { view, before, .. } | Gesture::Pinch { view, before, .. }) => self.views[view].user = before,
            None => return unhandled(),
        }
        json!({ "handled": true, "action": "cancel" })
    }

    /// A wheel step over a view (HI-4.5): zooms a spatial view about the pointer when the
    /// presentation permits `zoom`. `delta` is the vertical scroll in pixels (positive: out).
    pub fn wheel(&mut self, view: &str, x: f64, y: f64, size: Option<[f64; 2]>, delta: f64, time: f64) -> Json {
        let Some(vi) = self.view_index(view) else { return unhandled() };
        if let Some(size) = size {
            self.views[vi].size = Some(size);
        }
        if !self.permits("zoom") {
            return unhandled();
        }
        let Some(frame) = self.current_frame(time) else { return unhandled() };
        let Some(vf) = frame.views.iter().find(|v| v.id == self.views[vi].id) else { return unhandled() };
        let Some(vp) = self.views[vi].viewport(vf, self.mode() == "interactive") else { return unhandled() };
        let Some(shown) = vp.shown() else { return unhandled() };
        let q = vp.map(self.views[vi].size.unwrap_or(vp.natural_size())).to_view([x, y]);
        let k = (delta * 0.0015).exp();
        self.views[vi].user = Some([q[0] - (q[0] - shown[0]) * k, q[1] - (q[1] - shown[1]) * k, shown[2] * k, shown[3] * k]);
        self.sync_sample();
        json!({ "handled": true, "action": "zoom" })
    }

    /// Returns a view to its own framing, undoing the learner's zoom and pan.
    pub fn view_reset(&mut self, view: &str) -> Json {
        match self.view_index(view) {
            Some(vi) => {
                self.views[vi].user = None;
                self.sync_sample();
                json!({ "handled": true, "action": "view_reset" })
            }
            None => unhandled(),
        }
    }

    /// Gives keyboard focus to a representation, or takes it away (`None`), for hosts whose
    /// own focus system moves focus (HI-4.5).
    pub fn set_focus(&mut self, rep: Option<&str>, time: f64) -> Json {
        let Some(rep) = rep else {
            self.focus = None;
            return json!({ "handled": true, "action": "focus", "focus": null });
        };
        let frame = self.current_frame(time);
        let found = frame.as_ref().and_then(|f| focus_order(f).into_iter().find(|r| r.id == rep || r.name.as_deref() == Some(rep)).map(|r| r.id.clone()));
        match found {
            Some(id) => {
                self.focus = Some(id.clone());
                json!({ "handled": true, "action": "focus", "focus": id })
            }
            None => json!({ "handled": false, "action": null, "ok": false, "message": format!("`{rep}` does not take focus") }),
        }
    }

    /// A key the host captured, named as in the W3C `KeyboardEvent.key` values (HI-4.5):
    /// `Tab` (with `shift`, backwards) moves focus through the focus order and gives it back
    /// to the host past either end; arrow keys step the focused draggable representation or
    /// control (PK-11.2); `Enter` and space press a focused button or flip a focused toggle;
    /// `Escape` cancels a gesture. A key the engine does not use is answered
    /// `handled: false`, for the host's own use (play and pause, for example).
    pub fn key_down(&mut self, key: &str, shift: bool, time: f64) -> Json {
        let Some(frame) = self.current_frame(time) else { return unhandled() };
        let order: Vec<RepFrame> = focus_order(&frame).into_iter().cloned().collect();
        let focused = self.focus.as_ref().and_then(|f| order.iter().position(|r| &r.id == f));
        let session = self.mode() == "interactive";
        match key {
            "Tab" => {
                let next = match (focused, shift) {
                    (None, false) => Some(0).filter(|_| !order.is_empty()),
                    (None, true) => order.len().checked_sub(1),
                    (Some(i), false) => Some(i + 1).filter(|&j| j < order.len()),
                    (Some(i), true) => i.checked_sub(1),
                };
                self.focus = next.map(|i| order[i].id.clone());
                json!({ "handled": self.focus.is_some(), "action": "focus", "focus": self.focus })
            }
            "Escape" => self.cancel_gesture(),
            "ArrowLeft" | "ArrowRight" | "ArrowUp" | "ArrowDown" => {
                let Some(r) = focused.map(|i| &order[i]) else { return unhandled() };
                let dir = &key[5..].to_lowercase();
                let toggle_or_button = matches!(r.shape, Shape::Button { .. }) || matches!(&r.shape, Shape::Control { control, .. } if control == "toggle");
                if toggle_or_button {
                    return unhandled();
                }
                if session {
                    return merge(self.key(&r.id, dir), json!({ "handled": true, "action": "step", "target": { "rep": r.id } }));
                }
                match &r.shape {
                    Shape::Control { value, min, max, step, .. } => {
                        let step = step.or(min.zip(*max).map(|(a, b)| (b - a) / 100.0)).unwrap_or(1.0);
                        let sign = if dir == "right" || dir == "up" { 1.0 } else { -1.0 };
                        let v = (value + sign * step).clamp(min.unwrap_or(f64::MIN), max.unwrap_or(f64::MAX));
                        let id = r.id.clone();
                        self.lesson_step(time, &id, v)
                    }
                    _ => json!({ "handled": true, "action": "step", "ok": false, "message": "in a lesson the learner acts through the controls of explore beats (D-025)" }),
                }
            }
            "Enter" | " " => {
                let Some(r) = focused.map(|i| &order[i]) else { return unhandled() };
                let id = r.id.clone();
                match &r.shape {
                    // Enter or space activates a clickable representation (D-026, D-059).
                    _ if r.click.is_some() && session => merge(self.click(&id), json!({ "handled": true, "action": "click", "target": { "rep": id } })),
                    _ if r.click.is_some() => json!({ "handled": true, "action": "click", "ok": false, "message": "clicks act in labs; in a lesson the timeline requests events" }),
                    Shape::Button { .. } if session => merge(self.press(&id), json!({ "handled": true, "action": "press", "target": { "rep": id } })),
                    Shape::Button { .. } => {
                        let res = self.lesson_press(time, &id);
                        Self::lesson_answer("press", time, res)
                    }
                    Shape::Control { control, value, .. } if control == "toggle" => {
                        let v = if *value != 0.0 { 0.0 } else { 1.0 };
                        if session {
                            merge(self.set_control(&id, v), json!({ "handled": true, "action": "toggle", "target": { "rep": id } }))
                        } else {
                            self.lesson_step(time, &id, v)
                        }
                    }
                    _ => unhandled(),
                }
            }
            _ => unhandled(),
        }
    }

    /// A control set from the keyboard in a lesson, answered as the raw inputs are.
    fn lesson_step(&mut self, time: f64, rep: &str, value: f64) -> Json {
        let res = self.lesson_set_control(time, rep, value);
        Self::lesson_answer("step", time, res)
    }

    /// A lesson input's answer to raw input: refused when the lesson refused it at `time`.
    fn lesson_answer(action: &str, time: f64, res: Result<Json, Json>) -> Json {
        match res {
            Ok(info) => {
                let refused = info["refusals"].as_array().into_iter().flatten().find(|r| r["at"].as_f64() == Some(time)).map(|r| r["reason"].clone());
                match refused {
                    Some(reason) => json!({ "handled": true, "action": action, "ok": false, "message": reason, "lesson": info }),
                    None => json!({ "handled": true, "action": action, "ok": true, "lesson": info }),
                }
            }
            Err(e) => json!({ "handled": true, "action": action, "ok": false, "message": e }),
        }
    }

    // ------------------------------------------------------------ lesson mode

    fn lesson_input(&mut self, input: Input) -> Result<Json, Json> {
        let Mode::Lesson(l) = &self.mode else { return Err(json!("no lesson is open")) };
        let mut inputs = l.inputs.clone();
        inputs.push(input);
        let next = Lesson::replay(&self.prog, &l.pres, l.medium, inputs)?;
        self.mode = Mode::Lesson(Box::new(next));
        let Mode::Lesson(l) = &self.mode else { unreachable!() };
        Ok(self.lesson_info(l))
    }

    /// A learner's control setting at presentation instant `p` (an explore beat's control;
    /// refused outside explore beats, PK-9.8). Returns the recomputed lesson.
    pub fn lesson_set_control(&mut self, p: f64, rep: &str, value: f64) -> Result<Json, Json> {
        let Mode::Lesson(l) = &self.mode else { return Err(json!("no lesson is open")) };
        let shown: Vec<&CRep> = l.pb.shown.iter().map(|s| &s.rep).collect();
        let Some(CKind::Control { control, binding, idx, .. }) = shown.iter().find(|r| r.rep.id == rep).map(|r| &r.kind) else {
            return Err(json!(format!("no control `{rep}` in this lesson")));
        };
        let value = literal(value, &l.pb.cm.bindings[*idx].ty);
        let input = Input { at: p, input: LearnerInput::SetControl { control: control.clone(), binding: binding.clone(), value } };
        self.lesson_input(input)
    }

    /// A learner's press of a button at presentation instant `p` (D-069): an explore beat's
    /// button requests its event on the learner's branch; refused outside explore beats
    /// (PK-9.8). Runtime-control buttons act in labs only.
    pub fn lesson_press(&mut self, p: f64, rep: &str) -> Result<Json, Json> {
        let Mode::Lesson(l) = &self.mode else { return Err(json!("no lesson is open")) };
        let event = l.pb.shown.iter().map(|s| &s.rep).find(|r| r.rep.id == rep).map(|r| &r.kind);
        let event = match event {
            Some(CKind::Button { event, .. }) => event.clone(),
            // An explore beat's `restart` or `reseed` starts the learner's branch again (D-088).
            Some(CKind::RunButton { control, .. }) if ["restart", "reseed"].contains(&control.as_str()) => {
                let input = LearnerInput::Control { control: control.clone() };
                return self.lesson_input(Input { at: p, input });
            }
            Some(CKind::RunButton { control, .. }) => return Err(json!(format!("`{control}` is a runtime control of labs; in a lesson the learner directs the timeline (PK-9.7)"))),
            _ => return Err(json!(format!("no button `{rep}` in this lesson"))),
        };
        self.lesson_input(Input { at: p, input: LearnerInput::Press { event } })
    }

    /// The learner continues at presentation instant `p` (ends an explore beat).
    pub fn lesson_continue(&mut self, p: f64) -> Result<Json, Json> {
        self.lesson_input(Input { at: p, input: LearnerInput::Continue })
    }

    /// Plays the lesson again without the learner's inputs.
    pub fn lesson_restart(&mut self) -> Result<Json, Json> {
        let Mode::Lesson(l) = &self.mode else { return Err(json!("no lesson is open")) };
        let next = Lesson::replay(&self.prog, &l.pres, l.medium, vec![])?;
        self.mode = Mode::Lesson(Box::new(next));
        Ok(self.layout())
    }

}

/// A control's value as a literal: a Boolean for a toggle, else a number in coherent SI units.
fn literal(value: f64, ty: &prismal_ir::Type) -> prismal_ir::Expr {
    match ty {
        prismal_ir::Type::Boolean => prismal_ir::Expr::Bool { bool: value != 0.0 },
        // A select's value is the number of a case, from 0 (D-094).
        prismal_ir::Type::Enum { cases, .. } => prismal_ir::Expr::Case { case: cases.get(value.max(0.0) as usize).or(cases.last()).cloned().unwrap_or_default() },
        _ => si_literal(value, ty),
    }
}

/// A payload given in the protocol, as the expression a request supplies (HI-4.3b).
fn payload_expr(p: &prismal_ir::Payload, v: &Json) -> Result<prismal_ir::Expr, String> {
    use prismal_ir::{Expr, Type};
    if !p.items.is_empty() {
        let items = v.as_array().filter(|a| a.len() == p.items.len()).ok_or_else(|| format!("the payload `{}` is an array of {} values", p.name, p.items.len()))?;
        return Ok(Expr::Tuple { tuple: p.items.iter().zip(items).map(|(i, x)| payload_expr(i, x)).collect::<Result<_, _>>()? });
    }
    // D-059: a member is named by its collection and number, `balls[2]`, or by the number.
    if let Some(path) = &p.members {
        let k = match v {
            Json::String(s) => s.strip_prefix(path.as_str()).and_then(|r| r.strip_prefix('[')).and_then(|r| r.strip_suffix(']')).and_then(|n| n.trim().parse::<u32>().ok()),
            Json::Number(n) => n.as_u64().map(|n| n as u32),
            _ => None,
        };
        return match k {
            Some(k) => Ok(Expr::Num { num: k as f64, unit: None }),
            None => Err(format!("the payload `{}` is a member of `{path}`: `\"{path}[1]\"`", p.name)),
        };
    }
    let nums = |a: &Vec<Json>| a.iter().map(Json::as_f64).collect::<Option<Vec<f64>>>();
    match (v, &p.ty) {
        (Json::Bool(b), Type::Boolean) => Ok(Expr::Bool { bool: *b }),
        (Json::Number(n), ty) => Ok(literal(n.as_f64().unwrap_or(0.0), ty)),
        (Json::Array(a), Type::Vector { dim, .. }) => match nums(a) {
            Some(xs) => Ok(Expr::Tuple { tuple: xs.into_iter().map(|x| si_literal(x, &Type::Quantity { dim: *dim })).collect() }),
            None => Err(format!("the payload `{}` is an array of numbers", p.name)),
        },
        // A point: its coordinates in metres in the space's standard axes.
        (Json::Array(a), Type::Point { space }) => match nums(a) {
            Some(xs) => Ok(prismal_ir::build::origin(space) + Expr::Tuple { tuple: xs.into_iter().map(|x| prismal_ir::build::num(x, "m")).collect() }),
            None => Err(format!("the payload `{}` is an array of coordinates", p.name)),
        },
        // An enumeration's case by name (D-049).
        (Json::String(c), Type::Enum { cases, .. }) if cases.contains(c) => Ok(Expr::Case { case: c.clone() }),
        (_, Type::Enum { cases, .. }) => Err(format!("the payload `{}` is one of {}", p.name, cases.iter().map(|c| format!("\"{c}\"")).collect::<Vec<_>>().join(", "))),
        _ => Err(format!("the payload `{}` is not given as its type needs", p.name)),
    }
}

fn control_type<'a>(cm: &CModel, reps: impl Iterator<Item = &'a CRep>, rep: &str) -> prismal_ir::Type {
    for r in reps {
        if r.rep.id == rep || r.rep.name.as_deref() == Some(rep) {
            if let CKind::Control { idx, .. } = &r.kind {
                return cm.bindings[*idx].ty.clone();
            }
        }
    }
    prismal_ir::Type::real()
}


/// A view's coordinate system from its entry in the layout.
fn view_state(v: &Json) -> ViewState {
    let (id, name) = (v["id"].as_str().unwrap_or_default(), v["name"].as_str().unwrap_or_default());
    let n = |k: &str, i: usize| v[k][i].as_f64().unwrap_or(0.0);
    let kind = match v["kind"].as_str() {
        Some("spatial") => return ViewState::spatial(id, name, [n("extent", 0), n("extent", 1), n("extent", 2), n("extent", 3)]),
        Some("plot") => {
            let f = |i: usize| v["follow"][i].as_bool().unwrap_or(false);
            ViewKind::Plot { x: (n("x", 0), n("x", 1)), y: (n("y", 0), n("y", 1)), follow: [f(0), f(1)], window: v["window"].as_f64() }
        }
        _ => ViewKind::Panel,
    };
    ViewState { id: id.into(), name: name.into(), kind, user: None, size: None }
}

/// The extent `[xmin, ymin, xmax, ymax]` of shapes in view coordinates, including the origin
/// and the anchors of texts placed in the view (D-080).
fn extent<'a>(shapes: impl Iterator<Item = &'a Shape>) -> [f64; 4] {
    fn walk(s: &Shape, add: &mut impl FnMut(&[f64; 2])) {
        match s {
            // A text placed in the view is framed by its anchor; it runs on from there.
            Shape::Point { at } | Shape::Note { at, .. } => add(at),
            Shape::Arrow { from, to } | Shape::Segment { from, to } => {
                add(from);
                add(to);
            }
            Shape::Polyline { points } | Shape::Polygon { points } => points.iter().for_each(add),
            Shape::Ellipse { .. } => s.curve_points(64).iter().for_each(add),
            Shape::Raster { from, to, .. } => {
                add(from);
                add(to);
            }
            Shape::Group { members } => members.iter().for_each(|m| walk(&m.shape, add)),
            _ => {}
        }
    }
    let mut e = [0.0f64, 0.0, 0.0, 0.0];
    let mut add = |p: &[f64; 2]| {
        if p[0].is_finite() && p[1].is_finite() {
            e[0] = e[0].min(p[0]);
            e[1] = e[1].min(p[1]);
            e[2] = e[2].max(p[0]);
            e[3] = e[3].max(p[1]);
        }
    };
    for s in shapes {
        walk(s, &mut add);
    }
    e
}

fn obs_type(cm: &CModel, o: &Observation) -> Option<prismal_ir::Type> {
    match &o.source {
        Source::Expr { expr } => compile_expr(cm, expr, None).ok().map(|x| x.1),
        _ => None,
    }
}

fn fmt_op(cm: &CModel, run: &prismal_runtime::Run, t: f64, op: &Op) -> String {
    match op {
        Op::Set { target, value } => {
            // A proposal is shown by its value: a constant, or evaluated on the state from
            // before the intervention at its instant (it may read other bindings).
            let v = match prismal_present::constant(cm, value) {
                Ok(v) => Some(v),
                Err(_) => run.committed.iter().find(|c| c.t == t).and_then(|c| compile_expr(cm, value, None).ok().and_then(|(ce, _)| run.eval_state(&ce, &c.vals, t).ok())),
            };
            let shown = match (v, cm.index.get(&target.binding)) {
                (Some(v), Some(&i)) => fmt_binding(cm, i, &v),
                _ => print(value, cm, &[]),
            };
            format!("set {} = {shown}", symbol(cm, &target.binding))
        }
        other => format!("{other:?}"),
    }
}

/// Observation data as text lines (PK-3.5).
fn fmt_data(cm: &CModel, run: &prismal_runtime::Run, o: &Observation, d: &Data) -> Vec<String> {
    let ty = obs_type(cm, o);
    let v = |x: &prismal_kernel::Value| ty.as_ref().map(|t| fmt_value(x, t)).unwrap_or_else(|| x.to_string());
    match d {
        Data::Value(x) => vec![v(x)],
        Data::Series(s) => s.iter().map(|(t, x)| format!("t = {} s: {}", fmt_num(*t), v(x))).collect(),
        // An occurrence with a payload shows it: `kick(2 N s) at 1 s` (D-050).
        Data::Events(es) => es
            .iter()
            .map(|e| {
                let payload = match (&e.payload, cm.ir.events.iter().find(|x| x.id == e.event).and_then(|x| x.payload.as_ref())) {
                    (Some(v), Some(p)) => format!("({})", fmt_payload(v, p)),
                    _ => String::new(),
                };
                format!("{}{payload} at {} s", e.name, fmt_num(e.t))
            })
            .collect(),
        Data::Diagnostics(ds) => ds.iter().map(|d| d.message.clone()).collect(),
        Data::Interventions(is) => is
            .iter()
            .map(|s| match &s.action {
                Action::Intervene(ops) => format!("at {} s: {}", fmt_num(s.t), ops.iter().map(|o| fmt_op(cm, run, s.t, o)).collect::<Vec<_>>().join("; ")),
                Action::Request(e) => format!("at {} s: request {}", fmt_num(s.t), e.rsplit('.').next().unwrap_or(e)),
                Action::RequestWith(e, v) => {
                    let declared = cm.ir.events.iter().find(|x| &x.id == e).and_then(|x| x.payload.as_ref());
                    let shown = match (declared, prismal_present::constant_as(cm, v, declared.map(|p| &p.ty))) {
                        (Some(p), Ok(x)) => fmt_payload(&x, p),
                        _ => print(v, cm, &[]),
                    };
                    format!("at {} s: request {}({shown})", fmt_num(s.t), e.rsplit('.').next().unwrap_or(e))
                }
                Action::Input(b, v) => format!("at {} s: input {} = {}", fmt_num(s.t), cm.ir.binding(b).map(|x| x.name.as_str()).unwrap_or(b), print(v, cm, &[])),
            })
            .collect(),
    }
}
