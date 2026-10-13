//! Histograms of one run (D-089): the values of a sampled source, `x every Δ`, or at each
//! occurrence of an event, `x on E`, up to the instant shown.

mod common;
use common::*;
use prismal_present::frame::{Frame, Shape};
use prismal_present::interact::Interactive;
use prismal_runtime::Config;
use prismal_syntax::{compile, format};

const SRC: &str = r#"model Die {
  discrete { face: Real = 1 }
  event roll on every 1 s from t0 + 1 s { set face = integer(1, 6) }
}

model Clicks {
  param { lambda: Frequency = 0.5 /s }
  discrete {
    last: Time = 0 s
    gap:  Time = 0 s
  }
  event click on random(lambda) {
    set gap = elapsed - last
    set last = elapsed
  }
}

model Tank {
  state { h: Length = 2 m }
  flow der(h) = -0.1 m/s
}

presentation Rolls for Die {
  view faces: plot(x: [0.5, 6.5], y: [0, 400]) { histogram(face on roll, bins: 6) }
}

presentation Waits for Clicks {
  view gaps: plot(x: [0 s, 12 s], y: [0, 300]) { histogram(gap on click, bins: 24) }
}

presentation Levels for Tank {
  view levels: plot(x: [0.9 m, 2.1 m], y: [0, 5]) { histogram(h every 1 s, bins: 12) }
}
"#;

fn histogram(f: &Frame) -> (Vec<[f64; 2]>, String) {
    let r = f.reps().find(|r| r.kind == "histogram").unwrap();
    let Shape::Polygon { points } = &r.shape else { panic!("{:?}", r.shape) };
    (points.clone(), r.text.clone())
}

fn stat(text: &str, word: &str) -> f64 {
    text.split(&format!("{word} ")).nth(1).unwrap().split([',', ';']).next().unwrap().parse().unwrap()
}

/// The faces of 1200 rolls: six bars, one per face, each near 200, growing as the run plays.
#[test]
fn values_at_each_occurrence() {
    let prog = program(SRC);
    let lab = Interactive::new(&prog, "Rolls", Config::until(1200.0).seed(1)).unwrap_or_else(|d| panic!("{d:?}"));
    let (pts, text) = histogram(&lab.frame_at(1200.0));
    assert!(text.starts_with("histogram of face on roll up to 1200 s, 1200 values: mean "), "{text}");
    let bars: Vec<f64> = pts[1..pts.len() - 1].chunks(2).map(|c| c[0][1]).filter(|h| *h > 0.0).collect();
    assert_eq!(bars.len(), 6, "{pts:?}");
    assert!(bars.iter().all(|h| (h - 200.0).abs() < 5.0 * (1200.0f64 * (1.0 / 6.0) * (5.0 / 6.0)).sqrt()), "{bars:?}");
    assert!((stat(&text, "mean") - 3.5).abs() < 0.25, "{text}");
    let (_, early) = histogram(&lab.frame_at(10.5));
    assert!(early.contains("up to 10.5 s, 10 values"), "{early}");
    let (_, start) = histogram(&lab.frame_at(0.0));
    assert!(start.ends_with("no values yet"), "{start}");
}

/// The waiting times between random events are exponential: mean and deviation `1 / λ`.
#[test]
fn waiting_times_are_exponential() {
    let prog = program(SRC);
    let lab = Interactive::new(&prog, "Waits", Config::until(4000.0).seed(2)).unwrap_or_else(|d| panic!("{d:?}"));
    let (_, text) = histogram(&lab.frame_at(4000.0));
    let (mean, dev) = (stat(&text, "mean"), stat(&text, "deviation"));
    assert!((mean - 2.0).abs() < 5.0 * 2.0 / 2000f64.sqrt(), "{text}");
    assert!((dev - 2.0).abs() < 0.3, "{text}");
}

/// Samples of a flowing value: a level falling evenly spends equal time at every height.
#[test]
fn values_sampled_through_the_run() {
    let prog = program(SRC);
    let lab = Interactive::new(&prog, "Levels", Config::until(10.0)).unwrap_or_else(|d| panic!("{d:?}"));
    let (pts, text) = histogram(&lab.frame_at(10.0));
    assert!(text.starts_with("histogram of h every 1 s up to 10 s, 11 values: mean 1.5"), "{text}");
    assert!(pts.iter().all(|p| p[1] <= 1.0), "one sample per bin: {pts:?}");
}

#[test]
fn canonical_text_prints_histogram_sources() {
    let text = format(&compile(SRC).unwrap().doc);
    assert!(text.contains("histogram(face on roll, bins: 6)"), "{text}");
    assert!(text.contains("histogram(h every 1 s, bins: 12)"), "{text}");
    assert_eq!(format(&compile(&text).unwrap().doc), text);
}

#[test]
fn values_at_events_belong_to_representations() {
    let d = compile("model M { discrete { x: Real = 0 }\n event e on every 1 s\n fn f(a: Real): Real = a\n derived { y: Real = f(x on e) } }").err().unwrap();
    assert!(d.iter().any(|x| x.code == "SX-E08" && x.message.contains("on E")), "{d:?}");
}
