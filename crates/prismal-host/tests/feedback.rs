//! Words at a slider's ends and the percent unit (D-077), and a value told in words by
//! bands (D-076's `when:`).

use prismal_host::{Content, Engine};
use serde_json::Value as Json;

const ROOM: &str = "model Room {
  param {
    temp:  Quantity<Θ> = 293 K  in [273 K, 313 K]
    humid: Real        = 40 %   in [0 %, 100 %]   unit %
  }
}
presentation Lab for Room {
  panel controls {
    slider(temp, range: [273 K, 313 K], ends: (\"cold\", \"hot\")) as dial
    slider(humid, range: [0 %, 100 %]) as wet
    label(humid) as reading
    text(\"Cold\", when: temp < 288 K) as cold
    text(\"Comfortable\", when: temp >= 288 K and temp < 298 K) as fine
    text(\"Hot\", when: temp >= 298 K) as hot
  }
}
";

fn rep(f: &Json, name: &str) -> Json {
    f["views"].as_array().unwrap().iter().flat_map(|v| v["reps"].as_array().unwrap().iter()).find(|r| r["name"] == name).cloned().unwrap_or(Json::Null)
}

fn open() -> (Engine, String) {
    let mut e = Engine::new();
    let doc = e.load(Content::Text(ROOM.into())).unwrap();
    let d = e.document(&doc).unwrap().diagnostics();
    assert!(d.is_empty(), "{d:?}");
    let (h, _) = e.open(&doc, "Lab", false).unwrap();
    (e, h)
}

#[test]
fn a_slider_names_its_ends() {
    let (mut e, h) = open();
    let f = e.instance(&h).unwrap().frame(0.0, 0.0);
    let dial = rep(&f, "dial");
    assert_eq!(dial["ends"], serde_json::json!(["cold", "hot"]));
    assert!(dial["text"].as_str().unwrap().ends_with("from cold to hot"), "{dial}");
}

#[test]
fn a_share_shows_in_percent() {
    let (mut e, h) = open();
    let f = e.instance(&h).unwrap().frame(0.0, 0.0);
    assert_eq!(rep(&f, "reading")["text"], "reading = 40 %");
}

#[test]
fn a_value_is_told_in_words() {
    let (mut e, h) = open();
    let i = e.instance(&h).unwrap();
    let shown = |f: &Json| ["cold", "fine", "hot"].iter().filter(|n| !rep(f, n).is_null()).map(|n| n.to_string()).collect::<Vec<_>>();
    assert_eq!(shown(&i.frame(0.0, 0.0)), ["fine"]);
    assert_eq!(i.set_control("dial", 305.0)["ok"], true);
    assert_eq!(shown(&i.frame(0.0, 0.0)), ["hot"]);
}
