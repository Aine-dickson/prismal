# Learn Prismal

A guide to writing Prismal programs in the working syntax (D-028), from a first formula to a narrated lesson. New to simulations and mathematical programming? Read [Start here](00-start-here.md) first. New to Prismal only? Start with [the tour](00-tour.md). Two pages help at any point: [What goes where](00-structure.md) (which block holds what) and [I want to...](14-i-want-to.md) (which words to combine for a goal). Each chapter introduces a few forms, shows a complete program that uses them, and ends with exercises. After the last chapter, `10-reference.md` lists every form on one page.

- **Status:** living. Written 2026-09-30 against the prototype (`crates/`). The syntax is the working syntax, version 1 (D-081): later versions add forms without breaking version 1 programs.
- **Checked:** every program in the guide is compiled and every case in it is run by `cargo test` (`crates/prismal-web/tests/guide.rs`). A program that fails, or an example error that is not reported as stated, fails the test suite.

## Chapters

| Chapter | Introduces |
|---|---|
| [Start here](00-start-here.md) | the ideas before the code, in plain words: models, quantities, rates of change, simulation, events, tests, and a glossary |
| [What goes where](00-structure.md) | one program with every block labelled by its purpose, and a table of what each block holds |
| [0. A tour](00-tour.md) | one program in four steps: a model, a test, a lab and a narrated lesson, with its video |
| [1. A first program](01-first-program.md) | models, parameters, derived values, functions, a plot, a slider, a formula, a test case |
| [2. Quantities and units](02-quantities-and-units.md) | dimensions, units, named types, ranges, how unit errors are reported |
| [3. Space, points and vectors](03-space-and-vectors.md) | spaces, points, vectors, components, the spatial view, dragging |
| [4. Motion](04-motion.md) | state, flows, time, solvers, traces and series plots |
| [5. Events and modes](05-events-and-modes.md) | discrete state, events and triggers, operations, Zeno behavior |
| [6. Checks and tests](06-checks-and-tests.md) | constraints, equations, observations, run cases, expectations |
| [7. Presentations](07-presentations.md) | views, representations, controls, inverses, what the learner may do |
| [8. Lessons](08-lessons.md) | timelines, beats, narration, explore beats, requested events, learner scripts |
| [9. Systems of objects](09-objects.md) | object types, parts, collections, members, aggregates, forces between members |
| [10. Reference](10-reference.md) | every form, reserved words, diagnostic codes |
| [11. Solutions](11-solutions.md) | worked answers to the chapters' exercises, each checked by the tests |
| [12. Recipes](12-recipes.md) | short programs for common tasks, and how to test, export and embed |
| [13. Common mistakes](13-common-mistakes.md) | the errors met first, what they mean and how to fix them |
| [14. I want to...](14-i-want-to.md) | goals, and the combination of words that achieves each, with the block each word goes in |
| [15. Chance](15-chance.md) | random events at a rate, distributions, seeds, testing random models |
| [16. Fields](16-fields.md) | values at every point of a line or a plane: heat, waves, grids, boundaries, heatmaps |
| [17. Solving equations](17-solving-equations.md) | unknowns found from equations: circuits, equilibria, equations that give rates |
| [18. Constrained motion](18-constraints.md) | constraints the runtime enforces: a bob on a rod, a bead on a wire, a double pendulum, a point held on a curve, contact with a floor, a slack rope |
| [19. Shared definitions and modules](19-modules.md) | definitions outside models, for models of any space; `use` and the first-party `mechanics` module |
| [20. Three dimensions](20-three-dimensions.md) | spaces of three dimensions, `cross`, views turned by azimuth and elevation, in perspective, with opaque bodies; turned by the learner or a lesson: a charge in a magnetic field, a tilted orbit, a door on a hinge |
| [21. Algorithms](21-algorithms.md) | statements run in order and paused at `step`: bubble sort drawn as bars, binary search by hand or on a clock, Euclid's algorithm; members chosen by computed numbers, `floor` and `mod` |
| [22. Light and rays](22-light-and-rays.md) | the first-party `optics` module: Snell's law and total internal reflection with a draggable laser, a thin lens and its image, a prism splitting white light |
| [23. Frames](23-frames.md) | axes of the model's own: gravity read along a slope, a carousel seen from its seat, Coriolis and centrifugal forces on a turntable |
| [24. Light as a wave](24-light-as-a-wave.md) | the wave functions of `optics`: two sources interfering as a field, fringes behind slits and gratings in the colour of their light, polarizers |

## Running a program

**In the web player.** Build and serve the player (`web/README.md`), open the Source tab, paste a program, and press Compile and open (Ctrl+Enter). Diagnostics are listed with their line and column; clicking one selects the text. The guide's programs are also in the player's Program list.

**From the command line.** A program saved in a file (or a Markdown file whose `text` and `cases` blocks form the program) runs its cases headless:

```sh
cargo run -p prismal-web --bin prismal -- cases my-program.prismal
```

Each expectation is printed as `pass` or `FAIL` with the reason. `cargo run -p prismal-syntax --example prismalc -- FILE` prints the IR, or the diagnostics.

## How a program is organized

A Prismal program has up to four kinds of top-level items:

1. `space`: a geometric space, such as the plane.
2. `model`: what exists and how it behaves: parameters, state, derived values, flows, events, equations, constraints. A model never mentions how it is shown.
3. `presentation`: how a model is shown and what the learner may do: views, representations, controls, observations, and optionally a timeline (a lesson).
4. `run`: a test case: a configuration of a run of the model, with expected results.

One model can have several presentations (a lab, a lesson, a presentation that only observes for tests). The guide builds each of these in turn.

## Conventions in this guide

- Programs are in `text` blocks and test cases in `cases` blocks. A block that starts with `space` or `model` begins a new program; the blocks after it belong to it.
- Fragments that are not whole programs (one declaration shown on its own) are in `prismal` blocks and are not compiled by themselves; the same forms appear in the chapter's full programs.
- Examples of mistakes are in `error` blocks. Each starts with a comment naming the diagnostic the compiler reports, `// error: MK-E01`.
- Numbers in expectations were computed independently (by hand or with `tools/refvals.py`-style closed forms), not copied from the implementation.
