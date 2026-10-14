# Why a Metal Spoon Gets Hot

A complete example: a spoon standing in hot water, as a field of temperature along its length, with a lab where the learner changes what the spoon is made of and a narrated lesson. Every program on this page is compiled and its cases run by `cargo test` (`crates/prismal-web/tests/examples.rs`).

**Goal.** A learner sees heat spread along a spoon from the hot end, finds that a copper spoon's handle gets hot within a minute while a steel spoon's handle hardly warms, and sees that each settles to a curve where the heat coming in balances the heat lost to the air.
**How it is built.**

- *the spoon*: a grid of 50 cells along its 10 cm, and the temperature above the room's as a field on it, `T(x in spoon)`;
- *the heat*: one flow, conduction along the spoon (`alpha * laplacian(T)`) less cooling to the air (`cool * T`); the end in the water is held at the water's temperature (`boundary`), the handle's end loses heat only to the air;
- *the material*: an enumeration parameter, `material: Material = copper`, and the thermal diffusivity `alpha` derived from it by `match`;
- *the lab*: the temperature along the spoon, the handle's temperature against time, a choice of material (`select`) and a button to start again;
- *the lesson*: a copper spoon until its handle is warm, a steel spoon for the same time, then the learner's choice.

## The model

```text
model Spoon {
  enum Material { copper, steel, glass }
  grid spoon(x: [0 m, 0.1 m], cells: 50)
  param {
    hot:      Temperature = 60 K        // the water, above the room
    cool:     Frequency   = 0.01 /s     // how fast the spoon loses heat to the air
    material: Material    = copper
  }
  state { T(x in spoon): Temperature = 0 K }
  derived {
    // How fast heat spreads through each material: its thermal diffusivity.
    alpha: Quantity<L^2/T> = match material {
      copper => 1.1e-4 m^2/s,
      steel  => 4e-6 m^2/s,
      glass  => 3.4e-7 m^2/s
    }
    handle: Temperature = T(0.099 m)
    middle: Temperature = T(0.05 m)
  }
  flow der(T) = alpha * laplacian(T) - cool * T
  boundary T { left = hot }
  event warm on rising(handle - 10 K)
}
```

- Temperatures are measured above the room's: `0 K` is room temperature, and a difference of 1 K is a difference of 1 °C.
- `boundary T { left = hot }` holds the end in the water at the water's temperature. The other end says nothing, so it is insulated: heat leaves it only through `cool * T`, as everywhere along the spoon.
- Steady state: where heat flowing in balances heat lost, `alpha T'' = cool T`, so `T(x) = hot cosh(m (L - x)) / cosh(m L)` with `m = sqrt(cool / alpha)` and `L = 10 cm`. For copper `m L = 0.95`: the handle settles at about two thirds of the water's temperature. For steel `m L = 5`: it settles below 1 K.
- `material` is a parameter whose values are the cases of `Material`: the learner chooses it, a case or a lesson sets it, and `restart` keeps it. `alpha` follows it.
- `warm` happens when the handle passes 10 K above the room, warm to the touch.

## The lab

```text
presentation SpoonLab for Spoon {
  title "Why a metal spoon gets hot"
  until t0 + 300 s
  view along: plot(x: [0 m, 0.1 m], y: [0 K, 70 K], x_unit: cm) {
    title("Temperature along the spoon")
    function_graph(T, color: red)
  }
  view over_time: plot(x: [0 s, 300 s], y: [0 K, 70 K]) {
    title("The handle")
    series_plot(handle every 1 s, color: orange)
  }
  panel controls {
    select(material)
    button(restart, label: "Into the water again")
    text("Handle: {handle:.1} above the room.")
  }
  layout row(column(along, over_time), controls)
  observe {
    warm_at = elapsed on warm
    handle_late = handle at t0 + 600 s
    middle_late = middle at t0 + 600 s
  }
}
```

- `select(material)` offers the cases of `Material`. A new choice changes the spoon at the instant shown: a copper spoon made steel halfway through keeps the heat it has and goes on more slowly. "Into the water again" (`restart`) starts again from a cold spoon of the material chosen.
- The left plot is the field itself, redrawn as it changes; the right plot is one point of it, the handle, against time.

```cases
run copper of Spoon with SpoonLab {
  until t0 + 600 s
  expect {
    handle_late == 40.27 K within 0.05 K      // 60 K cosh(m (L - x)) / cosh(m L) at x = 9.9 cm
    middle_late == 44.93 K within 0.05 K
    warm_at[1] in [8 s, 20 s]
  }
}

run steel of Spoon with SpoonLab {
  config { rtol = 1e-8 }
  param { material = steel }
  until t0 + 600 s
  expect {
    handle_late == 0.81 K within 0.05 K
    middle_late == 4.96 K within 0.05 K
    warm_at == []
  }
}
```

The reference values are the steady temperatures above, with `m = sqrt(cool / alpha)`: for copper `9.53 /m`, for steel `50 /m`. After 600 s the spoon is within a hundredth of a kelvin of its steady state; the grid's error is of the order of `(m h)^2 / 24` of the profile, `h = 2 mm` the cell width.

## The lesson

```text
presentation SpoonLesson for Spoon {
  title "Why a metal spoon gets hot"
  view along: plot(x: [0 m, 0.1 m], y: [0 K, 70 K], x_unit: cm) {
    function_graph(T, color: red)
  }
  panel notes {
    label(material)
    text("Handle: {handle:.1} above the room.")
  }
  layout row(along, notes)
  permit learner { timeline_controls }

  timeline {
    scene copper {
      beat intro { narrate "A copper spoon stands in hot water. The red line is its temperature, from the end in the water to the handle." for 8 s }
      beat spread { run rate 5 until warm; narrate "Heat spreads along the copper. The handle is warm in seconds." for 6 s }
      beat steady { run rate 20; narrate "Then the spoon settles. Heat still flows in, and the air takes it away as fast." for 7 s }
    }
    scene steel {
      beat change { intervene { set material = steel }; restart; narrate "Now a steel spoon, from cold." for 3.5 s }
      beat slow { run rate 20; narrate "Two minutes in, the steel handle is still cold. Heat spreads through steel thirty times more slowly than through copper." for 8.5 s }
    }
    scene yours {
      beat choose {
        explore limit 30 s keep material {
          select(material)
          button(restart, label: "Into the water again")
        } fallback {
          sequence { intervene { set material = glass }; wait 3 s }
        }
      }
      beat watch { restart; run rate 20; narrate "Glass is slower still. That is why a glass rod makes a cool handle." for 6 s }
    }
  }
}
```

- `run rate 5 until warm` plays five seconds of the model per second of the lesson until the handle is warm. `run rate 20` plays twenty seconds per second for as long as the beat lasts, here its narration: 7 s of narration show 140 s of the spoon.
- `intervene { set material = steel }` changes the material; `restart` then puts a cold spoon into the water, keeping the change (D-085).
- The explore beat keeps the learner's material (`keep material`), and the beat after it starts again with it. The video plays the fallback: a glass spoon.

```cases
run lesson of Spoon with SpoonLesson {
  expect {
    end of intro == 8 s within 1e-9 s
    start of change == end of steady within 1e-9 s
  }
}
```

## Exporting the lesson

```sh
cargo run --release -p prismal-media -- examples/heat.md SpoonLesson spoon.mp4 --speech system
```

The narrations are timed for the system voice: the exporter reports any narration whose voice lasts longer than its `for` (D-079). Without [ffmpeg](https://ffmpeg.org), give a folder name instead of `spoon.mp4` to get the frames as images.
