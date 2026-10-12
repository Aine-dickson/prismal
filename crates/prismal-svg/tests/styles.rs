//! Author styles (D-061): `color:` and `line:` on representations reach frames by name and
//! are drawn with the theme's values; mistakes are static errors; printing keeps them.

use prismal_host::{Content, Document, Instance};
use prismal_svg::{render, Options, Theme, COLORS};
use serde_json::Value as Json;

const SRC: &str = "
space Plane = euclidean(2)
model M in Plane {
  state { pos: Point = origin + (1 m, 0 m) }
  flow { der(pos) = (0 m/s, 1 m/s) }
}
presentation Lab for M {
  view scene: spatial(Plane, scale: 1 m -> 50 px, y: up) {
    marker(pos, color: red) as ball
    segment(origin, pos, color: blue, line: dashed) as rod
    trace(pos every 0.1 s, line: dotted, color: green) as path
    circle(origin, 1 m, color: purple) as ring
    marker(origin) as plain
  }
}
";

fn frame(src: &str) -> (Json, Json) {
    let doc = Document::load(Content::Text(src.into())).unwrap_or_else(|d| panic!("{d:?}"));
    let mut inst = Instance::new(doc.program().unwrap_or_else(|| panic!("{:?}", doc.diagnostics())));
    let layout = inst.open("Lab", false).unwrap_or_else(|d| panic!("{d}"));
    inst.seek(0.5);
    (layout, inst.frame(0.0, 0.0))
}

fn rep<'a>(f: &'a Json, suffix: &str) -> &'a Json {
    f["views"][0]["reps"].as_array().unwrap().iter().find(|r| r["id"].as_str().unwrap().ends_with(suffix)).unwrap()
}

#[test]
fn styles_reach_frames_and_drawings() {
    let (layout, f) = frame(SRC);
    assert_eq!(rep(&f, "ball")["color"], "red");
    assert!(rep(&f, "ball").get("line").is_none());
    assert_eq!((rep(&f, "rod")["color"].as_str(), rep(&f, "rod")["line"].as_str()), (Some("blue"), Some("dashed")));
    assert_eq!(rep(&f, "path")["line"], "dotted");
    assert!(rep(&f, "plain").get("color").is_none(), "no style, the kind's own");
    for (th, dark) in [(Theme::LIGHT, false), (Theme::DARK, true)] {
        let svg = render(&layout, &f, &Options { theme: th.clone(), ..Options::default() });
        for c in ["red", "blue", "green", "purple"] {
            let v = th.color(c).unwrap();
            assert!(svg.contains(v), "{c} {v} dark {dark}");
        }
        assert!(svg.contains("stroke-dasharray=\"6 4\""), "dashed rod");
        assert!(svg.contains("stroke-linecap=\"round\""), "dotted trace");
    }
    assert_eq!(COLORS.len(), Theme::LIGHT.palette.len());
}

#[test]
fn mistakes() {
    let code = |from: &str, to: &str| {
        let bad = SRC.replace(from, to);
        assert_ne!(bad, SRC);
        match Document::load(Content::Text(bad)) {
            Err(e) => format!("{e:?}"),
            Ok(d) => format!("{:?}", d.diagnostics()),
        }
    };
    let e = code("color: red", "color: mauve");
    assert!(e.contains("PK-E05") && e.contains("one of red, orange"), "{e}");
    let e = code("color: red", "line: dashed");
    assert!(e.contains("SX-E08") && e.contains("`marker` takes no option `line`"), "{e}");
    let e = code("line: dashed", "line: wavy");
    assert!(e.contains("PK-E05") && e.contains("solid, dashed, dotted"), "{e}");
}

#[test]
fn printing_keeps_styles() {
    let doc = prismal_syntax::compile(SRC).unwrap().doc;
    let printed = prismal_syntax::format::format(&doc);
    assert!(printed.contains("marker(pos, color: red) as ball"), "{printed}");
    assert!(printed.contains("segment(origin, pos, color: blue, line: dashed) as rod"), "{printed}");
    assert_eq!(prismal_syntax::compile(&printed).unwrap().doc, doc);
}
