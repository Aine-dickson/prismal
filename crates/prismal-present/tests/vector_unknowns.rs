//! Vector and point unknowns (D-097): unknowns with one component per axis of their space,
//! in solved equations, in rates given by equations and in enforced constraints.

mod common;
use common::*;
use prismal_runtime::{run, Config, RunStatus};
use prismal_syntax::{compile, format};

const SRC: &str = r#"space Plane = euclidean(2)

model Hanger in Plane {
  param {
    g: Acceleration      = 9.81 m/s^2
    m: Mass              = 1 kg
    k: Quantity<M/T^2>   = 10 N/m
    A: Point             = origin + (-1 m, 0 m)
    B: Point             = origin + (1 m, 0 m)
  }
  unknown { P: Point }
  equation balance: k * (A - P) + k * (B - P) + (0 N, -m * g) == (0 N, 0 N)
}

model Stretched in Plane {
  param {
    g:  Acceleration    = 9.81 m/s^2
    m:  Mass            = 1 kg
    k:  Quantity<M/T^2> = 10 N/m
    L0: Length          = 1.2 m
    A:  Point           = origin + (-1 m, 0 m)
    B:  Point           = origin + (1 m, 0 m)
  }
  unknown { P: Point = origin + (0 m, -1 m) }
  derived {
    pull_a: Vector<Force> = k * (|A - P| - L0) * (A - P) / |A - P|
    pull_b: Vector<Force> = k * (|B - P| - L0) * (B - P) / |B - P|
  }
  equation balance: pull_a + pull_b + (0 N, -m * g) == (0 N, 0 N)
}

model Held in Plane {
  param {
    g: Acceleration = 9.81 m/s^2
    m: Mass         = 2 kg
  }
  unknown { F: Vector<Force> }
  equation rest: F + (0 N, -m * g) == (0 N, 0 N)
}

model Thrown in Plane {
  param {
    g: Acceleration = 9.81 m/s^2
    m: Mass         = 0.5 kg
  }
  state {
    pos: Point            = origin
    vel: Vector<Velocity> = (3 m/s, 4 m/s)
  }
  flow der(pos) = vel
  equation newton: m * der(vel) == (0 N, -m * g)
}

model Bead in Plane {
  param {
    g: Acceleration = 9.81 m/s^2
    m: Mass         = 2 kg
    α: Angle        = 30 deg
  }
  state {
    pos: Point            = origin
    vel: Vector<Velocity> = 0
  }
  unknown { N: Vector<Force> }
  flow {
    der(pos) = vel
    der(vel) = (0 m/s^2, -g) + N / m
  }
  equation smooth: dot(N, (cos(α), -sin(α))) == 0 N
  constraint wire: pos.y == -tan(α) * pos.x policy enforce
}
"#;

fn completed(model: &str, until: f64) -> (prismal_present::Program, prismal_runtime::Run) {
    let prog = program(SRC);
    let r = run(prog.model(model), Config::until(until.max(1e-9)));
    assert!(matches!(r.status, RunStatus::Completed), "{:?}", r.status);
    (prog, r)
}

fn at(model: &str, binding: &str, t: f64) -> Vec<f64> {
    let (prog, r) = completed(model, t);
    let cm = prog.model(model);
    let v = &r.state_at(t)[cm.idx(binding)];
    let mut out = vec![0.0; 2];
    v.write_to(&mut out);
    out
}

/// Two springs without rest length hold a weight between their anchors: it hangs midway,
/// `m g / 2k` below them.
#[test]
fn a_point_unknown_finds_an_equilibrium() {
    let p = at("Hanger", "Hanger.P", 0.0);
    assert!(p[0].abs() < 1e-12 && (p[1] + 9.81 / 20.0).abs() < 1e-12, "{p:?}");
}

/// Springs with a rest length: the balance is not linear in `P`; Newton finds it from the
/// guess, on the symmetry line, with the forces balanced.
#[test]
fn a_point_unknown_of_a_nonlinear_balance() {
    let p = at("Stretched", "Stretched.P", 0.0);
    assert!(p[0].abs() < 1e-12 && p[1] < 0.0, "{p:?}");
    let d = (1.0 + p[1] * p[1]).sqrt();
    let up = 2.0 * 10.0 * (d - 1.2) * (-p[1]) / d;
    assert!((up - 9.81).abs() < 1e-9, "{up}");
}

/// A force that holds a weight at rest, found as one vector.
#[test]
fn a_vector_unknown() {
    let f = at("Held", "Held.F", 0.0);
    assert!(f[0].abs() < 1e-12 && (f[1] - 2.0 * 9.81).abs() < 1e-12, "{f:?}");
}

/// `m der(vel) == F` gives the rate of a vector state: a throw lands where the closed form
/// says.
#[test]
fn an_equation_gives_the_rate_of_a_vector() {
    let p = at("Thrown", "Thrown.pos", 0.5);
    assert!((p[0] - 1.5).abs() < 1e-9 && (p[1] - (2.0 - 0.5 * 9.81 * 0.25)).abs() < 1e-9, "{p:?}");
}

/// A bead on a smooth wire, its holding force one vector: the force is perpendicular to the
/// wire (`smooth`) and of size `m g cos α`, and the bead slides with `g sin α`, as with the
/// force written as a size along the normal (D-093).
#[test]
fn a_vector_holding_force_in_an_enforced_constraint() {
    let (prog, r) = completed("Bead", 2.0);
    let cm = prog.model("Bead");
    let a = 30f64.to_radians();
    let s = r.state_at(2.0);
    let mut p = [0.0; 2];
    s[cm.idx("Bead.pos")].write_to(&mut p);
    let along = 0.5 * 9.81 * a.sin() * 4.0;
    assert!((p[0] - along * a.cos()).abs() < 1e-9 && (p[1] + along * a.sin()).abs() < 1e-9, "{p:?}");
    let mut n = [0.0; 2];
    s[cm.idx("Bead.N")].write_to(&mut n);
    let size = 2.0 * 9.81 * a.cos();
    assert!((n[0] - size * a.sin()).abs() < 1e-9 && (n[1] - size * a.cos()).abs() < 1e-9, "{n:?}");
}

#[test]
fn canonical_text_prints_vector_unknowns() {
    let text = format(&compile(SRC).unwrap().doc);
    assert!(text.contains("unknown { N: Vector<Force> }") || text.contains("unknown {\n    N: Vector<Force>\n  }"), "{text}");
    assert_eq!(format(&compile(&text).unwrap().doc), text);
}

fn codes(src: &str) -> Vec<String> {
    match compile(src) {
        Err(ds) => ds.iter().map(|d| format!("{}: {}", d.code, d.message)).collect(),
        Ok(c) => match prismal_present::Program::new(c.doc) {
            Ok(_) => vec![],
            Err(ds) => ds.iter().map(|d| d.to_string()).collect(),
        },
    }
}

#[test]
fn vector_unknowns_are_counted_by_component() {
    // One equation component for a vector of two.
    let d = codes("space Plane = euclidean(2)
model M in Plane {
  unknown { F: Vector<Force> }
  equation a: dot(F, (1, 0)) == 1 N
}");
    assert!(d.iter().any(|m| m.contains("MK-E34") && m.contains("2 unknown components") && m.contains("fewer")), "{d:?}");
    // Two scalar equations for a vector: as many components as unknowns.
    let d = codes("space Plane = euclidean(2)
model M in Plane {
  unknown { F: Vector<Force> }
  equation a: dot(F, (1, 0)) == 1 N
  equation b: dot(F, (0, 1)) == 2 N
}");
    assert!(d.is_empty(), "{d:?}");
    // An unknown is a number, a quantity, a vector or a point.
    let d = codes("model M { unknown { b: Boolean }\n equation a: b == true }");
    assert!(d.iter().any(|m| m.contains("MK-E01") && m.contains("a vector or a point")), "{d:?}");
}
