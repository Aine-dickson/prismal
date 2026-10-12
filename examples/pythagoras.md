# Why a² + b² = c²

A complete example: a proof of Pythagoras' theorem by rearrangement, as a lab where the triangle can be reshaped and as a narrated lesson where the pieces move. Every program on this page is compiled and its cases run by `cargo test` (`crates/prismal-web/tests/examples.rs`).

**Goal.** A learner sees why, in a right triangle with legs `a` and `b` and hypotenuse `c`, the areas `a²` and `b²` add up to `c²`: four copies of the triangle in a square of side `a + b` leave a square of side `c` uncovered; moved around, they leave two squares, of sides `a` and `b`.
**How it is built.**

- *the triangle*: a model with the legs `a` and `b` as parameters and the hypotenuse `c` derived from them; nothing moves in time;
- *the two arrangements*: polygons whose corners are written with `a` and `b`, so that every piece follows the sliders;
- *the lab*: both arrangements side by side, sliders and handles for `a` and `b`, and live texts with the three areas;
- *the lesson*: the first arrangement, a camera move into the uncovered square, the triangles morphed into the second arrangement, the two squares revealed, then the learner's own triangle.

## The model

```text
space Plane = euclidean(2)

model RightTriangle in Plane {
  param {
    a: Length = 3 m   in [1 m, 6 m]
    b: Length = 4 m   in [1 m, 6 m]
  }
  derived {
    c:    Length = sqrt(a^2 + b^2)     // the hypotenuse
    side: Length = a + b               // the side of the big square
    gap:  Area   = side^2 - 4 * (a * b / 2) - c^2    // what the first arrangement leaves, minus c²
  }
}
```

- The model holds no state: a figure is a function of its parameters. A run of it only evaluates the derived values.
- `gap` is the big square minus the four triangles minus `c²`. The proof says it is zero for every triangle; the cases check that.

## The lab

The big square has its lower left corner at the origin. In the first arrangement the triangles sit in its four corners; in the second they pair up into two `a` by `b` rectangles.

```text
presentation PythagorasLab for RightTriangle {
  title "Two ways to fill a square"
  view first: spatial(Plane, scale: 1 m -> 24 px, y: up) {
    title("Four triangles and c²")
    polygon(origin, origin + (side, 0 m), origin + (side, side), origin + (0 m, side), color: gray)
    polygon(origin, origin + (a, 0 m), origin + (0 m, b), color: blue)
    polygon(origin + (side, 0 m), origin + (side, a), origin + (a, 0 m), color: blue)
    polygon(origin + (side, side), origin + (b, side), origin + (side, a), color: blue)
    polygon(origin + (0 m, side), origin + (0 m, b), origin + (b, side), color: blue)
    polygon(origin + (a, 0 m), origin + (side, a), origin + (b, side), origin + (0 m, b), color: red)
    text("c²", at: origin + (side / 2, side / 2))
    marker(origin + (a, 0 m), label: "a") { on drag as p { propose a = p.x } }
    marker(origin + (0 m, b), label: "b") { on drag as p { propose b = p.y } }
  }
  view second: spatial(Plane, scale: 1 m -> 24 px, y: up) {
    title("Four triangles, a² and b²")
    polygon(origin, origin + (side, 0 m), origin + (side, side), origin + (0 m, side), color: gray)
    polygon(origin + (0 m, a), origin + (a, a), origin + (0 m, side), color: blue)
    polygon(origin + (side, 0 m), origin + (side, a), origin + (a, 0 m), color: blue)
    polygon(origin + (a, side), origin + (0 m, side), origin + (a, a), color: blue)
    polygon(origin + (a, a), origin + (side, a), origin + (a, 0 m), color: blue)
    polygon(origin, origin + (a, 0 m), origin + (a, a), origin + (0 m, a), color: orange)
    polygon(origin + (a, a), origin + (side, a), origin + (side, side), origin + (a, side), color: orange)
    text("a²", at: origin + (a / 2, a / 2))
    text("b²", at: origin + (a + b / 2, a + b / 2))
  }
  panel controls {
    title("Your triangle")
    slider(a, range: [1 m, 6 m])
    slider(b, range: [1 m, 6 m])
    text("a² + b² = {a^2 + b^2:.2}")
    text("c² = {c^2:.2}")
    text("Each triangle: {a * b / 2:.2}")
  }
  layout row(first, second, controls)
  observe {
    gaps  = gap at t0
    hyps  = c   at t0
  }
}
```

- Every corner is written with `a`, `b` and `side`, so moving a slider or a handle redraws both arrangements at once.
- The two handles sit on the corners of the lower left triangle. `propose a = p.x` reads the dragged position as a length along the bottom edge; `b` along the left edge.
- Both views have the same scale, so the two big squares are the same size on the page.

The cases check that nothing is left over, for three triangles:

```cases
run three_four of RightTriangle with PythagorasLab {
  until t0 + 1 s
  expect {
    hyps == 5 m within 1e-12 m
    gaps == 0 m^2 within 1e-12 m^2
  }
}

run thin of RightTriangle with PythagorasLab {
  param { a = 1 m; b = 6 m }
  until t0 + 1 s
  expect { gaps == 0 m^2 within 1e-12 m^2 }
}

run isosceles of RightTriangle with PythagorasLab {
  param { a = 2 m; b = 2 m }
  until t0 + 1 s
  expect {
    hyps == 2.82842712474619 m within 1e-12 m    // 2√2
    gaps == 0 m^2 within 1e-12 m^2
  }
}
```

## The lesson

```text
presentation PythagorasLesson for RightTriangle {
  title "Why a² + b² = c²"
  view figure: spatial(Plane, scale: 1 m -> 40 px, y: up) {
    title("A square of side a + b")
    polygon(origin, origin + (side, 0 m), origin + (side, side), origin + (0 m, side), color: gray) as frame
    polygon(origin, origin + (a, 0 m), origin + (0 m, b), color: blue) as t1
    polygon(origin + (side, 0 m), origin + (side, a), origin + (a, 0 m), color: blue) as t2
    polygon(origin + (side, side), origin + (b, side), origin + (side, a), color: blue) as t3
    polygon(origin + (0 m, side), origin + (0 m, b), origin + (b, side), color: blue) as t4
    polygon(origin + (a, 0 m), origin + (side, a), origin + (b, side), origin + (0 m, b), color: red) as hole
    polygon(origin + (0 m, a), origin + (a, a), origin + (0 m, side)) as r1
    polygon(origin + (a, side), origin + (0 m, side), origin + (a, a)) as r3
    polygon(origin + (a, a), origin + (a, 0 m), origin + (side, a)) as r4
  }
  panel areas {
    title("Areas")
    text("a² = {a^2:.0}, b² = {b^2:.0}, c² = {c^2:.0}")
  }
  layout row(figure, areas)
  permit learner { timeline_controls }

  timeline {
    scene first {
      beat intro {
        hide r1; hide r3; hide r4
        narrate "Take a right triangle with legs a and b, and make four copies. They fit in the corners of a square with side a plus b." for 8.5 s
      }
      beat look {
        highlight hole
        camera figure to origin + (side / 2, side / 2) zoom 1.3 for 1.5 s
        narrate "What they leave uncovered is a square. Its side is the hypotenuse c, so its area is c squared." for 7.5 s
      }
      beat back { camera figure to origin + (side / 2, side / 2) zoom 1 for 1 s }
    }
    scene second {
      beat move {
        hide hole for 1 s
        animate t1 morph to r1 for 3 s
        animate t3 morph to r3 for 3 s
        animate t4 morph to r4 for 3 s
        narrate "Now slide three of the triangles. Nothing is cut and nothing is added." for 5.5 s
      }
      beat squares {
        reveal fade for 1 s in figure {
          polygon(origin, origin + (a, 0 m), origin + (a, a), origin + (0 m, a), color: orange) as square_a
          polygon(origin + (a, a), origin + (side, a), origin + (side, side), origin + (a, side), color: orange) as square_b
        }
        narrate "The space left is two squares: one with side a, one with side b." for 5.5 s
      }
      beat small {
        camera figure to origin + (a / 2, a / 2) zoom 1.6 for 1.5 s
        highlight square_a
        narrate "This one has area a squared," for 3 s
      }
      beat large {
        camera figure to origin + (a + b / 2, a + b / 2) zoom 1.3 for 1.5 s
        highlight square_b
        narrate "and this one b squared." for 3 s
      }
      beat close {
        camera figure to origin + (side / 2, side / 2) zoom 1 for 1 s
        narrate "The big square and the four triangles never changed, so neither did the space they leave. a squared plus b squared equals c squared." for 9.5 s
      }
    }
    scene yours {
      beat choose {
        hold
        explore limit 30 s keep a, b {
          slider(a, range: [1 m, 6 m])
          slider(b, range: [1 m, 6 m])
        } fallback {
          sequence { intervene { set a = 2 m; set b = 5 m }; wait 2 s }
        }
      }
      beat end { narrate "Whatever the triangle, the pieces fit the same way." for 4 s }
    }
  }
}
```

- `r1`, `r3` and `r4` are only there to give the morphs their shapes: each is the place of a triangle in the second arrangement. They are hidden in the first beat. `t2` is already in its place.
- Each morph moves a triangle's corners in order to the target's corners. The corners are listed so that each triangle slides without turning.
- The cameras move to points written with `a` and `b`, so they frame the right squares for any triangle.
- After the morphs, `t1`, `t3` and `t4` have the shapes of `r1`, `r3` and `r4`, which are written with `a` and `b`: in the explore beat, the rearranged figure follows the learner's sliders.

The lesson's timing: 8.5 s, 7.5 s, 1 s, 5.5 s, 5.5 s, 3 s, 3 s, 9.5 s, then the explore beat. Each narration is given as long as a system voice takes to say it, so that the exported video does not cut it short.

```cases
run lesson of RightTriangle with PythagorasLesson {
  expect {
    end of back    == 17 s exactly
    end of move    == 22.5 s exactly
    end of close   == 43.5 s exactly
  }
}
```

## Exporting the lesson

```sh
cargo run --release -p prismal-media -- examples/pythagoras.md PythagorasLesson pythagoras.mp4 --speech system
```

The video plays the explore beat's fallback: a triangle with legs 2 m and 5 m.
