# 24. Light as a Wave

Rays explain shadows, mirrors and lenses, but not the bright and dark lines light makes after passing two close slits, nor why a polarizing filter turned across another blocks the light. Those come from light being a wave: waves that meet add up, crest on crest or crest on trough. This chapter draws waves from two sources as a field, the patterns that slits and gratings throw on a screen, and polarizers, with the wave functions of the `optics` module (D-107).

> **In plain words.**
> - A wave repeats every **wavelength** `λ` along its way. Two waves that meet **interfere**: where their crests arrive together they add to a brighter wave, where a crest meets a trough they cancel.
> - Behind two narrow slits `d` apart, the light on a far screen is bright where the two paths differ by a whole number of wavelengths, and dark halfway between: **fringes**, about `λ L / d` apart on a screen at distance `L`.
> - One slit of width `a` spreads light too (**diffraction**): a wide bright middle, dark where `a sin θ` is a whole number of wavelengths. A **grating** of many slits makes sharp bright lines.
> - Light's wave shakes across its way, in a direction called its **polarization**. A polarizer passes the part along its own axis: `cos² θ` of the light, for a polarizer turned `θ` from it (**Malus's law**).

## Two sources, one pattern

**Goal.** Two sources sending out waves in step, as two slits do: lines where the waves always cancel fan out between lines where they add.
**How it is built.** A grid covers a stretch of water (the same holds for light, with wavelengths a million times shorter). The derived field `wave(p in tank)` adds the two waves at each cell: each is `cos(k r - w t)`, with `r` the distance from its source and `k = wavenumber(λ)`. The field reads time, so the pattern moves while the lines of calm stay put.

```text
use optics

space Plane = euclidean(2)

model Ripples in Plane {
  grid tank(x: [0 m, 1.2 m], y: [-0.6 m, 0.6 m], cells: (120, 120))
  param {
    λ: Length    = 0.1 m in [0.05 m, 0.3 m]
    d: Length    = 0.3 m in [0.1 m, 0.6 m]
    w: Frequency = 4 /s
  }
  derived {
    s1: Point = origin + (0 m, d / 2)
    s2: Point = origin + (0 m, -d / 2)
    wave(p in tank): Real = cos(wavenumber(λ) * |p - s1| - w * (t - t0)) + cos(wavenumber(λ) * |p - s2| - w * (t - t0))
    amplitude(p in tank): Real = 2 * |cos(wavenumber(λ) * (|p - s1| - |p - s2|) / 2)|
  }
}

presentation RippleLab for Ripples {
  until t0 + 20 s
  view tank: spatial(Plane, scale: 1 m -> 400 px, y: up) {
    heatmap(wave, range: [-2, 2])
    marker(s1) as source1
    marker(s2) as source2
  }
  panel controls {
    slider(λ, range: [0.05 m, 0.3 m])
    slider(d, range: [0.1 m, 0.6 m])
  }
  observe {
    middle = amplitude(origin + (1 m, 0 m)) at t0
    calm   = amplitude(origin + (1 m, 0.17 m)) at t0
  }
}
```

```cases
run in_step of Ripples with RippleLab {
  until t0 + 1 s
  expect {
    middle == 2 within 1e-2         // on the middle line both paths are equal: the waves add
    calm in [0, 0.1]                // near a line of calm the paths differ by half a wavelength
  }
}
```

- `wave(p in tank)` is a derived field: one value per cell, `p` the cell's centre. Reading `t` makes it change with time; nothing is integrated.
- `amplitude(p in tank)` is how far the water there swings: 2 where the waves arrive in step, 0 where they cancel. It depends only on the difference of the two paths. A field read at a point between cell centres is interpolated, so the case checks it to a hundredth.
- Along the middle line the two paths are equal, so the waves always add. Lines of calm lie where the paths differ by half a wavelength, one and a half, and so on.
- A shorter wavelength or sources further apart pack the lines closer together: drag the sliders.

## Fringes on a screen

**Goal.** The brightness on a screen 1 m behind two slits, one slit, and a grating of five slits, for light of a wavelength the learner chooses, drawn in its colour.
**How it is built.** The functions `two_slit`, `single_slit` and `grating` give the brightness, as a share of the brightest, at an angle `θ` from the middle. A point `y` millimetres up the screen is at the angle `atan2(y mm, L)`. Each is a derived function of `y` drawn by `function_graph`, coloured by `color: λ`.

```text
use optics

space Plane = euclidean(2)

model Screen in Plane {
  param {
    λ: Length = 600 nm in [380 nm, 750 nm] unit nm
    d: Length = 0.1 mm
    a: Length = 0.02 mm
    N: Real   = 5
    L: Length = 1 m
  }
  derived {
    θ(y: Real): Angle    = atan2(y * 1 mm, L)
    two(y: Real): Real   = two_slit(d, λ, θ(y))
    one(y: Real): Real   = single_slit(a, λ, θ(y))
    many(y: Real): Real  = grating(N, d, λ, θ(y))
    spacing: Length      = λ * L / d
  }
}

presentation ScreenLab for Screen {
  view screen: plot(x: [-40, 40], y: [0, 1.1]) {
    title("Brightness against height on the screen (mm)")
    function_graph(two, color: λ, label: "two slits")
    function_graph(one, color: gray, label: "one slit")
    function_graph(many, color: λ, line: dashed, label: "five slits")
  }
  panel controls {
    slider(λ, range: [380 nm, 750 nm])
    label(spacing)
  }
  observe {
    fringe = spacing at t0
    bright = two(6) at t0
    dark   = two(3) at t0
    edge   = one(30) at t0
    line   = many(6) at t0
  }
}
```

```cases
run red_light of Screen with ScreenLab {
  expect {
    fringe == 0.006 m within 1e-12 m       // λ L / d = 600 nm × 1 m / 0.1 mm = 6 mm
    bright == 1 within 1e-6                // the first bright fringe, 6 mm up
    dark == 0 within 1e-6                  // dark halfway, 3 mm up
    edge == 0 within 1e-6                  // one slit's first dark place: λ L / a = 30 mm
    line == 1 within 1e-6                  // a grating is bright where two slits are
  }
}
```

- The fringes of two slits are evenly spaced; their spacing grows with the wavelength, so red fringes lie further apart than blue ones. Drag `λ` and watch the colour and the spacing change together.
- One slit gives a wide middle band, as wide as the slit is narrow. A grating keeps the two slits' bright places but makes them sharp, with faint ripples between: more slits, sharper lines, which is why gratings sort colours in spectrometers.
- A real pair of slits of width `a` gives the two-slit fringes inside the one-slit envelope: `two(y) * one(y)`.

## Polarizers

**Goal.** Light through three polarizing filters. Two crossed filters block it all; a third turned halfway between them, slipped in the middle, lets some through again.
**How it is built.** Unpolarized light loses half at the first filter. Each next filter passes `malus(θ)` of what reaches it, `θ` the angle between it and the one before.

```text
use optics

space Plane = euclidean(2)

model Filters in Plane {
  param {
    I0:     Real  = 1
    first:  Angle = 0 deg unit deg
    middle: Angle = 45 deg unit deg in [0 deg, 90 deg]
    last:   Angle = 90 deg unit deg
  }
  derived {
    after_first:  Real = I0 / 2
    crossed:      Real = after_first * malus(last - first)
    after_middle: Real = after_first * malus(middle - first)
    through:      Real = after_middle * malus(last - middle)
    glare:        Angle = brewster(1, 1.33) unit deg
  }
}

presentation FilterLab for Filters {
  panel filters {
    slider(middle, range: [0 deg, 90 deg])
    text("without the middle filter: {crossed:.3}")
    text("with it: {through:.3}")
    text("light off water is polarized at {glare:.1}")
  }
  observe {
    blocked = crossed at t0
    passed  = through at t0
  }
}
```

```cases
run halfway of Filters with FilterLab {
  expect {
    blocked == 0 within 1e-12        // crossed filters pass nothing
    passed == 0.125 within 1e-12     // 1/2 × cos² 45° × cos² 45° = 1/8
  }
}
```

- The middle filter does not add light: it turns the light's polarization halfway, so that the last filter passes some of it.
- `brewster(1, 1.33)` is about 53 degrees: light reflected off water at that angle is polarized along the surface, which is why polarized sunglasses cut glare from lakes.

## The module's wave functions

| Function | What it gives |
|---|---|
| `wavenumber(λ)` | `2 π / λ`, the phase gained per length |
| `two_slit(d, λ, θ)` | brightness behind two narrow slits `d` apart at angle `θ`, from 0 to 1 |
| `single_slit(a, λ, θ)` | brightness behind one slit of width `a`, 1 in the middle |
| `grating(N, d, λ, θ)` | brightness behind `N` narrow slits `d` apart |
| `bright_angle(m, d, λ)` | the angle of the bright line of order `m`, `d sin θ = m λ` |
| `malus(θ)` | the share a polarizer turned `θ` from the light's polarization passes |
| `brewster(n1, n2)` | the angle at which reflected light is polarized |

The patterns are those of a far screen (Fraunhofer diffraction) and of narrow slits where so stated. A field of waves, as in the first section, follows them anywhere.

## Exercises

1. In the ripple tank, put a third source on the middle line behind the other two. Which lines of calm survive?
2. On the screen, draw `two(y) * one(y)`, the fringes of two real slits, and change `a`. How many bright fringes fit inside the middle band when `a = d / 5`?
3. Find, with a slider for `middle`, the angle that lets the most light through three filters. Why is it halfway?
