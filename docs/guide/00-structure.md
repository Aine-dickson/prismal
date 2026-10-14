# What Goes Where

A Prismal program is a set of nested blocks, and each block accepts only certain things. This page is the map: one complete program in which every block is labelled with what it is for, then a table of what each block holds. Keep it open while reading the other chapters. When an example is hard to follow, find each line's block here and the question that block answers.

## Three questions, three places

Every program answers up to three questions, each in its own place:

| Question | Place | Never contains |
|---|---|---|
| What exists and how does it behave? | `model Name { ... }` | anything about drawing, sliders or narration |
| How is it shown, and what may the learner do? | `presentation Name for Model { ... }` | rules of behavior (rates, events) |
| Does it behave as expected? | `run name of Model with Presentation { ... }` | new behavior or drawing |

Reading an unfamiliar example becomes three steps. In the model, find what changes (`state`) and why (`flow`, `event`). In the presentation, find what is drawn (`view`) and what the learner can touch (`panel`). In the run, find what is being checked (`expect`).

## The map

The program below is complete and runs (it is compiled and tested with the guide). It drops a ball that bounces, draws it, plots its height, and checks the bounce times. The comment on each block says what the block is for; the comment on a line says what that line achieves.

```text
// A space: the flat plane the ball lives in. Written once, outside everything.
space Plane = euclidean(2)

// MODEL: what exists and how it behaves. Nothing here is about the screen.
model Drop in Plane {

  // PARAM: settings chosen before the run. The learner may change them with sliders.
  param {
    g: Acceleration = 9.81 m/s^2                       // gravity
    h: Length       = 10 m      in [1 m, 50 m]         // drop height; sliders cannot leave the range
    e: Real         = 0.8       in [0, 1]              // share of speed kept by a bounce
  }

  // STATE: values that change by themselves over time. Each has a starting value.
  state {
    pos: Point            = origin + (0 m, h)          // where the ball is
    vel: Vector<Velocity> = 0                          // how fast and which way it moves
  }

  // DISCRETE: values that change only at events (here a yes/no switch).
  discrete { resting: Boolean = false }

  // DERIVED: values computed from the others, always up to date.
  derived { height: Length = pos.y }

  // FLOW: how fast each state value changes. One line per state value.
  flow {
    der(pos) = if resting then 0 else vel               // position changes at the velocity
    der(vel) = if resting then 0 else (0 m/s^2, -g)     // velocity changes by gravity
  }

  // EVENT: something that happens at one instant, and what it changes.
  event bounce on falling(pos.y) {                      // when the height crosses 0 going down
    set vel = (vel.x, -e * vel.y)                       // reverse the fall, keeping the share e
  } zeno settle {                                       // when bounces crowd together: come to rest
    set pos = origin + (pos.x, 0 m)
    set vel = 0
    set resting = true
  }

  // EVENT ON REQUEST: happens when a button or a lesson asks for it.
  event again on request {
    set pos = origin + (0 m, h)
    set vel = 0
    set resting = false
  }
}

// PRESENTATION: how the model is shown and what the learner may do. `for Drop` names the model.
presentation Lab for Drop {

  // VIEW (spatial): a drawing of the plane. Inside go representations: things drawn.
  view scene: spatial(Plane, scale: 1 m -> 20 px, y: up) {
    trace(pos every 0.02 s) as path                     // the path so far
    marker(pos) as ball                                 // the ball itself
  }

  // VIEW (plot): a graph with axes. Inside go representations drawn against the axes.
  view heights: plot(x: [0 s, 10 s], y: [0 m, 12 m], follow: (x, y)) {
    series_plot(height every 0.02 s)                    // height against time
  }

  // PANEL: a place for controls and readouts.
  panel controls {
    slider(h, range: [1 m, 50 m])                       // a control for a parameter
    slider(e, range: [0.1, 1], step: 0.05)
    button(again)                                       // a control that requests an event
    label(height)                                       // a live readout
  }

  // PERMIT: what the learner may do besides the controls.
  permit learner { timeline_controls; zoom; pan }

  // OBSERVE: values recorded for tests (and shown as lists in the player).
  observe { bounces = elapsed on bounce }               // the times of every bounce
}

// RUN: a test case. It runs the model with a presentation and compares observations.
run two_bounces of Drop with Lab {
  until t0 + 5 s
  expect {
    bounces[1] == 1.42784312292706 s within 1e-9 s      // sqrt(2 h / g)
    bounces[2] == 3.71239211961037 s within 1e-9 s      // 2.6 times that: up and down at 0.8 of the speed
  }
}
```

## What each block holds

| Block | Lives in | Holds | Example line |
|---|---|---|---|
| `space` | the top level | nothing: one line | `space Plane = euclidean(2)` |
| `model Name [in Space] { }` | the top level | the blocks below, down to `object` | |
| `const { }` | a model | values that never change | `c: Velocity = 299792458 m/s` |
| `param { }` | a model | settings: `name: Type = value`, with an optional range | `h: Length = 10 m in [1 m, 50 m]` |
| `state { }` | a model | values that change over time: `name: Type = start` | `x: Length = 0 m` |
| `discrete { }` | a model | values changed only by events | `on: Boolean = false` |
| `derived { }` | a model | values and functions computed from others | `speed: Velocity = \|vel\|` |
| `unknown { }` | a model | values the equations determine together: `name: Type = guess` | `I: Current = 1 mA` |
| `flow { }` | a model | rates of change: `der(x) = ...` | `der(x) = v` |
| `event name on trigger { }` | a model | operations: `set`, `emit` | `set vel = 0` |
| `process name { }` | a model | flows and events grouped under a name | `flow der(v) += -(k / m) * x` |
| `equation`, `constraint` | a model | stated relations, solved when they read an unknown; checked conditions, or kept true with `policy enforce` | `constraint rod: \|bob - pivot\| == L policy enforce` |
| `presentation Name for Model { }` | the top level | the blocks below | |
| `view name: spatial(...) { }` | a presentation | representations drawn in the plane | `marker(pos) as ball` |
| `view name: plot(...) { }` | a presentation | representations drawn against axes | `series_plot(x every 0.02 s)` |
| `panel name { }` | a presentation | controls and readouts | `slider(k, range: [1 N/m, 20 N/m])` |
| `title "..."` | a presentation | the page's title | `title "Dropping a ball"` |
| `title("...")`, `text("...")` | a view or panel | its caption; a sentence with `{values}` | `text("Height: {pos.y}", when: airborne)` |
| `permit learner { }` | a presentation | what the learner may do | `zoom; pan` |
| `observe { }` | a presentation | recorded values: `name = expression schedule` | `x5 = x at t0 + 5 s` |
| `timeline { scene { beat { } } }` | a presentation | a lesson: narration, running, pausing | `narrate "..." for 3 s` |
| `layout` | a presentation | where views go on the page | `layout row(scene, controls)` |
| `run name of Model with Presentation { }` | the top level | `param`, `config`, `until`, `learner`, `expect` | `until t0 + 10 s` |
| `expect { }` | a run | comparisons with tolerances | `x5 == 0.1 m within 1e-6 m` |

Some words look alike but belong to different places:

- `param { k = 2 N/m }` inside a **run** changes a setting for that test only; `param { k: ... = ... }` inside a **model** declares it. The run's form has no type.
- `observe` (in a presentation) records values; `expect` (in a run) checks them.
- A control (`slider`) belongs in a panel or view; the value it moves (`param`) belongs in the model. The slider names the parameter: `slider(h, ...)` moves `h`.
- `set x = ...` happens at an instant and belongs in an event. `der(x) = ...` happens all the time and belongs in `flow`.

When something is in the wrong block, the error message says where it belongs:

```prismal
model M {
  param { k: Real = 1 }
  slider(k, range: [0, 2])
}
```

reports `SX-E02: expected a declaration (...), found slider. A control belongs in a presentation, inside panel name { ... } (or inside a view's braces)`.

## Next

[I want to...](14-i-want-to.md) lists common goals and the combination of words that achieves each, with the block each word goes in.
