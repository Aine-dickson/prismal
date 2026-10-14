# 20. Three Dimensions

Every space so far was a plane. Some motion does not stay in one: a charge in a magnetic field spirals along it, a top's axis sweeps a cone, a satellite's orbit is tilted. This chapter declares a space of three dimensions, computes with vectors in it, and shows it in a view turned to look from any side (D-100).

> **In plain words.**
> - `space Room = euclidean(3)` is a space with three axes, `x`, `y` and `z`. Points have three coordinates and vectors three components: `(1 m, 0 m, 2 m)`.
> - `cross(a, b)` is the **cross product** of two vectors: a vector at right angles to both, as long as the area of the parallelogram they span. Forces in magnetism and turning effects (torques) are written with it.
> - A view of a three-dimensional space is a picture taken from one side: `azimuth` says how far round the viewer stands, `elevation` how high. `z` is always up on the screen.

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

## Views of three dimensions

- `spatial(Room, scale: 1 m -> 40 px)` looks from azimuth 30 deg and elevation 20 deg. `azimuth: a` turns the viewer around the `z` axis, from the side of `x` (0 deg) toward the side of `y` (90 deg); `elevation: e` raises the viewer from the floor (0 deg) to straight above (90 deg). `z` is up on the screen.
- The picture is an orthographic projection: sizes do not shrink with distance, so lengths along each axis can be compared, as in a textbook figure.
- `axes` draws the three axes through the origin, labelled; `grid` draws the floor, the `x y` plane at `z = 0`.
- Markers, segments, polylines, polygons, traces and arrows take points and vectors of the space. `circle(P, r)` is a sphere's outline: a disc of radius `r` facing the viewer.
- Ellipses, arcs, heatmaps and groups lie in a plane, and a view of three dimensions does not draw them yet (PK-E05). `y: up` belongs to views of a plane.
- A click or a drag in a view of three dimensions lands in the plane through the origin that faces the viewer.

## Exercises

1. Give the particle no speed along `z`. What does its path become? And with no speed across the field, only along it?
2. Add a second view of the same room from straight above, `azimuth: -90 deg, elevation: 90 deg`. Why does the path look like a circle there? (Hint: which axis points at the viewer?)
3. Write a torque `τ = r × F` for a force `F` applied at a point `r` from a pivot, and draw both and the torque as arrows. Which way does the torque point when the force would turn the lever anticlockwise seen from above?
