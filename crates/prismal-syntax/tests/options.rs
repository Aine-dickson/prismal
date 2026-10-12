//! Options and clauses (D-081): named options inside parentheses are checked against the
//! kind's list, clauses after a statement's head come in any order, each once, and a line
//! that starts with an operator says how to continue a line.

use prismal_syntax::{compile, format::format, Diag};

const SRC: &str = "space Plane = euclidean(2)
model Drop in Plane {
  param { h: Length = 10 m in [1 m, 20 m] }
  state { y: Length = h; v: Velocity = 0 }
  flow { der(y) = v; der(v) = -9.81 m/s^2 }
  constraint low: y >= -1 m within 0.01 m policy stop
}
presentation Lab for Drop {
  view scene: spatial(Plane, scale: 1 m -> 20 px, y: up) {
    marker(origin + (0 m, y), color: red) as ball
  }
  view graph: plot(x: [0 s, 2 s], y: [0 m, 20 m], follow: y) {
    series_plot(y every 0.01 s, when: y > 0 m)
  }
  panel controls { slider(h, ends: (\"low\", \"high\")) }
  timeline {
    scene s {
      beat b { camera scene to origin zoom 2 for 1 s; animate ball scale to 2 for 1 s ease out }
    }
  }
}
";

fn errors(src: &str) -> Vec<Diag> {
    compile(src).expect_err("must be rejected")
}

fn messages(src: &str) -> Vec<String> {
    errors(src).iter().map(|d| format!("{}: {}", d.code, d.message)).collect()
}

fn variant(from: &str, to: &str) -> String {
    let s = SRC.replace(from, to);
    assert_ne!(s, SRC, "{from}");
    s
}

#[test]
fn the_base_program_compiles() {
    compile(SRC).unwrap_or_else(|ds| panic!("{:?}", ds.iter().map(|d| &d.message).collect::<Vec<_>>()));
}

#[test]
fn unknown_options_are_named_with_a_suggestion() {
    let m = messages(&variant("color: red", "colour: red"));
    assert_eq!(m.len(), 1, "{m:?}");
    assert!(m[0].starts_with("SX-E08") && m[0].contains("`marker` takes no option `colour`; did you mean `color`?"), "{m:?}");
    let m = messages(&variant("follow: y", "folow: y"));
    assert!(m.iter().any(|x| x.contains("`plot` takes no option `folow`; did you mean `follow`?")), "{m:?}");
    let m = messages(&variant("y: up)", "y: up, follow: y)"));
    assert!(m.iter().any(|x| x.contains("`spatial` takes no option `follow`; its options are `scale`, `y`")), "{m:?}");
    let m = messages(&variant("when: y > 0 m", "wen: y > 0 m"));
    assert!(m.iter().any(|x| x.contains("did you mean `when`?")), "{m:?}");
    let m = messages(&variant("color: red)", "color: red, color: blue)"));
    assert!(m.iter().any(|x| x.contains("the option `color` is given twice")), "{m:?}");
}

#[test]
fn clauses_come_in_any_order() {
    let reordered = SRC
        .replace("camera scene to origin zoom 2 for 1 s", "camera scene for 1 s zoom 2 to origin")
        .replace("scale to 2 for 1 s ease out", "scale to 2 ease out for 1 s")
        .replace("within 0.01 m policy stop", "policy stop within 0.01 m")
        .replace("h: Length = 10 m in [1 m, 20 m]", "h: Length = 10 m symbol \"h\" in [1 m, 20 m]");
    let a = compile(SRC).unwrap().doc;
    let b = compile(&reordered).unwrap_or_else(|ds| panic!("{:?}", ds.iter().map(|d| &d.message).collect::<Vec<_>>())).doc;
    // The same IR apart from the symbol, and the formatter prints one order.
    let printed = format(&b);
    assert!(printed.contains("camera scene to origin zoom 2 for 1 s"), "{printed}");
    assert!(printed.contains("animate ball scale to 2 for 1 s ease out"), "{printed}");
    assert!(printed.contains("within 0.01 m policy stop"), "{printed}");
    assert_eq!(a.presentations, b.presentations);
}

#[test]
fn a_clause_is_given_once() {
    let m = messages(&variant("zoom 2 for 1 s", "zoom 2 for 1 s zoom 3"));
    assert!(m.iter().any(|x| x.starts_with("SX-E09") && x.contains("`zoom` is given twice")), "{m:?}");
    let m = messages(&variant("in [1 m, 20 m]", "in [1 m, 20 m] where h > 0 m"));
    assert!(m.iter().any(|x| x.contains("one range")), "{m:?}");
}

#[test]
fn a_line_that_starts_with_an_operator() {
    let m = messages(&variant("der(v) = -9.81 m/s^2", "der(v) = -9.81 m/s^2\n    + 0 m/s^2"));
    assert!(m.iter().any(|x| x.contains("A line cannot start with `+`") && x.contains("end the line before with the operator")), "{m:?}");
    // Ending the line with the operator continues it.
    compile(&variant("der(v) = -9.81 m/s^2", "der(v) = -9.81 m/s^2 +\n    0 m/s^2")).unwrap();
}
