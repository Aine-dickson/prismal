//! Projection and frame descriptions (PK sections 5 to 7, 11 and 12).
//!
//! A frame description lists, for one presented instant, the visible representations with
//! their resolved properties in view coordinates, their text alternatives, and the
//! captions and announcements due (PK-12.1). It is medium-independent and serializable.
//!
//! View coordinates: in a spatial view, pixels with the model origin at `(0, 0)` and `y`
//! pointing down on screen (a view with `y: up` negates model `y`); in a plot view, the
//! plot's own coordinates; in a panel, none.

use crate::math::{self, MathLayout};
use crate::measure::{self, Meter};
use crate::text::{fmt_binding, fmt_num, fmt_value, print, symbol, unit_text};
use crate::{constant, number, PDiag};
use prismal_ir::build::{num, origin, tuple};
use prismal_ir::present::{Arg, Presentation, Rep, View, ViewKind};
use prismal_ir::{Dim, Expr, Id, Lambda, Role, Type};
use prismal_kernel::{compile_expr, CExpr, CModel, Ctx, Value};
use prismal_runtime::Run;
use serde::Serialize;

// ---------------------------------------------------------------- frame description

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct Frame {
    /// Presentation time (PK-8.1).
    pub time: f64,
    /// The run shown (`lesson`, `branch 1`, `session`).
    pub run: String,
    /// The simulation instant shown by every view (PK-7.5).
    pub t: f64,
    pub views: Vec<ViewFrame>,
    /// Representations shown over the presentation, outside any view.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub overlay: Vec<RepFrame>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub captions: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub announcements: Vec<String>,
}

impl Frame {
    /// Every representation in the frame, views first, a group before its members.
    pub fn reps(&self) -> impl Iterator<Item = &RepFrame> {
        fn walk<'a>(r: &'a RepFrame, out: &mut Vec<&'a RepFrame>) {
            out.push(r);
            if let Shape::Group { members } = &r.shape {
                members.iter().for_each(|m| walk(m, out));
            }
        }
        let mut out = vec![];
        self.views.iter().flat_map(|v| v.reps.iter()).chain(self.overlay.iter()).for_each(|r| walk(r, &mut out));
        out.into_iter()
    }
    pub fn rep(&self, id_or_name: &str) -> Option<&RepFrame> {
        self.reps().find(|r| r.id == id_or_name || r.name.as_deref() == Some(id_or_name) || r.id.ends_with(&format!(".{id_or_name}")))
    }
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct ViewFrame {
    pub id: Id,
    pub kind: &'static str,
    pub reps: Vec<RepFrame>,
    /// For a view that requests an event when a point of it is clicked: the event's name
    /// (D-060).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub click: Option<String>,
    /// A camera set by the timeline (D-042).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub camera: Option<Camera>,
}

/// A view's camera (D-042): the centre in view coordinates, the zoom, and `blend`, how far
/// the renderer moves from its own framing to this centre (1 once a move has ended).
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct Camera {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub center: Option<[f64; 2]>,
    pub zoom: f64,
    pub blend: f64,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct RepFrame {
    pub id: Id,
    pub kind: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(flatten)]
    pub shape: Shape,
    /// Text alternative (PK-11.1).
    pub text: String,
    #[serde(skip_serializing_if = "std::ops::Not::not")]
    pub highlighted: bool,
    /// During a drag preview: whether the proposal passed validation (PK-10.6).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub valid: Option<bool>,
    /// During a reveal or a hide: opacity from 0 to 1 (PK-8.4, D-042).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub opacity: Option<f64>,
    /// During a `reveal draw`: the fraction of a line or path drawn so far.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub drawn: Option<f64>,
    /// The label drawn with a marker, an arrow or a graph: its author name or its source.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    /// For a representation with a declared inverse (PK-5.6): the part it is dragged by,
    /// `body` or the part's name (`head`).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub drag: Option<String>,
    /// For a representation that requests an event when clicked or activated: the event's
    /// name (D-059).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub click: Option<String>,
    /// The author's color, a name from the palette every medium maps to its own values
    /// (PK-6.6a, D-061).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub color: Option<String>,
    /// The author's line style: `dashed` or `dotted`; absent is the kind's own (D-061).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub line: Option<String>,
    /// During a color animation (D-075): the color being blended to, and how far, from 0
    /// (`color`) to 1 (`color_to`). Media mix the two colors they map the names to.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub color_to: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub color_mix: Option<f64>,
    /// An animated size factor (D-075): geometry is already scaled; media draw a marker's
    /// dot and an arrow's head this much larger.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub scale: Option<f64>,
    /// For a path sampled over the run: the indices of points that start a new piece,
    /// where the value jumped at an instant (a ball thrown again). Media lift the pen there.
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub breaks: Vec<usize>,
}

/// Named colors an author may give a representation (D-061). Media map them to values that
/// suit their theme; the names are the frame's.
pub const COLORS: &[&str] = &["red", "orange", "yellow", "green", "teal", "blue", "purple", "pink", "gray", "ink"];
/// Line styles (D-061).
pub const LINES: &[&str] = &["solid", "dashed", "dotted"];

/// Kinds that take `color` and `line` (D-061): what is drawn with ink or strokes.
fn styled(kind: &str) -> (bool, bool) {
    match kind {
        "marker" => (true, false),
        "arrow" | "segment" | "polyline" | "polygon" | "circle" | "ellipse" | "arc" | "trace" | "function_graph" | "series_plot" | "histogram" => (true, true),
        _ => (false, false),
    }
}

/// Checks and removes the style properties of a representation (D-061), so that each kind's
/// own checks see only its own properties.
fn style_props(rep: &Rep) -> Result<Rep, PDiag> {
    let d = |message: String| PDiag { code: "PK-E05", message, element: rep.id.clone() };
    let (color, line) = styled(&rep.kind);
    for p in rep.props.iter().filter(|p| p.name == "color" || p.name == "line") {
        let (ok, words) = if p.name == "color" { (color, COLORS) } else { (line, LINES) };
        if !ok {
            return Err(d(format!("a {} takes no `{}` (PK-6.6a)", rep.kind, p.name)));
        }
        match &p.value {
            Arg::Word { word } if words.contains(&word.as_str()) => {}
            _ => return Err(d(format!("`{}` is one of {} (PK-6.6a)", p.name, words.join(", ")))),
        }
    }
    Ok(Rep { props: rep.props.iter().filter(|p| p.name != "color" && p.name != "line").cloned().collect(), ..rep.clone() })
}

/// The author's style word (D-061). `line: solid` is carried too: a trace is dashed unless
/// its author says otherwise (D-090, a Brownian path).
fn style_word(rep: &Rep, name: &str) -> Option<String> {
    match rep.prop(name) {
        Some(Arg::Word { word }) => Some(word.clone()),
        _ => None,
    }
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(tag = "shape", rename_all = "snake_case")]
pub enum Shape {
    Point { at: [f64; 2] },
    Arrow { from: [f64; 2], to: [f64; 2] },
    Segment { from: [f64; 2], to: [f64; 2] },
    Polyline { points: Vec<[f64; 2]> },
    Axes,
    Grid,
    Text { value: String },
    /// Typeset from the IR (PK-6.5, D-034): `rhs` is the symbolic form; each symbol carries
    /// its binding's identity and, when live, its current value.
    /// `layout` places it for any medium (D-046).
    Formula { lhs: String, rhs: Expr, symbols: Vec<Symbol>, layout: MathLayout },
    /// `value`, `min`, `max` and `step` are in coherent SI units; `symbol` is the binding's
    /// display symbol, `display_unit` its display unit (`value / scale` in `text`), else
    /// `unit` names the coherent SI unit.
    Control {
        control: String,
        binding: Id,
        value: f64,
        #[serde(skip_serializing_if = "Option::is_none")]
        min: Option<f64>,
        #[serde(skip_serializing_if = "Option::is_none")]
        max: Option<f64>,
        #[serde(skip_serializing_if = "Option::is_none")]
        step: Option<f64>,
        symbol: String,
        #[serde(skip_serializing_if = "Option::is_none")]
        display_unit: Option<DisplayUnit>,
        #[serde(skip_serializing_if = "Option::is_none")]
        unit: Option<String>,
        /// Words at a slider's low and high ends (D-077).
        #[serde(skip_serializing_if = "Option::is_none")]
        ends: Option<[String; 2]>,
        /// A select's choices, the cases of its enumeration in order; `value` is the number
        /// of the chosen one, from 0 (D-094).
        #[serde(skip_serializing_if = "Option::is_none")]
        options: Option<Vec<String>>,
    },
    /// The source has no value at this instant (PK-3.3).
    Status { status: String },
    /// A closed path through points.
    Polygon { points: Vec<[f64; 2]> },
    /// Cells coloured by value (D-091): `cols` by `rows` cells filling the rectangle with
    /// opposite corners `from` (the grid's first cell's corner) and `to`, in view
    /// coordinates; `values` row by row from `from`, each from 0 (`lo`) to 1 (`hi`), on the
    /// named palette (`sequential`, or `diverging` about 0), which each medium maps to its
    /// colours.
    Raster { from: [f64; 2], to: [f64; 2], cols: usize, rows: usize, values: Vec<f64>, lo: f64, hi: f64, palette: String },
    /// An elliptical arc in view coordinates: the points `center + radii[0] cos(a) u +
    /// radii[1] sin(a) v` for `a` from `start` to `start + sweep`, where `u` is the unit
    /// vector at `rotation` and `v` is `u` turned by +90 degrees (from +x towards +y of the
    /// view). Angles are in radians. `closed` for a whole circle or ellipse. Circles, ellipses
    /// and arcs of the model's space become this shape, with the view's orientation and any
    /// group's transform applied (PK-6.3c).
    Ellipse { center: [f64; 2], radii: [f64; 2], rotation: f64, start: f64, sweep: f64, closed: bool },
    /// A model equation typeset from the IR (PK-6.5); `name` is the equation's name.
    Equation { name: String, lhs: Expr, rhs: Expr, symbols: Vec<Symbol>, layout: MathLayout },
    /// A button that requests an event (D-027), or that is a runtime control: `reset`,
    /// `undo` or `redo` (D-069).
    Button {
        #[serde(skip_serializing_if = "Option::is_none")]
        event: Option<Id>,
        #[serde(skip_serializing_if = "Option::is_none")]
        control: Option<String>,
        label: String,
    },
    /// Rows of values: the sample instant, then one column per component.
    Table { columns: Vec<String>, rows: Vec<Vec<String>> },
    /// A group's members, already placed by its transform (D-043).
    Group { members: Vec<RepFrame> },
    /// An author's text placed at a point of a spatial or plot view (D-076).
    Note { at: [f64; 2], value: String },
    /// A title: the caption of the view or panel it is in (D-076).
    Title { value: String },
}

impl Shape {
    /// `n + 1` points along an `Ellipse` shape from its start to its end, in view
    /// coordinates; empty for other shapes.
    pub fn curve_points(&self, n: usize) -> Vec<[f64; 2]> {
        let Shape::Ellipse { center, radii, rotation, start, sweep, .. } = self else { return vec![] };
        let (s, c) = rotation.sin_cos();
        (0..=n)
            .map(|k| {
                let a = start + sweep * k as f64 / n as f64;
                let (x, y) = (radii[0] * a.cos(), radii[1] * a.sin());
                [center[0] + c * x - s * y, center[1] + s * x + c * y]
            })
            .collect()
    }
}

/// Applies `f` to every representation in `reps`, a group before its members.
pub fn each_rep_mut(reps: &mut [RepFrame], f: &mut impl FnMut(&mut RepFrame)) {
    for r in reps {
        f(r);
        if let Shape::Group { members } = &mut r.shape {
            each_rep_mut(members, f);
        }
    }
}

/// Removes the representations for which `keep` is false, members of groups included.
pub fn retain_reps(reps: &mut Vec<RepFrame>, keep: &impl Fn(&RepFrame) -> bool) {
    reps.retain(|r| keep(r));
    for r in reps {
        if let Shape::Group { members } = &mut r.shape {
            retain_reps(members, keep);
        }
    }
}

/// A display unit: a value in coherent SI units shows as `value / scale` followed by `text`.
#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct DisplayUnit {
    pub text: String,
    pub scale: f64,
}

#[derive(Clone, Debug, PartialEq, Serialize)]
pub struct Symbol {
    pub binding: Id,
    pub symbol: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<String>,
}

// ---------------------------------------------------------------- views and representations

/// The coordinate context of a view (PK-7.2, PK-7.3).
#[derive(Clone, Debug)]
pub enum ViewCtx {
    Spatial { space: Id, px_per_m: f64, y_up: bool },
    /// Ranges in coherent SI units, and the dimension of each axis (PK-7.3); the axes that
    /// follow the data and each axis's display unit (D-070).
    /// `sample_x`: the `x` range a host shows when the learner has zoomed or panned, over
    /// which function graphs are drawn instead of the declared range.
    /// `window`: the length of the latest span the `x` axis shows (D-070).
    Plot { x: (f64, f64), y: (f64, f64), dims: (Dim, Dim), follow: [bool; 2], units: [Option<prismal_ir::Unit>; 2], window: Option<f64>, sample_x: Option<(f64, f64)> },
    Panel,
}

impl ViewCtx {
    pub fn of(cm: &CModel, v: &View) -> Result<ViewCtx, PDiag> {
        let err = |code, message: String| PDiag { code, message, element: v.id.clone() };
        match &v.kind {
            ViewKind::Spatial { space, scale, y_up } => {
                if !cm.spaces.iter().any(|s| &s.id == space) {
                    return Err(err("PK-E01", format!("unknown space `{space}` (PK-2.3)")));
                }
                let q = match compile_expr(cm, &scale.quantity, None) {
                    Ok((_, Type::Quantity { dim })) if dim == Dim::length() => number(cm, &scale.quantity).map_err(|m| err("PK-E02", m))?,
                    _ => return Err(err("PK-E04", "a spatial view's scale maps a length to pixels: `1 m -> 40 px` (PK-5.5)".into())),
                };
                Ok(ViewCtx::Spatial { space: space.clone(), px_per_m: scale.px / q, y_up: *y_up })
            }
            ViewKind::Plot { x, y, follow, units, window } => {
                let n = |e: &Expr| number(cm, e).map_err(|m| err("PK-E02", m));
                let dim = |e: &Expr| match compile_expr(cm, e, None) {
                    Ok((_, Type::Quantity { dim })) => Ok(dim),
                    _ => Err(err("PK-E04", "a plot axis range is a pair of numbers or quantities: `x: [0 s, 5 s]` (PK-7.3)".into())),
                };
                let (dx, dy) = (dim(&x[0])?, dim(&y[0])?);
                if dim(&x[1])? != dx || dim(&y[1])? != dy {
                    return Err(err("PK-E04", "both ends of a plot axis range have the same dimension (PK-7.3)".into()));
                }
                let unit = |u: &Option<String>, d: &Dim| -> Result<Option<prismal_ir::Unit>, PDiag> {
                    let Some(u) = u else { return Ok(None) };
                    match prismal_ir::Unit::parse(u) {
                        Ok(unit) if unit.dim == *d && unit.offset == 0.0 => Ok(Some(unit)),
                        Ok(_) => Err(err("PK-E04", format!("the display unit `{u}` does not measure the axis's dimension (D-070)"))),
                        Err(m) => Err(err("PK-E02", m)),
                    }
                };
                let units = [unit(&units[0], &dx)?, unit(&units[1], &dy)?];
                let window = match window {
                    Some(w) if dim(w)? != dx => return Err(err("PK-E04", "a plot's window is a span of its `x` axis: `window: 10 s` on a time axis (D-070)".into())),
                    Some(w) => match n(w)? {
                        v if v > 0.0 => Some(v),
                        _ => return Err(err("PK-E05", "a plot's window is positive (D-070)".into())),
                    },
                    None => None,
                };
                Ok(ViewCtx::Plot { x: (n(&x[0])?, n(&x[1])?), y: (n(&y[0])?, n(&y[1])?), dims: (dx, dy), follow: *follow, units, window, sample_x: None })
            }
            ViewKind::Panel => Ok(ViewCtx::Panel),
        }
    }

    pub fn kind(&self) -> &'static str {
        match self {
            ViewCtx::Spatial { .. } => "spatial",
            ViewCtx::Plot { .. } => "plot",
            ViewCtx::Panel => "panel",
        }
    }

    /// Model coordinates to view coordinates.
    pub fn to_view(&self, p: &[f64]) -> [f64; 2] {
        let (x, y) = (p.first().copied().unwrap_or(0.0), p.get(1).copied().unwrap_or(0.0));
        match self {
            ViewCtx::Spatial { px_per_m, y_up, .. } => [x * px_per_m, if *y_up { -y * px_per_m } else { y * px_per_m }],
            _ => [x, y],
        }
    }

    /// The coordinates of a position as text shows them. In a spatial view a coordinate that
    /// lies within a millionth of a pixel of 0 is rounding left by the solver (a ball resting
    /// on the ground at `y = -3.7e-10 m`) and is shown as 0; the view's scale decides, so a
    /// model of atoms drawn at a fitting scale keeps its small lengths. In a plot view the same
    /// holds for a billionth of the axis span. Elsewhere `components`.
    pub fn shown(&self, c: &[f64]) -> Vec<f64> {
        match self {
            ViewCtx::Spatial { px_per_m, .. } => c.iter().map(|&x| if (x * px_per_m).abs() < 1e-6 { 0.0 } else { x }).collect(),
            ViewCtx::Plot { x, y, .. } => {
                let spans = [(x.1 - x.0).abs(), (y.1 - y.0).abs()];
                c.iter().enumerate().map(|(i, &v)| if i < 2 && v.abs() < 1e-9 * spans[i] { 0.0 } else { v }).collect()
            }
            ViewCtx::Panel => crate::text::components(c),
        }
    }

    /// A pointer position in view coordinates as a point in the frame of a group (D-043):
    /// the value of a drag on a member of the group.
    pub fn pointer_in(&self, p: [f64; 2], tf: &Tf) -> Expr {
        match self {
            ViewCtx::Spatial { space, px_per_m, y_up } => {
                let m = [p[0] / px_per_m, if *y_up { -p[1] } else { p[1] } / px_per_m];
                let l = tf.unapply(m);
                origin(space) + tuple(vec![num(l[0], "m"), num(l[1], "m")])
            }
            _ => self.pointer(p),
        }
    }

    /// A pointer position in view coordinates as a model expression: a point of the view's
    /// space, or the pair of plot coordinates. This is the value of a gesture (PK-5.6).
    pub fn pointer(&self, p: [f64; 2]) -> Expr {
        match self {
            ViewCtx::Spatial { space, px_per_m, y_up } => {
                let x = p[0] / px_per_m;
                let y = if *y_up { -p[1] } else { p[1] } / px_per_m;
                origin(space) + tuple(vec![num(x, "m"), num(y, "m")])
            }
            ViewCtx::Plot { dims, .. } => tuple(vec![si_num(p[0], &dims.0), si_num(p[1], &dims.1)]),
            ViewCtx::Panel => tuple(vec![prismal_ir::build::lit(p[0]), prismal_ir::build::lit(p[1])]),
        }
    }
}

/// A number in coherent SI units of a dimension, as a literal (bare when dimensionless).
fn si_num(x: f64, dim: &Dim) -> Expr {
    if dim.is_none() {
        prismal_ir::build::lit(x)
    } else {
        Expr::Num { num: x, unit: Some(prismal_ir::Unit { text: crate::text::unit_text(dim), scale: 1.0, offset: 0.0, dim: *dim }) }
    }
}

/// A representation compiled for its view.
#[derive(Clone, Debug)]
pub struct CRep {
    pub rep: Rep,
    pub kind: CKind,
    /// Drawn only while this holds: a member of a collection that changes, alive (D-057).
    pub when: Option<CExpr>,
}

impl CRep {
    /// Whether the representation is drawn on a state (D-057).
    pub fn shown(&self, run: &Run, vals: &[Value], t: f64) -> bool {
        match &self.when {
            None => true,
            Some(c) => matches!(run.eval_state(c, vals, t), Ok(Value::Bool(true))),
        }
    }
}

#[derive(Clone, Debug)]
pub enum CKind {
    Marker { pos: CExpr, label: String },
    Arrow { vec: CExpr, ty: Type, from: Option<CExpr>, px_per_unit: f64, label: String },
    Segment { a: CExpr, b: CExpr },
    Graph { f: CExpr, lo: f64, hi: f64, label: String },
    /// A field drawn over its grid (D-091): an operation on the field, through which its
    /// edge values and cell values are read.
    FieldGraph { field: CExpr, label: String },
    /// A field in the plane drawn as coloured cells (D-091), over `range` or its own extent.
    Heatmap { field: CExpr, range: Option<(f64, f64)>, label: String },
    /// The path of a point, sampled every `every` seconds from the run's start (PK-6.3).
    Trace { pos: CExpr, every: f64, label: String, meter: Option<Meter> },
    /// A scalar against elapsed time, sampled every `every` seconds (PK-6.3). `steps`: the
    /// value changes only at transitions (it reads no continuous state and no time), so it
    /// is drawn as a staircase (D-084).
    Series { value: CExpr, every: f64, label: String, steps: bool, meter: Option<Meter> },
    /// `idx`: the binding shown, whose display unit the text uses (MK-3.12).
    Label { value: CExpr, ty: Type, label: String, idx: Option<usize> },
    /// An author's text: the pieces between values, the values with their types and, for a
    /// binding, its index (shown in its display unit), and where it is placed (D-076).
    /// `places`: the decimals each value is shown with, when the author chose (D-078).
    Text { pieces: Vec<String>, values: Vec<(CExpr, Type, Option<usize>)>, places: Vec<Option<usize>>, at: Option<CExpr> },
    Title { value: String },
    Formula { lhs: String, rhs: Expr, params: Vec<String>, refs: Vec<(Id, usize)>, live: bool },
    Control { control: String, binding: Id, idx: usize, min: Option<f64>, max: Option<f64>, step: Option<f64>, ends: Option<[String; 2]>, options: Option<Vec<String>> },
    Poly { points: Vec<CExpr>, closed: bool },
    /// `circle`, `ellipse` and `arc` (PK-6.3c): radii are lengths, angles numbers (radians);
    /// `ry` absent for a circle or an arc, `from` and `to` present for an arc only.
    Round { center: CExpr, rx: CExpr, ry: Option<CExpr>, rotate: Option<CExpr>, from: Option<CExpr>, to: Option<CExpr> },
    Equation { name: String, lhs: Expr, rhs: Expr, refs: Vec<(Id, usize)>, live: bool },
    /// `payload`: the value a press requests the event with (D-072).
    Button { event: Id, label: String, payload: Option<Expr> },
    /// A runtime control (D-069): `reset`, `undo` or `redo`.
    RunButton { control: String, label: String },
    Table { value: CExpr, tys: Vec<Type>, every: f64, columns: Vec<String>, meter: Option<Meter> },
    Axes,
    Grid,
    /// How many values fall in each of `bins` equal bins over the plot's x range: the values
    /// of many runs at one instant (D-084), or of one run up to the instant shown (D-089).
    Histogram { source: HistSource, bins: usize, label: String, meter: Option<Meter> },
    /// Members drawn with a shared transform: placed at `at`, turned by `rotate`, scaled by
    /// `scale` (D-043).
    Group { at: Option<CExpr>, rotate: Option<CExpr>, scale: Option<CExpr>, members: Vec<CRep> },
}

/// Where a histogram's values come from (PK-6.3e).
#[derive(Clone, Debug)]
pub enum HistSource {
    /// `n at τ over k runs` (D-084); the values are kept for the run configuration they came
    /// from.
    Ensemble { expr: Expr, at: f64, runs: usize, cache: EnsembleCache },
    /// `x every Δ`: the samples of the run up to the instant shown (D-089).
    Every { value: CExpr, every: f64 },
    /// `x on E`: the values at each occurrence of `E` up to the instant shown (D-089).
    On { expr: Expr, event: String },
}

/// The values of an ensemble and the configuration of the run they were computed for: a
/// histogram recomputes its runs only when the learner changes the run (D-084).
pub type EnsembleCache = std::rc::Rc<std::cell::RefCell<Option<(prismal_runtime::Config, Vec<f64>)>>>;

/// A group's transform in model coordinates (D-043): `p -> at + k R(angle) p` for points,
/// `v -> k R(angle) v` for vectors.
#[derive(Clone, Copy, Debug)]
pub struct Tf {
    at: [f64; 2],
    angle: f64,
    k: f64,
}

impl Tf {
    pub const ID: Tf = Tf { at: [0.0, 0.0], angle: 0.0, k: 1.0 };

    fn turn(&self, a: &[f64]) -> [f64; 2] {
        let (x, y) = (a.first().copied().unwrap_or(0.0), a.get(1).copied().unwrap_or(0.0));
        let (s, c) = self.angle.sin_cos();
        [self.k * (c * x - s * y), self.k * (s * x + c * y)]
    }

    /// The transform of the groups enclosing a member, outermost first, at a state: as
    /// projection composes them (D-043).
    pub fn of_groups(chain: &[CRep], run: &Run, vals: &[Value], t: f64) -> Option<Tf> {
        let mut tf = Tf::ID;
        for g in chain {
            let CKind::Group { at, rotate, scale, .. } = &g.kind else { return None };
            let at = match at {
                Some(c) => coords(&tf.apply(run.eval_state(c, vals, t).ok()?)),
                None => tf.at.to_vec(),
            };
            let num = |c: &Option<CExpr>, d: f64| match c.as_ref().map(|c| run.eval_state(c, vals, t)) {
                Some(Ok(Value::Num(x))) => Some(x),
                Some(Ok(_)) | None => Some(d),
                Some(Err(_)) => None,
            };
            tf = Tf { at: [at[0], at[1]], angle: tf.angle + num(rotate, 0.0)?, k: tf.k * num(scale, 1.0)? };
        }
        Some(tf)
    }

    /// Model coordinates in the view's space as coordinates in the group's frame: the
    /// inverse of `apply` on points.
    pub fn unapply(&self, m: [f64; 2]) -> [f64; 2] {
        let (x, y) = ((m[0] - self.at[0]) / self.k, (m[1] - self.at[1]) / self.k);
        let (s, c) = self.angle.sin_cos();
        [c * x + s * y, -s * x + c * y]
    }

    /// A point in the group's frame as model coordinates in the view's space.
    pub fn point(&self, a: [f64; 2]) -> [f64; 2] {
        let r = self.turn(&a);
        [self.at[0] + r[0], self.at[1] + r[1]]
    }

    /// A value in the group's frame as a value in the view's space.
    pub fn apply(&self, v: Value) -> Value {
        if self.at == [0.0, 0.0] && self.angle == 0.0 && self.k == 1.0 {
            return v;
        }
        match v {
            Value::Point(a) => {
                let r = self.turn(a.as_slice());
                Value::Point(prismal_kernel::Arr::from_slice(&[self.at[0] + r[0], self.at[1] + r[1]]))
            }
            Value::Vec(a) => Value::Vec(prismal_kernel::Arr::from_slice(&self.turn(a.as_slice()))),
            other => other,
        }
    }
}

/// Replaces the gesture parameter `{"param": 0}` by `v`.
pub fn subst(e: &Expr, v: &Expr) -> Expr {
    let s = |x: &Expr| Box::new(subst(x, v));
    match e {
        Expr::Param { param: 0 } => v.clone(),
        Expr::Neg { neg } => Expr::Neg { neg: s(neg) },
        Expr::Not { not } => Expr::Not { not: s(not) },
        Expr::Norm { norm } => Expr::Norm { norm: s(norm) },
        Expr::Bin { bin, l, r } => Expr::Bin { bin: *bin, l: s(l), r: s(r) },
        Expr::Call { call, args } => Expr::Call { call: *call, args: args.iter().map(|a| subst(a, v)).collect() },
        Expr::Apply { apply, args } => Expr::Apply { apply: s(apply), args: args.iter().map(|a| subst(a, v)).collect() },
        Expr::If { r#if, then, r#else } => Expr::If { r#if: s(r#if), then: s(then), r#else: s(r#else) },
        Expr::Tuple { tuple } => Expr::Tuple { tuple: tuple.iter().map(|a| subst(a, v)).collect() },
        Expr::Comp { comp, axis } => Expr::Comp { comp: s(comp), axis: *axis },
        Expr::Otherwise { otherwise, default } => Expr::Otherwise { otherwise: s(otherwise), default: s(default) },
        other => other.clone(),
    }
}

fn source(rep: &Rep, i: usize) -> Option<&Expr> {
    match rep.sources.get(i) {
        Some(Arg::Expr { expr }) => Some(expr),
        _ => None,
    }
}

fn prop_expr<'a>(rep: &'a Rep, name: &str) -> Option<&'a Expr> {
    match rep.prop(name) {
        Some(Arg::Expr { expr }) => Some(expr),
        _ => None,
    }
}

/// Compiles a representation for a view, checking its sources, scales and inverse.
pub fn compile_rep(cm: &CModel, ctx: &ViewCtx, full: &Rep, instruments: &[prismal_ir::present::Instrument]) -> Result<CRep, Vec<PDiag>> {
    let stripped = style_props(full).map_err(|d| vec![d])?;
    let rep = &stripped;
    let d = |code: &'static str, message: String| vec![PDiag { code, message, element: rep.id.clone() }];
    let ce = |e: &Expr, exp: Option<&Type>| -> Result<(CExpr, Type), Vec<PDiag>> {
        compile_expr(cm, e, exp).map_err(|ds| d("PK-E02", ds.iter().map(|x| format!("{}: {}", x.code, x.message)).collect::<Vec<_>>().join("; ")))
    };
    let needs = |what: &str| d("PK-E05", format!("`{}` needs {what} (PK-6.3)", rep.kind));
    let label = |e: &Expr| rep.name.clone().unwrap_or_else(|| print(e, cm, &[]));
    let kind = match rep.kind.as_str() {
        "marker" => {
            let e = source(rep, 0).or(prop_expr(rep, "at")).ok_or_else(|| needs("a position"))?;
            let (c, t) = match ctx {
                ViewCtx::Spatial { space, .. } => ce(e, Some(&Type::Point { space: space.clone() }))?,
                ViewCtx::Plot { .. } => ce(e, None)?,
                ViewCtx::Panel => return Err(d("PK-E05", "a marker belongs in a spatial or plot view".into())),
            };
            let ok = match (&t, ctx) {
                (Type::Point { .. }, ViewCtx::Spatial { .. }) => true,
                // A pair of plot coordinates with the dimensions of the plot's axes (PK-7.3).
                (Type::Tuple { items }, ViewCtx::Plot { dims, .. }) => {
                    items.len() == 2 && items[0] == (Type::Quantity { dim: dims.0 }) && items[1] == (Type::Quantity { dim: dims.1 })
                }
                _ => false,
            };
            if !ok {
                return Err(needs("a point of the view's space, or a pair of plot coordinates in the units of the plot's axes"));
            }
            CKind::Marker { pos: c, label: label(e) }
        }
        "arrow" => {
            let ViewCtx::Spatial { space, px_per_m, .. } = ctx else { return Err(d("PK-E05", "an arrow belongs in a spatial view".into())) };
            let e = source(rep, 0).ok_or_else(|| needs("a vector"))?;
            // A tuple is typed as a vector by context (D-032): the dimension of its first
            // dimensioned component.
            let (mut c, mut t) = ce(e, None)?;
            if let Type::Tuple { items } = &t {
                let dim = items.iter().find_map(|i| match i {
                    Type::Quantity { dim } if !dim.is_none() => Some(*dim),
                    _ => None,
                });
                let want = Type::Vector { space: space.clone(), dim: dim.unwrap_or(Dim::NONE) };
                (c, t) = ce(e, Some(&want))?;
            }
            let Type::Vector { space: vs, dim } = &t else { return Err(needs("a vector")) };
            if vs != space {
                return Err(d("PK-E05", format!("the vector is in space `{vs}`, the view shows `{space}` (D-022)")));
            }
            let px_per_unit = match rep.prop("scale") {
                Some(Arg::Scale { scale }) => {
                    let (_, qt) = ce(&scale.quantity, None)?;
                    if qt != (Type::Quantity { dim: *dim }) {
                        return Err(d("PK-E04", format!("the arrow's scale maps {}, the vector has dimension {dim} (PK-5.5)", crate::check::show_type(&qt))));
                    }
                    scale.px / number(cm, &scale.quantity).map_err(|m| d("PK-E02", m))?
                }
                _ if *dim == Dim::length() => *px_per_m,
                _ => {
                    return Err(d(
                        "PK-E04",
                        format!("an arrow of dimension {dim} needs a declared scale, `scale: 1 {} -> n px` (PK-5.5)", crate::text::unit_text(dim)),
                    ))
                }
            };
            let from = match prop_expr(rep, "from") {
                Some(f) => Some(ce(f, Some(&Type::Point { space: space.clone() }))?.0),
                None => None,
            };
            CKind::Arrow { vec: c, ty: t, from, px_per_unit, label: label(e) }
        }
        "segment" => {
            let ViewCtx::Spatial { space, .. } = ctx else { return Err(d("PK-E05", "a segment belongs in a spatial view".into())) };
            let p = Type::Point { space: space.clone() };
            let a = ce(source(rep, 0).ok_or_else(|| needs("two points"))?, Some(&p))?.0;
            let b = ce(source(rep, 1).ok_or_else(|| needs("two points"))?, Some(&p))?.0;
            CKind::Segment { a, b }
        }
        "function_graph" => {
            let ViewCtx::Plot { x, .. } = ctx else { return Err(d("PK-E05", "a function graph belongs in a plot view".into())) };
            let e = source(rep, 0).ok_or_else(|| needs("a function"))?;
            let (c, t) = ce(e, None)?;
            // A field is a function over its grid (D-091): drawn through its cells.
            if let (Type::Field { grid, of }, Expr::Ref { .. }, ViewCtx::Plot { dims, .. }) = (&t, e, ctx) {
                if cm.grids.get(grid).is_some_and(|g| g.y.is_some()) {
                    return Err(d("PK-E05", "a field in the plane is drawn by `heatmap(T)` in a spatial view".into()));
                }
                let gdim = cm.grids.get(grid).map(|g| g.dim).unwrap_or_default();
                let Type::Quantity { dim } = **of else { return Err(needs("a field of numbers or quantities")) };
                if gdim != dims.0 || dim != dims.1 {
                    return Err(d("PK-E04", format!("the field runs over {gdim} with values of dimension {dim}; the plot's axes are {} and {} (PK-7.3)", dims.0, dims.1)));
                }
                let probe = Expr::Call { call: prismal_ir::Func::Mean, args: vec![e.clone()] };
                let (field, _) = ce(&probe, None)?;
                CKind::FieldGraph { field, label: label(e) }
            } else {
                if t != Type::func(vec![Type::real()], Type::real()) {
                    return Err(needs("a function from Real to Real, or a field"));
                }
                CKind::Graph { f: c, lo: x.0, hi: x.1, label: label(e) }
            }
        }
        // A field in the plane as coloured cells (D-091).
        "heatmap" => {
            let ViewCtx::Spatial { .. } = ctx else { return Err(d("PK-E05", "a heatmap belongs in a spatial view, where its grid lies".into())) };
            let e = source(rep, 0).ok_or_else(|| needs("a field in the plane: `heatmap(T)`"))?;
            let (_, t) = ce(e, None)?;
            let (Type::Field { grid, of }, Expr::Ref { .. }) = (&t, e) else { return Err(needs("a field in the plane: `heatmap(T)`")) };
            if !cm.grids.get(grid).is_some_and(|g| g.y.is_some()) {
                return Err(d("PK-E05", "a heatmap draws a field in the plane; a field on a line is drawn by `function_graph(T)` in a plot".into()));
            }
            let range = match rep.prop("range") {
                Some(Arg::Range { lo, hi }) => {
                    ce(lo, Some(of))?;
                    ce(hi, Some(of))?;
                    let (a, b) = (number(cm, lo).map_err(|m| d("PK-E02", m))?, number(cm, hi).map_err(|m| d("PK-E02", m))?);
                    if !(b > a) {
                        return Err(d("PK-E05", "a heatmap's range goes from low to high".into()));
                    }
                    Some((a, b))
                }
                _ => None,
            };
            let probe = Expr::Call { call: prismal_ir::Func::Mean, args: vec![e.clone()] };
            let (field, _) = ce(&probe, None)?;
            CKind::Heatmap { field, range, label: label(e) }
        }
        "histogram" => {
            let ViewCtx::Plot { dims, x, .. } = ctx else { return Err(d("PK-E05", "a histogram belongs in a plot view: x for the values, y for how many of them fall in each bin".into())) };
            let what = "an ensemble source, `n at t0 + 7 s over 200 runs`, a sampled source, `face every 1 s`, or the values at an event, `gap on decay`";
            let (e, instrument) = match rep.sources.first() {
                Some(Arg::Ensemble { expr, instrument, .. } | Arg::Sampled { expr, instrument, .. } | Arg::On { expr, instrument, .. }) => (expr, instrument),
                _ => return Err(needs(what)),
            };
            let (c, ty) = ce(e, None)?;
            let meter = measure::meter(cm, instruments, instrument.as_ref(), e, &ty).map_err(|m| d("PK-E04", m))?;
            let Type::Quantity { dim } = ty else { return Err(needs("a number or a quantity")) };
            if dim != dims.0 {
                return Err(d("PK-E04", format!("the values have dimension {dim}, the plot's x axis {} (PK-7.3)", dims.0)));
            }
            if !dims.1.is_none() {
                return Err(d("PK-E04", "a histogram's y axis counts values: a range of plain numbers, `y: [0, 50]` (PK-7.3)".into()));
            }
            let whole = |e: &Expr, lo: f64, hi: f64, what: &str| -> Result<usize, Vec<PDiag>> {
                match number(cm, e) {
                    Ok(n) if n.fract() == 0.0 && n >= lo && n <= hi => Ok(n as usize),
                    Ok(n) => Err(d("PK-E05", format!("{what} is a whole number from {lo} to {hi}, not {n}"))),
                    Err(m) => Err(d("PK-E02", m)),
                }
            };
            let source = match rep.sources.first() {
                Some(Arg::Ensemble { at, runs, .. }) => {
                    ce(at, Some(&Type::Instant { dim: Dim::time() }))?;
                    let at = number(cm, at).map_err(|m| d("PK-E02", m))?;
                    let runs = whole(runs, 1.0, crate::data::MAX_RUNS as f64, "a number of runs")?;
                    HistSource::Ensemble { expr: e.clone(), at, runs, cache: Default::default() }
                }
                Some(Arg::Sampled { every, .. }) => {
                    let dt = match compile_expr(cm, every, None) {
                        Ok((_, Type::Quantity { dim })) if dim == Dim::time() => number(cm, every).map_err(|m| d("PK-E02", m))?,
                        _ => return Err(d("PK-E04", "a sampling interval is a duration: `every 1 s`".into())),
                    };
                    if !(dt > 0.0) {
                        return Err(d("PK-E05", "a sampling interval is positive".into()));
                    }
                    HistSource::Every { value: c, every: dt }
                }
                Some(Arg::On { event, .. }) => {
                    let Some(ev) = cm.ir.events.iter().find(|x| &x.id == event) else { return Err(d("PK-E01", format!("unknown event `{event}`"))) };
                    HistSource::On { expr: e.clone(), event: ev.name.clone() }
                }
                _ => unreachable!(),
            };
            let bins = match prop_expr(rep, "bins") {
                Some(b) => whole(b, 1.0, 1000.0, "a number of bins")?,
                None => 20,
            };
            if !(x.1 > x.0) {
                return Err(d("PK-E05", "a histogram needs an x range from low to high".into()));
            }
            CKind::Histogram { source, bins, label: label(e), meter }
        }
        "trace" | "series_plot" => {
            let Some(Arg::Sampled { expr: e, every, instrument }) = rep.sources.first() else {
                return Err(needs("a sampled source: `pos every 0.02 s`"));
            };
            let dt = match compile_expr(cm, every, None) {
                Ok((_, Type::Quantity { dim })) if dim == Dim::time() => number(cm, every).map_err(|m| d("PK-E02", m))?,
                _ => return Err(d("PK-E04", "a sampling interval is a duration: `every 0.02 s`".into())),
            };
            if !(dt > 0.0) {
                return Err(d("PK-E05", "a sampling interval is positive".into()));
            }
            if rep.kind == "trace" {
                let ViewCtx::Spatial { space, .. } = ctx else { return Err(d("PK-E05", "a trace belongs in a spatial view".into())) };
                let (c, t) = ce(e, Some(&Type::Point { space: space.clone() }))?;
                let meter = measure::meter(cm, instruments, instrument.as_ref(), e, &t).map_err(|m| d("PK-E04", m))?;
                CKind::Trace { pos: c, every: dt, label: label(e), meter }
            } else {
                let ViewCtx::Plot { dims, .. } = ctx else { return Err(d("PK-E05", "a series plot belongs in a plot view".into())) };
                let (c, t) = ce(e, None)?;
                let Type::Quantity { dim } = t else { return Err(needs("a number or a quantity")) };
                if dims.0 != Dim::time() {
                    return Err(d("PK-E04", "a series plot draws against elapsed time: its plot's x axis is a time range, `x: [0 s, 5 s]` (PK-7.3)".into()));
                }
                if dim != dims.1 {
                    return Err(d("PK-E04", format!("the series has dimension {dim}, the plot's y axis {} (PK-7.3)", dims.1)));
                }
                let meter = measure::meter(cm, instruments, instrument.as_ref(), e, &t).map_err(|m| d("PK-E04", m))?;
                // Readings are taken at the samples: a staircase would show the exact value.
                let steps = meter.is_none() && piecewise_constant(cm, e);
                CKind::Series { value: c, every: dt, label: label(e), steps, meter }
            }
        }
        "label" => {
            let e = source(rep, 0).ok_or_else(|| needs("a value"))?;
            let (c, t) = ce(e, None)?;
            // A binding is shown in its display unit, as controls show it (MK-3.12).
            let idx = match e {
                Expr::Ref { r#ref } => cm.index.get(r#ref).copied(),
                _ => None,
            };
            CKind::Label { value: c, ty: t, label: label(e), idx }
        }
        // `text("bounced {n} times", at: P)` (D-076): the template's `{}` are its values.
        "text" => {
            let Some(Arg::Text { text }) = rep.sources.first() else { return Err(needs("a text in quotes: `text(\"Drag the ball\")`")) };
            let (pieces, places) = split_template(text);
            let mut values = vec![];
            for a in &rep.sources[1..] {
                let Arg::Expr { expr } = a else { return Err(needs("values in braces")) };
                let (c, t) = ce(expr, None)?;
                let idx = match expr {
                    Expr::Ref { r#ref } => cm.index.get(r#ref).copied(),
                    _ => None,
                };
                values.push((c, t, idx));
            }
            if pieces.len() != values.len() + 1 {
                return Err(needs("as many values as `{}` in its text"));
            }
            let at = match (prop_expr(rep, "at"), ctx) {
                (None, _) => None,
                (Some(e), ViewCtx::Spatial { space, .. }) => Some(ce(e, Some(&Type::Point { space: space.clone() }))?.0),
                (Some(e), ViewCtx::Plot { dims, .. }) => {
                    let q = |d: &Dim| Type::Quantity { dim: *d };
                    Some(ce(e, Some(&Type::Tuple { items: vec![q(&dims.0), q(&dims.1)] }))?.0)
                }
                (Some(_), ViewCtx::Panel) => return Err(d("PK-E05", "a text in a panel has no place to be put `at` (D-076)".into())),
            };
            CKind::Text { pieces, values, places, at }
        }
        // `title("The flight")`: the caption of its view or panel (D-076).
        "title" => match rep.sources.first() {
            Some(Arg::Text { text }) => CKind::Title { value: text.clone() },
            _ => return Err(needs("a title in quotes")),
        },
        "formula" => {
            let live = matches!(prop_expr(rep, "live"), Some(Expr::Bool { bool: true }));
            let (lhs, rhs, params) = match (rep.sources.first(), rep.sources.get(1)) {
                (Some(Arg::Text { text }), Some(Arg::Expr { expr })) => {
                    ce(expr, None)?;
                    (text.clone(), expr.clone(), vec![])
                }
                (Some(Arg::Expr { expr: Expr::Ref { r#ref } }), None) => {
                    let b = cm.ir.binding(r#ref).ok_or_else(|| d("PK-E01", format!("unknown binding `{ref}`", ref = r#ref)))?;
                    match (&b.role, &b.def) {
                        (Role::Derived, Some(Expr::Lambda { lambda: Lambda { body, names, params } })) => {
                            let names = if names.is_empty() { (1..=params.len()).map(|i| format!("x{i}")).collect() } else { names.clone() };
                            (format!("{}({})", symbol(cm, r#ref), names.join(", ")), (**body).clone(), names)
                        }
                        (Role::Derived, Some(def)) => (symbol(cm, r#ref), def.clone(), vec![]),
                        _ => return Err(d("PK-E05", format!("a formula of `{}` shows its definition; it is not a derived binding (D-034)", b.name))),
                    }
                }
                // A declared function shows its definition: `kinetic(m, v) = 0.5 m v²` (D-048).
                (Some(Arg::Expr { expr: Expr::Fn { r#fn } }), None) => {
                    let f = cm.ir.function(r#fn).ok_or_else(|| d("PK-E01", format!("unknown function `{}`", r#fn)))?;
                    let names = f.param_names();
                    (format!("{}({})", f.name, names.join(", ")), f.body.clone(), names)
                }
                _ => return Err(needs("a derived binding, or a label and an expression (D-034)")),
            };
            let refs = rhs
                .refs()
                .into_iter()
                .filter_map(|id| cm.index.get(&id).map(|&i| (id, i)))
                .filter(|(_, i)| !matches!(cm.bindings[*i].ty, Type::Function { .. }))
                .collect();
            CKind::Formula { lhs, rhs, params, refs, live }
        }
        "slider" | "number_input" | "toggle" | "select" => {
            let Some(Expr::Ref { r#ref }) = source(rep, 0) else { return Err(needs("the binding it controls")) };
            let idx = *cm.index.get(r#ref).ok_or_else(|| d("PK-E01", format!("unknown binding `{ref}`", ref = r#ref)))?;
            if !cm.bindings[idx].intervenable {
                return Err(d("PK-E03", format!("`{}` is not intervenable, so no control can target it (PK-6.4, D-023)", cm.bindings[idx].name)));
            }
            let (min, max) = match rep.prop("range") {
                Some(Arg::Range { lo, hi }) => (Some(number(cm, lo).map_err(|m| d("PK-E02", m))?), Some(number(cm, hi).map_err(|m| d("PK-E02", m))?)),
                _ => (None, None),
            };
            let step = match prop_expr(rep, "step") {
                Some(s) => Some(number(cm, s).map_err(|m| d("PK-E02", m))?),
                None => None,
            };
            let ends = match rep.prop("ends") {
                None => None,
                Some(Arg::Texts { texts }) if texts.len() == 2 && rep.kind == "slider" => Some([texts[0].clone(), texts[1].clone()]),
                Some(_) if rep.kind != "slider" => return Err(d("PK-E05", format!("a {} has no ends; `ends` names a slider's two ends (D-077)", rep.kind))),
                Some(_) => return Err(needs("two words for its ends")),
            };
            // D-094: a select offers the cases of an enumeration; the other controls take
            // numbers, quantities or (a toggle) a Boolean.
            let options = match (&cm.bindings[idx].ty, rep.kind.as_str()) {
                (Type::Enum { cases, .. }, "select") => Some(cases.clone()),
                (_, "select") => return Err(d("PK-E05", format!("a select chooses one case of an enumeration; `{}` is not one (D-094)", cm.bindings[idx].name))),
                (Type::Enum { .. }, kind) => return Err(d("PK-E05", format!("`{}` is an enumeration: choose its case with `select({})`, not a {kind} (D-094)", cm.bindings[idx].name, cm.bindings[idx].name))),
                _ => None,
            };
            CKind::Control { control: rep.kind.clone(), binding: r#ref.clone(), idx, min, max, step, ends, options }
        }
        "polyline" | "polygon" => {
            let ViewCtx::Spatial { space, .. } = ctx else { return Err(d("PK-E05", format!("a {} belongs in a spatial view", rep.kind))) };
            let p = Type::Point { space: space.clone() };
            let mut points = vec![];
            for a in &rep.sources {
                let Arg::Expr { expr } = a else { return Err(needs("points")) };
                points.push(ce(expr, Some(&p))?.0);
            }
            if points.len() < 2 {
                return Err(needs("at least two points"));
            }
            CKind::Poly { points, closed: rep.kind == "polygon" }
        }
        "circle" | "ellipse" | "arc" => {
            let ViewCtx::Spatial { space, .. } = ctx else { return Err(d("PK-E05", format!("a {} belongs in a spatial view", rep.kind))) };
            let what = match rep.kind.as_str() {
                "circle" => "a centre and a radius: `circle(P, 1 m)`",
                "ellipse" => "a centre and two radii: `ellipse(P, 2 m, 1 m)`",
                _ => "a centre and a radius: `arc(P, 1 m, from: 0 deg, to: 90 deg)`",
            };
            let n = if rep.kind == "ellipse" { 3 } else { 2 };
            if rep.sources.len() != n {
                return Err(needs(what));
            }
            let center = ce(source(rep, 0).ok_or_else(|| needs(what))?, Some(&Type::Point { space: space.clone() }))?.0;
            let length = |e: &Expr| -> Result<CExpr, Vec<PDiag>> {
                match ce(e, None)? {
                    (c, Type::Quantity { dim }) if dim == Dim::length() => Ok(c),
                    _ => Err(d("PK-E04", format!("a radius of a {} is a length: `1 m` (PK-6.3c)", rep.kind))),
                }
            };
            let angle = |name: &str| -> Result<Option<CExpr>, Vec<PDiag>> {
                let Some(e) = prop_expr(rep, name) else { return Ok(None) };
                match ce(e, None)? {
                    (c, Type::Quantity { dim }) if dim.is_none() => Ok(Some(c)),
                    _ => Err(d("PK-E04", format!("`{name}` of a {} is an angle: `90 deg` (PK-6.3c)", rep.kind))),
                }
            };
            let allowed: &[&str] = match rep.kind.as_str() {
                "circle" => &[],
                "ellipse" => &["rotate"],
                _ => &["from", "to"],
            };
            if let Some(p) = rep.props.iter().find(|p| !allowed.contains(&p.name.as_str())) {
                return Err(d("PK-E05", format!("a {} takes no `{}` (PK-6.3c)", rep.kind, p.name)));
            }
            let rx = length(source(rep, 1).ok_or_else(|| needs(what))?)?;
            let ry = if rep.kind == "ellipse" { Some(length(source(rep, 2).ok_or_else(|| needs(what))?)?) } else { None };
            let (from, to) = (angle("from")?, angle("to")?);
            if rep.kind == "arc" && (from.is_none() || to.is_none()) {
                return Err(needs(what));
            }
            CKind::Round { center, rx, ry, rotate: angle("rotate")?, from, to }
        }
        "equation" => {
            let Some(Arg::Element { element }) = rep.sources.first() else { return Err(needs("the name of a model equation")) };
            let q = cm.ir.equations.iter().find(|q| &q.id == element).ok_or_else(|| needs("the name of a model equation"))?;
            let live = matches!(prop_expr(rep, "live"), Some(Expr::Bool { bool: true }));
            let mut refs: Vec<(Id, usize)> = vec![];
            for id in q.lhs.refs().into_iter().chain(q.rhs.refs()) {
                if let Some(&i) = cm.index.get(&id) {
                    if !refs.iter().any(|r| r.0 == id) && !matches!(cm.bindings[i].ty, Type::Function { .. }) {
                        refs.push((id, i));
                    }
                }
            }
            CKind::Equation { name: q.name.clone(), lhs: q.lhs.clone(), rhs: q.rhs.clone(), refs, live }
        }
        "button" => {
            let msg = "the name of an event declared `on request` (D-027), or `reset`, `undo`, `redo` (D-069) or `reseed` (D-084) or `restart` (D-085)";
            let label_or = |w: &str| match rep.prop("label") {
                Some(Arg::Text { text }) => text.clone(),
                _ => w.to_string(),
            };
            match rep.sources.first() {
                Some(Arg::Word { word }) if ["reset", "undo", "redo", "reseed", "restart"].contains(&word.as_str()) => CKind::RunButton { control: word.clone(), label: label_or(word) },
                Some(Arg::Element { element }) => {
                    let Some(ev) = cm.ir.events.iter().find(|e| &e.id == element) else { return Err(needs(msg)) };
                    if !matches!(ev.trigger, prismal_ir::Trigger::Request) {
                        return Err(d("PK-E03", format!("`{}` is not declared `on request`, so no button can request it (D-027)", ev.name)));
                    }
                    // `button(add(2 m))` (D-072): a payload exactly when the event declares one,
                    // of its type; a button cannot name a member.
                    let payload = match rep.prop("payload") {
                        Some(Arg::Expr { expr }) => Some(expr.clone()),
                        _ => None,
                    };
                    match (&ev.payload, &payload) {
                        (Some(p), Some(e)) => {
                            if p.of.is_some() {
                                return Err(d("PK-E05", format!("`{}` takes a member; a member is chosen by clicking it, not by a button (D-059)", ev.name)));
                            }
                            match compile_expr(cm, e, Some(&p.ty)) {
                                Ok((_, t)) if t == p.ty => {}
                                Ok((_, t)) => return Err(d("PK-E04", format!("`{}` takes a {:?}; the button gives a {:?}", ev.name, p.ty, t))),
                                Err(ds) => return Err(d("PK-E04", format!("the value for `{}`: {}", ev.name, ds.iter().map(|x| x.message.clone()).collect::<Vec<_>>().join("; ")))),
                            }
                        }
                        (Some(_), None) => return Err(d("PK-E05", format!("`{}` takes a value: write it after the event, `button({}(...))` (D-072)", ev.name, ev.name))),
                        (None, Some(_)) => return Err(d("PK-E05", format!("`{}` takes no value", ev.name))),
                        (None, None) => {}
                    }
                    CKind::Button { event: element.clone(), label: label_or(&ev.name), payload }
                }
                _ => return Err(needs(msg)),
            }
        }
        "table" => {
            let Some(Arg::Sampled { expr: e, every, instrument }) = rep.sources.first() else {
                return Err(needs("a sampled source: `table((pos.x, pos.y) every 0.5 s)`"));
            };
            let dt = match compile_expr(cm, every, None) {
                Ok((_, Type::Quantity { dim })) if dim == Dim::time() => number(cm, every).map_err(|m| d("PK-E02", m))?,
                _ => return Err(d("PK-E04", "a sampling interval is a duration: `every 0.5 s`".into())),
            };
            if !(dt > 0.0) {
                return Err(d("PK-E05", "a sampling interval is positive".into()));
            }
            let (c, t) = ce(e, None)?;
            let (tys, columns) = match (&t, e) {
                (Type::Tuple { items }, Expr::Tuple { tuple }) => (items.clone(), tuple.iter().map(|x| print(x, cm, &[])).collect()),
                _ => (vec![t.clone()], vec![print(e, cm, &[])]),
            };
            let meter = measure::meter(cm, instruments, instrument.as_ref(), e, &t).map_err(|m| d("PK-E04", m))?;
            CKind::Table { value: c, tys, every: dt, columns, meter }
        }
        "group" => {
            let ViewCtx::Spatial { space, .. } = ctx else { return Err(d("PK-E05", "a group belongs in a spatial view (PK-6.3b)".into())) };
            if !rep.sources.is_empty() || rep.props.iter().any(|p| !["at", "rotate", "scale"].contains(&p.name.as_str())) {
                return Err(d("PK-E05", "a group takes `at`, `rotate` and `scale`; its members are written in its block (PK-6.3b)".into()));
            }
            let at = match prop_expr(rep, "at") {
                Some(e) => Some(ce(e, Some(&Type::Point { space: space.clone() }))?.0),
                None => None,
            };
            let real = |name: &str| -> Result<Option<CExpr>, Vec<PDiag>> {
                let Some(e) = prop_expr(rep, name) else { return Ok(None) };
                match ce(e, None)? {
                    (c, Type::Quantity { dim }) if dim.is_none() => Ok(Some(c)),
                    _ => Err(d("PK-E04", format!("a group's `{name}` is a number{} (PK-6.3b)", if name == "rotate" { " or an angle" } else { "" }))),
                }
            };
            let (rotate, scale) = (real("rotate")?, real("scale")?);
            if prop_expr(rep, "scale").is_some_and(|e| number(cm, e).is_ok_and(|k| k <= 0.0)) {
                return Err(d("PK-E02", "a group's scale is positive (PK-6.3b)".into()));
            }
            if rep.members.is_empty() {
                return Err(needs("members"));
            }
            let mut members = vec![];
            let mut diags = vec![];
            for m in &rep.members {
                if !["marker", "arrow", "segment", "polyline", "polygon", "circle", "ellipse", "arc", "group"].contains(&m.kind.as_str()) {
                    diags.push(PDiag {
                        code: "PK-E05",
                        message: format!("a group holds markers, arrows, segments, polylines, polygons, circles, ellipses, arcs and groups, not `{}` (PK-6.3b)", m.kind),
                        element: m.id.clone(),
                    });
                    continue;
                }
                match compile_rep(cm, ctx, m, instruments) {
                    Ok(c) => members.push(c),
                    Err(e) => diags.extend(e),
                }
            }
            if !diags.is_empty() {
                return Err(diags);
            }
            CKind::Group { at, rotate, scale, members }
        }
        "axes" => CKind::Axes,
        "grid" => CKind::Grid,
        other => return Err(d("PK-E06", format!("the representation kind `{other}` is not implemented by the prototype"))),
    };
    // The declared inverse (PK-5.6): targets must be intervenable, proposals must check with
    // the gesture's value in place.
    if let Some(inv) = &rep.inverse {
        let parts: &[&str] = match rep.kind.as_str() {
            "arrow" => &["head"],
            _ => &[],
        };
        if let Some(p) = &inv.part {
            if !parts.contains(&p.as_str()) {
                return Err(d("PK-E05", format!("`{}` has no part `{p}`", rep.kind)));
            }
        }
        for p in &inv.proposals {
            let i = *cm.index.get(&p.target).ok_or_else(|| d("PK-E01", format!("unknown binding `{}`", p.target)))?;
            if !cm.bindings[i].intervenable {
                return Err(d("PK-E03", format!("the inverse proposes `{}`, which is not intervenable (D-023)", cm.bindings[i].name)));
            }
            ce(&subst(&p.value, &ctx.pointer([0.0, 0.0])), Some(&cm.bindings[i].ty))?;
        }
    }
    // A click requests an event declared `on request`, with the payload it declares (D-027,
    // D-050, D-059).
    if let Some(c) = &rep.click {
        let ev = cm.ir.events.iter().find(|e| e.id == c.event).ok_or_else(|| d("PK-E01", format!("unknown event `{}`", c.event)))?;
        if !matches!(ev.trigger, prismal_ir::Trigger::Request) {
            return Err(d("PK-E03", format!("`{}` is not declared `on request`, so no click can request it (D-027)", ev.name)));
        }
        match (&ev.payload, &c.payload) {
            (None, None) => {}
            (Some(p), Some(v)) => {
                ce(v, Some(&p.ty))?;
            }
            (Some(p), None) => return Err(d("PK-E02", format!("`request {}` supplies its payload `{}`", ev.name, p.name))),
            (None, Some(_)) => return Err(d("PK-E02", format!("`{}` declares no payload", ev.name))),
        }
    }
    let when = match &rep.when {
        Some(w) => Some(compile_expr(cm, w, Some(&Type::Boolean)).map_err(|ds| d("PK-E02", ds.iter().map(|x| format!("{}: {}", x.code, x.message)).collect::<Vec<_>>().join("; ")))?.0),
        None => None,
    };
    Ok(CRep { rep: full.clone(), kind, when })
}

fn coords(v: &Value) -> Vec<f64> {
    crate::flat(v)
}

/// Projects a compiled representation on a state of a run (PK-5.1, PK-5.2).
pub fn project(cm: &CModel, ctx: &ViewCtx, r: &CRep, run: &Run, vals: &[Value], t: f64) -> RepFrame {
    project_in(cm, ctx, r, run, vals, t, Tf::ID)
}

/// Projects a representation whose values are in the frame of a group (D-043).
fn project_in(cm: &CModel, ctx: &ViewCtx, r: &CRep, run: &Run, vals: &[Value], t: f64, tf: Tf) -> RepFrame {
    let eval = |c: &CExpr| run.eval_state(c, vals, t).map(|v| tf.apply(v));
    let status = |s: prismal_kernel::Status| (Shape::Status { status: s.cause.clone() }, format!("{}: not available ({})", r.rep.name.clone().unwrap_or(r.rep.kind.clone()), s.cause));
    // Breaks of a sampled path (trace, series): where the value jumped between two samples.
    let mut breaks = vec![];
    let (shape, text) = match &r.kind {
        CKind::Marker { pos, label } => match eval(pos) {
            Ok(v) => {
                let c = coords(&v);
                let unit = if matches!(ctx, ViewCtx::Spatial { .. }) { " m" } else { "" };
                let s = ctx.shown(&c);
                (Shape::Point { at: ctx.to_view(&c) }, format!("{label} at x = {}{unit}, y = {}{unit}", fmt_num(s[0]), fmt_num(s[1])))
            }
            Err(s) => status(s),
        },
        CKind::Arrow { vec, ty, from, px_per_unit, label } => {
            let base = match from.as_ref().map(eval) {
                Some(Ok(v)) => coords(&v),
                Some(Err(s)) => {
                    let (sh, tx) = status(s);
                    return RepFrame { id: r.rep.id.clone(), kind: r.rep.kind.clone(), name: r.rep.name.clone(), shape: sh, text: tx, highlighted: false, valid: None, opacity: None, drawn: None, label: None, drag: None, click: None, color: None, line: None, color_to: None, color_mix: None, scale: None, breaks: vec![] };
                }
                None => vec![0.0, 0.0],
            };
            match eval(vec) {
                Ok(v) => {
                    let c = coords(&v);
                    let from_v = ctx.to_view(&base);
                    let ViewCtx::Spatial { y_up, .. } = ctx else { unreachable!() };
                    let dy = if *y_up { -c[1] } else { c[1] };
                    let to = [from_v[0] + c[0] * px_per_unit, from_v[1] + dy * px_per_unit];
                    let from_text = fmt_value(&Value::Point(prismal_kernel::Arr::from_slice(&base)), &Type::point(""));
                    (Shape::Arrow { from: from_v, to }, format!("arrow {label} = {} from {from_text}", fmt_value(&v, ty)))
                }
                Err(s) => status(s),
            }
        }
        CKind::Segment { a, b } => match (eval(a), eval(b)) {
            (Ok(x), Ok(y)) => (Shape::Segment { from: ctx.to_view(&coords(&x)), to: ctx.to_view(&coords(&y)) }, "segment".into()),
            (Err(s), _) | (_, Err(s)) => status(s),
        },
        CKind::Graph { f, lo, hi, label } => match eval(f) {
            Ok(Value::Func(body)) => {
                // Over the range the learner has zoomed or panned to, where a host says so.
                let (lo, hi) = match ctx {
                    ViewCtx::Plot { sample_x: Some(s), .. } => (&s.0, &s.1),
                    _ => (lo, hi),
                };
                let n = 100;
                let mut pts = Vec::with_capacity(n + 1);
                for i in 0..=n {
                    let x = lo + (hi - lo) * i as f64 / n as f64;
                    let args = [Value::Num(x)];
                    let y = body.eval(&Ctx { vals, der: None, t, t0: run.config.t0, args: &args, payloads: &[], draws: None, noise: &[], point: None });
                    if let Ok(Value::Num(y)) = y {
                        pts.push([x, y]);
                    }
                }
                (Shape::Polyline { points: pts }, format!("graph of {label} for x from {} to {}", fmt_num(*lo), fmt_num(*hi)))
            }
            Ok(_) => (Shape::Status { status: "not a function".into() }, "graph: not available".into()),
            Err(s) => status(s),
        },
        CKind::Heatmap { field, range, label } => {
            let c = Ctx { vals, der: None, t, t0: run.config.t0, args: &[], payloads: &[], draws: None, noise: &[], point: None };
            match field.field_cells(&c) {
                Ok((op, u)) => {
                    let g = op.grid;
                    let y = g.y.expect("a grid in the plane");
                    let (mn, mx) = u.iter().fold((f64::INFINITY, f64::NEG_INFINITY), |(a, b), v| (a.min(*v), b.max(*v)));
                    let (mut lo, mut hi) = range.unwrap_or((mn, mx));
                    // Values on both sides of 0 are shown on a diverging palette, even about 0.
                    let diverging = lo < 0.0 && hi > 0.0;
                    if diverging && range.is_none() {
                        let m = lo.abs().max(hi);
                        (lo, hi) = (-m, m);
                    }
                    if !(hi > lo) {
                        hi = lo + 1.0;
                    }
                    let values: Vec<f64> = u.iter().map(|v| ((v - lo) / (hi - lo)).clamp(0.0, 1.0)).collect();
                    let from = ctx.to_view(&[g.x.lo, y.lo]);
                    let to = ctx.to_view(&[g.x.hi(), y.hi()]);
                    let k = u.iter().enumerate().fold(0, |b, (k, v)| if *v > u[b] { k } else { b });
                    let at = coords(&g.coord(k));
                    let text = format!(
                        "heatmap of {label}: {} by {} cells, values from {} to {}, highest near ({}, {})",
                        g.x.n, y.n, fmt_num(mn), fmt_num(mx), fmt_num(at[0]), fmt_num(at[1])
                    );
                    let palette = if diverging { "diverging" } else { "sequential" }.to_string();
                    (Shape::Raster { from, to, cols: g.x.n, rows: y.n, values, lo, hi, palette }, text)
                }
                Err(s) => status(s),
            }
        }
        CKind::FieldGraph { field, label } => {
            let c = Ctx { vals, der: None, t, t0: run.config.t0, args: &[], payloads: &[], draws: None, noise: &[], point: None };
            match field.field_profile(&c) {
                Ok(pts) => {
                    let (lo, hi) = (pts[0][0], pts[pts.len() - 1][0]);
                    let (mn, mx) = pts.iter().fold((f64::INFINITY, f64::NEG_INFINITY), |(a, b), p| (a.min(p[1]), b.max(p[1])));
                    let text = format!("field {label} over x from {} to {}, {} cells: values from {} to {}", fmt_num(lo), fmt_num(hi), pts.len() - 2, fmt_num(mn), fmt_num(mx));
                    (Shape::Polyline { points: pts }, text)
                }
                Err(s) => status(s),
            }
        }
        CKind::Trace { pos, every, label, meter } => {
            // A member's trace starts when it is made and ends when it is destroyed (D-057).
            let mut prev: Option<f64> = None;
            let mut pts: Vec<[f64; 2]> = vec![];
            for s in samples(run, t, *every) {
                let st = run.state_at(s);
                if !r.shown(run, &st, s) {
                    continue;
                }
                let Ok(v) = run.eval_state(pos, &st, s) else { continue };
                let Some(v) = read(meter, v, run.config.seed, s) else { continue };
                if prev.is_some_and(|a| jumped(run, pos, a, s)) && !pts.is_empty() {
                    breaks.push(pts.len());
                }
                prev = Some(s);
                pts.push(ctx.to_view(&coords(&v)));
            }
            let text = match (pts.first(), pts.last()) {
                (Some(_), Some(_)) => format!("trace of {label}: {} samples every {} s up to t = {} s", pts.len(), fmt_num(*every), fmt_num(t - run.config.t0)),
                _ => format!("trace of {label}: no samples"),
            };
            (Shape::Polyline { points: pts }, measured(text, meter))
        }
        CKind::Histogram { source, bins, label, meter } => {
            let ViewCtx::Plot { x: (lo, hi), .. } = ctx else { unreachable!() };
            let seed = run.config.seed;
            let shown = t - run.config.t0;
            // Readings outside an instrument's range are left out (PK-3.7).
            let num = |v: Option<Value>| match v {
                Some(Value::Num(x)) => Some(x),
                _ => None,
            };
            let (values, whence) = match source {
                HistSource::Ensemble { expr, at, runs, cache } => {
                    let cached = cache.borrow().as_ref().filter(|c| c.0 == run.config).map(|c| c.1.clone());
                    let values = match cached {
                        Some(v) => Ok(v),
                        None => crate::data::ensemble(run, expr, *at, *runs).map(|vs| {
                            // The k-th run has the k-th seed (PK-3.8).
                            let xs: Vec<f64> = vs.into_iter().enumerate().filter_map(|(k, v)| num(read(meter, v, seed.wrapping_add(k as u64), *at))).collect();
                            *cache.borrow_mut() = Some((run.config.clone(), xs.clone()));
                            xs
                        }),
                    };
                    (values, format!("at {} s over {runs} runs", fmt_num(at - run.config.t0)))
                }
                HistSource::Every { value, every } => {
                    let xs: Vec<f64> = samples(run, t, *every).filter_map(|s| run.eval_state(value, &run.state_at(s), s).ok().and_then(|v| num(read(meter, v, seed, s)))).collect();
                    let n = xs.len();
                    (Ok(xs), format!("every {} s up to {} s, {n} values", fmt_num(*every), fmt_num(shown)))
                }
                HistSource::On { expr, event } => {
                    let times = run.times(event);
                    let xs: Vec<f64> = times.iter().zip(run.on(event, expr, None)).filter(|(s, _)| **s <= t).filter_map(|(s, v)| num(read(meter, v, seed, *s))).collect();
                    let n = xs.len();
                    (Ok(xs), format!("on {event} up to {} s, {n} values", fmt_num(shown)))
                }
            };
            match values {
                Ok(xs) if xs.is_empty() => (Shape::Polygon { points: vec![[*lo, 0.0], [*hi, 0.0]] }, measured(format!("histogram of {label} {whence}: no values yet"), meter)),
                Ok(xs) => {
                    let w = (hi - lo) / *bins as f64;
                    let outside = xs.iter().filter(|&&v| !(v >= *lo && v <= *hi)).count();
                    // Values on a grid coarser than the bins (counts, shares of a few
                    // molecules) get one bar per grid value, centred on it and as wide as the
                    // grid's step, so that no empty bins fall between them (PK-6.3e).
                    let bars: Vec<(f64, f64, usize)> = match grid_step(&xs) {
                        Some((start, d)) if d >= w * (1.0 - 1e-9) => {
                            let mut bars: Vec<(f64, f64, usize)> = vec![];
                            for &v in xs.iter().filter(|&&v| v >= *lo && v <= *hi) {
                                let k = ((v - start) / d).round();
                                let c = start + k * d;
                                match bars.iter_mut().find(|b| (b.0 + d / 2.0 - c).abs() < d / 4.0) {
                                    Some(b) => b.2 += 1,
                                    None => bars.push((c - d / 2.0, c + d / 2.0, 1)),
                                }
                            }
                            bars.sort_by(|a, b| a.0.total_cmp(&b.0));
                            bars
                        }
                        _ => {
                            let mut counts = vec![0usize; *bins];
                            for &v in xs.iter().filter(|&&v| v >= *lo && v <= *hi) {
                                counts[(((v - lo) / w) as usize).min(bins - 1)] += 1;
                            }
                            counts.iter().enumerate().map(|(k, c)| (lo + k as f64 * w, lo + (k + 1) as f64 * w, *c)).collect()
                        }
                    };
                    // The outline of the bars, from the axis up and along each bar, back to
                    // the axis between bars that do not touch.
                    let mut pts = vec![[*lo, 0.0]];
                    for (a, b, c) in &bars {
                        let (a, b) = (a.max(*lo), b.min(*hi));
                        if pts.last().is_some_and(|p| p[0] < a - 1e-12 * (hi - lo)) {
                            pts.push([pts.last().unwrap()[0], 0.0]);
                            pts.push([a, 0.0]);
                        }
                        pts.push([a, *c as f64]);
                        pts.push([b, *c as f64]);
                    }
                    pts.push([pts.last().unwrap()[0], 0.0]);
                    pts.push([*hi, 0.0]);
                    let n = xs.len() as f64;
                    let mean = xs.iter().sum::<f64>() / n;
                    let dev = if n > 1.0 { (xs.iter().map(|x| (x - mean) * (x - mean)).sum::<f64>() / (n - 1.0)).sqrt() } else { 0.0 };
                    let more = if outside > 0 { format!("; {outside} outside the axis") } else { String::new() };
                    let text = format!("histogram of {label} {whence}: mean {}, deviation {}{more}", fmt_num(mean), fmt_num(dev));
                    (Shape::Polygon { points: pts }, measured(text, meter))
                }
                Err(m) => (Shape::Status { status: m.clone() }, format!("histogram of {label}: not available ({m})")),
            }
        }
        CKind::Series { value, every, label, steps, meter } => {
            let t0 = run.config.t0;
            let mut prev: Option<f64> = None;
            let mut pts: Vec<[f64; 2]> = vec![];
            // A value that changes only at transitions holds until it changes: a staircase
            // with a step at each instant it changed, up to `STEPS` steps (D-084).
            let staircase = if *steps { stairs(run, value, t) } else { None };
            // A reading outside an instrument's range leaves a gap (PK-3.7).
            let mut gap = false;
            for s in samples(run, t, *every).filter(|_| staircase.is_none()) {
                let Ok(v) = run.eval_state(value, &run.state_at(s), s) else { continue };
                let Some(Value::Num(y)) = read(meter, v, run.config.seed, s) else {
                    gap = true;
                    continue;
                };
                // A flowing value that jumped breaks the path (D-078).
                if (gap || prev.is_some_and(|a| jumped(run, value, a, s))) && !pts.is_empty() {
                    breaks.push(pts.len());
                }
                gap = false;
                prev = Some(s);
                pts.push([s - t0, y]);
            }
            if let Some(st) = staircase {
                pts = st;
            }
            // Readings are separate measurements, drawn as points, not joined into a path
            // (PK-3.7, D-087).
            if meter.is_some() {
                breaks.clear();
                let text = match pts.last() {
                    Some(p) => format!("readings of {label} against elapsed time: {} readings, last {} at {} s", pts.len(), fmt_num(p[1]), fmt_num(p[0])),
                    None => format!("readings of {label}: none"),
                };
                let color = style_word(&r.rep, "color");
                let members = pts.iter().enumerate().map(|(k, p)| dot(format!("{}.reading[{}]", r.rep.id, k + 1), *p, color.clone())).collect();
                (Shape::Group { members }, measured(text, meter))
            } else {
                let text = match pts.last() {
                    Some(p) => format!("series of {label} against elapsed time: {} samples, last {} at {} s", pts.len(), fmt_num(p[1]), fmt_num(p[0])),
                    None => format!("series of {label}: no samples"),
                };
                (Shape::Polyline { points: pts }, text)
            }
        }
        CKind::Text { pieces, values, places, at } => {
            let mut s = pieces[0].clone();
            for (k, (c, ty, idx)) in values.iter().enumerate() {
                match eval(c) {
                    Ok(v) => s.push_str(&crate::text::superscripts(&match (idx, places[k]) {
                        (_, Some(dp)) => crate::text::fmt_places(cm, *idx, &v, ty, dp),
                        (Some(i), None) => fmt_binding(cm, *i, &v),
                        (None, None) => fmt_value(&v, ty),
                    })),
                    // A value that is not available shows as `?`, as a label shows its status.
                    Err(_) => s.push('?'),
                }
                s.push_str(&pieces[k + 1]);
            }
            match at.as_ref().map(eval) {
                None => (Shape::Text { value: s.clone() }, s),
                Some(Ok(p)) => (Shape::Note { at: ctx.to_view(&coords(&p)), value: s.clone() }, s),
                Some(Err(st)) => status(st),
            }
        }
        CKind::Title { value } => (Shape::Title { value: value.clone() }, value.clone()),
        CKind::Label { value, ty, label, idx } => match eval(value) {
            Ok(v) => {
                let s = match idx {
                    Some(i) => fmt_binding(cm, *i, &v),
                    None => fmt_value(&v, ty),
                };
                (Shape::Text { value: s.clone() }, format!("{label} = {s}"))
            }
            Err(s) => status(s),
        },
        CKind::Formula { lhs, rhs, params, refs, live } => {
            let mut symbols = vec![];
            for (id, i) in refs {
                let value = if *live { Some(fmt_binding(cm, *i, &vals[*i])) } else { None };
                symbols.push(Symbol { binding: id.clone(), symbol: symbol(cm, id), value });
            }
            let mut text = format!("{lhs} = {}", print(rhs, cm, params));
            let values: Vec<String> = symbols.iter().filter_map(|s| s.value.as_ref().map(|v| format!("{} = {v}", s.symbol))).collect();
            if !values.is_empty() {
                text = format!("{text}, where {}", values.join(", "));
            }
            let layout = math::layout(&math::formula(lhs, rhs, cm, params));
            (Shape::Formula { lhs: lhs.clone(), rhs: rhs.clone(), symbols, layout }, text)
        }
        CKind::Control { control, binding, idx, min, max, step, ends, options } => {
            let v = match &vals[*idx] {
                Value::Num(x) => *x,
                Value::Bool(b) => *b as u8 as f64,
                Value::Case(c) => *c as f64,
                _ => f64::NAN,
            };
            let range = match (min, max) {
                (Some(a), Some(b)) => format!(", from {} to {}", fmt_binding(cm, *idx, &Value::Num(*a)), fmt_binding(cm, *idx, &Value::Num(*b))),
                _ => String::new(),
            };
            let words = ends.as_ref().map(|[a, b]| format!(", from {a} to {b}")).unwrap_or_default();
            let choices = options.as_ref().map(|o| format!(", one of {}", o.join(", "))).unwrap_or_default();
            let shown = match &vals[*idx] {
                Value::Case(c) => options.as_ref().and_then(|o| o.get(*c as usize)).cloned().unwrap_or_default(),
                v => fmt_binding(cm, *idx, v),
            };
            let text = format!("{control} for {} = {shown}{range}{words}{choices}", symbol(cm, binding));
            let display_unit = cm.ir.binding(binding).and_then(|b| b.display.unit.as_ref()).and_then(|u| prismal_ir::Unit::parse(u).ok()).map(|u| DisplayUnit { text: u.text, scale: u.scale });
            let unit = match (&display_unit, &cm.bindings[*idx].ty) {
                (None, Type::Quantity { dim }) if !dim.is_none() => Some(unit_text(dim)),
                _ => None,
            };
            let shape = Shape::Control { control: control.clone(), binding: binding.clone(), value: v, min: *min, max: *max, step: *step, symbol: symbol(cm, binding), display_unit, unit, ends: ends.clone(), options: options.clone() };
            (shape, text)
        }
        CKind::Poly { points, closed } => {
            let mut pts = vec![];
            for p in points {
                match eval(p) {
                    Ok(v) => pts.push(ctx.to_view(&coords(&v))),
                    Err(s) => {
                        let (sh, tx) = status(s);
                        return RepFrame { id: r.rep.id.clone(), kind: r.rep.kind.clone(), name: r.rep.name.clone(), shape: sh, text: tx, highlighted: false, valid: None, opacity: None, drawn: None, label: None, drag: None, click: None, color: None, line: None, color_to: None, color_mix: None, scale: None, breaks: vec![] };
                    }
                }
            }
            let text = format!("{} through {} points", r.rep.kind, pts.len());
            (if *closed { Shape::Polygon { points: pts } } else { Shape::Polyline { points: pts } }, text)
        }
        CKind::Round { center, rx, ry, rotate, from, to } => {
            let num = |c: &CExpr| match run.eval_state(c, vals, t) {
                Ok(Value::Num(x)) => Ok(x),
                Ok(_) => Err(None),
                Err(s) => Err(Some(s)),
            };
            let opt = |c: &Option<CExpr>, default: f64| c.as_ref().map(num).unwrap_or(Ok(default));
            let rx_v = num(rx);
            let ry_v = ry.as_ref().map(num).unwrap_or_else(|| rx_v.clone());
            match (eval(center), rx_v, ry_v, opt(rotate, 0.0), opt(from, 0.0), opt(to, std::f64::consts::TAU)) {
                (Ok(c), Ok(a), Ok(b), Ok(rot), Ok(t1), Ok(t2)) if a >= 0.0 && b >= 0.0 => {
                    // In the model's space: centre c, radii scaled by the group, axes turned
                    // by the ellipse's rotation and the group's.
                    let c = coords(&c);
                    let (a, b, rot) = (a * tf.k, b * tf.k, rot + tf.angle);
                    let (s, co) = rot.sin_cos();
                    let cv = ctx.to_view(&c);
                    let ue = ctx.to_view(&[c[0] + a * co, c[1] + a * s]);
                    let ve = ctx.to_view(&[c[0] - b * s, c[1] + b * co]);
                    let (u, v) = ([ue[0] - cv[0], ue[1] - cv[1]], [ve[0] - cv[0], ve[1] - cv[1]]);
                    // A view with y up reverses the turning sense: angles change sign.
                    let sense = if u[0] * v[1] - u[1] * v[0] >= 0.0 { 1.0 } else { -1.0 };
                    let rotation = u[1].atan2(u[0]);
                    let closed = r.rep.kind != "arc";
                    let shape = Shape::Ellipse { center: cv, radii: [u[0].hypot(u[1]), v[0].hypot(v[1])], rotation, start: sense * t1, sweep: sense * (t2 - t1), closed };
                    let name = r.rep.name.as_deref().map(|n| format!(" {n}")).unwrap_or_default();
                    let s = ctx.shown(&c);
                    let at = format!("x = {} m, y = {} m", fmt_num(s[0]), fmt_num(s[1]));
                    let deg = |x: f64| fmt_num(x.to_degrees());
                    let text = match r.rep.kind.as_str() {
                        "circle" => format!("circle{name} around {at}, radius {} m", fmt_num(a)),
                        "ellipse" => format!("ellipse{name} around {at}, radii {} m and {} m, turned {} deg", fmt_num(a), fmt_num(b), deg(rot)),
                        _ => format!("arc{name} around {at}, radius {} m, from {} deg to {} deg", fmt_num(a), deg(t1 + tf.angle), deg(t2 + tf.angle)),
                    };
                    (shape, text)
                }
                (Err(s), ..) => status(s),
                (_, Err(Some(s)), ..) | (_, _, Err(Some(s)), ..) | (_, _, _, Err(Some(s)), ..) | (_, _, _, _, Err(Some(s)), _) | (_, _, _, _, _, Err(Some(s))) => status(s),
                _ => (Shape::Status { status: "negative radius".into() }, format!("{}: not available (negative radius)", r.rep.name.clone().unwrap_or(r.rep.kind.clone()))),
            }
        }
        CKind::Equation { name, lhs, rhs, refs, live } => {
            let symbols: Vec<Symbol> = refs
                .iter()
                .map(|(id, i)| Symbol { binding: id.clone(), symbol: symbol(cm, id), value: if *live { Some(crate::text::fmt_binding(cm, *i, &vals[*i])) } else { None } })
                .collect();
            let mut text = format!("equation {name}: {} = {}", print(lhs, cm, &[]), print(rhs, cm, &[]));
            let values: Vec<String> = symbols.iter().filter_map(|s| s.value.as_ref().map(|v| format!("{} = {v}", s.symbol))).collect();
            if !values.is_empty() {
                text = format!("{text}, where {}", values.join(", "));
            }
            let layout = math::layout(&math::equation(lhs, rhs, cm));
            (Shape::Equation { name: name.clone(), lhs: lhs.clone(), rhs: rhs.clone(), symbols, layout }, text)
        }
        CKind::Button { event, label, .. } => {
            let name = cm.ir.events.iter().find(|e| &e.id == event).map(|e| e.name.clone()).unwrap_or_default();
            (Shape::Button { event: Some(event.clone()), control: None, label: label.clone() }, format!("button {label}: requests {name}"))
        }
        CKind::RunButton { control, label } => (Shape::Button { event: None, control: Some(control.clone()), label: label.clone() }, format!("button {label}: {control}")),
        CKind::Table { value, tys, every, columns, meter } => {
            let t0 = run.config.t0;
            let mut rows = vec![];
            for s in samples(run, t, *every) {
                let Ok(v) = run.eval_state(value, &run.state_at(s), s) else { continue };
                let mut row = vec![fmt_num(s - t0)];
                let Some(v) = read(meter, v, run.config.seed, s) else {
                    row.extend(tys.iter().map(|_| "out of range".to_string()));
                    rows.push(row);
                    continue;
                };
                match (&v, tys.len()) {
                    (Value::Tuple(items), n) if n > 1 => row.extend(items.iter().zip(tys).map(|(x, ty)| fmt_value(x, ty))),
                    _ => row.push(fmt_value(&v, &tys[0])),
                }
                rows.push(row);
            }
            let mut cols = vec!["t (s)".to_string()];
            cols.extend(columns.iter().cloned());
            let text = format!("table of {} with {} rows, every {} s", columns.join(", "), rows.len(), fmt_num(*every));
            (Shape::Table { columns: cols, rows }, measured(text, meter))
        }
        CKind::Group { at, rotate, scale, members } => {
            let num = |c: &Option<CExpr>, default: f64| match c.as_ref().map(eval) {
                Some(Ok(Value::Num(x))) => Ok(x),
                Some(Ok(_)) | None => Ok(default),
                Some(Err(s)) => Err(s),
            };
            // `at` is evaluated in the enclosing frame, so nested groups compose.
            let at = match at.as_ref().map(eval) {
                Some(Ok(v)) => Ok(coords(&v)),
                Some(Err(s)) => Err(s),
                None => Ok(tf.at.to_vec()),
            };
            match (at, num(rotate, 0.0), num(scale, 1.0)) {
                (Ok(a), Ok(angle), Ok(k)) => {
                    let inner = Tf { at: [a[0], a[1]], angle: tf.angle + angle, k: tf.k * k };
                    let ms: Vec<RepFrame> = members.iter().filter(|m| m.shown(run, vals, t)).map(|m| project_in(cm, ctx, m, run, vals, t, inner)).collect();
                    let name = r.rep.name.clone().unwrap_or("group".into());
                    let text = format!("{name}: {}", ms.iter().map(|m| m.text.as_str()).collect::<Vec<_>>().join("; "));
                    (Shape::Group { members: ms }, text)
                }
                (Err(s), _, _) | (_, Err(s), _) | (_, _, Err(s)) => status(s),
            }
        }
        CKind::Axes => (Shape::Axes, "axes".into()),
        CKind::Grid => (Shape::Grid, "grid".into()),
    };
    // The label drawn beside a marker, an arrow or a graph: its name or source, the
    // author's `label: "..."`, or none with `label: none` (D-078). The text alternative
    // keeps the name either way.
    let label = match (&r.kind, r.rep.prop("label")) {
        (CKind::Marker { .. } | CKind::Arrow { .. } | CKind::Graph { .. }, Some(Arg::Word { word })) if word == "none" => None,
        (CKind::Marker { .. } | CKind::Arrow { .. } | CKind::Graph { .. }, Some(Arg::Text { text })) => Some(text.clone()),
        (CKind::Marker { label, .. } | CKind::Arrow { label, .. } | CKind::Graph { label, .. }, _) => Some(label.clone()),
        _ => None,
    };
    let drag = r.rep.inverse.as_ref().map(|inv| inv.part.clone().unwrap_or_else(|| "body".into()));
    // A clickable representation says what activating it does (PK-11.1, D-059).
    let click = r.rep.click.as_ref().map(|c| cm.ir.events.iter().find(|e| e.id == c.event).map(|e| e.name.clone()).unwrap_or_else(|| c.event.clone()));
    let text = match &click {
        Some(e) => format!("{text}, activate: {e}"),
        None => text,
    };
    let (color, line) = (style_word(&r.rep, "color"), style_word(&r.rep, "line"));
    RepFrame { id: r.rep.id.clone(), kind: r.rep.kind.clone(), name: r.rep.name.clone(), shape, text, highlighted: false, valid: None, opacity: None, drawn: None, label, drag, click, color, line, color_to: None, color_mix: None, scale: None, breaks }
}

/// The grid the finite values lie on, `start + k d`, when there are at least two distinct
/// values and every one is within a millionth of `d` of a grid point (D-085).
fn grid_step(xs: &[f64]) -> Option<(f64, f64)> {
    let mut v: Vec<f64> = xs.iter().copied().filter(|x| x.is_finite()).collect();
    v.sort_by(f64::total_cmp);
    v.dedup_by(|a, b| (*a - *b).abs() <= 1e-12 * b.abs().max(1.0));
    if v.len() < 2 {
        return None;
    }
    let d = v.windows(2).map(|p| p[1] - p[0]).fold(f64::INFINITY, f64::min);
    let on = v.iter().all(|x| {
        let k = (x - v[0]) / d;
        (k - k.round()).abs() < 1e-6
    });
    on.then_some((v[0], d))
}

/// The most steps a staircase draws; a value that changed more often is drawn from samples.
const STEPS: usize = 4000;

/// The staircase of a value that changes only at transitions, from `t0` to `t` (D-084): one
/// step at each instant where its value changed, in elapsed time. `None` past `STEPS` steps.
fn stairs(run: &Run, e: &CExpr, t: f64) -> Option<Vec<[f64; 2]>> {
    let t0 = run.config.t0;
    let mut pts: Vec<[f64; 2]> = vec![];
    let mut k = 0;
    while k < run.committed.len() && run.committed[k].t <= t {
        let tau = run.committed[k].t;
        // The last state committed at this instant.
        while k + 1 < run.committed.len() && run.committed[k + 1].t == tau {
            k += 1;
        }
        if let Ok(Value::Num(y)) = run.eval_state(e, &run.committed[k].vals, tau) {
            match pts.last() {
                None => pts.push([tau - t0, y]),
                Some(&[_, py]) if py != y => {
                    if pts.len() > 2 * STEPS {
                        return None;
                    }
                    pts.push([tau - t0, py]);
                    pts.push([tau - t0, y]);
                }
                _ => {}
            }
        }
        k += 1;
    }
    let last = pts.last().copied()?;
    pts.push([t - t0, last[1]]);
    Some(pts)
}

/// Whether `e` changes only at transitions: it reads, directly or through derived values,
/// no continuous state, no time and no derivative (D-084).
fn piecewise_constant(cm: &CModel, e: &Expr) -> bool {
    let mut seen: Vec<Id> = vec![];
    let mut stack: Vec<&Expr> = vec![e];
    while let Some(x) = stack.pop() {
        let mut ok = true;
        x.walk(&mut |y| match y {
            Expr::Builtin { builtin: prismal_ir::Builtin::T | prismal_ir::Builtin::Elapsed } | Expr::Der { .. } => ok = false,
            _ => {}
        });
        if !ok {
            return false;
        }
        for r in x.refs() {
            if seen.contains(&r) {
                continue;
            }
            let Some(b) = cm.ir.bindings.iter().find(|b| b.id == r) else { return false };
            match b.role {
                // An unknown follows the state its equations read (D-092).
                Role::Continuous | Role::Unknown => return false,
                Role::Derived => match &b.def {
                    Some(d) => stack.push(d),
                    None => return false,
                },
                _ => {}
            }
            seen.push(r);
        }
    }
    true
}

/// Whether the value of `e` jumped at an instant in `(a, b]`: changed by a transition (an
/// event or an intervention) rather than by flowing. Such an instant has several committed
/// states; the value is compared before and after them.
fn jumped(run: &Run, e: &CExpr, a: f64, b: f64) -> bool {
    let mut i = run.committed.partition_point(|c| c.t <= a);
    while i < run.committed.len() && run.committed[i].t <= b {
        let tau = run.committed[i].t;
        let j = run.committed[i..].iter().take_while(|c| c.t == tau).count() + i;
        if j - i > 1 {
            let before = run.eval_state(e, &run.committed[i].vals, tau).ok();
            let after = run.eval_state(e, &run.committed[j - 1].vals, tau).ok();
            if let (Some(x), Some(y)) = (before, after) {
                let (x, y) = (coords(&x), coords(&y));
                if x.iter().zip(&y).any(|(p, q)| (p - q).abs() > 1e-9 * (1.0 + p.abs().max(q.abs()))) {
                    return true;
                }
            }
        }
        i = j;
    }
    false
}

/// One reading of a measured series: a point with no label.
fn dot(id: Id, at: [f64; 2], color: Option<String>) -> RepFrame {
    RepFrame {
        id,
        kind: "marker".into(),
        name: None,
        shape: Shape::Point { at },
        text: format!("reading {} at {} s", fmt_num(at[1]), fmt_num(at[0])),
        highlighted: false,
        valid: None,
        opacity: None,
        drawn: None,
        label: None,
        drag: None,
        click: None,
        color,
        line: None,
        color_to: None,
        color_mix: None,
        scale: None,
        breaks: vec![],
    }
}

/// A value as an instrument reads it (PK-3.7), or the value itself without one.
fn read(meter: &Option<Meter>, v: Value, seed: u64, t: f64) -> Option<Value> {
    match meter {
        Some(m) => m.read(&v, seed, t),
        None => Some(v),
    }
}

/// A text alternative, with the instrument that read its values.
fn measured(text: String, meter: &Option<Meter>) -> String {
    match meter {
        Some(m) => format!("{text}; {}", m.describe()),
        None => text,
    }
}

/// Sample instants `t0, t0 + dt, ...` up to `t`, and `t` itself (PK-6.3, PK-7.5: a trace
/// ends at the instant shown).
fn samples(run: &Run, t: f64, dt: f64) -> impl Iterator<Item = f64> {
    let t0 = run.config.t0;
    let end = t.min(run.end_time());
    let n = ((end - t0) / dt).floor().max(0.0) as usize;
    (0..=n).map(move |k| t0 + k as f64 * dt).filter(move |s| *s < end).chain(std::iter::once(end))
}

/// The compiled views of a presentation.
#[derive(Clone, Debug)]
pub struct Projector {
    pub views: Vec<(Id, ViewCtx, Vec<CRep>)>,
    /// Clicks on a point of a view, by view (D-060), with the event's name.
    pub clicks: Vec<(Id, prismal_ir::present::Click, String)>,
}

impl Projector {
    /// Draws the function graphs of plot view `view` over `x` (`None`: the declared range),
    /// the range a host shows after the learner's zoom and pan (D-070).
    pub fn set_sample_x(&mut self, view: &str, x: Option<(f64, f64)>) {
        for (id, ctx, _) in &mut self.views {
            if let ViewCtx::Plot { sample_x, .. } = ctx {
                if id == view {
                    *sample_x = x;
                }
            }
        }
    }
}

/// Checks a view's `on click as p request E(p)` (D-060): the event is declared `on request`
/// and the payload, read with the point clicked, has the type the event declares.
fn compile_view_click(cm: &CModel, ctx: &ViewCtx, v: &prismal_ir::present::View, c: &prismal_ir::present::Click) -> Result<String, PDiag> {
    let d = |code: &'static str, message: String| PDiag { code, message, element: v.id.clone() };
    if matches!(ctx, ViewCtx::Panel) {
        return Err(d("PK-E05", "a panel has no points to click (D-060)".into()));
    }
    let ev = cm.ir.events.iter().find(|e| e.id == c.event).ok_or_else(|| d("PK-E01", format!("unknown event `{}`", c.event)))?;
    if !matches!(ev.trigger, prismal_ir::Trigger::Request) {
        return Err(d("PK-E03", format!("`{}` is not declared `on request`, so no click can request it (D-027)", ev.name)));
    }
    match (&ev.payload, &c.payload) {
        (None, None) => {}
        (Some(p), Some(x)) => {
            compile_expr(cm, &subst(x, &ctx.pointer([0.0, 0.0])), Some(&p.ty)).map_err(|ds| d("PK-E02", ds.iter().map(|x| format!("{}: {}", x.code, x.message)).collect::<Vec<_>>().join("; ")))?;
        }
        (Some(p), None) => return Err(d("PK-E02", format!("`request {}` supplies its payload `{}`", ev.name, p.name))),
        (None, Some(_)) => return Err(d("PK-E02", format!("`{}` declares no payload", ev.name))),
    }
    Ok(ev.name.clone())
}

impl Projector {
    pub fn new(cm: &CModel, p: &Presentation) -> Result<Projector, Vec<PDiag>> {
        let mut views = vec![];
        let mut clicks = vec![];
        let mut diags = vec![];
        for v in &p.views {
            match ViewCtx::of(cm, v) {
                Ok(ctx) => {
                    if let Some(c) = &v.click {
                        match compile_view_click(cm, &ctx, v, c) {
                            Ok(name) => clicks.push((v.id.clone(), c.clone(), name)),
                            Err(d) => diags.push(d),
                        }
                    }
                    let mut reps = vec![];
                    for r in &v.representations {
                        match compile_rep(cm, &ctx, r, &p.instruments) {
                            Ok(c) => reps.push(c),
                            Err(d) => diags.extend(d),
                        }
                    }
                    views.push((v.id.clone(), ctx, reps));
                }
                Err(d) => diags.push(d),
            }
        }
        if diags.is_empty() {
            Ok(Projector { views, clicks })
        } else {
            Err(diags)
        }
    }

    pub fn ctx(&self, view: Option<&str>) -> ViewCtx {
        view.and_then(|v| self.views.iter().find(|x| x.0 == v)).map(|x| x.1.clone()).unwrap_or(ViewCtx::Panel)
    }

    /// Projects every view, plus extra representations shown by a timeline (by view, or
    /// over the presentation when the view is `None`), on one state: every view shows the
    /// same instant (PK-7.5).
    pub fn frame(&self, cm: &CModel, run: &Run, vals: &[Value], t: f64, extra: &[(Option<Id>, CRep)]) -> (Vec<ViewFrame>, Vec<RepFrame>) {
        let mut out = vec![];
        for (id, ctx, reps) in &self.views {
            let mut rs: Vec<RepFrame> = reps.iter().filter(|r| r.shown(run, vals, t)).map(|r| project(cm, ctx, r, run, vals, t)).collect();
            for (_, r) in extra.iter().filter(|(v, r)| v.as_deref() == Some(id.as_str()) && r.shown(run, vals, t)) {
                rs.push(project(cm, ctx, r, run, vals, t));
            }
            let click = self.clicks.iter().find(|c| &c.0 == id).map(|c| c.2.clone());
            out.push(ViewFrame { id: id.clone(), kind: ctx.kind(), reps: rs, click, camera: None });
        }
        let overlay = extra.iter().filter(|(v, r)| v.is_none() && r.shown(run, vals, t)).map(|(_, r)| project(cm, &ViewCtx::Panel, r, run, vals, t)).collect();
        (out, overlay)
    }
}

/// Evaluates a constant expression as a value (used for proposals shown in previews).
pub fn value_of(cm: &CModel, e: &Expr) -> Option<Value> {
    constant(cm, e).ok()
}

/// The view a representation is drawn in: `Some(view)`, `Some(None)` over the presentation
/// (a panel), or `None` when the presentation has no representation `id`. Members of groups
/// are found in their group's view.
pub fn rep_view(p: &prismal_ir::present::Presentation, id: &str) -> Option<Option<Id>> {
    use prismal_ir::present::Action;
    fn has(reps: &[Rep], id: &str) -> bool {
        reps.iter().any(|r| r.id == id || has(&r.members, id))
    }
    fn in_actions(acts: &[Action], id: &str) -> Option<Option<Id>> {
        for a in acts {
            let found = match a {
                Action::Show { view, reps } | Action::Reveal { view, reps, .. } => has(reps, id).then(|| view.clone()),
                Action::Explore { controls, fallback, .. } => if has(controls, id) { Some(None) } else { in_actions(fallback, id) },
                Action::Sequence { actions } | Action::WaitLearner { fallback: actions, .. } => in_actions(actions, id),
                _ => None,
            };
            if found.is_some() {
                return found;
            }
        }
        None
    }
    if let Some(v) = p.views.iter().find(|v| has(&v.representations, id)) {
        return Some(Some(v.id.clone()));
    }
    p.timeline.iter().flat_map(|t| t.scenes.iter().flat_map(|s| &s.beats)).find_map(|b| in_actions(&b.actions, id))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rounding_near_zero_is_shown_as_zero() {
        let ground = ViewCtx::Spatial { space: "S".into(), px_per_m: 50.0, y_up: true };
        assert_eq!(ground.shown(&[0.0, -3.74851e-10]), vec![0.0, 0.0]);
        assert_eq!(ground.shown(&[2.0, 0.5]), vec![2.0, 0.5]);
        // Atoms drawn at 1e11 px/m keep a length of 1e-10 m.
        let atoms = ViewCtx::Spatial { space: "S".into(), px_per_m: 1e11, y_up: true };
        assert_eq!(atoms.shown(&[1e-10, 0.0]), vec![1e-10, 0.0]);
        let plot = ViewCtx::Plot { x: (0.0, 10.0), y: (-1.0, 1.0), dims: (Dim::default(), Dim::default()), follow: [false; 2], units: [None, None], window: None, sample_x: None };
        assert_eq!(plot.shown(&[1e-12, 0.25]), vec![0.0, 0.25]);
    }
}

/// The pieces of a `text` template between its `{}` or `{:.N}` (literal braces written
/// doubled), and the decimals of each value (D-076, D-078).
pub fn split_template(t: &str) -> (Vec<String>, Vec<Option<usize>>) {
    let (mut pieces, mut places, mut cur) = (vec![], vec![], String::new());
    let mut chars = t.chars().peekable();
    while let Some(c) = chars.next() {
        match (c, chars.peek()) {
            ('{', Some('{')) | ('}', Some('}')) => {
                chars.next();
                cur.push(c);
            }
            ('{', Some('}')) => {
                chars.next();
                pieces.push(std::mem::take(&mut cur));
                places.push(None);
            }
            ('{', Some(':')) => {
                let spec: String = chars.by_ref().take_while(|c| *c != '}').collect();
                pieces.push(std::mem::take(&mut cur));
                places.push(spec.trim_start_matches(":.").parse().ok());
            }
            _ => cur.push(c),
        }
    }
    pieces.push(cur);
    (pieces, places)
}

