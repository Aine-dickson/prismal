//! Views of three dimensions (D-104): perspective, angles that read the model, turns by a
//! timeline's camera, opaque bodies that hide what is behind them, round shapes in a plane,
//! groups turned about an axis, and drags across a plane.

mod common;
use common::*;
use prismal_present::frame::{Frame, Shape};
use prismal_present::interact::{si_literal, Interactive};
use prismal_present::timeline::{play, Medium};
use prismal_present::Program;
use prismal_runtime::Config;
use prismal_syntax::{compile, format};

const SRC: &str = r#"space Room = euclidean(3)

model Scene in Room {
  param {
    turn: Angle = 0 deg
    O: Point = origin
    A: Point = origin + (1 m, 1 m, 0 m)
    B: Point = origin + (-1 m, 1 m, 0 m)
    C: Point = origin + (0 m, 1 m, 0 m)
    X: Point = origin + (1 m, 0 m, 0 m)
    P: Point = origin + (0 m, 0 m, 1 m)
    M: Point = origin + (0 m, 0 m, 1 m)
    Front: Point = origin + (2 m, 0 m, 0 m)
    Behind: Point = origin + (-2 m, 0 m, 0 m)
    Aside: Point = origin + (-2 m, 2 m, 0 m)
  }
}

presentation Deep for Scene {
  view room: spatial(Room, scale: 1 m -> 100 px, azimuth: 0 deg, elevation: 0 deg, distance: 4 m) {
    marker(A) as a
    marker(B) as b
    marker(C) as c
    circle(O, 0.5 m) as ball
  }
}

presentation Turning for Scene {
  view room: spatial(Room, scale: 1 m -> 100 px, azimuth: turn, elevation: 0 deg) {
    marker(X) as x
  }
  panel controls { slider(turn, range: [0 deg, 90 deg]) as knob }
}

presentation Tour for Scene {
  view room: spatial(Room, scale: 1 m -> 100 px, azimuth: 0 deg, elevation: 0 deg) {
    marker(X) as x
  }
  timeline {
    scene s {
      beat turn { camera room azimuth 90 deg for 2 s ease linear }
      beat rest { hold; wait 1 s }
    }
  }
}

presentation Hidden for Scene {
  view room: spatial(Room, scale: 1 m -> 100 px, azimuth: 0 deg, elevation: 0 deg) {
    marker(Behind) as behind
    marker(Front) as front
    marker(Aside) as aside
    segment(origin + (-2 m, -2 m, 0 m), origin + (-2 m, 2 m, 0 m)) as back
    segment(origin + (2 m, -2 m, 0 m), origin + (2 m, 2 m, 0 m)) as fore
    circle(O, 1 m, opaque: true) as ball
  }
}

presentation Flat for Scene {
  view room: spatial(Room, scale: 1 m -> 100 px, azimuth: -90 deg, elevation: 90 deg) {
    circle(O, 1 m, normal: (0, 0, 1)) as ring
    ellipse(O, 2 m, 1 m) as oval
    arc(O, 1 m, from: 0 deg, to: 90 deg) as quarter
  }
}

presentation Turned for Scene {
  view room: spatial(Room, scale: 1 m -> 100 px, azimuth: 0 deg, elevation: 0 deg) {
    group(at: O, rotate: 90 deg, axis: (1, 0, 0)) {
      marker(P) as tipped
    }
  }
}

presentation Floor for Scene {
  view room: spatial(Room, scale: 1 m -> 100 px, azimuth: 0 deg, elevation: 45 deg) {
    marker(M) as m { on drag across (0, 0, 1) as p { propose M = p } }
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

fn close(a: [f64; 2], b: [f64; 2], what: &str) {
    assert!((a[0] - b[0]).abs() < 1e-6 && (a[1] - b[1]).abs() < 1e-6, "{what}: {a:?} vs {b:?}");
}

/// The viewer stands 4 m out along `x`: a point 1 m nearer is drawn 4/3 as far from the
/// centre, one 1 m farther 4/5 as far, and the plane through the origin keeps the scale.
#[test]
fn perspective_enlarges_what_is_near() {
    let prog = program(SRC);
    let f = frame(&prog, "Deep");
    close(at(&f, "a"), [100.0 * 4.0 / 3.0, 0.0], "nearer");
    close(at(&f, "b"), [100.0 * 4.0 / 5.0, 0.0], "farther");
    close(at(&f, "c"), [100.0, 0.0], "in the plane through the origin");
    let v = &f.views[0];
    let look = v.look.expect("a view of three dimensions says where it looks from");
    assert!(look[0].abs() < 1e-12 && look[1].abs() < 1e-12, "{look:?}");
}

/// `azimuth: turn` reads a parameter: moving its slider turns the view.
#[test]
fn a_view_turns_with_a_parameter() {
    let prog = program(SRC);
    let mut i = Interactive::new(&prog, "Turning", Config::until(1.0)).unwrap();
    close(at(&i.frame(), "x"), [0.0, 0.0], "x points at the viewer");
    i.set_control("knob", si_literal(90f64.to_radians(), &prismal_ir::Type::real())).unwrap();
    close(at(&i.frame(), "x"), [-100.0, 0.0], "from the side of y, x points left");
}

/// `camera room azimuth 90 deg for 2 s ease linear`: halfway, the view looks from 45 deg.
#[test]
fn a_timeline_camera_turns_a_view() {
    let prog = program(SRC);
    let pb = play(&prog, "Tour", Config::until(5.0), Medium::Video, vec![]).unwrap_or_else(|d| panic!("{d:?}"));
    close(at(&pb.frame(0.0, 0.0), "x"), [0.0, 0.0], "before");
    let s = 45f64.to_radians().sin();
    close(at(&pb.frame(1.0, 0.0), "x"), [-100.0 * s, 0.0], "halfway");
    close(at(&pb.frame(2.5, 0.0), "x"), [-100.0, 0.0], "after");
    let look = pb.frame(2.5, 0.0).views[0].look.unwrap();
    assert!((look[0] - 90f64.to_radians()).abs() < 1e-9, "{look:?}");
}

/// An opaque ball at the origin, seen from `+x`: a marker behind it is not drawn, one in
/// front or beside it is; a segment passing behind it is cut where it is hidden, one in
/// front is whole; the ball, a fill, is drawn before lines and markers.
#[test]
fn an_opaque_body_hides_what_is_behind_it() {
    let prog = program(SRC);
    let f = frame(&prog, "Hidden");
    assert!(f.rep("behind").is_none(), "hidden behind the ball");
    assert!(f.rep("front").is_some() && f.rep("aside").is_some());
    match &f.rep("back").unwrap().shape {
        Shape::Polyline { points } => {
            let b = &f.rep("back").unwrap().breaks;
            assert_eq!(points.len(), 4, "{points:?}");
            assert_eq!(b, &vec![2]);
            close(points[0], [-200.0, 0.0], "start");
            close(points[1], [-100.0, 0.0], "hidden from");
            close(points[2], [100.0, 0.0], "seen again");
            close(points[3], [200.0, 0.0], "end");
        }
        s => panic!("{s:?}"),
    }
    assert!(matches!(f.rep("fore").unwrap().shape, Shape::Segment { .. }));
    let ball = f.rep("ball").unwrap();
    assert!(ball.opaque);
    assert_eq!(f.views[0].reps[0].id, ball.id, "fills first");
    let json = serde_json::to_value(ball).unwrap();
    assert_eq!(json["opaque"], true);
    assert!(json.get("geo").is_none(), "model geometry stays out of the frame description");
}

/// From straight above, the usual map: a ring in the floor is a circle of 100 px, an
/// ellipse lies in the floor by default, an arc runs from `x` to `y`.
#[test]
fn round_shapes_lie_in_a_plane() {
    let prog = program(SRC);
    let f = frame(&prog, "Flat");
    match &f.rep("ring").unwrap().shape {
        Shape::Polygon { points } => {
            assert!(points.len() >= 64);
            assert!(points.iter().all(|p| (p[0].hypot(p[1]) - 100.0).abs() < 1e-6), "{points:?}");
        }
        s => panic!("{s:?}"),
    }
    match &f.rep("oval").unwrap().shape {
        Shape::Polygon { points } => {
            let xmax = points.iter().map(|p| p[0]).fold(f64::MIN, f64::max);
            let ymax = points.iter().map(|p| -p[1]).fold(f64::MIN, f64::max);
            assert!((xmax - 200.0).abs() < 1e-6 && (ymax - 100.0).abs() < 1e-6, "{xmax} {ymax}");
        }
        s => panic!("{s:?}"),
    }
    match &f.rep("quarter").unwrap().shape {
        Shape::Polyline { points } => {
            close(points[0], [100.0, 0.0], "from x");
            close(*points.last().unwrap(), [0.0, -100.0], "to y");
        }
        s => panic!("{s:?}"),
    }
    assert!(f.rep("ring").unwrap().text.contains("in the plane of normal (0, 0, 1)"), "{}", f.rep("ring").unwrap().text);
}

/// A group turned 90 deg about `x` takes `z` to `-y`: from the front, to the left.
#[test]
fn a_group_turns_about_an_axis() {
    let prog = program(SRC);
    let f = frame(&prog, "Turned");
    close(at(&f, "tipped"), [-100.0, 0.0], "z turned to -y");
}

/// Seen from 45 deg up, a pointer at the centre of the view lies on the line of sight
/// through the origin; across the floor at the marker's height, 1 m, that is `(1, 0, 1)`.
#[test]
fn a_drag_moves_across_a_plane() {
    let prog = program(SRC);
    let mut i = Interactive::new(&prog, "Floor", Config::until(1.0)).unwrap();
    i.pointer_down("m", None).unwrap();
    assert!(i.pointer_move([0.0, 0.0]));
    i.pointer_up().unwrap();
    let cm = prog.model("Scene");
    let mut p = [0.0; 3];
    i.session.current.state_at(i.t)[cm.idx("Scene.M")].write_to(&mut p);
    assert!((p[0] - 1.0).abs() < 1e-9 && p[1].abs() < 1e-9 && (p[2] - 1.0).abs() < 1e-9, "{p:?}");
    close(at(&i.frame(), "m"), [0.0, 0.0], "drawn under the pointer");
}

#[test]
fn canonical_text_of_views_of_three_dimensions() {
    let text = format(&compile(SRC).unwrap().doc);
    for want in [
        "azimuth: 0 deg, elevation: 0 deg, distance: 4 m)",
        "azimuth: turn, elevation: 0 deg)",
        "camera room azimuth 90 deg for 2 s ease linear",
        "circle(O, 1 m, opaque: true) as ball",
        "circle(O, 1 m, normal: (0, 0, 1)) as ring",
        "group(at: O, rotate: 90 deg, axis: (1, 0, 0)) {",
        "on drag across (0, 0, 1) as p { propose M = p }",
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
fn views_of_three_dimensions_check_their_new_words() {
    let both = "space Room = euclidean(3)\nspace Plane = euclidean(2)\nmodel M in Room { param { P: Point = origin } }\nmodel N in Plane { param { Q: Point = origin } }\n";
    let d = codes(&format!("{both}presentation L for N {{ view v: spatial(Plane, scale: 1 m -> 10 px, distance: 5 m) {{ marker(Q) }} }}"));
    assert!(d.iter().any(|m| m.contains("SX-E08") && m.contains("perspective")), "{d:?}");
    let d = codes(&format!("{both}presentation L for M {{ view v: spatial(Room, scale: 1 m -> 10 px, distance: 5 s) {{ marker(P) }} }}"));
    assert!(d.iter().any(|m| m.contains("PK-E04") && m.contains("distance")), "{d:?}");
    let d = codes(&format!("{both}presentation L for N {{ view v: spatial(Plane, scale: 1 m -> 10 px) {{ circle(Q, 1 m, normal: (0, 0, 1)) }} }}"));
    assert!(d.iter().any(|m| m.contains("PK-E05") && m.contains("normal")), "{d:?}");
    let d = codes(&format!("{both}presentation L for N {{ view v: spatial(Plane, scale: 1 m -> 10 px) {{ group(axis: (0, 0, 1)) {{ marker(Q) }} }} }}"));
    assert!(d.iter().any(|m| m.contains("PK-E05") && m.contains("axis")), "{d:?}");
    let d = codes(&format!("{both}presentation L for N {{ view v: spatial(Plane, scale: 1 m -> 10 px) {{ marker(Q) {{ on drag across (0, 0, 1) as p {{ propose Q = p }} }} }} }}"));
    assert!(d.iter().any(|m| m.contains("PK-E05") && m.contains("across")), "{d:?}");
    let d = codes(&format!("{both}presentation L for M {{ view v: spatial(Room, scale: 1 m -> 10 px) {{ segment(P, P, opaque: true) }} }}"));
    assert!(d.iter().any(|m| m.contains("SX-E08") && m.contains("opaque")), "{d:?}");
}
