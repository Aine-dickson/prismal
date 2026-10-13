# Few Molecules, Many Molecules

A complete example: a reaction in which molecules switch between two forms, A and B, as a lab where the learner sets how many molecules there are and runs the reaction many times, and as a narrated lesson. Every program on this page is compiled and its cases run by `cargo test` (`crates/prismal-web/tests/examples.rs`).

**Goal.** A learner finds that the rate equations of chemistry describe many molecules well and few molecules poorly: with a thousand molecules the share in form B follows the smooth curve, with twenty it jumps around it, and the spread over many runs shrinks as the number of molecules grows.
**How it is built.**

- *the molecules*: two counts, `a` and `b`, as discrete state: they change only when a molecule switches;
- *the reactions*: two random events, `forward` (A to B) and `back` (B to A), each at a rate proportional to the molecules that can take it (`random(k1 * a)`), which is Gillespie's stochastic simulation;
- *the rate equation*: the share in form B as the chemist writes it, `der(x) = k1 (1 - x) - k2 x`, as continuous state in the same model, so both are drawn on one plot;
- *the lab*: the share against time with the rate equation's curve, a histogram of the share at 15 s over 200 runs, a slider for the number of molecules, and buttons to replay or draw another run;
- *the lesson*: a thousand molecules, then twenty, then the spread of many runs of each, then the learner's own number.

## The model

```text
model Isomer {
  param {
    n0: Real      = 1000  in [10, 2000]   // molecules, all in form A at the start
    k1: Frequency = 0.3 /s                // A to B, per molecule
    k2: Frequency = 0.1 /s                // B to A, per molecule
  }
  discrete {
    a: Real = n0
    b: Real = 0
  }
  state {
    x: Real = 0                           // the share in form B, by the rate equation
  }
  derived {
    share: Real = b / n0                  // the share in form B, counted
    x_eq:  Real = k1 / (k1 + k2)          // where both settle
    tau:   Time = 1 / (k1 + k2)           // how quickly they settle
  }
  flow {
    der(x) = k1 * (1 - x) - k2 * x
  }

  event forward on random(k1 * a) { set a = a - 1; set b = b + 1 }
  event back    on random(k2 * b) { set a = a + 1; set b = b - 1 }
}
```

- Each event happens at random instants. With `a` molecules in form A, each switching at rate `k1`, some molecule switches at rate `k1 * a`; the runtime computes the waiting times exactly, and both rates change after every switch.
- `x` is not counted: it is the rate equation's answer, `x(t) = x_eq (1 - e^(-t / tau))`, which the counted share should follow on average.
- With the defaults, `x_eq = 0.75` and `tau = 2.5 s`.

## The lab

```text
presentation IsomerLab for Isomer {
  title "Few molecules, many molecules"
  view curve: plot(x: [0 s, 20 s], y: [0, 1]) {
    title("Share in form B")
    series_plot(x every 0.1 s, color: gray, line: dashed)
    series_plot(share every 0.1 s, color: blue)
  }
  view spread: plot(x: [0.5, 1], y: [0, 40], follow: y) {
    title("Share at 15 s, in 200 runs")
    histogram(share at t0 + 15 s over 200 runs, bins: 25, color: purple)
  }
  panel controls {
    slider(n0, range: [10, 2000], step: 10, ends: ("few", "many"))
    button(restart, label: "Start again")
    button(reseed, label: "Another run")
    text("Counted: {share:.3}. Rate equation: {x:.3}.")
  }
  layout row(column(curve, spread), controls)
  observe {
    early   = x at t0 + 2.5 s
    shares  = share at t0 + 15 s over 200 runs
  }
}
```

- The dashed gray curve is the rate equation; the blue staircase is the counted share, one step per switch.
- The histogram runs the reaction 200 times with the learner's number of molecules and counts where the share is at 15 s. It is computed again when the slider moves or the learner draws another run.
- The slider changes `n0`, the number of molecules the run *starts* with: changing it does not add or remove molecules from a run under way. "Start again" (`restart`) runs the reaction again from all A with the slider's value; "Another run" (`reseed`) does the same with the next seed. (`button(reset)` would go back to the lab's own starting values, a thousand molecules.)

```cases
run thousand of Isomer with IsomerLab {
  config { seed = 1 }
  until t0 + 15 s
  expect {
    early == 0.474090 within 1e-4
    mean(shares) == 0.7481 within 0.003
    deviation(shares) in [0.010, 0.018]
  }
}

run twenty of Isomer with IsomerLab {
  param { n0 = 20 }
  config { seed = 1 }
  until t0 + 15 s
  expect {
    mean(shares) == 0.7481 within 0.02
    deviation(shares) in [0.07, 0.12]
  }
}
```

The numbers: `x(2.5 s) = 0.75 (1 - e^-1) = 0.474090`. At 15 s each molecule is in form B with probability `p = 0.75 (1 - e^-6) = 0.748141`, independently of the others, so the count of B is binomial: the share has mean `p` and deviation `sqrt(p (1 - p) / n0)`, which is `0.0137` for a thousand molecules and `0.0971` for twenty. The mean of 200 runs is within `0.003` (thousand) and `0.02` (twenty), each about three standard errors.

## The lesson

```text
presentation IsomerLesson for Isomer {
  title "Few molecules, many molecules"
  view curve: plot(x: [0 s, 20 s], y: [0, 1]) {
    series_plot(x every 0.1 s, color: gray, line: dashed)
    series_plot(share every 0.1 s, color: blue)
  }
  view spread: plot(x: [0.5, 1], y: [0, 40], follow: y) {
    histogram(share at t0 + 15 s over 200 runs, bins: 25, color: purple)
  }
  panel notes {
    text("Molecules: {n0:.0}. Share in form B: {share:.2}.")
  }
  layout row(column(curve, spread), notes)
  permit learner { timeline_controls }

  timeline {
    scene many {
      beat intro { narrate "A thousand molecules, all in form A. Each one switches to form B, and back, at random moments." for 9 s }
      beat watch { run rate 1; narrate "The blue line counts the molecules in form B. The dashed line is the rate equation from the textbook." for 8 s }
      beat close { hold; narrate "With a thousand molecules the count follows the equation closely." for 5 s }
    }
    scene few {
      beat fewer { intervene { set n0 = 20 }; restart; narrate "Now only twenty molecules." for 3 s }
      beat jumpy { run rate 1; narrate "The count jumps around the curve. Each switch moves the share by a twentieth." for 10 s }
      beat average { hold; narrate "The equation still gives the average, but no single run follows it." for 5 s }
    }
    scene yours {
      beat choose {
        explore limit 30 s keep n0 {
          slider(n0, range: [10, 2000], step: 10, ends: ("few", "many"))
        } fallback {
          sequence { intervene { set n0 = 200 }; wait 3 s }
        }
      }
      beat again  { restart; run rate 1; narrate "The reaction again, with your number of molecules." for 6 s }
      beat spread { narrate "The histogram shows two hundred runs. The fewer the molecules, the wider the spread." for 7 s }
    }
  }
}
```

- `intervene { set n0 = 20 }` changes the number of molecules at the instant shown; `restart` then runs the reaction again from all A with that number. Without `restart` the run would go on with the thousand molecules it has, and `reset` would replay the run from its start, where there were still a thousand.
- The explore beat keeps the learner's `n0` (`keep n0`); the beat after it restarts with it. A video plays the fallback, 200 molecules.

```cases
run lesson of Isomer with IsomerLesson {
  config { seed = 3 }
  expect {
    end of intro == 9 s within 1e-9 s
    start of jumpy == 25 s within 1e-9 s
  }
}
```

## Exporting the lesson

```sh
cargo run --release -p prismal-media -- examples/reaction.md IsomerLesson reaction.mp4 --speech system
```

The narrations are timed for the system voice: the exporter reports any narration whose voice lasts longer than its `for` (D-079). Without [ffmpeg](https://ffmpeg.org), give a folder name instead of `reaction.mp4` to get the frames as images.
