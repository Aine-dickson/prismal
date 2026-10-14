//! Spaces of three dimensions (D-100): `cross`, motion in a room, and views of three
//! dimensions projected from an azimuth and an elevation, with `z` up.

mod common;
use common::*;
use prismal_present::frame::{Frame, Shape};
use prismal_present::interact::Interactive;
use prismal_present::Program;
use prismal_runtime::{run, Config, RunStatus};
use prismal_syntax::{compile, format};
use std::f64::consts::PI;

const SRC: &str = r#"space Room = euclidean(3)

model Cyclotron in Room {
  param {
    q: Charge = 1 C
    m: Mass   = 2 kg
    B: Vector<M/T^2/I> = (0 kg/s^2/A, 0 kg/s^2/A, 0.5 kg/s^2/A)
  }
  state {
    pos: Point            = origin
    vel: Vector<Velocity> = (1 m/s, 0 m/s, 0.2 m/s)
  }
  derived { force: Vector<Force> = q * cross(vel, B) }
  flow {
    der(pos) = vel
    der(vel) = force / m
  }
}

model Corner in Room {
  param {
    A: Point = origin + (1 m, 0 m, 0 m)
    B: Point = origin + (0 m, 1 m, 0 m)
    C: Point = origin + (0 m, 0 m, 1 m)
  }
}

presentation Look for Corner {
  view room: spatial(Room, scale: 1 m -> 100 px, azimuth: 0 deg, elevation: 0 deg) {
    axes
    grid
    marker(A) as a
    marker(B) as b
    marker(C) as c
    segment(A, B) as ab
    circle(C, 0.5 m) as ball
    arrow(B - A, from: A, scale: 1 m -> 100 px) as along
  }
}

presentation Default for Corner {
  view room: spatial(Room, scale: 1 m -> 100 px) {
    marker(C) as c
    marker(A) as a
  }
}

presentation Turned for Corner {
  view room: spatial(Room, scale: 1 m -> 100 px, azimuth: 90 deg, elevation: 90 deg) {
    marker(A) as a
    marker(B) as b
  }
}
"#;

fn frame(prog: &Program, pres: &str) -> Frame {
    Interactive::new(prog, pres, Config::until(1.0)).unwrap_or_else(|d| panic!("{d:?}")).frame()
}

fn at(f: &Frame, name: &str) -> [f64; 2] {
    match &f.rep(name).unwrap_or_else(|| panic!("no {name}")).shape {
        Shape::Point { at } => *at,
        s => panic!("{s:?}"),
    }
}

fn of_kind<'a>(f: &'a Frame, kind: &str) -> &'a prismal_present::frame::RepFrame {
    f.views.iter().flat_map(|v| &v.reps).find(|r| r.kind == kind).unwrap_or_else(|| panic!("no {kind}"))
}

fn close(a: [f64; 2], b: [f64; 2], what: &str) {
    assert!((a[0] - b[0]).abs() < 1e-9 && (a[1] - b[1]).abs() < 1e-9, "{what}: {a:?} vs {b:?}");
}

/// A charge in a magnetic field along `z`, `F = q v × B`: it circles at the cyclotron period
/// `2 π m / (q B)` while drifting along the field.
#[test]
fn a_charge_spirals_in_a_magnetic_field() {
    let prog = program(SRC);
    let cm = prog.model("Cyclotron");
    let period = 2.0 * PI * 2.0 / 0.5;
    let r = run(cm, Config::until(period));
    assert!(matches!(r.status, RunStatus::Completed), "{:?}", r.status);
    let mut p = [0.0; 3];
    r.state_at(period)[cm.idx("Cyclotron.pos")].write_to(&mut p);
    assert!(p[0].abs() < 1e-5 && p[1].abs() < 1e-5 && (p[2] - 0.2 * period).abs() < 1e-9, "{p:?}");
    // Half a turn later the charge is on the far side of its circle, of radius `m v / (q B)`.
    let mut h = [0.0; 3];
    r.state_at(period / 2.0)[cm.idx("Cyclotron.pos")].write_to(&mut h);
    let radius = 2.0 * 1.0 / 0.5;
    assert!(h[0].abs() < 1e-5 && (h[1].abs() - 2.0 * radius).abs() < 1e-5, "{h:?}");
    // The force is at right angles to the velocity: speed is kept.
    for c in &r.committed {
        let mut v = [0.0; 3];
        c.vals[cm.idx("Cyclotron.vel")].write_to(&mut v);
        let s = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
        assert!((s - (1.04f64).sqrt()).abs() < 1e-6, "{s} at {}", c.t);
    }
}

/// From azimuth 0 and elevation 0 the viewer looks along `-x`: `y` is to the right, `z` up.
#[test]
fn a_view_from_the_front() {
    let prog = program(SRC);
    let f = frame(&prog, "Look");
    close(at(&f, "a"), [0.0, 0.0], "x points at the viewer");
    close(at(&f, "b"), [100.0, 0.0], "y to the right");
    close(at(&f, "c"), [0.0, -100.0], "z up");
    match &f.rep("ab").unwrap().shape {
        Shape::Segment { from, to } => {
            close(*from, [0.0, 0.0], "segment from");
            close(*to, [100.0, 0.0], "segment to");
        }
        s => panic!("{s:?}"),
    }
    match &f.rep("along").unwrap().shape {
        Shape::Arrow { from, to } => {
            close(*from, [0.0, 0.0], "arrow from");
            close(*to, [100.0, 0.0], "arrow to");
        }
        s => panic!("{s:?}"),
    }
    // A circle is a sphere's outline: a disc facing the viewer.
    match &f.rep("ball").unwrap().shape {
        Shape::Ellipse { center, radii, closed, .. } => {
            close(*center, [0.0, -100.0], "sphere centre");
            assert!((radii[0] - 50.0).abs() < 1e-9 && (radii[1] - 50.0).abs() < 1e-9 && *closed);
        }
        s => panic!("{s:?}"),
    }
    // Axes and grid are projected lines: three axes with labels, a grid in the floor.
    match &of_kind(&f, "axes").shape {
        Shape::Lines { lines, labels } => {
            assert_eq!(lines.len(), 3);
            assert_eq!(labels.iter().map(|l| l.text.as_str()).collect::<Vec<_>>(), ["x (m)", "y (m)", "z (m)"]);
            close(lines[2][1], [0.0, -160.0], "z axis up");
        }
        s => panic!("{s:?}"),
    }
    assert!(matches!(&of_kind(&f, "grid").shape, Shape::Lines { lines, .. } if !lines.is_empty()));
    // The text alternative names all three coordinates.
    assert!(f.rep("c").unwrap().text.contains("x = 0 m, y = 0 m, z = 1 m"), "{}", f.rep("c").unwrap().text);
}

/// Without `azimuth` and `elevation`, a view looks from 30 deg and 20 deg.
#[test]
fn the_default_look() {
    let prog = program(SRC);
    let f = frame(&prog, "Default");
    let (a, e) = (30f64.to_radians(), 20f64.to_radians());
    close(at(&f, "c"), [0.0, -100.0 * e.cos()], "z up, foreshortened");
    close(at(&f, "a"), [-100.0 * a.sin(), 100.0 * e.sin() * a.cos()], "x toward the viewer and left");
}

/// From straight above (elevation 90 deg) at azimuth 90 deg the viewer stands on the side of
/// `y`, looking down: `x` points left and `y` down the screen. At azimuth -90 deg it would be
/// the usual map, `x` right and `y` up.
#[test]
fn a_view_from_above() {
    let prog = program(SRC);
    let f = frame(&prog, "Turned");
    close(at(&f, "a"), [-100.0, 0.0], "x to the left");
    close(at(&f, "b"), [0.0, 100.0], "y down the screen");
}

#[test]
fn canonical_text_of_a_view_of_three_dimensions() {
    let text = format(&compile(SRC).unwrap().doc);
    assert!(text.contains("view room: spatial(Room, scale: 1 m -> 100 px, azimuth: 0 deg, elevation: 0 deg) {"), "{text}");
    assert!(text.contains("view room: spatial(Room, scale: 1 m -> 100 px) {"), "{text}");
    assert!(text.contains("force: Vector<Force> = q * cross(vel, B)"), "{text}");
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
fn views_of_three_dimensions_are_checked() {
    let room = "space Room = euclidean(3)\nspace Plane = euclidean(2)\nmodel M in Room { param { P: Point = origin } }\n";
    let d = codes(&format!("{room}presentation L for M {{ view v: spatial(Room, scale: 1 m -> 10 px, y: up) {{ marker(P) }} }}"));
    assert!(d.iter().any(|m| m.contains("SX-E08") && m.contains("`z` up")), "{d:?}");
    let d = codes(&format!("{room}presentation L for M {{ view v: spatial(Plane, scale: 1 m -> 10 px, azimuth: 10 deg) {{ axes }} }}"));
    assert!(d.iter().any(|m| m.contains("SX-E08") && m.contains("three dimensions")), "{d:?}");
    let d = codes(&format!("{room}presentation L for M {{ view v: spatial(Room, scale: 1 m -> 10 px) {{ ellipse(P, 1 m, 2 m) }} }}"));
    assert!(d.iter().any(|m| m.contains("PK-E05") && m.contains("lies in a plane")), "{d:?}");
    let d = codes(&format!("{room}presentation L for M {{ view v: spatial(Room, scale: 1 m -> 10 px) {{ group(at: P) {{ marker(P) }} }} }}"));
    assert!(d.iter().any(|m| m.contains("PK-E05") && m.contains("group")), "{d:?}");
    // `cross` takes two vectors of a space of three dimensions.
    let d = codes("space Plane = euclidean(2)\nmodel M in Plane { param { a: Vector<Length> = (1 m, 0 m) }\n derived { c: Vector<L^2> = cross(a, a) } }");
    assert!(d.iter().any(|m| m.contains("MK-E01") && m.contains("three dimensions")), "{d:?}");
}
