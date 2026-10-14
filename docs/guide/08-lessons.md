# 8. Lessons

A **lesson** is a presentation with a **timeline**: a sequence of scenes and beats that narrate, run the simulation, pause it, show and highlight things, and give the learner moments to explore. The timeline directs the simulation (play, pause, seek, change); the model never depends on it (D-009).

> **In plain words.**
> - A **timeline** is the script of a lesson, like the script of a short film. It is split into **scenes**, and each scene into **beats**: small steps such as "say this", "let the ball fall", "pause and highlight it".
> - **Narration** is the text of the lesson, shown as captions and, when a voice is chosen, spoken.
> - An **explore** beat hands control to the learner for a while: the lesson waits while they try things with the controls it offers.
> - **Seek** jumps to another moment of the simulation, like scrubbing a video.

## Timeline structure

```prismal
timeline {
  scene name {
    beat name { actions }
    beat name { actions }
  }
  scene name { ... }
}
```

Beats play one after another. Within a beat, the actions that direct the run (`run`, `hold`, `seek`, `reset`, `branch`, `intervene`, `request`) apply first, in written order, at the beat's start; then the other actions start together, and the beat ends when the last of them ends (D-033). `sequence { ... }` runs actions one after another inside a beat.

## Actions

| Action | Effect | Duration |
|---|---|---|
| `narrate "text" for 3 s` | a caption (and narration) | the given time, or a reading time |
| `run rate r` | simulation time advances at `r` seconds per presentation second (`0.25` is slow motion) | none |
| `run rate r until E` | as `run rate r`, and the beat waits until event `E` occurs | until `E` |
| `hold` | the simulation stops advancing; the picture stays | none |
| `seek τ` | shows simulation instant `τ` (`seek t0` goes back to the start) | none |
| `reset` | a new run from the start | none |
| `highlight name` | marks a named representation until the beat ends | none |
| `hide name` | stops showing a named representation | none |
| `in view { reps }` | adds representations to a view from now on | none |
| `show rep` | adds a representation over the presentation (for example a formula) | none |
| `intervene { set p = value }` | changes a parameter at the instant shown | none |
| `request E` | makes an `on request` event of the model happen now (D-027) | none |
| `wait 2 s` | waits | the given time |
| `explore limit L keep p { controls } fallback { actions }` | the learner's turn (below) | until the learner continues, or `L` |
| `wait learner [limit L] [fallback { actions }]` | a continue point (below) | until the learner continues, or `L` |
| `animate name opacity\|offset to v for d`, `release name`, `bind name for d` | animate a drawing, hold it, hand it back (below) | `d` |

The simulation keeps the rate set last: a beat without `run` or `hold` continues at the previous rate.

## Requested events

A lesson can only change the model in ways the model offers. A model offers an action with an event triggered `on request`:

```prismal
event drop on request {
  set pos = origin + (0 m, h)
  set vel = 0
  set airborne = true
}
```

`request drop` in a beat makes it happen at the instant shown, as if the learner had pressed a button. The handler is ordinary model code: it is checked, validated and logged like any event.

## Explore beats

```prismal
beat choose {
  hold
  explore limit 30 s keep h {
    slider(h, range: [1 m, 50 m])
  } fallback {
    sequence { intervene { set h = 20 m }; wait 2 s }
  }
}
```

- During an explore beat the learner may use the listed controls, and only those (D-025). Outside explore beats the learner controls the timeline and the view, never the model.
- The explore beat runs on a **branch** of the lesson's run: the learner's changes do not rewrite the lesson. `keep h` carries the learner's `h` into the rest of the lesson; without `keep`, the lesson returns to its own run where the beat started.
- The beat ends when the learner continues, or after `limit`.
- `fallback { ... }` is what plays instead in media without interaction (a video export, PK-9.10).

### Buttons in explore beats

An explore beat may offer a button among its controls. Pressing it requests the event on the learner's branch, at the instant shown:

```text
presentation DropTry for FreeFall {
  view scene: spatial(Plane, scale: 1 m -> 12 px, y: up) {
    axes
    marker(pos) as ball
  }
  timeline {
    scene try_it {
      beat first { run rate 1 until landed }
      beat yours {
        explore limit 20 s keep h {
          slider(h, range: [1 m, 50 m])
          button(drop, label: "Drop it")
        }
      }
    }
  }
}
```

```cases
run pressed of FreeFall with DropTry {
  learner {
    at 2 s: set slider h = 20 m
    at 3 s: press drop
    at 6 s: continue
  }
  expect {
    end of yours == 6 s exactly
  }
}
```

- `at 3 s: press drop` presses the button that requests `drop`. With `h` set to 20 m first, the ball falls from 20 m on the branch.
- Outside an explore beat, or when the beat offers no button for the event, a press is refused, as a slider's setting is.
- An explore beat may also offer `button(restart)`: it starts the learner's branch again from the beginning with the values the learner has set, so a learner can try a starting height or a number of molecules as often as they like. In a model with chance, `button(reseed)` does the same with another seed (chapter 15). A learner script presses them with `press restart` and `press reseed`. Other runtime controls (`reset`, `undo`, `redo`) belong to labs.

**Goal of this chapter's program.** A narrated lesson: a ball is dropped, the lesson pauses to ask how long the fall took, lets the learner choose a new height, and drops it again.
**How it is built.** The model only knows about falling (`state`, `flow`, an event `landed`) and an event `drop on request` that a lesson or button can ask for. Everything about the story is in the presentation's `timeline`: `scene`s made of `beat`s, each with actions (`narrate`, `run ... until landed`, `hold`, `highlight`, `explore` with a slider, `request drop`).

## A program

```text
space Plane = euclidean(2)

model FreeFall in Plane {
  param {
    g: Acceleration = 9.81 m/s^2
    h: Length       = 10 m     in [1 m, 50 m]
  }
  state {
    pos: Point            = origin + (0 m, h)
    vel: Vector<Velocity> = 0
  }
  discrete {
    airborne: Boolean = true
  }
  flow {
    der(pos) = if airborne then vel else 0
    der(vel) = if airborne then (0, -g) else 0
  }
  event landed on falling(pos.y) {
    set airborne = false
    set vel = 0
  }
  event drop on request {
    set pos = origin + (0 m, h)
    set vel = 0
    set airborne = true
  }
}
```

```text
presentation DropLesson for FreeFall {
  view scene: spatial(Plane, scale: 1 m -> 12 px, y: up) {
    axes
    marker(pos) as ball
  }
  permit learner { timeline_controls }
  observe {
    fall_times = elapsed on landed
  }

  timeline {
    scene first_drop {
      beat intro { narrate "A ball is dropped from 10 metres." for 3 s }
      beat fall  { run rate 1 until landed }
      beat look  {
        hold
        highlight ball
        narrate "How long did the fall take?" for 3 s
      }
      beat slow  {
        seek t0
        show formula("t", sqrt(2 * h / g), live: true)
        run rate 0.25 until landed
      }
    }
    scene your_turn {
      beat choose {
        hold
        explore limit 30 s keep h {
          slider(h, range: [1 m, 50 m])
        } fallback {
          sequence { intervene { set h = 20 m }; wait 2 s }
        }
      }
      beat again   { request drop; run rate 1 until landed }
      beat compare { narrate "Twice the height does not give twice the time." for 3 s }
    }
  }
}
```

Timing, from the fall time `T = sqrt(2 h / g) = 1.42784 s` for 10 m:

| Beat | Starts | Ends |
|---|---|---|
| `intro` | 0 s | 3 s (narration) |
| `fall` | 3 s | `3 + T` (the landing) |
| `look` | `3 + T` | `6 + T` (narration; the run holds) |
| `slow` | `6 + T` | `6 + 5T`: back to the start, then the fall at a quarter of real speed takes `4T` |
| `choose` | `6 + 5T` | when the learner continues, or 30 s later |
| `again` | | the drop from the chosen height, `sqrt(2 h / g)` |
| `compare` | | 3 s later |

## Testing a lesson

A case can play a lesson with a **learner script**: inputs at presentation instants.

```cases
run with_learner of FreeFall with DropLesson {
  learner {
    at 15 s: set slider h = 20 m
    at 16 s: continue
  }
  expect {
    end of fall   == 4.42784312292706 s within 1e-8 s
    end of slow   == 13.1392156146353 s within 1e-8 s
    end of choose == 16 s within 1e-9 s
    end of again  == 18.0192751093846 s within 1e-8 s
    fall_times[1] == 1.42784312292706 s within 1e-8 s
    fall_times[2] == 3.44711823231167 s within 1e-8 s    // T + sqrt(2 * 20 m / g)
  }
}

run without_learner of FreeFall with DropLesson {
  expect {
    end of choose == 43.1392156146353 s within 1e-8 s    // the 30 s limit
    fall_times[2] == 2.85568624585413 s within 1e-8 s    // dropped again from 10 m
  }
}
```

- `at 15 s: set slider h = 20 m` moves the slider for `h` at presentation time 15 s; `at 16 s: continue` ends the explore beat. `at τ: press E` presses an explore beat's button for `E`.
- `end of fall` and `start of fall` are beat times in presentation seconds.
- An input outside an explore beat is refused (the model is not the learner's to change then), and a case can check that the lesson is unaffected.
- The observation `fall_times` reads the lesson's run as it is at the end: the first landing, and the landing after the second drop. `elapsed` is simulation time since `t0`; the second drop happens at simulation time `T`, where the run was held.

In the player, a lesson has play, pause, a time scrubber and a bar of beats; during `choose` the slider appears and a Continue button ends the beat. Choosing the video medium plays the fallback instead.

## Continue points

`wait learner` waits for the learner to press Continue. It is the simplest way to let a learner think before the lesson goes on: no controls, no branch of the run.

```text
presentation DropQuiz for FreeFall {
  view scene: spatial(Plane, scale: 1 m -> 12 px, y: up) {
    axes
    marker(pos) as ball
  }
  timeline {
    scene quiz {
      beat ask   { narrate "Guess: how long will the fall take?" for 3 s; wait learner }
      beat fall  { run rate 1 until landed }
      beat think {
        hold
        narrate "Was your guess close?" for 3 s
        wait learner limit 20 s fallback { wait 2 s }
      }
      beat tell  { show formula("t", sqrt(2 * h / g), live: true) }
    }
  }
}
```

- A continue point opens when the other actions of its beat have ended: `ask` narrates for 3 s, then waits. Inside a `sequence` it waits at its place.
- It ends when the learner continues, or after `limit`. Without a limit it waits as long as the learner needs: the player stops there and shows Continue.
- In a video, `fallback { ... }` plays instead; a continue point without a fallback is left out.
- A continue the learner gives before the point opens is not taken by it.

```cases
run quick_learner of FreeFall with DropQuiz {
  learner {
    at 5 s: continue
    at 10 s: continue
  }
  expect {
    end of ask   == 5 s exactly
    end of fall  == 6.42784312292706 s within 1e-8 s    // 5 s + T
    end of think == 10 s exactly
  }
}

run no_learner of FreeFall with DropQuiz {
  expect {
    end of ask   == 3 s exactly                         // the narration; no wait recorded
    end of think == 27.4278431229271 s within 1e-8 s    // 3 s + T, 3 s of narration, the 20 s limit
  }
}
```

A case without a learner script shows how the lesson is laid out before the learner acts: a continue point with no limit takes no time. Continuing later only moves what follows.

An explore beat without a `limit` waits for the learner in the same way.

## Animation

Three actions animate the presentation itself, never the model (D-042):

| Action | Effect | Duration |
|---|---|---|
| `reveal fade [for d] [in view] { reps }` | shows representations, their opacity rising from 0 to 1 | `d`, 1 s by default |
| `reveal draw [for d] [in view] { reps }` | shows lines and paths drawn from start to end (points fade) | `d`, 1 s by default |
| `hide name for d` | fades a representation out | `d` |
| `camera view [to P] [zoom z] [for d]` | moves a view's camera to centre on `P` and magnify `z` times: a point in a spatial view, a pair of axis values in a plot (`camera graph to (5 s, 0 m) zoom 2`, D-075) | `d`, 1 s by default |

The camera's point is evaluated at every frame: `camera scene to pos` keeps following the ball after the move. `zoom 1` returns to the view's own scale. Animations are functions of presentation time, so a lesson looks the same every time it plays and in a video export.

```text
presentation DropMovie for FreeFall {
  view scene: spatial(Plane, scale: 1 m -> 12 px, y: up) {
    axes
  }
  timeline {
    scene only {
      beat appear   { reveal fade in scene { marker(pos) as ball } }
      beat ground   { reveal draw for 2 s in scene { polyline(origin + (-5 m, 0 m), origin + (5 m, 0 m)) } }
      beat close_up { camera scene to pos zoom 2 for 1.5 s }
      beat fall     { run rate 0.5 until landed }
      beat away     { camera scene zoom 1 for 1 s; hide ball for 1 s }
    }
  }
}
```

```cases
run movie of FreeFall with DropMovie {
  expect {
    end of appear   == 1 s exactly
    end of close_up == 4.5 s exactly
    end of fall     == 7.35568624585413 s within 1e-8 s    // 4.5 s + T / 0.5
    end of away     == 8.35568624585413 s within 1e-8 s
  }
}
```

## Moving, fading and handing back

`animate` changes a presentation property of a representation over time (D-068). It never changes the model: the ball's position is the model's, so an animation moves the drawing, not the ball.

| Action | Effect | Duration |
|---|---|---|
| `animate name opacity to x [for d]` | fades to opacity `x`, from 0 (invisible) to 1 | `d`, 1 s by default |
| `animate name offset to v [for d]` | moves the drawing by the vector `v` of the view's space, such as `(2 m, 0 m)` | `d`, 1 s by default |
| `release name` | the drawing stays as it is now, whatever the model does | none |
| `bind name [for d]` | the drawing follows the model again, gliding back over `d` | `d`, none by default |

Each animation starts from where the last one left the property, and the property keeps its last value: `animate ball offset to (0 m, 0 m)` moves the drawing back. Offsets and opacity apply to released drawings too.

`release` and `bind` hand a drawing over from the model to the lesson and back. Here the ball stays where it landed while the lesson rewinds, then glides up to the start before the slow replay:

```text
presentation DropRewind for FreeFall {
  view scene: spatial(Plane, scale: 1 m -> 12 px, y: up) {
    axes
    marker(pos) as ball
  }
  timeline {
    scene replay {
      beat fall   { run rate 1 until landed }
      beat land   {
        hold
        release ball
        animate ball opacity to 0.5 for 1 s
        narrate "It lands here." for 2 s
      }
      beat rewind { seek t0; bind ball for 1.5 s; animate ball opacity to 1 for 1.5 s }
      beat slow   { run rate 0.25 until landed }
    }
  }
}
```

```cases
run rewind of FreeFall with DropRewind {
  expect {
    end of land   == 3.42784312292706 s within 1e-8 s    // T + 2 s
    end of rewind == 4.92784312292706 s within 1e-8 s
    end of slow   == 10.6392156146353 s within 1e-8 s    // and the fall at a quarter speed, 4T
  }
}
```

Without `release`, `seek t0` would make the ball jump to the top at once. Run-directing actions such as `seek` apply first in their beat, so `release` goes in the beat before.

## Exporting a video

A lesson can be exported as a video file (D-052). The export opens the lesson in the video medium, so explore beats play their `fallback`, and draws every frame at a fixed rate:

```sh
cargo run --release -p prismal-media -- lesson.md DropMovie drop.mp4 --fps 30 --scale 2
```

- The first argument is the program (a source file, or a Markdown document whose `text` blocks form it); the second names the presentation.
- The output's extension chooses the format: `mp4`, `mov`, `mkv`, `webm` or `gif`. The encoding is done by ffmpeg, which must be installed (or named with `--encoder` or the `PRISMAL_FFMPEG` variable).
- `drop.png --at 4` writes the single frame at 4 s instead. Any other name is a directory: the frames as PNG images, the captions, and the ffmpeg command that makes a video of them.
- Narration appears as captions, drawn into the frames. `--captions track` puts them only in a subtitle track the viewer can turn off; `--captions both` does both. They are always written beside the video as `drop.vtt`. The events the video shows (`landed`, ...) are written as a descriptions track, `drop.descriptions.vtt`, for a player's `<track kind="descriptions">` and screen readers.
- An explore beat without a `fallback` cannot be shown in a video: the export says so, and plays on.
- A presentation without a timeline is recorded as a run from its start; `--until 10` sets its length in simulation seconds.
- `--dark` uses the dark theme.

Since frames depend only on the program and presentation time, exporting twice gives the same video.

## Narration sound

A program carries narration as text; sound is added by whatever plays it (D-053). Each narration is a **cue** named after its beat: `b3` for the narration of beat `b3`, and `b3.2`, `b3.3` for further narrations in the same beat. A voice is a folder of recordings named by cue (`b1.mp3`, `b3.wav` ...), or a speech synthesizer for cues without a recording.

```sh
cargo run --release -p prismal-media -- lesson.md DropLesson script.txt
cargo run --release -p prismal-media -- lesson.md DropLesson drop.mp4 --voice voice/ --speech system
```

- `script.txt` lists every cue with its name, start, length and text: what a narrator records.
- `--voice voice/` uses the recordings in `voice/`; `--speech system` speaks the cues that have none with the computer's voice. `--music theme.mp3` plays music under the narration.
- In the web player, the Voice menu offers synthesized speech, or recordings chosen as files.

The lesson's timing never depends on the sound: a narration lasts its `for d`, or its reading time. A recording longer than that is reported with the duration to write, for example `give the narration for 3.6 s or more`. So a lesson plays, and its cases pass, the same with or without a voice.

## A lab for the same model

The requested event also serves a presentation without a timeline: a `button` requests it whenever the learner presses it, at the instant shown.

```text
presentation DropLab for FreeFall {
  view scene: spatial(Plane, scale: 1 m -> 12 px, y: up) {
    axes
    marker(pos) as ball
    polyline(origin + (-2 m, 0 m), origin + (2 m, 0 m))
  }
  panel controls {
    slider(h, range: [1 m, 50 m])
    button(drop, label: "Drop again")
    table((pos.y, vel.y) every 0.25 s)
  }
  observe {
    drops = elapsed on drop
  }
}
```

- `button(drop, label: "Drop again")` requests the event `drop`, which must be declared `on request`. A button is an action of the learner like a slider: it takes effect at the instant shown and is logged as an intervention; undo removes it.
- A lab can offer its own runtime controls as buttons: `button(undo)`, `button(redo)`, `button(reset, label: "Start again")`. They act on the session, never on the model's rules; a lesson has none (its learner uses the player's controls).
- `table((pos.y, vel.y) every 0.25 s)` lists the height and vertical speed every quarter second up to the instant shown, one column per component of the tuple, with the time first.
- `polyline(P, Q, ...)` draws a path through points (here the ground); `polygon` closes it.

Changing `h` with the slider and pressing the button drops the ball from the new height: the handler of `drop` reads `h` when it happens.

```cases
run lab_drops of FreeFall with DropLab {
  until t0 + 10 s
  expect { drops == [] }
}
```

A case plays the model without a learner, so the button is never pressed. The player's tests press it (`crates/prismal-present/tests/labs.rs`).

## Unwrapping a circle

A geometry lesson: why the circumference of a circle is `2π r`. The circle rolls along a line without slipping, so each piece of its edge touches the line once. The part of the edge that has touched is laid on the line, and taken off the circle, so the circle unwinds onto the line; after one turn, the line is the whole edge.

```text
space Plane = euclidean(2)

model Circle in Plane {
  param {
    r: Length = 1 m   in [0.25 m, 2 m]
  }
  state {
    φ: Angle = 0                       // how far the circle has turned
  }
  discrete {
    rolling: Boolean = true
  }
  derived {
    unrolled: Length = r * φ           // the edge laid on the line so far
    centre:   Point  = origin + (unrolled, r)
    // The point of the edge that first touched the line, turned with the circle.
    first:    Point  = centre + (r * cos(-90 deg - φ), r * sin(-90 deg - φ))
  }
  flow {
    der(φ) = if rolling then 1 rev / (6 s) else 0
  }
  event done on rising(φ - 1 rev) {
    set rolling = false
  }
}
```

- `φ` is how far the circle has turned; it grows at one turn per 6 s until `done`, at one full turn (`1 rev`).
- Rolling without slipping: the centre has moved by `r φ`, the length of edge laid on the line (`unrolled`).
- `first` is the point of the edge that touched the line first, turned with the circle.

```text
presentation Unwrap for Circle {
  view line: spatial(Plane, scale: 1 m -> 60 px, y: up) {
    axes
    polyline(origin, origin + (unrolled, 0 m)) as laid
    arc(centre, r, from: -90 deg, to: 270 deg - φ) as edge
    segment(centre, first) as radius
    marker(first) as seam
  }
  panel numbers {
    label(unrolled)
  }
  observe {
    laid_length = unrolled on done
  }
  timeline {
    scene unwrap {
      beat meet {
        narrate "A circle of radius 1 metre. How long is the way around it?" for 5 s
      }
      beat roll {
        run rate 1 until done
      }
      beat laid {
        hold
        highlight laid
        narrate "Its edge, unwrapped, is a straight line: the circumference." for 5.2 s
      }
      beat measure {
        reveal draw for 3 s in line {
          segment(origin + (0 m, -0.4 m), origin + (2 * r, -0.4 m)) as d1
          segment(origin + (2 * r, -0.6 m), origin + (4 * r, -0.6 m)) as d2
          segment(origin + (4 * r, -0.4 m), origin + (6 * r, -0.4 m)) as d3
        }
        narrate "Three diameters, and a little more." for 3.4 s
      }
      beat name {
        show formula("C", 2π * r, live: true)
        narrate "The circumference is π diameters: C = 2 π r." for 5 s
      }
    }
  }
}
```

- `arc(centre, r, from: -90 deg, to: 270 deg - φ)` is the edge still on the circle: from the contact point at the bottom (`-90 deg`), counterclockwise, to `first`. At `φ = 0` it is the whole circle; after one turn it is empty.
- `polyline(origin, origin + (unrolled, 0 m))` is the edge laid on the line. It is a polyline, not a segment, only to be drawn in the same colour as the arc.
- In `measure`, three diameters are drawn under the line: they fall a little short of it. `name` shows `C = 2πr` with the value of `r`.

The laid length after one turn is `2π r`, whatever the radius:

```cases
run one_turn of Circle with Unwrap {
  expect {
    end of roll        == 11 s within 1e-9 s
    laid_length[1]     == 6.28318530717959 m within 1e-9 m    // 2π r
  }
}

run bigger of Circle with Unwrap {
  param { r = 1.5 m }
  expect {
    end of roll        == 11 s within 1e-9 s                 // one turn takes 6 s at any size
    laid_length[1]     == 9.42477796076938 m within 1e-9 m    // 2π × 1.5 m
  }
}
```

The end of `roll` is found by locating the event `done`, so it is checked within a tolerance.

## Color, size and shape

**Goal.** A lesson draws attention to parts of a figure: a side turns red, a point grows, a square becomes a circle, and a graph is looked at more closely.
**How it is built.** Each change is an `animate` action on a named representation, with the property to change: `color`, `line`, `scale` or `morph`. `ease` chooses how the change speeds up and slows down; `camera` on a plot zooms its axes (D-075).

| Action | Effect |
|---|---|
| `animate name color to red [for d]` | blends to a color of the palette (chapter 7) |
| `animate name line to dashed [for d]` | changes the line style, halfway through `d` (a style has no values between) |
| `animate name scale to 2 [for d]` | draws the shape twice as large about its centre (an arrow about its base); a marker's dot and an arrow's head grow too |
| `animate name morph to other [for d]` | changes the drawn shape into the shape of the representation `other`, point by point |
| `... ease linear`, `ease in`, `ease out`, `ease smooth` | at a constant rate; starting slowly; ending slowly; both (the default). Also after `reveal`, `hide ... for`, `camera` and `bind`: `camera scene zoom 2 for 1 s ease in` |

A morph's target is usually hidden: it is there to give a shape, and `hide` it in the first beat. Between two shapes of one form, corners are matched in the order they are written: list the target's corners in the same order as the source's, or the shape turns inside out on the way (a triangle whose corners are matched in mirrored order flattens into a line halfway). The morph ends with exactly the target's shape, so a square that becomes a circle is a circle afterwards, drawn with the square's own style.

```text
space Plane = euclidean(2)

model Figure in Plane {
  param { a: Real = 0.5 }
  derived { y(x: Real): Real = a * x * x }
}

presentation Shapes for Figure {
  view scene: spatial(Plane, scale: 1 m -> 40 px, y: up) {
    polygon(origin, origin + (2 m, 0 m), origin + (2 m, 2 m), origin + (0 m, 2 m)) as square
    circle(origin + (1 m, 1 m), 1 m) as round
    segment(origin, origin + (2 m, 0 m)) as base
    marker(origin + (1 m, 1 m)) as centre
  }
  view graph: plot(x: [-4, 4], y: [0, 8]) { function_graph(y) }
  timeline {
    scene look {
      beat intro  { hide round; narrate "A square." for 2 s }
      beat side   { animate base color to red for 1 s; animate base line to dashed for 1 s }
      beat point  { animate centre scale to 2 for 1 s ease out }
      beat change { animate square morph to round for 2 s ease linear }
      beat closer { camera graph to (2, 2) zoom 2 for 1 s }
    }
  }
}
```

```cases
run shapes of Figure with Shapes {
  expect {
    end of side   == 3 s exactly
    end of change == 6 s exactly
    end of closer == 7 s exactly
  }
}
```

## Mistakes

```error
// error: SX-E03
space Plane = euclidean(2)
model M in Plane {
  state { pos: Point = origin }
}
presentation L for M {
  view scene: spatial(Plane, scale: 1 m -> 10 px, y: up) { marker(pos) as ball }
  timeline {
    scene s { beat b { highlight bal } }
  }
}
```

A highlight names a representation of the presentation; `bal` is not one.

## Exercises

Solutions to the exercises not solved here are in [chapter 11](11-solutions.md).

1. Add a beat after `slow` that shows the velocity as an arrow from the ball (`in scene { arrow(vel, from: pos, scale: 1 m/s -> 5 px) }`) and replays the fall.
2. Make `choose` return to the lesson's own run (remove `keep h`). What does `again` drop from now? Write the case.
3. RP-08 in the reference programs is a complete narrated lesson with an explore beat, a requested relaunch, and cases A to D. Read it and predict `end of b8` for a learner who sets 30 degrees.

<details>
<summary>A solution to exercise 2</summary>

Without `keep h`, the lesson returns to its own run, where `h` is 10 m, so the second drop is again from 10 m whatever the learner chose:

```text
space Plane = euclidean(2)

model FreeFall in Plane {
  param {
    g: Acceleration = 9.81 m/s^2
    h: Length       = 10 m     in [1 m, 50 m]
  }
  state {
    pos: Point            = origin + (0 m, h)
    vel: Vector<Velocity> = 0
  }
  discrete { airborne: Boolean = true }
  flow {
    der(pos) = if airborne then vel else 0
    der(vel) = if airborne then (0, -g) else 0
  }
  event landed on falling(pos.y) { set airborne = false; set vel = 0 }
  event drop on request {
    set pos = origin + (0 m, h)
    set vel = 0
    set airborne = true
  }
}

presentation NoKeep for FreeFall {
  view scene: spatial(Plane, scale: 1 m -> 12 px, y: up) { marker(pos) as ball }
  observe { fall_times = elapsed on landed }
  timeline {
    scene only {
      beat fall   { run rate 1 until landed }
      beat choose {
        hold
        explore limit 30 s { slider(h, range: [1 m, 50 m]) }
      }
      beat again  { request drop; run rate 1 until landed }
    }
  }
}
```

```cases
run learner_ignored of FreeFall with NoKeep {
  learner {
    at 2 s: set slider h = 20 m
    at 3 s: continue
  }
  expect {
    end of again  == 4.42784312292706 s within 1e-8 s   // 3 s + T
    fall_times[2] == 2.85568624585413 s within 1e-8 s   // 2T: from 10 m again
  }
}
```

</details>
