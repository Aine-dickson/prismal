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
| `fn f(x: A, y: B): R = e` | a declared function: reads only its parameters, constants and other functions | 2 |
| `enum Phase { a, b, c }` | an enumeration: a type whose values are its cases | 5 |
| `flow { der(x) = e; der(y) += e }` | rate of change: one definition, or summed contributions | 4 |
| `process Name { flow ...; event ... }` | a named group of flows and events | 4 |
| `event E on trigger [if c] [{ ops }] [zeno settle { ops } \| zeno stop]` | an event | 5 |
| `equation N: a == b [checked within tol]` | a stated relation, shown and optionally checked | 6 |
| `constraint N: c [within tol] [policy reject \| report \| stop]` | a checked condition (`within` for `==` only) | 6 |
| `object Name { ... }` | an object type, with the body of a model; declared in a model | 9 |
| `parts { a: T { x = e }  c: T[n] { x = e } }` | one object, or a collection of `n` members; overrides and input connections, with `index` the member's number | 9 |
| `flow { for b in c { der(b.x) = e } }` | flows the model writes for each member | 9 |

Each block keyword also has a one-line form: `param g: Acceleration = 9.81 m/s^2`, `flow der(x) = v`.

**Modifiers:** `symbol "θ"` (display symbol), `unit deg` (display unit of an angle), `intervenable` (state the learner may change), `private`, `combine sum|product|min|max|any|all` (discrete state that takes contributions).

**Triggers:** `rising(g)`, `falling(g)`, `crossing(g)` (zero crossings of `g`), `at τ`, `every Δ [from τ0]`, `start`, `request`, `E` (after event `E` occurs or is emitted), `input(i)`. `request(p: T)` and `E(p: T)` receive a payload named `p` (chapter 5); `request(b in c)` receives a member of the collection `c`, and `request(b in c, p: T)` several payloads (chapter 9).

**Inputs:** `input { x: T [= default] }`, values from the environment; a case supplies them with `input { x = v; x = v at τ }` (chapter 5).

**Members** (chapter 9): `a.x`, `c[k].x`; `sum`, `min`, `max`, `any`, `all` of `(e for b in c [if cond])`; `count(c)`, `count(b for b in c if cond)`. In views, `for b in c { reps }` draws one representation per member, named `name[k]`.

**Operations:** `set x = e`, `set b.x = e` (a member's binding), `emit E`, `emit E(v)` (with a payload), `emit E(b, v)` (several), `contribute x += e` (into discrete state declared with `combine`; any number combine, a `set` of the same binding conflicts, chapter 5).

## Types and values

| Form | Meaning | Chapter |
|---|---|---|
| `Real`, `Boolean`, `Angle` | numbers, truth values, angles (dimensionless) | 1, 2 |
| `Length`, `Mass`, `Time`, `Velocity`, `Acceleration`, `Force`, `Energy`, `Power`, `Pressure`, `Frequency`, `Momentum`, `Area`, `Volume`, `Current`, `Amount` | named dimensions | 2 |
| `Quantity<M/T^2>` | any dimension from `L M T I Θ N J` | 2 |
| `Point`, `Vector<Length>`, `Vector<Velocity>` | geometry in the model's space | 3 |
| `9.81 m/s^2`, `45 deg`, `2π`, `0` | literals; `0` has any dimension and is the zero vector | 2, 3 |
| `origin + (1 m, 2 m)` | a point | 3 |
| `v.x`, `\|v\|` | component, norm | 3 |
| `[a, b]`, `(a, b)`, `[a, b)`, `(a, inf)` | intervals | 1 |
| `if c then a else b` | conditional | 4 |
| `match p { a => x, b => y }` | a value per case of an enumeration, every case once | 5 |
| `a`, `Phase.a` | a case of an enumeration | 5 |
| `t`, `t0`, `elapsed` | current time, start, time since start | 4 |
| `sin cos tan sqrt exp log abs min max atan2` | functions | 2 |

Operators: `+ - * / ^`, `== != < <= > >=`, `and or not`. Units: `m cm mm km s ms min h kg g N J W Pa Hz A K mol cd rad deg rev %`.

## Presentation

| Form | Meaning | Chapter |
|---|---|---|
| `view n: spatial(Space, scale: 1 m -> 40 px, y: up) { reps }` | spatial view | 3 |
| `view n: plot(x: [a, b], y: [c, d]) { reps }` | plot view; ranges give the axes their dimensions | 1, 4 |
| `plot(..., follow: y)`, `follow: (x, y)` | those axes grow to keep the data in view | 7 |
| `plot(..., x_unit: ms, y_unit: cm)` | display units for the axes | 7 |
| `plot(..., window: 10 s)` | the `x` axis shows the latest 10 s once the data passes its range | 7 |
| `panel n { reps }` | region for controls and text | 1 |
| `permit learner { zoom; pan; timeline_controls }` | what the learner may do with views and time | 7, 8 |
| `observe { n = e schedule }` | observations | 1, 6 |
| `timeline { scene s { beat b { actions } } }` | a lesson | 8 |
| `layout row(a, column(b, c))` | where views go on the page | 7 |
| `title "..."` | the presentation's title, above its views | 7 |

**Representations:** `text("... {e} ... {e:.N} ..." [, at: P])` (an author's text with values in braces, `:.N` for N decimals; `at` places it in a spatial or plot view), `title("...")` (the caption of its view or panel), `marker`, `arrow`, `segment`, `polyline`, `polygon`, `circle(P, r)`, `ellipse(P, a, b, rotate: θ)`, `arc(P, r, from: θ1, to: θ2)`, `trace(P every Δ)`, `function_graph`, `series_plot(e every Δ)`, `axes`, `grid`, `label`, `formula`, `equation(name)`, `table(e every Δ)`, `slider` (with `ends: ("low", "high")`, words at its ends), `number_input`, `toggle`, `button(E)`, `button(E(v))`, `button(reset|undo|redo)`, `group(at: P, rotate: θ, scale: k) { members }` (chapter 7). `color: red|orange|yellow|green|teal|blue|purple|pink|gray|ink` and `line: solid|dashed|dotted` style drawn kinds (chapter 7); `as name` names one; `{ on drag [head] [live] as p { propose x = e } }` declares its inverse (`live`: the run keeps going while dragging), and `{ on click request E(v) }` the event a click requests; in `for b in c { ... }`, `propose b.x = e` and `request E(b)` act on that member (chapter 9). A view's `on click as q request E(q)` requests `E` with an empty point clicked (chapter 9). Any representation takes `when: c`: it is shown only while the condition `c` holds (chapter 7). Markers, arrows and graphs take `label: "..."` or `label: none` (chapter 7).

**Observation schedules:** `live`, `every Δ`, `at τ`, `on E [microstep n]`, `over [a, b]`. **Sources:** any expression, `event_log [of E] [where zeno_applied]`, `diagnostics [of element]`, `intervention_log`.

**Timeline actions:** `narrate "..." [for d]`, `run rate r [until E]`, `hold`, `seek τ`, `reset`, `branch`, `highlight name`, `hide name [for d]`, `reveal fade|draw [for d] [in view] { reps }`, `camera view [to P] [zoom z] [for d]`, `in view { reps }`, `show rep`, `intervene { set p = e }`, `request E`, `request E(v)`, `request E(c[k], v)`, `wait d`, `wait learner [limit L] [fallback { actions }]`, `animate name opacity|offset|scale to v [for d] [ease e]`, `animate name color|line to word [for d] [ease e]`, `animate name morph to other [for d] [ease e]`, `release name`, `bind name [for d]`, `sequence { ... }`, `explore [limit L] [keep p, ...] { controls } [fallback { actions }]` (chapter 8).

## Options and clauses

Two forms add details to a form, one for each place they appear (D-081):

- **Inside parentheses**, a representation or a view takes **named options**, `name: value`, after its positional arguments: `marker(pos, color: red, label: none)`, `plot(x: [0 s, 5 s], y: [0 m, 2 m], follow: y)`. Each kind takes the options below and no others; a misspelled or repeated option is an error (SX-E08, SX-E09).
- **After a statement's head**, a declaration, event, constraint or timeline action takes **clauses**, a word and its value without a colon: `param g: Acceleration = 9.81 m/s^2 in [1 m/s^2, 20 m/s^2] symbol "g"`, `camera scene to P zoom 2 for 1 s`. Clauses come in any order, each at most once (an event keeps the order shown, since its block sits among its clauses); the formatter prints them in the order of this page.

| Kind | Options (every representation also takes `when: c`) |
|---|---|
| `marker` | `at`, `color`, `label` |
| `arrow` | `from`, `scale`, `color`, `line`, `label` |
| `function_graph` | `color`, `line`, `label` |
| `segment`, `polyline`, `polygon`, `circle`, `trace`, `series_plot` | `color`, `line` |
| `ellipse` | `rotate`, `color`, `line` |
| `arc` | `from`, `to`, `color`, `line` |
| `text` | `at` |
| `formula`, `equation` | `live` |
| `slider` | `range`, `step`, `ends` |
| `number_input` | `range`, `step` |
| `button` | `label` |
| `group` | `at`, `rotate`, `scale` |
| `axes`, `grid`, `label`, `title`, `table`, `toggle` | none |
| view `spatial(Space, ...)` | `scale`, `y` |
| view `plot(...)` | `x`, `y`, `follow`, `x_unit`, `y_unit`, `window` |

| Statement | Clauses |
|---|---|
| declaration in `param`, `state`, ... | `in I` or `where c` (one range), `intervenable`, `private`, `combine k`, `symbol "s"`, `unit u` |
| `equation` | `checked within tol` |
| `constraint` | `within tol`, `policy p` |
| `event` | `on trigger` first, then `if c`, the block, `zeno ...` |
| `narrate "..."`, `hide`, `bind` | `for d` |
| `run rate r` | `until E` |
| `reveal fade\|draw` | `for d`, `in view` |
| `camera view` | `to P`, `zoom z`, `for d` |
| `animate name property to v` | `for d`, `ease e` |
| `explore` | `limit L`, `keep p, ...` |
| `wait learner` | `limit L` |

## Runs

```prismal
run Name of Model with Presentation {
  param  { p = value; ... }
  config { solver = rk4; h = 0.01 s }        // or rtol = ...; atol = ...
  until t0 + 10 s
  learner { at 15 s: set slider h = 20 m; at 16 s: continue }
  expect { ... }
}
```

**Expectations** (chapter 6): `a == b within tol`, `a == b within rel r`, `a == b exactly`, `a in [lo, hi]`, `obs[k]`, `(e on E [microstep 0])`, `log == [E1, E2]`, `start of b`, `end of b`, `initialization fails`, `configuration rejected`.

## Glossary

Every word of the language, what it does, and the chapter that teaches it.

| Word | What it does | Chapter |
|---|---|---|
| `and`, `or`, `not` | Boolean operators | 1 |
| `as` | names a representation: `marker(pos) as ball` | 7 |
| `at` | an event at an instant (`at τ`); an observation at an instant; a learner step or input change at a time | 5, 6, 8 |
| `beat` | a step of a scene in a timeline | 8 |
| `animate` | a timeline animation of a drawing's opacity or offset | 8 |
| `bind`, `release` | timeline actions: a drawing follows the model again, or stays as it is | 8 |
| `branch` | a timeline action: the lesson continues on a new run version | 8 |
| `camera` | a timeline animation of a view's centre and zoom | 8 |
| `checked` | `equation N: a == b checked within tol` is checked while running | 6 |
| `config` | a run's solver settings | 6 |
| `const` | constants: never change | 2 |
| `constraint` | a condition checked while running, with a policy | 6 |
| `continue` | a learner step that ends an explore beat or a continue point | 8 |
| `press` | a learner step that presses an explore beat's button: `at 3 s: press drop` | 8 |
| `crossing` | an event when a value crosses zero either way | 5 |
| `derived` | values computed from others, always current | 1 |
| `discrete` | state changed only by events | 5 |
| `click` | the gesture that requests an event: `on click request E(b)`; in a view, `on click as q request E(q)` | 9 |
| `drag` | the gesture of an inverse: `on drag as p`; of a member, `propose b.pos = p` | 7, 9 |
| `else`, `if`, `then` | conditional values; `if` also filters an aggregate and enables an event | 1, 5, 9 |
| `emit` | an operation that makes another event happen | 5 |
| `enum`, `match` | enumerations and a choice by case | 5 |
| `combine` | how contributions combine into discrete state: `combine sum`, `product`, `min`, `max`, `any`, `all` | 5 |
| `ease` | the easing of an animation: `animate R scale to 2 for 1 s ease linear` (`linear`, `smooth`, `in`, `out`) | 8 |
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
| `in` | the space of a model (`model M in Plane`), a range (`x in [a, b]`), a view (`in scene { ... }`), a collection (`for b in row`), a member payload (`request(b in row)`) | 1, 3, 8, 9 |
| `index` | the number of a member in a collection's overrides | 9 |
| `intervene` | a timeline action that changes a parameter at the instant shown: `intervene { set p = e }` | 8 |
| `input` | values supplied from outside, with an optional default; an object's connections | 5, 9 |
| `intervenable` | state the learner may change | 3 |
| `layout`, `row`, `column` | a presentation's page layout: views side by side or one below the other | 7 |
| `learner` | a run's scripted learner inputs; `wait learner`, a continue point | 8 |
| `live`, `over`, `microstep` | observation schedules: current value, over an interval, at a microstep; `on drag live`: a drag that steers the running model | 6, 7 |
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
| `presentation` | how a model is shown | 1 |
| `private` | a binding hidden from outside | 2 |
| `process` | a named group of flows and events | 4 |
| `propose` | the binding change an inverse proposes | 7 |
| `relation` | a relation type, whose endpoints are members of collections | 9 |
| `undirected` | before `relation`: a relation whose two endpoints have no order, read with `s.has(o)` and `s.other(o)` | 9 |
| `request` | an event requested from outside (a button, a lesson) | 5 |
| `undo`, `redo` | in `button(...)`, a lab's runtime controls, with `reset` | 7, 8 |
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

**Contextual** (keywords only where expected, names elsewhere): `for view panel observe live over microstep show as drag click propose permit timeline layout row column scene beat sequence rate until hold seek reset branch intervene wait explore limit keep fallback narrate highlight hide reveal zoom animate camera bind release config expect exactly rel of with learner continue press undo redo combine ease title`.

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

**Presentation** (`PK`): E01 broken reference, E02 expression does not check (the kernel's message is quoted), E03 control or action targets something the learner may not change, E04 missing or wrong scale or axis dimension, E05 representation unsuited to its sources or view, E06 representation kind not implemented.

**Run diagnostics** have categories rather than codes: configuration, initialization, constraint, equation, conflict, cascade, Zeno, computational, intervention (`docs/spec/02-runtime-contract.md` section 10).
