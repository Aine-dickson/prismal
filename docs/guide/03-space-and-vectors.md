# 3. Space, Points and Vectors

Geometry in Prismal distinguishes **points** (places) from **vectors** (displacements, velocities, forces). A point minus a point is a vector; a point plus a vector is a point; two points cannot be added. Both belong to a **space**, which gives them their axes.

> **In plain words.**
> - A **point** is a place, like a street address. A **vector** is a step with a size and a direction, like "3 blocks east and 4 blocks north". Subtracting two addresses gives the step from one to the other; adding a step to an address gives a new address; adding two addresses means nothing, so Prismal refuses it.
> - Velocities and forces are vectors too: they have a size and a direction.
> - A **space** is the flat surface (or volume) the points live in. **Euclidean** means ordinary school geometry: straight lines, right angles, distances measured with a ruler. The **origin** is the point at (0, 0).
> - The **components** of a vector are its parts along each axis: `(3 m, 4 m)` has `x` component 3 m and `y` component 4 m. Its **norm**, written `|v|`, is its length: 5 m here.

## Spaces

```prismal
space Plane = euclidean(2)
```

declares a two-dimensional Euclidean space named `Plane`, with axes `x` and `y` and an origin. A model written `model Triangle in Plane { ... }` works in that space: `Point`, `Vector<D>` and `origin` refer to it.

## Points and vectors

| Written | Type | Meaning |
|---|---|---|
| `origin` | `Point` | the space's origin |
| `origin + (4 m, 0 m)` | `Point` | the point 4 m along `x` |
| `(3 m, 0 m)` | `Vector<Length>` when a vector is expected | values in brackets (a **tuple**) are a vector where a vector is needed |
| `B - A` | `Vector<Length>` | from `A` to `B` |
| `A + v` | `Point` | `A` moved by `v` |
| `2 * v`, `v / 2` | `Vector<Length>` | scaled |
| `v.x`, `A.y` | `Length` | a component along an axis (for a point, its coordinate) |
| `\|v\|` | `Length` | the norm (length) of `v` |
| `0` | any vector type | the zero vector |

`Vector<Velocity>` is a velocity vector, `Vector<Force>` a force. The dimension of a vector's components is part of its type: adding a velocity vector to a length vector is a dimension error.

## A program

**Goal.** A learner drags the corners of a triangle and watches its centre, perimeter and area follow.
**How it is built.** The corners are `Point` parameters (`param`); the centre and measures are `derived`. In a `spatial` view, each corner is a `marker` with `on drag as p { propose A = p }`, which turns a drag into a new value of the parameter; `segment`s draw the sides; `label`s in a `panel` show the measures.

A triangle whose vertices the learner can drag; the centroid, side lengths and area follow.

```text
space Plane = euclidean(2)

model Triangle in Plane {
  param {
    A: Point = origin
    B: Point = origin + (4 m, 0 m)
    C: Point = origin + (0 m, 3 m)
  }
  derived {
    G:         Point          = A + ((B - A) + (C - A)) / 3   // centroid
    ab:        Length         = |B - A|
    bc:        Length         = |C - B|
    ca:        Length         = |A - C|
    perimeter: Length         = ab + bc + ca
    area:      Quantity<L^2>  = abs((B - A).x * (C - A).y - (B - A).y * (C - A).x) / 2
  }
}
```

- The **centroid** is the triangle's balance point, the average of its corners. It is a point plus a vector: `(B - A) + (C - A)` is a sum of vectors, divided by 3, then added to `A`. Writing `(A + B + C) / 3` would add points and is rejected.
- `area` uses the **cross product** of two sides, written out with components: for two sides `u` and `v` from one corner, `u.x * v.y - u.y * v.x` is the area of the parallelogram they span, and half of it is the triangle's.

## Showing it in space

```text
presentation TriangleLab for Triangle {
  view scene: spatial(Plane, scale: 1 m -> 50 px, y: up) {
    grid; axes
    segment(A, B); segment(B, C); segment(C, A)
    marker(A) as a { on drag as p { propose A = p } }
    marker(B) as b { on drag as p { propose B = p } }
    marker(C) as c { on drag as p { propose C = p } }
    marker(G) as centroid
    arrow(G - A, from: A)
  }
  panel measures {
    label(perimeter)
    label(area)
  }
  observe {
    gx    = G.x       live
    gy    = G.y       live
    per   = perimeter live
    size  = area      live
  }
}
```

- `spatial(Plane, scale: 1 m -> 50 px, y: up)` is a **spatial view** of `Plane`: one metre is drawn as 50 pixels, and `y` points up on screen.
- `grid; axes` draw the coordinate grid and axes.
- `segment(A, B)` draws a straight segment between two points; `marker(G)` a point; `arrow(v, from: P)` a vector drawn from a point. An arrow of a length vector uses the view's scale; an arrow of any other dimension needs its own, `arrow(vel, from: pos, scale: 1 m/s -> 4 px)` (chapter 4).
- `as a` names a representation, so that a timeline or a drag can refer to it.
- `on drag as p { propose A = p }` declares an **inverse**: dragging the marker gives the pointer's position as the point `p`, and the drag proposes a new value for `A`. The proposal is checked like any other change: an invalid one is shown as invalid and not applied. Dragging an arrow by its head is written `on drag head as h { propose u = h - A }`.
- Every drag can also be done from the keyboard: focus the marker and use the arrow keys (10 px per step in a spatial view).

```cases
run right_triangle of Triangle with TriangleLab {
  expect {
    gx   == 1.33333333333333 m within 1e-12 m
    gy   == 1 m within 1e-12 m
    per  == 12 m within 1e-12 m                   // 4 + 3 + 5
    size == 6 m^2 within 1e-12 m^2
  }
}

run moved_vertex of Triangle with TriangleLab {
  param { C = origin + (4 m, 3 m) }
  expect {
    gx   == 2.66666666666667 m within 1e-12 m
    size == 6 m^2 within 1e-12 m^2               // same base, same height
  }
}
```

A case sets a point parameter with the same syntax as the model: `C = origin + (4 m, 3 m)`.

## Mistakes

```error
// error: MK-E04
space Plane = euclidean(2)
model Wrong in Plane {
  param { A: Point = origin; B: Point = origin + (1 m, 2 m) }
  derived { S: Point = A + B }
}
```

Two points cannot be added (`adding two points`). The midpoint is `A + (B - A) / 2`.

```error
// error: MK-E01
space Plane = euclidean(2)
model Wrong in Plane {
  param { A: Point = origin; B: Point = origin + (1 m, 2 m) }
  derived { d: Length = B - A }
}
```

`B - A` is a vector, not a length; the distance is `|B - A|`.

```error
// error: SX-E03
space Plane = euclidean(2)
model Wrong in Plane {
  param { A: Point = origin }
  derived { z: Length = A.z }
}
```

The plane has axes `x` and `y` only.

## Exercises

Solutions to the exercises not solved here are in [chapter 11](11-solutions.md).

1. Add the midpoint `M` of `A` and `B` to `Triangle` and draw the median from `C` to `M` as a segment.
2. Add the perpendicular height from `C` onto the line `AB`: `h: Length = 2 * area / ab`. Check it for the default triangle (`3 m`).
3. Write a model `Forces in Plane` with two force vectors `F1: Vector<Force> = (3 N, 0 N)` and `F2: Vector<Force> = (0 N, 4 N)`, their resultant `R = F1 + F2`, and a spatial view that draws all three from `origin` with `scale: 1 N -> 30 px`. Check `|R| == 5 N`.

<details>
<summary>A solution to exercise 3</summary>

```text
space Plane = euclidean(2)

model Forces in Plane {
  param {
    F1: Vector<Force> = (3 N, 0 N)
    F2: Vector<Force> = (0 N, 4 N)
  }
  derived {
    R: Vector<Force> = F1 + F2
    size: Force = |R|
  }
}

presentation ForceDiagram for Forces {
  view scene: spatial(Plane, scale: 1 m -> 30 px, y: up) {
    axes
    arrow(F1, from: origin, scale: 1 N -> 30 px) { on drag head as h { propose F1 = (h - origin) * (1 N / 1 m) } }
    arrow(F2, from: origin, scale: 1 N -> 30 px)
    arrow(R, from: origin, scale: 1 N -> 30 px)
  }
  observe { resultant = size live }
}
```

```cases
run three_four_five of Forces with ForceDiagram {
  expect { resultant == 5 N within 1e-12 N }
}
```

The drag of `F1`'s head gives a point `h` of the plane; `h - origin` is a length vector, and multiplying by `1 N / 1 m` turns it into a force at the drawing's scale of one metre of plane per newton.

</details>
