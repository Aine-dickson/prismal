# Working Syntax

Candidate A amended as accepted in D-028: A's keyword-led statements, with grouped declaration and flow blocks, interval ranges and the timeline shorthand (owner amendment), C's view trees, nested interactions, explicit sequences and named processes, and B's default space.

- **Status:** **version 1** (2026-10-12, D-081), implemented by the parser (`crates/prismal-syntax`). Stable for outside authors under the rules of section 1.7.
- **Fixed parts:** the shared expression and type sublanguage of the study (`README.md`), with interval notation as the single range form.

## 1. Design

### 1.1 Model

| Form | Kernel element |
|---|---|
| `space Plane = euclidean(2)` | space (MK-4.1) |
| `model Name [in Space] { ... }` | root object type; with `in Space`, `origin`, `Point` and `Vector<D>` refer to that space; `object` for contained types |
| `const { x: T = e ... }` | constants |
| `param { x: T = e [in I \| where cond] [modifiers] ... }` | parameters; `in I` or `where` attaches a `reject` constraint (MK-12.4) |
| `input { x: T ... }` | inputs |
| `state { x: T = e ... }` | continuous state with initial definitions |
| `discrete { x: T = e ... }` | discrete state |
| `derived { x: T = e; f(x: A): B = e ... }` | derived bindings, including function-valued ones |
| `unknown { x: T [= guess] ... }` | unknowns (MK-11.6, D-092, D-097): numbers, quantities, vectors or points that the equations reading them determine together; the value is the solver's starting guess, 0 when omitted |
| `enum Phase { a, b, c }` | enumeration (MK-2.2a, D-049); cases by name (`a`) or qualified (`Phase.a`); in a model, or outside models, shared (D-099) |
| `fn f(x: A): B = e` | declared function (MK-10.3a, D-048): reads only its parameters, constants and other functions; in a model, or outside models, shared (D-099) |
| `flow { der(x) = e; der(y) += e ... }` | defining flows and contributions (MK-14.7) |
| `process P { flow ...; event ... }` | named process (MK-14.1), optional |
| `event E on <trigger> [if cond] [{ ops }] [zeno stop \| zeno settle { ops }]` | event (MK-15) |
| `algorithm A on <trigger> [if cond] { statements }`; statements `set`, `emit`, `create`, `destroy`, `if c { } else { }`, `while c { }`, `for i in [a, b] { }`, `step` | algorithm: an event whose statements run in order, one `step` per occurrence (MK-14.17, D-101) |
| `equation N: lhs == rhs [checked within tol]` | equation (MK-11); without `checked`, an equation that reads an unknown is solved (MK-11.7), and one that reads `der(x)` of a state without a flow gives that state its rate (MK-11.9) |
| `constraint [N:] cond [within tol] [policy p] [with F]` | constraint (MK-12); `p` is `reject`, `report` (the default), `stop` or `enforce`: an enforced constraint is an equation `a == b`, without `within`, kept true by the runtime (MK-12.6, D-093), or an inequality `a >= b`, `a <= b` held by the unknown `F` it names with `with`, in contact or not (MK-12.11, D-098) |
| `object Name { ... }` | object type (MK-7.13, D-055), declared in a model, with the body of a model; or outside models, shared (MK-7.1a, D-099) |
| `use M`, `use M.{A, B}` | at the top of a file: the shared definitions of module `M` (object types, enumerations and functions written outside models), or those named (MK-7.1a, D-099); `use` is a word only there |
| `parts { a: T [{ x = e ... }]  c: T[n] [{ x = e ... }] }` | a contained object, or a collection of `n` members; overrides and input connections, with `index` the member's number (MK-8.3a) |
| `flow { for b in c { der(b.x) = e ... } }` | flows written by the container for each member (MK-8.8) |
| `parts { c: T[max m]  d: T[n, max m] { ... } }` | a collection whose membership changes (MK-8.2a, D-057): none or `n` members at the start, at most `m` made in a run |
| `for b in c { event E on ... { ops } }` | an event of the model repeated for each member (MK-15.1b) |
| `grid g(x: [a, b], cells: n)`, `grid g(x: [a, b], y: [c, d], cells: (n, m))` | a grid of cells on a line or in the model's plane (MK-9.5, D-091) |
| `state { T(x in g): R = e }`, `derived { q(x in g): R = e }`, `der(T) = e`, `der(T(x)) = e`, `set T(p) = e` | a field over a grid, its flow and a pointwise set; `laplacian(T)`, `gradient(T)` in them, `T(x)`, `integral(T)`, `mean(T)`, `min(T)`, `max(T)` anywhere (MK-9.4 to MK-9.8, D-091) |
| `boundary T { left = e; right = e; bottom = e; top = e }`, `boundary T periodic` | a field's boundary conditions; edges not named are insulated (MK-9.6) |
| `relation R(a in c, b in d) { ... }` | relation type (MK-8.5a, D-058); its endpoints are members of `c` and `d`; a part of type `R` is a relation set |

Every block keyword also has a one-line form for a single declaration (`param g: Acceleration = 9.81 m/s^2`, `flow der(x) = v`). Both parse to the same IR; the formatter prints blocks.

**Intervals** are the one range notation: `[a, b]`, `(a, b)`, `[a, b)`, `(a, b]`, with `inf` for an unbounded end. `x in [a, b)` is the constraint `a <= x < b`; an unbounded end gives a one-sided comparison (`in [0, inf)` is `x >= 0`, the same as `where x >= 0`). After a declaration's value, `in` starts the declaration's range; a membership test used as a value is written in parentheses: `ok: Boolean = (x in [0, 1))`. The formatter prints a parameter's `reject` constraint as an interval when it bounds only that parameter from both sides, as `where` otherwise.

Modifiers after a declaration: `intervenable`, `private`, `symbol "v"`, `unit deg`, and on discrete state `combine sum` (also `product`, `min`, `max`, `any`, `all`; D-073), the combination of `contribute` operations.

Expressions add `match e { a => x, b => y }` (one arm per case, arms separated by commas or lines) to the shared sublanguage, and members (D-055): `a.x`, `c[k].x` with a constant `k`, `b.x` for a loop variable; aggregates `sum(e for b in c [if cond])`, likewise `min`, `max`, `any`, `all`, and `count(c)` or `count(b for b in c if cond)`. In views and representation blocks, `for b in c { reps }` repeats each representation per member. `parts` and `index` are contextual words.

Triggers: `rising(g)`, `falling(g)`, `crossing(g)`, `at τ`, `every Δ [from τ0]`, `on E [(p: T)]`, `on start`, `on input(i)`, `on request [(p: T)]`, `random(λ)` (a contextual word, an addition under section 1.7; `random(p: T)` still follows an event named `random`; D-083); `(p: T)` names the payload the occurrence receives (D-050). A payload may be a member, `(b in balls)`, and an event may declare several, `(b in balls, j: Momentum)` (D-059). `emit E(v)` and `emit E(b, j)` supply payloads. A target may go through an endpoint: `set s.a.vel = e`. `input { x: T [= default] }` declares an input with an optional default (D-051). Operations: `set`, `contribute`, `create`, `destroy`, `connect`, `disconnect`, `emit`. `create c { x = e ... }` makes a member of `c` with starting values; `destroy b` ends the member `b`; `set b.x = e` writes a member's binding from its container (D-057). `max` in `[n, max m]` is a contextual word; `[max inf]` declares no limit (D-066). `connect rs(x, y) { k = e }` makes a relation, `disconnect s` ends one; `s.a` is the member at an endpoint (D-058); `relation` is a contextual word. The distributions `uniform`, `normal`, `exponential`, `bernoulli`, `integer`, `poisson`, `binomial` and `choice` are built-in functions written only in an event's condition and operations; a name the author declares wins over them (D-083). `dot(a, b)` is the dot product of two vectors of one space, or the product of two quantities (D-093). `undirected relation R(a in c, b in c)` has no order outside its body; `s.has(o)` and `s.other(o)` read endpoints without one (D-064). A collection of a contained object is named by its path, `left.atoms`, in endpoints, members, loops and aggregates (D-065).

### 1.2 Presentation

| Form | Element |
|---|---|
| `presentation Name for Model { ... }` | presentation |
| `observe { name = expr schedule ... }` | observations; schedules `live`, `every Δ`, `at τ`, `on E [microstep n]`, `over I` |
| `view name: spatial(Space, scale: 1 m -> 40 px, y: up) { reps }`, `view name: plot(x: I, y: I) { reps }` | views containing their representations; a view of a space of three dimensions takes `azimuth: a, elevation: e` instead of `y` (PK-7.2a, D-100) |
| `panel name { reps }` | region without a coordinate system (controls, formulas) |
| `rep(...) [as name] { on drag [part] as p { propose x = e } }` | representation with its declared inverse; in `for b in c { ... }`, `propose b.x = e` moves that member (D-059) |
| `rep(...) [as name] { on click request E(v) }` | representation that requests an event when clicked; `on click request E(b)` for its member (D-059) |
| `segment(P, Q, color: blue, line: dashed)` | author styles: a named color and a line style (D-061) |
| `view v: ... { on click as q request E(q) ... }` | a view that requests an event with the point clicked, where no representation takes the click (D-060) |
| `group(at: P, rotate: θ, scale: k) [as name] { reps }` | representations placed, turned and scaled together; members are written in the group's frame (D-043) |
| `trace(pos every 0.02 s)`, `series_plot(y every 0.01 s)` | a sampled source: `expr every Δ`, only as a representation's argument (PK-6.3a) |
| `heatmap(T, range: [a, b])` | a field in the plane drawn as coloured cells in a spatial view; a field on a line is a `function_graph(T)` (PK-6.3f, D-091) |
| `until t0 + 20 s` (in a presentation) | how long the presentation's run lasts, as in a case (PK-10.1a, D-091) |
| `histogram(face on roll)`, `histogram(h every 1 s)` | a histogram of one run: the values at each occurrence of an event, or sampled, up to the instant shown (PK-6.3e, D-089) |
| `histogram(n at t0 + 7 s over 200 runs, bins: 20)` | an ensemble source: `expr at τ over n runs`, the values in `n` runs with consecutive seeds; also an observation's schedule, `ends = n at τ over n runs` (PK-3.8, D-084) |
| `instrument ruler(resolution: 1 cm, range: [0 m, 2 m], noise: 2 cm)`; `ruler(y) every 0.05 s` | an instrument, and a source read through it in an observation, a sampled or an ensemble source: rounded, limited, with normal noise (PK-3.7, D-087) |
| `plot(x: [0 s, 6 s], y: [0 m, 1.1 m])` | plot axes with dimensions; a plot marker is at a pair in those dimensions, `marker(at: (0, y))` (PK-7.3a) |
| `permit learner { ... }` | permissions |
| `layout row(scene, column(plot, controls))` | page layout: views side by side in a row, one below the other in a column, nested (D-063) |
| `timeline { scene S { beat B { actions } } }` | timeline; run-directing actions apply in written order at the beat's start, then the others start together (D-033); `sequence { }` orders actions that take time |
| `run rate r until E` | shorthand for `run rate r` and `wait until E` in the same beat |
| `reveal fade\|draw [for d] [in view] { reps }`, `hide name [for d]`, `camera view [to P] [zoom z] [for d]` | animations of the presentation (D-042) |
| `animate name opacity\|offset to v [for d]`, `release name`, `bind name [for d]` | animated properties and the handover of a bound representation (D-068) |
| `wait learner [limit d] [fallback { actions }]` | a continue point (D-067) |

### 1.3 Runs

`run Name of Model with Presentation { param { ... } input { x = v; x = v at τ } config { ... } until τ; learner { ... } expect { ... } }`. `input` gives inputs their starting values and later changes (D-051). `config { seed = n }` names the run's seed, a whole number (D-083). A timeline's `request E(v)` supplies a payload (D-050); `request E(balls[2], v)` supplies a member and a value (D-059). A learner step is `at τ: continue`, `at τ: set slider x = v` or `at τ: press E` (an explore beat's button, D-069). `button(reset)`, `button(undo)` and `button(redo)` are runtime-control buttons of labs (D-069); `button(reseed)` starts again with the next seed (D-084); `button(restart)` and the timeline action `restart` start a new run from `t0` with the parameter values of the instant shown (D-085). Expectations may read `mean(obs)` and `deviation(obs)` (D-084).

### 1.4 Canonical printing (C4)

The formatter prints, from the IR: enumerations, then declarations grouped in blocks by role, in the order kind (const, param, input, state, discrete, derived), then functions, flows, processes, events, equations, constraints; parameter ranges as intervals when two-sided on that parameter alone; `run ... until` when a beat holds exactly those two actions; representations inside their views. Everything else prints in its only form. Author notes print as `///` comments before their element; other comments are not in the IR and are not printed (D-036). Implemented by `prismal_syntax::format`; `cargo run -p prismal-syntax --example fmt -- FILE` prints a program in this form.

### 1.5 Lexical rules and reserved words (D-035)

- **Source text** is UTF-8. Identifiers are a letter or `_` followed by letters, digits or `_`; letters include Unicode letters (`θ`, `ω`, `θ0`). Identifiers are case-sensitive.
- **Numbers:** `12`, `0.5`, `1e-9`, `2.5e3`. A number directly followed by `π` or `pi` (`2π`) is a product; any other name directly after a number is an error (write `2 * x`). A minus sign written on a number is part of the literal (`-5`).
- **Units** follow a number and are written with unit symbols, `*`, `/` and integer powers: `9.81 m/s^2`, `4 N/m`, `0.01 /m`, `45 deg`. The unit ends at the first token that cannot continue it. A unit has no spaces inside it, so a space before `/`, `*` or `^` ends it: `0.01 /m` is a unit, while `4 / m` divides by the binding `m`. Unit symbols are recognized only directly after a number; elsewhere `m`, `s` and `g` are ordinary names.
- **Operators** are ASCII: `+ - * / ^ == != < <= > >= = += -> => .. | . , : ;` and the words `and`, `or`, `not`, `if`, `then`, `else`, `otherwise`. `|v|` is the norm.
- **Comments:** `//` to end of line. `///` is a documentation comment; together with an ordinary comment block directly before a declaration it becomes that element's author notes in the IR (D-036).
- **Statements** end at a newline or `;`. Several statements on one line are separated by `;`. A line ending in an operator (other than a closing `|`), `,`, an open bracket or one of the operator words continues on the next line; so does every line inside `( )` or `[ ]`.
- **Reserved words** (D-040), never names: `space model presentation run object const param input state discrete derived fn flow process event equation constraint on if then else and or not otherwise in where true false zeno stop settle set contribute create destroy connect disconnect emit enter checked within policy reject report intervenable private symbol unit rising falling crossing at every from start request enum match`.
- **Representation kinds** are read where a representation is expected; there `equation`, reserved elsewhere, is the representation kind of PK-6.3.
- **Contextual keywords** (D-040), recognized only where such a word is expected and ordinary names elsewhere (`process drag`, `view scene`): `for view panel observe live over microstep show as drag click propose permit timeline scene beat sequence rate until hold seek reset branch intervene wait explore limit keep fallback narrate highlight hide reveal zoom animate camera bind release config expect exactly rel of with learner continue press undo redo combine ease title random runs reseed restart layout row column instrument grid boundary periodic unknown enforce use algorithm while step`.
- **Built-in names** (not reserved, but predefined): `t`, `t0`, `elapsed`, `origin`, `der`, `π` and `pi` (D-039), `inf` (only as an interval bound), the SI units and named dimensions of MK section 3. Named dimensions: `Length`, `Mass`, `Time`, `Current`, `Amount`, `Area`, `Volume`, `Velocity`, `Acceleration`, `Frequency`, `Momentum`, `Force`, `Energy`, `Power`, `Pressure`, `Temperature`, `Charge`, `Voltage`, `Resistance`, `Capacitance`, `Inductance` (D-092), and `Angle` (dimensionless, D-021); base symbols `L M T I Θ N J` inside `Quantity<...>`.


### 1.6 Options and clauses (D-081)

Details are added to a form in one of two ways, decided by where the form is written:

- **Named options inside parentheses**, `name: value`, after the positional arguments of a representation, a view kind or a group: `marker(pos, color: red)`, `plot(x: I, y: I, follow: y)`, `group(at: P, rotate: θ)`. Each kind has a fixed list of options (guide chapter 10, Options and clauses); every representation also takes `when: c`. An option not on the list is SX-E08, with the nearest option suggested; an option given twice is SX-E09.
- **Clauses after a statement's head**, a word and its value with no colon: ranges and modifiers of declarations (`in I`, `where c`, `intervenable`, `private`, `combine k`, `symbol "s"`, `unit u`), `within`, `policy` and `with` of a constraint, `checked within` of an equation, and the clauses of timeline actions (`for d`, `in view`, `to P`, `zoom z`, `ease e`, `until E`, `limit L`, `keep p, ...`). Clauses come in any order, each at most once (SX-E09 otherwise); the formatter prints them in one order. An event keeps the order `on trigger [if c] [{ ops }] [zeno ...]`, since its block sits among its clauses.

The two families do not mix: a statement never takes `name: value`, and parentheses never take a clause word. A colon elsewhere labels rather than configures: a declaration's type (`x: Length`), a named equation or constraint (`equation energy: ...`), a learner step's time (`at 3 s: continue`). Inside parentheses, `every Δ` belongs to an argument (a sampled source), not to the representation.

A line that starts with a binary operator (`+ - * / ^ == != < <= > >= -> => .`, `and`, `or`) where a statement is expected is reported with the rule of section 1.5: the operator belongs at the end of the line before.

### 1.7 Version 1 and stability (D-081)

The forms of sections 1.1 to 1.6 are version 1. From version 1:

- A program that compiles under version 1 compiles under every later 1.x version, with the same meaning.
- Additions are allowed in a 1.x version: new representation kinds, options, clauses, timeline actions, functions, units and contextual keywords. A new contextual keyword cannot break a program, since it is a word only where it is expected.
- A new reserved word, a renamed or removed form, or a changed meaning breaks programs. Such a change needs a register entry that names the version it starts in and how programs are rewritten; where the rewrite is mechanical, the formatter performs it.
- Diagnostic messages may improve in any version; their codes keep their meanings.

---

## RP-01 Projectile, no drag

```text
space Plane = euclidean(2)

model Projectile in Plane {
  param {
    g:     Acceleration  = 9.81 m/s^2
    k:     Quantity<1/L> = 0        where k >= 0              // quadratic drag coefficient
    speed: Velocity      = 20 m/s   where speed > 0 m/s
    angle: Angle         = 45 deg   in (0 deg, 90 deg)
  }
  state {
    pos: Point            = origin
    vel: Vector<Velocity> = speed * (cos(angle), sin(angle))
  }
  discrete {
    flying: Boolean = true
  }

  flow {
    der(pos) = if flying then vel else 0
  }
  process gravity { flow der(vel) += if flying then (0, -g) else 0 }
  process drag    { flow der(vel) += -k * |vel| * vel }

  event apex   on falling(vel.y)
  event landed on falling(pos.y) {
    set flying = false
    set vel = 0
  }
}

presentation ProjectileChecks for Projectile {
  observe {
    t_apex  = elapsed   on apex
    h_apex  = pos.y     on apex
    t_land  = elapsed   on landed
    range   = pos.x     on landed
    y_land  = pos.y     on landed
    at_rest = pos       at t0 + 5 s
    events  = event_log over [t0, t0 + 5 s]
  }
}
```

```cases
run A_45deg of Projectile with ProjectileChecks {
  until t0 + 5 s
  expect {
    t_land == 2.88320807823261 s within 1e-9 s
    range  == 40.7747196738022 m within 1e-8 m
    y_land in [-1e-8 m, 0 m]
    t_apex == 1.44160403911630 s within 1e-9 s
    h_apex == 10.1936799184506 m within 1e-8 m
    at_rest == (pos on landed) exactly
    events == [apex, landed]
  }
}

run B_60deg of Projectile with ProjectileChecks {
  param { angle = 60 deg }
  until t0 + 5 s
  expect {
    t_land == 3.53119430697019 s within 1e-9 s
    range  == 35.3119430697019 m within 1e-8 m
  }
}
```

## RP-02 Projectile with drag

The RP-01 model, unchanged. Because gravity and drag are named processes, each contribution can also be observed on its own.

```text
presentation DragChecks for Projectile {
  observe {
    t_apex  = elapsed on apex
    h_apex  = pos.y   on apex
    t_land  = elapsed on landed
    range   = pos.x   on landed
    v_land  = vel     on landed microstep 0                         // before the reset
    energy  = 0.5 * |vel|^2 + g * (pos - origin).y   every 0.01 s   // per unit mass
    balance = der(vel) - ((0, -g) - k * |vel| * vel) every 0.01 s
  }
}
```

```cases
run A_default of Projectile with DragChecks {
  param { k = 0.01 /m }
  until t0 + 5 s
  expect {
    t_land == 2.67328857282834 s within rel 1e-5
    range  == 31.3229266146783 m within rel 1e-5
    v_land == (9.76677238322685 m/s, -12.3148244403059 m/s) within rel 1e-5
    (vel on landed) == 0 exactly
  }
}

run B_tight of Projectile with DragChecks {
  param  { k = 0.01 /m }
  config { rtol = 1e-10; atol = 1e-12 }
  until t0 + 5 s
  expect { t_land == 2.67328857282834 s within rel 1e-8 }
}
```

## RP-03 Bouncing ball

```text
model BouncingBall {
  param {
    g:  Acceleration = 9.81 m/s^2
    h0: Length       = 1 m      where h0 > 0 m
    e:  Real         = 0.8      in [0, 1)                // restitution
  }
  state {
    y: Length   = h0
    v: Velocity = 0
  }
  discrete {
    resting: Boolean = false
  }

  flow {
    der(y) = v
    der(v) = if resting then 0 else -g
  }

  event bounce on falling(y) {
    set v = -e * v
  } zeno settle {
    set y = 0 m
    set v = 0 m/s
    set resting = true
  }

  event top on falling(v)                                // apex of each flight
}

presentation BounceChecks for BouncingBall {
  observe {
    bounce_times = elapsed on bounce
    top_heights  = y       on top
    zeno         = event_log of bounce where zeno_applied
    final        = (y, v, resting) at t0 + 6 s
    late_events  = event_log over [t0 + 4.1 s, t0 + 6 s]
  }
}
```

```cases
run A_settle of BouncingBall with BounceChecks {
  until t0 + 6 s
  expect {
    bounce_times[1] == 0.451523640985731 s within 1e-8 s
    top_heights[1]  == 0.64 m within 1e-8 m
    final == (0 m, 0 m/s, true) exactly
    late_events == []
  }
}

run C_invalid_e of BouncingBall {
  param  { e = 1.2 }
  expect { initialization fails }
}
```

Case `B-stop` is a model variant (comparison S-4).

## RP-04 Pendulum

```text
space Plane = euclidean(2)

model Pendulum in Plane {
  param {
    g:     Acceleration = 9.81 m/s^2
    L:     Length       = 1 m      where L > 0 m
    m:     Mass         = 1 kg     where m > 0 kg
    pivot: Point        = origin
    θ0:    Angle        = 10 deg
  }
  state {
    θ: Angle         = θ0                 // from the downward vertical
    ω: Quantity<1/T> = 0
  }
  derived {
    bob:    Point  = pivot + L * (sin(θ), -cos(θ))
    energy: Energy = 0.5 * m * (L * ω)^2 + m * g * L * (1 - cos(θ))
  }

  flow {
    der(θ) = ω
    der(ω) = -(g / L) * sin(θ)
  }

  constraint rod: |bob - pivot| == L within 1e-9 m policy report

  event upswing on rising(θ)              // once per period
}

presentation PendulumChecks for Pendulum {
  observe {
    theta_start = θ           at t0
    upswings    = elapsed     on upswing
    energy      = energy      every 0.01 s
    diagnostics = diagnostics over [t0, t_end]
  }
}
```

```cases
run A_dopri5 of Pendulum with PendulumChecks {
  until t0 + 100 s
  expect {
    upswings[1] == 1.50741947047395 s within rel 1e-5
    diagnostics == []
  }
}

run B_rk4 of Pendulum with PendulumChecks {
  config { solver = rk4; h = 0.01 s }
  until t0 + 100 s
}

run C_rk4_no_step of Pendulum {
  config { solver = rk4 }
  expect { configuration rejected }
}
```

## RP-05 Spring-mass

```text
model SpringMass {
  param {
    m:  Mass            = 1 kg      where m > 0 kg
    k:  Quantity<M/T^2> = 4 N/m     where k > 0 N/m
    x0: Length          = 0.1 m
  }
  state {
    x: Length   = x0                  // displacement from equilibrium
    v: Velocity = 0
  }
  derived {
    energy: Energy = 0.5 * m * v^2 + 0.5 * k * x^2
    T:      Time   = 2π * sqrt(m / k)
  }

  flow {
    der(x) = v
    der(v) = -(k / m) * x
  }

  equation period_law:   T == 2π * sqrt(m / k)
  equation conservation: energy == 0.5 * k * x0^2 checked within 2e-5 J

  event pass on rising(x)             // once per period
}

presentation SpringChecks for SpringMass {
  observe {
    x_10      = x                           at t0 + 10 s
    passes    = elapsed                     on pass
    energy    = energy                      every 0.01 s
    residuals = diagnostics of conservation over [t0, t_end]
  }
}
```

```cases
run A_dopri5 of SpringMass with SpringChecks {
  until t0 + 100 s
  expect {
    x_10 == 0.0408082061813392 m within 1e-5 m
    residuals == []
  }
}

run B_rk4 of SpringMass with SpringChecks {
  config { solver = rk4; h = 0.01 s }
  until t0 + 100 s
  expect { x_10 == 0.0408082061813392 m within 1e-7 m }
}
```

## RP-06 Function plot with draggable parameter

```text
model QuadraticDemo {
  param   { a: Real = 1  in [-5, 5] }
  derived { f(x: Real): Real = a * x^2 }
}

presentation QuadraticPlot for QuadraticDemo {
  view plot: plot(x: [-3, 3], y: [-5, 20]) {
    function_graph(f)
    marker(at: (1, f(1))) {
      on drag as p { propose a = p.y / 1^2 }            // x fixed at 1
    }
  }
  panel controls {
    slider(a, range: [-5, 5], step: 0.1)
    formula(f, live: true)
  }
  observe {
    a_now = a    live
    f2    = f(2) live
    log   = intervention_log
  }
}
```

## RP-07 Vector addition

```text
space Plane = euclidean(2)

model VectorDemo in Plane {
  param {
    A: Point          = origin + (1 m, 2 m)
    u: Vector<Length> = (3 m, 0 m)
    w: Vector<Length> = (0 m, 2 m)
  }
  derived {
    sum:    Vector<Length> = u + w
    length: Length         = |sum|
    B:      Point          = A + sum
    mid:    Point          = A + u            // where w starts
  }
}

presentation VectorPlot for VectorDemo {
  view scene: spatial(Plane, scale: 1 m -> 40 px, y: up) {
    axes; grid
    marker(A)
    arrow(u, from: A) {
      on drag head as h { propose u = h - A }
    }
    arrow(w, from: mid)
    arrow(sum, from: A)
    label(length)
  }
  observe { values = (sum, length, B) live }
}
```

## RP-08 Narrated projectile lesson

The RP-01 model with display symbols and one addition (D-027):

```text
  param {
    ...
    speed: Velocity = 20 m/s   where speed > 0 m/s       symbol "v"
    angle: Angle    = 45 deg   in (0 deg, 90 deg)        symbol "θ"  unit deg
  }

  event relaunch on request {
    set pos    = origin
    set vel    = speed * (cos(angle), sin(angle))
    set flying = true
  }
```

```text
presentation ProjectileLesson for Projectile {
  view scene: spatial(Plane, scale: 1 m -> 10 px, y: up) {
    axes
  }
  permit learner { timeline_controls; zoom; pan }
  observe {
    landings = (elapsed, pos.x) on landed
    log      = event_log
  }

  timeline {
    scene launch {
      beat b1 {
        in scene { marker(pos) as ball; arrow(vel, from: pos, scale: 1 m/s -> 2 px) }
        narrate "A ball is launched at 45 degrees." for 4 s
      }
      beat b2 { run rate 1 until landed }
      beat b3 {
        hold
        highlight ball
        narrate "It lands here. Why this distance?" for 3 s
      }
      beat b4 {
        seek t0                                  // applies first (PK-9.2a, D-033)
        show formula("R", speed^2 * sin(2 * angle) / g, live: true)
        narrate "Watch the horizontal speed." for 3 s
      }
      beat b5 {
        in scene { arrow((vel.x, 0), from: pos, scale: 1 m/s -> 2 px) }
        run rate 0.5 until landed
      }
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
run A_keep of Projectile with ProjectileLesson {
  learner {
    at 23.0 s: set slider angle = 60 deg
    at 25.0 s: continue
  }
  expect {
    start of b3 == 6.88320807823261 s within 1e-8 s
    end of b8   == 28.5311943069702 s within 1e-8 s
  }
}
```

---

## 2. Notes

- **Blocks versus local marks.** Grouped blocks make the role sectional rather than local (comparison section 3), but unlike B's sections each block is a brace-delimited keyword block whose role word is never more than a screen away, and it survives copy and paste. The one-line form is still available where a single declaration is quoted or moved.
- **`in` has two uses:** a range on a declaration (`in [0, 1)`) and a target view in a timeline (`in scene { ... }`). They occur in different contexts; the keyword pass may still separate them.
- **Mode conditions remain** (`if flying then ... else 0`); modes stay deferred (D-028).
- **The reference programs are now written in this syntax** (`docs/spec/reference-programs/`), with the corrections of 2026-09-30. Where the two differ, the reference programs are current.

## History

- 2026-09-29 written after D-028 was accepted with amendment.
- 2026-09-30 zero vectors written `0` (D-030); `sequence` dropped where D-033 orders run-directing actions; reference programs converted.
- 2026-09-30 payloads (`on E(p: T)`, `request E(v)`) and inputs (defaults, a run's `input` block) implemented (D-050, D-051).
- 2026-10-01 member payloads and several payloads (`on request(b in balls, j: T)`, `request E(balls[2], v)`) implemented (D-059).
- 2026-10-01 drags on members (`propose b.pos = p`) and clicks (`on click request E(b)`) implemented (D-059).
- 2026-10-01 clicks on an empty point of a view (`on click as q request E(q)`) implemented (D-060).
- 2026-10-01 author styles `color:` and `line:` implemented (D-061).
- 2026-09-30 `enum`, `fn` and `match` implemented (D-048, D-049); `enum` and `match` reserved; `=>` added.
- 2026-09-30 implemented by the text parser. Lexical rules made precise (units without spaces, names after numbers, statement separators, `in` after a declaration's value, `inf`); reserved words split into reserved words and contextual keywords (D-040); `π` kept by name (D-039); RP-07 observation `state` renamed `values`.
- 2026-09-30 sampled sources (`expr every Δ`) and plot axes with dimensions, from the web player.
- 2026-09-30 canonical printing implemented (`prismal-syntax/src/format.rs`); a one-sided interval lowers as the comparison `where` writes (`x >= lo`), so both spellings give one IR.
- 2026-09-30 `reveal`, `hide ... for`, `camera` (D-042); `hide`, `reveal` and `zoom` added to the contextual keywords.
- 2026-09-30 `group` with its members in a block (D-043).
- 2026-09-30 `object` declarations, `parts`, member expressions, aggregates, `for` in flows and representation blocks (D-055).
- 2026-09-30 `relation` declarations, `connect`, `disconnect`, endpoints `s.a` (D-058).
- 2026-09-30 capacities `[max m]` and `[n, max m]`, `create`, `destroy`, member targets `set b.x`, `for` blocks of events (D-057).
- 2026-10-01 `layout`, `row`, `column`: page layout of views (D-063); added to the contextual keywords.
- 2026-10-01 `undirected relation`, `s.has(o)`, `s.other(o)` (D-064); part paths `left.atoms` (D-065).
- 2026-10-01 `[max inf]`: collections without a declared limit (D-066).
- 2026-10-01 `wait learner` (D-067); `animate`, `release`, `bind` defined (D-068).
- 2026-10-01 learner step `press E`; runtime-control buttons; `press`, `undo` and `redo` added to the contextual keywords (D-069).
- 2026-10-12 syntax stability review (D-081): options and clauses (section 1.6), closed option lists per kind, clauses in any order, a message for a line that starts with an operator; marked version 1 (section 1.7).
- 2026-10-13 `unknown { ... }` blocks, solved equations and rates from equations (D-092); electrical named dimensions; the contextual keyword list brought up to date with the parser (`layout`, `row`, `column`, `instrument`, `grid`, `boundary`, `periodic`, `unknown`).
- 2026-10-14 `policy enforce` and the built-in function `dot` (D-093); `enforce` added to the contextual keywords. Additions under section 1.7: `enforce` was not a policy before, and a declared name `dot` wins over the function.
- 2026-10-14 `select(p)`, a control choosing one case of an enumeration parameter (D-094); an addition under section 1.7 (a new representation kind).
- 2026-10-14 `ease e` after `reveal`, `hide ... for`, `camera` and `bind` (D-095), as after `animate`; additions under section 1.7.
- 2026-10-14 `width: k` on drawn kinds that take a line style, and `animate R width to k` (D-095); additions under section 1.7.
- 2026-10-14 `align: left|center|right` for a text put `at` a point (D-095); an addition under section 1.7.
