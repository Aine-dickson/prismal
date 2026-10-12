//! Authors' text with values, text shown under a condition, and titles (D-076).

use prismal_host::{Content, Engine};
use serde_json::Value as Json;

const DROP: &str = "space Plane = euclidean(2)
model Drop in Plane {
  param { h: Length = 10 m in [1 m, 50 m] }
  state { pos: Point = origin + (0 m, h); vel: Vector<Velocity> = 0 }
  discrete { airborne: Boolean = true; bounces: Real = 0 }
  flow {
    der(pos) = if airborne then vel else 0
    der(vel) = if airborne then (0 m/s^2, -9.81 m/s^2) else 0
  }
  event landed on falling(pos.y) { set airborne = false; set vel = 0; set bounces = bounces + 1 }
}
presentation Lab for Drop {
  title \"Dropping a ball\"
  view scene: spatial(Plane, scale: 1 m -> 10 px, y: up) {
    title(\"The fall\")
    marker(pos) as ball
    text(\"start\", at: origin + (1 m, h))
  }
  panel notes {
    title(\"What happens\")
    text(\"The ball is {pos.y} above the ground.\") as height
    text(\"Falling...\", when: airborne) as flying
    text(\"Landed after {bounces} fall.\", when: not airborne) as landed
    text(\"Braces stay: {{like this}}.\") as braces
  }
}
";

fn open() -> (Engine, String) {
    let mut e = Engine::new();
    let doc = e.load(Content::Text(DROP.into())).unwrap();
    let d = e.document(&doc).unwrap().diagnostics();
    assert!(d.is_empty(), "{d:?}");
    let (h, _) = e.open(&doc, "Lab", false).unwrap();
    (e, h)
}

fn rep(f: &Json, name: &str) -> Json {
    f["views"].as_array().unwrap().iter().flat_map(|v| v["reps"].as_array().unwrap().iter()).find(|r| r["name"] == name).cloned().unwrap_or(Json::Null)
}

#[test]
fn titles_name_the_presentation_and_its_views() {
    let (mut e, h) = open();
    let l = e.instance(&h).unwrap().layout();
    assert_eq!(l["title"], "Dropping a ball");
    let titles: Vec<&str> = l["views"].as_array().unwrap().iter().map(|v| v["title"].as_str().unwrap_or("")).collect();
    assert_eq!(titles, ["The fall", "What happens"]);
}

#[test]
fn text_shows_values_and_follows_conditions() {
    let (mut e, h) = open();
    let i = e.instance(&h).unwrap();
    let f = i.frame(0.0, 0.0);
    assert_eq!(rep(&f, "height")["text"], "The ball is 10 m above the ground.");
    assert_eq!(rep(&f, "flying")["text"], "Falling...");
    assert!(rep(&f, "landed").is_null(), "not shown while the ball falls");
    assert_eq!(rep(&f, "braces")["text"], "Braces stay: {like this}.");
    i.seek(2.0);
    let f = i.frame(0.0, 0.0);
    assert!(rep(&f, "flying").is_null());
    assert_eq!(rep(&f, "landed")["text"], "Landed after 1 fall.");
}

#[test]
fn text_can_be_placed_at_a_point() {
    let (mut e, h) = open();
    let f = e.instance(&h).unwrap().frame(0.0, 0.0);
    let note = f["views"][0]["reps"].as_array().unwrap().iter().find(|r| r["shape"] == "note").cloned().unwrap();
    assert_eq!(note["value"], "start");
    // 1 m right and 10 m up, at 10 px per metre with y up.
    assert_eq!(note["at"], serde_json::json!([10.0, -100.0]));
    // The view is framed to include the text's anchor, right of the ball (D-080).
    let l = e.instance(&h).unwrap().layout();
    assert_eq!(l["views"][0]["extent"][2], 10.0);
}
