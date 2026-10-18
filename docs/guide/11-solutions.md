# 11. Solutions to the exercises

Each chapter ends with exercises. For chapters 1 to 9, which solve one exercise in place, this chapter solves the others; for chapters 15 to 24 it solves every exercise. Every program here is compiled and every case run by the guide's tests, like the programs of the chapters, so each solution is known to work.

A solution is one way to write the answer; others may be as good. Try the exercise first: a solution read before trying teaches much less.

## Chapter 1

**Exercises 1 and 2.** A vertical shift `c` with its slider, and a case with `k = 0.5`:

```text
model ShiftedWave {
  param {
    A: Real = 1     in [0, 3]
    k: Real = 1     in [0.5, 4]
    c: Real = 0     in [-2, 2]
  }
  derived {
    y(x: Real): Real = A * sin(k * x) + c
  }
}

presentation ShiftedPlot for ShiftedWave {
  view graph: plot(x: [-6, 6], y: [-5, 5]) {
    function_graph(y)
  }
  panel controls {
    slider(A, range: [0, 3], step: 0.1)
    slider(k, range: [0.5, 4], step: 0.1)
    slider(c, range: [-2, 2], step: 0.1)
    formula(y, live: true)
  }
  observe { y1 = y(1) live }
}
```

```cases
run shifted of ShiftedWave with ShiftedPlot {
  param { c = 1 }
  expect { y1 == 1.84147098480790 within 1e-12 }     // sin(1) + 1
}

run longer_wave of ShiftedWave with ShiftedPlot {
  param { k = 0.5 }
  expect { y1 == 0.479425538604203 within 1e-12 }    // sin(0.5)
}
```

A parameter's range (`in [-2, 2]`) is checked when the program starts: a case that sets `c = 3` fails to initialize, as `too_tall` does in chapter 1.

## Chapter 2

**Exercise 1.** Fuel for the trip. `Quantity<M/L>` is a mass per length; times a length it is a mass:

```text
model FuelTrip {
  param {
    distance:  Length        = 120 km    where distance > 0 m
    fuel_rate: Quantity<M/L> = 0.06 kg/km
  }
  derived {
    fuel: Mass = fuel_rate * distance
  }
}

presentation FuelChecks for FuelTrip {
  observe { used = fuel live }
}
```

```cases
run default_fuel of FuelTrip with FuelChecks {
  expect { used == 7.2 kg within 1e-9 kg }
}
```

**Exercise 2.** Without the square, `0.5 * mass * speed` is a momentum, `M L T^-1`, and an energy is `M L^2 T^-2`:

```error
// error: MK-E01
model Wrong {
  param {
    mass:  Mass     = 1200 kg
    speed: Velocity = 20 m/s
  }
  derived { kinetic: Energy = 0.5 * mass * speed }
}
```

The message names both dimensions: the declared type's and the expression's.

## Chapter 3

**Exercises 1 and 2.** The midpoint of `AB` with the median from `C`, and the height onto `AB`:

```text
space Plane = euclidean(2)

model Median in Plane {
  param {
    A: Point = origin
    B: Point = origin + (4 m, 0 m)
    C: Point = origin + (0 m, 3 m)
  }
  derived {
    M:    Point         = A + (B - A) / 2
    ab:   Length        = |B - A|
    area: Quantity<L^2> = abs((B - A).x * (C - A).y - (B - A).y * (C - A).x) / 2
    h:    Length        = 2 * area / ab
  }
}

presentation MedianLab for Median {
  view scene: spatial(Plane, scale: 1 m -> 50 px, y: up) {
    grid; axes
    segment(A, B); segment(B, C); segment(C, A)
    segment(C, M, line: dashed) as median
    marker(A) as a { on drag as p { propose A = p } }
    marker(B) as b { on drag as p { propose B = p } }
    marker(C) as c { on drag as p { propose C = p } }
    marker(M) as midpoint
  }
  panel measures { label(h) }
  observe {
    mx     = M.x live
    my     = M.y live
    height = h   live
  }
}
```

```cases
run right_triangle of Median with MedianLab {
  expect {
    mx     == 2 m within 1e-12 m
    my     == 0 m within 1e-12 m
    height == 3 m within 1e-12 m
  }
}
```

`(B - A) / 2` halves a vector; adding it to the point `A` gives a point. `area / ab` divides an area by a length, which is a length, as `h` declares.

## Chapter 4

**Exercise 1.** A driving force at the natural frequency. Undamped, the spring alone would swing at `sqrt(k / m) = 2 /s`; driven there, the swing grows until the damping takes as much energy per cycle as the force gives, at an amplitude of `F0 / (c w) = 0.125 m`:

```text
model Driven {
  param {
    m:  Mass            = 1 kg        where m > 0 kg
    k:  Quantity<M/T^2> = 4 N/m       where k > 0 N/m
    c:  Quantity<M/T>   = 0.4 N*s/m   where c >= 0 N*s/m
    x0: Length          = 0.1 m
    F0: Force           = 0.1 N
    w:  Frequency       = 2 /s
  }
  state {
    x: Length   = x0
    v: Velocity = 0
  }
  flow { der(x) = v }
  process spring  { flow der(v) += -(k / m) * x }
  process damping { flow der(v) += -(c / m) * v }
  process drive   { flow der(v) += (F0 / m) * cos(w * (t - t0)) }
}

presentation DrivenLab for Driven {
  view position: plot(x: [0 s, 40 s], y: [-0.15 m, 0.15 m]) {
    series_plot(x every 0.02 s)
  }
  panel controls { slider(w, range: [0.5 /s, 4 /s]) }
  observe { x40 = x at t0 + 40 s }
}
```

```cases
run resonance of Driven with DrivenLab {
  until t0 + 40 s
  expect { x40 == -0.124218697747 m within 1e-6 m }
}
```

`t` is an instant, not a duration: `w * t` is an error (MK-E04, scaling an instant), and `t - t0`, the time since the start, is what the cosine takes. The reference value comes from an independent fine-step integration. Move the slider away from `2 /s` and the swing stays small: that peak is resonance.

**Exercise 3.** A projectile in the plane. Gravity is a process, so it can be switched off or replaced later:

```text
space Plane = euclidean(2)

model Throw in Plane {
  param {
    g:     Acceleration = 9.81 m/s^2
    speed: Velocity     = 20 m/s
    angle: Angle        = 45 deg     unit deg
  }
  state {
    pos: Point            = origin
    vel: Vector<Velocity> = (speed * cos(angle), speed * sin(angle))
  }
  flow { der(pos) = vel }
  process gravity { flow der(vel) += (0 m/s^2, -g) }
}

presentation ThrowView for Throw {
  view scene: spatial(Plane, scale: 1 m -> 10 px, y: up) {
    axes
    marker(pos) as ball
    trace(pos every 0.05 s)
  }
  observe {
    x1 = pos.x at t0 + 1 s
    y1 = pos.y at t0 + 1 s
  }
}
```

```cases
run one_second of Throw with ThrowView {
  until t0 + 2 s
  expect {
    x1 == 14.1421356237310 m within 1e-6 m     // 20 cos 45 deg
    y1 == 9.23713562373095 m within 1e-6 m     // 20 sin 45 deg - g / 2
  }
}
```

Without an event the ball flies through the ground; RP-01 adds the landing.

## Chapter 5

**Exercise 1.** An overflow valve at 1.1 m. The event as the exercise writes it is refused:

```error
// error: MK-E16
model Spill {
  param {
    inflow: Quantity<L/T> = 0.02 m/s
    drain:  Quantity<L/T> = 0.01 m/s
  }
  state { level: Length = 0.2 m }
  flow { der(level) = inflow - drain }
  event spill on rising(level - 1.1 m) { set level = 1.1 m }
}
```

`spill` sets the level its own guard reads, and the level keeps rising after it, so the event would happen again at once, forever. The compiler asks for a Zeno policy (chapter 5), but the better fix is the physics: while the tank overflows, what comes in goes out over the rim, and the level stays at 1.1 m. That is a mode:

```text
model Overflow {
  param {
    inflow: Quantity<L/T> = 0.02 m/s
    drain:  Quantity<L/T> = 0.01 m/s
    high:   Length = 1 m
    low:    Length = 0.5 m
  }
  state { level: Length = 0.2 m }
  discrete {
    pumping:     Boolean = true
    overflowing: Boolean = false
  }
  flow {
    der(level) = if overflowing then 0 m/s else (if pumping then inflow else 0) - drain
  }
  event full  on rising(level - high)  { set pumping = false }
  event empty on falling(level - low)  { set pumping = true }
  event spill on rising(level - 1.1 m) { set overflowing = true }
}

presentation OverflowChecks for Overflow {
  observe {
    spills = elapsed on spill
    fulls  = elapsed on full
    later  = level at t0 + 200 s
  }
}
```

```cases
run default_high of Overflow with OverflowChecks {
  until t0 + 200 s
  expect { spills == [] }
}

run high_set_above of Overflow with OverflowChecks {
  param { high = 1.2 m }
  until t0 + 200 s
  expect {
    spills[1] == 90 s within 1e-9 s      // 0.9 m at 0.01 m/s
    fulls     == []
    later     == 1.1 m within 1e-9 m
  }
}
```

With the default `high = 1 m` the pump stops first, so the tank never overflows. With `high` above the rim, the level reaches 1.1 m at 90 s and stays there: the pump never reaches `high`, and the tank overflows for good. (This model never leaves the overflowing mode; a pump that stops would need an event that ends it.)

**Exercise 2.** A leak from 150 s. At 150 s the pump is on (it restarted at 130 s) and the level is 0.7 m; from then the doubled drain, 0.02 m/s, equals the inflow, so the level stays at 0.7 m and never reaches `low` again:

```text
model Leaky {
  param {
    inflow: Quantity<L/T> = 0.02 m/s
    drain:  Quantity<L/T> = 0.01 m/s
    high:   Length = 1 m
    low:    Length = 0.5 m
  }
  state { level: Length = 0.2 m }
  discrete {
    pumping: Boolean = true
    leaking: Boolean = false
  }
  flow {
    der(level) = (if pumping then inflow else 0) - (if leaking then 2 * drain else drain)
  }
  event full  on rising(level - high) { set pumping = false }
  event empty on falling(level - low) { set pumping = true }
  event leak  on at t0 + 150 s        { set leaking = true }
}

presentation LeakyChecks for Leaky {
  observe {
    empties = elapsed on empty
    later   = level at t0 + 300 s
  }
}
```

```cases
run leak of Leaky with LeakyChecks {
  until t0 + 300 s
  expect {
    empties[1] == 130 s within 1e-9 s
    later      == 0.7 m within 1e-9 m
  }
}
```

With the leak at 100 s instead, while the pump is off at 0.8 m, the level would fall at 0.02 m/s and reach `low` at 115 s.

## Chapter 6

**Exercise 1.** A parachute that opens at 12 s. When it opens, the terminal speed `vt` drops from 56 m/s to 6.3 m/s while the diver still falls at 54 m/s, so `subterminal` is violated. With `reject`, the transition made by `deploy` is refused and the run fails at 12 s (RC section 10). With `report`, the run goes on: the diver slows towards the new terminal speed, and every step until then records a diagnostic:

```text
model Chute {
  param {
    m:      Mass          = 80 kg
    g:      Acceleration  = 9.81 m/s^2
    c:      Quantity<M/L> = 0.25 kg/m
    c_open: Quantity<M/L> = 20 kg/m
  }
  state {
    fallen: Length   = 0 m
    v:      Velocity = 0
  }
  discrete { open: Boolean = false }
  derived {
    drag: Quantity<M/L> = if open then c_open else c
    vt:   Velocity      = sqrt(m * g / drag)
  }
  flow {
    der(fallen) = v
    der(v)      = g - (drag / m) * v^2
  }
  event deploy on at t0 + 12 s { set open = true }
  constraint subterminal: v <= vt policy report
}

presentation ChuteChecks for Chute {
  observe {
    v12      = v at t0 + 12 s
    v20      = v at t0 + 20 s
    problems = diagnostics over [t0, t0 + 11 s]
  }
}
```

```cases
run chute of Chute with ChuteChecks {
  until t0 + 20 s
  expect {
    v12      == 54.3765192220466 m/s within 1e-5 m/s    // vt tanh(g t / vt)
    v20      == 6.26418390547676 m/s within 1e-5 m/s
    problems == []                                     // none before the chute opens
  }
}
```

After opening, `v = vt coth(g (t - 12 s) / vt + arcoth(v12 / vt))`, which is within a millionth of `vt` by 20 s. A constraint states what must hold for the model to be valid; "never faster than terminal speed" is true of a free fall but not of a fall whose drag changes, so the model's constraint was too strong.

**Exercise 2.** Energy is conserved without damping and lost with it:

```text
model Conserving {
  param {
    m:  Mass            = 1 kg        where m > 0 kg
    k:  Quantity<M/T^2> = 4 N/m       where k > 0 N/m
    c:  Quantity<M/T>   = 0.4 N*s/m   where c >= 0 N*s/m
    x0: Length          = 0.1 m
  }
  state {
    x: Length   = x0
    v: Velocity = 0
  }
  derived { energy: Energy = 0.5 * m * v^2 + 0.5 * k * x^2 }
  flow {
    der(x) = v
    der(v) = -(k / m) * x - (c / m) * v
  }
  equation conservation: energy == 0.5 * k * x0^2 checked within 1e-5 J
}

presentation ConservingChecks for Conserving {
  observe { problems = diagnostics over [t0, t_end] }
}
```

```cases
run undamped of Conserving with ConservingChecks {
  param { c = 0 N*s/m }
  until t0 + 10 s
  expect { problems == [] }
}
```

With the default damping the case fails, and its first diagnostic reads: check equation `conservation` at t = 0.1007 s, residual 2.098e-5 exceeds 1e-5. The residual is the energy lost so far; it passes the tolerance a tenth of a second in, and a diagnostic follows at every step after. The equation is not a law of the damped model, and the check says so.

## Chapter 7

**Exercise 2.** A marker at the apex whose drag sets the angle. The apex height is `(v sin θ)^2 / (2 g)`, so a dragged height `y` gives `sin θ = sqrt(2 g y) / v`. The language has `atan2` but no `asin`; `atan2(s, sqrt(1 - s^2))` is the angle whose sine is `s`:

```text
model Apex {
  param {
    speed: Velocity = 20 m/s   where speed > 0 m/s
    angle: Angle    = 45 deg   in [5 deg, 85 deg]   unit deg
  }
  derived {
    g:     Acceleration = 9.81 m/s^2
    reach: Length       = speed^2 * sin(2 * angle) / g
    top:   Length       = (speed * sin(angle))^2 / (2 * g)
    path(x: Real): Real = x * tan(angle) - (g / (1 m/s^2)) * x^2 / (2 * (speed / (1 m/s))^2 * cos(angle)^2)
  }
}

presentation ApexLab for Apex {
  view flight: plot(x: [0, 60], y: [0, 25]) {
    function_graph(path)
    marker(at: (reach / (2 m), top / (1 m))) as apex {
      on drag as p {
        propose angle = atan2(sqrt(2 * g * p.y * 1 m) / speed, sqrt(1 - 2 * g * p.y * 1 m / speed^2))
      }
    }
  }
  panel controls { slider(angle, range: [5 deg, 85 deg]) }
  observe {
    ax = reach / (2 m) live
    ay = top / (1 m)   live
  }
}
```

```cases
run apex of Apex with ApexLab {
  expect {
    ax == 20.3873598369011 within 1e-9
    ay == 10.1936799184506 within 1e-9
  }
}

run steeper of Apex with ApexLab {
  param { angle = 60 deg }
  expect { ay == 15.2905198776758 within 1e-9 }     // (20 sin 60)^2 / (2 g)
}
```

A height above `v^2 / (2 g)`, about 20.4 m, has no angle: the square root of a negative number is a status, and the drag's preview shows the proposal as not valid.

**Exercise 3.** Controls target whole bindings, and a point is one binding: a `number_input` edits a number or a quantity, not a coordinate of a point. The model therefore declares the coordinates as parameters and builds the point from them:

```text
space Plane = euclidean(2)

model Corner in Plane {
  param {
    Ax: Length = 0 m   in [-5 m, 5 m]
    Ay: Length = 0 m   in [-5 m, 5 m]
    B:  Point  = origin + (4 m, 0 m)
    C:  Point  = origin + (0 m, 3 m)
  }
  derived {
    A: Point = origin + (Ax, Ay)
    G: Point = A + ((B - A) + (C - A)) / 3
  }
}

presentation CornerLab for Corner {
  view scene: spatial(Plane, scale: 1 m -> 50 px, y: up) {
    segment(A, B); segment(B, C); segment(C, A)
    marker(G) as centroid
  }
  panel inputs {
    number_input(Ax, range: [-5 m, 5 m])
    number_input(Ay, range: [-5 m, 5 m])
  }
  observe { gx = G.x live }
}
```

```cases
run moved of Corner with CornerLab {
  param { Ax = 1 m }
  expect { gx == 1.66666666666667 m within 1e-12 m }    // (1 + 4 + 0) / 3
}
```

`A` is derived now, so the marker of `A` could no longer be dragged by proposing `A`; a drag would propose `Ax` and `Ay` instead: `on drag as p { propose Ax = p.x; propose Ay = p.y }`.

## Chapter 8

**Exercise 1.** A beat that shows the velocity and replays the fall. The arrow is shown by the beat (`in scene { ... }`) and stays for the rest of the lesson:

```text
space Plane = euclidean(2)

model Dropped in Plane {
  param {
    g: Acceleration = 9.81 m/s^2
    h: Length       = 10 m     in [1 m, 50 m]
  }
  state {
    pos: Point            = origin + (0 m, h)
    vel: Vector<Velocity> = 0
  }
  discrete { airborne: Boolean = true }
  flow {
    der(pos) = if airborne then vel else 0
    der(vel) = if airborne then (0, -g) else 0
  }
  event landed on falling(pos.y) {
    set airborne = false
    set vel = 0
  }
}

presentation ArrowLesson for Dropped {
  view scene: spatial(Plane, scale: 1 m -> 12 px, y: up) {
    axes
    marker(pos) as ball
  }
  timeline {
    scene first_drop {
      beat fall   { run rate 1 until landed }
      beat slow   { seek t0; run rate 0.25 until landed }
      beat arrows {
        seek t0
        in scene { arrow(vel, from: pos, scale: 1 m/s -> 5 px) as speed }
        run rate 1 until landed
      }
    }
  }
}
```

```cases
run with_arrow of Dropped with ArrowLesson {
  expect {
    end of fall   == 1.42784312292706 s within 1e-8 s     // T = sqrt(2 h / g)
    end of slow   == 7.13921561463530 s within 1e-8 s     // T + 4 T
    end of arrows == 8.56705873756236 s within 1e-8 s     // T + 4 T + T
  }
}
```

The run-directing actions, `seek` then `run`, apply first, in order; the arrow appears as the replay starts.

**Exercise 3.** RP-08 with a learner who sets 30 degrees. Until the explore beat nothing changes. The learner continues at 25 s, so `b7` ends there; `b8` relaunches at 30 degrees and runs until the landing, which takes `T30 = 2 v sin(30°) / g = 20 / 9.81 = 2.03873598 s`. So `b8` ends at `27.0387359836901 s`. The range is `v^2 sin(60°) / g = 35.3119 m`, the same as at 60 degrees: complementary angles reach equally far, and the narration's "compare the distance" has a surprise in it.

```text
space Plane = euclidean(2)

model Launch in Plane {
  param {
    g:     Acceleration = 9.81 m/s^2
    speed: Velocity     = 20 m/s   where speed > 0 m/s
    angle: Angle        = 45 deg   in (0 deg, 90 deg)   unit deg
  }
  state {
    pos: Point            = origin
    vel: Vector<Velocity> = speed * (cos(angle), sin(angle))
  }
  discrete { flying: Boolean = true }
  flow {
    der(pos) = if flying then vel else 0
    der(vel) = if flying then (0, -g) else 0
  }
  event landed on falling(pos.y) { set flying = false; set vel = 0 }
  event relaunch on request {
    set pos    = origin
    set vel    = speed * (cos(angle), sin(angle))
    set flying = true
  }
}

presentation Relaunch for Launch {
  view scene: spatial(Plane, scale: 1 m -> 10 px, y: up) {
    axes
    marker(pos) as ball
  }
  observe {
    times  = elapsed on landed
    ranges = pos.x on landed
  }
  timeline {
    scene launch {
      beat b1 { narrate "A ball is launched at 45 degrees." for 4 s }
      beat b2 { run rate 1 until landed }
      beat b3 { hold; narrate "It lands here. Why this distance?" for 3 s }
      beat b4 { seek t0; narrate "Watch the horizontal speed." for 3 s }
      beat b5 { run rate 0.5 until landed }
    }
    scene try_it {
      beat b6 { hold; narrate "Choose your own angle." for 3 s }
      beat b7 {
        explore limit 60 s keep angle {
          slider(angle, range: [10 deg, 80 deg])
        } fallback {
          sequence { intervene { set angle = 60 deg }; wait 3 s }
        }
      }
      beat b8 { request relaunch; run rate 1 until landed }
      beat b9 { narrate "Compare the distance with the first launch." for 3 s }
    }
  }
}
```

```cases
run thirty of Launch with Relaunch {
  learner {
    at 23 s: set slider angle = 30 deg
    at 25 s: continue
  }
  expect {
    end of b8 == 27.0387359836901 s within 1e-8 s
    times[2]  == 4.92194406192272 s within 1e-8 s     // T45 + T30
    ranges[2] == 35.3119430697019 m within 1e-6 m     // as at 60 degrees
  }
}
```

## Chapter 9

**Exercise 1.** A fourth ball: `row: Ball[4]`. The new ball starts at `(4 m, 4 m)`, so it is the highest: `top` changes at 0.2 s (from 2.8038 m to 3.8038 m), and `rested` at 10 s becomes 4. A ball from 4 m bounces for about 8 s in all (`sqrt(2 × 3.9 m / g) × (1 + 0.8) / (1 - 0.8)`), so it rests before 10 s. `second` does not change:

```text
space Plane = euclidean(2)

model Drops4 in Plane {
  object Ball {
    param { r: Length = 0.1 m }
    state {
      pos: Point            = origin + (0 m, 1 m)
      vel: Vector<Velocity> = 0
    }
    discrete { resting: Boolean = false }
    flow {
      der(pos) = vel
      der(vel) = if resting then 0 else (0 m/s^2, -9.81 m/s^2)
    }
    event bounce on falling(pos.y - r) {
      set vel = (vel.x, -0.8 * vel.y)
    } zeno settle {
      set vel = 0
      set pos = origin + (pos.x, r)
      set resting = true
    }
  }
  parts {
    row: Ball[4] { pos = origin + (index * 1 m, index * 1 m) }
  }
  derived {
    highest: Length = max(b.pos.y for b in row)
    landed:  Real   = count(b for b in row if b.resting)
  }
}

presentation Drops4View for Drops4 {
  view scene: spatial(Plane, scale: 1 m -> 60 px, y: up) {
    for b in row { marker(b.pos) as ball }
  }
  observe {
    second = row[2].pos.y live
    top    = highest live
    rested = landed live
  }
}
```

```cases
run first_moments of Drops4 with Drops4View {
  until t0 + 0.2 s
  expect {
    second == 1.8038 m within 1e-9 m
    top    == 3.8038 m within 1e-9 m
  }
}

run at_rest of Drops4 with Drops4View {
  until t0 + 10 s
  expect { rested == 4 within 1e-12 }
}
```

**Exercise 4.** Two drops per spray. Each `create` makes the next drop, so a spray at `0.5 k` s makes drops `2k + 1` and `2k + 2`. At 1.2 s the pair from 0 s has landed (at 1.0194 s), and the pairs from 0.5 s and 1 s fly: four drops. The capacity now lasts half as long, so it is doubled:

```text
space Plane = euclidean(2)

model Fountain2 in Plane {
  object Drop {
    state {
      pos: Point            = origin
      vel: Vector<Velocity> = 0
    }
    flow {
      der(pos) = vel
      der(vel) = (0 m/s^2, -9.81 m/s^2)
    }
  }
  param { speed: Velocity = 5 m/s }
  parts { drops: Drop[max 80] }
  event spray on every 0.5 s {
    create drops { vel = (1 m/s, speed) }
    create drops { vel = (-1 m/s, speed) }
  }
  for d in drops {
    event land on falling(d.pos.y) { destroy d }
  }
  derived { flying: Real = count(drops) }
}

presentation Spray2 for Fountain2 {
  view scene: spatial(Plane, scale: 1 m -> 50 px, y: up) {
    for d in drops { marker(d.pos) as drop }
  }
  observe {
    n     = flying live
    right = drops[3].pos.x live
    left  = drops[4].pos.x live
  }
}
```

```cases
run pairs of Fountain2 with Spray2 {
  until t0 + 1.2 s
  expect {
    n     == 4 within 1e-12
    right == 0.7 m within 1e-9 m      // thrown at 0.5 s, 1 m/s for 0.7 s
    left  == -0.7 m within 1e-9 m
  }
}
```

**Exercise 3.** Three beads on a line joined by springs of rest length `L`. Each bead is given its number, `n = index`; the force on bead `b` sums over the other beads whose number differs by one. The spring to a neighbour `o` pulls with `k` times its stretch, `(o.x - b.x) - (o.n - b.n) L`:

```text
model Chain {
  object Bead {
    param { n: Real = 0 }
    state {
      x: Length   = 0 m
      v: Velocity = 0
    }
    flow { der(x) = v }
  }
  param {
    k: Quantity<M/T^2> = 4 N/m
    m: Mass            = 1 kg
    L: Length          = 1 m
    a: Length          = 0.1 m
  }
  parts {
    beads: Bead[3] {
      n = index
      x = (index - 1) * L + (if index == 1 then -a else if index == 3 then a else 0 m)
    }
  }
  flow {
    for b in beads {
      der(b.v) = sum(k * ((o.x - b.x) - (o.n - b.n) * L) for o in beads if abs(o.n - b.n) == 1) / m
    }
  }
}

presentation ChainChecks for Chain {
  observe {
    middle = beads[2].x at t0 + 1 s
    last   = beads[3].x at t0 + 1 s
  }
}
```

```cases
run stretched of Chain with ChainChecks {
  until t0 + 2 s
  expect {
    middle == 1 m within 1e-9 m
    last   == 1.95838531634529 m within 1e-6 m     // 2 m + a cos(2 t)
  }
}
```

With both end beads pulled out by `a`, the middle bead feels equal and opposite pulls and stays put. Each end bead then swings on one spring against a fixed point, at `sqrt(k / m) = 2 /s`. A relation set (the chapter's springs) says the same with the springs as values of their own; the filter is the way when neighbours follow from numbers.

**Exercise 5.** A second marker above each planet requests `kick` for it. A lesson requests the same event to check it; with `mu = 0` there is no sun, so a kicked planet moves in a straight line:

```text
space Plane = euclidean(2)

model Kicks in Plane {
  object Planet {
    state {
      pos: Point            = origin + (1 m, 0 m) intervenable
      vel: Vector<Velocity> = (0 m/s, 1 m/s)
    }
    flow { der(pos) = vel }
  }
  param { mu: Quantity<L^3/T^2> = 1 m^3/s^2 }
  parts {
    planets: Planet[2, max 6] {
      pos = origin + (index * 1 m, 0 m)
      vel = (0 m/s, sqrt(mu / (index * 1 m)))
    }
  }
  flow {
    for p in planets {
      der(p.vel) = -mu * (p.pos - origin) / |p.pos - origin|^3
    }
  }
  event kick on request(p in planets, j: Vector<Momentum>) { set p.vel = p.vel + j / 1 kg }
}

presentation KickSky for Kicks {
  view sky: spatial(Plane, scale: 1 m -> 80 px, y: up) {
    marker(origin) as sun
    for p in planets {
      marker(p.pos) as planet { on drag as q { propose p.pos = q } }
      marker(p.pos + (0 m, 0.2 m), color: orange) as kicker {
        on click request kick(p, (0 kg*m/s, 0.5 kg*m/s))
      }
    }
  }
}

presentation KickTour for Kicks {
  view sky: spatial(Plane, scale: 1 m -> 80 px, y: up) {
    for p in planets { marker(p.pos) as planet }
  }
  observe {
    x1 = planets[1].pos.x at t0 + 1 s
    y2 = planets[2].pos.y at t0 + 1 s
  }
  timeline {
    scene only {
      beat kick_one { request kick(planets[1], (0.5 kg*m/s, 0 kg*m/s)); run rate 1; wait 1 s }
    }
  }
}
```

```cases
run straight of Kicks with KickTour {
  param { mu = 0 m^3/s^2 }
  expect {
    x1 == 1.5 m within 1e-9 m     // from 1 m at 0.5 m/s
    y2 == 0 m within 1e-9 m       // not kicked, and at rest without a sun
  }
}
```

The kicker is a separate marker because a representation has one click; the planet keeps its drag. The orange color tells the two apart, and the names `planet` and `kicker` say so in the text alternatives.

## Chapter 15

**Exercise 1.** A half-life of 20 s. Half the nuclei are left when `e^(-lambda t) = 1/2`, so `lambda = ln 2 / 20 s = 0.0346574 /s`. After 20 s each nucleus has survived with probability one half, so of 1000 about 500 remain, with a deviation of `sqrt(1000 * 0.5 * 0.5) = 15.8`:

```text
model Slower {
  param {
    lambda: Frequency = 0.034657359 /s
    n0:     Real      = 1000
  }
  discrete { n: Real = n0 }
  derived { half_life: Time = log(2) / lambda }

  event decay on random(lambda * n) { set n = n - 1 }
}

presentation SlowerFacts for Slower {
  observe {
    hl   = half_life at t0
    left = n at t0 + 20 s
  }
}
```

```cases
run twenty_seconds of Slower with SlowerFacts {
  config { seed = 1 }
  until t0 + 20 s
  expect {
    hl == 20 s within 1e-6 s
    left in [420, 580]
  }
}
```

The range is five deviations wide on each side, so a correct model fails it in fewer than one run in a million.

**Exercise 2.** A die. `integer(1, 6)` draws a whole number from 1 to 6. To test that every face lies in `[1, 6]`, the model keeps the lowest and highest face thrown: the event `seen` follows each throw (`on throw`) and reads the new face. After 100 throws both 1 and 6 have come up, except in about one run in ten million (`6 * (5/6)^100`). The mean of the faces is near 3.5: one throw has a deviation of `sqrt(35 / 12) = 1.71`, the mean of 100 throws `0.171`.

```text
model Die {
  discrete {
    face:    Real = 1
    lowest:  Real = 6
    highest: Real = 1
  }
  event throw on every 1 s from t0 + 1 s { set face = integer(1, 6) }
  event seen on throw {
    set lowest  = min(lowest, face)
    set highest = max(highest, face)
  }
}

presentation DieFacts for Die {
  observe {
    faces = face on throw
    low   = lowest at t0 + 100.5 s
    high  = highest at t0 + 100.5 s
  }
}
```

```cases
run hundred_throws of Die with DieFacts {
  config { seed = 3 }
  until t0 + 100.5 s
  expect {
    low == 1 exactly
    high == 6 exactly
    mean(faces) == 3.5 within 0.85
  }
}
```

**Exercise 3.** A walker that steps only half the time. `if bernoulli(0.5)` enables the event at each tick with probability one half, so of the 100 ticks in 20 s about 50 are steps, with a deviation of `sqrt(100 * 0.5 * 0.5) = 5`. The walker spreads more slowly: after `n` steps its deviation is `s sqrt(n)`, and it now takes twice as long to make the same number of steps.

```text
space Plane = euclidean(2)

model LazyWalk in Plane {
  param { s: Length = 0.5 m }
  discrete {
    pos:   Point = origin
    steps: Real  = 0
  }

  event step on every 0.2 s from t0 + 0.2 s if bernoulli(0.5) {
    set pos = pos + normal(0, s)
    set steps = steps + 1
  }
}

presentation LazyFacts for LazyWalk {
  observe { taken = steps at t0 + 20 s }
}
```

```cases
run half_steps of LazyWalk with LazyFacts {
  config { seed = 5 }
  until t0 + 20 s
  expect { taken in [25, 75] }
}
```

## Chapter 16

**Exercise 1.** An insulated right end. Without a condition at `right`, the edge is insulated: no heat crosses it. Heat keeps flowing in at the left until nothing is colder than the left end, so the whole rod reaches the left end's temperature, 100. It gets there slowly: the slowest pattern of a rod held at one end and insulated at the other fades as `e^(-alpha (π / 2 L)^2 t)`, with a time constant of `1 / (0.002 m^2/s * 2.467 /m^2) = 203 s`, so after 3000 s the difference is far below a hundredth.

```text
model InsulatedRod {
  grid rod(x: [0 m, 1 m], cells: 50)
  param {
    alpha: Quantity<L^2/T> = 0.002 m^2/s
    hot:   Real            = 100
  }
  state { T(x in rod): Real = 20 }
  flow der(T) = alpha * laplacian(T)
  boundary T { left = hot }
}

presentation InsulatedFacts for InsulatedRod {
  observe {
    far  = T(0.99 m) at t0 + 3000 s
    half = T(0.5 m) at t0 + 3000 s
  }
}
```

```cases
run warms_through of InsulatedRod with InsulatedFacts {
  until t0 + 3000 s
  expect {
    far == 100 within 0.01
    half == 100 within 0.01
  }
}
```

**Exercise 2.** A string plucked in the middle. A wave on a string is two copies of the starting shape travelling in opposite directions at speed `c`, each half as high (d'Alembert's solution), turned upside down at the fixed ends. After a quarter period, 0.25 s, each copy has moved 0.5 m: the rising side of one exactly cancels the falling side of the other, and the string is **flat**, all its energy in motion. After half a period it is the triangle upside down.

```text
model Plucked {
  grid line(x: [0 m, 1 m], cells: 100)
  param { c: Velocity = 2 m/s }
  state {
    u(x in line): Length = 0.02 m * (1 - |x - 0.5 m| / 0.5 m)
    v(x in line): Velocity = 0 m/s
  }
  flow {
    der(u) = v
    der(v) = c^2 * laplacian(u)
  }
  boundary u {
    left = 0 m
    right = 0 m
  }
}

presentation PluckedFacts for Plucked {
  observe {
    middle_quarter  = u(0.5 m) at t0 + 0.25 s
    side_quarter    = u(0.25 m) at t0 + 0.25 s
    side_half       = u(0.25 m) at t0 + 0.5 s
  }
}
```

```cases
run quarter of Plucked with PluckedFacts {
  until t0 + 0.5 s
  expect {
    middle_quarter == 0 m within 0.0005 m
    side_quarter == 0 m within 0.0005 m
    side_half == -0.01 m within 0.0005 m
  }
}
```

The corner of the triangle is blurred by the grid's cells, so the checks allow a fortieth of the height.

**Exercise 3.** A tank that wraps around. With `boundary h periodic`, a ripple that leaves through one wall comes back in through the opposite one, as if the tank were tiled with copies of itself. Here the stone falls 0.1 m from the left wall. The water 0.2 m to its right, at `x = 0.3 m`, and the water 0.2 m to its left, which is through the wall at `x = 0.9 m`, rise and fall together at every instant. With walls held at 0, the ripple is reflected instead and the two points differ.

```text
space Plane = euclidean(2)

model Wrapped in Plane {
  grid tank(x: [0 m, 1 m], y: [0 m, 1 m], cells: (60, 60))
  param { c: Velocity = 0.3 m/s }
  state {
    h(p in tank): Length = 0.01 m * exp(-(|p - (origin + (0.1 m, 0.5 m))| / 0.04 m)^2)
    v(p in tank): Velocity = 0 m/s
  }
  flow {
    der(h) = v
    der(v) = c^2 * laplacian(h)
  }
  boundary h periodic
  derived {
    right_of: Length = h(origin + (0.3 m, 0.5 m))
    left_of:  Length = h(origin + (0.9 m, 0.5 m))
    gap:      Length = right_of - left_of
    size:     Length = |right_of|
  }
}

presentation WrappedLab for Wrapped {
  until t0 + 3 s
  view tank: spatial(Plane, scale: 1 m -> 400 px, y: up) {
    heatmap(h, range: [-0.004 m, 0.004 m])
  }
  observe {
    gap_early = gap at t0 + 0.75 s
    gap_late  = gap at t0 + 1.5 s
    arrived   = size at t0 + 0.75 s
  }
}
```

```cases
run wraps of Wrapped with WrappedLab {
  until t0 + 1.5 s
  expect {
    gap_early == 0 m within 1e-6 m
    gap_late == 0 m within 1e-6 m
    arrived in [1e-5 m, 0.01 m]       // the ripple has reached both points
  }
}
```

## Chapter 17

**Exercise 1.** A third resistor in parallel with `R2`. The current now splits at the junction, so there is one more current, and the current law there is the extra equation: four unknowns (`I`, `I2`, `I3`, `V2`) and four equations. With `R2 = R3 = 200 Ω` the pair acts as 100 Ω, the circuit as 200 Ω, so `I = 9 V / 200 Ω = 45 mA`, `V2 = 4.5 V`, and each branch carries half:

```text
model Parallel {
  param {
    V0: Voltage    = 9 V
    R1: Resistance = 100 Ω
    R2: Resistance = 200 Ω
    R3: Resistance = 200 Ω
  }
  unknown {
    I:  Current
    I2: Current
    I3: Current
    V2: Voltage
  }
  equation loop:     V0 == I * R1 + V2
  equation branch2:  V2 == I2 * R2
  equation branch3:  V2 == I3 * R3
  equation junction: I == I2 + I3
}

presentation ParallelFacts for Parallel {
  observe {
    total = I at t0
    split = I3 at t0
    v2    = V2 at t0
  }
}
```

```cases
run splits of Parallel with ParallelFacts {
  expect {
    total == 0.045 A within 1e-9 A
    split == 0.0225 A within 1e-9 A
    v2 == 4.5 V within 1e-9 V
  }
}
```

**Exercise 2.** The battery switched off at 3 s. Until then the capacitor charges to `6 V * (1 - e^-3) = 5.7013 V`. When the battery goes, the loop equation becomes `0 == I R + Vc`: the current **reverses** at once, to `-5.70 mA`, and the capacitor discharges through the resistor, its voltage and the current both falling as `e^(-(t - 3 s) / RC)`. Half a second later the current is `-5.7013 V * e^-0.5 / 1 kΩ = -3.4580 mA`.

```text
model Switched {
  param {
    V0: Voltage     = 6 V
    R:  Resistance  = 1 kΩ
    C:  Capacitance = 1000 μF
  }
  state { Vc: Voltage = 0 V }
  discrete { connected: Boolean = true }
  unknown { I: Current }
  flow der(Vc) = I / C
  equation kvl: V0 * (if connected then 1 else 0) == I * R + Vc
  event off on at t0 + 3 s { set connected = false }
}

presentation SwitchedFacts for Switched {
  observe {
    before = I at t0 + 2.9 s
    after  = I at t0 + 3.5 s
  }
}
```

```cases
run discharges of Switched with SwitchedFacts {
  until t0 + 4 s
  expect {
    before == 0.33014 mA within 0.001 mA      // 6 V e^-2.9 / 1 kΩ, still charging
    after == -3.4580 mA within 0.001 mA
  }
}
```

**Exercise 3.** The battery reversed. With `V0` negative the diode is turned against the current: `exp(Vd / Vt)` is tiny for a negative `Vd`, so the diode lets through at most `Is`, a millionth of a microampere. Almost no current flows, so the resistor takes almost no voltage and the diode takes nearly all of it: `Vd` follows `V0`. That is what a diode is for: it conducts one way only.

```text
model Reversed {
  param {
    V0: Voltage    = -5 V      in [-10 V, 10 V]
    R:  Resistance = 1 kΩ
    Is: Current    = 1e-12 A
    Vt: Voltage    = 25.85 mV
  }
  unknown {
    I:  Current = 0 A
    Vd: Voltage = -1 V
  }
  equation resistor: V0 - Vd == I * R
  equation shockley: I == Is * (exp(Vd / Vt) - 1)
}

presentation ReversedLab for Reversed {
  panel controls { slider(V0, range: [-10 V, 10 V]) }
  panel results {
    label(I)
    label(Vd)
  }
  observe {
    leak = I at t0
    drop = Vd at t0
  }
}
```

```cases
run blocks of Reversed with ReversedLab {
  expect {
    leak == -1e-12 A within 1e-15 A
    drop == -5 V within 1e-6 V
  }
}
```

The guesses matter here, as the chapter says: a guess on the reverse side, `Vd = -1 V`, leads the solver to this solution.

## Chapter 18

**Exercise 1.** A starting speed of `(1 m/s, 0 m/s)`. At 10 degrees from the bottom the rod points along `(sin 10°, -cos 10°)`, and the velocity has a part along it, `sin 10° = 0.174 m/s`: the bob would leave its circle at once. The runtime moves the state onto the constraint and its rates at the start, by the smallest change: it removes the part along the rod and keeps the part along the circle, `cos 10° = 0.985 m/s`. The bob starts at that speed, moving along its circle.

```text
space Plane = euclidean(2)

model Pushed in Plane {
  param {
    g:     Acceleration = 9.81 m/s^2
    L:     Length       = 1 m
    m:     Mass         = 1 kg
    pivot: Point        = origin
    θ0:    Angle        = 10 deg
  }
  state {
    bob: Point            = pivot + L * (sin(θ0), -cos(θ0))
    vel: Vector<Velocity> = (1 m/s, 0 m/s)
  }
  unknown { tension: Force = 10 N }
  derived {
    along: Vector<Length> = bob - pivot
    pull:  Vector<Force>  = -tension * along / |along|
    speed: Velocity       = |vel|
    outward: Velocity     = dot(vel, along) / |along|
  }
  flow {
    der(bob) = vel
    der(vel) = (0 m/s^2, -g) + pull / m
  }
  constraint rod: |bob - pivot| == L policy enforce
}

presentation PushedFacts for Pushed {
  observe {
    start_speed = speed at t0
    start_out   = outward at t0
  }
}
```

```cases
run kept_on of Pushed with PushedFacts {
  until t0 + 1 s
  expect {
    start_speed == 0.984808 m/s within 1e-6 m/s     // cos 10°
    start_out == 0 m/s within 1e-9 m/s
  }
}
```

**Exercise 2.** A bead in a bowl, `pos.y == k * pos.x^2`. The wire still pushes at right angles to itself, but the direction now changes along the way: at `x` it is `(-2 k x, 1)` divided by its length, up and in towards the bottom. Released at `x = 1 m` with `k = 0.5 /m`, the slope there is 45 degrees and the push starts at `m g cos 45° = 13.8734 N`. At the bottom, 0.5 m lower, the bead's speed is `sqrt(2 g * 0.5 m) = 3.13209 m/s`; the bowl's radius of curvature there is `1 / (2 k) = 1 m`, so the wire pushes with the weight plus what turns the bead's path: `m (g + v² / 1 m) = 39.24 N`.

```text
space Plane = euclidean(2)

model Bowl in Plane {
  param {
    g: Acceleration    = 9.81 m/s^2
    m: Mass            = 2 kg
    k: Quantity<1/L>   = 0.5 /m
  }
  state {
    pos: Point            = origin + (1 m, 0.5 m)
    vel: Vector<Velocity> = 0
  }
  unknown { N: Force }
  derived {
    normal: Vector<1> = (-2 * k * pos.x, 1) / sqrt(1 + 4 * k^2 * pos.x^2)
    speed:  Velocity  = |vel|
  }
  flow {
    der(pos) = vel
    der(vel) = (0 m/s^2, -g) + N / m * normal
  }
  constraint wire: pos.y == k * pos.x^2 policy enforce
  event bottom on falling(pos.x)
}

presentation BowlFacts for Bowl {
  observe {
    start_push = N at t0
    low_speed  = speed on bottom
    low_push   = N on bottom
  }
}
```

```cases
run slides of Bowl with BowlFacts {
  until t0 + 2 s
  expect {
    start_push == 13.8734 N within 1e-4 N
    low_speed[1] == 3.13209 m/s within 1e-4 m/s
    low_push[1] == 39.24 N within 1e-3 N
  }
}
```

**Exercise 3.** A spring for the upper rod. The spring's force is computed from the positions, so it is not an unknown: `T1` and the constraint `upper` go. Kept, `T1` is an unknown with nothing left to determine it, two unknowns and one constraint:

```error
// error: MK-E34
space Plane = euclidean(2)

model Kept in Plane {
  param {
    g:  Acceleration     = 9.81 m/s^2
    k:  Quantity<M/T^2>  = 200 N/m
    L1: Length = 1 m
    L2: Length = 0.7 m
    m1: Mass   = 1 kg
    m2: Mass   = 0.5 kg
  }
  state {
    p1: Point            = origin + (1 m, 0 m)
    v1: Vector<Velocity> = 0
    p2: Point            = origin + (1 m, -0.7 m)
    v2: Vector<Velocity> = 0
  }
  unknown {
    T1: Force
    T2: Force
  }
  derived {
    u1: Vector<1> = (p1 - origin) / |p1 - origin|
    u2: Vector<1> = (p2 - p1) / |p2 - p1|
  }
  flow {
    der(p1) = v1
    der(v1) = (0 m/s^2, -g) - k * (|p1 - origin| - L1) / m1 * u1 - T1 / m1 * u1 + T2 / m1 * u2
    der(p2) = v2
    der(v2) = (0 m/s^2, -g) - T2 / m2 * u2
  }
  constraint lower: |p2 - p1| == L2 policy enforce
}
```

Without `T1`, the model runs. The energy now includes the spring's, `k (|p1| - L1)² / 2`, and is conserved as before; it starts at the same `-3.4335 J`, since the spring starts at its rest length:

```text
space Plane = euclidean(2)

model SpringRod in Plane {
  param {
    g:  Acceleration     = 9.81 m/s^2
    k:  Quantity<M/T^2>  = 200 N/m
    L1: Length = 1 m
    L2: Length = 0.7 m
    m1: Mass   = 1 kg
    m2: Mass   = 0.5 kg
  }
  state {
    p1: Point            = origin + (1 m, 0 m)
    v1: Vector<Velocity> = 0
    p2: Point            = origin + (1 m, -0.7 m)
    v2: Vector<Velocity> = 0
  }
  unknown { T2: Force }
  derived {
    u1: Vector<1> = (p1 - origin) / |p1 - origin|
    u2: Vector<1> = (p2 - p1) / |p2 - p1|
    stretch: Length = |p1 - origin| - L1
    energy: Energy = 0.5 * m1 * |v1|^2 + 0.5 * m2 * |v2|^2 + m1 * g * p1.y + m2 * g * p2.y + 0.5 * k * stretch^2
  }
  flow {
    der(p1) = v1
    der(v1) = (0 m/s^2, -g) - k * stretch / m1 * u1 + T2 / m1 * u2
    der(p2) = v2
    der(v2) = (0 m/s^2, -g) - T2 / m2 * u2
  }
  constraint lower: |p2 - p1| == L2 policy enforce
}

presentation SpringRodFacts for SpringRod {
  observe {
    initial = energy at t0
    later   = energy at t0 + 5 s
  }
}
```

```cases
run springs of SpringRod with SpringRodFacts {
  until t0 + 5 s
  expect {
    initial == -3.4335 J within 1e-9 J
    later == -3.4335 J within 0.01 J
  }
}
```

## Chapter 19

**Exercise 1.** Balls for the Earth and the Moon. Moved to the top level, `Ball` reads nothing of the model around it, so the only thing that differs between the two worlds, gravity, must come in from outside: `g` is the input. Chapter 9's `Ball` already declared `g` as an input, which is why it moves out unchanged. Each model connects it to its own gravity:

```text
space Plane = euclidean(2)

object Ball {
  param { r: Length = 0.1 m }
  input { g: Acceleration = 9.81 m/s^2 }
  state {
    pos: Point            = origin + (0 m, 1 m)
    vel: Vector<Velocity> = 0
  }
  flow {
    der(pos) = vel
    der(vel) = (0 m/s^2, -g)
  }
}

model EarthDrop in Plane {
  parts { ball: Ball { g = 9.81 m/s^2 } }
}

model MoonDrop in Plane {
  parts { ball: Ball { g = 1.62 m/s^2 } }
}

presentation EarthFacts for EarthDrop {
  observe { height = ball.pos.y at t0 + 0.4 s }
}

presentation MoonFacts for MoonDrop {
  observe { height = ball.pos.y at t0 + 1 s }
}
```

```cases
run earth of EarthDrop with EarthFacts {
  until t0 + 0.4 s
  expect { height == 0.2152 m within 1e-9 m }    // 1 m - 9.81 m/s² (0.4 s)² / 2
}

run moon of MoonDrop with MoonFacts {
  until t0 + 1 s
  expect { height == 0.19 m within 1e-9 m }      // 1 m - 1.62 m/s² (1 s)² / 2
}
```

**Exercise 2.** A module `snell`. Snell's law gives the sine of the refracted angle, `sin θ2 = n1 sin θ1 / n2`; the angle whose sine is `s` is `atan2(s, sqrt(1 - s²))`. The function is a shared definition. Saved on its own as `snell.prismal`, it is the module `snell`, and a model file next to it starts with `use snell`; here it is written in the same file so that the guide's tests can run it. Into water at 30 degrees, the ray turns to `asin(sin 30° / 1.33) = 22.0824°`. The first-party `optics` module works with directions instead of angles: `refract` turns a direction, and `angle_between` measures the result. Both give the same angle:

```text
use optics

space Plane = euclidean(2)

fn refraction_angle(n1: Real, n2: Real, θ: Angle): Angle = atan2(n1 * sin(θ) / n2, sqrt(1 - (n1 * sin(θ) / n2)^2))

model Ray in Plane {
  param {
    n1: Real  = 1
    n2: Real  = 1.33
    θ:  Angle = 30 deg unit deg
  }
  derived {
    by_angle:  Angle     = refraction_angle(n1, n2, θ)
    along:     Vector<1> = (sin(θ), -cos(θ))
    onward:    Vector<1> = refract(along, (0, 1), n1, n2)
    by_optics: Angle     = angle_between(onward, (0, -1))
  }
}

presentation RayFacts for Ray {
  observe {
    snell_angle  = by_angle at t0
    optics_angle = by_optics at t0
  }
}
```

```cases
run into_water of Ray with RayFacts {
  expect {
    snell_angle == 22.0824 deg within 1e-4 deg
    optics_angle == 22.0824 deg within 1e-4 deg
  }
}
```

`refraction_angle` answers one question about angles; `optics` describes rays as points and directions, so the same functions follow a ray through any number of surfaces, in a plane or a room, and handle total internal reflection.

## Chapter 20

**Exercises 1 and 2.** Without speed along `z`, the force `q v × B` has no part along `z` either, and nothing ever gives the particle one: it goes round a circle of radius 4 m in the floor, coming to the far side, `y = -4 m`, after half a turn, `4 π s`. With speed only along the field, `v × B` is zero: no force, and the particle goes straight up. A view from straight above (`elevation: 90 deg`) looks down the `z` axis, which points at the viewer; motion along `z` does not show there, so even the spiral of the chapter is drawn as a circle.

```text
space Room = euclidean(3)

model Flat in Room {
  param {
    q:   Charge           = 1 C
    m:   Mass             = 2 kg
    B:   Vector<M/T^2/I>  = (0 kg/s^2/A, 0 kg/s^2/A, 0.5 kg/s^2/A)
    v0:  Vector<Velocity> = (1 m/s, 0 m/s, 0 m/s)
  }
  state {
    pos: Point            = origin + (0 m, 4 m, 0 m)
    vel: Vector<Velocity> = v0
  }
  flow {
    der(pos) = vel
    der(vel) = q * cross(vel, B) / m
  }
}

presentation TwoSides for Flat {
  until t0 + 30 s
  view side: spatial(Room, scale: 1 m -> 30 px, azimuth: 35 deg, elevation: 25 deg) {
    grid
    axes
    trace(pos every 0.05 s)
  }
  view above: spatial(Room, scale: 1 m -> 30 px, azimuth: -90 deg, elevation: 90 deg) {
    axes
    trace(pos every 0.05 s)
  }
  observe {
    across = pos.y at t0 + 12.566370614359172 s
    height = pos.z at t0 + 12.566370614359172 s
  }
}
```

```cases
run in_the_floor of Flat with TwoSides {
  until t0 + 13 s
  expect {
    across == -4 m within 1e-4 m          // half a turn, 4 π s: the far side of the circle
    height == 0 m within 1e-9 m
  }
}

run along_the_field of Flat with TwoSides {
  param { v0 = (0 m/s, 0 m/s, 0.15 m/s) }
  until t0 + 13 s
  expect {
    across == 4 m within 1e-9 m           // no force: it never leaves the line x = 0, y = 4 m
    height == 1.88496 m within 1e-5 m     // 0.15 m/s for 4 π s
  }
}
```

**Exercise 3.** A torque. `τ = r × F` is at right angles to both the lever and the force, along the axis the force would turn the lever about. A force of 2 N across a 1 m lever along `x`, pushing toward `y`, would turn it anticlockwise seen from above, and the torque points **up**, `(0, 0, 2) N·m`: curl the fingers of the right hand in the direction of turning and the thumb gives the torque. The opposite push gives a torque pointing down.

```text
space Room = euclidean(3)

model Lever in Room {
  param {
    r: Vector<Length> = (1 m, 0 m, 0 m)
    F: Vector<Force>  = (0 N, 2 N, 0 N)
  }
  derived { τ: Vector<M*L^2/T^2> = cross(r, F) }
}

presentation LeverView for Lever {
  view room: spatial(Room, scale: 1 m -> 80 px, azimuth: 35 deg, elevation: 25 deg) {
    grid
    arrow(r, from: origin, scale: 1 m -> 80 px, color: gray) as lever
    arrow(F, from: origin + r, scale: 1 N -> 30 px, color: red) as force
    arrow(τ, from: origin, scale: 1 N*m -> 30 px, color: blue) as torque
  }
  observe { up = τ.z at t0 }
}
```

```cases
run anticlockwise of Lever with LeverView {
  expect { up == 2 N*m within 1e-12 N*m }
}

run clockwise of Lever with LeverView {
  param { F = (0 N, -2 N, 0 N) }
  expect { up == -2 N*m within 1e-12 N*m }
}
```

**Exercises 4 and 5.** An azimuth that reads time turns the view steadily, here 6 degrees a second, once round a minute, while the planet goes round its own orbit: the two motions add, and the planet seems to go round faster or slower depending on whether the view turns with it or against it. With `distance: 5 m` the viewer stands only 2 m beyond the orbit's near side and 8 m from its far side, so the near side is drawn `8 / 2 = 4` times as large as the far side. The door that opens the other way is a second group, hinged 3 m along `x`, with its panel toward the first door and turned by `-swing`:

```text
space Room = euclidean(3)

model TwoDoors in Room {
  param {
    swing: Angle     = 30 deg unit deg
    ball:  Point     = origin + (1.5 m, 1.6 m, 0 m)
    R:     Length    = 3 m
    w:     Frequency = 0.5 /s
  }
  state { phase: Angle = 0 deg }
  derived { planet: Point = origin + (R * cos(phase), R * sin(phase), 0 m) }
  flow { der(phase) = w }
}

presentation Turning for TwoDoors {
  until t0 + 30 s
  view room: spatial(Room, scale: 1 m -> 45 px, azimuth: 6 deg/s * (t - t0), elevation: 25 deg, distance: 5 m) {
    grid
    arc(origin, R, from: 0 deg, to: 360 deg) as path
    circle(planet, 0.3 m, opaque: true, color: blue) as earth
  }
}

presentation Doors for TwoDoors {
  view room: spatial(Room, scale: 1 m -> 60 px, azimuth: -35 deg, elevation: 25 deg) {
    grid
    group(at: origin, rotate: swing, axis: (0, 0, 1)) {
      polygon(origin, origin + (1.4 m, 0 m, 0 m), origin + (1.4 m, 0 m, 2 m), origin + (0 m, 0 m, 2 m), opaque: true) as left_panel
    }
    group(at: origin + (3 m, 0 m, 0 m), rotate: -swing, axis: (0, 0, 1)) {
      polygon(origin, origin + (-1.4 m, 0 m, 0 m), origin + (-1.4 m, 0 m, 2 m), origin + (0 m, 0 m, 2 m), opaque: true) as right_panel
    }
    marker(ball) as b { on drag across (0, 0, 1) as p { propose ball = p } }
  }
  panel controls { slider(swing, range: [0 deg, 120 deg]) }
  permit learner { orbit }
}
```

Both doors turn their free edges toward `+y` as `swing` grows, the left one anticlockwise and the right one clockwise seen from above, like a pair of double doors opening away from the viewer.

## Chapter 21

**Exercise 1.** Selection sort. For each position `i`, an inner loop keeps in `smallest` the number of the smallest value seen from `i` on; after it, that value is swapped into place. It makes the same 28 comparisons as bubble sort, but at most one swap per position: 7 positions, since when the first seven are in place the eighth is too. On this list it swaps 6 times, because at `i = 2` the smallest of the rest, 2, is already in place.

```text
model Selection {
  object Bar {
    param { k: Real = 0 }
    discrete { v: Real = 0 }
  }
  parts {
    bars: Bar[8] {
      k = index
      v = (5, 2, 7, 1, 8, 4, 6, 3)[index]
    }
  }
  discrete {
    i:           Real    = 0
    j:           Real    = 0
    smallest:    Real    = 0
    held:        Real    = 0
    comparisons: Real    = 0
    swaps:       Real    = 0
    sorted:      Boolean = false
  }
  algorithm sort on every 0.5 s {
    for i in [1, 7] {
      set smallest = i
      for j in [i + 1, 8] {
        set comparisons = comparisons + 1
        if bars[j].v < bars[smallest].v {
          set smallest = j
        }
        step
      }
      if smallest != i {
        set held = bars[i].v
        set bars[i].v = bars[smallest].v
        set bars[smallest].v = held
        set swaps = swaps + 1
      }
    }
    set sorted = true
  }
}

presentation SelectionFacts for Selection {
  observe {
    first     = bars[1].v at t0 + 15 s
    last      = bars[8].v at t0 + 15 s
    compared  = comparisons at t0 + 15 s
    exchanged = swaps at t0 + 15 s
    finished  = sorted at t0 + 15 s
  }
}
```

```cases
run select_eight of Selection with SelectionFacts {
  until t0 + 15 s
  expect {
    first == 1 exactly
    last == 8 exactly
    compared == 28 exactly       // 7 + 6 + ... + 1
    exchanged == 6 exactly       // positions 1, 3, 4, 5, 6, 7; position 2 already holds 2
    finished == true
  }
}
```

**Exercise 2.** The sort stepped by hand. As in the search, the algorithm follows an event, `compare`, which the learner requests with a button and a clock emits while `auto` is on. With `auto` off, nothing is compared until the learner presses the button:

```text
model HandSort {
  object Bar {
    param { k: Real = 0 }
    discrete { v: Real = 0 }
  }
  parts {
    bars: Bar[8] {
      k = index
      v = (5, 2, 7, 1, 8, 4, 6, 3)[index]
    }
  }
  param { auto: Boolean = true }
  discrete {
    i:           Real = 0
    j:           Real = 0
    held:        Real = 0
    comparisons: Real = 0
  }
  event compare on request
  event tick on every 0.5 s if auto { emit compare }
  algorithm sort on compare {
    for i in [1, 7] {
      for j in [1, 8 - i] {
        set comparisons = comparisons + 1
        if bars[j].v > bars[j + 1].v {
          set held = bars[j].v
          set bars[j].v = bars[j + 1].v
          set bars[j + 1].v = held
        }
        step
      }
    }
  }
}

presentation HandSortLab for HandSort {
  until t0 + 16 s
  view chart: plot(x: [0, 9], y: [0, 9]) {
    for b in bars {
      segment((b.k, 0), (b.k, b.v), color: gray, width: 8, when: b.k != j and b.k != j + 1) as bar
      segment((b.k, 0), (b.k, b.v), color: orange, width: 8, when: b.k == j or b.k == j + 1) as compared
    }
  }
  panel controls {
    toggle(auto)
    button(compare, label: "Compare")
    label(comparisons)
    button(restart)
  }
  observe { compared = comparisons at t0 + 15 s }
}
```

```cases
run on_the_clock of HandSort with HandSortLab {
  until t0 + 15 s
  expect { compared == 28 exactly }
}

run by_hand of HandSort with HandSortLab {
  param auto = false
  until t0 + 15 s
  expect { compared == 0 exactly }   // no one pressed the button
}
```

**Exercise 3.** Insertion sort. Each value in turn is lifted into `held`, and larger values to its left move one place right until the place for it is found. The loop cannot read `bars[0]`, which does not exist, so instead of testing `j >= 1 and bars[j].v > held` in one condition it keeps a discrete `moving` and stops when `j` reaches 0. A list already sorted needs one comparison per value, 7; a list in reverse order needs the most, every value compared with every value to its left, `1 + 2 + ... + 7 = 28`. The chapter's list needs 19.

```text
model Insertion {
  object Bar {
    param { k: Real = 0 }
    discrete { v: Real = 0 }
  }
  param {
    a1: Real = 5
    a2: Real = 2
    a3: Real = 7
    a4: Real = 1
    a5: Real = 8
    a6: Real = 4
    a7: Real = 6
    a8: Real = 3
  }
  parts {
    bars: Bar[8] {
      k = index
      v = (a1, a2, a3, a4, a5, a6, a7, a8)[index]
    }
  }
  discrete {
    i:           Real    = 0
    j:           Real    = 0
    held:        Real    = 0
    moving:      Boolean = false
    comparisons: Real    = 0
  }
  algorithm sort on every 0.5 s {
    for i in [2, 8] {
      set held = bars[i].v
      set j = i - 1
      set moving = true
      while moving {
        set comparisons = comparisons + 1
        if bars[j].v > held {
          set bars[j + 1].v = bars[j].v
          set j = j - 1
          set moving = j >= 1
        } else {
          set moving = false
        }
        step
      }
      set bars[j + 1].v = held
    }
  }
}

presentation InsertionFacts for Insertion {
  observe {
    first    = bars[1].v at t0 + 15 s
    last     = bars[8].v at t0 + 15 s
    compared = comparisons at t0 + 15 s
  }
}
```

```cases
run chapter_list of Insertion with InsertionFacts {
  until t0 + 15 s
  expect {
    first == 1 exactly
    last == 8 exactly
    compared == 19 exactly
  }
}

run reversed of Insertion with InsertionFacts {
  param { a1 = 8; a2 = 7; a3 = 6; a4 = 5; a5 = 4; a6 = 3; a7 = 2; a8 = 1 }
  until t0 + 15 s
  expect {
    first == 1 exactly
    compared == 28 exactly
  }
}

run already_sorted of Insertion with InsertionFacts {
  param { a1 = 1; a2 = 2; a3 = 3; a4 = 4; a5 = 5; a6 = 6; a7 = 7; a8 = 8 }
  until t0 + 15 s
  expect { compared == 7 exactly }
}
```

**Exercise 4.** Euclid by subtraction. The larger number is replaced by the difference until the two are equal; both are then the greatest common divisor. For 1071 and 462 it takes **11** steps, where the remainders took 3: `1071 - 462 - 462` is two subtractions for one remainder, and `147` is taken from `462` three times and `21` from `147` six times.

```text
model Subtract {
  discrete {
    a:     Real = 1071
    b:     Real = 462
    steps: Real = 0
  }
  algorithm gcd on every 1 s {
    while a != b {
      if a > b {
        set a = a - b
      } else {
        set b = b - a
      }
      set steps = steps + 1
      step
    }
  }
}

presentation SubtractChecks for Subtract {
  observe {
    divisor = a at t0 + 12 s
    taken   = steps at t0 + 12 s
  }
}
```

```cases
run gcd_by_subtraction of Subtract with SubtractChecks {
  until t0 + 12 s
  expect {
    divisor == 21 exactly
    taken == 11 exactly      // 2 + 3 + 6: the quotients of the remainder method
  }
}
```

## Chapter 22

**Exercise 1.** Minimum deviation. The angle of incidence `θ` is a slider; the ray's direction is built from it, `θ` from the face's inward normal. The red ray's turn falls as `θ` grows from 40 degrees, is least at `θ = 49.164°`, and grows again. The least turn is where the ray crosses the glass parallel to the base, entering and leaving symmetrically: the two angles are the same. There `n sin(60° / 2) = sin θ`, so `θ = asin(1.51317 / 2) = 49.164°`, and the turn is `2 θ - 60° = 38.328°`.

```text
use optics

space Plane = euclidean(2)

model Deviation in Plane {
  param {
    θ: Angle = 49.164 deg unit deg in [30 deg, 80 deg]
    A: Real  = 1.5046
    B: Area  = 4200 nm^2
  }
  derived {
    left_n:   Vector<1> = (-sqrt(3) / 2, 0.5)
    right_n:  Vector<1> = (sqrt(3) / 2, 0.5)
    aim:      Vector<1> = (cos(θ) * sqrt(3) / 2 + sin(θ) * 0.5, -cos(θ) * 0.5 + sin(θ) * sqrt(3) / 2)
    n_red:    Real      = cauchy(A, B, 700 nm)
    in_red:   Vector<1> = refract(aim, left_n, 1, n_red)
    out_red:  Vector<1> = refract(in_red, right_n, n_red, 1)
    turn:     Angle     = angle_between(aim, out_red) unit deg
    climb:    Real      = in_red.y
  }
}

presentation DeviationLab for Deviation {
  panel controls {
    slider(θ, range: [30 deg, 80 deg])
    label(turn)
  }
  observe {
    least  = turn at t0
    inside = climb at t0
  }
}
```

```cases
run symmetric of Deviation with DeviationLab {
  expect {
    least == 38.3282 deg within 1e-4 deg
    inside == 0 within 1e-5             // parallel to the base inside the glass
  }
}

run steeper of Deviation with DeviationLab {
  param θ = 45 deg
  expect { least == 38.6111 deg within 1e-4 deg }
}

run shallower of Deviation with DeviationLab {
  param θ = 55 deg
  expect { least == 38.8080 deg within 1e-4 deg }
}
```

**Exercise 2.** A spherical mirror and its aberration. The mirror is the part of a circle of radius 40 cm, centre `c` 40 cm in front of it, near the axis. The rays start inside the circle, so `hit_sphere` finds the far side, the mirror. Each is reflected about the circle's normal there, and `hit_plane` finds where it crosses the axis. A ray at height `h` meets the mirror where the normal makes the angle `α` with the axis, `sin α = h / R`, and crosses the axis `R / (2 cos α)` from the centre: at 19.84 cm, 18.43 cm and 14.38 cm from the mirror for heights 5, 15 and 25 cm. Only rays near the axis cross at the paraxial focus, `R / 2 = 20 cm`. Far from the axis the circle curves more steeply than the parabola that would bring every ray to one point, so it turns those rays more and they cross nearer the mirror.

```text
use optics

space Plane = euclidean(2)

model Aberration in Plane {
  param { R: Length = 0.4 m }
  derived {
    c:    Point     = origin + (R, 0 m)
    back: Vector<1> = (-1, 0)
    up:   Vector<1> = (0, 1)
    p1:   Point     = origin + (0.6 m, 0.05 m)
    p2:   Point     = origin + (0.6 m, 0.15 m)
    p3:   Point     = origin + (0.6 m, 0.25 m)
    q1:   Point     = hit_sphere(p1, back, c, R)
    q2:   Point     = hit_sphere(p2, back, c, R)
    q3:   Point     = hit_sphere(p3, back, c, R)
    r1:   Vector<1> = reflect(back, sphere_normal(q1, c))
    r2:   Vector<1> = reflect(back, sphere_normal(q2, c))
    r3:   Vector<1> = reflect(back, sphere_normal(q3, c))
    x1:   Length    = (hit_plane(q1, r1, origin, up) - origin).x
    x2:   Length    = (hit_plane(q2, r2, origin, up) - origin).x
    x3:   Length    = (hit_plane(q3, r3, origin, up) - origin).x
  }
}

presentation MirrorBench for Aberration {
  view bench: spatial(Plane, scale: 1 m -> 800 px, y: up) {
    arc(c, R, from: 130 deg, to: 230 deg, color: blue, width: 3) as mirror
    segment(origin, origin + (0.7 m, 0 m), color: gray) as axis_line
    segment(p1, q1, color: red) as in1
    segment(q1, q1 + 0.5 m * r1, color: red) as out1
    segment(p2, q2, color: green) as in2
    segment(q2, q2 + 0.5 m * r2, color: green) as out2
    segment(p3, q3, color: purple) as in3
    segment(q3, q3 + 0.5 m * r3, color: purple) as out3
  }
  observe {
    near = x1 at t0
    mid  = x2 at t0
    far  = x3 at t0
  }
}
```

```cases
run spherical of Aberration with MirrorBench {
  expect {
    near == 0.198419 m within 1e-6 m    // R - R / (2 cos α), sin α = 5 / 40
    mid == 0.184256 m within 1e-6 m     // sin α = 15 / 40
    far == 0.143795 m within 1e-6 m     // sin α = 25 / 40
  }
}
```

**Exercise 3.** A fourth ray. `thin_lens` treats every ray alike, so any ray from the object's tip that crosses the lens meets the image's tip. Here the fourth ray leaves the tip slanting down, `(1, -0.05)`, and is checked for the object at 30 cm and at 20 cm (where `v = 20 cm` and the image is as tall as the object, upside down):

```text
use optics

space Plane = euclidean(2)

model FourthRay in Plane {
  param {
    u: Length = 30 cm in [12 cm, 60 cm]
    h: Length = 2 cm
    f: Length = 10 cm
  }
  derived {
    tip:   Point     = origin + (-u, h)
    axis:  Vector<1> = (1, 0)
    d4:    Vector<1> = (1, -0.05)
    q4:    Point     = hit_plane(tip, d4, origin, axis)
    out4:  Vector<1> = thin_lens(d4, q4, origin, axis, f)
    v:     Length    = image_distance(u, f)
    image: Point     = origin + (v, magnification(u, f) * h)
    at4:   Length    = (hit_plane(q4, out4, image, axis) - origin).y
  }
}

presentation FourthLab for FourthRay {
  view bench: spatial(Plane, scale: 1 m -> 1200 px, y: up) {
    segment(origin + (-0.65 m, 0 m), origin + (0.4 m, 0 m), color: gray) as axis_line
    segment(origin + (0 m, -0.05 m), origin + (0 m, 0.05 m), color: blue, width: 3) as lens
    segment(tip, q4, color: orange) as ray4_in
    segment(q4, q4 + 0.4 m * out4, color: orange) as ray4_out
    marker(image) as image_tip
  }
  panel controls { slider(u, range: [12 cm, 60 cm]) }
  observe {
    cross4 = at4 at t0
    tip_y  = (image - origin).y at t0
  }
}
```

```cases
run at_thirty of FourthRay with FourthLab {
  expect {
    cross4 == -0.01 m within 1e-12 m
    tip_y == -0.01 m within 1e-12 m
  }
}

run at_twenty of FourthRay with FourthLab {
  param u = 20 cm
  expect {
    cross4 == -0.02 m within 1e-12 m    // v = 20 cm, magnification -1
    tip_y == -0.02 m within 1e-12 m
  }
}
```

## Chapter 23

**Exercise 1.** The velocity in a level frame. A frame whose first axis is horizontal, `(1, 0)`, reads the velocity's level and vertical parts. Down the slope the ball's velocity is `g sin θ t` along the slope, so its vertical part is `-g sin² θ t`: it grows steadily, so it is largest at the latest moment, and larger on a steeper slope. At 30 degrees after 1 s it is `-9.81 * 0.25 = -2.4525 m/s`, and the level part `g sin θ cos θ t = 4.24785 m/s`.

```text
space Plane = euclidean(2)

model LevelRead in Plane {
  param {
    g: Acceleration = 9.81 m/s^2
    θ: Angle        = 30 deg unit deg
  }
  state {
    pos: Point            = origin
    vel: Vector<Velocity> = 0
  }
  derived {
    along:    Vector<1>            = (cos(θ), -sin(θ))
    gravity:  Vector<Acceleration> = (0 m/s^2, -g)
    down:     Acceleration         = gravity.in(slope).x
    sideways: Velocity             = vel.in(level).x
    upward:   Velocity             = vel.in(level).y
  }
  frame slope(at: origin, x: along)
  frame level(at: origin, x: (1, 0))
  flow {
    der(pos) = vel
    der(vel) = down * along
  }
}

presentation LevelFacts for LevelRead {
  observe {
    level_part    = sideways at t0 + 1 s
    vertical_part = upward at t0 + 1 s
  }
}
```

```cases
run read_level of LevelRead with LevelFacts {
  until t0 + 1 s
  expect {
    level_part == 4.247855 m/s within 1e-6 m/s
    vertical_part == -2.4525 m/s within 1e-6 m/s
  }
}
```

**Exercise 2.** A ball that keeps pace with the carousel. A point of the carousel 1 m from the axis moves at `w * 1 m = 0.5 m/s`, and at the start that point moves along `y`, exactly as the ball does. Ball and seat start with the same velocity, so from the seat the ball seems at rest. It only seems so for a moment: the ball goes on in a straight line while the seat turns away from it, and the ball drifts outward and back. After 0.2 s it has moved a third of a millimetre across the seat; a ball at rest on the ground would have moved 10 cm the other way.

```text
space Plane = euclidean(2)

model Paced in Plane {
  param {
    w:  Frequency        = 0.5 /s
    v0: Vector<Velocity> = (0 m/s, 0.5 m/s)
  }
  state {
    pos: Point            = origin + (1 m, 0 m)
    vel: Vector<Velocity> = v0
  }
  derived {
    turn:   Angle  = w * (t - t0)
    radial: Length = pos.in(turning).x
    side:   Length = pos.in(turning).y
  }
  frame turning(at: origin, x: (cos(turn), sin(turn)))
  flow {
    der(pos) = vel
    der(vel) = 0
  }
}

presentation PacedFacts for Paced {
  observe {
    out    = radial at t0 + 0.2 s
    across = side at t0 + 0.2 s
  }
}
```

```cases
run keeps_pace of Paced with PacedFacts {
  until t0 + 0.2 s
  expect {
    out == 1.004988 m within 1e-6 m       // cos(0.1) + 0.1 sin(0.1)
    across == -0.000333 m within 1e-6 m   // -sin(0.1) + 0.1 cos(0.1)
  }
}

run left_behind of Paced with PacedFacts {
  param { v0 = (0 m/s, 0 m/s) }
  until t0 + 0.2 s
  expect { across == -0.099833 m within 1e-6 m }   // -sin(0.1): the seat moves on without it
}
```

**Exercise 3.** The puck from the ground. Seen from the ground, a puck released at rest on the turntable 0.5 m from the axis is moving with the table, at `1 /s * 0.5 m = 0.5 m/s` across, and with no forces it goes on in a straight line. Read in the turning frame, its position is exactly where the chapter's model, with the Coriolis and centrifugal forces, puts it: at 2 s, `(0.5 cos 2 + sin 2, -0.5 sin 2 + cos 2) = (0.70122 m, -0.87080 m)`. The two descriptions are the same motion; the forces are what a turning observer must add to explain a straight path that looks curved to them.

```text
space Room = euclidean(3)

model Straight in Room {
  param { w: Frequency = 1 /s }
  state {
    pos: Point            = origin + (0.5 m, 0 m, 0 m)
    vel: Vector<Velocity> = (0 m/s, 0.5 m/s, 0 m/s)
  }
  derived {
    turn: Angle  = w * (t - t0)
    seen_x: Length = pos.in(turning).x
    seen_y: Length = pos.in(turning).y
  }
  frame turning(at: origin, x: (cos(turn), sin(turn), 0), y: (-sin(turn), cos(turn), 0))
  flow {
    der(pos) = vel
    der(vel) = 0
  }
}

presentation FromGround for Straight {
  until t0 + 6 s
  view top: spatial(Room, scale: 1 m -> 50 px, azimuth: -90 deg, elevation: 90 deg, frame: turning) {
    grid
    trace(pos every 0.05 s)
    marker(pos) as puck
  }
  observe {
    x2 = seen_x at t0 + 2 s
    y2 = seen_y at t0 + 2 s
  }
}
```

```cases
run straight_line of Straight with FromGround {
  until t0 + 2 s
  expect {
    x2 == 0.701224 m within 1e-6 m
    y2 == -0.870796 m within 1e-6 m
  }
}
```

The chapter's turntable model, written in the turning frame with the two forces, reaches the same point:

```text
use mechanics

space Room = euclidean(3)

model WithForces in Room {
  param {
    m:    Mass        = 1 kg
    spin: Vector<1/T> = (0 /s, 0 /s, 1 /s)
  }
  state {
    pos: Point            = origin + (0.5 m, 0 m, 0 m)
    vel: Vector<Velocity> = 0
  }
  flow {
    der(pos) = vel
    der(vel) = (coriolis(m, spin, vel) + centrifugal(m, spin, pos - origin)) / m
  }
}

presentation ForcesFacts for WithForces {
  observe {
    x2 = pos.x at t0 + 2 s
    y2 = pos.y at t0 + 2 s
  }
}
```

```cases
run same_point of WithForces with ForcesFacts {
  until t0 + 2 s
  expect {
    x2 == 0.701224 m within 1e-5 m
    y2 == -0.870796 m within 1e-5 m
  }
}
```

## Chapter 24

**Exercise 1.** A third source behind the two. On a line of calm the first two waves cancel, crest on trough. A third wave arriving there is not cancelled by anything, so the water still swings, with the third wave's own amplitude, 1: **no line of calm survives**. The three waves still cancel at some places, but at isolated points, not along lines. To check it, the amplitude of each pattern is the length of the sum of the waves as arrows turning at their phases, `k r`. At a point exactly on the pair's first line of calm (where the paths differ by half a wavelength, 17.087 cm up at 1 m), the pair's amplitude is 0 and the three's is 1:

```text
use optics

space Plane = euclidean(2)

model ThreeSources in Plane {
  grid tank(x: [-0.3 m, 1.2 m], y: [-0.6 m, 0.6 m], cells: (150, 120))
  param {
    λ: Length    = 0.1 m
    d: Length    = 0.3 m
    w: Frequency = 4 /s
  }
  derived {
    s1: Point = origin + (0 m, d / 2)
    s2: Point = origin + (0 m, -d / 2)
    s3: Point = origin + (-0.2 m, 0 m)
    wave(p in tank): Real = cos(wavenumber(λ) * |p - s1| - w * (t - t0)) + cos(wavenumber(λ) * |p - s2| - w * (t - t0)) + cos(wavenumber(λ) * |p - s3| - w * (t - t0))
    probe: Point = origin + (1 m, 0.17086962 m)
    a1: Real = wavenumber(λ) * |probe - s1|
    a2: Real = wavenumber(λ) * |probe - s2|
    a3: Real = wavenumber(λ) * |probe - s3|
    pair:  Real = sqrt((cos(a1) + cos(a2))^2 + (sin(a1) + sin(a2))^2)
    three: Real = sqrt((cos(a1) + cos(a2) + cos(a3))^2 + (sin(a1) + sin(a2) + sin(a3))^2)
  }
}

presentation ThreeLab for ThreeSources {
  until t0 + 20 s
  view tank: spatial(Plane, scale: 1 m -> 350 px, y: up) {
    heatmap(wave, range: [-3, 3])
    marker(s1) as source1
    marker(s2) as source2
    marker(s3) as source3
    marker(probe) as on_the_line
  }
  observe {
    two_waves   = pair at t0
    three_waves = three at t0
  }
}
```

```cases
run calm_lost of ThreeSources with ThreeLab {
  expect {
    two_waves == 0 within 1e-6       // on the pair's line of calm
    three_waves == 1 within 1e-6     // the third wave is left
  }
}
```

**Exercise 2.** Fringes of real slits. With `a = d / 5`, the slit's first dark place, `λ L / a = 30 mm`, falls on the two-slit pattern's fifth bright fringe, `5 × 6 mm`, which disappears (a **missing order**). Inside the middle band lie the bright fringes at 0, ±6, ±12, ±18 and ±24 mm: **nine**. Each is dimmed by the envelope: at 24 mm to `0.0548` of the middle.

```text
use optics

model RealSlits {
  param {
    λ: Length = 600 nm unit nm
    d: Length = 0.1 mm
    a: Length = 0.02 mm
    L: Length = 1 m
  }
  derived {
    θ(y: Real): Angle   = atan2(y * 1 mm, L)
    both(y: Real): Real = two_slit(d, λ, θ(y)) * single_slit(a, λ, θ(y))
  }
}

presentation RealScreen for RealSlits {
  view screen: plot(x: [-40, 40], y: [0, 1.1]) {
    title("Two slits of width a: brightness against height (mm)")
    function_graph(both, color: λ)
  }
  observe {
    middle = both(0) at t0
    fourth = both(24) at t0
    fifth  = both(30) at t0
  }
}
```

```cases
run envelope of RealSlits with RealScreen {
  expect {
    middle == 1 within 1e-12
    fourth == 0.054836 within 1e-6    // cos² × (sin β / β)², β = π a sin θ / λ at 24 mm
    fifth == 0 within 1e-6            // the missing order
  }
}
```

**Exercise 3.** The best middle filter. With the middle filter at `m` from the first, the light through all three is `1/2 cos² m cos²(90° - m) = 1/2 cos² m sin² m = 1/8 sin²(2 m)`, largest when `2 m = 90°`: at **45 degrees**, a quarter of the light that crossed the first filter. It is halfway because the formula does not change when `m` and `90° - m` are swapped: the two turns play the same part, so the best is where they are equal. At 30 and at 60 degrees the same `0.09375` passes.

```text
use optics

model ThreeFilters {
  param {
    I0:     Real  = 1
    middle: Angle = 45 deg unit deg in [0 deg, 90 deg]
  }
  derived { through: Real = I0 / 2 * malus(middle) * malus(90 deg - middle) }
}

presentation ThreeFiltersLab for ThreeFilters {
  panel filters {
    slider(middle, range: [0 deg, 90 deg])
    label(through)
  }
  observe { passed = through at t0 }
}
```

```cases
run halfway of ThreeFilters with ThreeFiltersLab {
  expect { passed == 0.125 within 1e-12 }
}

run thirty of ThreeFilters with ThreeFiltersLab {
  param middle = 30 deg
  expect { passed == 0.09375 within 1e-12 }
}

run sixty of ThreeFilters with ThreeFiltersLab {
  param middle = 60 deg
  expect { passed == 0.09375 within 1e-12 }
}
```
