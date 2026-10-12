//! Found while writing a complete lesson (D-078): paths break where the value jumps, texts
//! round values, and labels can be chosen or left out. (`on E` reading the event's own state
//! is checked by the lesson cases of `examples/projectile.md`, which throw again at the
//! instant of landing.)

use prismal_host::{Content, Engine};
use serde_json::Value as Json;

const THROW: &str = "space Plane = euclidean(2)
model Throw in Plane {
  state { pos: Point = origin; vel: Vector<Velocity> = (3 m/s, 4 m/s) }
  discrete { flying: Boolean = true; last: Length = 0 m }
  flow {
    der(pos) = if flying then vel else 0
    der(vel) = if flying then (0 m/s^2, -9.81 m/s^2) else 0
  }
  event landed on falling(pos.y) { set flying = false; set vel = 0; set last = pos.x }
  event relaunch on request { set pos = origin; set vel = (3 m/s, 4 m/s); set flying = true }
}
presentation Lab for Throw {
  view field: spatial(Plane, scale: 1 m -> 10 px, y: up) {
    trace(pos every 0.05 s) as path
    marker(pos, label: none) as ball
    marker(origin, label: \"start\") as home
  }
  panel notes { text(\"Landed {last:.1} away; {last:.0} rounded.\") as note }
  observe { landings = pos.x on landed }
}
";

fn rep(f: &Json, name: &str) -> Json {
    f["views"].as_array().unwrap().iter().flat_map(|v| v["reps"].as_array().unwrap().iter()).find(|r| r["name"] == name).cloned().unwrap_or(Json::Null)
}

/// A lab thrown twice: the second throw after the first has landed, at 2.44648 m.
fn thrown_twice() -> (Engine, String) {
    let mut e = Engine::new();
    let doc = e.load(Content::Text(THROW.into())).unwrap();
    let d = e.document(&doc).unwrap().diagnostics();
    assert!(d.is_empty(), "{d:?}");
    let (h, _) = e.open(&doc, "Lab", false).unwrap();
    let i = e.instance(&h).unwrap();
    // The first flight lasts 2 * 4 / 9.81 s = 0.8155 s.
    i.seek(1.0);
    assert_eq!(i.request("relaunch", None)["ok"], true, "thrown again");
    i.seek(1.5);
    (e, h)
}

#[test]
fn a_path_breaks_where_the_value_jumps() {
    let (mut e, h) = thrown_twice();
    let path = rep(&e.instance(&h).unwrap().frame(0.0, 0.0), "path");
    let breaks = path["breaks"].as_array().cloned().unwrap_or_default();
    assert_eq!(breaks.len(), 1, "one new piece after the second throw: {path}");
    let k = breaks[0].as_u64().unwrap() as usize;
    let pts = path["points"].as_array().unwrap();
    // The piece before ends far from the start; the next begins near it.
    assert!(pts[k - 1][0].as_f64().unwrap() > 20.0 && pts[k][0].as_f64().unwrap() < 5.0, "{} then {}", pts[k - 1], pts[k]);
}

#[test]
fn texts_round_and_labels_are_chosen() {
    let (mut e, h) = thrown_twice();
    let f = e.instance(&h).unwrap().frame(0.0, 0.0);
    assert_eq!(rep(&f, "note")["text"], "Landed 2.4 m away; 2 m rounded.");
    assert!(rep(&f, "ball")["label"].is_null(), "no label");
    assert_eq!(rep(&f, "home")["label"], "start");
    assert!(rep(&f, "ball")["text"].as_str().unwrap().starts_with("ball at"), "the text alternative keeps the name");
}
