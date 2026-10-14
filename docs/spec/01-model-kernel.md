# 01 Model Kernel

- **Version:** v0 (draft), 2026-09-29, revised 2026-09-30
- **Part:** model kernel (D-011). The runtime contract (`02`) and presentation kernel (`03`) are separate documents.
- **Scope:** the meaning of a model: what exists in it, what values it holds, what is true of it, and how it may change. How changes are computed and scheduled is the runtime contract. How the model is shown and manipulated is the presentation kernel.

Every section follows accepted decisions (including D-020 to D-023, raised by this document) or carried-forward resolutions, including D-027 (raised by reference program RP-08) and D-029, D-030, D-032 (raised by the syntax study).

## Contents

1. Overview
2. Values and types
3. Quantities, units and dimensions
4. Spaces, points and vectors
5. Status and validity
6. Bindings and roles
7. Objects, identity and composition
8. Collections and relations
9. Domains and fields
10. Expressions and functions
11. Equations
12. Constraints
13. Dependency analysis
14. Evolution: targets, flows and contributions
15. Events
16. Operations
17. Interface to the other kernels
18. Static diagnostics
19. Check against the first-slice reference programs
20. New positions taken in this document
21. Deferred

---

## 1. Overview

- **Restates:** R-04, R-05, R-06, R-07, R-45, R-47, R-63, R-66.
- **Decisions:** D-011, D-019.
- **Prior art:** Modelica and FMI separate a model's declaration from its simulation; this kernel follows that split. It departs from Modelica in being causal in v1 (D-007).

A **model** is a definition. It is not a simulation, a scene or a run (R-07). A model definition, together with a run configuration (parameter values, solver settings, seeds), is what the runtime executes.

The model kernel has these concepts. Every other construct in the language (property, parameter, variable, constant, derived value, predicate, rule, equation as written by an author) is a role of these concepts or a convenience that lowers into them (R-45, R-63).

| Concept | Section |
|---|---|
| Value, type | 2, 3, 4 |
| Status (validity) | 5 |
| Binding | 6 |
| Object, identity, system | 7 |
| Collection, relation | 8 |
| Domain, field | 9 |
| Expression, function | 10 |
| Equation | 11 |
| Constraint | 12 |
| State, process, flow | 14 |
| Event | 15 |
| Operation | 16 |

- **MK-1.1** The model kernel MUST NOT reference any concept of the presentation kernel. A model is complete and executable without any presentation (D-011).
- **MK-1.2** Author-facing constructs MUST NOT express implementation detail: solver, buffer, cache, storage layout or hit testing (R-04). Execution configuration is separate from the model and is visible to the author where it changes results (D-010).
- **MK-1.3** Every construct in this document MUST have a representation in the semantic IR, carrying at least the information this document assigns to it, with a stable identity independent of its source name (R-47, D-019).

### Example notation

Examples use the working syntax (D-028, `docs/syntax-study/working-syntax.md`). It is at version 1 (D-081); the normative content is the semantics, not the notation.

```text
model Ball in Plane {
  param   { mass: Mass = 0.5 kg; e: Real = 0.8 in [0, 1) }   // parameters
  state   { pos: Point = origin; vel: Vector<Velocity> = 0 }  // continuous state
  derived { ke: Energy = 0.5 * mass * |vel|^2 }                // derived binding
  flow {
    der(pos)  = vel                                            // definition
    der(vel) += (0, -9.81 m/s^2)                               // contribution
  }
  event bounce on falling(pos.y) { set vel.y = -e * vel.y } zeno stop
}
```

---

## 2. Values and types

- **Restates:** R-48, R-49, R-50, R-51.
- **Decisions:** D-014, D-015.
- **Prior art:** follows ML-family type systems (nominal records and variants, structural functions) and Rust traits for explicit capabilities (R-49). Follows IEEE 754 binary64 for real arithmetic, as Modelica tools, Simulink and most scientific runtimes do.

### 2.1 Type forms

| Type form | Meaning | Identity |
|---|---|---|
| `Boolean` | true, false | built in |
| `Integer` | mathematical integers | built in |
| `Real` | real numbers; identical to the dimensionless quantity (section 3) | built in |
| `Quantity<D>` | magnitude with dimension `D` (section 3) | structural in `D` |
| `Instant<D>` | affine point on a quantity line, such as a time instant or a temperature on a scale (section 3.4) | structural in `D` |
| `Point<S>`, `Vector<S, D>` | points and vectors of a space `S` (section 4) | structural in `S`, `D` |
| `Tuple<A, B, ...>` | ordered fixed-size product | structural |
| record | named fields | nominal |
| enumeration | tagged alternatives, each with an optional payload | nominal |
| `Option<T>` | a `T` or absent | structural |
| `Sequence<T>`, `Set<T>` | ordered and unordered finite collections of values | structural |
| `A -> B` | function | structural |
| `Distribution<T>` | probability distribution over `T` (R-10) | structural |
| `Equation` | symbolic equation (section 11) | built in |
| object type | objects with identity (section 7) | nominal |
| relation type | relation instances with identity (section 8) | nominal |
| `Ref<T>` | reference to an object or relation of type `T` by identity | structural in `T` |

- **MK-2.1** Types describe values. A type is not a value, identity, role, state or machine representation (R-48).
- **MK-2.2** Declared records, enumerations, object types and relation types are nominal: two declarations with the same structure are distinct types. A type alias is not a new nominal type (R-48, R-49).
- **MK-2.2a** In v0 an enumeration is declared in a model (`enum Phase { climbing, sinking, landed }`) and is known by its identity (D-049). Its cases are distinct names in the namespace of the model's bindings and functions. A case is typed by its context: the enumeration expected where it is written. Values of an enumeration are compared only with `==` and `!=`; they have no order and no arithmetic. Payloads of cases (section 2 table) are deferred.
- **MK-2.3** A capability (interface) is a named set of operations and bindings. A nominal type satisfies a capability only by explicit declaration (R-49). A generic parameter may be constrained by capabilities. In v0 the only generic parameters the kernel itself requires are spaces and dimensions (sections 3 and 4); general user generics follow the same rule.
- **MK-2.4** Value restrictions such as `mass > 0 kg` are constraints (section 12), not refined types (R-50).
- **MK-2.5** `Option<T>` expresses a value that may legitimately be absent. Absence is a value. It is distinct from every status of section 5 (R-51). There is no `null`.
- **MK-2.6** Values are immutable. State change is expressed only by operations on bindings (section 16), never by mutating a value.

### 2.2 Numbers

- **MK-2.7** `Real` denotes the real numbers. Its computational form is IEEE 754 binary64. Deviations between the two are numerical error, governed by the determinism levels of the runtime contract (D-015).
- **MK-2.8** A non-finite result (NaN, infinity) is never a value. An operation that would produce one yields the status `invalid` (section 5) with the cause recorded.
- **MK-2.9** `Integer` denotes the integers. Its computational form is a 64-bit two's-complement integer. Overflow yields `invalid`; it never wraps.
- **MK-2.10** `Integer` converts to `Real` implicitly where a `Real` is required. `Real` converts to `Integer` only through explicit functions (`round`, `floor`, `ceil`, `trunc`).

---

## 3. Quantities, units and dimensions

- **Restates:** R-08.
- **Decisions:** D-014, D-021, D-030.
- **Prior art:** follows the SI (9th edition, 2019) for base dimensions and coherent units. Follows F# units of measure in checking dimensions statically. Follows Pint and Boost.Units in treating offset scales (degree Celsius) as affine, where Pint separates `degC` from `delta_degC` and Boost.Units separates `absolute<>` temperatures from differences.

### 3.1 Dimensions

- **MK-3.1** A dimension is a product of powers of the seven SI base dimensions (length L, mass M, time T, electric current I, thermodynamic temperature Θ, amount of substance N, luminous intensity J) with rational exponents. The dimension with all exponents zero is **dimensionless**.
- **MK-3.2** `Quantity<1>` (dimensionless) and `Real` are the same type.
- **MK-3.3** Named dimensions are aliases (R-48): `Velocity = L/T`, `Acceleration = L/T^2`, `Force = M L/T^2`, `Energy = M L^2/T^2`; for circuits (D-092) `Voltage = M L^2/(T^3 I)`, `Charge = T I`, `Resistance = M L^2/(T^3 I^2)`, `Capacitance = T^4 I^2/(M L^2)`, `Inductance = M L^2/(T^2 I^2)`, with `Current = I` and `Temperature = Θ`. Libraries MAY define more. Quantities of the same dimension are interchangeable in the kernel; distinguishing quantity kinds of equal dimension (torque and energy) is deferred (section 21).

### 3.2 Arithmetic

- **MK-3.4** Addition, subtraction and comparison require operands of the same dimension. Multiplication and division combine dimensions. Integer and rational powers scale exponents. Functions such as `sin`, `exp`, `log` require dimensionless arguments. `sqrt` halves exponents. Violations are static errors.
- **MK-3.5** Dimension checking is static. Every expression has a dimension known before execution. No run-time dimension check is required.

### 3.3 Units

- **MK-3.6** A unit is a named scale factor (and, for affine units, an offset) relative to the coherent SI unit of its dimension. The meaning of a quantity is independent of the unit it was written in: `100 cm` and `1 m` are the same value.
- **MK-3.7** Units appear in literals, in inputs, and as display preferences in binding metadata (section 6). They do not affect computation. The computational form of a quantity is its magnitude in coherent SI units.
- **MK-3.8** **Literals** (D-014): a numeric literal written with a unit has that unit's dimension. A bare literal is dimensionless, with one exception: the literal `0` adopts whatever dimension its context requires, and also stands for the zero vector of any required `Vector<S, D>` (D-030). It never stands for a `Point` or an `Instant`, which have no zero. Any other bare literal combined with a dimensioned quantity is a static error.

### 3.4 Affine quantities

- **MK-3.9** `Instant<D>` is the affine counterpart of `Quantity<D>` (D-014). The rules match points and vectors (section 4):
  - `Instant<D> - Instant<D>` gives `Quantity<D>`.
  - `Instant<D> + Quantity<D>` and `Instant<D> - Quantity<D>` give `Instant<D>`.
  - `Instant<D> + Instant<D>`, scaling an instant, and comparing an instant with a `Quantity<D>` are static errors.
- **MK-3.10** Affine units carry an offset: `°C` and `K` both measure `Instant<Θ>`; a temperature difference is `Quantity<Θ>` and is written in `K` (or a difference unit a library defines). Simulation time `t` is `Instant<T>` (section 14.6).

### 3.5 Angles

- **MK-3.11** Plane angle is dimensionless, as in the SI: `rad` is the coherent unit with scale 1, `deg` has scale π/180, `rev` has scale 2π. Trigonometric functions take dimensionless arguments, so `sin(30 deg)` is valid and equals `sin(π/6)`.
- **MK-3.12** A binding MAY carry a display unit of `deg` or `rad` in its metadata so that presentation shows angles in the author's chosen unit.

---

## 4. Spaces, points and vectors

- **Restates:** R-09, R-11.
- **Decisions:** D-014, D-022, D-032.
- **Prior art:** follows the mathematical definition of an affine space (points, a vector space of displacements, no addition of points). Follows the frame-tagged data discipline of robotics libraries such as ROS tf2, where every geometric value names its frame and conversion between frames is explicit.

- **MK-4.1** A **space** is a domain (section 9) declared by the model with a dimension `n` (a type parameter, D-014) and, in v0, a Euclidean metric. The kernel does not assume any particular `n`; the first slice uses `n = 2` (D-001).
- **MK-4.2** A space has a **standard frame**: an origin and an ordered orthonormal basis with axis names (by default `x`, `y`, `z`). Coordinates of points and components of vectors are read in a frame. A space MAY declare further frames, each defined by a transform from another frame of the same space (D-022).
- **MK-4.3** `Point<S>` is a position in space `S`. Its coordinates have dimension L. `Vector<S, D>` is a vector in `S` whose components have dimension `D`: displacement is `Vector<S, L>`, velocity `Vector<S, L/T>`, force `Vector<S, M L/T^2>`.
- **MK-4.4** Point and vector rules (R-09):
  - `Point<S> - Point<S>` gives `Vector<S, L>`.
  - `Point<S> + Vector<S, L>` gives `Point<S>`.
  - `Point<S> + Point<S>`, scaling a point, and adding a point to a vector of dimension other than L are static errors.
  - Vectors of the same space and dimension add; scaling by `Quantity<E>` gives `Vector<S, D·E>`; dot product of `Vector<S, D>` and `Vector<S, E>` gives `Quantity<D·E>`; the norm `|v|` gives `Quantity<D>`.
- **MK-4.5** Values from different spaces never combine. Moving a value between frames of one space, or between spaces, requires an explicit function (D-022).
- **MK-4.6** Component access `v.x`, `p.y` reads in the standard frame. `v.in(F).x` reads in frame `F` (D-022).
- **MK-4.7** A space is model space. It is not render space (R-07); the mapping to a screen belongs to projection.
- **MK-4.8** **Vector literals** (D-032). A tuple literal `(a, b)` is a `Tuple` unless its context expects a vector, in which case it is that vector, with that space and dimension. The expected type comes from a declaration, a parameter of a called function, a flow target, or the other operand of `+`, `-` or a comparison, and propagates through scaling: in `pivot + L * (sin(θ), -cos(θ))` the tuple is expected to be `Vector<Plane, 1>`. Using a tuple as a vector where no expected type made it one is a static error. A space's origin is `S.origin`, or `origin` inside a model that declares `S` as its default space. A vector MAY also be written with an explicit constructor of its space where no context supplies the type.

---

## 5. Status and validity

- **Restates:** R-16, R-51, R-54.
- **Prior art:** follows IEEE 754 in making invalid results propagate instead of trapping, but departs from it by making the propagated thing a status with a recorded cause rather than a NaN value. Follows spreadsheet error propagation (an error in a cell propagates to its dependents with its kind preserved).

- **MK-5.1** Evaluating an expression yields either a value of its type or a **status**. The statuses are:

| Status | Meaning | Example |
|---|---|---|
| `unknown` | not yet determined | a binding read before its initialization |
| `unavailable` | the source cannot supply a value | a sensor input that is disconnected |
| `pending` | a value is expected but not yet produced | an asynchronous external computation |
| `invalid` | evaluation failed | division by zero, `sqrt` of a negative, overflow, non-finite result |

- **MK-5.2** A status is not a value. It cannot be stored in a binding as if it were one, compared, or passed through arithmetic as a number (R-16). An expression that reads a status yields a status.
- **MK-5.3** When several operands carry statuses, the result status is the highest in the order `invalid` > `unavailable` > `pending` > `unknown`, and it records every cause.
- **MK-5.4** Every status records its **provenance**: the binding or operation where it originated and the cause (R-54). Propagation preserves provenance.
- **MK-5.5** An expression MAY handle a status explicitly with the built-in forms `is_valid(e)`, `status(e)` and `e otherwise d` (use `d` when `e` is not a value). Handling is the only way a status stops propagating.
- **MK-5.6** What the runtime does when a status reaches state (stop, pause, report) is policy of the runtime contract. The kernel only describes what happened (R-54).

---

## 6. Bindings and roles

- **Restates:** R-13, R-46, R-52, R-58, R-63.
- **Decisions:** D-004, D-019, D-023.
- **Prior art:** follows Modelica variability (constant, parameter, discrete, continuous) and FMI causality (parameter, input, output, local), which separate how often a value may change from who may change it. Follows GeoGebra's free and dependent objects for derived bindings: a dependent object is recomputed from its definition and cannot be set directly.

### 6.1 Binding

- **MK-6.1** A **binding** has: identity, name, owner (an object, section 7), type, role, definition (initial value or defining expression), visibility (public or private to its owner), and metadata (label, description, display symbol, display unit).
- **MK-6.2** A source name identifies a binding; the binding has a semantic identity that does not depend on the name (R-52). Renaming a binding does not change its identity.
- **MK-6.3** Lexical scope decides which names are visible where. Ownership decides what a binding belongs to in the model (R-52). Name resolution is a syntax concern; the IR stores resolved identities.

### 6.2 Roles

Every binding has exactly one role. The role fixes when the value may change and what may change it.

| Role | Changes | Changed by | Stored |
|---|---|---|---|
| `constant` | never | nothing | yes |
| `parameter` | only at event instants | interventions and run configuration | yes |
| `input` | at any time | the environment (external source, SEM-07) | yes |
| `discrete state` | only at event instants | operations in event handlers; interventions | yes |
| `continuous state` | continuously, by flows | flows; replaced only by resets and interventions | yes |
| `derived` | whenever its sources change | nothing: it is defined by an expression | no |
| `unknown` | whenever its sources change | nothing: it is determined by the equations that read it (MK-11.6) | no |

- **MK-6.4** A derived binding is a definition, not stored state (R-13). At every instant its value equals its defining expression evaluated on current values. It cannot be the target of any operation.
- **MK-6.4a** (D-092) An unknown is, like a derived binding, a value at every instant and not stored state; where a derived binding has its own defining expression, an unknown is determined jointly with the other unknowns by equations (section 11). It is read like a derived binding and cannot be the target of any operation (MK-E06).
- **MK-6.5** Continuous state changes only by flows (section 14). Its value is replaced only by a reset in an event handler or by an intervention (D-004).
- **MK-6.6** Parameters do not change during continuous evolution. A parameter change is an intervention and happens at an event instant (section 17). Event handlers MUST NOT set parameters; a value that the model itself switches belongs in discrete state.
- **MK-6.7** Every stored binding except `input` has an **initial definition**: an expression over constants, parameters and other initial definitions. Initial definitions are evaluated once, at run start, in dependency order (section 13). An `input` has no initial definition; until the environment supplies it, it is `unavailable`.
- **MK-6.7a** An `input` MAY declare a default (`input { thrust: Force = 0 N }`): its value until the environment supplies one. Without a default, the run configuration supplies its starting value; otherwise the run does not start (D-051).
- **MK-6.8** A parameter's initial definition is its default. A run configuration or the instantiating object MAY override it.
- **MK-6.9** Property, variable, parameter, constant and derived value are roles of the one binding concept. An object's properties are its bindings. Bindings owned directly by the root object are the model's standalone bindings (R-58).

### 6.3 Intervenability

- **MK-6.10** A binding is **intervenable** if the model permits it to be changed from outside the model (by the learner, a timeline or an experiment) through an intervention. Parameters are intervenable by default. State bindings are intervenable only if the model declares them so. Constants, inputs and derived bindings are never intervenable.
- **MK-6.11** A binding MAY declare an intervention range as a constraint (section 12). Interventions outside it are rejected (R-41).

---

## 7. Objects, identity and composition

- **Restates:** R-12, R-21, R-53.
- **Decisions:** D-015, D-019.
- **Prior art:** follows Modelica's single structured class concept (`model`), where a system is a model that contains components. Follows NetLogo in giving every agent a stable identity that is never reused during a run. Departs from Modelica's acausal connectors in v1: connections are causal (D-007).

### 7.1 Objects

- **MK-7.1** An **object type** is a nominal declaration of bindings, contained objects, collections, relations, flows, events, equations and constraints. An **object** is an instance of an object type. It has an identity, and its content is its bindings and contained elements.
- **MK-7.2** A **system** is an object that contains other objects. The kernel has no separate system concept; a subsystem is a contained object. The **model** is one root object type. Surface syntax MAY use `system`, `entity` or other words (R-45).
- **MK-7.3** An object type definition is distinct from its instances (R-53). Instantiation supplies parameter overrides and input connections.
- **MK-7.4** An object type MAY declare that it satisfies capabilities (MK-2.3). There is no implicit inheritance. Specialization is by composition (R-49, R-53).

### 7.2 Identity

- **MK-7.5** Every object, relation instance and binding has an identity. Identity is semantic: it does not depend on names, collection position, order of evaluation or appearance (R-12).
- **MK-7.6** A **declared** element (written in the model definition) has an identity fixed in the IR, stable across edits that do not remove it (D-019). Its identity within a run is the path of declaration identities from the root.
- **MK-7.7** A **created** element (made by a `create` or `connect` operation during a run) has an identity formed from the identity of the creating operation instance and a per-run counter. Identities are deterministic for a given model, run configuration and input history, so replay reproduces them (D-015). An identity is never reused within a run.
- **MK-7.7a** (D-057) In v0 the `k`-th member a collection makes in a run, counting its starting members, has the identity of each declaration followed by `@` and the path `c[k]`, like a declared member (MK-8.3b). The counter is per collection and per run, so identities are deterministic and never reused within a run; iteration order is creation order (MK-8.3).

### 7.3 Composition

- **MK-7.8** Each object owns its bindings. No two objects share a mutable binding (R-53).
- **MK-7.9** A binding may be **read** by its owner, and by any object if it is public.
- **MK-7.10** A stored binding may be **written** (by operations, flow contributions or resets) only by behaviors declared in its owner or in an object that contains its owner. Sibling and contained objects MUST NOT write it. A contained object communicates upward by public bindings and emitted events (section 15).
- **MK-7.11** An object type MAY declare **inputs** (role `input`, section 6.2) and **outputs** (public derived bindings). A **connection** binds a contained object's input to an expression in the containing object. Connections are explicit; there is no implicit namespace or state merging (R-53).
- **MK-7.12** Composition fixes neither execution order nor representation (R-53). Execution order comes from dependency analysis (section 13).
- **MK-7.13** (D-055) In v0 object types are declared in a model (`object Ball { ... }`), with the body of a model, and are used by that model and by the object types it declares. An override given where an object is declared sets the starting value of one of its stored bindings or connects one of its inputs (MK-7.11); a connected input follows the container's expression. An unconnected input keeps its default (MK-6.7a).

---

## 8. Collections and relations

- **Restates:** R-14, R-59, SEM-01 (R-62).
- **Prior art:** follows NetLogo in treating links as first-class agents with identity and properties, distinct from any drawn line. Follows entity-component systems in keeping membership changes as explicit structural operations.

### 8.1 Collections

- **MK-8.1** A **collection** is a binding-like element owned by an object whose content is a finite set of objects of one object type. Its membership is either declared (fixed) or dynamic.
- **MK-8.2** Dynamic membership changes only through `create` and `destroy` operations (section 16), at event instants.
- **MK-8.3** A collection's iteration order is the order of member identities (declared order, then creation order). Expressions over collections (`count`, `map`, `filter`, `any`, `all`, `sum`, `reduce`) use this order, so results are deterministic (D-015).
- **MK-8.4** Every member of a collection is an object with its own bindings, flows and events. A flow or event declared on the member type applies to each member.
- **MK-8.3a** (D-055) In v0 a collection declared without a capacity (MK-8.2a) has fixed membership: a part holds one object or `n` members, numbered from 1 in declaration order. `sum`, `min`, `max`, `any`, `all` and `count` take the members in that order, each named in the aggregate's body and optional filter. Over no member, `sum` is zero, `any` false, `all` true and `count` 0; `min` and `max` have no value. A filter of `min` or `max` may only compare members, which is decided without a value.
- **MK-8.3b** (D-055) A model is elaborated before it is checked: each member becomes the bindings, flows, events, equations and constraints of its type, with the identity of each declaration followed by `@` and the member's path (`row[2]`, `cart.wheels[1]`) and the name `path.name` (MK-7.6). Checks apply to the elaborated model; a diagnostic about an element of a member refers to it and is located at its declaration.
- **MK-8.2a** (D-057) In v0 a collection whose membership changes declares its **capacity**, the most members it makes in one run (`drops: Drop[max 50]`, or `Drop[2, max 50]` with two starting members), and is elaborated to that many members, each **alive** or not. A member not yet made or already destroyed has no flows, events, equations or constraints in effect, is left out of aggregates (`count` counts the members alive; `min` and `max` over none have no value, MK-5.5) and is not drawn (PK-5.1). Read by its number it has its starting values before it is made and its last values after it is destroyed. Making more members than the capacity stops the run at the `stop` constraint `c.capacity` (MK-12).
- **MK-8.2b** (D-066) A collection whose membership changes may declare no limit, `drops: Drop[max inf]`. Its capacity is then a working capacity chosen by the runtime: at least 8, twice its starting members, and every number by which the program names a member (`drops[12]`). A run that makes more members than the working capacity is computed again from its start with the working capacity doubled, up to a limit of the implementation (16384 in the prototype), beyond which it stops as in MK-8.2a. The result is the run of the program with the final working capacity declared, so it is deterministic and replayed identically (D-015).
- **MK-8.8** (D-055) A containing object may write the derivatives of its members' continuous state with flows over a collection (`for b in row { der(b.vel) += e }`, MK-7.10), read in its own scope with the member named. Interactions between members are written this way, with aggregates over the other members.

### 8.2 Relations

- **MK-8.5** A **relation type** is a nominal declaration of endpoint roles (each typed by an object type), whether it is directed, and its bindings (relation properties). A **relation instance** has identity, its endpoints and its bindings (R-14).
- **MK-8.6** Relation instances are held in a **relation set** owned by an object. Membership changes only through `connect` and `disconnect` operations. Destroying an object disconnects every relation instance with that object as an endpoint, in the same transition.
- **MK-8.5a** (D-058) In v0 a relation type is declared in a model with its endpoint roles, each a member of a part of that model: `relation Spring(a in balls, b in balls) { ... }`, with the body of an object type, where `a` is the member at the endpoint. Relations are directed; several relations may join the same members. `s.a` is a member; `s.a == o` compares members and holds only for the same member of the same collection.
- **MK-8.6a** (D-058) In v0 a relation set is a part whose type is a relation type (`springs: Spring[1, max 4] { a = balls[1], b = balls[2] }`), with the capacity, liveness and identities of MK-8.2a and MK-7.7a. Destroying a member disconnects, in the same transition, every relation with that member at an endpoint, in any relation set of the model (D-065). A derived binding of a member or relation that is not alive has no value.
- **MK-8.5b** (D-064) A relation type declared `undirected relation R(a in c, b in c)` has two endpoints in one collection and no order outside its body: there the model reads its endpoints only as `s.has(o)` (whether `o` is an endpoint) and `s.other(o)` (the endpoint that is not `o`, or `o` when both are), which also apply to any relation with two endpoints in one collection. Reading `s.a` of an undirected relation in the model outside its body, or declaring one over two collections, is a static error (MK-E26); a presentation may read `s.a` and `s.b` to draw it.
- **MK-8.5c** (D-065) An endpoint's collection may be a part of a contained object, named by a path through contained objects (`a in left.atoms`); the same paths name members, loops and aggregates (`left.atoms[1]`, `for o in left.atoms`, `count(left.atoms)`). A path never goes through a collection.
- **MK-8.7** A relation is semantic. A line or shape drawn between two objects is a representation and does not imply a relation (R-59). Geometry derived from endpoint positions is a derived binding, not the relation.

The first slice (D-001) exercises neither dynamic collections nor relations. They are defined here so that later slices do not change the kernel.

---

## 9. Domains and fields

- **Restates:** R-11, R-15.
- **Decisions:** D-010, D-014.
- **Prior art:** follows mathematical practice (a function has a domain and codomain). For field state, follows finite-difference and finite-element practice, where the grid and boundary treatment are part of the problem statement because they change the answer.

- **MK-9.1** A **domain** is a set over which a function or field is defined. Domain forms: interval of a quantity (`[0 m, 5 m]`), integer range, finite set, enumeration, space (section 4), product of domains, and the members of a collection. A domain is distinct from a value (R-11).
- **MK-9.2** Time is a domain: the line of `Instant<T>`. A run covers an interval of it.
- **MK-9.3** A **formula field** is a function value whose domain is a space or other domain, for example `gravity : Point<Plane> -> Vector<Plane, L/T^2>`. It is an ordinary function (section 10) and has no storage (D-010).
- **MK-9.4** A **field state** is a state binding whose value is a function over a domain and which evolves. Its discretization (grid or mesh, resolution, boundary treatment) MUST be declared in the model and is visible to the author (D-010). In v0.2 (D-091) the domain is a grid (MK-9.5) and a field holds a number or quantity in each cell: `state { T(x in g): R = e }`, the initial value a function of the cell's coordinate `x`, evaluated in each cell. A **derived field** `derived { q(x in g): R = e }` is defined in each cell the same way. A field is not a parameter, input or discrete state.
- **MK-9.5** (D-091) A **grid** `grid g(x: [a, b], cells: n)` cuts an interval of a quantity into `n` equal cells; `grid g(x: [a, b], y: [c, d], cells: (n, m))` (or `cells: n` for both axes) cuts a rectangle of the model's plane, which requires the model to be `in` a space of dimension 2, with lengths on both axes. Bounds and counts are constant numbers; a grid has at most a million cells. A field's value is the value at each cell's centre; the coordinate of a cell is its centre, a quantity on a line and a point in the plane. Cells are numbered along `x` first.
- **MK-9.6** (D-091) **Boundary conditions** `boundary T { left = e; right = e; bottom = e; top = e }` hold the named edges of field `T` at values (expressions of the type of `T`'s values, evaluated when used; they read no field); edges not named are **insulated** (no flux). `boundary T periodic` wraps the grid around on every axis. The edges of a line are `left` and `right`; of a rectangle also `bottom` and `top` (MK-E33).
- **MK-9.7** (D-091) **Pointwise expressions.** A field's initial value, definition, flow (`der(T) = e`, or `der(T(x)) = e` to name the coordinate), a `set T(p) = e` in a handler and an equation that reads a field are evaluated in each cell of the field's grid: there a field of that grid reads its value in the cell, and `der(T)` its rate (a field of another grid is MK-E31). `laplacian(T)` and `gradient(T)` are written only there (MK-E32) and are second-order central differences across the cell: `(T[i-1] - 2 T[i] + T[i+1]) / h²` summed over the axes, and `(T[i+1] - T[i-1]) / 2h` per axis (a vector of the plane in two dimensions). Beyond an edge held at `g` the difference reads the ghost value `2 g - T` of the cell inside; beyond an insulated edge, the cell inside; on a periodic grid, the cell on the far side. A pointwise `set` reads the values before the transition; it holds no draw (MK-E28), and a field's flow holds no `noise` (MK-E29): each cell would need its own random numbers.
- **MK-9.8** (D-091) **Reading a field.** `T(x)` is the field at coordinate `x`: linear between cell centres (bilinear in the plane), reaching the edge value on the edges (held: its value; insulated: the edge cell; periodic: the mean of the two edge cells), unavailable outside the grid. `integral(T)` is the sum of the cells times the cell's length (area), `mean(T)` their mean, `min(T)` and `max(T)` their extremes. A field is drawn by `function_graph(T)` on a line and `heatmap(T)` in the plane (PK-6.3f).

---

## 10. Expressions and functions

- **Restates:** R-17, R-46.
- **Decisions:** D-007, D-013, D-029, D-039.
- **Prior art:** follows pure functional expression languages (and Modelica functions, which are side-effect free apart from declared external calls). Follows FRP (Elliott and Hudak) in treating a derived binding as a time-varying value defined for every instant, not as a cached result.

- **MK-10.1** An **expression** is a pure, deterministic term: literals, binding reads, operators, function applications, conditionals, collection expressions, and status handling (MK-5.5). Evaluating an expression never changes state (R-17).
- **MK-10.2** Conditionals are total: every `if` has an `else` branch, and `match` on an enumeration covers every alternative.
- **MK-10.3** A **declared function** has typed parameters, a result type and a body expression. Functions are values (`A -> B`). A declared function is pure and cannot read bindings other than its parameters and constants; a declared function that needs model state takes it as an argument (D-029).
- **MK-10.3a** In v0 a declared function is declared in a model and is known by its identity (D-048). Its body may read its parameters, constants and other declared functions; reading any other binding, the time (`t`, `t0`, `elapsed`) or a derivative is MK-E18. A function calls no function that calls it back, directly or through others (MK-E23). A declared function is a value of function type (`A -> B`) and is applied like a derived function value (MK-10.7). A `match` (MK-10.2) has exactly one arm per case of its enumeration; a missing or repeated case is MK-E17.
- **MK-10.4** Expressions do not draw random numbers, except a **draw**, `D(args)` for a distribution `D` (D-083), written only in an event's enabling condition and operations (MK-E28 elsewhere). A draw's value is fixed by its stream (RC section 13), so evaluating it twice at one occurrence gives one value. Distributions: `uniform(a, b)` (a quantity in `[a, b)`), `normal(m, s)` (a quantity, vector or point `m`, each component with deviation `s`), `exponential(r)` (a rate `r`; the result has the inverse dimension), `bernoulli(p)` (a Boolean), `integer(a, b)` (a whole number from `a` to `b`), `poisson(m)`, `binomial(n, p)` (whole numbers of type `Real`), `choice(x1, ..., xk)` (one argument, equally likely). Invalid arguments make the draw `invalid` (MK-2.8).
- **MK-10.5** Evaluation that does not terminate within the runtime's evaluation limit yields `invalid` (runtime contract).
- **MK-10.6** Every expression keeps its symbolic structure in the IR. Its evaluated value is separate from it (R-48: expression is not evaluated value). This lets equations and definitions be displayed, differentiated symbolically by libraries, and consumed by a later acausal solver (D-007).
- **MK-10.7** A **derived function value** is a derived binding (MK-6.4) of function type whose body MAY read model bindings, for example `f(x) = a * x^2` with `a` a parameter. Like every derived binding it is evaluated at every instant on current values, so an application `f(2)` always uses the current `a`; nothing is captured or stored. Its dependencies (section 13) are the bindings its body reads. A stored binding (constant, parameter, input, state) of function type MUST hold a closed function: one that reads no bindings other than its parameters and constants (D-029).
- **MK-10.7a** **Built-in functions** (D-093 adds `dot`): `sin`, `cos`, `tan`, `exp`, `log` of dimensionless values; `sqrt` (half the dimension's exponents), `abs`, `min`, `max`, `atan2(y, x)` of quantities; `dot(a, b)` of two vectors of one space, of dimension the product of theirs, or of two quantities, their product; a tuple of quantities of one dimension beside a vector is a vector of its space (D-032, D-097). A name the author declares wins over a built-in function.
- **MK-10.8** The **named constant** `π` (also written `pi`) is a dimensionless literal kept by name in the IR, so that formulas show it (D-039, D-034). MK-3.8 applies to it as to a bare literal.

---

## 11. Equations

- **Restates:** R-18, R-64.
- **Decisions:** D-007, D-092.
- **Prior art:** follows computer algebra systems (and Desmos and GeoGebra) in keeping equations as displayable, symbolic objects that can be bound to live values. Solved equations (MK-11.6 to MK-11.9) follow Modelica in letting equations determine values, but only for declared unknowns: there is no causalization of the whole model, no tearing and no index reduction.

- **MK-11.1** An **equation** is a first-class element with a left side, a right side (expressions), a relation (`=` in v0), and a role (R-18).
- **MK-11.2** v0 roles (D-007):
  - `display`: shown with its symbols linked to bindings; no runtime effect.
  - `check`: in addition, the runtime evaluates the residual `lhs - rhs` and reports when `|residual|` exceeds the declared tolerance. A violated check produces a diagnostic, never a state change.
- **MK-11.3** Both sides of an equation MUST have the same dimension; a mismatch is a static error, whatever the role.
- **MK-11.4** An equation does not define, assign or evolve any binding, except as a solved equation (MK-11.7, MK-11.9). Evolution is written causally with flows and derived bindings (sections 6, 14), and values that only equations determine together are declared as unknowns (D-092). A constraint the runtime enforces is a constraint with the policy `enforce` (MK-12.6, D-093); its rate that reads an unknown is a solved equation (MK-12.7).
- **MK-11.5** An equation MAY reference `der(x)` for a continuous state `x`; in a check, `der(x)` evaluates to the combined flow of `x` (section 14). This lets `m a = F` be checked against the running model.

**Solved equations (D-092).**

- **MK-11.6** An **unknown** is a binding with role `unknown`, declared in an `unknown` block, of type `Real`, `Quantity<D>`, `Vector<D>` or `Point` (MK-E01 otherwise; D-097 adds vectors and points). A vector or point unknown has one component per axis of its space. Its optional value is its **guess**, an expression of its type that MAY read any binding that does not depend on the unknowns (MK-E14 otherwise); without one the guess is 0. The guess is where the solve starts (RC-5.6) and is never the unknown's value unless it solves the equations.
- **MK-11.7** An equation that is not a check and reads an unknown, directly or through derived bindings, is a **solved equation**. A solved equation keeps its role and is displayed as written. The solved equations of a model determine all its unknowns together:
  - each component of a solved equation is one scalar equation, a vector equation counting once per axis of its space;
  - there MUST be exactly as many equation components as unknown components, every unknown MUST be read by a solved equation, and the unknown components MUST admit a **matching**: each assigned to an equation component that reads its unknown, no equation component assigned twice. A failure of any of these is MK-E34, which names the unknowns and equations involved.
  - The matching is structural. Equations that pass it can still be dependent at a given instant (`x + y == 1` and `2 * x + 2 * y == 2`); that is found by the solve (MK-11.8).
- **MK-11.8** At every instant, the unknowns are values for which every solved equation holds, found by the solve of RC-5.6. Derived bindings that depend on unknowns and are read by the solved equations are computed inside the solve. The solve is one node of the dependency graph (section 13): bindings that read an unknown are evaluated after it, and the values the equations and guesses read before it. Where several solutions exist, the value is the one the solve reaches from the guesses; a guess is part of the problem's statement. Where the solve finds no solution, or the equations are dependent at that instant, every unknown is `invalid` (section 5) with a cause naming the unknowns, and the run fails as for any invalid value (RC section 10).
- **MK-11.9** **Rates from equations.** `der(x)` read by an equation that is not a check, where `x` is a continuous state of type `Real`, `Quantity<D>`, `Vector<D>` or `Point` that no flow defines, is an unknown of the solve: elaboration adds a hidden unknown for the rate of `x` (of dimension `D / T`; for a vector, a vector of dimension `D / T`; for a point, a velocity vector) and the flow `der(x)` equal to it, and the equations read that unknown. So `L * der(I) == V0 - R * I` gives the current through a coil its rate, `m * der(v) == -k * x` a spring its acceleration, and `m * der(vel) == (0 N, -m * g)` a thrown body its acceleration as a vector (D-097). The authored IR keeps the equation as written (04-ir). A solved equation MUST NOT read `der(x)` of a state that a flow defines, nor a field (MK-E35).

---

## 12. Constraints

- **Restates:** R-25, R-41, R-50.
- **Decisions:** D-007, D-020, D-093.
- **Prior art:** checked constraints follow design-by-contract (Eiffel) and database integrity constraints, which check and reject. Enforced constraints (MK-12.6 to MK-12.9) follow the index reduction of differential-algebraic equations (Pantelides 1988; Modelica tools), with the constraint differentiated until it determines the multiplier, and the projection of the state onto the constraint manifold after each step (Hairer, Lubich and Wanner, Geometric Numerical Integration, section IV.4; Ascher and Petzold 1998). They depart from physics engines (Box2D joints), which compute constraint forces from masses the model would have to declare, and from Baumgarte stabilization, whose coefficients are tuned per model.

- **MK-12.1** A **constraint** is a Boolean expression over bindings, with an optional tolerance, and a **policy**:
  - `reject`: an intervention or operation whose result violates the constraint is rejected; the state before it stands (R-25, R-41).
  - `report`: a violation produces a diagnostic; execution continues.
  - `stop`: a violation stops the run with an `invalid` status carrying the constraint as cause.
  - `enforce` (D-093): the runtime keeps the constraint true (MK-12.6 to MK-12.9); it is never violated, so never reported.
- **MK-12.2** A constraint with the policy `reject`, `report` or `stop` is **checked, never enforced**: the runtime never changes state to satisfy it. A system whose motion is constrained MAY be written in coordinates that satisfy the constraint by construction (a pendulum in its angle), with the constraint stated as a check; or with the constraint enforced (MK-12.6).
- **MK-12.3** Constraints are checked at initialization, after every committed transition at an event instant, after every intervention, and on accepted continuous steps. A violation that begins and ends between two accepted steps is not guaranteed to be detected; an author who needs exact detection writes the boundary as an event guard (section 15).
- **MK-12.4** Value restrictions on parameters (`mass > 0 kg`, R-50) are constraints with default policy `reject`. Other constraints have default policy `report`.
- **MK-12.5** Enforced constraints are not checked as in MK-12.3; MK-12.10 says when the runtime projects onto them.
- **MK-12.6** **Enforced constraints** (D-093). A constraint with the policy `enforce` MUST be an equality `a == b` of numbers, quantities, vectors or points, without a tolerance (MK-E01). It is kept true at every committed state.
- **MK-12.7** **Rates.** The constraint's **rates** are its time derivatives along the flows: each continuous state is replaced by its rate (its flow, MK-14.5), parameters, constants, inputs and discrete state have rate zero, derived values are differentiated through their definitions, `t` and `elapsed` have rate 1. Differentiation covers `+ - * /`, powers with a number as exponent, `sqrt`, `sin`, `cos`, `tan`, `exp`, `log`, `abs`, `atan2`, `dot`, components and norms; any other form (`if`, `match`, `otherwise`, `min`, `max`, function values, draws, fields, `der`) is MK-E36. The constraint, then its first rate, then its second, up to its third, are taken in turn:
  - the first that reads an unknown (MK-6.4a), directly or through derived values, is a **solved equation** (MK-11.7) of the model, named after the constraint, and counts in the matching of unknowns with equations (MK-E34). For a body held by a force, the force is the unknown and the second rate reads it through the body's flow: `|bob - pivot| == L` with `der(vel) = g - tension / m * (bob - pivot) / |bob - pivot|`;
  - the ones before it, which read no unknown, are the constraint's **levels**, onto which the state is projected (MK-12.8);
  - if a rate is zero (nothing the constraint reads changes along the flows), the constraint is enforced by projection alone, with no solved equation;
  - if none of the constraint and its first three rates reads an unknown, MK-E36 says so and asks for the force that holds it, as an unknown read by the flows.
- **MK-12.8** **What moves.** Each level moves the state it reads that the levels before it do not read (velocities after positions), continuous state before discrete state. Parameters move only when the change being committed sets them (MK-12.10), and so does a state that the change sets when the level reads it: the bob a learner drags moves, not the bob it hangs from. A level that reads no state and no parameter is MK-E36.
- **MK-12.9** A rate read by the solve is computed from the state at that instant; the solve is the one of MK-11.7 with the enforced constraints' solved equations among its equations. Equations of the first degree in the force (every mechanical constraint) are solved exactly from any guess.
- **MK-12.10** **Projection** (RC-5.10) moves the state onto the levels, in order, by the smallest change: at initialization, at the end of every accepted continuous step, and in every transition (an event's operations, an intervention), before constraints are checked. A projection that fails (dependent levels, no convergence, or levels over parameters alone that the values do not satisfy) makes the state invalid: at initialization the run does not start.

---

## 13. Dependency analysis

- **Restates:** R-35, R-53 (composition invariant 7).
- **Decisions:** D-008.
- **Prior art:** follows Simulink algebraic-loop detection and Lustre and Esterel causality analysis, which reject instantaneous cycles and accept cycles broken by a delay (`pre` in Lustre, an integrator in Simulink).

- **MK-13.1** Before execution, the kernel builds a **dependency graph** whose nodes are bindings and flow targets, and whose edges point from each binding to the bindings its definitions read.
- **MK-13.2** Edges are classified:
  - **instantaneous**: a derived binding reads its sources; an initial definition reads its sources; a flow right-hand side reads its sources.
  - **through state**: a continuous state depends on its flow only by integration; a discrete state depends on event handlers only across an event instant.
- **MK-13.3** A cycle that passes through at least one state edge is valid (R-35).
- **MK-13.4** A cycle consisting only of instantaneous edges is an **algebraic loop** and a static error, unless a solver is declared for it (D-008). Unknowns are that declaration (D-092): values that determine each other are written as unknowns and solved equations (MK-11.6 to MK-11.8), and the solve is one node of the graph. A cycle that passes through the solve node, for example a guess that reads a value depending on the unknowns, is still MK-E14.
- **MK-13.5** The graph of initial definitions MUST be acyclic.
- **MK-13.6** Evaluation order within an instant is a topological order of instantaneous edges, with ties broken by identity order. This order is semantically invisible (results do not depend on it) but it is fixed, so floating-point results are reproducible (D-015).

---

## 14. Evolution: targets, flows and contributions

- **Restates:** R-17, R-56, R-57, SEM-03 and SEM-04 (R-62).
- **Decisions:** D-004, D-005.
- **Prior art:** follows hybrid automata (Alur, Henzinger and others): continuous state evolves by flows within a discrete mode and changes discontinuously only at transitions. Follows Modelica's `der()` for first-order state derivatives. Follows physics engines and Modelica force balances in summing force contributions (D-005).

### 14.1 Processes

- **MK-14.1** A **process** is a named group of behaviors (flows, events, operations) with a **process kind** (R-56, called execution mode there): `continuous`, `discrete`, `algorithmic` or `stochastic`. The IR carries the kind. The word "mode" is reserved for the discrete state that selects flows (section 14.4).
- **MK-14.2** v0 specifies `continuous` processes (flows) and events. `discrete` processes are events on a time schedule (section 15). `algorithmic` and `stochastic` kinds are reserved (section 21).

### 14.2 Evolution targets

- **MK-14.3** An **evolution target** is anything whose value a behavior changes: a stored binding, a component of one (a vector component, a record field), a collection's membership, or a relation set. A flow's target is `der(x)` for a continuous state `x`.
- **MK-14.4** Two targets **overlap** if one contains the other (a vector and its component) or they are the same.

### 14.3 Flows

- **MK-14.5** A flow targets `der(x)` for a continuous state `x` of type `Quantity<D>`, `Vector<S, D>` or `Point<S>`. The flow's value has the derivative type: `Quantity<D/T>` or `Vector<S, D/T>` (for a point, `Vector<S, L/T>`).
- **MK-14.6** Derivatives are first order. A second-order law is written as two states (position and velocity), as in Modelica.
- **MK-14.7** Each `der(x)` is either **defined** by exactly one flow (`der(x) = e`) or receives **contributions** (`der(x) += e`) from any number of flows. Defining and contributing to the same target is a static error (R-57).
- **MK-14.8** Contributions combine through the target's **combination** (D-005): an associative, commutative operation with an identity element. The combination of zero contributions is the identity element.
- **MK-14.9** For every `der(x)`, the default combination is sum (identity `0`): rates of change add (D-005). A binding MAY override its combination. For non-derivative targets (section 16, `contribute`), the combination comes from the target's type if the type declares one, from the binding if it declares one, and otherwise contributing to it is a static error. The core assumes no combination.
- **MK-14.10** Contributions are combined in identity order of their flows, so floating-point results are reproducible (D-015).
- **MK-14.11** A continuous state MUST have a defining flow or a derivative combination; the default sum makes `der(x) = 0` when no flow contributes, which is the correct law for a free particle's velocity.

### 14.4 Modes

- **MK-14.12** A flow MAY be conditional. Its condition and every conditional expression inside a flow's right-hand side MUST depend only on discrete state, parameters and constants, not on continuous state or `t`. The discrete state that selects which flows are active plays the role of the hybrid automaton's mode.
- **MK-14.13** A change of behavior that depends on continuous state (the ball reaches the floor, the spring reaches its rest length) is written as an event (section 15) that sets discrete state. This keeps every discontinuity at an event instant where the runtime can locate it (R-28).

Continuous functions that are not smooth (`abs`, `min`, `max`, `|v|`) are permitted in flows. The runtime contract states the solver accuracy expected at their kinks.

### 14.5 Mathematical meaning

Between two consecutive event instants `t_a` and `t_b`, with discrete state fixed, every continuous state `x` satisfies

```text
x(t) = x(t_a⁺) + ∫[t_a, t] F_x(s) ds
```

where `x(t_a⁺)` is the value committed at `t_a` and `F_x` is the combined flow of `x`. Derived bindings hold their defining expressions at every `t`. The runtime computes an approximation of this solution; its accuracy is execution configuration (runtime contract).

- **MK-14.16** (D-090) A flow MAY contain the built-in **white noise** `noise`, of type `Quantity<T^(-1/2)>`, linearly: `der(x) = a + b_1 * noise + ... + b_k * noise`, with `a` (the drift) and each `b_i` free of noise. Each occurrence of `noise` in the elaborated model is an independent standard white noise `ξ_i = dW_i / dt`, and the flow means the Itô stochastic differential equation `dx = a dt + Σ b_i dW_i`. Linear means: noise appears only in sums, differences, negations, products and quotients by factors free of noise, conditional branches whose condition is free of noise, and the components of a tuple (MK-E30). `noise` elsewhere than in a flow is MK-E29. A name the author declares wins over `noise`. Expressions that read `der(x)` (MK-11.5) read the drift.

### 14.6 Time

- **MK-14.14** Every model has the built-in binding `t : Instant<T>` (simulation time, R-23) and `t0 : Instant<T>` (the run's start instant). `elapsed = t - t0` is a built-in derived `Quantity<T>`.
- **MK-14.15** A model that declares no flows, events or time-dependent derived bindings is **static**. It has meaning without a simulation clock (R-32).

---

## 15. Events

- **Restates:** R-28, R-29, R-60, RUN-02 and RUN-05 (R-62).
- **Decisions:** D-004, D-005, D-009, D-027, D-038.
- **Prior art:** follows hybrid automata (guards and resets), Modelica `when` clauses with `reinit` and `pre`, and FMI event indicators (zero-crossing functions). Follows superdense time (Maler, Manna and Pnueli; Lee and Zheng, Ptolemy II) for event cascades at one instant.

### 15.1 Structure

- **MK-15.1** An **event** has identity, an owner, a **trigger**, an optional **enabling condition**, a **handler**, and an optional typed **payload**.
- **MK-15.1a** In v0 a payload has a name and a type, declared where it enters the event: `on request(j: Momentum)` (the request supplies it) or `on E(j: Momentum)` (the payload `E` occurred or was emitted with, of the same type). Any other source is MK-E24. The payload is read by its name in the event's enabling condition and handler, and nowhere else. `emit E(v)` supplies the payload of `E`'s followers and requires `E` to declare a payload of `v`'s type. Every occurrence in the event log carries its payload (RC-15.1) (D-050).
- **MK-15.1b** (D-057) A model may repeat an event for each member of a collection: `for d in drops { event land on falling(d.pos.y) { destroy d } }`. Each member has its own event, named `land[k]`, in effect while the member is alive; its trigger, condition and handler read the member as `d`, and its handler may write the member's bindings (`set d.vel = ...`, MK-7.10).
- **MK-15.1c** (D-059) In v0 a payload may be a **member** of a collection of the event's model: `on request(b in balls)`, `on E(b in balls)`. The occurrence carries the member's number in its collection; the condition and handler read the member as `b` (`b.pos`, `b == o`), write its bindings (`set b.vel = ...`, one conditional operation per member after elaboration) and may destroy it. The event is enabled only for a member that exists and is alive. An event may declare **several payloads**, `on request(b in balls, j: Momentum)`, carried as one tuple and supplied as a list: `request kick(balls[2], j)`, `emit kick(b, j)`. `emit E(b)` passes a member on; a follower receives members of the same collection (MK-E24 otherwise). Through a relation, `set s.a.vel = ...` writes the member at an endpoint.
- **MK-15.2** An event **occurs** at an event instant `(t, n)` (superdense time: `n` counts successive transitions at the same `t`). Locating `t` and ordering occurrences is the runtime contract; the kernel defines when an occurrence is due.

### 15.2 Triggers

- **MK-15.3** Trigger kinds:

| Trigger | Due when | Notes |
|---|---|---|
| `rising(g)` | `g` crosses zero from negative to positive | `g` is a real-valued or quantity-valued expression over state |
| `falling(g)` | `g` crosses zero from positive to negative | |
| `crossing(g)` | either direction | |
| `at(τ)` | `t` reaches instant `τ` | `τ` may depend on parameters and discrete state |
| `every(Δ, from τ0)` | `t = τ0 + kΔ`, `k = 0, 1, ...` | the discrete process mode |
| `on(E)` | event `E` occurred at the previous microstep | cascades; payload of `E` is readable |
| `on start` | run start, after initialization | |
| `on input(i)` | the environment changes input `i` | |
| `on request` | an intervention requests this event (D-027) | the event may declare a typed payload, supplied by the request |
| `random(λ)` | at the jumps of a Poisson process of rate `λ` (MK-15.15, D-083) | `λ : Quantity<1/T>`, read on the committed state |

- **MK-15.4** Continuous guards are crossings, never level conditions (D-004). A crossing is due at the instant the guard function changes sign in the declared direction; a guard that touches zero without changing sign is not a crossing.
- **MK-15.5** A level condition (a Boolean over state) is permitted only as an enabling condition or in a discrete-time trigger. The enabling condition is evaluated at the instant the trigger is due; the event occurs only if it is true.
- **MK-15.6** Level-style guards written by authors (`y <= 0`) lower to crossings (`falling(y)`) when used as continuous triggers; the lowering is part of the surface language, and the IR stores the crossing and its direction (R-45).

### 15.3 Handlers

- **MK-15.7** A handler is a set of operations (section 16). All reads in a handler see the state **before** the handler's operations (the committed state at the preceding microstep), whatever the order in which the operations are written (RUN-06). Modelica's `pre()` is therefore implicit.
- **MK-15.8** A handler's operations commit together or not at all (R-25). An operation set with a conflict (section 16.3) does not commit, and the conflict is reported.
- **MK-15.9** A **reset** is a `set` operation on continuous state in a handler. It is the only way a model itself replaces continuous state (D-004).
- **MK-15.10** An `emit(E, payload)` operation makes `on(E)` triggers due at the next microstep `(t, n+1)`, as an occurrence of `E` does (MK-15.3, D-041); both at one microstep make each `on(E)` due once. A cascade longer than the declared maximum iteration count stops the run with a diagnostic (D-004; the default count is set in the runtime contract).

### 15.4 Zeno policy

- **MK-15.11** An event is **self-retriggering** if its handler writes a binding that its trigger depends on, directly, through derived bindings, or through the flows of continuous state it depends on. Before a flow is followed, conditionals that test discrete state the handler sets to a constant are resolved to the branch that constant selects; conditions that cannot be resolved keep both branches (D-038). This is decided statically from the dependency graph.
- **MK-15.12** Every self-retriggering event with a crossing trigger MUST declare a **Zeno policy** (D-004):
  - `stop`: when accumulation is detected, stop the run with a diagnostic.
  - `settle { operations }`: when accumulation is detected, apply the given operations instead of the handler, typically setting discrete state that switches to a resting mode (section 14.4).
- **MK-15.13** Accumulation is detected when the event occurs more than `N` times within a window of simulation time `w`, or when successive occurrences are closer than `ε`. The event MAY declare `N`, `w` and `ε`; defaults are set in the runtime contract.

### 15.4a Random events (D-083)

- **MK-15.15** An event with the trigger `random(λ)` is due at the jump instants of a Poisson process whose intensity at `t` is `λ` evaluated on the state at `t`. A rate that reads only parameters, constants, discrete state and derived values of those is constant between two transitions, and the waiting time is exponential; a transition that changes `λ` changes the law from that instant on. `λ = 0` makes no occurrence; a negative `λ` is a model failure.
- **MK-15.16** (D-086) `λ` may also read continuous state and time, directly or through derived values. Such a rate varies between transitions: the event is due when the integral of `λ` along the trajectory reaches the clock's next target (RC-13.8a), an inhomogeneous Poisson process. MK-E27 is retired.
- **MK-15.17** A random event is not a crossing: MK-15.12 does not apply, and its handler may change its own rate. A random event repeated per member (MK-15.1b) has rate 0 while its member is not alive.

### 15.5 Ordering

- **MK-15.14** Events due at the same superdense instant are handled in one transition when their operation sets do not conflict. Order of writing and machine order never decide the result (R-29). Conflicting operation sets follow section 16.3.

---

## 16. Operations

- **Restates:** R-25, R-26, R-41, SEM-08 (R-62).
- **Decisions:** D-004, D-005.
- **Prior art:** follows database transactions (atomic commit, conflict detection) and Modelica's rule that a variable may be reinitialized by only one `when` clause at a time.

### 16.1 Operation kinds

| Operation | Effect | Allowed in |
|---|---|---|
| `set(target, value)` | replace the value of a stored binding or component | handlers; interventions |
| `contribute(target, value)` | combine `value` into a discrete-state target through its combination (MK-14.9) | handlers |
| `create(collection, overrides)` | add a new object; returns its reference | handlers |
| `destroy(object)` | remove an object and disconnect its relations | handlers |
| `connect(relation set, endpoints, overrides)` | add a relation instance; returns its reference | handlers |
| `disconnect(relation)` | remove a relation instance | handlers |
| `emit(event, payload)` | make `on(event)` due at the next microstep | handlers |

- **MK-16.1** Operations are the only way state changes other than by flows. Operations never target derived bindings, constants or inputs. `set` targets parameters only in interventions (MK-6.6).
- **MK-16.2** Operation targets must be writable by the behavior's owner (MK-7.10).

### 16.2 Transitions

- **MK-16.3** The operations of all events handled at one microstep, together with any interventions delivered at it, form one **transition**. The transition reads the state before it, computes the proposed state, validates it against constraints (section 12), and commits it or rejects it as a whole (R-25, R-41). Proposed state is not committed state (SEM-08).

### 16.3 Conflicts

- **MK-16.4** Two `set` operations in one transition whose targets overlap are a **conflict**, even when they write equal values (R-26, D-005). A `set` and a `contribute` on overlapping targets are a conflict. Any number of `contribute` operations on one target combine and do not conflict.
- **MK-16.5** Two `destroy` operations on one object are not a conflict (the result is the same); an operation targeting an object destroyed in the same transition is a conflict.
- **MK-16.6** The runtime never resolves a conflict by picking a winner (R-26). A conflicted transition is rejected and reported; what follows is runtime policy (D-005).
- **MK-16.1b** (D-058) In v0 `connect rs(x, y) { k = e }` makes the next relation of `rs` with the members `x`, `y` at its endpoints, in the order of the roles, and starting values; `disconnect s` ends the relation `s`. Starting values given by `create` and `connect` may set parameters of the member made (MK-6.6 concerns members already alive).
- **MK-16.1c** (D-074) `create` and `connect` name their collection or relation set by a part path through contained objects (`create left.atoms { pos = e }`): the member is made where the collection is held, with its capacity and count of members made; starting values and endpoints are read in the scope of the handler.
- **MK-16.1a** (D-057) In v0 `create c { x = e, ... }` makes the next member of `c` alive, its stored bindings `x` starting at `e`, read in the handler's scope on the state before the transition; bindings not given keep their declared starting values. Several creates of one collection in one handler make consecutive members. `destroy b` ends the member `b`. Creates of one collection from two events handled in the same transition conflict (MK-16.4) on the count of members made. Creation and destruction by intervention (MK-17.2) are not in v0: a presentation requests an event whose handler creates (MK-17.2a).

---

## 17. Interface to the other kernels

- **Restates:** R-19, R-24, R-33, R-40, SEM-07 (R-62).
- **Decisions:** D-009, D-011, D-019, D-023.
- **Prior art:** follows FMI, whose model interface exposes variables with declared causality and variability and allows tunable parameters to change only at event instants.

The model kernel exposes a narrow interface (D-011). The runtime contract and the presentation kernel use only what is listed here.

- **MK-17.1** **Read** (observation, R-19): any public binding, the value of any equation residual, event occurrences with payloads, and structure (object, collection and relation identities). Reading never changes the model.
- **MK-17.2** **Intervene**: a set of `set` operations on intervenable bindings (MK-6.10), and `create` or `destroy` on collections the model declares intervenable. An intervention is delivered at an event instant and validated like any transition (section 16.2). Interventions name semantic targets, never visual ones (R-40).
- **MK-17.2a** **Request** (D-027): an intervention may request an event the model declares with the trigger `on request` (MK-15.3), with its payload. The event's handler runs as model code at the intervention's instant: ownership (MK-7.10), conflicts (MK-16.4) and constraints apply as to any transition. Events without that trigger cannot be requested.
- **MK-17.3** **Supply input**: the environment sets `input` bindings, or marks them `unavailable` or `pending`. External input is distinct from intervention (SEM-07).
- **MK-17.4** **Execution control** (play, pause, seek, reset, branch, used by the explanation timeline, D-009) belongs to the runtime contract. It acts on runs, not on the model, and is not an intervention.
- **MK-17.5** The model never depends on any reader, intervener or controller: removing every presentation leaves its meaning unchanged (D-009, D-011).

---

## 18. Static diagnostics

The kernel defines these static errors. They are detected before execution, on the IR, so a visual editor reports them the same way a compiler does (D-019).

| Code | Error | Rule |
|---|---|---|
| MK-E01 | Dimension mismatch in addition, subtraction, comparison or equation | MK-3.4, MK-11.3 |
| MK-E02 | Dimensioned argument to a dimensionless-only function | MK-3.4 |
| MK-E03 | Bare non-zero literal combined with a dimensioned quantity | MK-3.8 |
| MK-E04 | Invalid affine or point operation (instant + instant, point + point) | MK-3.9, MK-4.4 |
| MK-E05 | Values from different spaces combined | MK-4.5 |
| MK-E06 | Operation targets a derived binding, unknown, constant or input | MK-16.1, MK-6.4a |
| MK-E07 | Continuous state set outside a handler or intervention | MK-6.5 |
| MK-E08 | Handler sets a parameter | MK-6.6 |
| MK-E09 | Write from an object that does not own or contain the target | MK-7.10 |
| MK-E10 | Target both defined and contributed to | MK-14.7 |
| MK-E11 | Contribution to a target with no combination | MK-14.9 |
| MK-E12 | Flow condition depends on continuous state or `t` | MK-14.12 |
| MK-E13 | Level condition used as a continuous trigger | MK-15.4 |
| MK-E14 | Algebraic loop | MK-13.4 |
| MK-E15 | Cycle among initial definitions | MK-13.5 |
| MK-E16 | Self-retriggering crossing event without a Zeno policy | MK-15.12 |
| MK-E17 | Non-exhaustive conditional or match | MK-10.2 |
| MK-E18 | Declared function reads a binding other than its parameters and constants | MK-10.3 |
| MK-E19 | Two `set` operations on overlapping targets in one handler | MK-16.4 |
| MK-E20 | Stored binding of function type holds a function that reads bindings | MK-10.7 |
| MK-E21 | Tuple used as a vector with no expected vector type | MK-4.8 |
| MK-E22 | Target defined twice: two defining flows for one `der(x)`, or two definitions of one binding; a case declared twice | MK-14.7, MK-6.1, MK-2.2a |
| MK-E23 | Declared function that calls itself, directly or through other functions | MK-10.3a |
| MK-E24 | Event payload without a source: not requested, and not following an event that carries a payload of the same type (members of the same collection) | MK-15.1a, MK-15.1c |
| MK-E25 | `index` read outside the overrides of a collection | MK-8.3a |
| MK-E26 | A member that does not exist or is not constant, a part used as a value, an unknown object type, a derived binding given a value in a part, or an aggregate without a value | MK-7.13, MK-8.3a |
| MK-E31 | A field of another grid read in a pointwise expression | MK-9.7 |
| MK-E32 | `laplacian` or `gradient` outside a pointwise expression of the field's grid | MK-9.7 |
| MK-E33 | A boundary of a binding that is not a field, an edge the grid lacks or held twice, or periodic with held edges | MK-9.6 |
| MK-E34 | Unknowns not determined by the solved equations: an unknown no solved equation reads, more or fewer equation components than unknowns, or no matching of unknowns to components | MK-11.7 |
| MK-E35 | A solved equation that reads the rate of a state with a flow, or a field | MK-11.9 |
| MK-E36 | An enforced constraint that cannot be enforced: a form without a rate, no rate up to the third that reads an unknown, or a level that reads nothing the runtime can move | MK-12.7, MK-12.8 |
| MK-E29 | `noise` outside a flow | MK-14.16 |
| MK-E30 | A flow that is not linear in each `noise` | MK-14.16 |

Conflicts between different events can only be detected at run time.

---

## 19. Check against the first-slice reference programs

Each first-slice program (D-012) is checked for expressibility in the model kernel. Correct results are checked later, by execution. Models are given in the working syntax (D-028), non-binding. The complete programs, with presentations and cases, are in `reference-programs/`.

### 19.1 Projectile, with and without drag

```text
space Plane = euclidean(2)

model Projectile in Plane {
  param {
    g:     Acceleration  = 9.81 m/s^2
    k:     Quantity<1/L> = 0        where k >= 0          // drag coefficient; 0 means no drag
    speed: Velocity      = 20 m/s   where speed > 0 m/s
    angle: Angle         = 45 deg   in (0 deg, 90 deg)
  }
  state    { pos: Point = origin; vel: Vector<Velocity> = speed * (cos(angle), sin(angle)) }
  discrete { flying: Boolean = true }                     // the mode
  flow {
    der(pos)  = if flying then vel else 0
    der(vel) += if flying then (0, -g) else 0             // gravity
    der(vel) += -k * |vel| * vel                          // quadratic drag; zero once vel is zero
  }
  event landed on falling(pos.y) { set flying = false; set vel = 0 }
}
```

- Two contributions to `der(vel)` combine by the default sum (MK-14.9, D-005). Expressible.
- `-k * |vel| * vel`: `k` has dimension 1/L, `|vel|` has L/T, `vel` has L/T; the product is L/T², matching `der(vel)`. The dimension check covers the drag term.
- The landing condition is a falling crossing (MK-15.4). At launch `pos.y = 0` while rising, which is not a falling crossing, so no enabling condition is needed.
- Landing switches the mode (MK-14.12) and resets velocity (a reset, MK-15.9); the projectile then stays at the landing point.
- Gap found: the expected result (range `speed^2 sin(2 angle) / g` when `k = 0`) is a check evaluated at one event instant, not continuously. The model kernel has no rule for "evaluate this check when event E occurs". Resolved in the presentation kernel by `on(E)` observations (PK-3.4, R-19).

### 19.2 Bouncing ball

```text
model BouncingBall {
  param    { g: Acceleration = 9.81 m/s^2; e: Real = 0.8 in [0, 1) }    // restitution
  state    { y: Length = 1 m; v: Velocity = 0 }
  discrete { resting: Boolean = false }                                 // the mode
  flow {
    der(y) = v
    der(v) = if resting then 0 else -g
  }
  event bounce on falling(y) {
    set v = -e * v
  } zeno settle { set y = 0 m; set v = 0 m/s; set resting = true }
}
```

- The guard is a falling crossing, so the defect found in the exploration record (level guard `y <= 0` retriggering after `y = 0`, audit F-02) cannot occur: after the reset `v > 0`, so `y` rises.
- `bounce` writes `v`, which its trigger `y` depends on through the flow; it is self-retriggering (MK-15.11) and MUST declare a Zeno policy. Without one, MK-E16.
- The flow condition reads only discrete state `resting` (MK-14.12).
- Expected result: bounce times form a geometric series with ratio `e`; accumulation is detected before the limit `t_∞`. Expressible.

### 19.3 Pendulum

```text
model Pendulum in Plane {
  param {
    g: Acceleration = 9.81 m/s^2; L: Length = 1 m; m: Mass = 1 kg
    pivot: Point = origin
  }
  state   { θ: Angle = 10 deg; ω: Quantity<1/T> = 0 }    // θ dimensionless (MK-3.11)
  derived {
    bob:    Point  = pivot + L * (sin(θ), -cos(θ))
    energy: Energy = 0.5 * m * (L * ω)^2 + m * g * L * (1 - cos(θ))
  }
  flow { der(θ) = ω; der(ω) = -(g / L) * sin(θ) }
  constraint |bob - pivot| == L within 1e-9 m policy report
}
```

- The rod constraint holds by construction (MK-12.2); the constraint is a check. The same pendulum written as a free bob with an enforced rod is in guide chapter 18 (MK-12.6).
- `L * (sin(θ), -cos(θ))`: the sum is added to the point `pivot`, so the tuple is expected to be a vector of `Plane` and is `Vector<Plane, 1>` (MK-4.8, D-032); scaling by `L` gives `Vector<Plane, L>`, so `pivot + ...` is a point (MK-4.4).
- Energy drift is observed through `energy` (a derived binding). Expressible.

### 19.4 Spring-mass

Two continuous states `x`, `v`; `der(x) = v`; `der(v) = -(k/m) * (x - rest)`; derived energy; a `display` equation `T = 2π sqrt(m/k)`. Expressible with the constructs of 19.3. The display equation's two sides have dimension T (MK-11.3).

### 19.5 Function plot with a draggable parameter

```text
model QuadraticDemo {
  param   { a: Real = 1 in [-5, 5] }
  derived { f(x: Real): Real = a * x^2 }
}
```

- The model is static (MK-14.15). `a` is intervenable as a parameter (MK-6.10); its range lowers to a `reject` constraint. Dragging is a presentation-kernel interaction that submits `set(a, value)` as an intervention (MK-17.2); the constraint rejects out-of-range values.
- `f` is a derived function value (MK-10.7, D-029): it reads `a` and changes when `a` changes; MK-10.3 does not apply to it. The plot is a projection of `f`; synchronization of views is a presentation-kernel property. Expressible in the model kernel.

### 19.6 Vector addition

```text
model VectorDemo in Plane {
  param {
    A: Point          = origin + (1 m, 2 m)
    u: Vector<Length> = (3 m, 0 m)
    w: Vector<Length> = (0 m, 2 m)
  }
  derived {
    sum: Vector<Length> = u + w
    B:   Point          = A + sum
    // bad:  Point = A + B            -> MK-E04
    // bad2: Vector<Length> = u + (3, 0)  -> MK-E03
  }
}
```

- Point and vector rules are enforced statically (MK-4.4). Expressible, with diagnostics MK-E03 and MK-E04.

### 19.7 Narrated projectile lesson

The model is 19.1 unchanged. It exposes the event `landed` (MK-17.1). Waiting on it, pausing and seeking are timeline and runtime-contract behavior (MK-17.4). The model kernel needs nothing further. Expressible, pending `02` and `03`.

### 19.8 Findings

| Program | Model kernel | Depends on |
|---|---|---|
| Projectile, no drag | Expressible | Runtime contract (event location) |
| Projectile, drag | Expressible | Runtime contract (solver accuracy) |
| Bouncing ball | Expressible | Runtime contract (Zeno detection defaults) |
| Pendulum | Expressible | Runtime contract |
| Spring-mass | Expressible | Runtime contract |
| Function plot | Expressible | Presentation kernel |
| Vector addition | Expressible | - |
| Narrated lesson | Expressible | Runtime contract, presentation kernel |

One gap: evaluating a check or measurement at an event instant (19.1). It belongs to observation (presentation kernel, R-19), which gets it as a requirement.

---

## 20. New positions taken in this document

Positions that elaborate accepted decisions without changing them are specified here directly. Positions with design weight of their own are register entries.

**Register entries:**

| Entry | Position | Sections |
|---|---|---|
| D-020 | Constraints are checked, never enforced, in v1 (Accepted) | 12 |
| D-021 | Plane angle is dimensionless, with `rad`, `deg`, `rev` units (Accepted) | 3.5 |
| D-022 | Spaces have frames; cross-frame and cross-space values need explicit conversion (Accepted) | 4 |
| D-023 | Only declared bindings are intervenable; parameters by default (Accepted) | 6.3, 17 |
| D-027 | Requestable events: the outside may request events the model declares `on request` (Accepted; raised by RP-08) | 15, 17 |
| D-030 | Literal `0` adopts zero vectors (Accepted; raised by syntax study S-3) | 3.3 |
| D-032 | Tuple literals are typed as vectors by their expected type (Accepted; raised by syntax study S-5) | 4, 18 |
| D-038 | Self-retriggering decided through flows, specialized on the handler's constant discrete writes (Accepted; raised by the prototype) | 15.4 |
| D-029 | Derived function values may read bindings; declared and stored functions stay closed (Accepted; raised by syntax study S-1) | 10, 18 |
| D-039 | Named mathematical constants are kept by name in the IR (Accepted; raised by the text parser) | 10 |
| D-092 | Solved equations: declared unknowns, determined together by the equations that read them; rates given by equations (Accepted) | 6.2, 11, 13 |

**Elaborations (in this specification only):**

- Object types, parts and fixed collections, elaborated before checking (MK-7.13, MK-8.3a, MK-8.3b, MK-8.8; D-055).
- Collections whose membership changes, with a declared capacity; `create`, `destroy` and events per member (MK-7.7a, MK-8.2a, MK-15.1b, MK-16.1a; D-057).
- Relation types with endpoints in collections, relation sets, `connect` and `disconnect` (MK-8.5a, MK-8.6a, MK-16.1b; D-058).
- Members as payloads and several payloads per event (MK-15.1c; D-059).

- `Real` is binary64; non-finite results are `invalid`; integer overflow is `invalid` (2.2).
- Quantities compute in coherent SI units; units affect input and display only (3.3).
- Status precedence and explicit handling forms (5).
- Parameters change only by intervention, never by handlers (MK-6.6).
- Writes only from the owner or a containing object (MK-7.10).
- Deterministic identities for created elements (MK-7.7).
- Combination is a commutative monoid; zero contributions give the identity (MK-14.8).
- Flow conditions depend only on discrete state, parameters and constants (MK-14.12).
- Handlers read pre-transition state (MK-15.7).
- Self-retriggering is decided statically and requires a Zeno policy (MK-15.11, MK-15.12).

---

## 21. Deferred

| Item | Where it will be specified |
|---|---|
| Event location, microstep ordering, cascade and Zeno defaults, solver accuracy | `02-runtime-contract.md` |
| Random streams and stochastic processes (D-013) | Done: MK-10.4, MK-15.15 to MK-15.17, RC-13.5 to RC-13.8 (D-083); rates that vary between events, MK-15.16, RC-13.8a (D-086); white noise in flows, MK-14.16, RC-13.9 (D-090) |
| Random rates that depend on continuous state or time | Later extension of D-083 |
| Algorithmic process mode (RUN-04) | Slice that needs it |
| Field state evolution (D-010) | Done on grids of one and two axes: MK-9.4 to MK-9.8 (D-091); meshes, vector fields, fields in objects, noise in fields later |
| Acausal equations, constraint enforcement (D-007, D-020) | Solved equations done: MK-11.6 to MK-11.9, RC-5.5 to RC-5.9 (D-092). Enforced constraints and constrained mechanics done: MK-12.6 to MK-12.10, RC-5.10 (D-093). Vector and point unknowns done (D-097). Solved equations over fields, inequality constraints (contact, a rope that goes slack), constraints in object types across members (they work as written, untested) later |
| Quantity kinds of equal dimension (torque and energy) | Open; to be raised when a reference program needs it |
| Checks and measurements evaluated at event instants (19.1) | Resolved: `03-presentation-kernel.md` PK-3.4 (`on(E)` observations) and section 4 (expectations) |
| Modules, imports, packaging, versioning of libraries (R-55) | Extension specification |
| IR format | IR specification |
- 2026-10-01 undirected relations: MK-8.5b (D-064); relations across containers: MK-8.5c, MK-8.6a amended (D-065).
- 2026-10-01 collections without a declared limit: MK-8.2b (D-066).
