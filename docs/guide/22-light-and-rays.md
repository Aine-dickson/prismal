# 22. Light and Rays

Light travels in straight lines until it meets a surface: there it is reflected, or it bends as it passes into another material. Ray diagrams follow a few rays through mirrors, lenses and prisms to show where images form and why a prism splits white light. This chapter draws them with the first-party module `optics`, written in Prismal like the `mechanics` module of chapter 19 (D-103).

> **In plain words.**
> - A **ray** is a starting point and a **direction**, a vector without a unit, `Vector<1>`, such as `(1, -1)`: as far right as down.
> - The **refractive index** `n` of a material says how much it slows light: about 1 for air, 1.33 for water, 1.5 for glass. Passing into a larger index, a ray bends toward the **normal**, the line at right angles to the surface; into a smaller one, away from it.
> - **Snell's law**, `n1 sin θ1 = n2 sin θ2`, gives the angles. From water into air, beyond the **critical angle** no ray gets out: it is **totally reflected**.
> - A **thin lens** bends every ray so that rays parallel to its axis meet at its **focus**, at the **focal length** `f` from it. The rays from one point of an object meet again at one point of its **image**.

## Bending at a water surface

**Goal.** A laser pointing at a water surface, which the learner drags: the ray bends into the water, and with water above and air below (`n1` and `n2` swapped) it is totally reflected past the critical angle.
**How it is built.** `use optics` imports the module's functions. The laser is a draggable point; the ray aims at the origin, where it meets the surface. `refract(along, (0, 1), n1, n2)` turns its direction by Snell's law, or reflects it when it cannot get out; `incidence` measures the angles from the normal.

```text
use optics

space Plane = euclidean(2)

model Refraction in Plane {
  param {
    laser: Point = origin + (-1 m, 1 m)
    n1:    Real  = 1 in [1, 2.5]
    n2:    Real  = 1.33 in [1, 2.5]
  }
  derived {
    along:     Vector<1> = direction((origin - laser) / 1 m)
    onward:    Vector<1> = refract(along, (0, 1), n1, n2)
    θ1:        Angle     = incidence(along, (0, 1)) unit deg
    θ2:        Angle     = incidence(onward, (0, 1)) unit deg
    reflected: Boolean   = totally_reflected(along, (0, 1), n1, n2)
    tip:       Point     = origin + 1.5 m * onward
  }
}

presentation RefractionLab for Refraction {
  view tank: spatial(Plane, scale: 1 m -> 150 px, y: up) {
    segment(origin + (-2 m, 0 m), origin + (2 m, 0 m), color: blue) as surface
    segment(origin + (0 m, -1.2 m), origin + (0 m, 1.2 m), line: dashed, color: gray) as normal
    segment(laser, origin, color: red, width: 2) as incident
    segment(origin, tip, color: red, width: 2) as onward_ray
    marker(laser) as source { on drag as p { propose laser = p } }
    text("Totally reflected", at: origin + (0.2 m, -0.4 m), when: reflected)
  }
  panel controls {
    slider(n1, range: [1, 2.5])
    slider(n2, range: [1, 2.5])
    label(θ1)
    label(θ2)
  }
  observe {
    angle_in  = θ1 at t0
    angle_out = θ2 at t0
    bounced   = reflected at t0
  }
}
```

```cases
run into_water of Refraction with RefractionLab {
  expect {
    angle_in == 0.785398 within 1e-6       // 45 degrees, in radians
    angle_out == 0.560558 within 1e-6      // sin θ2 = sin 45° / 1.33: 32.12 degrees
    bounced == false
  }
}
```

With water above (`n1 = 1.33`) and air below, the ray gets out while `1.33 sin θ1 < 1`: at 45 degrees it does (`1.33 sin 45° = 0.94`). The critical angle of water is `asin(1 / 1.33)`, 48.75 degrees; with the laser lower, at `origin + (-1 m, 0.8 m)`, the angle is 51.3 degrees and the ray is reflected:

```cases
run out_of_water of Refraction with RefractionLab {
  param {
    n1 = 1.33
    n2 = 1
    laser = origin + (-1 m, 0.8 m)
  }
  expect {
    bounced == true                        // 1.33 sin 51.3° = 1.04 > 1
    angle_out == 0.896055 within 1e-6      // reflected: the angle of incidence again
  }
}
```

- `direction(v)` scales a direction to length 1; `(origin - laser) / 1 m` turns the displacement into a direction without a unit.
- `incidence(d, n)` is the angle between a ray and the normal on the side the ray comes from. For the reflected ray it is the angle of reflection, equal to the angle of incidence.
- `unit deg` shows the angles in degrees in labels; in expectations they are numbers in radians, as every angle is (chapter 2).

## A lens and its image

**Goal.** A ray diagram of a converging lens: an object arrow 30 cm in front of a lens of focal length 10 cm, three rays from its tip, and the image where they meet, 15 cm behind the lens, half as tall and upside down.
**How it is built.** The lens is at the origin with its axis along `x`. Each ray starts at the object's tip with its own direction: along the axis, through the centre, through the near focus. `hit_plane` finds where it meets the lens's plane and `thin_lens` turns it there. `image_distance` and `magnification` give the image from `1/u + 1/v = 1/f`, so a check can compare where the rays cross with where the formula puts the image.

```text
use optics

space Plane = euclidean(2)

model Lens in Plane {
  param {
    u: Length = 30 cm in [12 cm, 60 cm]
    h: Length = 2 cm
    f: Length = 10 cm
  }
  derived {
    tip:   Point     = origin + (-u, h)
    axis:  Vector<1> = (1, 0)
    // The three rays: parallel, through the centre, through the near focus.
    d1:    Vector<1> = (1, 0)
    d2:    Vector<1> = direction((origin - tip) / 1 m)
    d3:    Vector<1> = direction((origin + (-f, 0 m) - tip) / 1 m)
    q1:    Point     = hit_plane(tip, d1, origin, axis)
    q2:    Point     = hit_plane(tip, d2, origin, axis)
    q3:    Point     = hit_plane(tip, d3, origin, axis)
    out1:  Vector<1> = thin_lens(d1, q1, origin, axis, f)
    out2:  Vector<1> = thin_lens(d2, q2, origin, axis, f)
    out3:  Vector<1> = thin_lens(d3, q3, origin, axis, f)
    v:     Length    = image_distance(u, f)
    image: Point     = origin + (v, magnification(u, f) * h)
    // Where each ray reaches the image's plane.
    at1:   Length    = (hit_plane(q1, out1, image, axis) - origin).y
    at2:   Length    = (hit_plane(q2, out2, image, axis) - origin).y
    at3:   Length    = (hit_plane(q3, out3, image, axis) - origin).y
  }
}

presentation LensLab for Lens {
  view bench: spatial(Plane, scale: 1 m -> 1200 px, y: up) {
    segment(origin + (-0.65 m, 0 m), origin + (0.4 m, 0 m), color: gray) as axis_line
    segment(origin + (0 m, -0.05 m), origin + (0 m, 0.05 m), color: blue, width: 3) as lens
    marker(origin + (f, 0 m)) as focus
    marker(origin + (-f, 0 m)) as near_focus
    arrow(tip - (origin + (-u, 0 m)), from: origin + (-u, 0 m), scale: 1 m -> 1200 px) as thing
    segment(tip, q1, color: red) as ray1_in
    segment(q1, q1 + 0.4 m * out1, color: red) as ray1_out
    segment(tip, q2, color: green) as ray2_in
    segment(q2, q2 + 0.4 m * out2, color: green) as ray2_out
    segment(tip, q3, color: purple) as ray3_in
    segment(q3, q3 + 0.4 m * out3, color: purple) as ray3_out
    marker(image) as image_tip
  }
  panel controls {
    slider(u, range: [12 cm, 60 cm])
    label(v)
  }
  observe {
    distance = v at t0
    height   = (image - origin).y at t0
    cross1   = at1 at t0
    cross2   = at2 at t0
    cross3   = at3 at t0
  }
}
```

```cases
run real_image of Lens with LensLab {
  expect {
    distance == 0.15 m within 1e-12 m      // 1/30 + 1/v = 1/10: v = 15 cm
    height == -0.01 m within 1e-12 m       // magnification -v/u = -1/2
    cross1 == -0.01 m within 1e-12 m       // all three rays pass through the image's tip
    cross2 == -0.01 m within 1e-12 m
    cross3 == -0.01 m within 1e-12 m
  }
}
```

- The three rays are the ones a textbook draws, because each is easy to follow by hand: along the axis, then through the far focus; straight through the centre; through the near focus, then along the axis. `thin_lens` treats every ray alike, so any other ray from the tip meets them at the image too.
- Drag `u` below `f` (10 cm): the rays leave the lens spreading out, `v` becomes negative and the image is **virtual**: on the object's side, upright, larger, seen by looking through the lens (a magnifying glass).
- A negative `f` is a diverging lens. `lensmaker(n, r1, r2)` gives `f` from the glass and the curvature of the faces.

## A prism splits white light

**Goal.** A ray of white light through a glass prism: red, green and violet are bent by different amounts, so they leave in different directions.
**How it is built.** The prism is an equilateral triangle with its apex at `(0 m, 0.5 m)`; its two faces are planes through the apex with normals `left_n` and `right_n`. The index of the glass depends on the wavelength, by Cauchy's formula `cauchy(A, B, λ)`. Each colour's ray enters at the same point, bends by its own index, crosses the glass, and bends again on the way out.

```text
use optics

space Plane = euclidean(2)

model Prism in Plane {
  param {
    source: Point     = origin + (-1.5 m, -0.2 m)
    aim:    Vector<1> = (1, 0.3)
    A:      Real      = 1.5046
    B:      Area      = 4200 nm^2
  }
  derived {
    apex:     Point     = origin + (0 m, 0.5 m)
    left_n:   Vector<1> = (-sqrt(3) / 2, 0.5)
    right_n:  Vector<1> = (sqrt(3) / 2, 0.5)
    entry:    Point     = hit_plane(source, aim, apex, left_n)
    n_red:    Real      = cauchy(A, B, 700 nm)
    n_green:  Real      = cauchy(A, B, 530 nm)
    n_violet: Real      = cauchy(A, B, 400 nm)
    in_red:    Vector<1> = refract(aim, left_n, 1, n_red)
    in_green:  Vector<1> = refract(aim, left_n, 1, n_green)
    in_violet: Vector<1> = refract(aim, left_n, 1, n_violet)
    exit_red:    Point = hit_plane(entry, in_red, apex, right_n)
    exit_green:  Point = hit_plane(entry, in_green, apex, right_n)
    exit_violet: Point = hit_plane(entry, in_violet, apex, right_n)
    out_red:    Vector<1> = refract(in_red, right_n, n_red, 1)
    out_green:  Vector<1> = refract(in_green, right_n, n_green, 1)
    out_violet: Vector<1> = refract(in_violet, right_n, n_violet, 1)
    turn_red:    Angle = angle_between(aim, out_red) unit deg
    turn_violet: Angle = angle_between(aim, out_violet) unit deg
  }
}

presentation PrismLab for Prism {
  view bench: spatial(Plane, scale: 1 m -> 200 px, y: up) {
    polygon(apex, origin + (-0.5 m, -0.366 m), origin + (0.5 m, -0.366 m), color: teal) as glass
    segment(source, entry, color: ink, width: 2) as white
    segment(entry, exit_red, color: red) as red_inside
    segment(entry, exit_violet, color: purple) as violet_inside
    segment(exit_red, exit_red + 1.5 m * out_red, color: red, width: 2) as red_ray
    segment(exit_green, exit_green + 1.5 m * out_green, color: green, width: 2) as green_ray
    segment(exit_violet, exit_violet + 1.5 m * out_violet, color: purple, width: 2) as violet_ray
  }
  panel numbers {
    label(turn_red)
    label(turn_violet)
  }
  observe {
    red_turn    = turn_red at t0
    violet_turn = turn_violet at t0
  }
}
```

```cases
run spectrum of Prism with PrismLab {
  expect {
    red_turn == 0.670629 within 1e-6       // 38.42 degrees for n = 1.5132
    violet_turn == 0.699295 within 1e-6    // 40.07 degrees for n = 1.5309: violet is turned more
  }
}
```

- The spread between red and violet is **dispersion**: the index of glass is larger for shorter wavelengths, so violet bends more at both faces.
- `4200 nm^2` is Cauchy's `B` for a common glass; wavelengths are written in `nm`.

## What the module holds

| Function | What it gives |
|---|---|
| `direction(d)` | `d` scaled to length 1 |
| `reflect(d, n)` | a ray along `d` reflected by a surface of normal `n` |
| `refract(d, n, n1, n2)` | Snell's law from index `n1` into `n2`; reflected beyond the critical angle |
| `totally_reflected(d, n, n1, n2)`, `critical_angle(n1, n2)` | total internal reflection |
| `incidence(d, n)`, `incidence_cos(d, n)`, `angle_between(u, v)` | angles of rays |
| `hit_plane(p, d, a, n)` | where a ray from `p` meets the plane (a line, in a plane) through `a` with normal `n` |
| `hit_sphere(p, d, c, r)`, `meets_sphere(p, d, c, r)`, `sphere_normal(q, c)` | the first point where a ray meets a sphere (a circle) ahead of it, whether it does, the normal there |
| `thin_lens(d, q, c, a, f)` | a ray crossing an ideal thin lens at `q` |
| `image_distance(u, f)`, `magnification(u, f)`, `lensmaker(n, r1, r2)` | images and focal lengths |
| `cauchy(A, B, λ)` | the index of a glass at a wavelength |

A curved mirror or a thick lens is a chain of `hit_sphere` and `reflect` or `refract`, one derived value per surface, as the prism's two faces are. A ray that misses a sphere has no point there: its drawing is not shown, and `meets_sphere` says so ahead of time.

## Exercises

1. Make the prism's angle of incidence a slider and find the angle where the red ray turns least (the **minimum deviation**). How does it compare with the angle where it leaves symmetrically?
2. Draw a concave mirror as a circle of radius 40 cm centred 40 cm in front of it, send rays parallel to its axis at several heights with `hit_sphere` and `reflect`, and find where they cross the axis. Why do the rays far from the axis cross nearer the mirror? (Spherical aberration.)
3. Give the lens lab a fourth ray from the object's tip at any angle, and check it also passes through the image.
