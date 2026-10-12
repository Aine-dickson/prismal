//! The reference's glossary (guide chapter 10) covers every word of the parser's lists,
//! and its "Words" section lists them exactly.

use prismal_syntax::parser::{CONTEXTUAL, RESERVED};
use std::collections::BTreeSet;

fn reference() -> String {
    let path = concat!(env!("CARGO_MANIFEST_DIR"), "/../../docs/guide/10-reference.md");
    std::fs::read_to_string(path).expect("guide chapter 10")
}

fn section<'a>(text: &'a str, heading: &str) -> &'a str {
    let start = text.find(heading).unwrap_or_else(|| panic!("no section {heading}"));
    let rest = &text[start + heading.len()..];
    let end = rest.find("\n## ").unwrap_or(rest.len());
    &rest[..end]
}

/// Words in backticks in the first column of the glossary table.
fn glossary_words(text: &str) -> BTreeSet<String> {
    let mut words = BTreeSet::new();
    for line in section(text, "## Glossary").lines() {
        let Some(first) = line.strip_prefix('|').and_then(|l| l.split('|').next()) else { continue };
        for (i, part) in first.split('`').enumerate() {
            if i % 2 == 1 {
                words.insert(part.trim().to_string());
            }
        }
    }
    words
}

/// Words in backticks after a bold label in the "Words" section.
fn listed(text: &str, label: &str) -> BTreeSet<String> {
    let words = section(text, "## Words");
    let line = words.lines().find(|l| l.starts_with(label)).unwrap_or_else(|| panic!("no line {label}"));
    let inner = line.split('`').nth(1).expect("backticked list");
    inner.split_whitespace().map(str::to_string).collect()
}

#[test]
fn glossary_covers_every_word() {
    let text = reference();
    let glossary = glossary_words(&text);
    let missing: Vec<_> = RESERVED.iter().chain(CONTEXTUAL).filter(|w| !glossary.contains(**w)).collect();
    assert!(missing.is_empty(), "words missing from the glossary: {missing:?}");
}

#[test]
fn word_lists_match_the_parser() {
    let text = reference();
    let reserved: BTreeSet<String> = RESERVED.iter().map(|w| w.to_string()).collect();
    let contextual: BTreeSet<String> = CONTEXTUAL.iter().map(|w| w.to_string()).collect();
    let r = listed(&text, "**Reserved**");
    let c = listed(&text, "**Contextual**");
    assert_eq!(r, reserved, "reserved words differ: doc-only {:?}, parser-only {:?}", r.difference(&reserved).collect::<Vec<_>>(), reserved.difference(&r).collect::<Vec<_>>());
    assert_eq!(c, contextual, "contextual words differ: doc-only {:?}, parser-only {:?}", c.difference(&contextual).collect::<Vec<_>>(), contextual.difference(&c).collect::<Vec<_>>());
}

/// A word in the wrong block is reported with where it belongs (guide, What goes where).
#[test]
fn misplaced_words_say_where_they_belong() {
    let cases = [
        ("model M {\n  param { k: Real = 1 }\n  slider(k, range: [0, 2])\n}\n", "belongs in a presentation, inside `panel"),
        ("model M {\n  state { x: Real = 1; der(x) = -x }\n}\n", "belongs in the model's `flow { ... }` block"),
        ("model M {\n  state { x: Real = 1 }\n  set x = 1\n}\n", "belongs in an event's braces"),
        ("model M {\n  state { x: Real = 1 }\n}\npresentation P for M {\n  marker(x)\n}\n", "belongs inside `view name: plot(...)"),
    ];
    for (src, hint) in cases {
        let (_, ds) = prismal_syntax::parse(src);
        assert!(ds.iter().any(|d| d.message.contains(hint)), "{src}: {:?}", ds.iter().map(|d| &d.message).collect::<Vec<_>>());
    }
}

/// The formatter writes newer forms back as written: payload buttons (D-072), live drags
/// (D-071), plot options (D-070).
#[test]
fn newer_forms_print_back() {
    let src = "space Plane = euclidean(2)
model M in Plane {
  state { pos: Point = origin intervenable }
  discrete { h: Length = 0 m }
  event push on request(d: Length) { set h = h + d }
}
presentation P for M {
  view s: spatial(Plane, scale: 1 m -> 10 px, y: up) { marker(pos) as puck { on drag live as p { propose pos = p } } }
  view g: plot(x: [0 s, 5 s], y: [0 m, 1 m], follow: y, window: 5 s, y_unit: cm) { series_plot(h every 0.1 s) }
  panel c { button(push(2 m), label: \"two\") }
}
";
    let compiled = prismal_syntax::compile(src).unwrap_or_else(|ds| panic!("{:?}", ds.iter().map(|d| &d.message).collect::<Vec<_>>()));
    let out = prismal_syntax::format::format(&compiled.doc);
    for want in ["button(push(2 m), label: \"two\")", "on drag live as p", "follow: y, window: 5 s, y_unit: cm"] {
        assert!(out.contains(want), "missing `{want}` in:\n{out}");
    }
    // And the printed program compiles to the same document.
    let again = prismal_syntax::compile(&out).unwrap_or_else(|ds| panic!("{:?}\n{out}", ds.iter().map(|d| &d.message).collect::<Vec<_>>()));
    assert_eq!(prismal_syntax::format::format(&again.doc), out);
}

/// `create` through a part path prints back as written (D-074).
#[test]
fn create_through_a_path_prints_back() {
    let src = "space Plane = euclidean(2)
model Feed in Plane {
  object Atom { state { pos: Point = origin } }
  object Cell { parts { atoms: Atom[1, max 4] { pos = origin } } }
  parts { left: Cell }
  event add on request { create left.atoms { pos = origin + (1 m, 0 m) } }
}
";
    let compiled = prismal_syntax::compile(src).unwrap_or_else(|ds| panic!("{:?}", ds.iter().map(|d| &d.message).collect::<Vec<_>>()));
    let out = prismal_syntax::format::format(&compiled.doc);
    assert!(out.contains("create left.atoms"), "{out}");
    let again = prismal_syntax::compile(&out).unwrap_or_else(|ds| panic!("{:?}\n{out}", ds.iter().map(|d| &d.message).collect::<Vec<_>>()));
    assert_eq!(prismal_syntax::format::format(&again.doc), out);
}

/// Animations of color, line, scale and shape, with easing, print back as written (D-075).
#[test]
fn animations_print_back() {
    let src = "space Plane = euclidean(2)
model Figure in Plane { param { a: Real = 1 } }
presentation Shapes for Figure {
  view scene: spatial(Plane, scale: 1 m -> 40 px, y: up) {
    segment(origin, origin + (2 m, 0 m)) as base
    circle(origin, 1 m) as round
    marker(origin) as centre
  }
  timeline {
    scene look {
      beat b { animate base color to red for 1 s; animate base line to dashed; animate centre scale to 2 for 1 s ease out; animate base morph to round ease linear }
    }
  }
}
";
    let compiled = prismal_syntax::compile(src).unwrap_or_else(|ds| panic!("{:?}", ds.iter().map(|d| &d.message).collect::<Vec<_>>()));
    let out = prismal_syntax::format::format(&compiled.doc);
    for want in ["animate base color to red for 1 s", "animate base line to dashed", "animate centre scale to 2 for 1 s ease out", "animate base morph to round ease linear"] {
        assert!(out.contains(want), "missing `{want}` in:\n{out}");
    }
    let again = prismal_syntax::compile(&out).unwrap_or_else(|ds| panic!("{:?}\n{out}", ds.iter().map(|d| &d.message).collect::<Vec<_>>()));
    assert_eq!(prismal_syntax::format::format(&again.doc), out);
}

/// Titles, text with values and conditions print back as written (D-076).
#[test]
fn text_and_titles_print_back() {
    let src = "space Plane = euclidean(2)
model Drop in Plane {
  state { pos: Point = origin }
  discrete { up: Boolean = true }
}
presentation Lab for Drop {
  title \"Dropping a ball\"
  view scene: spatial(Plane, scale: 1 m -> 10 px, y: up) {
    title(\"The fall\")
    text(\"start\", at: origin + (1 m, 0 m))
  }
  panel notes { text(\"Height {pos.y}, {{braces}}.\", when: not up) as h }
}
";
    let compiled = prismal_syntax::compile(src).unwrap_or_else(|ds| panic!("{:?}", ds.iter().map(|d| &d.message).collect::<Vec<_>>()));
    let out = prismal_syntax::format::format(&compiled.doc);
    for want in ["title \"Dropping a ball\"", "title(\"The fall\")", "text(\"start\", at: origin + (1 m, 0 m))", "text(\"Height {pos.y}, {{braces}}.\", when: not up) as h"] {
        assert!(out.contains(want), "missing `{want}` in:\n{out}");
    }
    let again = prismal_syntax::compile(&out).unwrap_or_else(|ds| panic!("{:?}\n{out}", ds.iter().map(|d| &d.message).collect::<Vec<_>>()));
    assert_eq!(prismal_syntax::format::format(&again.doc), out);
}

/// Slider ends and the percent unit print back as written (D-077).
#[test]
fn slider_ends_and_percent_print_back() {
    let src = "model Room {
  param { humid: Real = 40 % in [0 %, 100 %] unit % }
}
presentation Lab for Room {
  panel c { slider(humid, range: [0 %, 100 %], ends: (\"dry\", \"wet\")) }
}
";
    let compiled = prismal_syntax::compile(src).unwrap_or_else(|ds| panic!("{:?}", ds.iter().map(|d| &d.message).collect::<Vec<_>>()));
    let out = prismal_syntax::format::format(&compiled.doc);
    for want in ["40 %", "unit %", "ends: (\"dry\", \"wet\")"] {
        assert!(out.contains(want), "missing `{want}` in:\n{out}");
    }
    let again = prismal_syntax::compile(&out).unwrap_or_else(|ds| panic!("{:?}\n{out}", ds.iter().map(|d| &d.message).collect::<Vec<_>>()));
    assert_eq!(prismal_syntax::format::format(&again.doc), out);
}

/// Rounded values in texts and chosen labels print back as written (D-078).
#[test]
fn rounding_and_labels_print_back() {
    let src = "space Plane = euclidean(2)
model M in Plane { state { pos: Point = origin } }
presentation P for M {
  view s: spatial(Plane, scale: 1 m -> 10 px, y: up) {
    marker(pos, label: none) as ball
    marker(origin, label: \"start\")
  }
  panel n { text(\"At {pos.x:.1}, {{not}} {pos.y}.\") }
}
";
    let compiled = prismal_syntax::compile(src).unwrap_or_else(|ds| panic!("{:?}", ds.iter().map(|d| &d.message).collect::<Vec<_>>()));
    let out = prismal_syntax::format::format(&compiled.doc);
    for want in ["label: none", "label: \"start\"", "text(\"At {pos.x:.1}, {{not}} {pos.y}.\")"] {
        assert!(out.contains(want), "missing `{want}` in:\n{out}");
    }
    let again = prismal_syntax::compile(&out).unwrap_or_else(|ds| panic!("{:?}\n{out}", ds.iter().map(|d| &d.message).collect::<Vec<_>>()));
    assert_eq!(prismal_syntax::format::format(&again.doc), out);
}
