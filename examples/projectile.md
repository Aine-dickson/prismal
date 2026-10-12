# Which Angle Throws Farthest?

A complete example: a model of a thrown ball, a lab to play with it, and a narrated lesson that can be exported as a video. Every program on this page is compiled and its cases run by `cargo test` (`crates/prismal-web/tests/examples.rs`).

**Goal.** A learner discovers that, without air resistance, a throw at 45 degrees goes farthest, and that two angles adding up to 90 degrees land at the same place.
**How it is built.**

- *the ball and its flight*: a model with the launch speed and angle as parameters, position and velocity as state, a flying/landed mode, and an event `landed`;
- *throwing again with a new angle*: an event `relaunch` requested by a button or by the lesson;
- *remembering the result*: discrete state `last`, set when the ball lands, and a count of throws;
- *the lab*: sliders with words at their ends, a Throw button, a path drawn as it goes, a height plot that grows with the data, live texts and the range formula;
- *the lesson*: three throws (30, 60 and 45 degrees) with narration, then the learner's own.

## The model

```text
space Plane = euclidean(2)

model Throw in Plane {
  param {
    g:     Acceleration = 9.81 m/s^2
    speed: Velocity     = 20 m/s   in [5 m/s, 30 m/s]     symbol "v"
    angle: Angle        = 30 deg   in [5 deg, 85 deg]     symbol "θ"  unit deg
  }
  state {
    pos: Point            = origin
    vel: Vector<Velocity> = speed * (cos(angle), sin(angle))
  }
  discrete {
    flying: Boolean = true
    last:   Length  = 0 m        // where the last throw landed
    throws: Real    = 1
  }
  derived {
    reach: Length = speed^2 * sin(2 * angle) / g     // the range the formula predicts
  }
  flow {
    der(pos) = if flying then vel else 0
    der(vel) = if flying then (0 m/s^2, -g) else 0
  }
  event landed on falling(pos.y) {
    set flying = false
    set vel = 0
    set last = pos.x
  }
  /// Throws again from the origin, with the speed and angle set now.
  event relaunch on request {
    set pos = origin
    set vel = speed * (cos(angle), sin(angle))
    set flying = true
    set throws = throws + 1
  }
}
```

- `angle` has the display unit `deg`: it is stored in radians, and every slider, label and text shows degrees.
- `reach` is the range the formula `R = v² sin(2θ) / g` predicts. The simulation does not use it; the lab shows it beside the measured landing point, so the learner can compare the two.
- At launch `pos.y` is 0 and rising, which is not a falling crossing, so `landed` does not happen at the start.

## The lab

```text
presentation ThrowLab for Throw {
  title "Which angle throws farthest?"
  view field: spatial(Plane, scale: 1 m -> 12 px, y: up) {
    title("The throw")
    axes
    trace(pos every 0.05 s, color: gray) as path
    marker(pos, label: none) as ball
    arrow(vel, from: pos, scale: 1 m/s -> 2 px, when: flying) as velocity
  }
  view height: plot(x: [0 s, 4 s], y: [0 m, 10 m], follow: (x, y)) {
    title("Height over time")
    series_plot(pos.y every 0.02 s)
  }
  panel controls {
    title("Your throw")
    slider(angle, range: [5 deg, 85 deg], ends: ("flat", "steep"))
    slider(speed, range: [5 m/s, 30 m/s], ends: ("gentle", "hard"))
    button(relaunch, label: "Throw")
    text("In the air, {pos.x:.1} from the start.", when: flying)
    text("Landed {last:.1} away (throw {throws}).", when: not flying)
    text("The formula predicts {reach:.1}.")
    formula("R", speed^2 * sin(2 * angle) / g, live: true)
  }
  layout row(field, column(height, controls))
  permit learner { timeline_controls; zoom; pan }
  observe {
    range  = pos.x   on landed
    flight = elapsed on landed
  }
}
```

- The path is drawn as the ball flies (`trace`); throwing again keeps the earlier paths, so throws can be compared side by side.
- `when: flying` shows the velocity arrow only while the ball is in the air; the two texts take turns in the same way.
- `{last:.1}` shows the value with one decimal, `35.3 m`, which is as exact as a learner needs; the cases check the full precision.
- `label: none` keeps the marker unlabelled: it is named `ball` so that the lesson can highlight it, not to write "ball" on the field.
- Throwing again lifts the pen: each throw's path is drawn on its own, with no line back to the start.
- The height plot starts with 4 s and 10 m and grows with the data (`follow`), so a long, high throw stays in view.

The cases check the landing point against the formula, for three angles:

```cases
run thirty of Throw with ThrowLab {
  until t0 + 5 s
  expect {
    range[1]  == 35.3119430697019 m within 1e-8 m     // v² sin(60°) / g
    flight[1] == 2.03873598369011 s within 1e-9 s     // 2 v sin(30°) / g
  }
}

run sixty of Throw with ThrowLab {
  param { angle = 60 deg }
  until t0 + 5 s
  expect { range[1] == 35.3119430697019 m within 1e-8 m }     // the same as 30°
}

run fortyfive of Throw with ThrowLab {
  param { angle = 45 deg }
  until t0 + 5 s
  expect { range[1] == 40.7747196738022 m within 1e-8 m }     // the farthest
}
```

## The lesson

```text
presentation ThrowLesson for Throw {
  title "Which angle throws farthest?"
  view field: spatial(Plane, scale: 1 m -> 12 px, y: up) {
    title("Three throws at 20 m/s")
    axes
    trace(pos every 0.05 s, color: gray) as path
    marker(pos, label: none) as ball
  }
  panel notes {
    title("Result")
    text("Angle {angle:.0}: landed {last:.1} away.", when: not flying)
  }
  permit learner { timeline_controls }
  observe { landings = pos.x on landed }

  timeline {
    scene first {
      beat intro  { narrate "Which angle throws a ball farthest? Every throw starts at 20 metres per second." for 4 s }
      beat throw1 { run rate 1 until landed }
      beat look1  { hold; highlight ball; narrate "At 30 degrees it lands 35 metres away." for 3 s }
    }
    scene steeper {
      beat aim2   { intervene { set angle = 60 deg }; request relaunch; narrate "Now steeper: 60 degrees." for 2 s }
      beat throw2 { run rate 1 until landed }
      beat look2  { hold; highlight ball; narrate "The same distance. A steep throw goes higher but moves forward slowly." for 4 s }
    }
    scene best {
      beat aim3   { intervene { set angle = 45 deg }; request relaunch; narrate "Halfway between: 45 degrees." for 2 s }
      beat throw3 { run rate 1 until landed }
      beat look3  { hold; highlight ball; animate ball scale to 2 for 1 s; narrate "45 degrees goes farthest: almost 41 metres." for 3 s }
    }
    scene yours {
      beat choose {
        hold
        animate ball scale to 1 for 0.5 s
        explore limit 30 s keep angle {
          slider(angle, range: [5 deg, 85 deg], ends: ("flat", "steep"))
        } fallback {
          sequence { intervene { set angle = 50 deg }; wait 2 s }
        }
      }
      beat throw4 { request relaunch; run rate 1 until landed }
      beat close  { narrate "Angles that add up to 90 degrees land at the same place." for 4 s }
    }
  }
}
```

- Each throw beat runs the model until `landed`, at real speed; the beats around it hold the run while the narration speaks.
- `intervene` sets the angle for the next throw and `request relaunch` throws again from the origin. Both happen at the held instant, before the run goes on.
- The explore beat hands the angle slider to the learner for up to 30 s. A video has no learner, so it plays the fallback: 50 degrees.

The lesson's timing, from the flight times `T(θ) = 2 v sin θ / g`: 4 s of narration, the 30° flight, 3 s, 2 s, the 60° flight, 4 s, 2 s, the 45° flight, 3 s.

```cases
run lesson of Throw with ThrowLesson {
  expect {
    end of throw1 == 6.03873598369011 s within 1e-8 s
    end of throw2 == 14.5699302906603 s within 1e-8 s
    end of look3  == 26.4531383688929 s within 1e-8 s
    landings[1] == 35.3119430697019 m within 1e-8 m
    landings[2] == 35.3119430697019 m within 1e-8 m
    landings[3] == 40.7747196738022 m within 1e-8 m
  }
}
```

## Exporting the lesson

```sh
cargo run --release -p prismal-media -- examples/projectile.md ThrowLesson throw.mp4 --speech system
```

The video plays the explore beat's fallback (a throw at 50 degrees, landing about 40.2 m away). Without [ffmpeg](https://ffmpeg.org), give a folder name instead of `throw.mp4` to get the frames as images.
