//! Viewports, targeting and focus for raw input (HI section 4.5, D-047).
//!
//! The host captures input in its own layer and forwards it: pointer positions in pixels
//! of a view as the host drew it, wheel steps, key names. This module holds what the engine
//! needs to interpret them the same way for every host: the box of view coordinates each
//! view shows (its viewport, also given in frames so that every renderer draws with it),
//! the mapping between drawn pixels and view coordinates, hit testing of draggable parts
//! (PK-10.2) and the order in which representations take keyboard focus (PK-11.2).

use prismal_present::frame::{Camera, Frame, RepFrame, Shape, ViewFrame};

/// Margin of a plot's frame inside its drawn box, in pixels.
pub const PLOT_MARGIN: f64 = 44.0;
/// The size a plot is drawn at when the host has not said otherwise.
pub const PLOT_SIZE: [f64; 2] = [560.0, 347.0];
/// Room around a spatial view's content, in view pixels, and its least size.
const PAD: f64 = 40.0;
const MIN_SIZE: [f64; 2] = [320.0, 220.0];
/// Room kept around what a session shows when its framing grows to fit it.
const FIT_PAD: f64 = 30.0;

/// Radii of the drawn marks that can be grabbed, in pixels (the reference renderers').
const MARKER_R: f64 = 8.0;
const HANDLE_R: f64 = 7.0;
const STROKE_R: f64 = 3.0;

/// The coordinate system of one view of an open presentation, and the learner's own zoom
/// and pan of it.
#[derive(Clone, Debug)]
pub struct ViewState {
    pub id: String,
    pub name: String,
    pub kind: ViewKind,
    /// The box the learner's zoom and pan left (`None`: the view's own framing). For a plot
    /// it is in plot coordinates, `[x0, y0, width, height]` with `y` upwards.
    pub user: Option<[f64; 4]>,
    /// The size in pixels the host last said it draws the view at.
    pub size: Option<[f64; 2]>,
    /// The azimuth and elevation, in radians, the learner turned a view of three dimensions
    /// to (D-104); `None`: as its author aims it.
    pub turn: Option<[f64; 2]>,
}

#[derive(Clone, Debug)]
pub enum ViewKind {
    /// `base` is the framing of the content over the run: its extent with room around it.
    Spatial { base: [f64; 4] },
    /// `follow`: the axes whose range grows to keep what the view draws in view (D-070).
    /// `window`: the `x` axis shows the latest span of this length once the data passes it.
    Plot { x: (f64, f64), y: (f64, f64), follow: [bool; 2], window: Option<f64> },
    Panel,
}

impl ViewState {
    pub fn spatial(id: &str, name: &str, extent: [f64; 4]) -> ViewState {
        let [x0, y0, x1, y1] = extent;
        let w = (x1 - x0 + 2.0 * PAD).max(MIN_SIZE[0]);
        let h = (y1 - y0 + 2.0 * PAD).max(MIN_SIZE[1]);
        let base = [(x0 + x1) / 2.0 - w / 2.0, (y0 + y1) / 2.0 - h / 2.0, w, h];
        ViewState { id: id.into(), name: name.into(), kind: ViewKind::Spatial { base }, user: None, size: None, turn: None }
    }

    /// What the view shows in a frame (`grow`: a session keeps its content in view).
    pub fn viewport(&self, vf: &ViewFrame, grow: bool) -> Option<Viewport> {
        match &self.kind {
            ViewKind::Spatial { base } => {
                let own = if grow { grow_to_fit(*base, &vf.reps) } else { *base };
                let shown = self.user.unwrap_or(own);
                Some(Viewport::Spatial { r#box: camera_box(*base, shown, vf.camera.as_ref()), shown, size: [own[2], own[3]] })
            }
            ViewKind::Plot { x, y, follow, window } => {
                let (mut x, y) = if follow[0] || follow[1] { follow_data((*x, *y), *follow, &vf.reps) } else { (*x, *y) };
                if let Some(w) = window {
                    x = slide(x, *w, &vf.reps);
                }
                // A lesson's camera moves and zooms a plot as it does a spatial view (D-075).
                let base = [x.0, y.0, x.1 - x.0, y.1 - y.0];
                let [bx, by, bw, bh] = camera_box(base, self.user.unwrap_or(base), vf.camera.as_ref());
                Some(Viewport::Plot { x: (bx, bx + bw), y: (by, by + bh) })
            }
            ViewKind::Panel => None,
        }
    }
}

/// The part of view coordinates a view shows, and how it is drawn.
#[derive(Clone, Debug)]
pub enum Viewport {
    /// `box` (x, y, width, height in view coordinates) is fitted into the drawn area,
    /// uniformly scaled and centred. `shown` is the framing before a timeline camera,
    /// which zoom and pan change; `size` is the natural drawn size in pixels, one pixel per
    /// view unit.
    Spatial { r#box: [f64; 4], shown: [f64; 4], size: [f64; 2] },
    /// The plot's ranges fill the drawn area less a margin of `PLOT_MARGIN` on every side,
    /// `y` upwards.
    Plot { x: (f64, f64), y: (f64, f64) },
}

impl Viewport {
    /// The mapping between pixels of the view drawn `size` large and view coordinates.
    pub fn map(&self, size: [f64; 2]) -> Map {
        match *self {
            Viewport::Spatial { r#box: b, .. } => {
                let s = (size[0] / b[2]).min(size[1] / b[3]);
                Map::Fit { b, s, off: [(size[0] - b[2] * s) / 2.0, (size[1] - b[3] * s) / 2.0] }
            }
            Viewport::Plot { x, y } => Map::Plot { x, y, size },
        }
    }

    /// The framing zoom and pan start from, `[x, y, width, height]` in view coordinates.
    pub fn shown(&self) -> Option<[f64; 4]> {
        match *self {
            Viewport::Spatial { shown, .. } => Some(shown),
            Viewport::Plot { x, y } => Some([x.0, y.0, x.1 - x.0, y.1 - y.0]),
        }
    }

    pub fn natural_size(&self) -> [f64; 2] {
        match self {
            Viewport::Spatial { size, .. } => *size,
            Viewport::Plot { .. } => PLOT_SIZE,
        }
    }
}

/// Pixels of a drawn view to view coordinates and back.
#[derive(Clone, Copy, Debug)]
pub enum Map {
    Fit { b: [f64; 4], s: f64, off: [f64; 2] },
    Plot { x: (f64, f64), y: (f64, f64), size: [f64; 2] },
}

impl Map {
    pub fn to_view(&self, p: [f64; 2]) -> [f64; 2] {
        match *self {
            Map::Fit { b, s, off } => [b[0] + (p[0] - off[0]) / s, b[1] + (p[1] - off[1]) / s],
            Map::Plot { x, y, size: [w, h] } => {
                let m = PLOT_MARGIN;
                [x.0 + (p[0] - m) * (x.1 - x.0) / (w - 2.0 * m), y.0 + (h - m - p[1]) * (y.1 - y.0) / (h - 2.0 * m)]
            }
        }
    }

    pub fn to_px(&self, v: [f64; 2]) -> [f64; 2] {
        match *self {
            Map::Fit { b, s, off } => [off[0] + (v[0] - b[0]) * s, off[1] + (v[1] - b[1]) * s],
            Map::Plot { x, y, size: [w, h] } => {
                let m = PLOT_MARGIN;
                [m + (v[0] - x.0) * (w - 2.0 * m) / (x.1 - x.0), h - m - (v[1] - y.0) * (h - 2.0 * m) / (y.1 - y.0)]
            }
        }
    }

    /// Pixels per view unit of a spatial view.
    pub fn scale(&self) -> f64 {
        match *self {
            Map::Fit { s, .. } => s,
            Map::Plot { .. } => 1.0,
        }
    }
}

/// The framing grown to keep every point, arrow and segment at least `FIT_PAD` inside it.
fn grow_to_fit(vb: [f64; 4], reps: &[RepFrame]) -> [f64; 4] {
    fn collect(reps: &[RepFrame], pts: &mut Vec<[f64; 2]>) {
        for r in reps {
            match &r.shape {
                Shape::Point { at } => pts.push(*at),
                Shape::Arrow { from, to } | Shape::Segment { from, to } => pts.extend([*from, *to]),
                Shape::Group { members } => collect(members, pts),
                _ => {}
            }
        }
    }
    let mut pts = vec![];
    collect(reps, &mut pts);
    let [mut x, mut y, mut w, mut h] = vb;
    for [px, py] in pts.into_iter().filter(|p| p[0].is_finite() && p[1].is_finite()) {
        if px - FIT_PAD < x {
            w += x - (px - FIT_PAD);
            x = px - FIT_PAD;
        }
        if py - FIT_PAD < y {
            h += y - (py - FIT_PAD);
            y = py - FIT_PAD;
        }
        w = w.max(px + FIT_PAD - x);
        h = h.max(py + FIT_PAD - y);
    }
    [x, y, w, h]
}

/// A plot's ranges grown on the axes that follow the data (D-070) to hold every point, segment,
/// polyline and polygon the view draws, with a twentieth of the span as room on the side it
/// grew. The declared range is always kept: a curve that shrinks back leaves the axis where
/// it was declared.
fn follow_data((x, y): ((f64, f64), (f64, f64)), follow: [bool; 2], reps: &[RepFrame]) -> ((f64, f64), (f64, f64)) {
    fn collect(reps: &[RepFrame], pts: &mut Vec<[f64; 2]>) {
        for r in reps {
            match &r.shape {
                Shape::Point { at } => pts.push(*at),
                Shape::Arrow { from, to } | Shape::Segment { from, to } => pts.extend([*from, *to]),
                Shape::Polyline { points } | Shape::Polygon { points } => pts.extend(points.iter().copied()),
                Shape::Group { members } => collect(members, pts),
                _ => {}
            }
        }
    }
    let mut pts = vec![];
    collect(reps, &mut pts);
    let grow = |(lo, hi): (f64, f64), vals: &mut dyn Iterator<Item = f64>| {
        let (mut a, mut b) = (lo, hi);
        for v in vals.filter(|v| v.is_finite()) {
            a = a.min(v);
            b = b.max(v);
        }
        let room = (b - a) / 20.0;
        (if a < lo { a - room } else { lo }, if b > hi { b + room } else { hi })
    };
    let x = if follow[0] { grow(x, &mut pts.iter().map(|p| p[0])) } else { x };
    let y = if follow[1] { grow(y, &mut pts.iter().map(|p| p[1])) } else { y };
    (x, y)
}

/// A plot's `x` range as a window of length `w` (D-070): the declared range while the data
/// stays inside it, then the latest `w` of the data, with a twentieth of `w` as room ahead.
fn slide(x: (f64, f64), w: f64, reps: &[RepFrame]) -> (f64, f64) {
    fn last(reps: &[RepFrame], m: &mut f64) {
        for r in reps {
            match &r.shape {
                Shape::Point { at } => *m = m.max(at[0]),
                Shape::Arrow { from, to } | Shape::Segment { from, to } => *m = m.max(from[0]).max(to[0]),
                Shape::Polyline { points } | Shape::Polygon { points } => points.iter().for_each(|p| *m = m.max(p[0])),
                Shape::Group { members } => last(members, m),
                _ => {}
            }
        }
    }
    let mut m = f64::NEG_INFINITY;
    last(reps, &mut m);
    if !m.is_finite() || m <= x.1 {
        return x;
    }
    let hi = m + w / 20.0;
    (hi - w, hi)
}

/// The box a timeline camera shows (D-042): from the centre of the base framing to the
/// camera's centre, `blend` of the way, at its zoom. Without a camera, `shown`.
fn camera_box(base: [f64; 4], shown: [f64; 4], cam: Option<&Camera>) -> [f64; 4] {
    let Some(cam) = cam else { return shown };
    let [x, y, w, h] = base;
    let c0 = [x + w / 2.0, y + h / 2.0];
    let c = match cam.center {
        Some(t) => [c0[0] + (t[0] - c0[0]) * cam.blend, c0[1] + (t[1] - c0[1]) * cam.blend],
        None => c0,
    };
    let (cw, ch) = (w / cam.zoom, h / cam.zoom);
    [c[0] - cw / 2.0, c[1] - ch / 2.0, cw, ch]
}

// ---------------------------------------------------------------- targeting

/// A representation and the part of it a gesture targets (`None`: its body), and whether it
/// can be dragged and clicked (D-059).
#[derive(Clone, Debug, PartialEq)]
pub struct Target {
    pub rep: String,
    pub part: Option<String>,
    pub drag: bool,
    pub click: bool,
}

fn dist_to_segment(p: [f64; 2], a: [f64; 2], b: [f64; 2]) -> f64 {
    let (dx, dy) = (b[0] - a[0], b[1] - a[1]);
    let l2 = dx * dx + dy * dy;
    let t = if l2 > 0.0 { (((p[0] - a[0]) * dx + (p[1] - a[1]) * dy) / l2).clamp(0.0, 1.0) } else { 0.0 };
    (p[0] - a[0] - t * dx).hypot(p[1] - a[1] - t * dy)
}

fn inside(p: [f64; 2], poly: &[[f64; 2]]) -> bool {
    let mut c = false;
    let n = poly.len();
    for i in 0..n {
        let (a, b) = (poly[i], poly[(i + n - 1) % n]);
        if (a[1] > p[1]) != (b[1] > p[1]) && p[0] < (b[0] - a[0]) * (p[1] - a[1]) / (b[1] - a[1]) + a[0] {
            c = !c;
        }
    }
    c
}

/// The draggable or clickable part under pixel `p` of a view drawn with `map` (PK-10.2,
/// D-059): the topmost such representation whose grabbed part lies within its drawn radius
/// plus `tolerance` pixels. Draggable representations are drawn over the others, later ones
/// on top; invisible ones are not targets. A click takes the body.
pub fn hit(vf: &ViewFrame, map: &Map, p: [f64; 2], tolerance: f64) -> Option<Target> {
    hit_in(&vf.reps, map, p, tolerance)
}

/// Targeting among representations, the members of groups included (D-043): a group's
/// members are placed in the frame already, and the later ones are on top.
fn hit_in(reps: &[RepFrame], map: &Map, p: [f64; 2], tolerance: f64) -> Option<Target> {
    let px = |v: &[f64; 2]| map.to_px(*v);
    let near = |d: f64, r: f64| d <= r + tolerance;
    for r in reps.iter().rev() {
        if let Shape::Group { members } = &r.shape {
            if r.opacity.is_some_and(|o| o < 0.05) {
                continue;
            }
            if let Some(t) = hit_in(members, map, p, tolerance) {
                return Some(t);
            }
            continue;
        }
        let part = match (&r.drag, &r.click) {
            (Some(part), _) => part.clone(),
            (None, Some(_)) => "body".to_string(),
            (None, None) => continue,
        };
        if r.opacity.is_some_and(|o| o < 0.05) {
            continue;
        }
        let body = part == "body";
        let target = || Target { rep: r.id.clone(), part: if body { None } else { Some(part.clone()) }, drag: r.drag.is_some(), click: r.click.is_some() };
        let hit = match (&r.shape, body) {
            (Shape::Point { at }, _) => near(dist(px(at), p), MARKER_R),
            (Shape::Arrow { to, .. }, false) => near(dist(px(to), p), HANDLE_R),
            (Shape::Arrow { from, to } | Shape::Segment { from, to }, true) => near(dist_to_segment(p, px(from), px(to)), STROKE_R),
            (Shape::Polyline { points }, true) => points.windows(2).any(|w| near(dist_to_segment(p, px(&w[0]), px(&w[1])), STROKE_R)),
            (Shape::Ellipse { closed, .. }, true) => {
                let pts: Vec<[f64; 2]> = r.shape.curve_points(64).iter().map(px).collect();
                (*closed && inside(p, &pts)) || pts.windows(2).any(|w| near(dist_to_segment(p, w[0], w[1]), STROKE_R))
            }
            (Shape::Polygon { points }, true) => {
                let pts: Vec<[f64; 2]> = points.iter().map(px).collect();
                inside(p, &pts) || pts.iter().zip(pts.iter().cycle().skip(1)).any(|(a, b)| near(dist_to_segment(p, *a, *b), STROKE_R))
            }
            _ => false,
        };
        if hit {
            return Some(target());
        }
    }
    None
}

pub fn dist(a: [f64; 2], b: [f64; 2]) -> f64 {
    (a[0] - b[0]).hypot(a[1] - b[1])
}

/// Extra reach of a pointer, in pixels, by kind: a finger covers more than a cursor.
pub fn tolerance(pointer: &str) -> f64 {
    match pointer {
        "touch" => 16.0,
        "pen" => 8.0,
        _ => 4.0,
    }
}

// ---------------------------------------------------------------- focus

/// The representations that take keyboard focus, in order (PK-11.2): in each view in turn,
/// the draggable and clickable ones, controls and buttons, in the order the presentation declares them;
/// then those over the presentation.
pub fn focus_order(frame: &Frame) -> Vec<&RepFrame> {
    fn walk<'a>(r: &'a RepFrame, out: &mut Vec<&'a RepFrame>) {
        if let Shape::Group { members } = &r.shape {
            members.iter().for_each(|m| walk(m, out));
        } else if r.drag.is_some() || r.click.is_some() || matches!(r.shape, Shape::Control { .. } | Shape::Button { .. }) {
            out.push(r);
        }
    }
    let mut out = vec![];
    frame.views.iter().flat_map(|v| v.reps.iter()).chain(frame.overlay.iter()).for_each(|r| walk(r, &mut out));
    out
}
