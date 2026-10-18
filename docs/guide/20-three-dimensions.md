# 20. Three Dimensions

Every space so far was a plane. Some motion does not stay in one: a charge in a magnetic field spirals along it, a top's axis sweeps a cone, a satellite's orbit is tilted. This chapter declares a space of three dimensions, computes with vectors in it, and shows it in a view turned to look from any side (D-100), in perspective, with bodies that hide what is behind them (D-104).

> **In plain words.**
> - `space Room = euclidean(3)` is a space with three axes, `x`, `y` and `z`. Points have three coordinates and vectors three components: `(1 m, 0 m, 2 m)`.
> - `cross(a, b)` is the **cross product** of two vectors: a vector at right angles to both, as long as the area of the parallelogram they span. Forces in magnetism and turning effects (torques) are written with it.
> - A view of a three-dimensional space is a picture taken from one side: `azimuth` says how far round the viewer stands, `elevation` how high. `z` is always up on the screen.
> - **Perspective** draws what is near larger, as an eye or a camera sees it; `distance` says how far away the viewer stands.
> - An **opaque** body hides what is behind it; other shapes are see-through.

## A charge in a magnetic field

**Goal.** A charged particle moving through a uniform magnetic field: it circles around the field while drifting along it, a helix, as in a cyclotron or the Earth's radiation belts.
**How it is built.** The magnetic force is `q v × B`, a derived vector `force = q * cross(vel, B)`. The field points along `z`; the particle starts moving along `x` with a little speed along `z`. A view of the room shows the floor grid, the axes, the particle's path and the force.

```text
space Room = euclidean(3)

model Cyclotron in Room {
  param {
    q: Charge          = 1 C
    m: Mass            = 2 kg
    B: Vector<M/T^2/I> = (0 kg/s^2/A, 0 kg/s^2/A, 0.5 kg/s^2/A)
  }
  state {
    pos: Point            = origin + (0 m, 4 m, 0 m)
    vel: Vector<Velocity> = (1 m/s, 0 m/s, 0.15 m/s)
  }
  derived { force: Vector<Force> = q * cross(vel, B) }
  flow {
    der(pos) = vel
    der(vel) = force / m
  }
}

presentation Spiral for Cyclotron {
  until t0 + 40 s
  view room: spatial(Room, scale: 1 m -> 40 px, azimuth: 35 deg, elevation: 25 deg) {
    grid
    axes
    trace(pos every 0.05 s)
    circle(pos, 0.25 m) as charge
    arrow(force, from: pos, scale: 1 N -> 60 px) as F
  }
  observe {
    around = pos.y at t0 + 25.132741228718345 s
    up     = pos.z at t0 + 25.132741228718345 s
  }
}
```

```cases
run one_turn of Cyclotron with Spiral {
  until t0 + 26 s
  expect {
    around == 4 m within 1e-4 m                 // back where it started after 2 π m / (q B)
    up == 3.7699 m within 1e-4 m                // 0.15 m/s for one period, 8 π s
  }
}
```

- The force is always at right angles to the velocity, so the speed never changes: the field turns the particle without speeding it up.
- The circle's radius is `m v / (q B)` = 4 m, and one turn takes `2 π m / (q B)` = 8 π s whatever the speed: the reason a cyclotron can accelerate particles with a fixed frequency.
- The field's dimension, `M/T^2/I`, is the tesla's: `B: Vector<M/T^2/I>`, written in `kg/s^2/A`.

## A tilted orbit, seen from every side

**Goal.** A planet on a circular orbit tilted to the floor, around a sun that hides it when it passes behind. The view is in perspective, the learner can turn it by dragging, and a lesson flies the camera from nearly edge on to above.
**How it is built.** The planet's position is a derived point from its phase, which flows at a constant rate. In the presentation, `distance: 14 m` in the view puts the viewer 14 m from the origin, in perspective. The sun and the planet are spheres with `opaque: true`, so a body behind another is covered and the part of the orbit behind the sun is not drawn. The orbit itself is an `arc` all the way round in the plane of normal `n`: an arc is a line, where a `circle` with a `normal` would be a filled disc. `permit learner { orbit }` lets the learner turn the view; in the lesson, `camera room elevation 80 deg for 4 s` turns it.

```text
space Room = euclidean(3)

model TiltedOrbit in Room {
  param {
    R:    Length    = 3 m
    tilt: Angle     = 30 deg unit deg
    w:    Frequency = 0.5 /s
  }
  state { phase: Angle = 0 deg }
  derived {
    n:      Vector<1> = (0, -sin(tilt), cos(tilt))
    planet: Point     = origin + (R * cos(phase), R * cos(tilt) * sin(phase), R * sin(tilt) * sin(phase))
  }
  flow { der(phase) = w }
}

presentation Sky for TiltedOrbit {
  until t0 + 30 s
  view room: spatial(Room, scale: 1 m -> 45 px, azimuth: -60 deg, elevation: 25 deg, distance: 14 m) {
    grid
    arc(origin, R, from: 0 deg, to: 360 deg, normal: n) as path
    circle(origin, 1 m, opaque: true, color: orange) as sun
    circle(planet, 0.3 m, opaque: true, color: blue) as earth
  }
  panel controls { slider(tilt, range: [0 deg, 90 deg]) }
  permit learner { orbit; timeline_controls }
  observe { high = planet.z at t0 + 3.141592653589793 s }
}

presentation Around for TiltedOrbit {
  until t0 + 30 s
  view room: spatial(Room, scale: 1 m -> 45 px, azimuth: -60 deg, elevation: 3 deg, distance: 14 m) {
    grid
    arc(origin, R, from: 0 deg, to: 360 deg, normal: n) as path
    circle(origin, 1 m, opaque: true, color: orange) as sun
    circle(planet, 0.3 m, opaque: true, color: blue) as earth
  }
  timeline {
    scene tour {
      beat edge  { narrate "Nearly edge on, the orbit is a thin ellipse." for 3 s }
      beat above { camera room elevation 80 deg for 4 s ease smooth; narrate "From above it is a circle." for 3 s }
      beat round { camera room azimuth 200 deg for 6 s; hold }
    }
  }
}
```

```cases
run quarter of TiltedOrbit with Sky {
  until t0 + 4 s
  expect {
    high == 1.5 m within 1e-6 m       // a quarter turn: R sin(tilt) above the floor
  }
}
```

- `distance: 14 m` makes the view a perspective: what is nearer the viewer is drawn larger. The plane through the origin facing the viewer keeps the view's `scale`; a point 1 m nearer than it is enlarged by `14 / 13`.
- `opaque: true` makes a circle (a sphere), an ellipse or a polygon a body that hides what lies behind it. Bodies are drawn from the farthest to the nearest; lines, markers and arrows are drawn over them, with the pieces a body hides cut away.
- `arc(origin, R, from: 0 deg, to: 360 deg, normal: n)` is the orbit's path, in the plane at right angles to `n`. With a `normal`, a circle, an ellipse or an arc lies in that plane; an ellipse or an arc without one lies in the floor.
- The slider tilts the orbit while it runs: `n` and `planet` are derived values, so the path and the planet follow.
- `camera room elevation 80 deg for 4 s` turns the view over 4 s; `camera room azimuth 200 deg for 6 s` takes it round. A turn starts from where the last one left the view.

## A door on a hinge

**Goal.** A door that swings about a vertical hinge as a slider turns it, and a ball the learner drags across the floor, which the door hides when it passes behind.
**How it is built.** The door is a `group` turned about the axis `(0, 0, 1)` by the parameter `swing`; its one member is an opaque polygon in the group's own frame. The ball's marker is dragged with `on drag across (0, 0, 1) as p`: the pointer moves it across the horizontal plane through where it was, so it stays on the floor whichever way the view looks.

```text
space Room = euclidean(3)

model Door in Room {
  param {
    swing: Angle = 30 deg unit deg
    ball:  Point = origin + (0.4 m, 1.6 m, 0 m)
  }
}

presentation Hinge for Door {
  view room: spatial(Room, scale: 1 m -> 60 px, azimuth: -35 deg, elevation: 25 deg) {
    grid
    axes
    group(at: origin, rotate: swing, axis: (0, 0, 1)) {
      polygon(origin, origin + (1.5 m, 0 m, 0 m), origin + (1.5 m, 0 m, 2 m), origin + (0 m, 0 m, 2 m), opaque: true) as panel
    }
    marker(ball) as b { on drag across (0, 0, 1) as p { propose ball = p } }
  }
  panel controls { slider(swing, range: [0 deg, 120 deg]) }
  permit learner { orbit }
}
```

- `group(at: origin, rotate: swing, axis: (0, 0, 1))` turns its members by `swing` about the vertical axis through `origin`. In a view of a plane a group turns about `z`, which is why `axis` belongs only in three dimensions.
- `on drag across (0, 0, 1) as p` names the plane the drag moves in by its normal; the plane passes through where the marker was when the drag began. Without `across`, a drag moves in the plane through that point facing the viewer.
- Dragging the ball behind the door hides it: the door is opaque.

## Views of three dimensions

| Written | Where | Meaning |
|---|---|---|
| `spatial(Room, scale: 1 m -> 40 px)` | view | looks from azimuth 30 deg and elevation 20 deg, `z` up on the screen |
| `azimuth: a`, `elevation: e` | view options | where the viewer stands: `a` turns round the `z` axis from the side of `x` (0 deg) toward the side of `y` (90 deg); `e` rises from the floor (0 deg) to straight above (90 deg). Either may read parameters and time: `azimuth: turn` follows a slider |
| `distance: d` | view option | the viewer stands `d` from the origin and the view is in perspective; without it, orthographic |
| `camera room azimuth a elevation e for d` | timeline action | turns the view over `d`, with `ease` as other animations; with `to P` and `zoom z` as in a plane |
| `permit learner { orbit }` | presentation | dragging an empty point of the view turns it, half a degree per pixel; "Reset view" turns it back |
| `opaque: true` | circle, ellipse, polygon option | a body that hides what lies behind it |
| `normal: n` | circle, ellipse, arc option | the plane the shape lies in, at right angles to `n` |
| `axis: n` | group option | the axis a group turns about, `z` by default |
| `on drag across n as p` | representation | the drag moves in the plane of normal `n` through where it began |

- The picture without `distance` is an orthographic projection: sizes do not shrink with distance, so lengths along each axis can be compared, as in a textbook figure. With `distance` the plane through the origin facing the viewer keeps the view's scale; nearer points are drawn larger.
- `axes` draws the three axes through the origin, labelled; `grid` draws the floor, the `x y` plane at `z = 0`.
- Markers, segments, polylines, polygons, traces and arrows take points and vectors of the space. `circle(P, r)` without a `normal` is a sphere's outline: a disc of radius `r` facing the viewer.
- Bodies (circles, closed ellipses and polygons) are drawn from the farthest to the nearest. Lines, markers, arrows and texts follow in the order written. What an opaque body hides is not drawn: the hidden pieces of lines are cut away, and markers and arrows whose tip is hidden are left out. A body that is not opaque is see-through.
- Heatmaps lie in a plane and a view of three dimensions does not draw them (PK-E05). `y: up` belongs to views of a plane.
- A click lands in the plane through the origin that faces the viewer; a drag in the plane through the dragged point, facing the viewer unless `across` names another.

## Exercises

1. Give the particle no speed along `z`. What does its path become? And with no speed across the field, only along it?
2. Add a second view of the same room from straight above, `azimuth: -90 deg, elevation: 90 deg`. Why does the path look like a circle there? (Hint: which axis points at the viewer?)
3. Write a torque `τ = r × F` for a force `F` applied at a point `r` from a pivot, and draw both and the torque as arrows. Which way does the torque point when the force would turn the lever anticlockwise seen from above?
4. In the tilted orbit, give the view `azimuth: 6 deg/s * (t - t0)`: what does the view do while the planet moves? Try `distance: 5 m`: why does the near side of the orbit look so much larger?
5. Add a second door to the hinge model that opens the other way, `rotate: -swing`. Drag the ball between the doors.
