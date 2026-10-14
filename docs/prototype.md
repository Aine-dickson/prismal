# Rust Kernel Prototype

The first implementation of Prismal's core semantics, built kernel-first (D-002): a Rust library and API that runs the first-slice reference programs (D-012) against the specification (`docs/spec/`). Programs are written in the working syntax (D-028) and lowered to the IR by the text parser, or built through the IR API.

- **Status:** v0 prototype, 2026-09-30. Every expectation of RP-01 to RP-08 is checked, model side and presentation side. The text parser reads all eight programs from their documents and lowers them, presentations and runs included, to the IR. The web player runs them in a browser.
- **Build and test:** `cargo test` at the repository root (Rust 1.94 or later). `cargo run --release -p prismal-runtime --example report` prints the measured errors quoted below.

## Structure

| Crate | Specification | Content |
|---|---|---|
| `crates/prismal-ir` | `04-ir.md` | IR types with JSON serialization (D-037), dimensions and units, a builder API, elaboration of objects and collections (`elaborate.rs`, D-055) |
| `crates/prismal-kernel` | `01-model-kernel.md` | Type and dimension checking with expected-type propagation (D-030, D-032), static diagnostics MK-E01 to MK-E22, dependency analysis, compilation, evaluation with statuses |
| `crates/prismal-syntax` | `docs/syntax-study/working-syntax.md`, `04-ir.md` | Lexer, parser to a syntax tree, lowering of spaces, models, presentations and runs to the IR with a source map, located diagnostics |
| `crates/prismal-present` | `03-presentation-kernel.md`, `04-ir.md` section 7 | Presentation checks, observation, expectations, projection and frame descriptions, formula typesetting for every medium (`math.rs`, D-046), interaction, the explanation timeline |
| `crates/prismal-host` | `05-host-interface.md` | The host interface: an engine of documents (from text or the IR, updated with identities kept) and instances (sessions and lessons), with a Rust API and the JSON protocol |
| `crates/prismal-web` and `web/` | D-018, `03-presentation-kernel.md` section 12 | The web player: the host interface compiled to WebAssembly and a browser front end that renders frame descriptions |
| `crates/prismal-stdio` | `05-host-interface.md` HI-6.4 | The host interface as a process: the JSON protocol over standard input and output, one request and one response per line, with request ids; a Python host (`client.py`) |
| `crates/prismal-svg` | `03-presentation-kernel.md` section 12, `05-host-interface.md` section 5 | The SVG renderer: frame descriptions drawn as standalone SVG documents with no browser, for still images, vector documents and image sequences |
| `crates/prismal-media` | `03-presentation-kernel.md` section 12, D-052 | Media export: the SVG renderer's frames rasterized (resvg) and encoded by an external encoder (ffmpeg) as video files, PNG stills and PNG sequences, with captions as WebVTT |
| `crates/prismal-runtime` | `02-runtime-contract.md` | Runs, `dopri5` and `rk4` with dense output, crossing detection and location, event iteration in superdense time, Zeno detection, constraints and equation checks, interventions and requests, time events, observation, an interactive session with undo and redo |

The reference programs are in `crates/prismal-runtime/tests/common/mod.rs`, each mirroring its working-syntax text in `docs/spec/reference-programs/`. Each program has its own test file.

## Text parser

`crates/prismal-syntax` reads the working syntax (D-028, D-035, D-040) and lowers it to the IR the kernel checks and the runtime runs.

1. **Lexer** (`lexer.rs`): tokens with line and column, comments kept for notes (D-036). Newlines end statements except inside `( )` and `[ ]` and after a continuing operator or word (working syntax section 1.5).
2. **Parser** (`parser.rs`): recursive descent to a syntax tree (`ast.rs`) for all four top-level forms: `space`, `model`, `presentation`, `run`. A statement that fails to parse is reported and skipped to the next statement end at the same brace depth, so every error in a file is reported in one pass.
3. **Lowering** (`lower.rs`, `lower_present.rs`): spaces, models, presentations and runs to the IR (04-ir sections 5 and 7), with a source map from each IR identity to its declaration.
4. **Checking** (`check`): the kernel's static checks, with each diagnostic placed at the declaration of the element it names.

`cargo run -p prismal-syntax --example prismalc -- FILE [--ir]` checks a source file or a Markdown document (its `text` and `cases` blocks) and prints the diagnostics, or the IR as JSON:

```text
bad.prismal:4:22: MK-E01: type or dimension mismatch: found Quantity<L T^-1>, expected Quantity<L T^-2>
      flow { der(y) = v; der(v) = -g * 2 s }
                         ^^^^^^^^^^^^^^^^^
```

### Lowering choices

| Construct | IR | Reason |
|---|---|---|
| Identities | declaration paths: `Model.name`, `Model.event.name`, `Model.process.name`, `Model.equation.name`, `Model.constraint.name`, `Model.flow.n` (n-th flow in source order) | D-036; the same identities the builder API gives |
| `where cond`, `in I` on a parameter | a `reject` constraint `name_range`, attached to the parameter | MK-12.4, 04-ir section 5.4 |
| `constraint` without a name | named `c1`, `c2`, ... by position | the IR needs a name; a named constraint keeps its identity across edits |
| Constraint without `policy` | `report` | MK-12.4 |
| `every Δ` without `from` | `from t0` | MK-15.3 |
| Process kind | `continuous` if the process has flows, otherwise `discrete` | MK-14.1: the IR carries the kind; the working syntax has no spelling for it yet |
| `-5`, `-1e-8 m` | a negative literal | IR-1.1: one surface form of a negative literal (04-ir section 6) |
| `2π` | `2 * {"const": "pi"}` | D-039 |
| `in [0, inf)` | `0 <= x` | the IR holds no infinities (04-ir section 6) |
| `a < b <= c` | `a < b and b <= c` | 04-ir section 6 |
| `v.y` | component by the axes of the default space (`x`, `y`, `z` without one) | MK-4.2 |

### Diagnostics of the text

| Code | Meaning |
|---|---|
| SX-E01 | lexical error (unexpected character, unterminated string) |
| SX-E02 | syntax error; includes `=` written where `==` is meant, and a name directly after a number |
| SX-E03 | unknown name, event, space or component |
| SX-E04 | unknown type or dimension |
| SX-E05 | unknown unit (including `px` in a model) |
| SX-E06 | construct not in the v0 IR (`object`, collections) |
| SX-E07 | reserved word used as a name (D-040) |
| SX-E08 | form not allowed here (presentation forms in a model, `origin` without a default space, a range on a non-parameter, arity) |
| SX-E09 | duplicate declaration (spaces, models, processes, events, equations, constraints; duplicate bindings are the kernel's MK-E22) |

### Acceptance by text

The reference programs are read from their documents in `docs/spec/reference-programs/`, so the documents are the test input.

| Test | Content |
|---|---|
| `prismal-syntax/tests/documents.rs` | every `text` and `cases` block of the reference programs and of the working-syntax document parses (28 blocks); every reference-program document lowers |
| `prismal-runtime/tests/text_programs.rs` | the lowered models of RP-01 to RP-08 equal the builder models of `tests/common` exactly, survive JSON and check; 19 diagnostic variants and model variant RP-03.V1, written as one-line text edits, give their named codes; (the `cases` blocks are run by `prismal-present`) |
| `prismal-syntax/tests/rules.rs` | lexical rules, lowering choices, notes, located and recovered diagnostics |

RP-01.D1 is not covered by text: it is stored in IR form because the working syntax has no level-style trigger.

### Formatter and identities

`prismal_syntax::format` prints an IR document in the canonical form of working syntax section 1.4: declarations in blocks by role with aligned columns, then flows, processes, events, equations and constraints; parameter ranges as intervals when two-sided, as `where` otherwise; `run rate r until E`; representations in their views; author notes as `///` comments. Dimensions print as authors write them (`Quantity<1/L>`, `Quantity<M/T^2>`), a dimensionless binding with an angle display unit as `Angle`.

`prismal_syntax::identity` implements D-036. `reconcile(new, prev)` gives each element of a newly lowered document the identity it had in the previous IR, matched by declaration path (by name within its container; flows by target, kind, process and position), restores the previous order of every list with new elements appended (IR-1.7), and gives an unmatched element whose path identity a renamed element holds a fresh one (`Model.speed~2`). `rename(doc, id, name)` is a tool rename that keeps the identity (a parameter's range constraint follows it). `diff(prev, new)` lists removed and added identities: a rename made in plain text shows as both.

`cargo run -p prismal-syntax --example fmt -- FILE [--previous IR.json] [--ir OUT.json]` prints the canonical form, matching and reporting identities against a previous IR. The web player's Source tab has a Format button.

Acceptance (`prismal-web/tests/format.rs`): every reference program and every program of the guide (26) goes from text to IR to canonical text to IR and gives back the same IR after `reconcile`, and printing it again gives the same text; RP-01's canonical form; a tool rename of `speed` in RP-08 prints the new name at every reference and keeps the identity through another round trip; a plain-text rename in RP-06 is reported by `diff`.

## Presentation prototype

`crates/prismal-present` implements the presentation kernel (`03-presentation-kernel.md`) over the runtime, and the presentation and run IR (04-ir section 7), which the text parser now lowers.

| Module | Content |
|---|---|
| `check` | static checks of a presentation against its model, PK-E01 to PK-E06 (03 section 16) |
| `data` | observations of a run for every source (expression, event log, diagnostics, intervention log) and schedule (PK section 3) |
| `expect` | headless evaluation of run cases (PK-4.2); a case whose presentation has a timeline is played as a lesson |
| `frame` | representations compiled for their view with unit-aware scales (PK-5.5), projection on a state, frame descriptions with text alternatives (PK-6, PK-7.5, PK-11.1, PK-12.1) |
| `interact` | controls, drag transactions with validated previews, keyboard operation, undo and redo, refusal of actions a presentation does not offer (PK section 10, PK-11.2) |
| `timeline` | the lesson player: time mapping, D-033 order at a beat's start, `wait_until` from located event instants, explore beats on a branch with and without `keep`, video fallbacks, captions, announcements, frames at any presentation instant and export (PK sections 8, 9) |
| `text` | numbers, quantities with units, expressions printed with display symbols (D-034) |

`cargo run -p prismal-present --example cases -- FILE...` runs the cases of a program headless and reports every expectation (for a lesson, also the beat timings).

Representations implemented: `marker`, `arrow`, `segment`, `polyline`, `polygon`, `trace`, `function_graph`, `series_plot`, `label`, `formula`, `equation`, `table`, `slider`, `number_input`, `toggle`, `button`, `axes`, `grid`. A representation argument that names a model event or equation lowers to an `element` argument (04-ir section 7): `button(drop)` requests an `on request` event (D-027) at the instant shown, `equation(name)` typesets a model equation with its symbols' values. Timeline actions: all of PK-9.2. `hide` stops showing a named representation; `reveal`, `hide ... for` and `camera` are named effects (D-042); `wait learner` is a continue point (D-067); `animate` drives opacity and an offset, `release` and `bind` hand a representation's geometry between its projection and the lesson (D-068). Others are reported as PK-E06, never dropped.

An interactive session has a display clock (RC section 12): its run is computed to the configuration's end, `seek` chooses the instant shown, and the learner's actions take effect at that instant (RC-11.2), so the trajectory after it is recomputed and the one before it is unchanged; `reset` starts a new run with an empty log. A static model has one instant (RC-12.4).

### Acceptance

| Test | Content |
|---|---|
| `prismal-present/tests/cases.rs` | every case written in the working syntax (RP-01 to RP-05, RP-08) runs headless: all 36 expectations pass, including event lists, empty diagnostics and beat timings; the presentation and run IR survive JSON |
| `prismal-present/tests/rp06.rs` | RP-06's learner script through the presentation, E1 to E14; every frame shows one `a` in the graph, slider, formula and marker (E10) |
| `prismal-present/tests/rp07.rs` | RP-07's drag of `u`'s head through its declared inverse; the frame after the drag, E1 to E5 |
| `prismal-present/tests/rp08.rs` | RP-08 cases A (keep), B (no keep), C (video) and D (restricted), E1 to E16, including bit-identical landings on replay, captions, announcements, two identical exports, and the fallback report |
| `prismal-present/tests/checks.rs` | PK-E01 to PK-E06 on edits of RP-06, RP-07 and RP-08 |

## Host interface

`crates/prismal-host` implements `docs/spec/05-host-interface.md` (D-044, D-045): the interface through which any system embeds Prismal.

| Part | Content |
|---|---|
| `Engine` (`lib.rs`) | documents and instances by handle; `load`, `open`, `close`; `handle`, the JSON protocol (HI-6.2) with `capabilities` and the interface diagnostics HI-E01 to HI-E03 |
| `Document` (`document.rs`) | a program from source text, a Markdown document or the IR; loaded with its check diagnostics (by identity, located when from text); `update` with identities kept by `reconcile` and spans following them; `rename` keeping identity; IR, canonical text, catalogue, cases |
| `Instance` (`instance.rs`) | a presentation opened as a lesson or an interactive session: layout (views, coordinate systems, content extent, beats, captions, explore windows), frame descriptions with viewports and focus, the math box tree of a formula (for media with a math engine), observations as text, semantic and raw inputs, lesson inputs |
| `input.rs` | raw input (D-047): viewports of views (framing from the extent, growth to fit in sessions, the learner's zoom and pan, the timeline's camera), the mapping between a view's drawn pixels and view coordinates, hit testing of draggable parts with a reach by pointer kind (PK-10.2a), the focus order (PK-11.2b) |

Snapshots (RC section 14.1, RC-14.3a): a run takes a snapshot at its start, where logged actions apply and every 8 steps, holding the state, sign references, Zeno monitors, time-event schedules, payloads, the solver's proposed step, the lengths of its records and `theta`, the latest stop an earlier step depended on. `resume(prev, cfg)` continues `prev` from its latest snapshot before the first changed action and after `theta`; sessions (commits, drag previews, undo, redo) and lesson run versions resume instead of recomputing from `t0`. Showing an earlier instant needs no recomputation: a run keeps its whole trajectory as dense output. `prismal-runtime/tests/snapshots.rs` compares resumed runs with runs from the start, bit for bit, for actions added, removed and replaced, at bounces, at snapshot instants, at the start and the end, with requests and the fixed-step solver; a change at 9.5 s of a 10 s pendulum run resumes after 9 s.

Sessions and playbacks own their compiled model and presentation, and documents share their checked program with their instances by reference count, so closing a document or an instance frees it (HI-2.2); before, the web player kept every loaded program for the life of the page.

Raw input (D-047, HI-4.5): the host captures pointer, wheel and key events and forwards them in its own terms; the engine targets them, runs drags, pans and zooms, keeps keyboard focus, and answers whether it used each one. `prismal-web/tests/input.rs` drives it through the protocol alone, as a host other than a browser would: RP-07's arrow head grabbed and moved in a view drawn at its natural size and at twice that size, a touch reaching where a mouse misses, hover, cancel; RP-06's handle in plot pixels with an invalid proposal previewed and the last valid one committed; focus with Tab and Shift+Tab, arrow-key steps of the handle and the slider, focus returned to the host past the end, `focus` set directly; a button pressed and a toggle flipped with Enter and space; RP-08's wheel zoom about the pointer, a pan cancelled by Escape, `view_reset`, and an explore slider stepped from the keyboard in the lesson.

Acceptance (`prismal-host/tests/protocol.rs`): capabilities and malformed requests; a document from text and the same document from its IR give the same catalogue, canonical text and frames; located syntax and check diagnostics; a tool rename kept through a later text update, with spans following; a plain-text rename that breaks references refused with the document unchanged; removed identities reported; an interactive session (seek, control, undo) and a lesson (refused and accepted explore input, continue, the second drop from 20 m); inputs of the wrong mode refused (HI-E03); programs freed when their instances and documents close.

## Web player

`crates/prismal-web` is the reference web host (D-018): the host interface compiled to WebAssembly; `web/` renders what it returns. Build and run instructions are in `web/README.md`.

| Part | Content |
|---|---|
| `Player` (`prismal-web/src/lib.rs`) | one document and one instance, as the reference renderer uses them; source diagnostics as line and column |
| `mathml` | a formula's math box tree written as MathML for the browser (D-046), added to frames by `Player` and by the `Engine` binding |
| `examples` | the reference programs embedded from their documents at build time, assembled as the acceptance tests read them |
| `wasm` | JavaScript bindings (`wasm-bindgen`): `WebPlayer` for the reference renderer, and `Engine`, the host protocol for any web host; values cross as JSON strings |
| `web/player.js` | spatial views as SVG in the frame's viewport (grid and axes with ticks in metres), plot views as SVG with their ranges, panels and overlays as HTML (sliders, number inputs, toggles, MathML formulas with live values, labels), pointer, wheel and key events forwarded raw to the engine (D-047) with pointer capture and a grab cursor on hover, the browser's own focus told to the engine, lesson transport, captions, announcements, text alternatives |

A lesson is replayed from the learner's recorded inputs whenever one is added (`timeline::play` with the inputs), which the determinism of playback (PK-8.7) makes safe: frames before the new input are unchanged, which a test checks. Zoom, pan, pause and seeking are the renderer's; they never reach the model.

Lab presentations: RP-01, RP-03, RP-04 and RP-05 each declare a lab presentation with views (the projectile with its trace and height series, the bouncing ball in a plot with a length axis, the pendulum with its rod, trace, angle and energy series, the spring's mass and displacement series), and sliders for parameters that act during the run (gravity, drag, restitution, rod length, stiffness, mass). They play in the player as sessions of the running model; drags pause the run and resume it after the commit (PK-10.9, `hold`). `prismal-present/tests/labs.rs` checks them: positions at displayed instants against closed forms, the trace and series ending at the instant shown, seeking back and forth reproducing frames (RC-12.2), an intervention applying at the instant shown with an unchanged past and a later landing, undo, reset, the bouncing ball resting after the Zeno limit, the energy of the pendulum jumping when `L` changes, the spring's checked equation reported after `k` changes, and the static errors of sampled sources.

Acceptance: `prismal-web/tests/player.rs` loads every example, runs the 36 case expectations through the player, checks located diagnostics, RP-06's slider, drag, rejected drag and undo, RP-07's drag of `u`'s head, and RP-08's lesson (MathML of `R`, explore input at 60 deg landing at R60, unchanged earlier frames, refusal outside the explore beat, video medium). The front end was also driven in a headless browser through the same scripts (drags, keys, the explore slider, compile errors, phone width, light and dark themes).

## SVG renderer

`crates/prismal-svg` is the second renderer (PK-12.2): it draws a frame description as a standalone SVG document, with no browser. It reads only what the host interface gives every host, a layout and frames as JSON (HI section 5), and depends on no other Prismal crate, so it shows that frame descriptions are enough to draw a presentation and that renderers are swappable.

| Part | Content |
|---|---|
| `render(layout, frame, options)` | one document per frame: a header naming the presentation, run and instant; each view in a frame with its caption; spatial views in view coordinates with grid and axes, plot views with ticks, frame and clip; markers, arrows (colored in turn, labels, drag handles), segments, graphs, traces, polygons and groups; panels and the overlay with formulas and equations drawn from their layouts (D-046) with live values, controls in their current state (slider, toggle, number) with display units, buttons, tables, labels and statuses; captions; highlights, reveal opacity, drawn fractions and cameras (D-042); drag previews marked invalid |
| Accessibility | the document's title names the presentation, run and instant; its description lists the text alternatives; every representation carries its text alternative as a title (PK-11.1, D-026) |
| `Options` | plot width, largest view width, pixels per em of formulas, header, light and dark themes (the web player's colors) |
| `examples/render.rs` | `cargo run -p prismal-svg --example render -- PROGRAM PRESENTATION [--at T]... [--fps N] [--video] [--dark] [--out DIR]`: frames at chosen instants, or every frame at N per second, an image sequence for video |

Each view is drawn with the viewport its frame gives (D-047), as the web player draws it; the rest follows the web player's drawing (`web/player.js`): tick steps, marks and sizes. Sizes the web player keeps constant on screen are drawn at one document pixel per view pixel. Colors are presentation attributes, not style sheets, so any SVG consumer draws them. Panels, which the browser lays out as HTML, are stacked under their view.

Acceptance:

- `prismal-svg/tests/frames.rs`: every presentation with views of the reference programs and the guide, drawn at its start, a third of the way and its end (at least 60 documents): each is well formed, every representation is present with its text alternative, markers stand at the frame's view coordinates, formulas have one text element per text run of their layouts; panels, controls, buttons, tables, captions and the dark theme; DropMovie as an image sequence at 10 frames per second, with the ball fading in, the ground drawn, the view box halved by `zoom 2` and restored by `zoom 1`. The documents are written to `target/svg-tests` for inspection.
- `node crates/prismal-svg/compare.mjs` compares frames drawn by this renderer with the same frames drawn by the web player in headless Edge (15 frames of RP-01, RP-03 to RP-08 and the guide's wheel, DropMovie and Unwrap, sessions and lessons, cameras, fades, circles and arcs): the view boxes of spatial views, and for every representation its text alternative and the geometry of its marks (marker and handle centres, line ends, path and polygon points; plots normalized to their frame). All 15 agree; a camera zoom perturbed by 1% is reported. The player opens a program, presentation and instant from its address for this (`#rp08/ProjectileLesson@20`).

## Objects and collections

Object types, contained objects and collections of fixed membership (D-055) are kept as written in the IR and elaborated to a flat model before checking (`prismal_ir::elaborate`, MK-8.3b). `Program::new` elaborates the document; the host keeps the structured document for printing and editing, and locates a member's diagnostics at their declaration (`id@path` to `id`).

| Part | Content |
|---|---|
| Syntax | `object Name { ... }` in a model; `parts { a: T { x = e }  c: T[n] { x = index * e } }`; `a.x`, `c[k].x`, `b.x` in loops; `sum`, `min`, `max`, `any`, `all`, `count` over `(e for b in c [if cond])`; `flow { for b in c { der(b.x) = e } }`; `for b in c { reps }` in views and representation blocks |
| IR | `objects`, `parts` with overrides, `field`, `part`, `item`, `var` and `aggregate` expressions, `builtin: index`, `member` and `each` on flows, `each` on representations (04-ir section 5.6) |
| Elaboration | members' bindings, functions, processes, flows, events, equations and constraints with identities `declaration@path` and names `path.name`; overrides in the container's scope with `index`; connected inputs become derived; enumerations shared; aggregates expanded (`o != b` decided statically, dynamic filters as conditionals); container loops per member; representations per member (`name[k]`), `highlight` and `hide` of a family per member; observations and expectations |
| Formatter | object types as indented `object` blocks, `parts`, member expressions, aggregates, loops of flows grouped in `for` blocks, `for` around repeated representations |

Acceptance: `prismal-present/tests/objects.rs` (a row of bouncing balls and a Moon ball with aggregates and cases; identities, names, overrides, connections; three bodies under mutual gravity with momentum conserved to 1e-12; representations per member and a highlighted family; printing round trips; diagnostics MK-E26, SX-E03, SX-E08, MK-E04 at the declaration once per member); guide chapter 9, whose programs and mistakes `prismal-web/tests/guide.rs` checks.

Found on the way: a ball bouncing on a floor that is not at zero fell through after about fifty bounces, because the sign reference after a bounce was the rounding-level sign of the guard; fixed by RC-7.2a (D-056).

### Relations

Relation types (D-058) are object types with endpoint roles, each a member of a part of the model; a part of a relation type is a relation set, a collection in every other respect (D-057). An endpoint is a private discrete binding holding the member's number in its collection.

| Part | Content |
|---|---|
| Syntax | `relation R(a in c, b in d) { ... }`; `rs: R[n, max m] { a = c[index] ... }`; `connect rs(x, y) { k = e }`, `disconnect s`; `s.a`, `s.a.pos`, `a.pos` inside the relation type; `s.a == o` |
| Elaboration | endpoint bindings `R.a@rs[k]`; a binding of the member at an endpoint becomes `pick(index, [binding of member 1, ...])`; comparisons of members become comparisons of numbers when a member is chosen during the run, constants otherwise; `connect` is a create whose values include the endpoints; `destroy` adds, for each relation of the container ending at the member, a conditional disconnection; derived bindings carry `when` (their member alive); crossing guards of events of members not alive are replaced by zero |
| Kernel | the `pick` expression (only the item picked is evaluated); derived bindings with `when`, not evaluated otherwise and holding a value that is not a number; the `make` operation, which may set a new member's parameters; an event whose handler destroys the member its condition requires is not self-retriggering (D-038) |

Acceptance: `prismal-present/tests/relations.rs` (two balls on a spring oscillating as `1 + cos 2t` to 1e-6; cut, connect, a ball destroyed and its spring with it, frames drawing the springs alive; a thread snapping at exactly 0.5 s; printing; diagnostics); guide chapter 9, Relations between members.

Found on the way: `create` could not give a parameter of the new member a starting value (MK-E08); guards of events of members not alive were evaluated on values that do not exist; a mistake in a member expression was reported twice.

### Members as payloads

A request, and an event following another, may carry a member of a collection (D-059). Elaboration treats the payload as D-058 treats an endpoint: a member chosen during the run by its number.

| Part | Content |
|---|---|
| Syntax | `on request(b in balls)`, `on E(c in balls)`, several payloads `on request(b in balls, j: T)`; `emit E(b, j)`, `request E(balls[2], j)`; targets through endpoints, `set s.b.vel.x = e` |
| IR | a payload's `of` (the collection's part), `members` (its path after elaboration), `items` (several payloads, carried as a tuple and read as its components) |
| Elaboration | a loop variable may be a member chosen during the run; the event's condition gains `pick(k, [alive of member 1, ...])`; `set b.x` becomes one conditional `set` per member; the endpoint of a chosen relation picks among the relations' endpoint bindings; a member supplied to `emit` or a timeline `request` becomes its number, checked against the collection declared |
| Kernel | a tuple expected of a tuple type is checked item by item (a vector literal inside a tuple payload) |
| Runtime | a request whose event is not enabled is rejected with the reason (a member not alive, no member of that number, the condition false) instead of being dropped |
| Host | the protocol's `request` with a JSON payload (`"balls[2]"`, `["balls[3]", [0, 2]]`); event logs print members by name, `remove(balls[2]) at 1 s` |

Acceptance: `prismal-present/tests/member_payloads.rs` (removal, refusals of members not alive or not existing, a member and an impulse, a condition reading the member, `emit` passing a member on, `set` through a relation's endpoint, replay, a lesson's timeline requesting both, printing, diagnostics); `prismal-host/tests/protocol.rs`, `requests_name_members`.

### Drags and clicks on members

| Part | Content |
|---|---|
| Syntax | `for b in c { marker(b.pos) as ball { on drag as p { propose b.pos = p }; on click request remove(b) } }`; `click` is a contextual word |
| IR | a proposal's `member`; a representation's `click` (`event`, `payload`) |
| Elaboration | a proposal's member resolved to the member drawn (`Ball.pos@balls[2]`), refused for a member at an endpoint; a click's member payload becomes its number |
| Presentation | a click's event must be `on request` with the payload it declares; frames carry `click` and the text alternative ends `activate: E`; a drag or a click on a representation not drawn is refused, and a proposal after which the member dragged is not alive is invalid |
| Host | targeting covers clickable representations (`hover` says `drag` and `click`); a press and release within the pointer's reach clicks, further movement drags; clickable representations take focus and Enter or space clicks them; the protocol's `click`; the web player's pointer cursor and keyboard activation |

Acceptance: `prismal-present/tests/member_input.rs` (a keyboard drag of member 2 of 3, refusals for members not alive, clicks with the member as payload, frames, printing, diagnostics); `prismal-web/tests/input.rs`, `clicks_and_drags_on_members` (a click, a drag, Enter on a focused planet, the button, through raw input); `web/check-input.mjs` (a click and a keyboard activation in a browser); guide chapter 9, Labs with members.

### Members that come and go

Collections whose membership changes (D-057) declare a capacity, the most members a run makes (`drops: Drop[max 40]`, `Drop[2, max 4]`), and are elaborated to that many members, the `k`-th member made being `drops[k]` for the whole run. Nothing below elaboration knows about membership, except three small forms.

| Part | Content |
|---|---|
| Syntax | `c: T[max m]`, `c: T[n, max m]`; `create c { x = e ... }` and `destroy b` in handlers and settle clauses; `for b in c { event ... }` in a model; `set b.x = e`, `set c[k].x.y = e` |
| Elaboration | per member a private liveness binding `part.alive@c[k]`, per collection a count `part.created` and a `stop` constraint `c.capacity`; flows guarded by liveness (`if alive then e else 0`), events enabled only while alive, equations satisfied and constraints true while not alive; aggregates filtered by liveness; a create becomes one conditional operation per member it could make, reading the count on the state before the transition; events per member named `land[k]`; representations per member carry `when` |
| Kernel | conditional operations (`if`) checked on their own; `destroy` of a liveness binding; the `extreme` expression for `min` and `max` over members alive (no value over none); identical diagnostics reported once |
| Runtime | the transition performs the conditional operations whose condition holds before it, then decides conflicts on the operations performed: two destroys of a member do not conflict, a set on a member destroyed in the same transition does (MK-16.5) |
| Presentation | representations whose `when` does not hold are not drawn and have no text alternative; a trace of a member starts when it is made; `Interactive::request` requests an event with a payload from a session |

Acceptance: `prismal-present/tests/membership.rs` (a fountain that sprays a drop every half second and destroys it on landing, with counts, heights and a destroyed member read by number checked against closed forms; two creates in one request with a payload; the capacity stopping a run at the instant of the create; frames drawing only the members alive; a container reversing a member's velocity; conflicts; printing; diagnostics); guide chapter 9, Members that come and go.

Found on the way: a type error in a `create` override was reported once for each member the create could make (the kernel now reports identical diagnostics once); aggregates over 30 or more members overflowed the 1 MB main-thread stack of debug builds, because they were folded into a chain as deep as the collection (they are now folded as a balanced tree, depth `log2 n`, which also sums pairwise).

## Media export

`crates/prismal-media` turns a presentation into a video file or raster images (D-052). Frames are the SVG renderer's; the crate rasterizes them with resvg and leaves encoding to ffmpeg, fed raw RGBA frames through a pipe.

| Part | Content |
|---|---|
| `clip(instance, presentation, settings)` | opens a lesson in the `video` medium (fallbacks, PK-9.10) or a session from the start of its run, and draws frame `k` at `k / fps` up to and including the end; collects captions and what the medium could not show (PK-12.3) |
| `Raster` | resvg with the system's fonts and any font directories; generic families resolve to the first installed common font; each font file is mapped once for the whole export |
| `parallel` | Work on every core with results in order: frames are described one after another (a lesson's state depends on the frames before it), then drawn as SVG, rasterized and written in parallel; frames to the encoder go in batches so memory stays bounded. RP-08's lesson (936 frames) exports as PNG frames in 3.4 s instead of 30 s on a 12-core machine |
| `canvas`, `draw` | one canvas per clip, the largest frame rounded up to even pixels; each frame drawn from its top left corner on the theme's background |
| `encode` | the encoder's arguments per container (`mp4`, `mov`, `mkv`: H.264; `webm`: VP9; `gif`: a generated palette), captions as a subtitle track when asked, a WebVTT file beside the video |
| `still`, `write_frames` | a PNG of one instant; a directory of PNG frames with `captions.vtt`, the recording script `narration.txt` and the encoder command |
| `voice` (D-053) | recordings found by cue name, speech synthesized by the system (Windows speech, `say`, `espeak-ng`) or a command, lengths read with ffprobe and overruns reported, the recording script; `encode` delays each narration to its cue and mixes it with looped music |
| `prismal-media` (binary) | `prismal-media PROGRAM PRESENTATION OUT [--fps N] [--scale S] [--at T] [--until T] [--captions burned\|track\|both] [--voice DIR] [--speech system\|COMMAND] [--music FILE] [--music-volume V] [--dark] [--header] [--fonts DIR]... [--encoder PATH]` |

Acceptance: `prismal-media/tests/export.rs` checks frame instants, WebVTT, canvases, encoder arguments and that parallel work keeps the frames' order; exports RP-08 in the video medium (no report, captions from the narration, drawn or not, identical on a second export, frames of different sizes on one opaque canvas); reports a lesson whose explore beat has no fallback; records RP-04's session; writes a still and a frame directory; names a missing encoder; names cues after their beats (`intro`, `intro.2`), writes the recording script, finds recordings by cue and reports silent cues, and builds the mixing arguments. When ffmpeg is found (`PRISMAL_FFMPEG` or the path), RP-08 is encoded as MP4, WebM and GIF, and ffprobe counts the MP4's frames and finds its caption track; generated tones stand in for recordings of `b1` and `b3`, the longer `b3` is reported, and the video is loud inside both cues and silent between them. Otherwise those tests say so and pass. In the browser, `node web/check-voice.mjs` plays RP-08 with speech and audio playback recorded: every cue spoken once at its start and at the playback speed, none when joining a cue part way, a recording started at the offset into its cue, notes on missing and overlong recordings, and silence on pause.


| Program | Covered by the prototype | Not yet covered |
|---|---|---|
| RP-01 | E1 to E9, D1 to D5, IR JSON round trip | - |
| RP-02 | E1 to E12 | - |
| RP-03 | E1 to E15, D1 to D3, model variant V1 | - |
| RP-04 | E1 to E11, D1, D2 | - |
| RP-05 | E1 to E10, D1 to D3 | - |
| RP-06 | E1 to E14, through the presentation and on the session alone | - |
| RP-07 | E1 to E5 through the presentation (frame after the drag), D1 to D6 | - |
| RP-08 | E1 to E16: cases A to D | Audio narration (not required: PK-11.3) |

Every run in the suite is also run twice and compared bit for bit (RC-14.5).

## Measured accuracy

Provisional tolerances in the reference programs were confirmed with these measurements and are now fixed (their histories record the values).

| Program | Quantity | Measured | Tolerance |
|---|---|---|---|
| RP-01 | landing time, range | 1.4e-12 s, 2.0e-11 m | 1e-9 s, 1e-8 m |
| RP-02 default | landing time, range, apex height (relative) | 1.7e-7, 8.9e-8, 4.5e-7 | 1e-5 |
| RP-02 tight | same | 2.9e-12, 2.7e-12, 1.1e-11 | 1e-8 |
| RP-03 | Zeno detection | bounce 63 at 4.06370922186394 s (predicted: bounce 63 at 4.06370922604679 s) | bounce 61 to 65, within 1e-4 s of the limit |
| RP-04 `dopri5` | worst period (relative), energy drift | 2.0e-7, 7.7e-5 | 1e-5, 1e-3 |
| RP-04 `rk4` | same | 7.9e-9, 1.35e-7 | 1e-6, 1e-6 |
| RP-05 `dopri5` | `x(10 s)`, energy drift | 2.1e-7 m, 4.9e-5 | 1e-5 m, 1e-3 |
| RP-05 `rk4` | same | 2.4e-9 m, 1.8e-8 | 1e-7 m, 1e-7 |

The energy drifts agree with the predictions of `tools/refvals.py` (RP-04: 7.8e-5 and 1.3e-7).

## Findings

- **D-038.** The specification's rule for self-retriggering events (MK-15.11) contradicted the reference programs: read literally it accepted the bouncing ball without a Zeno policy; read as RP-03 describes it, it rejected the valid projectile. The rule now follows flows, specialized on the discrete values the handler sets.
- **Diagnostics come in groups.** An invalid variant can produce a secondary diagnostic besides the named one (RP-01.D4 also gives MK-E16, because the invalid flow condition defeats the specialization of D-038). The suite requires the named diagnostic to be among those reported.
- **D-039.** The IR had no form for `π`, so `2π` could only be stored as `6.283...` and formulas (D-034) could not show it. Named constants are now kept by name.
- **D-040.** The reserved-word list of D-035 rejected names the reference programs use (`process drag`, `view scene`, the observation `state`). Presentation, timeline and run words are now contextual keywords; RP-07's observation is renamed `values`.
- **Lexical rules made precise** in the working syntax: a unit has no spaces inside it (`0.01 /m` is a unit, `4 / m` a division); only `π` may follow a number directly; after a declaration's value `in` starts its range, so a membership test there needs parentheses.
- **RP-08 arrows without scales.** `arrow(vel, from: pos)` draws a velocity in a view whose scale maps lengths; PK-5.5 requires a declared scale of the vector's dimension, as the specification's own example (13.1) has. The presentation checker rejected RP-08 (PK-E04); the program now declares `scale: 1 m/s -> 2 px` (its history records it).
- **Formulas need parameter names.** `formula(f)` shows `f(x) = a x^2` (D-034), but IR lambdas kept no parameter names. Lambdas now carry display-only `names` (04-ir section 6).
- **Exact event instants at the end of a wait.** After `run rate 0.5 until landed`, the time mapping's arithmetic (`0 + 0.5 × elapsed presentation time`) gave an instant just before the landing, so the explore branch and the relaunch of RP-08 happened before the landing and playback passed it twice. A beat ended by `wait_until(E)` now shows `E`'s located instant exactly (PK-9.3a).
- **Rules the timeline needed**, recorded as elaborations in 03: the mapping persists across beats (PK-8.2a), reading time and highlight duration (PK-9.2b), waits while holding (PK-9.4a), lesson run end and run versions (PK-9.5a), refusal of learner actions (PK-9.8a), keyboard steps (PK-11.2a; RP-06's marker declares no step, so it moves by 1/100 of the axis span), announcements (PK-11.3a).
- **Display units were not used.** RP-08 narrates the launch angle in degrees and ranges its explore slider in degrees, but the slider showed radians: RP-01's `angle` declared no display unit, and the presentation kernel ignored display units in text alternatives and live formula values although PK-11.1 requires them. RP-01's `angle` now declares `unit deg` (its history records it) and the kernel formats a binding's value in its display unit (`text::fmt_binding`).
- **Keyboard operation of arrow heads.** PK-11.2 requires every drag to be available from the keyboard; the kernel only moved markers. A keyboard step now moves the part a representation is dragged by (a marker, an arrow's head).
- **Plots had no dimensions.** A plot's axes were bare numbers, so neither a scalar model (the bouncing ball's `y` is a length) nor a series against time could be drawn. Plot axes now take their dimension from their ranges (PK-7.3a), and `trace` and `series_plot` take sampled sources (PK-6.3a) with their syntax, `expr every Δ`, and IR form.
- **Equation diagnostics.** A failed equation check printed its residual as `Some(0.0148...)`. The message now gives the instant and the residual.
- **D-041.** Writing the guide, `event alarm on full` never happened: the runtime made `on(E)` due only after `emit E`, following RC section 8.1, while MK-15.3 says after `E` occurs. `on(E)` now follows both.
- **Proposals that read bindings.** The player's intervention log evaluated a drag proposal (`speed = sqrt(p.x * 1 m * g / sin(2 * angle))`) as a constant, and the kernel indexed an empty state and panicked. `constant` now refuses expressions that read bindings or derivatives; the log evaluates proposals on the run's state at their instant.
- **Native and WebAssembly runs differ in the last digits.** Comparing the renderers, RP-08's ball at rest after landing is at `y = -7.06e-10 m` natively and `-1.06e-10 m` in the browser: the elementary functions of the two builds round differently. This is the tolerance-equivalence across platforms that D-015 allows; the comparison checks numbers in text alternatives to 1e-6.
- **Italic correction.** Superscripts and closing parentheses touched the slanted top of italic letters in formula layouts (`v²`, `f(x)`). An italic run now takes 0.06 em more room after it (`math.rs`); renderers draw layouts with serif faces that have true italics, whose widths the layout approximates.
- **Input belonged to each host.** The host interface took only semantic inputs, so each host had to hit-test, map screen positions to view coordinates and keep focus, and the web player's framing (extent, growth, camera, zoom and pan) had to be copied into the SVG renderer. The host now captures and forwards input and the engine interprets it (D-047); both renderers draw with the viewport in the frame. A session's framing now grows to fit each frame on its own, where the web player's used to keep the growth of earlier frames, so the same instant always gives the same frame.
- **Requests ignored enabling conditions.** A requested event ran its handler even when its `if` condition was false, contrary to MK-15.5; found with payloads (`on request(j: Momentum) if j > 0 kg*m/s`). Requests now obey their conditions (RC-11.6b).
- **Constants in constant expressions.** Case parameters and expected values were evaluated with no binding values at all, so a declared function that reads a constant (`apex(10 m/s)` reading `g`) could not be called there. Constant expressions now see the model's constants (`CModel::constant_values`).
- **Expected cases need a type.** `(phase on top) == sinking` compiled the expected side with no expected type, and a case has none of its own; an expected value now takes the enumeration of its subject.
- **Products of words.** `0.5 * mass * speed^2` was typeset `0.5massspeed²`; a thin space now separates a name of more than one letter from its neighbours in a product.
- **`emit` semantics.** A first version made the emitted event itself due; MK-15.10 makes the events triggered `on(E)` due. Fixed and covered by a unit test with the cascade limit (RC-8.3).

## Not implemented

| Item | Where specified |
|---|---|
| Keeping comments that are not notes when formatting (the formatter prints from the IR) | D-036 |
| Relations between relations, relation sets with at most one relation per pair, relation types declared in object types (D-074), creation and destruction as interventions (set aside by D-059), native growable state for very large populations (D-066 option 1); function libraries shared between models | MK sections 8, 16, 17, MK section 2, R-55 |
| Sampled sources `over I`; a button's value read from a control (D-072 option 3) | PK-6.3, PK-6.3b |
| Easing for `reveal`, `hide` and `camera`; animated line width (D-075); narration speed as a learner control separate from playback speed (the web player offers pause, seek, replay, speed, zoom and pan) | PK-8.4, PK-9.7 |
| Learner predictions as expected values; instrument options read from parameters (a slider of measurement noise); higher-order stochastic integrators (Milstein, stochastic Runge-Kutta); a reading's uncertainty as data (it is stated in text alternatives) | PK-4.3, PK-7.4, PK-3.7, RC-13.9 |
| Combinations supplied by types (D-073 option 2) | MK-14.9 |
| Failure policies chosen per category in a run's configuration (D-071) | RC-10.3 |
| Frames of spaces (D-022), affine temperature units | MK sections 3.4, 4 |
| Fields: noise and draws per cell, fields of vectors, fields in object types, meshes, implicit solvers for stiff fields on fine grids, computing a lab's run as it is shown (heavy fields now shorten the default horizon instead) | MK-9.4 to MK-9.8, RC-6.2a, HI-5.1a (D-091) |
| Solved equations and constraints: vector unknowns, solved equations over fields, inequality constraints (contact, a slack rope), enforced constraints whose rates read `der` | MK-11.6 to MK-11.9, MK-12.6 to MK-12.10 (D-092, D-093) |

## History

- 2026-09-30 prototype written; RP-01 to RP-07 model-side expectations and RP-08 model behavior pass; D-038 raised.
- 2026-09-30 text parser (`prismal-syntax`): all eight programs read from their documents lower to the builder IR; diagnostic variants reproduced as text edits; cases run from text; D-039 and D-040 raised.
- 2026-09-30 presentation prototype (`prismal-present`), presentation and run IR (04-ir section 7) and their lowering: every expectation of RP-01 to RP-08 checked; RP-08 arrows given scales; lambda parameter names; elaborations of 03 and its static diagnostics.
- 2026-09-30 web player (`prismal-web`, `web/`): RP-06 to RP-08 interactive in a browser, cases of every program; display units in text alternatives (RP-01 `angle` in degrees); keyboard steps for arrow heads.
- 2026-09-30 dynamic interactive sessions (display clock, interventions at the instant shown, reset); `trace` and `series_plot` with sampled sources; plot axes with dimensions; lab presentations for RP-01, RP-03, RP-04 and RP-05, played in the web player.
- 2026-09-30 language guide (`docs/guide/`) checked by `prismal-web/tests/guide.rs`; D-041; `constant` guarded against expressions that read state.
- 2026-09-30 formatter and identities across edits (D-036); one-sided intervals lower as `where` does.
- 2026-09-30 `button`, `equation`, `table`, `polyline`, `polygon`; `hide`; a series observation compared with a list (`drops == []`).
- 2026-09-30 animations as named effects (D-042): `reveal`, `hide ... for`, `camera`.
- 2026-09-30 `group` (D-043), with members placed in model space; frame tests of the animations; the web player's animations checked in headless Edge.
- 2026-09-30 host interface (`prismal-host`, D-044, D-045): the web player's logic moved into an engine any host embeds; sessions and playbacks own their model; the JSON protocol exported to JavaScript.
- 2026-09-30 formulas typeset for every medium (D-046): math box tree and layout in frame descriptions; labels, drag parts and control symbols and units in the core frame; MathML only in the browser binding.
- 2026-09-30 `prismal-stdio`: the protocol over standard input and output for hosts in any language (HI-6.4); `tests/process.rs` drives the running binary; `client.py` feeds an input from Python.
- 2026-09-30 event payloads (D-050) and inputs (D-051): runtime, syntax, run cases, host `set_input`; `prismal-present/tests/payloads_inputs.rs`.
- 2026-09-30 declared functions (D-048) and enumerations with `match` (D-049): IR, checker (MK-E17, MK-E18, MK-E23), syntax, formatter, identities, text and formula output; guide sections in chapters 2 and 5; `prismal-syntax/tests/functions_enums.rs`.
- 2026-09-30 raw input and viewports (D-047): the host captures, the engine targets, pans, zooms and keeps focus; frames carry viewports; the web player forwards raw events, checked with real events in headless Edge (`web/check-input.mjs`).
- 2026-09-30 SVG renderer (`prismal-svg`): frames as standalone SVG documents and image sequences, compared with the web player's drawing in a browser; italic correction in formula layouts; the player's address selects a presentation and instant.
- 2026-09-30 media export (`prismal-media`, D-052): video files through ffmpeg, PNG stills and sequences, captions drawn, as a track and as WebVTT.
- 2026-09-30 narration sound (D-053): cues named after their beats; recordings, synthesized speech and music in video export; a Voice menu in the web player.
- 2026-09-30 round geometry (D-054): `circle`, `ellipse`, `arc` as exact elliptical arcs in frames, drawn by both renderers; the guide's wheel is a circle; guide lesson Unwrapping a circle.
- 2026-09-30 objects and fixed collections (D-055), elaborated before checking; guide chapter 9; sign references after a transition (RC-7.2a, D-056).
- 2026-09-30 relations (D-058): relation types, relation sets, `connect`, `disconnect`, endpoints; derived values of members not alive not evaluated.
- 2026-09-30 collections whose membership changes (D-057): capacity, `create`, `destroy`, events per member, member targets; aggregates folded as balanced trees.
- 2026-10-01 members as payloads and several payloads per event (D-059); requests refused with a reason; the protocol's `request`.
- 2026-10-01 drags and clicks on members (D-059); guide chapter 9, Labs with members.
- 2026-10-01 plot axes that follow the data, axis display units, zoom and pan of plots (D-070); solver rounding near 0 shown as 0 in point descriptions.
- 2026-10-01 drag mode `live` and the interactive failure policy `pause` (D-071); a step breaking a `reject` or `stop` constraint cut at the last valid instant.
- 2026-10-01 buttons whose event takes a payload, `button(kick(2 kg*m/s))` (D-072).
- 2026-10-01 contributions to discrete state with `combine` (D-073).
- 2026-10-01 `create` and `connect` through part paths, `create left.atoms { ... }` (D-074).
- 2026-10-01 animated color, line style and scale, shape morphs, easing functions; cameras on plots (D-075).
- 2026-10-01 authors' text with values, `when:` on representations, titles of presentations, views and panels (D-076).
- 2026-10-12 words at a slider's ends, the percent unit, labels in display units (D-077); a banded label left to `text ... when:`.
- 2026-10-12 first complete example (`examples/projectile.md`, lab, lesson, video); found and fixed: `on E` read past logged actions at the event's instant, paths joined across jumps; rounding in texts and chosen labels (D-078).
- 2026-10-13 stochastic leftovers: rates that vary between events (D-086), instruments (D-087), `restart` and `reseed` in explore beats (D-088), histograms of one run (D-089), noise in flows by Euler-Maruyama (D-090).
- 2026-10-13 fields on grids of one and two axes (D-091): pointwise evaluation, vectorized where the form allows; compact storage of large states.
- 2026-10-14 enforced constraints (D-093): constraints differentiated along the flows to the rate that reads the holding force, solved with the model's equations, and the state projected onto the constraint after each step and transition; `dot`; residuals judged against the size of their terms; `prismal-present/tests/constraints.rs`.
