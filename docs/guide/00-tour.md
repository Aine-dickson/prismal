# 0. A Tour

This page builds one program in four steps: a model of a thrown ball, a test that checks it, a lab where a learner can change it, and a narrated lesson that can be exported as a video. Each step adds to the program before it. The chapters that follow explain every form used here in detail; this page shows what they are for.

**For** readers who know what a rate of change is and want to see what Prismal does before learning it form by form. New to simulation? Read [Start here](00-start-here.md) first; its ideas are assumed here.

You need the repository built (`cargo test` passes, see the [README](../../README.md)). Save the blocks of each step one after another in a file, `throw.prismal`, and run its cases with:

```sh
cargo run -p prismal-web --bin prismal -- cases throw.prismal
```

## Step 1: a model

A model says what exists and how it behaves. Nothing in it is about drawing.

```text
space Plane = euclidean(2)

model Throw in Plane {
  param {
    g: Acceleration = 9.81 m/s^2
    e: Real         = 0.8   in [0, 1]     // the share of upward speed a bounce keeps
  }
  state {
    pos: Point            = origin + (0 m, 1 m)
    vel: Vector<Velocity> = (3 m/s, 4 m/s)
  }
  discrete { resting: Boolean = false }
  flow {
    der(pos) = if resting then 0 else vel
    der(vel) = if resting then 0 else (0 m/s^2, -g)
  }
  event bounce on falling(pos.y) {
    set vel = (vel.x, -e * vel.y)
  } zeno settle {
    set pos = origin + (pos.x, 0 m)
    set vel = 0
    set resting = true
  }
  event top on falling(vel.y)
}
```

- `space Plane = euclidean(2)` declares the plane the ball moves in; `in Plane` makes it the model's space.
- `param` values are fixed for a run unless something changes them; `e` has a range, so a value outside `[0, 1]` is refused wherever it comes from.
- `state` values change over time, by the `flow`: `der(pos) = vel` reads "the rate of change of the position is the velocity".
- Every value has a unit, and the units are checked: `der(pos) = g` would be refused, because a position cannot change at the rate of an acceleration (chapter 2).
- `discrete { resting: Boolean = false }` is state that changes only at events; the flows stop the ball while it is `true`.
- `event bounce on falling(pos.y)` happens when the height crosses zero going down; its handler reverses the vertical velocity, keeping the share `e`. A bouncing ball makes ever shorter bounces, infinitely many in a finite time; `zeno settle { ... }` says what happens when they crowd together: the ball comes to rest on the ground (chapter 5).
- `event top on falling(vel.y)` has no handler: it only marks the instants where the ball is highest.

## Step 2: a test

A test runs the model and compares what it observes with values computed independently. A presentation chooses what to observe; this one shows nothing.

```text
presentation Measure for Throw {
  observe {
    bounces = elapsed on bounce
    apex    = pos.y on top
    landing = pos.x on bounce
  }
}
```

```cases
run first_bounces of Throw with Measure {
  until t0 + 3 s
  expect {
    bounces[1] == 1.016131429618 s within 1e-9 s     // (4 + sqrt(16 + 2 g)) / g
    apex[1]    == 1.815494393476 m within 1e-9 m     // 1 m + (4 m/s)^2 / (2 g)
    landing[1] == 3.048394288853 m within 1e-9 m     // 3 m/s times the first time
    bounces[2] == 1.989546202225 s within 1e-9 s     // up again at 0.8 of the speed
  }
}

run softer of Throw with Measure {
  until t0 + 3 s
  param { e = 0.5 }
  expect { bounces[2] == 1.624515662498 s within 1e-9 s }
}
```

- `elapsed on bounce` is the list of times since the start at which `bounce` happened; `[1]` is the first. `pos.y on top` is the height at each `top`.
- `run first_bounces of Throw with Measure` runs the model for 3 s and checks each expectation within its tolerance. `param { e = 0.5 }` changes a parameter for one case.
- The expected numbers come from the formulas in the comments, not from running the program: a test that copies the program's output tests nothing (chapter 6).

## Step 3: a lab

A lab draws the model and lets the learner act on it, in the web player.

```text
presentation Lab for Throw {
  view scene: spatial(Plane, scale: 1 m -> 60 px, y: up) {
    axes
    trace(pos every 0.02 s, color: blue) as path
    marker(pos) as ball
    arrow(vel, from: pos, scale: 1 m/s -> 15 px) as velocity
  }
  panel controls {
    slider(e, range: [0.1, 1], step: 0.05)
    formula("speed", |vel|, live: true)
  }
  layout row(scene, controls)
  permit learner { timeline_controls; zoom; pan }
  observe { height = pos.y live }
}
```

- `view scene: spatial(...)` draws the plane at 60 pixels per metre, `y` up. `trace` draws the path, `marker` the ball, `arrow` its velocity at 15 pixels per metre per second.
- `slider(e, ...)` lets the learner change `e` at the instant on screen; the rest of the run is computed again from there. A slider can only change what the model allows to change: parameters, by default (chapter 7).
- `layout row(scene, controls)` puts the controls beside the scene.
- `permit learner { ... }` lets the learner play, pause and scrub the run, and zoom and pan the view.

To see it, build and serve the web player (README, "See it in the browser"), open the Source tab, paste the program and press Compile and open. Choose `Lab` in the presentation list.

## Step 4: a lesson

A lesson is a presentation with a timeline: beats that narrate, run and pause the model, and show things. The model does not know about it.

```text
presentation Story for Throw {
  view scene: spatial(Plane, scale: 1 m -> 60 px, y: up) {
    axes
    trace(pos every 0.02 s) as path
    marker(pos) as ball
  }
  observe { bounces = elapsed on bounce }
  timeline {
    scene throw {
      beat intro  { narrate "A ball is thrown up and to the right." for 3 s }
      beat flight { run rate 1 until bounce }
      beat look   {
        hold
        highlight ball
        narrate "A bounce keeps 80 percent of the upward speed." for 4 s
      }
      beat again  { run rate 0.5 until bounce }
    }
  }
}
```

```cases
run story of Throw with Story {
  expect {
    end of flight == 4.0161314296178 s within 1e-9 s     // 3 s of narration, then the flight
    end of look   == 8.0161314296178 s within 1e-9 s
    end of again  == 9.962960974832 s within 1e-9 s     // the next bounce at half speed
  }
}
```

- `narrate "..." for 3 s` shows a caption (and speaks it, when a voice is chosen) for 3 s.
- `run rate 1 until bounce` plays the model at real speed until the bounce; `hold` stops it; `run rate 0.5` plays at half speed.
- The case checks the lesson's timing in presentation seconds: the second bounce comes `1.9895 - 1.0161 = 0.9734 s` of simulation after the first, which takes `1.9468 s` at half speed.

Export it as a video, with the captions drawn in and spoken by the system's voice (needs [ffmpeg](https://ffmpeg.org)):

```sh
cargo run --release -p prismal-media -- throw.prismal Story throw.mp4 --speech system
```

## Where next

- [Chapter 1](01-first-program.md) starts the guide properly: models, parameters and a first plot.
- [Recipes](12-recipes.md) answer "how do I ..." with short programs.
- [Common mistakes](13-common-mistakes.md) lists the errors you will meet first, and how to fix them.
- [Reference](10-reference.md) has every form on one page.
