# Divergence Register

This register records every point where Prismal departs from, amends, or has not yet settled a position in the exploration record (`docs/design/01` to `09.11`).

The exploration documents are frozen. They are not edited to reflect later decisions; this register is the authoritative record of what changed and why.

## How to use this register

- Each entry has a stable ID (`D-###`). IDs are never reused or renumbered.
- **Status values:**
  - `Open`: a question the owner must decide. No default is assumed.
  - `Proposed`: a recommended position awaiting owner review.
  - `Accepted`: the current position of the project. Supersedes the exploration record.
  - `Rejected`: considered and declined; the original position (or another entry) stands.
  - `Superseded`: replaced by a later entry (named in the entry).
- When a status changes, update the entry in place and append a line to its **History**. Do not delete entries.
- Every entry names its source (`Original position`) and its evidence (`Audit finding`), so the reasoning can be reconstructed later without the conversation that produced it.

## Summary

| ID | Topic | Status |
|---|---|---|
| D-017 | Product purpose and long-term scope | Accepted |
| D-001 | v1 scope: first slice is mechanics + math | Accepted |
| D-002 | Language form: kernel-first; authorship open, programming-literate authors expected first | Accepted |
| D-003 | Prior art as a design input | Accepted |
| D-004 | Hybrid dynamics semantics | Accepted |
| D-005 | Multiple contributions: type-supplied default combination | Accepted |
| D-006 | Syntax chosen after semantics, by comparison; early sketch allowed | Accepted |
| D-007 | Causal (one-way) equations in v1 | Accepted |
| D-008 | Algebraic cycles detected statically | Accepted |
| D-009 | Explanatory narrative as a first-class layer | Accepted |
| D-010 | Field as function and field as state | Accepted |
| D-011 | Three kernels instead of one | Accepted |
| D-012 | Validation by executable reference programs | Accepted |
| D-013 | Stochastic event-time sampling | Accepted |
| D-014 | Affine quantities, literals and dimension | Accepted |
| D-015 | Scoped determinism guarantees | Accepted |
| D-016 | Exploration record frozen, normative docs separate | Accepted |
| D-018 | Output targets: web player first; output form follows content and author intent | Accepted |
| D-019 | Relationship to Mava Studio | Accepted |
| D-020 | Constraints checked, not enforced, in v1 | Accepted |
| D-021 | Plane angle is dimensionless | Accepted |
| D-022 | Spaces have frames; conversions are explicit | Accepted |
| D-023 | Intervenable bindings are declared by the model | Accepted |
| D-024 | No event priorities; ordering by cascades | Accepted |
| D-025 | Learner exploration during a narrated lesson | Accepted |
| D-026 | Accessibility baseline in v1 | Accepted |
| D-027 | Requestable events | Accepted |
| D-028 | Syntax direction: candidate A amended with author conveniences from B and C | Accepted |
| D-029 | Function values that read model bindings | Accepted |
| D-030 | Literal `0` adopts zero vectors | Accepted |
| D-031 | Model variants in reference-program cases | Accepted |
| D-032 | Tuple literals as vectors: typing by expected type | Accepted |
| D-033 | Order of run-directing actions within a beat | Accepted |
| D-034 | Formula representation for expressions | Accepted |
| D-035 | Keywords and spelling of the working syntax | Accepted |
| D-036 | Comments and identities across text and visual editing | Accepted |
| D-037 | IR serialization: versioned JSON | Accepted |
| D-038 | Self-retriggering decided through flows, specialized on the handler's discrete values | Accepted |
| D-039 | Named mathematical constants kept by name in the IR | Accepted |
| D-040 | Reserved words and contextual keywords | Accepted |
| D-041 | `on(E)` follows an occurrence of `E` as well as an emission | Accepted |
| D-042 | Animations in v0: named effects (`reveal`, `hide ... for`, `camera`) | Accepted |
| D-043 | `group`: members placed by a shared rigid transform with scale, applied in model space | Accepted |
| D-044 | Prismal is a general embeddable system; Mava Studio is one consumer among others | Accepted |
| D-045 | Host interface: an engine with handles, a Rust API and one JSON protocol offered by every binding | Accepted |
| D-046 | Formulas typeset for every medium: a math box tree and its layout in frames; MathML only in the browser binding | Accepted |
| D-047 | Input: the host captures and forwards, the engine interprets (targeting, viewports, zoom and pan, focus); raw and semantic inputs | Accepted |
| D-048 | Declared functions in v0: model-level `fn`, closed, non-recursive, in the IR as `functions` and `{"fn": id}` | Accepted |
| D-049 | Enumerations in v0: model-level `enum`, nominal by identity, cases typed by context, `match` with one arm per case | Accepted |
| D-050 | Event payloads in v0: declared where they enter (`on request(p: T)`, `on E(p: T)`), supplied by requests and `emit E(v)` | Accepted |
| D-051 | Inputs in v0: optional defaults, starting values from the run, piecewise-constant changes from runs and hosts (`set_input`) | Accepted |
| D-052 | Video export: frames from the SVG renderer rasterized in process, encoding by an external encoder through a pipe, one canvas per clip, captions drawn and as a track | Accepted |
| D-053 | Narration sound belongs to hosts, not programs: cues named after their beats, voiced by recordings named by cue or by synthesized speech; the timeline keeps the timing | Accepted |
| D-054 | Round geometry: `circle`, `ellipse` and `arc` representations with radii in model units, carried in frames as exact elliptical arcs | Accepted |
| D-055 | Contained objects and fixed collections in v0: object types in a model, `parts`, member expressions and aggregates, container flows over members, `for` in views; elaborated to a flat model before checking | Accepted |
| D-056 | After a transition, a crossing guard near zero takes the sign it is heading to as its reference (RC-7.2a) | Accepted |
| D-058 | Relations: relation types with endpoints in collections, relation sets with a capacity, `connect` and `disconnect`, endpoints read and compared, relations ended with their endpoints | Accepted |
| D-057 | Collections whose membership changes: a declared capacity (`Drop[max 50]`), `create` and `destroy` in handlers, events per member, `set b.x` from the container; elaborated to members with a liveness binding | Accepted |
| D-059 | Members as payloads (`on request(b in balls)`), several payloads per event, drags and clicks on members; requests refused with a reason | Accepted |
| D-060 | Clicks on an empty point of a view: `on click as q request E(q)` in the view, the point as payload | Accepted |
| D-062 | Drags on members of a group: the gesture's value is the pointer in the group's frame | Accepted |
| D-061 | Author styles: `color:` from a named palette each medium maps to its theme; `line:` solid, dashed or dotted | Accepted |
| D-063 | Page layout: `layout` places views in nested rows and columns; views left out follow it; media adapt it to their size | Accepted |
| D-064 | Undirected relations: `undirected relation`, endpoints in one collection read with `s.has(o)` and `s.other(o)` | Accepted |
| D-065 | Relations across containers: endpoints, members and loops named by part paths through contained objects (`left.atoms`) | Accepted |
| D-066 | Collections without a declared limit: `[max inf]`, a working capacity that doubles and recomputes the run when filled | Accepted |
| D-067 | Continue points: `wait learner [limit d] [fallback { ... }]`, opened when the beat's other actions end; a pending point stops the player | Accepted |
| D-068 | Animated properties and handover: `animate R opacity` or `offset to v [for d]`, `release R`, `bind R [for d]` | Accepted |
| D-069 | Buttons in lessons and runtime-control buttons: an explore beat's `button(E)`, the learner step `press E`, `button(reset)`, `button(undo)`, `button(redo)` in labs | Accepted |
| D-070 | Plot axes that follow the data, display units on plot axes, zoom and pan of plots | Accepted |
| D-071 | Drag mode `live`, the interactive failure policy `pause`, and failures located inside a step | Accepted |
| D-072 | Buttons whose event takes a payload | Accepted |
| D-073 | Contributions to discrete state | Accepted |
| D-074 | `create` and `connect` through part paths | Accepted |
| D-075 | Animated color, line style and scale, shape morphs, easing, and cameras on plots | Accepted |
| D-076 | Authors' text, conditions on representations, and titles | Accepted |
| D-077 | Words at a slider's ends, the percent unit, and values in words (lab authors' requests) | Accepted |
| D-078 | Findings from the first complete lesson: path breaks, rounding, labels, and `on E` | Accepted |
| D-079 | Findings from the geometry example: powers in units, morph order | Accepted |
| D-080 | Findings from the objects example: event detection cost, texts in the framing | Accepted |
| D-081 | Syntax stability review: options and clauses, and version 1 | Accepted |
| D-082 | Release numbering: 0.x releases, branch per minor version | Accepted |
| D-083 | The stochastic slice: random events, draws and seeds | Accepted |
| D-084 | Many runs: another seed in labs, ensembles, histograms, statistics, staircases | Accepted |
| D-085 | Findings from the reaction example: `restart`, histogram bars on the values' grid | Accepted |

---

## D-017: Product purpose and long-term scope

- **Status:** Accepted
- **Original position:** The exploration record generalized the project into a universal scientific modeling language (01 to 09.11).
- **Audit finding:** F-03 described this as scope drift away from education. The owner corrected that reading (see History).
- **Accepted position:** Prismal is a universal scientific modeling language **whose purpose is education**. The broad scientific scope is intended. Every language output serves one of three use modes:
  1. **Self-study:** a learner explores a concept on their own.
  2. **Deep study:** intensive investigation of a concept (experiments, parameters, multiple representations, comparison).
  3. **Syllabus-based content creation:** authors produce scientifically grounded educational content aligned to a curriculum (including narrated and exported material).
- **Consequences:**
  - The case catalogue (01) stands as the scope horizon; it is not cut back.
  - The scope question moves to sequencing: which part of the horizon comes first (D-001).
  - The three use modes become design drivers. A construct that serves none of them needs a justification.
  - Use mode 3 supports D-009 (explanatory narrative as a first-class layer).
- **History:**
  - 2026-09-29 accepted by the owner in response to audit finding F-03: "somewhere chatgpt got my intent, more so on universal scientific language; however this doesn't go away from the basis of education."

## D-001: v1 scope

- **Status:** Accepted
- **Original position:** No scope limit. "Don't limit ourselves before limits show up" (01, 02); final validation matrix covers math, physics, chemistry, biology, algorithms, GPU, video, sensors (09.11.7.4).
- **Audit finding:** F-03, as corrected by D-017.
- **Question:** The long-term scope is universal (D-017). Which slice of it is built and validated first? This is an ordering question, not a limit on the language's design.
- **Options considered:** mechanics + math; math only (no time evolution); chemistry kinetics (stochastic, many-body); one real syllabus unit.
- **Accepted position:** The first slice is **2D mechanics plus supporting mathematics**:
  - Kinematics and dynamics: projectile, pendulum, bouncing ball, spring-mass systems.
  - Mathematics: functions and their graphs, vectors, parameters.
  - Slice shape (as presented with the decision): thin but end to end through every layer: model, simulation, multiple synchronized views, interaction, and a narrated explanation timeline, so all three use modes of D-017 are exercised. The output medium for the slice is the web player; see D-018, which amended this line (video export was removed from the first slice).
- **Reasons:**
  - Forces the hardest core rules early: continuous evolution, events (collisions, bounces), and multiple contributions to one variable (D-004, D-005).
  - Expected results are known analytically (range, period, bounce times, energy), which makes executable validation straightforward (D-012).
  - Chemistry kinetics and stochastic processes come in a later slice; the core must not preclude them.
- **Consequences:** The first reference programs are drawn from this slice: projectile (with and without drag), pendulum, bouncing ball, spring-mass, function plot with a draggable parameter, vector addition.
- **History:**
  - 2026-09-29 opened by the design audit.
  - 2026-09-29 reframed from "scope cut" to "first slice of a universal scope" following D-017.
  - 2026-09-29 accepted by the owner: mechanics + math.
  - 2026-09-29 slice shape amended by D-018: web player only; video export later.

## D-002: Language form and primary author

- **Status:** Accepted
- **Original position:** Not addressed. The series assumes a standalone DSL implicitly.
- **Audit finding:** F-10.
- **Options considered:**
  1. Kernel-first: Rust library and API first, textual syntax designed later over proven semantics.
  2. Standalone language as the first deliverable (parser, type checker, diagnostics, editor support).
  3. Visual editor plus data format (possibly Mava Studio), with a text language later or never.
- **Accepted position:**
  - **Form: kernel-first.** The semantic kernel and runtime are built as a Rust library and API. The reference programs (D-012) are written against that API. The authoring surface (textual syntax, visual editor, or both) is designed afterward, over semantics already proven by execution.
  - **Authorship is open.** Anyone may author, including people and content the language makers did not target. No author profile is built into the language.
  - **Expected first authors: educators and content creators, most likely with programming knowledge.** The textual surface may assume programming literacy.
  - **Non-programmers are reached through Mava Studio** (see D-019), not by simplifying the language itself.
- **Reasons:**
  - Avoids repeating the exploration record's main failure: choosing syntax before semantics were defined (F-05).
  - Defers the fixed costs of a language (grammar, diagnostics, editor tooling) until the semantics justify them.
- **Consequences:**
  - The Rust API is an internal and developer-facing layer, not the main authoring surface. It may be verbose.
  - The expected main authoring surface is a textual language for programming-literate educators and creators. The syntax study (D-006) is evaluated against that audience: familiar to programmers, but reading close to the science and mathematics being taught. A visual editor is optional, not required.
  - Because authorship is open, the language must stay general: domain libraries, representations and interactions are open to any author, and nothing assumes a curriculum, subject or audience.
  - Output targets were split out as D-018.
- **History:**
  - 2026-09-29 opened by the design audit.
  - 2026-09-29 accepted by the owner: kernel-first; primary author educators and creators.
  - 2026-09-29 amended by the owner: "chances are high it will be people with programming knowledge being authors". Consequences updated accordingly.
  - 2026-09-29 amended by the owner: "Author can even be some person coming up with their own content that we as the makers of the language didn't target, so who an author is can't be entirely predictable." Authorship recorded as open; non-programmers reached via Mava Studio (D-019).

## D-003: Prior art as a design input

- **Status:** Accepted
- **Original position:** Design derived from first principles; no prior systems cited.
- **Audit finding:** F-04.
- **Accepted position:** Each section of the core semantics spec cites the prior art it adopts or departs from (Modelica, hybrid automata and superdense time, Gillespie SSA, NetLogo, GeoGebra/Desmos, Manim/Motion Canvas, FRP), with a sentence explaining the choice. Sections also cite the carried-forward resolutions (`R-##`) they restate, so both external prior art and the project's own earlier analysis are traceable.
- **Reason:** These systems document the failure modes the series missed (F-02, F-06, F-07, F-08).
- **History:**
  - 2026-09-29 proposed by the design audit.
  - 2026-09-29 accepted by the owner ("Go to next natural item" in reply to the proposal to accept D-003 and move to D-004). Citation of `R-##` IDs added, following the owner's request that the earlier analysis be visibly reflected.

## D-004: Hybrid dynamics semantics

- **Status:** Accepted
- **Original position:** Informal hybrid loop (06 sections 22-23); guards written as level conditions (`y <= 0`); simultaneous events handled by ordering policy (06 section 17).
- **Audit finding:** F-02, as corrected in the audit addendum.
- **Builds on:** R-28 (event times are located), R-29 (deterministic same-time policy), R-57 (contributions), R-60 (event occurs when the guard becomes true).
- **Accepted position:** Dynamics follow a hybrid-automaton formulation:
  - **Flows:** continuous variables evolve by declared derivatives.
  - **Guards:** zero-crossing conditions with explicit direction (rising, falling, either). Level conditions are allowed only for discrete-time rules, never as continuous event guards.
  - **Resets:** event handlers assign new values; they are the only place continuous state is replaced.
  - **Event instants:** superdense time `(t, n)`; an event cascade at one instant advances `n`, with a declared maximum iteration count.
  - **Zeno policy:** required for any model where events can accumulate (detect and stop, or apply a declared threshold rule such as resting contact).
- **Reason:** Standard, well-understood semantics (hybrid automata; Modelica, Simulink and Ptolemy practice) with known implementation strategies. It closes the gaps left by R-60: crossing direction, same-instant cascades, and Zeno behavior.
- **Consequence:** The bouncing ball becomes the first reference program (D-012): its bounce times form a geometric series, so the result is exactly checkable, and it exercises the Zeno policy.
- **History:**
  - 2026-09-29 proposed by the design audit.
  - 2026-09-29 accepted by the owner.

## D-005: Multiple contributions to one variable

- **Status:** Accepted
- **Original position:** RUN-01 (09.10b) was resolved in 09.10c Part A section 2 (R-57): evolution distinguishes definition from contribution, and multiple contributions require explicitly defined combination semantics; 09.10c deliberately declined to make sum a default. 09.11 section 7 relisted the question as open.
- **Audit finding:** F-06 (corrected in the audit addendum).
- **Point of difference with R-57:** only whether a target's type may supply a default combination. Everything else in R-57 is adopted.
- **Options considered:** always explicit (R-57 as written); type-supplied default; per-model declaration.
- **Accepted position:**
  - Everything in R-57 stands: behaviors contribute to an evolution target; they do not each define it; combination semantics must be defined.
  - **Combination may be supplied by the target's type.** A quantity type can declare its natural combination; rates of change (derivatives such as acceleration, velocity, reaction rate) declare sum. A target can override its type's combination. A type without a declared combination requires every target of that type to state one explicitly.
  - Sum is therefore never assumed by the core; it is declared at the type level, once.
  - **Direct replacement** of a value happens only in event resets (D-004) and interventions. Two replacements of the same value at the same instant are a reported conflict, never a silent winner (R-26).
- **Reason:** Matches Modelica force balances and physics-engine force accumulation, removes repetitive declarations in the common case, and keeps R-57's safeguard that no combination is assumed by the core.
- **History:**
  - 2026-09-29 proposed by the design audit.
  - 2026-09-29 accepted by the owner: type-supplied default.

## D-006: Syntax chosen after semantics, by comparison

- **Status:** Accepted
- **Original position:** Syntax options enumerated (09.3), operators deferred (09.4 section 17), examples written in inconsistent notations and reported validated (09.10, 09.11).
- **Audit finding:** F-05.
- **Builds on:** R-03 (no syntax before meaning), R-44 (hybrid surface style, as direction), R-46 (distinctions the syntax must preserve), R-69 (09.x example syntax has no standing).
- **Accepted position:**
  1. **No syntax is adopted until the core semantics spec exists.**
  2. **A provisional sketch syntax is allowed early**, only so that spec examples and reference programs are readable. It is explicitly non-binding, is labeled as sketch wherever it appears, and gets no preference in the comparison.
  3. **Selection by comparison:** two or three candidate syntaxes in R-44's hybrid style are compared by writing every reference program (D-012) in each.
  4. **Criteria:** readability for programming-literate authors (D-002), closeness to the science being taught, clear visibility of the R-46 distinctions, and clean round-tripping with the serializable model representation edited by Mava Studio (D-019).
- **Reason:** Syntax chosen before semantics was the main source of inconsistency in the series. The sketch allowance keeps the spec readable without letting an early notation harden by habit.
- **History:**
  - 2026-09-29 proposed by the design audit.
  - 2026-09-29 accepted by the owner, with an early non-binding sketch syntax allowed.

## D-007: Causal (one-way) equations in v1

- **Status:** Accepted
- **Original position:** Equations are relationships whose role is set by context; `F = m * a` does not imply a direction (05c section 11, 09.4 section 18, 09.11 section 5). Implies acausal solving without stating it.
- **Audit finding:** F-07.
- **Options considered:** causal in v1 with acausal later; acausal from the start; defer the decision.
- **Accepted position:**
  - **v1 is causal.** Evolution is written as explicit derivatives and assignments (the author states which quantity is computed from which).
  - **Equations remain first-class values.** An equation such as `F = m a` can be displayed, labeled, bound to live model values, and checked at runtime (residual reported), but it does not drive the simulation.
  - **Acausal solving is a later, explicit extension.** The core must not preclude it: equation objects keep their symbolic form so a future solver can consume them.
- **Reasons:**
  - Acausal solving (causalization, algebraic-loop solving, DAE index reduction) is a major subsystem; Modelica took years to mature it.
  - The first slice (D-001, mechanics + math) needs only causal evolution.
  - The educational value of equations (display, linking symbols to live values, checking) is preserved.
- **History:**
  - 2026-09-29 opened by the design audit.
  - 2026-09-29 accepted by the owner: one-way in v1.

## D-008: Algebraic cycles detected statically

- **Status:** Accepted
- **Original position:** "Cycles are not automatically errors"; the runtime determines what kind of cycle it is (05c section 48, 09.11 section 5).
- **Audit finding:** F-08.
- **Builds on:** R-35 (cycles are not automatically errors), R-53 (composition invariant 7), D-007 (causal formulas).
- **Accepted position:** Dependency analysis runs before execution and classifies cycles. A cycle through state (integrator, delay, discrete step) is valid. A cycle among instantaneous derived values is an algebraic loop: a compile-time error unless an explicit solver is declared for it.
- **Reason:** Solvability is not decidable at runtime in general; the structural check is standard and cheap.
- **History:**
  - 2026-09-29 proposed by the design audit.
  - 2026-09-29 accepted by the owner.

## D-009: Explanatory narrative as a first-class layer

- **Status:** Accepted
- **Original position:** Author-driven animation is a presentation concern and a "pure animation" execution mode (05c section 63, 06 section 60).
- **Audit finding:** F-03.
- **Builds on:** R-22 (education above the core), R-23 (three clocks), R-32 (static models and pure animation), R-07 (simulation is not animation), D-017, D-018.
- **Accepted position:** The explanatory timeline (scenes, beats, reveals, camera direction, narration cues) is a peer of the model, with a precise synchronization contract to simulation time (for example: a beat can wait for a model event; a scene can pause, seek or branch the simulation). The model never depends on the timeline; the timeline observes and directs it. The same timeline drives the interactive player (where the learner may pause, intervene or branch) and later video export (linear playback).
- **Reason:** It is the capability that motivated the project (Manim-style explanation with interactive simulation) and the main difference from a general modeling tool.
- **History:**
  - 2026-09-29 proposed by the design audit.
  - 2026-09-29 accepted by the owner; the core spec gets a timeline section alongside dynamics.

## D-010: Field as function and field as state

- **Status:** Accepted
- **Original position:** Field reduced to a library specialization of Function (09.11 section G).
- **Audit finding:** F-09.
- **Builds on:** R-15 (fields first-class, meaning independent of storage); supersedes R-65.
- **Accepted position:** Two forms. Analytic fields are functions over a domain. Field state is a core state form over a domain, and its discretization (grid, resolution, boundary treatment) is declared execution configuration visible to the author.
- **Reason:** Discretization changes results, so it cannot be hidden as a runtime optimization.
- **History:**
  - 2026-09-29 proposed by the design audit.
  - 2026-09-29 accepted by the owner; formula fields are needed in the first slice, evolving fields in a later slice.

## D-011: Three kernels instead of one

- **Status:** Accepted
- **Original position:** One 21-concept semantic kernel covering model, presentation and interaction (09.11.7.1).
- **Audit finding:** F-11.
- **Builds on:** R-66 (the 21 kernel concepts, all carried forward), R-05 (layered separation), R-42 (08 section 97 layers), D-009, D-019.
- **Accepted position:** The same concepts, organized in three parts connected by narrow interfaces:
  - **Model kernel:** values, types, bindings, identity, objects, relations, collections, domains, expressions, functions, constraints, state, processes (flows), events, operations.
  - **Runtime contract:** clocks, steps, commit, event instants, randomness, snapshots, replay.
  - **Presentation kernel:** observation and data, projection, representation, view, explanation timeline (D-009), interaction.
  - A model runs without any presentation (headless testing, offline export). One model can carry several presentations. The serializable model representation (D-019) keeps model and presentation sections separate.
- **Reason:** Consistent with the layer separation the series itself argues for; each part can be validated independently.
- **History:**
  - 2026-09-29 proposed by the design audit.
  - 2026-09-29 accepted by the owner; the core spec is structured this way.

## D-012: Validation by executable reference programs

- **Status:** Accepted
- **Original position:** Validation by construct-to-concept mapping with PASS verdicts (09.10, 09.11).
- **Audit finding:** F-01.
- **Builds on:** Supersedes R-68 and R-70. 09.5b section 66 had called for a set of actual programs.
- **Accepted position:** A fixed set of reference programs with expected results serves as the acceptance suite for the semantics spec (expressible?), the syntax study (D-006), and the prototype (correct results?). The set grows with each slice; programs are never silently removed.
- **First-slice set (D-001):**

| Program | Checkable result |
|---|---|
| Projectile, no drag | Range and flight time match the analytic formulas. |
| Projectile with drag | Two contributions combine (D-005); matches a high-precision reference solution. |
| Bouncing ball | Bounce times form a geometric series; the Zeno policy triggers (D-004). |
| Pendulum | Small-angle period close to 2π√(L/g); energy drift within declared bounds per solver. |
| Spring-mass | Period 2π√(m/k); energy conserved within tolerance. |
| Function plot with draggable parameter | Dragging updates the model value; all views stay synchronized (R-37, R-40). |
| Vector addition | Point/vector rules enforced (R-09); invalid operations rejected with a diagnostic. |
| Narrated projectile lesson | The timeline waits on the landing event, then pauses and seeks the simulation (D-009). |
- **Reason:** Assertion-based validation passed a defective example.
- **History:**
  - 2026-09-29 proposed by the design audit.
  - 2026-09-29 accepted by the owner; first-slice program list adopted, to grow with later slices.

## D-013: Stochastic event-time sampling

- **Status:** Accepted
- **Original position:** Per-step probability sampling (06 section 19; 09.11 section 3 with an incorrect formula).
- **Audit finding:** F-12.
- **Builds on:** R-10 (distribution is not random source), R-30 (reproducible random streams), R-61 (stochastic concepts), D-004 (event machinery).
- **Accepted position:** Two stochastic modes: per-step sampling and exact event-time sampling (exponential waiting times, Gillespie SSA). Formulas in the exploration record are not normative.
- **History:**
  - 2026-09-29 proposed by the design audit.
  - 2026-09-29 accepted by the owner; specified in detail when the stochastic slice is planned.

## D-014: Affine quantities, literals and dimension

- **Status:** Accepted
- **Original position:** Units and Point/Vector distinction (05c, 09.5); examples use `°C`, unitless `0` in dimensioned vectors, and `Point<2>` in a general interface.
- **Audit finding:** F-13.
- **Builds on:** R-08 (units and dimensions), R-09 (point vs vector), R-11 (no assumed 2D).
- **Accepted position:** Affine quantities (temperature scales with an offset, time instants) follow the Point/Vector rules. A policy for dimensionless literals is defined: a literal `0` adopts the required dimension; any other bare number combined with a dimensioned quantity is an error (for example `5` written where `5 m` was meant). Spatial dimension is a type parameter in general interfaces.
- **History:**
  - 2026-09-29 proposed by the design audit.
  - 2026-09-29 accepted by the owner.

## D-015: Scoped determinism guarantees

- **Status:** Accepted
- **Original position:** Three determinism levels, including "strong deterministic" without platform scope (06 section 67).
- **Audit finding:** F-14.
- **Builds on:** R-34 (determinism levels), R-30 (reproducible random streams), R-25 (transactional commit).
- **Accepted position:** Bit-identical replay is guaranteed only for the same build and platform. Across platforms, results are equivalent within declared tolerances unless a deterministic math mode is selected. That mode (identical results everywhere at some speed cost, for example for graded assessment) is not built in the first slice, but the design must leave room for it.
- **History:**
  - 2026-09-29 proposed by the design audit.
  - 2026-09-29 accepted by the owner.

## D-016: Exploration record frozen, normative docs separate

- **Status:** Accepted
- **Original position:** Each document in the chain treated as the foundation for the next.
- **Audit finding:** F-15.
- **Builds on:** R-01 to R-70 (carried-forward record).
- **Accepted position:**
  - `docs/design/01` to `09.11` are frozen as the exploration record. Their original text is not rewritten.
  - **Correction notes are allowed** inside the frozen documents, next to the passage they correct, in exactly this form:
    `> **Correction note (YYYY-MM-DD; D-### or R-##; audit F-## if any):** what is wrong and where the current position lives.`
    Notes point to the register or carried-forward record; they do not restate decisions in full.
  - Mechanical formatting repairs (such as broken code fences) are allowed and are not content edits.
  - Continuity is recorded in the carried-forward record (R-##); changes in this register (D-###). Precedence: register, carried-forward record, exploration record.
  - New normative documents are short and decision-oriented, and cite R-## and D-### for their sources (D-003).
- **Reason:** Rewriting 750,000 characters of conversational material is costly and would destroy the historical record.
- **History:**
  - 2026-09-29 proposed by the design audit.
  - 2026-09-29 accepted by the owner with an amendment: clearly marked correction notes may be added inside the frozen documents.

## D-018: Output targets

- **Status:** Accepted
- **Original position:** Multiple renderers and video export assumed (05c, 06, 07); no target platforms chosen. Separately, the owner's early direction in the design conversation (preceding 01): "we don't have to force the output to be in one presentational form; based on the analysis, we can come up with different adaptable forms of representing the concepts". The resulting principle, one concept with several valid representations (for example radioactive decay as particles, an N(t) graph, a half-life timeline, or an interactive experiment), underlies 07.
- **Audit finding:** F-10 (split from D-002).
- **Options considered:** web player (WASM); video export; native desktop; Mava Studio as host.
- **Accepted position:**
  - **First slice target: an interactive web player** (Rust compiled to WebAssembly, running in the browser).
  - **Output is not fixed per project.** Both the presentational form (spatial scene, graph, timeline, equation, experiment panel, and so on) and the output medium (interactive player, video, still image, and others) are chosen according to the nature of the content and the author's intention. The architecture must treat every medium as one projection target among several, never as the defining output.
  - Video export, desktop, and Mava Studio integration are later additions that must fit this principle without redesign.
- **Reasons:**
  - The browser reaches learners without installation and can be embedded in learning platforms (self-study and deep study, D-017).
  - Tying the core to one medium would contradict the representation principle the owner set at the start.
- **Consequences:**
  - The presentation kernel (D-011) must separate "what representation" from "which medium" so the same scene can later render to video.
  - D-009 (narrative timeline) must be medium-independent: the same timeline drives the interactive player now and video later.
- **History:**
  - 2026-09-29 split from D-002.
  - 2026-09-29 accepted by the owner: web player (WASM) first, with the explicit instruction not to ignore the early principle that output depends on the nature of the content and the author's intention.

## D-019: Relationship to Mava Studio

- **Status:** Accepted
- **Original position:** Not addressed in the exploration record. Mava Studio (the owner's Tauri + Vue authoring tool) was mentioned only at the start of the design conversation.
- **Accepted position:**
  - Prismal is a **prerequisite for the redevelopment of Mava Studio**, which is due for a redesign.
  - The redesigned Mava Studio is expected to be the main entry point for **non-programmer authors**. Programming-literate authors can use Prismal's textual surface directly.
  - Mava Studio is built on Prismal, not the other way around: Prismal must not depend on Mava Studio.
- **Consequences:**
  - Prismal needs a **complete, serializable model representation** (the semantic IR that 09.4 section 34 sketched) that a visual editor can create, inspect and modify, not only a textual syntax.
  - Text and editor should be two views of the same model. Round-tripping between them is a design goal to evaluate in the syntax study (D-006).
  - Kernel APIs should expose what an editor needs: introspection of bindings, types, units, dependencies, diagnostics, and the representations available for a value.
  - The web player (D-018) should be embeddable in Mava Studio (Tauri renders web content).
- **History:**
  - 2026-09-29 accepted by the owner: "Mava is going to face a redesign soon, and chances are that that's where most non programmers will be reconciled from. This initiative is one of the prerequisites of the redevelopment of Mava Studio."
  - 2026-09-30 the redesigned Mava Studio's architecture is not decided (Tauri + Vue above describes the current tool). Integration is designed for any host, Mava Studio being one (D-044).

## D-020: Constraints checked, not enforced, in v1

- **Status:** Accepted
- **Original position:** Constraints restrict valid states; "their enforcement strategy belongs to computation/runtime semantics" (05c section 37). Constrained direct manipulation is listed as supported (09.11.7.4, `PASS*`). No enforcement mechanism was specified.
- **Raised by:** core semantics spec v0, model kernel section 12.
- **Builds on:** R-25 (no partial commits), R-41 (proposed state, validated commit), R-50 (value restrictions are constraints), D-007 (causal v1).
- **Question:** When a constraint is violated, does the runtime move state to satisfy it (enforce), or only detect the violation (check)?
- **Options considered:**
  1. Check only: every constraint has a policy (`reject` the change, `report` a diagnostic, or `stop` the run). The runtime never changes state to satisfy it.
  2. Enforce in v1: the runtime projects state back onto the constraint (as physics-engine joints and GeoGebra's point-on-object do).
- **Accepted position:** option 1 for v1. Constrained systems are written in coordinates that satisfy the constraint by construction (a pendulum in its angle, not as a free bob held by a rod), and the constraint can be stated as a check. Enforcement is a later extension, together with acausal solving (D-007), because it needs the same solver machinery.
- **Consequences:** Constrained dragging (a point dragged along a curve) is not a model-kernel feature in v1; the presentation kernel may still map a drag onto a curve parameter. The first-slice programs do not need enforcement.
- **History:**
  - 2026-09-29 proposed in the model kernel spec v0.
  - 2026-09-29 accepted by the owner: check only.

## D-021: Plane angle is dimensionless

- **Status:** Accepted
- **Original position:** Not addressed. Units and dimensions are distinct and checked (05c sections 5-6, 09.5), with no statement on angles.
- **Raised by:** core semantics spec v0, model kernel section 3.5 (the pendulum reference program).
- **Builds on:** R-08 (units and dimensions), D-014.
- **Question:** Is plane angle its own dimension, or dimensionless as in the SI?
- **Options considered:**
  1. Dimensionless (SI 2019): `rad` is 1, `deg` is π/180, `rev` is 2π. Trigonometric functions take dimensionless arguments. A binding may carry `deg` as its display unit.
  2. Separate base dimension (as Boost.Units does): catches passing a non-angle to `sin`, but formulas such as `v = ω r` and `E = ½ I ω²` then need explicit `/rad` corrections everywhere.
- **Accepted position:** option 1. It matches the SI and the physics textbooks learners use, and keeps standard formulas unchanged. The degree/radian confusion that option 2 guards against is handled by units: `30 deg` is converted correctly wherever it is written.
- **Consequence:** angular velocity and frequency share the dimension 1/T; telling them apart is part of the deferred question of quantity kinds of equal dimension (model kernel section 21).
- **History:**
  - 2026-09-29 proposed in the model kernel spec v0.
  - 2026-09-29 accepted by the owner.

## D-022: Spaces have frames; conversions are explicit

- **Status:** Accepted
- **Original position:** Vectors are `Vector2`, `Vector3` or `Vector<Domain>` (05c section 7); Point and Vector are distinct (09.5 section 12). Coordinate transformations are listed as an open question (09.11.9, question 25).
- **Raised by:** core semantics spec v0, model kernel section 4.
- **Builds on:** R-09 (point vs vector), R-11 (no assumed 2D), D-014 (dimension as a type parameter).
- **Question:** How do points and vectors relate to coordinate systems?
- **Accepted position:**
  - A space is declared by the model with a dimension `n` and has a standard frame (origin and orthonormal axes).
  - Points and vectors belong to one space: `Point<S>`, `Vector<S, D>`. Values of different spaces never combine.
  - A space may declare further frames (for example axes along an inclined plane), each defined by a transform from another frame. Components are read in the standard frame by default (`v.x`) or in a named frame (`v.in(F).x`).
  - Moving a value between spaces needs an explicit function.
- **Reason:** a vector's meaning does not depend on the frame it is read in, but its components do. Keeping the frame explicit prevents mixing components from different axes, a common error in mechanics teaching (inclined planes, rotating frames), and follows the frame-tagged discipline of robotics libraries such as ROS tf2.
- **Consequence:** the first slice needs only one space with its standard frame; extra frames are needed from the inclined-plane and rotating-frame cases onward.
- **History:**
  - 2026-09-29 proposed in the model kernel spec v0.
  - 2026-09-29 accepted by the owner.

## D-023: Intervenable bindings are declared by the model

- **Status:** Accepted
- **Original position:** Interactions identify their target domain (R-33) and model-changing interactions name their semantic target (R-40). Which model values may be changed from outside was not specified.
- **Raised by:** core semantics spec v0, model kernel sections 6.3 and 17.
- **Builds on:** R-33, R-40, R-41, D-009 (the timeline directs the simulation), D-017 (use modes).
- **Question:** Can any model value be changed from outside the model (by a learner, a timeline or an experiment), or only values the model author allows?
- **Options considered:**
  1. Everything stored is intervenable.
  2. Parameters are intervenable by default; state bindings only when the model declares them intervenable; constants, inputs and derived values never. Allowed ranges are constraints with policy `reject`.
- **Accepted position:** option 2. The model author knows which changes keep the model meaningful (changing a ball's position mid-flight is meaningful; changing an internal counter is not). This follows FMI, which marks parameters as tunable or fixed and allows changes only at event instants.
- **Consequence:** the presentation kernel can list, for any model, exactly what a learner may manipulate; Mava Studio can show it in the editor (D-019).
- **History:**
  - 2026-09-29 proposed in the model kernel spec v0.
  - 2026-09-29 accepted by the owner.

## D-024: No event priorities; ordering by cascades

- **Status:** Accepted
- **Original position:** Same-time events are collected, ordered "by explicit semantics", evaluated and resolved (06 section 17). "Events may carry priority", which "must be semantic only when the model explicitly requires it" (06 section 18).
- **Raised by:** core semantics spec v0, runtime contract section 8.
- **Builds on:** R-25, R-26, R-29, D-004 (superdense time), D-005 (conflicts reported, never resolved by a winner).
- **Question:** When several events are due at the same instant, may the author give them priorities that decide which is handled first?
- **Options considered:**
  1. No priorities. All events due at one microstep form one transition; every handler reads the same state; overlapping writes are a conflict. An author who needs "A then B" writes A's handler to emit an event that performs B at the next microstep.
  2. Numeric priorities (as in many discrete-event simulators): higher-priority events are handled first, each seeing the previous one's result.
- **Accepted position:** option 1. Priorities make results depend on a number chosen far from the events it affects, and silently turn a conflict into a winner, which D-005 and R-26 rule out. Cascades express the same intent where the order is visible in the model, and superdense time (D-004) already provides the microsteps they need.
- **Consequence:** 06 section 18 (priority) is not carried forward in v0. Priorities can be added later if a reference program shows that cascades are too awkward.
- **History:**
  - 2026-09-29 proposed in the runtime contract spec v0.
  - 2026-09-29 accepted by the owner.

## D-025: Learner exploration during a narrated lesson

- **Status:** Accepted
- **Original position:** Not addressed. The exploration record treats guided interaction and experiments as educational layers (08 sections 80-83) without saying how learner changes interact with an authored sequence. D-009 made the timeline able to direct the simulation and noted that the learner may pause, intervene or branch.
- **Raised by:** core semantics spec v0, presentation kernel section 9.3.
- **Builds on:** D-009, D-017 (use modes), D-023 (intervenable bindings), RC section 12 (branching).
- **Question:** While a narrated lesson plays, what happens when the learner changes the model (moves the ball, changes gravity)? The narration was written for the author's run, so learner changes can make it wrong.
- **Options considered:**
  1. Learner changes allowed at any time and carried into the lesson. Maximum freedom; narration may stop matching what is on screen.
  2. Learner changes allowed only in author-marked `explore` beats, on a branch of the lesson run. When the learner continues, the lesson returns to its own run where it left off, unless the beat is marked `keep`, in which case the lesson continues with the learner's choices (and its narration is written for that).
  3. No learner model changes during lessons; exploration only in separate self-study presentations.
- **Accepted position:** option 2. Outside `explore` beats the learner still controls the timeline (pause, replay, narration speed) and the view (zoom, pan) where the presentation allows, but not the model.
- **Reason:** keeps narration truthful while supporting "try it yourself" moments inside a lesson (use mode 3 content with use mode 1 and 2 moments). Branching already exists in the runtime contract, so no new mechanism is needed.
- **Consequence:** in video export, `explore` beats use a declared fallback (a pause, a scripted demonstration, or omission).
- **History:**
  - 2026-09-29 proposed in the presentation kernel spec v0.
  - 2026-09-29 accepted by the owner.

## D-026: Accessibility baseline in v1

- **Status:** Accepted
- **Original position:** Accessibility and non-visual representations are named as representation concerns (07 sections 70-71) without requirements.
- **Raised by:** core semantics spec v0, presentation kernel section 11.
- **Builds on:** R-37, D-017 (education), D-018 (web player first).
- **Question:** Is accessibility a v1 requirement of the presentation kernel, or a later addition?
- **Options considered:**
  1. Baseline in v1: every representation that conveys model information has a text alternative generated from its bindings (label, value, unit), overridable by the author; every control and every drag works from the keyboard; narration always has captions; color is never the only encoding of a value.
  2. Later: build the web player first and add accessibility after.
- **Accepted position:** option 1.
- **Reason:** Prismal's representations are bound to named, typed, unit-carrying model values, so text alternatives can be generated rather than written by hand; that is cheap now and expensive to retrofit. Educational content is often required to meet accessibility rules (WCAG 2.2) in schools and universities. PhET has shown that accessible interactive simulations are practical.
- **Consequence:** the frame description (presentation kernel section 12) carries text alternatives and announcements from the start; the web player exposes them to assistive technology.
- **History:**
  - 2026-09-29 proposed in the presentation kernel spec v0.
  - 2026-09-29 accepted by the owner.

## D-027: Requestable events

- **Status:** Accepted
- **Original position:** Interactions produce operations on the model (08 sections 73-76); a button may trigger an action (08). The model kernel (MK-17.2) limits interventions to `set` on intervenable bindings and `create` or `destroy` on intervenable collections.
- **Raised by:** reference program RP-08 (narrated lesson). The lesson needs a "launch again" step, for the timeline and for a learner's button. With `set` alone this cannot be expressed: changing `angle` after the start does not relaunch the ball, because the launch velocity is an initial value; and a presentation that set `pos`, `vel` and `flying` itself would be doing the model's physics outside the model.
- **Builds on:** D-023 (the model declares what may be changed from outside), MK section 15 (events), RC section 11 (interventions).
- **Question:** How does the outside (a timeline, a learner's button, an experiment) ask the model to perform one of its own actions?
- **Options considered:**
  1. **Requestable events.** A model may declare an event with the trigger `on request`, optionally with a typed payload. An intervention may request it. The event's handler is ordinary model code: it runs with the model's own rules (ownership, conflicts, constraints), at the intervention's instant, and is logged like any intervention.
  2. Presentation-side scripts that `set` several state bindings. Puts model logic (how to relaunch) in the presentation, and requires every bound state to be intervenable.
  3. `reset` with new parameter values. Loses everything else in the run (the log, the lesson's history), and cannot express actions in the middle of a run (a "drop another ball" button).
- **Accepted position:** option 1. Only events the model declares `on request` can be requested; this extends D-023 from values to actions, and keeps the model author in control of what the outside can do.
- **Consequences:** MK-15.3 gains the trigger `on request`; MK-17.2 gains "request a requestable event"; the timeline gains the action `request(E)`; a `button` control can target a requestable event. The RP-01 model gains `relaunch on request` for RP-08.
- **History:**
  - 2026-09-29 proposed while writing reference program RP-08.
  - 2026-09-29 accepted by the owner.

## D-028: Syntax direction

- **Status:** Accepted (amended)
- **Original position:** 09.3 enumerates syntax options and names a hybrid direction (R-44) without choosing; 09.10 and 09.11 write examples in inconsistent notations (F-05, R-69).
- **Raised by:** the syntax study (`docs/syntax-study/`), carried out under D-006.
- **Builds on:** D-006 (selection by comparison on all reference programs), R-44 (hybrid style), R-46 (distinctions to preserve), R-47 and D-019 (serializable IR edited by Mava Studio), D-002 (programming-literate authors).
- **Question:** Which of the three candidate syntaxes, each used to write RP-01 to RP-08, becomes the base direction?
- **Options considered:**
  1. **A, flat keyword statements:** every statement starts with a keyword naming its kind; behaviors listed flat; braces. Every R-46 distinction is marked on the line; nearly one-to-one with the IR.
  2. **B, mathematical sections:** roles by section headings, mathematical operators (`x'`, `:=`, `←`, intervals), inferred types, indentation. Closest to the science and 22 % fewer tokens, but `=` has four meanings chosen by section, inferred types move errors from their cause, and many IR elements have two surface forms.
  3. **C, nested structure:** A's declarations, with behavior grouped in modes and processes, representations nested in views and interactions in representations. Removes repeated mode conditions, but modes hide a discrete state, add an implicit hold rule, and need a new IR construct or pattern recognition to print back.
- **Recommended position:** option 1, amended with C's presentation structure (view trees, interactions inside representations, explicit `sequence` in beats), C's optional named processes (which map to MK-14.1), and B's default space for a model (`model Name in Space`). Modes, sections, inferred types and Unicode-first spelling are not adopted in v1; each may return as its own proposal with a program that needs it. Scientific notation for learners is carried by typeset equations (PK-6.5), not by the source.
- **Reason:** A is strongest on local visibility of the R-46 distinctions and on round-tripping with the IR, the two criteria that are checkable properties rather than taste, and it suits the expected first authors (D-002). C's presentation structure is where C was clearly better and it round-trips one to one. The comparison, the diagnostic variants and the round-trip analysis are in `docs/syntax-study/comparison.md`.
- **Owner amendment:** A is the base, and forms from B and C that improve the author's experience are adopted where they still lower to the same IR and print canonically:
  1. **Grouped declaration blocks:** `const { }`, `param { }`, `input { }`, `state { }`, `discrete { }`, `derived { }`, each listing its declarations below the keyword. The one-line form (`param g: Acceleration = 9.81 m/s^2`) stays valid; the formatter prints blocks.
  2. **Grouped flows:** `flow { der(x) = e; der(v) += e }`, so the laws of motion sit together. The one-line `flow` form stays valid.
  3. **Interval ranges:** `angle: Angle = 45 deg in (0 deg, 90 deg)` as an alternative to `where`; both lower to one `reject` constraint. Intervals are also the one range notation in presentations and runs (plot axes, sliders, observation windows).
  4. **Timeline shorthand:** `run rate 1 until landed` for the two actions `run rate 1` and `wait until landed`.
  5. The recommended additions stand: C's view trees, interactions inside representations and explicit `sequence` in beats; C's optional named processes; B's default space (`model Name in Space`).
  - Not adopted: types inferred from unit literals (keeps errors at their declaration), C's modes, B's sections, indentation and Unicode-first spelling.
- **Accepted position:** the recommended position with the owner amendment. The working syntax is in `docs/syntax-study/working-syntax.md`.
- **Consequences:** the combined candidate is written for all eight programs and becomes the working syntax for the prototype parser, still non-binding until frozen; a keyword and spelling pass follows as a separate entry; the study's specification findings S-1 to S-12 are resolved in the specification and reference programs; comments and identities across visual edits are settled with the IR format.
- **History:**
  - 2026-09-29 proposed by the syntax study.
  - 2026-09-29 accepted with amendment by the owner: grouped declaration blocks (one-line form kept, formatter groups), grouped flows, interval ranges, timeline shorthand; type inference from unit literals declined.

## D-029: Function values that read model bindings

- **Status:** Accepted
- **Original position:** MK-10.3: functions are pure and cannot read bindings other than their parameters and constants; a function that needs model state takes it as an argument. RP-06 defines `derived f : Real -> Real = fn(x) => a * x^2`, which reads the parameter `a`. MK 19.5 calls this expressible without addressing the conflict.
- **Raised by:** syntax study, finding S-1 (`docs/syntax-study/comparison.md` section 7).
- **Builds on:** R-13 (derived values are definitions), R-17 (expressions are pure), D-008 (dependency analysis), MK-10.6 (expressions keep their symbolic form).
- **Question:** May a function value depend on the model's bindings, and if so, where?
- **Options considered:**
  1. **Derived function values.** A derived binding may have a function type, and its body may read model bindings (`derived { f(x: Real): Real = a * x^2 }`). It is re-evaluated at every instant like any derived binding, so `f(2)` always uses the current `a`. Its dependencies are the bindings its body reads, as for any derived binding. MK-10.3 keeps applying to declared functions (`fn`), which stay closed. Stored bindings (parameters, state, constants) of function type may hold only closed functions.
  2. **Strict MK-10.3.** RP-06 becomes `fn f(a: Real, x: Real): Real = a * x^2`, and the presentation passes `a` in (`function_graph(x => f(a, x))`). The model then has no binding for "the function being studied"; the displayed formula is `f(a, x)`, not `f(x) = a x^2`, and lambdas move into presentations.
  3. **Drop MK-10.3.** Any function may read bindings. Library functions would carry hidden dependencies on the model that calls them, and a function reused in two models would mean different things in each.
- **Accepted position:** option 1.
- **Reason:** "the function f with parameter a" is how the mathematics is taught, and the learner's slider acts on `a` while the graph and formula follow; that needs `f` to be a model element that depends on `a`. Keeping it a derived binding reuses existing rules: no stored value, no capture semantics (it is recomputed every instant), dependencies visible to the analysis of MK section 13. Restricting stored function values to closed functions avoids the question of which instant's `a` a stored function would remember.
- **Consequences:** MK-10.3 is reworded to apply to declared functions; MK section 6 or 10 gains the rule for derived function values and the restriction on stored ones; a new static diagnostic for a stored binding holding a function that reads bindings; MK 19.5 cites this entry. RP-06 is unchanged in meaning.
- **History:**
  - 2026-09-29 proposed from syntax study finding S-1.
  - 2026-09-29 accepted by the owner. Applied to the model kernel (MK-10.3, MK-10.7, MK-E20, MK 19.5).

## D-030: Literal `0` adopts zero vectors

- **Status:** Accepted
- **Original position:** MK-3.8 (D-014): the bare literal `0` adopts whatever dimension its context requires. It does not say whether `0` may stand for a vector.
- **Raised by:** syntax study, finding S-3. Every candidate had to write `(0, 0)` or `(0 m/s, 0 m/s)` for a zero velocity or rate.
- **Builds on:** D-014, MK-4.4.
- **Question:** May the literal `0` stand for the zero of a vector type?
- **Options considered:**
  1. `0` adopts the zero of any required `Quantity<D>` or `Vector<S, D>`. It never stands for a `Point` or an `Instant`, which have no zero (their counterparts are `origin` and `t0`).
  2. Keep `0` scalar-only; vectors are written `(0, 0)`.
- **Accepted position:** option 1.
- **Reason:** the zero vector is as unambiguous as the zero quantity: it is the additive identity whatever the space and dimension, which the expected type supplies. `if flying then vel else 0` reads as the physics does. Points and instants are excluded because a zero there would silently mean "the origin", which D-014 and D-022 require to be explicit.
- **Consequences:** MK-3.8 extended; the reference programs and working syntax write `0` for zero vectors.
- **History:**
  - 2026-09-30 proposed from syntax study finding S-3 and accepted under the owner's standing delegation of 2026-09-30.

## D-031: Model variants in reference-program cases

- **Status:** Accepted
- **Original position:** reference-program README: a case lists parameter overrides and configuration settings. RP-03 case `B-stop` changes the event's Zeno policy, which is neither.
- **Raised by:** syntax study, finding S-4.
- **Builds on:** D-012, R-53 (no implied inheritance), RC section 16.
- **Question:** How does a reference program test a model that differs from its main model in structure?
- **Options considered:**
  1. **Named model variants in the reference-program format.** A program may list model variants, each with an ID and a stated change of named elements (like its diagnostic variants). A case names the variant it runs. Nothing is added to the language.
  2. **A language construct for variants** (a model defined as another model with replacements). This is a form of inheritance, which R-53 limits, and no first-slice program needs it outside testing.
  3. **Make the Zeno policy run configuration.** It is model meaning (what happens at accumulation), and RC-16.1 keeps configuration to settings that change accuracy or execution, not behavior.
- **Accepted position:** option 1.
- **Reason:** the need is a testing need, and the diagnostic variants already establish the pattern. A language construct can be proposed later if authors need model variants for teaching (comparing two models side by side).
- **Consequences:** the reference-program README gains a "Model variants" part; RP-03 declares variant `V1` (Zeno policy `stop`) and case `B-stop` runs it.
- **History:**
  - 2026-09-30 proposed from syntax study finding S-4 and accepted under the owner's standing delegation of 2026-09-30.

## D-032: Tuple literals as vectors: typing by expected type

- **Status:** Accepted
- **Original position:** MK section 4 defines points and vectors but not how a vector is written. The sketches use tuples, typed by a declaration (`param u : Vector<Plane, Length> = (3 m, 0 m)`) or by nothing visible (`pivot + L * (sin(θ), -cos(θ))`).
- **Raised by:** syntax study, finding S-5.
- **Builds on:** D-014, D-022 (explicit spaces), MK-4.5.
- **Question:** How does a tuple literal get its space and dimension?
- **Options considered:**
  1. **Expected type.** A tuple literal is a `Tuple` unless its context expects a vector, in which case it is that vector. The expected type comes from a declaration, a parameter of a called function, a flow target, or the other operand of `+`, `-` or a comparison, and propagates through scaling (the tuple in `L * (sin(θ), -cos(θ))` added to a `Point<Plane>` is expected to be `Vector<Plane, 1>`). Combining a `Tuple` with a vector where no expected type made it a vector is a static error.
  2. **Explicit constructors** (`Plane.vec(1 m, 2 m)`). Unambiguous, but noisy in every formula.
  3. **The model's default space** (D-028, `model Name in Space`) for every tuple. Fails silently in models with two spaces.
- **Accepted position:** option 1, with an explicit constructor available where no context supplies the type.
- **Reason:** the expected type is present wherever a physicist writes a vector (a declaration, a flow, an addition to a point), so the literal can stay mathematical while D-022's rule against implicit cross-space values still holds: the space always comes from a typed operand, never from a default.
- **Consequences:** MK section 4 gains the rule; new static diagnostic MK-E21 (tuple used as a vector with no expected vector type); MK 19.3 cites it.
- **History:**
  - 2026-09-30 proposed from syntax study finding S-5 and accepted under the owner's standing delegation of 2026-09-30.

## D-033: Order of run-directing actions within a beat

- **Status:** Accepted
- **Original position:** PK-9.1: a beat's actions start together. RP-08 has `seek(t0)` with a live formula in one beat, and `request(relaunch)` with `run(rate 1)` in another; the order in which they take effect is not specified.
- **Raised by:** syntax study, finding S-8.
- **Builds on:** D-009, D-027, RC section 12, PK-8.7.
- **Question:** When a beat starts, in what order do its actions that act on the lesson run take effect, and what do the other actions see?
- **Options considered:**
  1. **Written order for run-directing actions.** At a beat's start, its run-directing actions (`seek`, `reset`, `branch`, `intervene`, `request`, `run`, `hold`) take effect in the order written, at the same presentation instant and before any presentation time passes. The beat's other actions (show, narrate, animate, wait) then start together and see the result. `sequence` is needed only to order actions that take time.
  2. **Explicit sequence required.** Two run-directing actions in one beat without `sequence` are a static error.
  3. **Unordered.** Left to implementations. Breaks deterministic frames (PK-8.7).
- **Accepted position:** option 1.
- **Reason:** authors read a beat top to bottom, and the instantaneous run actions have an obvious intended order ("go back to the start, then play"). Applying them before presentation time passes keeps "actions start together" true for everything the learner sees, and keeps frames deterministic.
- **Consequences:** PK-9.1 gains the rule; RP-08 needs no `sequence` in `b4` and `b8`.
- **History:**
  - 2026-09-30 proposed from syntax study finding S-8 and accepted under the owner's standing delegation of 2026-09-30.

## D-034: Formula representation for expressions

- **Status:** Accepted
- **Original position:** PK-6.3 lists an `equation` representation typeset from an equation or expression, with symbols linked to bindings (PK-6.5). RP-06 shows a string (`"y = a x^2"`); RP-08 shows `R = v^2 sin(2θ) / g`, where `R`, `v` and `θ` are not bindings.
- **Raised by:** syntax study, finding S-10.
- **Builds on:** MK-10.6 (symbolic expressions), MK-6.1 (display symbol metadata), D-026 (text alternatives), D-029.
- **Question:** How does a presentation show a formula that is not a model equation, and how do its symbols relate to bindings?
- **Options considered:**
  1. **A `formula` representation.** `formula(e)` typesets an expression from its symbolic IR; `formula(f)` of a derived binding or derived function value shows its definition (`f(x) = a x²`); `formula("R", e)` adds a left-hand label, which is presentation text, not a binding. Symbols use each binding's display symbol (MK-6.1). `live` substitutes current values. Strings are never parsed as mathematics. `equation` stays the representation of a model equation (MK section 11).
  2. **Require a model equation** for everything shown. Forces lesson-only formulas into the model.
  3. **Typeset strings.** Loses the link to bindings, live values and generated text alternatives.
- **Accepted position:** option 1.
- **Reason:** every displayed symbol stays linked to a binding, so highlighting, live values and generated text alternatives work (D-026), without making lesson formulas part of the model.
- **Consequences:** PK-6.3 gains `formula`; PK-6.5 covers both; RP-06 shows `formula(f, live)`; RP-01's `speed` and `angle` declare display symbols `v` and `θ`; RP-08 shows `formula("R", speed^2 * sin(2 * angle) / g, live)`.
- **History:**
  - 2026-09-30 proposed from syntax study finding S-10 and accepted under the owner's standing delegation of 2026-09-30.

## D-035: Keywords and spelling of the working syntax

- **Status:** Accepted
- **Original position:** D-028 fixed the structure of the working syntax and left keywords and spellings to a later pass (`docs/syntax-study/comparison.md` section 8.2).
- **Raised by:** syntax study, open item "keyword and spelling pass".
- **Builds on:** D-028, R-46, D-002, D-019.
- **Question:** Which keywords and spellings does the working syntax use?
- **Options considered (per item):**
  1. Derived bindings: `derived` (kept) or `let` (rejected: in most programming languages `let` computes once, while a derived binding is re-evaluated at every instant) or `def`.
  2. Discrete state: `discrete` (kept) or `mode` (rejected: "mode" is the concept, not the binding, MK-14.4).
  3. Crossing triggers: `falling(g)`, `rising(g)`, `crossing(g)` (kept: the kernel terms, short, and they state the direction) or phrases such as `g falls through 0` (rejected: longer, and a second spelling per trigger).
  4. Microsteps in observations: `on E microstep n` (kept: one general form) or a named form such as `before handlers` (rejected: covers one case only).
  5. Equality: `==` for equations, constraints and expectations; `=` only gives a value (kept, D-028).
  6. `in`: a range after a declaration (`in [0, 1)`) and a target view in a timeline (`in scene { }`) (kept: the two contexts never overlap).
  7. Operators: ASCII only (`^`, `-`, `<=`, `->`). Unicode letters are allowed in identifiers (`θ`, `ω`, `π`). The formatter never introduces Unicode.
  8. Comments: `//` line comments; `///` documentation comments attached to the next element, carried into the IR (D-036).
  9. Constraint policy: `policy reject | report | stop`; equation role: `checked within tol`; parameter range: `where cond` or `in I`.
- **Accepted position:** the kept choices above. The full reserved-word list and lexical rules are in `docs/syntax-study/working-syntax.md` section 1.5.
- **Reason:** each choice keeps one spelling per construct (canonical printing, D-019), uses the kernel's own term where one exists, and avoids words whose common programming meaning contradicts the semantics.
- **History:**
  - 2026-09-30 proposed by the keyword pass and accepted under the owner's standing delegation of 2026-09-30.
  - 2026-09-30 the reserved-word list split into reserved words and contextual keywords (D-040).

## D-036: Comments and identities across text and visual editing

- **Status:** Accepted
- **Original position:** R-47 and D-019: the IR is complete and serializable; declared elements have identities stable across edits (MK-7.6). How identities and comments survive when the same model is edited as text and in Mava Studio was left open (`docs/syntax-study/comparison.md` section 8.3).
- **Raised by:** syntax study, round-trip analysis.
- **Builds on:** R-47, D-019, MK-6.2, MK-7.6, PK-2.3.
- **Question:** Where do element identities and author comments live, so that a text edit and a visual edit of one model both preserve them?
- **Options considered:**
  1. **Identities inline in the text** (`param g @id(4f2a) ...`). Always recoverable, but every line carries noise that authors must not touch.
  2. **A sidecar identity file** maintained by the compiler, mapping declaration paths to identities. Survives plain text edits except renames, and adds a second file to every model.
  3. **Identities in the IR, matched by declaration path.** The compiler matches each declaration to the previous IR by its path (`Projectile.param.speed`) and keeps its identity. A rename made through a tool (language server, Mava Studio) is an identity-preserving rename operation. A rename made by plain text editing is a removal and an addition; references that break are reported (PK-2.3), never silently rebound.
- **Accepted position:** option 3 for identities. For comments: `///` documentation comments and a comment block directly before a declaration are **author notes** of that element in the IR, and print back before it; other comments (inside expressions, at end of line) are kept by the text formatter but may be lost by a visual edit of that element, which the editor reports before saving.
- **Reason:** keeps source text free of machine data (D-002 authors), keeps the IR authoritative (R-47), and turns the failure case (a plain-text rename) into a reported breakage instead of a silent change of meaning. Notes attached to elements are what a visual editor can show and keep.
- **Consequences:** the IR format (`docs/spec/04-ir.md`) gives every declared element an identity and a `notes` field; the compiler keeps the previous IR to match paths.
- **History:**
  - 2026-09-30 proposed and accepted under the owner's standing delegation of 2026-09-30.

## D-037: IR serialization

- **Status:** Accepted
- **Original position:** D-019 and R-47 require a complete, serializable IR; the format was left to a separate specification.
- **Raised by:** IR format specification (`docs/spec/04-ir.md`), before the Rust prototype.
- **Builds on:** D-019, D-036, R-47.
- **Question:** How is the IR serialized?
- **Options considered:**
  1. **JSON**, versioned, with opaque string identities and tagged expression trees. Readable in every language, native in the browser (D-018, the web player and Mava Studio), diffable in version control, supported by serde in Rust.
  2. **A binary format** (CBOR, Protocol Buffers). Smaller and faster, but not readable or diffable, and needs a schema toolchain in every consumer.
  3. **The text syntax itself as the stored form.** Rejected by D-036: text carries no identities or notes reliably, and a visual editor would need the parser.
- **Accepted position:** option 1. A binary encoding of the same structure MAY be added later for large models; JSON stays the reference form.
- **Reason:** the IR's first consumers are the web player and Mava Studio, both in the browser, and authors reviewing changes in version control. Size is not a first-slice concern.
- **History:**
  - 2026-09-30 proposed and accepted under the owner's standing delegation of 2026-09-30.

## D-038: Self-retriggering is decided through flows, specialized on the handler's discrete values

- **Status:** Accepted
- **Original position:** MK-15.11: an event is self-retriggering if its handler writes a binding its trigger depends on "directly or through derived bindings". MK 19.2 and RP-03 say the bouncing ball's `bounce` is self-retriggering because its guard `y` depends on `v` "through the flow".
- **Raised by:** the Rust kernel prototype. The two readings disagree on the reference programs: following derived bindings only, `bounce` is not self-retriggering and RP-03.D1 (expects MK-E16) fails; following flows as well, RP-01's `landed` (which writes `vel` and `flying`, both read by the flow of `pos`) is self-retriggering and the valid RP-01 model is rejected.
- **Builds on:** D-004 (declared Zeno policy), MK-13 (dependency graph), MK-14.12 (flow conditions depend only on discrete state).
- **Question:** When must a crossing event declare a Zeno policy?
- **Options considered:**
  1. **Derived bindings only** (the text of MK-15.11). Misses the bouncing ball, the canonical Zeno case.
  2. **Derived bindings and flows.** Catches the bouncing ball, but also flags every event that stops the motion it detects (a landing that switches the flight mode off), which cannot retrigger.
  3. **Derived bindings and flows, with flows specialized on the handler's constant discrete writes.** Where the handler sets discrete state to a constant (`set flying = false`), conditionals in flows that test that state are resolved before following the flow. The projectile's `der(pos)` becomes `0` after `landed`, so `pos.y` no longer depends on anything the handler writes; the bouncing ball's `der(y) = v` does not depend on discrete state, so `bounce` is still self-retriggering.
- **Accepted position:** option 3. The analysis stays static and conservative: when a condition cannot be resolved, both branches are followed.
- **Reason:** it gives the intended answer on every reference program and matches the physics: an event is self-retriggering only if the state after its handler can still carry the guard back across zero. Because flow conditions may read only discrete state, parameters and constants (MK-14.12), the specialization is always well defined.
- **Consequences:** MK-15.11 is reworded; the prototype implements it (`self_retriggering` in `crates/prismal-kernel/src/check.rs`); RP-01 and RP-03 are unchanged.
- **History:**
  - 2026-09-30 found by the prototype, proposed and accepted under the owner's standing delegation of 2026-09-30.

## D-039: Named mathematical constants in the IR

- **Status:** Accepted
- **Original position:** the working syntax predefines `π` and `pi` (section 1.5) and writes `2π` as a product (RP-05); the IR (`docs/spec/04-ir.md` section 6) has no form for a named constant, so lowering would store π as a decimal literal.
- **Raised by:** the text parser, lowering RP-05's `T: Time = 2π * sqrt(m / k)`.
- **Builds on:** MK-10.6 (expressions keep their symbolic structure), D-034 (the formula representation typesets from the IR), IR-1.1.
- **Question:** How does the IR store a named mathematical constant?
- **Options considered:**
  1. **As its value** (`{"num": 3.141592653589793}`). No IR change, but the formula for RP-05's period shows `6.28319 √(m/k)` and cannot be printed back as `2π`.
  2. **As a named constant** `{"const": "pi"}`: dimensionless, with MK-3.8 applying as to a bare literal. One more expression form; typesetting and printing keep π.
  3. **As a predefined binding** of every model. Mixes a mathematical constant with model state: it would enter dependency analysis and could be a target of interventions.
- **Accepted position:** option 2, with `pi` the only constant in v0. Others are added when a program needs them; `e` is not predefined because it is a common binding name (the restitution in RP-03).
- **Reason:** keeps the symbolic structure that MK-10.6 and D-034 require, at the cost of one small expression form. Evaluation is unchanged: `2 * π` in binary64 equals the folded value, so the measurements of RP-05 are identical.
- **Consequences:** 04-ir section 6 gains the form, MK-10.8 states the rule, the kernel checks it as a dimensionless literal. `inf` is not a constant: it is only an interval bound, and an unbounded end lowers to a one-sided comparison, so the IR never holds an infinity (which JSON cannot represent).
- **History:**
  - 2026-09-30 found by the text parser, proposed and accepted under the owner's standing delegation of 2026-09-30.

## D-040: Reserved words and contextual keywords

- **Status:** Accepted
- **Original position:** D-035 and working syntax section 1.5 reserve every keyword of the language, including those of presentations, timelines and runs (`drag`, `scene`, `rate`, `until`, `limit`, `release`, ...).
- **Raised by:** the text parser. The reference programs use reserved words as names: RP-01 declares `process drag`, RP-07 and RP-08 name a view `scene`, RP-07 names an observation `state`. Read literally, D-035 rejects all three.
- **Builds on:** D-035, D-028, D-002 (authors are scientists and educators, whose ordinary names include drag, scene, rate, limit and release).
- **Question:** Which keywords are reserved?
- **Options considered:**
  1. **Every keyword reserved**, and the elements of the programs renamed (`air_drag`, `stage`). One simple rule, but ordinary scientific words become unusable as names, and the list grows with every presentation feature, breaking existing models.
  2. **Reserve the words of the model language, expressions and top-level items; recognize presentation, timeline and run words only where such a word is expected** (contextual keywords, as in C# and Kotlin). No ambiguity arises: those words never occur where a model expression or the name of a model element can start, and the places that expect them (the actions of a beat, the schedule of an observation, the items of a run) never hold a name.
  3. **Any word may be a name wherever the grammar expects a name.** Loses the guarantee that a model reads unambiguously to a person (`param { in: Real = 1 }`).
- **Accepted position:** option 2. Reserved: `space model presentation run object const param input state discrete derived fn flow process event equation constraint on if then else and or not otherwise in where true false zeno stop settle set contribute create destroy connect disconnect emit enter checked within policy reject report intervenable private symbol unit rising falling crossing at every from start request`. Every other keyword is contextual (working syntax section 1.5). The observation `state` of RP-07 is renamed `values`, because `state` begins a model block.
- **Reason:** keeps the model language unambiguous for readers and the parser, and leaves natural scientific names to authors. The contextual list can grow with the presentation kernel without breaking existing models.
- **Consequences:** working syntax section 1.5 lists the two groups; RP-07 changed (recorded in its history); the parser implements the split (`crates/prismal-syntax/src/parser.rs`).
- **History:**
  - 2026-09-30 found by the text parser, proposed and accepted under the owner's standing delegation of 2026-09-30.

## D-041: `on(E)` follows an occurrence of `E` as well as an emission

- **Status:** Accepted
- **Original position:** MK-15.3 defines the trigger `on(E)` as due when "event `E` occurred at the previous microstep". MK-15.10 says `emit(E)` makes `on(E)` triggers due at the next microstep, and RC section 8.1 step 1 collects "`on(E)` for each `E` emitted at `n - 1`". The prototype implemented the runtime contract: `on(E)` followed only emissions.
- **Raised by:** the language guide (`docs/guide/05-events-and-modes.md`). The tank's `event alarm on full` never happened: `full` occurred through its own trigger but was not emitted. The texts contradict each other for any event that occurs without being emitted.
- **Builds on:** D-004 (cascades at one instant through superdense time), D-024 (ordering only by cascades), MK-15.10.
- **Question:** When is `on(E)` due?
- **Options considered:**
  1. **Only after `emit(E)`** (the runtime contract). Chaining one event after another requires every handler to emit its own name (`event full ... { set pumping = false; emit full }`), which is redundant and easy to forget; MK-15.3 would be reworded.
  2. **After `E` occurs or is emitted.** `on(E)` reads as written: after `E`. `emit(E)` remains the way to signal `E` from another handler without `E`'s own trigger. One occurrence and one emission at the same microstep make `on(E)` due once.
  3. **`emit(E)` makes `E` itself occur** (with its handler), and `on(E)` follows occurrences only. Changes the meaning of `emit` established by MK-15.10 and the prototype's cascade tests, and runs `E`'s handler when another event only meant to signal it.
- **Accepted position:** option 2.
- **Reason:** it is the reading of MK-15.3 an author expects, it keeps MK-15.10, and it makes D-024's rule (order is expressed by cascades) usable without boilerplate.
- **Consequences:** MK-15.10 and RC section 8.1 step 1 state both sources; the runtime collects `on(E)` for events handled or emitted at the previous microstep (`crates/prismal-runtime/src/run.rs`, test `on_follows_occurrence`). An event triggered on itself (`event a on a`) cascades until the cascade limit, as an endless chain of emissions already did.
- **History:**
  - 2026-09-30 found by the language guide, proposed and accepted under the owner's standing delegation of 2026-09-30.

## D-042: Animations in v0 are named effects

- **Status:** Accepted
- **Original position:** PK-8.4 defines an animation as a change of a presentation property over presentation time, from a value to a value with a duration and easing; PK-9.2 lists `reveal(style)`, `animate`, `camera`, `bind` and `release`. The working syntax had no spelling for any of them.
- **Raised by:** the remaining timeline actions of the prototype (`docs/prototype.md`, "Not implemented").
- **Builds on:** D-009, D-018 (video export), PK-8.5 (animations never drive bound properties), PK-8.7 (deterministic frames).
- **Question:** How are animations written in v0?
- **Options considered:**
  1. **A general property animation**, `animate ball.opacity from 0 to 1 for 1 s ease in_out`, as in Motion Canvas and the Web Animations model. Complete, but it needs a presentation property model (which properties each representation has, their types and defaults) that the specification does not have yet, and most lessons use a few effects.
  2. **Named effects:** `reveal fade|draw [for d] [in view] { reps }`, `hide name [for d]`, `camera view [to P] [zoom z] [for d]`, with one easing. They cover the typical uses PK-8.4 lists (reveal by fade or drawing, fading out, camera moves), need no property model, and read as the lesson's intent.
  3. **Defer all animation.** Lessons stay static apart from the simulation; video output (D-018) loses the effects authors expect from Manim-like tools.
- **Accepted position:** option 2. `animate`, `bind` and `release` remain unspecified until a program needs them; they can be added as option 1 later without changing the named effects, which become shorthands.
- **Reason:** delivers the effects lessons need now, keeps frames deterministic and the syntax readable, and leaves the general mechanism open.
- **Consequences:** PK-9.2c; working syntax section 1.2; the IR actions `reveal`, `hide` with a duration and `camera` (04-ir section 7); frame descriptions carry `opacity`, `drawn` and a view `camera`; the web player renders them.
- **History:**
  - 2026-09-30 proposed and accepted under the owner's standing delegation of 2026-09-30.

## D-043: A group places its members by a shared transform in model space

- **Status:** Accepted
- **Original position:** PK-6.3 lists `group`, "a set of representations with a shared transform", without saying what the transform is, what a group may hold, or how a group is written. The prototype did not implement it.
- **Raised by:** the next steps after D-042 (`docs/PROJECT-STATE.md`); rigid bodies (a wheel, a pendulum drawn as a body, a car) need a shape drawn once in its own coordinates and placed by the model.
- **Builds on:** D-021 (angles are numbers), D-022 (points and vectors of a space), PK-6.1 (representations are not pixels), PK-11.1 (a group's text alternative summarizes its members), D-042 (timeline effects).
- **Question:** What is a group's transform, where is it applied, and what may a group hold?
- **Options considered:**
  1. **A transform in view coordinates**, as SVG's `transform` on `<g>`: members are projected as usual, then moved on screen. Simple for renderers, but the members' text alternatives and any value a renderer or assistive technology reads would be in the group's local coordinates, not in the space the view shows.
  2. **A rigid transform with scale, applied in model space:** `group(at: P, rotate: θ, scale: k) { members }`. A member's points `p` become `P + k R(θ) (p - origin)` and its vectors `v` become `k R(θ) v` before projection, so a member is an ordinary representation of points of the view's space. Nested groups compose. Text alternatives and frame values are in the view's space.
  3. **A general affine or projective transform** (shear, reflection, a matrix). More than any first-slice program needs, and a matrix is harder to read than a placement, an angle and a scale.
- **Accepted position:** option 2. `at` is a point of the view's space (default: the group's `origin` stays where it is), `rotate` an angle (default 0, counterclockwise in the model), `scale` a positive number (default 1). A group belongs in a spatial view and holds markers, arrows, segments, polylines, polygons and groups; a sampled representation (`trace`) is drawn from a model point outside the group, since it samples over time while the transform is taken at the instant shown. Members may be named and targeted by timeline actions; `reveal draw` of a group draws its paths and fades in its markers. A drag on a member (an inverse through the transform) is specified by composing the member's inverse with the inverse transform, but not implemented by the prototype (PK-E06).
- **Reason:** keeps members in the space the view shows, so text alternatives, frame values and tests agree with the model; reads as the placement of a body; covers rigid bodies with no new representation kinds.
- **Consequences:** PK-6.3b; working syntax section 1.2; the IR `members` of a representation (04-ir section 7.1); the frame shape `group` with placed members; the web player draws groups; guide chapter 7 (Groups).
- **History:**
  - 2026-09-30 proposed and accepted under the owner's standing delegation of 2026-09-30.

## D-044: Prismal is a general embeddable system

- **Status:** Accepted
- **Original position:** D-019 names Mava Studio as the editor for non-programmer authors and asks that the web player be embeddable in it. Nothing said whether Prismal's integration surface is designed for Mava Studio or for any system.
- **Raised by:** the next step after D-043, "Mava Studio groundwork", which could not be designed without knowing Mava Studio's architecture.
- **Builds on:** D-018, D-019, D-036, D-037.
- **Question:** Who is Prismal's integration surface designed for?
- **Accepted position (owner):** Mava Studio is a consumer of Prismal, and its new architecture is not decided (it will be a Rust stack; authors will work through a graphical interface or code, and output will render inside the studio, probably with the Prismal runtime embedded). Prismal's design is general, so that any system can plug it in and play: other organizations may integrate it into their own systems. No Prismal interface assumes a particular host.
- **Reason:** the owner's direction; Mava Studio's design is open, and other integrators are expected.
- **Consequences:** the host interface (`docs/spec/05-host-interface.md`, D-045); the web player becomes one host of that interface; D-019's consequences hold for every host, not only Mava Studio.
- **History:**
  - 2026-09-30 stated by the owner: "The design for prismal should simply be general for systems to plug and play since there could be other societies that would need to integrate into there systems not necessarily mava studio."

## D-045: The host interface

- **Status:** Accepted
- **Original position:** the only integration surface was the web player's `Player` (`prismal-web`): one program from source text, one open presentation, JSON answers shaped for the reference renderer, and each loaded program kept in memory for the life of the page.
- **Raised by:** D-044.
- **Builds on:** D-036 (identities and `reconcile`), D-037 (JSON IR), PK-12.1 (frame descriptions), PK-8.7 (deterministic frames).
- **Question:** What interface do hosts use, and through which bindings?
- **Options considered:**
  1. **A Rust library only.** Natural for Rust hosts; every other host (a browser, another language) would need its own wrapper, each exposing something slightly different.
  2. **An editor protocol of fine-grained edit operations** (insert, move, change a field). Commits Prismal to one style of editor before any editor is designed; a host that edits text or rewrites the IR gains nothing from it.
  3. **An engine with handles, offered as a Rust API and as one JSON protocol through every binding.** Documents are loaded from text or IR and updated by whole replacement, with identities kept by `reconcile` and an identity-keeping `rename`; presentations open as instances driven by the host's clock and the learner's inputs; answers are layouts, frame descriptions and observations. Bindings (in-process Rust, WebAssembly, later a C ABI or a process on standard input and output) carry the same protocol. Follows the Language Server Protocol and FMI.
- **Accepted position:** option 3, under the owner's standing delegation of 2026-09-30. Fine-grained edit operations (option 2) can be added to the protocol later if a host needs them; whole replacement with `reconcile` serves both text and visual editing now.
- **Reason:** any host can embed Prismal without Prismal knowing it; one protocol keeps bindings identical; whole-document updates work for editors of any design.
- **Consequences:** `docs/spec/05-host-interface.md` (HI-1 to HI-6, HI-E01 to HI-E03); the `prismal-host` crate; `prismal-web` reduced to the WebAssembly binding and the reference renderer; sessions and playbacks own their compiled model, so documents can be closed and freed.
- **History:**
  - 2026-09-30 proposed and accepted under the owner's standing delegation of 2026-09-30.

## D-046: Formulas are typeset for every medium

- **Status:** Accepted
- **Original position:** D-034 and PK-6.5 require formulas to be typeset from the IR. The prototype typeset them only as MathML, which browsers draw and no other medium does, and the host interface (HI-5.2) put that MathML in every frame. PK-12.2 names still image, video and vector documents as media alongside the web player.
- **Raised by:** the owner's review of whether rendering is swappable to other targets, and the owner's requirement that formulas remain visible outside the browser.
- **Builds on:** D-018, D-034, PK-6.5, PK-12.1, PK-12.2, D-045.
- **Question:** How does a medium without a math engine draw a formula?
- **Options considered:**
  1. **Leave typesetting to each renderer**, from the symbolic IR in the frame. Every renderer re-implements math layout, and the same formula looks different in each medium.
  2. **A TeX string** in the frame. Readable by math engines, but most media (images, video, native interfaces) have none, so it moves the problem rather than solving it.
  3. **A math box tree and its layout in the core.** The presentation kernel builds a medium-independent box tree from the IR and places it as text runs, rules and stroked paths in em units, with approximate serif metrics and each run's width, so any renderer that draws text and lines draws the formula. A medium with a math engine may typeset the box tree instead; the browser binding writes it as MathML, so both come from one structure.
- **Accepted position:** option 3, under the owner's standing delegation of 2026-09-30. The layout is in `frame::Shape::Formula` and `Equation` as `layout`; MathML leaves the host interface for the WebAssembly binding. Frame descriptions also gain what the host used to add after the fact (labels, drag parts, control symbols and display units), so that they are complete for every renderer.
- **Reason:** one typesetting for all media keeps formulas consistent and visible everywhere, without a math engine in each renderer; the frame description stays free of any one medium's format.
- **Consequences:** PK-6.5a, PK-12.1a, HI-5.2; `prismal-present/src/math.rs`; `prismal-web/src/mathml.rs` renders the box tree; `prismal-present/tests/math.rs` checks layouts and writes them as SVG for inspection. Layout metrics are approximate: a renderer with a different font fits each run to its width.
- **History:**
  - 2026-09-30 proposed and accepted under the owner's standing delegation of 2026-09-30.

## D-047: The host captures input; the engine interprets it

- **Status:** Accepted
- **Original position:** HI-4.3 offered only semantic inputs: `pointer_down(rep, part)`, `pointer_move(x, y)` in view coordinates, `key(rep, direction)`. The host had to find what a pointer was over, convert screen positions to view coordinates, and keep keyboard focus. The web player did so with the browser's DOM, and framed its views (extent, growth to fit, camera, zoom and pan) in its own code, which the SVG renderer then had to copy.
- **Raised by:** the SVG renderer (a second renderer duplicated the framing logic), and the owner's question of how hosts other than browsers handle input, with the concern that Prismal must not take over the capture of events.
- **Builds on:** D-025, D-026, D-042, D-044, D-045, PK-10.2, PK-11.2, PK-12.2.
- **Question:** Who targets input, frames views and keeps focus: each host, or the engine?
- **Options considered:**
  1. **Each host** (the position before). Every host re-implements hit testing, viewport mapping, zoom and pan and focus order from the frame; the same lesson then behaves differently in each host, and PK-12.2 (a renderer defines no representation semantics) is broken in practice, since what can be grabbed and how is a presentation rule.
  2. **Prismal captures input** (listeners registered in the host's event loop). Couples Prismal to every windowing system and toolkit, and takes control a host must keep (HI-1.4).
  3. **The host captures and forwards; the engine interprets.** The host catches pointer, wheel and key events in its own layer and forwards them in its own terms (pixels of a view as it drew it, the drawn size, the pointer kind, W3C key names). The engine targets them (PK-10.2a), runs drags, pans and zooms, keeps focus (PK-11.2b), and answers what it did and whether it used the input, so that the host can use what it did not. Frames carry each view's viewport, which every renderer draws with. Semantic inputs stay for hosts that target input themselves (native widgets, accessibility trees).
- **Accepted position:** option 3, under the owner's standing delegation of 2026-09-30, after the owner confirmed that the host keeps the capture of events. Protocol operations `pointer`, `wheel`, `key_down`, `focus` and `view_reset` (HI-4.5); viewports in frames (HI-5.2a). Panels (controls, buttons, formulas outside a view's coordinates) are laid out by the host, so pointers on them are the host's widgets' input, forwarded semantically; keyboard focus reaches them through the focus order.
- **Reason:** one interpretation of input for every host keeps behavior identical across media and moves the hard part (targeting, viewport mapping, focus) out of every integrator's hands, while the host keeps its event loop and its devices.
- **Consequences:** HI-1.4, HI-4.5, HI-5.2a, HI-6.5; PK-10.2a, PK-11.2b; `prismal-host/src/input.rs` and the raw operations of `Instance`; the web player forwards raw events and draws with the frame's viewport (its own framing, hit testing and zoom and pan code removed); the SVG renderer draws with the frame's viewport. A session's framing grows to fit each frame without memory of earlier frames, so the same instant gives the same frame (HI-4.2).
- **History:**
  - 2026-09-30 proposed and accepted under the owner's standing delegation of 2026-09-30, after the owner's direction: "my worry was us doing the actual event catching".

## D-048: Declared functions in v0

- **Status:** Accepted
- **Original position:** MK-10.3 specifies declared functions (typed parameters, a result, a closed body) and the working syntax lists `fn f(x: A): B = e`, but the v0 IR had no form for them and the parser rejected `fn` (SX-E06). Authors could only write derived function values (MK-10.7), which read model bindings and cannot be shared between formulas without repeating them.
- **Raised by:** the language completeness step after D-047 (owner's direction to complete the language items listed as not implemented).
- **Builds on:** MK-10.3, MK-10.7, D-029, D-034, D-036, D-037.
- **Question:** Where are functions declared, what may their bodies read, and how are they represented and called?
- **Options considered:**
  1. **Lower `fn` to a constant binding holding a lambda.** No new IR form, but a function would appear as a binding (in controls, observations and intervention checks), and constants of function type already carry the closedness rule MK-E20 with a different meaning.
  2. **Model-level declarations with their own IR list and a function-value expression.** `functions: [{id, name, params, result, body}]`, `{"fn": id}` as a value, applied with the existing `apply`. The body reads parameters as lambda parameters; the checker enforces closedness (MK-E18) and forbids recursion (MK-E23).
  3. **Document-level function libraries shared across models.** Useful later (R-55, modules), but needs imports and packaging, which are deferred.
- **Accepted position:** option 2, under the owner's standing delegation of 2026-09-30. A function body may read its parameters, constants and other declared functions; reading any other binding, the time or a derivative is MK-E18, so a function's meaning never depends on when it is called. Recursion, direct or mutual, is MK-E23: with no conditionals over unbounded data in v0, recursion would only loop. Functions share the namespace of bindings (SX-E09). `formula(f)` shows the definition. Constants are available wherever constant expressions are evaluated (case parameters, expectations), so a function that reads `g` can be called there.
- **Reason:** keeps functions distinct from bindings in every tool that lists bindings, uses the application form the kernel already has, and keeps each function a self-contained piece of mathematics that can be typeset and reused.
- **Consequences:** MK-10.3a, MK-E23; 04-ir sections 5.5 and 6; working syntax `fn`; `CModel::functions`, `CModel::constant_values`; formatter, identity matching and rename for functions; guide chapter 2 (Functions).
- **History:**
  - 2026-09-30 proposed and accepted under the owner's standing delegation of 2026-09-30.

## D-049: Enumerations in v0

- **Status:** Accepted
- **Original position:** MK-2.2 makes enumerations nominal and MK-10.2 requires `match` to cover every case, but the v0 IR had an anonymous enumeration type (`{"kind": "enum", "cases": [...]}`), no declaration, no `match`, and the working syntax no spelling. Modes with more than two values had to be coded as numbers or several Booleans.
- **Raised by:** the language completeness step after D-047.
- **Builds on:** MK-2.2, MK-10.2, D-036, D-037, D-038.
- **Question:** How are enumerations declared, how is a case written, and how is a value chosen by case?
- **Options considered:**
  1. **Structural enumeration types** (the type is its list of cases). Simple, but two unrelated enumerations with the same cases would be interchangeable, contrary to MK-2.2.
  2. **Declared, nominal enumerations whose type carries the declaration's identity and its cases.** `enum Phase { a, b }` in a model; the type is `{"kind": "enum", "enum": id, "cases": [...]}`, so equality is by identity while every value can still be printed by name without a lookup. A case is written by name and typed by its context; `Phase.a` qualifies it. `match e { a => x, b => y }` has exactly one arm per case.
  3. **Enumerations with payloads** (tagged unions). Specified in MK section 2, but no reference program needs them yet and they need pattern binding in `match`.
- **Accepted position:** option 2, under the owner's standing delegation of 2026-09-30; payloads deferred. Cases have no order and no arithmetic (`==`, `!=` only). Cases share the namespace of bindings and functions. `enum` and `match` become reserved words; `=>` is added to the operators. The self-retriggering analysis of D-038 decides `phase == c` and `phase != c` on the cases a handler sets.
- **Reason:** gives modes names a learner can read in text alternatives and formulas (`phase = sinking`, one typeset row per case), and makes forgetting a case a static error.
- **Consequences:** MK-2.2a, MK-10.3a (match), MK-E17 and MK-E22 for enumerations; 04-ir sections 3, 5.5 and 6; working syntax `enum`, `match`, `=>`; `CExpr::Match`; expectations type an expected case from their subject; formatter, identity matching and rename for enumerations; guide chapter 5 (Modes with more than two values).
- **History:**
  - 2026-09-30 proposed and accepted under the owner's standing delegation of 2026-09-30.

## D-050: Event payloads in v0

- **Status:** Accepted
- **Original position:** MK-15.1 gives events an optional typed payload, readable by `on(E)` triggers and supplied by requests and `emit(E, payload)`; the IR listed a payload type and the working syntax `on request (payload: T)`, but nothing said how a handler names the payload, where an event that is not requested gets one, and the parser rejected payloads (SX-E06).
- **Raised by:** the language completeness step after D-049.
- **Builds on:** MK-15.1, MK-15.3, MK-15.10, D-027, D-041.
- **Question:** Where is a payload declared, where does it come from, and how is it read?
- **Options considered:**
  1. **A payload per event, readable anywhere as `E.payload`.** Simple to write, but a crossing event has no value to carry, and reading `E.payload` elsewhere has no meaning between occurrences.
  2. **A payload declared where it enters the event, with a name.** `on request(j: Momentum)` receives it from the request; `on E(j: Momentum)` receives the payload `E` occurred or was emitted with. The name is read in the event's condition and handler only. `emit E(v)` supplies the payload of `E`'s followers. An event with a payload and no such source is MK-E24.
- **Accepted position:** option 2, under the owner's standing delegation of 2026-09-30. Requests obey enabling conditions (they did not before, contrary to MK-15.5), supply the payload the event declares, and are rejected otherwise. The event log records each occurrence's payload. Hosts request with a payload through the runtime's `RequestWith` action and timelines with `request E(v)`; buttons request without one.
- **Reason:** every payload has a source a reader can find in the text, types are checked where the value enters, and nothing is readable where it has no value.
- **Consequences:** MK-15.1a, MK-E24, RC-11.6b; 04-ir payload forms; `CExpr::Payload`, `Ctx::payloads`; runtime event iteration carries payloads between microsteps; guide chapter 5.
- **History:**
  - 2026-09-30 proposed and accepted under the owner's standing delegation of 2026-09-30.

## D-051: Inputs in v0

- **Status:** Accepted
- **Original position:** MK-6 and RC-11.6 define `input` bindings, supplied by the environment, piecewise constant, `unavailable` until supplied, with `on input(i)` triggers. The prototype refused any model with an input, and MK-6.7 forbids an input an initial definition, so a model with an input could not start until something outside supplied every input at the start.
- **Raised by:** the language completeness step after D-049; the host interface (D-044, D-045), whose hosts are the environment of an embedded model.
- **Builds on:** MK-6.7, MK-17.3, RC-11.6, RC-11.7, D-045.
- **Question:** How does an input get its first value, how do runs and hosts change it, and what is due when it changes?
- **Options considered:**
  1. **As specified: unavailable until supplied.** Every expression reading an input would be `unavailable` at the start, so derived bindings and flows could not be evaluated and the run could not begin without an explicit starting value for every input.
  2. **An optional default in the model, a starting value from the run configuration, and logged changes.** `input { thrust: Force = 0 N }`; a run's `input { thrust = 4 N; thrust = 0 N at t0 + 1 s }`; a host's `set_input`. A value supplied at a later instant is a change and makes `on input(i)` due; the starting value is not a change.
- **Accepted position:** option 2, under the owner's standing delegation of 2026-09-30; a departure from MK-6.7, recorded as MK-6.7a. Without a default or a starting value the run does not start, with a message naming the input. Values are evaluated against the input's type and read only constants; a wrong type or a binding that is not an input is rejected and logged. Inputs stay outside interventions: no control, handler or intervention sets them.
- **Reason:** a model can be written, checked and run standalone with sensible defaults, and the same model embedded in a host receives the host's values; each change is logged, so runs replay without the host (RC-11.7).
- **Consequences:** MK-6.7a, RC-11.6a; the runtime's `Input` action; 04-ir run `inputs`; HI-4.3a `set_input`; guide chapter 5.
- **History:**
  - 2026-09-30 proposed and accepted under the owner's standing delegation of 2026-09-30.

## D-052: Video export

- **Status:** Accepted
- **Original position:** PK-12.2 lists video among the media a renderer produces, and PK-8.7 makes frames deterministic so that video export is possible; how pixels and video frames are produced is left to renderers (PK section 1). The SVG renderer (`prismal-svg`) wrote image sequences of SVG documents and left encoding to external tools.
- **Raised by:** the media step after the second renderer (PROJECT-STATE next steps).
- **Builds on:** PK-8.7, PK-9.10, PK-11.3, PK-12.2, PK-12.3, D-015, D-018, D-046.
- **Question:** How does a presentation become a video file, and what does Prismal own in that path?
- **Options considered:**
  1. **Encode in process with a codec written in Rust.** One binary with no outside program, but a pure Rust encoder of a widely played codec (H.264) does not exist; AV1 encoders are slow and large, and codec and container work is outside the language's specification.
  2. **Rasterize in process, encode with an external encoder fed through a pipe.** Frames are drawn by the SVG renderer and rasterized with resvg (pure Rust); raw frames go to ffmpeg's standard input. Without an encoder, the frames are written as PNG images with the captions and the encoder command.
  3. **Keep writing SVG sequences only.** Every user rasterizes and encodes; results depend on the tools chosen and on their fonts, and captions are lost.
- **Accepted position:** option 2, in a crate `prismal-media` with a command `prismal-media PROGRAM PRESENTATION OUT`, under the owner's standing delegation of 2026-09-30. The output's extension decides the medium: `mp4`, `mov`, `mkv`, `webm` or `gif` for a video, `png` for a still, anything else for a directory of frames. Settings:
  - A lesson is opened in the `video` medium: explore beats play their fallbacks (PK-9.10) and elements without one are reported on standard error (PK-12.3). A session is recorded from the start of its run to its end or to `--until`.
  - Frame `k` is the instant `k / fps`, up to and including the last instant not after the end, so the final state is shown.
  - Frames of one clip may differ in size (a caption or a panel appears); every frame is drawn from its top left corner on one canvas, the largest frame rounded up to even pixels, filled with the theme's background, so views stay in place.
  - Captions (PK-11.3) are drawn into the frames by default, or carried only as a subtitle track (`--captions track`), or both; they are always written beside the video as WebVTT.
  - The SVG renderer's header line is off: it names the instant, which a video shows by playing.
  - Generic font families resolve to the first installed of a list of common fonts; `--fonts DIR` adds fonts, so an export can be made identical across machines.
- **Reason:** Prismal owns what its specification defines (the frames, their instants, fallbacks, captions) and nothing of codecs and containers; a pipe to one widely installed encoder gives every common format, and the frame directory keeps export possible with no encoder at all.
- **Consequences:** `crates/prismal-media`; `docs/prototype.md` media section; guide chapter 8 (Exporting a video). Not yet done: narration audio (the language carries narration text only; when recorded or synthesized audio is added, its cues are placed at the captions' start times and muxed as an audio track), a descriptions track from announcements, and a layout of several views on one page (PK-7.4).
- **History:**
  - 2026-09-30 proposed and accepted under the owner's standing delegation of 2026-09-30.
  - 2026-09-30 narration audio settled by D-053: sound is supplied by hosts, video export included, not by the language.

## D-053: Narration sound belongs to hosts

- **Status:** Accepted
- **Original position:** PK-9.2 makes a narration cue "caption text and, optionally, recorded or synthesized audio" that "ends when the audio or the reading time ends", and PK section 15 defers narration audio. D-052 left video without sound, expecting audio to enter the language.
- **Raised by:** the owner, after D-052: sound should not be a language feature; and it is needed wherever captions play (the web player and other hosts), not only in video export.
- **Builds on:** PK-8.7, PK-9.2, PK-9.2b, PK-11.3, D-026, D-045, D-052.
- **Question:** Where does narration sound come from, how does a host find the sound of a cue, and what decides a cue's timing?
- **Options considered:**
  1. **Audio in the language** (`narrate "..." audio "b3.mp3"`), the audio's length ending the cue as PK-9.2 says. Programs would name media files, a lesson's timing and its cases would depend on assets outside the program, and every host would need the files at the same paths.
  2. **Sound only in video export.** Keeps the language free of media, but every other host that shows captions would invent its own way to voice them.
  3. **Sound supplied by hosts, keyed by cue names the engine gives every host.** The program carries text only; each caption cue is named after the beat that narrates it; a voice is a set of recordings named by cue, or a speech synthesizer for cues without one. The timeline keeps deciding when a cue starts and how long it lasts.
- **Accepted position:** option 3, the owner's direction of 2026-09-30, with the details under the owner's standing delegation. A departure from PK-9.2, recorded as PK-9.2d:
  - A cue's name is its beat's name, and `beat.2`, `beat.3` ... for the later narrations of the same beat (beat names are unique in a presentation and are what cases refer to, so names survive edits elsewhere in the lesson). Layouts list captions with `cue`, `start`, `end` and `text` (HI-5.1a); WebVTT files use the names as cue identifiers.
  - A recording is a file named after its cue (`b3.wav`, `b3.mp3`, `.ogg`, `.opus`, `.m4a`, `.flac`, `.aac`, `.aiff`). A cue without one may be synthesized (the system's voice, a command, or a browser's speech synthesis) or stay silent, which is reported.
  - Timing never depends on sound: a cue lasts its `for d` or its reading time (PK-9.2b), so captions, sound, frames and cases agree with or without a voice (PK-8.7). A recording longer than its cue is reported with the duration to write; it plays on, overlapping what follows.
  - A host joining a cue part way starts its recording at the offset into the cue; synthesized speech starts only at a cue's start, since a sentence cannot be spoken from its middle.
  - Video export: `--voice DIR`, `--speech system|COMMAND`, `--music FILE` (looped under the narration at `--music-volume`), and a recording script (`OUT.txt`, and `narration.txt` in a frame directory) listing every cue's name, start, length and text. The web player: a Voice menu (off, synthesized, recordings chosen as files).
- **Reason:** programs stay text that runs the same everywhere; sound is a property of a medium, like captions drawn or in a track; one naming rule lets every host voice the same lesson, and recordings can be made from the script without touching the program.
- **Consequences:** PK-9.2d; HI-5.1a; `Caption::cue` in `prismal-present`; `prismal-media` `voice` module; `web/player.js` narration sound and `web/check-voice.mjs`; guide chapter 8. Not yet done: voices for several languages (a voice directory per language is the natural extension), and fitting timing to recordings as an explicit, reported option.
- **History:**
  - 2026-09-30 raised by the owner and accepted; details under the owner's standing delegation of 2026-09-30.

## D-054: Round geometry in the representation set

- **Status:** Accepted
- **Original position:** PK-6.3's first-slice representation set has straight geometry only (`segment`, `polyline`, `polygon`); `marker` draws a point as a dot of fixed screen size. The guide's rolling wheel (chapter 7) was drawn as a square, noting that v0 had no circle.
- **Raised by:** the owner, 2026-09-30, asking why the wheel was a square; and a lesson on the circumference of a circle, unwrapped onto a line, which needs a circle whose edge can be partly drawn.
- **Builds on:** PK-5.5, PK-6.3, PK-6.3b, PK-12.1a, D-042, D-043, D-046.
- **Question:** How are circles drawn, and what does a frame carry for them?
- **Options considered:**
  1. **A polygon of many points.** No new kind, but the language has no loops or collections to write the points, and a drawn polygon is not a circle to a reader, a renderer or a text alternative.
  2. **Frames carry sampled points of new kinds.** Every renderer draws them, but curves are approximate when zoomed, frames grow, and the shape's meaning is lost to renderers.
  3. **`circle`, `ellipse`, `arc` kinds carried as exact elliptical arcs in view coordinates.** Renderers draw true curves (SVG paths, canvas arcs) from centre, radii, rotation, start and sweep; the engine applies the view's orientation and groups' transforms, so no renderer handles angles of the model.
- **Accepted position:** option 3, under the owner's standing delegation of 2026-09-30 (the owner asked for the circle). Radii are lengths in the model's units; angles counterclockwise from the space's `x` axis; `arc` takes `from` and `to`, `ellipse` takes `rotate`. One frame shape, `ellipse`, serves all three; `closed` marks a whole circle or ellipse. Hit testing and view extents sample the curve.
- **Reason:** a circle of a model's size is basic to geometry, physics and chemistry content; exact curves keep frames small and renderers simple; the same shape drawn part way (`reveal draw`, or an arc whose end follows the model) shows a circle being drawn or unwound.
- **Consequences:** PK-6.3c; `CKind::Round` and `Shape::Ellipse` in `prismal-present`; both renderers; guide chapter 7 (the wheel is a circle) and chapter 8 (Unwrapping a circle); `prismal-present/tests/shapes.rs`. Not included: circles as model values (a point constrained to a circle, intersections), which belong to geometry types in the model kernel, not to representations; author styling (colour, dashes, fill) of representations.
- **History:**
  - 2026-09-30 raised by the owner and accepted under the owner's standing delegation of 2026-09-30.

## D-055: Contained objects and fixed collections in v0

- **Status:** Accepted
- **Original position:** MK section 7 defines object types, contained objects and their identity; MK section 8 defines collections (declared or dynamic membership), relations, and expressions over collections (`count`, `map`, `filter`, `any`, `all`, `sum`, `reduce`); MK section 16 lists `create`, `destroy`, `connect` and `disconnect`. The first slice exercised none of them (MK-8.7), the working syntax reserves `object` and the structural operations without a syntax for holding objects or iterating over them, and the prototype rejected `object`.
- **Raised by:** the language completeness step (PROJECT-STATE next steps); the owner chose it as the next work on 2026-09-30.
- **Builds on:** MK-7.1 to MK-7.12, MK-8.1 to MK-8.4, MK-14.8, D-015, D-036, D-051.
- **Question:** How are objects and collections written, carried in the IR, and executed, and which part comes first?
- **Options considered:**
  1. **Lower objects away in the parser.** The IR would be flat; but the formatter prints programs from the IR (D-036), so a program with objects could not be printed back, and hosts would not see the model's structure.
  2. **Structure in the IR, executed natively.** The runtime would hold objects with per-object state and evaluate expressions against member indices. Needed for membership that changes during a run, but it changes the kernel's compiled form, the solver's state layout and every consumer of frames at once.
  3. **Structure in the IR, elaborated to a flat model before checking.** Object types, parts, member expressions and aggregates are kept in the IR; a pure IR-to-IR pass expands each member of fixed membership into bindings, flows, events, equations and constraints with identities derived from the declaration and the member's path, and expands `for` in presentations. The checker, runtime, presentations and renderers are unchanged. This is how Modelica compiles components.
- **Accepted position:** option 3 for contained objects and collections of fixed membership, under the owner's standing delegation of 2026-09-30. Dynamic membership (`create`, `destroy`) and relations are the next step and will need option 2 or a bounded form of it; they are not decided here.
  - **Object types** are declared in a model: `object Ball { ... }`, with the body of a model (bindings, flows, events, equations, constraints, functions, enumerations, parts). An object reads its container only through its `input`s, which the container connects (MK-7.11); an unconnected input keeps its default.
  - **Parts:** `parts { ball: Ball { pos = ... }  row: Ball[3] { pos = origin + (index * 1 m, 2 m) } }`. An override sets a member's starting value (state, discrete, parameter) or connects an input; it is written in the container's scope, where `index` is the member's number, from 1. Members are numbered in declaration order (MK-8.3).
  - **Expressions:** `ball.pos`; `row[2].pos` with a constant index; aggregates `sum`, `min`, `max`, `any`, `all` written `sum(e for b in row)` with an optional filter `if c`, and `count(row)`. A filter may compare members (`o != b`); `min` and `max` take only such filters. `sum` over no member is `0`, `any` false, `all` true.
  - **Container behaviour:** `flow { for b in row { der(b.vel) += e } }` writes members' derivatives from the container (MK-7.10), with `e` read in the container's scope; pairwise interactions are written with aggregates inside the loop.
  - **Presentations:** `for b in row { marker(b.pos) as ball }` in views and timeline blocks draws one representation per member, named `ball[1]`, `ball[2]` ...; expressions in presentations, observations and expectations may use members and aggregates.
  - **Identity:** an element of a member has the identity of its declaration followed by `@` and the member's path (`M.Ball.pos@row[2]`), and the name `row[2].pos`; paths nest (`cart.wheels[1]`). Identities are deterministic and stable across edits that keep the declaration (MK-7.6, D-036).
- **Reason:** the structure a reader writes stays in the IR and in printed programs; fixed membership covers systems of several bodies, chains of springs, pendulum arrays and lattices, with every existing checker rule, solver and renderer applying unchanged; and identities remain declaration paths.
- **Consequences:** MK-7.13, MK-8.3a, MK-8.8 (elaborations); 04-ir objects, parts, member and aggregate expressions, `each` on flows and representations; `prismal_ir::elaborate`; working syntax `parts`, `for`, `index`, aggregates; formatter; guide chapter on systems of objects. Not included: `create` and `destroy`, relations, container events per member, drags on members of collections.
- **History:**
  - 2026-09-30 proposed and accepted under the owner's standing delegation of 2026-09-30.

## D-056: Sign references after a transition

- **Status:** Accepted
- **Original position:** RC-7.2 keeps each guard's sign reference as the sign of `g` where it was last non-zero; RC-7.6 places a located event on the far side of the crossing, so the guard there is small and already has its new sign; RC-7.4 allows two sign changes inside one step to be missed.
- **Raised by:** the collections work (D-055): a row of balls bouncing on a floor at their radius (`falling(pos.y - r)`) fell through the floor after about fifty bounces instead of settling. The reference program bounces on a floor at zero, where steps near the floor are short, and never showed it.
- **Builds on:** RC-7.2, RC-7.4, RC-7.6, RC-9, D-005.
- **Question:** What sign reference does a guard take right after an event whose handler reverses it?
- **Options considered:**
  1. **As specified.** After a bounce the guard is a few `ε_t` below zero and its reference is negative; when the next hop is shorter than a step, the guard goes up and down again unsampled, and the landing is missed: RC-7.4 permits it, and the ball falls through.
  2. **Ask authors to bound the step (`h_max`).** RC-7.4's remedy; but no step bound works for all hops, which shrink geometrically towards the accumulation point.
  3. **Take the sign the guard is heading to when it is within the event tolerance of zero.** After a transition, advance the committed state `4 ε_t` by the flows; a guard that changes sign in that time takes the new sign as its reference.
- **Accepted position:** option 3, under the owner's standing delegation of 2026-09-30, as the elaboration RC-7.2a. Only guards within about `4 ε_t` of a crossing are affected, where the guard's own sign is below the location accuracy; everywhere else RC-7.2 applies unchanged.
- **Reason:** a handler that reverses the motion is the common case of a repeated crossing (bounces, reflections, relays); the reference then describes where the model is going, which is what the next detection needs. Every reference program and guide case gives the same results.
- **Consequences:** RC-7.2a; `Engine::retake_refs` and `Engine::just_ahead` in `prismal-runtime`; `prismal-present/tests/objects.rs` (a ball on a raised floor settles).
- **History:**
  - 2026-09-30 raised by the collections work and accepted under the owner's standing delegation of 2026-09-30.

## D-057: Collections whose membership changes, bounded by a declared capacity

- **Status:** Accepted
- **Original position:** MK-8.1 and MK-8.2 allow collections of dynamic membership, changed only by `create` and `destroy` at event instants; MK-7.7 gives a created element an identity from its creating operation and a per-run counter, never reused within a run; MK-8.3 orders members by identity (declared, then created); MK-16.5 makes two destroys of one object no conflict. D-055 implemented fixed membership only, by elaboration, and left dynamic membership to "option 2 or a bounded form of it".
- **Raised by:** the language completeness step (PROJECT-STATE next steps, item 5), the next step after D-055.
- **Builds on:** MK-7.7, MK-7.10, MK-8.1 to MK-8.4, MK-15, MK-16, D-015, D-027, D-036, D-050, D-055.
- **Question:** How are members made and removed during a run, and how does the prototype execute it without changing the solver, the checker's rules, the frame format or the renderers?
- **Options considered:**
  1. **Objects held natively by the runtime** (D-055 option 2): state vectors that grow and shrink, expressions evaluated against member indices. Unbounded, but it changes the compiled model, the solver's state layout, dense output, snapshots and every consumer of frames at once.
  2. **A bounded collection, reusing places.** The author declares the most members alive at once; a destroyed member's place is taken by the next one made. Long runs never fill up, but a place then holds several members in turn: identities are no longer declaration paths, iteration order is no longer creation order (MK-8.3), and a representation or a trace would jump from one member to the next.
  3. **A bounded collection, one place per member made.** The author declares the most members a run makes (`drops: Drop[max 50]`); the `k`-th member made is `drops[k]` for the whole run and its place is never reused. Elaboration expands the collection to that many members, each with a liveness binding; everything else follows D-055.
- **Accepted position:** option 3, under the owner's standing delegation of 2026-09-30. Option 1 remains the route for unbounded populations when content needs them; option 3's syntax and semantics carry over to it unchanged, since the capacity then becomes a limit of the runtime rather than of the language.
  - **Declaration:** `drops: Drop[max 50]` starts empty; `still: Stone[2, max 4] { ... }` starts with two members. The overrides apply to every member, with `index` its number; inputs are connected for every member.
  - **Operations** (MK section 16), in handlers and Zeno settle clauses: `create drops { pos = p, vel = v }` makes the next member, its stored bindings starting at the overrides, read in the handler's scope on the state before the transition (payloads included); bindings not given keep their declared starting values. `destroy b` ends a member: `b` a loop variable, a contained member or `drops[k]`. Several creates of one collection in one handler make consecutive members.
  - **Events per member:** `for d in drops { event land on falling(d.pos.y) { destroy d } }` in a model repeats the event for each member, named `land[k]`; its trigger, condition and handler read the member as `d`. A container's handler may write a member's binding: `set b.vel.x = -b.vel.x` (MK-7.10).
  - **A member not alive** (not made yet, or destroyed) has no flows, events, equations or constraints in effect; aggregates leave it out (`count(drops)` counts the members alive; `min` and `max` have no value over none, so they are written with `otherwise`); its representations are not drawn and a trace of it starts when it is made. Reading one by number (`drops[3].pos`) gives its starting values before it is made and its last values after it is destroyed.
  - **Capacity:** making more members than declared stops the run at a `stop` constraint `drops.capacity` (MK-12), with the instant.
  - **Identity** (MK-7.7): the `k`-th member made in a run has the identity `declaration@drops[k]` and the name `drops[k].x`; it is deterministic, replayed identically (D-015) and never reused. Iteration order is creation order (MK-8.3).
  - **Conflicts:** two destroys of one member in one transition are not a conflict (MK-16.5); a set on a member destroyed in the same transition is. Creates of one collection from two events handled in the same transition conflict on the count of members made, and the transition is rejected (MK-16.6); in one handler they do not.
- **Reason:** the populations first slices need (particles emitted by a source, drops from a tap, bodies that merge or leave a region, spawned objects in a lab) have a bound an author can state; with it, every existing rule, the solver, snapshots and frames apply unchanged, identities stay declaration paths, and the program reads as the model it describes. Places are not reused so that a member's identity, its trace and its representation belong to one member for the whole run.
- **Consequences:** MK-7.7a, MK-8.2a, MK-15.1b, MK-16.1a (elaborations); 04-ir `capacity` on parts, the operations `create`, `destroy` and `if`, `member` on targets, `each` on events, the `extreme` expression, `when` on representations; `prismal_ir::elaborate` (liveness bindings `part.alive@path`, the count `part.created`, the capacity constraint); the kernel's conditional operations and destroys; the runtime's transition performs conditional operations read on the state before it; representations with `when`; `Interactive::request` for requests with payloads from a session; working syntax; formatter; guide chapter 9 (a fountain). Found and fixed: a type error in a `create` override was reported once for each member it could make; the kernel now reports identical diagnostics once; aggregates over 30 or more members overflowed the stack of debug builds, being folded into a chain as deep as the collection, and are now folded as a balanced tree. Not included: relations (`connect`, `disconnect`), creation and destruction as interventions from outside (MK-17.2; a lab requests an event instead, D-027), `on start` of a member made later (it applies at the run's start only), unbounded populations (option 1).
- **History:**
  - 2026-09-30 proposed and accepted under the owner's standing delegation of 2026-09-30.

## D-058: Relations, with endpoints chosen during the run

- **Status:** Accepted
- **Original position:** MK-8.5 declares a relation type by its endpoint roles (each typed by an object type), whether it is directed, and its bindings; MK-8.6 holds relation instances in a relation set changed only by `connect` and `disconnect`, and destroying an object disconnects its relations in the same transition; MK-8.7 separates a relation from any line drawn. D-057 left relations out.
- **Raised by:** the language completeness step (PROJECT-STATE next steps, item 5), after D-057.
- **Builds on:** MK-7.7, MK-7.10, MK-8.5 to MK-8.7, MK-16, D-038, D-055, D-057.
- **Question:** How are relations declared, made, read and ended, and how does elaboration represent an endpoint that is chosen during the run?
- **Options considered:**
  1. **Endpoints typed by object type only** (MK-8.5 as written), resolved to any object of that type anywhere in the model. The member at an endpoint could then be in any collection, and a flat model would need every object of the type as a candidate.
  2. **Endpoints in named collections of the containing model**, each held as the member's number in its collection; reading `s.a.pos` picks the binding of that member among the collection's members. Relation sets are collections (D-057): a capacity, liveness, identities never reused.
  3. **Relations elaborated away into pairs fixed when the program is read.** Cheap, but `connect` during a run would be impossible.
- **Accepted position:** option 2, under the owner's standing delegation of 2026-09-30.
  - **Declaration:** `relation Spring(a in balls, b in balls) { param ...  derived ... }` in a model; each endpoint is a member of a part of the model (a collection or one object). The body is that of an object type; `a` and `a.pos` read the member at the endpoint.
  - **Relation sets** are parts: `springs: Spring[1, max 4] { a = balls[1]; b = balls[2] }`; starting relations name their endpoints in the overrides, with `index`.
  - **Operations:** `connect springs(x, y) { k = e }` makes the next relation, endpoints in the order of the roles, with starting values (parameters included); `disconnect s` ends one. Destroying a member disconnects, in the same transition, every relation of the container with that member at an endpoint (MK-8.6).
  - **Expressions:** `s.a` is a member (not a value); `s.a.pos` a binding of it; `s.a == o` and `s.a != s.b` compare members, true only for the same member of the same collection. Aggregates, `for` in views and events per relation (`for s in springs { event snap on ... { disconnect s } }`) work as for collections.
  - **Relations are directed:** the roles are named, and an undirected relation is written by testing both roles. Several relations between the same members are allowed.
  - **Values not alive:** a derived binding of a member or relation that is not alive is not evaluated and has no value (a value that is not a number); the guards of its events are not evaluated either. This also applies to collections (D-057).
- **Reason:** named collections make an endpoint a number, which a run can store, compare and change, and let every existing rule, the solver and the frames apply unchanged; relation sets reuse D-057's capacities and identities, so a spring's name, its drawing and its values belong to one spring for the whole run.
- **Consequences:** MK-8.5a, MK-8.6a, MK-16.1b (elaborations); PK-6.3c extended to relations; 04-ir `ends` of an object type, the `end` and `pick` expressions, the `connect`, `disconnect` and `make` operations, `when` on bindings; `prismal_ir::elaborate` (endpoint bindings `Rel.a@rs[k]`, picks, conditional disconnection); the kernel's `pick`, derived bindings evaluated only while alive, and the rule that an event whose handler destroys the member its condition requires alive does not retrigger itself (D-038); working syntax (`relation` is a contextual word); formatter; guide chapter 9 (a spring between two balls, a thread that snaps). Found and fixed: `create` could not give a new member's parameter a starting value (MK-E08, handlers do not set parameters), now the `make` operation of elaboration; the guard of an event of a member not alive was evaluated and failed on its missing values; a mistake in a member expression was also reported as a missing component. Not included: undirected relations as a declared property, relations whose endpoints are relations, relations between members of different containers, creation and destruction by intervention (MK-17.2).
- **History:**
  - 2026-09-30 proposed and accepted under the owner's standing delegation of 2026-09-30.

## D-059: Members as payloads, drags and clicks on members

- **Status:** Accepted
- **Original position:** D-050 gave an event at most one payload, a value of a declared type. D-057 and D-058 let a handler act on a member known when the program is read (`destroy balls[2]`), on the member of an event repeated per member, or on the member at a relation's endpoint, but a request could not say which member it is about: a lab with a population could not let the learner remove, kick or drag one ball of many. Drags targeted bindings of the model only (PK-10.5), and a representation could not be clicked. A request whose enabling condition was false did not occur and was not reported.
- **Raised by:** the language completeness step (PROJECT-STATE next steps, item 5), planned with the owner on 2026-09-30 after D-058.
- **Builds on:** MK-7.10, MK-8.2a, MK-15.1a, MK-15.5, MK-17.2, RC-11.6b, PK-10.2a, PK-10.5, PK-11.2, D-023, D-026, D-027, D-043, D-050, D-055, D-057, D-058.
- **Question:** How does a request, a drag or a click name the member it concerns, and how does elaboration carry a member chosen during the run into handlers and proposals?
- **Options considered:**
  1. **Creation and destruction as direct host interventions** (MK-17.2): hosts create and destroy members themselves. It bypasses the author, who decides what learners may make (D-023), and it does not help a kick or a drag.
  2. **Members as payloads.** `on request(b in balls)` declares a payload that is a member of a collection; the occurrence carries the member's number, and in the handler `b` is a member chosen during the run, handled by D-058's machinery (picks, conditional operations per member). Drags and clicks on a representation repeated per member supply that member.
  3. **Events per member requested by name** (`remove[2]`). Works with D-057's events per member, but hosts and timelines would name generated events, and one event could not take a member and a value together.
- **Accepted position:** option 2, under the owner's standing delegation of 2026-09-30, with the syntax recommended to the owner (payloads `(b in balls)` like relation endpoints; `on click request E(b)` beside `on drag`).
  - **A. Members as payloads.** `event remove on request(b in balls) { destroy b }`; `event kick on request(b in balls, j: Momentum) { set b.vel = b.vel + j / b.m }`. The payload is the member's number in its collection, so replay is deterministic. The handler and condition read the member as `b`; `set b.x` on the chosen member becomes one conditional operation per member, and the same holds through a relation's endpoint (`set s.a.vel`). The event is implicitly enabled only when the member exists and is alive. Several named payloads per event are carried as one tuple and supplied as a list: timelines `request kick(balls[2], j)`, `emit kick(b, j)`; a follower `on E(c in balls)` receives members of the same collection. Hosts name members in the protocol's request payload, `"balls[2]"` (HI-4.3b).
  - **Requests refused with a reason:** a request whose event is not enabled is rejected, stating whether the member is not alive, has no member of that number, or the condition does not hold (RC-11.6b). Before, it was dropped silently.
  - **B. Drags on members.** A representation repeated per member may be dragged: `for b in balls { marker(b.pos) as ball on drag as p { propose b.pos = p } }` proposes a value for that member's binding, which must be `intervenable` in the object type (D-023). A drag on a member that stops being alive is refused; a drag proposing an endpoint's binding (`propose s.a.pos`) is refused with the advice to drag the member.
  - **C. Clicks on members.** `for b in balls { marker(b.pos) as ball on click request remove(b) }`: a representation may request an event when clicked, with its member as payload. Clicks only request events, never set values. The engine's targeting tells a click from a drag by movement under the drag tolerance; frames carry the event a click requests; clickable representations take keyboard focus, and Enter or space activates them (D-026).
  - **Later, not in this decision:** clicks on an empty point of a view (a position as payload).
- **Reason:** the payload is a number a run can log, replay and compare, and everything else (conditions, handlers, relations, conflicts, identities) is D-057 and D-058 unchanged; the author keeps control of what a learner may do, since only declared events and intervenable bindings are reachable. Option 1 was set aside because requests already let a host create members (`on request(p: Point) { create ... }`) while the author decides what can be made; the missing piece was naming a member, which option 2 provides.
- **Consequences:** MK-15.1c; RC-11.6b amended; PK-10.5a, PK-10.5b, PK-10.2a and PK-11.2b extended; HI-4.3 (`click`), HI-4.3b (the protocol's `request`), HI-4.5 (clicks told from drags); 04-ir payload fields `of`, `members`, `items`, a proposal's `member`, a representation's `click`; `prismal_ir::elaborate` (loop variables that are members chosen during the run, conditional targets, endpoints of a chosen relation, payloads supplied as member numbers, proposals resolved to the member drawn); the checker checks a tuple against an expected tuple type item by item; the runtime rejects requests its event does not enable, with the reason; the parser reads targets through endpoints (`set s.b.vel.x`) and `propose b.pos = p`; `click` is a contextual word; formatter; frames carry `click`, and the text alternative ends `activate: E`; the host's targeting, gestures and focus order; the web player's pointer cursor and keyboard activation; host event logs print members by name (`remove(balls[2]) at 1 s`); guide chapter 9, Labs with members (an orbits lab); tests `member_payloads.rs`, `member_input.rs`, the protocol's `requests_name_members`, the raw input test `clicks_and_drags_on_members`, and `web/check-input.mjs` in a browser. A mistake in a representation repeated per member is reported once for each member, as mistakes in object types are.
- **History:**
  - 2026-09-30 planned with the owner (PROJECT-STATE next steps, item 5).
  - 2026-10-01 accepted under the owner's standing delegation of 2026-09-30; step A implemented.
  - 2026-10-01 steps B (drags on members), C (clicks) and D (guide chapter 9, Labs with members) implemented.

## D-060: Clicks on an empty point of a view

- **Status:** Accepted
- **Original position:** D-059 let a representation request an event when clicked, and left clicks on an empty point of a view for later. A lab could not let the learner place something where they click (a planet, a charge, a point of a polygon); it needed a button and a fixed place, or a drag of something already there.
- **Raised by:** D-059 ("Later, not in this decision"); PROJECT-STATE next steps, item 3.
- **Builds on:** PK-5.6, PK-10.2a, PK-10.5, PK-10.5b, HI-4.3, HI-4.5, D-009, D-011, D-026, D-027, D-047, D-050, D-059.
- **Question:** How does a click on a point where nothing is drawn request an event, and what does it carry?
- **Options considered:**
  1. **A click written in the view**: `view sky: spatial(...) { on click as q request place(q) ... }`. The point is a gesture value, read as a drag's is (PK-10.5), so the payload is an expression of it. Representations keep their clicks; the view's applies only where no representation takes the press.
  2. **An invisible representation covering the view** (`area(on click ...)`): no new place for interactions, but a representation with no drawing and no extent is a special case in framing, hit testing, text alternatives and focus.
  3. **Clicks delivered to the model as an input** (`input click: Point`): the model would depend on a presentation gesture, which D-009 and D-011 rule out.
- **Accepted position:** option 1, under the owner's standing delegation of 2026-09-30. `on click as q request E(q)` in a spatial or plot view (at most one per view; none in a panel, SX-E06). The payload has the event's declared type (PK-E02); the event is declared `on request` (PK-E03). A press on an empty point that is released within the pointer's reach clicks; with `pan` permitted, a press that moves pans. The host's `click_at` names a view and a point in view coordinates, for hosts that target input themselves. Frames carry the event a view's click requests; the web player shows a crosshair over an empty point.
- **Reason:** the point is data the author turns into a payload with the language's own expressions, and nothing reaches the model but a request the author declared, validated, logged and undone like any other (D-027, D-059). Option 2 was set aside because it adds a representation that draws nothing; option 3 because the model would depend on the presentation.
- **Consequences:** PK-10.5c; HI-4.3 (`click_at`), HI-4.5 (presses on an empty point); 04-ir: a view's `click`; working syntax; the parser reads `on click as q request E(q)` in a view's block, the formatter prints it first; elaboration carries the payload; `Projector` checks view clicks; `Interactive::click_at`; the host's gestures (`Point`, and a pan that may still click); the web player's crosshair; guide chapter 9 (`place` in the orbits lab); tests `view_clicks.rs`, the raw input test `clicks_on_an_empty_point`, and `web/check-input.mjs` in a browser. Keyboard users reach the same event through a control or button (D-026): a point is not chosen with keys in v0.
- **History:**
  - 2026-10-01 accepted under the owner's standing delegation of 2026-09-30 and implemented.

## D-061: Author styles of representations

- **Status:** Accepted
- **Original position:** PK-2.4 and PK-5.5 name colors as presentation configuration, but no syntax or frame field existed: every kind was drawn with the renderer's own colors (arrows taking turns through four). The guide's circumference lesson (session of 2026-09-30) had to draw a laid edge as a polyline to share the arc's color, and an author could not tell two segments apart except by name.
- **Raised by:** the circumference lesson (PROJECT-STATE session log, 2026-09-30, D-054); the language completeness step.
- **Builds on:** PK-2.4, PK-5.5, PK-6.2, PK-11.4, PK-12.1, PK-12.2, D-026, D-046.
- **Question:** How does an author choose how a representation looks, while frames stay free of any one medium?
- **Options considered:**
  1. **Named colors and line styles as set properties**: `color: blue`, `line: dashed`. Frames carry the names; each medium maps them to values for its theme (light, dark, print, video).
  2. **Color values** (`color: "#2458c6"`): exact, but a value chosen for a light page is wrong on a dark one, and frames would carry a medium's values.
  3. **Style sheets or classes** (`class: highlight` with a separate style block): flexible, but a second language inside the presentation, not needed by any content yet.
- **Accepted position:** option 1, under the owner's standing delegation of 2026-09-30. Ten colors (`red`, `orange`, `yellow`, `green`, `teal`, `blue`, `purple`, `pink`, `gray`, `ink`) and three lines (`solid`, `dashed`, `dotted`). Markers take a color; stroked kinds take both; other kinds and groups take neither (PK-E05). Styles are decoration: not in text alternatives, never the only encoding (PK-11.4). Styles bound to model values (color scales) are later work.
- **Reason:** names keep the frame description medium-independent (PK-12.1a) and let every renderer keep contrast in its own theme; a small fixed palette is what educational diagrams use, and the names read as intent. Option 2 was set aside for dark themes and medium independence, option 3 as more than content needs.
- **Consequences:** PK-6.6a; HI-5.2; 04-ir (`word` arguments `color`, `line`); working syntax; lowering of style words; `compile_rep` checks and strips style properties before each kind's checks; `RepFrame` gains `color` and `line`; `prismal-svg` themes gain a palette (`Theme::color`, `COLORS`); the web player's style sheet gains `--color-*` for light and dark and draws `--c` and dash styles as `prismal-svg` does; guide chapter 7, Color and line (the wheel's spoke and valve), reference; tests `prismal-svg/tests/styles.rs`.
- **History:**
  - 2026-10-01 accepted under the owner's standing delegation of 2026-09-30 and implemented.

## D-062: Drags on members of a group

- **Status:** Accepted
- **Original position:** D-043 placed a group's members by a shared transform; a member that declared an inverse was refused (PK-E06, "not implemented by the prototype"), so a hand of a dial, a handle on a rotating body or a knob in a turned panel could not be dragged.
- **Raised by:** PROJECT-STATE next steps, item 3 (input follow-ups).
- **Builds on:** PK-5.6, PK-6.3b, PK-10.2a, PK-10.5, PK-11.2, D-023, D-043, D-047.
- **Question:** In which frame does a dragged member of a group read the pointer?
- **Options considered:**
  1. **The group's frame.** The member is written in the group's frame (`marker(origin + (r, 0 m))`), so the proposal reads the pointer there too: `propose r = p.x` is the distance along the hand whatever the group's placement, turn and scale. The runtime inverts the transform at the instant shown.
  2. **The view's space.** The gesture value is the same as outside a group; the author undoes the placement and turn in the proposal by hand, repeating the group's expressions.
- **Accepted position:** option 1, under the owner's standing delegation of 2026-09-30. Nested groups compose; the transform is evaluated on the committed state at the instant shown, so a drag that moves the group does not move the frame under the pointer while it is in progress. Hit testing, the focus order and keyboard steps include members of groups.
- **Reason:** a member's geometry and its inverse are then written in one frame, which is the point of a group; option 2 would make every inverse in a group restate the group's transform, and break when the group changes.
- **Consequences:** PK-6.3b amended; `Tf::of_groups`, `Tf::unapply`, `ViewCtx::pointer_in`; `Interactive` finds representations inside groups with their enclosing groups (drags, keys); the host's `hit` and `focus_order` recurse into groups; guide chapter 7 (a dial in a turned group); tests `prismal-present/tests/group_drags.rs`, the raw input test `drags_on_members_of_groups`, and `web/check-input.mjs` in a browser.
- **History:**
  - 2026-10-01 accepted under the owner's standing delegation of 2026-09-30 and implemented.

## D-063: Page layout of views

- **Status:** Accepted
- **Original position:** PK-7.4 says layout "places views in the presentation and adapts them to the output's size and orientation", and PK-2.1 lists "a layout of views" among a presentation's contents, but no form for it existed: the web player filled a grid in declaration order, putting a wide scene on a row of its own, and the SVG renderer and video export stacked views one below the other. An author could not put a plot beside a scene or a panel beside a plot. D-052 listed it as not done.
- **Raised by:** PROJECT-STATE next steps, item 2 (media follow-ups); the owner's list of 2026-10-01.
- **Builds on:** PK-2.1, PK-2.4, PK-7.4, PK-12.2, D-040, D-052.
- **Question:** How does an author say where views go, and how much of the arrangement belongs to the program rather than to the medium?
- **Options considered:**
  1. **Nested rows and columns of view names.** `layout row(scene, column(plot, controls))`: a tree that says which views are side by side and which are one below the other. Sizes stay the views' own (a spatial view's extent and scale, a plot's size), and each medium adapts the tree to its page: a browser turns a row into a column on a narrow screen.
  2. **A grid with areas.** Named cells in rows and columns, spans, and fractions of the width, as CSS grid areas. Precise for one page size, but sizes and spans are a medium's business (PK-7.4: layout never changes what a view shows), and a grid written for a wide screen has no meaning on a phone or in a video frame of another shape.
  3. **Positions and sizes per view.** Absolute placement in pixels: ties the program to one output size, against PK-12.2.
- **Accepted position:** option 1, under the owner's standing delegation of 2026-09-30. A presentation has at most one `layout` item, a view name, `row(...)` or `column(...)` with at least one item each, nested freely. Every name is a view or panel of the presentation, placed at most once (SX-E03 for an unknown name, SX-E09 for a view placed twice or a second layout; PK-E01 for a layout written in the IR). Views the layout leaves out follow it, one below the other, in declaration order. `layout`, `row` and `column` are contextual keywords (D-040). The IR carries the tree with view identities (`{"row": [{"view": id}, {"column": [...]}]}`), so renames keep it; the host's layout gives it as `page`, with the views left out appended. Without a layout, each medium arranges views as before.
- **Reason:** the author decides relations (this plot belongs beside this scene; the readout under the controls), which hold on every page; sizes and breakpoints depend on the medium, which knows its page. A tree of rows and columns is the smallest form that says this, and it reads as the page looks.
- **Consequences:** PK-7.4a; 04-ir section 7.1; working syntax 1.2; IR `present::Layout` and `Presentation::layout`; parser, lowering, formatter; presentation checks; the host's `page`; `prismal-svg` places views in rows (top aligned) and columns, so still images and videos follow the layout; the web player builds rows and columns and turns rows into columns below 700 px; guide chapter 7, Laying out views; `prismal-svg/tests/layout.rs`. Not yet: weights or sizes per view, alignment choices, and layouts that change during a lesson.
- **History:**
  - 2026-10-01 accepted under the owner's standing delegation of 2026-09-30 and implemented.

## D-064: Undirected relations

- **Status:** Accepted
- **Original position:** MK-8.5 declares "whether it is directed" as part of a relation type. D-058 made every relation directed, with named roles, and left undirected relations to be "written by testing both roles": guide chapter 9's springs sum the pulls at the `a` ends and subtract those at the `b` ends, so a spring declared with its endpoints swapped changes the model's text but must not change its motion.
- **Raised by:** PROJECT-STATE, "Not implemented" (undirected relations as a declared property); the owner's list of 2026-10-01.
- **Builds on:** MK-8.5, MK-8.5a, MK-8.6a, D-055, D-058, D-059.
- **Question:** What does declaring a relation undirected change, and how does a model read an undirected relation's endpoints?
- **Options considered:**
  1. **A declared property with order-free readings.** `undirected relation Link(a in balls, b in balls)`: its two endpoints are in one collection; two expressions read any relation's endpoints without an order, `s.has(o)` (is `o` an endpoint) and `s.other(o)` (the endpoint that is not `o`); outside the relation's body, the model reads an undirected relation's endpoints only through them. Its body still names `a` and `b` to define its values.
  2. **Only the two expressions**, with no declared property. Convenient, but the model can still read `s.a` and make its motion depend on an order the relation does not have; nothing records the author's intent.
  3. **Unordered pairs throughout**, with a relation set holding at most one relation per pair and `connect` of a joined pair refused. A stronger statement (a simple graph), which some content wants (bonds) and some does not (two springs in parallel); it can be added later as its own property.
- **Accepted position:** option 1, under the owner's standing delegation of 2026-09-30. `undirected` is a word only before `relation`. `has` and `other` apply to any relation with two endpoints in one collection; `s.other(o)` is a member (its bindings are read, `s.other(o).pos`, and it compares with `==`); when both endpoints are `o`, it is `o`; a member of another collection is never an endpoint (`has` is false, `other` is an error). An undirected relation over two collections, or reading `s.a` of one in the model outside its body, is MK-E26. Presentations may read `s.a` and `s.b` to draw it. Several relations between the same members stay allowed. The IR marks the relation type `"undirected": true` and has the expressions `{"other": member, "rel": relation}` and `{"has": member, "rel": relation}`, elaborated to comparisons and picks of endpoint numbers.
- **Reason:** the property records what the author means and lets the language hold the model to it: forces summed over `s.has(o)` with `s.other(o)` are the same whichever end was written first. The body keeps role names because a relation's values are often written from one end to the other (`b.pos - a.pos`); what must not depend on the order is how the model uses them.
- **Consequences:** MK-8.5b; 04-ir; working syntax; the parser, lowering, formatter and elaboration; guide chapter 9, Relations without a direction; `prismal-present/tests/undirected.rs`. Not yet: relation sets with at most one relation per pair (option 3), and enforcing order independence inside the body.
- **History:**
  - 2026-10-01 accepted under the owner's standing delegation of 2026-09-30 and implemented.

## D-065: Relations across containers

- **Status:** Accepted
- **Original position:** MK-8.6 has relation instances held "in a relation set owned by an object", with endpoints typed by object types anywhere. D-058 restricted endpoints to collections of the model that declares the relation and left out "relations between members of different containers": a bond between atoms of two cells, each cell holding its atoms, could not be written. Nor could the model name `left.atoms` at all: member expressions and loops took collections of the scope only.
- **Raised by:** PROJECT-STATE, "Not implemented"; the owner's list of 2026-10-01.
- **Builds on:** MK-7.10, MK-8.5a, MK-8.6, MK-8.6a, D-055, D-057, D-058.
- **Question:** How does a relation reach members held by contained objects, and how are such collections named elsewhere?
- **Options considered:**
  1. **Part paths through contained objects.** A collection is named by a path, `left.atoms`, through parts that are one object each; the same path names its members (`left.atoms[1]`), loops (`for o in left.atoms`) and aggregates (`count(left.atoms)`). In the IR, a part reference may be a path of part identities joined by `/`. The relation set lives in the container that contains both cells.
  2. **Relations declared inside an object type with endpoints in its container's collections.** Turns scope inside out (an object reaching outward), against MK-7.10's rule that a container reads its parts and not the reverse.
  3. **Endpoints in any collection of a type anywhere.** D-058 option 1: every collection of the type becomes a candidate for every endpoint, which elaboration cannot bound and readers cannot follow.
- **Accepted position:** option 1, under the owner's standing delegation of 2026-09-30. A path goes through contained objects only; a collection inside it (`cells[1].atoms`) is SX-E08 (MK-E26 in the IR). Destroying a member disconnects every relation with that member at an endpoint, in any relation set of the model, not only in its own container (MK-8.6 as written; MK-8.6a narrowed it to the container while endpoints could only be there). Creating members of a collection inside a contained object from outside it is not included: a cell makes its own atoms.
- **Reason:** paths are how the language already reads a contained object's bindings (`ball.pos`); extending them to collections keeps one way of naming things, and part identities joined by `/` keep identities stable under renames (D-036) without a new IR form for every place a collection is named.
- **Consequences:** MK-8.5c, MK-8.6a amended; 04-ir (part paths, `PART_PATH`); working syntax; parser (`part_path`), lowering (`part_ref`), formatter, elaboration (`members` and `coll_of` walk paths; `relation_sets` finds relation sets anywhere); guide chapter 9, Relations across containers; `prismal-present/tests/across.rs`. Not yet: `create` and `connect` written outside the container of the collection they change, paths through collections chosen by number.
- **History:**
  - 2026-10-01 accepted under the owner's standing delegation of 2026-09-30 and implemented.

## D-066: Collections without a declared limit

- **Status:** Accepted
- **Original position:** D-057 bounded every collection whose membership changes by a declared capacity and kept "option 1" (objects held natively by the runtime, state that grows and shrinks) as the route to unbounded populations, noting that its syntax and semantics would carry over with the capacity becoming "a limit of the runtime rather than of the language". A lab left running (a fountain, a particle source) stopped when its capacity was reached.
- **Raised by:** PROJECT-STATE, "Not implemented" (unbounded populations); the owner's list of 2026-10-01.
- **Builds on:** MK-7.7a, MK-8.2a, RC-14.2, D-015, D-055, D-057, D-059.
- **Question:** How does a model declare a collection with no limit, and how does the prototype run it without changing the solver, the frame format or the renderers?
- **Options considered:**
  1. **Native growable state** (D-057 option 1): the runtime adds state to the solver when a member is made. Unbounded, but it changes the compiled model, the state layout, dense output, snapshots, the checker and every consumer of frames at once.
  2. **A working capacity that grows by elaborating again.** `Drop[max inf]` is elaborated with a working capacity; a run that fills it is computed again from its start with the working capacity doubled. Everything D-057 built applies unchanged; identities stay `drops[k]`, so a session's logged actions, member payloads and drawings keep their meaning across growth.
  3. **A large fixed default** (say 10000 places). No regrowth, but every model with such a collection pays for all places in every step from the start.
- **Accepted position:** option 2, under the owner's standing delegation of 2026-09-30. The working capacity starts at the largest of 8, twice the starting members, and every number by which the program names a member (so `drops[12]` is valid); it doubles up to 16384, beyond which the run stops at the capacity constraint as for a declared limit. Growth happens wherever runs are made: cases (`run_case`), lessons (`play`) and sessions (`Interactive`, after each committed action and on reset, recommitting the session's actions on the new run). The IR marks the part `"unbounded": true` with no `capacity`; `Program` keeps the document as written and the working capacities.
- **Reason:** the populations content needs are bounded in practice but not by a number an author should have to choose; option 2 gives them without touching the kernel, and its results are exactly those of a declared capacity, so every guarantee of D-057 (identities never reused, deterministic replay) holds. The cost, recomputing from the start at each doubling, is logarithmic in the members made and paid only by runs that need it.
- **Consequences:** MK-8.2b; 04-ir `unbounded`; working syntax; parser and formatter (`max inf`); `elaborate::document_with`, `Capacities`, `starting_capacity`, `overflowed`, `MAX_CAPACITY`; `Program::sized`, `overflow`, `grown`; growth in `run_case`, `play` and `Interactive`; guide chapter 9; `prismal-present/tests/unbounded.rs`. Not yet: native growable state (option 1), if content needs very large populations made over long sessions; a lesson's explore beats grow only when the lesson is opened, not when a learner's action fills the collection later.
- **History:**
  - 2026-10-01 accepted under the owner's standing delegation of 2026-09-30 and implemented.

## D-067: Continue points

- **Status:** Accepted
- **Original position:** PK-9.2 lists `wait_for_learner`, "wait for the learner to continue (a continue point)", and PK-9.10 replaces it in linear media by a declared fallback. The working syntax gave it no form and the prototype did not implement it; the only way a lesson waited for the learner was an `explore` beat, which also branches the run and offers controls. An `explore` beat without a limit that the learner had not yet ended took no time and was reported as a diagnostic, so the web player never showed its Continue button.
- **Raised by:** PROJECT-STATE, next steps (timeline actions); `docs/prototype.md`, "Not implemented".
- **Builds on:** PK-9.1, PK-9.2, PK-9.7, PK-9.8, PK-9.10, PK-12.3, D-025, D-033.
- **Question:** How is a continue point written, when does it open within its beat, what does a recorded playback do before the learner has continued, and what do linear media do?
- **Options considered:**
  1. **`wait learner [limit d] [fallback { ... }]`, opened when the beat's other actions end.** The form follows `wait until E` and the `limit` and `fallback` of `explore`. A recorded playback without a continue input treats the point as **pending**: it takes no time, and hosts are told where it is, so a player stops there until the learner continues. Linear media play the fallback, or omit the point.
  2. **The same form, starting with the beat like every other action** (PK-9.1). Consistent, but `beat ask { narrate "..."; wait learner }` would stop the player before the narration, which is never what is meant; authors would always have to write `sequence { narrate "..."; wait learner }`.
  3. **Lay out the rest of the lesson only after the learner continues.** Honest about the unknown, but a lesson's beats, captions and length would be unknown to hosts until then, and a video export could not be planned the same way.
- **Accepted position:** option 1, under the owner's standing delegation of 2026-09-30. The point opens at the end of its beat's other actions (inside a `sequence`, at its place). It ends at the first `continue` of the learner at or after it opens, else at `limit`; with neither it is pending and takes no time. A `continue` before the point opens is not taken by it and is refused as before. The same applies to an `explore` beat without a limit (it becomes a pending continue point instead of a diagnostic). Linear media play `fallback`, and omit a point without one: a continue point has nothing to show, so it is not reported as unsupported (unlike `explore`, PK-12.3). The playback records every continue point (`waits`: beat, opening instant, end, whether pending, whether an explore beat); the host's lesson layout lists them; the web player stops at a pending point, and its Continue and Play buttons continue there. `learner` after `wait` is a contextual keyword (D-040).
- **Reason:** a pending point that takes no time gives exactly the playback the learner makes by continuing at once, so hosts can lay out the lesson in advance and a player only has to stop the clock; continuing later is an input like any other (PK-8.7). Opening after the beat's other actions matches how a continue point is read in prose ("say this, then wait").
- **Consequences:** PK-9.2e, PK-9.10a; 04-ir (`wait_learner`); working syntax; the parser, lowering, formatter, identities and elaboration; `Playback::waits`, `WaitPoint`; HI-5.1 (`waits` in a lesson's layout); the web player; guide chapter 8, Continue points; `prismal-present/tests/timeline_actions.rs`. Not yet: narration speed as a learner control separate from playback speed (PK-9.7).
- **History:**
  - 2026-10-01 accepted under the owner's standing delegation of 2026-09-30 and implemented.

## D-068: Animated properties and handover

- **Status:** Accepted
- **Original position:** PK-8.4 defines an animation as a change of an animated property "from a value, to a value, with a duration and an easing function", and PK-8.5 forbids animating a bound property, offering two ways around it: "a separate property (opacity, style, a presentation offset)" or "an explicit handover (`release` and `bind`)". D-042 implemented named effects only (`reveal`, `hide ... for`, `camera`); `animate`, `release` and `bind` were parsed and reported as not defined by the working syntax (SX-E06).
- **Raised by:** PROJECT-STATE, next steps (timeline actions); `docs/prototype.md`, "Not implemented".
- **Builds on:** PK-6.2, PK-8.4, PK-8.5, PK-8.7, PK-9.2, PK-9.2c, D-042, D-043, D-055.
- **Question:** Which properties can `animate` drive in v0, how are values written, and what do `release` and `bind` hand over?
- **Options considered:**
  1. **Two separate properties and a geometry handover.** `animate R opacity to x [for d]` (0 to 1, multiplying any reveal or hide) and `animate R offset to v [for d]` (a vector of the view's space, of dimension length, moving the drawn representation); `release R` holds R's bound geometry at the instant shown; `bind R [for d]` hands it back to its projection, blending from the held geometry to the live one. Durations default to 1 s for `animate`, none for `bind`; the easing is D-042's `3k² - 2k³`. Each animation starts from the value the previous one left, and the last value holds.
  2. **General property animation** (`animate R color ...`, `animate R scale ...`, morphs between shapes, chosen easing functions). The full PK-8.4, but each property needs its own interpolation in every medium, and content has not asked for them yet.
  3. **Offsets only, no handover.** Smaller, but leaves PK-8.5's handover without a form, so a lesson cannot keep a marker where it was while the model moves on, or return it smoothly.
- **Accepted position:** option 1, under the owner's standing delegation of 2026-09-30. An offset is evaluated once, at the instant shown when the animation starts, and applies to markers, arrows, segments, paths, polygons, circles, ellipses, arcs and groups (members included); a representation not drawn in a spatial view takes no offset (PK-E05). An opacity outside 0 to 1 is PK-E02. A released representation shows its geometry and text at the release instant whatever the run does; offsets and opacity still apply. `bind` blends shape by shape when the held and live geometry have the same form (the same kind and number of points) and switches at the end of the blend otherwise. Releasing a released representation, or binding one that is not released, is a playback diagnostic. A family of representations (D-055) is animated, released and bound member by member.
- **Reason:** opacity and an offset are the separate properties PK-8.5 names, and with the handover they cover the uses PK-8.4 lists for emphasis and moves without letting an animation drive a bound property. Frames stay functions of presentation time (PK-8.7); the model and its runs are untouched.
- **Consequences:** PK-9.2f; 04-ir (`animate`, `release`, `bind`); working syntax; parser (SX-E06 for these words removed), lowering, formatter, elaboration, checks; `Playback::animations`, `released`; guide chapter 8, Moving and fading, Holding and handing back; `prismal-present/tests/timeline_actions.rs`. Not yet: animated color, line and scale, shape morphs, chosen easing functions (option 2).
- **History:**
  - 2026-10-01 accepted under the owner's standing delegation of 2026-09-30 and implemented.

## D-069: Buttons in lessons and runtime-control buttons

- **Status:** Accepted
- **Original position:** PK-6.3 gives `button` a requestable event (D-027) "or a runtime control" as its source, and PK-9.8 lets an explore beat declare "which controls and intervenable bindings are available". The prototype implemented event buttons in labs only: a button in a lesson was refused ("buttons act in labs; in a lesson the timeline requests events"), a learner script could only set controls and continue, and no button offered a runtime control.
- **Raised by:** PROJECT-STATE, next steps; `docs/prototype.md`, "Not implemented".
- **Builds on:** PK-6.3, PK-9.7, PK-9.8, PK-9.8a, PK-10.4, RC section 12, D-025, D-027, D-047.
- **Question:** How does a learner press a button in a lesson, how does a learner script say so, and which runtime controls can a button offer, where?
- **Options considered:**
  1. **Explore-beat buttons and three session controls.** A `button(E)` among an explore beat's controls requests `E` on the learner's branch at the instant pressed, like a slider's setting; elsewhere in a lesson a press is refused (PK-9.8a). A learner script writes `at τ: press E`. `button(reset)`, `button(undo)` and `button(redo)` are runtime controls of a lab's session; the clock (play, pause, seek) stays the host's (HI-4.2), so no button offers it.
  2. **The same, with timeline buttons in lessons** (`button(continue)`, `button(replay)`). The learner already has these from the player (PK-9.7); a second set inside the views would duplicate them and depend on each host's transport.
  3. **Runtime controls as their own representation kind** (`run_control(reset)`). Clear, but PK-6.3 lists them under `button`, and authors would learn two kinds for one widget.
- **Accepted position:** option 1, under the owner's standing delegation of 2026-09-30. In `button(...)`, the words `reset`, `undo` and `redo` name runtime controls unless the model declares an event of that name, which then wins (the IR stores a control as a `word` argument and an event as an `element`, so the meaning is fixed once lowered). A runtime-control button in a presentation with a timeline is PK-E05. A press names the event, not the button: an explore beat that offers no button for it refuses the press with a reason. `press`, `undo` and `redo` are contextual keywords (D-040). In frames a button carries `event` or `control`; hosts send `press` with `time` in a lesson (HI-4.3), and Enter or space on a focused button presses it in both modes.
- **Reason:** an explore beat is where the learner acts on the model (D-025), and a button is one of its controls as a slider is; the branch keeps the lesson's own run unchanged (PK-9.9). Reset, undo and redo act on the session, which the engine owns (HI-1.4); the clock does not.
- **Consequences:** PK-6.3d, PK-9.8b; 04-ir (learner step `press`, `word` argument of `button`); working syntax; HI-4.3; parser, lowering, formatter; `CKind::RunButton`, `Shape::Button { event, control }`; `Interactive::press`, `Instance::lesson_press`, the protocol's `press` in lessons; the web player; guide chapters 7, 8 and 10; `prismal-present/tests/buttons.rs`. Not yet: buttons whose event takes a payload (a button supplies none).
- **History:**
  - 2026-10-01 accepted under the owner's standing delegation of 2026-09-30 and implemented.

## D-070: Plot axes that follow the data, display units on plot axes, zoom and pan of plots

- **Status:** Accepted
- **Original position:** PK-7.3 gives a plot axis "a dimension and a display unit, with a range that is fixed, follows the data, or is controlled by the camera". PK-7.3a implemented only fixed ranges in coherent SI units, and the learner's zoom and pan (D-047) applied to spatial views only: a curve that left a plot's ranges could not be followed or looked at.
- **Raised by:** the owner (2026-10-01): an oscillation plotted beyond the visible part of an axis could not be scrolled to; PROJECT-STATE, next steps; `docs/prototype.md`, "Not implemented".
- **Builds on:** PK-7.3, PK-7.3a, MK-3.12, D-047.
- **Question:** How does an author ask for an axis that follows the data and for an axis display unit, and how does the learner look beyond a plot's ranges?
- **Options considered:**
  1. **Named arguments of `plot`, and the learner's zoom and pan for plots.** `follow: y` or `follow: (x, y)` makes those axes grow to hold what the view draws; `x_unit: ms` and `y_unit: cm` choose display units. Plots zoom and pan under the same `permit learner { zoom; pan }` as spatial views.
  2. **Words after the range** (`y: [-1 m, 1 m] follow unit cm`), mirroring a parameter's `unit deg`. Reads well, but needs a new argument grammar in the parser for one view kind.
  3. **A sliding window** that keeps the last span of time in view instead of growing. Suits long runs, but hides the start of the curve; it can be added later as another `follow` mode.
- **Accepted position:** option 1, under the owner's standing delegation of 2026-09-30. A following axis is the declared range grown to hold every point, segment, polyline and polygon the frame draws, with a twentieth of the span as room on the side it grew; it never shrinks below the declared range, and it follows in every medium (sessions, lessons, video). The learner's zoom and pan act on what is shown, followed or not, and `Reset view` returns to it. A display unit must measure the axis's dimension and have no offset (PK-E04); ticks are placed and labelled in it, values in frames stay in coherent SI units.
- **Reason:** named arguments are the form every other view setting already takes, so no new grammar is needed. Growing keeps the whole curve visible, which is what a learner checking where an oscillation ends needs; panning covers looking beyond any range.
- **Consequences:** PK-7.3a, PK-7.3b; 04-ir (plot `follow`, `units`); parser, lowering, formatter; `ViewCtx::Plot { follow, units }`; the host's plot viewport, pan and wheel; layout `follow`, `units`, `unit_scale`; both renderers; guide chapters 7 and 10; `prismal-host/tests/plot_axes.rs`. Added the same day: a sliding window (option 3) as `window: 10 s`, the `x` axis showing the latest span of that length once the data passes the declared range; function graphs drawn over the `x` range the learner has zoomed or panned to (`Projector::set_sample_x`); two-touch pinch zoom on plots. Not yet: plot axes controlled by a timeline camera.
- **History:**
  - 2026-10-01 accepted under the owner's standing delegation of 2026-09-30 and implemented.
  - 2026-10-01 window, function graphs over the range shown, pinch zoom on plots.

## D-071: Drag mode `live`, the interactive failure policy `pause`, and failures located inside a step

- **Status:** Accepted
- **Original position:** PK-10.9 lets a presentation choose drag mode `live`, "where each proposed value is committed as a separate intervention while the run advances", without a syntax. RC-10.3 and RC-10.4 make `pause` the failure policy of interactive runs. The prototype had only `hold` drags; a failed interactive run ended where the failing step ended, with no status for hosts; and a step whose end broke a `reject` or `stop` constraint was kept whole, so the run showed states past the failure, against RC-10.1.
- **Raised by:** PROJECT-STATE, next steps; `docs/prototype.md`, "Not implemented"; the pause test, which found the failure 9 s late with a constant-rate flow.
- **Builds on:** PK-10.5 to PK-10.9, RC-10.1 to RC-10.4, RC-11.1, HI-4.2.
- **Question:** How does an author choose `live`, what does a live drag commit and when, and what do hosts see when an interactive run fails?
- **Options considered:**
  1. **Per representation, `on drag live as p`; commits at each instant the clock passes.** While the pointer is held, the proposal pending at the instant shown is committed when the clock moves on, and the last pointer position is proposed again at the new instant. A pointer held still keeps the value pinned; moves at one instant replace each other before the commit, so the log has one intervention per instant shown.
  2. **Per presentation** (`permit learner { live_drag }`). Simpler, but one presentation may hold some drags (placing) and steer with others.
  3. **Commit on every pointer move.** Ties the log to the pointer's event rate and leaves a still pointer without effect while the model moves the value away.
- **Accepted position:** option 1, under the owner's standing delegation of 2026-09-30. The web player keeps its clock running during a live drag. An interactive session reports `status` (`running`, `paused`, `failed`) and `failure` (message, category, instant); a paused run is resumed by any intervention at the paused instant, which recomputes it from there. A step whose end breaks a `reject` or `stop` constraint is cut, by bisection to the time tolerance, at the last instant found valid; that state is committed and the failure is reported at the first instant found broken. Failure policies chosen per category in a run's configuration (RC-10.3) are not given a syntax: headless runs stop, interactive runs pause (RC-10.4).
- **Reason:** steering is a property of one gesture, like its part; committing per instant matches how a run is computed and logged (RC-11.1) and keeps undo meaningful. Cutting the failing step is what RC-10.1 requires, and it makes the paused instant the one the learner needs to see.
- **Consequences:** PK-10.9a, RC-10.3a; 04-ir (`inverse.live`); parser, lowering, formatter; `Interactive::seek` commits live drags, `Interactive::drag_live`; the host's `pointer_down` answers `live`, `session` answers `status` and `failure`; the web player; `Run::fail_in_step`; guide chapters 7, 10 and 14; `prismal-host/tests/live_and_pause.rs`.
- **History:**
  - 2026-10-01 accepted under the owner's standing delegation of 2026-09-30 and implemented.

## D-072: Buttons whose event takes a payload

- **Status:** Accepted
- **Original position:** D-069 left "buttons whose event takes a payload (a button supplies none)" for later; a button for an event declared `on request(j: Momentum)` could not be written.
- **Raised by:** D-069, "Not yet"; PROJECT-STATE, next steps.
- **Builds on:** PK-6.3, PK-6.3d, D-050, D-059, D-069.
- **Question:** How does a button give the value its event takes?
- **Options considered:**
  1. **The event written as a call:** `button(kick(2 kg*m/s))`, as a timeline writes `request kick(2 kg*m/s)`. Several values are written as several arguments.
  2. **A named property:** `button(kick, value: 2 kg*m/s)`. Explicit, but a second way to write what a request already writes.
  3. **A value read from a control** (a number input beside the button). Useful, but a different feature: a form, not a button.
- **Accepted position:** option 1, under the owner's standing delegation of 2026-09-30. The IR keeps the event as the button's `element` source and the value as its `payload` property, an expression in the model's scope evaluated when pressed. A button gives a value exactly when its event takes one, of the declared type (PK-E05, PK-E04); an event that takes a member is requested by clicking the member (D-059), not by a button. In a lesson, an explore beat's button requests its event with its value on the branch.
- **Reason:** the same form as a request keeps one way to write "this event with this value"; the IR needs no new argument kind.
- **Consequences:** lowering, formatter; `CKind::Button { payload }`; `Interactive::press` and lesson presses request with the value; guide chapters 5, 7, 10 and 14; `prismal-host/tests/payload_buttons.rs`, `prismal-syntax/tests/glossary.rs`. Not yet: option 3.
- **History:**
  - 2026-10-01 accepted under the owner's standing delegation of 2026-09-30 and implemented.

## D-073: Contributions to discrete state

- **Status:** Accepted
- **Original position:** MK-14.9 and MK-16.4 define `contribute(target, value)` for discrete targets whose type or binding declares a combination, with any number of contributions combining and a `set` on the same target conflicting. No type or binding could declare a combination, so the prototype rejected every contribution (MK-E11).
- **Raised by:** PROJECT-STATE, next steps; `docs/prototype.md`, "Not implemented".
- **Builds on:** MK-14.9, MK-16.3, MK-16.4, D-005.
- **Question:** How does a binding declare its combination, which combinations exist, and with what does a contribution combine?
- **Options considered:**
  1. **A binding modifier, `combine sum`,** among a fixed set: `sum` and `product` (numbers; `sum` also vectors), `min`, `max` (numbers and quantities), `any`, `all` (Booleans). The contributions of one transition combine with the value before it: `contribute total += 1` adds one.
  2. **Combinations on types** (D-005's type-supplied default). Needs declared quantity types with combinations, which v0 does not have; a binding modifier works now and a type default can be added later.
  3. **Contributions replacing the value** (combined only with each other). Then `contribute n += 1` would set `n` to 1, against the `+=` written.
- **Accepted position:** option 1, under the owner's standing delegation of 2026-09-30. `combine` is a contextual word, accepted only on discrete state (SX-E06). A contribution to a binding without a combination, or of a type its combination does not apply to, is MK-E11; a `set` and a `contribute` of one binding in one handler are MK-E19, and across handlers of one transition a conflict at run time (MK-16.4). Contributions to components are not allowed.
- **Reason:** a modifier is where a binding already declares how it is shown and changed; reading `+=` as "combine with the current value" is what authors expect from a score or a counter.
- **Consequences:** working syntax (modifier, contextual word); IR `Binding.combine`; parser, lowering, formatter; `COp::Contribute`, the runtime's `combine`; guide chapters 5, 10 and 14. Not yet: combinations supplied by types (option 2).
- **History:**
  - 2026-10-01 accepted under the owner's standing delegation of 2026-09-30 and implemented.

## D-074: `create` and `connect` through part paths

- **Status:** Accepted
- **Original position:** D-065 let a model name a contained object's collection by its path (`left.atoms`) in member expressions, loops, aggregates and relation endpoints, but left out "`create` and `connect` written outside the container of the collection they change": only a cell's own events could make its atoms.
- **Raised by:** D-065, "Not yet"; PROJECT-STATE, next steps.
- **Builds on:** MK-7.10, MK-16.1a, MK-16.1b, MK-16.6, D-057, D-058, D-065.
- **Question:** Can a container make members of a collection held by one of its contained objects, and in which scope are the starting values read?
- **Options considered:**
  1. **Part paths in `create` and `connect`.** `create left.atoms { pos = e }` makes the next member where the collection is held, with its capacity and its count of members made; the starting values and endpoints are read in the scope where the operation is written. Creates of one collection from any handlers of one transition conflict on its count, inside or outside (MK-16.6).
  2. **Requests into the contained object.** The container requests an event of the cell (`request left.grow`), which makes the atom. Keeps each object the only writer of its parts, but needs events addressed to contained objects and payloads to pass the starting values.
  3. **Keep the restriction.** Authors move the collection up to the container, losing the structure the cell expresses.
- **Accepted position:** option 1, under the owner's standing delegation of 2026-09-30. MK-7.10 already lets a container write a part's bindings (`set b.vel = ...`); making a part's members is the same direction, container to part. A path goes through contained objects only (SX-E03 at a collection). In v0 relation sets are held by the model declaring the relation type, so `connect` paths name sets of that model; the elaboration resolves them generally for when relation types may be declared in object types.
- **Reason:** one way to name a collection everywhere (D-065); the existing conflict rule on the count of members made already covers creates from inside and outside at one instant.
- **Consequences:** parser (`create` and `connect` take part paths), lowering (`part_ref`), elaboration (`holder`; the count of members made keyed by its binding; endpoints read in the handler's scope against collections of the set's holder); guide chapter 9, Making members of a contained object; `prismal-syntax/tests/glossary.rs`. Not yet: option 2; relation types declared in object types.
- **History:**
  - 2026-10-01 accepted under the owner's standing delegation of 2026-09-30 and implemented.

## D-075: Animated color, line style and scale, shape morphs, easing, and cameras on plots

- **Status:** Accepted
- **Original position:** PK-8.4 defines an animation "from a value, to a value, with a duration and an easing function". D-068 implemented opacity and offsets with one easing, and left "animated color, line and scale, shape morphs and chosen easing functions" (its option 2). D-042's `camera` moved spatial views only, and PK-7.3 lists plot axes "controlled by the camera" (D-070, "Not yet").
- **Raised by:** the owner (2026-10-01); D-068 and D-070, "Not yet".
- **Builds on:** PK-6.6a, PK-7.3, PK-8.4, PK-8.5, PK-8.7, D-042, D-061, D-068, D-070.
- **Question:** How are these properties written and interpolated so that every medium draws the same animation?
- **Options considered:**
  1. **More properties of `animate`, interpolated by the kernel where it can be and by media where it must be.** `color` blends between palette names: the frame carries the color reached, the color being blended to and the fraction, and each medium mixes the values it maps the names to. `line` switches style halfway, having no values between. `scale` multiplies the drawn geometry about the shape's centre (an arrow about its base) in the kernel; frames carry the factor so that media draw a marker's dot and an arrow's head larger. `morph to R` moves the drawn points to the shape of the representation `R`: shapes of one form point by point, others resampled by length to the same number of points, a closed path turned so that each point travels least, ending in `R`'s own shape. `ease linear|smooth|in|out` chooses the easing; `smooth` stays the default. A camera on a plot centres on a pair of axis values and zooms the axes, through the same viewport as the learner's zoom and pan.
  2. **Colors as values the kernel blends** (RGB in frames). One computation, but frames would fix colors that each medium chooses for its theme (D-061).
  3. **Morphs only between shapes of one form.** Simpler, but the useful morphs (a square into a circle, a path into another) are between forms.
- **Accepted position:** option 1, under the owner's standing delegation of 2026-09-30. A color animation from a representation with no author color starts from the kind's own color. A morph's target may be hidden: its shape is taken before hidden representations are removed from the frame. Unknown colors and line styles are PK-E02, a scale that is not positive is PK-E02, a morph between representations not both drawn in views is PK-E05. Easing applies to `animate`; `reveal`, `hide` and `camera` keep `smooth`.
- **Reason:** names in frames keep each medium free to choose its colors (D-061) while all media agree on the blend; geometry done in the kernel keeps renderers simple and video export identical to the player.
- **Consequences:** PK-8.4a, PK-9.2g; 04-ir (`animate` `word`, `ease`; `to` optional); working syntax (`ease` contextual); parser, lowering, formatter, checks; `AnimValue`, `Ease`, `morph`, `scale_shape`; frame fields `color_to`, `color_mix`, `scale`; the host's plot viewport takes the camera; both renderers (`color-mix` in the browser, `mix_hex` in SVG); guide chapters 8, 10 and 14; `prismal-host/tests/animation.rs`, `prismal-syntax/tests/glossary.rs`. Found and fixed while checking stills: a camera's centre in a plot was not compiled, so the camera zoomed about the axes' middle. Not yet: easing for `reveal`, `hide` and `camera`; line width as an animated property.
- **History:**
  - 2026-10-01 accepted under the owner's standing delegation of 2026-09-30 and implemented.

## D-076: Authors' text, conditions on representations, and titles

- **Status:** Accepted
- **Original position:** PK-6.3 lists `label` (a value shown as text) and `formula`; a lesson narrates. An author could not write a sentence of their own in a view or panel, show something only while a condition holds, or title a presentation or a view: views were captioned with their name and kind (`scene (spatial)`). An early reader asked how Prismal "handles titles and conditional text"; it did not.
- **Raised by:** the owner (2026-10-01).
- **Builds on:** PK-6.1, PK-6.3, PK-6.5, PK-7.2, PK-11.1, MK-3.12, D-057 (representations drawn only while a condition holds).
- **Question:** How does an author write text with live values, text or any representation shown under a condition, and titles?
- **Options considered:**
  1. **`text` with values in braces, a `when:` condition on any representation, and titles.** `text("The ball is {pos.y} above the ground.")`, any model expression in braces, shown with its unit (a binding in its display unit); `{{` and `}}` for literal braces; `at: P` places the text at a point of a spatial view or a pair of a plot. `when: c` on any representation shows it only while `c` holds at the instant shown, reusing the condition D-057 gives members. `title "..."` names the presentation; `title("...")` in a view or panel is its caption.
  2. **String values in the model language** (`if landed then "Landed" else "Falling"`). One sentence could choose between texts, but models would gain a type that only presentations use, against PK-1 (a model never mentions how it is shown).
  3. **A `caption:` argument on views** instead of a `title` representation. Panels take no arguments, so they would need a second form.
- **Accepted position:** option 1, under the owner's standing delegation of 2026-09-30. A text whose value is not available shows `?` for it. A text in a panel cannot be placed `at` a point (PK-E05). The IR keeps a text's template with `{}` for each value as its first source and the values as the following sources, so the formatter writes the text back as authored. Frames carry a `text` as a text item and a placed text as the shape `note`; a `title` is the shape `title`, which renderers use as the view's or panel's caption, keeping its identity in the drawing; the layout gives `title` for the presentation and for each titled view. `title` is a contextual word.
- **Reason:** text that reports the model in the author's words is what a learner reads first; braces are the form authors know from templates, and a condition on representations covers conditional text without bringing strings into models.
- **Consequences:** PK-6.3e, PK-7.2b; 04-ir (presentation `title`, representation kinds `text` and `title`, representation `when` written by authors); working syntax (`title`); parser (`title` item, `parse_expr`), lowering (`interpolate`, `when:`), formatter; `CKind::Text`, `CKind::Title`, shapes `note` and `title`; host layout titles; the web player and the SVG renderer; guide chapters 7, 10, 14 and What goes where; `prismal-host/tests/text_and_titles.rs`, `prismal-syntax/tests/glossary.rs`. Not yet: option 2; text styles (size, emphasis).
- **History:**
  - 2026-10-01 accepted under the owner's standing delegation of 2026-09-30 and implemented; finished by the owner after the session ended, with the SVG caption drawn from the `title` representation.

## D-077: Words at a slider's ends, the percent unit, and values in words (lab authors' requests)

- **Status:** Accepted
- **Original position:** a lab author writing a "state of the system" lab asked for four presentation options: `ends:` on a slider (a caption at each end), `show:` on a slider (a 0 to 10 position in place of the value with its unit), `as: percent` on a label, and a label that maps a value to words by thresholds. None existed; the author worked around them by scaling values in the model. While checking, labels were found to ignore a binding's display unit, which the guide (chapter 2) said they used.
- **Raised by:** third-party lab authors (feedback passed on by the owner, 2026-10-12).
- **Builds on:** PK-6.3, PK-6.4, MK-3.12, D-061, D-076.
- **Question:** Which of the four belong in the language, and in what form?
- **Options considered:**
  1. **Words in the presentation, units in the model.** `ends: ("cold", "hot")` on a slider; `%` as a unit (a hundredth, dimensionless), which as a display unit (`unit %`) shows a share as a percentage in every label, control and text, not only one label; values told in words by one `text` per band with `when:` (D-076), or by an enumeration of the model shown with `label`; `show:` not adopted.
  2. **All four as presentation options,** as asked. `as: percent` would format one label while sliders and formulas of the same value showed a fraction; `show:` would display a scale that is not the model's value.
  3. **A dedicated banded label** (`label(temp, bands: [288 K: "cold", ...])`). Shorter than three texts, but a second way to write what `text ... when:` writes; it can be added if lessons show the pattern is frequent.
- **Accepted position:** option 1, under the owner's standing delegation of 2026-09-30. `ends` applies to sliders only (PK-E05 elsewhere), is drawn by both renderers beside the bar and read in the slider's text alternative (`from cold to hot`). `%` is lexed as a unit symbol and is never a name. A label of a binding shows it in the binding's display unit, as controls and texts do. `show:` is let go: a slider shows the value the model has, with its unit; a dial from 0 to 10 is a parameter of the model (`level: Real in [0, 10]`) from which the physical value is computed, which is what the lab author did.
- **Reason:** words that help a learner read a control belong to the presentation; how a value is measured belongs to the model, where every view of it agrees. A scale that hides the unit would teach the learner a number the model does not have.
- **Consequences:** PK-6.3f; IR argument kind `texts`; lexer (`%`), units, parser guard; lowering (`ends`), formatter; `CKind::Control { ends }`, `Shape::Control { ends }`, `CKind::Label { idx }`; the web player and the SVG renderer; guide chapters 2, 7 (Words for values), 10 and 14; `prismal-host/tests/feedback.rs`, `prismal-syntax/tests/glossary.rs`. Not yet: option 3.
- **History:**
  - 2026-10-12 accepted under the owner's standing delegation of 2026-09-30 and implemented.

## D-078: Findings from the first complete lesson: path breaks, rounding, labels, and `on E`

- **Status:** Accepted
- **Original position:** PROJECT-STATE (2026-10-12) put real content first: complete lessons written as an author would, to find what the language lacks. The first, `examples/projectile.md` (a projectile lab and a lesson of three throws), was written and its stills and video were inspected.
- **Raised by:** writing `examples/projectile.md`.
- **Builds on:** PK-3.1, PK-6.3, PK-6.3e, PK-11.1, RC-8.7, RC-11.1, D-076.
- **Question:** The lesson exposed four problems. What does each need?
  1. **A wrong value, a runtime bug.** In the lesson, `pos.x on landed` read 0 for every landing. The lesson throws again at the instant the ball lands, and `on E` read the last state at that instant, after the relaunch. PK-3.1 says `on(E)` reads "the committed state at the event's final microstep".
  2. **A path drawn back to the start.** A `trace` joined its samples across the relaunch, drawing a line from the landing point to the origin.
  3. **Too many digits.** Texts showed `35.3119 m` where a learner needs `35.3 m`.
  4. **An unwanted label.** Naming the marker `as ball`, so that the lesson can highlight it, wrote "ball" on the field, where it collided with the axis label.
- **Options considered:** for 1, mark the states committed while a logged action applies and stop `on E` before them (taken), or record the action instants separately (a new record that snapshots would also have to cut); for 2, a gap list on the path's frame entry beside its points (taken), or a new field on the path shape (fifteen pattern matches changed for one optional list); for 3, a decimals suffix in braces, `{last:.1}`, the form authors know from templates (taken), or a property on the text (one precision for all its values); for 4, `label: "..."` and `label: none` on markers, arrows and graphs (taken).
- **Accepted position:** the options taken above, under the owner's standing delegation of 2026-09-30. A path (`trace`, `series_plot`) breaks between two samples when, at an instant between them, transitions changed its value (events or interventions), compared before and after that instant; a bounce, which changes the velocity and not the position, does not break a trace of the position. Frames carry `breaks`, the indices that start a new piece; both renderers lift the pen there. `{e:.N}` shows `e` with `N` decimals in its unit or display unit; `-0` shows as `0`. A chosen label replaces the drawn label only; the text alternative keeps the name.
- **Reason:** each is what the lesson needed and none adds a concept: a decimals suffix and a label property are presentation options, the gap list is how a medium should draw a path that jumped, and the `on E` fix makes the runtime do what the specification already says.
- **Consequences:** `Committed::by_action`, `Run::on`; `RepFrame::breaks`, `jumped`; `CKind::Text { places }`, `split_template`, `fmt_places`; lowering of `{e:.N}` and `label: none`, formatter; both renderers; guide chapters 7, 10 and 14; `examples/` with its README and `crates/prismal-web/tests/examples.rs`; `prismal-host/tests/authoring.rs`, `prismal-syntax/tests/glossary.rs`. Not yet: avoiding collisions between labels and axis labels in the renderers.
- **History:**
  - 2026-10-12 accepted under the owner's standing delegation of 2026-09-30 and implemented.

## D-079: Findings from the geometry example: powers in units, morph order

- **Status:** Accepted
- **Original position:** PROJECT-STATE (2026-10-12) listed a geometry proof with morphs and cameras as the second complete example. `examples/pythagoras.md` (a proof of Pythagoras' theorem by rearrangement: a lab with both arrangements, a lesson with cameras and morphs) was written, and its stills and video were inspected.
- **Raised by:** writing `examples/pythagoras.md`.
- **Builds on:** PK-8.4a, PK-11.1, D-075, D-076, D-078.
- **Question:** The example ran as written, with every case passing, and showed two things. What does each need?
  1. **Units written for a parser.** Author texts showed areas as `25.00 m^2`. The `^` is how source text writes a power, not how a learner reads one.
  2. **A morph that flattens.** A triangle morphed to a target whose corners were listed in mirrored order collapsed into a line halfway. This is what PK-8.4a specifies (shapes of one form morph point by point), so the fix is in the author's program; the rule was not stated where authors learn morphs.
- **Options considered:** for 1, superscript powers in the values of author texts (taken), in every human-facing string (labels, controls, text alternatives), or in `unit_text` itself. `unit_text` also builds source text (control values, drag proposals) that the parser reads back, so it must keep `^`. Labels and controls show a parameter's own unit, rarely a power; text alternatives are read by screen readers, which read `m^2` as well as `m²`. For 2, match corners by least travel for polygons too (a rotation of the list cannot fix a mirrored order, and choosing the author's order silently would hide mistakes elsewhere), or state the rule in the guide (taken).
- **Accepted position:** the options taken above, under the owner's standing delegation of 2026-09-30. A value in an author's text writes powers of units as superscripts: `m²`, `m/s²`, `m⁻¹`. Guide chapter 8 says that corners are matched in the order they are written.
- **Reason:** both are small and neither adds a concept: the first is how a medium should show a unit to people, the second documents existing behaviour.
- **Consequences:** `text::superscripts`, applied to the values of `CKind::Text`; guide chapters 7 and 8; `examples/pythagoras.md`, `examples/README.md`. Not yet: superscripts in labels and control values when content shows a power there.
- **History:**
  - 2026-10-12 accepted under the owner's standing delegation of 2026-09-30 and implemented.

## D-080: Findings from the objects example: event detection cost, texts in the framing

- **Status:** Accepted
- **Original position:** PROJECT-STATE (2026-10-12) listed a model of objects and relations as the third complete example. `examples/chain.md` (a chain of beads and links between two hooks; links snap past a strength; the learner hangs weights by clicking beads and pulls the hooks apart) was written, and its stills and video were inspected.
- **Raised by:** writing `examples/chain.md`.
- **Builds on:** RC-7.1 to RC-7.7, D-058, D-064, D-076.
- **Question:** The example ran as written, and showed two problems and one gap in the guide. What does each need?
  1. **A slow run.** A 10 s run of the chain took 2.8 s in a release build, too close to real time for a lab in a browser. Event detection samples each crossing event's guard at four instants of every step, and each sample interpolated the state and computed every derived value again: with a `snap` event per link, 32 full updates of the derived values per step, against 7 evaluations of the flows.
  2. **Texts cut off.** A view is framed by the extent of its shapes, and texts placed in it (`text(..., at: P)`, D-076) were left out, so a text beyond the shapes was cut at the edge of the view.
  3. **A long formula broken before an operator.** The working syntax continues a line that ends in an operator, but the guide did not say so, and a line starting with `+` gives `expected der, found +`.
- **Options considered:** for 1, compute the state at each sample instant once per step and share it among the events (taken), or compute only the derived values each guard depends on (a dependency closure per event; more work, and the shared samples already remove the factor of the number of events); for 2, include a text's anchor in the extent (taken), or measure its width (a medium's fonts are not known to the engine); for 3, a mistake in guide chapter 13 (taken), or a parser message for a line that starts with an operator (worth doing in the syntax review).
- **Accepted position:** the options taken above, under the owner's standing delegation of 2026-09-30. Sample states are computed lazily, in the order events ask for them, so a status raised at a sample is reported exactly as before; results are unchanged. A 10 s run of the chain now takes 1.5 s. A view's extent includes the anchors of its texts; a text runs on to the right of its anchor, inside the margin the renderers keep.
- **Reason:** the first is a cost the language should not charge for repeating an event per member, the second is what an author placing a text expects, and the third documents an existing rule.
- **Consequences:** `Runner::state_in_step`, `guard_in`; `extent` in `prismal-host`; guide chapter 13; `prismal-host/tests/text_and_titles.rs`; `examples/chain.md`, `examples/README.md`. Not yet: computing in each flow evaluation only the derived values the flows need (the chain computes its six presentation aggregates in every evaluation); text alignment at a point (texts start at their anchor, so a label left of a line is placed by hand); a parser message that names a line starting with an operator (syntax review).
- **History:**
  - 2026-10-12 accepted under the owner's standing delegation of 2026-09-30 and implemented.


## D-081: Syntax stability review: options and clauses, and version 1

- **Status:** Accepted
- **Original position:** PROJECT-STATE (2026-10-12) planned a syntax stability review before outside authors write programs: one style for options (named arguments such as `follow: y`, `when: c`, `ends: (...)` against trailing words such as `combine sum`, `ease out`, `unit %`), a parser message for a line that starts with an operator (D-080), then marking the working syntax as version 1.
- **Raised by:** the plan of 2026-10-12, after the three complete examples (D-078 to D-080).
- **Builds on:** D-028, D-035, D-040, D-061, D-070, D-075, D-076, D-077, D-080.
- **Question:** Should options have one spelling, and what has to hold before the syntax is called stable?
- **Findings:** an inventory of the parser showed two families, each used consistently: named options `name: value` only inside parentheses (representations, view kinds, groups), and clause words with a value only after a statement's head (declarations, constraints, equations, events, timeline actions). No form mixed them. Three faults were found:
  1. **Unknown options were ignored.** Named options passed through as generic properties, so `plot(..., folow: y)`, `marker(p, sise: 3)` or `series_plot(..., wen: c)` compiled and did nothing, and `marker(p, colour: red)` was reported as "unknown name `red`". An option written twice was accepted.
  2. **Clause order was fixed and unstated.** `camera v to P zoom z for d` compiled; `camera v for d zoom z` did not, with "expected end of statement". The same held for `reveal`, `animate`, `explore`, a constraint's `within` and `policy`, and a declaration's range before its modifiers. A modifier given twice was accepted.
  3. **A line that starts with an operator** gave `expected der, found +` (D-080).
- **Options considered:**
  1. One family everywhere, named options (`animate R scale to 2, for: 1 s, ease: out`; `param g: T = e, unit: deg`). Statements would read as calls, and every program would change.
  2. One family everywhere, clause words (`marker(pos) color red`). Representations would lose the call form shared with functions, and a representation's options could no longer be told from the start of the next statement on a line.
  3. Keep the two families with one rule, by place: options inside parentheses, clauses after a statement's head; make each family closed and order-free (taken).
- **Accepted position:** option 3, under the owner's standing delegation of 2026-09-30.
  - Each representation kind and view kind has a fixed list of named options (guide chapter 10, Options and clauses); every representation also takes `when`. An option not on the list is SX-E08 with the list and, within an edit distance of two, the nearest option; an option given twice is SX-E09. The value of a rejected option is not lowered, so no second, misleading error follows.
  - Clauses after a statement's head come in any order, each at most once (SX-E09). A declaration has one range, `in` or `where`. The formatter keeps printing one order. An event keeps its order, since its block sits among its clauses.
  - A statement expected where a line starts with a binary operator or `and`/`or` adds: "A line cannot start with `+`: to continue an expression on the next line, end the line before with the operator".
  - The working syntax is version 1, with a stability rule (working syntax section 1.7): version 1 programs compile with the same meaning in every 1.x version; additions, including contextual keywords, are allowed; a new reserved word, a renamed or removed form or a changed meaning needs a register entry naming the version and the rewrite.
- **Reason:** both families read naturally where they are used and are already consistent, so unifying them would cost every program for no gain in clarity; what outside authors need is that a mistake in either is reported, and that the order of clauses is not a rule to memorize. Version 1 gives authors and hosts a promise to build on without closing the language to additions.
- **Consequences:** `Parser::clause`, `BINARY`, `Parser::unexpected` in `prismal-syntax/src/parser.rs`; `options`, `view_options`, `check_options` in `lower_present.rs`; `prismal-syntax/tests/options.rs`; the tests of a circle's `rotate` and a marker's `line` now expect SX-E08 (`prismal-present/tests/shapes.rs`, `prismal-svg/tests/styles.rs`); working syntax sections 1.6 and 1.7 and status; guide chapter 10 (Options and clauses, SX-E08 and SX-E09), chapter 13 (a misspelled option, the operator message); README, guide README, spec README and 01 status lines.
- **History:**
  - 2026-10-12 accepted under the owner's standing delegation of 2026-09-30 and implemented.

## D-082: Release numbering: 0.x releases, branch per minor version

- **Status:** Accepted
- **Original position:** none. The workspace carried version `0.1.0` since the prototype began, with no tags and no statement of what a release number promises. D-081 called the working syntax "version 1", which could be read as a 1.0 release.
- **Raised by:** the owner, 2026-10-13, when opening work beyond the first slice.
- **Builds on:** D-001, D-081.
- **Question:** What does a Prismal release number promise, and how is the next body of work separated from the finished first slice?
- **Options considered:**
  1. Tag the first slice as `v1.0.0`. In Rust (Cargo's semantic versioning) 1.0 promises a stable public API: no breaking change without 2.0. The crates' Rust APIs, the IR and the host protocol carry no such promise yet.
  2. Tag the first slice as `v0.1.0` and continue on a `v0.2` branch (taken).
- **Accepted position:** option 2, chosen by the owner.
  - Releases are numbered `0.x.y` until the project chooses to promise API stability. Under Cargo's rules a 0.x minor version may break the crates' APIs, the IR format and the host protocol.
  - The language promise is separate and unchanged: programs follow the working syntax's own version (D-081, working syntax section 1.7). Working syntax version 1 programs keep compiling with the same meaning, whatever the release number; a break needs a register entry naming the syntax version and the rewrite.
  - `v0.1.0` tags `main` at the end of the first slice (2D mechanics and supporting math, D-001; working syntax version 1). Work for 0.2 is done on the long-lived branch `v0.2`, with feature branches merged into it; `main` receives only fixes to 0.1 until 0.2 is merged.
  - 0.2 opens with the stochastic slice (D-013, D-083).
- **Reason:** a 1.0 number would claim a stability nobody has checked; the crates are a prototype and hosts are expected to drive the API's shape (D-044). Keeping the syntax version separate lets authors rely on their programs while the implementation is still free to change.
- **Consequences:** tag `v0.1.0`; branch `v0.2`; workspace version `0.2.0` on that branch; PROJECT-STATE conventions.
- **History:**
  - 2026-10-13 chosen by the owner and recorded.

## D-083: The stochastic slice: random events, draws and seeds

- **Status:** Accepted
- **Original position:** D-013 accepted two stochastic modes, per-step sampling and exact event-time sampling (exponential waiting times, Gillespie), to be specified when the stochastic slice is planned. MK-10.4 keeps expressions free of draws; RC section 13 fixed only reproducibility (one seed per run, counter-based streams keyed by element identity). MK-14.2 reserved the `stochastic` process kind.
- **Raised by:** the owner, 2026-10-13: version 0.2 starts with the stochastic slice (D-082).
- **Builds on:** D-004, D-013, D-015, D-050, D-057, R-10, R-30, R-61; RC section 13.
- **Question:** How does a model write random behavior, what exactly does a run compute, and how is it reproduced?
- **Content the slice serves:** radioactive decay of individual atoms and of a population, random walks and diffusion, reaction kinetics (A + B to C, reversible reactions) compared with their rate equations, epidemics, dice, coins and sampling experiments.
- **Options considered:**
  1. **Stochastic processes as a new kind of block** (`process decay stochastic { ... }`), with their own rules for time. A second event machinery beside the existing one.
  2. **A random trigger and distribution functions inside the existing event machinery** (taken). Event-time sampling is a trigger, `random(λ)`, whose occurrences are the instants of a Poisson process of rate `λ`; per-step sampling is an ordinary time event (`every Δ`) whose condition or handler draws from a distribution. Handlers, payloads, members, conflicts, logs and snapshots stay as they are.
  3. **Random expressions anywhere** (`derived { noise = normal(0, 1) }`). Breaks MK-10.4: a value read twice would differ, and replay would depend on how often the runtime evaluates.
- **Accepted position:** option 2, under the owner's standing delegation of 2026-09-30.
  - **Random trigger.** `event E on random(λ)`: `λ` is a rate, `Quantity<1/T>` (a frequency; `0.5 /s`). Its occurrences are the jumps of a Poisson process whose intensity is `λ` evaluated on the committed state: between two transitions the rate is constant and the waiting time is exponential, and a transition that changes the rate changes the law from that instant on (memorylessness). A rate of `0` makes no occurrence; a negative rate is a model failure. In v0.2 the rate reads only parameters, constants, discrete state and derived values of those, not continuous state or `t` (MK-E27, as MK-14.12 for flow conditions); rates that vary between events are a later extension. Gillespie's direct method is the special case of one event per reaction channel; the runtime computes the same law by the modified next reaction method (Anderson 2007), which keeps one clock per event and so needs no global ordering of draws.
  - **Zeno.** A random event is not a crossing; MK-15.12 does not apply. A handler may change its own rate (a population decaying at `λ N`).
  - **Draws.** A distribution is a built-in function, written as any call, `normal(0 m, s)`, and allowed only in an event's enabling condition and its operations (handler and Zeno settle operations), MK-E28 elsewhere; the IR keeps it as its own form, `draw`. A name the author declares wins over a distribution, so no version 1 program changes meaning. Distributions: `uniform(a, b)`, `normal(m, s)` (for a vector `m`, independent components with deviation `s`), `exponential(r)` (a rate; the result has the inverse dimension), `bernoulli(p)` (a Boolean), `integer(a, b)` (both ends included), `poisson(m)`, `binomial(n, p)`, `choice(x1, ..., xk)` (equally likely). Invalid arguments (`s < 0`, `p` outside `[0, 1]`, `a > b`) make the draw invalid, a model failure.
  - **Streams.** A draw is a pure function of the run's seed, the event's identity, the draw site's number in the event (sites numbered in order through the elaborated condition, handler and settle operations) and the event's due count (how many times the event has been due, from 0). A random event's clock is one more stream of the event. Draws do not depend on the order in which the runtime evaluates events, nor on draws of other events (RC-13.3). The generator is Philox4x32-10 (Salmon and others 2011).
  - **Seed.** A run's configuration has a `seed` (a whole number, default 0), one more entry of the existing block: `config { seed = 7 }`. The same seed reproduces the run bit for bit on one build and platform (D-015).
  - **Members.** A random event repeated per member (`for a in atoms { event decay on random(lambda) { destroy a } }`) has one clock per member; a member not alive has rate 0.
- **Reason:** the event machinery already gives what random behavior needs (atomic handlers, cascades, payloads, members, logs, snapshots), so random events cost one trigger and one expression and keep every guarantee. Keying draws by event identity and due count keeps a run reproducible under edits elsewhere in the model, and lets a learner compare two seeds of one model.
- **New syntax kept to one form** (owner, 2026-10-13: no new syntax where existing syntax covers the need). Per-step sampling needs none: `every Δ` and a distribution. The seed needs none: an entry of `config`. A keyword `draw` before distributions was considered and dropped: calls already mark a computed value, and the checker confines distributions to events. The trigger `random(λ)` is the one addition, written in the existing trigger form and contextual (no reserved word, working syntax section 1.7). Exact event times written with existing triggers (`at next` with `set next = t + exponential(r)`) are correct only when the author resamples every clock whose rate an event changes, which silently fails for reactions sharing species.
- **Consequences:** MK-10.4, MK-15.3 (trigger table), MK-15.15 to MK-15.17, MK-E27, MK-E28; RC-13.5 to RC-13.8; 04-ir `random` trigger and `draw` expression; working syntax sections 1.1, 1.3, 1.5; `prismal_kernel::random` (Philox4x32-10 checked against the Random123 known answers, samplers checked by their moments); `CExpr::Draw`, `Ctx::draws`; the runtime's clocks and due counts, kept in snapshots; `Config::seed`, `RunConfig::seed`; parser, lowering and formatter; guide chapter 15 (Chance) and the reference; `prismal-present/tests/random.rs` (population and per-member decay against the exponential law, a random walk, a reversible reaction at equilibrium, seeds, resumed runs, diagnostics, shadowing).
- **History:**
  - 2026-10-13 accepted under the owner's standing delegation of 2026-09-30.

## D-084: Many runs: another seed in labs, ensembles, histograms, statistics, staircases

- **Status:** Accepted
- **Original position:** D-083 gave runs a seed and left two needs of probability lessons open (PROJECT-STATE, 2026-10-13): a lab replays one seed, so a learner never sees that runs differ; and a lesson or a test could not show or check how results spread over many runs.
- **Raised by:** the plan of D-083; the owner's guidance of 2026-10-13: reuse existing forms where they serve, and where they limit, choose what serves both old and new needs.
- **Builds on:** D-069 (runtime-control buttons), D-070, D-078 (breaks of sampled paths), D-083; PK-3, PK-4, PK-6.3a.
- **Question:** How does a learner draw another run, how are many runs shown and checked, and how is a value that changes only at events plotted?
- **Options considered:**
  1. **A seed control** (`slider(seed)`): the seed is configuration, not a model binding, so a control would need a new kind of target; a learner gains nothing from choosing seed 17 over 18.
  2. **A runtime-control word** in the existing `button(...)` form, beside `reset`, `undo` and `redo` (taken): `button(reseed)`.
  3. For ensembles, **a reference from a representation to an observation by name** (`histogram(ends)`): one declaration for views and tests, but representations have never referred to observations, and every place that compiles representations would need the presentation's observations.
  4. **An ensemble source** (taken), the same kind of argument as the sampled source `pos every Δ` of PK-6.3a: `n at τ over 200 runs`. The same phrase is an observation's schedule, so tests and views write ensembles alike. Reuses the words `at` and `over`; adds the contextual word `runs`.
  5. For statistics, **new expectation forms** (`ends has mean 500 within 6`) or **function calls on an observation** (taken): `mean(ends) == 500 within 6`, `deviation(ends) in [11, 21]`, in the existing expectation forms.
- **Accepted position:** options 2, 4 and 5, under the owner's standing delegation of 2026-09-30.
  - `button(reseed)` starts a new run from `t0` with the next seed; `reset` keeps replaying the run shown. Hosts receive the seed in the session's clock (`seed`).
  - PK-3.8: `x at τ over n runs`, run `k` with the observed run's seed plus `k - 1` (the first run is the one observed), each ending at `τ`; at most 10 000 runs.
  - PK-6.3e: `histogram(e at τ over n runs, bins: k)` in a plot view whose `x` axis has the values' dimension and whose `y` axis counts runs; drawn as one closed outline, so every renderer already draws it; computed again only when the run's configuration changes.
  - PK-4.1a: `mean(obs)`, `deviation(obs)` (sample standard deviation).
  - PK-6.3d: **found while checking a still of the guide's lab**: a `series_plot` of a count was drawn as separate points, because the breaks of D-078 cut the path at every event. A value that changes only at transitions is now drawn as an exact staircase from the committed states (up to 4000 steps); a flowing value that jumps keeps its breaks. Sampling a staircase every `Δ` was tried first and rejected: with 0.5 s samples a decay of single nuclei looked like jumps of 40.
- **Reason:** each need is met by a form authors already know (a runtime-control button, an argument like a sampled source, a function in an expectation), with one new contextual word. The ensemble keeps the model free of presentation concerns: the runs are the run configuration's, varied only in seed.
- **Consequences:** `Interactive::reseed`, `seed`; host `session_info.seed`; IR `Arg::Ensemble`, `Schedule::At.runs`, `Subject::Statistic`; `data::ensemble`, `MAX_RUNS`; `CKind::Histogram` with its cache, `piecewise_constant`, `stairs`; parser (`at τ over n runs` in arguments and schedules), lowering, formatter; word lists (`runs`, `reseed`); PK-3.8, PK-4.1a, PK-6.3d, PK-6.3e; 04-ir; working syntax; guide chapter 15 (Many runs) and the reference; tests: a lab that replays or draws another run, a histogram of many runs, statistics in a case, and the count at the half-life against the binomial law by a chi-square test (the earlier tests checked means only).
- **History:**
  - 2026-10-13 accepted under the owner's standing delegation of 2026-09-30 and implemented.

## D-085: Findings from the reaction example: `restart`, histogram bars on the values' grid

- **Status:** Accepted
- **Original position:** D-069 gave labs `button(reset)`: a new run with the presentation's configuration and an empty log. PK-9.2 gave lessons `reset`: the lesson's run shown again from `t0`. D-084's histogram divided its range into equal bins.
- **Raised by:** the fourth complete example, `examples/reaction.md` (a reversible reaction with few and many molecules), checked by rendering its lab and lesson.
- **Builds on:** D-069, D-078 to D-080 (findings from examples), D-083, D-084; PK-9.2, PK-9.2a.
- **Findings:**
  1. **A setting that is a starting value could not be tried.** The number of molecules `n0` sets the starting counts. In the lesson, `intervene { set n0 = 20 }; reset` showed the first run again (where `n0` was still 1000) and then applied the change at 8 s to the old counts: the panel read "share 37.50". In the lab, a learner who moved the slider to 20 and pressed Replay (`reset`) lost the setting. Neither form can mean "start again with the settings as they are", and `reset` keeps its meaning in `examples/chain.md`, whose lesson resets to the starting spread.
  2. **A histogram of values on a coarse grid showed a comb.** With 20 molecules the share takes the values `k / 20`; with 25 bins over `[0.5, 1]`, most bins fell between two values and stayed empty.
  3. A fixed `y` range suits either a narrow histogram (a thousand molecules, over 100 runs in one bin) or a wide one: the plot option `follow: y` (D-070) already grows the range to the bars; the example uses it.
  4. Two narrations ran shorter than the system voice; the exporter's notes (D-079) gave the durations.
- **Options considered (finding 1):**
  1. **Change `reset`** to keep the parameter values. Breaks the meaning `examples/chain.md` relies on and makes "back to the start" impossible to write.
  2. **A modeling idiom**: an `on request` event that sets the counts from `n0` (as `relaunch` in `examples/projectile.md`). Time does not start again, so plots against elapsed time continue, and every model would repeat its starting values in a handler.
  3. **A new word, `restart`** (taken), in both places a reset is written: a timeline action and a lab's runtime-control button. A new run from `t0`, every parameter overridden by its value at the instant shown, an empty log.
- **Accepted position:** option 3 for finding 1; for finding 2, bars on the values' grid. Under the owner's standing delegation of 2026-09-30 and the owner's guidance of 2026-10-13 (choose the form that serves both existing and new needs).
  - `restart` (contextual): in a lab, `button(restart)`; in a timeline, an action that makes a new run version and maps the beat to its `t0`. `reseed` is now a restart with the next seed (D-084 had it reset, which also dropped the learner's settings). `reset` is unchanged.
  - PK-6.3e: values on a grid coarser than the bins get one bar per grid value, `d` wide; values on finer grids or none are binned as before.
- **Reason:** "start again with what I have chosen" and "back to how it was" are both things learners and lesson authors mean, and one word cannot carry both. The run already separates configuration (parameters, seed) from the log of actions, so a restart is the configuration of the instant shown with an empty log: no new runtime machinery.
- **Consequences:** `prismal_present::restarted`; `Interactive::restart`, `reseed` reworked; timeline `Restart` (a new run version); IR action `restart`; parser, lowering, formatter, checker, host action list, word lists; frame `grid_step` and grid bars; PK-6.3d, PK-6.3e, PK-9.2 table; reference, working syntax, guide chapter 15; `examples/reaction.md` and its index row; tests `restart_keeps_the_settings` and `histogram_bars_follow_the_grid_of_the_values`.
- **History:**
  - 2026-10-13 accepted under the owner's standing delegation of 2026-09-30 and implemented.

## D-086: Rates that vary between events

- **Status:** Accepted
- **Original position:** D-083 (MK-15.16) let a random event's rate read only parameters, constants, discrete state and derived values of those, so that a rate is constant between transitions; a rate reading continuous state or `t` was MK-E27, "a later extension".
- **Raised by:** the 2026-10-13 handover (stochastic leftovers); the owner chose on 2026-10-13 to finish the stochastic leftovers before the next capability.
- **Builds on:** D-004, D-083; MK-15.15, RC-7.5, RC-7.6, RC-13.8.
- **Question:** How does a model write chance whose rate changes continuously: a counter moving away from a source, an ageing part that fails more often with time, a reaction rate that follows a temperature?
- **Content the change serves:** inverse-square counting, hazard rates that grow with age (Weibull lifetimes), reactions in a changing environment, infection pressure from a continuous quantity.
- **Options considered:**
  1. **Keep MK-E27**; authors approximate with `every Δ` and `bernoulli(λ Δ)`. Wrong by a step-size error, and the step is the author's to choose.
  2. **Thinning** (Lewis and Shedler 1979): draw candidates at a bounding rate and accept with probability `λ / λ_max`. Needs a bound the author declares or the runtime guesses; a wrong bound gives a wrong law silently.
  3. **The integral of the rate as hidden continuous state** in the solver, with the occurrence located as a crossing of `Λ = P`. Exact to the solver's accuracy, but the rate's integral enters the solver's error control and changes the steps of every flow in the model.
  4. **The integral of the rate by quadrature on the dense output of each step** (taken). Within a step the discrete state is fixed and the dense output gives the continuous state at any instant, so the rate is a smooth function of time on the step: its integral is computed by adaptive Gauss-Legendre panels, and the occurrence is located by bisection and cut as a crossing is.
- **Accepted position:** option 4, under the owner's standing delegation of 2026-09-30.
  - MK-15.16: a rate may read continuous state and time, directly or through derived values; MK-E27 is retired. The checker marks such a rate as varying (`CTrigger::Random { varying }`).
  - RC-13.8a: a varying clock has no scheduled instant; its integral grows along each accepted step; reaching its target cuts the step there (to `ε_t`), and it fires as any clock. Targets come from the same stream as before, so a constant rate made varying keeps its draws.
  - A negative rate inside a step is a model failure, located to `ε_t`.
  - No new syntax: `random(λ)` takes any rate expression.
- **Reason:** option 4 is exact to the solver's accuracy (the law tested against the closed forms below), adds no state to the solver, leaves models without varying rates bit for bit as they were, and keeps every guarantee of D-083 (streams by event identity, snapshots, resumed runs).
- **Consequences:** MK-15.15, MK-15.16; RC-13.8a; `prismal_kernel::check::random_rate_varies`; runtime `integrate_rates`, `locate_clocks`, `fire_clock`; guide chapter 15 (Rates that change between events) and the reference (MK-E27 retired); tests: a count under the rate `k t / tau` against its Poisson law (mean and variance), its waiting time (first two moments), a rate read from cooling continuous state, resumed runs, a rate that turns negative.
- **History:**
  - 2026-10-13 accepted under the owner's standing delegation of 2026-09-30 and implemented.
