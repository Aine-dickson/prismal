//! The `prismal` command: runs the cases of an example by name, or of a program file.
//!
//! `cargo run -p prismal-web --bin prismal -- list`
//! `cargo run -p prismal-web --bin prismal -- cases NAME|FILE`
//!
//! NAME is an example (`rp01` to `rp08`, `g1-...`, `g9-drops` ...; `list` prints them all).
//! FILE is a program in the working syntax, or a Markdown document whose `text` and `cases`
//! blocks form one program.

use prismal_present::expect::run_case;
use prismal_present::Program;
use prismal_syntax::{compile_with, markdown_source, DirModules};
use prismal_web::examples;

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match args.first().map(String::as_str) {
        Some("list") => {
            for e in examples::all() {
                println!("{:<16} {}", e.key, e.title);
            }
        }
        Some("cases") if args.len() == 2 => std::process::exit(cases(&args[1])),
        _ => {
            eprintln!("usage: prismal list | prismal cases NAME|FILE");
            std::process::exit(2);
        }
    }
}

fn cases(what: &str) -> i32 {
    let src = match examples::all().into_iter().find(|e| e.key == what) {
        Some(e) => e.source,
        None => match std::fs::read_to_string(what) {
            Ok(t) if what.ends_with(".md") => markdown_source(&t),
            Ok(t) => t,
            Err(_) => {
                eprintln!("`{what}` is neither an example (see `prismal list`) nor a file");
                return 2;
            }
        },
    };
    // `use` finds modules beside the file, then the first-party ones (D-099).
    let dir = std::path::Path::new(what).parent().map(|p| p.to_path_buf()).unwrap_or_default();
    let doc = match compile_with(&src, &DirModules { roots: vec![dir] }) {
        Ok(c) => c.doc,
        Err(ds) => {
            for d in &ds {
                eprintln!("{}\n", d.render(&src, what));
            }
            return 1;
        }
    };
    let prog = match Program::new(doc) {
        Ok(p) => p,
        Err(ds) => {
            for d in &ds {
                eprintln!("{d}");
            }
            return 1;
        }
    };
    if prog.doc.runs.is_empty() {
        println!("no cases");
    }
    let mut failed = 0;
    for case in &prog.doc.runs {
        match run_case(&prog, case) {
            Ok(report) => {
                println!("run {}", case.name);
                for r in &report.results {
                    let id = r.id.rsplit('.').next().unwrap_or(&r.id);
                    if r.pass {
                        println!("  expect {id}: pass");
                    } else {
                        failed += 1;
                        println!("  expect {id}: FAIL: {}", r.message);
                    }
                }
            }
            Err(e) => {
                failed += 1;
                println!("run {}: {e}", case.name);
            }
        }
    }
    if failed > 0 {
        1
    } else {
        0
    }
}
