//! Solved equations (D-092): unknowns found at every instant from the equations that read
//! them, by Newton's method; checked against circuits and an equilibrium with known answers.

mod common;
use common::*;
use prismal_runtime::{run, Config, RunStatus};
use prismal_syntax::{compile, format};

const SRC: &str = r#"model Divider {
  param {
    V0: Voltage    = 9 V
    R1: Resistance = 100 Ω
    R2: Resistance = 200 Ω
  }
  unknown {
    I:  Current
    V2: Voltage
  }
  derived { P2: Power = V2 * I }
  equation loop: V0 == I * R1 + V2
  equation load: V2 == I * R2
}

model Charging {
  param {
    V0: Voltage     = 6 V
    R:  Resistance  = 1 kΩ
    C:  Capacitance = 1000 μF
  }
  state { Vc: Voltage = 0 V }
  unknown { I: Current }
  flow der(Vc) = I / C
  equation kvl: V0 == I * R + Vc
  event half on rising(Vc - V0 / 2)
}

model Bridge {
  param {
    V0: Voltage = 10 V
    R1: Resistance = 100 Ω
    R2: Resistance = 200 Ω
    R3: Resistance = 150 Ω
    R4: Resistance = 300 Ω
    R5: Resistance = 50 Ω
  }
  unknown {
    Va: Voltage
    Vb: Voltage
  }
  derived {
    I5: Current = (Va - Vb) / R5
    I0: Current = (V0 - Va) / R1 + (V0 - Vb) / R3
  }
  equation node_a: (V0 - Va) / R1 == Va / R2 + I5
  equation node_b: (V0 - Vb) / R3 + I5 == Vb / R4
}

model Diode {
  param {
    V0: Voltage = 5 V
    R:  Resistance = 1 kΩ
    Is: Current = 1e-12 A
    Vt: Voltage = 25.85 mV
  }
  unknown {
    I:  Current = 1 mA
    Vd: Voltage = 0.6 V
  }
  equation resistor: V0 - Vd == I * R
  equation shockley: I == Is * (exp(Vd / Vt) - 1)
}

model Equilibrium {
  param {
    K:  Real = 4
    a0: Real = 1
    b0: Real = 1
  }
  unknown { x: Real = 0.5 }
  equation mass_action: K * (a0 - x) * (b0 - x) == x^2
}
"#;

fn at(model: &str, binding: &str, t: f64) -> f64 {
    let prog = program(SRC);
    let cm = prog.model(model);
    let r = run(cm, Config::until(t.max(1e-9)));
    assert!(matches!(r.status, RunStatus::Completed), "{:?}", r.status);
    r.state_at(t)[cm.idx(binding)].num()
}

#[test]
fn a_voltage_divider() {
    assert!((at("Divider", "Divider.I", 0.0) - 0.03).abs() < 1e-12);
    assert!((at("Divider", "Divider.V2", 0.0) - 6.0).abs() < 1e-12);
    assert!((at("Divider", "Divider.P2", 0.0) - 0.18).abs() < 1e-12);
}

/// The current is an unknown of the loop equation; the capacitor charges as
/// `V0 (1 - e^(-t / RC))` and reaches half at `RC ln 2`.
#[test]
fn a_capacitor_charges() {
    let v = at("Charging", "Charging.Vc", 1.0);
    let exact = 6.0 * (1.0 - (-1.0f64).exp());
    assert!((v - exact).abs() < 1e-5, "{v} against {exact}");
    let i = at("Charging", "Charging.I", 1.0);
    assert!((i - 6e-3 * (-1.0f64).exp()).abs() < 1e-8, "{i}");
    let prog = program(SRC);
    let r = run(prog.model("Charging"), Config::until(2.0));
    let t = r.log.iter().find(|e| e.name == "half").expect("half").t;
    assert!((t - 2f64.ln()).abs() < 1e-6, "{t}");
}

/// A balanced bridge (R1 / R2 = R3 / R4) carries no current through its middle resistor;
/// unbalanced, the node equations give it.
#[test]
fn a_bridge() {
    assert!(at("Bridge", "Bridge.I5", 0.0).abs() < 1e-12);
    let prog = program(SRC);
    let cm = prog.model("Bridge");
    let ohm = prismal_ir::Unit::parse("Ω").unwrap();
    let cfg = Config::until(1e-9).param("Bridge.R4", prismal_ir::Expr::Num { num: 100.0, unit: Some(ohm) });
    let r = run(cm, cfg);
    assert!(matches!(r.status, RunStatus::Completed), "{:?}", r.status);
    let v = r.state_at(0.0);
    let (va, vb) = (v[cm.idx("Bridge.Va")].num(), v[cm.idx("Bridge.Vb")].num());
    // Kirchhoff's current law at both nodes holds to rounding.
    assert!(((10.0 - va) / 100.0 - va / 200.0 - (va - vb) / 50.0).abs() < 1e-12);
    assert!(((10.0 - vb) / 150.0 + (va - vb) / 50.0 - vb / 100.0).abs() < 1e-12);
    assert!(v[cm.idx("Bridge.I5")].num() > 0.0);
}

/// A diode in series with a resistor: a nonlinear system, found from the guesses.
#[test]
fn a_diode() {
    let i = at("Diode", "Diode.I", 0.0);
    let vd = at("Diode", "Diode.Vd", 0.0);
    assert!((5.0 - vd - i * 1000.0).abs() < 1e-12, "{vd} {i}");
    assert!((i - 1e-12 * ((vd / 0.02585).exp() - 1.0)).abs() < 1e-12 * i.max(1e-12) * 1e3, "{vd} {i}");
    assert!(vd > 0.5 && vd < 0.7, "{vd}");
}

/// The same circuit and guesses over a range of batteries, reversed ones included: the
/// solve must not depend on guesses near each solution.
#[test]
fn a_diode_over_many_batteries() {
    let prog = program(SRC);
    let cm = prog.model("Diode");
    let volt = prismal_ir::Unit::parse("V").unwrap();
    let mut failed = vec![];
    for v0 in [-50.0, -5.0, -0.5, 0.0, 1e-3, 0.3, 0.6, 1.0, 5.0, 50.0, 1000.0] {
        let cfg = Config::until(1e-9).param("Diode.V0", prismal_ir::Expr::Num { num: v0, unit: Some(volt.clone()) });
        let r = run(cm, cfg);
        if !matches!(r.status, RunStatus::Completed) {
            failed.push(v0);
            continue;
        }
        let v = r.state_at(0.0);
        let (i, vd) = (v[cm.idx("Diode.I")].num(), v[cm.idx("Diode.Vd")].num());
        assert!((v0 - vd - i * 1000.0).abs() <= 1e-9 * (v0.abs() + vd.abs()).max(1e-12), "V0 = {v0}: {vd} {i}");
        let shockley = 1e-12 * ((vd / 0.02585).exp() - 1.0);
        assert!((i - shockley).abs() <= 1e-9 * (i.abs() + shockley.abs()).max(1e-24) + 1e-27, "V0 = {v0}: {vd} {i}");
        if v0 < 0.0 {
            // Reversed, the diode blocks: the leakage current, nearly all the battery across it.
            assert!((i + 1e-12).abs() < 1e-15 && (vd - v0).abs() < 1e-6, "V0 = {v0}: {vd} {i}");
        }
    }
    assert!(failed.is_empty(), "no solution for V0 = {failed:?}");
}

fn diode_guessed(guess: &str) -> String {
    format!("model D {{
  param {{
    V0: Voltage = 5 V
    R:  Resistance = 1 kΩ
    Is: Current = 1e-12 A
    Vt: Voltage = 25.85 mV
  }}
  unknown {{
    I:  Current
    Vd: Voltage{guess}
  }}
  equation resistor: V0 - Vd == I * R
  equation shockley: I == Is * (exp(Vd / Vt) - 1)
}}
")
}

/// Guesses far from the solution on either side of the diode's knee: below it a full Newton
/// step overshoots into the exponential, above it a full step moves only about `Vt`, and at
/// 10 V the diode's current is near 1e156 A. Each reaches the solution.
#[test]
fn a_diode_from_far_guesses() {
    let volt = prismal_ir::Unit::parse("V").unwrap();
    let mut failed = vec![];
    for guess in ["", " = 0.2 V", " = 2 V", " = 10 V", " = -3 V", " = -1000 V"] {
        let prog = program(&diode_guessed(guess));
        let cm = prog.model("D");
        for v0 in [-5.0, 0.0, 0.7, 5.0, 1000.0] {
            let r = run(cm, Config::until(1e-9).param("D.V0", prismal_ir::Expr::Num { num: v0, unit: Some(volt.clone()) }));
            if !matches!(r.status, RunStatus::Completed) {
                failed.push((guess, v0));
                continue;
            }
            let v = r.state_at(0.0);
            let (i, vd) = (v[cm.idx("D.I")].num(), v[cm.idx("D.Vd")].num());
            assert!((v0 - vd - i * 1000.0).abs() <= 1e-9 * (v0.abs() + vd.abs()).max(1e-12), "{guess} V0 = {v0}: {vd} {i}");
        }
    }
    assert!(failed.is_empty(), "no solution for {failed:?}");
    // At 25 V the guess itself overflows: the run cannot start, and the message says so.
    let prog = program(&diode_guessed(" = 25 V"));
    match &run(prog.model("D"), Config::until(1.0)).status {
        RunStatus::NotStarted(d) => assert!(d.message.contains("cannot be evaluated at the guesses"), "{}", d.message),
        s => panic!("{s:?}"),
    }
}

/// `K (1 - x)² = x²` with `K = 4` has the root `x = 2/3` in `[0, 1]`, found from 0.5.
#[test]
fn an_equilibrium() {
    assert!((at("Equilibrium", "Equilibrium.x", 0.0) - 2.0 / 3.0).abs() < 1e-12);
}

#[test]
fn canonical_text_prints_unknowns() {
    let text = format(&compile(SRC).unwrap().doc);
    assert!(text.contains("unknown {\n    I:  Current\n    V2: Voltage\n  }"), "{text}");
    assert!(text.contains("x: Real = 0.5"), "{text}");
    assert!(text.contains("R:  Resistance  = 1 kΩ"), "{text}");
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
fn solves_are_checked() {
    let d = codes("model M { unknown { x: Real; y: Real }\n equation a: x == 1 }");
    assert!(d.iter().any(|m| m.contains("MK-E34") && m.contains("`y`")), "{d:?}");
    let d = codes("model M { unknown { x: Real }\n equation a: x == 1\n equation b: 2 * x == 2 }");
    assert!(d.iter().any(|m| m.contains("MK-E34") && m.contains("more equations")), "{d:?}");
    let d = codes("model M { param { k: Real = 1 }\n unknown { x: Real; y: Real }\n equation a: x + y == k\n equation b: x * k == 1\n equation c: x == k }");
    assert!(d.iter().any(|m| m.contains("MK-E34")), "{d:?}");
    // Dependent equations are found when solving: the run cannot start.
    let prog = program("model M { unknown { x: Real; y: Real }\n equation a: x + y == 1\n equation b: 2 * x + 2 * y == 2 }");
    let r = run(prog.model("M"), Config::until(1.0));
    match &r.status {
        RunStatus::NotStarted(d) => assert!(d.message.contains("dependent"), "{}", d.message),
        s => panic!("{s:?}"),
    }
    // A checked equation stays a check.
    let d = codes("model M { unknown { x: Real }\n equation a: x == 2\n equation b: x == 2 checked within 1e-9 }");
    assert!(d.is_empty(), "{d:?}");
}

const RATES: &str = r#"model Coil {
  param {
    V0: Voltage    = 12 V
    R:  Resistance = 6 Ω
    L:  Inductance = 3 H
  }
  state { I: Current = 0 A }
  equation faraday: L * der(I) == V0 - R * I
}

model Spring {
  param {
    m: Mass = 2 kg
    k: Quantity<M/T^2> = 8 N/m
  }
  state {
    x: Length = 0.1 m
    v: Velocity = 0 m/s
  }
  flow der(x) = v
  equation newton: m * der(v) == -k * x
  event turn on rising(v)
}
"#;

/// An equation that gives a rate: the current through a coil rises as `V0/R (1 - e^(-Rt/L))`.
#[test]
fn equations_give_rates() {
    let prog = program(RATES);
    let cm = prog.model("Coil");
    let r = run(cm, Config::until(1.0));
    assert!(matches!(r.status, RunStatus::Completed), "{:?}", r.status);
    let i = r.state_at(1.0)[cm.idx("Coil.I")].num();
    let exact = 2.0 * (1.0 - (-2.0f64).exp());
    assert!((i - exact).abs() < 1e-6, "{i} against {exact}");
    // Newton's law as an equation: the spring turns after half a period, π sqrt(m / k).
    let cm = prog.model("Spring");
    let r = run(cm, Config::until(2.0));
    let t = r.log.iter().find(|e| e.name == "turn").expect("turn").t;
    assert!((t - std::f64::consts::PI / 2.0).abs() < 1e-6, "{t}");
    // The authored text keeps the equation as written.
    let text = format(&compile(RATES).unwrap().doc);
    assert!(text.contains("equation faraday: L * der(I) == V0 - R * I"), "{text}");
    assert!(!text.contains("@rate"), "{text}");
}
