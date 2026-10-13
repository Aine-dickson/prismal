# 15. Chance

Some things happen at random: a nucleus decays, a molecule meets another, a coin lands heads. Prismal writes chance with two things already met in chapter 5, events and operations, and one new trigger:

- `random(λ)` is a trigger: the event happens at random instants, on average `λ` times per unit of time.
- `normal(m, s)`, `uniform(a, b)`, `bernoulli(p)` and the other **distributions** are functions that give a random value. They are written only in an event's condition and operations.

Every run has a **seed**, a whole number in its `config` block. The same seed gives the same run, every time; another seed gives another run. Tests of random models check ranges that hold for any seed.

> **In plain words.**
> - A **rate** says how often something happens on average: `0.5 /s` is once every two seconds on average, though the gaps vary.
> - A **distribution** describes which values are likely: `normal(0 m, 1 m)` gives values near `0 m`, most within `1 m`.
> - A **seed** picks one of the many possible runs. Lessons and tests name it, so that they can be repeated.
> - Random models are tested on what must hold whatever the luck: a count stays within a range, an average is near its expected value.

## A decaying population

**Goal.** A sample of 1000 radioactive nuclei: the learner sees the count fall by single random decays and compares it with the smooth exponential law.
**How it is built.** The count is `discrete` state (it changes only by events). One `event` with the trigger `random(lambda * n)` removes one nucleus: with `n` nuclei each decaying at rate `lambda`, some nucleus decays at rate `lambda * n`. A `derived` value gives the exponential law for comparison. The presentation plots both with `series_plot`.

```text
model Sample {
  param {
    lambda: Frequency = 0.1 /s     // decay rate of one nucleus
    n0:     Real      = 1000
  }
  discrete {
    n: Real = n0
  }
  derived {
    law: Real = n0 * exp(-lambda * elapsed)
    half_life: Time = log(2) / lambda
  }

  event decay on random(lambda * n) { set n = n - 1 }
}
```

- `random(lambda * n)` takes a rate, a quantity per unit of time (`Frequency` is `1/T`). A rate without a unit is an error: write `0.1 /s`, not `0.1`.
- The rate is read again after every event, so it falls as nuclei decay. Between events the waiting time is random with the exponential law; the runtime computes it exactly, not by small time steps.
- A rate may read parameters and discrete state, not continuous state or `t` (MK-E27): a rate changes only when an event changes what it reads.
- When `n` reaches 0 the rate is 0 and the event never happens again.

```text
presentation SampleLab for Sample {
  view counts: plot(x: [0 s, 30 s], y: [0, 1000]) {
    series_plot(law every 0.5 s)
    series_plot(n every 0.5 s)
  }
  panel controls {
    slider(lambda, range: [0.02 /s, 0.5 /s])
    label(n)
    label(half_life)
  }
  observe {
    at_half = n at t0 + 6.931471805599453 s
    first   = elapsed on decay
  }
}
```

```cases
run halves of Sample with SampleLab {
  config { seed = 1 }
  until t0 + 7 s
  expect {
    at_half in [440, 560]
    first[1] in [0 s, 0.1 s]
  }
}

run another_seed of Sample with SampleLab {
  config { seed = 2 }
  until t0 + 7 s
  expect {
    at_half in [440, 560]
  }
}
```

At the half-life each nucleus survives with probability 1/2, so about 500 remain: the count is binomial, with a standard deviation of about 16. The range `[440, 560]` is more than 3.5 deviations wide on each side, so the test holds for any seed except a very unlucky one, and the seeds named here are fixed. The first decay comes after an average of `1 / (0.1 /s * 1000) = 0.01 s`.

## Many runs

**Goal.** The same sample run 200 times: the learner sees how the count at the half-life spreads around 500, draws another run with a button, and the test checks the average rather than one run.
**How it is built.** The model is `Sample` from the section before. Two runtime-control buttons in a `panel`: `button(reset)` replays the run shown, `button(reseed)` starts again with the next seed. A `histogram` in a `plot` view takes an **ensemble source**, `n at t0 + 6.93 s over 200 runs`: the value of `n` at that instant in each of 200 runs. The observation `halves` has the same form, and the test checks `mean(halves)` and `deviation(halves)`.

```text
model Sample2 {
  param {
    lambda: Frequency = 0.1 /s
    n0:     Real      = 1000
  }
  discrete { n: Real = n0 }
  event decay on random(lambda * n) { set n = n - 1 }
}

presentation ManyRuns for Sample2 {
  view counts: plot(x: [0 s, 30 s], y: [0, 1000]) {
    series_plot(n every 0.5 s)
  }
  view spread: plot(x: [420, 580], y: [0, 80]) {
    histogram(n at t0 + 6.93 s over 200 runs, bins: 16)
  }
  panel controls {
    slider(lambda, range: [0.05 /s, 0.2 /s])
    button(reset, label: "Replay")
    button(reseed, label: "Another run")
  }
  observe {
    halves = n at t0 + 6.93 s over 200 runs
  }
}
```

- An ensemble source is written `value at instant over count runs`, inside a representation's parentheses or after an observation's `=`. Run `k` uses the seed of the run shown plus `k - 1`, so the first run is the one shown. Each run stops at the instant, so a long lab stays quick.
- The histogram's `x` axis is the value, its `y` axis how many runs fall in each bin; `bins: 16` divides the `x` range into 16 equal bins. Its text alternative gives the mean and the deviation.
- When the learner moves the slider, the histogram is computed again for the new rate; `reseed` moves it to 200 other runs.

```cases
run spread of Sample2 with ManyRuns {
  config { seed = 4 }
  until t0 + 7 s
  expect {
    mean(halves) == 500 within 6
    deviation(halves) in [11, 21]
  }
}
```

`mean(halves)` is the average of the 200 values and `deviation(halves)` their standard deviation. At `6.93 s` each nucleus survives with probability `e^-0.693 = 0.5001`, so the count is binomial with mean `500.1` and deviation `sqrt(1000 * 0.5001 * 0.4999) = 15.8`. The mean of 200 runs then has a deviation of `15.8 / sqrt(200) = 1.1`, and the test's tolerance is five times that.

## Atoms one by one

**Goal.** Twenty atoms in a row, each vanishing at its own random moment, so that the learner sees that decay happens to single atoms with no memory of how long they have waited.
**How it is built.** The atoms are a collection whose membership changes (chapter 9, `[20, max 20]`). A `for a in atoms { event ... }` gives each atom its own event with the trigger `random(lambda)`, whose handler `destroy`s it. A view draws a marker per atom; an atom that is destroyed is no longer drawn.

```text
space Line = euclidean(2)

model Atoms in Line {
  param { lambda: Frequency = 0.2 /s }
  object Atom {
    param { pos: Point = origin }
  }
  parts {
    atoms: Atom[20, max 20] { pos = origin + (index * 1 m, 0 m) }
  }
  derived { left: Real = count(atoms) }

  for a in atoms {
    event decay on random(lambda) { destroy a }
  }
}

presentation AtomsLab for Atoms {
  view row: spatial(Line, scale: 1 m -> 30 px, y: up) {
    for a in atoms { marker(a.pos) }
  }
  panel counts { label(left) }
  observe { end = left at t0 + 50 s }
}
```

```cases
run all_gone of Atoms with AtomsLab {
  config { seed = 3 }
  until t0 + 50 s
  expect { end in [0, 3] }
}
```

Each atom has its own clock. After 50 s, ten mean lifetimes, each atom is left with probability `e^-10`, so almost surely none is.

## A random walk

**Goal.** A walker that takes a random step every fifth of a second, so that the learner sees diffusion: the walker wanders, and its distance from the start grows like the square root of time.
**How it is built.** The position is `discrete` state. An `event` on `every 0.2 s` adds a step drawn with `normal(0, s)`: for a vector, each component is drawn independently. A `trace` draws the path.

```text
space Plane = euclidean(2)

model Walk in Plane {
  param { s: Length = 0.5 m }
  discrete {
    pos:   Point = origin
    steps: Real  = 0
  }

  event step on every 0.2 s from t0 + 0.2 s {
    set pos = pos + normal(0, s)
    set steps = steps + 1
  }
}

presentation WalkLab for Walk {
  view scene: spatial(Plane, scale: 1 m -> 40 px, y: up) {
    trace(pos every 0.2 s)
    marker(pos)
  }
  panel controls {
    slider(s, range: [0.1 m, 1 m])
    label(steps)
  }
  observe { x_end = pos.x at t0 + 20 s }
}
```

```cases
run wanders of Walk with WalkLab {
  config { seed = 5 }
  until t0 + 20 s
  expect { x_end in [-25 m, 25 m] }
}
```

After 100 steps of deviation `0.5 m`, `pos.x` has a deviation of `0.5 m * sqrt(100) = 5 m`; the range is five deviations wide. The draws of one event are independent of each other and of other events (RC-13.5).

**Distributions.** Each takes its arguments in parentheses and is written in an event's condition or operations only.

| Function | Gives |
|---|---|
| `uniform(a, b)` | a quantity between `a` and `b`, every value equally likely |
| `normal(m, s)` | a quantity or vector near `m`, with deviation `s` (a vector: each component) |
| `exponential(r)` | a waiting time at rate `r` (`exponential(2 /s)` is a time) |
| `bernoulli(p)` | `true` with probability `p` |
| `integer(a, b)` | a whole number from `a` to `b`, both included: `integer(1, 6)` is a die |
| `poisson(m)` | a count with mean `m` |
| `binomial(n, p)` | how many of `n` tries succeed, each with probability `p` |
| `choice(x, y, ...)` | one of its arguments, equally likely: `choice(heads, tails)` |

A name declared in the program wins over these: a function the author names `normal` is that function.

## A reversible reaction

**Goal.** Molecules switching between two forms, A to B and back, so that the learner sees the counts settle at an equilibrium where both directions happen equally often.
**How it is built.** Two random events, one per direction, with rates proportional to the count they consume. This is Gillespie's stochastic simulation: one event per reaction.

```text
model Isomer {
  param {
    k1: Frequency = 2 /s    // A to B, per molecule
    k2: Frequency = 1 /s    // B to A, per molecule
  }
  discrete {
    a: Real = 300
    b: Real = 0
  }
  derived { b_eq: Real = (a + b) * k1 / (k1 + k2) }

  event forward on random(k1 * a) { set a = a - 1; set b = b + 1 }
  event back    on random(k2 * b) { set a = a + 1; set b = b - 1 }
}

presentation IsomerLab for Isomer {
  view counts: plot(x: [0 s, 5 s], y: [0, 300]) {
    series_plot(a every 0.05 s)
    series_plot(b every 0.05 s)
    series_plot(b_eq every 0.05 s)
  }
  observe { b_end = b at t0 + 5 s }
}
```

```cases
run settles of Isomer with IsomerLab {
  config { seed = 11 }
  until t0 + 5 s
  expect { b_end in [170, 230] }
}
```

At equilibrium `b` is near `300 * 2 / 3 = 200`, with a deviation of about 8. Each event changes both counts, so the rates of both change at every event; the runtime keeps one clock per event, which is exact however the rates change (D-083).

## Exercises

1. Change the population's rate so that the half-life is 20 s. What is `lambda`?
2. Write a die: an event every second that sets `face` to `integer(1, 6)`, and a test that `face` is in `[1, 6]`.
3. Make the walker of the random walk step only with probability one half: add `if bernoulli(0.5)` to its event.
