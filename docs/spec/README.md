# Prismal Core Semantics Specification

This directory holds the normative specification of Prismal's core semantics. It defines what a Prismal model means and how it behaves, independently of any syntax (D-006) and of any implementation.

- **Version:** v0 (draft)
- **Status:** in progress. Sections marked `Proposed`, if any, depend on divergence register entries that are not yet accepted.

## Documents

| File | Part (D-011) | Status |
|---|---|---|
| `01-model-kernel.md` | Model kernel: values, types, bindings, identity, objects, relations, collections, domains, expressions, functions, equations, constraints, state, processes, events, operations | v0 draft |
| `02-runtime-contract.md` | Runtime contract: clocks, steps, commit, event instants, randomness, snapshots, replay | v0 draft |
| `03-presentation-kernel.md` | Presentation kernel: observation and data, projection, representation, view, explanation timeline, interaction | v0 draft |
| `04-ir.md` | Semantic IR: serializable form of the model (full) and of presentations and runs (outline) | v0 draft |
| `05-host-interface.md` | Host interface: how any system embeds Prismal (engine, documents, instances, frames, JSON protocol, bindings), D-044, D-045 | v0 draft |
| `reference-programs/` | Acceptance suite with expected results (D-012): RP-01 to RP-08 | v0 draft |

## Conventions

- **Normative words.** MUST, MUST NOT, SHOULD and MAY are used as in RFC 2119. Text without them is explanatory.
- **Rule IDs.** Normative rules carry IDs of the form `MK-4.2` (model kernel, section 4, rule 2), `RC-x.y` (runtime contract) and `PK-x.y` (presentation kernel). Rule IDs are permanent once the containing document reaches v1; in v0 they may be renumbered.
- **Sources.** Every section opens with a sources line (D-003):
  - `Restates:` carried-forward resolutions (`R-##`, see `docs/decisions/carried-forward.md`).
  - `Decisions:` register entries (`D-###`, see `docs/decisions/divergence-register.md`).
  - `Prior art:` external systems or theory the section follows or departs from, with the reason.
- **Example syntax.** Examples use the working syntax chosen by the syntax study (D-028, `docs/syntax-study/working-syntax.md`). It is at version 1 (D-081, working syntax section 1.7). The normative content is the semantics, not the notation. Examples not yet converted are marked `sketch (non-binding, D-006)`.
- **Precedence.** Divergence register, then this specification, then the carried-forward record, then the exploration record (`docs/design/`). A conflict between this specification and an accepted register entry is a defect in the specification.
- **Serializable form.** Every construct defined here has a representation in the semantic IR required by D-019 and R-47. The IR format is specified separately; this specification defines the content it must carry.
