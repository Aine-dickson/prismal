# 2. Quantities and Units

Physical values in Prismal carry their dimension. A length cannot be added to a time, a velocity must come out as length per time, and every value is stored in coherent SI units whatever unit it was written in. This chapter covers how to write quantities and how dimension errors are reported.

> **In plain words.**
> - A **dimension** is the kind of thing a number measures: a length, a time, a mass, a speed. A **unit** is the scale it is written in: metres, centimetres and kilometres are all units of length.
> - **SI units** are the international standard units: the metre (m), kilogram (kg), second (s) and a few more. **Coherent** means that units built from them need no extra factor: a speed is metres per second, a force is a newton, which is exactly 1 kg times 1 m/s^2. Prismal turns every value into these units as it reads the program, so `120 km` is kept as 120000 metres. You can still write any unit you like.
> - **Dimensionless** means "a plain number with no unit": a count, a ratio, a percentage written as 0.25.
> - A **binding** is any named value in a model: a parameter, a state value, a derived value, a constant.

## Writing quantities

A number followed by a unit is a quantity: `120 km`, `1.5 h`, `9.81 m/s^2`, `4 N/m`, `0.01 /m`, `45 deg`.

- Units are written with unit symbols, `*`, `/` and powers `^2` (whole or half numbers, `s^0.5`). A unit has no spaces inside it: `9.81 m/s^2` is one unit, while `4 / m` divides 4 by a binding named `m`.
- Unit symbols known to the prototype: `m cm mm km μm nm` (`um` in ASCII), `s ms min h`, `kg g`, `N J W Pa Hz`, `A mA K mol cd`, the electrical `V mV kV Ω kΩ MΩ C F μF nF H mH` (`ohm`, `kohm`, `uF` in ASCII), the angle units `rad deg rev`, and `%` (a hundredth: `25 %` is `0.25`).
- Unit symbols are recognized only directly after a number. Elsewhere `m`, `s` and `g` are ordinary names, so a model may have a parameter `g` or `m`.
- The value is converted to coherent SI when the program is read: `120 km` is stored as `120000` metres and `1.5 h` as `5400` seconds. Displays and test expectations compare SI values; a test may write its expected value in any unit of the right dimension.

## Types

| Type | Values |
|---|---|
| `Real` | plain numbers (dimensionless) |
| `Length`, `Mass`, `Time`, `Velocity`, `Acceleration`, `Force`, `Energy`, `Power`, `Pressure`, `Frequency`, `Momentum`, `Area`, `Volume`, `Current`, `Amount` | named dimensions |
| `Quantity<L/T^2>` | any dimension, built from the base symbols `L M T I Θ N J` |
| `Angle` | angles; dimensionless, so `rad` is `1`, `deg` is `π/180`, `rev` is `2π` |
| `Boolean` | `true`, `false` |

The base symbols stand for length (`L`), mass (`M`), time (`T`), electric current (`I`), temperature (`Θ`), amount of substance (`N`) and luminous intensity (`J`). A dimension is built from them like a unit: `L/T^2` reads "length per time squared", an acceleration.

`Quantity<1/L>` is the type of a value per metre (for instance how much of something there is along each metre), `Quantity<M/T^2>` that of a spring's stiffness, how hard it pulls per metre stretched (N/m).

## Rules

- `+`, `-` and comparisons need both sides to have the same dimension: you can add metres to metres, not metres to seconds.
- `*` and `/` combine dimensions: a length divided by a time is a velocity.
- `^` takes a whole number (`speed^2`); `sqrt` takes a square root and halves the dimension (`sqrt(L / g)` is a time).
- `sin`, `cos`, `tan`, `exp`, `log` take dimensionless arguments. An angle counts as dimensionless: in radians it is an arc length divided by a radius, a length over a length. That is why `deg` is just the number `π/180`.
- A declared type is checked: `speed: Velocity = distance / duration` compiles only if the right side is a length per time.
- A bare number where a quantity is required is an error, with one exception: `0` stands for zero of any dimension (and for the zero vector, chapter 3). Write `5 m`, not `5`.

**Goal.** A learner sets a car trip's distance, time and slope, and reads the speed, energy and power that follow.
**How it is built.** Inputs are `param` values with units (moved by `slider`s in one `panel`); results are `derived` values (shown by `label`s in another `panel`). Gravity never changes, so it is a `const`. Units are checked everywhere: a mistake such as dividing a time by a length is refused before anything runs.

## A program

```text
model Trip {
  const {
    g: Acceleration = 9.81 m/s^2
  }
  param {
    distance: Length = 120 km   where distance > 0 m
    duration: Time   = 1.5 h    where duration > 0 s
    mass:     Mass   = 1200 kg  where mass > 0 kg
    slope:    Angle  = 3 deg    in [0 deg, 30 deg]   unit deg
  }
  derived {
    speed:  Velocity = distance / duration
    energy: Energy   = 0.5 * mass * speed^2
    climb:  Length   = distance * sin(slope)
    work:   Energy   = mass * g * climb
    power:  Power    = work / duration
  }
}
```

- `const { ... }` declares **constants**. Unlike parameters, a constant can never be changed by a case, a control or a lesson.
- `where distance > 0 m` is a range written as a condition. `in [0 deg, 30 deg]` and `where` both reject a run that starts outside, and both reject a control that tries to leave the range. Note that the bound `0 m` carries its unit: `distance > 0` would also be accepted, since `0` has any dimension.
- `unit deg` is a **display unit**: `slope` is stored in radians like every angle, but controls, labels and live formulas show it in degrees.
- A share or a ratio can be shown as a percentage the same way: `humidity: Real = 40 % in [0 %, 100 %] unit %` is stored as `0.4`, and controls, labels and texts show `40 %`.

```text
presentation TripPanel for Trip {
  panel inputs {
    slider(distance, range: [10 km, 300 km])
    slider(duration, range: [0.5 h, 5 h])
    slider(slope, range: [0 deg, 30 deg])
  }
  panel results {
    label(speed)
    label(energy)
    label(power)
  }
  observe {
    v = speed  live
    e = energy live
    p = power  live
  }
}
```

Labels and sliders show values in coherent SI units (`speed` as `22.2222 m/s`), except bindings with a display unit (`slope` in degrees).

```cases
run default_trip of Trip with TripPanel {
  expect {
    v == 22.2222222222222 m/s within 1e-9 m/s
    v == 80 km/h within 1e-9 m/s              // the same value, written in km/h
    e == 296296.296296296 J within 1e-6 J
    p == 13691.0861531541 W within 1e-6 W
  }
}

run long_trip of Trip with TripPanel {
  param { distance = 300 km; duration = 2.5 h }
  expect { v == 120 km/h within 1e-9 m/s }
}
```

`v == 80 km/h` shows that the expected value may be written in any unit of the right dimension; the tolerance is also a quantity, `1e-9 m/s`.

## Unit mistakes

Adding quantities of different dimensions:

```error
// error: MK-E01
model Wrong {
  param { d: Length = 1 m; t: Time = 1 s }
  derived { w: Length = d + t }
}
```

The compiler reports `cannot add Quantity<L> and Quantity<T>` at the expression.

A declared type that the definition does not have:

```error
// error: MK-E01
model Wrong {
  param { d: Length = 1 m; t: Time = 1 s }
  derived { v: Velocity = d * t }
}
```

Reported as `found Quantity<L T>, expected Quantity<L T^-1>`: the product is a length times a time.

A bare number where a quantity is required:

```error
// error: MK-E03
model Wrong {
  param { h: Length = 5 }
}
```

A dimensioned argument to a function that needs a pure number:

```error
// error: MK-E02
model Wrong {
  param { d: Length = 1 m }
  derived { a: Real = sin(d) }
}
```

A word that is not a unit symbol ends the number, and what follows is unexpected:

```error
// error: SX-E02
model Wrong {
  param { h: Length = 5 miles }
}
```

## Functions

A formula used more than once can be declared as a **function**: typed parameters, a result type and a body.

```text
model Energies {
  const { g: Acceleration = 9.81 m/s^2 }
  param {
    m: Mass     = 2 kg
    v: Velocity = 3 m/s
    h: Length   = 5 m
  }
  derived {
    moving:  Energy = kinetic(m, v)
    raised:  Energy = potential(m, h)
    total:   Energy = kinetic(m, v) + potential(m, h)
    reached: Length = height_for(v)
  }
  /// Kinetic energy of a mass moving at a speed.
  fn kinetic(mass: Mass, speed: Velocity): Energy = 0.5 * mass * speed^2
  fn potential(mass: Mass, height: Length): Energy = mass * g * height
  /// The height at which all kinetic energy has become potential energy.
  fn height_for(speed: Velocity): Length = speed^2 / (2 * g)
}

presentation EnergyChecks for Energies {
  observe {
    k    = moving  live
    p    = raised  live
    e    = total   live
    peak = reached live
  }
}
```

```cases
run energies of Energies with EnergyChecks {
  expect {
    k    == 9 J within 1e-9 J
    p    == 98.1 J within 1e-9 J
    e    == 107.1 J within 1e-9 J
    peak == 0.458715596330275 m within 1e-9 m
  }
}
```

- A function is checked like everything else: each argument must have its parameter's type (`kinetic(v, m)` is `MK-E01`), and the body must have the result type.
- A function is **closed**: its body reads only its parameters, constants (`g` here) and other functions. What it needs from the model is passed in as an argument, as `m` and `v` are. This keeps a function's meaning independent of when it is called.
- Functions may call other functions, but not themselves, directly or through others (`MK-E23`).
- `formula(kinetic)` in a presentation shows the definition, `kinetic(mass, speed) = 0.5 mass speed²`.

A function that reads a parameter instead of taking it as an argument:

```error
// error: MK-E18
model Wrong {
  param { m: Mass = 2 kg }
  fn kinetic(speed: Velocity): Energy = 0.5 * m * speed^2
}
```

The compiler reports that `kinetic` reads the parameter `m`: write `fn kinetic(mass: Mass, speed: Velocity)` and call `kinetic(m, v)`.

## Exercises

Solutions to the exercises not solved here are in [chapter 11](11-solutions.md).

1. Add `fuel_rate: Quantity<M/L> = 0.06 kg/km` to `Trip` and a derived `fuel: Mass = fuel_rate * distance`. Check it with a case (`7.2 kg` for the default trip).
2. Write `kinetic: Energy = 0.5 * mass * speed` and read the error. Which dimension does the compiler find?
3. A pendulum's period is `T = 2π sqrt(L / g)`. Write a model with `L: Length` and `g: Acceleration` and a derived `T: Time`. Check that `L = 1 m`, `g = 9.81 m/s^2` gives `2.00607 s` (to `1e-5 s`).

<details>
<summary>A solution to exercise 3</summary>

```text
model Period {
  param {
    L: Length       = 1 m        where L > 0 m
    g: Acceleration = 9.81 m/s^2 where g > 0 m/s^2
  }
  derived {
    T: Time = 2π * sqrt(L / g)
  }
}

presentation PeriodChecks for Period {
  observe { period = T live }
}
```

```cases
run one_metre of Period with PeriodChecks {
  expect { period == 2.00607 s within 1e-5 s }
}
```

`2π` is a product written without `*`: a number directly followed by `π` (or `pi`). Any other name directly after a number is an error.

</details>
