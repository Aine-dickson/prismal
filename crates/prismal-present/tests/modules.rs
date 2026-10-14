//! Shared definitions and modules (D-099): object types, enumerations and functions written
//! outside models, used by models of any space, and imported from modules with `use`.

use prismal_present::Program;
use prismal_runtime::{run, Config, RunStatus};
use prismal_syntax::{compile, compile_with, format, MapModules, Modules};

fn program_with(src: &str, modules: &dyn Modules) -> Program {
    let c = compile_with(src, modules).unwrap_or_else(|ds| panic!("{}", ds.iter().map(|d| d.render(src, "program")).collect::<Vec<_>>().join("\n")));
    Program::new(c.doc).unwrap_or_else(|ds| panic!("{}", ds.iter().map(|d| d.to_string()).collect::<Vec<_>>().join("\n")))
}

fn at(prog: &Program, model: &str, binding: &str, t: f64) -> Vec<f64> {
    let cm = prog.model(model);
    let r = run(cm, Config::until(t.max(1e-9)));
    assert!(matches!(r.status, RunStatus::Completed), "{:?}", r.status);
    // A member's bindings are named by their path, `ball.pos`.
    let name = binding.split_once('.').map(|x| x.1).unwrap_or(binding);
    let i = cm.bindings.iter().position(|b| b.name == name).unwrap_or_else(|| cm.idx(binding));
    let v = &r.state_at(t)[i];
    let mut out = vec![0.0; v.width()];
    v.write_to(&mut out);
    out
}

/// One object type, written outside the models, used in a plane and in a room: its points
/// and vectors are of each model's space.
const SHARED: &str = r#"space Plane = euclidean(2)
space Room = euclidean(3)

/// A point mass moved by the force on it.
object Body {
  param { m: Mass = 1 kg }
  input { force: Vector<Force> = 0 }
  state {
    pos: Point            = origin
    vel: Vector<Velocity> = 0
  }
  derived { energy: Energy = kinetic(m, |vel|) }
  flow {
    der(pos) = vel
    der(vel) = force / m
  }
}

fn kinetic(m: Mass, v: Velocity): Energy = m * v^2 / 2

enum Heading { up, down }

model Throw in Plane {
  parts { ball: Body { m = 2 kg; vel = (3 m/s, 4 m/s); force = (0 N, -2 kg * 9.81 m/s^2) } }
  derived { heading: Heading = if ball.vel.y > 0 m/s then up else down }
}

model Drop in Room {
  parts { ball: Body { force = (0 N, 0 N, -9.81 N) } }
}
"#;

#[test]
fn a_shared_object_type_lives_in_each_models_space() {
    let prog = program_with(SHARED, &prismal_syntax::NoModules);
    let p = at(&prog, "Throw", "Throw.ball.pos", 0.5);
    assert!((p[0] - 1.5).abs() < 1e-9 && (p[1] - (2.0 - 0.5 * 9.81 * 0.25)).abs() < 1e-9, "{p:?}");
    let q = at(&prog, "Drop", "Drop.ball.pos", 1.0);
    assert_eq!(q.len(), 3);
    assert!(q[0].abs() < 1e-12 && q[1].abs() < 1e-12 && (q[2] + 0.5 * 9.81).abs() < 1e-9, "{q:?}");
    // The shared function computes the energy, `m v² / 2` with `v = 5 m/s` at the start.
    let e = at(&prog, "Throw", "Throw.ball.energy", 0.0);
    assert!((e[0] - 25.0).abs() < 1e-9, "{e:?}");
}

#[test]
fn a_shared_enumeration() {
    let prog = program_with(SHARED, &prismal_syntax::NoModules);
    let cm = prog.model("Throw");
    let r = run(cm, Config::until(1.0));
    let up = r.state_at(0.1)[cm.idx("Throw.heading")].clone();
    let down = r.state_at(0.9)[cm.idx("Throw.heading")].clone();
    assert_ne!(format!("{up:?}"), format!("{down:?}"));
}

#[test]
fn canonical_text_keeps_shared_definitions_outside_models() {
    let text = format(&compile(SHARED).unwrap().doc);
    assert!(text.contains("\n/// A point mass moved by the force on it.\nobject Body {\n  param {\n    m: Mass = 1 kg\n  }"), "{text}");
    assert!(text.contains("\nfn kinetic(m: Mass, v: Velocity): Energy = m * v^2 / 2"), "{text}");
    assert!(text.contains("\nenum Heading { up, down }"), "{text}");
    assert!(text.contains("ball: Body {"), "{text}");
    assert!(!text.contains("model shared"), "{text}");
    assert_eq!(format(&compile(&text).unwrap().doc), text);
}

fn mechanics() -> MapModules {
    let mut m = MapModules::default();
    m.0.insert(
        "mechanics".into(),
        r#"/// A point mass moved by the force on it.
object Body {
  param { m: Mass = 1 kg }
  input { force: Vector<Force> = 0 }
  state {
    pos: Point            = origin
    vel: Vector<Velocity> = 0
  }
  flow {
    der(pos) = vel
    der(vel) = force / m
  }
}

fn kinetic(m: Mass, v: Velocity): Energy = m * v^2 / 2
fn momentum(m: Mass, v: Velocity): Quantity<M L/T> = m * v
"#
        .into(),
    );
    m.0.insert("units.extra".into(), "fn kinetic(m: Mass, v: Velocity): Energy = 0.5 * m * v * v\n".into());
    m.0.insert("loop.a".into(), "use loop.b\nfn f(x: Real): Real = x\n".into());
    m.0.insert("loop.b".into(), "use loop.a\nfn g(x: Real): Real = x\n".into());
    m.0.insert("broken".into(), "fn f(x: Real): Real = x +\n".into());
    m.0.insert("layered".into(), "use mechanics.{Body}\nobject Rocket {\n  parts { hull: Body { m = 10 kg } }\n}\n".into());
    m
}

const USES: &str = r#"use mechanics

space Plane = euclidean(2)

model Throw in Plane {
  parts { ball: Body { m = 2 kg; vel = (3 m/s, 4 m/s); force = (0 N, -2 kg * 9.81 m/s^2) } }
  derived { p: Quantity<M L/T> = momentum(2 kg, 5 m/s) }
}
"#;

#[test]
fn a_module_used_by_name() {
    let prog = program_with(USES, &mechanics());
    let p = at(&prog, "Throw", "Throw.ball.pos", 0.5);
    assert!((p[0] - 1.5).abs() < 1e-9, "{p:?}");
    assert!((at(&prog, "Throw", "Throw.p", 0.0)[0] - 10.0).abs() < 1e-12);
}

/// The document holds what it uses: its IR, written and read back, runs without the modules.
#[test]
fn a_document_is_complete_without_its_modules() {
    let doc = compile_with(USES, &mechanics()).unwrap().doc;
    let json = serde_json::to_string(&doc).unwrap();
    let back: prismal_ir::Document = serde_json::from_str(&json).unwrap();
    let prog = Program::new(back).unwrap();
    assert!((at(&prog, "Throw", "Throw.p", 0.0)[0] - 10.0).abs() < 1e-12);
    // The canonical text names the module and prints none of its definitions.
    let text = format(&doc);
    assert!(text.starts_with("use mechanics\n"), "{text}");
    assert!(!text.contains("object Body"), "{text}");
    assert_eq!(format(&compile_with(&text, &mechanics()).unwrap().doc), text);
}

/// A module may use another: the document holds both.
#[test]
fn a_module_that_uses_a_module() {
    let src = "use layered\nspace Plane = euclidean(2)\nmodel Launch in Plane {\n  parts { r: Rocket }\n}\n";
    let doc = compile_with(src, &mechanics()).unwrap().doc;
    assert!(doc.libraries.iter().any(|l| l.name == "mechanics") && doc.libraries.iter().any(|l| l.name == "layered"));
    let prog = Program::new(doc).unwrap();
    assert!(prog.model("Launch").bindings.iter().any(|b| b.name == "r.hull.m"), "{:?}", prog.model("Launch").bindings.iter().map(|b| b.name.clone()).collect::<Vec<_>>());
}

fn codes(src: &str) -> Vec<String> {
    match compile_with(src, &mechanics()) {
        Err(ds) => ds.iter().map(|d| format!("{}: {}", d.code, d.message)).collect(),
        Ok(c) => match Program::new(c.doc) {
            Ok(_) => vec![],
            Err(ds) => ds.iter().map(|d| d.to_string()).collect(),
        },
    }
}

#[test]
fn uses_are_checked() {
    let d = codes("use nowhere\nmodel M { }");
    assert!(d.iter().any(|m| m.contains("SX-E03") && m.contains("no module `nowhere`")), "{d:?}");
    let d = codes("use mechanics.{Body, Spring}\nmodel M { }");
    assert!(d.iter().any(|m| m.contains("SX-E03") && m.contains("defines no `Spring`")), "{d:?}");
    // Two modules that give one name: import it from one.
    let d = codes("use mechanics\nuse units.extra\nmodel M { }");
    assert!(d.iter().any(|m| m.contains("SX-E09") && m.contains("`kinetic`")), "{d:?}");
    let d = codes("use mechanics.{Body, momentum}\nuse units.extra\nmodel M { derived { e: Energy = kinetic(1 kg, 1 m/s) } }");
    assert!(d.is_empty(), "{d:?}");
    // A file's own definition and a module's of one name.
    let d = codes("use mechanics\nfn kinetic(m: Mass, v: Velocity): Energy = m * v^2\nmodel M { }");
    assert!(d.iter().any(|m| m.contains("SX-E09") && m.contains("module `mechanics`")), "{d:?}");
    // A model's own definition wins inside it.
    let d = codes("use mechanics\nmodel M {\n  fn kinetic(m: Mass, v: Velocity): Energy = m * v^2\n  derived { e: Energy = kinetic(1 kg, 1 m/s) }\n}");
    assert!(d.is_empty(), "{d:?}");
    let d = codes("use loop.a\nmodel M { }");
    assert!(d.iter().any(|m| m.contains("SX-E08") && m.contains("loop.a -> loop.b -> loop.a")), "{d:?}");
    let d = codes("use broken\nmodel M { }");
    assert!(d.iter().any(|m| m.contains("SX-E02") && m.contains("in module `broken`, line")), "{d:?}");
    let d = codes("use mechanics\nuse mechanics\nmodel M { }");
    assert!(d.iter().any(|m| m.contains("SX-E09") && m.contains("used twice")), "{d:?}");
}

/// The first-party library is found by `compile` itself: `use mechanics` works in every host.
#[test]
fn the_first_party_mechanics_library() {
    let src = "use mechanics

space Plane = euclidean(2)

model Swing in Plane {
  param {
    L: Length       = 2 m
    g: Acceleration = 9.81 m/s^2
  }
  parts { ball: Body { m = 0.5 kg; vel = (2 m/s, 0 m/s); force = (0 N, -0.5 kg * g) } }
  derived {
    period: Time = pendulum_period(L, g)
    pull: Force  = spring_force(10 N/m, 0.2 m)
  }
}
";
    let prog = program_with(src, &prismal_syntax::FirstParty);
    assert!((at(&prog, "Swing", "Swing.period", 0.0)[0] - 2.0 * std::f64::consts::PI * (2.0f64 / 9.81).sqrt()).abs() < 1e-12);
    assert!((at(&prog, "Swing", "Swing.pull", 0.0)[0] + 2.0).abs() < 1e-12);
    assert!((at(&prog, "Swing", "Swing.ball.kinetic", 0.0)[0] - 1.0).abs() < 1e-12);
    let p = at(&prog, "Swing", "Swing.ball.momentum", 0.0);
    assert!((p[0] - 1.0).abs() < 1e-12 && p[1].abs() < 1e-12, "{p:?}");
    // `compile` finds it too.
    assert!(compile(src).is_ok());
}
