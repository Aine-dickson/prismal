//! Circles, ellipses and arcs (PK-6.3c): exact elliptical arcs in view coordinates, in views
//! with `y` up and down and in groups, drawn by `reveal draw`, formatted, and checked.

mod common;
use common::*;
use prismal_present::frame::{Frame, Shape};
use prismal_present::interact::Interactive;
use prismal_present::timeline::{play, Medium};
use prismal_present::Program;
use prismal_runtime::Config;
use std::f64::consts::{PI, TAU};

const SRC: &str = "
space Plane = euclidean(2)
model Shapes in Plane {
  param {
    r: Length = 1 m
    turn: Angle = 90 deg
  }
  state { c: Point = origin + (2 m, 1 m) }
}
presentation Up for Shapes {
  view up: spatial(Plane, scale: 1 m -> 100 px, y: up) {
    circle(c, r) as disc
    arc(c, r, from: 0 deg, to: turn) as quarter
    ellipse(c, 2 m, 1 m, rotate: 30 deg) as oval
    group(at: origin + (1 m, 0 m), rotate: 90 deg, scale: 2) as g {
      arc(origin, 0.5 m, from: 0 deg, to: 90 deg) as inner
    }
  }
  timeline {
    scene s {
      beat a { reveal draw for 2 s in up { circle(c, 0.5 m) as drawn } }
    }
  }
}
presentation Down for Shapes {
  view down: spatial(Plane, scale: 1 m -> 100 px, y: down) {
    arc(c, r, from: 0 deg, to: turn) as quarter
  }
}
";

fn frame(prog: &Program, pres: &str) -> Frame {
    Interactive::new(prog, pres, Config::until(1.0)).unwrap_or_else(|d| panic!("{d:?}")).frame()
}

fn shape(f: &Frame, name: &str) -> Shape {
    let r = f.rep(name).unwrap_or_else(|| panic!("no {name}"));
    match &r.shape {
        Shape::Group { members } => members[0].shape.clone(),
        s => s.clone(),
    }
}

fn close(a: [f64; 2], b: [f64; 2], what: &str) {
    assert!((a[0] - b[0]).abs() < 1e-9 && (a[1] - b[1]).abs() < 1e-9, "{what}: {a:?} vs {b:?}");
}

#[test]
fn circles_ellipses_and_arcs() {
    let prog = program(SRC);
    let f = frame(&prog, "Up");
    let to = |p: [f64; 2]| [100.0 * p[0], -100.0 * p[1]]; // y up: view y is -100 y

    // A circle: whole, radius in view pixels, around the centre.
    let disc = shape(&f, "disc");
    let Shape::Ellipse { center, radii, closed, sweep, .. } = disc else { panic!("{disc:?}") };
    close(center, to([2.0, 1.0]), "centre");
    assert_eq!(radii, [100.0, 100.0]);
    assert!(closed && (sweep.abs() - TAU).abs() < 1e-12);
    assert_eq!(f.rep("disc").unwrap().text, "circle disc around x = 2 m, y = 1 m, radius 1 m");

    // An arc from 0 to 90 degrees, counterclockwise in the model: from the right of the
    // centre to above it, which is up on the screen.
    let q = shape(&f, "quarter");
    let pts = q.curve_points(8);
    close(pts[0], to([3.0, 1.0]), "arc starts at 0 deg");
    close(pts[8], to([2.0, 2.0]), "arc ends at 90 deg");
    close(pts[4], to([2.0 + (PI / 4.0).cos(), 1.0 + (PI / 4.0).sin()]), "arc passes 45 deg");
    let Shape::Ellipse { closed, .. } = q else { panic!() };
    assert!(!closed);
    assert_eq!(f.rep("quarter").unwrap().text, "arc quarter around x = 2 m, y = 1 m, radius 1 m, from 0 deg to 90 deg");

    // An ellipse turned by 30 degrees: its long axis points up and right.
    let oval = shape(&f, "oval");
    let a = 30f64.to_radians();
    close(oval.curve_points(4)[0], to([2.0 + 2.0 * a.cos(), 1.0 + 2.0 * a.sin()]), "end of the long axis");
    close(oval.curve_points(4)[1], to([2.0 - a.sin(), 1.0 + a.cos()]), "end of the short axis");

    // In a group placed at (1, 0), turned by 90 degrees and scaled by 2: radius 1 m, and the
    // arc turned a quarter, from straight up to straight left of the group's origin.
    let inner = shape(&f, "g");
    let Shape::Ellipse { radii, .. } = inner else { panic!("{inner:?}") };
    assert_eq!(radii, [100.0, 100.0]);
    let p = inner.curve_points(2);
    close(p[0], to([1.0, 1.0]), "turned start");
    close(p[2], to([0.0, 0.0]), "turned end");

    // The same arc in a view with y down turns the other way on the screen.
    let f = frame(&prog, "Down");
    let pts = shape(&f, "quarter").curve_points(8);
    close(pts[0], [300.0, 100.0], "y down: start");
    close(pts[8], [200.0, 200.0], "y down: end below the centre on the screen");
}

#[test]
fn a_circle_is_drawn_along_its_perimeter() {
    let prog = program(SRC);
    let pb = play(&prog, "Up", Config::until(5.0), Medium::Interactive, vec![]).unwrap();
    let f = pb.frame(0.5, 0.1);
    let drawn = f.rep("drawn").unwrap();
    assert!(matches!(drawn.shape, Shape::Ellipse { .. }));
    let k = drawn.drawn.unwrap();
    assert!((k - prismal_present::timeline::ease(0.25)).abs() < 1e-12, "{k}");
}

#[test]
fn formatting_and_diagnostics() {
    let doc = prismal_syntax::compile(SRC).unwrap().doc;
    let printed = prismal_syntax::format::format(&doc);
    for line in ["circle(c, r) as disc", "arc(c, r, from: 0 deg, to: turn) as quarter", "ellipse(c, 2 m, 1 m, rotate: 30 deg) as oval"] {
        assert!(printed.contains(line), "{line} in\n{printed}");
    }
    assert_eq!(prismal_syntax::compile(&printed).unwrap().doc, doc);

    let codes = |edit: &str, with: &str| -> Vec<&'static str> {
        let bad = SRC.replace(edit, with);
        assert_ne!(bad, SRC, "{edit}");
        let doc = prismal_syntax::compile(&bad).unwrap_or_else(|d| panic!("{d:?}")).doc;
        Program::new(doc).err().map(|ds| ds.iter().map(|d| d.code).collect()).unwrap_or_default()
    };
    assert_eq!(codes("circle(c, r) as disc", "circle(c, turn) as disc"), vec!["PK-E04"], "a radius is a length");
    assert_eq!(codes("circle(c, r) as disc", "circle(c) as disc"), vec!["PK-E05"], "a radius is needed");
    assert_eq!(codes(", to: turn) as quarter\n    ellipse", ") as quarter\n    ellipse"), vec!["PK-E05"], "an arc needs both ends");
    assert_eq!(codes("from: 0 deg, to: turn) as quarter\n    ellipse", "from: 0 m, to: turn) as quarter\n    ellipse"), vec!["PK-E04"], "an angle");
    // An option the kind does not take is a syntax error (D-081).
    let bad = prismal_syntax::compile(&SRC.replace("circle(c, r) as disc", "circle(c, r, rotate: turn) as disc")).unwrap_err();
    assert!(bad.iter().any(|d| d.code == "SX-E08" && d.message.contains("`circle` takes no option `rotate`")), "{bad:?}");
    assert_eq!(codes("view down: spatial(Plane, scale: 1 m -> 100 px, y: down) {", "view down: plot(x: [0, 1], y: [0, 1]) {"), vec!["PK-E05"], "spatial only");
}
