# Learn Prismal

A guide to writing Prismal programs, from a first formula to a narrated lesson and on to fields, solved equations, constrained motion, three dimensions, algorithms and optics.

- **Where to begin.** New to simulations and mathematical programming? Read [Start here](00-start-here.md) first. New to Prismal only? Start with [the tour](00-tour.md).
- **How it is laid out.** The guide has two kinds of page. The **learning path** is read in order: each chapter introduces a few forms, shows complete programs that use them, and ends with exercises. The **lookup pages** are kept open beside it at any point: which block holds what, which words to combine for a goal, every form on one page, worked answers, recipes and common mistakes. Chapter numbers are file names, not reading order: chapters 10 to 14 are lookup pages.
- **Checked:** every program in the guide is compiled and every case in it is run by `cargo test` (`crates/prismal-web/tests/guide.rs`). A program that fails, or an example error that is not reported as stated, fails the test suite.
- **Version:** the guide teaches version 1 of the syntax. Later versions add forms without breaking version 1 programs.

## The learning path

Read the start pages, then chapters 1 to 9 in order. They cover everything most programs need.

| Page | Introduces |
|---|---|
| [Start here](00-start-here.md) | the ideas before the code, in plain words: models, quantities, rates of change, simulation, events, tests, and a glossary |
| [0. A tour](00-tour.md) | one program in four steps: a model, a test, a lab and a narrated lesson, with its video |
| [1. A first program](01-first-program.md) | models, parameters, derived values, functions, a plot, a slider, a formula, a test case |
| [2. Quantities and units](02-quantities-and-units.md) | dimensions, units, named types, ranges, constants, how unit errors are reported |
| [3. Space, points and vectors](03-space-and-vectors.md) | spaces, points, vectors, components, the spatial view, dragging |
| [4. Motion](04-motion.md) | state, flows, contributions from several processes, time, solvers, traces and series plots |
| [5. Events and modes](05-events-and-modes.md) | discrete state, events and triggers, operations, Zeno behavior |
| [6. Checks and tests](06-checks-and-tests.md) | checked constraints, checked equations, observations, run cases, expectations |
| [7. Presentations](07-presentations.md) | views, representations, controls, inverses, what the learner may do |
| [8. Lessons](08-lessons.md) | timelines, beats, narration, explore beats, requested events, learner scripts |
| [9. Systems of objects](09-objects.md) | object types, parts, collections, members, aggregates, forces between members |

Then any of these, as a subject calls for it. Each builds on chapters 1 to 9 and on the chapters named.

| Chapter | Introduces | Builds on |
|---|---|---|
| [15. Chance](15-chance.md) | random events at a rate, distributions, seeds, testing random models | 5 |
| [16. Fields](16-fields.md) | values at every point of a line or a plane: heat, waves, grids, boundaries, heatmaps | 4, 7 |
| [17. Solving equations](17-solving-equations.md) | unknowns found from equations: circuits, equilibria, equations that give rates | 6 |
| [18. Constrained motion](18-constraints.md) | constraints the runtime enforces: a bob on a rod, a bead on a wire, a double pendulum, a point held on a curve, contact with a floor, a slack rope, friction and bounces | 4, 17 |
| [19. Shared definitions and modules](19-modules.md) | definitions outside models, for models of any space; `use` and the first-party `mechanics` module | 9 |
| [20. Three dimensions](20-three-dimensions.md) | spaces of three dimensions, `cross`, views turned by azimuth and elevation, in perspective, with opaque bodies; turned by the learner or a lesson: a charge in a magnetic field, a tilted orbit, a door on a hinge | 3, 7 |
| [21. Algorithms](21-algorithms.md) | statements run in order and paused at `step`: bubble sort drawn as bars, binary search by hand or on a clock, Euclid's algorithm; members chosen by computed numbers, `floor` and `mod` | 5 |
| [22. Light and rays](22-light-and-rays.md) | the first-party `optics` module: Snell's law and total internal reflection with a draggable laser, a thin lens and its image, a prism splitting white light | 3, 19 |
| [23. Frames](23-frames.md) | axes of the model's own: gravity read along a slope, a carousel seen from its seat, Coriolis and centrifugal forces on a turntable | 3, 4 |
| [24. Light as a wave](24-light-as-a-wave.md) | the wave functions of `optics`: two sources interfering as a field, fringes behind slits and gratings in the colour of their light, polarizers | 16, 22 |

## Lookup pages

| Page | Use it to |
|---|---|
| [What goes where](00-structure.md) | see one program with every block labelled by its purpose, and what each block holds |
| [14. I want to...](14-i-want-to.md) | start from a goal and find the words that combine for it, with the block each goes in |
| [10. Reference](10-reference.md) | look up any form, word, library function or diagnostic code |
| [11. Solutions](11-solutions.md) | check your answers to the chapters' exercises; each answer is checked by the tests |
| [12. Recipes](12-recipes.md) | copy a short program for a common task, and see how to test, export and embed |
| [13. Common mistakes](13-common-mistakes.md) | understand an error message and fix it |

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
- Numbers in expectations were computed independently, by hand or from closed-form formulas, not copied from the implementation.
