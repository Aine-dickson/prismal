# 10. Reference

Every form of the working syntax on one page, with the chapter that introduces it. The normative description is `docs/syntax-study/working-syntax.md`; meanings are in `docs/spec/`.

## Top level

| Form | Meaning | Chapter |
|---|---|---|
| `space Plane = euclidean(2)` | a Euclidean space with axes `x`, `y` | 3 |
| `model Name [in Space] { ... }` | a model; `in Space` makes `Point`, `Vector<D>`, `origin` refer to that space | 1, 3 |
| `presentation Name for Model { ... }` | a presentation of a model | 1, 7 |
| `run Name of Model [with Presentation] { ... }` | a test case | 1, 6 |

## Model

| Form | Meaning | Chapter |
|---|---|---|
| `const { x: T = e }` | constants: never changed | 2 |
| `param { x: T = e [in I \| where c] [modifiers] }` | parameters: changed only by cases, controls and lessons, within their range | 1 |
| `state { x: T = e [intervenable] }` | continuous state, with its initial value | 4 |
| `discrete { x: T = e }` | discrete state: changed only by events | 5 |
| `derived { x: T = e; f(x: A): B = e }` | derived values and functions, always current | 1 |
| `unknown { x: T [= guess] }` | unknowns (numbers, quantities, vectors or points): found at every instant from the equations that read them (the value is the solver's starting guess) | 17 |
| `fn f(x: A, y: B): R = e` | a declared function: reads only its parameters, constants and other functions | 2 |
| `enum Phase { a, b, c }` | an enumeration: a type whose values are its cases | 5 |
| `flow { der(x) = e; der(y) += e }` | rate of change: one definition, or summed contributions | 4 |
| `process Name { flow ...; event ... }` | a named group of flows and events | 4 |
| `event E on trigger [if c] [{ ops }] [zeno settle { ops } \| zeno stop]` | an event | 5 |
| `algorithm A on trigger [if c] { statements }` | an algorithm: `set`, `emit`, `create`, `destroy`, `if c { } else { }`, `while c { }`, `for i in [a, b] { }`, `step`; each occurrence runs to the next `step` | 21 |
| `equation N: a == b [checked within tol]` | a stated relation, shown and optionally checked; an equation that reads an unknown is solved for it | 6, 17 |
| `constraint N: c [within tol] [policy reject \| report \| stop]` | a checked condition (`within` for `==` only) | 6 |
| `constraint N: a == b policy enforce` | a condition the runtime keeps true: the state is moved onto it, and the unknown force that holds it is solved for | 18 |
| `constraint N: a >= b policy enforce with F` | a constraint on one side, held by the unknown `F` while in contact: contact with a floor, a slack rope | 18 |
| `constraint N: v == 0 m/s policy enforce with F where \|F\| <= mu * N` | friction: holds while the force needed is within the bound, slips with the force at the bound against the slip, holds again when the slip stops | 18 |
| `constraint N: y >= 0 m policy enforce with N restitution e` | impacts bounce: the gap's rate reversed and scaled by `e`; bounces that accumulate end in contact | 18 |
| `object Name { ... }` | an object type, with the body of a model; declared in a model, or outside models for every model of the file | 9, 19 |
| `use mechanics`, `use mechanics.{Body}` | at the top of a file: the shared definitions of a module, or the ones named | 19 |
| `parts { a: T { x = e }  c: T[n] { x = e } }` | one object, or a collection of `n` members; overrides and input connections, with `index` the member's number | 9 |
| `flow { for b in c { der(b.x) = e } }` | flows the model writes for each member | 9 |
| `grid g(x: [a, b], cells: n)`, `grid g(x: [a, b], y: [c, d], cells: (n, m))` | a grid: a line, or a rectangle of the model's plane, cut into equal cells | 16 |
| `state { T(x in g): R = e }`, `derived { q(x in g): R = e }` | a field over grid `g`: a value of type `R` in each cell, `x` the cell's position (a point in the plane) | 16 |
| `flow der(T) = e`, `flow der(T(x)) = e` | a field's flow, computed in each cell; `T` and other fields of the grid read the cell | 16 |
| `frame F(at: P, x: u)`, `frame F(at: P, x: u, y: v)` | a frame of the model's space: an origin and axes, which may move (a plane takes `x` only) | 23 |
| `boundary T { left = e; right = e; bottom = e; top = e }`, `boundary T periodic` | a field's edges held at values (the others insulated), or wrapped around | 16 |

Each block keyword also has a one-line form: `param g: Acceleration = 9.81 m/s^2`, `flow der(x) = v`.

**Modifiers:** `symbol "θ"` (display symbol), `unit deg` (display unit of an angle), `intervenable` (state the learner may change), `private`, `combine sum|product|min|max|any|all` (discrete state that takes contributions).

**Triggers:** `rising(g)`, `falling(g)`, `crossing(g)` (zero crossings of `g`), `at τ`, `every Δ [from τ0]`, `start`, `request`, `E` (after event `E` occurs or is emitted), `input(i)`, `random(λ)` (at random instants, on average `λ` per unit of time, chapter 15). `request(p: T)` and `E(p: T)` receive a payload named `p` (chapter 5); `request(b in c)` receives a member of the collection `c`, and `request(b in c, p: T)` several payloads (chapter 9).

**Inputs:** `input { x: T [= default] }`, values from the environment; a case supplies them with `input { x = v; x = v at τ }` (chapter 5).

**Members** (chapter 9): `a.x`, `c[k].x`; `sum`, `min`, `max`, `any`, `all` of `(e for b in c [if cond])`; `count(c)`, `count(b for b in c if cond)`; `first(e for b in c [if cond] by k)`, the value `e` of the member with the smallest key `k` (chapter 22). A derived value may have no value (a ray that meets nothing): what reads it has none either, a polyline ends before it, and `e otherwise d` gives `d` instead (chapters 6, 22). In views, `for b in c { reps }` draws one representation per member, named `name[k]`.

**Operations:** `set x = e`, `set b.x = e` (a member's binding), `set T(p) = e` (a field, in each cell, chapter 16), `emit E`, `emit E(v)` (with a payload), `emit E(b, v)` (several), `contribute x += e` (into discrete state declared with `combine`; any number combine, a `set` of the same binding conflicts, chapter 5).

## Types and values

| Form | Meaning | Chapter |
|---|---|---|
| `Real`, `Boolean`, `Angle` | numbers, truth values, angles (dimensionless) | 1, 2 |
| `Length`, `Mass`, `Time`, `Velocity`, `Acceleration`, `Force`, `Energy`, `Power`, `Pressure`, `Frequency`, `Momentum`, `Area`, `Volume`, `Current`, `Amount`, `Temperature`, `Charge`, `Voltage`, `Resistance`, `Capacitance`, `Inductance` | named dimensions | 2, 17 |
| `Quantity<M/T^2>` | any dimension from `L M T I Θ N J` | 2 |
| `Point`, `Vector<Length>`, `Vector<Velocity>` | geometry in the model's space | 3 |
| `9.81 m/s^2`, `45 deg`, `2π`, `0` | literals; `0` has any dimension and is the zero vector | 2, 3 |
| `origin + (1 m, 2 m)` | a point | 3 |
| `v.x`, `\|v\|` | component, norm | 3 |
| `v.in(F).x`, `p.in(F).y` | a component, or a coordinate from the frame's origin, along an axis of frame `F` | 23 |
| `[a, b]`, `(a, b)`, `[a, b)`, `(a, inf)` | intervals | 1 |
| `if c then a else b` | conditional | 4 |
| `match p { a => x, b => y }` | a value per case of an enumeration, every case once | 5 |
| `a`, `Phase.a` | a case of an enumeration | 5 |
| `t`, `t0`, `elapsed` | current time, start, time since start | 4 |
| `noise` | white noise in a flow, `der(x) = a + b * noise`: each occurrence independent, of dimension `T^(-1/2)` | 15 |
| `sin cos tan sqrt exp log abs min max atan2` | functions | 2 |
| `dot(a, b)` | the dot product of two vectors of one space (or the product of two quantities) | 18 |
| `cross(a, b)` | the cross product of two vectors of a space of three dimensions | 20 |
| `floor(x)`, `ceil(x)`, `round(x)`, `mod(a, b)` | whole numbers of a number; the remainder `a - b floor(a / b)`, of `a`'s dimension | 21 |
| `(5, 2, 6)[k]` | the `k`-th of a list of values, from 1: `v = (5, 2, 6)[index]` in a collection's overrides | 21 |
| `row[k]` with `k` computed | the member numbered `k` during the run; a number outside the collection stops the run | 21 |
| `T(x)`, `laplacian(T)`, `gradient(T)`, `integral(T)`, `mean(T)`, `min(T)`, `max(T)` | a field read at a position; its Laplacian and gradient in the cell being computed; over the whole field | 16 |
| `uniform normal exponential bernoulli integer poisson binomial choice` | distributions: a random value, only in an event's condition and operations | 15 |

Operators: `+ - * / ^`, `== != < <= > >=`, `and or not`. Units: `m cm mm km μm nm s ms min h kg g N J kJ W kW Pa Hz A mA μA K mol cd rad deg rev %`, and the electrical `V mV kV Ω kΩ MΩ C mC μC F mF μF nF pF H mH μH` (`ohm`, `kohm`, `uF`, `uA`... in ASCII); exponents are whole or half numbers, `s^0.5`.

## Presentation

| Form | Meaning | Chapter |
|---|---|---|
| `view n: spatial(Space, scale: 1 m -> 40 px, y: up) { reps }` | spatial view | 3 |
| `view n: spatial(Room, scale: 1 m -> 40 px, azimuth: 30 deg, elevation: 20 deg) { reps }` | a view of a space of three dimensions, `z` up; the angles may read parameters and time | 20 |
| `spatial(Room, ..., distance: 14 m)` | the viewer stands 14 m from the origin: a view in perspective | 20 |
| `spatial(Plane, ..., frame: F)` | the view is drawn as seen from frame `F`, which may move | 23 |
| `view n: plot(x: [a, b], y: [c, d]) { reps }` | plot view; ranges give the axes their dimensions | 1, 4 |
| `plot(..., follow: y)`, `follow: (x, y)` | those axes grow to keep the data in view | 7 |
| `plot(..., x_unit: ms, y_unit: cm)` | display units for the axes | 7 |
| `plot(..., window: 10 s)` | the `x` axis shows the latest 10 s once the data passes its range | 7 |
| `panel n { reps }` | region for controls and text | 1 |
| `permit learner { zoom; pan; orbit; timeline_controls }` | what the learner may do with views and time; `orbit` turns a view of three dimensions by dragging | 7, 8, 20 |
| `observe { n = e schedule }` | observations | 1, 6 |
| `timeline { scene s { beat b { actions } } }` | a lesson | 8 |
| `layout row(a, column(b, c))` | where views go on the page | 7 |
| `title "..."` | the presentation's title, above its views | 7 |
| `instrument n(resolution: r, range: [a, b], noise: s)` | an instrument; `n(e) every Δ` reads `e` through it in observations, series plots, traces, tables and histograms | 15 |

**Representations:** `text("... {e} ... {e:.N} ..." [, at: P] [, align: left|center|right])` (an author's text with values in braces, `:.N` for N decimals; `at` places it in a spatial or plot view, starting at `P`, centred on it or ending at it), `title("...")` (the caption of its view or panel), `marker`, `arrow`, `segment`, `polyline`, `polygon`, `circle(P, r)`, `ellipse(P, a, b, rotate: θ)`, `arc(P, r, from: θ1, to: θ2)`, `ray(P, d [, back: dotted])` (a half-line to the edge of the view, `back` its extension behind `P`, chapter 22), `trace(P every Δ)`, `function_graph`, `series_plot(e every Δ)`, `histogram(e at τ over n runs, bins: k)` (how many of `n` runs give each value), `histogram(e on E)`, `histogram(e every Δ)` (the values of one run up to the instant shown, chapter 15), `heatmap(T)` (a field in the plane, coloured cells, chapter 16), `function_graph(T)` (also a field on a line), `axes`, `grid`, `label`, `formula`, `equation(name)`, `table(e every Δ)`, `slider` (with `ends: ("low", "high")`, words at its ends), `number_input`, `toggle`, `select(p)` (one case of an enumeration parameter), `button(E)`, `button(E(v))`, `button(reset|undo|redo|restart|reseed)` (in a lesson, only `restart` and `reseed`, in an explore beat), `group(at: P, rotate: θ, scale: k) { members }` (chapter 7; in three dimensions `axis: n`, the axis it turns about, chapter 20). In a view of three dimensions, `circle`, `ellipse` and `arc` take `normal: n`, the plane they lie in, and `circle`, `ellipse` and `polygon` take `opaque: true`, a body that hides what is behind it (chapter 20). `color: red|orange|yellow|green|teal|blue|purple|pink|gray|ink` or a wavelength of light, `color: 650 nm`, `color: λ` (chapter 22), `line: solid|dashed|dotted` and `width: k` (a multiple of the usual line width) style drawn kinds (chapter 7); `as name` names one; `{ on drag [head] [live] [across n] as p { propose x = e } }` declares its inverse (`live`: the run keeps going while dragging; `across n`: in three dimensions, the drag moves in the plane of normal `n`, chapter 20), and `{ on click request E(v) }` the event a click requests; in `for b in c { ... }`, `propose b.x = e` and `request E(b)` act on that member (chapter 9). A view's `on click as q request E(q)` requests `E` with an empty point clicked (chapter 9). Any representation takes `when: c`: it is shown only while the condition `c` holds (chapter 7). Markers, arrows and graphs take `label: "..."` or `label: none` (chapter 7).

**Observation schedules:** `live`, `every Δ`, `at τ`, `on E [microstep n]`, `over [a, b]`. **Sources:** any expression, `event_log [of E] [where zeno_applied]`, `diagnostics [of element]`, `intervention_log`.

**Timeline actions:** `narrate "..." [for d]`, `run rate r [until E]`, `hold`, `seek τ`, `reset`, `restart` (a new run from `t0` with the parameters as they are, chapter 15), `branch`, `highlight name`, `hide name [for d] [ease e]`, `reveal fade|draw [for d] [in view] [ease e] { reps }`, `camera view [to P] [zoom z] [azimuth a] [elevation e] [for d] [ease e]` (a view of three dimensions turns, chapter 20), `in view { reps }`, `show rep`, `intervene { set p = e }`, `request E`, `request E(v)`, `request E(c[k], v)`, `wait d`, `wait learner [limit L] [fallback { actions }]`, `animate name opacity|offset|scale to v [for d] [ease e]`, `animate name color|line to word [for d] [ease e]`, `animate name width to k [for d] [ease e]`, `animate name morph to other [for d] [ease e]`, `release name`, `bind name [for d] [ease e]`, `sequence { ... }`, `explore [limit L] [keep p, ...] { controls } [fallback { actions }]` (chapter 8).

## Options and clauses

Two forms add details to a form, one for each place they appear:

- **Inside parentheses**, a representation or a view takes **named options**, `name: value`, after its positional arguments: `marker(pos, color: red, label: none)`, `plot(x: [0 s, 5 s], y: [0 m, 2 m], follow: y)`. Each kind takes the options below and no others; a misspelled or repeated option is an error (SX-E08, SX-E09).
- **After a statement's head**, a declaration, event, constraint or timeline action takes **clauses**, a word and its value without a colon: `param g: Acceleration = 9.81 m/s^2 in [1 m/s^2, 20 m/s^2] symbol "g"`, `camera scene to P zoom 2 for 1 s`. Clauses come in any order, each at most once (an event keeps the order shown, since its block sits among its clauses); the formatter prints them in the order of this page.

| Kind | Options (every representation also takes `when: c`) |
|---|---|
| `marker` | `at`, `color`, `label` |
| `arrow` | `from`, `scale`, `color`, `line`, `width`, `label` |
| `function_graph` | `color`, `line`, `width`, `label` |
| `segment`, `polyline`, `trace`, `series_plot` | `color`, `line`, `width` |
| `polygon` | `color`, `line`, `width`, `opaque` |
| `circle` | `normal`, `color`, `line`, `width`, `opaque` |
| `histogram` | `bins`, `color`, `line`, `width` |
| `heatmap` | `range` |
| `ellipse` | `rotate`, `normal`, `color`, `line`, `width`, `opaque` |
| `arc` | `from`, `to`, `normal`, `color`, `line`, `width` |
| `ray` | `back`, `color`, `line`, `width` |
| `text` | `at`, `align` |
| `formula`, `equation` | `live` |
| `slider` | `range`, `step`, `ends` |
| `number_input` | `range`, `step` |
| `button` | `label` |
| `group` | `at`, `rotate`, `axis`, `scale` |
| `instrument` (not a representation) | `resolution`, `range`, `noise` |
| `axes`, `grid`, `label`, `title`, `table`, `toggle`, `select` | none |
| view `spatial(Space, ...)` | `scale`, `y`, `frame`; in three dimensions `scale`, `azimuth`, `elevation`, `distance`, `frame` |
| view `plot(...)` | `x`, `y`, `follow`, `x_unit`, `y_unit`, `window` |

| Statement | Clauses |
|---|---|
| declaration in `param`, `state`, ... | `in I` or `where c` (one range), `intervenable`, `private`, `combine k`, `symbol "s"`, `unit u` |
| `equation` | `checked within tol` |
| `constraint` | `within tol`, `policy p`, `with F`, `where \|F\| <= B`, `restitution e` |
| `event` | `on trigger` first, then `if c`, the block, `zeno ...` |
| `narrate "..."` | `for d` |
| `hide`, `bind` | `for d`, `ease e` |
| `run rate r` | `until E` |
| `reveal fade\|draw` | `for d`, `in view`, `ease e` |
| `camera view` | `to P`, `zoom z`, `azimuth a`, `elevation e`, `for d`, `ease e` |
| `animate name property to v` | `for d`, `ease e` |
| `explore` | `limit L`, `keep p, ...` |
| `wait learner` | `limit L` |

## Runs

```prismal
run Name of Model with Presentation {
  param  { p = value; ... }
  config { solver = rk4; h = 0.01 s }        // or rtol = ...; atol = ...; seed = 7
  until t0 + 10 s
  learner { at 15 s: set slider h = 20 m; at 16 s: continue }
  expect { ... }
}
```

**Expectations** (chapter 6): `a == b within tol`, `a == b within rel r`, `a == b exactly`, `a in [lo, hi]`, `obs[k]`, `mean(obs)`, `deviation(obs)` (of an observation's values, chapter 15), `(e on E [microstep 0])`, `log == [E1, E2]`, `start of b`, `end of b`, `initialization fails`, `configuration rejected`.

## First-party modules

Modules that come with Prismal, written in Prismal as any author's library is (`lib/`). `use mechanics` or `use optics` at the top of a file makes their definitions visible; `use optics.{refract, thin_lens}` imports only those named. Chapters lists where a definition is used in the guide; `-` marks a helper the other definitions use.

### `mechanics`

| Definition | Gives | Chapters |
|---|---|---|
| `object Body` | A point mass moved by the force on it, `der(vel) = force / m`. Connect `force` to the sum of the forces acting on it. | 19 |
| `kinetic_energy(m: Mass, v: Velocity): Energy` | The kinetic energy of a mass `m` moving at speed `v`: `m v² / 2`. | - |
| `spring_force(k: Quantity<M/T^2>, x: Length): Force` | The force of a spring of stiffness `k` stretched by `x` beyond its rest length: `-k x`, toward its rest length. | - |
| `pendulum_period(L: Length, g: Acceleration): Time` | The period of small swings of a pendulum of length `L` where gravity is `g`. | 19 |
| `coriolis(m: Mass, w: Vector<1/T>, v: Vector<Velocity>): Vector<Force>` | The Coriolis force on a mass `m` moving at velocity `v`, measured in a frame that turns at angular velocity `w` (along the axis it turns about, as long as its rate): `-2 m w × v`. It turns what moves in the frame without changing its speed. Spaces of three dimensions. | 23 |
| `centrifugal(m: Mass, w: Vector<1/T>, r: Vector<Length>): Vector<Force>` | The centrifugal force on a mass `m` at displacement `r` from a point on the axis of a frame turning at angular velocity `w`: `-m w × (w × r)`, away from the axis, `m \|w\|² d` at a distance `d` from it. Spaces of three dimensions. | 23 |

### `optics`

| Definition | Gives | Chapters |
|---|---|---|
| `direction(d: Vector<1>): Vector<1>` | The direction `d` scaled to length 1. | 22 |
| `facing(n: Vector<1>, d: Vector<1>): Vector<1>` | The normal `n` of a surface turned against a ray along `d`, scaled to length 1: it points back to the side the ray comes from. | - |
| `reflect(d: Vector<1>, n: Vector<1>): Vector<1>` | A ray along `d` reflected by a surface of normal `n`: the angle of reflection equals the angle of incidence. | 22 |
| `incidence_cos(d: Vector<1>, n: Vector<1>): Real` | The cosine of the angle of incidence of a ray along `d` on a surface of normal `n`, between the ray and the normal: 1 head on, 0 grazing. | 22 |
| `incidence(d: Vector<1>, n: Vector<1>): Angle` | The angle of incidence of a ray along `d` on a surface of normal `n`. | 22 |
| `totally_reflected(d: Vector<1>, n: Vector<1>, n1: Real, n2: Real): Boolean` | Whether a ray along `d` going from a medium of index `n1` into one of index `n2` through a surface of normal `n` is totally reflected: `n1 sin θ1 > n2`. | 22 |
| `critical_angle(n1: Real, n2: Real): Angle` | The critical angle from index `n1` into a smaller index `n2`: `sin θc = n2 / n1`. | 22 |
| `refract(d: Vector<1>, n: Vector<1>, n1: Real, n2: Real): Vector<1>` | Snell's law, `n1 sin θ1 = n2 sin θ2`: the direction of a ray along `d` after it passes from index `n1` into index `n2` through a surface of normal `n` (pointing either way). Beyond the critical angle the ray is reflected instead. | 22 |
| `hit_plane(p: Point, d: Vector<1>, a: Point, n: Vector<1>): Point` | Where a ray from `p` along `d` meets the plane (in a plane, the line) through `a` with normal `n`. | 22, 14 |
| `plane_distance(p: Point, d: Vector<1>, a: Point, n: Vector<1>): Length` | How far along a ray from `p` along `d` it meets the plane (in a plane, the line) through `a` with normal `n`: negative behind `p`, infinite when the ray runs along the plane. With `first(... by plane_distance(...))` a ray finds the nearest of several surfaces. | 22 |
| `meets_segment(p: Point, d: Vector<1>, a: Point, n: Vector<1>, half: Length): Boolean` | Whether a ray from `p` along `d` meets, ahead of `p`, the flat mirror (in a plane, a segment) through `a` with normal `n` reaching `half` from `a` each way. A ray along the mirror never meets it. | 22 |
| `closest_along(p: Point, d: Vector<1>, c: Point): Length` | How far along a ray from `p` along `d` its line passes closest to `c`. | - |
| `half_chord_sq(p: Point, d: Vector<1>, c: Point, r: Length): Area` | The square of half the chord a ray's line cuts from the sphere of centre `c` and radius `r`; negative when it misses. | - |
| `sphere_distance(p: Point, d: Vector<1>, c: Point, r: Length): Length` | How far along a ray from `p` along `d` it first meets the sphere (in a plane, the circle) of centre `c` and radius `r`, ahead of `p`: the near side from outside, the far side from inside or from a point on the sphere. | - |
| `hit_sphere(p: Point, d: Vector<1>, c: Point, r: Length): Point` | Where a ray from `p` along `d` first meets the sphere of centre `c` and radius `r`. A ray that misses it has no such point. | 22 |
| `meets_sphere(p: Point, d: Vector<1>, c: Point, r: Length): Boolean` | Whether a ray from `p` along `d` meets the sphere of centre `c` and radius `r` ahead of `p`. | 22 |
| `sphere_normal(q: Point, c: Point): Vector<1>` | The outward normal of the sphere of centre `c` at a point `q` on it. | 22 |
| `thin_lens(d: Vector<1>, q: Point, c: Point, a: Vector<1>, f: Length): Vector<1>` | An ideal thin lens of focal length `f` (negative for a diverging lens), centre `c` and axis `a`: the direction of a ray along `d` after it crosses the lens at `q`, a point of the lens's plane. Rays parallel to the axis then pass through the focus; a ray through the centre goes straight on. | 22, 14 |
| `thin_mirror(d: Vector<1>, q: Point, c: Point, a: Vector<1>, f: Length): Vector<1>` | A curved mirror in the paraxial approximation: the direction of a ray along `d` after it meets, at `q`, the mirror of centre `c`, axis `a` (pointing back toward the light) and focal length `f` (positive for a concave mirror, negative for a convex one). Rays parallel to the axis come back through the focus, `f` in front of the mirror. | 22 |
| `mirror_focus(R: Length): Length` | The focal length of a spherical mirror of radius `R`: half its radius. | 22 |
| `image_distance(u: Length, f: Length): Length` | The distance of the image of an object at distance `u` in front of a thin lens or mirror of focal length `f`: `1/u + 1/v = 1/f`. Positive for a real image (behind a lens, in front of a mirror), negative for a virtual one. | 22 |
| `magnification(u: Length, f: Length): Real` | The magnification of that image: `-v / u`, negative when it is upside down. | 22 |
| `lensmaker(n: Real, r1: Length, r2: Length): Length` | The focal length of a thin lens of index `n` in air whose faces have radii `r1` and `r2` (positive when the centre of the face is behind the lens): the lensmaker's equation. | 22 |
| `cauchy(A: Real, B: Area, λ: Length): Real` | The refractive index of a glass at wavelength `λ` by Cauchy's formula, `A + B / λ²`: blue light is bent more than red. | 22 |
| `wavenumber(λ: Length): Quantity<1/L>` | The wavenumber of a wave of wavelength `λ`: `2 π / λ`, the phase it gains per length travelled. | 24 |
| `two_slit(d: Length, λ: Length, θ: Angle): Real` | The brightness, as a share of the brightest, that two narrow slits `d` apart send at angle `θ` with light of wavelength `λ`: `cos²(π d sin θ / λ)`. The two waves arrive in step where their paths differ by a whole number of wavelengths. | 24 |
| `single_slit(a: Length, λ: Length, θ: Angle): Real` | The brightness, as a share of the middle's, that one slit of width `a` sends at angle `θ`: `(sin β / β)²` with `β = π a sin θ / λ`; dark where `a sin θ` is a whole number of wavelengths. | 24 |
| `grating(N: Real, d: Length, λ: Length, θ: Angle): Real` | The brightness, as a share of the brightest, of a grating of `N` narrow slits `d` apart: `(sin(N φ) / (N sin φ))²` with `φ = π d sin θ / λ`. Its bright lines are where two slits do, `d sin θ = m λ`, and grow sharper with more slits. | 24 |
| `bright_angle(m: Real, d: Length, λ: Length): Angle` | The angle of the bright line of order `m` of slits `d` apart: `d sin θ = m λ`. | 24 |
| `malus(θ: Angle): Real` | Malus's law: the share of polarized light a polarizer passes when turned `θ` from the light's polarization, `cos² θ`. | 24 |
| `brewster(n1: Real, n2: Real): Angle` | Brewster's angle from index `n1` into index `n2`, `tan θ = n2 / n1`: light reflected at it is polarized along the surface. | 24 |
| `angle_between(u: Vector<1>, v: Vector<1>): Angle` | The angle between two directions, from 0 to 180 degrees: how far a ray has been turned. | 22 |

## Glossary

Every word of the language, what it does, and the chapter that teaches it.

| Word | What it does | Chapter |
|---|---|---|
| `and`, `or`, `not` | Boolean operators | 1 |
| `across` | `on drag across n as p`: in a view of three dimensions, the drag moves in the plane of normal `n` | 20 |
| `as` | names a representation: `marker(pos) as ball` | 7 |
| `at` | an event at an instant (`at τ`); an observation at an instant; a learner step or input change at a time | 5, 6, 8 |
| `beat` | a step of a scene in a timeline | 8 |
| `animate` | a timeline animation of a drawing's opacity or offset | 8 |
| `bind`, `release` | timeline actions: a drawing follows the model again, or stays as it is | 8 |
| `branch` | a timeline action: the lesson continues on a new run version | 8 |
| `azimuth`, `elevation` | where the viewer of a space of three dimensions stands: options of a view, clauses of `camera` | 20 |
| `by` | `first(e for b in c by k)`: the key that orders the members | 22 |
| `camera` | a timeline animation of a view's centre and zoom, and in three dimensions its turn | 8, 20 |
| `checked` | `equation N: a == b checked within tol` is checked while running | 6 |
| `config` | a run's solver settings and its seed (`seed = 7`) | 6, 15 |
| `const` | constants: never change | 2 |
| `constraint` | a condition checked while running, with a policy | 6 |
| `continue` | a learner step that ends an explore beat or a continue point | 8 |
| `press` | a learner step that presses an explore beat's button: `at 3 s: press drop`, or its runtime control: `press restart`, `press reseed` | 8, 15 |
| `crossing` | an event when a value crosses zero either way | 5 |
| `algorithm` | `algorithm name on trigger { statements }`: an event whose work is statements run in order, one `step` per occurrence | 21 |
| `derived` | values computed from others, always current | 1 |
| `discrete` | state changed only by events | 5 |
| `click` | the gesture that requests an event: `on click request E(b)`; in a view, `on click as q request E(q)` | 9 |
| `drag` | the gesture of an inverse: `on drag as p`; of a member, `propose b.pos = p` | 7, 9 |
| `else`, `if`, `then` | conditional values; `if` also filters an aggregate and enables an event | 1, 5, 9 |
| `emit` | an operation that makes another event happen | 5 |
| `enum`, `match` | enumerations and a choice by case | 5 |
| `combine` | how contributions combine into discrete state: `combine sum`, `product`, `min`, `max`, `any`, `all` | 5 |
| `ease` | the easing of an animation, a reveal, a hide, a camera move or a bind: `animate R scale to 2 for 1 s ease linear` (`linear`, `smooth`, `in`, `out`) | 8 |
| `title` | a presentation's title, `title "..."`; a view's or panel's, `title("...")` | 7 |
| `connect`, `disconnect` | operations that make a relation between members and remove one | 9 |
| `contribute` | an operation adding to discrete state by its combination: `contribute total += 1` | 5 |
| `create`, `destroy` | operations that make a member of a collection and remove one | 9 |
| `equation` | a named relation, shown and optionally checked | 6 |
| `event` | something that happens at an instant, with operations | 5 |
| `every` | an event every Δ; a sampled source (`pos every 0.1 s`); an observation schedule | 4, 5, 6 |
| `exactly`, `rel`, `within` | tolerances of an expectation, or of a check | 6 |
| `expect` | the checks of a run | 1, 6 |
| `explore`, `limit`, `keep`, `fallback` | the learner's turn in a lesson, its time limit, what it keeps, and what video plays instead | 8 |
| `falling`, `rising` | an event when a value crosses zero downwards or upwards | 5 |
| `flow` | how continuous state changes: `der(x) = e` | 4 |
| `fn` | a declared function | 2 |
| `for` | a loop over a collection's members, in flows, events, aggregates and views; also `narrate ... for d`, `presentation P for M` | 7, 8, 9 |
| `from` | the start of `every Δ from τ0` | 5 |
| `hide`, `reveal`, `highlight` | timeline actions on representations | 8 |
| `hold`, `run`, `rate`, `until`, `seek`, `reset`, `wait` | timeline control of the simulation: pause, play at a rate until an event, jump, restart, wait | 8 |
| `in` | the space of a model (`model M in Plane`), a range (`x in [a, b]`), a view (`in scene { ... }`), a collection (`for b in row`), a member payload (`request(b in row)`), a frame (`v.in(F).x`) | 1, 3, 8, 9, 23 |
| `index` | the number of a member in a collection's overrides | 9 |
| `intervene` | a timeline action that changes a parameter at the instant shown: `intervene { set p = e }` | 8 |
| `input` | values supplied from outside, with an optional default; an object's connections | 5, 9 |
| `intervenable` | state the learner may change | 3 |
| `layout`, `row`, `column` | a presentation's page layout: views side by side or one below the other | 7 |
| `learner` | a run's scripted learner inputs; `wait learner`, a continue point | 8 |
| `live`, `over`, `microstep` | observation schedules: current value, over an interval, at a microstep; `on drag live`: a drag that steers the running model | 6, 7 |
| `runs` | an ensemble: `n at t0 + 7 s over 200 runs`, one value from each of 200 runs with consecutive seeds, in an observation or a representation's argument | 15 |
| `max` | the capacity of a collection whose members come and go (`Drop[max 40]`, or `Drop[max inf]` for no limit); also the aggregate `max(...)` | 9 |
| `model` | what exists and how it behaves | 1 |
| `narrate`, `scene`, `sequence`, `timeline` | a lesson's narration, scenes, ordered actions and timeline | 8 |
| `object`, `parts` | an object type, and the objects and collections a model holds | 9 |
| `observe` | named values recorded for tests and display | 6 |
| `of`, `with` | `run r of Model with Presentation`; `event_log of E` | 6 |
| `on` | an event's trigger; an observation `on E` | 5, 6 |
| `otherwise` | a default when a value is not available | 6 |
| `panel`, `view` | regions of a presentation | 7 |
| `param` | parameters: changed by cases, controls and lessons | 1 |
| `permit` | what the learner may do | 7 |
| `policy`, `reject`, `report`, `stop` | what a failed constraint does; `zeno stop` | 5, 6 |
| `enforce` | `policy enforce`: the runtime keeps the constraint true instead of checking it | 18 |
| `restitution` | `restitution e` on an enforced inequality: the share of the speed into the boundary an impact gives back | 18 |
| `presentation` | how a model is shown | 1 |
| `private` | a binding hidden from outside | 2 |
| `process` | a named group of flows and events | 4 |
| `propose` | the binding change an inverse proposes | 7 |
| `relation` | a relation type, whose endpoints are members of collections | 9 |
| `undirected` | before `relation`: a relation whose two endpoints have no order, read with `s.has(o)` and `s.other(o)` | 9 |
| `random` | an event at random instants at a rate: `on random(lambda * n)` | 15 |
| `request` | an event requested from outside (a button, a lesson) | 5 |
| `undo`, `redo` | in `button(...)`, a lab's runtime controls, with `reset` | 7, 8 |
| `reseed` | in `button(...)`, a lab's runtime control: start again with the next seed, keeping the learner's settings | 15 |
| `unknown` | in a model, a block of unknowns that the equations reading them determine: `unknown { I: Current }` | 17 |
| `use` | at the top of a file, imports the shared definitions of a module: `use mechanics`, `use mechanics.{Body}`, `use optics` | 19, 22 |
| `while`, `step` | in an algorithm: `while c { ... }` repeats while `c` holds; `step` ends one occurrence's work | 21 |
| `frame` | in a model, a frame of its space: `frame slope(at: origin, x: along)` | 23 |
| `grid` | in a model, a grid of cells for fields: `grid rod(x: [0 m, 1 m], cells: 50)` | 16 |
| `boundary` | in a model, a field's boundary conditions: `boundary T { left = 100 }` | 16 |
| `periodic` | `boundary T periodic`: the field's grid wraps around | 16 |
| `instrument` | in a presentation, declares an instrument whose readings are rounded to `resolution`, limited to `range` and disturbed by normal `noise`; `n(e)` reads `e` through it | 15 |
| `restart` | a new run from `t0` with the parameters as they are now: a timeline action, and in `button(...)` a lab's runtime control (`reset` goes back to the starting values) | 15 |
| `run` | a test case; in timelines, `run rate r` | 1, 8 |
| `set` | an operation that replaces a value | 5 |
| `show` | a timeline action that adds a representation | 8 |
| `space` | a geometric space: `space Plane = euclidean(2)` | 3 |
| `start` | an event at the start of a run; `start of beat` | 5, 8 |
| `state` | continuous state | 4 |
| `symbol`, `unit` | display symbol and unit of a binding | 2 |
| `true`, `false` | Boolean values | 1 |
| `where` | a parameter's condition; an observation filter | 2, 6 |
| `zeno`, `settle` | what happens when an event repeats without end | 5 |
| `zoom` | the zoom of a `camera` action | 8 |
| `enter` | reserved | reserved for later |

## Words

**Reserved** (never names): `space model presentation run object const param input state discrete derived fn flow process event equation constraint on if then else and or not otherwise in where true false zeno stop settle set contribute create destroy connect disconnect emit enter checked within policy reject report intervenable private symbol unit rising falling crossing at every from start request enum match`.

**Contextual** (keywords only where expected, names elsewhere): `for view panel observe live over microstep show as drag click propose permit timeline layout row column scene beat sequence rate until hold seek reset branch intervene wait explore limit keep fallback narrate highlight hide reveal zoom animate camera bind release config expect exactly rel of with learner continue press undo redo combine ease title random runs reseed restart instrument grid boundary periodic unknown enforce use algorithm while step restitution across azimuth elevation frame by`.

## Diagnostics

**Syntax** (`SX`): E01 lexical error, E02 syntax error, E03 unknown name, E04 unknown type or dimension, E05 unknown unit, E06 not in the v0 IR, E07 reserved word used as a name, E08 form not allowed here (also an option the kind does not take), E09 duplicate declaration (also an option or clause given twice).

**Model** (`MK`):

| Code | Meaning |
|---|---|
| MK-E01 | dimension or type mismatch |
| MK-E02 | dimensioned argument to a dimensionless-only function |
| MK-E03 | bare number where a quantity is required |
| MK-E04 | invalid point operation (point + point) |
| MK-E05 | values from different spaces combined |
| MK-E06 | operation or flow targets something that cannot change that way |
| MK-E07 | continuous state set outside a handler or intervention |
| MK-E08 | handler sets a parameter |
| MK-E10 | target both defined and contributed to |
| MK-E11 | contribution to a target with no combination |
| MK-E12 | flow condition depends on continuous state or `t` |
| MK-E13 | level condition used as a continuous trigger |
| MK-E14 | algebraic loop |
| MK-E15 | cycle among initial definitions |
| MK-E16 | repeating crossing event without a Zeno policy |
| MK-E17 | `match` missing a case, or with a case twice |
| MK-E18 | declared function reads something other than its parameters, constants and functions |
| MK-E19 | two `set` operations on one target in one handler |
| MK-E20 | stored function value that reads bindings |
| MK-E21 | tuple used as a vector where no vector is expected |
| MK-E22 | target or case defined twice |
| MK-E23 | declared function calls itself |
| MK-E24 | event payload with no source |
| MK-E25 | `index` read outside the overrides of a collection |
| MK-E26 | a member that does not exist, a part used as a value, or an aggregate that has no value |
| MK-E27 | retired: rates may read continuous state and `t` |
| MK-E28 | a distribution outside an event's condition and operations |
| MK-E29 | `noise` outside a flow, or in a field's flow |
| MK-E30 | a flow that is not linear in `noise` |
| MK-E31 | a field of another grid read where a field is computed |
| MK-E32 | `laplacian` or `gradient` outside a flow or definition of a field of its grid |
| MK-E33 | a boundary on something that is not a field, an unknown or repeated edge, or periodic with held edges |
| MK-E34 | unknowns and the equations that read them do not match: an unknown no equation reads, more or fewer equations than unknowns, or equations that cannot determine their unknowns separately |
| MK-E35 | an equation solved for an unknown reads a rate (`der`) or a field |
| MK-E36 | an enforced constraint that cannot be enforced: no rate of it reads an unknown, it has a form without a rate (`if`, `match`, `min`, `max`), or it reads nothing the runtime can move; an enforced inequality without `with F`, or `with` where it does not apply |

**Presentation** (`PK`): E01 broken reference, E02 expression does not check (the kernel's message is quoted), E03 control or action targets something the learner may not change, E04 missing or wrong scale or axis dimension, E05 representation unsuited to its sources or view, E06 representation kind not implemented.

**Run diagnostics** have categories rather than codes: configuration, initialization, constraint, equation, conflict, cascade, Zeno, computational, intervention (`docs/spec/02-runtime-contract.md` section 10).
