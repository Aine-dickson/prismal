//! Prismal SVG renderer: draws frame descriptions (PK-12.1) as standalone SVG documents,
//! with no browser. It is the second renderer after the web player (D-018) and shows that
//! rendering is swappable (PK-12.2): it reads only what the host interface gives every
//! host, a presentation's layout and its frame descriptions as JSON (HI section 4), and it
//! defines no representation semantics.
//!
//! One frame gives one document: still images and vector documents, and with a frame per
//! instant (a lesson's export, a session's run) an image sequence for video. Formulas are
//! drawn from their layouts (D-046), so no math engine is needed.
//!
//! Each view is drawn with the viewport the frame gives (D-047), as every renderer draws it.
//! The drawing follows the web player (`web/player.js`): the same scales, ticks, marks,
//! colors and sizes, so a frame drawn by either renderer shows the same geometry in
//! the same view coordinates. Where the browser lays out HTML (panels, controls, tables,
//! captions), this renderer stacks the same items as SVG. Colors are written as presentation
//! attributes, not style sheets, so that any SVG consumer draws them.

use serde_json::Value as Json;
use std::fmt::Write;

// ---------------------------------------------------------------- options

/// The author's named colors (D-061), as frames carry them (the kernel's
/// `prismal_present::frame::COLORS`).
pub const COLORS: [&str; 10] = ["red", "orange", "yellow", "green", "teal", "blue", "purple", "pink", "gray", "ink"];

/// Colors of a rendering, as in the web player's style sheet.
#[derive(Clone, Debug)]
pub struct Theme {
    pub bg: &'static str,
    pub surface: &'static str,
    pub ink: &'static str,
    pub muted: &'static str,
    pub line: &'static str,
    pub faint: &'static str,
    pub accent: &'static str,
    pub accent_ink: &'static str,
    /// Arrow colors, taken in turn by the arrows of a view.
    pub reps: [&'static str; 4],
    pub bad: &'static str,
    pub hl: &'static str,
    /// Values of the author's named colors (D-061), in the order of
    /// `COLORS`.
    pub palette: [&'static str; 10],
}

impl Theme {
    pub const LIGHT: Theme = Theme {
        bg: "#f6f6f3",
        surface: "#ffffff",
        ink: "#1d1f24",
        muted: "#5d6270",
        line: "#d9dad4",
        faint: "#ecece7",
        accent: "#2458c6",
        accent_ink: "#ffffff",
        reps: ["#1d1f24", "#b3471d", "#1f7a4d", "#7a3fb0"],
        bad: "#b3261e",
        hl: "#e0a800",
        palette: ["#c62828", "#d9730d", "#b58900", "#2e7d32", "#00897b", "#2458c6", "#7a3fb0", "#c2185b", "#6b7080", "#1d1f24"],
    };
    pub const DARK: Theme = Theme {
        bg: "#15171b",
        surface: "#1d2026",
        ink: "#e7e8eb",
        muted: "#9ea3ae",
        line: "#353a44",
        faint: "#262a31",
        accent: "#7ea6ff",
        accent_ink: "#0d1120",
        reps: ["#e7e8eb", "#ff9d6e", "#6fd3a0", "#c9a2ff"],
        bad: "#ff8a80",
        hl: "#ffd24d",
        palette: ["#ff8a80", "#ffb070", "#ffd54f", "#81c784", "#4dd0c4", "#7ea6ff", "#c9a2ff", "#f48fb1", "#a0a5b0", "#e7e8eb"],
    };

    /// The value of an author's named color (D-061).
    pub fn color(&self, name: &str) -> Option<&'static str> {
        COLORS.iter().position(|c| *c == name).map(|i| self.palette[i])
    }
}

/// How frames are drawn.
#[derive(Clone, Debug)]
pub struct Options {
    /// Width of a plot view in pixels; its height is `min(0.62 width, 420)`, as in the web
    /// player.
    pub plot_width: f64,
    /// A spatial view is drawn one view pixel to one document pixel, scaled down to at most
    /// this width.
    pub max_view_width: f64,
    /// Size of one em of a formula layout, in pixels.
    pub math_em: f64,
    /// A line above the views naming the presentation, the run and the instant.
    pub header: bool,
    pub theme: Theme,
}

impl Default for Options {
    fn default() -> Options {
        Options { plot_width: 560.0, max_view_width: 960.0, math_em: 18.0, header: true, theme: Theme::LIGHT }
    }
}

// ---------------------------------------------------------------- entry point

/// Draws the frame description `frame` of the presentation whose layout is `layout`
/// (both as the host interface gives them) as a standalone SVG document.
pub fn render(layout: &Json, frame: &Json, opts: &Options) -> String {
    let th = &opts.theme;
    let mut blocks: Vec<Block> = vec![];
    if opts.header {
        let mut h = format!("{}, {} run, t = {} s", s(&layout["presentation"]), s(&frame["run"]), fmt(f(&frame["t"])));
        if layout["mode"] == "lesson" {
            let _ = write!(h, ", presentation time {} s", fmt(f(&frame["time"])));
        }
        blocks.push(text_block(&h, 12.0, false, th.muted));
    }
    // The presentation's title (D-076).
    if let Some(t) = layout["title"].as_str() {
        blocks.push(text_block(t, 18.0, false, th.ink));
    }
    let lesson = layout["mode"] == "lesson";
    // In a lesson, controls act only in explore beats (PK-9.8, D-025).
    let explore = !lesson || arr(&layout["lesson"]["explore"]).iter().any(|x| f(&frame["time"]) >= f(&x["start"]) && f(&frame["time"]) < f(&x["end"]));
    let mut clip = 0;
    let mut drawn: Vec<(String, Block)> = vec![];
    for vf in arr(&frame["views"]) {
        let Some(vl) = arr(&layout["views"]).iter().find(|v| v["id"] == vf["id"]) else { continue };
        clip += 1;
        drawn.push((s(&vf["id"]).to_string(), view_block(vl, vf, clip, opts)));
    }
    // The author's page layout places views in rows and columns (PK-7.4a, D-063); without
    // one, views are stacked in order.
    if let Some(b) = page(&layout["page"], &mut drawn) {
        blocks.push(b);
    }
    blocks.extend(drawn.into_iter().map(|(_, b)| b));
    let overlay = arr(&frame["overlay"]);
    if !overlay.is_empty() {
        let items: Vec<Block> = overlay.iter().map(|r| item(r, !explore, opts)).collect();
        blocks.push(framed(stack(items, 8.0), 12.0, 8.0, th));
    }
    let caps = arr(&frame["captions"]);
    if !caps.is_empty() {
        blocks.push(stack(caps.iter().map(|c| caption(s(c), th)).collect(), 6.0));
    }

    let pad = 16.0;
    let body = stack(blocks, 12.0);
    let (w, h) = (body.w + 2.0 * pad, body.h + 2.0 * pad);
    let mut out = String::new();
    let title = format!("{}: {} run, simulation time {} s", s(&layout["presentation"]), s(&frame["run"]), fmt(f(&frame["t"])));
    let desc: Vec<String> = all_reps(frame).into_iter().filter(|r| r["kind"] != "axes" && r["kind"] != "grid").map(|r| s(&r["text"]).to_string()).collect();
    let _ = writeln!(
        out,
        "<svg xmlns=\"http://www.w3.org/2000/svg\" width=\"{w}\" height=\"{h}\" viewBox=\"0 0 {w} {h}\" role=\"img\" aria-labelledby=\"title desc\">\
<title id=\"title\">{}</title><desc id=\"desc\">{}</desc>\
<defs><filter id=\"highlight\" x=\"-50%\" y=\"-50%\" width=\"200%\" height=\"200%\"><feDropShadow dx=\"0\" dy=\"0\" stdDeviation=\"3\" flood-color=\"{}\"/></filter></defs>\
<rect width=\"{w}\" height=\"{h}\" fill=\"{}\"/><g transform=\"translate({pad} {pad})\">{}</g></svg>",
        esc(&title),
        esc(&desc.join("\n")),
        th.hl,
        th.bg,
        body.body,
        w = n(w),
        h = n(h),
        pad = n(pad),
    );
    out
}

/// Every representation of a frame, views first then the overlay, a group before its
/// members.
pub fn all_reps(frame: &Json) -> Vec<&Json> {
    fn walk<'a>(r: &'a Json, out: &mut Vec<&'a Json>) {
        out.push(r);
        for m in arr(&r["members"]) {
            walk(m, out);
        }
    }
    let mut out = vec![];
    for v in arr(&frame["views"]) {
        for r in arr(&v["reps"]) {
            walk(r, &mut out);
        }
    }
    for r in arr(&frame["overlay"]) {
        walk(r, &mut out);
    }
    out
}

// ---------------------------------------------------------------- page layout

/// A drawn block with its top left corner at the origin.
struct Block {
    w: f64,
    h: f64,
    body: String,
}

/// Blocks one below the other, `gap` apart.
fn stack(blocks: Vec<Block>, gap: f64) -> Block {
    let mut y = 0.0;
    let mut w: f64 = 0.0;
    let mut body = String::new();
    for (i, b) in blocks.into_iter().enumerate() {
        if i > 0 {
            y += gap;
        }
        let _ = write!(body, "<g transform=\"translate(0 {})\">{}</g>", n(y), b.body);
        y += b.h;
        w = w.max(b.w);
    }
    Block { w, h: y, body }
}

/// Blocks side by side, `gap` apart, aligned at the top.
fn row(blocks: Vec<Block>, gap: f64) -> Block {
    let mut x = 0.0;
    let mut h: f64 = 0.0;
    let mut body = String::new();
    for (i, b) in blocks.into_iter().enumerate() {
        if i > 0 {
            x += gap;
        }
        let _ = write!(body, "<g transform=\"translate({} 0)\">{}</g>", n(x), b.body);
        x += b.w;
        h = h.max(b.h);
    }
    Block { w: x, h, body }
}

/// The views of a page layout node, taken from `drawn` as they are placed; `None` for an
/// absent node or one that places nothing drawn.
fn page(node: &Json, drawn: &mut Vec<(String, Block)>) -> Option<Block> {
    let gap = 12.0;
    let parts = |xs: &Json, drawn: &mut Vec<(String, Block)>| arr(xs).iter().filter_map(|x| page(x, drawn)).collect::<Vec<_>>();
    if let Some(id) = node["view"].as_str() {
        let k = drawn.iter().position(|(v, _)| v == id)?;
        return Some(drawn.remove(k).1);
    }
    let blocks = if node["row"].is_array() {
        row(parts(&node["row"], drawn), gap)
    } else if node["column"].is_array() {
        stack(parts(&node["column"], drawn), gap)
    } else {
        return None;
    };
    (!blocks.body.is_empty()).then_some(blocks)
}

/// A block inside a surface with a border, `pad` around it.
fn framed(b: Block, pad_x: f64, pad_y: f64, th: &Theme) -> Block {
    let (w, h) = (b.w + 2.0 * pad_x, b.h + 2.0 * pad_y);
    let body = format!(
        "<rect x=\"0.5\" y=\"0.5\" width=\"{}\" height=\"{}\" rx=\"8\" fill=\"{}\" stroke=\"{}\"/><g transform=\"translate({} {})\">{}</g>",
        n(w - 1.0),
        n(h - 1.0),
        th.surface,
        th.line,
        n(pad_x),
        n(pad_y),
        b.body
    );
    Block { w, h, body }
}

fn text_block(t: &str, size: f64, mono: bool, color: &str) -> Block {
    Block { w: text_w(t, size, mono), h: size * 1.4, body: text(0.0, size * 1.05, t, size, mono, color, "") }
}

fn caption(t: &str, th: &Theme) -> Block {
    let size = 17.0;
    let w = text_w(t, size, false) + 20.0;
    let body = format!(
        "<rect width=\"{}\" height=\"{}\" rx=\"4\" fill=\"{}\"/>{}",
        n(w),
        n(size * 1.6),
        th.ink,
        text(10.0, size * 1.15, t, size, false, th.bg, "")
    );
    Block { w, h: size * 1.6, body }
}

// ---------------------------------------------------------------- views

/// The mapping from a view's coordinates to the coordinates it is drawn in.
enum Map {
    /// A spatial view is drawn in its own view coordinates.
    Spatial,
    /// A plot view is drawn in a `w` by `h` box with margin `m`.
    Plot { x: (f64, f64), y: (f64, f64), w: f64, h: f64, m: f64 },
}

impl Map {
    fn to(&self, p: [f64; 2]) -> [f64; 2] {
        match *self {
            Map::Spatial => p,
            Map::Plot { x, y, w, h, m } => {
                let sx = (w - 2.0 * m) / (x.1 - x.0);
                let sy = (h - 2.0 * m) / (y.1 - y.0);
                [m + (p[0] - x.0) * sx, h - m - (p[1] - y.0) * sy]
            }
        }
    }
}

const DRAWN: [&str; 11] = ["lines", "point", "arrow", "segment", "polyline", "polygon", "ellipse", "group", "note", "raster", "ray"];

fn view_block(vl: &Json, vf: &Json, clip: usize, opts: &Options) -> Block {
    let th = &opts.theme;
    let kind = s(&vl["kind"]);
    let reps = arr(&vf["reps"]);
    // The author's `title(...)`, or the view's name and kind (D-076). Reusing the frame
    // representation preserves its identity and text alternative in the SVG.
    let caption = match (vl["title"].as_str(), reps.iter().find(|r| r["shape"] == "title")) {
        (Some(_), Some(r)) => item(r, false, opts),
        (Some(t), None) => text_block(t, 13.0, false, th.ink),
        (None, _) => text_block(&format!("{} ({})", s(&vl["name"]), kind), 12.0, false, th.muted),
    };
    let mut parts: Vec<Block> = vec![caption];
    let mut panel: Vec<Block> = vec![];
    if kind == "spatial" || kind == "plot" {
        // The engine gives the box of view coordinates the view shows (its framing, the
        // learner's zoom and pan, a timeline camera) and its natural size (D-047).
        let bx: Vec<f64> = arr(&vf["box"]).iter().map(f).collect();
        let bx = if bx.len() == 4 { [bx[0], bx[1], bx[2], bx[3]] } else { [0.0, 0.0, 320.0, 220.0] };
        let (map, dw, dh, view_box, u) = if kind == "spatial" {
            let size = pt(&vf["size"]);
            let size = if size[0] > 0.0 && size[1] > 0.0 { size } else { [bx[2], bx[3]] };
            let dw = size[0].min(opts.max_view_width);
            let dh = size[1] * dw / size[0];
            (Map::Spatial, dw, dh, bx, (bx[2] / dw).max(bx[3] / dh))
        } else {
            let w = opts.plot_width;
            let h = (w * 0.62).min(420.0).round();
            let m = vf["margin"].as_f64().unwrap_or(44.0);
            (Map::Plot { x: (bx[0], bx[0] + bx[2]), y: (bx[1], bx[1] + bx[3]), w, h, m }, w, h, [0.0, 0.0, w, h], 1.0)
        };
        let mut body = String::new();
        let _ = write!(
            body,
            "<svg x=\"0\" y=\"0\" width=\"{}\" height=\"{}\" viewBox=\"{} {} {} {}\" overflow=\"hidden\" role=\"group\" aria-label=\"{}\" data-view=\"{}\">",
            n(dw),
            n(dh),
            n(view_box[0]),
            n(view_box[1]),
            n(view_box[2]),
            n(view_box[3]),
            esc(&format!("{}: {}", s(&vl["name"]), if kind == "plot" { "plot" } else { "spatial view" })),
            esc(s(&vl["id"]))
        );
        body += "<g>";
        match &map {
            Map::Spatial => spatial_chrome(&mut body, vl, reps, view_box, u, th),
            Map::Plot { .. } => plot_chrome(&mut body, vl, &map, th),
        }
        body += "</g>";
        let mut drawn = String::new();
        let mut top = String::new();
        let mut arrows = 0;
        for r in reps {
            let shape = s(&r["shape"]);
            if DRAWN.contains(&shape) {
                let ci = if shape == "arrow" {
                    arrows += 1;
                    arrows - 1
                } else {
                    0
                };
                // Draggable representations are drawn last and outside the plot's clip.
                let target = if r["drag"].is_string() { &mut top } else { &mut drawn };
                draw_rep(target, &map, r, u, ci, th);
            } else if shape != "axes" && shape != "grid" && shape != "title" {
                panel.push(item(r, false, opts));
            }
        }
        if let Map::Plot { w, h, m, .. } = map {
            let _ = write!(
                body,
                "<clipPath id=\"clip-{clip}\"><rect x=\"{m}\" y=\"{m}\" width=\"{}\" height=\"{}\"/></clipPath><g clip-path=\"url(#clip-{clip})\">{drawn}</g>",
                n(w - 2.0 * m),
                n(h - 2.0 * m),
                m = n(m)
            );
        } else {
            let _ = write!(body, "<g>{drawn}</g>");
        }
        let _ = write!(body, "<g>{top}</g></svg>");
        parts.push(Block { w: dw, h: dh, body });
    } else {
        for r in reps.iter().filter(|r| r["shape"] != "title") {
            panel.push(item(r, false, opts));
        }
    }
    if !panel.is_empty() {
        parts.push(stack(panel, 8.0));
    }
    framed(stack(parts, 6.0), 8.0, 8.0, th)
}

fn spatial_chrome(out: &mut String, vl: &Json, reps: &[Json], bx: [f64; 4], u: f64, th: &Theme) {
    let [x, y, w, h] = bx;
    let ppm = f(&vl["px_per_m"]);
    let step = nice_step((60.0 * u) / ppm) * ppm;
    let metres = step / ppm;
    // In a view of three dimensions the axes and grid come drawn as projected lines (D-100).
    let has = |k: &str| reps.iter().any(|r| r["shape"] == k);
    let line = |out: &mut String, cls: &str, a: [f64; 2], b: [f64; 2]| {
        let color = if cls == "axis" { th.muted } else { th.faint };
        let _ = write!(
            out,
            "<line class=\"{cls}\" x1=\"{}\" y1=\"{}\" x2=\"{}\" y2=\"{}\" stroke=\"{color}\" stroke-width=\"{}\"/>",
            n(a[0]),
            n(a[1]),
            n(b[0]),
            n(b[1]),
            n(u)
        );
    };
    let ticks = |from: f64, len: f64| {
        let mut v = vec![];
        let mut g = (from / step).ceil() * step;
        while g <= from + len {
            v.push(g);
            g += step;
        }
        v
    };
    if has("grid") {
        for gx in ticks(x, w) {
            line(out, "gridline", [gx, y], [gx, y + h]);
        }
        for gy in ticks(y, h) {
            line(out, "gridline", [x, gy], [x + w, gy]);
        }
    }
    if has("axes") {
        line(out, "axis", [x, 0.0], [x + w, 0.0]);
        line(out, "axis", [0.0, y], [0.0, y + h]);
        let fs = 10.0 * u;
        for gx in ticks(x, w) {
            if gx.abs() < 1e-9 {
                continue;
            }
            line(out, "axis", [gx, -3.0 * u], [gx, 3.0 * u]);
            *out += &text(gx, 14.0 * u, &fmt(gx / ppm), fs, false, th.muted, " text-anchor=\"middle\" class=\"tick\"");
        }
        let up = if vl["y_up"] == true { -1.0 } else { 1.0 };
        for gy in ticks(y, h) {
            if gy.abs() < 1e-9 {
                continue;
            }
            line(out, "axis", [-3.0 * u, gy], [3.0 * u, gy]);
            *out += &text(-6.0 * u, gy + 3.0 * u, &fmt(up * gy / ppm), fs, false, th.muted, " text-anchor=\"end\" class=\"tick\"");
        }
        let axes: Vec<&str> = arr(&vl["axes"]).iter().map(s).collect();
        let axes = if axes.len() >= 2 { axes } else { vec!["x", "y"] };
        *out += &text(x + w - 6.0 * u, -6.0 * u, &format!("{} (m)", axes[0]), fs, false, th.muted, " text-anchor=\"end\" class=\"tick\"");
        let ty = if vl["y_up"] == true { y } else { y + h } + 14.0 * u;
        *out += &text(6.0 * u, ty, &format!("{} (m), grid {} m", axes[1], fmt(metres)), fs, false, th.muted, " class=\"tick\"");
    }
}

fn plot_chrome(out: &mut String, vl: &Json, map: &Map, th: &Theme) {
    let Map::Plot { x: (bx0, bx1), y: (by0, by1), w, h, m } = *map else { return };
    // Ticks in each axis's display unit (D-070): `k` is one display unit in SI units.
    let k: Vec<f64> = arr(&vl["unit_scale"]).iter().map(f).collect();
    let (kx, ky) = if k.len() == 2 && k[0] > 0.0 && k[1] > 0.0 { (k[0], k[1]) } else { (1.0, 1.0) };
    let (x0, x1, y0, y1) = (bx0 / kx, bx1 / kx, by0 / ky, by1 / ky);
    let map = &Map::Plot { x: (x0, x1), y: (y0, y1), w, h, m };
    let _ = write!(
        out,
        "<rect class=\"frame\" x=\"{m}\" y=\"{m}\" width=\"{}\" height=\"{}\" fill=\"none\" stroke=\"{}\"/>",
        n(w - 2.0 * m),
        n(h - 2.0 * m),
        th.line,
        m = n(m)
    );
    // About one tick per 70 px across and 32 px down.
    let xs = nice_step((x1 - x0) / ((w - 2.0 * m) / 70.0).max(2.0));
    let ys = nice_step((y1 - y0) / ((h - 2.0 * m) / 32.0).max(2.0));
    let line = |out: &mut String, cls: &str, a: [f64; 2], b: [f64; 2]| {
        let color = if cls == "axis" { th.muted } else { th.faint };
        let _ = write!(out, "<line class=\"{cls}\" x1=\"{}\" y1=\"{}\" x2=\"{}\" y2=\"{}\" stroke=\"{color}\"/>", n(a[0]), n(a[1]), n(b[0]), n(b[1]));
    };
    let mut x = (x0 / xs).ceil() * xs;
    while x <= x1 + 1e-9 {
        let [px, _] = map.to([x, y0]);
        line(out, "gridline", [px, m], [px, h - m]);
        *out += &text(px, h - m + 16.0, &fmt(if x.abs() < xs * 1e-9 { 0.0 } else { x }), 12.0, false, th.muted, " text-anchor=\"middle\" class=\"tick\"");
        x += xs;
    }
    let mut y = (y0 / ys).ceil() * ys;
    while y <= y1 + 1e-9 {
        let [_, py] = map.to([x0, y]);
        line(out, "gridline", [m, py], [w - m, py]);
        *out += &text(m - 6.0, py + 4.0, &fmt(if y.abs() < ys * 1e-9 { 0.0 } else { y }), 12.0, false, th.muted, " text-anchor=\"end\" class=\"tick\"");
        y += ys;
    }
    if x0 < 0.0 && x1 > 0.0 {
        let [px, _] = map.to([0.0, 0.0]);
        line(out, "axis", [px, m], [px, h - m]);
    }
    if y0 < 0.0 && y1 > 0.0 {
        let [_, py] = map.to([0.0, 0.0]);
        line(out, "axis", [m, py], [w - m, py]);
    }
    let units: Vec<&str> = arr(&vl["units"]).iter().map(s).collect();
    if let Some(ux) = units.first().filter(|u| !u.is_empty()) {
        let elapsed = vl["x_elapsed"].as_bool().unwrap_or(true);
        let t = if *ux == "s" && elapsed { "elapsed time (s)".to_string() } else { format!("({ux})") };
        *out += &text(w - m, h - 8.0, &t, 12.0, false, th.muted, " text-anchor=\"end\" class=\"tick\"");
    }
    if let Some(uy) = units.get(1).filter(|u| !u.is_empty()) {
        *out += &text(m, m - 10.0, &format!("({uy})"), 12.0, false, th.muted, " class=\"tick\"");
    }
}

fn arrow_head(from: [f64; 2], to: [f64; 2], size: f64) -> Option<String> {
    let (dx, dy) = (to[0] - from[0], to[1] - from[1]);
    let len = dx.hypot(dy);
    if len < 1e-9 {
        return None;
    }
    let (ux, uy) = (dx / len, dy / len);
    let s = size.min(len * 0.6);
    let (bx, by) = (to[0] - ux * s, to[1] - uy * s);
    Some(format!(
        "{},{} {},{} {},{}",
        n(to[0]),
        n(to[1]),
        n(bx - uy * s * 0.45),
        n(by + ux * s * 0.45),
        n(bx + uy * s * 0.45),
        n(by - ux * s * 0.45)
    ))
}

/// The colour of `v` in `[0, 1]` on a palette of the frames (D-091): `sequential`, from dark
/// blue through green to yellow (perceptually ordered, readable without colour vision of
/// red and green), or `diverging`, blue through near white to red. The web player uses the
/// same stops.
pub fn palette(name: &str, v: f64) -> String {
    const SEQ: [[f64; 3]; 5] = [[68.0, 1.0, 84.0], [59.0, 82.0, 139.0], [33.0, 145.0, 140.0], [94.0, 201.0, 98.0], [253.0, 231.0, 37.0]];
    const DIV: [[f64; 3]; 3] = [[59.0, 76.0, 192.0], [242.0, 242.0, 242.0], [180.0, 4.0, 38.0]];
    let stops: &[[f64; 3]] = if name == "diverging" { &DIV } else { &SEQ };
    let x = v.clamp(0.0, 1.0) * (stops.len() - 1) as f64;
    let i = (x.floor() as usize).min(stops.len() - 2);
    let w = x - i as f64;
    let c: Vec<u8> = (0..3).map(|k| (stops[i][k] + w * (stops[i + 1][k] - stops[i][k])).round() as u8).collect();
    format!("#{:02x}{:02x}{:02x}", c[0], c[1], c[2])
}

/// Draws one representation of a spatial or plot view, as the web player's `drawRep`.
fn draw_rep(out: &mut String, map: &Map, r: &Json, u: f64, ci: usize, th: &Theme) {
    let shape = s(&r["shape"]);
    let invalid = r["valid"] == false;
    let drawn = r["drawn"].as_f64();
    // `reveal draw`: every stroke drawn up to the fraction reached (D-042).
    // The author's color and line style (D-061).
    // During a color animation (D-075), the color blended from the author's, or the kind's
    // own, to the color being reached.
    let own = match shape {
        "point" => th.reps[1],
        "arrow" => th.reps[ci % 4],
        "segment" => th.reps[0],
        "polyline" if r["kind"] == "trace" => th.muted,
        _ => th.accent,
    };
    // A colour of light is given as `#rrggbb` (D-106); other colours are palette names.
    let author = r["color"].as_str().and_then(|c| if c.starts_with('#') { Some(c) } else { th.color(c) });
    let custom: Option<String> = match (r["color_to"].as_str().and_then(|c| th.color(c)), r["color_mix"].as_f64()) {
        (Some(to), Some(k)) => Some(mix_hex(author.unwrap_or(own), to, k)),
        _ => author.map(String::from),
    };
    // An animated size (D-075): a marker's dot and an arrow's head grow with it.
    let grow = r["scale"].as_f64().unwrap_or(1.0);
    // The author's line width (D-095), a multiple of each stroke's own.
    let thick = r["width"].as_f64().unwrap_or(1.0);
    let style = match r["line"].as_str() {
        Some("dashed") => format!(" stroke-dasharray=\"{} {}\"", n(6.0 * u), n(4.0 * u)),
        Some("dotted") => format!(" stroke-dasharray=\"{} {}\" stroke-linecap=\"round\"", n(0.01 * u), n(4.0 * u)),
        Some("solid") => " stroke-dasharray=\"none\"".to_string(),
        _ => String::new(),
    };
    let dash = match drawn {
        Some(d) => format!(" pathLength=\"1\" stroke-dasharray=\"1\" stroke-dashoffset=\"{}\"", n(1.0 - d)),
        None if invalid => " stroke-dasharray=\"4 3\"".to_string(),
        None => style.clone(),
    };
    let mut cls = vec!["rep".to_string()];
    if r["drag"].is_string() {
        cls.push("draggable".into());
    }
    if invalid {
        cls.push("invalid".into());
    }
    if shape == "arrow" {
        cls.push("arrow".into());
        cls.push(format!("c{}", ci % 4));
    }
    let mut attrs = String::new();
    if let Some(o) = r["opacity"].as_f64() {
        let _ = write!(attrs, " opacity=\"{}\"", n(o));
    }
    if shape == "group" && r["highlighted"] == true {
        attrs += " filter=\"url(#highlight)\"";
    }
    let _ = write!(out, "<g class=\"{}\" data-rep=\"{}\"{attrs}><title>{}</title>", cls.join(" "), esc(s(&r["id"])), esc(s(&r["text"])));
    let label = |out: &mut String, x: f64, y: f64, anchor: &str| {
        if let Some(l) = r["label"].as_str() {
            *out += &text(x, y, l, 12.0 * u, false, th.muted, &format!("{anchor} class=\"rep-label\""));
        }
    };
    let ring = |out: &mut String, p: [f64; 2]| {
        if r["highlighted"] == true {
            let _ = write!(out, "<circle class=\"ring\" cx=\"{}\" cy=\"{}\" r=\"{}\" fill=\"none\" stroke=\"{}\" stroke-width=\"{}\"/>", n(p[0]), n(p[1]), n(13.0 * u), th.hl, n(3.0 * u));
        }
    };
    let pts = |k: &str| arr(&r[k]).iter().map(|p| map.to(pt(p))).map(|p| format!("{},{}", n(p[0]), n(p[1]))).collect::<Vec<_>>().join(" ");
    match shape {
        // Axes and grid of a view of three dimensions, projected by the engine (D-100).
        "lines" => {
            let (cls, color) = if r["kind"] == "axes" { ("axis", th.muted) } else { ("gridline", th.faint) };
            for l in arr(&r["lines"]) {
                let (a, b) = (map.to(pt(&l[0])), map.to(pt(&l[1])));
                let _ = write!(out, "<line class=\"{cls}\" x1=\"{}\" y1=\"{}\" x2=\"{}\" y2=\"{}\" stroke=\"{color}\" stroke-width=\"{}\"/>", n(a[0]), n(a[1]), n(b[0]), n(b[1]), n(u));
            }
            for l in arr(&r["labels"]) {
                let p = map.to(pt(&l["at"]));
                *out += &text(p[0], p[1], s(&l["text"]), 10.0 * u, false, th.muted, " text-anchor=\"middle\" class=\"tick\"");
            }
        }
        "point" => {
            let p = map.to(pt(&r["at"]));
            ring(out, p);
            let rad = if r["drag"].is_string() { 8.0 } else { 6.0 } * u * grow;
            let (fill, stroke, sd) = if invalid { ("none", th.bad, " stroke-dasharray=\"4 3\"") } else { (custom.as_deref().unwrap_or(th.reps[1]), th.surface, "") };
            let _ = write!(
                out,
                "<circle class=\"marker\" cx=\"{}\" cy=\"{}\" r=\"{}\" fill=\"{fill}\" stroke=\"{stroke}\" stroke-width=\"{}\"{sd}/>",
                n(p[0]),
                n(p[1]),
                n(rad),
                n(1.5 * u)
            );
            label(out, p[0] + 10.0 * u, p[1] - 10.0 * u, "");
        }
        // An author's text at a point (D-076).
        "note" => {
            let p = map.to(pt(&r["at"]));
            // Lined up with its point (D-095): starting at it, centred on it, or ending at it.
            let anchor = match r["align"].as_str() {
                Some("center") => " text-anchor=\"middle\"",
                Some("right") => " text-anchor=\"end\"",
                _ => "",
            };
            *out += &text(p[0], p[1], s(&r["value"]), 13.0 * u, false, custom.as_deref().unwrap_or(th.ink), &format!(" class=\"note\"{anchor}"));
        }
        "arrow" => {
            let (a, b) = (map.to(pt(&r["from"])), map.to(pt(&r["to"])));
            let color = if invalid { th.bad } else { custom.as_deref().unwrap_or(th.reps[ci % 4]) };
            let _ = write!(
                out,
                "<line x1=\"{}\" y1=\"{}\" x2=\"{}\" y2=\"{}\" stroke=\"{color}\" stroke-width=\"{}\"{dash}/>",
                n(a[0]),
                n(a[1]),
                n(b[0]),
                n(b[1]),
                n(2.0 * u * thick)
            );
            if let Some(head) = arrow_head(a, b, 11.0 * u * grow) {
                // Under `reveal draw` the head appears once the shaft is nearly drawn.
                let op = match drawn {
                    Some(d) if d <= 0.95 => " opacity=\"0\"",
                    _ => "",
                };
                let _ = write!(out, "<polygon points=\"{head}\" fill=\"{}\"{op}/>", custom.as_deref().unwrap_or(th.reps[ci % 4]));
            }
            ring(out, b);
            let len = (b[0] - a[0]).hypot(b[1] - a[1]);
            if len > 24.0 * u {
                // At the middle, offset to the left of the direction of the arrow.
                let (nx, ny) = (-(b[1] - a[1]) / len, (b[0] - a[0]) / len);
                label(out, (a[0] + b[0]) / 2.0 + nx * 12.0 * u, (a[1] + b[1]) / 2.0 + ny * 12.0 * u + 4.0 * u, " text-anchor=\"middle\"");
            }
            if r["drag"] == "head" {
                let (stroke, sd) = if invalid { (th.bad, " stroke-dasharray=\"4 3\"") } else { (th.accent, "") };
                let _ = write!(
                    out,
                    "<circle class=\"handle\" cx=\"{}\" cy=\"{}\" r=\"{}\" fill=\"{}\" stroke=\"{stroke}\" stroke-width=\"{}\"{sd} data-part=\"head\"/>",
                    n(b[0]),
                    n(b[1]),
                    n(7.0 * u),
                    th.surface,
                    n(2.0 * u)
                );
            }
        }
        "segment" => {
            let (a, b) = (map.to(pt(&r["from"])), map.to(pt(&r["to"])));
            let color = if invalid { th.bad } else { custom.as_deref().unwrap_or(th.reps[0]) };
            let _ = write!(
                out,
                "<line class=\"segment\" x1=\"{}\" y1=\"{}\" x2=\"{}\" y2=\"{}\" stroke=\"{color}\" stroke-width=\"{}\"{dash}/>",
                n(a[0]),
                n(a[1]),
                n(b[0]),
                n(b[1]),
                n(2.0 * u * thick)
            );
        }
        "polyline" => {
            let trace = r["kind"] == "trace";
            let (cls, color, width, extra) = if trace { ("trace", th.muted, 1.5, " stroke-dasharray=\"3 3\"") } else { ("graph", th.accent, 2.0, "") };
            let color = custom.as_deref().unwrap_or(color);
            // A drawn fraction, or the author's line, replaces the trace's own dashes, as in
            // the browser.
            let extra = if drawn.is_some() || !style.is_empty() { dash.as_str() } else { extra };
            // A sampled path breaks where its value jumped (a ball thrown again).
            let all = arr(&r["points"]);
            let mut cuts: Vec<usize> = vec![0];
            cuts.extend(arr(&r["breaks"]).iter().filter_map(|b| b.as_u64()).map(|b| b as usize));
            cuts.push(all.len());
            for w in cuts.windows(2) {
                let piece = all[w[0].min(all.len())..w[1].min(all.len())].iter().map(|p| map.to(pt(p))).map(|p| format!("{},{}", n(p[0]), n(p[1]))).collect::<Vec<_>>().join(" ");
                let _ = write!(out, "<polyline class=\"{cls}\" points=\"{piece}\" fill=\"none\" stroke=\"{color}\" stroke-width=\"{}\"{extra}/>", n(width * u * thick));
            }
        }
        // Coloured cells of a field in the plane (D-091), one rectangle per cell.
        "raster" => {
            let (from, to) = (map.to(pt(&r["from"])), map.to(pt(&r["to"])));
            let (cols, rows) = (r["cols"].as_u64().unwrap_or(1) as usize, r["rows"].as_u64().unwrap_or(1) as usize);
            let (cw, ch) = ((to[0] - from[0]) / cols as f64, (to[1] - from[1]) / rows as f64);
            let pal = s(&r["palette"]);
            let _ = write!(out, "<g class=\"raster\" shape-rendering=\"crispEdges\">");
            for (k, v) in arr(&r["values"]).iter().enumerate() {
                let (i, j) = ((k % cols) as f64, (k / cols) as f64);
                let (x0, y0) = (from[0] + i * cw, from[1] + j * ch);
                let (x, y) = (x0.min(x0 + cw), y0.min(y0 + ch));
                let _ = write!(out, "<rect x=\"{}\" y=\"{}\" width=\"{}\" height=\"{}\" fill=\"{}\"/>", n(x), n(y), n(cw.abs() + 0.5), n(ch.abs() + 0.5), palette(pal, v.as_f64().unwrap_or(0.0)));
            }
            out.push_str("</g>");
        }
        // A ray (D-106): to the edge of the drawing, and back behind its start when asked.
        "ray" => {
            let (f, d) = (pt(&r["from"]), pt(&r["toward"]));
            let far = 1e5;
            let _ = write!(
                out,
                "<line x1=\"{}\" y1=\"{}\" x2=\"{}\" y2=\"{}\" stroke=\"{}\" stroke-width=\"{}\"{style}/>",
                n(f[0]),
                n(f[1]),
                n(f[0] + d[0] * far),
                n(f[1] + d[1] * far),
                custom.as_deref().unwrap_or(th.reps[0]),
                n(2.0 * u * thick)
            );
            if let Some(b) = r["back"].as_str().filter(|b| *b != "none") {
                let dash = match b {
                    "dashed" => format!(" stroke-dasharray=\"{} {}\"", n(6.0 * u), n(4.0 * u)),
                    "dotted" => format!(" stroke-dasharray=\"{} {}\" stroke-linecap=\"round\"", n(0.01 * u), n(4.0 * u)),
                    _ => String::new(),
                };
                let _ = write!(
                    out,
                    "<line x1=\"{}\" y1=\"{}\" x2=\"{}\" y2=\"{}\" stroke=\"{}\" stroke-width=\"{}\"{dash}/>",
                    n(f[0]),
                    n(f[1]),
                    n(f[0] - d[0] * far),
                    n(f[1] - d[1] * far),
                    custom.as_deref().unwrap_or(th.reps[0]),
                    n(1.5 * u * thick)
                );
            }
        }
        "polygon" => {
            // An opaque body (D-104) hides what is behind it: the surface under its tint.
            let (op, tint) = if r["opaque"] == true { (true, "0.3") } else { (false, "0.12") };
            if op {
                let _ = write!(out, "<polygon points=\"{}\" fill=\"{}\"/>", pts("points"), th.surface);
            }
            let _ = write!(
                out,
                "<polygon class=\"shape\" points=\"{}\" fill=\"{accent}\" fill-opacity=\"{tint}\" stroke=\"{accent}\" stroke-width=\"{}\"{dash}/>",
                pts("points"),
                n(2.0 * u * thick),
                accent = custom.as_deref().unwrap_or(th.accent)
            );
        }
        "ellipse" => {
            // A circle, ellipse or arc (PK-6.3c): a whole one filled as a polygon is, an
            // arc stroked only.
            let accent = custom.as_deref().unwrap_or(th.accent);
            // An opaque body (D-104) hides what is behind it: the surface under its tint.
            let opaque = r["opaque"] == true && r["closed"] == true;
            if opaque {
                let _ = write!(out, "<path d=\"{}\" fill=\"{}\"/>", curve_path(r), th.surface);
            }
            let tint = if opaque { "0.3" } else { "0.12" };
            let fill = if r["closed"] == true { format!("fill=\"{accent}\" fill-opacity=\"{tint}\"") } else { "fill=\"none\"".to_string() };
            let cls = if r["closed"] == true { "shape" } else { "curve" };
            let _ = write!(out, "<path class=\"{cls}\" d=\"{}\" {fill} stroke=\"{}\" stroke-width=\"{}\"{dash}/>", curve_path(r), accent, n(2.0 * u * thick));
        }
        "group" => {
            // Members are placed by the kernel; the group carries opacity and highlight
            // (D-043).
            let mut arrows = 0;
            for m in arr(&r["members"]) {
                let c = if m["shape"] == "arrow" {
                    arrows += 1;
                    ci + arrows - 1
                } else {
                    0
                };
                draw_rep(out, map, m, u, c, th);
            }
        }
        _ => {}
    }
    *out += "</g>";
}

/// The SVG path of an `ellipse` shape: elliptical arcs of at most a quarter turn each, so
/// that no arc flag is ambiguous; a whole one is closed.
pub fn curve_path(r: &Json) -> String {
    let c = pt(&r["center"]);
    let radii = pt(&r["radii"]);
    let (rot, start, sweep) = (f(&r["rotation"]), f(&r["start"]), f(&r["sweep"]));
    let (s, co) = rot.sin_cos();
    let at = |a: f64| {
        let (x, y) = (radii[0] * a.cos(), radii[1] * a.sin());
        [c[0] + co * x - s * y, c[1] + s * x + co * y]
    };
    let pieces = (sweep.abs() / std::f64::consts::FRAC_PI_2).ceil().max(1.0) as usize;
    let p0 = at(start);
    let mut d = format!("M{} {}", n(p0[0]), n(p0[1]));
    for k in 1..=pieces {
        let p = at(start + sweep * k as f64 / pieces as f64);
        let _ = write!(d, " A{} {} {} 0 {} {} {}", n(radii[0]), n(radii[1]), n(rot.to_degrees()), (sweep > 0.0) as u8, n(p[0]), n(p[1]));
    }
    if r["closed"] == true {
        d.push_str(" Z");
    }
    d
}

// ---------------------------------------------------------------- panel items

/// A representation drawn outside a view's coordinates: formulas, equations, controls,
/// buttons, tables, labels and statuses (the web player draws these as HTML).
fn item(r: &Json, disabled: bool, opts: &Options) -> Block {
    let th = &opts.theme;
    let mut b = match s(&r["shape"]) {
        "formula" => formula_block(r, None, opts),
        "equation" => formula_block(r, Some(&format!("equation {}", s(&r["name"]))), opts),
        "button" => {
            let l = s(&r["label"]);
            let w = text_w(l, 14.0, false) + 26.0;
            let body = format!(
                "<rect x=\"0.5\" y=\"0.5\" width=\"{}\" height=\"27\" rx=\"6\" fill=\"{}\" stroke=\"{}\"/>{}",
                n(w - 1.0),
                th.surface,
                th.line,
                text(13.0, 19.0, l, 14.0, false, th.ink, "")
            );
            Block { w, h: 28.0, body }
        }
        "table" => table_block(r, th),
        "control" => control_block(r, disabled, th),
        "title" => text_block(s(&r["text"]), 13.0, false, th.ink),
        // A label's value in a fixed-width face; an author's text in the text face (D-076).
        "text" => text_block(s(&r["text"]), 14.0, r["kind"] != "text", th.ink),
        "status" => text_block(s(&r["text"]), 14.0, false, th.bad),
        _ => text_block(s(&r["text"]), 12.0, false, th.muted),
    };
    let mut attrs = String::new();
    if let Some(o) = r["opacity"].as_f64() {
        let _ = write!(attrs, " opacity=\"{}\"", n(o));
    }
    if r["highlighted"] == true {
        let _ = write!(
            b.body,
            "<rect x=\"-3\" y=\"-3\" width=\"{}\" height=\"{}\" rx=\"4\" fill=\"none\" stroke=\"{}\" stroke-width=\"2\"/>",
            n(b.w + 6.0),
            n(b.h + 6.0),
            th.hl
        );
    }
    b.body = format!(
        "<g class=\"item {}\" data-rep=\"{}\"{attrs}><title>{}</title>{}</g>",
        esc(s(&r["shape"])),
        esc(s(&r["id"])),
        esc(s(&r["text"])),
        b.body
    );
    b
}

/// A formula or an equation drawn from its layout (D-046), with the values of its live
/// symbols below it.
fn formula_block(r: &Json, what: Option<&str>, opts: &Options) -> Block {
    let th = &opts.theme;
    let mut parts = vec![];
    if let Some(w) = what {
        parts.push(text_block(w, 12.0, false, th.muted));
    }
    parts.push(math(&r["layout"], opts.math_em, th.ink));
    let vals: Vec<String> = arr(&r["symbols"]).iter().filter(|x| x["value"].is_string()).map(|x| format!("{} = {}", s(&x["symbol"]), s(&x["value"]))).collect();
    if !vals.is_empty() {
        parts.push(text_block(&vals.join(",  "), 12.0, true, th.muted));
    }
    stack(parts, 4.0)
}

/// Draws a formula layout (D-046) at `em` pixels to the em: text runs fitted to their
/// widths, rules and stroked paths.
pub fn math_svg(layout: &Json, em: f64, color: &str) -> (f64, f64, String) {
    let b = math(layout, em, color);
    (b.w, b.h, b.body)
}

fn math(layout: &Json, em: f64, color: &str) -> Block {
    let asc = f(&layout["ascent"]);
    let (w, h) = (f(&layout["width"]) * em, (asc + f(&layout["descent"])) * em);
    let base = asc * em;
    let mut body = String::new();
    for it in arr(&layout["items"]) {
        match s(&it["item"]) {
            "text" => {
                let italic = if it["italic"] == true { " font-style=\"italic\"" } else { "" };
                let binding = match it["binding"].as_str() {
                    Some(b) => format!(" data-binding=\"{}\"", esc(b)),
                    None => String::new(),
                };
                let _ = write!(
                    body,
                    "<text x=\"{}\" y=\"{}\" font-size=\"{}\" textLength=\"{}\" lengthAdjust=\"spacingAndGlyphs\" font-family=\"{MATH_FONT}\" fill=\"{color}\"{italic}{binding}>{}</text>",
                    n(f(&it["x"]) * em),
                    n(base + f(&it["y"]) * em),
                    n(f(&it["size"]) * em),
                    n(f(&it["width"]) * em),
                    esc(s(&it["text"]))
                );
            }
            "rule" => {
                let _ = write!(
                    body,
                    "<rect x=\"{}\" y=\"{}\" width=\"{}\" height=\"{}\" fill=\"{color}\"/>",
                    n(f(&it["x"]) * em),
                    n(base + f(&it["y"]) * em),
                    n(f(&it["w"]) * em),
                    n(f(&it["h"]) * em)
                );
            }
            "path" => {
                let pts: Vec<String> = arr(&it["points"]).iter().map(pt).map(|p| format!("{},{}", n(p[0] * em), n(base + p[1] * em))).collect();
                let _ = write!(
                    body,
                    "<polyline points=\"{}\" fill=\"none\" stroke=\"{color}\" stroke-width=\"{}\" stroke-linejoin=\"round\"/>",
                    pts.join(" "),
                    n(f(&it["stroke"]) * em)
                );
            }
            _ => {}
        }
    }
    Block { w, h, body }
}

/// Serif faces with true italics, which the layout's advance widths approximate (D-046).
const MATH_FONT: &str = "'Times New Roman', 'STIX Two Text', 'Liberation Serif', 'Nimbus Roman', serif";
const SANS: &str = "system-ui, 'Segoe UI', sans-serif";
const MONO: &str = "ui-monospace, 'Cascadia Mono', Consolas, monospace";

/// A control's value as the web player shows it: in its display unit, else in coherent SI.
fn control_display(r: &Json, value: f64) -> String {
    if r["control"] == "toggle" {
        return if value != 0.0 { "true" } else { "false" }.into();
    }
    if let Some(options) = r["options"].as_array() {
        return options.get(value as usize).and_then(|o| o.as_str()).unwrap_or("").into();
    }
    if r["display_unit"].is_object() {
        return format!("{} {}", fmt(value / f(&r["display_unit"]["scale"])), s(&r["display_unit"]["text"]));
    }
    match r["unit"].as_str() {
        Some(u) => format!("{} {u}", fmt(value)),
        None => fmt(value),
    }
}

/// A control as a still: its symbol, the input in its current state, and its value.
fn control_block(r: &Json, disabled: bool, th: &Theme) -> Block {
    let value = f(&r["value"]);
    let sym = s(&r["symbol"]);
    let mut body = text(0.0, 17.0, sym, 12.0, false, th.muted, "");
    let mut x = text_w(sym, 12.0, false) + 8.0;
    match s(&r["control"]) {
        "slider" => {
            let lo = r["min"].as_f64().unwrap_or(-10.0);
            let hi = r["max"].as_f64().unwrap_or(10.0);
            let t = if hi > lo { ((value - lo) / (hi - lo)).clamp(0.0, 1.0) } else { 0.0 };
            let w = 160.0;
            // Words at the slider's two ends (D-077): the low one before the bar.
            let ends: Vec<&str> = arr(&r["ends"]).iter().map(s).collect();
            if let Some(low) = ends.first() {
                body += &text(x, 17.0, low, 12.0, false, th.muted, "");
                x += text_w(low, 12.0, false) + 6.0;
            }
            let _ = write!(
                body,
                "<rect x=\"{}\" y=\"10\" width=\"{w}\" height=\"4\" rx=\"2\" fill=\"{}\"/><rect x=\"{}\" y=\"10\" width=\"{}\" height=\"4\" rx=\"2\" fill=\"{}\"/><circle cx=\"{}\" cy=\"12\" r=\"7\" fill=\"{}\"/>",
                n(x),
                th.line,
                n(x),
                n(w * t),
                th.accent,
                n(x + w * t),
                th.accent
            );
            x += w + 8.0;
            if let Some(high) = ends.get(1) {
                body += &text(x, 17.0, high, 12.0, false, th.muted, "");
                x += text_w(high, 12.0, false) + 8.0;
            }
        }
        "toggle" => {
            let on = value != 0.0;
            let _ = write!(
                body,
                "<rect x=\"{}\" y=\"5\" width=\"14\" height=\"14\" rx=\"3\" fill=\"{}\" stroke=\"{}\"/>",
                n(x),
                if on { th.accent } else { th.surface },
                if on { th.accent } else { th.muted }
            );
            if on {
                let _ = write!(body, "<polyline points=\"{},12 {},15 {},8\" fill=\"none\" stroke=\"{}\" stroke-width=\"2\"/>", n(x + 3.0), n(x + 6.0), n(x + 11.0), th.accent_ink);
            }
            x += 22.0;
        }
        // A select (D-094): a box with a caret; its value is printed after it.
        "select" => {
            let _ = write!(
                body,
                "<rect x=\"{}\" y=\"1.5\" width=\"24\" height=\"21\" rx=\"4\" fill=\"{}\" stroke=\"{}\"/><polyline points=\"{},10 {},15 {},10\" fill=\"none\" stroke=\"{}\" stroke-width=\"1.5\"/>",
                n(x),
                th.bg,
                th.line,
                n(x + 7.0),
                n(x + 12.0),
                n(x + 17.0),
                th.ink
            );
            x += 32.0;
        }
        _ => {
            let v = fmt(value);
            let w = text_w(&v, 14.0, true).max(60.0) + 12.0;
            let _ = write!(body, "<rect x=\"{}\" y=\"1.5\" width=\"{}\" height=\"21\" rx=\"4\" fill=\"{}\" stroke=\"{}\"/>", n(x), n(w), th.bg, th.line);
            body += &text(x + 6.0, 17.0, &v, 14.0, true, th.ink, "");
            x += w + 8.0;
        }
    }
    let out = control_display(r, value);
    body += &text(x, 17.0, &out, 14.0, true, th.ink, "");
    x += text_w(&out, 14.0, true);
    if disabled {
        body = format!("<g opacity=\"0.5\">{body}</g>");
    }
    Block { w: x, h: 24.0, body }
}

/// A table's latest twelve rows, most recent last, under a count of its rows.
fn table_block(r: &Json, th: &Theme) -> Block {
    let size = 12.0;
    let cols: Vec<&str> = arr(&r["columns"]).iter().map(s).collect();
    let rows = arr(&r["rows"]);
    let shown = &rows[rows.len().saturating_sub(12)..];
    let mut widths: Vec<f64> = cols.iter().map(|c| text_w(c, size, true)).collect();
    for row in shown {
        for (i, c) in arr(row).iter().enumerate() {
            if i < widths.len() {
                widths[i] = widths[i].max(text_w(s(c), size, true));
            }
        }
    }
    let widths: Vec<f64> = widths.into_iter().map(|w| w + 20.0).collect();
    let total: f64 = widths.iter().sum();
    let lh = 20.0;
    let mut body = text(0.0, 13.0, &format!("{} rows", rows.len()), 12.0, false, th.muted, "");
    let mut y = 18.0;
    let row = |body: &mut String, y: f64, cells: Vec<&str>, color: &str| {
        let mut x = 0.0;
        for (i, c) in cells.iter().enumerate() {
            let w = widths.get(i).copied().unwrap_or(0.0);
            *body += &text(x + w - 10.0, y + 14.0, c, size, true, color, " text-anchor=\"end\"");
            x += w;
        }
        let _ = write!(body, "<line x1=\"0\" y1=\"{}\" x2=\"{}\" y2=\"{}\" stroke=\"{}\"/>", n(y + lh - 0.5), n(total), n(y + lh - 0.5), th.line);
    };
    row(&mut body, y, cols.clone(), th.muted);
    y += lh;
    for r in shown {
        row(&mut body, y, arr(r).iter().map(s).collect(), th.ink);
        y += lh;
    }
    Block { w: total.max(text_w("0000 rows", 12.0, false)), h: y, body }
}

// ---------------------------------------------------------------- helpers

fn text(x: f64, y: f64, t: &str, size: f64, mono: bool, color: &str, extra: &str) -> String {
    format!(
        "<text x=\"{}\" y=\"{}\" font-size=\"{}\" font-family=\"{}\" fill=\"{color}\"{extra}>{}</text>",
        n(x),
        n(y),
        n(size),
        if mono { MONO } else { SANS },
        esc(t)
    )
}

/// An estimate of the width of a line of text, for laying out items.
fn text_w(t: &str, size: f64, mono: bool) -> f64 {
    t.chars().count() as f64 * size * if mono { 0.6 } else { 0.56 }
}

fn nice_step(raw: f64) -> f64 {
    let p = 10f64.powf(raw.log10().floor());
    let m = raw / p;
    (if m < 1.5 {
        1.0
    } else if m < 3.5 {
        2.0
    } else if m < 7.5 {
        5.0
    } else {
        10.0
    }) * p
}

/// A number as the web player prints it: six significant digits, trailing zeros removed,
/// exponent form outside `[1e-4, 1e6)`.
pub fn fmt(x: f64) -> String {
    if !x.is_finite() {
        return if x.is_nan() { "NaN".into() } else if x > 0.0 { "Infinity".into() } else { "-Infinity".into() };
    }
    if x == 0.0 {
        return "0".into();
    }
    let mag = x.abs().log10().floor() as i32;
    if !(-4..6).contains(&mag) {
        let e = format!("{x:.4e}");
        let (m, ex) = e.split_once('e').unwrap();
        let m = if m.contains('.') { m.trim_end_matches('0').trim_end_matches('.') } else { m };
        let ex = if ex.starts_with('-') { ex.to_string() } else { format!("+{ex}") };
        return format!("{m}e{ex}");
    }
    let s = format!("{:.*}", (5 - mag).max(0) as usize, x);
    if s.contains('.') {
        s.trim_end_matches('0').trim_end_matches('.').to_string()
    } else {
        s
    }
}

/// A coordinate or size in the document: at most three decimals.
fn n(x: f64) -> String {
    if !x.is_finite() {
        return "0".into();
    }
    let s = format!("{:.3}", x);
    let s = s.trim_end_matches('0').trim_end_matches('.');
    if s == "-0" {
        "0".into()
    } else {
        s.to_string()
    }
}

fn esc(t: &str) -> String {
    t.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;").replace('"', "&quot;")
}

fn s(j: &Json) -> &str {
    j.as_str().unwrap_or("")
}

fn f(j: &Json) -> f64 {
    j.as_f64().unwrap_or(0.0)
}

fn pt(j: &Json) -> [f64; 2] {
    [f(&j[0]), f(&j[1])]
}

fn arr(j: &Json) -> &[Json] {
    j.as_array().map(|v| v.as_slice()).unwrap_or(&[])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn numbers_print_as_in_the_web_player() {
        assert_eq!(fmt(0.0), "0");
        assert_eq!(fmt(2.5), "2.5");
        assert_eq!(fmt(3.605551275), "3.60555");
        assert_eq!(fmt(-40.0), "-40");
        assert_eq!(fmt(1234567.0), "1.2346e+6");
        assert_eq!(fmt(0.00001234), "1.234e-5");
        assert_eq!(n(-0.0001), "0");
        assert_eq!(nice_step(0.3), 0.2);
        assert_eq!(nice_step(1.5), 2.0);
    }
}

/// `#rrggbb` colors mixed: `k` of the way from `a` to `b`.
fn mix_hex(a: &str, b: &str, k: f64) -> String {
    let rgb = |h: &str| -> [f64; 3] {
        let v = u32::from_str_radix(h.trim_start_matches('#'), 16).unwrap_or(0);
        [((v >> 16) & 255) as f64, ((v >> 8) & 255) as f64, (v & 255) as f64]
    };
    let (x, y) = (rgb(a), rgb(b));
    let c: Vec<u32> = (0..3).map(|i| (x[i] + (y[i] - x[i]) * k.clamp(0.0, 1.0)).round() as u32).collect();
    format!("#{:02x}{:02x}{:02x}", c[0], c[1], c[2])
}

