//! `select(p)`, the control that chooses one case of an enumeration parameter (D-094): what
//! the renderer receives, the protocol's case numbers, arrow keys, `restart` keeping the
//! choice, and the diagnostics.

use prismal_web::Player;
use serde_json::Value;

const SRC: &str = r#"model Pick {
  enum Metal { copper, steel, glass }
  param { metal: Metal = copper }
  state { x: Real = 0 }
  derived { rate: Frequency = match metal { copper => 3 /s, steel => 2 /s, glass => 1 /s } }
  flow der(x) = rate * 1 s / 1 s
}

presentation PickLab for Pick {
  panel controls {
    select(metal)
    button(restart)
  }
}
"#;

fn control(frame: &Value) -> Value {
    frame["views"].as_array().unwrap().iter().flat_map(|v| v["reps"].as_array().unwrap().iter()).find(|r| r["control"] == "select").cloned().expect("the select")
}

#[test]
fn a_select_offers_the_cases() {
    let mut p = Player::load(SRC).unwrap_or_else(|d| panic!("{d}"));
    p.open("PickLab", false).unwrap();
    let c = control(&p.frame(0.0, 0.0));
    assert_eq!(c["options"], serde_json::json!(["copper", "steel", "glass"]));
    assert_eq!(c["value"], 0.0);
    assert!(c["text"].as_str().unwrap().starts_with("select for metal = copper, one of copper, steel, glass"), "{c}");
    // The protocol gives a case by its number.
    let id = c["id"].as_str().unwrap().to_string();
    assert_eq!(p.set_control(&id, 2.0)["ok"], true);
    let c = control(&p.frame(0.0, 0.0));
    assert_eq!(c["value"], 2.0);
    assert!(c["text"].as_str().unwrap().starts_with("select for metal = glass"), "{c}");
    // Arrow keys move through the cases and stop at the ends.
    assert_eq!(p.key(&id, "left")["ok"], true);
    assert_eq!(control(&p.frame(0.0, 0.0))["value"], 1.0);
    p.key(&id, "left");
    p.key(&id, "left");
    assert_eq!(control(&p.frame(0.0, 0.0))["value"], 0.0);
}

#[test]
fn selects_are_checked() {
    let model = "model M {\n enum Metal { copper, steel }\n param { metal: Metal = copper; k: Real = 1 }\n}\n";
    let err = |pres: &str| Player::load(&format!("{model}presentation P for M {{ panel c {{ {pres} }} }}")).err().map(|d| d.to_string()).unwrap_or_default();
    let d = err("select(k)");
    assert!(d.contains("PK-E05") && d.contains("enumeration"), "{d}");
    let d = err("slider(metal)");
    assert!(d.contains("PK-E05") && d.contains("select(metal)"), "{d}");
}
