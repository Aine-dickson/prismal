# 4. Motion

So far every value was fixed or computed from fixed values. This chapter adds **state**: values that evolve in time according to **flows**, which say how fast each state variable changes. The runtime integrates the flows with a numerical solver.

> **In plain words.**
> - **State** is what changes by itself; a **flow** says how fast it changes. [Start here](00-start-here.md) explains both with a draining bathtub.
> - **Integrating** a flow means adding up its many small changes to find where the value has got to. The part of Prismal that does it, step by step, is the **solver**.
> - The example is an **oscillator**: a mass on a spring that swings back and forth around its resting place (its **equilibrium**). **Damping** is the friction that makes the swings smaller each time. The spring's **stiffness** `k` says how hard it pulls back per metre stretched.

**Goal of this chapter's program.** A mass on a spring swings back and forth while friction slowly stops it; the learner changes the stiffness and friction and watches the position and energy over time.
**How it is built.** What moves is `state` (`x`, `v`); how fast it moves is `flow` (`der(x) = v`), with each force as a contribution (`der(v) += ...`) in a named `process`. Over time is shown with `series_plot(... every ...)` in a `plot` view whose `x` axis is a time range. Tests compare `observe`d values with the formula for a damped oscillator.

## State and flows

```text
model Oscillator {
  param {
    m:  Mass            = 1 kg        where m > 0 kg
    k:  Quantity<M/T^2> = 4 N/m       where k > 0 N/m      // stiffness
    c:  Quantity<M/T>   = 0.4 N*s/m   where c >= 0 N*s/m   // damping
    x0: Length          = 0.1 m
  }
  state {
    x: Length   = x0          // displacement from equilibrium
    v: Velocity = 0
  }
  derived {
    energy: Energy = 0.5 * m * v^2 + 0.5 * k * x^2
  }

  flow {
    der(x) = v
  }
  process spring  { flow der(v) += -(k / m) * x }
  process damping { flow der(v) += -(c / m) * v }
}
```

- `state { x: Length = x0 }` declares a **continuous state variable** with its initial value. The initial value is read once, at the start of the run: changing `x0` later does not move the mass.
- `flow { der(x) = v }` is a **flow**: the rate of change of `x` is `v`. `der(x)` has the dimension of `x` per time, so `der(x) = x` is rejected.
- A state variable is defined by one flow (`=`) or by several **contributions** (`+=`) that are summed. Here the spring and the damping each contribute to `der(v)`; the acceleration is their sum.
- `process spring { ... }` groups flows (and events) under a name. Processes do not change the result; they name the parts of a model so that they can be observed, highlighted and switched off in a variant.
- A state variable with no flow, and no equation that gives its rate, stays constant. (An equation can give a rate in place of a flow: [chapter 17](17-solving-equations.md#equations-that-give-rates).)

Time is available as `t` (the current instant), `t0` (the start of the run) and `elapsed` (`t - t0`).

## Runs and solvers

A run starts at `t0` and ends at the case's `until`. The default solver is `dopri5`, an adaptive Runge-Kutta method with relative tolerance `1e-6` and absolute tolerance `1e-9`. A case can choose another configuration:

> **In plain words.** A solver moves time forward in small steps. **Runge-Kutta** methods look at the rate of change at a few points inside each step to make the step accurate. **Adaptive** means the solver chooses the step size itself: small steps where things change fast, large ones where they change slowly. The **tolerance** is the error it allows per step: a relative tolerance of `1e-6` means about one part in a million of the value, and the absolute tolerance `1e-9` keeps values near zero accurate too. `rk4` is the classic fixed-step Runge-Kutta method: it takes steps of exactly the size `h` you give it.

```cases
run damped of Oscillator with OscillatorLab {
  until t0 + 10 s
  expect {
    x5  == -0.0336851680590413 m within 1e-6 m
    x10 == 0.00791160236189625 m within 1e-6 m
  }
}

run precise of Oscillator with OscillatorLab {
  config { rtol = 1e-10; atol = 1e-12 }
  until t0 + 10 s
  expect { x5 == -0.0336851680590413 m within 1e-10 m }
}

run fixed_step of Oscillator with OscillatorLab {
  config { solver = rk4; h = 0.001 s }
  until t0 + 10 s
  expect { x5 == -0.0336851680590413 m within 1e-10 m }
}

run undamped of Oscillator with OscillatorLab {
  param { c = 0 N*s/m }
  until t0 + 10 s
  expect { e10 == 0.02 J within 1e-6 J }      // energy is conserved: k x0^2 / 2
}
```

The expected values come from the **closed-form solution** of the damped oscillator, a formula that gives the position at any time directly, without stepping (`e` is about 2.718, `γ` says how fast the swings die out and `ω` how fast they repeat): `x(t) = x0 e^(-γt) (cos(ωt) + (γ/ω) sin(ωt))` with `γ = c / 2m` and `ω = sqrt(k/m - γ²)`. Tolerances say how exact the numerical solution must be: the default solver meets `1e-6 m`, tighter tolerances or a small fixed step meet `1e-10 m`. `rk4` needs its step `h`; without one the configuration is rejected (`expect { configuration rejected }`).

## Watching motion

```text
presentation OscillatorLab for Oscillator {
  view mass: plot(x: [-0.12 m, 0.12 m], y: [-1, 1]) {
    marker(at: (x, 0)) as body
  }
  view position: plot(x: [0 s, 20 s], y: [-0.12 m, 0.12 m]) {
    series_plot(x every 0.02 s)
  }
  view energy_plot: plot(x: [0 s, 20 s], y: [0 J, 0.025 J]) {
    series_plot(energy every 0.05 s)
  }
  panel controls {
    slider(c, range: [0 N*s/m, 2 N*s/m])
    slider(k, range: [1 N/m, 20 N/m])
    label(energy)
  }
  observe {
    x5  = x      at t0 + 5 s
    x10 = x      at t0 + 10 s
    e10 = energy at t0 + 10 s
  }
}
```

- A **plot view**'s axis ranges give the axes their dimensions: `x: [-0.12 m, 0.12 m]` is a length axis, `[-1, 1]` a plain one. A marker in a plot is at a pair of values in those dimensions: `(x, 0)`.
- `series_plot(x every 0.02 s)` samples `x` every 0.02 s from the start and draws it against elapsed time, up to the instant shown. Its plot's `x` axis must be a time range.
- `trace(pos every 0.05 s)`, in a spatial view, draws the path of a moving point the same way (see RP-01's lab).
- `observe { x5 = x at t0 + 5 s }` records `x` at one instant. Other schedules: `live` (the instant shown), `every 0.1 s` (a series), `on E` (at each occurrence of an event, chapter 5), `over [t0, t0 + 5 s]` (for logs).

In the player, a presentation of a model that moves has a **clock**: play, pause, seek and reset. The run is computed ahead; the view shows the instant on the clock. Moving a slider while it plays changes the parameter **at the instant shown**: the past is unchanged and the rest of the run is recomputed. Here, turning up the damping in the middle of an oscillation makes it die out faster from that moment on, and undo restores the original run.

## Mistakes

```error
// error: MK-E01
model Wrong {
  state { x: Length = 0 m }
  flow { der(x) = x }
}
```

The rate of change of a length is a velocity.

```error
// error: MK-E06
model Wrong {
  param { k: Real = 1 }
  flow { der(k) = 1 }
}
```

Only state variables have flows; parameters change only by intervention.

```error
// error: MK-E22
model Wrong {
  state { x: Length = 0 m; v: Velocity = 1 m/s }
  flow { der(x) = v; der(x) = 2 * v }
}
```

A state variable has one defining flow; to combine several influences, use `+=` contributions.

## Exercises

Solutions to the exercises not solved here are in [chapter 11](11-solutions.md).

1. Add a driving force: `process drive { flow der(v) += (F0 / m) * cos(w * (t - t0)) }` with parameters `F0: Force` and `w: Frequency`. Plot `x` over 40 s and look for resonance near `w = 2 /s`.
2. Write a model of exponential cooling without units first: `state T: Real = 90`, `flow der(T) = -r * (T - Ta)` with `r: Quantity<1/T>`. Check `T` at 10 s against `Ta + (T0 - Ta) e^(-r t)`.
3. Write a projectile in the plane with `pos: Point` and `vel: Vector<Velocity>`, gravity as a process, and a spatial view with a `trace`. (RP-01 is a complete solution, with the landing as an event from chapter 5.)

<details>
<summary>A solution to exercise 2</summary>

```text
model Cooling {
  param {
    T0: Real          = 90
    Ta: Real          = 20
    r:  Quantity<1/T> = 0.1 /s
  }
  state { temp: Real = T0 }
  flow { der(temp) = -r * (temp - Ta) }
}

presentation CoolingChecks for Cooling {
  observe { at10 = temp at t0 + 10 s }
}
```

```cases
run cooling of Cooling with CoolingChecks {
  until t0 + 10 s
  expect { at10 == 45.7515608820010 within rel 1e-6 }    // 20 + 70 e^-1
}
```

`within rel 1e-6` is a relative tolerance, matching the solver's default relative tolerance. Temperatures are written as plain numbers here. Absolute temperatures with units are affine quantities, which the prototype does not implement yet.

</details>
