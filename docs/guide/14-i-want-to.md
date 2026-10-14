# I Want To...

Each entry is a goal, the **combination** of words that achieves it, and the block each word goes in. Most goals need two or three words working together, often in different blocks: a value declared in the model and a control or drawing in the presentation. The snippets are fragments; each entry links to a chapter with a complete program that uses them. [What goes where](00-structure.md) shows every block in one program.

Notation: **model / param** means "inside the model, in its `param` block"; **presentation / panel** means "inside the presentation, in a `panel`".

## Make things change

### Give the learner a setting to change

Combine a **parameter** with a **slider**.

```prismal
// model / param
k: Quantity<M/T^2> = 4 N/m   in [1 N/m, 20 N/m]
// presentation / panel
slider(k, range: [1 N/m, 20 N/m])
```

The slider names the parameter it moves. The `in [...]` range stops any value outside it, from a slider or a test. See [chapter 1](01-first-program.md).

### Let the learner choose one of a few kinds (a material, a planet)

Combine an **enumeration**, a **parameter** of it, a **derived** value by **match**, and **select**.

```prismal
// model
enum Material { copper, steel, glass }
// model / param
material: Material = copper
// model / derived
alpha: Quantity<L^2/T> = match material { copper => 1.1e-4 m^2/s, steel => 4e-6 m^2/s, glass => 3.4e-7 m^2/s }
// presentation / panel
select(material)
```

The learner picks a case by name. As a parameter, the choice is kept by `button(restart)` and set in a case with `param { material = steel }` or in a lesson with `intervene { set material = steel }`. See the spoon example (`examples/heat.md`).

### Make a value change over time

Combine a **state** value (its start) with a **flow** (its rate of change).

```prismal
// model / state
x: Length = 0.1 m
// model / flow
der(x) = v
```

A state value with no flow stays constant. See [Start here](00-start-here.md) and [chapter 4](04-motion.md).

### Make something move in the plane

Combine a **space**, a model `in` that space, a **Point** and a **Vector** in state, and two flows.

```prismal
space Plane = euclidean(2)                       // top level
model Ball in Plane {                            // `in Plane` gives Point and Vector a meaning
  state { pos: Point = origin + (0 m, 1 m); vel: Vector<Velocity> = (3 m/s, 4 m/s) }
  flow  { der(pos) = vel; der(vel) = (0 m/s^2, -9.81 m/s^2) }
}
```

See [chapter 3](03-space-and-vectors.md) and [chapter 4](04-motion.md).

### Give a quantity a value at every point

Combine a **grid**, a **field** in state, a **flow** with `laplacian`, and a **boundary**.

```prismal
// model
grid rod(x: [0 m, 1 m], cells: 50)
state { T(x in rod): Real = 20 }
flow der(T) = alpha * laplacian(T)
boundary T { left = 100 }
// presentation / a plot view
function_graph(T)
```

In the plane, the grid takes `y: [...]` and the presentation draws `heatmap(T)` in a spatial view. See [chapter 16](16-fields.md).

### Add up several influences on one rate

Write each influence as a **contribution** with `+=`, usually in a named **process**.

```prismal
// model
process spring  { flow der(v) += -(k / m) * x }
process damping { flow der(v) += -(c / m) * v }
```

The rate is the sum of the contributions. See [chapter 4](04-motion.md).

### Compute a value from others

Use **derived**.

```prismal
// model / derived
energy: Energy = 0.5 * m * v^2 + 0.5 * k * x^2
```

A derived value is always up to date; it never needs a flow. See [chapter 1](01-first-program.md).

### Find values that only equations give together

Combine **unknowns** with **equations** that read them.

```prismal
// model / unknown
I:  Current
V2: Voltage
// model
equation loop: V0 == I * R1 + V2
equation load: V2 == I * R2
```

Write one equation per unknown. The runtime solves them at every instant; a value after an unknown's type (`Vd: Voltage = 0.6 V`) is a guess that starts the solver. See [chapter 17](17-solving-equations.md).

### Give a rate by an equation

Write an **equation** that reads `der(x)` of a **state** with no flow.

```prismal
// model / state
I: Current = 0 A
// model
equation faraday: L * der(I) == V0 - R * I
```

The equation gives `I` its rate, as a flow would. See [chapter 17](17-solving-equations.md).

### Hold a body on a rod, a wire or a curve

Combine a **constraint** with `policy enforce`, an **unknown** for the force that holds the body, and a **flow** that reads it.

```prismal
// model / unknown
tension: Force
// model / flow
der(vel) = (0 m/s^2, -g) - tension / m * (bob - pivot) / |bob - pivot|
// model
constraint rod: |bob - pivot| == L policy enforce
```

The runtime keeps the bob at distance `L` and finds the tension at every instant. Write one unknown per constraint, acting in the direction the rod or wire pushes. A point that only the learner moves (no flows) needs no unknown: the constraint alone holds it where it is dragged. See [chapter 18](18-constraints.md).

### Make something happen when a value crosses a threshold

Combine an **event** with a **crossing trigger** and **set**.

```prismal
// model
event full on rising(level - high) { set pumping = false }
```

`rising(g)` fires when `g` crosses zero going up, so `level - high` fires when `level` passes `high`. `falling` is the downward crossing. The value changed is usually **discrete**. See [chapter 5](05-events-and-modes.md).

### Find where a thrown object lands, from a height or on a slope

Start the **state** at the height, and combine an **event** with **falling** on the height above the ground.

```prismal
// model / state
pos: Point = origin + (0 m, h)
// model
event landed on falling(pos.y) { set flying = false }
event on_hill on falling(pos.y - slope * pos.x)
// presentation / observe
range = pos.x on landed
```

The event fires where the path meets the ground, whatever the starting height: `falling(pos.y)` for flat ground at `y = 0`, `falling(pos.y - ground)` for any ground the model can compute. To show the formula beside the simulation, a **derived** value gives it: `speed * cos(angle) * (vy + sqrt(vy^2 + 2 * g * h)) / g`. See [chapter 5](05-events-and-modes.md) and the projectile example (`examples/projectile.md`).

### Let several events add to one value at the same instant

Combine **discrete** state declared with **combine** and **contribute** in each handler.

```prismal
// model / discrete
total: Real = 0   combine sum
// model
event small on every 1 s { contribute total += 1 }
event large on every 1 s { contribute total += 10 }
```

Two `set`s of one value at one instant are a conflict; contributions are not. See [chapter 5](05-events-and-modes.md).

### Switch between behaviors (on/off, modes)

Combine a **discrete** value, **if ... then ... else** in the flow, and **events** that switch it.

```prismal
// model / discrete
pumping: Boolean = true
// model / flow
der(level) = if pumping then inflow else -outflow
// model
event full  on rising(level - high) { set pumping = false }
event empty on falling(level - low) { set pumping = true }
```

For more than two modes use an `enum` with `match`. See [chapter 5](05-events-and-modes.md).

### Let the learner trigger something with a button

Combine an **event `on request`** in the model with a **button** in the presentation.

```prismal
// model
event again on request { set pos = origin + (0 m, h); set vel = 0 }
// presentation / panel
button(again)
```

See [What goes where](00-structure.md) and [chapter 7](07-presentations.md).

### Let a button give a value to the event it requests

Combine an **event `on request(name: Type)`** with **button(event(value))**.

```prismal
// model
event kick on request(j: Momentum) { set v = v + j / m }
// presentation / panel
button(kick(2 kg*m/s), label: "Small kick")
button(kick(6 kg*m/s), label: "Big kick")
```

See [chapter 5](05-events-and-modes.md).

### Let the learner drag a point to set a value

Combine a **marker** with an **inverse**: `on drag as p { propose ... }`.

```prismal
// presentation / view (spatial)
marker(A) as a { on drag as p { propose A = p } }
```

`A` must be a parameter (or state marked `intervenable`). See [chapter 3](03-space-and-vectors.md).

### Let the learner steer something while it moves

Combine **intervenable** state with a **live** drag.

```prismal
// model / state
pos: Point = origin   intervenable
// presentation / view (spatial)
marker(pos) as puck { on drag live as p { propose pos = p } }
```

Without `live`, the run holds while the learner drags. See [chapter 7](07-presentations.md), Steering a running model.

### Model many similar objects

Combine an **object** type, a **collection** in `parts`, and a flow **for each member**.

See [chapter 9](09-objects.md): the combination has several parts, and the chapter builds it step by step.

### Add members to a part's collection from outside it

Use **create** with the collection's **path** through the contained object.

```prismal
// model (the container of `left`)
event add on request { create left.atoms { pos = origin + (1 m, 0 m) } }
```

See [chapter 9](09-objects.md), Making members of a contained object.

### Join two members at most once

Combine an event **on request** with a **condition** that no relation already has both members.

```prismal
// model
event join on request(x in balls, y in balls) if not any(s.has(x) and s.has(y) for s in links) {
  connect links(x, y)
}
```

A second request for the same pair, in either order, is refused. See [chapter 9](09-objects.md).

## Show things

### Give the page and its views titles

Use **title** in the presentation, and **title(...)** inside a view or panel.

```prismal
// presentation
title "Dropping a ball"
// presentation / view or panel
title("The fall")
```

See [chapter 7](07-presentations.md), Text and titles.

### Write a sentence that shows a value

Use **text** with the value in braces.

```prismal
// presentation / panel (or view)
text("The ball is {pos.y} above the ground.")
text("start", at: origin + (0 m, h))        // placed at a point of a spatial view
```

### Round a number in a text

Add **:.N** after the value in braces.

```prismal
// presentation / panel
text("Landed {last:.1} away.")        // 35.3 m
```

### Name a drawing without labelling it

Add **label: none** (or **label: "..."** for another label).

```prismal
// presentation / view
marker(pos, label: none) as ball      // `ball` for the lesson, no word on the screen
```

### Show something only when a condition holds

Add **when:** to the representation: text, marker, arrow, slider, anything.

```prismal
// presentation / panel
text("Falling...", when: airborne)
text("Landed.", when: not airborne)
```

See [chapter 7](07-presentations.md), Text and titles.

### Plot a formula and change it live

Combine a **derived function**, a **plot view** and **function_graph**, plus sliders for its parameters.

```prismal
// model / derived
y(x: Real): Real = A * sin(k * x)
// presentation
view graph: plot(x: [-6, 6], y: [-3, 3]) { function_graph(y) }
panel controls { slider(A, range: [0, 3]) }
```

See [chapter 1](01-first-program.md).

### Plot a value against time

Combine a **plot view whose x axis is a time range** with **series_plot ... every**.

```prismal
// presentation
view position: plot(x: [0 s, 20 s], y: [-0.12 m, 0.12 m]) { series_plot(x every 0.02 s) }
```

The `x` range must be a time (`[0 s, 20 s]`), and `every` sets how often a point is recorded. See [chapter 4](04-motion.md).

### Keep a curve in view when it grows past the axes

Add **follow** to the plot, or **window** for long runs.

```prismal
view position: plot(x: [0 s, 5 s], y: [-5 cm, 5 cm], follow: (x, y)) { series_plot(x every 0.02 s) }
view recent:   plot(x: [0 s, 10 s], y: [-1 m, 1 m], window: 10 s)   { series_plot(x every 0.02 s) }
```

See [chapter 7](07-presentations.md), Plots that grow, zoom and pan.

### Let the learner zoom and scroll a view

Add **permit learner { zoom; pan }** to the presentation.

```prismal
// presentation
permit learner { zoom; pan }
```

This works for spatial views and plots alike. See [chapter 7](07-presentations.md).

### Show where something has been

Combine a **spatial view** with **trace ... every**.

```prismal
// presentation / view (spatial)
trace(pos every 0.02 s) as path
marker(pos) as ball
```

See [the tour](00-tour.md).

### Show a vector as an arrow

Use **arrow** with **from** (where it starts) and **scale** (how long it is drawn).

```prismal
// presentation / view (spatial)
arrow(vel, from: pos, scale: 1 m/s -> 15 px) as velocity
```

See [chapter 3](03-space-and-vectors.md).

### Show a number as it changes

Use **label** in a panel. For a formula with its current values, use **formula**.

```prismal
// presentation / panel
label(energy)
formula("speed", |vel|, live: true)
```

See [chapter 4](04-motion.md) and [the tour](00-tour.md).

### Tell the state of the system in words

Write one **text** per band, each with **when:**.

```prismal
// presentation / panel
text("Cold", when: temp < 288 K)
text("Comfortable", when: temp >= 288 K and temp < 298 K)
text("Hot", when: temp >= 298 K)
```

See [chapter 7](07-presentations.md), Words for values.

### Put words at the ends of a slider

Add **ends:** to the slider.

```prismal
// presentation / panel
slider(temp, range: [273 K, 313 K], ends: ("cold", "hot"))
```

### Show a share as a percentage

Give the value the display **unit %**.

```prismal
// model / param
humid: Real = 40 %   in [0 %, 100 %]   unit %
```

### Show a value in other units

On a model value, add **unit** to its declaration; on a plot axis, use **x_unit** or **y_unit**.

```prismal
// model / param
slope: Angle = 3 deg   in [0 deg, 30 deg]   unit deg
// presentation
view h: plot(x: [0 s, 5 s], y: [0 m, 1 m], y_unit: cm) { series_plot(height every 0.1 s) }
```

See [chapter 2](02-quantities-and-units.md).

### Put views side by side

Use **layout** with **row** and **column**.

```prismal
// presentation
layout row(scene, column(controls, readout))
```

See [chapter 7](07-presentations.md).

## Teach with it

### Narrate a lesson that plays the model

Add a **timeline** to a presentation, with **scene**, **beat**, **narrate** and **run**.

```prismal
// presentation
timeline {
  scene throw {
    beat intro  { narrate "A ball is thrown." for 3 s }
    beat flight { run rate 1 until bounce }
  }
}
```

A presentation with a timeline is a lesson; one without is a lab. See [chapter 8](08-lessons.md).

### Change a drawing's color, size or shape during a lesson

Use **animate** with the property: **color**, **line**, **scale** or **morph**, and optionally **ease**.

```prismal
// presentation / timeline / scene / beat
animate base color to red for 1 s
animate centre scale to 2 for 1 s ease out
animate square morph to round for 2 s       // `round` is a hidden representation
```

See [chapter 8](08-lessons.md), Color, size and shape.

### Zoom into part of a graph during a lesson

Use **camera** on the plot view, with a pair of axis values.

```prismal
// presentation / timeline / scene / beat
camera graph to (2, 2) zoom 2 for 1 s
```

See [chapter 8](08-lessons.md).

### Pause a lesson and let the learner try things

Use an **explore** beat with the controls the learner may use.

```prismal
// presentation / timeline / scene
beat choose { hold; explore limit 30 s keep h { slider(h, range: [1 m, 50 m]) } }
```

See [chapter 8](08-lessons.md).

### Export a lesson as a video

Use the command line, not the program:

```sh
cargo run --release -p prismal-media -- lesson.prismal Story lesson.mp4
```

See [Recipes](12-recipes.md).

## Check it

### Check a value at a given time

Combine an **observation** in the presentation with an **expectation** in a run.

```prismal
// presentation
observe { x5 = x at t0 + 5 s }
// top level
run damped of Oscillator with OscillatorLab {
  until t0 + 10 s
  expect { x5 == -0.0336851680590413 m within 1e-6 m }
}
```

See [chapter 6](06-checks-and-tests.md).

### Check when an event happens

Observe **elapsed on** the event, then index the list.

```prismal
// presentation
observe { bounces = elapsed on bounce }
// run / expect
bounces[1] == 1.42784312292706 s within 1e-9 s
```

See [the tour](00-tour.md).

### Test a different setting without changing the model

Put **param** in the **run**, with values only (no types).

```prismal
run softer of Throw with Measure {
  param { e = 0.5 }
  until t0 + 3 s
  expect { bounces[2] == 1.624515662498 s within 1e-9 s }
}
```

See [chapter 6](06-checks-and-tests.md).

### Make sure a value never goes out of bounds

Use a **constraint** in the model.

```prismal
// model
constraint above_ground: pos.y >= 0 m
```

See [chapter 6](06-checks-and-tests.md).
