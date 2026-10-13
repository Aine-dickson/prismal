//! Fields on grids (D-091): field state over a grid of cells, flows with `laplacian` and
//! `gradient`, boundary conditions, readings at a point and over the field, checked against
//! the exact solutions of the heat and wave equations.

mod common;
use common::*;
use prismal_kernel::Value;
use prismal_runtime::{run, Config, RunStatus};
use prismal_syntax::{compile, format};

const SRC: &str = r#"model Rod {
  grid rod(x: [0 m, 1 m], cells: 100)
  param { alpha: Quantity<L^2/T> = 0.01 m^2/s }
  state { T(x in rod): Real = sin(pi * x / 1 m) }
  derived {
    middle: Real = T(0.5 m)
    total:  Length = integral(T)
  }
  flow der(T) = alpha * laplacian(T)
  boundary T {
    left = 0
    right = 0
  }
  event cooled on falling(middle - 0.5)
}

model Insulated {
  grid rod(x: [0 m, 1 m], cells: 50)
  param { alpha: Quantity<L^2/T> = 0.01 m^2/s }
  state { T(x in rod): Real = 1 + cos(pi * x / 1 m) }
  derived {
    total: Length = integral(T)
    hot:   Real = max(T)
    cold:  Real = min(T)
  }
  flow der(T) = alpha * laplacian(T)
}

model Held {
  grid rod(x: [0 m, 2 m], cells: 40)
  param {
    alpha: Quantity<L^2/T> = 0.5 m^2/s
    hot:   Real = 100
  }
  state { T(x in rod): Real = 0 }
  derived { flux(x in rod): Quantity<L/T> = -alpha * gradient(T) }
  flow der(T) = alpha * laplacian(T)
  boundary T {
    left = hot
    right = 0
  }
}

model String {
  grid line(x: [0 m, 1 m], cells: 200)
  param { c: Velocity = 1 m/s }
  state {
    u(x in line): Length = 0.01 m * sin(pi * x / 1 m)
    v(x in line): Velocity = 0 m/s
  }
  flow {
    der(u) = v
    der(v) = c^2 * laplacian(u)
  }
  boundary u {
    left = 0 m
    right = 0 m
  }
}

model Ring {
  grid ring(x: [0 m, 1 m], cells: 64)
  param { D: Quantity<L^2/T> = 0.02 m^2/s }
  state { n(x in ring): Real = if x < 0.25 m then 4 else 0 }
  derived { amount: Length = integral(n) }
  flow der(n(x)) = D * laplacian(n) + 0 / 1 s * x / 1 m
  boundary n periodic
}
"#;

fn final_value(model: &str, binding: &str, t_end: f64) -> Value {
    let prog = program(SRC);
    let cm = prog.model(model);
    let r = run(cm, Config::until(t_end));
    assert!(matches!(r.status, RunStatus::Completed), "{:?}", r.status);
    r.state_at(t_end)[cm.idx(binding)].clone()
}

/// A sine mode between edges held at 0 decays as `e^(-α π² t / L²)`.
#[test]
fn heat_decays_as_its_mode() {
    let exact = (-0.01 * std::f64::consts::PI.powi(2) * 5.0).exp();
    let m = final_value("Rod", "Rod.middle", 5.0).num();
    assert!((m - exact).abs() < 1e-4, "{m} against {exact}");
    // The integral of the sine is 2 L / π at the start.
    let prog = program(SRC);
    let cm = prog.model("Rod");
    let r = run(cm, Config::until(10.0));
    let total0 = r.state_at(0.0)[cm.idx("Rod.total")].num();
    assert!((total0 - 2.0 / std::f64::consts::PI).abs() < 1e-4, "{total0}");
    // The middle falls below one half at `ln 2 / (α π²)`.
    let t = r.log.iter().find(|e| e.name == "cooled").expect("cooled").t;
    let exact_t = 2f64.ln() / (0.01 * std::f64::consts::PI.powi(2));
    assert!((t - exact_t).abs() < 1e-2, "{t} against {exact_t}");
}

/// Insulated edges keep the total; the cosine mode decays to the mean.
#[test]
fn insulated_edges_keep_the_heat() {
    let prog = program(SRC);
    let cm = prog.model("Insulated");
    let r = run(cm, Config::until(10.0));
    let at = |t: f64, b: &str| r.state_at(t)[cm.idx(b)].num();
    assert!((at(0.0, "Insulated.total") - 1.0).abs() < 1e-12);
    assert!((at(10.0, "Insulated.total") - 1.0).abs() < 1e-9, "{}", at(10.0, "Insulated.total"));
    let amp = (-0.01 * std::f64::consts::PI.powi(2) * 10.0).exp();
    let hot = at(10.0, "Insulated.hot");
    // The hottest cell is the first centre, at x = h / 2.
    let exact = 1.0 + amp * (std::f64::consts::PI * 0.01).cos();
    assert!((hot - exact).abs() < 1e-3, "{hot} against {exact}");
    assert!(at(10.0, "Insulated.cold") < 1.0);
}

/// Edges held at 100 and 0 settle to a straight line, with the same flux everywhere.
#[test]
fn held_edges_settle_to_a_line() {
    let v = final_value("Held", "Held.T", 20.0);
    let Value::Field(d) = &v else { panic!("{v}") };
    for (k, t) in d.iter().enumerate() {
        let x = (k as f64 + 0.5) * 0.05;
        assert!((t - (100.0 - 50.0 * x)).abs() < 0.01, "cell {k}: {t}");
    }
    let Value::Field(q) = final_value("Held", "Held.flux", 20.0) else { panic!() };
    assert!(q.iter().all(|f| (f - 25.0).abs() < 0.01), "{q:?}");
}

/// A string held at both ends, released from a sine, is the opposite sine after half a
/// period `L / c`.
#[test]
fn a_string_vibrates_with_its_period() {
    let prog = program(SRC);
    let cm = prog.model("String");
    let r = run(cm, Config::until(1.0));
    let Value::Field(u) = &r.state_at(1.0)[cm.idx("String.u")] else { panic!() };
    let mid = 0.5 * (u[99] + u[100]);
    let exact = -0.01 * (std::f64::consts::PI * 0.0025).cos();
    assert!((mid - exact).abs() < 1e-5, "{mid} against {exact}");
}

/// A periodic grid wraps around and keeps the amount; the flow may read the coordinate.
#[test]
fn periodic_fields_keep_their_amount() {
    let prog = program(SRC);
    let cm = prog.model("Ring");
    let r = run(cm, Config::until(5.0));
    let a = |t: f64| r.state_at(t)[cm.idx("Ring.amount")].num();
    assert!((a(0.0) - 1.0).abs() < 1e-12 && (a(5.0) - 1.0).abs() < 1e-9, "{} {}", a(0.0), a(5.0));
    // Spread both ways: the cell just before the wrap gains as the cell after the step does.
    let Value::Field(n) = &r.state_at(5.0)[cm.idx("Ring.n")] else { panic!() };
    assert!((n[63] - n[16]).abs() < 1e-6, "{} {}", n[63], n[16]);
}

#[test]
fn canonical_text_prints_fields() {
    let text = format(&compile(SRC).unwrap().doc);
    assert!(text.contains("grid rod(x: [0 m, 1 m], cells: 100)"), "{text}");
    assert!(text.contains("T(x in rod): Real = sin(pi * x / 1 m)") || text.contains("T(x in rod): Real = sin(π * x / 1 m)"), "{text}");
    assert!(text.contains("der(T) = alpha * laplacian(T)"), "{text}");
    assert!(text.contains("der(n(x)) = "), "{text}");
    assert!(text.contains("boundary n periodic"), "{text}");
    assert!(text.contains("boundary T {\n    left = 0\n    right = 0\n  }"), "{text}");
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
fn fields_are_checked() {
    let head = "model M {\n grid a(x: [0 m, 1 m], cells: 10)\n grid b(x: [0 m, 2 m], cells: 10)\n state { T(x in a): Real = 0; U(x in b): Real = 0; s: Real = 0 }\n";
    let with = |rest: &str| codes(&format!("{head}{rest}\n}}"));
    let d = with("flow der(s) = laplacian(T) * 1 m^2 / 1 s");
    assert!(d.iter().any(|m| m.contains("MK-E32")), "{d:?}");
    let d = with("flow der(T) = U / 1 s");
    assert!(d.iter().any(|m| m.contains("MK-E31")), "{d:?}");
    let d = with("boundary T { top = 0 }");
    assert!(d.iter().any(|m| m.contains("MK-E33")), "{d:?}");
    let d = with("flow der(T) = noise / 1 s^0.5");
    assert!(d.iter().any(|m| m.contains("MK-E29")), "{d:?}");
    let d = with("flow der(s) = integral(T) / 1 m / 1 s");
    assert!(d.is_empty(), "a whole-field reading is a number: {d:?}");
    let d = codes("model M {\n grid a(x: [0 m, 1 m], cells: 0)\n state { T(x in a): Real = 0 }\n}");
    assert!(d.iter().any(|m| m.contains("whole number of cells")), "{d:?}");
    let d = codes("model M {\n state { T(x in nowhere): Real = 0 }\n}");
    assert!(d.iter().any(|m| m.contains("unknown grid")), "{d:?}");
}

/// An equation over a field holds in every cell; a checked one is checked in each (D-091),
/// and a presentation typesets it.
#[test]
fn equations_over_fields() {
    let src = "model Heat {\n grid rod(x: [0 m, 1 m], cells: 20)\n param { alpha: Quantity<L^2/T> = 0.01 m^2/s }\n state { T(x in rod): Real = sin(pi * x / 1 m) }\n flow der(T) = alpha * laplacian(T)\n equation heat: der(T) == alpha * laplacian(T) checked within 1e-9 /s\n equation wrong: der(T) == 2 * alpha * laplacian(T) checked within 1e-9 /s\n}\npresentation Show for Heat {\n panel p { equation(heat) }\n}\n";
    let prog = program(src);
    let cm = prog.model("Heat");
    let r = run(cm, Config::until(1.0));
    let named = |n: &str| r.diagnostics.iter().filter(|d| d.message.contains(&format!("`{n}`"))).count();
    assert_eq!(named("heat"), 0, "{:?}", r.diagnostics);
    assert!(named("wrong") > 0, "{:?}", r.diagnostics);
    let lab = prismal_present::interact::Interactive::new(&prog, "Show", Config::until(1.0)).unwrap_or_else(|d| panic!("{d:?}"));
    let f = lab.frame_at(0.5);
    let e = f.reps().find(|r| r.kind == "equation").unwrap();
    assert!(e.text.contains("heat"), "{}", e.text);
}
