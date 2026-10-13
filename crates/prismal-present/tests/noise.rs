//! White noise in flows (D-090): stochastic differential equations integrated by
//! Euler-Maruyama (RC-13.9), checked against their exact laws over many seeds, within five
//! standard errors.

mod common;
use common::*;
use prismal_runtime::{resume, run, Action, Config, RunStatus};
use prismal_syntax::{compile, format};

const SRC: &str = r#"space Plane = euclidean(2)

model Brownian {
  param { D: Quantity<L^2/T> = 0.5 m^2/s }
  state { x: Length = 0 m }
  flow der(x) = sqrt(2 * D) * noise
}

model Spread in Plane {
  param { D: Quantity<L^2/T> = 0.5 m^2/s }
  state { pos: Point = origin }
  flow der(pos) = (sqrt(2 * D) * noise, sqrt(2 * D) * noise)
}

model Relax {
  param {
    tau: Time = 0.5 s
    s:   Velocity = 2 m/s
  }
  state { v: Velocity = 0 m/s }
  flow der(v) = -v / tau + s * sqrt(2 / tau) * noise
}

model Stock {
  param {
    mu:    Frequency = 0.1 /s
    sigma: Quantity<T^(-1/2)> = 0.4 /s^0.5
  }
  state { S: Real = 1 }
  flow der(S) = mu * S + sigma * S * noise
}

model Wall {
  param {
    v: Velocity = 1 m/s
    D: Quantity<L^2/T> = 0.1 m^2/s
  }
  state { x: Length = 1 m }
  discrete { stuck: Boolean = false }
  flow der(x) = if stuck then 0 else -v + sqrt(2 * D) * noise
  event hit on falling(x) { set stuck = true }
}
"#;

fn finals(model: &str, binding: &str, t_end: f64, seeds: u64) -> Vec<Vec<f64>> {
    let prog = program(SRC);
    let cm = prog.model(model);
    (0..seeds)
        .map(|seed| {
            let r = run(cm, Config::until(t_end).seed(seed));
            assert!(matches!(r.status, RunStatus::Completed), "{:?}", r.status);
            prismal_present::flat(&r.state_at(t_end)[cm.idx(binding)])
        })
        .collect()
}

fn mean(xs: &[f64]) -> f64 {
    xs.iter().sum::<f64>() / xs.len() as f64
}

fn near(xs: &[f64], expected: f64, var: f64) {
    let m = mean(xs);
    let se = (var / xs.len() as f64).sqrt();
    assert!((m - expected).abs() < 5.0 * se, "mean {m}, expected {expected} within {}", 5.0 * se);
}

/// Free diffusion: `x(T)` is normal with variance `2 D T`.
#[test]
fn brownian_motion_spreads_as_2_d_t() {
    let x: Vec<f64> = finals("Brownian", "Brownian.x", 1.0, 500).into_iter().map(|v| v[0]).collect();
    near(&x, 0.0, 1.0);
    let sq: Vec<f64> = x.iter().map(|v| v * v).collect();
    // The fourth moment of a normal is 3 var^2.
    near(&sq, 1.0, 2.0);
}

/// Each occurrence of `noise` is its own: the two coordinates are independent.
#[test]
fn each_noise_is_independent() {
    let p = finals("Spread", "Spread.pos", 1.0, 500);
    let xy: Vec<f64> = p.iter().map(|v| v[0] * v[1]).collect();
    near(&xy, 0.0, 1.0);
    let r2: Vec<f64> = p.iter().map(|v| v[0] * v[0] + v[1] * v[1]).collect();
    near(&r2, 2.0, 4.0);
}

/// Ornstein-Uhlenbeck: from rest, the variance grows to `s^2 (1 - e^(-2T / tau))`.
#[test]
fn a_relaxing_velocity_reaches_its_spread() {
    let v: Vec<f64> = finals("Relax", "Relax.v", 1.0, 500).into_iter().map(|v| v[0]).collect();
    let var = 4.0 * (1.0 - (-4.0f64).exp());
    let sq: Vec<f64> = v.iter().map(|x| x * x).collect();
    near(&sq, var, 2.0 * var * var);
}

/// Geometric Brownian motion read as Itô: the mean grows as `e^(mu T)`, and the mean of
/// `ln S` as `(mu - sigma^2 / 2) T`. A Stratonovich reading would give `mu T` for the log.
#[test]
fn noise_is_read_as_ito() {
    let s: Vec<f64> = finals("Stock", "Stock.S", 2.0, 800).into_iter().map(|v| v[0]).collect();
    let m = (0.2f64).exp();
    near(&s, m, m * m * ((0.32f64).exp() - 1.0));
    let ln: Vec<f64> = s.iter().map(|x| x.ln()).collect();
    near(&ln, (0.1 - 0.08) * 2.0, 0.16 * 2.0);
}

/// Events see noisy flows: first passage to a wall under drift `v` from distance `L` takes
/// `L / v` on average (variance `2 D L / v^3`).
#[test]
fn first_passage_under_noise() {
    let prog = program(SRC);
    let cm = prog.model("Wall");
    let times: Vec<f64> = (0..400).map(|seed| run(cm, Config::until(5.0).seed(seed)).log.iter().find(|e| e.name == "hit").expect("hits").t).collect();
    near(&times, 1.0, 0.2);
}

#[test]
fn seeds_reproduce_noisy_runs_and_resume() {
    let prog = program(SRC);
    let cm = prog.model("Wall");
    let bits = |r: &prismal_runtime::Run| r.committed.iter().map(|c| (c.t.to_bits(), prismal_present::flat(&c.vals[cm.idx("Wall.x")])[0].to_bits())).collect::<Vec<_>>();
    let a = run(cm, Config::until(3.0).seed(4));
    assert_eq!(bits(&a), bits(&run(cm, Config::until(3.0).seed(4))));
    assert_ne!(bits(&a), bits(&run(cm, Config::until(3.0).seed(5))));
    let cfg = Config::until(3.0).seed(4).at(0.5, Action::Intervene(vec![]));
    assert_eq!(bits(&resume(&a, cfg.clone())), bits(&run(cm, cfg)));
}

/// The step: `h` of the case, else a thousandth of `t_ref`.
#[test]
fn the_step_is_the_cases_h() {
    let prog = program(SRC);
    let cm = prog.model("Brownian");
    assert_eq!(run(cm, Config::until(1.0)).segments.len(), 1000);
    let mut cfg = Config::until(1.0);
    cfg.h_noise = Some(0.01);
    assert_eq!(run(cm, cfg).segments.len(), 100);
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
fn noise_is_checked() {
    let d = codes("model M { state { x: Real = 0 }\n derived { y: Quantity<T^(-1/2)> = noise } }");
    assert!(d.iter().any(|m| m.contains("MK-E29")), "{d:?}");
    let d = codes("model M { state { x: Real = 0 }\n flow der(x) = noise * noise * 1 s / 1 s^0.5 }");
    assert!(d.iter().any(|m| m.contains("MK-E30") && m.contains("noise multiplies noise")), "{d:?}");
    let d = codes("model M { state { x: Real = 0 }\n flow der(x) = sin(noise * 1 s^0.5) / 1 s }");
    assert!(d.iter().any(|m| m.contains("MK-E30")), "{d:?}");
    let d = codes("model M { state { x: Real = 0 }\n flow der(x) = noise }");
    assert!(d.iter().any(|m| m.contains("MK-E")), "noise has dimension T^(-1/2): {d:?}");
    // A name the author declares is not the noise.
    let d = codes("model M { param { noise: Frequency = 1 /s }\n state { x: Real = 0 }\n flow der(x) = noise }");
    assert!(d.is_empty(), "{d:?}");
    let text = format(&compile(SRC).unwrap().doc);
    assert!(text.contains("der(x) = sqrt(2 * D) * noise"), "{text}");
    assert!(text.contains("sigma: Quantity<1/T^(1/2)> = 0.4 /s^0.5"), "{text}");
    assert_eq!(format(&compile(&text).unwrap().doc), text);
}
