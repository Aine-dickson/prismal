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

const PLANE: &str = r#"space Plane = euclidean(2)

model Plate in Plane {
  grid plate(x: [0 m, 1 m], y: [0 m, 1 m], cells: (40, 40))
  param { alpha: Quantity<L^2/T> = 0.01 m^2/s }
  state { T(p in plate): Real = sin(pi * p.x / 1 m) * sin(pi * p.y / 1 m) }
  derived {
    centre: Real = T(origin + (0.5 m, 0.5 m))
    heat:   Area = integral(T)
  }
  flow der(T) = alpha * laplacian(T)
  boundary T {
    left = 0
    right = 0
    bottom = 0
    top = 0
  }
}

model Slope in Plane {
  grid sheet(x: [0 m, 2 m], y: [0 m, 1 m], cells: (20, 10))
  state { U(p in sheet): Real = p.x / 1 m }
  derived { gx(p in sheet): Quantity<1/L> = gradient(U).x }
  flow der(U) = 0 / 1 s
}

presentation PlateLab for Plate {
  view plate: spatial(Plane, scale: 1 m -> 300 px, y: up) {
    heatmap(T, range: [0, 1])
  }
}
"#;

/// A sine mode on a square plate held at 0 decays as `e^(-2 α π² t)`; its integral starts at
/// `4 / π²` square metres.
#[test]
fn heat_on_a_plate() {
    let prog = program(PLANE);
    let cm = prog.model("Plate");
    let r = run(cm, Config::until(5.0));
    assert!(matches!(r.status, RunStatus::Completed), "{:?}", r.status);
    let at = |t: f64, b: &str| r.state_at(t)[cm.idx(b)].num();
    let pi2 = std::f64::consts::PI.powi(2);
    assert!((at(0.0, "Plate.heat") - 4.0 / pi2).abs() < 1e-3, "{}", at(0.0, "Plate.heat"));
    // The centre lies between four cell centres: bilinear reading, close to the mode.
    let exact = (-2.0 * 0.01 * pi2 * 5.0).exp() * (std::f64::consts::PI / 80.0).cos().powi(2);
    assert!((at(5.0, "Plate.centre") - exact).abs() < 2e-3, "{} against {exact}", at(5.0, "Plate.centre"));
}

/// The gradient in the plane is a vector of the model's space.
#[test]
fn gradients_in_the_plane() {
    let prog = program(PLANE);
    let cm = prog.model("Slope");
    let r = run(cm, Config::until(0.1));
    let Value::Field(g) = &r.state_at(0.1)[cm.idx("Slope.gx")] else { panic!() };
    // Interior cells: 1 per metre; the insulated edges halve it at the ends of each row.
    for j in 0..10 {
        for i in 1..19 {
            assert!((g[i + 20 * j] - 1.0).abs() < 1e-9, "({i}, {j}): {}", g[i + 20 * j]);
        }
        assert!((g[20 * j] - 0.5).abs() < 1e-9);
    }
}

#[test]
fn a_heatmap_draws_the_plate() {
    use prismal_present::frame::Shape;
    let prog = program(PLANE);
    let lab = prismal_present::interact::Interactive::new(&prog, "PlateLab", Config::until(10.0)).unwrap_or_else(|d| panic!("{d:?}"));
    let f = lab.frame_at(0.0);
    let h = f.reps().find(|r| r.kind == "heatmap").unwrap();
    let Shape::Raster { cols, rows, values, palette, from, to, .. } = &h.shape else { panic!("{:?}", h.shape) };
    assert_eq!((*cols, *rows, values.len()), (40, 40, 1600));
    assert_eq!(palette, "sequential");
    assert_eq!((from[0], to[0]), (0.0, 300.0));
    assert!(to[1] < from[1], "y up: the top edge is above the bottom one in view coordinates");
    assert!(h.text.starts_with("heatmap of T: 40 by 40 cells, values from 0."), "{}", h.text);
    let text = format(&compile(PLANE).unwrap().doc);
    assert!(text.contains("grid plate(x: [0 m, 1 m], y: [0 m, 1 m], cells: (40, 40))"), "{text}");
    assert!(text.contains("T(p in plate): Real"), "{text}");
    assert_eq!(format(&compile(&text).unwrap().doc), text);
}

const TANK: &str = r#"space Plane = euclidean(2)

model Tank in Plane {
  grid tank(x: [0 m, 1 m], y: [0 m, 1 m], cells: (30, 30))
  param { c: Velocity = 0.5 m/s }
  state {
    h(p in tank): Length = 0 m
    v(p in tank): Velocity = 0 m/s
  }
  derived { level: Length = h(origin + (0.5 m, 0.5 m)) }
  flow {
    der(h) = v
    der(v) = c^2 * laplacian(h)
  }
  event drop on request(q: Point) {
    set h(p) = h + 0.01 m * exp(-(|p - q| / 0.05 m)^2)
  }
}

presentation TankLab for Tank {
  view tank: spatial(Plane, scale: 1 m -> 300 px, y: up) {
    heatmap(h)
    on click as q request drop(q)
  }
}
"#;

/// A requested event sets a field in each cell: a stone dropped where the learner clicks.
#[test]
fn a_handler_sets_a_field() {
    let prog = program(TANK);
    let mut lab = prismal_present::interact::Interactive::new(&prog, "TankLab", Config::until(4.0)).unwrap_or_else(|d| panic!("{d:?}"));
    let cm = prog.model("Tank");
    let level = |lab: &prismal_present::interact::Interactive, t: f64| lab.session.current.state_at(t)[cm.idx("Tank.level")].num();
    assert_eq!(level(&lab, 1.0), 0.0);
    lab.seek(1.0);
    let view = lab.frame().views[0].id.clone();
    // The middle of the tank, in view coordinates (300 px per metre, y up).
    lab.click_at(&view, [150.0, -150.0]).unwrap_or_else(|r| panic!("{r:?}"));
    let after = level(&lab, 1.0);
    // The middle is a corner of four cells, each centred 1/60 m from it along both axes.
    let exact = 0.01 * (-(2f64.sqrt() / 60.0 / 0.05).powi(2)).exp();
    assert!((after - exact).abs() < 1e-9, "{after} against {exact}");
    assert!(level(&lab, 3.0).abs() < 0.005, "the ripple spreads out: {}", level(&lab, 3.0));
    let text = format(&compile(TANK).unwrap().doc);
    assert!(text.contains("set h(p) = h + 0.01 m * exp("), "{text}");
    let d = codes(&TANK.replace("set h(p) = h +", "set h(p) = h + normal(0 m, 1 m) +"));
    assert!(d.iter().any(|m| m.contains("MK-E28")), "{d:?}");
}

/// A field flow with a choice per cell and `max` is evaluated over all cells at once (D-091);
/// the same flow through a declared function is evaluated cell by cell. Both give the same
/// run, bit for bit.
#[test]
fn choices_per_cell_match_cell_by_cell_evaluation() {
    let src = r#"model Direct {
  grid rod(x: [0 m, 1 m], cells: 80)
  param { k: Frequency = 2 /s }
  state { T(x in rod): Real = sin(pi * x / 1 m) }
  flow der(T(x)) = 0.01 m^2/s * laplacian(T) - (if x > 0.3 m and x < 0.7 m then k * (T - 0.5) else max(T - 0.9, 0) * k)
}

model Through {
  grid rod(x: [0 m, 1 m], cells: 80)
  param { k: Frequency = 2 /s }
  fn sink(t: Real, x: Length, k: Frequency): Frequency = if x > 0.3 m and x < 0.7 m then k * (t - 0.5) else max(t - 0.9, 0) * k
  state { T(x in rod): Real = sin(pi * x / 1 m) }
  flow der(T(x)) = 0.01 m^2/s * laplacian(T) - sink(T, x, k)
}
"#;
    let prog = program(src);
    let end = |m: &str| {
        let cm = prog.model(m);
        let r = run(cm, Config::until(2.0));
        assert!(matches!(r.status, RunStatus::Completed), "{:?}", r.status);
        match &r.final_value(&format!("{m}.T")) {
            Value::Field(v) => v.to_vec(),
            v => panic!("{v:?}"),
        }
    };
    let (a, b) = (end("Direct"), end("Through"));
    assert!(a.iter().zip(&b).all(|(x, y)| x.to_bits() == y.to_bits()), "{:?}\n{:?}", &a[..5], &b[..5]);
    assert!(a.iter().any(|x| *x < 0.5) && a.iter().any(|x| *x > 0.5));
}
