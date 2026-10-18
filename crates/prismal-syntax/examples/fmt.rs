//! Prints a program in the canonical form (working syntax section 1.4).
//!
//! `cargo run -p prismal-syntax --example fmt -- FILE [--previous IR.json] [--ir OUT.json]`
//!
//! With `--previous`, identities and order are matched to that IR (D-036): the identities
//! that disappeared and appeared are reported on standard error. With `--ir`, the matched IR
//! is written to OUT.json, to be the previous IR of the next edit.

use prismal_ir::Document;
use prismal_syntax::{compile_with, diff, format, markdown_source, reconcile, DirModules};

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    let opt = |name: &str| args.iter().position(|a| a == name).and_then(|i| args.get(i + 1)).cloned();
    let Some(path) = args.iter().enumerate().find(|(i, a)| !a.starts_with("--") && (*i == 0 || !args[i - 1].starts_with("--"))).map(|x| x.1.clone()) else {
        eprintln!("usage: fmt FILE [--previous IR.json] [--ir OUT.json]");
        std::process::exit(2);
    };
    let read = |p: &str| {
        std::fs::read_to_string(p).unwrap_or_else(|e| {
            eprintln!("{p}: {e}");
            std::process::exit(2);
        })
    };
    let text = read(&path);
    let src = if path.ends_with(".md") { markdown_source(&text) } else { text };
    // `use` finds modules beside the file, then the first-party ones (D-099).
    let dir = std::path::Path::new(&path).parent().map(|p| p.to_path_buf()).unwrap_or_default();
    let mut doc = match compile_with(&src, &DirModules { roots: vec![dir] }) {
        Ok(c) => c.doc,
        Err(ds) => {
            for d in &ds {
                eprintln!("{}\n", d.render(&src, &path));
            }
            std::process::exit(1);
        }
    };
    if let Some(prev_path) = opt("--previous") {
        let prev = Document::from_json(&read(&prev_path)).unwrap_or_else(|e| {
            eprintln!("{prev_path}: {e}");
            std::process::exit(2);
        });
        doc = reconcile(doc, &prev);
        let d = diff(&prev, &doc);
        for id in &d.removed {
            eprintln!("removed: {id} (references to it from elsewhere are broken)");
        }
        for id in &d.added {
            eprintln!("added: {id}");
        }
    }
    if let Some(out) = opt("--ir") {
        std::fs::write(&out, doc.to_json()).unwrap_or_else(|e| {
            eprintln!("{out}: {e}");
            std::process::exit(2);
        });
    }
    print!("{}", format(&doc));
}
