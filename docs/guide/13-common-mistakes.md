# 13. Common Mistakes

The errors you are likely to meet first, what they mean and how to fix them. Every program is checked against the code it names: each is reported with that code, a line and a column. The full list of codes is in the [reference](10-reference.md#diagnostics).

## Units

**A value of the wrong dimension.** A position changes at the rate of a velocity, not of an acceleration:

```error
// error: MK-E01
model Fall {
  param { g: Acceleration = 9.81 m/s^2 }
  state { h: Length = 10 m }
  flow { der(h) = -g }
}
```

`der(h)` has the dimension of `h` per second, a velocity. The fix is a velocity state: `state { v: Velocity = 0 m/s }`, `der(h) = v`, `der(v) = -g`. Most dimension errors are a missing or extra factor of time.

**A number without its unit.** A length needs a unit; Prismal never guesses one:

```error
// error: MK-E03
model Tower {
  param { height: Length = 50 }
}
```

Write `50 m`. The literal `0` is the one exception: it fits any unit, so `v: Velocity = 0` is fine.

**A unit Prismal does not know.** Units are written as symbols from a fixed list; a word that is not one of them is read as something else. Here `hr` is taken for a name:

```error
// error: SX-E03
model Road {
  param { speed: Velocity = 50 km/hr }
}
```

and `50 meters` is reported as a syntax error (SX-E02) at `meters`. Write `50 km/h` and `50 m`. Units are symbols, `m`, `km`, `s`, `min`, `h`, `kg`, `N`, `J`, `deg`, `rad` and others, combined as `m/s^2` or `kg*m/s`; chapter 2 lists them.

**Adding two points.** A point is a place; places do not add. Their difference is a vector, and a point plus a vector is a point:

```error
// error: MK-E04
space Plane = euclidean(2)
model Two in Plane {
  param {
    a: Point = origin + (1 m, 0 m)
    b: Point = origin + (0 m, 2 m)
  }
  derived { middle: Point = (a + b) / 2 }
}
```

Write `middle: Point = a + (b - a) / 2`.

## Names

**A misspelled or undeclared name.**

```error
// error: SX-E03
model Wave {
  param { amplitude: Real = 1 }
  derived { y(x: Real): Real = amplitde * sin(x) }
}
```

The message names the unknown word and points at it.

**A reserved word as a name.** Words of the language, such as `event`, `state` or `in`, are not names:

```error
// error: SX-E07
model Count {
  param { event: Real = 1 }
}
```

Choose another name (`events`, `count`). The reference lists the reserved words; words of presentations and lessons (`view`, `beat`, `scene` ...) are reserved only where they are used, so `process drag` or a binding named `scene` is fine.

**A long formula broken before an operator.** A line ends a statement unless it ends in an operator, a comma or an open bracket. A line that starts with `+` is read as a new statement, and the message says that a line cannot start with an operator:

```error
// error: SX-E02
space Line = euclidean(1)
model Spring in Line {
  state { x: Length = 1 m }
  flow {
    der(x) = -x / 1 s
      + 1 m/s
  }
}
```

Break the line after the operator instead (`der(x) = -x / 1 s +`, then `1 m/s` on the next line), or put the whole formula in parentheses.

## Models

**A handler that sets a parameter.** Parameters are fixed for a run; an event changes state:

```error
// error: MK-E08
model Lamp {
  param { brightness: Real = 1 }
  event dim on at t0 + 1 s { set brightness = 0.5 }
}
```

Make `brightness` discrete state (`discrete { brightness: Real = 1 }`), or let a lesson or a slider change the parameter from outside.

**Values that define each other.** A derived value cannot depend on itself, directly or through others:

```error
// error: MK-E14
model Loop {
  derived {
    a: Real = b + 1
    b: Real = 2 * a
  }
}
```

A derived value is computed in one direction, from values already known, so two derived values cannot be defined from each other. If the two formulas are meant to hold together, declare the values as **unknowns** and state the formulas as equations; the runtime solves them together ([chapter 17](17-solving-equations.md)):

```text
model Pair {
  unknown {
    a: Real
    b: Real
  }
  equation first:  a == b + 1
  equation second: b == 2 * a
}

presentation Values for Pair {
  observe {
    a_value = a at t0
    b_value = b at t0
  }
}
```

```cases
run together of Pair with Values {
  expect {
    a_value == -1 within 1e-9
    b_value == -2 within 1e-9
  }
}
```

If one of the values instead changes over time, make it state with a flow.

**A bounce without a Zeno policy.** A ball that bounces with less speed each time bounces infinitely often in a finite time. Prismal refuses an event that may repeat like this unless the model says what to do when it happens:

```error
// error: MK-E16
model Ball {
  param {
    g: Acceleration = 9.81 m/s^2
    e: Real = 0.8
  }
  state {
    y: Length   = 1 m
    v: Velocity = 0 m/s
  }
  flow {
    der(y) = v
    der(v) = -g
  }
  event bounce on falling(y) { set v = -e * v }
}
```

Add `zeno settle { ... }` after the handler, with the operations that bring the ball to rest, or `zeno stop` to end the run there. [The tour](00-tour.md) and chapter 5 show both.

## Presentations

**A control on something the learner may not change.** A slider changes a parameter; it cannot change a derived value, which always follows its formula:

```error
// error: PK-E03
model Cannon {
  param { speed: Velocity = 20 m/s }
  derived { reach: Length = speed^2 / (9.81 m/s^2) }
}
presentation Bad for Cannon {
  panel p { slider(reach, range: [0 m, 100 m]) }
}
```

Put the slider on `speed`. To let the learner set the reach, give the speed control an inverse instead: drag a marker at the reach and propose the speed that gives it (chapter 7).

**An arrow without a scale.** A view maps lengths to pixels; a velocity is not a length, so its arrow needs its own scale:

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

Write `arrow(vel, from: origin, scale: 1 m/s -> 10 px)`.

**A misspelled option.** Each representation and view takes a fixed list of named options. A name not on the list is an error, never ignored, and the message lists the options and suggests the nearest one:

```error
// error: SX-E08
space Plane = euclidean(2)
model Dot in Plane {
  param { pos: Point = origin }
}
presentation Bad for Dot {
  view scene: spatial(Plane, scale: 1 m -> 40 px, y: up) {
    marker(pos, colour: red)
  }
}
```

The message is "`marker` takes no option `colour`; did you mean `color`?". The options of every kind are in the [reference](10-reference.md#options-and-clauses).

## Runs

**A value outside its range.** Ranges are checked when a run starts, whoever sets the value:

```text
model Ramp {
  param { angle: Real = 30   in [0, 90] }
}
```

```cases
run too_steep of Ramp {
  param { angle = 120 }
  expect { initialization fails }
}
```

The run is refused before it starts; a test can expect that, as here. In the player the same value from a slider is refused with the range in the message, and the previous value stays.

**A run that stops.** A constraint with `policy stop`, a collection that reaches its capacity, or an event that keeps firing at one instant stops the run with a diagnostic naming the cause and the instant. The player shows it under the transport; a case can observe it with the observation source `diagnostics` (or `diagnostics of` a constraint) and check what happened before it. Chapter 6 covers constraints and run diagnostics.

## Later chapters

The mistakes met first in chapters 15 to 24. Each section names the chapter that teaches the form.

### Chance (chapter 15)

**A random draw in a derived value.** A distribution gives a new value each time it is drawn, so a value computed from one would change whenever it is read. Draws happen at events:

```error
// error: MK-E28
model Walk {
  param { s: Length = 1 m }
  derived { step: Length = normal(0 m, s) }
}
```

Draw in an event's operations and keep the result in discrete state: `event move on every 1 s { set pos = pos + normal(0 m, s) }`.

**`noise` outside a flow.** White noise is a random push at every instant; it only has a meaning as part of a rate:

```error
// error: MK-E29
model Jitter {
  param { D: Real = 1 }
  state { x: Real = 0 }
  derived { kick: Real = sqrt(2 * D) * noise }
  flow der(x) = kick
}
```

Write it in the flow itself: `flow der(x) = sqrt(2 * D) * noise`.

### Fields (chapter 16)

**`laplacian` of a field read where no field is computed.** `laplacian(T)` compares each cell with its neighbours, so it gives a value per cell. It belongs where a value per cell is computed: a field's flow or a field's definition.

```error
// error: MK-E32
model Rod {
  grid rod(x: [0 m, 1 m], cells: 50)
  state { T(x in rod): Real = 20 }
  derived { bend: Real = laplacian(T) }
  flow der(T) = 0
}
```

Write it in the field's flow, `flow der(T) = alpha * laplacian(T)`, or define a field of the grid with it, `bend(x in rod): Real = laplacian(T)`.

### Solving equations (chapter 17)

**Fewer equations than unknowns.** Every unknown needs an equation to determine it. Here there are two unknowns and one equation:

```error
// error: MK-E34
model Divider {
  param {
    V0: Voltage    = 9 V
    R1: Resistance = 100 Ω
  }
  unknown {
    I:  Current
    V2: Voltage
  }
  equation loop: V0 == I * R1 + V2
}
```

The message counts both and names them. Add the missing fact, here the second resistor's law, `equation load: V2 == I * R2` with `R2` a parameter. A vector or point unknown counts once per component.

### Constrained motion (chapter 18)

**An enforced constraint with nothing to hold it.** To keep a bob on its circle, something must pull it: the runtime finds that force, so the force must be an `unknown` that the flows read. Without one, the constraint cannot be enforced:

```error
// error: MK-E36
space Plane = euclidean(2)

model Bob in Plane {
  param {
    L: Length       = 1 m
    g: Acceleration = 9.81 m/s^2
  }
  state {
    bob: Point            = origin + (1 m, 0 m)
    vel: Vector<Velocity> = 0
  }
  flow {
    der(bob) = vel
    der(vel) = (0 m/s^2, -g)
  }
  constraint rod: |bob - origin| == L policy enforce
}
```

Declare the rod's pull, `unknown { tension: Force = 10 N }`, and add it to the flow along the rod, as chapter 18's first program does.

### Shared definitions and modules (chapter 19)

**A library definition used without `use`.** `Body` is defined in the first-party module `mechanics`, which a file must import:

```error
// error: SX-E03
space Plane = euclidean(2)

model Throw in Plane {
  parts { ball: Body { m = 2 kg } }
}
```

Put `use mechanics` at the top of the file. The same holds for the functions of `optics` (chapter 22): `refract`, `thin_lens` and the others are unknown names until the file says `use optics`.

### Three dimensions (chapter 20)

**`cross` in a plane.** The cross product is a vector at right angles to both its arguments, which only exists in a space of three dimensions:

```error
// error: MK-E01
space Plane = euclidean(2)

model Spin in Plane {
  param {
    u: Vector<1> = (1, 0)
    v: Vector<1> = (0, 1)
  }
  derived { w: Vector<1> = cross(u, v) }
}
```

Declare the model in a space of three dimensions, `space Room = euclidean(3)`, with vectors of three components. In a plane, the turning effect of two vectors is the number `u.x * v.y - u.y * v.x`.

### Algorithms (chapter 21)

**A counter that is continuous state.** An algorithm's counters change in steps, so they are discrete state; chapter 21 ends with this mistake (MK-E37).

### Frames (chapter 23)

**A frame of three dimensions with one axis.** In a plane, a frame's second axis is the first turned a quarter turn. In a space of three dimensions that leaves a choice, so the second axis is given:

```error
// error: SX-E08
space Room = euclidean(3)

model Tilt in Room {
  param { a: Vector<1> = (1, 0, 0) }
  frame tilted(at: origin, x: a)
}
```

Give it, `frame tilted(at: origin, x: a, y: (0, 1, 0))`; the third axis follows, at right angles to both.

### Light (chapters 22 and 24)

**A wavelength without a unit.** The functions of `optics` take wavelengths and distances as lengths, so a bare number is refused:

```error
// error: MK-E03
use optics

model Fringes {
  param {
    d: Length = 0.1 mm
    θ: Angle  = 0.2 deg unit deg
  }
  derived { brightness: Real = two_slit(d, 500, θ) }
}
```

Write the unit: `two_slit(d, 500 nm, θ)`.
