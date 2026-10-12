//! Every complete example (`examples/*.md`) compiles, every case in it passes, and every
//! presentation in it opens.

use prismal_web::examples::split_guide;
use prismal_web::Player;
use std::path::PathBuf;

fn examples() -> Vec<(String, String)> {
    let dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../examples");
    let mut out: Vec<(String, String)> = std::fs::read_dir(&dir)
        .unwrap()
        .filter_map(|e| e.ok())
        .map(|e| e.path())
        .filter(|p| p.extension().is_some_and(|x| x == "md") && p.file_name().is_some_and(|n| n != "README.md"))
        .map(|p| (p.file_name().unwrap().to_string_lossy().into_owned(), std::fs::read_to_string(&p).unwrap()))
        .collect();
    out.sort();
    out
}

#[test]
fn examples_run() {
    let mut failures = vec![];
    let mut expectations = 0;
    for (file, md) in examples() {
        let (progs, _) = split_guide(&md);
        if progs.is_empty() {
            failures.push(format!("{file}: no program"));
        }
        for p in progs {
            let mut player = match Player::load(&p.source) {
                Ok(pl) => pl,
                Err(d) => {
                    failures.push(format!("{file} {}: does not compile: {d}", p.title));
                    continue;
                }
            };
            for case in player.run_cases().as_array().unwrap() {
                if let Some(e) = case.get("error") {
                    failures.push(format!("{file} {}: {e}", case["case"]));
                }
                for r in case["results"].as_array().into_iter().flatten() {
                    expectations += 1;
                    if r["pass"] != true {
                        failures.push(format!("{file} {} {}: {}", case["case"], r["id"], r["message"]));
                    }
                }
            }
            for pres in player.catalogue()["presentations"].as_array().unwrap().clone() {
                if pres["kind"] != "observations" {
                    if let Err(d) = player.open(pres["name"].as_str().unwrap(), false) {
                        failures.push(format!("{file}: presentation {} does not open: {d}", pres["name"]));
                    }
                }
            }
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
    assert!(expectations > 0);
}
