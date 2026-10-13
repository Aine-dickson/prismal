//! Instruments (PK-3.7, D-087): readings rounded to a resolution, limited to a range and
//! disturbed by noise from the run's seed, in observations and in representations alike.

mod common;
use common::*;
use prismal_present::data::Data;
use prismal_present::expect::run_case;
use prismal_present::frame::Shape;
use prismal_present::interact::Interactive;
use prismal_runtime::Config;
use prismal_syntax::{compile, format};

const SRC: &str = r#"model Tank {
  state { h: Length = 2 m }
  flow der(h) = -0.1 m/s
}

presentation TankLab for Tank {
  instrument stick(resolution: 0.1 m)
  instrument gauge(range: [0 m, 1.5 m])
  instrument float(noise: 0.05 m)
  view level: plot(x: [0 s, 10 s], y: [0 m, 2 m]) {
    series_plot(h every 0.5 s)
    series_plot(float(h) every 0.5 s)
    series_plot(gauge(h) every 0.5 s)
  }
  view spread: plot(x: [1.6 m, 2 m], y: [0, 60]) {
    histogram(float(h) at t0 + 2 s over 200 runs, bins: 16)
  }
  panel readings { table(stick(h) every 1 s) }
  observe {
    rounded = stick(h) at t0 + 3.04 s
    limited = gauge(h) every 1 s
    noisy   = float(h) every 0.01 s
    drawn   = float(h) every 0.5 s
  }
}

run readings of Tank with TankLab {
  config { seed = 3 }
  until t0 + 10 s
  expect {
    rounded == 1.7 m exactly
    limited[1] == 1.5 m within 1e-9 m
    deviation(noisy) in [0.28, 0.30]
  }
}
"#;

fn series(i: &Interactive, name: &str) -> Vec<(f64, f64)> {
    match i.observe(name).unwrap() {
        Data::Series(s) => s.iter().map(|(t, v)| (*t, v.num())).collect(),
        d => panic!("{d:?}"),
    }
}

fn lab(seed: u64) -> Interactive {
    let prog = program(SRC);
    Interactive::new(&prog, "TankLab", Config::until(10.0).seed(seed)).unwrap_or_else(|d| panic!("{d:?}"))
}

/// A resolution rounds; a range leaves out what lies outside it; noise has the declared
/// deviation about the true value and no bias.
#[test]
fn readings_follow_the_instrument() {
    let i = lab(3);
    let limited = series(&i, "limited");
    assert_eq!(limited.len(), 6, "from 5 s, where h falls to 1.5 m: {limited:?}");
    assert!(limited.iter().all(|(_, h)| *h <= 1.5 + 1e-9));
    let noisy = series(&i, "noisy");
    let res: Vec<f64> = noisy.iter().map(|(t, h)| h - (2.0 - 0.1 * t)).collect();
    let n = res.len() as f64;
    let mean = res.iter().sum::<f64>() / n;
    let dev = (res.iter().map(|r| (r - mean) * (r - mean)).sum::<f64>() / (n - 1.0)).sqrt();
    assert!(mean.abs() < 5.0 * 0.05 / n.sqrt(), "mean error {mean}");
    assert!((dev - 0.05).abs() < 0.005, "deviation {dev}");
}

/// The case's expectations hold, and its seed pins the readings.
#[test]
fn cases_check_readings() {
    let prog = program(SRC);
    for case in &prog.doc.runs {
        let r = run_case(&prog, case).unwrap();
        assert!(r.failures().is_empty(), "{}: {:?}", case.name, r.failures());
    }
}

/// A reading depends on the seed, the instrument, the source and the instant only: the
/// series drawn is the observation, and the same seed gives the same readings.
#[test]
fn the_reading_drawn_is_the_reading_observed() {
    let i = lab(3);
    let f = i.frame_at(i.end_time());
    let r = f.reps().find(|r| r.text.contains("measured with float")).unwrap();
    let points = dots(&r.shape);
    assert!(r.text.starts_with("readings of h against elapsed time: 21 readings"), "{}", r.text);
    let drawn = series(&i, "drawn");
    for (p, (t, h)) in points.iter().zip(&drawn) {
        assert_eq!((p[0], p[1]), (*t, *h));
    }
    assert!(r.text.contains("uncertainty 0.05"), "{}", r.text);
    assert_eq!(series(&lab(3), "noisy"), series(&i, "noisy"));
    assert_ne!(series(&lab(4), "noisy"), series(&i, "noisy"));
}

/// The points of a measured series: its readings are drawn one by one (D-087).
fn dots(s: &Shape) -> Vec<[f64; 2]> {
    let Shape::Group { members } = s else { panic!("{s:?}") };
    members.iter().map(|m| if let Shape::Point { at } = m.shape { at } else { panic!("{:?}", m.shape) }).collect()
}

/// Out of range, a series plot shows no reading and a table says so.
#[test]
fn out_of_range_readings_leave_gaps() {
    let i = lab(0);
    let f = i.frame_at(i.end_time());
    let gauge = f.reps().find(|r| r.text.contains("measured with gauge")).unwrap();
    let points = dots(&gauge.shape);
    assert!(points[0][0] >= 5.0 - 1e-9, "{points:?}");
    let table = f.reps().find(|r| r.kind == "table").unwrap();
    let Shape::Table { rows, .. } = &table.shape else { panic!() };
    assert_eq!(rows[3][1], "1.7 m", "{rows:?}");
}

/// One quantity measured in many runs: the spread of the readings is the instrument's.
#[test]
fn repeated_measurements_spread_by_the_noise() {
    let i = lab(0);
    let f = i.frame_at(i.end_time());
    let h = f.reps().find(|r| r.kind == "histogram").unwrap();
    let dev: f64 = h.text.split("deviation ").nth(1).unwrap().split(';').next().unwrap().parse().unwrap();
    assert!((dev - 0.05).abs() < 0.01, "{}", h.text);
    let mean: f64 = h.text.split("mean ").nth(1).unwrap().split(',').next().unwrap().parse().unwrap();
    assert!((mean - 1.8).abs() < 5.0 * 0.05 / 200f64.sqrt(), "{}", h.text);
}

#[test]
fn canonical_text_prints_instruments() {
    let text = format(&compile(SRC).unwrap().doc);
    assert!(text.contains("instrument stick(resolution: 0.1 m)"), "{text}");
    assert!(text.contains("instrument gauge(range: [0 m, 1.5 m])"), "{text}");
    assert!(text.contains("series_plot(float(h) every 0.5 s)"), "{text}");
    assert!(text.contains("rounded = stick(h) at t0 + 3.04 s"), "{text}");
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
fn instruments_are_checked() {
    let model = "model Tank { state { h: Length = 2 m }\n flow der(h) = -0.1 m/s }\n";
    let with = |pres: &str| codes(&format!("{model}presentation P for Tank {{\n{pres}\n}}"));
    let d = with("instrument clock(noise: 1 s)\n observe { x = clock(h) every 1 s }");
    assert!(d.iter().any(|m| m.contains("PK-E04") && m.contains("dimension")), "{d:?}");
    let d = with("instrument stick(resolution: 1 cm)\n observe { x = stick(h) live }");
    assert!(d.iter().any(|m| m.contains("PK-E04") && m.contains("live")), "{d:?}");
    let d = with("instrument stick(resolutoin: 1 cm)");
    assert!(d.iter().any(|m| m.contains("SX-E08") && m.contains("did you mean `resolution`")), "{d:?}");
    let d = with("instrument stick(resolution: -1 cm)\n observe { x = stick(h) every 1 s }");
    assert!(d.iter().any(|m| m.contains("positive")), "{d:?}");
    let d = with("instrument stick(resolution: 1 cm)\n panel p { label(stick(h)) }");
    assert!(d.iter().any(|m| m.contains("SX-E08") && m.contains("reads at instants")), "{d:?}");
}
