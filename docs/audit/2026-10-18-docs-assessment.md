# Documentation Assessment: Learning Prismal as a Newcomer

- **Date:** 2026-10-18
- **Scope:** the root `README.md`, `docs/guide/` (start pages, chapters 1 to 24, reference, solutions, recipes, common mistakes, goal index), `examples/README.md`, and the project documents a contributor reads first (`docs/PROJECT-STATE.md`, `docs/prototype.md`).
- **Question:** can a reader who has never seen Prismal learn to use it from these pages, and what keeps the pages correct as the language grows?
- **Sources:** two reviews merged here. (1) An independent review of the public documentation on `main`, made from the published pages without building the project; its findings P-01 to P-08 are kept by reference below. (2) A review of the `v0.3` branch with the repository checked out and the test suite run (390 tests passing, every guide program included). Every finding below was verified against `v0.3` on this date.
- **Finding IDs:** `DQ-##`, permanent; DQ-16 was found while closing the others. Status is kept in section 4 and updated as findings are closed.

## 1. Summary verdict

The documentation is strong in teaching method and weak in **keeping early statements true as the language grows**.

What works: complete programs instead of fragments, each with a stated goal and how it is built; a page for readers new to simulation; a tour that grows one program from model to video; a page mapping every block; a goal index; worked mistakes with diagnostic codes; exercises; and every program compiled and run by the tests, so no example can silently stop working.

What fails: the guide was written chapter by chapter as the language was built. When a later decision extended an existing construct (solved equations, D-092; enforced constraints, D-093), the new chapter was written and the cross-cutting map (`00-structure.md`) was updated, but the **earlier chapters' categorical statements were not revisited**. A newcomer reading in order is told that constraints are "never enforced", that equations "do not compute anything" and that models "are causal", then later taught the opposite. The tests cannot catch this: they check that programs run, not that prose is true.

The same cause explains the coverage gaps: the pages that serve every chapter (solutions, common mistakes, the goal index, the reference) stopped keeping pace around chapter 9 to 15.

Neither problem needs a restructuring. Both need a fixed set of checks applied whenever a capability lands (section 6).

## 2. Strengths to preserve

| Strength | Where |
|---|---|
| Ideas before code, in plain words, assuming only school arithmetic | `00-start-here.md` (the bathtub) |
| One program grown in stages: model, test, lab, narrated lesson, video | `00-tour.md` |
| Every block labelled by purpose, with what it holds and where it goes | `00-structure.md` |
| Goal and "How it is built" before each program | chapters 1 to 24 |
| Prerequisites recalled before new material, and new execution models taught by contrast | chapters 15 (chance), 16 (fields), 19 (modules), 21 (algorithms) |
| Work backward from a goal to the words that combine for it | `14-i-want-to.md` |
| Wrong programs shown with their diagnostic and the reason | `13-common-mistakes.md`, `error` blocks in chapters |
| Every program and every stated diagnostic checked by `cargo test` | `crates/prismal-web/tests/guide.rs` |
| A practical entry path: install, test, run, browser, export | root `README.md` |

## 3. Findings

Findings are classed as in the independent review, so that the class decides the fix:

- **Contradiction:** two passages make incompatible claims. Fix the earlier passage.
- **Missing connection:** the information exists but the reader is not led to it when needed. Add a scoped sentence and a link.
- **Coverage gap:** a cross-cutting page does not cover later chapters. Extend the page.
- **Navigation:** the reader cannot tell where to go next or what a page is for.
- **Editorial:** text that is correct but harder than it needs to be for a newcomer.

### 3.1 Contradictions

**DQ-01. Constraints: "never enforced" against `policy enforce`.** High. (P-01)
`06-checks-and-tests.md` line 17: "A constraint is a condition on the model's values, checked by the runtime; it is never enforced by moving values (D-020)", followed by the policies `reject`, `report`, `stop`. Chapter 18 teaches `policy enforce` (D-093, D-098, D-102): the runtime keeps the condition true, moves the state onto it and solves for the holding force. The reference and `00-structure.md` already describe both.
*Fix:* a constraint is checked or enforced; chapter 6 teaches the three checking policies and names `enforce` with a link to chapter 18.

**DQ-02. Equations: "do not compute anything" against solved equations.** High. (P-02)
`06-checks-and-tests.md` line 7 ("it does not compute anything by itself") and line 34 ("It does not compute anything (models are causal in v1, D-007)"); `04-motion.md` line 43 ("A state variable with no flow stays constant"). Chapter 17 (D-092, D-097) teaches equations that determine unknowns and equations that give a state's rate when no flow does. "Models are causal in v1" is no longer true since D-092.
*Fix:* one distinction taught the same way in chapters 4, 6 and 17: an equation is checked; or, when it reads an unknown, solved for it; or, when it reads `der(x)` and no flow gives that rate, gives the rate. Chapter 4: "with no flow and no equation giving its rate".

**DQ-03. Events: "a handler (what it does)" read as required.** Medium. (P-03)
`05-events-and-modes.md` line 3: "An event has a trigger (when it happens) and a handler (what it does)." The handler is optional: the root README's `event landed on falling(h)` has none, and the tour and the goal index use events that only mark an instant.
*Fix:* "a trigger, and optionally a handler"; one sentence that an event without a handler marks an instant for observations, tests and other events.

### 3.2 Missing connections

**DQ-04. Circular derived values point to "solve by hand", not to unknowns.** Medium. (P-04)
`13-common-mistakes.md` line 125: "If the two equations are meant to be solved together, solve them by hand, or make one of the values state." Chapter 17's `unknown` block does exactly this.
*Fix:* keep why the cycle is invalid; point to `unknown` and chapter 17 as the way to state values found together.

**DQ-05. "Each plays one of three roles" reads as the complete list.** Medium. (P-05)
`00-start-here.md` line 33. `00-structure.md` lists `const` and `unknown` blocks too.
*Fix:* "three main roles"; one line naming constants and unknowns as forms met later, with links.

**DQ-06. "One line per state value" in the block map.** Low. (P-06)
`00-structure.md` line 47. Chapter 4 teaches several `der(x) += e` contributions from named processes, summed.
*Fix:* "here, one line per state value; several processes may each add to a rate (chapter 4)".

**DQ-07. The README's first event has no handler and no word about it.** Low. (P-08)
Root `README.md`, `event landed on falling(h)`.
*Fix:* one sentence: an event can exist only to mark an instant, here observed as the fall time.

### 3.3 Coverage gaps

**DQ-08. No worked solutions after chapter 9.** High.
`11-solutions.md` covers chapters 1 to 9. Chapters 16 and 18 to 24 end with exercises and have no answers anywhere; chapters 15 and 17 answer some inline. A reader learning alone cannot check the work in the newest half of the guide.
*Fix:* a section per chapter in `11-solutions.md`, each answer a program checked by the tests like the rest.

**DQ-09. Common mistakes stop at chapter 9 material.** Medium.
`13-common-mistakes.md` has sections Units, Names, Models, Presentations, Runs. Nothing on mistakes with unknowns and solved equations, enforced constraints, modules and `use`, algorithms, three-dimensional views, frames or optics.
*Fix:* a section per later topic, built from the diagnostics those chapters introduce (for example MK-E34 to MK-E37).

**DQ-10. The goal index does not reach the newest chapters.** Medium.
`14-i-want-to.md` has no entries leading to chapters 15, 19, 20, 23 or 24. Goals such as "see it in three dimensions", "describe motion from a turning seat" or "show fringes behind two slits" cannot be found from it.
*Fix:* entries for every chapter, in the existing Goal / words / block form.

**DQ-11. The reference lacks the wave optics functions.** Low.
`10-reference.md` has no `two_slit`, `single_slit`, `grating`, `bright_angle`, `malus`, `brewster`, `wavenumber` (D-107).
*Fix:* add them beside the other `optics` functions.

### 3.4 Navigation

**DQ-12. The guide index mixes the learning path with lookup pages.** Medium. (P-07, extended)
`docs/guide/README.md` says "After the last chapter, `10-reference.md` lists every form", but the reference is chapter 10 and chapters 15 to 24 follow it. Chapters 10 to 14 (reference, solutions, recipes, common mistakes, goal index) sit between learning chapters 9 and 15, so a reader cannot tell which pages to read in order.
*Fix:* split the index into "Learn, in order" (start pages, chapters 1 to 9, 15 to 24) and "Look things up" (10 to 14), without renumbering files.

**DQ-13. Three entry pages overlap without stating their roles.** Low, review. (independent review, F-04)
Start here, What goes where and the tour each introduce models, flows and events at different depths. Overlap is useful, but each page should say who it is for, what it assumes and where to go next.
*Fix:* a two-line "For / Next" note at the top of each.

### 3.5 Editorial

**DQ-14. Internal codes on reader pages.** Low.
Register IDs (`D-###`) appear in reader-facing chapters, up to 10 or 11 times each in chapters 7 to 9, and spec codes (`MK-`, `PK-`, `RC-`) in chapters 2, 5, 7, 8, 9 and 15. The guide index opens with "working syntax (D-028)" and cites `tools/refvals.py`. A newcomer does not need them; contributors can find decisions from the register.
*Fix:* keep diagnostic codes (readers meet them in compiler output); move decision and spec references out of running text, to a closing "Design notes" line per chapter or nowhere.

**DQ-15. Nothing can be seen without building.** Low, owner decision.
The first view of a running presentation needs Rust, the WebAssembly target and a local server. A hosted build of the web player, or a short animation in the README, would show what Prismal does before installation.

### 3.6 Closed in the same session

- `docs/prototype.md` stopped at D-093; brought up to D-107.
- `PROJECT-STATE.md` Document map described the guide as "chapters 1 to 8"; "Current position" still said no DSL was adopted. Both corrected.
- Guide index listed chapter 24 before 23; root README omitted later topics, `lib/` and `examples/`. Corrected.

## 4. Status

| ID | Class | Priority | Status |
|---|---|---|---|
| DQ-01 | Contradiction | High | Closed 2026-10-18: chapter 6 teaches the checking policies as those of the chapter and names `enforce` with a link to chapter 18 |
| DQ-02 | Contradiction | High | Closed 2026-10-18: chapter 6 gives the three roles of an equation in a table; chapter 4 qualifies "no flow" |
| DQ-03 | Contradiction | Medium | Closed 2026-10-18: chapter 5 and Start here make the handler optional and say what an event without one is for |
| DQ-04 | Missing connection | Medium | Closed 2026-10-18: the circular example points to `unknown`, with a tested program |
| DQ-05 | Missing connection | Medium | Closed 2026-10-18: "three main roles", constants and unknowns named with links |
| DQ-06 | Missing connection | Low | Closed 2026-10-18 |
| DQ-07 | Missing connection | Low | Closed 2026-10-18 |
| DQ-08 | Coverage gap | High | Closed 2026-10-18: every exercise of chapters 15 to 24 solved in chapter 11 with programs checked by the tests; each chapter links to its solutions |
| DQ-09 | Coverage gap | Medium | Closed 2026-10-18: a "Later chapters" section with nine checked error examples |
| DQ-10 | Coverage gap | Medium | Closed 2026-10-18: six goals for chapters 15, 19, 20, 23 and 24 |
| DQ-11 | Coverage gap | Low | Closed 2026-10-18: a "First-party modules" section lists every definition of `mechanics` and `optics` |
| DQ-12 | Navigation | Medium | Closed 2026-10-18: the guide index is a learning path (with what each later chapter builds on) and lookup pages |
| DQ-13 | Navigation | Low | Closed 2026-10-18: the tour and the block map say who they are for |
| DQ-14 | Editorial | Low | Closed 2026-10-18: decision and specification references removed from the guide, the root README and the examples; diagnostic codes kept |
| DQ-15 | Editorial | Low | Partly closed 2026-10-18: the root README opens with an animation of the tour's lesson. A hosted web player remains an owner decision |
| DQ-16 | Contradiction | Medium | Found and closed 2026-10-18 while solving the exercises: chapter 17's exercise 2 named a binding `on`, a reserved word; it is now `connected` |

**Lesson from DQ-16.** An exercise is text that no test runs. Solving every exercise with a checked program, as section 6 requires, is what finds such mistakes.

## 5. Principles for writing the guide

These hold for every page a learner reads. They extend the goal-first form the chapters already use.

1. **Scope every simplification.** An introduction may teach part of a construct, but it must say so: "the policies of this chapter", "the three main roles", "in this example". Words such as *never*, *only*, *always*, *does not*, *cannot* and *every* are used only for rules true of the whole language, now. When a later chapter extends a construct, the first introduction names the extension and links to it.
2. **Introduce a concept with its cooperating parts.** At a concept's first substantive introduction, give its role, the parts it works with and a complete minimal program. A first mention needs only a short definition; full detail belongs in the reference or a later chapter.

   | Level | The reader gets |
   |---|---|
   | First mention | a short definition, enough to follow the sentence |
   | First substantive introduction | the role, the cooperating parts and how they relate, one complete minimal program |
   | Later chapter or reference | full syntax, edge cases, advanced forms |

3. **Concept first, details later.** Define the concept and the problem it solves, show its whole structure, walk through one complete program, then explain each part, then return to the program.
4. **Every page is understandable at its place in the path.** Pages need not be self-contained, but a page must not rely on a term the reader has not met yet, and must say what it assumes.
5. **Reader pages carry no internal references.** Decision and specification codes belong to contributors' documents. Diagnostic codes stay: the compiler prints them.
6. **A finding names its class.** Contradiction, missing connection, coverage gap, navigation or editorial. This keeps preference from being reported as defect.

## 6. Checks when a capability lands

The guide stays correct only if these run with every capability, as part of its branch, before it merges. They are listed in `docs/PROJECT-STATE.md` (Working process) so each session sees them.

**When a decision extends an existing construct** (a new policy, a new role for an equation, a new kind of event):

- Search the guide for the construct's name together with *never*, *only*, *always*, *not*, *cannot*, *every*, *causal*; revise each statement the extension makes false, and add a forward link at the construct's first introduction.
- Update the construct's row in `00-structure.md` and its entry in `10-reference.md`.

**When a chapter or a form is added:**

- `docs/guide/README.md`: a row in the learning path, in reading order.
- `11-solutions.md`: a checked answer for every exercise.
- `13-common-mistakes.md`: the mistakes the new diagnostics catch.
- `14-i-want-to.md`: at least one goal leading to the chapter.
- `10-reference.md`: every new form, word and library function.
- Root `README.md`: the guide row's topic list, if the topic is new.
- `docs/prototype.md`: a row in "Capabilities of 0.2 and 0.3" (or its successor) and the "Not implemented" items from the decision's "Not yet".

**Before a version merges to `main`:**

- Read the start pages and chapters 1 to 6 against the decisions of the version, as a newcomer would.
- Check that relative links resolve and that the writing rules hold (CI checks the latter).
