# 03 Presentation Kernel

- **Version:** v0 (draft), 2026-09-29, revised 2026-09-30 (with the presentation prototype, `crates/prismal-present`)
- **Part:** presentation kernel (D-011). It reads the model through the model kernel's interface (MK section 17) and the runtime's observer output (RC section 15), and directs runs through execution control (RC section 12).
- **Scope:** how a model is observed, projected into representations, arranged in views, narrated over time, and manipulated by a learner or author. How pixels, audio or video frames are produced is a renderer concern outside this specification.

Every section follows accepted decisions (including D-025 and D-026, raised by this document, and D-033 and D-034, raised by the syntax study) or carried-forward resolutions.

## Contents

1. Overview
2. Presentations
3. Observation and data
4. Expectations
5. Projection
6. Representation
7. View
8. Presentation time and animation
9. Explanation timeline
10. Interaction
11. Accessibility
12. Output and media
13. Check against the first-slice reference programs
14. New positions taken in this document
15. Deferred
16. Static diagnostics

---

## 1. Overview

- **Restates:** R-07, R-19, R-20, R-37, R-38, R-39, R-40, R-42.
- **Decisions:** D-009, D-011, D-017, D-018, D-019.
- **Prior art:** follows the grammar-of-graphics separation of data, encoding and rendering (Wilkinson; Vega-Lite). Follows Manim and Motion Canvas for author-driven animation over a timeline. Follows PhET simulations for learner-facing interactive models with separate model and view code.

The presentation kernel has these concepts:

| Concept | Role | Section |
|---|---|---|
| Presentation | one way of presenting one model; a model may have several | 2 |
| Observation, data | reading model state into information | 3 |
| Expectation | an observation with an expected result | 4 |
| Projection | mapping sources to representations | 5 |
| Representation | what is shown: a marker, arrow, plot, equation, control | 6 |
| View | a region with a coordinate system, camera and layout | 7 |
| Animation | change of presentation properties over presentation time | 8 |
| Explanation timeline | scenes, beats, reveals, camera, narration, and run direction | 9 |
| Interaction | input turned into model, runtime, view or observation actions | 10 |

- **PK-1.1** Presentation state (camera, selection, visibility, animation progress, layout) is never model state (R-20, R-37 invariants 3, 11).
- **PK-1.2** Nothing in the presentation kernel changes the model except interventions (MK-17.2), which pass through the runtime's validation (RC section 11).
- **PK-1.3** A representation does not own scientific truth. Values shown come from committed model state, dense output or data; a representation MAY compute presentation values from them (an arrow length from a velocity) but never feeds such values back into the model (07 sections 66, 111).

---

## 2. Presentations

- **Restates:** R-39.
- **Decisions:** D-011, D-018, D-019.

- **PK-2.1** A **presentation** is a named, serializable unit that refers to one model and contains: observations, expectations, projections, views, a layout of views, interaction permissions, and optionally an explanation timeline.
- **PK-2.2** A model MAY have any number of presentations (D-011). Presentations do not affect each other or the model. The serializable form keeps them separate from the model (D-019).
- **PK-2.3** A presentation refers to model elements only by identity (MK-7.5), through the model's public interface. A presentation that refers to an element the model no longer has is invalid and reports which references are broken.
- **PK-2.4** Presentation configuration (colors, scales, layout, pacing) is distinct from model parameters (R-37 invariant 20). Changing it never changes model results.

---

## 3. Observation and data

- **Restates:** R-19, R-23, R-31, SEM-05 (R-62).
- **Decisions:** D-011, D-015.
- **Prior art:** follows data-logging practice in physics education (Vernier and PASCO loggers: sampled series with units and timestamps). Follows the W3C PROV idea of recording where each datum came from.

### 3.1 Observations

- **PK-3.1** An **observation** reads model state and produces data. It has identity, a **source** (an expression over public bindings, equation residuals, event payloads or structure, MK-17.1), and a **schedule**:

| Schedule | Produces |
|---|---|
| `live` | the current value at the displayed instant |
| `every(Δ)` | a series sampled at simulation instants `t0 + kΔ`, from the dense output |
| `at(τ)` | one value at simulation instant `τ` |
| `on(E)` | one value at each occurrence of model event `E`, read from the committed state at the event's final microstep, or at a named microstep |
| `over(τ1, τ2)` | a series of every committed event instant and dense-output sample in the interval |

- **PK-3.2** Observations never change the model or the trajectory (R-19, RC-15.2). Sampling from the dense output does not add solver steps.
- **PK-3.3** An observation's value may be a status (MK section 5). Data records statuses as statuses, never as numbers.
- **PK-3.4** `on(E)` resolves the model kernel's open item (MK 19.1): a quantity evaluated at an event instant, such as the projectile's range at landing, is an observation `pos.x on(landed)`. An equation's residual at an event is observed the same way.

### 3.2 Data

- **PK-3.5** **Data** is the result of observations: values with their types and units, the simulation instants they refer to (superdense where relevant), and provenance (run identity, observation identity, schedule).
- **PK-3.6** **Analysis** functions (min, max, mean, interpolation, fitting, differentiation of series) take data and produce derived data with provenance. They are pure functions, provided by libraries (05c section 53).
- **PK-3.8** (D-084) An observation at an instant MAY be taken **over runs**: `x at τ over n runs` is the series of the values of `x` at `τ` in `n` runs of the same model and configuration, run `k` (from 1) with the seed of the observed run plus `k - 1`, so the first is the observed run. Each run ends at `τ`. `n` is a whole number from 1 to 10 000. An **ensemble source**, the same phrase as a representation's argument, gives a representation those values.
- **PK-3.7** (D-087) A presentation MAY declare **instruments**, `instrument ruler(resolution: r, range: [a, b], noise: s)`, each option optional and a constant with the dimension of the values read. An observation, a sampled source (PK-6.3a) or an ensemble source (PK-3.8) reads its expression through an instrument when written `ruler(e)`. A **reading** of a value `v` at instant `t`: no reading when `v` is a number or quantity outside `[a, b]`; otherwise `v` plus a normal error of deviation `s` per component (RC-13.5a), rounded per component to a whole number of `r`. Vectors and points take `resolution` and `noise`, not `range`. The instrument's **standard uncertainty** is `sqrt(s^2 + r^2 / 12)`; text alternatives of what it reads state it. An instrument reads at instants: `live` observations and representations that are not sampled cannot use one (PK-E04, SX-E08). A series of readings leaves out readings that are not available; an observation `at τ` without a reading is an error of the expectation that reads it.

---

## 4. Expectations

- **Restates:** R-19.
- **Decisions:** D-012, D-017.
- **Prior art:** follows unit-testing assertions with tolerances (for example `assert_relative_eq` in Rust's `approx` crate). Follows PhET and other inquiry-based lessons, where learners predict an outcome before observing it (08 section 81).

- **PK-4.1** An **expectation** is an observation with an expected value (an expression, possibly over parameters) and a tolerance (absolute, relative, or both). Evaluating it gives `pass`, `fail` (with the observed value and the difference), or the observation's status.
- **PK-4.1a** (D-084) An expectation's subject MAY be a statistic of an observation's values: `mean(obs)`, or `deviation(obs)`, the sample standard deviation (divided by `n - 1`). Booleans count as 0 and 1.
- **PK-4.2** Reference programs (D-012) state their checkable results as expectations. A headless run evaluates them and reports each result.
- **PK-4.3** An expectation MAY take its expected value from the learner (a prediction entered through a control before the run). The same mechanism then serves self-study and assessment (D-017), without the model knowing about either.

---

## 5. Projection

- **Restates:** R-37, R-38, SEM-05, SEM-06 (R-62).
- **Decisions:** D-018, D-020.
- **Prior art:** follows Vega-Lite encodings (a field mapped through a scale to a visual channel). Follows D3's data join for identity-preserving updates of collections.

- **PK-5.1** A **projection** is `Projection(source, context, configuration) -> representation` (R-38). The source is model state (directly, MK-17.1) or data (section 3). The context is the view, its coordinate mapping and the displayed instant. The configuration is presentation configuration (PK-2.4).
- **PK-5.2** Projections are pure: the same source values, context and configuration give the same representation.
- **PK-5.3** A projection MAY be many-to-one (one representation from several sources) or one-to-many (several representations from one source) (SEM-05).
- **PK-5.4** A projection over a collection produces one representation per member. Each representation's identity is derived from its member's identity (MK-7.5), so members that persist keep their representations when the collection changes, and the representation of a destroyed member is removed (R-37 invariant 18 in reverse: removing the source removes its representation; removing a representation never removes its source).
- **PK-5.5** **Encodings** map source values to presentation values: a spatial mapping (model space to view space, section 7), a scale (quantity to length, color or size), or formatting (quantity to text in a display unit, MK-3.7). Encodings are unit-aware: mapping a `Quantity<L/T>` to an arrow length requires a declared scale with that dimension (for example `1 m/s -> 20 px`).
- **PK-5.6** A projection is **invertible** only if it declares an **inverse**: a function from a presentation value (a pointer position, a slider position) to a proposed source value. Invertibility is declared, never inferred, and is independent of whether interaction is permitted (SEM-06, R-37 invariants 9, 10).
- **PK-5.7** An inverse MAY map onto a restricted set, such as a point constrained to a curve through the curve's parameter. This is how constrained dragging is provided while model constraints are checked, not enforced (D-020).

---

## 6. Representation

- **Restates:** R-37, R-59.
- **Decisions:** D-018, D-034.
- **Prior art:** follows SVG and the Manim object model for geometric primitives and grouping. Follows MathML and TeX for equation layout from a symbolic tree.

- **PK-6.1** A **representation** is a presentation element with identity, a kind, properties, and the sources its properties are bound to. It is not pixels (R-37 invariant 2); a renderer turns it into output. A representation MAY carry an author name, unique within its presentation, by which timeline actions and interactions refer to it; the name is not its identity (MK-6.2).
- **PK-6.2** Each representation property is either **bound** (computed by a projection from sources), **set** (a constant in the presentation configuration), or **animated** (driven by presentation time, section 8). A property has exactly one of these at a time.
- **PK-6.3** The **first-slice representation set**:

| Kind | Shows | Typical source |
|---|---|---|
| `marker` | a point (dot, circle, custom glyph) | `Point<S>` |
| `arrow` | a vector drawn from a point | `Vector<S, D>` with an anchor point |
| `segment`, `polyline`, `polygon` | straight geometry | points |
| `circle`, `ellipse`, `arc` | round geometry (PK-6.3c) | a centre point, lengths, angles |
| `trace` | the path of a point over time | observation `over` or `every` of a point |
| `function_graph` | the graph of `f : Real -> Real` or of a quantity function over a domain | function binding |
| `series_plot` | data against an axis (for example `x(t)`) | data |
| `axes`, `grid` | coordinate reference | view coordinate mapping |
| `label` | text, optionally with a value readout | text, any binding |
| `equation` | a typeset model equation, with symbols linked to bindings | equation (MK section 11) |
| `formula` | a typeset expression, a definition, or a labeled expression (`R = ...`), with symbols linked to bindings | expression, derived binding, derived function value (D-034) |
| `table` | rows of values | data |
| `slider`, `number_input`, `toggle`, `select`, `button` | a control | intervenable binding (a `select`'s is of an enumeration, its choices the cases; D-094); for `button`, a requestable event (D-027) or a runtime control |
| `group` | a set of representations with a shared transform | representations |

- **PK-6.3a** A `trace` or `series_plot` takes a **sampled source**, `expr every Δ` with `Δ` a positive duration: the source is evaluated at `t0`, `t0 + Δ`, `t0 + 2Δ`, ... up to the instant shown, and at that instant, so the drawing ends where the other representations are (PK-7.5). A `trace` samples a point of the view's space and draws its path. A `series_plot` samples a number or quantity and draws it against elapsed time in a plot view whose `x` axis is a time range; read through an instrument (PK-3.7), it draws each reading as a separate point, not joined to the others (D-087). A `table` of readings writes `out of range` where an instrument has no reading. A source that is not sampled, an interval that is not a duration, or a series whose dimension differs from the plot's `y` axis is a static error (PK-E04, PK-E05).
- **PK-6.3d** (D-084) A `series_plot` of a value that changes only at transitions (it reads no continuous state, no time and no derivative, directly or through derived values) is drawn as a **staircase**: level between the instants where the value changed, with a vertical step at each, read from the committed states rather than the samples, up to 4000 steps (beyond, from the samples). Other series keep the breaks of D-078.
- **PK-6.3e** (D-084, D-089) A `histogram` takes an ensemble source (PK-3.8), a sampled source (`x every Δ`, the samples of the run up to the instant shown) or the values at each occurrence of an event (`x on E`, those up to the instant shown, each in the state after the occurrence's transition, as an observation `on E`), whose values have the dimension of its plot's `x` axis; its plot's `y` axis is a range of plain numbers. It divides the `x` range into `bins` equal bins (default 20, at most 1000) and draws, as one closed outline, how many values fall in each; with no values yet it draws the axis alone. A histogram of one run grows as the run is shown. When the values lie on a grid as coarse as the bins or coarser (D-087: the same width included, as readings rounded to an instrument's resolution) (counts, shares of a few molecules: every value within a millionth of a step of `start + k d`, with `d` the smallest gap between distinct values), each grid value gets its own bar, centred on it and `d` wide, so that no empty bins fall between them (D-085); values outside the range are counted in its text alternative, which also states the mean and the deviation. The values are computed again only when the observed run's configuration changes (parameters, interventions, seed).
- **PK-6.3b** (D-043) A `group` has a placement `at` (a point of its view's space), a rotation `rotate` (an angle) and a scale `scale` (a positive number), and members written in its own frame. Before projection, a member's points `p` become `at + scale R(rotate) (p - origin)` and its vectors `v` become `scale R(rotate) v`; a group inside a group is placed in its parent's frame. A group belongs in a spatial view and holds markers, arrows, segments, polylines, polygons, circles, ellipses, arcs and groups; any other member, a group outside a spatial view, or a `rotate` or `scale` that is not a number is a static error (PK-E04, PK-E05). A group's members may be named and are targets of timeline actions; showing, hiding or highlighting a group applies to all its members. Its text alternative lists its members' (PK-11.1). A member may declare an inverse (D-062): the gesture's value is the pointer's point in the group's frame, `origin + R(-rotate) (q - at) / scale` for the pointer at `q` in the view's space, composed through nested groups and evaluated at the instant shown, so a proposal reads the pointer in the frame the member is written in. Targeting, keyboard focus and keyboard steps reach members of groups as they reach other representations (PK-10.2a, PK-11.2).
- **PK-6.3c** (D-055, D-057) `for b in c { reps }` in a view repeats each representation for each member of `c`, named `name[k]`. For a collection whose membership changes, a member's representation is drawn only while the member is alive, and its text alternative is omitted otherwise; a trace of a member starts when it is made and ends when it is destroyed. Relation sets are repeated the same way (`for s in springs { segment(s.a.pos, s.b.pos) }`, D-058).
- **PK-6.3c** (D-054) `circle(P, r)`, `ellipse(P, a, b, rotate: θ)` and `arc(P, r, from: θ1, to: θ2)` belong in a spatial view. `P` is a point of the view's space, radii are lengths in the model's units (so the shape scales with the view, cameras and groups, unlike a `marker`), and angles are measured counterclockwise from the space's `x` axis, as a group's `rotate` is. A radius that is not a length, an angle that is not an angle, an arc without both ends, or a property the kind does not take is a static error (PK-E04, PK-E05); a negative radius at an instant is a status. A frame carries each as an exact elliptical arc in view coordinates (centre, radii, rotation, start, sweep, closed), with the view's orientation and any group's transform applied, so a renderer needs no model geometry; `reveal draw` draws it along its perimeter. Text alternatives give the centre, radii and angles.
- **PK-6.3d** (D-069) A `button` requests a model event declared `on request`, or is a runtime control of an interactive session: `button(reset)`, `button(undo)`, `button(redo)` (RC section 12), `button(reseed)` (D-084), `button(restart)` (D-085). `reset` goes back to the presentation's starting values with an empty log; `restart` starts a new run from `t0` with every parameter at its value at the instant shown and an empty log; `reseed` is a restart with the next seed. An event of the same name takes precedence. Playing, pausing and seeking are the host's clock and have no button. A runtime-control button in a presentation with a timeline is a static error (PK-E05), except `restart` and `reseed` among an explore beat's controls (PK-9.8c).
- **PK-6.3f** (D-091) A field on a line is drawn by `function_graph(T)` in a plot view whose axes have the dimensions of the grid and of the field's values: a line through the edge values and the cell values. A field in the plane is drawn by `heatmap(T [, range: [a, b]])` in a spatial view: each cell a square coloured by its value, the frame's `raster` shape holding the values scaled to `[0, 1]` over `range` (or the field's own extent, made even about 0 when it spans 0) and a palette name, `diverging` when the range spans 0, else `sequential`; each medium maps the palette to its colours. Its text alternative states the grid, the range of values and where the highest value lies.
- **PK-6.3e** (D-076) `text("... {e} ..." [, at: P])` shows an author's text, each expression in braces shown with its unit (a binding in its display unit); `at` places it in a spatial or plot view. `title("...")` in a view or panel is its caption; `title "..."` names the presentation. Any representation may carry `when: c`, shown only while `c` holds at the instant shown.
- **PK-6.3f** (D-077) A slider may name its ends, `ends: ("cold", "hot")`, shown beside it and read in its text alternative. A label of a binding shows the binding in its display unit (MK-3.12), as controls do.
- **PK-6.4** Controls are representations of bindings with a declared inverse (PK-5.6): a slider shows a parameter's value and, when moved, proposes a new one. A control can only target an intervenable binding (D-023).
- **PK-6.5** `equation` and `formula` representations are typeset from symbolic forms in the IR (MK-10.6), never from strings. A `formula` of a derived binding or derived function value shows its definition (`f(x) = a x²`); a `formula` with a label shows `label = expression`, where the label is presentation text and not a binding (D-034). Each symbol that refers to a binding carries that binding's identity, so views can highlight a symbol with the value it stands for, and a label can show live values substituted into the equation.
- **PK-6.5a** (D-046) A formula or equation is typeset for every medium. Its **math box tree** (rows, identifiers, numbers, operators, fractions, powers, subscripts, roots, fences, cases) is built from the IR with the precedences and display symbols of the text form; its **layout** places the tree as text runs, rules and stroked paths in em units of the formula's font size, `y` down from the baseline, each text run with the width it was given and, for a symbol, its binding's identity. A medium with its own math engine MAY typeset the box tree instead (MathML in a browser); any other medium draws the layout, which needs only text and lines.
- **PK-6.6** A representation may exist without a model source (a title, an annotation) (R-37 invariant 19). A line between two objects is not a relation unless the model declares one (R-59).
- **PK-6.6a** (D-061) **Author styles** are set properties (PK-6.2) of presentation configuration (PK-2.4): `color:` names a color from a fixed palette (`red`, `orange`, `yellow`, `green`, `teal`, `blue`, `purple`, `pink`, `gray`, `ink`) and `line:` a stroke (`solid`, `dashed`, `dotted`). A marker takes a color; arrows, segments, polylines, polygons, circles, ellipses, arcs, traces, function graphs and series plots take both; any other kind, a group, or a word outside the lists is a static error (PK-E05). Frames carry the names, never values: each medium maps a name to a value that suits its theme (PK-12.2), so a program reads on light and dark pages and in video. A style is decoration: it is not in the text alternative, and a color never carries meaning alone (PK-11.4). Without a style each kind is drawn its own way. Styles bound to model values (a color scale, PK-5.5) are later work, with one exception: `color:` may be a length, read as a wavelength of light (D-106), evaluated on the state shown; the frame then carries the colour of that light as `#rrggbb`, from a fixed fit to the visible spectrum (380 nm to 750 nm, dimmer toward its ends; gray outside it), the same in every medium and theme.
- **PK-6.3g** (D-106) `ray(P, d [, back: s])` in a spatial view is a half-line from point `P` along vector `d` (of any dimension; its direction only). The frame carries its start and its direction in view coordinates, the projection of a step along `d`; media draw it to the edge of what they show, so it does not enter a view's extent. `back: solid|dashed|dotted` draws its extension behind `P` in that line style, the back-extension of a ray toward a virtual image. A `polyline` whose points stop being available after its second is drawn up to its last available point (MK-5.6a); with fewer, it is not available.
- **PK-6.7** Libraries MAY add representation kinds. A new kind declares its properties, the source types it accepts, and a description of how a renderer draws it in each supported medium (section 12).

---

## 7. View

- **Restates:** R-07, R-37.
- **Decisions:** D-018, D-022.
- **Prior art:** follows camera and viewport separation in graphics (world, view and screen space) and responsive layout practice on the web.

- **PK-7.1** A **view** is a region of the presentation with its own coordinate system, a set of representations, and presentation state: camera, visibility, selection, and zoom.
- **PK-7.2** A **spatial view** shows one model space (D-022). Its **coordinate mapping** takes model coordinates in a chosen frame of that space to view coordinates: origin, scale (for example `1 m -> 100 px`), rotation and orientation (for example `y` up). Camera movement changes the mapping, never model values (R-07: model space is not render space).
- **PK-7.2a** **Views of three dimensions** (D-100). A spatial view of a space of three dimensions looks from an **azimuth** `a` (around the `z` axis, from the side of `x` toward the side of `y`) and an **elevation** `e` (from the `x y` plane toward `z`), `azimuth: a, elevation: e`, by default 30 deg and 20 deg; `y: up` and `y: down` are errors there (SX-E08), and `azimuth` and `elevation` in a view of a plane. The projection is orthographic along the line of sight: a point `p` is drawn at `(p · r, -(p · u))` times the scale, with `r = (-sin a, cos a, 0)` the view's right and `u = (-sin e cos a, -sin e sin a, cos e)` its up, so `z` is up on the screen. A vector of an arrow is projected the same way. A circle is a sphere's outline, a disc of its radius facing the viewer. `axes` are three axis lines through the origin, each 160 px long on either side, labelled at their positive ends; `grid` is the `x y` plane at `z = 0` in steps of a round length near 60 px; both are projected lines in the frame (shape `lines`), drawn by every medium alike. Heatmaps lie in a plane and are PK-E05 in such a view. A pointer position in such a view is the point of the plane through the origin that faces the viewer.
- **PK-7.2c** **Views drawn in a frame** (D-105). A spatial view may take `frame: F`, a declared frame of the model (MK-4.6a): every point is drawn at its coordinates in `F` and every vector by its components in `F`, on the state shown, then projected as in PK-7.2 and PK-7.2a. A trace reads each sample in `F` as `F` was at that sample's instant, so that a path at rest in the standard frame is drawn as it moves through `F`. The view's `axes` and `grid` are `F`'s. A pointer position is mapped back from `F` to the model's coordinates. A frame that is not available on a state is drawn where it last was.
- **PK-7.2b** **Perspective, turns and depth** (D-104). In a view of three dimensions:
  - `azimuth` and `elevation` are angle expressions that may read the model's values and time, evaluated at each frame. `distance: d`, a positive length, places the viewer at `d t` (`t` the unit vector toward the viewer) and draws the view in perspective: a point `p` is enlarged by `d / (d - p · t)`, so the plane through the origin facing the viewer keeps the view's scale. Without `distance` the projection is orthographic (PK-7.2a). A sphere's outline is enlarged as its centre is. `distance` in a view of a plane is SX-E08.
  - The view's look on a frame is, in order: the learner's turn (`permit learner { orbit }`: dragging an empty point turns the view by half a degree per pixel, elevation held between -90 deg and 90 deg; a view reset undoes it), else a timeline camera's turn (`camera v azimuth a elevation e for d`, from where the previous turn left it, eased as other camera moves), else the author's angles. Each frame of such a view carries its azimuth and elevation (`look`).
  - Circles, ellipses and arcs take `normal: n`: the shape lies in the plane through its centre at right angles to `n`; its first axis is the group's `x` laid into that plane, turned by `rotate`, its second the normal crossed with the first, and arcs' angles run from the first toward the second. An ellipse or an arc without a normal lies in the group's floor; a circle without one is a sphere's outline (PK-7.2a). Such shapes are drawn as polygons (closed) or polylines through points along them. `normal` in a view of a plane is PK-E05.
  - Groups turn their members by `rotate` about `axis: n`, a vector of the space, `z` by default; nested groups compose their rotations. `axis` in a view of a plane is PK-E05.
  - Circles, ellipses and polygons take `opaque: true`: a body that hides what lies behind it from the viewer. Frames of such a view draw closed shapes first, the farthest (by the depth of their centre or of the mean of their points) first; then the other representations in the order written. The pieces of segments, polylines, traces, axes and grids that an opaque body hides are cut away (a cut segment becomes a polyline with breaks); a marker, or an arrow whose tip, is hidden is not drawn. Texts are always drawn. `opaque` on another kind is SX-E08; in a view of a plane it fills the shape without showing what is under it.
  - A drag moves in a plane through the point the representation had when the drag began (a marker's position, an arrow's tip when dragged by its head, a shape's first point, a sphere's centre): the plane facing the viewer, or with `on drag across n as p` the plane of normal `n`. The gesture's value is where the line of sight through the pointer meets that plane, in the frame of the groups around the representation; a line of sight along the plane proposes nothing. `across` in a view of a plane is PK-E05.
- **PK-7.3** A **plot view** has axes, each bound to a dimension and a display unit, with a range that is fixed, follows the data, or is controlled by the camera.
- **PK-7.3b** (D-101) Segments, polylines and polygons in a plot view take pairs of plot coordinates, as markers do (PK-7.3a): `segment((k, 0), (k, v))` draws a bar of a bar chart.
- **PK-7.3a** In v0 a plot axis range is fixed: two numbers or two quantities of one dimension (`x: [0 s, 5 s]`, `y: [0 m, 1.1 m]`), which gives the axis its dimension; it is shown in coherent SI units. A marker in a plot view is at a pair whose components have the axes' dimensions (`(0, y)` with `y` a length on a length axis), and a drag in a plot view gives a pair of quantities in those dimensions (PK-5.6).
- **PK-7.3b** An axis range follows the data when the view says so (`follow: y`, `follow: (x, y)`): it is the declared range grown to hold everything the view draws, never smaller than declared. An axis can have a display unit (`x_unit: ms`, `y_unit: cm`) that measures its dimension; ticks are labelled in it. `window: w` makes the `x` axis show the latest span of length `w` once the data passes the declared range. The learner's zoom and pan (D-047), with the wheel, a drag or two touches, apply to plot views as to spatial views, and function graphs are drawn over the range shown (D-070).
- **PK-7.4** **Layout** places views in the presentation and adapts them to the output's size and orientation. Layout never changes what a view shows, only where and how large.
- **PK-7.4a** (D-063) A presentation's **page layout** is a tree: a view, `row(...)` (its items side by side) or `column(...)` (one below the other), each with at least one item, nested freely. Every name is a view or panel of the presentation, placed at most once; views the layout leaves out follow it, one below the other, in the order they are declared. Sizes remain the views' own; a medium adapts the tree to its page (a browser MAY turn a row into a column on a narrow screen) but keeps its order. Without a layout, a medium arranges the views in declaration order as it sees fit.
- **PK-7.5** **Consistency across views**: every view in one presented frame shows the same simulation instant (and the same microstep when one is selected), and the same committed state or dense-output value. A view never shows a newer state than another view of the same frame (07 section 96).

---

## 8. Presentation time and animation

- **Restates:** R-23, R-31, R-32.
- **Decisions:** D-009, D-015.
- **Prior art:** follows Manim and Motion Canvas (animations as functions of presentation time with easing curves) and the Web Animations model (timing, easing, fill).

- **PK-8.1** **Presentation time** is the clock of a playback or export (RC-2.1). Every presented frame is taken at a presentation instant.
- **PK-8.2** The **time mapping** relates presentation time to simulation time: at each presentation instant the presentation shows one simulation instant. The mapping is piecewise: playing at a rate (simulation seconds per presentation second, `1` for real time), holding (the simulation paused while presentation time runs, for narration), or jumping (a seek). Rates are presentation configuration and never change results (RC-2.3).
- **PK-8.2a** A lesson starts holding at `t0`. A `run` or `hold` sets the mapping until the next `run`, `hold`, `seek` or `reset`, across beat boundaries: after `run rate 1 until landed` the simulation keeps running into the next beat unless that beat holds.
- **PK-8.3** Between committed instants, the displayed state is the runtime's dense output at the mapped simulation instant (RC-15.1). A presentation MUST NOT add solver steps to get smoother frames (RC-6.6).
- **PK-8.4** An **animation** changes an animated property (PK-6.2) over an interval of presentation time: from a value, to a value, with a duration and an easing function. Typical uses: reveal (fade, draw), transform (move, morph one shape into another), emphasis (highlight, pulse), camera moves.
- **PK-8.4a** (D-075) Animated properties in v0: `opacity`, `offset` (D-068), `color` (blended between palette names, mixed by each medium), `line` (switched halfway), `scale` (geometry about the shape's centre; a marker's dot and an arrow's head grow), `morph` (the drawn shape moved to another representation's, resampled when their forms differ). Easing functions: `linear`, `smooth` (the default), `in`, `out`. A camera moves a plot view's axes as it moves a spatial view, centred on a pair of axis values.
- **PK-8.5** An animation cannot drive a bound property. To animate the appearance of a model-bound element, the presentation animates a separate property (opacity, style, a presentation offset) or temporarily replaces the binding by an explicit handover (section 9, `release` and `bind`).
- **PK-8.6** A presentation without a model clock (a static model, or a pure animation with no model) uses presentation time alone (R-32).
- **PK-8.7** Given the model, run configuration, presentation, and intervention log, presented frames are deterministic: the same presentation instant shows the same content. Video export relies on this (D-015, D-018).

---

## 9. Explanation timeline

- **Restates:** R-22, R-23, R-32.
- **Decisions:** D-009, D-017, D-018, D-025, D-027, D-033.
- **Prior art:** follows Manim scenes and Motion Canvas generator timelines (author-sequenced animation with waits). Follows interactive-video and slide tools for learner-paced continue points. Departs from both by synchronizing with a live simulation through events rather than fixed times.

The explanation timeline is a peer of the model (D-009): it observes the model and directs its runs; the model never depends on it.

### 9.1 Structure

- **PK-9.1** A timeline is a sequence of **scenes**. A scene is a sequence of **beats**. A beat is a set of **actions** that start together, and it ends when all of its actions have finished. Actions MAY be grouped as a sequence within a beat.
- **PK-9.2** Action kinds:

| Action | Effect |
|---|---|
| `show`, `hide`, `reveal(style)` | make representations visible, with an optional animation |
| `animate` | an animation of presentation properties (PK-8.4) |
| `camera` | move a view's camera (an animation of its mapping) |
| `highlight` | emphasize representations, equation symbols or bindings |
| `narrate(text)` | a narration cue: caption text and, optionally, recorded or synthesized audio; ends when the audio or the reading time ends |
| `run(rate)`, `hold` | set the time mapping (PK-8.2): play the simulation at a rate, or pause it |
| `seek(τ)`, `reset`, `restart`, `branch` | execution control on the lesson's run (RC section 12); `reset` shows the run again from `t0`, `restart` makes a new run from `t0` with the parameter values of the instant shown (D-085) |
| `intervene(ops)` | an intervention on intervenable bindings (MK-17.2), such as setting the launch angle for the next demonstration |
| `request(E)` | request a requestable model event (MK-17.2a, D-027), such as relaunching the projectile |
| `wait(d)` | wait a presentation duration |
| `wait_until(E)` | wait until model event `E` occurs on the lesson's run |
| `wait_until(t = τ)` | wait until the simulation reaches instant `τ` |
| `wait_for_learner` | wait for the learner to continue (a continue point) |
| `explore` | open a learner exploration period (section 9.3) |
| `bind`, `release` | hand a representation property between a projection and an animation (PK-8.5) |

- **PK-9.2a** **Order at a beat's start** (D-033). A beat's run-directing actions (`seek`, `reset`, `branch`, `intervene`, `request`, `run`, `hold`) take effect in the order written, at the beat's start and before any presentation time passes. The beat's other actions then start together and see the resulting state. `sequence` orders actions that take time; it is not needed for run-directing actions.
- **PK-9.2b** A `narrate` without a duration lasts its reading time: 0.4 s per word, at least 2 s. A `highlight` lasts until the end of its beat. Representations a beat shows stay shown to the end of the lesson; the controls of an `explore` beat are shown only during it.
- **PK-9.2c** (D-042) In v0 animations are named effects. `reveal fade` raises a representation's opacity from 0 to 1 and `reveal draw` draws its lines from start to end (points and text fade); `hide ... for d` lowers opacity to 0 and then stops showing it; `camera` moves a spatial view's centre to a point, evaluated at every frame, and its zoom. Each lasts its duration (1 s by default for `reveal` and `camera`, none for `hide` without `for`) with the easing `3k² - 2k³`, and a move starts from where the previous one left the camera. They change presentation properties only (PK-8.5): the model and its runs are unaffected, and frames stay deterministic (PK-8.7).
- **PK-9.2d** (D-053) Narration sound is not part of a program: `narrate` carries text, and its duration is its `for d` or its reading time (PK-9.2b), never the length of a recording. Hosts may voice each caption cue, identified by a name the engine gives it (its beat's name, then `beat.2`, `beat.3` ... for later narrations in the same beat), with a recording named after the cue or with synthesized speech. A recording longer than its cue is reported.
- **PK-9.2e** (D-067) A continue point is written `wait learner [limit d] [fallback { ... }]`. It opens when its beat's other actions have ended (inside a `sequence`, at its place) and ends at the learner's first `continue` at or after it opens, else at `limit`. With neither it is **pending**: it takes no time, and a player stops at it until the learner continues, which gives the same playback as continuing then. An `explore` beat without a limit is pending in the same way until the learner continues.
- **PK-9.2f** (D-068) `animate R opacity to x [for d]` and `animate R offset to v [for d]` animate the separate properties of PK-8.5: an opacity from 0 to 1, multiplying any reveal or hide, and an offset, a vector of the space of R's spatial view evaluated when the animation starts, moving R as drawn. Each starts from the value the previous animation of that property left and lasts `d` (1 s by default) with the easing of PK-9.2c; the last value holds. `release R` holds R's bound geometry and text at the instant shown; `bind R [for d]` hands R back to its projection, blending from the held geometry to the live one over `d` when both have the same form. The model and its runs are unaffected.
- **PK-9.3** `wait_until(E)` with the simulation running makes presentation duration depend on the model: the beat lasts exactly as long as the simulation takes to reach `E` at the current rate. Because runs are deterministic (RC-14.5), this duration is known before playback and is identical in export.
- **PK-9.3a** A beat ended by `wait_until(E)` shows, at its end, exactly the simulation instant at which `E` was located, not the value the time mapping gives by arithmetic (which may round to just before it). Actions of the next beat (a branch, a request) therefore apply at `E`'s instant, after `E`.
- **PK-9.4** If a `wait_until` can never be satisfied (the run ends or stops first), the timeline reports it at the moment the run ends, continues with the next beat, and records a diagnostic. It never hangs.
- **PK-9.4a** A `wait_until` while the simulation holds cannot be satisfied: it is reported at once and takes no time.

### 9.2 The lesson's run

- **PK-9.5** A timeline directs one **lesson run** of its model. Timeline actions act on that run. The lesson run's configuration is part of the presentation.
- **PK-9.5a** A lesson run whose configuration gives no end is computed to `t0 + 60 s`. An intervention or request makes a new version of the run, computed with the action in its log (RC-11.1), continuing the previous version from a snapshot where RC-14.3a allows; the time mapping keeps referring to the version it showed, so earlier frames never change.
- **PK-9.6** The timeline reads the model only through observation and events (MK-17.1) and changes it only through interventions (MK-17.2). The model runs unchanged with the timeline removed (D-009, MK-17.5).

### 9.3 Learner control

- **PK-9.7** During a narrated lesson, the learner always has runtime controls over the **timeline**: pause, resume, replay a beat or scene, change narration speed. Narration speed is the speed of the whole lesson: the voice and the timeline it narrates change together, so a narration stays with its beat and captions (D-095); the reference player offers 0.5x to 2x. View actions (zoom, pan) are available where the presentation permits them. These never change the lesson run's results.
- **PK-9.8** Model actions by the learner (interventions) are allowed only inside `explore` beats. An `explore` beat:
  - branches the lesson run at the current instant (RC-12.3); the learner's interventions and run controls act on the branch;
  - declares which controls and intervenable bindings are available, a subset of the presentation's permissions;
  - ends when the learner continues, or after a declared time limit.
- **PK-9.8a** A learner's model action outside an `explore` beat, or through a control the beat does not declare, is refused by the presentation and recorded; it never reaches the model. A learner script's inputs are processed in time order.
- **PK-9.8c** (D-088) An explore beat MAY offer `button(restart)` and `button(reseed)`. Pressed at instant `p`, either makes a new version of the learner's branch from `t0`, with every parameter at its value on the branch at the instant shown and an empty log (and, for `reseed`, the next seed), and shows it from `t0` at `p`. A learner script writes the press `at τ: press restart` (or `reseed`), unless the model has an event of that name. The beat still ends as PK-9.9 says: without `keep`, the lesson returns to its own run at the instant it branched.
- **PK-9.8b** (D-069) A `button` among an explore beat's controls requests its event on the learner's branch at the instant pressed. A learner script writes the press `at τ: press E`. A press outside an explore beat, or of an event no button of the beat requests, is refused with a reason.
- **PK-9.9** When an `explore` beat ends, the timeline returns to the lesson run at the instant where the branch was made, unless the beat declares `keep`, in which case the lesson continues on the learner's branch. A `keep` beat declares which bindings the learner sets; later narration can refer to them (for example "you chose `θ = 50°`").
- **PK-9.10** In linear media (video), `explore` beats and `wait_for_learner` are replaced by their declared fallback: a fixed-duration pause, a scripted intervention demonstrating the exploration, or omission (section 12).

### 9.4 Education layer boundary

- **PK-9.10a** (D-067) A continue point without a fallback is omitted in linear media: it has nothing to show, so it is not an unsupported element (PK-12.3).
- **PK-9.11** Questions, answers, feedback and scoring are the education layer above the core (R-22). They are deferred; the timeline provides what they need: continue points, explore beats, expectations with learner predictions (PK-4.3), and observation of learner interventions.

---

## 10. Interaction

- **Restates:** R-33, R-40, R-41, SEM-07, SEM-08 (R-62).
- **Decisions:** D-020, D-023.
- **Prior art:** follows the model-view-controller separation of input handling from model state, the command pattern for undo (Gamma and others), and GeoGebra and PhET for dragging model elements with validation.

### 10.1 The path from input to action

- **PK-10.1a** (D-091) A presentation MAY say how long its run lasts, `until τ`, the form a case uses. Without it a host chooses (HI-5.1).
- **PK-10.1** **Input** (pointer, touch, keyboard, device) is not interaction (R-40). An **interaction** interprets input under a mode or tool, resolves a **target**, and produces an **action** in one of four destinations (08 section 5, R-33):

| Destination | Action | Passes through |
|---|---|---|
| model | intervention | runtime validation (RC section 11) |
| runtime | execution control | RC section 12 |
| view | camera, selection, visibility, layout | presentation state only |
| observation | probe, measure, read out | observation (section 3) |

- **PK-10.2** **Targeting** resolves input to a representation (hit testing in view coordinates), then to the representation's source through its binding. A model action names its semantic target, never a visual one (R-40 invariant 5).
- **PK-10.2a** Targeting is done by the kernel from the frame description, the same for every medium (D-047): a pointer targets the topmost visible draggable or clickable (PK-10.5b) representation whose grabbed part lies within its drawn radius (a marker 8 px, an arrow's head 7 px, a stroked body 3 px) plus a reach that depends on the pointer (mouse 4 px, pen 8 px, touch 16 px). Draggable representations are drawn over the others, later ones on top. A press that targets nothing may pan the view, if the presentation permits it.
- **PK-10.3** When a target is ambiguous (overlapping representations), resolution uses a declared order (topmost, then nearest), and the interaction MAY offer a choice. It never picks by internal order.

### 10.2 Permissions

- **PK-10.4** A presentation declares which interactions it offers: which bindings are controllable and through which representations, which runtime controls appear, which view actions are allowed. A presentation can offer less than the model permits, never more: model actions target only intervenable bindings (D-023).

### 10.3 Direct manipulation

- **PK-10.5** Dragging a representation is a **transaction**: begin, preview, commit or cancel (R-41). During preview, the representation's inverse (PK-5.6) turns each pointer position into a **proposed** value (SEM-08).
- **PK-10.5a** (D-059) In a representation repeated per member of a collection (`for b in balls { ... }`), an inverse may propose a binding of that member: `on drag as p { propose b.pos = p }`. The binding must be intervenable in the object type (D-023). A representation of a member that is not alive is not drawn and cannot be dragged; a proposal after which the member dragged is not alive at the instant shown is invalid. A member chosen during the run, as the one at a relation's endpoint (`propose s.a.pos`), is not proposed through the relation (MK-E26): the member is dragged by its own representation.
- **PK-10.5b** (D-059) A representation may request an event when **clicked**: `on click request E(v)`, typically with its member as payload, `on click request remove(b)`. A click only requests an event (D-027), never sets a value; the event must be declared `on request` and the payload is the one it declares (PK-E03, PK-E02). A click is a press and release within the pointer's reach (PK-10.2a) of the press; moving further makes it a drag of a draggable representation, or nothing. Frames carry the event a click requests, and the text alternative says so (`ball[2] ..., activate: remove`). A clickable representation takes keyboard focus, and Enter or space activates it (D-026).
- **PK-10.5c** (D-060) A spatial or plot view may request an event when an **empty point** of it is clicked: `on click as q request E(q)`, written in the view. `q` is the point clicked, in the view's space (a pair of axis values in a plot view), read as a drag's gesture value is (PK-10.5); the payload may compute from it (`E(q + (0 m, 1 m))`). As in PK-10.5b the event must be declared `on request` with the payload's type (PK-E03, PK-E02), and a click only requests. A click on a representation that takes it (PK-10.2a) is that representation's; only a press that targets nothing clicks the view. When the presentation permits `pan`, a press that moves past the pointer's reach pans and one that does not clicks. A panel has no points and takes no such click. Frames carry the event a view's click requests. The view's click has no keyboard operation of its own: a point is chosen with the pointer, so a lab that relies on it offers the same event through a control or button as well (D-026).
- **PK-10.6** Proposed values are validated against the model's constraints as they arrive (MK section 12). A proposed value that a `reject` constraint would reject is shown as invalid (the drag stops at the boundary or is marked), and is never committed.
- **PK-10.7** Preview shows the effect of the proposed value on derived bindings, computed from the proposed state without committing it. Preview state is presentation state.
- **PK-10.8** On release, the last valid proposed value is committed as one intervention (RC-11.1). On cancel, nothing is committed. Only commits are logged (RC-11.5).
- **PK-10.9** Default drag mode for a running simulation is `hold`: the lesson or exploration run pauses at the current instant when the drag begins, and resumes after the commit. A presentation MAY choose `live` mode, where each proposed value is committed as a separate intervention while the run advances (for steering). `live` commits are logged individually.
- **PK-10.9a** A representation chooses `live` in its inverse: `on drag live as p { ... }`. While the pointer is held, the proposal pending at the instant shown is committed when the clock moves on, and the last pointer position is proposed again at the new instant; moves at one instant replace each other (D-071).

### 10.4 Undo and redo

- **PK-10.10** Undo of an intervention on a run is a seek to the intervention's instant with the intervention removed from the log (RC-11.2): the trajectory after that instant is recomputed. Redo restores it. For a static model this is the same as returning to the previous committed state.
- **PK-10.11** View actions have their own undo history in presentation state and do not enter the run's log.

---

## 11. Accessibility

- **Restates:** R-37 (non-visual representations, 07 sections 70-71).
- **Decisions:** D-017, D-018, D-026.
- **Prior art:** follows WCAG 2.2 (text alternatives, keyboard operability, not relying on color alone). Follows PhET's accessible simulations, which describe a simulation's state in text for screen readers and let every interaction be done by keyboard.

- **PK-11.1** Every representation that conveys model information has a **text alternative**, generated by default from its sources: the binding's label, value and display unit, or for a group, a summary of its members. Authors MAY override it.
- **PK-11.2** Every model action and runtime control a presentation offers is available without a pointer (keyboard or equivalent), including dragging: a draggable marker is movable by steps along its inverse (PK-5.6).
- **PK-11.2a** A keyboard step moves a draggable representation by its `step` property, in view units; without one, by 1/100 of each axis span in a plot view, or 10 px in a spatial view. A slider without `step` moves by 1/100 of its range. Each step is one committed intervention, as a pointer drag.
- **PK-11.2b** Keyboard focus moves through the representations that take it, in order: in each view in turn, the draggable and clickable ones, controls and buttons in the order the presentation declares them, then those over the presentation (D-047). A representation grabbed by a pointer takes focus, so that the keyboard continues where the pointer left off.
- **PK-11.3** Narration has captions (text is required; audio is optional). Timeline beats and event occurrences can be announced to assistive technology.
- **PK-11.3a** Playback announces every model event occurrence it passes while the simulation runs, at the presentation instant it is shown, and every occurrence an action causes at a held instant (a requested event). Linear media carry the announcements as a **descriptions track**: a cue at the first frame that shows the occurrences, naming them, until the next cue and for at most 2 s (WebVTT beside a video).
- **PK-11.4** Color is never the only encoding of a value: a color encoding is paired with a value readout on demand, a legend, or another channel.

---

## 12. Output and media

- **Restates:** R-37 (invariants 2, 14), R-39.
- **Decisions:** D-018.
- **Prior art:** follows the display-list separation between scene description and renderer (SVG, PDF, Vello scenes), and export pipelines that render a deterministic timeline frame by frame (Manim, Motion Canvas).

- **PK-12.1** The presentation produces, for each presentation instant, a **frame description**: the visible representations with their resolved properties, in view coordinates, plus captions, text alternatives and pending announcements. It is medium-independent.
- **PK-12.1a** In the prototype's frame description, view coordinates are pixels with the model origin at `(0, 0)` and `y` down on screen in a spatial view (a view with `y: up` negates model `y`), and the plot's own coordinates in a plot view. A formula carries its symbolic form from the IR, its layout (PK-6.5a) and, for each symbol, the binding's identity and, when live, its value (PK-6.5). A representation carries its label (markers, arrows, graphs) and, when it declares an inverse, the part it is dragged by; a control carries its binding's display symbol and display unit. Frame descriptions contain nothing specific to one medium.
- **PK-12.2** A **renderer** turns frame descriptions into a medium: interactive web player (first target, D-018), later still image, video, vector document, data export. A renderer MUST NOT define representation semantics (R-37 invariant 14).
- **PK-12.3** Each medium declares what it supports: interaction, audio, time. A presentation element that a medium cannot support uses its declared fallback (PK-9.10); an element with no fallback is reported, not silently dropped.
- **PK-12.4** Which forms and media a piece of content uses follows the content and the author's intent (D-018, R-39); the specification makes every medium one target among several.

---

## 13. Check against the first-slice reference programs

Each program (D-012) is checked for expressibility in the presentation kernel. Examples are in the working syntax (D-028), non-binding; the complete programs are in `reference-programs/`.

### 13.1 Projectile, with and without drag

```text
presentation ProjectileLab for Projectile {
  view scene: spatial(Plane, scale: 1 m -> 10 px, y: up) {
    marker(pos)
    arrow(vel, from: pos, scale: 1 m/s -> 4 px)
    trace(pos every 0.02 s)
  }
  view graph: plot(x: elapsed, y: pos.y) {
    series_plot(pos.y every 0.02 s)
  }
  observe { range = pos.x on landed }
}

run no_drag of Projectile with ProjectileLab {
  param  { k = 0 }
  expect { range == speed^2 * sin(2 * angle) / g within rel 1e-6 }
}
```

- The range at landing is an `on(landed)` observation (PK-3.4). The model kernel's gap (MK 19.1) is closed.
- The reference result is an expectation (PK-4.2). The no-drag result holds only when `k = 0`, so it belongs to a named case with that override, not to a conditional expectation (section 15).

### 13.2 Bouncing ball, pendulum, spring-mass

Marker, trace and series plot as in 13.1; energy as a `series_plot` of the derived `energy` binding. Expectations: bounce times `on(bounce)` against the geometric series; period from successive `on(...)` observations of a crossing event (for example an upward crossing of `θ`); energy drift as `max - min` of the `energy` series (an analysis, PK-3.6). Expressible.

### 13.3 Function plot with a draggable parameter

```text
presentation QuadraticPlot for QuadraticDemo {
  view plot: plot(x: [-3, 3], y: [-5, 20]) {
    function_graph(f)
    marker(at: (1, f(1))) {
      on drag as p { propose a = p.y / 1^2 }      // x fixed at 1
    }
  }
  panel controls {
    slider(a, range: [-5, 5], step: 0.1)
    formula(f, live: true)                        // f(x) = a x², D-034
  }
}
```

- The slider and the draggable point both target `a`, the only intervenable binding (D-023). The point's inverse maps a pointer position to a proposed `a` (PK-5.6).
- The constraint `-5 <= a <= 5` (policy `reject`) stops the drag at the boundary (PK-10.6).
- All views show the same committed state (PK-7.5); the formula shows the live value of `a` (PK-6.5). Expressible.

### 13.4 Vector addition

Arrows for `u`, `w`, `sum`; draggable arrow heads with inverse `head -> vector = head - tail`, only if the model marks `u` and `w` intervenable. Invalid operations are model-kernel static errors (MK-E03, MK-E04). Expressible.

### 13.5 Narrated projectile lesson

```text
timeline {
  scene launch {
    beat { in scene { marker(pos) as ball }; narrate "A ball is launched at 45 degrees." for 4 s }
    beat { run rate 1 until landed }
    beat { hold; highlight ball; narrate "It lands here. Why this distance?" for 3 s }
    beat {
      seek t0                                      // applies before the formula appears (PK-9.2a)
      show formula("R", speed^2 * sin(2 * angle) / g, live: true)
      narrate "Watch the horizontal speed." for 3 s
    }
    beat { in scene { arrow((vel.x, 0), from: pos, scale: 1 m/s -> 2 px) }; run rate 0.5 until landed }
  }
  scene try_it {
    beat { explore limit 60 s keep angle { slider(angle, range: [10 deg, 80 deg]) } }
    beat { request relaunch; run rate 1 until landed; narrate "Compare with your prediction." for 3 s }
  }
}
```

- The timeline waits on `landed`, holds, seeks back and replays (PK-9.2, PK-9.3), with the same trajectory both times (RC-12.2).
- The `explore ... keep` beat lets the learner set the launch angle, then continues the lesson with it (PK-9.9). Without `keep`, the lesson would return to its own run. In video, the explore beat uses its fallback.
- Gap (resolved): an earlier draft of the model exposed the launch velocity as one vector `v0`, which a slider for the angle cannot target. The model now has `speed` and `angle` parameters (MK 19.1, RP-01), and relaunching is a requestable event (D-027).

### 13.6 Findings

| Program | Presentation kernel |
|---|---|
| Projectile, with and without drag | Expressible; conditional expectations noted |
| Bouncing ball, pendulum, spring-mass | Expressible |
| Function plot | Expressible |
| Vector addition | Expressible |
| Narrated lesson | Expressible |

The model kernel's open item (checks at an event instant) is resolved by `on(E)` observations and expectations.

---

## 14. New positions taken in this document

**Register entries:**

| Entry | Position | Sections |
|---|---|---|
| D-025 | Learner model actions during a narrated lesson happen only in `explore` beats, on a branch; the lesson returns to its own run unless the beat keeps the learner's choice (Accepted) | 9.3 |
| D-026 | Accessibility baseline in v1: generated text alternatives, keyboard operation of every control and drag, captions, no color-only encoding (Accepted) | 11 |
| D-033 | Run-directing actions in a beat apply in written order before presentation time passes (Accepted; raised by syntax study S-8) | 9.1 |
| D-034 | `formula` representation for expressions, definitions and labeled expressions; never strings (Accepted; raised by syntax study S-10) | 6 |

**Elaborations (in this specification only):**

- Representations of members of collections, drawn while the member is alive (PK-6.3c; D-055, D-057).
- Observation schedules, including `on(E)` at event instants (PK-3.1, PK-3.4).
- Expectations as the form of reference-program results and learner predictions (section 4).
- Unit-aware encodings with declared scales (PK-5.5); invertibility declared, never inferred (PK-5.6).
- A representation property is bound, set or animated, one at a time (PK-6.2, PK-8.5).
- Controls are representations with inverses (PK-6.4).
- All views of a frame show one simulation instant (PK-7.5).
- Frames between steps come from dense output, never from extra steps (PK-8.3).
- `wait_until` never hangs (PK-9.4).
- Drag default `hold` for running simulations; only commits are logged (PK-10.8, PK-10.9).
- Undo as seek with the intervention removed (PK-10.10).
- Medium fallbacks declared, never silently dropped (PK-12.3).
- From the presentation prototype: mapping persistence across beats (PK-8.2a), reading time and highlight duration (PK-9.2b), exact event instants at the end of `wait_until` (PK-9.3a), waits while holding (PK-9.4a), lesson run end and run versions (PK-9.5a), refusal of learner actions (PK-9.8a), keyboard steps (PK-11.2a), announcements (PK-11.3a), view coordinates of frames (PK-12.1a), static diagnostics (section 16).
- From the web player: sampled sources (PK-6.3a), plot axes with dimensions (PK-7.3a).

---

## 15. Deferred

| Item | Where it will be specified |
|---|---|
| Education layer: questions, answers, feedback, scoring (R-22) | Education layer specification |
| Instrument noise (needs randomness, D-013) | Stochastic slice |
| Field representations (heatmaps, vector fields, D-010) | Slice with field state |
| 3D views and projection from 3D to 2D | Slice that needs them |
| Construction tools (creating points, lines, objects by interaction; 08 sections 43-46) | Slice with dynamic collections and relations |
| Conditional expectations (`expect ... when k = 0`) (13.1) | Resolved: expectations belong to named cases (run configurations), `reference-programs/README.md` |
| Video, image and vector renderers | After the web player (D-018) |
| Frame description format and renderer interface | Implementation specification; the prototype's format is `crates/prismal-present/src/frame.rs` (PK-12.1a) |
| Narration audio (recorded or synthesized) | Resolved: supplied by hosts by cue name (PK-9.2d, D-053) |
| Language and localization of narration | Later |

---

## 16. Static diagnostics

A presentation is checked against its model before it is used (PK-2.3). Each diagnostic names the element concerned.

| Code | Condition | Rule |
|---|---|---|
| PK-E01 | Broken reference: an element the model or the presentation does not have | PK-2.3 |
| PK-E02 | An expression does not check in the model's scope (the kernel's diagnostic is quoted) | MK section 18 |
| PK-E03 | A control, inverse, intervention or `keep` targets a binding that is not intervenable, or a `request` names an event not declared `on request` | PK-6.4, D-023, D-027 |
| PK-E04 | An encoding without a declared scale of the source's dimension | PK-5.5 |
| PK-E05 | A representation whose sources do not suit its kind or its view | PK-6.3 |
| PK-E06 | A representation kind the implementation does not provide (reported, never dropped) | PK-6.7, PK-12.3 |

## History

- 2026-09-29 written; D-025 and D-026 raised.
- 2026-09-30 revised with the syntax study (D-033, D-034).
- 2026-09-30 revised with the presentation prototype: elaborations PK-8.2a, PK-9.2b, PK-9.3a, PK-9.4a, PK-9.5a, PK-9.8a, PK-11.2a, PK-11.3a, PK-12.1a; section 16; the velocity arrow of 13.5 declares its scale (PK-5.5).
- 2026-09-30 revised with the web player: sampled sources for `trace` and `series_plot` (PK-6.3a); plot axes with dimensions (PK-7.3a).
- 2026-10-01 plot axes that follow the data, axis display units, zoom and pan of plots (PK-7.3b, D-070).
- 2026-09-30 animations as named effects: PK-9.2c (D-042).
- 2026-09-30 groups: PK-6.3b (D-043).
- 2026-09-30 formula layout for every medium: PK-6.5a (D-046); frame contents completed in PK-12.1a.
- 2026-09-30 targeting and focus order: PK-10.2a, PK-11.2b (D-047).
- 2026-10-01 drags on members and clicks: PK-10.5a, PK-10.5b; targeting and focus include clickable representations (D-059).
- 2026-10-01 clicks on an empty point of a view: PK-10.5c (D-060).
- 2026-10-01 author styles: PK-6.6a (D-061).
- 2026-10-01 drags on members of groups: PK-6.3b (D-062).
- 2026-10-01 descriptions track in linear media: PK-11.3a.
- 2026-09-30 narration sound supplied by hosts: PK-9.2d (D-053).
- 2026-09-30 round geometry: PK-6.3c `circle`, `ellipse`, `arc` (D-054).
- 2026-10-01 page layout of views: PK-7.4a (D-063).
- 2026-10-01 continue points and animated properties: PK-9.2e, PK-9.2f, PK-9.10a (D-067, D-068).
- 2026-10-01 buttons in lessons and runtime-control buttons: PK-6.3d, PK-9.8b (D-069).
- 2026-10-13 `restart` and `reseed` in explore beats: PK-9.8c (D-088).
- 2026-10-13 fields: `function_graph` of a field and `heatmap`, PK-6.3f; a presentation's `until`, PK-10.1a (D-091).
