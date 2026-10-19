# 7. Presentations

A presentation turns a model into something a learner sees and handles. It never changes the model's meaning: it observes values, draws them, and offers the learner a controlled set of actions, each of which the model validates. One model may have several presentations.

> **In plain words.**
> - A **view** is an area of the screen: a **spatial** view draws the model's space like a map, a **plot** draws values against axes, a **panel** holds controls and readouts.
> - A **representation** is one thing drawn or shown: a dot, an arrow, a curve, a slider, a number.
> - A **control** lets the learner change a value: a slider, a typed number, a switch, a button. An **inverse** lets the learner change a value by dragging the drawing itself.
> - A **lab** is a presentation without a timeline: the learner explores freely. A presentation with a timeline is a lesson (chapter 8).

## Structure

```prismal
presentation Name for Model {
  view name: spatial(Space, scale: 1 m -> 40 px, y: up) { representations }
  view name: plot(x: [a, b], y: [c, d]) { representations }
  panel name { representations }
  permit learner { zoom; pan; timeline_controls }
  layout row(name, column(name, name))
  observe { name = expression schedule }
  timeline { ... }                                  // chapter 8
}
```

| Part | Purpose |
|---|---|
| `view ...: spatial(...)` | a region showing a space, with a scale from lengths to pixels and an orientation (`y: up` or `y: down`) |
| `view ...: plot(...)` | a region with two axes; each range gives its axis a dimension (`[0 s, 5 s]` is a time axis) |
| `panel ...` | a region without coordinates, for controls, formulas and labels |
| `layout ...` | where the views go on the page: `row(...)` side by side, `column(...)` one below the other, nested |
| `permit learner { ... }` | what the learner may do with the views and the timeline: `zoom`, `pan`, `timeline_controls` |
| `observe { ... }` | named values recorded for tests and for the Observations tab |

## Representations

| Representation | Shows | Where |
|---|---|---|
| `marker(P)`, `marker(at: (x, y))` | a point | spatial (a `Point`), plot (a pair in the axes' dimensions) |
| `arrow(v, from: P [, scale: 1 m/s -> 4 px])` | a vector drawn from a point | spatial |
| `segment(P, Q)` | a straight segment | spatial |
| `polyline(P, Q, ...)`, `polygon(P, Q, ...)` | an open or closed path through points | spatial |
| `circle(P, r)`, `ellipse(P, a, b [, rotate: θ])` | a circle of radius `r`, an ellipse with radii `a` and `b` turned by `θ`, around `P` | spatial |
| `arc(P, r, from: θ1, to: θ2)` | part of the circle of radius `r` around `P`, from angle `θ1` to `θ2` | spatial |
| `trace(P every Δ)` | the path of a point up to the instant shown | spatial |
| `function_graph(f)` | the graph of a function `Real -> Real` over the plot's `x` range | plot |
| `series_plot(e every Δ)` | a value against elapsed time | plot with a time `x` axis |
| `axes`, `grid` | coordinate reference | spatial |
| `label(e)` | a value as text | any |
| `text("... {e} ...")` | an author's text, with values in braces (see Text and titles) | any; `at: P` in spatial and plot views |
| `title("...")` | the caption of its view or panel | any |

A marker, an arrow or a graph is labelled with its name (`as ball`) or its source. `label: "Ball"` writes another label, and `label: none` draws none: a representation named only so that a lesson can highlight it need not show its name. Its text alternative keeps the name.
| `formula(f [, live: true])` | the definition of a derived binding or function, typeset | any |
| `formula("R", expression [, live: true])` | a labeled expression, `R = ...`, typeset | any |
| `equation(name [, live: true])` | a model equation, typeset, with its symbols' current values | any |
| `table(e every Δ)` | rows of sampled values; a tuple `e` gives one column per component | any |
| `button(E [, label: "..."])` | requests the `on request` event `E` when pressed | any |
| `button(E(v) [, label: "..."])` | requests `E` with the value `v`, for an event that takes one (chapter 5) | any |
| `button(reset)`, `button(undo)`, `button(redo)` | a lab's runtime controls: start the run again, take back the last action, put it back | any |
| `slider(p, range: [a, b] [, step: s] [, ends: ("low", "high")])` | a control for a parameter; `ends` puts a word at each end | any |
| `number_input(p [, range: [a, b]])` | a typed value for a parameter | any |
| `toggle(p)` | a switch for a Boolean parameter | any |
| `select(p)` | a choice of one case of an enumeration parameter (`material: Material = copper`), listed by name | any |
| `group(at: P, rotate: θ, scale: k) { ... }` | its members, placed, turned and scaled together (see Groups) | spatial |

`as name` after a representation names it (`marker(pos) as ball`), for timelines and for readers.

A `marker` is a dot of fixed size on the screen. A `circle`, `ellipse` or `arc` has radii in the model's units, so it grows when the view is zoomed. Their angles are measured counterclockwise from the space's `x` axis: `arc(P, r, from: 0 deg, to: 90 deg)` is the quarter from the right of `P` to above it.

### Color and line

A drawn representation may say how it looks: `color:` names a color, and `line:` how a stroke is drawn.

```prismal
segment(origin, pos, color: blue, line: dashed) as rod
trace(pos every 0.1 s, color: green, line: dotted)
marker(pos, color: red) as ball
```

- Colors are named: `red`, `orange`, `yellow`, `green`, `teal`, `blue`, `purple`, `pink`, `gray`, `ink`. Each medium draws a name with a value that suits its theme, so a program looks right on a light page, a dark one, or in a video; `ink` is the color of text.
- Lines are `solid`, `dashed` or `dotted`. `width: 2` draws the strokes twice as thick as the kind's own (a positive number, a multiple of the usual width).
- Markers take a color; arrows, segments, polylines, polygons, circles, ellipses, arcs, traces, graphs, series and histograms take a color, a line and a width. Other kinds, and groups, take neither.
- Without them, each kind is drawn its own way (arrows take turns through a few colors, a trace is dashed).
- Color never carries meaning alone: what a color distinguishes is also named (`as rod`) or labeled, and the text alternative does not mention it.

Every representation has a **text alternative** generated from what it shows ("slider for θ = 45 deg, from 5 deg to 85 deg"), for screen readers and the player's Description tab.

## Controls and inverses

A control changes a binding; so can a drawn representation that declares an **inverse**:

```prismal
marker(at: (x, 0)) { on drag as p { propose x = p.x * 1 m } }
arrow(u, from: A)  { on drag head as h { propose u = h - A } }
```

- `on drag as p` gives the drag's position as `p`: a `Point` in a spatial view, a pair `(p.x, p.y)` in a plot view, in the axes' dimensions.
- `propose x = ...` says which binding the gesture changes and how. The target must be something the learner may change.
- Proposals go through the model's checks. While dragging, an invalid proposal (outside a range, violating a `reject` constraint) is shown as invalid with the reason; releasing commits the last valid one. Each commit is one intervention, which undo removes.
- While the learner drags, a running model is held at the instant shown and goes on after the release (drag mode `hold`). `on drag live as p` keeps it running instead, so the learner can steer it (see Steering a running model, below).

What the learner may change: **parameters**, by default; **state**, only when the model declares it `intervenable` (`state { x: Length = 0 m intervenable }`); never constants, derived bindings or inputs. A control on anything else is rejected when the presentation is checked (PK-E03).

## A program

**Goal.** A learner aims a cannon: changes speed and angle, switches to the Moon's gravity, and drags the landing point to find the speed that reaches it.
**How it is built.** Every control is a different kind for a different value type: `number_input` for a speed, `slider` for an angle, `toggle` for a Boolean. The path is a derived function drawn by `function_graph`; the landing point is a `marker` whose drag `propose`s a new speed (computed back from where it was dropped). `formula(..., live: true)` shows the range formula with current numbers.

A cannon on flat ground, with gravity switchable to the Moon's. The trajectory is drawn as a function of horizontal distance; dragging the landing point along the ground sets the launch speed.

```text
model Cannon {
  param {
    speed: Velocity = 20 m/s   where speed > 0 m/s   symbol "v"
    angle: Angle    = 45 deg   in [5 deg, 85 deg]    symbol "θ"  unit deg
    moon:  Boolean  = false
  }
  derived {
    g:     Acceleration = if moon then 1.62 m/s^2 else 9.81 m/s^2
    reach: Length       = speed^2 * sin(2 * angle) / g
    top:   Length       = (speed * sin(angle))^2 / (2 * g)
    // Heights in metres against distances in metres, as plain numbers for the plot.
    path(x: Real): Real = x * tan(angle) - (g / (1 m/s^2)) * x^2 / (2 * (speed / (1 m/s))^2 * cos(angle)^2)
  }
}
```

- `symbol "v"` is a **display symbol**: formulas show `v` for `speed`.
- A `function_graph` draws functions from `Real` to `Real`. Dividing a quantity by a unit gives a plain number: `speed / (1 m/s)` is `20` for `20 m/s`.

```text
presentation CannonLab for Cannon {
  view flight: plot(x: [0, 60], y: [0, 25]) {
    function_graph(path)
    marker(at: (reach / (1 m), 0)) as landing {
      on drag as p { propose speed = sqrt(p.x * 1 m * g / sin(2 * angle)) }
    }
  }
  panel controls {
    number_input(speed, range: [1 m/s, 30 m/s])
    slider(angle, range: [5 deg, 85 deg])
    toggle(moon)
    formula("R", speed^2 * sin(2 * angle) / g, live: true)
    label(top)
  }
  observe {
    R   = reach live
    H   = top   live
    log = intervention_log
  }
}
```

- The landing marker sits at `(reach / (1 m), 0)`: the plot's axes are plain numbers, so the reach is converted to metres as a number.
- Dragging it to `p.x` proposes the speed whose reach is `p.x` metres: `R = v² sin 2θ / g` solved for `v`. Dragging to a distance that needs more than the parameter allows is shown as invalid.
- `formula("R", ..., live: true)` typesets `R = v² sin(2θ) / g` with the current values of `v`, `θ` and `g`. `label(top)` shows the apex height.
- `intervention_log` observes every committed intervention: the learner's actions, in order, with their instants.

```cases
run default_cannon of Cannon with CannonLab {
  expect {
    R == 40.7747196738022 m within 1e-9 m
    H == 10.1936799184506 m within 1e-9 m
    log == []
  }
}

run on_the_moon of Cannon with CannonLab {
  param { moon = true }
  expect { R == 246.913580246914 m within 1e-9 m }
}

run flatter of Cannon with CannonLab {
  param { angle = 30 deg }
  expect { R == 35.3119430697019 m within 1e-9 m }
}
```

## Laying out views

Without a layout, each medium arranges the views itself: the web player fills a grid in the order they are declared, and still images and videos stack them. `layout` places them as the author intends: `row(...)` puts views side by side, `column(...)` one below the other, and either may hold the other.

```text
presentation CannonPage for Cannon {
  view flight: plot(x: [0, 60], y: [0, 25]) {
    function_graph(path)
    marker(at: (reach / (1 m), 0)) as landing
  }
  panel controls {
    slider(angle, range: [5 deg, 85 deg])
    toggle(moon)
  }
  panel readout {
    formula("R", speed^2 * sin(2 * angle) / g, live: true)
    label(top)
  }
  layout row(flight, column(controls, readout))
  observe { R = reach live }
}
```

- The plot is on the left; the controls sit above the readout on its right.
- A layout says where views go and how they share the page, never what they show. A medium adapts it to its size: on a narrow screen the web player turns a row into a column.
- Every name in a layout is a view or panel of the presentation, placed once. Views a layout leaves out follow it, in the order they are declared.

```cases
run page_layout of Cannon with CannonPage {
  expect { R == 40.7747196738022 m within 1e-9 m }
}
```

## Groups

A **group** draws several representations with one shared placement: its members are written in the group's own frame, then moved to `at`, turned by `rotate` and scaled by `scale`. A rigid body is drawn this way: its shape once, in its own coordinates, and its position and angle from the model.

**Goal.** A wheel rolling along the ground with its spoke turning, drawn as one rigid body.
**How it is built.** The model has the wheel's position `x` and angle `φ`; a `group` placed at the hub and turned by `-φ` draws the rim and the spoke in the wheel's own frame.

```text
space Plane = euclidean(2)

model Wheel in Plane {
  param {
    r: Length   = 0.5 m
    v: Velocity = 1 m/s
  }
  state {
    x: Length = 0 m
    φ: Angle  = 0
  }
  discrete {
    rolling: Boolean = true
  }
  derived {
    // The point on the rim that starts on the right, in the plane.
    rim: Point = origin + (x + r * cos(φ), r + r * sin(φ))
  }
  flow {
    der(x) = if rolling then v else 0
    der(φ) = if rolling then -v / r else 0      // rolling without slipping, clockwise
  }
  event road_end on rising(x - 4 m) {
    set rolling = false
  }
}
```

```text
presentation WheelView for Wheel {
  view road: spatial(Plane, scale: 1 m -> 100 px, y: up) {
    axes
    group(at: origin + (x, r), rotate: φ) as wheel {
      circle(origin, r) as tyre
      segment(origin, origin + (r, 0 m), color: gray)
      marker(origin + (r, 0 m), color: red) as valve
    }
    trace(rim every 0.02 s)
  }
  observe {
    lowest = rim.y at t0 + 0.785398163397448 s
  }
}
```

- The wheel is drawn around its own centre, `origin`, as a circle with one spoke; the group places that centre at `(x, r)` and turns it by `φ`. The marker `valve` is therefore always at the model's `rim`, and the trace of `rim` draws a cycloid behind it.
- `at` is a point (the group's `origin` goes there; default the view's origin), `rotate` an angle (default 0), `scale` a number (default 1). Points of members are moved, turned and scaled; vectors of arrows are turned and scaled.
- A group holds markers, arrows, segments, polylines, polygons, circles, ellipses, arcs and other groups; a group inside a group is placed in its parent's frame. Groups belong in spatial views.
- A group and each named member can be the target of `highlight`, `hide` and `reveal` in a timeline. The text alternative of a group lists those of its members.
- A trace is sampled over time, so it is drawn outside the group, from a model point (`rim`).
- The wheel stops at the end of a 4 m road (`road_end`), so the whole path fits the view.

After a quarter turn, at `t = π r / (2 v)`, the rim point touches the road:

```cases
run quarter_turn of Wheel with WheelView {
  until t0 + 2 s
  expect { lowest == 0 m within 1e-9 m }
}
```

A member of a group may be dragged. The gesture's value is then the pointer's point **in the group's frame**, the frame the member is written in:

**Goal.** The learner sets a radius by dragging the tip of a dial that is drawn turned.
**How it is built.** The tip is a member of a turned `group`; its drag gives the pointer in the group's frame, so `p.x` is the radius whichever way the dial is turned.

```text
space Plane = euclidean(2)

model Dial in Plane {
  param { r: Length = 1 m  in [0.2 m, 3 m] }
  state { x: Length = 0 m }
  flow { der(x) = 1 m/s }
}

presentation Knob for Dial {
  view scene: spatial(Plane, scale: 1 m -> 50 px, y: up) {
    group(at: origin + (2 m, 1 m), rotate: 90 deg) as dial {
      segment(origin, origin + (r, 0 m))
      marker(origin + (r, 0 m), color: blue) as tip { on drag as p { propose r = p.x } }
    }
  }
  observe { length = r live }
}
```

```cases
run dial of Dial with Knob {
  expect { length == 1 m exactly }
}
```

- The group turns its members a quarter turn, so the hand points up the screen. In the group's frame the tip is at `(r, 0 m)`, and `p.x` is how far along the hand the pointer is: dragging the tip up lengthens the hand, dragging it sideways does not. Written in the view's space, the same proposal would need the group's placement and turn undone by hand.
- Arrow keys step the tip in the view, as for any drag, and the proposal reads the step in the group's frame.

## Checks on a presentation

A presentation is checked against its model before anything is shown. Some mistakes:

```error
// error: PK-E03
model Cannon {
  param { speed: Velocity = 20 m/s }
  derived { twice: Velocity = 2 * speed }
}
presentation Bad for Cannon {
  panel p { slider(twice, range: [0 m/s, 50 m/s]) }
}
```

A derived value cannot be set; the slider must target a parameter (`speed`).

```error
// error: SX-E09
model Cannon {
  param { speed: Velocity = 20 m/s }
}
presentation Bad for Cannon {
  panel a { label(speed) }
  panel b { label(speed) }
  layout row(a, column(b, a))
}
```

A layout places each view once.

```error
// error: PK-E04
space Plane = euclidean(2)
model Moving in Plane {
  param { vel: Vector<Velocity> = (3 m/s, 4 m/s) }
}
presentation Bad for Moving {
  view scene: spatial(Plane, scale: 1 m -> 40 px, y: up) {
    arrow(vel, from: origin)
  }
}
```

A velocity arrow in a view that maps lengths needs its own scale: `arrow(vel, from: origin, scale: 1 m/s -> 10 px)`.

## Exercises

Solutions to the exercises not solved here are in [chapter 11](11-solutions.md).

1. Add a second function graph to `CannonLab` for the same launch without the Moon toggle (always `9.81 m/s^2`), to compare the two trajectories.
2. Add a marker at the apex, `(reach / (2 m), top / (1 m))`, with an inverse that sets the angle from the dragged height.
3. For the `Triangle` of chapter 3, add `number_input` controls for the coordinates of `A`. (Hint: a control targets a whole binding; which bindings would the model need?)

<details>
<summary>A solution to exercise 1</summary>

The second graph needs its own derived function in the model, since a presentation shows the model's values and does not compute new ones:

```text
model Cannon2 {
  param {
    speed: Velocity = 20 m/s   where speed > 0 m/s
    angle: Angle    = 45 deg   in [5 deg, 85 deg]   unit deg
    moon:  Boolean  = false
  }
  derived {
    g: Acceleration = if moon then 1.62 m/s^2 else 9.81 m/s^2
    path(x: Real): Real  = x * tan(angle) - (g / (1 m/s^2)) * x^2 / (2 * (speed / (1 m/s))^2 * cos(angle)^2)
    earth(x: Real): Real = x * tan(angle) - 9.81 * x^2 / (2 * (speed / (1 m/s))^2 * cos(angle)^2)
  }
}

presentation Compare for Cannon2 {
  view flight: plot(x: [0, 60], y: [0, 25]) {
    function_graph(path)
    function_graph(earth)
  }
  panel controls { toggle(moon); slider(angle, range: [5 deg, 85 deg]) }
  observe { e10 = earth(10) live }
}
```

```cases
run compare of Cannon2 with Compare {
  expect { e10 == 7.5475 within 1e-12 }     // 10 - 9.81 * 100 / (2 * 400 * 0.5)
}
```

</details>

## Plots that grow, zoom and pan

A plot's ranges say what part of the curve is drawn. A curve that leaves them is cut off at the frame. Two things help when the curve does not stay where the author expected:

- `follow: y` lets the `y` axis grow to hold everything the view draws; `follow: x` does the same for `x`, and `follow: (x, y)` for both. The declared range is where the axis starts and the least it shows: it grows, it never shrinks.
- `window: 10 s` suits long runs: once the data passes the declared `x` range, the axis shows the latest 10 s and moves along with the data, like a heart monitor. Growing with `follow: x` instead keeps the whole run visible, squeezed into the same width.
- `permit learner { zoom; pan }` lets the learner scroll the wheel over a plot to zoom about the pointer, pinch it with two fingers, and drag it to look beyond its ranges, as in a spatial view. A function graph is drawn over whatever range is shown. `Reset view` returns to the author's framing.

`x_unit` and `y_unit` choose the unit the axis is labelled in. The axis keeps its dimension: `y_unit: cm` on a length axis shows `5` where the value is `0.05 m`.

**Goal.** A spring's swing plotted against time, staying in view when the learner makes it larger than the plot's range, and open to zoom and pan.
**How it is built.** The plot declares `follow: (x, y)` so its axes grow to the data, `y_unit: cm` to label lengths in centimetres; `permit learner { zoom; pan }` lets the learner look around.

```text
model Spring {
  param {
    k:  Quantity<M/T^2> = 4 N/m      where k > 0 N/m
    m:  Mass            = 1 kg       where m > 0 kg
    x0: Length          = 0.1 m      in [0 m, 0.5 m]
  }
  state {
    x: Length   = x0
    v: Velocity = 0
  }
  flow {
    der(x) = v
    der(v) = -(k / m) * x
  }
}

presentation SpringLab for Spring {
  view position: plot(x: [0 s, 5 s], y: [-0.05 m, 0.05 m], follow: (x, y), y_unit: cm) {
    series_plot(x every 0.02 s)
  }
  panel controls { slider(x0, range: [0 m, 0.5 m]) }
  permit learner { zoom; pan }
  observe { x2 = x at t0 + 2 s }
}
```

The declared `y` range, -5 cm to 5 cm, is smaller than the swing of 10 cm: with `follow` the axis grows to about -11 cm to 11 cm as soon as the curve gets there, and the time axis grows past 5 s while the session plays on. Without `follow`, the curve would leave the frame and the learner could still zoom out or drag the plot to see it.

```cases
run swing of Spring with SpringLab {
  until t0 + 2 s
  expect { x2 == -0.0653643620863612 m within 1e-6 m }
}
```

## Steering a running model

**Goal.** The learner grabs a moving puck and steers it while time keeps running, instead of pausing it to place it.
**How it is built.** The puck's position is `state` marked `intervenable` (the learner may change it); its marker has `on drag live as p { propose pos = p }`.

By default a drag holds the run at the instant shown, and the run goes on from the new value after the release: good for placing something carefully. With `live`, the run keeps going during the drag. At each instant the clock passes, the pointer's position is committed as its own intervention, so the puck follows the pointer, and keeping the pointer still holds the puck still. After the release, the model takes over again from the last position.

```text
space Plane = euclidean(2)

model Puck in Plane {
  state {
    pos: Point            = origin                  intervenable
    vel: Vector<Velocity> = (1 m/s, 0.5 m/s)
  }
  flow { der(pos) = vel }
}

presentation Rink for Puck {
  view ice: spatial(Plane, scale: 1 m -> 40 px, y: up) {
    trace(pos every 0.05 s) as path
    marker(pos) as puck { on drag live as p { propose pos = p } }
  }
  permit learner { timeline_controls }
  observe { at2 = pos at t0 + 2 s }
}
```

```cases
run drift of Puck with Rink {
  until t0 + 3 s
  expect { at2 == origin + (2 m, 1 m) within 1e-9 m }
}
```

## When a run fails in a lab

A run can fail: a `constraint` with `policy stop` is broken, an event cascade does not end, a value becomes undefined. A test then stops and reports it. A lab **pauses** instead: the run holds at the last valid state, just before the failure, and the player says what failed and when. The learner can look at it, then change something at that instant (a slider, a drag, a button) and the run continues from there with the change, or press Reset.

```prismal
model Rise {
  param { limit: Real = 2 in [1, 10] }
  state { x: Real = 0 }
  flow { der(x) = 1 / 1 s }
  constraint below: x <= limit policy stop
}
```

In a lab of `Rise` with a slider for `limit`, the run pauses at 2 s with "constraint `below` violated". Raising `limit` to 10 at that instant lets it go on to 10 s.

## Text and titles

**Goal.** Tell the learner what they are looking at: a title for the page, captions for the views, a sentence that reports a value, and a message that appears only when something has happened.
**How it is built.** `title "..."` names the presentation; `title("...")` inside a view or panel replaces its caption; `text("...")` writes a sentence, with any value of the model in braces; `when:` on any representation shows it only while a condition holds.

```text
space Plane = euclidean(2)

model Dropped in Plane {
  param { h: Length = 10 m in [1 m, 50 m] }
  state {
    pos: Point            = origin + (0 m, h)
    vel: Vector<Velocity> = 0
  }
  discrete { airborne: Boolean = true }
  flow {
    der(pos) = if airborne then vel else 0
    der(vel) = if airborne then (0 m/s^2, -9.81 m/s^2) else 0
  }
  event landed on falling(pos.y) { set airborne = false; set vel = 0 }
}

presentation DropLab for Dropped {
  title "Dropping a ball"
  view scene: spatial(Plane, scale: 1 m -> 12 px, y: up) {
    title("The fall")
    marker(pos) as ball
    text("start", at: origin + (0.5 m, h))
  }
  panel notes {
    title("What is happening")
    text("The ball is {pos.y} above the ground, moving at {|vel|}.")
    text("Falling...", when: airborne)
    text("Landed. Move the slider to drop it from another height.", when: not airborne)
    slider(h, range: [1 m, 50 m])
  }
  observe { t_land = elapsed on landed }
}
```

```cases
run drop of Dropped with DropLab {
  until t0 + 2 s
  expect { t_land[1] == 1.42784312292706 s within 1e-9 s }
}
```

- Inside the quotes, `{...}` holds any value of the model: a binding, a component (`{pos.y}`), a calculation (`{|vel|}`). It is shown with its unit, in the binding's display unit when it has one. To write a brace itself, double it: `{{` and `}}`.
- `{pos.y:.1}` shows the value with one decimal (`8.3 m`), `{angle:.0}` with none (`45 deg`). Without it a value has up to six significant digits, more than a learner usually needs. Powers in units are written as people read them: an area shows as `25 m²`, an acceleration as `9.81 m/s²`.
- `at: P` places a text in a spatial view (`P` a point) or a plot (`P` a pair of axis values); without it, the text is a line in the panel or below the view. A placed text starts at `P`; `align: center` centres it on `P` and `align: right` ends it there (a label under a point, a note left of a wall).
- `when: c` works on every representation, not only text: `marker(pos, when: airborne)` hides the marker once the ball has landed, `slider(h, ..., when: not airborne)` offers the slider only then. The condition is read at the instant shown.
- There is no text that is chosen by a condition inside one sentence; write one text per case, each with its `when`, as above.

## Words for values

**Goal.** Let a learner read the state of a system in words, not only in numbers: "Comfortable" rather than "293 K", a slider that runs from "cold" to "hot", a share shown as "40 %".
**How it is built.** `ends:` on a slider names its two ends; the display unit `%` shows a share as a percentage; and one `text` per band, each with its own `when:`, tells a value in words.

```text
model Room {
  param {
    temp:  Quantity<Θ> = 293 K  in [273 K, 313 K]
    humid: Real        = 40 %   in [0 %, 100 %]   unit %
  }
}

presentation RoomLab for Room {
  title "How does the room feel?"
  panel controls {
    slider(temp, range: [273 K, 313 K], ends: ("cold", "hot"))
    slider(humid, range: [0 %, 100 %], ends: ("dry", "damp"))
    label(humid)
    text("Cold", when: temp < 288 K)
    text("Comfortable", when: temp >= 288 K and temp < 298 K)
    text("Hot", when: temp >= 298 K)
  }
  observe { h = humid live }
}
```

```cases
run damp of Room with RoomLab {
  param { humid = 75 % }
  expect { h == 0.75 exactly }
}
```

- The bands are written so that exactly one holds for any temperature: `<` on one side, `>=` on the next.
- A band that is a mode of the model, used by flows or events, belongs in the model instead: declare an enumeration (chapter 5) and show it with `label`, which prints the case's name.
- A slider shows the real value with its unit, beside its end words. A dial from 0 to 10 with no unit is written in the model, as a parameter `level: Real in [0, 10]` from which the physical value is computed, so that the model says what the learner is really setting.

