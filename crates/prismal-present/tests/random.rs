//! The stochastic slice (D-083): random events, draws in handlers, seeds. Statistical
//! checks compare means over many seeds with the exact law, within five standard errors.

mod common;
use common::*;
use prismal_present::expect::run_case;
use prismal_runtime::{run, Config, RunStatus};
use prismal_syntax::{compile, format};

const SRC: &str = r#"model Decay {
  param { lambda: Frequency = 0.5 /s }
  discrete { n: Real = 1000 }
  event decay on random(lambda * n) { set n = n - 1 }
}

model Atoms {
  param { lambda: Frequency = 0.5 /s }
  object Atom { }
  parts { atoms: Atom[100, max 100] }
  derived { left: Real = count(atoms) }
  for a in atoms {
    event decay on random(lambda) { destroy a }
  }
}

model Walk {
  param { s: Length = 1 m }
  discrete { x: Length = 0 m; heads: Real = 0 }
  event step on every 1 s from t0 + 1 s {
    set x = x + normal(0 m, s)
    set heads = heads + (if bernoulli(0.5) then 1 else 0)
  }
}

model Flip {
  param { k1: Frequency = 2 /s; k2: Frequency = 1 /s }
  discrete { a: Real = 300; b: Real = 0 }
  event forward on random(k1 * a) { set a = a - 1; set b = b + 1 }
  event back on random(k2 * b) { set a = a + 1; set b = b - 1 }
}

presentation DecayLab for Decay {
  panel controls {
    button(reset, label: "Replay")
    button(reseed, label: "Another run")
    label(n)
  }
  view spread: plot(x: [300, 440], y: [0, 30]) {
    histogram(n at t0 + 2 s over 100 runs, bins: 14)
  }
}

presentation DecayChecks for Decay {
  observe {
    n_end = n at t0 + 2 s
    ends  = n at t0 + 2 s over 100 runs
  }
}

run same of Decay with DecayChecks {
  config { seed = 42 }
  until t0 + 2 s
  expect { n_end == 378 exactly }
}

run many of Decay with DecayChecks {
  config { seed = 42 }
  until t0 + 2 s
  expect {
    mean(ends) == 367.88 within 8
    deviation(ends) in [11, 20]
    ends[1] == 378 exactly
  }
}
"#;

fn final_of(model: &str, binding: &str, t_end: f64, seeds: u64) -> Vec<f64> {
    let prog = program(SRC);
    let cm = prog.model(model);
    (0..seeds)
        .map(|seed| {
            let r = run(cm, Config::until(t_end).seed(seed));
            assert!(matches!(r.status, RunStatus::Completed), "{:?}", r.status);
            r.state_at(t_end)[cm.idx(binding)].num()
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

/// A population at rate `lambda n`: at the half-life, each of the 1000 nuclei is left with
/// probability 1/2 (binomial: mean 500, variance 250).
#[test]
fn population_decay_follows_the_exponential_law() {
    let half = 2f64.ln() / 0.5;
    near(&final_of("Decay", "Decay.n", half, 200), 500.0, 250.0);
}

/// One clock per member: 100 atoms, each left at 2 s with probability e^-1.
#[test]
fn atoms_decay_one_by_one() {
    let p = (-1f64).exp();
    near(&final_of("Atoms", "Atoms.left", 2.0, 200), 100.0 * p, 100.0 * p * (1.0 - p));
}

/// Draws at every step: after 25 steps, x has variance 25 s^2 and `heads` is binomial.
#[test]
fn random_walk_by_draws() {
    let x = final_of("Walk", "Walk.x", 25.0, 400);
    near(&x, 0.0, 25.0);
    let var = x.iter().map(|v| v * v).sum::<f64>() / x.len() as f64;
    assert!((var - 25.0).abs() < 5.0, "variance {var}");
    near(&final_of("Walk", "Walk.heads", 25.0, 400), 12.5, 6.25);
}

/// A reversible reaction reaches the equilibrium of its rate equations: b = N k1 / (k1 + k2),
/// binomial with p = 2/3.
#[test]
fn reversible_reaction_reaches_equilibrium() {
    near(&final_of("Flip", "Flip.b", 6.0, 100), 200.0, 300.0 * 2.0 / 3.0 / 3.0);
}

/// A seed reproduces a run bit for bit; another seed gives another run (RC-13.1, D-015).
#[test]
fn seeds_reproduce_runs() {
    let prog = program(SRC);
    let cm = prog.model("Decay");
    let times = |seed| run(cm, Config::until(2.0).seed(seed)).log.iter().map(|e| e.t.to_bits()).collect::<Vec<_>>();
    assert_eq!(times(7), times(7));
    assert_ne!(times(7), times(8));
    // The case's seed is honored, and its expectation pins this build's stream.
    for case in &prog.doc.runs {
        let r = run_case(&prog, case).unwrap();
        assert!(r.failures().is_empty(), "{}: {:?}", case.name, r.failures());
    }
}

/// An intervention resumed from a snapshot gives the same run as one computed from the start.
#[test]
fn resumed_runs_keep_their_streams() {
    let prog = program(SRC);
    let cm = prog.model("Flip");
    let first = run(cm, Config::until(4.0).seed(3));
    let cfg = Config::until(4.0).seed(3).at(2.0, prismal_runtime::Action::Intervene(vec![]));
    let resumed = prismal_runtime::resume(&first, cfg.clone());
    let fresh = run(cm, cfg);
    let bits = |r: &prismal_runtime::Run| r.log.iter().map(|e| (e.name.clone(), e.t.to_bits())).collect::<Vec<_>>();
    assert_eq!(bits(&resumed), bits(&fresh));
    assert_eq!(bits(&fresh), bits(&first));
}

#[test]
fn canonical_text_prints_random_triggers_draws_and_seeds() {
    let text = format(&compile(SRC).unwrap().doc);
    assert!(text.contains("event decay on random(lambda * n) {"), "{text}");
    assert!(text.contains("set x = x + normal(0 m, s)"), "{text}");
    assert!(text.contains("config { seed = 42 }"), "{text}");
    assert!(text.contains("histogram(n at t0 + 2 s over 100 runs, bins: 14)"), "{text}");
    assert!(text.contains("= n at t0 + 2 s over 100 runs"), "{text}");
    assert!(text.contains("mean(ends) == 367.88 within 8"), "{text}");
}

fn codes(src: &str) -> Vec<String> {
    let c = compile(src).unwrap();
    match prismal_present::Program::new(c.doc) {
        Ok(_) => vec![],
        Err(ds) => ds.iter().map(|d| d.to_string()).collect(),
    }
}

#[test]
fn draws_only_in_events_and_rates_without_continuous_state() {
    let d = codes("model M { param { s: Length = 1 m }\n derived { x: Length = normal(0 m, s) } }");
    assert!(d.iter().any(|m| m.contains("MK-E28")), "{d:?}");
    let d = codes("model M { state { y: Real = 1 }\n flow der(y) = -y / 1 s\n event e on random(y / 1 s) }");
    assert!(d.iter().any(|m| m.contains("MK-E27")), "{d:?}");
    let d = codes("model M { event e on random(2) }");
    assert!(d.iter().any(|m| m.contains("MK-E")), "a rate needs a unit per time: {d:?}");
}

/// A name declared by the author is not a distribution: version 1 programs keep their
/// meaning (working syntax section 1.7).
#[test]
fn declared_names_shadow_distributions() {
    let d = codes("model M { fn normal(x: Real): Real = 2 * x\n derived { y: Real = normal(3) } }");
    assert!(d.is_empty(), "{d:?}");
}

/// `reset` replays the run shown; `reseed` starts again with the next seed (D-084).
#[test]
fn a_lab_replays_or_draws_another_run() {
    use prismal_present::interact::Interactive;
    let prog = program(SRC);
    let mut lab = Interactive::new(&prog, "DecayLab", Config::until(5.0)).unwrap_or_else(|d| panic!("{d:?}"));
    let times = |i: &Interactive| i.session.current.log.iter().map(|e| e.t.to_bits()).collect::<Vec<_>>();
    let first = times(&lab);
    let id = |lab: &Interactive, k: usize| lab.frame().rep(&format!("button.{k}")).unwrap().id.clone();
    let (reset, reseed) = (id(&lab, 1), id(&lab, 2));
    lab.press(&reset).unwrap();
    assert_eq!(times(&lab), first, "reset replays the same run");
    assert_eq!(lab.seed(), 0);
    lab.press(&reseed).unwrap();
    assert_eq!(lab.seed(), 1);
    assert_ne!(times(&lab), first, "another seed, another run");
    assert_eq!(lab.t, 0.0, "a new run shows t0");
    let r = lab.frame().rep(&reseed).unwrap().text.clone();
    assert_eq!(r, "button Another run: reseed");
}

/// A histogram counts the runs of an ensemble in bins over the plot's x range (D-084); its
/// first run is the lab's own, and it is computed again only when the run changes.
#[test]
fn a_histogram_of_many_runs() {
    use prismal_present::frame::Shape;
    use prismal_present::interact::Interactive;
    let prog = program(SRC);
    let mut lab = Interactive::new(&prog, "DecayLab", Config::until(5.0)).unwrap_or_else(|d| panic!("{d:?}"));
    let hist = |lab: &Interactive| {
        let f = lab.frame();
        let r = f.reps().find(|r| r.kind == "histogram").unwrap().clone();
        let Shape::Polygon { points } = &r.shape else { panic!("{:?}", r.shape) };
        let total: f64 = points[1..points.len() - 1].chunks(2).map(|c| c[0][1]).sum();
        (total, r.text.clone())
    };
    let (total, text) = hist(&lab);
    assert!(text.starts_with("histogram of n at 2 s over 100 runs: mean "), "{text}");
    assert!(total > 95.0 && total <= 100.0, "nearly every run falls inside the axis: {total}");
    let before = text;
    lab.press(&lab.frame().rep("button.2").unwrap().id.clone()).unwrap();
    assert_ne!(hist(&lab).1, before, "another seed, another ensemble");
}


/// The whole distribution, not only its mean: the count at the half-life against the
/// binomial law in eight bins, by a chi-square test at the 0.1 % level (26.1 for 8 degrees of
/// freedom).
#[test]
fn population_decay_is_binomial() {
    let half = 2f64.ln() / 0.5;
    let xs = final_of("Decay", "Decay.n", half, 1000);
    let lnf = prismal_kernel::random::ln_factorial;
    let pmf = |k: f64| (lnf(1000.0) - lnf(k) - lnf(1000.0 - k) - 1000.0 * 2f64.ln()).exp();
    let mut chi = 0.0;
    for b in 0..8 {
        let (lo, hi) = (460 + 10 * b, 470 + 10 * b);
        let e = (lo..hi).map(|k| pmf(k as f64)).sum::<f64>() * xs.len() as f64;
        let o = xs.iter().filter(|&&x| x >= lo as f64 && x < hi as f64).count() as f64;
        chi += (o - e) * (o - e) / e;
    }
    assert!(chi < 26.1, "chi-square {chi}");
}

const POOL: &str = r#"model Pool {
  param { n0: Real = 1000 in [1, 5000]; lambda: Frequency = 0.5 /s }
  discrete { n: Real = n0 }
  event decay on random(lambda * n) { set n = n - 1 }
}

presentation PoolLab for Pool {
  panel controls {
    slider(n0, range: [1, 5000])
    button(reset)
    button(restart)
    button(reseed)
  }
  view spread: plot(x: [0, 20], y: [0, 100]) {
    histogram(n at t0 + 1 s over 100 runs, bins: 100)
  }
}

presentation PoolLesson for Pool {
  observe { left = n at t0 }
  timeline {
    scene s {
      beat a { run rate 1; wait 1 s }
      beat b { intervene { set n0 = 10 }; restart; hold; wait 1 s }
    }
  }
}

run lesson of Pool with PoolLesson {
  expect { left == 10 exactly }
}
"#;

/// `restart` runs again from `t0` with the parameters as they are; `reset` goes back to the
/// starting values; `reseed` restarts with the next seed (D-085).
#[test]
fn restart_keeps_the_settings() {
    use prismal_present::interact::Interactive;
    let prog = program(POOL);
    let mut lab = Interactive::new(&prog, "PoolLab", Config::until(5.0)).unwrap_or_else(|d| panic!("{d:?}"));
    let cm = prog.model("Pool");
    let n_at = |lab: &Interactive, t: f64| lab.session.current.state_at(t)[cm.idx("Pool.n")].num();
    let id = |lab: &Interactive, name: &str| lab.frame().rep(name).unwrap().id.clone();
    lab.seek(2.0);
    lab.set_control(&id(&lab, "slider.1"), prismal_ir::build::lit(10.0)).unwrap();
    assert!(n_at(&lab, 2.0) > 100.0, "a setting does not change the molecules of a run under way");
    lab.press(&id(&lab, "button.2")).unwrap();
    assert_eq!(n_at(&lab, 0.0), 10.0, "restart: from t0 with n0 = 10");
    assert_eq!(lab.t, 0.0);
    lab.press(&id(&lab, "button.3")).unwrap();
    assert_eq!((n_at(&lab, 0.0), lab.seed()), (10.0, 1), "reseed keeps the setting");
    lab.press(&id(&lab, "button.1")).unwrap();
    assert_eq!(n_at(&lab, 0.0), 1000.0, "reset: the presentation's own starting values");
    // In a lesson (the case above): after `restart` the run holds at t0 with ten nuclei.
    for case in &prog.doc.runs {
        let r = run_case(&prog, case).unwrap();
        assert!(r.failures().is_empty(), "{}: {:?}", case.name, r.failures());
    }
}

/// Counts lie on a grid of step 1, coarser than bins of 0.2: one bar per count, as wide as
/// the step, and no empty bins between them (PK-6.3e, D-085).
#[test]
fn histogram_bars_follow_the_grid_of_the_values() {
    use prismal_present::frame::Shape;
    use prismal_present::interact::Interactive;
    let prog = program(POOL);
    let mut lab = Interactive::new(&prog, "PoolLab", Config::until(5.0).param("Pool.n0", prismal_ir::build::lit(12.0))).unwrap();
    lab.seek(0.0);
    let f = lab.frame();
    let r = f.reps().find(|r| r.kind == "histogram").unwrap();
    let Shape::Polygon { points } = &r.shape else { panic!() };
    // Every bar edge is at a half-integer.
    for p in &points[1..points.len() - 1] {
        assert!(((p[0] - 0.5).fract()).abs() < 1e-9 || p[0] == 0.0 || p[0] == 20.0, "{p:?}");
    }
    let tops: f64 = points.windows(2).filter(|w| w[0][1] == w[1][1] && w[0][1] > 0.0).map(|w| w[0][1]).sum();
    assert_eq!(tops, 100.0, "every run in one bar");
}
