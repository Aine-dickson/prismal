//! Enforced inequality constraints (D-098): contact with a floor, a body leaving a curved
//! surface, a rope that goes slack and taut; the holding force named with `with`.

mod common;
use common::*;
use prismal_runtime::{run, Config, RunStatus};
use prismal_syntax::{compile, format};

const SRC: &str = r#"space Plane = euclidean(2)

model Drop {
  param {
    g: Acceleration = 9.81 m/s^2
    m: Mass         = 2 kg
  }
  state {
    y: Length   = 1 m
    v: Velocity = 0 m/s
  }
  unknown { N: Force }
  flow {
    der(y) = v
    der(v) = -g + N / m
  }
  constraint floor: y >= 0 m policy enforce with N
}

model Kick {
  param {
    g: Acceleration = 9.81 m/s^2
    m: Mass         = 1 kg
  }
  state {
    y: Length   = 0 m
    v: Velocity = 3 m/s
  }
  unknown { N: Force }
  flow {
    der(y) = v
    der(v) = -g + N / m
  }
  constraint floor: y >= 0 m policy enforce with N
}

model Hill in Plane {
  param {
    g:  Acceleration = 9.81 m/s^2
    m:  Mass         = 1 kg
    R:  Length       = 1 m
    θ0: Angle        = 5 deg
  }
  state {
    pos: Point            = origin + (R * sin(θ0), R * cos(θ0))
    vel: Vector<Velocity> = 0
  }
  unknown { N: Force = 1 N }
  flow {
    der(pos) = vel
    der(vel) = (0 m/s^2, -g) + N / m * (pos - origin) / |pos - origin|
  }
  constraint surface: |pos - origin| >= R policy enforce with N
}

model Rope in Plane {
  param {
    g: Acceleration = 9.81 m/s^2
    m: Mass         = 1 kg
    L: Length       = 1 m
  }
  state {
    bob: Point            = origin + (0.6 m, 0 m)
    vel: Vector<Velocity> = 0
  }
  unknown { T: Force }
  flow {
    der(bob) = vel
    der(vel) = (0 m/s^2, -g) - T / m * (bob - origin) / |bob - origin|
  }
  constraint rope: |bob - origin| <= L policy enforce with T
}

model Reversed {
  param {
    g: Acceleration = 9.81 m/s^2
    m: Mass         = 1 kg
  }
  state {
    y: Length   = 1 m
    v: Velocity = 0 m/s
  }
  unknown { N: Force }
  flow {
    der(y) = v
    der(v) = -g - N / m
  }
  constraint floor: y >= 0 m policy enforce with N
}
"#;

fn completed(model: &str, until: f64) -> (prismal_present::Program, prismal_runtime::Run) {
    let prog = program(SRC);
    let r = run(prog.model(model), Config::until(until));
    assert!(matches!(r.status, RunStatus::Completed), "{:?}", r.status);
    (prog, r)
}

/// A ball dropped from 1 m lands at `sqrt(2 h / g)` and stays: at rest on the floor, the
/// floor pushes with its weight.
#[test]
fn a_ball_lands_and_rests_on_a_floor() {
    let (prog, r) = completed("Drop", 1.5);
    let cm = prog.model("Drop");
    let touches = r.times("touch of floor");
    assert_eq!(touches.len(), 1, "{touches:?}");
    assert!((touches[0] - (2.0f64 / 9.81).sqrt()).abs() < 1e-6, "{touches:?}");
    for t in [0.2, 0.4] {
        let s = r.state_at(t);
        assert!((s[cm.idx("Drop.y")].num() - (1.0 - 0.5 * 9.81 * t * t)).abs() < 1e-9);
        assert_eq!(s[cm.idx("Drop.N")].num(), 0.0, "no push in the air at {t}");
    }
    let s = r.state_at(1.5);
    assert!(s[cm.idx("Drop.y")].num().abs() < 1e-12 && s[cm.idx("Drop.v")].num().abs() < 1e-12);
    assert!((s[cm.idx("Drop.N")].num() - 2.0 * 9.81).abs() < 1e-9, "{}", s[cm.idx("Drop.N")]);
    assert!(r.committed.iter().all(|c| c.vals[cm.idx("Drop.y")].num() >= -1e-6));
}

/// A ball on the floor thrown up leaves it at once, flies for `2 v / g` and lands.
#[test]
fn a_ball_thrown_up_from_a_floor_leaves_it() {
    let (prog, r) = completed("Kick", 1.0);
    let cm = prog.model("Kick");
    let y = r.state_at(0.3)[cm.idx("Kick.y")].num();
    assert!((y - (0.9 - 0.5 * 9.81 * 0.09)).abs() < 1e-9, "{y}");
    let touches = r.times("touch of floor");
    assert_eq!(touches.len(), 1, "{touches:?}");
    assert!((touches[0] - 6.0 / 9.81).abs() < 1e-6, "{touches:?}");
    assert!(r.state_at(1.0)[cm.idx("Kick.y")].num().abs() < 1e-12);
}

/// A particle sliding from near the top of a smooth hemisphere leaves it where
/// `cos θ = 2/3 cos θ0`, and flies clear of it afterwards.
#[test]
fn a_particle_leaves_a_hemisphere() {
    let (prog, r) = completed("Hill", 2.0);
    let cm = prog.model("Hill");
    let releases = r.times("release of surface");
    assert_eq!(releases.len(), 1, "{releases:?}");
    let s = r.state_at(releases[0]);
    let mut p = [0.0; 2];
    s[cm.idx("Hill.pos")].write_to(&mut p);
    let cos = p[1] / (p[0] * p[0] + p[1] * p[1]).sqrt();
    assert!((cos - 2.0 / 3.0 * 5f64.to_radians().cos()).abs() < 1e-6, "{cos}");
    let mut q = [0.0; 2];
    r.state_at(releases[0] + 0.2)[cm.idx("Hill.pos")].write_to(&mut q);
    assert!((q[0] * q[0] + q[1] * q[1]).sqrt() > 1.0 + 1e-3, "{q:?}");
    assert!(r.times("touch of surface").is_empty());
}

/// A bob on a slack rope falls freely until the rope is taut, `sqrt(2 · 0.8 m / g)` later,
/// then swings on it: the rope never stretches and never pushes.
#[test]
fn a_slack_rope_goes_taut() {
    let (prog, r) = completed("Rope", 2.0);
    let cm = prog.model("Rope");
    let touches = r.times("touch of rope");
    assert!(!touches.is_empty());
    assert!((touches[0] - (1.6f64 / 9.81).sqrt()).abs() < 1e-6, "{touches:?}");
    for c in &r.committed {
        let mut b = [0.0; 2];
        c.vals[cm.idx("Rope.bob")].write_to(&mut b);
        let len = (b[0] * b[0] + b[1] * b[1]).sqrt();
        assert!(len <= 1.0 + 1e-9, "{len} at {}", c.t);
        assert!(c.vals[cm.idx("Rope.T")].num() >= -1e-9, "{} at {}", c.vals[cm.idx("Rope.T")], c.t);
    }
    // The jerk removes the velocity along the rope and keeps the velocity across it.
    let mut v = [0.0; 2];
    r.state_at(touches[0] + 1e-9)[cm.idx("Rope.vel")].write_to(&mut v);
    let fall = (2.0f64 * 9.81 * 0.8).sqrt();
    assert!((v[0] * 0.6 - v[1] * 0.8).abs() < 1e-6, "{v:?}");
    assert!(((v[0] * v[0] + v[1] * v[1]).sqrt() - fall * 0.6).abs() < 1e-6, "{v:?}");
}

/// A holding force written to pull instead of push: the run stops at the touch and says why.
#[test]
fn a_force_of_the_wrong_sign_is_reported() {
    let prog = program(SRC);
    let r = run(prog.model("Reversed"), Config::until(1.5));
    match &r.status {
        RunStatus::Stopped(d) => assert!(d.message.contains("closes it instead of holding it") && d.message.contains("`N`"), "{}", d.message),
        s => panic!("{s:?}"),
    }
}

#[test]
fn canonical_text_prints_with() {
    let text = format(&compile(SRC).unwrap().doc);
    assert!(text.contains("constraint floor: y >= 0 m policy enforce with N"), "{text}");
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

fn model(constraint: &str) -> String {
    format!(
        "model M {{
  param {{ g: Acceleration = 9.81 m/s^2; m: Mass = 1 kg }}
  state {{ y: Length = 1 m; v: Velocity = 0 m/s }}
  unknown {{ N: Force }}
  flow {{ der(y) = v; der(v) = -g + N / m }}
  {constraint}
}}"
    )
}

#[test]
fn inequalities_are_checked() {
    let d = codes(&model("constraint floor: y >= 0 m policy enforce"));
    assert!(d.iter().any(|m| m.contains("MK-E36") && m.contains("with N")), "{d:?}");
    let d = codes(&model("constraint floor: y == 0 m policy enforce with N"));
    assert!(d.iter().any(|m| m.contains("MK-E36") && m.contains("`with` names")), "{d:?}");
    let d = codes(&model("constraint floor: y >= 0 m with N"));
    assert!(d.iter().any(|m| m.contains("MK-E36")), "{d:?}");
    let d = codes(&model("constraint floor: y >= 0 m policy enforce with g"));
    assert!(d.iter().any(|m| m.contains("MK-E36") && m.contains("unknown")), "{d:?}");
    let d = codes(&model("constraint floor: y >= 0 m policy enforce with nothing"));
    assert!(!d.is_empty(), "{d:?}");
    // A plain inequality is still a check.
    let d = codes("model M { state { y: Length = 1 m }\n flow der(y) = -1 m/s\n constraint floor: y >= 0 m }");
    assert!(d.is_empty(), "{d:?}");
}

