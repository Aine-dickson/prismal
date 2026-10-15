//! Depth in views of three dimensions (D-104): bodies drawn from the back to the front, and
//! what lies behind an opaque body hidden.
//!
//! Every representation of such a view carries its geometry in model coordinates
//! (`RepFrame::geo`). Filled shapes (spheres, polygons, round shapes in a plane) are drawn
//! first, the farthest first. Lines, markers and texts follow in the author's order; the
//! pieces of lines that an opaque body hides are cut away, and markers and arrow tips behind
//! one are not drawn. Lines are never left behind a body that hides them, so drawing them over
//! every fill is right; translucent fills let what is behind them show anyway.

use crate::frame::{Geo, Look, RepFrame, Shape, ViewCtx};

/// A body that hides what lies behind it.
enum Body {
    Sphere { center: [f64; 3], radius: f64 },
    /// A flat polygon: its corners, its normal of length 1, and the two axes its corners are
    /// compared in (the normal's largest component dropped).
    Face { corners: Vec<[f64; 3]>, normal: [f64; 3], axes: (usize, usize) },
}

fn sub(a: &[f64; 3], b: &[f64; 3]) -> [f64; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

fn dot(a: &[f64; 3], b: &[f64; 3]) -> f64 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

fn lerp(a: &[f64; 3], b: &[f64; 3], s: f64) -> [f64; 3] {
    [a[0] + (b[0] - a[0]) * s, a[1] + (b[1] - a[1]) * s, a[2] + (b[2] - a[2]) * s]
}

impl Body {
    fn of(r: &RepFrame) -> Option<Body> {
        if !r.opaque {
            return None;
        }
        match &r.geo {
            Some(Geo::Sphere { center, radius }) if *radius > 0.0 => Some(Body::Sphere { center: *center, radius: *radius }),
            Some(Geo::Path { points, closed: true }) if points.len() >= 3 => {
                // Newell's normal: right for any flat polygon, convex or not.
                let mut n = [0.0; 3];
                for (i, a) in points.iter().enumerate() {
                    let b = &points[(i + 1) % points.len()];
                    n[0] += (a[1] - b[1]) * (a[2] + b[2]);
                    n[1] += (a[2] - b[2]) * (a[0] + b[0]);
                    n[2] += (a[0] - b[0]) * (a[1] + b[1]);
                }
                let len = dot(&n, &n).sqrt();
                if len == 0.0 {
                    return None;
                }
                let normal = [n[0] / len, n[1] / len, n[2] / len];
                let big = (0..3).max_by(|&i, &j| normal[i].abs().total_cmp(&normal[j].abs())).unwrap_or(2);
                let axes = match big {
                    0 => (1, 2),
                    1 => (0, 2),
                    _ => (0, 1),
                };
                Some(Body::Face { corners: points.clone(), normal, axes })
            }
            _ => None,
        }
    }

    /// The size of the body, for tolerances.
    fn size(&self) -> f64 {
        match self {
            Body::Sphere { radius, .. } => *radius,
            Body::Face { corners, .. } => corners.windows(2).map(|w| dot(&sub(&w[1], &w[0]), &sub(&w[1], &w[0])).sqrt()).fold(0.0, f64::max),
        }
    }

    /// Whether the body lies between point `p` and the viewer.
    fn hides(&self, look: &Look, p: &[f64; 3]) -> bool {
        let (w, far) = look.to_viewer(p);
        let eps = 1e-7 * self.size().max(1e-300);
        match self {
            Body::Sphere { center, radius } => {
                // Where `p + s w` meets the sphere: a point inside it, or with the sphere ahead
                // of it on the way to the viewer, is hidden.
                let m = sub(p, center);
                let b = dot(&m, &w);
                let c = dot(&m, &m) - radius * radius;
                let disc = b * b - c;
                if disc <= 0.0 {
                    return false;
                }
                let s = -b + disc.sqrt();
                s > eps && (-b - disc.sqrt()) < far
            }
            Body::Face { corners, normal, axes } => {
                let den = dot(normal, &w);
                if den.abs() < 1e-12 {
                    return false;
                }
                let s = dot(normal, &sub(&corners[0], p)) / den;
                if s <= eps || s >= far {
                    return false;
                }
                let h = [p[0] + s * w[0], p[1] + s * w[1], p[2] + s * w[2]];
                let (i, j) = *axes;
                let (x, y) = (h[i], h[j]);
                let mut inside = false;
                let n = corners.len();
                for k in 0..n {
                    let (a, b) = (&corners[k], &corners[(k + n - 1) % n]);
                    if (a[j] > y) != (b[j] > y) && x < (b[i] - a[i]) * (y - a[j]) / (b[j] - a[j]) + a[i] {
                        inside = !inside;
                    }
                }
                inside
            }
        }
    }
}

/// The pieces of segment `a b` no body hides, as fractions of the way from `a` to `b`.
/// `px`: the segment's length on the screen, which sets how finely it is examined.
fn visible(bodies: &[(&Body, usize)], me: Option<usize>, look: &Look, a: &[f64; 3], b: &[f64; 3], px: f64) -> Vec<(f64, f64)> {
    let seen = |s: f64| {
        let p = lerp(a, b, s);
        !bodies.iter().any(|(body, k)| Some(*k) != me && body.hides(look, &p))
    };
    let n = (px / 3.0).ceil().clamp(1.0, 400.0) as usize;
    let at: Vec<bool> = (0..=n).map(|k| seen(k as f64 / n as f64)).collect();
    if at.iter().all(|x| *x) {
        return vec![(0.0, 1.0)];
    }
    // Where visibility changes between two samples, found by halving.
    let edge = |lo: f64, hi: f64, from: bool| {
        let (mut lo, mut hi) = (lo, hi);
        for _ in 0..30 {
            let m = (lo + hi) / 2.0;
            if seen(m) == from {
                lo = m;
            } else {
                hi = m;
            }
        }
        (lo + hi) / 2.0
    };
    let mut out = vec![];
    let mut start = if at[0] { Some(0.0) } else { None };
    for k in 1..=n {
        let (s0, s1) = ((k - 1) as f64 / n as f64, k as f64 / n as f64);
        match (at[k - 1], at[k]) {
            (true, false) => {
                if let Some(s) = start.take() {
                    out.push((s, edge(s0, s1, true)));
                }
            }
            (false, true) => start = Some(edge(s0, s1, false)),
            _ => {}
        }
    }
    if let Some(s) = start {
        out.push((s, 1.0));
    }
    out
}

fn px_len(ctx: &ViewCtx, a: &[f64; 3], b: &[f64; 3]) -> f64 {
    let (p, q) = (ctx.to_view(a), ctx.to_view(b));
    (p[0] - q[0]).hypot(p[1] - q[1])
}

/// A path cut where bodies hide it: its points in model coordinates, the breaks of its shape,
/// and whether it closes on itself. Returns the points and breaks of what is seen.
fn cut_path(bodies: &[(&Body, usize)], me: Option<usize>, ctx: &ViewCtx, look: &Look, pts: &[[f64; 3]], breaks: &[usize], closed: bool) -> Option<(Vec<[f64; 2]>, Vec<usize>)> {
    let n = pts.len();
    let segs: Vec<(usize, usize)> = (0..n.saturating_sub(1)).filter(|i| !breaks.contains(&(i + 1))).map(|i| (i, i + 1)).chain(closed.then_some((n - 1, 0)).filter(|_| n > 2)).collect();
    let pieces: Vec<Vec<(f64, f64)>> = segs.iter().map(|&(i, j)| visible(bodies, me, look, &pts[i], &pts[j], px_len(ctx, &pts[i], &pts[j]))).collect();
    if pieces.iter().all(|p| p == &[(0.0, 1.0)]) {
        return None;
    }
    let mut out: Vec<[f64; 2]> = vec![];
    let mut brk = vec![];
    let mut joined = false;
    for (&(i, j), ps) in segs.iter().zip(&pieces) {
        for &(s0, s1) in ps {
            let (a, b) = (lerp(&pts[i], &pts[j], s0), lerp(&pts[i], &pts[j], s1));
            // A piece that starts where the last one ended continues it.
            if !(joined && s0 == 0.0) && !out.is_empty() {
                brk.push(out.len());
            }
            if !(joined && s0 == 0.0) {
                out.push(ctx.to_view(&a));
            }
            out.push(ctx.to_view(&b));
            joined = s1 == 1.0;
        }
        if ps.last().is_none_or(|p| p.1 < 1.0) {
            joined = false;
        }
    }
    Some((out, brk))
}

/// Arranges the representations of a view of three dimensions (D-104): fills from the back,
/// then the rest in the author's order, with what opaque bodies hide cut away. Members of
/// groups are arranged within their group.
pub fn arrange(ctx: &ViewCtx, reps: &mut Vec<RepFrame>) {
    let Some(look) = ctx.look() else { return };
    let mut all: Vec<&RepFrame> = vec![];
    fn walk<'a>(rs: &'a [RepFrame], out: &mut Vec<&'a RepFrame>) {
        for r in rs {
            out.push(r);
            if let Shape::Group { members } = &r.shape {
                walk(members, out);
            }
        }
    }
    walk(reps, &mut all);
    let owned: Vec<(Body, String)> = all.iter().filter_map(|r| Body::of(r).map(|b| (b, r.id.clone()))).collect();
    let bodies: Vec<(&Body, usize)> = owned.iter().enumerate().map(|(k, (b, _))| (b, k)).collect();
    cut(ctx, &look, &bodies, &owned, reps);
}

fn cut(ctx: &ViewCtx, look: &Look, bodies: &[(&Body, usize)], owned: &[(Body, String)], reps: &mut Vec<RepFrame>) {
    let hidden = |p: &[f64; 3], me: Option<usize>| bodies.iter().any(|(b, k)| Some(*k) != me && b.hides(look, p));
    let mut keep = vec![true; reps.len()];
    for (i, r) in reps.iter_mut().enumerate() {
        let me = owned.iter().position(|(_, id)| id == &r.id);
        if let Shape::Group { members } = &mut r.shape {
            cut(ctx, look, bodies, owned, members);
            continue;
        }
        if bodies.is_empty() {
            continue;
        }
        match (&r.geo, &mut r.shape) {
            (Some(Geo::Point(p)), Shape::Point { .. }) => keep[i] = !hidden(p, me),
            (Some(Geo::Path { points, .. }), Shape::Arrow { .. }) => keep[i] = !points.last().is_some_and(|p| hidden(p, me)),
            (Some(Geo::Path { points, .. }), Shape::Segment { .. }) => {
                if let Some((pts, brk)) = cut_path(bodies, me, ctx, look, points, &[], false) {
                    keep[i] = !pts.is_empty();
                    r.shape = Shape::Polyline { points: pts };
                    r.breaks = brk;
                }
            }
            (Some(Geo::Path { points, closed: false }), Shape::Polyline { .. }) => {
                if let Some((pts, brk)) = cut_path(bodies, me, ctx, look, points, &r.breaks, false) {
                    keep[i] = !pts.is_empty();
                    r.shape = Shape::Polyline { points: pts };
                    r.breaks = brk;
                }
            }
            (Some(Geo::Lines(lines)), Shape::Lines { lines: drawn, .. }) => {
                let mut out = vec![];
                for [a, b] in lines {
                    for (s0, s1) in visible(bodies, None, look, a, b, px_len(ctx, a, b)) {
                        out.push([ctx.to_view(&lerp(a, b, s0)), ctx.to_view(&lerp(a, b, s1))]);
                    }
                }
                *drawn = out;
            }
            // Bodies cover one another by being drawn from the back.
            _ => {}
        }
    }
    let mut k = 0;
    reps.retain(|_| {
        k += 1;
        keep[k - 1]
    });
    // Fills first, the farthest first; the rest keeps the author's order.
    let depth = |r: &RepFrame| match &r.geo {
        Some(Geo::Sphere { center, .. }) => Some(look.depth(center)),
        Some(Geo::Path { points, closed: true }) if !points.is_empty() => Some(points.iter().map(|p| look.depth(p)).sum::<f64>() / points.len() as f64),
        _ => None,
    };
    let mut fills: Vec<(f64, RepFrame)> = vec![];
    let mut rest = vec![];
    for r in reps.drain(..) {
        match depth(&r) {
            Some(d) => fills.push((d, r)),
            None => rest.push(r),
        }
    }
    fills.sort_by(|a, b| a.0.total_cmp(&b.0));
    reps.extend(fills.into_iter().map(|(_, r)| r));
    reps.extend(rest);
}
