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
- A rate may read anything in the model, including continuous state and `t`; see [Rates that change between events](#rates-that-change-between-events).
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
- A slider that sets a starting value (how many nuclei there are, `n0`) changes nothing in a run under way: the count was set at the start. `button(restart)` starts a new run from `t0` with the parameters as the learner has set them; `reseed` does the same with the next seed; `reset` goes back to the presentation's own starting values. In a lesson, the timeline action `restart` does the same after an `intervene`, and an explore beat may offer `button(restart)` and `button(reseed)` to the learner (D-088; `examples/reaction.md`).

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

## Rates that change between events

**Goal.** A counter carried away from a radioactive source: the learner hears the clicks thin out as the distance grows, because the rate falls with the square of the distance.
**How it is built.** The distance `x` is continuous `state` with a `flow`. A `derived` value `level` gives the click rate from `x`, and the event's trigger `random(level)` reads it. Nothing else is needed: the runtime follows the rate as it changes between clicks.

```text
model Counter {
  param {
    r0: Frequency = 40 /s     // clicks per second at the reference distance
    d0: Length    = 1 m       // reference distance
    v:  Velocity  = 0.5 m/s   // the counter's speed away from the source
  }
  state { x: Length = 1 m }
  discrete { clicks: Real = 0 }
  derived { level: Frequency = r0 * (d0 / x)^2 }

  flow der(x) = v

  event click on random(level) { set clicks = clicks + 1 }
}

presentation CounterLab for Counter {
  view counts: plot(x: [0 s, 10 s], y: [0, 80]) {
    series_plot(clicks every 0.1 s)
  }
  panel controls {
    slider(v, range: [0.1 m/s, 2 m/s])
    label(level)
    label(clicks)
  }
  observe { total = clicks at t0 + 10 s }
}
```

```cases
run thins_out of Counter with CounterLab {
  config { seed = 4 }
  until t0 + 10 s
  expect { total in [40, 95] }
}
```

- The rate reads `x`, which flows, so it is different at every instant. The event happens when the rate, added up over time, reaches a random amount drawn from the event's stream; the runtime adds it up along the solver's steps (RC-13.8a). The result is exact up to the solver's accuracy, not a sampling on a grid.
- Over 10 s the expected number of clicks is `r0 d0^2 (1/x(0) - 1/x(10 s)) / v = 40 * (1 - 1/6) / 0.5`, about 67, with a deviation of about 8.
- A rate that reads `t` works the same way: `random(k * elapsed / tau)` is an ageing part that fails more often the older it is.
- A rate that turns negative stops the run at the instant it does: write rates that stay at or above 0.

## Waiting times

**Goal.** A counter near a steady source clicks at random. The learner sees the gaps between clicks collect into a histogram that takes the shape of the exponential law: short gaps are common, long ones rare.
**How it is built.** The model keeps the time of the last click and the gap before the current one, both set in the click's handler. `histogram(gap on click)` collects the gap at every click of the run, up to the instant shown, so the histogram grows while the lab plays.

```text
model Geiger {
  param { lambda: Frequency = 2 /s }
  discrete {
    last:   Time = 0 s     // when the last click came
    gap:    Time = 0 s     // the wait before the last click
    clicks: Real = 0
  }

  event click on random(lambda) {
    set gap    = elapsed - last
    set last   = elapsed
    set clicks = clicks + 1
  }
}

presentation GeigerLab for Geiger {
  view gaps: plot(x: [0 s, 3 s], y: [0, 20], follow: y) {
    histogram(gap on click, bins: 30)
  }
  panel controls {
    slider(lambda, range: [0.5 /s, 5 /s])
    button(reseed, label: "Another run")
    label(clicks)
  }
  observe { gaps = gap on click }
}
```

```cases
run waits of Geiger with GeigerLab {
  config { seed = 9 }
  until t0 + 300 s
  expect {
    mean(gaps) == 0.5 s within 0.06 s
    deviation(gaps) in [0.42 s, 0.58 s]
  }
}
```

- `gap on click` is the value of `gap` just after each click, the same phrase as the observation `gaps = gap on click`. `histogram(x every Δ)` collects samples of a value through the run in the same way.
- Both handler lines read the values from before the click: `gap` gets the time since the previous click, then `last` moves on.
- With a rate of 2 per second the mean wait is half a second, and the deviation is the same: for the exponential law the two are equal. A histogram over many runs (`at τ over n runs`, above) shows how runs differ; a histogram of one run shows what happens again and again within it.

## Measuring with an instrument

**Goal.** A ball dropped from 2 m and measured with a ruler that reads to the centimetre and errs by about 2 cm: the learner sees the readings scatter about the true height, and repeated measurements spread by the ruler's uncertainty.
**How it is built.** The model is exact; the instrument belongs to the presentation. `instrument ruler(...)` declares how its readings differ from the truth: `resolution` rounds them, `range` leaves out values the instrument cannot read, `noise` adds a random error with that deviation. Writing `ruler(y)` instead of `y` in a sampled source, an ensemble or an observation reads `y` through the ruler.

```text
model Drop {
  param { g: Acceleration = 9.81 m/s^2 }
  state {
    y: Length   = 2 m
    v: Velocity = 0 m/s
  }
  flow {
    der(y) = v
    der(v) = -g
  }
}

presentation DropLab for Drop {
  instrument ruler(resolution: 1 cm, range: [0 m, 2 m], noise: 2 cm)

  view height: plot(x: [0 s, 0.6 s], y: [0 m, 2.2 m]) {
    series_plot(y every 0.01 s)
    series_plot(ruler(y) every 0.05 s, color: orange)
  }
  view repeats: plot(x: [1.48 m, 1.64 m], y: [0, 80]) {
    histogram(ruler(y) at t0 + 0.3 s over 300 runs, bins: 16)
  }
  panel controls {
    button(reseed, label: "Measure again")
  }
  observe {
    reading  = ruler(y) at t0 + 0.3 s
    readings = ruler(y) at t0 + 0.3 s over 300 runs
  }
}
```

```cases
run measured of Drop with DropLab {
  config { seed = 5 }
  until t0 + 0.6 s
  expect {
    reading in [1.48 m, 1.68 m]
    mean(readings) == 1.5586 m within 0.006 m
    deviation(readings) in [0.017 m, 0.023 m]
  }
}
```

- The true height at 0.3 s is `2 m - g (0.3 s)^2 / 2`, about 1.5586 m. A single reading may be several centimetres off; the mean of 300 is within a few millimetres.
- Each option has the dimension of what the instrument reads: a ruler's options are lengths. An instrument without options reads exactly.
- A series plot draws readings as separate points, not joined into a line: each is one measurement.
- A reading outside the `range` is not available: a series plot shows no point, a table writes "out of range", an ensemble leaves the run out.
- The noise of a reading depends on the seed, the instrument, the source and the instant: the orange points and the observation `reading` agree, the same seed gives the same readings, and "Measure again" gives others.
- Text alternatives add the instrument's **standard uncertainty**, `sqrt(noise^2 + resolution^2 / 12)`: the noise together with the rounding.
- An instrument reads at instants, so a `label` or a `live` observation cannot use one.

## Exercises

1. Change the population's rate so that the half-life is 20 s. What is `lambda`?
2. Write a die: an event every second that sets `face` to `integer(1, 6)`, and a test that `face` is in `[1, 6]`.
3. Make the walker of the random walk step only with probability one half: add `if bernoulli(0.5)` to its event.
