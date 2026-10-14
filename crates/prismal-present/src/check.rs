//! Static checks of a presentation against its model.
//!
//! | Code | Meaning |
//! |---|---|
//! | PK-E01 | broken reference: an element the model or presentation does not have (PK-2.3) |
//! | PK-E02 | an expression does not check in the model's scope (the kernel's code is quoted) |
//! | PK-E03 | a control, inverse, intervention or explore beat targets a binding that is not intervenable, or requests an event that is not requestable (D-023, D-027) |
//! | PK-E04 | an encoding without a declared scale of the right dimension (PK-5.5) |
//! | PK-E05 | a representation whose sources do not suit its kind or view (PK-6.3) |
//! | PK-E06 | a representation kind the prototype does not implement |

use crate::frame::{compile_rep, rep_view, Projector, ViewCtx};
use crate::{number, PDiag};
use prismal_ir::present::{Action, Animated, Presentation, Schedule, Source};
use prismal_ir::{Expr, Trigger, Type};
use prismal_kernel::{check_intervention, compile_expr, CModel};

pub fn show_type(t: &Type) -> String {
    prismal_kernel::show(t)
}

fn expr(cm: &CModel, e: &Expr, want: Option<&Type>, element: &str, out: &mut Vec<PDiag>) {
    match compile_expr(cm, e, want) {
        Ok((_, t)) => {
            if let Some(w) = want {
                if &t != w {
                    out.push(PDiag { code: "PK-E02", message: format!("found {}, expected {}", show_type(&t), show_type(w)), element: element.into() });
                }
            }
        }
        Err(ds) => out.push(PDiag {
            code: "PK-E02",
            message: ds.iter().map(|d| format!("{}: {}", d.code, d.message)).collect::<Vec<_>>().join("; "),
            element: element.into(),
        }),
    }
}

fn event_exists(cm: &CModel, id: &str, element: &str, out: &mut Vec<PDiag>) -> bool {
    let ok = cm.ir.events.iter().any(|e| e.id == id);
    if !ok {
        out.push(PDiag { code: "PK-E01", message: format!("unknown event `{id}` (PK-2.3)"), element: element.into() });
    }
    ok
}

/// Checks a presentation against its model and returns every diagnostic.
pub fn check_presentation(cm: &CModel, p: &Presentation) -> Vec<PDiag> {
    let mut out = vec![];
    let time = Type::Instant { dim: prismal_ir::Dim::time() };
    let duration = Type::Quantity { dim: prismal_ir::Dim::time() };
    let projector = match Projector::new(cm, p) {
        Ok(pr) => Some(pr),
        Err(ds) => {
            out.extend(ds);
            None
        }
    };
    // A layout places views of this presentation, each once (PK-7.4a, D-063).
    if let Some(l) = &p.layout {
        let placed = l.views();
        for (k, v) in placed.iter().enumerate() {
            if !p.views.iter().any(|x| &x.id == *v) {
                out.push(PDiag { code: "PK-E01", message: format!("the layout places `{v}`, which is not a view of this presentation (PK-7.4a)"), element: p.id.clone() });
            } else if placed[..k].contains(v) {
                out.push(PDiag { code: "PK-E01", message: format!("the layout places `{v}` twice (PK-7.4a)"), element: p.id.clone() });
            }
        }
    }
    // A runtime control acts on a lab's session; a lesson's timeline is directed by the
    // learner through the player instead (PK-9.7, D-069). An explore beat may offer
    // `restart` and `reseed`, which start the learner's branch again (D-088).
    if p.timeline.is_some() {
        let mut ids = vec![];
        for v in &p.views {
            run_buttons(&v.representations, &mut ids);
        }
        for b in p.timeline.iter().flat_map(|t| t.scenes.iter().flat_map(|s| &s.beats)) {
            run_buttons_in(&b.actions, &mut ids);
        }
        for id in ids {
            out.push(PDiag { code: "PK-E05", message: "a runtime-control button belongs in a lab; in a lesson, an explore beat may offer `button(restart)` and `button(reseed)` (D-069, D-088)".into(), element: id });
        }
    }
    for o in &p.observations {
        match &o.source {
            Source::Expr { expr: e } => expr(cm, e, None, &o.id, &mut out),
            Source::EventLog { event: Some(ev), .. } => {
                event_exists(cm, ev, &o.id, &mut out);
            }
            Source::Diagnostics { element: Some(el) } => {
                let m = &cm.ir;
                let known = m.equations.iter().any(|x| &x.id == el)
                    || m.constraints.iter().any(|x| &x.id == el)
                    || m.events.iter().any(|x| &x.id == el)
                    || m.bindings.iter().any(|x| &x.id == el);
                if !known {
                    out.push(PDiag { code: "PK-E01", message: format!("unknown element `{el}` (PK-2.3)"), element: o.id.clone() });
                }
            }
            _ => {}
        }
        match &o.schedule {
            Schedule::Every { period } => expr(cm, period, Some(&duration), &o.id, &mut out),
            Schedule::At { time: t, .. } => expr(cm, t, Some(&time), &o.id, &mut out),
            Schedule::On { event, .. } => {
                event_exists(cm, event, &o.id, &mut out);
            }
            Schedule::Over { from, to, .. } => {
                expr(cm, from, Some(&time), &o.id, &mut out);
                if let Some(t) = to {
                    expr(cm, t, Some(&time), &o.id, &mut out);
                }
            }
            Schedule::Live | Schedule::Run => {}
        }
        // PK-3.7 (D-087): an instrument reads an expression at instants.
        if let Some(id) = &o.instrument {
            let problem = match (&o.source, &o.schedule) {
                (_, Schedule::Live) => Some("an instrument reads at instants: observe `every`, `at`, `on` or `over`, not `live`".to_string()),
                (Source::Expr { expr: e }, _) => match compile_expr(cm, e, None) {
                    Ok((_, ty)) => crate::measure::meter(cm, &p.instruments, Some(id), e, &ty).err(),
                    Err(_) => None,
                },
                _ => Some("an instrument reads values, not logs".to_string()),
            };
            if let Some(m) = problem {
                out.push(PDiag { code: "PK-E04", message: format!("{m} (PK-3.7)"), element: o.id.clone() });
            }
        }
    }
    // Every instrument is valid for some value, so that an unused one is still checked.
    for i in &p.instruments {
        for e in i.resolution.iter().chain(i.noise.iter()).chain(i.range.iter().flat_map(|(a, b)| [a, b])) {
            expr(cm, e, None, &i.id, &mut out);
        }
    }
    let Some(projector) = projector else { return out };
    let mut rep_ids: Vec<String> = vec![];
    for r in projector.views.iter().flat_map(|v| &v.2) {
        with_members(&r.rep, &mut rep_ids);
    }
    if let Some(tl) = &p.timeline {
        for b in tl.scenes.iter().flat_map(|s| &s.beats) {
            check_actions(cm, p, &projector, &b.actions, &b.id, &mut rep_ids, &mut out, &time, &duration);
        }
    }
    out
}

#[allow(clippy::too_many_arguments)]
fn check_actions(cm: &CModel, p: &Presentation, pr: &Projector, acts: &[Action], beat: &str, reps: &mut Vec<String>, out: &mut Vec<PDiag>, time: &Type, duration: &Type) {
    for a in acts {
        match a {
            Action::Reveal { view, reps: rs, duration: d, .. } => {
                if let Some(d) = d {
                    expr(cm, d, Some(duration), beat, out);
                }
                if let Some(v) = view {
                    if !pr.views.iter().any(|x| &x.0 == v) {
                        out.push(PDiag { code: "PK-E01", message: format!("unknown view `{v}`"), element: beat.into() });
                        continue;
                    }
                }
                let ctx = pr.ctx(view.as_deref());
                for r in rs {
                    if let Err(d) = compile_rep(cm, &ctx, r, &p.instruments) {
                        out.extend(d);
                    }
                    with_members(r, reps);
                }
            }
            Action::Camera { view, center, zoom, duration: d, .. } => {
                match pr.views.iter().find(|x| &x.0 == view) {
                    Some((_, ViewCtx::Spatial { space, .. }, _)) => {
                        if let Some(c) = center {
                            expr(cm, c, Some(&Type::Point { space: space.clone() }), beat, out);
                        }
                    }
                    // A plot's camera centers on a pair in the axes' dimensions (D-075).
                    Some((_, ViewCtx::Plot { dims, .. }, _)) => {
                        if let Some(c) = center {
                            let q = |d: &prismal_ir::Dim| Type::Quantity { dim: *d };
                            expr(cm, c, Some(&Type::Tuple { items: vec![q(&dims.0), q(&dims.1)] }), beat, out);
                        }
                    }
                    Some(_) => out.push(PDiag { code: "PK-E05", message: "a camera moves a spatial or plot view (D-042, D-075)".into(), element: beat.into() }),
                    None => out.push(PDiag { code: "PK-E01", message: format!("unknown view `{view}`"), element: beat.into() }),
                }
                if let Some(z) = zoom {
                    expr(cm, z, Some(&Type::real()), beat, out);
                    if number(cm, z).is_ok_and(|z| z <= 0.0) {
                        out.push(PDiag { code: "PK-E02", message: "a zoom is positive".into(), element: beat.into() });
                    }
                }
                if let Some(d) = d {
                    expr(cm, d, Some(duration), beat, out);
                }
            }
            Action::Show { view, reps: rs } => {
                if let Some(v) = view {
                    if !pr.views.iter().any(|x| &x.0 == v) {
                        out.push(PDiag { code: "PK-E01", message: format!("unknown view `{v}`"), element: beat.into() });
                        continue;
                    }
                }
                let ctx = pr.ctx(view.as_deref());
                for r in rs {
                    if let Err(d) = compile_rep(cm, &ctx, r, &p.instruments) {
                        out.extend(d);
                    }
                    with_members(r, reps);
                }
            }
            Action::Highlight { target } | Action::Hide { target, .. } => {
                if !reps.contains(target) {
                    out.push(PDiag { code: "PK-E01", message: format!("unknown representation `{target}`"), element: beat.into() });
                }
                if let Action::Hide { duration: Some(d), .. } = a {
                    expr(cm, d, Some(duration), beat, out);
                }
            }
            Action::Narrate { duration: Some(d), .. } | Action::Wait { duration: d } => expr(cm, d, Some(duration), beat, out),
            Action::Narrate { .. } | Action::Hold | Action::Reset | Action::Restart | Action::Branch => {}
            Action::Run { rate } => {
                expr(cm, rate, Some(&Type::real()), beat, out);
                if number(cm, rate).is_ok_and(|r| r < 0.0) {
                    out.push(PDiag { code: "PK-E02", message: "a playback rate is not negative (PK-8.2)".into(), element: beat.into() });
                }
            }
            Action::Seek { time: t } => expr(cm, t, Some(time), beat, out),
            Action::Intervene { ops } => {
                if let Err(ds) = check_intervention(cm, ops) {
                    out.extend(ds.into_iter().map(|d| PDiag { code: "PK-E03", message: format!("{}: {}", d.code, d.message), element: beat.into() }));
                }
            }
            Action::Request { event, payload } => {
                if event_exists(cm, event, beat, out) {
                    let ev = cm.ir.events.iter().find(|e| &e.id == event).unwrap();
                    if !matches!(ev.trigger, Trigger::Request) {
                        out.push(PDiag { code: "PK-E03", message: format!("`{event}` is not declared `on request` (D-027)"), element: beat.into() });
                    }
                    // D-050: a request supplies the payload the event declares, of its type.
                    match (&ev.payload, payload) {
                        (Some(p), Some(v)) => expr(cm, v, Some(&p.ty), beat, out),
                        (Some(p), None) => out.push(PDiag { code: "PK-E02", message: format!("`request {}` supplies its payload `{}`", ev.name, p.name), element: beat.into() }),
                        (None, Some(_)) => out.push(PDiag { code: "PK-E02", message: format!("`{}` declares no payload", ev.name), element: beat.into() }),
                        (None, None) => {}
                    }
                }
            }
            Action::WaitUntil { event } => {
                event_exists(cm, event, beat, out);
            }
            Action::Explore { limit, keep, controls, fallback } => {
                if let Some(l) = limit {
                    expr(cm, l, Some(duration), beat, out);
                }
                for k in keep {
                    match cm.index.get(k) {
                        Some(&i) if cm.bindings[i].intervenable => {}
                        _ => out.push(PDiag { code: "PK-E03", message: format!("`{k}` cannot be kept: not an intervenable binding (PK-9.9)"), element: beat.into() }),
                    }
                }
                for r in controls {
                    if let Err(d) = compile_rep(cm, &ViewCtx::Panel, r, &p.instruments) {
                        out.extend(d);
                    }
                    with_members(r, reps);
                }
                check_actions(cm, p, pr, fallback, beat, reps, out, time, duration);
            }
            Action::Sequence { actions } => check_actions(cm, p, pr, actions, beat, reps, out, time, duration),
            Action::WaitLearner { limit, fallback } => {
                if let Some(l) = limit {
                    expr(cm, l, Some(duration), beat, out);
                }
                check_actions(cm, p, pr, fallback, beat, reps, out, time, duration);
            }
            Action::Animate { target, property, to, word, duration: d, .. } => {
                if !known(reps, target, beat, out) {
                    continue;
                }
                if let Some(d) = d {
                    expr(cm, d, Some(duration), beat, out);
                }
                let diag = |out: &mut Vec<PDiag>, code: &'static str, message: String| out.push(PDiag { code, message, element: beat.into() });
                // Words: a palette color, a line style, a representation to morph into (D-075).
                match (property, word.as_deref()) {
                    (Animated::Color, Some(w)) if !crate::frame::COLORS.contains(&w) => {
                        diag(out, "PK-E02", format!("unknown color `{w}`: {} (D-061)", crate::frame::COLORS.join(", ")));
                    }
                    (Animated::Line, Some(w)) if !crate::frame::LINES.contains(&w) => {
                        diag(out, "PK-E02", format!("unknown line style `{w}`: {} (D-061)", crate::frame::LINES.join(", ")));
                    }
                    (Animated::Morph, Some(w)) => {
                        if known(reps, w, beat, out) && (rep_view(p, w).flatten().is_none() || rep_view(p, target).flatten().is_none()) {
                            diag(out, "PK-E05", "a morph changes a drawn shape into another drawn shape (D-075)".into());
                        }
                    }
                    _ => {}
                }
                let Some(to) = to else { continue };
                match property {
                    Animated::Color | Animated::Line | Animated::Morph => {}
                    Animated::Scale => {
                        expr(cm, to, Some(&Type::real()), beat, out);
                        if number(cm, to).is_ok_and(|x| x <= 0.0) {
                            diag(out, "PK-E02", "a scale is positive (D-075)".into());
                        }
                    }
                    Animated::Opacity => {
                        expr(cm, to, Some(&Type::real()), beat, out);
                        if number(cm, to).is_ok_and(|x| !(0.0..=1.0).contains(&x)) {
                            out.push(PDiag { code: "PK-E02", message: "an opacity is from 0 to 1 (D-068)".into(), element: beat.into() });
                        }
                    }
                    // An offset is a vector of the space of the view the representation is
                    // drawn in, of dimension length.
                    Animated::Offset => match rep_view(p, target).flatten().map(|v| pr.ctx(Some(&v))) {
                        Some(ViewCtx::Spatial { space, .. }) => expr(cm, to, Some(&Type::Vector { space, dim: prismal_ir::Dim::length() }), beat, out),
                        _ => out.push(PDiag { code: "PK-E05", message: format!("`{target}` is not drawn in a spatial view; only those take an offset (D-068)"), element: beat.into() }),
                    },
                }
            }
            Action::Release { target } | Action::Bind { target, duration: None, .. } => {
                known(reps, target, beat, out);
            }
            Action::Bind { target, duration: Some(d), .. } => {
                known(reps, target, beat, out);
                expr(cm, d, Some(duration), beat, out);
            }
        }
    }
}

/// Runtime-control buttons among representations (D-069).
fn run_buttons(reps: &[prismal_ir::present::Rep], out: &mut Vec<String>) {
    run_buttons_but(reps, &[], out)
}

/// Runtime-control buttons among `reps`, except those whose control is in `allowed`.
fn run_buttons_but(reps: &[prismal_ir::present::Rep], allowed: &[&str], out: &mut Vec<String>) {
    for r in reps {
        if let (true, Some(prismal_ir::present::Arg::Word { word })) = (r.kind == "button", r.sources.first()) {
            if !allowed.contains(&word.as_str()) {
                out.push(r.id.clone());
            }
        }
        run_buttons_but(&r.members, allowed, out);
    }
}

fn run_buttons_in(acts: &[Action], out: &mut Vec<String>) {
    for a in acts {
        match a {
            Action::Show { reps, .. } | Action::Reveal { reps, .. } => run_buttons(reps, out),
            Action::Explore { controls, fallback, .. } => {
                run_buttons_but(controls, &["restart", "reseed"], out);
                run_buttons_in(fallback, out);
            }
            Action::Sequence { actions } | Action::WaitLearner { fallback: actions, .. } => run_buttons_in(actions, out),
            _ => {}
        }
    }
}

/// Whether a timeline action's target is a representation shown so far (PK-2.3).
fn known(reps: &[String], target: &str, beat: &str, out: &mut Vec<PDiag>) -> bool {
    let ok = reps.iter().any(|r| r == target);
    if !ok {
        out.push(PDiag { code: "PK-E01", message: format!("unknown representation `{target}`"), element: beat.into() });
    }
    ok
}

/// A representation's identity and those of a group's members (D-043).
fn with_members(r: &prismal_ir::present::Rep, out: &mut Vec<String>) {
    out.push(r.id.clone());
    for m in &r.members {
        with_members(m, out);
    }
}
