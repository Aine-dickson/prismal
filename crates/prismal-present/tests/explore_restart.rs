//! Runtime-control buttons in explore beats (D-088): `restart` and `reseed` start the
//! learner's branch again from `t0` with the branch's settings, without leaving the beat.

mod common;
use common::*;
use prismal_ir::present::LearnerInput;
use prismal_present::timeline::{play, Input, Medium, Playback};
use prismal_runtime::Config;
use prismal_syntax::{compile, format};

const SRC: &str = r#"model Sample {
  param {
    lambda: Frequency = 0.5 /s
    n0:     Real      = 1000 in [1, 5000]
  }
  discrete { n: Real = n0 }
  event decay on random(lambda * n) { set n = n - 1 }
}

presentation Lesson for Sample {
  view counts: plot(x: [0 s, 10 s], y: [0, 1000]) { series_plot(n every 0.1 s) }
  timeline {
    scene s {
      beat watch { run rate 1; wait 2 s }
      beat try {
        explore limit 30 s {
          slider(n0, range: [10, 1000])
          button(restart, label: "Start again")
          button(reseed, label: "Another run")
        }
      }
      beat after { run rate 1; wait 3 s }
    }
  }
}

run tries of Sample with Lesson {
  config { seed = 7 }
  until t0 + 10 s
  learner {
    at 3 s: set slider n0 = 20
    at 4 s: press restart
    at 6 s: press reseed
    at 8 s: continue
  }
}
"#;

const N0: &str = "Sample.n0";

fn lesson(inputs: Vec<Input>) -> Playback {
    play(&program(SRC), "Lesson", Config::until(10.0).seed(7), Medium::Interactive, inputs).unwrap_or_else(|d| panic!("{d:?}"))
}

fn at(at: f64, input: LearnerInput) -> Input {
    Input { at, input }
}

fn control(c: &str) -> LearnerInput {
    LearnerInput::Control { control: c.into() }
}

fn set_n0(v: f64) -> LearnerInput {
    LearnerInput::SetControl { control: "slider".into(), binding: N0.into(), value: prismal_ir::build::lit(v) }
}

/// The count shown at presentation instant `p`.
fn n_at(pb: &Playback, p: f64) -> (u64, f64, f64) {
    let (ri, t) = pb.sim_at(p);
    let v = &pb.runs[ri];
    let n = v.run.state_at(t)[v.run.model.idx("Sample.n")].num();
    (v.run.config.seed, t, n)
}

#[test]
fn restart_and_reseed_start_the_branch_again() {
    let pb = lesson(vec![at(3.0, set_n0(20.0)), at(4.0, control("restart")), at(6.0, control("reseed")), at(8.0, LearnerInput::Continue)]);
    assert!(pb.refusals.is_empty(), "{:?}", pb.refusals);
    // Just after the restart: the branch from t0, with the learner's 20 nuclei.
    let (seed, t, n) = n_at(&pb, 4.0);
    assert_eq!((seed, t), (7, 0.0));
    assert_eq!(n, 20.0);
    // A second later the branch has played a second of its new run.
    let (_, t, n) = n_at(&pb, 5.0);
    assert!((t - 1.0).abs() < 1e-9 && n <= 20.0, "{t} {n}");
    // `reseed`: the next seed, the same settings.
    let (seed, t, n) = n_at(&pb, 6.0);
    assert_eq!((seed, t, n), (8, 0.0, 20.0));
    // After the beat, the lesson goes back to its own run at the instant it branched.
    let (seed, t, n) = n_at(&pb, 8.0);
    assert_eq!((seed, t), (7, 2.0));
    assert!(n > 300.0, "{n}");
}

#[test]
fn controls_not_offered_are_refused() {
    let pb = lesson(vec![at(1.0, control("restart"))]);
    assert_eq!(pb.refusals.len(), 1);
    assert!(pb.refusals[0].reason.contains("outside an explore beat"), "{:?}", pb.refusals);
    let src = SRC.replace("          button(reseed, label: \"Another run\")\n", "");
    let pb = play(&program(&src), "Lesson", Config::until(10.0), Medium::Interactive, vec![at(3.0, control("reseed"))]).unwrap();
    assert_eq!(pb.refusals.len(), 1);
    assert_eq!(pb.refusals[0].reason, "try offers no button(reseed) (PK-9.8)");
}

#[test]
fn other_runtime_controls_stay_in_labs() {
    let src = SRC.replace("button(restart, label: \"Start again\")", "button(reset)");
    let c = compile(&src).unwrap();
    let d = match prismal_present::Program::new(c.doc) {
        Ok(_) => vec![],
        Err(ds) => ds.iter().map(|d| d.to_string()).collect::<Vec<_>>(),
    };
    assert!(d.iter().any(|m| m.contains("PK-E05") && m.contains("explore beat may offer")), "{d:?}");
}

#[test]
fn learner_presses_of_controls_are_written_and_read() {
    let text = format(&compile(SRC).unwrap().doc);
    assert!(text.contains("at 4 s: press restart"), "{text}");
    assert!(text.contains("at 6 s: press reseed"), "{text}");
    let doc = compile(&text).unwrap().doc;
    assert!(matches!(&doc.runs[0].learner[1].input, LearnerInput::Control { control } if control == "restart"));
}
