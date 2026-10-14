//! Friction and bouncing contact (D-102): an enforced equality held by a bounded force,
//! `with F where |F| <= mu * N`, that sticks and slips; impacts with restitution.

mod common;
use common::*;
use prismal_runtime::{run, Config, RunStatus};
use prismal_syntax::{compile, format};

const SRC: &str = r#"model Push {
  param {
    m:  Mass         = 2 kg
    g:  Acceleration = 9.81 m/s^2
    mu: Real         = 0.5
    k:  Quantity<M*L/T^3> = 2 N/s
    v0: Velocity     = 0 m/s
  }
  state {
    x: Length   = 0 m
    v: Velocity = v0
    y: Length   = 0 m
    w: Velocity = 0 m/s
  }
  unknown {
    N: Force
    F: Force
  }
  derived { P: Force = k * elapsed }
  flow {
    der(x) = v
    der(v) = (P + F) / m
    der(y) = w
    der(w) = -g + N / m
  }
  constraint floor: y >= 0 m policy enforce with N
  constraint grip: v == 0 m/s policy enforce with F where |F| <= mu * N
}
"#;

const G: f64 = 9.81;

/// Pushed by `P = 2 N/s × t`, the block sticks while the push is at most `mu m g = 9.81 N`,
/// until 4.905 s, the friction equal and opposite to the push; then it slides with
/// `a = (P - mu m g) / m = t - 4.905 s`, so `v = (t - 4.905)² / 2`.
#[test]
fn a_block_sticks_until_the_push_passes_the_bound() {
    let prog = program(SRC);
    let cm = prog.model("Push");
    let r = run(cm, Config::until(7.0));
    assert!(matches!(r.status, RunStatus::Completed), "{:?}", r.status);
    let at = |t: f64, n: &str| r.state_at(t)[cm.idx(&format!("Push.{n}"))].num();
    let t_slip = 0.5 * 2.0 * G / 2.0;
    assert_eq!(at(3.0, "v"), 0.0);
    assert!((at(3.0, "F") + 6.0).abs() < 1e-9, "friction against a 6 N push: {}", at(3.0, "F"));
    assert!((at(3.0, "N") - 2.0 * G).abs() < 1e-9);
    let slips = r.times("slip of grip");
    assert_eq!(slips.len(), 1, "{slips:?}");
    assert!((slips[0] - t_slip).abs() < 1e-6, "{slips:?}");
    let t = t_slip + 2.0;
    assert!((at(t, "v") - 2.0).abs() < 1e-6, "v = {}", at(t, "v"));
    assert!((at(t, "F") + 0.5 * 2.0 * G).abs() < 1e-9, "kinetic friction: {}", at(t, "F"));
}

/// Sliding at 3 m/s with no push, the block slows at `mu g` and stops after
/// `v0 / (mu g) = 0.6116 s`, `v0² / (2 mu g) = 0.9174 m` on; then it holds, friction zero.
#[test]
fn a_sliding_block_stops_and_stays() {
    let prog = program(SRC);
    let cm = prog.model("Push");
    let cfg = Config::until(2.0).param("Push.k", prismal_ir::Expr::Num { num: 0.0, unit: prismal_ir::Unit::parse("N/s").ok() }).param("Push.v0", prismal_ir::Expr::Num { num: 3.0, unit: prismal_ir::Unit::parse("m/s").ok() });
    let r = run(cm, cfg);
    assert!(matches!(r.status, RunStatus::Completed), "{:?}", r.status);
    let at = |t: f64, n: &str| r.state_at(t)[cm.idx(&format!("Push.{n}"))].num();
    let stop = 3.0 / (0.5 * G);
    let grips = r.times("grip of grip");
    assert!((grips[0] - stop).abs() < 1e-6, "{grips:?}");
    assert!((at(0.3, "v") - (3.0 - 0.5 * G * 0.3)).abs() < 1e-9);
    assert!((at(2.0, "x") - 9.0 / (2.0 * 0.5 * G)).abs() < 1e-6, "x = {}", at(2.0, "x"));
    assert_eq!(at(2.0, "v"), 0.0);
    assert!(at(2.0, "F").abs() < 1e-9);
}

#[test]
fn canonical_text_of_friction() {
    let text = format(&compile(SRC).unwrap().doc);
    assert!(text.contains("constraint grip: v == 0 m/s policy enforce with F where |F| <= mu * N"), "{text}");
    assert_eq!(format(&compile(&text).unwrap().doc), text);
}

const BALL: &str = r#"model Ball {
  param {
    g: Acceleration = 9.81 m/s^2
    m: Mass         = 1 kg
    e: Real         = 0.5
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
  constraint floor: y >= 0 m policy enforce with N restitution e
}
"#;

/// Dropped from 1 m with restitution 0.5: it lands at `t1 = sqrt(2 h / g)`, leaves at half
/// the speed, rises to `e² h = 0.25 m`, lands again at `2 t1`, and the bounces, each half
/// as long, accumulate at `t1 (1 + 2 e / (1 - e)) = 3 t1`: then it rests, held by `m g`.
#[test]
fn a_ball_bounces_with_restitution_then_rests() {
    let prog = program(BALL);
    let cm = prog.model("Ball");
    let r = run(cm, Config::until(3.0));
    assert!(matches!(r.status, RunStatus::Completed), "{:?}", r.status);
    let at = |t: f64, n: &str| r.state_at(t)[cm.idx(&format!("Ball.{n}"))].num();
    let t1 = (2.0 / G).sqrt();
    let touches = r.times("touch of floor");
    assert!((touches[0] - t1).abs() < 1e-6, "{touches:?}");
    assert!((touches[1] - 2.0 * t1).abs() < 1e-6, "{touches:?}");
    let apex = 1.5 * t1;
    assert!((at(apex, "y") - 0.25).abs() < 1e-6, "apex {}", at(apex, "y"));
    // Just after the impact (at its instant, the state before it is read).
    assert!((at(t1 + 1e-6, "v") - (0.5 * (2.0 * G).sqrt() - G * 1e-6)).abs() < 1e-6, "rebound {}", at(t1 + 1e-6, "v"));
    // Resting soon after the bounces accumulate.
    let last = *touches.last().unwrap();
    assert!(last < 3.0 * t1 + 1e-3 && last > 3.0 * t1 - 0.05, "last touch {last}, accumulation at {}", 3.0 * t1);
    assert_eq!(at(3.0, "y"), 0.0);
    assert_eq!(at(3.0, "v"), 0.0);
    assert!((at(3.0, "N") - G).abs() < 1e-9);
}

#[test]
fn friction_and_restitution_are_checked() {
    let codes = |src: &str| match compile(src) {
        Err(ds) => ds.iter().map(|d| format!("{}: {}", d.code, d.message)).collect::<Vec<_>>(),
        Ok(c) => match prismal_present::Program::new(c.doc) {
            Ok(_) => vec![],
            Err(ds) => ds.iter().map(|d| d.to_string()).collect(),
        },
    };
    let base = "model M { state { v: Velocity = 0 m/s }\n unknown { F: Force }\n flow { der(v) = F / 1 kg }\n";
    let d = codes(&format!("{base} constraint c: v == 0 m/s policy enforce with F where F <= 2 N }}"));
    assert!(d.iter().any(|m| m.contains("SX-E08") && m.contains("|F| <= B")), "{d:?}");
    let d = codes(&format!("{base} constraint c: v == 0 m/s policy enforce with F restitution 0.5 }}"));
    assert!(d.iter().any(|m| m.contains("MK-E36") && m.contains("restitution")), "{d:?}");
    let d = codes(&format!("{base} constraint c: v >= 0 m/s policy enforce with F where |F| <= 2 N }}"));
    assert!(d.iter().any(|m| m.contains("MK-E36") && m.contains("where")), "{d:?}");
    let d = codes(&format!("{base} constraint c: v == 0 m/s policy enforce with F where |F| <= 2 m }}"));
    assert!(d.iter().any(|m| m.contains("MK-E")), "{d:?}");
}

/// The friction force written the other way, `der(v) = -F / m`: a block sliding from the
/// start still slows down (RC-5.13), and stops at the same place.
#[test]
fn friction_written_the_other_way_still_opposes_the_slip() {
    let src = SRC.replace("der(v) = (P + F) / m", "der(v) = (P - F) / m");
    let prog = program(&src);
    let cm = prog.model("Push");
    let cfg = Config::until(2.0).param("Push.k", prismal_ir::Expr::Num { num: 0.0, unit: prismal_ir::Unit::parse("N/s").ok() }).param("Push.v0", prismal_ir::Expr::Num { num: 3.0, unit: prismal_ir::Unit::parse("m/s").ok() });
    let r = run(cm, cfg);
    assert!(matches!(r.status, RunStatus::Completed), "{:?}", r.status);
    let x = r.state_at(2.0)[cm.idx("Push.x")].num();
    assert!((x - 9.0 / (2.0 * 0.5 * G)).abs() < 1e-6, "x = {x}");
}

#[test]
fn canonical_text_of_restitution() {
    let text = format(&compile(BALL).unwrap().doc);
    assert!(text.contains("constraint floor: y >= 0 m policy enforce with N restitution e"), "{text}");
    assert_eq!(format(&compile(&text).unwrap().doc), text);
}
