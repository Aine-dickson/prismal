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
