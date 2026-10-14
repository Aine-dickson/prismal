//! Lowering of presentations and runs (04-ir section 7).
//!
//! Identities follow declaration paths (D-036): `Pres.observe.name`, `Pres.view.name`,
//! `Pres.scene.name`, `Pres.beat.name`; a representation is `container.name` when named,
//! otherwise `container.kind.n` (the n-th of its kind in its view or beat). Runs are
//! top-level: `Run`, with expectations `Run.expect.n`.

use crate::ast::{self, Bound, ExprKind};
use crate::lower::{ModelCx, SourceMap};
use crate::{Diag, Span};
use prismal_ir::present::*;
use prismal_ir::{Expr, Id, Model, Space};
use std::collections::HashMap;

/// Runtime controls a `button` can offer instead of an event (PK-6.3, D-069).
pub const RUN_CONTROLS: &[&str] = &["reset", "undo", "redo", "reseed", "restart"];

/// The runtime controls an explore beat may offer: they start the learner's branch again
/// (D-088).
pub const EXPLORE_CONTROLS: &[&str] = &["restart", "reseed"];

/// Representation kinds of the first slice (PK-6.3).
const KINDS: &[&str] = &[
    "marker", "arrow", "segment", "polyline", "polygon", "circle", "ellipse", "arc", "trace", "function_graph", "series_plot", "axes", "grid", "label",
    "equation", "formula", "table", "slider", "number_input", "toggle", "select", "button", "group", "text", "title", "histogram", "heatmap",
];

/// The named options each representation kind takes, besides `when` (D-081). A name not in
/// the list is an error, so a misspelled option is never ignored.
fn options(kind: &str) -> &'static [&'static str] {
    match kind {
        "marker" => &["at", "color", "label"],
        "arrow" => &["from", "scale", "color", "line", "width", "label"],
        "function_graph" => &["color", "line", "width", "label"],
        "segment" | "polyline" | "polygon" | "circle" | "trace" | "series_plot" => &["color", "line", "width"],
        "ellipse" => &["rotate", "color", "line", "width"],
        "arc" => &["from", "to", "color", "line", "width"],
        "text" => &["at"],
        "formula" | "equation" => &["live"],
        "slider" => &["range", "step", "ends"],
        "number_input" => &["range", "step"],
        "button" => &["label"],
        "histogram" => &["bins", "color", "line", "width"],
        "heatmap" => &["range"],
        "group" => &["at", "rotate", "scale"],
        _ => &[],
    }
}

/// An easing word of a timeline animation (D-075, D-095).
fn ease_of(word: &str) -> Option<prismal_ir::present::Ease> {
    use prismal_ir::present::Ease;
    Some(match word {
        "linear" => Ease::Linear,
        "smooth" => Ease::Smooth,
        "in" => Ease::In,
        "out" => Ease::Out,
        _ => return None,
    })
}

/// The named options of each view kind (D-081).
fn view_options(kind: &str) -> &'static [&'static str] {
    match kind {
        "spatial" => &["scale", "y"],
        "plot" => &["x", "y", "follow", "x_unit", "y_unit", "window"],
        _ => &[],
    }
}

/// Edit distance, for suggesting the option an unknown name was meant to be.
fn distance(a: &str, b: &str) -> usize {
    let b: Vec<char> = b.chars().collect();
    let mut row: Vec<usize> = (0..=b.len()).collect();
    for (i, ca) in a.chars().enumerate() {
        let mut prev = row[0];
        row[0] = i + 1;
        for (j, cb) in b.iter().enumerate() {
            let cur = row[j + 1];
            row[j + 1] = (prev + usize::from(ca != *cb)).min(row[j] + 1).min(cur + 1);
            prev = cur;
        }
    }
    row[b.len()]
}

/// Checks the named options of a representation or view: each is one the kind takes, and
/// each is given once. Returns the names to skip.
fn check_options(args: &[ast::Arg], kind: &str, allowed: &[&str], diags: &mut Vec<(String, Span)>) -> Vec<usize> {
    let mut skip = vec![];
    let mut seen: Vec<&str> = vec![];
    for (i, a) in args.iter().enumerate() {
        let Some(n) = &a.name else { continue };
        let name = n.text.as_str();
        if !allowed.contains(&name) {
            let near = allowed.iter().filter(|o| distance(name, o) <= 2).min_by_key(|o| distance(name, o));
            let msg = match (near, allowed.is_empty()) {
                (_, true) => format!("`{kind}` takes no option `{name}`: it has no named options"),
                (Some(o), _) => format!("`{kind}` takes no option `{name}`; did you mean `{o}`? Its options are {}", list(allowed)),
                (None, _) => format!("`{kind}` takes no option `{name}`; its options are {}", list(allowed)),
            };
            diags.push((msg, n.span));
            skip.push(i);
        } else if seen.contains(&name) {
            diags.push((format!("the option `{name}` is given twice"), n.span));
            skip.push(i);
        } else {
            seen.push(name);
        }
    }
    skip
}

fn list(words: &[&str]) -> String {
    words.iter().map(|w| format!("`{w}`")).collect::<Vec<_>>().join(", ")
}

struct PresCx<'a, 'b> {
    cx: ModelCx<'a>,
    /// Equations of the model by name, for `equation(name)`.
    equations: HashMap<String, Id>,
    pres: &'b str,
    views: HashMap<String, Id>,
    aliases: HashMap<String, Id>,
    /// The presentation's instruments by name (PK-3.7, D-087).
    instruments: HashMap<String, Id>,
}

impl PresCx<'_, '_> {
    /// A page layout (PK-7.4a, D-063): every name a view of this presentation, placed once.
    fn layout(&mut self, n: &ast::LayoutNode, placed: &mut HashMap<String, Span>) -> Option<Layout> {
        match n {
            ast::LayoutNode::View(name) => {
                let Some(id) = self.views.get(&name.text).cloned() else {
                    self.err("SX-E03", format!("unknown view `{}`", name.text), name.span);
                    return None;
                };
                if placed.insert(name.text.clone(), name.span).is_some() {
                    self.err("SX-E09", format!("view `{}` placed twice in the layout", name.text), name.span);
                    return None;
                }
                Some(Layout::View(id))
            }
            ast::LayoutNode::Row(_, xs) => Some(Layout::Row(xs.iter().filter_map(|x| self.layout(x, placed)).collect())),
            ast::LayoutNode::Column(_, xs) => Some(Layout::Column(xs.iter().filter_map(|x| self.layout(x, placed)).collect())),
        }
    }

    /// The easing a clause `ease e` names; an unknown word is SX-E02.
    fn ease(&mut self, e: Option<&ast::Name>) -> Option<prismal_ir::present::Ease> {
        let e = e?;
        let found = ease_of(&e.text);
        if found.is_none() {
            self.err("SX-E02", format!("unknown easing `{}`: `linear`, `smooth`, `in` or `out` (D-075)", e.text), e.span);
        }
        found
    }

    fn err(&mut self, code: &'static str, msg: impl Into<String>, span: Span) {
        self.cx.err(code, msg, span);
    }

    fn expr(&mut self, e: &ast::Expr) -> Expr {
        self.cx.expr(e, &[])
    }

    fn range(&mut self, e: &ast::Expr) -> Option<(Expr, Expr)> {
        match &e.kind {
            ExprKind::List(items) if items.len() == 2 => Some((self.expr(&items[0]), self.expr(&items[1]))),
            _ => {
                self.err("SX-E02", "a range is written `[lo, hi]`", e.span);
                None
            }
        }
    }

    fn scale(&mut self, e: &ast::Expr) -> Option<Scale> {
        match &e.kind {
            ExprKind::Map(q, px) => match &px.kind {
                ExprKind::Num(v, Some(u)) if u.text == "px" => Some(Scale { quantity: self.expr(q), px: *v }),
                _ => {
                    self.err("SX-E02", "a scale maps a quantity to view pixels: `1 m -> 40 px`", px.span);
                    None
                }
            },
            _ => {
                self.err("SX-E02", "a scale is written `quantity -> n px` (PK-5.5)", e.span);
                None
            }
        }
    }

    fn rep(&mut self, r: &ast::Rep, container: &str, counts: &mut HashMap<String, usize>) -> Rep {
        let kind = r.kind.text.clone();
        if !KINDS.contains(&kind.as_str()) {
            self.err("SX-E06", format!("unknown representation kind `{kind}` (PK-6.3)"), r.kind.span);
        }
        let n = counts.entry(kind.clone()).or_insert(0);
        *n += 1;
        let id = match &r.alias {
            Some(a) => format!("{container}.{}", a.text),
            None => format!("{container}.{kind}.{n}"),
        };
        if let Some(a) = &r.alias {
            if self.aliases.insert(a.text.clone(), id.clone()).is_some() {
                self.err("SX-E09", format!("representation name `{}` used twice", a.text), a.span);
            }
        }
        self.cx.map.insert(id.clone(), r.span);
        // `for b in row { ... }`: the member is in scope in the representation (D-055).
        let each = match &r.each {
            Some((var, over)) => match self.cx.part_ref(&over.text).ok() {
                Some(p) => {
                    self.cx.vars.push((var.text.clone(), p.object.clone()));
                    Some(prismal_ir::Each { var: var.text.clone(), over: p.id })
                }
                None => {
                    self.err("SX-E03", format!("unknown collection `{}`", over.text), over.span);
                    None
                }
            },
            None => None,
        };
        let mut sources = vec![];
        let mut props = vec![];
        // `when: cond`: drawn only while the condition holds (D-076).
        let mut when = None;
        let mut allowed = options(&kind).to_vec();
        allowed.push("when");
        let mut bad = vec![];
        let skip = check_options(&r.args, &kind, &allowed, &mut bad);
        for (m, s) in bad {
            self.err("SX-E08", m, s);
        }
        for (i, a) in r.args.iter().enumerate() {
            if skip.contains(&i) {
                continue;
            }
            // `ends: ("cold", "hot")`: words for a slider's two ends (D-077).
            if let (Some(n), ExprKind::Tuple(items)) = (&a.name, &a.value.kind) {
                if n.text == "ends" {
                    let texts: Vec<String> = items.iter().filter_map(|e| if let ExprKind::Str(s) = &e.kind { Some(s.clone()) } else { None }).collect();
                    if texts.len() != 2 || items.len() != 2 {
                        self.err("SX-E02", "a slider's ends are two words in quotes: `ends: (\"cold\", \"hot\")`", a.value.span);
                    }
                    props.push(Prop { name: "ends".into(), value: Arg::Texts { texts } });
                    continue;
                }
            }
            if a.name.as_ref().is_some_and(|n| n.text == "when") {
                when = Some(self.expr(&a.value));
                continue;
            }
            // `text("... {e} ...")`: the template with `{}` for each value, then the values (D-076).
            if let (ExprKind::Str(s), None, "text") = (&a.value.kind, &a.name, r.kind.text.as_str()) {
                let (template, values) = self.interpolate(s, a.value.span);
                sources.push(Arg::Text { text: template });
                sources.extend(values.into_iter().map(|expr| Arg::Expr { expr }));
                continue;
            }
            let value = match (&a.value.kind, a.name.as_ref().map(|n| n.text.as_str())) {
                (ExprKind::Str(s), _) => Arg::Text { text: s.clone() },
                // Style words (D-061): `color: blue`, `line: dashed`.
                (ExprKind::Name(w), Some("color" | "line")) => Arg::Word { word: w.clone() },
                // `label: none`: no label drawn beside the representation (D-078).
                (ExprKind::Name(w), Some("label")) if w == "none" => Arg::Word { word: w.clone() },
                (ExprKind::List(_), _) => match self.range(&a.value) {
                    Some((lo, hi)) => Arg::Range { lo, hi },
                    None => continue,
                },
                (ExprKind::Map(..), _) => match self.scale(&a.value) {
                    Some(scale) => Arg::Scale { scale },
                    None => continue,
                },
                // A runtime control of a button (D-069): `button(reset)`, unless the model has an
                // event of that name.
                (ExprKind::Name(n), None) if r.kind.text == "button" && RUN_CONTROLS.contains(&n.as_str()) && !self.cx.bindings.contains_key(n) && !self.cx.events.contains_key(n) => {
                    Arg::Word { word: n.clone() }
                }
                // `button(add(2 m))`: a button that requests an event with a payload; the payload
                // is the property `payload` (D-072).
                (ExprKind::Call(callee, args), None) if r.kind.text == "button" && matches!(&callee.kind, ExprKind::Name(n) if self.cx.events.contains_key(n)) => {
                    let ExprKind::Name(n) = &callee.kind else { unreachable!() };
                    let element = self.cx.events.get(n).cloned().unwrap();
                    let items: Vec<prismal_ir::Expr> = args.iter().map(|x| self.expr(&x.value)).collect();
                    let payload = if items.len() == 1 { items.into_iter().next().unwrap() } else { prismal_ir::Expr::Tuple { tuple: items } };
                    props.push(Prop { name: "payload".into(), value: Arg::Expr { expr: payload } });
                    Arg::Element { element }
                }
                // A positional name of an event or equation, not a binding: a model element.
                (ExprKind::Name(n), None) if a.every.is_none() && a.on.is_none() && !self.cx.bindings.contains_key(n) && (self.cx.events.contains_key(n) || self.equations.contains_key(n)) => {
                    let element = self.cx.events.get(n).or_else(|| self.equations.get(n)).cloned().unwrap();
                    Arg::Element { element }
                }
                _ => match (&a.every, &a.ensemble) {
                    (Some(dt), _) => {
                        let (expr, instrument) = self.measured(&a.value);
                        Arg::Sampled { expr, every: self.expr(dt), instrument }
                    }
                    (None, Some((at, n))) => {
                        let (expr, instrument) = self.measured(&a.value);
                        Arg::Ensemble { expr, at: self.expr(at), runs: self.expr(n), instrument }
                    }
                    (None, None) if a.on.is_some() => {
                        let (expr, instrument) = self.measured(&a.value);
                        Arg::On { expr, event: self.cx.event_id(a.on.as_ref().unwrap()), instrument }
                    }
                    (None, None) => {
                        if let Some(name) = self.instrument_call(&a.value) {
                            self.err("SX-E08", format!("instrument `{name}` reads at instants: a sampled source, `{name}(x) every 0.1 s`, or an ensemble"), a.value.span);
                        }
                        Arg::Expr { expr: self.expr(&a.value) }
                    }
                },
            };
            match &a.name {
                None => sources.push(value),
                Some(n) => props.push(Prop { name: n.text.clone(), value }),
            }
        }
        let (clicks, drags): (Vec<&ast::Interaction>, Vec<&ast::Interaction>) = r.interactions.iter().partition(|i| i.request.is_some());
        if drags.len() > 1 {
            self.err("SX-E06", "one declared inverse per representation in v0", drags[1].span);
        }
        if clicks.len() > 1 {
            self.err("SX-E06", "one click per representation", clicks[1].span);
        }
        // `on click request E(v)` (D-059): the payload is read in the representation's scope.
        let click = clicks.first().and_then(|i| i.request.as_ref()).map(|(e, p)| Click { event: self.cx.event_id(e), payload: p.as_ref().map(|v| self.expr(v)) });
        let inverse = drags.first().map(|i| {
            let locals = vec![i.bind.text.clone()];
            let proposals = i
                .proposals
                .iter()
                .map(|(t, v)| {
                    // A proposal sets a whole binding, of the model or of a member (D-059).
                    let target = self.cx.target(t);
                    if target.component.is_some() {
                        self.err("SX-E06", "a proposal sets a whole binding: `propose pos = p`", t.name.span);
                    }
                    Proposal { target: target.binding, value: self.cx.expr(v, &locals), member: target.member }
                })
                .collect();
            if i.live && i.gesture.text != "drag" {
                self.err("SX-E06", "only a drag can be `live`", i.gesture.span);
            }
            Inverse { gesture: i.gesture.text.clone(), part: i.part.as_ref().map(|p| p.text.clone()), proposals, live: i.live }
        });
        // Members are numbered within their group, which is their container (D-043).
        let mut inner = HashMap::new();
        let members = r.members.iter().map(|m| self.rep(m, &id, &mut inner)).collect();
        if each.is_some() {
            self.cx.vars.pop();
        }
        Rep { each, id, name: r.alias.as_ref().map(|a| a.text.clone()), kind, sources, props, inverse, members, click, when }
    }

    /// A text with values in braces, `"bounced {n} times"`: the template with `{}` where each
    /// value goes (literal braces doubled), and the values (D-076).
    fn interpolate(&mut self, s: &str, span: Span) -> (String, Vec<prismal_ir::Expr>) {
        let (mut template, mut values) = (String::new(), vec![]);
        let mut chars = s.chars().peekable();
        while let Some(c) = chars.next() {
            match c {
                '{' if chars.peek() == Some(&'{') => {
                    chars.next();
                    template.push_str("{{");
                }
                '}' if chars.peek() == Some(&'}') => {
                    chars.next();
                    template.push_str("}}");
                }
                '{' => {
                    let inner: String = chars.by_ref().take_while(|c| *c != '}').collect();
                    // `{last:.1}`: the value with one decimal (D-078).
                    let (inner, places) = match inner.rsplit_once(":.") {
                        Some((e, n)) if !n.is_empty() && n.chars().all(|c| c.is_ascii_digit()) => (e.to_string(), Some(n.to_string())),
                        _ => (inner, None),
                    };
                    match crate::parse_expr(&inner) {
                        Ok(e) => {
                            values.push(self.expr(&e));
                            match places {
                                Some(n) => template.push_str(&format!("{{:.{n}}}")),
                                None => template.push_str("{}"),
                            }
                        }
                        Err(m) => self.err("SX-E02", format!("in `{{{inner}}}`: {m}"), span),
                    }
                }
                '}' => self.err("SX-E02", "a `}` with no `{`; write `}}` for a brace in the text", span),
                c => template.push(c),
            }
        }
        (template, values)
    }

    fn view(&mut self, v: &ast::ViewDecl) -> Option<View> {
        let id = format!("{}.view.{}", self.pres, v.name.text);
        let mut bad = vec![];
        check_options(&v.args, &v.kind.text, view_options(&v.kind.text), &mut bad);
        for (m, s) in bad {
            self.err("SX-E08", m, s);
        }
        let named = |n: &str| v.args.iter().find(|a| a.name.as_ref().is_some_and(|x| x.text == n)).map(|a| &a.value);
        let kind = match v.kind.text.as_str() {
            "spatial" => {
                let space = match v.args.first() {
                    Some(ast::Arg { name: None, value: ast::Expr { kind: ExprKind::Name(s), span }, every: None, ensemble: None, on: None }) => match self.cx.space_by_name(s) {
                        Some(id) => id,
                        None => {
                            self.err("SX-E03", format!("unknown space `{s}`"), *span);
                            return None;
                        }
                    },
                    _ => {
                        self.err("SX-E02", "a spatial view names its space first: `spatial(Plane, scale: ...)`", v.kind.span);
                        return None;
                    }
                };
                let scale = match named("scale") {
                    Some(e) => self.scale(e)?,
                    None => {
                        self.err("SX-E02", "a spatial view declares its scale: `scale: 1 m -> 40 px` (PK-7.2)", v.kind.span);
                        return None;
                    }
                };
                let y_up = match named("y").map(|e| &e.kind) {
                    None => true,
                    Some(ExprKind::Name(w)) if w == "up" => true,
                    Some(ExprKind::Name(w)) if w == "down" => false,
                    Some(_) => {
                        self.err("SX-E02", "the orientation is `y: up` or `y: down`", named("y").unwrap().span);
                        true
                    }
                };
                ViewKind::Spatial { space, scale, y_up }
            }
            "plot" => {
                let (x, y) = match (named("x"), named("y")) {
                    (Some(x), Some(y)) => (self.range(x)?, self.range(y)?),
                    _ => {
                        self.err("SX-E02", "a plot view declares its axes: `plot(x: [a, b], y: [c, d])`", v.kind.span);
                        return None;
                    }
                };
                // `follow: y` or `follow: (x, y)`: those axes grow to keep the data in view;
                // `x_unit: ms`, `y_unit: cm`: the axes' display units (D-070).
                let mut follow = [false, false];
                if let Some(f) = named("follow") {
                    let axes: Vec<&ast::Expr> = match &f.kind {
                        ExprKind::Tuple(items) => items.iter().collect(),
                        _ => vec![f],
                    };
                    for a in axes {
                        match &a.kind {
                            ExprKind::Name(w) if w == "x" => follow[0] = true,
                            ExprKind::Name(w) if w == "y" => follow[1] = true,
                            _ => self.err("SX-E02", "a plot follows its data on `x`, `y` or both: `follow: y`, `follow: (x, y)`", a.span),
                        }
                    }
                }
                let mut units = [None, None];
                for (i, key) in ["x_unit", "y_unit"].iter().enumerate() {
                    if let Some(u) = named(key) {
                        match &u.kind {
                            ExprKind::Name(w) => units[i] = Some(w.clone()),
                            _ => self.err("SX-E02", format!("an axis display unit is a unit symbol: `{key}: cm`"), u.span),
                        }
                    }
                }
                // `window: 10 s`: the latest span of the `x` axis once the data passes it.
                let window = named("window").map(|w| self.cx.expr(w, &[]));
                ViewKind::Plot { x: [x.0, x.1], y: [y.0, y.1], follow, units, window }
            }
            "panel" => ViewKind::Panel,
            other => {
                self.err("SX-E06", format!("unknown view kind `{other}`: `spatial`, `plot` or `panel`"), v.kind.span);
                return None;
            }
        };
        self.cx.map.insert(id.clone(), v.span);
        let mut counts = HashMap::new();
        let representations = v.reps.iter().map(|r| self.rep(r, &id, &mut counts)).collect();
        // `on click as p request E(p)` (D-060): the point is the gesture value, `{"param": 0}`.
        if let Some(extra) = v.clicks.get(1) {
            self.err("SX-E06", "one click per view", extra.span);
        }
        let click = v.clicks.first().map(|i| {
            if matches!(kind, ViewKind::Panel) {
                self.err("SX-E06", "a panel has no points to click; clicks on a point are for spatial and plot views", i.span);
            }
            let (e, p) = i.request.as_ref().unwrap();
            let locals = vec![i.bind.text.clone()];
            Click { event: self.cx.event_id(e), payload: p.as_ref().map(|v| self.cx.expr(v, &locals)) }
        });
        Some(View { id, name: v.name.text.clone(), kind, representations, click })
    }

    /// The name of the instrument `e` reads through, when `e` is `ruler(x)` for an
    /// instrument `ruler` of the presentation.
    fn instrument_call(&self, e: &ast::Expr) -> Option<String> {
        let ExprKind::Call(f, args) = &e.kind else { return None };
        let ExprKind::Name(n) = &f.kind else { return None };
        (self.instruments.contains_key(n) && args.len() == 1 && args[0].name.is_none()).then(|| n.clone())
    }

    /// A source and the instrument it is read through: `ruler(x)` is `x` read by `ruler`
    /// (PK-3.7, D-087); anything else is read exactly.
    fn measured(&mut self, e: &ast::Expr) -> (Expr, Option<Id>) {
        match self.instrument_call(e) {
            Some(n) => {
                let ExprKind::Call(_, args) = &e.kind else { unreachable!() };
                (self.expr(&args[0].value), self.instruments.get(&n).cloned())
            }
            None => (self.expr(e), None),
        }
    }

    /// `instrument ruler(resolution: 1 mm, range: [0 m, 1 m], noise: 2 mm)`.
    fn instrument(&mut self, name: &ast::Name, args: &[ast::Arg], span: Span) -> Instrument {
        let id = format!("{}.instrument.{}", self.pres, name.text);
        self.cx.map.insert(id.clone(), span);
        let mut bad = vec![];
        let skip = check_options(args, "instrument", &["resolution", "range", "noise"], &mut bad);
        for (m, s) in bad {
            self.err("SX-E08", m, s);
        }
        let mut out = Instrument { id, name: name.text.clone(), resolution: None, range: None, noise: None };
        for (i, a) in args.iter().enumerate() {
            match a.name.as_ref().map(|n| n.text.as_str()) {
                _ if skip.contains(&i) => {}
                None => self.err("SX-E08", "an instrument takes named options: `resolution`, `range`, `noise`", a.value.span),
                Some("range") => out.range = self.range(&a.value),
                Some("resolution") => out.resolution = Some(self.expr(&a.value)),
                Some(_) => out.noise = Some(self.expr(&a.value)),
            }
        }
        out
    }

    fn observation(&mut self, o: &ast::Observation, model: &Model) -> Observation {
        let id = format!("{}.observe.{}", self.pres, o.name.text);
        let mut instrument = None;
        self.cx.map.insert(id.clone(), o.span);
        let word = match &o.expr.kind {
            ExprKind::Name(n) if matches!(n.as_str(), "event_log" | "diagnostics" | "intervention_log") => Some(n.as_str()),
            _ => None,
        };
        if word.is_none() && (o.of.is_some() || o.filter.is_some()) {
            self.err("SX-E08", "`of` and `where` apply to `event_log` and `diagnostics`", o.span);
        }
        let source = match word {
            Some("event_log") => {
                let event = o.of.as_ref().map(|n| self.cx.event_id(n));
                let zeno_only = match &o.filter {
                    None => false,
                    Some(ast::Expr { kind: ExprKind::Name(w), .. }) if w == "zeno_applied" => true,
                    Some(f) => {
                        self.err("SX-E06", "the only event-log filter in v0 is `where zeno_applied`", f.span);
                        false
                    }
                };
                Source::EventLog { event, zeno_only }
            }
            Some("diagnostics") => {
                let element = o.of.as_ref().map(|n| {
                    let candidates = [
                        format!("{}.equation.{}", model.name, n.text),
                        format!("{}.constraint.{}", model.name, n.text),
                        format!("{}.event.{}", model.name, n.text),
                        format!("{}.{}", model.name, n.text),
                    ];
                    let known = |id: &String| {
                        model.equations.iter().any(|x| &x.id == id)
                            || model.constraints.iter().any(|x| &x.id == id)
                            || model.events.iter().any(|x| &x.id == id)
                            || model.bindings.iter().any(|x| &x.id == id)
                    };
                    match candidates.iter().find(|c| known(c)) {
                        Some(id) => id.clone(),
                        None => {
                            self.err("SX-E03", format!("unknown element `{}` in model `{}`", n.text, model.name), n.span);
                            candidates[0].clone()
                        }
                    }
                });
                Source::Diagnostics { element }
            }
            Some(_) => Source::InterventionLog,
            None => {
                let (expr, i) = self.measured(&o.expr);
                instrument = i;
                Source::Expr { expr }
            }
        };
        let schedule = match &o.schedule {
            None => Schedule::Run,
            Some(ast::Schedule::Live) => Schedule::Live,
            Some(ast::Schedule::Every(e)) => Schedule::Every { period: self.expr(e) },
            Some(ast::Schedule::At(e, runs)) => Schedule::At { time: self.expr(e), runs: runs.as_ref().map(|n| self.expr(n)) },
            Some(ast::Schedule::On(ev, micro)) => Schedule::On { event: self.cx.event_id(ev), microstep: *micro },
            Some(ast::Schedule::Over(i)) => {
                let from = match &i.lo {
                    Bound::Value(e) => self.expr(e),
                    Bound::Inf => {
                        self.err("SX-E08", "an observation window starts at an instant", i.span);
                        prismal_ir::build::t0()
                    }
                };
                let to = match &i.hi {
                    Bound::Value(e) if matches!(&e.kind, ExprKind::Name(n) if n == "t_end") => None,
                    Bound::Value(e) => Some(self.expr(e)),
                    Bound::Inf => None,
                };
                Schedule::Over { from, to, lo_closed: i.lo_closed, hi_closed: i.hi_closed }
            }
        };
        Observation { id, name: o.name.text.clone(), source, schedule, instrument }
    }

    #[allow(clippy::only_used_in_recursion)]
    fn actions(&mut self, acts: &[ast::Action], beat: &str, counts: &mut HashMap<String, usize>, span: Span) -> Vec<Action> {
        let mut out = vec![];
        for a in acts {
            match a {
                ast::Action::In { view, reps } => {
                    let v = match self.views.get(&view.text) {
                        Some(v) => Some(v.clone()),
                        None => {
                            self.err("SX-E03", format!("unknown view `{}`", view.text), view.span);
                            None
                        }
                    };
                    let reps = reps.iter().map(|r| self.rep(r, beat, counts)).collect();
                    out.push(Action::Show { view: v, reps });
                }
                ast::Action::Show(r) => {
                    let rep = self.rep(r, beat, counts);
                    out.push(Action::Show { view: None, reps: vec![rep] });
                }
                ast::Action::Narrate { text, duration } => {
                    out.push(Action::Narrate { text: text.clone(), duration: duration.as_ref().map(|d| self.expr(d)) })
                }
                ast::Action::Run { rate, until } => {
                    out.push(Action::Run { rate: self.expr(rate) });
                    if let Some(e) = until {
                        out.push(Action::WaitUntil { event: self.cx.event_id(e) });
                    }
                }
                ast::Action::Hold => out.push(Action::Hold),
                ast::Action::Reset => out.push(Action::Reset),
                ast::Action::Restart => out.push(Action::Restart),
                ast::Action::Branch => out.push(Action::Branch),
                ast::Action::Highlight(n) => match self.aliases.get(&n.text) {
                    Some(id) => out.push(Action::Highlight { target: id.clone() }),
                    None => self.err("SX-E03", format!("no representation named `{}` before this beat", n.text), n.span),
                },
                ast::Action::Hide(n, d, e) => match self.aliases.get(&n.text).cloned() {
                    Some(id) => {
                        let duration = d.as_ref().map(|d| self.expr(d));
                        let ease = self.ease(e.as_ref());
                        out.push(Action::Hide { target: id, duration, ease })
                    }
                    None => self.err("SX-E03", format!("no representation named `{}` before this beat", n.text), n.span),
                },
                ast::Action::Reveal { style, duration, view, ease, reps } => {
                    let ease = self.ease(ease.as_ref());
                    let style = match style.text.as_str() {
                        "fade" => RevealStyle::Fade,
                        "draw" => RevealStyle::Draw,
                        other => {
                            self.err("SX-E02", format!("a reveal is `fade` or `draw`, not `{other}` (D-042)"), style.span);
                            RevealStyle::Fade
                        }
                    };
                    let v = match view {
                        Some(n) => match self.views.get(&n.text) {
                            Some(v) => Some(v.clone()),
                            None => {
                                self.err("SX-E03", format!("unknown view `{}`", n.text), n.span);
                                None
                            }
                        },
                        None => None,
                    };
                    let duration = duration.as_ref().map(|d| self.expr(d));
                    let reps = reps.iter().map(|r| self.rep(r, beat, counts)).collect();
                    out.push(Action::Reveal { view: v, style, duration, ease, reps });
                }
                ast::Action::Camera { view, center, zoom, duration, ease } => match self.views.get(&view.text).cloned() {
                    Some(v) => {
                        let ease = self.ease(ease.as_ref());
                        let center = center.as_ref().map(|e| self.expr(e));
                        let zoom = zoom.as_ref().map(|e| self.expr(e));
                        let duration = duration.as_ref().map(|e| self.expr(e));
                        out.push(Action::Camera { view: v, center, zoom, duration, ease });
                    }
                    None => self.err("SX-E03", format!("unknown view `{}`", view.text), view.span),
                },
                ast::Action::Seek(e) => out.push(Action::Seek { time: self.expr(e) }),
                ast::Action::Explore { limit, keep, reps, fallback } => {
                    let limit = limit.as_ref().map(|l| self.expr(l));
                    let keep = keep.iter().map(|k| self.cx.binding(k)).collect();
                    let controls = reps.iter().map(|r| self.rep(r, beat, counts)).collect();
                    let fallback = self.actions(fallback, beat, counts, span);
                    out.push(Action::Explore { limit, keep, controls, fallback });
                }
                ast::Action::Sequence(acts) => {
                    let actions = self.actions(acts, beat, counts, span);
                    out.push(Action::Sequence { actions });
                }
                ast::Action::Intervene(ops) => {
                    let ops = ops.iter().map(|o| self.cx.op(o)).collect();
                    out.push(Action::Intervene { ops });
                }
                ast::Action::Wait(e) => out.push(Action::Wait { duration: self.expr(e) }),
                ast::Action::WaitUntil(e) => out.push(Action::WaitUntil { event: self.cx.event_id(e) }),
                ast::Action::Request(e, p) => {
                    let payload = p.as_ref().map(|v| self.expr(v));
                    out.push(Action::Request { event: self.cx.event_id(e), payload })
                }
                ast::Action::WaitLearner { limit, fallback } => {
                    let limit = limit.as_ref().map(|l| self.expr(l));
                    let fallback = self.actions(fallback, beat, counts, span);
                    out.push(Action::WaitLearner { limit, fallback });
                }
                ast::Action::Animate { target, property, to, duration, ease } => {
                    let property = match property.text.as_str() {
                        "opacity" => Animated::Opacity,
                        "offset" => Animated::Offset,
                        "color" => Animated::Color,
                        "line" => Animated::Line,
                        "scale" => Animated::Scale,
                        "width" => Animated::Width,
                        "morph" => Animated::Morph,
                        other => {
                            self.err("SX-E02", format!("`animate` drives `opacity`, `offset`, `color`, `line`, `width`, `scale` or `morph`, not `{other}` (D-068, D-075, D-095)"), property.span);
                            Animated::Opacity
                        }
                    };
                    let ease = self.ease(ease.as_ref());
                    match self.aliases.get(&target.text).cloned() {
                        Some(id) => {
                            // A palette word, a line style or a representation's name is a word;
                            // the other properties take an expression.
                            let (to, word) = match property {
                                Animated::Color | Animated::Line => match &to.kind {
                                    ExprKind::Name(w) => (None, Some(w.clone())),
                                    _ => {
                                        self.err("SX-E02", "a color or line style is a word: `to red`, `to dashed`", to.span);
                                        (None, None)
                                    }
                                },
                                Animated::Morph => match &to.kind {
                                    ExprKind::Name(w) => match self.aliases.get(w).cloned() {
                                        Some(other) => (None, Some(other)),
                                        None => {
                                            self.err("SX-E03", format!("no representation named `{w}` before this beat"), to.span);
                                            (None, None)
                                        }
                                    },
                                    _ => {
                                        self.err("SX-E02", "a morph takes the shape of a named representation: `morph to square`", to.span);
                                        (None, None)
                                    }
                                },
                                _ => (Some(self.expr(to)), None),
                            };
                            let duration = duration.as_ref().map(|d| self.expr(d));
                            out.push(Action::Animate { target: id, property, to, word, duration, ease });
                        }
                        None => self.err("SX-E03", format!("no representation named `{}` before this beat", target.text), target.span),
                    }
                }
                ast::Action::Release(n) => match self.aliases.get(&n.text) {
                    Some(id) => out.push(Action::Release { target: id.clone() }),
                    None => self.err("SX-E03", format!("no representation named `{}` before this beat", n.text), n.span),
                },
                ast::Action::Bind(n, d, e) => match self.aliases.get(&n.text).cloned() {
                    Some(id) => {
                        let duration = d.as_ref().map(|d| self.expr(d));
                        let ease = self.ease(e.as_ref());
                        out.push(Action::Bind { target: id, duration, ease })
                    }
                    None => self.err("SX-E03", format!("no representation named `{}` before this beat", n.text), n.span),
                },
            }
        }
        out
    }
}

fn find_model<'m>(models: &'m [Model], n: &ast::Name, diags: &mut Vec<Diag>) -> Option<&'m Model> {
    let m = models.iter().find(|m| m.name == n.text);
    if m.is_none() {
        diags.push(Diag::new("SX-E03", format!("unknown model `{}`", n.text), n.span));
    }
    m
}

pub(crate) fn presentation(p: &ast::PresentationDecl, models: &[Model], spaces: &[Space], diags: &mut Vec<Diag>, map: &mut SourceMap) -> Option<Presentation> {
    let model = find_model(models, &p.model, diags)?;
    let id = p.name.text.clone();
    map.insert(id.clone(), p.span);
    let equations = model.equations.iter().map(|q| (q.name.clone(), q.id.clone())).collect();
    let mut px = PresCx { cx: ModelCx::scope(model, spaces, diags, map), equations, pres: &id, views: HashMap::new(), aliases: HashMap::new(), instruments: HashMap::new() };
    // Instruments first: views and observations read through them.
    let mut instruments = vec![];
    for it in &p.items {
        if let ast::PresItem::Instrument { name, args, span } = it {
            if px.instruments.insert(name.text.clone(), format!("{id}.instrument.{}", name.text)).is_some() {
                px.err("SX-E09", format!("instrument `{}` declared twice", name.text), name.span);
                continue;
            }
            instruments.push(px.instrument(name, args, *span));
        }
    }
    let mut views = vec![];
    for it in &p.items {
        if let ast::PresItem::View(v) = it {
            if px.views.insert(v.name.text.clone(), format!("{id}.view.{}", v.name.text)).is_some() {
                px.err("SX-E09", format!("view `{}` declared twice", v.name.text), v.name.span);
                continue;
            }
            if let Some(view) = px.view(v) {
                views.push(view);
            }
        }
    }
    let mut observations: Vec<Observation> = vec![];
    let mut permissions = vec![];
    let mut timeline = None;
    let mut layout = None;
    let mut title = None;
    let mut until = None;
    for it in &p.items {
        match it {
            ast::PresItem::Until(e) => {
                if until.is_some() {
                    px.err("SX-E09", "one `until` per presentation", e.span);
                }
                until = Some(px.cx.expr(e, &[]));
            }
            ast::PresItem::Title(t) => {
                if title.is_some() {
                    px.err("SX-E09", "one title per presentation", p.name.span);
                }
                title = Some(t.clone());
            }
            ast::PresItem::Layout(node) => {
                if layout.is_some() {
                    px.err("SX-E09", "one layout per presentation", p.name.span);
                }
                layout = px.layout(node, &mut HashMap::new());
            }
            ast::PresItem::Observe(obs) => {
                for o in obs {
                    if observations.iter().any(|x| x.name == o.name.text) {
                        px.err("SX-E09", format!("observation `{}` declared twice", o.name.text), o.name.span);
                        continue;
                    }
                    observations.push(px.observation(o, model));
                }
            }
            ast::PresItem::Permit { who, items } => {
                permissions.push(Permission { role: who.text.clone(), allows: items.iter().map(|i| i.text.clone()).collect() })
            }
            ast::PresItem::Timeline(scenes) => {
                if timeline.is_some() {
                    px.err("SX-E09", "one timeline per presentation", p.name.span);
                }
                let mut out = vec![];
                let mut beat_names: HashMap<String, Span> = HashMap::new();
                for s in scenes {
                    let sid = format!("{id}.scene.{}", s.name.text);
                    px.cx.map.insert(sid.clone(), s.span);
                    let mut beats = vec![];
                    for b in &s.beats {
                        if beat_names.insert(b.name.text.clone(), b.span).is_some() {
                            px.err("SX-E09", format!("beat `{}` declared twice", b.name.text), b.name.span);
                        }
                        let bid = format!("{id}.beat.{}", b.name.text);
                        px.cx.map.insert(bid.clone(), b.span);
                        let mut counts = HashMap::new();
                        let actions = px.actions(&b.actions, &bid, &mut counts, b.span);
                        beats.push(Beat { id: bid, name: b.name.text.clone(), actions });
                    }
                    out.push(Scene { id: sid, name: s.name.text.clone(), beats });
                }
                timeline = Some(Timeline { scenes: out });
            }
            ast::PresItem::View(_) | ast::PresItem::Instrument { .. } => {}
        }
    }
    Some(Presentation { id: id.clone(), name: id.clone(), model: model.id.clone(), observations, instruments, views, permissions, timeline, layout, notes: vec![], title, until })
}

pub(crate) fn run(
    r: &ast::RunDecl,
    models: &[Model],
    spaces: &[Space],
    presentations: &[Presentation],
    diags: &mut Vec<Diag>,
    map: &mut SourceMap,
) -> Option<RunCase> {
    let model = find_model(models, &r.model, diags)?;
    let pres = match &r.presentation {
        None => None,
        Some(n) => match presentations.iter().find(|p| p.name == n.text) {
            Some(p) if p.model == model.id => Some(p),
            Some(_) => {
                diags.push(Diag::new("SX-E08", format!("presentation `{}` is for another model", n.text), n.span));
                return None;
            }
            None => {
                diags.push(Diag::new("SX-E03", format!("unknown presentation `{}`", n.text), n.span));
                return None;
            }
        },
    };
    let id = r.name.text.clone();
    map.insert(id.clone(), r.span);
    let mut cx = ModelCx::scope(model, spaces, diags, map);
    let params = r.params.iter().map(|(n, v)| Override { binding: cx.binding(n), value: cx.expr(v, &[]) }).collect();
    let inputs = r.inputs.iter().map(|(n, v, at)| InputValue { binding: cx.binding(n), value: cx.expr(v, &[]), at: at.as_ref().map(|a| cx.expr(a, &[])) }).collect();
    let mut config = RunConfig::default();
    for (k, v) in &r.config {
        match k.text.as_str() {
            "solver" => match &v.kind {
                ExprKind::Name(s) if s == "dopri5" || s == "rk4" => config.solver = Some(s.clone()),
                _ => cx.err("SX-E03", "the solver is `dopri5` or `rk4` (RC section 16)", v.span),
            },
            "h" => config.h = Some(cx.expr(v, &[])),
            "rtol" => config.rtol = Some(cx.expr(v, &[])),
            "atol" => config.atol = Some(cx.expr(v, &[])),
            "seed" => match &v.kind {
                ExprKind::Num(x, None) if *x >= 0.0 && x.fract() == 0.0 && *x < 9.007_199_254_740_992e15 => config.seed = Some(*x as u64),
                _ => cx.err("SX-E03", "a seed is a whole number, 0 or more (RC-13.1)", v.span),
            },
            other => cx.err("SX-E03", format!("unknown run configuration `{other}`: `solver`, `h`, `rtol`, `atol`, `seed`"), k.span),
        }
    }
    let end = r.until.as_ref().map(|u| cx.expr(u, &[]));
    let mut learner = vec![];
    for s in &r.learner {
        let input = match &s.action {
            ast::LearnerAction::Continue => LearnerInput::Continue,
            // `press restart`: a runtime-control button, unless the model has an event of
            // that name (D-088, as `button(restart)`).
            ast::LearnerAction::Press(e) if EXPLORE_CONTROLS.contains(&e.text.as_str()) && !cx.events.contains_key(&e.text) => LearnerInput::Control { control: e.text.clone() },
            ast::LearnerAction::Press(e) => LearnerInput::Press { event: cx.event_id(e) },
            ast::LearnerAction::Set { control, value } => match control.as_slice() {
                [kind, target] => LearnerInput::SetControl { control: kind.text.clone(), binding: cx.binding(target), value: cx.expr(value, &[]) },
                _ => {
                    cx.err("SX-E02", "a learner input is written `set <control> <binding> = value`", s.span);
                    continue;
                }
            },
        };
        learner.push(LearnerStep { at: cx.expr(&s.at, &[]), input });
    }

    let observation = |n: &str| pres.and_then(|p| p.observations.iter().find(|o| o.name == n)).map(|o| o.id.clone());
    let beat = |n: &str| {
        pres.and_then(|p| p.timeline.as_ref())
            .and_then(|t| t.scenes.iter().flat_map(|s| &s.beats).find(|b| b.name == n))
            .map(|b| b.id.clone())
    };
    let subject = |cx: &mut ModelCx, e: &ast::Expr| -> Option<Subject> {
        match &e.kind {
            ExprKind::Name(n) => match observation(n) {
                Some(o) => Some(Subject::Observation { observation: o, index: None }),
                None => {
                    cx.err("SX-E03", format!("no observation `{n}` in the run's presentation"), e.span);
                    None
                }
            },
            ExprKind::Index(x, i) => match (&x.kind, &i.kind) {
                (ExprKind::Name(n), ExprKind::Num(k, None)) if *k >= 1.0 && k.fract() == 0.0 => match observation(n) {
                    Some(o) => Some(Subject::Observation { observation: o, index: Some(*k as usize) }),
                    None => {
                        cx.err("SX-E03", format!("no observation `{n}` in the run's presentation"), x.span);
                        None
                    }
                },
                _ => {
                    cx.err("SX-E02", "an index is a whole number from 1: `bounce_times[1]`", i.span);
                    None
                }
            },
            ExprKind::On(x, ev, micro) => Some(Subject::On { expr: cx.expr(x, &[]), event: cx.event_id(ev), microstep: *micro }),
            // `mean(ends)`, `deviation(ends)` (D-084).
            ExprKind::Call(f, args) if matches!(&f.kind, ExprKind::Name(n) if n == "mean" || n == "deviation") => {
                let ExprKind::Name(stat) = &f.kind else { unreachable!() };
                let statistic = if stat == "mean" { prismal_ir::present::Statistic::Mean } else { prismal_ir::present::Statistic::Deviation };
                match args.as_slice() {
                    [ast::Arg { value: ast::Expr { kind: ExprKind::Name(n), span }, .. }] => match observation(n) {
                        Some(o) => Some(Subject::Statistic { statistic, observation: o }),
                        None => {
                            cx.err("SX-E03", format!("no observation `{n}` in the run's presentation"), *span);
                            None
                        }
                    },
                    _ => {
                        cx.err("SX-E08", format!("`{stat}` takes the name of an observation: `{stat}(ends)`"), e.span);
                        None
                    }
                }
            }
            ExprKind::BeatTime { start, beat: b } => match beat(&b.text) {
                Some(id) if *start => Some(Subject::BeatStart { beat: id }),
                Some(id) => Some(Subject::BeatEnd { beat: id }),
                None => {
                    cx.err("SX-E03", format!("no beat `{}` in the run's presentation", b.text), b.span);
                    None
                }
            },
            _ => {
                cx.err("SX-E08", "an expectation is about an observation, `mean(obs)` or `deviation(obs)`, `(e on E)`, or `start of` / `end of` a beat", e.span);
                None
            }
        }
    };
    let mut expectations = vec![];
    for x in &r.expects {
        let n = expectations.len() + 1;
        let (check, span) = match x {
            ast::Expect::Outcome { words, span } => {
                let text: Vec<&str> = words.iter().map(|w| w.text.as_str()).collect();
                let outcome = match text.as_slice() {
                    ["initialization", "fails"] => Outcome::InitializationFails,
                    ["configuration", "rejected"] => Outcome::ConfigurationRejected,
                    _ => {
                        cx.err("SX-E02", "an outcome is `initialization fails` or `configuration rejected`", *span);
                        continue;
                    }
                };
                (Check::Outcome { outcome }, *span)
            }
            ast::Expect::In { expr, interval, span } => {
                let Some(subject) = subject(&mut cx, expr) else { continue };
                let (Bound::Value(lo), Bound::Value(hi)) = (&interval.lo, &interval.hi) else {
                    cx.err("SX-E08", "an expected interval has finite bounds", interval.span);
                    continue;
                };
                let (lo, hi) = (cx.expr(lo, &[]), cx.expr(hi, &[]));
                (Check::Within { subject, lo, hi, lo_closed: interval.lo_closed, hi_closed: interval.hi_closed }, *span)
            }
            ast::Expect::Compare { lhs, rhs, tol, span } => {
                let Some(subj) = subject(&mut cx, lhs) else { continue };
                let expected = match &rhs.kind {
                    ExprKind::List(items) => Operand::List {
                        items: items
                            .iter()
                            .map(|i| match &i.kind {
                                ExprKind::Name(n) if cx.events.contains_key(n) && !cx.bindings.contains_key(n) => Item::Event { event: cx.events[n].clone() },
                                _ => Item::Value { expr: cx.expr(i, &[]) },
                            })
                            .collect(),
                    },
                    ExprKind::On(..) | ExprKind::BeatTime { .. } => match subject(&mut cx, rhs) {
                        Some(s) => Operand::Subject { subject: s },
                        None => continue,
                    },
                    _ => Operand::Value { expr: cx.expr(rhs, &[]) },
                };
                let tolerance = match tol {
                    ast::Tolerance::None | ast::Tolerance::Exact => Tolerance::Exact,
                    ast::Tolerance::Abs(t) => Tolerance::Abs { tol: cx.expr(t, &[]) },
                    ast::Tolerance::Rel(t) => Tolerance::Rel { tol: cx.expr(t, &[]) },
                };
                (Check::Equal { subject: subj, expected, tolerance }, *span)
            }
        };
        let eid = format!("{id}.expect.{n}");
        cx.map.insert(eid.clone(), span);
        expectations.push(Expectation { id: eid, check });
    }
    Some(RunCase {
        id: id.clone(),
        name: id,
        model: model.id.clone(),
        presentation: pres.map(|p| p.id.clone()),
        params,
        inputs,
        config,
        end,
        learner,
        expectations,
    })
}
