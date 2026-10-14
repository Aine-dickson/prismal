# 02 Runtime Contract

- **Version:** v0 (draft), 2026-09-29
- **Part:** runtime contract (D-011). It depends on the model kernel (`01`) and is used by the presentation kernel (`03`).
- **Scope:** what any Prismal runtime must do when it executes a model: how time advances, how continuous evolution is approximated, how events are located and ordered, how transitions commit or fail, how interventions and inputs enter, how runs are controlled, and what replay guarantees. It is a contract, not an implementation: any runtime that meets it is correct.

Every section follows accepted decisions (including D-024, raised by this document) or carried-forward resolutions.

## Contents

1. Overview
2. Clocks
3. Runs and run configuration
4. Superdense time and the trajectory
5. Initialization
6. Continuous evolution
7. Event detection and location
8. Event instants and transitions
9. Zeno detection
10. Failures and diagnostics
11. Interventions and inputs
12. Execution control
13. Randomness
14. Snapshots, replay and determinism
15. Output to observers
16. Execution configuration and defaults
17. Check against the first-slice reference programs
18. New positions taken in this document
19. Deferred

---

## 1. Overview

- **Restates:** R-06, R-20, R-24, R-36.
- **Decisions:** D-011, D-015.
- **Prior art:** follows FMI for Model Exchange, whose contract separates the model (states, derivatives, event indicators) from the environment that integrates it and handles events. Follows SUNDIALS (CVODE) and the DifferentialEquations.jl callback design for root-finding event location on dense output.

The runtime contract has the shape given in 06 section 73:

```text
Given   model M (IR), run configuration C, input and intervention log I
produce the trajectory S (committed states), the event log E and diagnostics D
```

- **RC-1.1** Semantic state is authoritative (R-24). Computational state (solver history, step sizes, caches) and presentation state are derived from it or kept alongside it; neither defines the model's meaning.
- **RC-1.2** The execution strategy (solver, tolerances, step control) MAY change the accuracy of results. It MUST NOT change their meaning: which bindings exist, which events can occur, which operations are legal (R-36, invariant 10).
- **RC-1.3** Solvers, root finders and random generators are replaceable components behind this contract (R-06). The contract names required behavior and default choices, not a single implementation.

---

## 2. Clocks

- **Restates:** R-23, R-31.
- **Decisions:** D-009, D-018.
- **Prior art:** follows game-loop practice (fixed simulation step, variable render rate) and video pipelines (media time distinct from wall time).

- **RC-2.1** There are three clocks (R-23):
  - **simulation time** `t : Instant<T>`, the model's time, with superdense extension (section 4);
  - **presentation time**, the time of a playback or export, owned by the presentation kernel;
  - **wall-clock time**, the time of the machine running the program.
- **RC-2.2** The model never reads wall-clock or presentation time. Wall-clock time reaches a run only through inputs and interventions, which are logged with the simulation instant at which they take effect (section 11).
- **RC-2.3** The mapping between simulation and presentation time (real time, slow motion, time lapse, pauses) belongs to the presentation kernel. The runtime's results MUST NOT depend on it: the same run played at any speed produces the same trajectory.
- **RC-2.4** When an interactive player cannot compute simulation time as fast as its pacing requires, it slows playback. It MUST NOT skip, coarsen or reorder semantic steps to keep up.

---

## 3. Runs and run configuration

- **Restates:** R-21, R-27.
- **Decisions:** D-012, D-015.

- **RC-3.1** A **run** is one execution of a model. It is defined by:
  - the model (its IR, identified by a content hash);
  - the **run configuration**: parameter overrides, start instant `t0`, end condition, solver and tolerance settings, event settings (section 16), random seed, failure policies and determinism mode;
  - the **input and intervention log**, which grows while an interactive run is live.
- **RC-3.2** A run's **end condition** is one of: an end instant `t_end`; the occurrence of a named event; no end (interactive runs, ended by the user). A run also ends when a failure policy says `stop` (section 10).
- **RC-3.3** A run has a status: `ready`, `running`, `paused`, `completed` (end condition reached), `stopped` (stopped by a policy, with the diagnostic that caused it), or `failed` (runtime environment failure).
- **RC-3.4** Two runs with the same model, run configuration and log are the same run for the purposes of replay (section 14).

---

## 4. Superdense time and the trajectory

- **Restates:** R-29.
- **Decisions:** D-004.
- **Prior art:** follows superdense time as used in Ptolemy II (Lee and Zheng) and in the hybrid-systems semantics of Maler, Manna and Pnueli: an instant is a pair `(t, n)` of a real time and a microstep index.

- **RC-4.1** A **superdense instant** is a pair `(t, n)` with `t` a simulation instant and `n` a natural number. Instants are ordered lexicographically.
- **RC-4.2** The **trajectory** of a run is its sequence of committed states:
  - between two consecutive event times `t_a < t_b`, one continuous segment: a state for every `t` in `(t_a, t_b)`, with discrete state constant;
  - at each event time `t`, a finite sequence of states `(t, 0), (t, 1), ..., (t, k)`. `(t, 0)` is the state reached by continuous evolution (the left limit); each later microstep is the result of one committed transition.
- **RC-4.3** "The state at time `t`" without a microstep means the **final** state `(t, k)`. Observers that need intermediate microsteps (the state just before a bounce) read them explicitly (section 15).
- **RC-4.4** At a time `t` where no event occurs, `k = 0` and the state is the continuous value.

---

## 5. Initialization

- **Restates:** R-25.
- **Decisions:** D-008, D-020, D-092.
- **Prior art:** the solve (RC-5.6) follows damped Newton methods with finite-difference Jacobians (Dennis and Schnabel; MINPACK's `hybrd` scales unknowns in the same way), and SPICE's convergence test on the Newton correction as well as on the residual.

- **RC-5.1** Initialization evaluates, in dependency order (MK-13.5, MK-13.6): parameter values (defaults with run-configuration overrides applied), then the initial definitions of all stored bindings. The result is the state at `(t0, 0)`.
- **RC-5.2** Constraints are checked on the initial state (MK-12.3). A violated constraint with policy `reject` or `stop` makes initialization fail: the run does not start, and the diagnostic names the constraint and the values involved. A violated `report` constraint produces a diagnostic and the run starts.
- **RC-5.3** Events with trigger `on start` are due at `(t0, 0)`. Their transition, if any, commits `(t0, 1)` (section 8). Time events `at(t0)` and `every(Δ, from t0)` are due at the same microstep.
- **RC-5.4** The sign reference of every crossing guard (section 7) is taken from the final state at `t0`.

**Solved equations (D-092).** The solve below is part of every evaluation of a model's values, at `t0` and at every later instant; it is specified here because initialization is its first use.

- **RC-5.5** Initialization evaluates the solve (RC-5.6) at its place in the dependency order of initial definitions (MK-13.5, MK-11.8): after the values its equations and guesses read, before the initial definitions that read unknowns. If the solve fails at `t0` (RC-5.8), the run does not start, and the diagnostic names the unknowns.
- **RC-5.6** **The solve.** Whenever the runtime evaluates a model's values (initialization, every evaluation of flows during a step, event guards, transitions and observations), it computes the unknowns by Newton's method on the residuals `lhs - rhs` of the solved equations' components (MK-11.7):
  - It starts at the guesses, evaluated at that instant, every time. It never starts from an earlier solve, so the unknowns at an instant depend only on the other values at that instant, a run is a function of its configuration (D-015), and an observation between steps gives what a step would.
  - The **scale** `s_j` of unknown `j` is the magnitude of its guess, or 1 in coherent SI units when the guess is 0.
  - The Jacobian is estimated by forward differences, changing unknown `j` by `1e-7 (|u_j| + s_j)`.
  - The linear system is solved by Gaussian elimination with partial pivoting after each row, then each column, is scaled to a largest entry of 1, so that the units of equations and unknowns play no part. A pivot below `1e-13` in the scaled system means the equations are dependent at that instant.
  - The size of the residuals is their Euclidean norm, computed without overflow. A full Newton step is taken if it lowers that size; if it does not, the step is halved up to ten times, and a step below a thousandth of the full step is taken in any case. If the full step lowers the size, steps of 2, 4, 8 ... times the full step (up to `2^30`) are tried and the last that still lowers it is taken: on an exponential guessed from above, a full step moves only a fixed amount (about `Vt` for a diode).
  - At most 60 iterations.
- **RC-5.6a** **Size of a residual** (D-093). A residual is judged against the size of the terms its two sides are computed from, not of their values: sums add their terms' sizes, products multiply them, a quotient divides its numerator's by the divisor, a dot product sums its components' products, and other forms have the size of their value. `V0 - Vd` with `Vd` near `V0` has the size `|V0| + |Vd|`; a constraint's second rate, a sum that cancels to 0 at the solution, is judged against its terms.
- **RC-5.7** **Convergence.** The solve ends with the unknowns at their current values when, after an iteration:
  - every component's residual is 0 or at most `1e-12` of the size of its terms (RC-5.6a); or
  - the full Newton correction of every unknown is at most `1e-12 |u_j| + 1e-15 s_j` and the residuals are finite: the corrected values are taken. A residual is computed no more precisely than its terms (in `V0 - Vd == I * R` with `Vd` near `V0`, the rounding of the left side can exceed both sides); the correction measures the distance to the solution instead.
  - When the iterations run out, a step cannot lower the residuals, or a step changes no unknown by more than `1e-14 (|u_j| + s_j)`, the values reached are taken if every residual is at most `1e-9` of the size of its terms.
- **RC-5.8** **Failure.** Otherwise every unknown is `invalid` (MK-11.8), with one of three causes, each naming the unknowns: the equations cannot be evaluated at the guesses (an overflow or an invalid value there); the equations are dependent at that instant; no solution was found from the guesses. During a run, an invalid value in a flow rejects the step (RC-6.8): an adaptive solver retries with smaller steps, and a solve that keeps failing is a `model` failure (section 10).
- **RC-5.9** The tolerances, limits and step rules of RC-5.6, RC-5.7 and RC-5.10 are fixed in v0; they are not part of the run configuration.
- **RC-5.10** **Projection onto enforced constraints** (MK-12.10, D-093). The levels of the enforced constraints (MK-12.7) are taken in order, level `k` of every constraint together. For each, the components the level moves (MK-12.8) are corrected by Gauss-Newton steps of minimum norm, each component measured against the size of its binding's value (1 in coherent SI units for a zero value): with `J` the Jacobian of the level's residuals by central differences and `S` the diagonal of those sizes, the step is `-S (J S)^T ((J S)(J S)^T)^-1 r`. A level has converged when every residual is within `1e-15` of the size of its terms (RC-5.6a), or when a step changes no component by more than `1e-15` of its size; after 30 steps it is accepted within `1e-9` of the size of its terms, and fails otherwise. Projection happens at initialization (after the initial definitions, followed by the derived values and the solve again), at the end of every accepted step, before event detection's committed state is stored and before the next step starts, and in every transition before constraints are checked (RC-10). Event location uses the step's dense output, which is not projected: between two committed states the constraint holds to the solver's accuracy.

---

## 6. Continuous evolution

- **Restates:** R-18, R-27, R-31.
- **Decisions:** D-004, D-015.
- **Prior art:** follows standard ODE practice: an explicit fixed-step Runge-Kutta method (RK4) and an adaptive embedded pair with dense output (Dormand and Prince 5(4), as in MATLAB `ode45` and SciPy `RK45`). Follows Hairer, Nørsett and Wanner on dense output for event location.

### 6.1 What is approximated

- **RC-6.1** Between event times the runtime approximates the solution defined by the model kernel (MK section 14.5): every continuous state integrates its combined flow, derived bindings hold their definitions, discrete state is constant.
- **RC-6.2** Each **accepted step** `[t_i, t_{i+1}]` produces the continuous state at `t_{i+1}` and a **dense output**: an interpolant over the step, of stated order, used for event location (section 7) and for observation between steps (section 15).
- **RC-6.2a** (D-091) A run whose continuous state has more than 4096 components (fields) locates events on each step's dense output, and then keeps the step as the straight line between its two states, each state shared with the neighbouring step. Observations between steps of such a run are linear interpolations; at step ends they are exact.

### 6.2 Solvers

- **RC-6.3** A runtime MUST provide at least:
  - `rk4`: classical fourth-order Runge-Kutta with a fixed step `h`, and cubic Hermite dense output;
  - `dopri5`: adaptive Dormand-Prince 5(4) with relative and absolute tolerances, and its fourth-order dense output.
- **RC-6.4** The default solver is `dopri5` with relative tolerance `1e-6` and absolute tolerance `1e-9` (in coherent SI units, per state component). A maximum step `h_max` MAY be set; its default is unbounded.
- **RC-6.5** Additional solvers (symplectic, implicit, stiff) are components behind the same contract (RC-1.3). A solver declares its order, whether it is adaptive, and the order of its dense output. A structure-preserving solver that needs a particular model structure (for example position and velocity pairs) MUST reject models that lack it with a diagnostic, rather than run them approximately.
- **RC-6.6** The step sequence depends only on the model, the state, the run configuration and the logged instants of interventions and inputs. It MUST NOT depend on frame rate, observation requests, wall-clock time or thread scheduling (R-31).
- **RC-6.7** A step never crosses a time event, a logged intervention instant or a logged input instant: the step is shortened to land on it exactly.

### 6.3 Step failure

- **RC-6.8** If evaluating a flow during a step yields a status instead of a value (MK section 5), the step is rejected. An adaptive solver retries with a smaller step, down to its minimum step. If the status persists, or the solver cannot meet its tolerance at its minimum step, the result is a failure (section 10) with the status's provenance or the solver's report.

---

## 7. Event detection and location

- **Restates:** R-28, R-60.
- **Decisions:** D-004.
- **Prior art:** follows zero-crossing detection with root finding on dense output (SUNDIALS `CVodeRootInit`, FMI event indicators, Modelica tools). Follows the Illinois variant of regula falsi and Brent's method as standard bracketing root finders.

### 7.1 Detection

- **RC-7.1** Each crossing trigger (`rising(g)`, `falling(g)`, `crossing(g)`) has a **guard function** `g`, evaluated on the state and on the dense output.
- **RC-7.2** Each guard keeps a **sign reference**: the sign of `g` at the last point where it was non-zero. A zero value does not change the reference.
- **RC-7.2a** (D-056) After a transition, a crossing guard within the event time tolerance of zero takes as its reference the sign it is heading to: when the guard changes sign within `4 ε_t` under the flows of the committed state, its reference is that new sign. A located crossing leaves its guard on the far side by at most about `|ġ| ε_t` (RC-7.6); a handler that reverses the motion (a bounce) would otherwise leave a reference that makes the next crossing, if it falls inside one step, two sign changes that RC-7.4 allows to be missed. With this rule, bounces on a floor at any height accumulate until the Zeno policy applies.
- **RC-7.3** A crossing is **detected** in an accepted step when the sign of `g` at the step's end, or at any interior point the runtime samples, is non-zero and opposite to the reference, in the trigger's direction: from `+` to `-` for `falling`, from `-` to `+` for `rising`, either for `crossing`. A guard that reaches zero and returns to its previous sign is not a crossing (MK-15.4).
- **RC-7.4** Detection is guaranteed for a crossing when `g` has one sign change in the step. Two sign changes inside one step (a graze in and out) may be missed. A runtime SHOULD sample each guard at interior points of the dense output; the author controls the risk with `h_max` (section 16). This limitation is standard for zero-crossing methods and is stated so that reference programs can test within it.

### 7.2 Location

- **RC-7.5** For a detected crossing, the runtime finds a bracket `[t_lo, t_hi]` in the step with the guard on opposite sides of zero at the two ends, narrowed by a bracketing root finder on the dense output until `t_hi - t_lo <= ε_t` (event time tolerance, section 16).
- **RC-7.6** The event time is `t_hi`, the end of the bracket on the far side of the crossing. The state `(t_hi, 0)` is the dense-output value at `t_hi`, so the guard already has its post-crossing sign when the handler reads it. The accepted step is truncated at `t_hi`.
- **RC-7.7** When several guards are detected in one step, the event time is the earliest located `t_hi`. Every guard whose own located crossing lies within `ε_t` of it is due at the same instant. The runtime locates each of them before deciding.
- **RC-7.8** Crossings caused by a jump rather than by continuous evolution are handled in section 8 (RC-8.5).

### 7.3 Time events

- **RC-7.9** `at(τ)` and `every(Δ, from τ0)` are due exactly at their instants; no location is needed (RC-6.7). The `k`-th instant of `every` is computed as `τ0 + k·Δ`, not by accumulating `Δ`, so no drift builds up.

---

## 8. Event instants and transitions

- **Restates:** R-25, R-26, R-29.
- **Decisions:** D-004, D-005, D-020, D-024.
- **Prior art:** follows FMI and Modelica event iteration (repeat until no event is due, then resume integration) and superdense time for its bookkeeping. Departs from priority-based event ordering (SIMAN and other discrete-event simulators; 06 section 18): see D-024.

### 8.1 Event iteration

At an event time `t`, starting from `(t, 0)`:

1. Collect every event **due** at the current microstep `(t, n)`: located crossings (at `n = 0`), time events (at `n = 0`), `on start` (at `(t0, 0)`), `on(E)` for each `E` that occurred or was emitted at `n - 1` (D-041), crossings caused by jumps at `n - 1` (RC-8.5), and `on input(i)` for inputs changed at this instant.
2. Discard events whose enabling condition is false on the state at `(t, n)`.
3. If none remain, the iteration ends. The final state is `(t, n)`.
4. Otherwise, form **one transition** from the operations of all remaining handlers (MK-16.3). Every handler reads the state at `(t, n)` (MK-15.7).
5. Check the transition for conflicts (MK-16.4, MK-16.5). A conflict is a failure (section 10).
6. Compute the proposed state and check constraints (MK-12.3). A violated `reject` or `stop` constraint is a failure; a violated `report` constraint produces a diagnostic and does not block the commit.
7. Commit the proposed state as `(t, n + 1)`. Append every handled event to the event log. Continue at step 1 with `n + 1`.

- **RC-8.1** Every event due at a microstep is handled in the same transition. The runtime MUST NOT handle same-microstep events one after another, and results MUST NOT depend on any order among them (R-29).
- **RC-8.2** There are no event priorities in v0. An author who needs one event's effect to happen before another's expresses it as a cascade: the first handler emits an event whose handler performs the second step at the next microstep.
- **RC-8.3** The iteration is bounded by the **cascade limit** `N_micro` (default 100 microsteps per event time). Exceeding it is a failure with a diagnostic listing the events of the last microsteps (MK-15.10).
- **RC-8.4** After the iteration ends, the sign references of all guards are re-taken from the final state `(t, k)`. A reset that flips a velocity therefore does not itself count as a crossing of a guard that depends on it.
- **RC-8.5** If a committed transition changes a guard's value discontinuously (a reset jumps a state across zero), and the sign of `g` at `(t, n + 1)` is non-zero and opposite to its sign at `(t, n)` in the trigger's direction, that crossing event is due at `(t, n + 1)`. This follows Modelica, where a relation that changes value after `reinit` triggers further event iteration.

### 8.2 After the iteration

- **RC-8.6** Continuous evolution resumes from the final state `(t, k)`. The solver restarts its step control there; an adaptive solver's first step after an event is chosen from the post-event state alone, so the result does not depend on how the previous segment was stepped.
- **RC-8.7** Interventions delivered at time `t` are applied after the model's own iteration at `t` has ended (section 11). They may make further events due, in which case the iteration continues.

---

## 9. Zeno detection

- **Restates:** R-28.
- **Decisions:** D-004.
- **Prior art:** follows the hybrid-systems literature on Zeno behavior (Zhang, Johansson, Lygeros and Sastry) and the practical approach of simulators that detect event accumulation and switch to a sliding or resting mode (for example the resting-contact thresholds of rigid-body engines).

- **RC-9.1** For each event with a Zeno policy (MK-15.12), the runtime monitors its occurrences. **Accumulation** is detected when two successive occurrences at different event times are closer than `ε_zeno`, or when the event occurs more than `N_zeno` times within any interval of length `w_zeno`.
- **RC-9.2** Defaults: `ε_zeno = 1e-6 · T_ref`, `N_zeno = 1000`, `w_zeno = T_ref`, where `T_ref` is the run's **reference time scale** (run configuration, default `1 s`). A model whose natural time scale is years sets `T_ref = 1 yr` once rather than retuning each threshold. An event MAY override any of the three (MK-15.13).
- **RC-9.3** When accumulation is detected at an occurrence, the runtime applies the event's policy **in place of its handler at that occurrence**: `stop` is a failure with a diagnostic; `settle { ... }` uses the given operations as the event's handler for this transition. The transition is otherwise normal (section 8).
- **RC-9.4** Cascade loops at one event time are not Zeno behavior; they are bounded by the cascade limit (RC-8.3).

---

## 10. Failures and diagnostics

- **Restates:** R-16, R-25, R-26, R-54.
- **Decisions:** D-005, D-020.
- **Prior art:** follows database transactions (an aborted transaction leaves committed state unchanged). Follows the error categories of 06 section 56, which the audit judged sound.

### 10.1 Categories

| Category | Cause | Examples |
|---|---|---|
| model | an expression yields `invalid` where a value is required | division by zero in a flow, `sqrt` of a negative |
| constraint | a `reject` or `stop` constraint is violated | restitution set above 1 |
| conflict | a transition has conflicting operations | two events set the same velocity at one instant |
| cascade | the cascade limit is exceeded | two events emitting each other |
| Zeno | accumulation under a `stop` policy | a bouncing ball declared with `stop` |
| computational | the solver fails | tolerance unreachable at minimum step |
| environment | the machine fails | out of memory, device lost |

- **RC-10.1** A failure never commits partial state (R-25). The last committed state stands, and the diagnostic records the proposed transition, the category, the cause and its provenance (R-54).
- **RC-10.2** The runtime never resolves a failure by guessing: it never picks a winner among conflicting operations, replaces an invalid value with a number, or drops an operation (R-16, R-26).
- **RC-10.3** What happens next is the run's **failure policy** (per category, in the run configuration):
  - `stop`: the run ends with status `stopped`.
  - `pause`: the run pauses at the last committed state with the diagnostic shown, so a learner or author can inspect it. It can be resumed only after an intervention changes the state, or stopped.
- **RC-10.4** Defaults: `stop` for headless runs (tests, export); `pause` for interactive runs. `environment` failures always end the run with status `failed`.
- **RC-10.3a** In v0 policies are not chosen per category: headless runs stop, interactive runs pause. A paused session reports its failure (message, category, instant) to its host and is resumed by an intervention at the paused instant. A step whose end breaks a `reject` or `stop` constraint is cut at the last instant found valid, by bisection to the time tolerance; the failure is reported at the first instant found broken (D-071).
- **RC-10.5** A rejected **intervention** is not a run failure. It is reported to its source (the presentation kernel), and the run continues from the unchanged state.

---

## 11. Interventions and inputs

- **Restates:** R-33, R-40, R-41, SEM-07 (R-62).
- **Decisions:** D-009, D-015, D-023.
- **Prior art:** follows FMI, where tunable parameters and inputs may change only at event instants and the environment then runs event iteration. Follows record-and-replay debugging (for example rr), where nondeterministic inputs are logged so execution can be reproduced.

### 11.1 Interventions

- **RC-11.1** An intervention (MK-17.2) takes effect at a simulation instant `τ`, which is recorded in the log with the intervention. Its effect is exactly that of a time event at `τ` whose handler is the intervention's operations, applied after the model's own event iteration at `τ` (RC-8.7).
- **RC-11.2** The runtime chooses `τ`. In an interactive run it is normally the simulation time on display when the learner acts. If the runtime has already computed beyond `τ`, it restores the latest snapshot at or before `τ` and recomputes to `τ` (section 14), discarding the states computed beyond it. The trajectory after `τ` is recomputed from the new state.
- **RC-11.3** An intervention is one transition: its operations commit together or not at all, and are validated against constraints (MK-16.3). Interventions arriving for the same `τ` from different sources are applied as separate transitions in the order they are logged.
- **RC-11.4** Only intervenable targets are accepted (D-023, MK-6.10). An intervention on anything else is rejected with a diagnostic and never reaches the model.
- **RC-11.4a** (D-027) A request for an `on request` event (MK-17.2a) is an intervention: it takes effect at its logged instant, after the model's own event iteration there. The requested event is due at the intervention's microstep and is handled with any other operations of that intervention in one transition.
- **RC-11.5** Direct manipulation (dragging) produces a stream of proposed values. Only interventions that are committed are logged; the stream's intermediate proposals belong to the presentation kernel (R-41).

### 11.2 Inputs

- **RC-11.6** An input binding holds the last value supplied, or `unavailable` or `pending` (MK-17.3). In v0 an input is piecewise constant: it changes only at logged instants, and each change is an event instant where `on input(i)` triggers are due.
- **RC-11.6a** An input starts at the value the run configuration supplies, else at its declared default (MK-6.7a). The starting value is not a change: `on input(i)` is due only for values supplied at later instants. Every value supplied is a change, even one equal to the current value. A value is evaluated against the input's type, reading only constants; a value of another type, or for a binding that is not an input, is rejected and logged as an intervention is (D-051).
- **RC-11.6b** A requested event obeys its enabling condition (MK-15.5), and a request supplies the payload the event declares (MK-15.1a); a request without it, or with one the event does not declare, is rejected. A request whose event is not enabled is rejected with the reason: the member it names is not alive, or has no member of that number (MK-15.1c), or the condition does not hold (D-059).
- **RC-11.7** Every input change is logged with its simulation instant and value, so that a replay reproduces the run without the external source.

---

## 12. Execution control

- **Restates:** R-21, R-32, R-33.
- **Decisions:** D-009, D-015.
- **Prior art:** follows media players (play, pause, seek) and reversible debuggers (checkpoint and re-execute to go backward).

Execution control acts on runs, not on the model (MK-17.4). It is how the explanation timeline (D-009) and the player's controls direct a simulation.

| Control | Effect |
|---|---|
| `play`, `pause` | start or stop advancing simulation time. No effect on the trajectory. |
| `step` | advance to the next event time, or by a given simulation duration. |
| `seek(τ)` | make `τ` the current instant. Forward: compute to `τ`. Backward: restore the latest snapshot at or before `τ` and recompute to `τ`. |
| `reset` | start a new run with the same model and configuration and an empty log, at `t0`. |
| `branch(τ)` | create a new run that shares this run's trajectory up to `τ` and has its own configuration changes and log after it. |
| `set_config` | change run configuration (solver, tolerances); only by `reset` or `branch`, never within a run. |

- **RC-12.1** `seek` to an event time lands on its final microstep `(τ, k)` unless a microstep is given.
- **RC-12.2** Seeking backward and returning MUST reproduce the same trajectory (section 14). Execution control never changes results.
- **RC-12.3** A branch is a separate run: its states, log and diagnostics are its own. Its shared prefix is identical to the parent's by construction.
- **RC-12.4** A static model (MK-14.15) has no simulation clock. Its runs have a single instant; interventions still apply to it, and `play`, `step` and `seek` do nothing.

---

## 13. Randomness

- **Restates:** R-10, R-30, R-34.
- **Decisions:** D-013, D-015.
- **Prior art:** follows counter-based random number generators (Salmon and others, "Parallel random numbers: as easy as 1, 2, 3", the Random123 library), where each value is a pure function of a key and a counter, so draws do not depend on execution order.

The stochastic modes (D-013) are specified by D-083: random events (MK-15.15) and draws (MK-10.4). RC-13.1 to RC-13.4 make them reproducible; RC-13.5 to RC-13.8 fix the streams, the generator, the distributions and the clocks of random events.

- **RC-13.1** A run has one **seed** in its run configuration.
- **RC-13.2** Randomness is drawn from **streams**. A stream is identified by the seed and the identity of the element that consumes it (a stochastic event or process, and the object it acts on). Draws from a stream are numbered by a counter. A draw is a pure function of seed, stream identity and counter.
- **RC-13.3** There is no global generator whose state depends on the order in which elements draw. Adding, removing or reordering one element changes only its own stream.
- **RC-13.4** The generator algorithm is part of the runtime build and is named in the reproducibility record (section 14).
- **RC-13.5** (D-083) The stream of a draw is identified by the event that draws (its identity in the elaborated model) and the draw site's number in it: sites are numbered from 0 in order through the event's enabling condition, handler and Zeno settle operations. Its counter is the event's **due count**: how many times the event was due before, counting each microstep at which it was due or requested once, whether or not its condition held. A draw needing several uniform numbers takes them in order from its stream.
- **RC-13.5a** (D-087) The noise of an instrument's reading (PK-3.7) comes from the generator of RC-13.6 with the run's seed, the stream `fnv1a64` of the instrument's identity and the source's IR together, site 0 and the counter the bits of the instant `t` as a 64-bit float; one standard normal number per component, in order. A reading thus depends on the seed, the instrument, the source and the instant only: an observation and a representation that read one source through one instrument at one instant show the same reading, and readings never change the run (PK-3.2).
- **RC-13.6** The generator is Philox4x32-10 (Salmon and others 2011). The key is `splitmix64(seed) XOR fnv1a64(event identity)`, split into two 32-bit words (low first); the counter is `(k low, k high, site, j)` for due count `k` and block `j`. Uniform number `i` of a draw is taken from block `i / 2`, words `2 (i mod 2)` and `2 (i mod 2) + 1` as the high and low halves of 64 bits, whose top 53 bits divided by `2^53` give a number in `[0, 1)`.
- **RC-13.7** Distributions are computed from uniform numbers `u`: uniform `a + (b - a) u`; normal by Box-Muller, `m + s sqrt(-2 ln(1 - u1)) cos(2π u2)` per component; exponential `-ln(1 - u) / r`; Bernoulli `u < p`; integer `a + floor(u (b - a + 1))`; choice the argument numbered `floor(u k)`; Poisson by multiplication of uniforms below a mean of 30 and by Hörmann's PTRS from 30; binomial by inversion when `n min(p, 1 - p) < 30` and by Hörmann's BTRS otherwise (with `n - X` for `p > 1/2`).
- **RC-13.8** A random event keeps a **clock** (the modified next reaction method, Anderson 2007): the integral `Λ` of its rate since the run's start and a target `P`, a sum of standard exponential numbers from the event's stream at site `2^32 - 1` (counter 0 for the first, `k` after the k-th occurrence). After initialization and after every transition (including actions) the clock adds the previous rate times the time elapsed to `Λ`, reads the new rate `λ`, and schedules the event as a time event at `t + (P - Λ) / λ` (never at the current instant; none when `λ = 0`). When it occurs, `Λ` becomes `P` and `P` grows by the next exponential number. Snapshots (RC-14.1) hold the clocks and the due counts.
- **RC-13.8a** (D-086) The clock of a rate that reads continuous state or time (MK-15.16) has no scheduled instant. Over every accepted step the runtime integrates the rate along the step's dense output, with three-point Gauss-Legendre panels halved until each half-pair agrees with its whole to `1e-10` (absolute and relative), and adds the integral to `Λ`. When the integral over a step reaches `P - Λ`, the occurrence is located by bisection to `ε_t` and the step is cut there, as for a crossing (RC-7.5, RC-7.6); events found within `ε_t` of it are due with it. A rate found negative inside a step is a model failure at the first instant located negative, to `ε_t`. Varying and constant clocks draw their targets from the same stream (RC-13.8), so an edit that makes a rate vary keeps the event's targets.
- **RC-13.9** (D-090) A model with noise in its flows (MK-14.16) is integrated by the **Euler-Maruyama** method with a fixed step `h`: the case's `h`, else `t_ref / 1000`; the configured solver is not used. A step from `t` of length `h` (shorter where it lands on a stop, RC-6.7) evaluates the flows once at `t` with each noise `i` equal to `N_i / sqrt(h)`, `N_i` a standard normal number from the generator of RC-13.6 with the run's seed, the stream `fnv1a64` of the flow's identity, `|noise|` and the occurrence's number in the flow, site 0 and the counter the bits of `t`; the step adds `h` times the result. Its dense output is the straight line between its two states (the Brownian bridge's mean), on which events are located (RC section 7) and random clocks integrate (RC-13.8a). The method has strong order 1/2 (order 1 for additive noise) and weak order 1: averages over runs converge with `h`. Runs are reproducible from snapshots like any other (RC-14).

---

## 14. Snapshots, replay and determinism

- **Restates:** R-20, R-21, R-30, R-34.
- **Decisions:** D-015, D-019.
- **Prior art:** follows checkpoint and re-execute debugging for backward navigation. Follows deterministic lockstep simulation in games, which achieves bit-identical replay on one platform by fixing evaluation order and avoiding nondeterministic sources.

### 14.1 Snapshots

- **RC-14.1** A **snapshot** captures a committed superdense instant: every stored binding's value, collection and relation membership, identity counters (MK-7.7), guard sign references, Zeno monitors, time-event schedules, random stream counters, and the computational state needed to continue exactly (for example an adaptive solver's proposed next step).
- **RC-14.2** Restoring a snapshot and continuing MUST produce the same trajectory as never having stopped, under the determinism level in force (RC-14.5).
- **RC-14.3** A runtime takes snapshots at least at every intervention instant and at run start. Other snapshot instants (for fast backward seek) are its choice and MUST NOT affect results (RC-6.6).
- **RC-14.3a** When a run's action log changes (an action added, removed or replaced), the new run MAY continue from a snapshot of the previous run instead of starting again, provided every computation before the snapshot is unaffected by the change: the snapshot lies before the first instant `t_d` at which the two logs, in the order runs apply them, differ, and no solver step before it depended on a stop at or after `t_d`. An adaptive step depends on the next stop when it lands on it, or when the stop limits its size (including the initial step after an event); a runtime records, with each snapshot, the latest stop any earlier step depended on. The continued run is bit-identical to the run computed from the start (RC-14.2).

### 14.2 Reproducibility record

- **RC-14.4** Every run can produce a **reproducibility record**: the model's IR hash, the runtime build identity and platform, the full run configuration (including seed and solver settings), and the input and intervention log. Replaying the record reproduces the run.

### 14.3 Determinism levels (D-015)

- **RC-14.5** Guarantees:
  - **Same build and platform:** replay is bit-identical: the same committed states, the same event log, the same diagnostics.
  - **Across platforms** (for example native and browser): continuous states agree within the run's declared tolerances. Discrete outcomes (whether and in what order events occur) are expected to agree, but may differ when a guard passes within about `ε_t` of zero or a constraint is within rounding of its boundary; the runtime reports the run as tolerance-equivalent, not identical.
  - **Deterministic math mode** (reserved): identical results on every platform, at a speed cost. Not built in the first slice. It will require correctly rounded elementary functions and no fused or reordered floating-point operations.
- **RC-14.6** To meet the same-platform guarantee a runtime MUST: use the fixed evaluation and combination orders of the model kernel (MK-13.6, MK-14.10, MK-8.3); not let results depend on hash-map iteration order, thread scheduling, wall-clock time or observation requests (RC-6.6); draw randomness only as in section 13.

---

## 15. Output to observers

- **Restates:** R-19, R-24, R-31.
- **Decisions:** D-011, D-018.

This is what the runtime offers the presentation kernel. Observation itself (sampling, measurement, data) is specified there.

- **RC-15.1** Observers can read:
  - the committed state at any event instant, including every microstep;
  - the state at any time between event instants, from the dense output (RC-6.2), with the accuracy of the solver's dense output;
  - the event log: every handled event with its superdense instant, payload, and whether a Zeno policy replaced its handler;
  - diagnostics and the run status.
- **RC-15.2** Reading never changes the trajectory: an observation request at a time between steps is answered from the dense output and does not add a step (RC-6.6).
- **RC-15.3** Observers never see proposed state (SEM-08); only committed states and the dense output of accepted steps.

---

## 16. Execution configuration and defaults

| Setting | Meaning | Default |
|---|---|---|
| `solver` | integration method | `dopri5` |
| `rtol`, `atol` | adaptive tolerances | `1e-6`, `1e-9` (coherent SI) |
| `h` | fixed step for fixed-step solvers | none: MUST be set when a fixed-step solver is chosen |
| `h_max` | maximum step | unbounded |
| `ε_t` | event time tolerance | `1e-10 · T_ref` |
| `T_ref` | reference time scale | `1 s` |
| `N_micro` | cascade limit per event time | `100` |
| `ε_zeno`, `N_zeno`, `w_zeno` | Zeno detection | `1e-6 · T_ref`, `1000`, `T_ref` |
| `seed` | random seed | `0` |
| failure policies | per category | `stop` headless, `pause` interactive |
| determinism mode | standard or deterministic math | standard |

- **RC-16.1** Every setting that can change results is part of the run configuration, recorded in the reproducibility record and visible to the author (D-010, MK-1.2). No setting is hidden inside a runtime.

---

## 17. Check against the first-slice reference programs

Each program (D-012) is checked for whether this contract determines its expected result. Numbers here are acceptance criteria to be confirmed when the reference programs are written.

| Program | What the contract must deliver | Covered by |
|---|---|---|
| Projectile, no drag | Landing time within `ε_t` of `2 v0.y / g`; range then within `v0.x · ε_t` plus solver error of `2 v0.x v0.y / g`. | RC-7.5, RC-7.6, RC-6.4 |
| Projectile, drag | Landing time and range within stated tolerance of a high-precision reference solution computed with tight tolerances. | RC-6.3, RC-6.4 |
| Bouncing ball | Bounce times match the geometric series within `ε_t` plus solver error per bounce; accumulation detected and `settle` applied before the limit time `t_∞`; ball then at rest. | RC-7.6, RC-8.4, RC-9 |
| Pendulum | Small-angle period within tolerance of `2π√(L/g)`; energy drift measured per solver (`rk4` with given `h`, `dopri5` with given tolerances) and within the bound stated in the program. | RC-6.3, RC-15.1 |
| Spring-mass | Period `2π√(m/k)`; energy conserved within tolerance. | RC-6.3 |
| Function plot | Static model: dragging the parameter commits interventions; each is validated (`reject` outside range) and logged. | RC-11, RC-12.4 |
| Vector addition | Static model; no runtime behavior beyond initialization. | RC-5 |
| Narrated lesson | The timeline waits on `landed` (event log), pauses, seeks back to launch and forward again, and sees the same trajectory both times. | RC-12, RC-14.2, RC-15.1 |

**Findings.**

- The bouncing ball after `settle`: with `resting = true` the flows are zero and `y = 0`, so `falling(y)` has no sign change and no further event occurs (RC-7.3, RC-8.4). Consistent with the model kernel check (MK 19.2).
- The projectile's landing: under RC-7.6 the committed landing state has `pos.y` slightly below zero (at most about `|vel.y| · ε_t`). Reference programs that compare the landing position must allow for this, or read the located time instead of the position.
- No gaps found in the runtime contract for the first slice. The one model-kernel gap (a check evaluated at an event instant, MK 19.1) is still a presentation-kernel item; RC-15.1 gives it what it needs (committed state at the event's instant).

---

## 18. New positions taken in this document

**Register entries:**

| Entry | Position | Sections |
|---|---|---|
| D-024 | No event priorities; same-instant events form one transition; ordering by cascades only (Accepted) | 8 |

**Elaborations (in this specification only):**

- Event time is the far end of the location bracket (RC-7.6).
- Sign references re-taken after event iteration; jumps across zero trigger crossings (RC-8.4, RC-8.5).
- Interventions apply after the model's own event iteration at that instant, as time events at a logged instant, with rollback when the runtime has computed ahead (RC-8.7, RC-11.1, RC-11.2).
- Required solvers `rk4` and `dopri5`; default `dopri5` at `1e-6` / `1e-9` (RC-6.3, RC-6.4).
- Step sequence independent of frame rate and observation (RC-6.6); players slow down rather than skip (RC-2.4).
- Zeno and cascade defaults scaled by a reference time scale `T_ref` (RC-9.2, section 16).
- Failure policies: `stop` headless, `pause` interactive (RC-10.4).
- The solve of unknowns (D-092): Newton from the guesses at every instant, never warm-started; equilibrated elimination; halving and doubling of steps; convergence by residual or by Newton correction; fixed tolerances (RC-5.5 to RC-5.9).
- Projection onto enforced constraints (D-093): Gauss-Newton of minimum scaled norm, level by level, after every step and transition (RC-5.10); residuals measured against the size of their terms (RC-5.6a).
- Counter-based random streams keyed by element identity (RC-13.2).
- Configuration changes only by `reset` or `branch` (section 12).

---

## 19. Deferred

| Item | Where it will be specified |
|---|---|
| Stochastic modes: per-step sampling and event-time sampling (D-013) | Done (D-083, RC-13.5 to RC-13.8); rates varying between events (D-086, RC-13.8a) |
| Algorithmic process execution (RUN-04) | Slice that needs it |
| Field state discretization and solvers (D-010) | Slice that needs it |
| Stiff and implicit solvers; multiple solvers in one model (06 sections 49-51) | When a reference program needs them |
| Parallel and GPU execution (06 sections 52-53) | Later; bound by RC-14.6 |
| Continuously varying inputs (interpolated rather than piecewise constant) | When the sensor cases are sliced |
| Deterministic math mode (D-015) | Later |
| Mapping of simulation to presentation time, sampling, measurement | `03-presentation-kernel.md` |
