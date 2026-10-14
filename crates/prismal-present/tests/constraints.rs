//! Enforced constraints (D-093): the runtime keeps `policy enforce` constraints true by
//! projecting the state onto them, and solves the unknowns that hold them (a rod's tension)
//! from their rates; checked against closed forms of constrained mechanics.

mod common;
use common::*;
use prismal_runtime::{run, Config, RunStatus};
use prismal_syntax::{compile, format};

const SRC: &str = r#"space Plane = euclidean(2)

model Rod in Plane {
  param {
    g:     Acceleration = 9.81 m/s^2
    L:     Length       = 1 m
    m:     Mass         = 1 kg
    pivot: Point        = origin
    θ0:    Angle        = 10 deg
  }
  state {
    bob: Point            = pivot + L * (sin(θ0), -cos(θ0))
    vel: Vector<Velocity> = 0
  }
  unknown { tension: Force }
  derived {
    along:  Vector<Length> = bob - pivot
    energy: Energy = 0.5 * m * |vel|^2 + m * g * (bob.y - pivot.y)
  }
  flow {
    der(bob) = vel
    der(vel) = (0 m/s^2, -g) - tension / m * along / |along|
  }
  constraint rod: |bob - pivot| == L policy enforce
  event upswing on rising(bob.x - pivot.x)
  event shove on at t0 + 0.75 s { set bob = pivot + (2 m, 0 m) }
}

model Slope in Plane {
  param {
    g: Acceleration = 9.81 m/s^2
    m: Mass         = 2 kg
    α: Angle        = 30 deg
  }
  state {
    pos: Point            = origin
    vel: Vector<Velocity> = 0
  }
  unknown { N: Force }
  flow {
    der(pos) = vel
    der(vel) = (0 m/s^2, -g) + N / m * (sin(α), cos(α))
  }
  constraint wire: pos.y == -tan(α) * pos.x policy enforce
}

model Double in Plane {
  param {
    g:  Acceleration = 9.81 m/s^2
    L1: Length = 1 m
    L2: Length = 0.7 m
    m1: Mass   = 1 kg
    m2: Mass   = 0.5 kg
  }
  state {
    p1: Point            = origin + (1 m, 0 m)
    v1: Vector<Velocity> = 0
    p2: Point            = origin + (1 m, -0.7 m)
    v2: Vector<Velocity> = 0
  }
  unknown {
    T1: Force
    T2: Force
  }
  derived {
    u1: Vector<1> = (p1 - origin) / |p1 - origin|
    u2: Vector<1> = (p2 - p1) / |p2 - p1|
    energy: Energy = 0.5 * m1 * |v1|^2 + 0.5 * m2 * |v2|^2 + m1 * g * p1.y + m2 * g * p2.y
  }
  flow {
    der(p1) = v1
    der(v1) = (0 m/s^2, -g) - T1 / m1 * u1 + T2 / m1 * u2
    der(p2) = v2
    der(v2) = (0 m/s^2, -g) - T2 / m2 * u2
  }
  constraint upper: |p1 - origin| == L1 policy enforce
  constraint lower: |p2 - p1| == L2 policy enforce
}

model Ring in Plane {
  param {
    centre: Point  = origin
    r:      Length = 2 m
    p:      Point  = origin + (1.2 m, 1.6 m)
  }
  constraint on_ring: |p - centre| == r policy enforce
}
"#;

fn completed(model: &str, until: f64) -> (prismal_present::Program, prismal_runtime::Run) {
    let prog = program(SRC);
    let r = run(prog.model(model), Config::until(until));
    assert!(matches!(r.status, RunStatus::Completed), "{:?}", r.status);
    (prog, r)
}

/// A pendulum written as a free bob held by a rod: the swing has the exact period of a
/// pendulum of 10 degrees (RP-04), and the rod's tension is `m g cos θ0` at rest and
/// `m g (3 - 2 cos θ0)` at the bottom.
#[test]
fn a_bob_on_a_rod_swings_as_a_pendulum() {
    let prog = program(SRC);
    let cm = prog.model("Rod");
    let r = run(cm, Config::until(0.7));
    assert!(matches!(r.status, RunStatus::Completed), "{:?}", r.status);
    let c = 10f64.to_radians().cos();
    let start = r.state_at(0.0)[cm.idx("Rod.tension")].num();
    assert!((start - 9.81 * c).abs() < 1e-9, "{start}");
    // Without the shove: the first two upswings.
    let quiet = program(&SRC.replace("event shove on at t0 + 0.75 s { set bob = pivot + (2 m, 0 m) }", ""));
    let cm = quiet.model("Rod");
    let r = run(cm, Config::until(4.0));
    let ups: Vec<f64> = r.log.iter().filter(|e| e.name == "upswing").map(|e| e.t).collect();
    assert!((ups[0] - 1.50741947047395).abs() < 1e-5 * 1.5, "{ups:?}");
    assert!((ups[1] - ups[0] - 2.00989262729860).abs() < 1e-5 * 2.0, "{ups:?}");
    let bottom = r.state_at(ups[0])[cm.idx("Rod.tension")].num();
    assert!((bottom - 9.81 * (3.0 - 2.0 * c)).abs() < 1e-5, "{bottom}");
    // The rod's length holds at every committed state to rounding, and between them to the
    // solver's tolerance.
    for s in &r.committed {
        let p = s.vals[cm.idx("Rod.bob")].arr();
        let len = (p.v[0] * p.v[0] + p.v[1] * p.v[1]).sqrt();
        assert!((len - 1.0).abs() < 1e-14, "{len} at {}", s.t);
    }
    for k in 0..400 {
        let p = r.state_at(k as f64 * 0.01)[cm.idx("Rod.bob")].arr();
        let len = (p.v[0] * p.v[0] + p.v[1] * p.v[1]).sqrt();
        assert!((len - 1.0).abs() < 1e-6, "{len} at {}", k as f64 * 0.01);
    }
    // Energy is conserved by the model; projection keeps the drift at the solver's level.
    let e0 = r.state_at(0.0)[cm.idx("Rod.energy")].num();
    for k in 0..40 {
        let e = r.state_at(k as f64 * 0.1)[cm.idx("Rod.energy")].num();
        assert!((e - e0).abs() < 1e-5 * e0.abs(), "{e} against {e0}");
    }
}

/// A transition that moves the bob off the rod's circle: the committed state is on it, and
/// the velocity keeps only its part along the circle.
#[test]
fn a_shove_lands_on_the_circle() {
    let (prog, r) = completed("Rod", 1.0);
    let cm = prog.model("Rod");
    let after = r.committed.iter().find(|s| s.t >= 0.75 && s.n > 0).expect("the shove's transition");
    let p = after.vals[cm.idx("Rod.bob")].arr();
    assert!((p.v[0] - 1.0).abs() < 1e-14 && p.v[1].abs() < 1e-14, "{:?}", p.v);
    let v = after.vals[cm.idx("Rod.vel")].arr();
    assert!(v.v[0].abs() < 1e-14, "the radial part is removed: {:?}", v.v);
}

/// A bead on a straight wire at 30 degrees: it slides with `g sin α` along the wire, and the
/// wire pushes with `m g cos α`.
#[test]
fn a_bead_slides_down_a_wire() {
    let (prog, r) = completed("Slope", 2.0);
    let cm = prog.model("Slope");
    let a = 30f64.to_radians();
    let s = r.state_at(2.0);
    let p = s[cm.idx("Slope.pos")].arr();
    let along = 0.5 * 9.81 * a.sin() * 4.0;
    assert!((p.v[0] - along * a.cos()).abs() < 1e-9 && (p.v[1] + along * a.sin()).abs() < 1e-9, "{:?}", p.v);
    let n = s[cm.idx("Slope.N")].num();
    assert!((n - 2.0 * 9.81 * a.cos()).abs() < 1e-9, "{n}");
}

/// A double pendulum, two rods and two tensions: chaotic, so no closed form, but both rods
/// keep their lengths and the energy stays as it started.
#[test]
fn a_double_pendulum_keeps_its_rods_and_its_energy() {
    let (prog, r) = completed("Double", 5.0);
    let cm = prog.model("Double");
    let e0 = r.state_at(0.0)[cm.idx("Double.energy")].num();
    for s in &r.committed {
        let (p1, p2) = (s.vals[cm.idx("Double.p1")].arr(), s.vals[cm.idx("Double.p2")].arr());
        let l1 = (p1.v[0] * p1.v[0] + p1.v[1] * p1.v[1]).sqrt();
        let l2 = ((p2.v[0] - p1.v[0]).powi(2) + (p2.v[1] - p1.v[1]).powi(2)).sqrt();
        assert!((l1 - 1.0).abs() < 1e-13 && (l2 - 0.7).abs() < 1e-13, "{l1} {l2} at {}", s.t);
    }
    for k in 0..=50 {
        let e = r.state_at(k as f64 * 0.1)[cm.idx("Double.energy")].num();
        assert!((e - e0).abs() < 1e-3 * e0.abs(), "{e} against {e0} at {}", k as f64 * 0.1);
    }
    // Released from the horizontal, the upper rod pulls at first only to hold the lower bob
    // up against nothing: both tensions start at zero.
    let s = r.state_at(0.0);
    assert!(s[cm.idx("Double.T1")].num().abs() < 1e-9 && s[cm.idx("Double.T2")].num().abs() < 1e-9);
}

/// A model without flows: the constraint is enforced by projection alone. A change of the
/// point lands on the ring, nearest to where it was set (constrained dragging); the centre and
/// radius, parameters the change does not set, stay. A point declared off the ring is an
/// error at the start, since the runtime does not choose which parameter to move.
#[test]
fn a_point_is_held_on_a_ring() {
    let prog = program(SRC);
    let cm = prog.model("Ring");
    let pt = |x: f64, y: f64| prismal_ir::Expr::Bin {
        bin: prismal_ir::BinOp::Add,
        l: Box::new(prismal_ir::Expr::Origin { origin: "Plane".into() }),
        r: Box::new(prismal_ir::Expr::Tuple { tuple: vec![prismal_ir::Expr::Num { num: x, unit: Some(prismal_ir::Unit::parse("m").unwrap()) }, prismal_ir::Expr::Num { num: y, unit: Some(prismal_ir::Unit::parse("m").unwrap()) }] }),
    };
    let set = prismal_ir::Op::Set { target: prismal_ir::Target::of("Ring.p"), value: pt(0.0, -5.0) };
    let r = run(cm, Config::until(1.0).at(0.0, prismal_runtime::Action::Intervene(vec![set])));
    assert!(matches!(r.status, RunStatus::Completed), "{:?}", r.status);
    let p = r.final_value("Ring.p").arr();
    assert!(p.v[0].abs() < 1e-14 && (p.v[1] + 2.0).abs() < 1e-14, "{:?}", p.v);
    assert_eq!(r.final_value("Ring.r").num(), 2.0);
    let r = run(cm, Config::until(1.0).param("Ring.p", pt(3.0, 4.0)));
    match &r.status {
        RunStatus::NotStarted(d) => assert!(d.message.contains("only parameters"), "{}", d.message),
        s => panic!("{s:?}"),
    }
}

/// A series plot of an unknown follows the state its equations read: it is sampled, not drawn
/// as a staircase of committed states (a tension that would look constant between events).
#[test]
fn a_series_of_an_unknown_varies_between_events() {
    let prog = program(&format!(
        "{SRC}
presentation Plot for Rod {{
  view force: plot(x: [0 s, 3 s], y: [9 N, 11 N]) {{ series_plot(tension every 0.02 s) }}
}}"
    ));
    let i = prismal_present::interact::Interactive::new(&prog, "Plot", Config::until(3.0)).unwrap_or_else(|d| panic!("{d:?}"));
    let f = i.frame_at(0.7);
    let r = f.reps().find(|r| r.kind == "series_plot").expect("the series");
    let prismal_present::frame::Shape::Polyline { points } = &r.shape else { panic!("{:?}", r.shape) };
    let (lo, hi) = points.iter().fold((f64::INFINITY, f64::NEG_INFINITY), |(a, b), p| (a.min(p[1]), b.max(p[1])));
    assert!(lo < 9.67 && hi > 10.1, "{lo} {hi}");
    assert!(points.windows(2).all(|w| w[1][0] > w[0][0]), "no vertical risers of a staircase");
}

/// A learner drags the bob of a rod pendulum off its circle: it lands on the circle, on the
/// line from the pivot to where it was dropped.
#[test]
fn a_dragged_bob_lands_on_its_circle() {
    let src = SRC.replace("bob: Point            = pivot + L * (sin(θ0), -cos(θ0))", "bob: Point            = pivot + L * (sin(θ0), -cos(θ0))   intervenable");
    let prog = program(&format!(
        "{src}
presentation Drag for Rod {{
  view scene: spatial(Plane, scale: 1 m -> 100 px, y: up) {{
    marker(bob) as ball {{ on drag as p {{ propose bob = p }} }}
  }}
}}"
    ));
    let mut i = prismal_present::interact::Interactive::new(&prog, "Drag", Config::until(2.0)).unwrap_or_else(|d| panic!("{d:?}"));
    i.seek(0.5);
    i.pointer_down("ball", None).unwrap();
    i.pointer_move([3.0, 4.0]);
    assert!(i.pointer_up().unwrap(), "the drop is committed");
    let f = i.frame_at(0.5);
    let ball = f.reps().find(|r| r.kind == "marker").expect("the ball");
    let prismal_present::frame::Shape::Point { at } = ball.shape else { panic!("{:?}", ball.shape) };
    // In the view's pixels, 100 to the metre.
    assert!((at[0] - 60.0).abs() < 1e-6 && (at[1] - 80.0).abs() < 1e-6, "{at:?}");
}

#[test]
fn canonical_text_prints_enforce() {
    let text = format(&compile(SRC).unwrap().doc);
    assert!(text.contains("constraint rod: |bob - pivot| == L policy enforce"), "{text}");
    assert_eq!(format(&compile(&text).unwrap().doc), text);
    assert!(!text.contains("@solved") && !text.contains("@flow"), "{text}");
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
fn enforced_constraints_are_checked() {
    let rod = |flow: &str, constraint: &str| {
        format!(
            "space Plane = euclidean(2)
model M in Plane {{
  param {{ g: Acceleration = 9.81 m/s^2; L: Length = 1 m }}
  state {{ bob: Point = origin + (0 m, -1 m); vel: Vector<Velocity> = 0 }}
  unknown {{ T: Force }}
  flow {{ der(bob) = vel; der(vel) = {flow} }}
  {constraint}
}}"
        )
    };
    // Nothing that holds the bob: its rates never read an unknown.
    let d = codes(&rod("(0 m/s^2, -g)", "constraint rod: |bob - origin| == L policy enforce"));
    assert!(d.iter().any(|m| m.contains("MK-E36") && m.contains("declare what holds it")), "{d:?}");
    // An enforced constraint holds exactly.
    let d = codes(&rod("(0 m/s^2, -g) - T / 1 kg * (bob - origin) / L", "constraint rod: |bob - origin| == L within 1e-9 m policy enforce"));
    assert!(d.iter().any(|m| m.contains("MK-E01") && m.contains("drop `within`")), "{d:?}");
    // A choice has no rate where it switches.
    let d = codes(&rod("(0 m/s^2, -g) - T / 1 kg * (bob - origin) / L", "constraint rod: (if bob.x > 0 m then |bob - origin| else L) == L policy enforce"));
    assert!(d.iter().any(|m| m.contains("MK-E36") && m.contains("`if`")), "{d:?}");
    // The force is declared but no constraint determines it.
    let d = codes(&rod("(0 m/s^2, -g) - T / 1 kg * (bob - origin) / L", ""));
    assert!(d.iter().any(|m| m.contains("MK-E34") && m.contains("`T`")), "{d:?}");
    // An unknown policy names `enforce` among the policies.
    let d = codes(&rod("(0 m/s^2, -g)", "constraint rod: |bob - origin| == L policy hold"));
    assert!(d.iter().any(|m| m.contains("SX-E02") && m.contains("`enforce`")), "{d:?}");
}

/// `dot` of two vectors of one space, of two quantities; other pairs are errors.
#[test]
fn dot_products() {
    let prog = program(
        "space Plane = euclidean(2)
model D in Plane {
  param { a: Vector<Length> = (3 m, 4 m); f: Vector<Force> = (2 N, -1 N) }
  derived { w: Energy = dot(f, a); s: Area = dot(2 m, 3 m) }
}",
    );
    let cm = prog.model("D");
    let r = run(cm, Config::until(1e-9));
    let v = r.state_at(0.0);
    assert_eq!(v[cm.idx("D.w")].num(), 2.0);
    assert_eq!(v[cm.idx("D.s")].num(), 6.0);
    let d = codes("space Plane = euclidean(2)\nmodel D in Plane { param { a: Vector<Length> = (3 m, 4 m) }\n derived { w: Length = dot(a, 1 m) } }");
    assert!(d.iter().any(|m| m.contains("MK-E01") && m.contains("dot of")), "{d:?}");
}
