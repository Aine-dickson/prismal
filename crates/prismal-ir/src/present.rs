//! Presentation and run IR (04-ir section 7; 03-presentation-kernel.md).
//!
//! A presentation refers to model elements only by identity (PK-2.3). Expressions are model
//! expressions (04-ir section 6) read in the model's scope; inside an inverse, `{"param": 0}`
//! is the gesture's value (a pointer position).

use crate::{Expr, Id, Op};
use serde::{Deserialize, Serialize};

/// A presentation (PK-2.1).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Presentation {
    pub id: Id,
    pub name: String,
    pub model: Id,
    #[serde(default)]
    pub observations: Vec<Observation>,
    /// Instruments that observations and sampled sources read through (PK-3.7, D-087).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub instruments: Vec<Instrument>,
    #[serde(default)]
    pub views: Vec<View>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub permissions: Vec<Permission>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub timeline: Option<Timeline>,
    /// Where the views go on the page (PK-7.4, D-063); `None` leaves it to the renderer.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub layout: Option<Layout>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub notes: Vec<String>,
    /// The presentation's title, shown above its views (D-076).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub title: Option<String>,
    /// How long the presentation's run lasts, `until t0 + 20 s` (PK-10.1a, D-091); hosts
    /// choose otherwise.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub until: Option<Expr>,
}

/// A page layout (PK-7.4a, D-063): views side by side in a row or one below the other in a
/// column, nested. Views it does not place follow it, in the order they are declared.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Layout {
    View(Id),
    Row(Vec<Layout>),
    Column(Vec<Layout>),
}

impl Layout {
    /// The views placed, in reading order.
    pub fn views(&self) -> Vec<&Id> {
        match self {
            Layout::View(v) => vec![v],
            Layout::Row(xs) | Layout::Column(xs) => xs.iter().flat_map(|x| x.views()).collect(),
        }
    }
}

// ---------------------------------------------------------------- observation (PK section 3)

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Observation {
    pub id: Id,
    pub name: String,
    pub source: Source,
    pub schedule: Schedule,
    /// The instrument that reads the source (PK-3.7, D-087).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub instrument: Option<Id>,
}

/// An instrument (PK-3.7, D-087): how its readings differ from the values it reads. Each
/// option has the dimension of the values; an instrument without options reads exactly.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Instrument {
    pub id: Id,
    pub name: String,
    /// Readings are rounded to a whole number of this step.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub resolution: Option<Expr>,
    /// Values outside `[lo, hi]` give no reading.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub range: Option<(Expr, Expr)>,
    /// The deviation of a normal error added to each reading.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub noise: Option<Expr>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Source {
    /// An expression over the model's bindings.
    Expr { expr: Expr },
    /// Event occurrences (RC-15.1), optionally of one event, optionally only those where a
    /// Zeno policy replaced the handler.
    EventLog {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        event: Option<Id>,
        #[serde(default, skip_serializing_if = "std::ops::Not::not")]
        zeno_only: bool,
    },
    /// Run diagnostics (RC section 10), optionally about one element.
    Diagnostics {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        element: Option<Id>,
    },
    /// Committed interventions (RC-11.5).
    InterventionLog,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Schedule {
    Live,
    Every { period: Expr },
    /// `at τ`; with `runs`, `at τ over n runs`: one value from each of `n` runs, seeded from
    /// the run's seed on (PK-3.8, D-084).
    At {
        time: Expr,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        runs: Option<Expr>,
    },
    On {
        event: Id,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        microstep: Option<u32>,
    },
    /// `over [from, to]`; `to` absent is the end of the run (`t_end`).
    Over {
        from: Expr,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        to: Option<Expr>,
        lo_closed: bool,
        hi_closed: bool,
    },
    /// No schedule written: the whole run.
    Run,
}

// ---------------------------------------------------------------- views and representations

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct View {
    pub id: Id,
    pub name: String,
    pub kind: ViewKind,
    #[serde(default)]
    pub representations: Vec<Rep>,
    /// `on click as p request E(p)`: a click on a point of the view where no representation
    /// takes it requests an event; the payload reads the point as `{"param": 0}` (D-060).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub click: Option<Click>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum ViewKind {
    /// A spatial view of one space (PK-7.2): `scale` maps a model length to view pixels. A
    /// view of a space of three dimensions looks from `look`, its azimuth and elevation
    /// (PK-7.2a, D-100), with `z` up; `y_up` applies to a plane. Both angles may read the
    /// model's values and time, evaluated at each frame (D-104). `distance`: the viewer's
    /// distance from the origin, a length, for a view in perspective (D-104); absent, the
    /// view is orthographic.
    Spatial {
        space: Id,
        scale: Scale,
        y_up: bool,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        look: Option<[Expr; 2]>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        distance: Option<Expr>,
    },
    /// A plot view (PK-7.3): the axis ranges, the axes whose range follows the data (`x`,
    /// `y`; D-070) and each axis's display unit.
    Plot {
        x: [Expr; 2],
        y: [Expr; 2],
        #[serde(default, skip_serializing_if = "no_follow")]
        follow: [bool; 2],
        #[serde(default, skip_serializing_if = "no_units")]
        units: [Option<String>; 2],
        /// `window: 10 s`: the `x` axis shows the latest span of this length once the data
        /// passes its declared range (D-070).
        #[serde(default, skip_serializing_if = "Option::is_none")]
        window: Option<Expr>,
    },
    /// A region without a coordinate system (controls, formulas).
    Panel,
}

fn no_follow(f: &[bool; 2]) -> bool {
    !f[0] && !f[1]
}

fn no_units(u: &[Option<String>; 2]) -> bool {
    u[0].is_none() && u[1].is_none()
}

/// A unit-aware encoding scale (PK-5.5): `quantity` maps to `px` view units.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Scale {
    pub quantity: Expr,
    pub px: f64,
}

/// A representation (PK-6.1).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Rep {
    /// `for b in row { ... }`: one representation per member, named `name[k]` (D-055).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub each: Option<crate::Each>,
    pub id: Id,
    /// Author name for timeline actions and interactions; not the identity.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    pub kind: String,
    #[serde(default)]
    pub sources: Vec<Arg>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub props: Vec<Prop>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub inverse: Option<Inverse>,
    /// The members of a `group`, drawn with its transform (PK-6.3b, D-043).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub members: Vec<Rep>,
    /// `on click request E(v)`: clicking or activating the representation requests an event
    /// (D-059).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub click: Option<Click>,
    /// Drawn only while this holds: a representation of a member of a collection whose
    /// membership changes is shown while the member is alive (D-057). Made by elaboration.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub when: Option<Expr>,
}

impl Rep {
    pub fn prop(&self, name: &str) -> Option<&Arg> {
        self.props.iter().find(|p| p.name == name).map(|p| &p.value)
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Prop {
    pub name: String,
    pub value: Arg,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Arg {
    Expr { expr: Expr },
    Text { text: String },
    Range { lo: Expr, hi: Expr },
    Scale { scale: Scale },
    Word { word: String },
    /// A source sampled over the run: `pos every 0.02 s` (PK-6.3, `trace`, `series_plot`);
    /// read through an instrument, `ruler(x) every 0.1 s` (PK-3.7, D-087).
    Sampled {
        expr: Expr,
        every: Expr,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        instrument: Option<Id>,
    },
    /// `n at t0 + 7 s over 200 runs`: the value of `expr` at instant `at` in each of `runs`
    /// runs, with the seeds from the run's own on (PK-3.8, D-084).
    Ensemble {
        expr: Expr,
        at: Expr,
        runs: Expr,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        instrument: Option<Id>,
    },
    /// `gap on decay`: the value of `expr` at each occurrence of an event, up to the instant
    /// shown (PK-6.3e, D-089).
    On {
        expr: Expr,
        event: Id,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        instrument: Option<Id>,
    },
    /// A model element that is not a value: an event (`button`) or an equation (`equation`).
    Element { element: Id },
    /// Several texts: the words at a slider's two ends, `ends: ("cold", "hot")` (D-077).
    Texts { texts: Vec<String> },
}

/// A declared inverse (PK-5.6): the gesture's value, `{"param": 0}`, gives proposals.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Inverse {
    pub gesture: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub part: Option<String>,
    pub proposals: Vec<Proposal>,
    /// Drag mode `live` (PK-10.9, D-071): each valid proposal is committed while the run
    /// advances, instead of once on release with the run held.
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub live: bool,
    /// `on drag across n as p`: in a view of three dimensions the pointer moves the point in
    /// the plane of normal `n` through where the drag began, instead of the plane facing the
    /// viewer (D-104).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub across: Option<Expr>,
}

/// The event a click on a representation requests, with its payload: in a representation
/// repeated per member, typically that member (`on click request remove(b)`, D-059).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Click {
    pub event: Id,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub payload: Option<Expr>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Proposal {
    pub target: Id,
    pub value: Expr,
    /// The member whose binding `target` is, in a representation repeated per member:
    /// `propose b.pos = p` (`{"var": "b"}`, D-059). Elaboration resolves it.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub member: Option<Expr>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Permission {
    pub role: String,
    pub allows: Vec<String>,
}

// ---------------------------------------------------------------- timeline (PK section 9)

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Timeline {
    pub scenes: Vec<Scene>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Scene {
    pub id: Id,
    pub name: String,
    pub beats: Vec<Beat>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Beat {
    pub id: Id,
    pub name: String,
    pub actions: Vec<Action>,
}

/// Timeline actions (PK-9.2). `run rate r until E` is written as `run` then `wait_until`.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "action", rename_all = "snake_case")]
pub enum Action {
    /// Show representations, in a view or (without one) over the presentation.
    Show {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        view: Option<Id>,
        reps: Vec<Rep>,
    },
    Highlight { target: Id },
    /// Stops showing a representation from now on (PK-9.2), fading out over `duration`.
    Hide {
        target: Id,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        duration: Option<Expr>,
        /// The easing function; absent is `smooth` (D-095).
        #[serde(default, skip_serializing_if = "Option::is_none")]
        ease: Option<Ease>,
    },
    /// Shows representations with an animation (PK-9.2, PK-8.4, D-042).
    Reveal {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        view: Option<Id>,
        style: RevealStyle,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        duration: Option<Expr>,
        /// The easing function; absent is `smooth` (D-095).
        #[serde(default, skip_serializing_if = "Option::is_none")]
        ease: Option<Ease>,
        reps: Vec<Rep>,
    },
    /// Moves a view's camera (PK-9.2, D-042): its centre follows `center`, evaluated at every
    /// frame, and its zoom; the move takes `duration`. In a view of three dimensions the
    /// camera also turns to `azimuth` and `elevation` (D-104).
    Camera {
        view: Id,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        center: Option<Expr>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        zoom: Option<Expr>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        azimuth: Option<Expr>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        elevation: Option<Expr>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        duration: Option<Expr>,
        /// The easing function; absent is `smooth` (D-095).
        #[serde(default, skip_serializing_if = "Option::is_none")]
        ease: Option<Ease>,
    },
    Narrate {
        text: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        duration: Option<Expr>,
    },
    Run { rate: Expr },
    Hold,
    Seek { time: Expr },
    Reset,
    /// The run again from `t0`, with the parameter values of the instant shown (D-085).
    Restart,
    Branch,
    Intervene { ops: Vec<Op> },
    Request {
        event: Id,
        /// The payload the request supplies (D-050).
        #[serde(default, skip_serializing_if = "Option::is_none")]
        payload: Option<Expr>,
    },
    Wait { duration: Expr },
    WaitUntil { event: Id },
    Explore {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        limit: Option<Expr>,
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        keep: Vec<Id>,
        controls: Vec<Rep>,
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        fallback: Vec<Action>,
    },
    Sequence { actions: Vec<Action> },
    /// A continue point (PK-9.2, D-067): the timeline waits for the learner to continue, at
    /// most `limit`; linear media play `fallback` instead (PK-9.10).
    WaitLearner {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        limit: Option<Expr>,
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        fallback: Vec<Action>,
    },
    /// An animation of a presentation property of a representation (PK-8.4, D-068): from
    /// the value it has to `to`, over `duration`.
    Animate {
        target: Id,
        property: Animated,
        /// The value of `opacity`, `offset` and `scale`.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        to: Option<Expr>,
        /// The value of `color` and `line` (a palette word), or the representation `morph`
        /// takes the shape of (D-075).
        #[serde(default, skip_serializing_if = "Option::is_none")]
        word: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        duration: Option<Expr>,
        /// The easing function; absent is `smooth` (D-075).
        #[serde(default, skip_serializing_if = "Option::is_none")]
        ease: Option<Ease>,
    },
    /// Holds a representation's bound geometry at the instant shown (PK-8.5, D-068).
    Release { target: Id },
    /// Hands a released representation back to its projection, over `duration` (D-068).
    Bind {
        target: Id,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        duration: Option<Expr>,
        /// The easing function; absent is `smooth` (D-095).
        #[serde(default, skip_serializing_if = "Option::is_none")]
        ease: Option<Ease>,
    },
}

/// Presentation properties `animate` drives (D-068).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Animated {
    /// From 0 (invisible) to 1, multiplying any reveal or hide.
    Opacity,
    /// A displacement of the representation in its view, a vector of the view's space.
    Offset,
    /// A color of the palette, blended from the color the representation has (D-075).
    Color,
    /// A line style, switched halfway through the animation (D-075).
    Line,
    /// A factor on the drawn size, about the shape's center (D-075).
    Scale,
    /// A multiple of the kind's own line width (D-095).
    Width,
    /// The shape of another representation, reached by moving the points (D-075).
    Morph,
}

/// How an animation moves from its start to its end over its duration (PK-8.4, D-075).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Ease {
    /// At a constant rate.
    Linear,
    /// Starting and ending slowly: `3k^2 - 2k^3` (D-042's).
    Smooth,
    /// Starting slowly: `k^2`.
    In,
    /// Ending slowly: `1 - (1 - k)^2`.
    Out,
}

impl Ease {
    pub fn apply(self, k: f64) -> f64 {
        let k = k.clamp(0.0, 1.0);
        match self {
            Ease::Linear => k,
            Ease::Smooth => k * k * (3.0 - 2.0 * k),
            Ease::In => k * k,
            Ease::Out => 1.0 - (1.0 - k) * (1.0 - k),
        }
    }

    pub fn word(self) -> &'static str {
        match self {
            Ease::Linear => "linear",
            Ease::Smooth => "smooth",
            Ease::In => "in",
            Ease::Out => "out",
        }
    }
}

/// How `reveal` shows a representation (D-042).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RevealStyle {
    /// Opacity from 0 to 1.
    Fade,
    /// Lines and paths drawn from their start to their end; points fade.
    Draw,
}

impl Action {
    /// Run-directing actions apply in written order at a beat's start (PK-9.2a, D-033).
    pub fn is_run_directing(&self) -> bool {
        matches!(self, Action::Seek { .. } | Action::Reset | Action::Restart | Action::Branch | Action::Intervene { .. } | Action::Request { .. } | Action::Run { .. } | Action::Hold)
    }
}

// ---------------------------------------------------------------- runs (RC section 3, PK section 4)

/// A named run configuration with expectations: a reference-program case.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct RunCase {
    pub id: Id,
    pub name: String,
    pub model: Id,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub presentation: Option<Id>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub params: Vec<Override>,
    /// Values the environment supplies to `input` bindings (D-051): at the start, or at an
    /// instant.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub inputs: Vec<InputValue>,
    #[serde(default)]
    pub config: RunConfig,
    /// `until τ`: the end of the run.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub end: Option<Expr>,
    /// A learner script: test-harness input fed through the interaction path.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub learner: Vec<LearnerStep>,
    #[serde(default)]
    pub expectations: Vec<Expectation>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Override {
    pub binding: Id,
    pub value: Expr,
}

/// An input's value from the environment (D-051): from the start, or from instant `at`.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct InputValue {
    pub binding: Id,
    pub value: Expr,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub at: Option<Expr>,
}

/// Run configuration (RC section 16); absent entries take their defaults.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct RunConfig {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub solver: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub h: Option<Expr>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rtol: Option<Expr>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub atol: Option<Expr>,
    /// The seed of the run's random streams (RC-13.1, D-083); 0 when absent.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub seed: Option<u64>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct LearnerStep {
    /// Presentation time.
    pub at: Expr,
    pub input: LearnerInput,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum LearnerInput {
    Continue,
    /// `set slider angle = 60 deg`: set the control of that kind targeting the binding.
    SetControl { control: String, binding: Id, value: Expr },
    /// `press drop`: press the button that requests the event (D-069).
    Press { event: Id },
    /// `press restart`, `press reseed`: press an explore beat's runtime-control button (D-088).
    Control { control: String },
}

/// An expectation (PK section 4).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Expectation {
    pub id: Id,
    #[serde(flatten)]
    pub check: Check,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "check", rename_all = "snake_case")]
pub enum Check {
    Equal { subject: Subject, expected: Operand, tolerance: Tolerance },
    Within { subject: Subject, lo: Expr, hi: Expr, lo_closed: bool, hi_closed: bool },
    Outcome { outcome: Outcome },
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Subject {
    /// An observation's value; with `index`, the n-th value (from 1) of a series.
    Observation {
        observation: Id,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        index: Option<usize>,
    },
    /// `(e on E)`: the value at the first occurrence of `E`.
    On {
        expr: Expr,
        event: Id,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        microstep: Option<u32>,
    },
    BeatStart { beat: Id },
    BeatEnd { beat: Id },
    /// `mean(ends)`, `deviation(ends)`: a statistic of an observation's values (D-084).
    Statistic { statistic: Statistic, observation: Id },
}

/// Statistics of the values of an observation (D-084): the mean, and the sample standard
/// deviation (divided by `n - 1`).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Statistic {
    Mean,
    Deviation,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Operand {
    Value { expr: Expr },
    Subject { subject: Subject },
    List { items: Vec<Item> },
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Item {
    Event { event: Id },
    Value { expr: Expr },
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum Tolerance {
    Exact,
    Abs { tol: Expr },
    Rel { tol: Expr },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Outcome {
    InitializationFails,
    ConfigurationRejected,
}
