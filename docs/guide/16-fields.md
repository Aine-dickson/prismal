# 16. Fields

Some quantities have a value at every point of a region: the temperature along a metal rod, the height of the water in a tank, the concentration of a dye spreading in a dish. Such a quantity is a **field**. Prismal stores a field on a **grid**: the region cut into equal cells, with one value per cell. The field changes by a flow, as any state does, and its rate in each cell can read the cells around it.

> **In plain words.**
> - A **field** is a value at every point: `T(x)` is the temperature at `x`.
> - A **grid** cuts a line or a rectangle into equal **cells**. More cells give a finer picture and cost more time.
> - The **Laplacian**, `laplacian(T)`, compares the value in a cell with its neighbours: it is positive where the cell is colder than its surroundings. Heat flows down hill, so `der(T) = alpha * laplacian(T)` is the heat equation: every cell moves towards the average of its neighbours.
> - **Boundary conditions** say what happens at the edges: held at a temperature, insulated (nothing crosses), or wrapped around (periodic).

## A heated rod

**Goal.** A metal rod at room temperature, one end held at a temperature the learner chooses: the learner watches the heat creep along the rod and sees the temperature settle to a straight line.
**How it is built.** `grid rod(...)` in the model cuts the rod into 50 cells. The state `T(x in rod)` is a field over that grid, starting at 20 everywhere; `x` is the position, which an initial value may use. One flow, the heat equation, says how every cell changes. `boundary T { ... }` holds the two ends. The presentation draws the field with `function_graph(T)`, as any function of position.

```text
model Rod {
  grid rod(x: [0 m, 1 m], cells: 50)
  param {
    alpha: Quantity<L^2/T> = 0.002 m^2/s    // thermal diffusivity
    hot:   Real            = 100 in [20, 200]
  }
  state { T(x in rod): Real = 20 }
  derived {
    middle: Real = T(0.5 m)
    warmth: Length = integral(T)
  }
  flow der(T) = alpha * laplacian(T)
  boundary T {
    left = hot
    right = 20
  }
  equation heat: der(T) == alpha * laplacian(T)
}

presentation RodLab for Rod {
  until t0 + 120 s
  view temperature: plot(x: [0 m, 1 m], y: [0, 210]) {
    function_graph(T)
  }
  view middle_plot: plot(x: [0 s, 120 s], y: [0, 210]) {
    series_plot(middle every 1 s)
  }
  panel controls {
    slider(hot, range: [20, 200])
    label(middle)
    equation(heat)
  }
  observe {
    settled = T(0.25 m) at t0 + 400 s
    early   = middle at t0 + 10 s
  }
}
```

```cases
run settles of Rod with RodLab {
  until t0 + 400 s
  expect {
    settled == 80 within 0.5
    early in [20, 25]
  }
}
```

- `grid rod(x: [0 m, 1 m], cells: 50)` is the rod from 0 m to 1 m in 50 cells of 2 cm. The cells' centres are at 1 cm, 3 cm, ..., 99 cm.
- `T(x in rod): Real = 20` declares the field and its starting value in each cell. The starting value may depend on the position: `T(x in rod): Real = 20 + 80 * exp(-((x - 0.5 m) / 0.1 m)^2)` starts with a warm spot in the middle.
- `laplacian(T)` is written only where the field is, in its flow or in a field's definition (MK-E32): it reads the neighbouring cells of the cell being computed.
- `T(0.5 m)` reads the field at a position, between the cells' centres. `integral(T)`, `mean(T)`, `min(T)` and `max(T)` read the whole field; `integral(T)` has the unit of `T` times a length.
- With the left end held at 100 and the right at 20, the rod settles to a straight line from 100 to 20: at 0.25 m, 80.
- The equation is typeset as `∂T/∂t = α ∇²T` when the binding has the symbol `α` (`alpha: ... symbol "α"`).
- `until t0 + 120 s` sets how long the lab's run lasts. Without it, a lab runs 60 s or as long as its longest time axis, and a model with a large field (more than 4096 values) 20 s.

## A vibrating string

**Goal.** A guitar string, held at both ends and pulled into an arc: the learner sees it swing through and back, a standing wave.
**How it is built.** A wave needs two fields: the displacement `u` and its speed `v`, as a pendulum needs an angle and an angular speed. The flows say that `u` changes at speed `v`, and that `v` changes at `c^2` times the Laplacian of `u`: the wave equation.

```text
model String {
  grid line(x: [0 m, 1 m], cells: 100)
  param { c: Velocity = 2 m/s }               // wave speed
  state {
    u(x in line): Length = 0.02 m * sin(pi * x / 1 m)
    v(x in line): Velocity = 0 m/s
  }
  derived { middle: Length = u(0.5 m) }
  flow {
    der(u) = v
    der(v) = c^2 * laplacian(u)
  }
  boundary u {
    left = 0 m
    right = 0 m
  }
}

presentation StringLab for String {
  until t0 + 4 s
  view string: plot(x: [0 m, 1 m], y: [-0.03 m, 0.03 m]) {
    function_graph(u)
  }
  observe { half = middle at t0 + 0.5 s }
}
```

```cases
run swings of String with StringLab {
  until t0 + 0.5 s
  expect { half == -0.02 m within 0.0002 m }
}
```

- A string of length `L` swings back and forth with period `2 L / c`, here 1 s: after half a period the arc is upside down.
- Only `u` has boundary conditions: `laplacian(u)` reads beyond the edges, `v` is never differentiated.

## A ripple tank

**Goal.** A square tank of water seen from above: the learner clicks to drop a stone and watches the circular ripple spread, reflect from the walls and interfere with the next one.
**How it is built.** A grid with two axes lies in the model's plane, so the model is `in` a space of two dimensions. The coordinate of a cell is a point, `p`. The fields and the wave equation are those of the string. A requested event `drop` receives the clicked point and adds a bump to the water in every cell, `set h(p) = ...`. The presentation draws the water with `heatmap(h)`: one coloured square per cell.

```text
space Plane = euclidean(2)

model Tank in Plane {
  grid tank(x: [0 m, 1 m], y: [0 m, 1 m], cells: (60, 60))
  param { c: Velocity = 0.3 m/s }
  state {
    h(p in tank): Length = 0 m
    v(p in tank): Velocity = 0 m/s
  }
  flow {
    der(h) = v
    der(v) = c^2 * laplacian(h)
  }
  boundary h {
    left = 0 m
    right = 0 m
    bottom = 0 m
    top = 0 m
  }
  event drop on request(q: Point) {
    set h(p) = h + 0.01 m * exp(-(|p - q| / 0.04 m)^2)
  }
}

presentation TankLab for Tank {
  until t0 + 20 s
  view tank: spatial(Plane, scale: 1 m -> 400 px, y: up) {
    heatmap(h, range: [-0.004 m, 0.004 m])
    on click as q request drop(q)
  }
  panel controls { slider(c, range: [0.1 m/s, 0.6 m/s]) }
}
```

- `grid tank(x: [...], y: [...], cells: (60, 60))` is a square of 60 by 60 cells in the model's plane; `cells: 60` gives the same count on both axes.
- `h(p in tank)` makes `p` a point: `|p - q|` is the distance from the cell to the clicked point.
- `set h(p) = h + ...` in a handler sets the field in every cell; `h` on the right is the value in the cell before the event.
- A heatmap colours each cell by its value. Values on both sides of 0 use a diverging palette, blue below and red above, so troughs and crests show; values of one sign use a sequential palette. `range: [...]` fixes the scale; without it the scale follows the field. The text alternative states the range of values and where the highest is.
- The edges of a grid in the plane are `left`, `right`, `bottom` and `top`; `boundary h periodic` wraps the tank around on both axes instead.

## How fields are computed

- A grid's cells are equal, and a field's value is the value at each cell's centre. `laplacian` and `gradient` take differences between neighbouring cells (second-order central differences). Beyond an edge held at `g` the difference reads `2 g` minus the inside cell, so the value on the edge is `g`; beyond an insulated edge it reads the inside cell again, so nothing crosses; a periodic grid reads the cell on the far side.
- Every cell is a component of the state, and the same solver advances them all. Finer grids are more accurate and slower: halving the cells' size doubles the cells on a line, quadruples them in the plane, and, for diffusion, makes each step about four times shorter.
- A run whose state has more than 4096 values keeps, between the solver's steps, the straight line from one state to the next (values at the steps are exact), and a lab of such a model runs 20 s unless its presentation says `until`.
- `noise` in a field's flow and random draws in a field's `set` are not available yet: each cell would need its own random numbers (MK-E29, MK-E28).

## Exercises

1. Make the rod's right end insulated instead of held at 20 (remove it from `boundary T`). What temperature does the whole rod reach in the end?
2. Pluck the string in the middle instead of bending it into an arc: start `u` as a triangle, `0.02 m * (1 - |x - 0.5 m| / 0.5 m)`. What does it look like after a quarter period?
3. In the tank, make the walls absorb nothing and wrap around instead: `boundary h periodic`. Drop a stone near a wall and watch where the ripple comes back.

Worked solutions to these exercises are in [chapter 11](11-solutions.md#chapter-16).
