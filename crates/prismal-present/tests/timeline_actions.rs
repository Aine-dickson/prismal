//! Continue points (`wait learner`, D-067), animated properties (`animate`), and the handover
//! of a representation between its projection and an animation (`release`, `bind`, D-068).

mod common;
use common::*;
use prismal_ir::present::LearnerInput;
use prismal_present::frame::{Frame, Shape};
use prismal_present::timeline::{ease, play, Input, Medium};
use prismal_present::Program;
use prismal_runtime::Config;

const SRC: &str = "
space Plane = euclidean(2)
model FreeFall in Plane {
  param {
    g: Acceleration = 9.81 m/s^2
    h: Length       = 10 m
  }
  state {
    pos: Point            = origin + (0 m, h)
    vel: Vector<Velocity> = 0
  }
  discrete { airborne: Boolean = true }
  flow {
    der(pos) = if airborne then vel else 0
    der(vel) = if airborne then (0, -g) else 0
  }
  event landed on falling(pos.y) {
    set airborne = false
    set vel = 0
  }
}
presentation Pause for FreeFall {
  view scene: spatial(Plane, scale: 1 m -> 10 px, y: up) { marker(pos) as ball }
  timeline {
    scene s {
      beat ask   { narrate \"Where will it land?\" for 2 s; wait learner }
      beat fall  { run rate 1 until landed }
      beat timed { hold; wait learner limit 5 s fallback { wait 1 s } }
      beat last  { narrate \"Done.\" for 2 s }
    }
  }
}
presentation Moves for FreeFall {
  view scene: spatial(Plane, scale: 1 m -> 10 px, y: up) { marker(pos) as ball }
  timeline {
    scene s {
      beat dim    { animate ball opacity to 0.5 for 2 s }
      beat shift  { animate ball offset to (2 m, 0 m) for 1 s }
      beat freeze { release ball; run rate 1 until landed }
      beat back   { hold; bind ball for 1 s }
    }
  }
}
";

/// The fall time from 10 m.
fn fall() -> f64 {
    (2.0 * 10.0 / 9.81_f64).sqrt()
}

fn lesson(pres: &str, medium: Medium, inputs: Vec<Input>) -> prismal_present::timeline::Playback {
    play(&program(SRC), pres, Config::until(10.0), medium, inputs).unwrap_or_else(|d| panic!("{d:?}"))
}

fn at(f: &Frame) -> [f64; 2] {
    match f.rep("ball").unwrap().shape {
        Shape::Point { at } => at,
        ref s => panic!("{s:?}"),
    }
}

fn close(a: f64, b: f64, what: &str) {
    assert!((a - b).abs() < 1e-9, "{what}: {a} vs {b}");
}

fn cont(at: f64) -> Input {
    Input { at, input: LearnerInput::Continue }
}

#[test]
fn a_continue_point_waits_for_the_learner() {
    let pb = lesson("Pause", Medium::Interactive, vec![cont(5.0)]);
    // The continue point opens after the narration, at 2 s, and ends when the learner
    // continues.
    close(pb.beat("ask").end, 5.0, "ask");
    close(pb.beat("fall").end, 5.0 + fall(), "fall");
    // A limit ends a continue point the learner does not end.
    close(pb.beat("timed").end, 10.0 + fall(), "timed");
    assert_eq!(pb.waits.len(), 2);
    assert!(pb.waits.iter().all(|w| !w.pending && !w.explore));
    close(pb.waits[0].at, 2.0, "opens after the narration");
    assert!(pb.refusals.is_empty(), "{:?}", pb.refusals);
}

#[test]
fn without_the_learner_a_continue_point_is_pending() {
    let pb = lesson("Pause", Medium::Interactive, vec![]);
    close(pb.beat("ask").end, 2.0, "ask");
    assert!(pb.waits[0].pending, "a player stops here");
    assert!(!pb.waits[1].pending, "a limit is not pending");
    // A continue before the point opens is not taken by it.
    let pb = lesson("Pause", Medium::Interactive, vec![cont(1.0)]);
    close(pb.beat("ask").end, 2.0, "ask");
    assert_eq!(pb.refusals.len(), 1);
    assert_eq!(pb.refusals[0].reason, "no continue point at this instant");
}

#[test]
fn linear_media_play_the_fallback() {
    let pb = lesson("Pause", Medium::Video, vec![]);
    close(pb.beat("ask").end, 2.0, "omitted");
    close(pb.beat("timed").end, 3.0 + fall(), "the fallback's 1 s");
    assert!(pb.waits.is_empty() && pb.unsupported.is_empty());
}

#[test]
fn animations_drive_opacity_and_offset() {
    let pb = lesson("Moves", Medium::Interactive, vec![]);
    close(pb.beat("dim").end, 2.0, "dim");
    close(pb.beat("shift").end, 3.0, "shift");
    let o = pb.frame(1.0, 0.1).rep("ball").unwrap().opacity.unwrap();
    close(o, 1.0 - 0.5 * ease(0.5), "opacity on the way");
    // The last value holds after the animation.
    close(pb.frame(2.5, 0.1).rep("ball").unwrap().opacity.unwrap(), 0.5, "held");
    // 10 m up is -100 px in a view with y up; the offset moves 20 px right.
    let p = at(&pb.frame(2.5, 0.1));
    close(p[0], 20.0 * ease(0.5), "offset on the way");
    close(p[1], -100.0, "y");
}

#[test]
fn a_released_representation_holds_until_bound() {
    let pb = lesson("Moves", Medium::Interactive, vec![]);
    let landed = 3.0 + fall();
    close(pb.beat("freeze").end, landed, "freeze");
    // The ball falls in the model; the released marker stays where it was.
    let mid = at(&pb.frame(3.0 + fall() / 2.0, 0.1));
    close(mid[0], 20.0, "offset still applies");
    close(mid[1], -100.0, "held");
    // `bind` hands it back over 1 s: half way at the midpoint of the easing.
    let p = at(&pb.frame(landed + 0.5, 0.1));
    // The landed ball rests within the event location's tolerance of the ground.
    assert!((p[1] + 100.0 * (1.0 - ease(0.5))).abs() < 1e-6, "blending: {p:?}");
    let p = at(&pb.frame(landed + 1.0, 0.1));
    assert!(p[1].abs() < 1e-6, "bound: {p:?}");
    close(p[0], 20.0, "offset");
    assert!(pb.diagnostics.is_empty(), "{:?}", pb.diagnostics);
}

#[test]
fn formatting_and_diagnostics() {
    let doc = prismal_syntax::compile(SRC).unwrap().doc;
    let printed = prismal_syntax::format::format(&doc);
    for line in ["wait learner limit 5 s fallback { wait 1 s }", "animate ball opacity to 0.5 for 2 s", "animate ball offset to (2 m, 0 m) for 1 s", "release ball", "bind ball for 1 s"] {
        assert!(printed.contains(line), "{line} in\n{printed}");
    }
    assert_eq!(prismal_syntax::compile(&printed).unwrap().doc, doc);

    let codes = |edit: &str, with: &str| -> Vec<String> {
        let bad = SRC.replace(edit, with);
        assert_ne!(bad, SRC, "{edit}");
        match prismal_syntax::compile(&bad) {
            Err(ds) => ds.iter().map(|d| d.code.to_string()).collect(),
            Ok(c) => Program::new(c.doc).err().map(|ds| ds.iter().map(|d| d.code.to_string()).collect()).unwrap_or_default(),
        }
    };
    assert_eq!(codes("opacity to 0.5", "opacity to 2"), ["PK-E02"], "an opacity is from 0 to 1");
    assert_eq!(codes("opacity to 0.5", "opacity to 1 m"), ["PK-E02"], "dimensionless");
    assert_eq!(codes("offset to (2 m, 0 m)", "offset to (2 s, 0 s)"), ["PK-E02"], "an offset is a length");
    assert_eq!(codes("animate ball opacity", "animate ball size"), ["SX-E02"], "opacity or offset");
    assert_eq!(codes("release ball;", "release nothing;"), ["SX-E03"], "unknown name");
    assert_eq!(codes("wait learner limit 5 s", "wait learner limit 5 m"), ["PK-E02"], "a limit is a duration");

    // `bind` without a `release` is reported when the lesson plays.
    let src = SRC.replace("release ball; ", "");
    let pb = play(&program(&src), "Moves", Config::until(10.0), Medium::Interactive, vec![]).unwrap();
    assert!(pb.diagnostics.iter().any(|d| d.contains("is not released")), "{:?}", pb.diagnostics);
}

#[test]
fn an_offset_needs_a_spatial_view() {
    let src = SRC.replace("beat dim    { animate ball opacity to 0.5 for 2 s }", "beat dim { show label(h) as tag; animate tag offset to (1 m, 0 m) }");
    let doc = prismal_syntax::compile(&src).unwrap_or_else(|d| panic!("{d:?}")).doc;
    let codes: Vec<&str> = Program::new(doc).err().map(|ds| ds.iter().map(|d| d.code).collect()).unwrap_or_default();
    assert_eq!(codes, ["PK-E05"]);
}

/// `ease e` on `reveal`, `hide ... for`, `camera` and `bind` (D-095), as on `animate`: a
/// quarter of the way through, `linear` has gone a quarter, `in` a sixteenth; without the
/// clause, `smooth` as before.
#[test]
fn easing_of_reveals_hides_cameras_and_binds() {
    let src = SRC.replace(
        "presentation Moves for FreeFall {",
        "presentation Eased for FreeFall {
  view scene: spatial(Plane, scale: 1 m -> 10 px, y: up) { marker(pos) as ball }
  timeline {
    scene s {
      beat show  { reveal fade for 4 s in scene ease linear { marker(origin) as spot } }
      beat look  { camera scene zoom 2 for 4 s ease in }
      beat gone  { hide spot for 4 s ease out }
      beat plain { hide ball for 4 s }
    }
  }
}
presentation Moves for FreeFall {",
    );
    let pb = play(&program(&src), "Eased", Config::until(10.0), Medium::Interactive, vec![]).unwrap_or_else(|d| panic!("{d:?}"));
    close(pb.frame(1.0, 0.1).rep("spot").unwrap().opacity.unwrap(), 0.25, "reveal, linear");
    let zoom = pb.frame(5.0, 0.1).views[0].camera.as_ref().map(|c| c.zoom).unwrap();
    close(zoom, 1.0 + 1.0 / 16.0, "camera, in");
    close(pb.frame(9.0, 0.1).rep("spot").unwrap().opacity.unwrap(), 1.0 - (1.0 - 0.75f64.powi(2)), "hide, out");
    close(pb.frame(13.0, 0.1).rep("ball").unwrap().opacity.unwrap(), 1.0 - ease(0.25), "hide, smooth");
    let printed = prismal_syntax::format::format(&prismal_syntax::compile(&src).unwrap().doc);
    for line in ["reveal fade for 4 s in scene ease linear", "camera scene zoom 2 for 4 s ease in", "hide spot for 4 s ease out", "hide ball for 4 s"] {
        assert!(printed.contains(line), "{line} in\n{printed}");
    }
    assert!(!printed.contains("hide ball for 4 s ease"), "no clause when none was written");
    assert_eq!(prismal_syntax::compile(&printed).unwrap().doc, prismal_syntax::compile(&src).unwrap().doc);
}
