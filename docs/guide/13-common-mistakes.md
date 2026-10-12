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

Prismal computes formulas in one direction (D-007). If the two equations are meant to be solved together, solve them by hand (`a = -1`, `b = -2`), or make one of the values state that changes over time.

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
