//! Ray optics, continued (D-106): the nearest of several surfaces with `first ... by`,
//! polylines that end where a ray leaves, rays drawn to the edge of a view and back, colours
//! from a wavelength, and curved mirrors.

mod common;
use common::*;
use prismal_present::frame::{spectral, Frame, Shape};
use prismal_present::interact::{si_literal, Interactive};
use prismal_present::Program;
use prismal_runtime::{run, Config};
use prismal_syntax::{compile, format};

const SRC: &str = r#"use optics

space Plane = euclidean(2)

model Walls in Plane {
  object Mirror {
    param {
      place: Point    = origin
      n:    Vector<1> = (1, 0)
      half: Length    = 1 m
    }
  }
  param {
    src:   Point     = origin + (0 m, 0.7 m)
    aim:   Vector<1> = (1, 0)
    λ:     Length    = 650 nm
  }
  parts {
    walls: Mirror[3] {
      place = origin + (index * 1 m, 0 m)
      half = index * 0.5 m
    }
  }
  derived {
    s1:   Length = first(plane_distance(src, aim, b.place, b.n) for b in walls if |hit_plane(src, aim, b.place, b.n) - b.place| <= b.half and plane_distance(src, aim, b.place, b.n) > 0 m by plane_distance(src, aim, b.place, b.n))
    hit1: Point  = src + s1 * direction(aim)
    gone: Point  = src + first(plane_distance(src, aim, b.place, b.n) for b in walls if b.half > 10 m by plane_distance(src, aim, b.place, b.n)) * direction(aim)
  }
}

model Concave in Plane {
  param {
    f: Length = mirror_focus(0.4 m)
    h: Length = 0.05 m
  }
  derived {
    q:    Point     = origin + (0 m, h)
    back: Vector<1> = thin_mirror((1, 0), q, origin, (-1, 0), f)
    seen: Length    = (hit_plane(q, back, origin + (-f, 0 m), (1, 0)) - origin).y
  }
}

presentation Lab for Walls {
  view room: spatial(Plane, scale: 1 m -> 100 px, y: up) {
    polyline(src, hit1, gone) as path
    ray(hit1, (1, 0), back: dotted) as onward
    segment(src, hit1, color: λ) as red
    segment(src, hit1, color: 300 nm) as unseen
  }
  panel controls { slider(λ, range: [380 nm, 750 nm]) as knob }
}
"#;

fn value(prog: &Program, model: &str, name: &str) -> Vec<f64> {
    let cm = prog.model(model);
    let r = run(cm, Config::until(0.1));
    let v = &r.state_at(0.0)[cm.idx(&format!("{model}.{name}"))];
    prismal_present::flat(v)
}

/// The ray at height 0.7 m misses the first wall (half 0.5 m) and meets the second, at 2 m:
/// `first` takes the nearest wall that passes the filter, not the nearest of all.
#[test]
fn the_nearest_surface_a_ray_meets() {
    let prog = program(SRC);
    assert_eq!(value(&prog, "Walls", "s1"), vec![2.0]);
    let h = value(&prog, "Walls", "hit1");
    assert!((h[0] - 2.0).abs() < 1e-12 && (h[1] - 0.7).abs() < 1e-12, "{h:?}");
}

/// A concave mirror of radius 0.4 m has its focus 0.2 m in front: a ray parallel to the axis
/// comes back through it.
#[test]
fn a_curved_mirror_sends_parallel_rays_through_its_focus() {
    let prog = program(SRC);
    let s = value(&prog, "Concave", "seen");
    assert!(s[0].abs() < 1e-12, "{s:?}");
}

fn frame(prog: &Program) -> (Interactive, Frame) {
    let i = Interactive::new(prog, "Lab", Config::until(1.0)).unwrap_or_else(|d| panic!("{d:?}"));
    let f = i.frame();
    (i, f)
}

/// `gone` is not available (no wall is longer than 10 m): the polyline ends at `hit1`.
#[test]
fn a_polyline_ends_where_its_points_stop() {
    let prog = program(SRC);
    let (_, f) = frame(&prog);
    let path = f.rep("path").unwrap();
    match &path.shape {
        Shape::Polyline { points } => assert_eq!(points.len(), 2, "{points:?}"),
        s => panic!("{s:?}"),
    }
    assert!(path.text.contains("2 of its 3 points"), "{}", path.text);
}

/// A ray is a src and a direction on the screen, drawn to the edge by the medium; its
/// extension behind the src is dotted.
#[test]
fn a_ray_runs_to_the_edge_of_the_view() {
    let prog = program(SRC);
    let (_, f) = frame(&prog);
    match &f.rep("onward").unwrap().shape {
        Shape::Ray { from, toward, back } => {
            assert!((from[0] - 200.0).abs() < 1e-9 && (from[1] + 70.0).abs() < 1e-9, "{from:?}");
            assert!((toward[0] - 1.0).abs() < 1e-12 && toward[1].abs() < 1e-12, "{toward:?}");
            assert_eq!(back.as_deref(), Some("dotted"));
        }
        s => panic!("{s:?}"),
    }
}

/// `color: λ` draws in the colour of light of that wavelength, following its slider; light
/// no eye sees is gray.
#[test]
fn colours_from_a_wavelength() {
    let prog = program(SRC);
    assert_eq!(spectral(650e-9), "#ff0000");
    assert_eq!(spectral(530e-9).len(), 7);
    let (mut i, f) = frame(&prog);
    assert_eq!(f.rep("red").unwrap().color.as_deref(), Some("#ff0000"));
    assert_eq!(f.rep("unseen").unwrap().color.as_deref(), Some("#808080"));
    i.set_control("knob", si_literal(470e-9, &prismal_ir::Type::qty("L"))).unwrap();
    let blue = i.frame().rep("red").unwrap().color.clone().unwrap();
    assert_eq!(blue, spectral(470e-9));
    assert!(blue.starts_with("#00"), "{blue}");
}

#[test]
fn canonical_text_of_rays_and_light() {
    let text = format(&compile(SRC).unwrap().doc);
    for want in ["for b in walls if", "by plane_distance(src, aim, b.place, b.n))", "ray(hit1, (1, 0), back: dotted) as onward", "segment(src, hit1, color: λ) as red", "color: 300 nm"] {
        assert!(text.contains(want), "missing {want}:\n{text}");
    }
    assert_eq!(format(&compile(&text).unwrap().doc), text);
}

fn codes(src: &str) -> Vec<String> {
    match compile(src) {
        Err(ds) => ds.iter().map(|d| format!("{}: {}", d.code, d.message)).collect(),
        Ok(c) => match Program::new(c.doc) {
            Ok(_) => vec![],
            Err(ds) => ds.iter().map(|d| d.to_string()).collect(),
        },
    }
}

#[test]
fn rays_and_first_are_checked() {
    let m = "space Plane = euclidean(2)\nmodel M in Plane {\n object O { param { k: Real = 1 } }\n parts { os: O[2] }\n param { P: Point = origin }\n";
    let d = codes(&format!("{m} derived {{ a: Real = first(o.k for o in os) }} }}"));
    assert!(d.iter().any(|x| x.contains("SX-E02") && x.contains("key")), "{d:?}");
    let d = codes(&format!("{m} derived {{ a: Real = sum(o.k for o in os by o.k) }} }}"));
    assert!(d.iter().any(|x| x.contains("SX-E02") && x.contains("`by`")), "{d:?}");
    let d = codes(&format!("{m} derived {{ a: Real = first(o.k for o in os by P) }} }}"));
    assert!(d.iter().any(|x| x.contains("MK-E01") && x.contains("first")), "{d:?}");
    let d = codes(&format!("{m} }}\npresentation L for M {{ view v: spatial(Plane, scale: 1 m -> 10 px) {{ segment(P, P, color: 3 s) }} }}"));
    assert!(d.iter().any(|x| x.contains("PK-E04") && x.contains("wavelength")), "{d:?}");
    let d = codes(&format!("{m} }}\npresentation L for M {{ view v: spatial(Plane, scale: 1 m -> 10 px) {{ ray(P, (1, 0), back: wavy) }} }}"));
    assert!(d.iter().any(|x| x.contains("PK-E05") && x.contains("back")), "{d:?}");
}
