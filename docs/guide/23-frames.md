# 23. Frames

A vector does not depend on the axes it is measured against, but its components do. Gravity on a slope is easier to think about as a part along the slope and a part across it; a ball rolling straight across a turning carousel looks, to a rider, as if it curves. This chapter declares **frames**, axes of the model's own, reads points and vectors in them, draws a view as seen from one, and writes the forces a turning frame adds (D-022, D-105).

> **In plain words.**
> - A **frame** is an origin and a set of axes. Every space has its standard frame (`origin`, `x`, `y`, `z`); a model can declare others, turned or moving.
> - `frame slope(at: origin, x: along)` declares a frame whose first axis points along `along`. In a plane the second axis is the first turned a quarter turn anticlockwise; in a space of three dimensions it is given too, `y: v`.
> - `v.in(slope).x` is the component of `v` along the frame's first axis; for a point, its coordinate measured from the frame's origin.
> - A frame whose axes read time **turns**. Seen from it, things that move straight seem to curve: the **Coriolis** and **centrifugal** forces are what it takes to describe motion as seen from a turning frame.

## A ball on a slope, read along it

**Goal.** A ball released on a frictionless slope: gravity split into a part along the slope and a part across it, and the distance travelled along the slope, which grows as `g sin θ t² / 2`.
**How it is built.** The direction down the slope is a derived vector `along`; the frame `slope` has its first axis along it. The flow uses only the part of gravity along the slope. `gravity.in(slope).x` and `gravity.in(slope).y` are the two parts; `pos.in(slope).x` is the distance down the slope and `pos.in(slope).y` the height above it, which stays 0.

```text
space Plane = euclidean(2)

model Slope in Plane {
  param {
    g: Acceleration = 9.81 m/s^2
    θ: Angle        = 30 deg unit deg
  }
  state {
    pos: Point            = origin
    vel: Vector<Velocity> = 0
  }
  derived {
    along:   Vector<1>            = (cos(θ), -sin(θ))
    gravity: Vector<Acceleration> = (0 m/s^2, -g)
    down:    Acceleration         = gravity.in(slope).x
    into:    Acceleration         = gravity.in(slope).y
    s:       Length               = pos.in(slope).x
    above:   Length               = pos.in(slope).y
  }
  frame slope(at: origin, x: along)
  flow {
    der(pos) = vel
    der(vel) = down * along
  }
}

presentation Incline for Slope {
  until t0 + 1.5 s
  view side: spatial(Plane, scale: 1 m -> 60 px, y: up) {
    segment(origin, origin + 8 m * along) as ground
    marker(pos) as ball
    arrow(gravity, from: pos, scale: 1 m/s^2 -> 6 px) as g_full
    arrow(down * along, from: pos, scale: 1 m/s^2 -> 6 px, color: green) as g_along
  }
  panel numbers {
    slider(θ, range: [5 deg, 60 deg])
    text("along the slope: {down:.2}, into it: {into:.2}")
    text("distance down the slope: {s:.2}")
  }
  observe {
    travelled = s at t0 + 1 s
    lifted    = above at t0 + 1 s
  }
}
```

```cases
run released of Slope with Incline {
  until t0 + 1.5 s
  expect {
    travelled == 2.4525 m within 1e-6 m     // g sin 30° t² / 2 at t = 1 s
    lifted == 0 m within 1e-9 m              // the ball stays on the slope
  }
}
```

- `frame slope(at: origin, x: along)` sits beside the model's blocks, as a `grid` does. Its origin is a point, its axis a vector of the space; the axis's length does not matter, only its direction.
- `down = gravity.in(slope).x` is `g sin θ`, the part that speeds the ball up; `into = gravity.in(slope).y` is `-g cos θ`, the part the slope holds back.
- Moving the slider changes `θ`, the slope and its frame together: a frame's expressions are read like any derived value.

## A ride on a carousel

**Goal.** A ball rolls straight across the ground while a carousel turns under it. From the ground its path is a line; from a seat on the carousel it curves.
**How it is built.** The frame `turning` has its first axis at the angle `w (t - t0)`, so it turns at the rate `w`. One view shows the ground; the other, `frame: turning`, shows everything as seen from the carousel. Its trace reads each sample in the frame as it was at that instant, which is why the path curves.

```text
space Plane = euclidean(2)

model Carousel in Plane {
  param { w: Frequency = 0.5 /s }
  state {
    pos: Point            = origin + (1 m, 0 m)
    vel: Vector<Velocity> = (0 m/s, 1 m/s)
  }
  derived {
    turn:   Angle    = w * (t - t0)
    radial: Length   = pos.in(turning).x
    across: Velocity = vel.in(turning).y
  }
  frame turning(at: origin, x: (cos(turn), sin(turn)))
  flow {
    der(pos) = vel
    der(vel) = 0
  }
}

presentation Ride for Carousel {
  until t0 + 4 s
  view ground: spatial(Plane, scale: 1 m -> 50 px, y: up) {
    title("From the ground")
    axes
    trace(pos every 0.05 s)
    marker(pos) as ball
    segment(origin, origin + 2 m * (cos(turn), sin(turn))) as arm
  }
  view seat: spatial(Plane, scale: 1 m -> 50 px, y: up, frame: turning) {
    title("From the carousel")
    axes
    trace(pos every 0.05 s)
    marker(pos) as seen
  }
  panel controls { slider(w, range: [0 /s, 2 /s]) }
  observe { out = radial at t0 + 2 s }
}
```

```cases
run two_seconds of Carousel with Ride {
  until t0 + 2.5 s
  expect {
    out == 2.223244 m within 1e-6 m        // cos 1 + 2 sin 1: the ball at (1 m, 2 m), the frame turned 1 rad
  }
}
```

- `view seat: spatial(..., frame: turning)` draws every point and vector in the frame: its `axes` are the carousel's, turning with it, and what is at rest on the carousel stays still in the view.
- With `w` at 0 the two views agree. The faster the carousel turns, the more the path seen from the seat curls.

## A puck on a turntable

**Goal.** A puck on a frictionless turntable, described by someone turning with it. Released at rest on the turntable, it is flung outward by the centrifugal force and turned by the Coriolis force, so its path in the turning frame curls away.
**How it is built.** The model is written in the turning frame from the start: `pos` and `vel` are the puck's position and velocity as the rider measures them, and the flow adds the two forces of a frame turning at angular velocity `spin` (`coriolis` and `centrifugal` from the `mechanics` module, chapter 19). The quantity `jacobi`, the speed squared less the frame's own speed there squared, stays constant: a check that the forces are right.

```text
use mechanics

space Room = euclidean(3)

model Turntable in Room {
  param {
    m:    Mass        = 1 kg
    spin: Vector<1/T> = (0 /s, 0 /s, 1 /s)
  }
  state {
    pos: Point            = origin + (0.5 m, 0 m, 0 m)
    vel: Vector<Velocity> = 0
  }
  derived {
    turning: Vector<Force>      = coriolis(m, spin, vel)
    outward: Vector<Force>      = centrifugal(m, spin, pos - origin)
    jacobi:  Quantity<L^2/T^2>  = |vel|^2 - |cross(spin, pos - origin)|^2
  }
  flow {
    der(pos) = vel
    der(vel) = (turning + outward) / m
  }
}

presentation Spin for Turntable {
  until t0 + 6 s
  view top: spatial(Room, scale: 1 m -> 50 px, azimuth: -90 deg, elevation: 90 deg) {
    grid
    trace(pos every 0.05 s)
    marker(pos) as puck
    arrow(turning, from: pos, scale: 1 N -> 20 px, color: purple) as coriolis_force
    arrow(outward, from: pos, scale: 1 N -> 20 px, color: orange) as centrifugal_force
  }
  observe {
    first = jacobi at t0
    later = jacobi at t0 + 5 s
  }
}
```

```cases
run released of Turntable with Spin {
  until t0 + 5.5 s
  expect {
    first == -0.25 m^2/s^2 within 1e-12 m^2/s^2     // at rest, 0.5 m from the axis: -(1 × 0.5)^2
    later == -0.25 m^2/s^2 within 1e-5 m^2/s^2      // kept while the puck moves, up to the solver's error
  }
}
```

- `coriolis(m, spin, vel)` is `-2 m spin × vel`: at right angles to the motion, it turns the puck without changing its speed. `centrifugal(m, spin, r)` is `-m spin × (spin × r)`: away from the axis, growing with the distance from it.
- The forces exist only in the turning description. Seen from the ground, the same puck at rest on the turntable moves in a straight line at the speed the turntable had under it.
- `spin` is a vector along the axis of turning, as long as the rate: `(0 /s, 0 /s, 1 /s)` turns anticlockwise seen from above at 1 radian per second.
- Seen from the ground, the released puck keeps the speed the turntable gave it, `|spin| r0`, in a straight line, so its distance from the axis grows as `r0 sqrt(1 + (|spin| t)^2)`: 3 m after 6 s. Seen from the turntable, that line winds into the spiral the view draws.

## Frames in brief

| Written | Where | Meaning |
|---|---|---|
| `frame F(at: P, x: u)` | model | a frame of a plane: origin `P`, first axis along `u`, second axis `u` turned a quarter turn anticlockwise |
| `frame F(at: P, x: u, y: v)` | model | a frame of a space of three dimensions: the second axis is `v` made square to the first, the third their cross product |
| `v.in(F).x`, `p.in(F).y` | any expression | a component of a vector, or a coordinate of a point measured from `P`, along an axis of `F` |
| `spatial(..., frame: F)` | view option | the view is drawn as seen from `F`; traces read each sample in `F` as it was then |
| `coriolis(m, w, v)`, `centrifugal(m, w, r)` | `use mechanics` | the forces of a frame turning at angular velocity `w`, in three dimensions |

- A frame's expressions may read parameters, state, derived values and time. A frame that reads time moves; derived values that read components in it are updated with it.
- An axis of no length has no direction: components read in such a frame are not available.
- Frames belong to models; members of object types are read in a frame of the model, `b.pos.in(F).x`.

## Exercises

1. In the slope model, add a frame whose first axis is horizontal and read the ball's velocity in it. When is its vertical part largest?
2. In the carousel, set the ball's start velocity to `(0 m/s, 0.5 m/s)` and `w` to `0.5 /s`. Why does the ball seem to stay put on the carousel at first? (Hint: compare its speed with the speed of the carousel under it.)
3. Write the turntable puck again in the ground's frame, with no forces at all, and draw it in a view with `frame:` a turning frame. Compare its path with the one the forces give.
