//! The learner turns a view of three dimensions by dragging an empty point of it, where the
//! presentation permits `orbit` (D-104).

use prismal_host::{Content, Engine, PointerEvent};

const ROOM: &str = "space Room = euclidean(3)
model Corner in Room {
  param { A: Point = origin + (1 m, 0 m, 0 m) }
}
presentation Look for Corner {
  view room: spatial(Room, scale: 1 m -> 100 px, azimuth: 30 deg, elevation: 20 deg) {
    axes
    marker(A) as a
  }
  permit learner { orbit }
}
presentation Fixed for Corner {
  view room: spatial(Room, scale: 1 m -> 100 px) { marker(A) as a }
}
";

fn open(pres: &str) -> (Engine, String) {
    let mut e = Engine::new();
    let doc = e.load(Content::Text(ROOM.into())).unwrap();
    let (h, _) = e.open(&doc, pres, false).unwrap();
    (e, h)
}

/// The azimuth and elevation the frame says the view is drawn from, in degrees.
fn look(e: &mut Engine, h: &str) -> [f64; 2] {
    let f = e.instance(h).unwrap().frame(0.0, 0.0);
    let v = &f["views"][0]["look"];
    [v[0].as_f64().unwrap().to_degrees(), v[1].as_f64().unwrap().to_degrees()]
}

fn close(a: [f64; 2], b: [f64; 2]) {
    assert!((a[0] - b[0]).abs() < 1e-9 && (a[1] - b[1]).abs() < 1e-9, "{a:?} vs {b:?}");
}

#[test]
fn dragging_an_empty_point_turns_the_view() {
    let (mut e, h) = open("Look");
    close(look(&mut e, &h), [30.0, 20.0]);
    let i = e.instance(&h).unwrap();
    let ev = |phase, x, y| PointerEvent { phase, view: "room", x, y, size: None, pointer: "mouse", id: None, time: 0.0 };
    assert_eq!(i.pointer(&ev("down", 20.0, 20.0))["action"], "orbit");
    // Half a degree per pixel: 60 px to the right turns the viewer 30 deg the other way, and
    // 40 px down tilts it 20 deg higher.
    i.pointer(&ev("move", 80.0, 60.0));
    assert_eq!(i.pointer(&ev("up", 80.0, 60.0))["action"], "orbit");
    close(look(&mut e, &h), [0.0, 40.0]);
    // Elevation stops at straight above.
    let i = e.instance(&h).unwrap();
    i.pointer(&ev("down", 20.0, 20.0));
    i.pointer(&ev("move", 20.0, 400.0));
    i.pointer(&ev("up", 20.0, 400.0));
    close(look(&mut e, &h), [0.0, 90.0]);
    // Resetting the view returns it to its author's look.
    e.instance(&h).unwrap().view_reset("room");
    close(look(&mut e, &h), [30.0, 20.0]);
}

#[test]
fn a_cancelled_turn_goes_back() {
    let (mut e, h) = open("Look");
    let i = e.instance(&h).unwrap();
    let ev = |phase, x| PointerEvent { phase, view: "room", x, y: 20.0, size: None, pointer: "mouse", id: None, time: 0.0 };
    i.pointer(&ev("down", 20.0));
    i.pointer(&ev("move", 120.0));
    i.pointer(&ev("cancel", 120.0));
    close(look(&mut e, &h), [30.0, 20.0]);
}

#[test]
fn without_permission_the_view_stays() {
    let (mut e, h) = open("Fixed");
    let i = e.instance(&h).unwrap();
    let ev = |phase, x| PointerEvent { phase, view: "room", x, y: 20.0, size: None, pointer: "mouse", id: None, time: 0.0 };
    assert_ne!(i.pointer(&ev("down", 20.0))["action"], "orbit");
    i.pointer(&ev("move", 120.0));
    i.pointer(&ev("up", 120.0));
    close(look(&mut e, &h), [30.0, 20.0]);
}
