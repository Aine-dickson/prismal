# Start Here: Ideas Before Code

This page is for readers who have never written a simulation or done mathematical programming. It explains the handful of ideas every Prismal program is built from, in plain words, with one small example: a bathtub draining. No knowledge beyond school arithmetic is assumed. Readers who already know what a differential equation is can skip to [the tour](00-tour.md).

## What a Prismal program is

A Prismal program describes a small piece of the world and how it behaves, then says how to show it. It does not give the computer step-by-step instructions ("add 3, then repeat"). It states facts ("the water drains faster when the tub is fuller") and Prismal works out what happens.

A program has separate parts for separate jobs:

| Part | Answers | Example |
|---|---|---|
| `model` | What exists, and how does it behave? | a tub of water that drains |
| `presentation` | How is it shown, and what may the learner change? | a graph of the water level, a slider for the drain |
| `run` | Does the model do what we expect? | after 10 seconds the level is about 30 cm |

The model never mentions graphs or colors. The same model can be shown in several ways (a lab to play with, a narrated lesson, a video) without changing it.

## Numbers that mean something: quantities and units

In most programming languages a number is just a number: `0.5`. In Prismal a number usually measures something, and says what: `0.5 m` (half a metre), `20 s` (twenty seconds), `9.81 m/s^2` (an acceleration).

Prismal checks that the units make sense, the way a careful science teacher would:

- `0.5 m + 20 cm` is fine: both are lengths, and the answer is `0.7 m`.
- `0.5 m + 20 s` is refused: a length plus a time means nothing.
- `0.5 m / 20 s` is a speed, `0.025 m/s`. Dividing a length by a time gives a speed.

The kind of thing a quantity measures (length, time, speed, mass) is its **dimension**. The **unit** is the scale it is written in (m, cm, km are all units of length). Most mistakes in scientific programs are unit mistakes, so catching them early saves a lot of confusion. [Chapter 2](02-quantities-and-units.md) covers units in full.

## Model values: settings, state and calculations

The named values a model works with are its **model values**. (The reference chapters call them *bindings*: a name bound to a value.) Most play one of three main roles. Think of a bathtub:

- A **parameter** is a setting chosen before things start, like how wide the drain is. It stays the same during a run unless the learner moves a slider. Written in a `param` block.
- **State** is the information the model carries forward from one moment to the next: its memory of how things are right now, like the water level. To know the level a second from now, the model needs the level now. Each state value is written with its starting value. State comes in two forms:
  - **continuous state** changes smoothly as time passes, like the level falling. Written in a `state` block.
  - **discrete state** changes only at particular moments, all at once, like a switch flipping from "draining" to "empty". Written in a `discrete` block (see Events, below).
- A **derived** value is calculated from the others and is always up to date, like "the level in centimetres" or "the amount of water". It needs no memory: it can be worked out again at any moment. Written in a `derived` block.

```
                       MODEL VALUES
                            |
          +-----------------+-----------------+
          |                 |                 |
     Parameters           State            Derived
     (settings)      (carried forward)   (calculated)
                            |
                    +-------+-------+
                    |               |
               continuous        discrete
               (`state`)       (`discrete`)
```

These three roles are enough for most models and for every program in the first chapters. Two more kinds appear later: **constants**, values fixed by nature that no slider may change, such as the speed of light (`const`, [chapter 2](02-quantities-and-units.md)); and **unknowns**, values found by solving equations together, such as the currents in a circuit (`unknown`, [chapter 17](17-solving-equations.md)).

## Change over time: rates

The heart of a simulation is saying **how fast** each state value changes. This is called its **rate of change**.

You already know rates from everyday life:

- A car's speed is the rate at which its position changes: at 50 km/h, its position grows by 50 km every hour.
- A tap filling a bucket at 2 litres per minute is the rate at which the amount of water changes.

In Prismal, `der(h)` means "the rate of change of `h`" (`der` is short for derivative, the mathematical name for a rate of change). A line such as

```prismal
der(h) = -h / tau
```

reads: "the level `h` goes down (the minus sign) at a rate equal to the level divided by `tau`". When the tub is full, `h` is large, so it drains fast; as it empties, it drains more slowly. `tau` is a time (seconds), so `h / tau` is metres per second: a speed, as a rate of change of a length must be. Prismal checks this too.

`tau` (the Greek letter τ) is the **time constant**: the time that sets how fast the whole story unfolds. A small `tau` empties the tub quickly, a large one slowly. It has a simple meaning you can check on the graph: after one `tau`, about 37% of the water is left; after two, about 14%; after three, about 5%. With `tau = 20 s`, a tub that starts at 50 cm is down to about 18 cm after 20 s. Many things in science change this way, with a time constant of their own: a cup of tea cooling, a capacitor discharging, a medicine leaving the body.

The lines that give rates are called **flows** and are written in a `flow` block.

## What simulating means

Knowing the rate is enough to find out everything that happens. Prismal starts from the starting values and takes many tiny steps forward in time. At each step it asks "how fast is each value changing right now?" and moves each value on by that much. Thousands of tiny steps trace out the whole story: the level falling quickly at first, then more and more slowly.

The part of Prismal that takes these steps is the **solver**. It chooses the step sizes itself to stay accurate, so a program never has to mention them. [Chapter 4](04-motion.md) explains the choices for readers who need control over them.

## Things that happen at a moment: events

Some things do not change smoothly. They happen at one instant: a ball hits the floor, a tank runs dry, a switch flips. These are **events**.

An event says when it happens and, usually, what changes at that instant:

```prismal
event nearly_empty on falling(h - 1 cm) {
  set empty = true
}
```

`falling(h - 1 cm)` means "when `h - 1 cm` crosses zero going down", that is, when the level drops below one centimetre. The lines in braces run at that instant. `empty` is a **discrete** value: one that only changes at events (here, a yes-or-no value, `true` or `false`). An event may not change the value its own trigger watches (`h` here) without saying what to do if it fires again and again; setting `empty` and letting the flow stop the draining avoids that question. [Chapter 5](05-events-and-modes.md) covers events.

## The whole bathtub

Here is the complete program. Every line is one of the ideas above.

```text
model Bathtub {
  param {
    h0:  Length = 50 cm     in [10 cm, 100 cm]   // the starting level
    tau: Time   = 20 s      in [5 s, 60 s]       // the time constant; a smaller drain makes it larger
  }
  state {
    h: Length = h0                                // the water level
  }
  discrete {
    empty: Boolean = false
  }
  flow {
    der(h) = if empty then 0 else -h / tau
  }
  event nearly_empty on falling(h - 1 cm) {
    set empty = true
  }
}

presentation BathLab for Bathtub {
  view level: plot(x: [0 s, 100 s], y: [0 m, 1 m], y_unit: cm) {
    series_plot(h every 0.5 s)
  }
  panel controls {
    slider(h0, range: [10 cm, 100 cm])
    slider(tau, range: [5 s, 60 s])
    label(h)
  }
  observe {
    h10     = h       at t0 + 10 s
    emptied = elapsed on nearly_empty
  }
}
```

Reading it from the top:

- `model Bathtub { ... }` starts the model. Everything inside the outer braces belongs to it.
- `h0` and `tau` are parameters, each with a type (`Length`, `Time`), a starting value and a range of allowed values (`in [10 cm, 100 cm]`). `//` starts a comment: a note for people, ignored by Prismal.
- `h` is the state: the level, starting at `h0`.
- `empty` is discrete: `false` until the event says otherwise.
- The flow gives the rate of change of `h`: draining while the tub is not empty, `0` (no change) once it is. `if ... then ... else ...` chooses between two values.
- The event marks the tub empty when the level reaches one centimetre, and from then on the flow says the level no longer changes.
- The presentation draws the level against time on a plot labelled in centimetres, adds two sliders and a live reading of `h`, and records two **observations** for testing: the level after 10 seconds, and the times at which the tub empties.

## Checking the model: tests

How do we know the program is right? We work out a few answers another way and ask Prismal to compare. This draining rule has a known formula: the level after time `t` is `h0 × e^(-t / tau)`, where `e` is a mathematical constant (about 2.718). With `h0 = 50 cm` and `tau = 20 s`:

- after 10 s the level is `50 cm × e^(-0.5)`, about `30.33 cm`;
- it reaches 1 cm when `e^(-t / 20 s) = 1/50`, at `t = 20 s × ln(50)`, about `78.24 s`.

A `run` states these expectations. `within` gives the allowed difference, because a simulation's tiny steps are very accurate but not perfect:

```cases
run drains of Bathtub with BathLab {
  until t0 + 100 s
  expect {
    h10        == 0.303265329856317 m within 1e-6 m
    emptied[1] == 78.2404601085629 s  within 1e-3 s
  }
}
```

`emptied[1]` is the first time the event happened. Running the cases prints `pass` or `FAIL` for each line, so a change that breaks the model is noticed at once. [Chapter 6](06-checks-and-tests.md) covers testing.

## Words used in this guide

| Word | Meaning |
|---|---|
| model | a description of what exists and how it behaves |
| presentation | a description of how a model is shown and what the learner may change |
| model value (binding) | any named value of a model: a parameter, state or a derived value |
| parameter | a setting fixed for a run (unless a slider changes it) |
| state | information the model carries forward from one moment to the next |
| time constant (`tau`, τ) | the time that sets how fast a value changes; after one time constant about 37% of a decaying value is left |
| derived value | a value computed from others, always up to date |
| discrete value | a value that changes only at events |
| quantity | a number with a unit, such as `3 m` |
| dimension | what a quantity measures: length, time, speed, ... |
| rate of change, derivative, `der` | how fast a value changes, per second |
| flow | a line giving a rate of change |
| simulation, run | working out what happens over time, step by step from the start |
| solver | the part of Prismal that takes the steps |
| event | something that happens at one instant |
| function | a rule that turns an input into an output, such as `sin` or `y(x) = 2 x` |
| point, vector | a place, and a step or direction between places ([chapter 3](03-space-and-vectors.md)) |
| `sin`, `cos` | functions that describe waves and circles; they repeat every full turn |
| `e^x`, exponential decay | the shape of "the more there is, the faster it shrinks" seen in the bathtub |
| interval `[a, b]` | all values from `a` to `b`; `[` and `]` include the end, `(` and `)` exclude it |
| tolerance (`within`) | how far a computed answer may be from the expected one |

## Where to go next

- [What goes where](00-structure.md) labels every block of a program by its purpose; [I want to...](14-i-want-to.md) answers "which words do I combine to get this?".
- [The tour](00-tour.md) builds a bouncing ball in four steps, up to a narrated lesson exported as a video.
- [Chapter 1](01-first-program.md) starts the step-by-step guide.
- [Chapter 13](13-common-mistakes.md) lists the errors met first and how to fix them.

## Exercises

1. Make the drain smaller by setting `tau = 40 s` in the run (`param { tau = 40 s }` before `until`). Does the tub empty sooner or later? Work out the new emptying time with `40 s × ln(50)` and change the expectation to match.
2. Add a derived value `water: Volume = h * 1.2 m * 0.6 m` for a tub 1.2 m long and 0.6 m wide, and show it with `label(water)`.
3. Write `der(h) = -h` instead of `-h / tau`. What does Prismal say, and why? (Hint: what are the units of each side?)
