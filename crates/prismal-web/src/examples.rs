//! The reference programs (docs/spec/reference-programs), embedded in the player as example
//! sources. Each is assembled from its documents the way the acceptance tests read them, so
//! the player always shows the programs as specified.

use prismal_syntax::markdown_blocks;

const WORKING_SYNTAX: &str = include_str!("../../../docs/syntax-study/working-syntax.md");
const RP01: &str = include_str!("../../../docs/spec/reference-programs/RP-01-projectile.md");
const RP02: &str = include_str!("../../../docs/spec/reference-programs/RP-02-projectile-drag.md");
const RP03: &str = include_str!("../../../docs/spec/reference-programs/RP-03-bouncing-ball.md");
const RP04: &str = include_str!("../../../docs/spec/reference-programs/RP-04-pendulum.md");
const RP05: &str = include_str!("../../../docs/spec/reference-programs/RP-05-spring-mass.md");
const RP06: &str = include_str!("../../../docs/spec/reference-programs/RP-06-function-plot.md");
const RP07: &str = include_str!("../../../docs/spec/reference-programs/RP-07-vector-addition.md");
const RP08: &str = include_str!("../../../docs/spec/reference-programs/RP-08-narrated-lesson.md");

/// The chapters of the guide that hold programs.
const GUIDE: &[(&str, &str)] = &[
    ("1", include_str!("../../../docs/guide/01-first-program.md")),
    ("2", include_str!("../../../docs/guide/02-quantities-and-units.md")),
    ("3", include_str!("../../../docs/guide/03-space-and-vectors.md")),
    ("4", include_str!("../../../docs/guide/04-motion.md")),
    ("5", include_str!("../../../docs/guide/05-events-and-modes.md")),
    ("6", include_str!("../../../docs/guide/06-checks-and-tests.md")),
    ("7", include_str!("../../../docs/guide/07-presentations.md")),
    ("8", include_str!("../../../docs/guide/08-lessons.md")),
    ("9", include_str!("../../../docs/guide/09-objects.md")),
    ("15", include_str!("../../../docs/guide/15-chance.md")),
    ("16", include_str!("../../../docs/guide/16-fields.md")),
    ("17", include_str!("../../../docs/guide/17-solving-equations.md")),
    ("18", include_str!("../../../docs/guide/18-constraints.md")),
    ("19", include_str!("../../../docs/guide/19-modules.md")),
    ("20", include_str!("../../../docs/guide/20-three-dimensions.md")),
];

/// An example program: a title and its source in the working syntax.
pub struct Example {
    pub key: String,
    pub title: String,
    pub source: String,
}

fn blocks(md: &str) -> Vec<String> {
    markdown_blocks(md).into_iter().map(|b| b.1).collect()
}

/// The `text` and `cases` blocks of a working-syntax section (`RP-01` ...).
fn section(name: &str) -> Vec<String> {
    WORKING_SYNTAX.split("\n## ").find(|p| p.starts_with(name)).map(blocks).unwrap_or_default()
}

/// The cases (`run` blocks) of a working-syntax section.
fn cases(name: &str) -> Vec<String> {
    section(name).into_iter().filter(|b| b.trim_start().starts_with("run ")).collect()
}

fn join(parts: &[String]) -> String {
    parts.iter().map(|p| p.trim_end()).collect::<Vec<_>>().join("\n\n") + "\n"
}

/// RP-08: the RP-01 model with RP-08's addition inside it, RP-08's presentation, and the
/// cases of RP-08's working-syntax section.
fn rp08() -> String {
    let rp01 = blocks(RP01);
    let rp08 = blocks(RP08);
    let model = &rp01[0];
    let end = model.rfind("\n}").expect("end of the model");
    let mut parts = vec![format!("{}\n{}{}", &model[..end], rp08[0].trim_end_matches('\n'), &model[end..])];
    parts.extend(rp08[1..].iter().cloned());
    parts.extend(cases("RP-08"));
    join(&parts)
}

pub fn all() -> Vec<Example> {
    // A program from its documents, with the cases of its working-syntax sections.
    let prog = |docs: &[&str], sections: &[&str]| {
        let mut parts: Vec<String> = docs.iter().flat_map(|d| blocks(d)).collect();
        parts.extend(sections.iter().flat_map(|s| cases(s)));
        join(&parts)
    };
    let ex = |key: &str, title: &str, source: String| Example { key: key.into(), title: title.into(), source };
    let mut out = vec![
        ex("rp08", "RP-08 Narrated projectile lesson", rp08()),
        ex("rp01", "RP-01 Projectile, no drag", prog(&[RP01], &["RP-01"])),
        ex("rp02", "RP-02 Projectile with drag", prog(&[RP01, RP02], &["RP-01", "RP-02"])),
        ex("rp03", "RP-03 Bouncing ball", prog(&[RP03], &["RP-03"])),
        ex("rp04", "RP-04 Pendulum", prog(&[RP04], &["RP-04"])),
        ex("rp05", "RP-05 Spring-mass", prog(&[RP05], &["RP-05"])),
        ex("rp06", "RP-06 Function plot with draggable parameter", join(&blocks(RP06))),
        ex("rp07", "RP-07 Vector addition", join(&blocks(RP07))),
    ];
    // The guide's programs, as `g4-oscillator`, "Guide 4: Oscillator".
    for (chapter, md) in GUIDE {
        for p in split_guide(md).0 {
            let key = format!("g{chapter}-{}", p.title.to_lowercase());
            if out.iter().any(|e| e.key == key) {
                continue;
            }
            out.push(Example { key, title: format!("Guide {chapter}: {}", p.title), source: p.source });
        }
    }
    out
}

/// A program of the guide (`docs/guide`): its blocks, and the model it declares.
pub struct GuideProgram {
    pub title: String,
    pub source: String,
}

/// An example of a mistake in the guide: a program that must fail with `code`.
pub struct GuideError {
    pub code: String,
    pub source: String,
}

/// Splits a guide chapter into its programs and its error examples.
///
/// `text` and `cases` blocks form programs: a `text` block whose first line of code starts
/// with `space` or `model` begins a new program once the current one has a model, and the
/// blocks after it belong to it.
/// An `error` block is a whole program that starts with `// error: CODE`.
pub fn split_guide(md: &str) -> (Vec<GuideProgram>, Vec<GuideError>) {
    let mut programs: Vec<GuideProgram> = vec![];
    let mut errors = vec![];
    let mut block: Option<(String, String)> = None;
    for line in md.lines() {
        let fence = line.trim_start().starts_with("```");
        match (&mut block, fence) {
            (None, true) => block = Some((line.trim_start().trim_start_matches('`').trim().to_string(), String::new())),
            (Some((_, text)), false) => {
                text.push_str(line);
                text.push('\n');
            }
            (Some(_), true) => {
                let (lang, text) = block.take().unwrap();
                match lang.as_str() {
                    "text" | "cases" => {
                        let first = text.lines().map(str::trim).find(|l| !l.is_empty() && !l.starts_with("//")).unwrap_or("");
                        let begins = lang == "text" && (first.starts_with("model ") || first.starts_with("space ") || first.starts_with("use "));
                        // A space declared on its own belongs with the model that follows it.
                        let current_has_model = programs.last().is_some_and(|p| !p.title.is_empty());
                        if programs.is_empty() || (begins && current_has_model) {
                            programs.push(GuideProgram { title: String::new(), source: String::new() });
                        }
                        let p = programs.last_mut().unwrap();
                        if p.title.is_empty() {
                            if let Some(name) = text.lines().find_map(|l| l.trim().strip_prefix("model ")) {
                                p.title = name.split(|c: char| !c.is_alphanumeric() && c != '_').next().unwrap_or("").to_string();
                            }
                        }
                        p.source.push_str(&text);
                        p.source.push('\n');
                    }
                    "error" => {
                        let code = text.lines().next().and_then(|l| l.trim().strip_prefix("// error:")).map(|c| c.trim().to_string()).unwrap_or_default();
                        errors.push(GuideError { code, source: text });
                    }
                    _ => {}
                }
            }
            (None, false) => {}
        }
    }
    (programs, errors)
}
