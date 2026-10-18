# 18. Constrained Motion

A pendulum's bob cannot leave its circle; a bead cannot leave its wire. Chapter 4 wrote such systems in a coordinate that obeys the constraint by itself (the pendulum's angle), and a `constraint` only checked them. That is still the simplest way when one coordinate does it. When it does not (a bob in `x` and `y`, two linked rods, a point held on a curve while the learner drags it), the runtime can **enforce** the constraint: it keeps the constraint true, and computes the force that holds it.

> **In plain words.**
> - A **constraint** is a condition the motion must obey: `|bob - pivot| == L` says the bob stays at distance `L` from the pivot.
> - `policy enforce` asks the runtime to keep it true, instead of only reporting when it fails.
> - Something must hold the bob on its circle: a rod pulling with a **tension**. You do not know the tension in advance; it is whatever keeps the bob on the circle. So you declare it as an `unknown` (chapter 17) and write it into the bob's flow, as the force a rod exerts along itself.
> - The runtime finds the tension at every instant from the constraint, and nudges the state back onto the circle after every step, so small numerical errors never add up.

## A pendulum on a rod

**Goal.** A pendulum written as a physics book draws it: a bob at a point, pulled down by gravity and along the rod by its tension. The learner sees the tension change through the swing, largest at the bottom, and may drag the bob to another starting place on its circle.
**How it is built.** The bob's position and velocity are `state` (a `Point` and a `Vector`), with flows: gravity and the tension along the rod. `tension` is an `unknown`. The constraint `rod` with `policy enforce` keeps the bob at distance `L`. `bob` is `intervenable` and its marker proposes the dragged point: the runtime moves it onto the circle.

```text
space Plane = euclidean(2)

model RodPendulum in Plane {
  param {
    g:     Acceleration = 9.81 m/s^2
    L:     Length       = 1 m
    m:     Mass         = 1 kg
    pivot: Point        = origin
    θ0:    Angle        = 10 deg
  }
  state {
    bob: Point            = pivot + L * (sin(θ0), -cos(θ0))   intervenable
    vel: Vector<Velocity> = 0
  }
  unknown { tension: Force = 10 N }
  derived {
    along: Vector<Length> = bob - pivot
    pull:  Vector<Force>  = -tension * along / |along|       // the rod's force on the bob
  }
  flow {
    der(bob) = vel
    der(vel) = (0 m/s^2, -g) + pull / m
  }
  constraint rod: |bob - pivot| == L policy enforce
  event upswing on rising(bob.x - pivot.x)
}

presentation RodLab for RodPendulum {
  view scene: spatial(Plane, scale: 1 m -> 150 px, y: up) {
    segment(pivot, bob)
    marker(pivot)
    marker(bob, label: none) as ball { on drag as p { propose bob = p } }
    arrow(pull, from: bob, scale: 1 N -> 4 px, color: red, label: none)
  }
  view force: plot(x: [0 s, 6 s], y: [9 N, 11 N]) {
    series_plot(tension every 0.02 s)
  }
  panel readings { label(tension) }
  observe {
    swings = elapsed on upswing
    at_rest = tension at t0
    at_bottom = tension on upswing
  }
}
```

```cases
run swing of RodPendulum with RodLab {
  until t0 + 4 s
  expect {
    swings[1] == 1.50742 s within 1e-5 s           // 3/4 of the period of chapter 4's pendulum
    at_rest == 9.66096 N within 1e-5 N             // m g cos(10°): the bob at rest
    at_bottom[1] == 10.10807 N within 1e-5 N       // m g (3 - 2 cos(10°)): weight plus the turn
  }
}
```

- `der(vel) = (0 m/s^2, -g) + pull / m` is Newton's second law with two forces; the tension's size is unknown, its direction (along the rod) is not.
- The bob swings exactly as the pendulum of chapter 4 written in its angle: the same period, to the solver's accuracy.
- The tension is `m g cos θ` when the bob is at rest at angle `θ`, and more at the bottom, where the rod must also turn the bob's path: `m g + m v² / L`.
- A drag that drops the bob off the circle puts it on the circle at the nearest point. The velocity keeps only its part along the circle.

## A bead on a wire

**Goal.** A bead slides without friction down a straight wire at an angle. The learner compares its motion with a free fall, and sees the wire push on the bead with less than its weight.
**How it is built.** The wire is the constraint `pos.y == -tan(α) * pos.x`. The wire pushes at right angles to itself, so its force is an unknown size `N` times the direction `(sin(α), cos(α))`.

```text
space Plane = euclidean(2)

model Bead in Plane {
  param {
    g: Acceleration = 9.81 m/s^2
    m: Mass         = 2 kg
    α: Angle        = 30 deg   in [5 deg, 85 deg]
  }
  state {
    pos: Point            = origin
    vel: Vector<Velocity> = 0
  }
  unknown { N: Force }
  derived { travelled: Length = |pos - origin| }
  flow {
    der(pos) = vel
    der(vel) = (0 m/s^2, -g) + N / m * (sin(α), cos(α))
  }
  constraint wire: pos.y == -tan(α) * pos.x policy enforce
}

presentation BeadLab for Bead {
  until t0 + 2 s
  view scene: spatial(Plane, scale: 1 m -> 20 px, y: up) {
    trace(pos every 0.05 s)
    marker(pos)
  }
  panel controls { slider(α, range: [5 deg, 85 deg]) }
  panel readings {
    label(N)
    label(travelled)
  }
  observe {
    push = N at t0 + 1 s
    after_two = travelled at t0 + 2 s
  }
}
```

```cases
run slide of Bead with BeadLab {
  until t0 + 2 s
  expect {
    push == 16.9914 N within 1e-4 N             // m g cos(30°)
    after_two == 9.81 m within 1e-6 m           // g sin(30°) t² / 2 along the wire
  }
}
```

- Along the wire the bead accelerates at `g sin α`, half of `g` at 30 degrees: in 2 s it travels 9.81 m instead of the 19.62 m of a free fall.
- The wire's push `N = m g cos α` is constant here; on a curved wire it would change along the way.
- The push can also be one vector unknown, `unknown { N: Vector<Force> }` with `der(vel) = (0 m/s^2, -g) + N / m`. A vector in a plane is two unknowns and the constraint gives one equation, so a second equation says the wire is smooth, pushing at right angles to itself: `equation smooth: dot(N, (cos(α), -sin(α))) == 0 N`. Friction along the wire would be a different second equation.

## Two rods

**Goal.** A double pendulum: a second rod hangs from the first bob. No single angle describes it, and its motion is chaotic, yet both rods keep their lengths and the energy stays as it started.
**How it is built.** Two bobs, two unknown tensions, two enforced constraints. The upper rod pulls the upper bob towards the pivot; the lower rod pulls the two bobs towards each other, so its tension appears in both flows, with opposite signs.

```text
space Plane = euclidean(2)

model DoublePendulum in Plane {
  param {
    g:  Acceleration = 9.81 m/s^2
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
    u1: Vector<1> = (p1 - origin) / |p1 - origin|     // along the upper rod, outwards
    u2: Vector<1> = (p2 - p1) / |p2 - p1|             // along the lower rod, outwards
    energy: Energy = 0.5 * m1 * |v1|^2 + 0.5 * m2 * |v2|^2 + m1 * g * p1.y + m2 * g * p2.y
  }
  flow {
    der(p1) = v1
    der(v1) = (0 m/s^2, -g) - T1 / m1 * u1 + T2 / m1 * u2
    der(p2) = v2
    der(v2) = (0 m/s^2, -g) - T2 / m2 * u2
  }
  constraint upper: |p1 - origin| == L1 policy enforce
  constraint lower: |p2 - p1| == L2 policy enforce
}

presentation DoubleLab for DoublePendulum {
  until t0 + 10 s
  view scene: spatial(Plane, scale: 1 m -> 120 px, y: up) {
    trace(p2 every 0.02 s, color: gray)
    segment(origin, p1)
    segment(p1, p2)
    marker(p1)
    marker(p2)
  }
  view energy_plot: plot(x: [0 s, 10 s], y: [-4 J, -3 J]) {
    series_plot(energy every 0.05 s)
  }
  observe {
    initial = energy at t0
    later   = energy at t0 + 5 s
  }
}
```

```cases
run chaos of DoublePendulum with DoubleLab {
  until t0 + 5 s
  expect {
    initial == -3.4335 J within 1e-9 J
    later == -3.4335 J within 0.005 J
  }
}
```

- `Vector<1>` is a vector without a dimension: a direction.
- Each rod's tension is found together with the other's, from both constraints at once.
- Energy is conserved by the model; what drifts is the solver's error, small at the default tolerance and smaller with a tighter one (`config { rtol = 1e-9 }`).

## A point held on a curve

**Goal.** A learner drags a point around a circle and reads its angle, as in a unit-circle lesson: wherever the pointer goes, the point stays on the circle.
**How it is built.** A model without flows: the point is a parameter, and the enforced constraint holds it on the circle. Nothing moves by itself, so no force is needed: the runtime only moves the dragged point onto the circle.

```text
space Plane = euclidean(2)

model UnitCircle in Plane {
  param {
    r: Length = 1 m
    P: Point  = origin + (0.6 m, 0.8 m)
  }
  derived { angle: Angle = atan2(P.y, P.x)   unit deg }
  constraint circle: |P - origin| == r policy enforce
}

presentation CircleLab for UnitCircle {
  view scene: spatial(Plane, scale: 1 m -> 150 px, y: up) {
    axes
    circle(origin, r)
    segment(origin, P)
    marker(P) as point { on drag as p { propose P = p } }
  }
  panel readings { label(angle) }
  observe { first = angle at t0 }
}
```

```cases
run held of UnitCircle with CircleLab {
  expect { first == 53.1301 deg within 1e-4 deg }
}
```

- A parameter moves only when a change sets it: the dragged point moves, the radius does not. A starting value off the circle is an error, since the runtime would not know which parameter to move.

## Contact: constraints on one side

**Goal.** A ball dropped onto a floor, which stops it but never pulls it down; and a particle sliding off the top of a smooth dome, which leaves the dome where the dome would have to pull to keep it.
**How it is built.** Each is an inequality, `y >= 0 m` and `|pos - origin| >= R`, with `policy enforce` and `with N`: `N` is the unknown force that holds it, read by the flows as before. The runtime keeps the constraint as an equation while `N` pushes, and lets go when `N` would have to pull. The event `leaves` marks that instant for the observation.

```text
space Plane = euclidean(2)

model Drop {
  param {
    g: Acceleration = 9.81 m/s^2
    m: Mass         = 2 kg
  }
  state {
    y: Length   = 1 m
    v: Velocity = 0 m/s
  }
  unknown { N: Force }
  flow {
    der(y) = v
    der(v) = -g + N / m
  }
  constraint floor: y >= 0 m policy enforce with N
}

model Dome in Plane {
  param {
    g:  Acceleration = 9.81 m/s^2
    m:  Mass         = 1 kg
    R:  Length       = 1 m
    θ0: Angle        = 5 deg
  }
  state {
    pos: Point            = origin + (R * sin(θ0), R * cos(θ0))
    vel: Vector<Velocity> = 0
  }
  unknown { N: Force = 1 N }
  flow {
    der(pos) = vel
    der(vel) = (0 m/s^2, -g) + N / m * (pos - origin) / |pos - origin|
  }
  constraint surface: |pos - origin| >= R policy enforce with N
  event leaves on falling(N)
}

presentation DropLab for Drop {
  observe {
    resting = y at t0 + 1 s
    push    = N at t0 + 1 s
  }
}

presentation DomeLab for Dome {
  view scene: spatial(Plane, scale: 1 m -> 150 px, y: up) {
    circle(origin, R)
    marker(pos) as particle
    trace(pos every 0.02 s)
  }
  observe { height = pos.y on leaves }
}
```

```cases
run lands of Drop with DropLab {
  until t0 + 1 s
  expect {
    resting == 0 m within 1e-9 m
    push == 19.62 N within 1e-9 N              // m g: at rest the floor carries the weight
  }
}

run slides of Dome with DomeLab {
  until t0 + 2 s
  expect { height[1] == 0.66413 m within 1e-5 m }   // R · 2/3 cos(θ0)
}
```

- `with N` pairs the constraint with the force that holds it. `N` is positive while it acts: a floor's push, a rope's pull. While the constraint is in contact, `N` is solved for, as for an equation; out of contact `N` is 0 and the constraint holds by itself.
- **Coming into contact.** When the gap closes (the ball reaches the floor), the runtime puts the constraint in contact and removes the velocity into it: the impact is inelastic, the ball stops, unless the constraint has a `restitution` (Friction and bounces, below).
- **Leaving contact.** When `N` reaches 0 and would turn negative, the constraint leaves contact: on the dome this happens where gravity can no longer supply the pull the circular path needs, at `cos θ = 2/3 cos θ0`, about 48 degrees from the top. A ball on the floor that an event throws upward leaves at once.
- A `<=` inequality works the same way: `constraint rope: |bob - pivot| <= L policy enforce with T` is a rope that can go slack. The flows read the tension pulling the bob toward the pivot, `- T / m * (bob - pivot) / |bob - pivot|`, so a positive `T` is a pull.
- If `N` is written the wrong way in the flows (a positive `N` pulling the ball into the floor), the run stops at the first contact and says so.

## Friction and bounces

**Goal.** A crate on a floor pushed by a force that grows: it stays put while friction can match the push, then slides; and a ball that bounces on the floor, lower each time, until it rests.
**How it is built.** Friction is a constraint on the sliding speed, `v == 0 m/s`, held by an unknown `F` that cannot exceed `mu * N`: `with F where |F| <= mu * N`. While the push needs less, the crate holds; when it needs more, the crate slips and `F` stays at its bound, against the slip. A bounce is the floor's inequality with `restitution e`: each impact sends the ball back up with `e` times its speed.

```text
model Crate {
  param {
    m:  Mass              = 2 kg
    g:  Acceleration      = 9.81 m/s^2
    mu: Real              = 0.5
    k:  Quantity<M*L/T^3> = 2 N/s
  }
  state {
    x: Length   = 0 m
    v: Velocity = 0 m/s
    y: Length   = 0 m
    w: Velocity = 0 m/s
  }
  unknown {
    N: Force
    F: Force
  }
  derived { P: Force = k * elapsed }
  flow {
    der(x) = v
    der(v) = (P + F) / m
    der(y) = w
    der(w) = -g + N / m
  }
  constraint floor: y >= 0 m policy enforce with N
  constraint grip: v == 0 m/s policy enforce with F where |F| <= mu * N
}

model Bouncing {
  param {
    g: Acceleration = 9.81 m/s^2
    m: Mass         = 1 kg
    e: Real         = 0.5 in [0, 1]
  }
  state {
    y: Length   = 1 m
    v: Velocity = 0 m/s
  }
  unknown { N: Force }
  flow {
    der(y) = v
    der(v) = -g + N / m
  }
  constraint floor: y >= 0 m policy enforce with N restitution e
}

presentation CrateLab for Crate {
  until t0 + 8 s
  view speeds: plot(x: [0 s, 8 s], y: [-10 N, 10 N]) {
    series_plot(P every 0.05 s)
    series_plot(F every 0.05 s)
  }
  observe {
    held_by   = F at t0 + 3 s
    still     = v at t0 + 3 s
    speed     = v at t0 + 6.905 s
    sliding_F = F at t0 + 6.905 s
  }
}

presentation BounceLab for Bouncing {
  until t0 + 3 s
  view floor: plot(x: [0 s, 3 s], y: [0 m, 1.1 m]) { series_plot(y every 0.01 s) }
  panel controls { slider(e, range: [0, 1]) }
  observe {
    apex    = y at t0 + 0.67734 s
    resting = y at t0 + 3 s
    weight  = N at t0 + 3 s
  }
}
```

```cases
run push of Crate with CrateLab {
  until t0 + 8 s
  expect {
    held_by == -6 N within 1e-9 N        // at 3 s the push is 6 N: friction matches it
    still == 0 m/s exactly
    speed == 2 m/s within 1e-6 m/s       // slipping from 4.905 s: v = (t - 4.905 s)² / 2 per s
    sliding_F == -9.81 N within 1e-9 N   // mu m g, against the slip
  }
}

run bounce of Bouncing with BounceLab {
  until t0 + 3 s
  expect {
    apex == 0.25 m within 1e-6 m         // e² h, at 1.5 sqrt(2 h / g)
    resting == 0 m within 1e-9 m
    weight == 9.81 N within 1e-9 N
  }
}
```

- `with F where |F| <= mu * N`: `F` holds `v == 0 m/s` as long as the force needed is within the bound. Here that is `-P`, so the crate holds until the push reaches `mu m g = 9.81 N`, at 4.905 s.
- **Slipping.** From then on `F` is `-mu N`, at its bound and against the slip, and the crate speeds up at `(P - mu m g) / m`. If the slip stops (with no push, a sliding crate slows down and stops), the constraint holds again, and slips again at once if the force needed is still past the bound.
- `F` is positive in whichever direction the flows say: the runtime finds which way opposes the slip, also for a crate that is sliding from the start.
- The bound can read the other unknowns: `mu * N` is the normal force of the floor, solved at the same instant. Off the floor `N` is 0, and so is the friction.
- `restitution e` on an inequality: each impact reverses the speed into the boundary and keeps the share `e` of it. With `e = 0.5` the ball rises to a quarter of its height, and each bounce lasts half as long as the one before; they add up to a finite time, `3 sqrt(2 h / g)`, after which the ball rests (when bounces accumulate, the contact becomes lasting, as `zeno settle` does in chapter 5). `restitution 0` is the inelastic impact of the floor without it.
- One coefficient of friction serves for holding and for sliding; a static coefficient larger than the sliding one is not yet written this way.

## How constraints are enforced

- `constraint N: a == b policy enforce`: an equation of the model's values, kept true at every instant. `within` is not allowed: an enforced constraint holds exactly. `a >= b` and `a <= b` with `with F` are kept on one side, in contact or not (Contact, above).
- **The force that holds it.** The runtime differentiates the constraint along the flows: `|bob - pivot| == L` gives "the bob moves along the circle" (its rate), and then "the bob's acceleration turns it along the circle" (its second rate). The first rate that reads an unknown is solved with the model's other equations, as in chapter 17: for the rod, the second rate reads `tension` through `der(vel)`. So the constraint needs an unknown that the flows read, one component for each constraint, and the flows must say in which direction it acts; a vector unknown needs further equations for its other components (chapter 17, Vectors and points as unknowns).
- **Staying on the constraint.** After every step, at the start and after every event or change, the runtime moves the state back onto the constraint and its rates, by the smallest change: positions first, then velocities. The solver's small errors therefore never add up into a rod that stretches.
- **What moves.** State moves, never parameters, unless a change sets them (a drag, a slider, an event's `set`). When a change sets some of the values a constraint reads, only those move: a dragged lower bob moves, the upper bob stays.
- **What can be differentiated.** Arithmetic, powers by numbers, `sqrt`, `sin`, `cos`, `tan`, `exp`, `log`, `abs`, `atan2`, `dot`, components (`p.x`) and norms (`|v|`). A constraint with `if`, `match`, `min` or `max` cannot be enforced (MK-E36).
- **When a coordinate does it.** If one coordinate obeys the constraint (an angle for a single pendulum), writing the model in it is simpler and cheaper; enforce a constraint when no such coordinate is natural.
- Errors: MK-E36 when the constraint never reaches an unknown ("declare what holds it as an unknown and read it in the flows"), or has a form that cannot be differentiated; MK-E34 when unknowns and constraints do not match, as for equations.

## Exercises

1. Give the rod pendulum a starting speed: `vel = (1 m/s, 0 m/s)`. Does the bob start on its circle with this velocity? What does the runtime do to it? (Hint: the velocity along the rod.)
2. Bend the wire of the bead into a bowl, `pos.y == k * pos.x^2`, starting the bead at `x = 1 m`. In which direction does the wire push now? (Hint: at right angles to the curve, `(-2 k x, 1)` divided by its length.)
3. Replace the upper rod of the double pendulum by a spring (a flow force `-k (|p1| - L1) u1`, no constraint, no `T1`). Which errors does the checker give if you keep `T1`?

Worked solutions to these exercises are in [chapter 11](11-solutions.md#chapter-18).
