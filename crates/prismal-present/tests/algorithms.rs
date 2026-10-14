//! Algorithms (D-101): statements run in order, one `step` per occurrence of the trigger,
//! members chosen by numbers computed during the run, lists of values indexed by number.

mod common;
use common::*;
use prismal_present::Program;
use prismal_runtime::{run, Config, RunStatus};
use prismal_syntax::{compile, format};

const SORT: &str = r#"model Sorting {
  object Bar { discrete { v: Real = 0 } }
  parts {
    bars: Bar[6] { v = (5, 2, 6, 1, 4, 3)[index] }
  }
  discrete {
    i:     Real = 0
    j:     Real = 0
    held:  Real = 0
    swaps: Real = 0
    done:  Boolean = false
  }
  algorithm sort on every 1 s {
    for i in [1, 5] {
      for j in [1, 6 - i] {
        if bars[j].v > bars[j + 1].v {
          set held = bars[j].v
          set bars[j].v = bars[j + 1].v
          set bars[j + 1].v = held
          set swaps = swaps + 1
        }
        step
      }
    }
    set done = true
  }
}
"#;

fn values(prog: &Program, r: &prismal_runtime::Run, t: f64) -> Vec<f64> {
    let cm = prog.model("Sorting");
    let s = r.state_at(t);
    (1..=6).map(|k| s[cm.idx(&format!("Sorting.Bar.v@bars[{k}]"))].num()).collect()
}

/// Bubble sort of six values: one comparison per second, 15 comparisons in all, and as many
/// swaps as the list has inversions (9).
#[test]
fn bubble_sort_one_comparison_per_step() {
    let prog = program(SORT);
    let cm = prog.model("Sorting");
    let r = run(cm, Config::until(20.0));
    assert!(matches!(r.status, RunStatus::Completed), "{:?}", r.status);
    // At t = 0 the first comparison (5 > 2) has swapped.
    assert_eq!(values(&prog, &r, 0.0), [2.0, 5.0, 6.0, 1.0, 4.0, 3.0]);
    assert_eq!(values(&prog, &r, 1.0), [2.0, 5.0, 6.0, 1.0, 4.0, 3.0]);
    assert_eq!(values(&prog, &r, 2.0), [2.0, 5.0, 1.0, 6.0, 4.0, 3.0]);
    // The largest value has bubbled to the end after the first pass, 5 comparisons.
    assert_eq!(values(&prog, &r, 4.0)[5], 6.0);
    assert_eq!(values(&prog, &r, 20.0), [1.0, 2.0, 3.0, 4.0, 5.0, 6.0]);
    let fin = r.state_at(20.0);
    assert_eq!(fin[cm.idx("Sorting.swaps")].num(), 9.0);
    assert_eq!(fin[cm.idx("Sorting.done")], prismal_kernel::Value::Bool(true));
    // 15 comparisons, each ending at its `step`; a 16th occurrence, at 15 s, runs what follows
    // the last step (`set done`), and the finished algorithm occurs no more.
    let times = r.times("sort");
    assert_eq!(times, (0..16).map(f64::from).collect::<Vec<_>>());
}

#[test]
fn canonical_text_of_an_algorithm() {
    let text = format(&compile(SORT).unwrap().doc);
    assert!(text.contains("algorithm sort on every 1 s {"), "{text}");
    assert!(text.contains("for j in [1, 6 - i] {"), "{text}");
    assert!(text.contains("v = (5, 2, 6, 1, 4, 3)[index]"), "{text}");
    assert_eq!(format(&compile(&text).unwrap().doc), text);
}

const SEARCH: &str = r#"model Search {
  object Cell { discrete { v: Real = 0 } }
  parts {
    cells: Cell[8] { v = (3, 8, 11, 15, 21, 30, 34, 42)[index] }
  }
  param { target: Real = 30 }
  discrete {
    lo:    Real = 1
    hi:    Real = 8
    mid:   Real = 0
    found: Real = 0
    looks: Real = 0
  }
  event located on found_it
  event found_it on request
  algorithm search on request {
    while lo <= hi {
      set mid = floor((lo + hi) / 2)
      set looks = looks + 1
      if cells[mid].v == target {
        set found = mid
        emit found_it
        set lo = hi + 1
      } else if cells[mid].v < target {
        set lo = mid + 1
      } else {
        set hi = mid - 1
      }
      step
    }
  }
}

model Euclid {
  discrete {
    a: Real = 1071
    b: Real = 462
    r: Real = 0
  }
  algorithm gcd on every 1 s {
    while b != 0 {
      set r = mod(a, b)
      set a = b
      set b = r
      step
    }
  }
}
"#;

/// Binary search for 30 among eight sorted values, one look per request: 4 (21), then 6
/// (30), found after two looks. The `emit` makes the follower of `found_it` occur.
#[test]
fn binary_search_stepped_by_requests() {
    use prismal_runtime::Action;
    let prog = program(SEARCH);
    let cm = prog.model("Search");
    let mut cfg = Config::until(10.0);
    for k in 1..=5 {
        cfg = cfg.at(k as f64, Action::Request("Search.event.search".into()));
    }
    let r = run(cm, cfg);
    assert!(matches!(r.status, RunStatus::Completed), "{:?}", r.status);
    let at = |t: f64, n: &str| r.state_at(t)[cm.idx(&format!("Search.{n}"))].num();
    assert_eq!((at(0.5, "mid"), at(0.5, "looks")), (0.0, 0.0));
    assert_eq!((at(1.0, "mid"), at(1.0, "lo"), at(1.0, "hi")), (4.0, 5.0, 8.0));
    assert_eq!((at(2.0, "mid"), at(2.0, "found")), (6.0, 6.0));
    assert_eq!(r.times("located"), [2.0]);
    // The third request ends the loop (`lo > hi`); the fourth and fifth find it finished.
    assert_eq!(r.times("search"), [1.0, 2.0, 3.0]);
    assert_eq!(at(10.0, "looks"), 2.0);
}

/// Euclid's algorithm: gcd(1071, 462) = 21 after three remainders.
#[test]
fn euclid_by_remainders() {
    let prog = program(SEARCH);
    let cm = prog.model("Euclid");
    let r = run(cm, Config::until(10.0));
    assert!(matches!(r.status, RunStatus::Completed), "{:?}", r.status);
    let at = |t: f64, n: &str| r.state_at(t)[cm.idx(&format!("Euclid.{n}"))].num();
    assert_eq!((at(0.0, "a"), at(0.0, "b")), (462.0, 147.0));
    assert_eq!((at(1.0, "a"), at(1.0, "b")), (147.0, 21.0));
    assert_eq!((at(2.0, "a"), at(2.0, "b")), (21.0, 0.0));
    assert_eq!(r.times("gcd"), [0.0, 1.0, 2.0, 3.0]);
}

fn codes(src: &str) -> Vec<String> {
    match compile(src) {
        Err(ds) => ds.iter().map(|d| format!("{}: {}", d.code, d.message)).collect(),
        Ok(c) => match Program::new(c.doc) {
            Ok(_) => vec![],
            Err(ds) => ds.iter().map(|d| d.to_string()).collect(),
        },
    }
}

#[test]
fn algorithms_are_checked() {
    // A counter is a discrete number of the model.
    let d = codes("model M { state { x: Real = 0 }\n algorithm a on start { for x in [1, 3] { step } } }");
    assert!(d.iter().any(|m| m.contains("MK-E37") && m.contains("counter `x`")), "{d:?}");
    let d = codes("model M { discrete { n: Real = 0 }\n algorithm a on start { contribute n += 1 } }");
    assert!(d.iter().any(|m| m.contains("MK-E37") && m.contains("contribute")), "{d:?}");
    let d = codes("model M { discrete { n: Real = 0 }\n algorithm a on start { if n { step } } }");
    assert!(d.iter().any(|m| m.contains("MK-E01")), "{d:?}");
    let d = codes("model M { discrete { n: Real = 0 }\n derived { f: Real = floor(2 m / 1 m) + floor(n) }\n param { L: Length = floor(3 m) } }");
    assert!(d.iter().any(|m| m.contains("MK-E02") && m.contains("floor")), "{d:?}");
}

/// A loop that never reaches `step` stops the run; so does a member number out of range.
#[test]
fn runaway_loops_and_missing_members_fail() {
    let src = "model M { discrete { n: Real = 0 }\n algorithm a on start { while true { set n = n + 1 } } }";
    let prog = program(src);
    let r = run(prog.model("M"), Config::until(1.0));
    match &r.status {
        RunStatus::Stopped(d) => assert!(d.message.contains("without reaching `step`"), "{d:?}"),
        s => panic!("{s:?}"),
    }
    let src = "model M { object C { discrete { v: Real = 0 } }\n parts { cs: C[3] }\n discrete { k: Real = 4 }\n algorithm a on start { set cs[k].v = 1 } }";
    let prog = program(src);
    let r = run(prog.model("M"), Config::until(1.0));
    assert!(matches!(&r.status, RunStatus::Stopped(d) if d.message.contains("no member number 4")), "{:?}", r.status);
}
