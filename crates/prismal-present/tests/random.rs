//! The stochastic slice (D-083): random events, draws in handlers, seeds. Statistical
//! checks compare means over many seeds with the exact law, within five standard errors.

mod common;
use common::*;
use prismal_present::expect::run_case;
use prismal_runtime::{run, Config, RunStatus};
use prismal_syntax::{compile, format};

const SRC: &str = "model Decay {
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

presentation DecayChecks for Decay {
  observe { n_end = n at t0 + 2 s }
}

run same of Decay with DecayChecks {
  config { seed = 42 }
  until t0 + 2 s
  expect { n_end == 378 exactly }
}
";

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
