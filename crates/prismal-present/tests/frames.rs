//! Declared frames (D-105): components read in a frame that may turn, views drawn as seen
//! from a frame, and the forces of a turning frame from `mechanics`.

mod common;
use common::*;
use prismal_present::frame::{Frame, Shape};
use prismal_present::interact::Interactive;
use prismal_present::Program;
use prismal_runtime::{run, Config};
use prismal_syntax::{compile, format};

const SRC: &str = r#"use mechanics

space Plane = euclidean(2)
space Room = euclidean(3)

model Carousel in Plane {
  param { w: Frequency = 0.5 /s }
  state {
    pos: Point            = origin + (1 m, 0 m)
    vel: Vector<Velocity> = (0 m/s, 1 m/s)
  }
  derived {
    radial: Length   = pos.in(turning).x
    along:  Velocity = vel.in(turning).y
    turn:   Angle    = w * (t - t0)
  }
  frame turning(at: origin, x: (cos(turn), sin(turn)))
  flow {
    der(pos) = vel
    der(vel) = 0
  }
}

model Cabin in Room {
  param {
    P: Point = origin + (1 m, 2 m, 3 m)
    m: Mass  = 2 kg
    spin: Vector<1/T> = (0 /s, 0 /s, 1 /s)
    v: Vector<Velocity> = (1 m/s, 0 m/s, 0 m/s)
  }
  derived {
    a: Length = P.in(cabin).x
    b: Length = P.in(cabin).y
    c: Length = P.in(cabin).z
    sideways: Vector<Force> = coriolis(m, spin, v)
    outward:  Vector<Force> = centrifugal(m, spin, P - origin)
  }
  frame cabin(at: origin + (0 m, 0 m, 1 m), x: (0, 1, 0), y: (1, 0, 0))
}

presentation Ride for Carousel {
  view ground: spatial(Plane, scale: 1 m -> 100 px, y: up) {
    marker(pos) as ball
  }
  view seat: spatial(Plane, scale: 1 m -> 100 px, y: up, frame: turning) {
    marker(pos) as rider
    trace(pos every 0.5 s) as path
  }
}
"#;

fn num(prog: &Program, model: &str, name: &str, t: f64) -> f64 {
    let cm = prog.model(model);
    let r = run(cm, Config::until(t.max(0.1)));
    let mut x = [0.0; 1];
    r.state_at(t)[cm.idx(&format!("{model}.{name}"))].write_to(&mut x);
    x[0]
}

fn vec3(prog: &Program, model: &str, name: &str) -> [f64; 3] {
    let cm = prog.model(model);
    let r = run(cm, Config::until(0.1));
    let mut x = [0.0; 3];
    r.state_at(0.0)[cm.idx(&format!("{model}.{name}"))].write_to(&mut x);
    x
}

fn close(a: f64, b: f64, what: &str) {
    assert!((a - b).abs() < 1e-9, "{what}: {a} vs {b}");
}

/// The rider moves straight at 1 m/s from `(1 m, 0 m)`; the frame turns at 0.5 rad/s. At
/// 2 s the rider is at `(1 m, 2 m)`, which in the turned frame is `cos 1 + 2 sin 1` along
/// its first axis; the velocity along its second axis is `cos 1`.
#[test]
fn components_in_a_turning_frame() {
    let prog = program(SRC);
    close(num(&prog, "Carousel", "radial", 2.0), 1f64.cos() + 2.0 * 1f64.sin(), "radial");
    close(num(&prog, "Carousel", "along", 2.0), 1f64.cos(), "along");
    close(num(&prog, "Carousel", "radial", 0.0), 1.0, "at the start");
}

/// A frame of three dimensions: its first axis along `y`, its second along `x`, its third
/// their cross product, `-z`; its origin 1 m up.
#[test]
fn components_in_a_frame_of_three_dimensions() {
    let prog = program(SRC);
    close(num(&prog, "Cabin", "a", 0.0), 2.0, "along the first axis");
    close(num(&prog, "Cabin", "b", 0.0), 1.0, "along the second");
    close(num(&prog, "Cabin", "c", 0.0), -2.0, "along the third, downward");
}

/// `coriolis(m, w, v) = -2 m w × v` and `centrifugal(m, w, r) = -m w × (w × r)`.
#[test]
fn the_forces_of_a_turning_frame() {
    let prog = program(SRC);
    let s = vec3(&prog, "Cabin", "sideways");
    assert!(s[0].abs() < 1e-12 && (s[1] + 4.0).abs() < 1e-12 && s[2].abs() < 1e-12, "{s:?}");
    let o = vec3(&prog, "Cabin", "outward");
    assert!((o[0] - 2.0).abs() < 1e-12 && (o[1] - 4.0).abs() < 1e-12 && o[2].abs() < 1e-12, "away from the axis: {o:?}");
}

fn at(f: &Frame, name: &str) -> [f64; 2] {
    match &f.rep(name).unwrap_or_else(|| panic!("no {name}")).shape {
        Shape::Point { at } => *at,
        s => panic!("{s:?}"),
    }
}

/// From the ground the rider is at `(1 m, 2 m)`; from the seat, at its coordinates in the
/// turned frame. The path seen from the seat is read at each sample in the frame as it was
/// then: its first point is where the rider started, `(1 m, 0 m)`, and it curves.
#[test]
fn a_view_drawn_from_a_turning_frame() {
    let prog = program(SRC);
    let mut i = Interactive::new(&prog, "Ride", Config::until(2.0)).unwrap();
    i.t = 2.0;
    let f = i.frame();
    let g = at(&f, "ball");
    assert!((g[0] - 100.0).abs() < 1e-6 && (g[1] + 200.0).abs() < 1e-6, "{g:?}");
    let (c, s) = (1f64.cos(), 1f64.sin());
    let r = at(&f, "rider");
    let want = [100.0 * (c + 2.0 * s), -100.0 * (-s + 2.0 * c)];
    assert!((r[0] - want[0]).abs() < 1e-6 && (r[1] - want[1]).abs() < 1e-6, "{r:?} vs {want:?}");
    match &f.rep("path").unwrap().shape {
        Shape::Polyline { points } => {
            assert!((points[0][0] - 100.0).abs() < 1e-6 && points[0][1].abs() < 1e-6, "{:?}", points[0]);
            let last = points.last().unwrap();
            assert!((last[0] - want[0]).abs() < 1e-6 && (last[1] - want[1]).abs() < 1e-6, "{last:?}");
            // Not a straight line: the middle sample lies off the chord.
            let mid = points[points.len() / 2];
            let (a, b) = (points[0], *last);
            let cross = (b[0] - a[0]) * (mid[1] - a[1]) - (b[1] - a[1]) * (mid[0] - a[0]);
            assert!(cross.abs() > 100.0, "{cross}");
        }
        s => panic!("{s:?}"),
    }
}

#[test]
fn canonical_text_of_frames() {
    let text = format(&compile(SRC).unwrap().doc);
    for want in [
        "frame turning(at: origin, x: (cos(turn), sin(turn)))",
        "frame cabin(at: origin + (0 m, 0 m, 1 m), x: (0, 1, 0), y: (1, 0, 0))",
        "radial: Length   = pos.in(turning).x",
        "view seat: spatial(Plane, scale: 1 m -> 100 px, y: up, frame: turning) {",
    ] {
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
fn frames_are_checked() {
    let plane = "space Plane = euclidean(2)\nspace Room = euclidean(3)\n";
    let d = codes(&format!("{plane}model M in Plane {{ param {{ P: Point = origin }}\n frame f(at: origin, x: (1, 0), y: (0, 1)) }}"));
    assert!(d.iter().any(|m| m.contains("SX-E08") && m.contains("plane")), "{d:?}");
    let d = codes(&format!("{plane}model M in Room {{ param {{ P: Point = origin }}\n frame f(at: origin, x: (1, 0, 0)) }}"));
    assert!(d.iter().any(|m| m.contains("SX-E08") && m.contains("second axis")), "{d:?}");
    let d = codes(&format!("{plane}model M in Plane {{ param {{ P: Point = origin }}\n derived {{ a: Length = P.in(g).x }} }}"));
    assert!(d.iter().any(|m| m.contains("SX-E03") && m.contains("frame")), "{d:?}");
    let d = codes(&format!("{plane}model M in Plane {{ param {{ P: Point = origin }}\n frame f(at: origin, x: (1, 0))\n derived {{ a: Length = P.in(f) }} }}"));
    assert!(d.iter().any(|m| m.contains("SX-E02") && m.contains("components")), "{d:?}");
    let d = codes(&format!("{plane}model M in Plane {{ param {{ P: Point = origin }}\n frame f(at: 1 m, x: (1, 0))\n derived {{ a: Length = P.in(f).x }} }}"));
    assert!(d.iter().any(|m| m.contains("MK-E")), "{d:?}");
}
