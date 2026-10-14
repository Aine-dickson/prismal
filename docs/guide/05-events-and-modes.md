# 5. Events and Modes

Flows describe smooth change. **Events** describe sudden change: a ball bounces, a pump switches off, a counter ticks. An event has a **trigger** (when it happens) and a **handler** (what it does). **Discrete state** holds values that change only at events, such as a mode (`pumping`) or a count.

> **In plain words.**
> - An **event** is a moment when something happens all at once: a switch flips, a ball hits the floor.
> - Its **trigger** says when: at a set time, or when a value **crosses** a threshold (goes from below it to above it, or back).
> - Its **handler** says what changes at that moment, as a list of **operations** such as `set pumping = false`.
> - A **mode** is a value that says which way the system currently behaves: the pump on or off, a ball flying or resting.

## A pumped tank

**Goal.** A tank that a pump keeps between two levels: the learner sees the level rise and fall in a zigzag and counts the times it was full.
**How it is built.** The level is `state` with a `flow`. Whether the pump runs is a `discrete` Boolean, used in the flow with `if pumping then ... else ...`. Two `event`s with crossing triggers (`rising(level - high)`, `falling(level - low)`) switch it with `set`. A third event, triggered by the first, counts. The presentation plots the level against time and shows the count.

A tank drains steadily. A pump fills it; a controller switches the pump off when the level reaches `high` and on again when it falls to `low`.

```text
model Tank {
  param {
    inflow: Quantity<L/T> = 0.02 m/s    // level rise from the pump
    drain:  Quantity<L/T> = 0.01 m/s    // level fall from the outlet
    high:   Length = 1 m
    low:    Length = 0.5 m
    h0:     Length = 0.2 m
  }
  state {
    level: Length = h0
  }
  discrete {
    pumping: Boolean = true
    alarms:  Real    = 0
  }

  flow {
    der(level) = (if pumping then inflow else 0) - drain
  }

  event full  on rising(level - high)  { set pumping = false }
  event empty on falling(level - low)  { set pumping = true }
  event alarm on full                  { set alarms = alarms + 1 }
}
```

- `discrete { pumping: Boolean = true }` is **discrete state**: constant between events, changed only by handlers. A Boolean discrete variable is a **mode**; flows can depend on it with `if ... then ... else`.
- `event full on rising(level - high) { ... }`: the trigger `rising(g)` happens when `g` crosses zero upward, here when `level` rises through `high`. `falling(g)` is a downward crossing, `crossing(g)` either way. The runtime locates the crossing instant precisely, not at the next step.
- The handler `{ set pumping = false }` is a list of **operations**. `set` replaces a value. All operations of a handler read the values from before the event, and commit together.
- `event alarm on full` is triggered by another event: it happens right after `full`, at the same time, in the next **microstep** (D-041). A microstep is a step in the order of things that happen at one instant: the clock does not move, but `alarm` still comes after `full`. A handler can also signal an event explicitly with `emit E`. Events that cause events form a cascade at one instant; the runtime limits its length.
- A continuous guard is always a crossing. `level >= high` would be a level condition, which cannot trigger an event by itself (D-004): a level condition may only enable a trigger, as in `event full on rising(level - high) if pumping { ... }`.

With the default values the level rises at `0.01 m/s` from `0.2 m`, reaches `1 m` at 80 s, falls at `0.01 m/s` to `0.5 m` at 130 s, rises again to `1 m` at 180 s, and so on.

```text
presentation TankLab for Tank {
  view levels: plot(x: [0 s, 400 s], y: [0 m, 1.2 m]) {
    series_plot(level every 1 s)
  }
  view tank: plot(x: [-1, 1], y: [0 m, 1.2 m]) {
    marker(at: (0, level)) as surface
  }
  panel controls {
    slider(inflow, range: [0.011 m/s, 0.05 m/s])
    slider(high, range: [0.6 m, 1.2 m])
    label(alarms)
  }
  observe {
    fulls   = elapsed on full
    empties = elapsed on empty
    log     = event_log over [t0, t0 + 200 s]
    count   = alarms at t0 + 200 s
  }
}
```

- `fulls = elapsed on full` records `elapsed` at each occurrence of `full`: a series. `fulls[1]` in a test is its first value.
- `event_log over [t0, t0 + 200 s]` is the list of event occurrences in that interval, in order.

```cases
run cycles of Tank with TankLab {
  until t0 + 200 s
  expect {
    fulls[1]   == 80 s  within 1e-9 s
    empties[1] == 130 s within 1e-9 s
    fulls[2]   == 180 s within 1e-9 s
    log   == [full, alarm, empty, full, alarm]
    count == 2 exactly
  }
}

run strong_pump of Tank with TankLab {
  param { inflow = 0.05 m/s }
  until t0 + 200 s
  expect {
    fulls[1]   == 20 s within 1e-9 s      // 0.8 m at 0.04 m/s
    empties[1] == 70 s within 1e-9 s      // 0.5 m at 0.01 m/s
  }
}
```

## Time events

Events can also happen at given times:

- `event e on at t0 + 5 s { ... }` happens once, at that instant.
- `event e on every 0.5 s { ... }` happens at `t0`, `t0 + 0.5 s`, `t0 + 1 s`, ... (each instant computed as `t0 + k Δ`, so no rounding accumulates). `on every 0.5 s from t0 + 1 s` starts later.
- `event e on start { ... }` happens once when the run starts.

Every trigger follows the word `on`: `on rising(g)`, `on every 0.5 s`, `on full`.

**Goal.** A metronome that counts its beats, and a test of when they come.
**How it is built.** A time event, `on every 0.5 s`, adds one to a `discrete` count; observations read the count at an instant and the time of every tick.

```text
model Metronome {
  param { period: Time = 0.5 s  where period > 0 s }
  state { phase: Real = 0 }
  discrete { beats: Real = 0 }
  flow { der(phase) = 1 / period }
  event tick on every 0.5 s { set beats = beats + 1 }
}

presentation MetronomeChecks for Metronome {
  observe {
    after2 = beats at t0 + 2.25 s
    ticks  = elapsed on tick
  }
}
```

```cases
run two_seconds of Metronome with MetronomeChecks {
  until t0 + 2.25 s
  expect {
    after2   == 5 exactly               // at 0, 0.5, 1, 1.5 and 2 s
    ticks[3] == 1 s within 1e-12 s
  }
}
```

## Repeating events and Zeno behavior

An event whose handler changes what its own trigger depends on can happen again and again. A bouncing ball whose bounces lose energy bounces infinitely often in a finite time (a **Zeno** behavior, after the old puzzle of a runner who must first cover half the distance, then half of the rest, and so on forever). Each bounce is shorter than the last, so infinitely many of them fit before the ball would come to rest, and a simulation that tried to compute each one would never get past that moment. Such an event must say what happens at the limit:

**Goal.** A ball that bounces lower each time and comes to rest, without the run stalling on infinitely many bounces.
**How it is built.** A `falling` crossing reverses the velocity and keeps a share `e` of it; the event's `zeno settle { ... }` says what happens when the bounces pile up: the ball rests.

```text
model Ball {
  param {
    g: Acceleration = 9.81 m/s^2
    e: Real = 0.8  in [0, 1)            // restitution: the share of speed a bounce keeps
  }
  state {
    y: Length   = 1 m
    v: Velocity = 0
  }
  discrete { resting: Boolean = false }
  flow {
    der(y) = v
    der(v) = if resting then 0 else -g
  }
  event bounce on falling(y) {
    set v = -e * v
  } zeno settle {
    set y = 0 m
    set v = 0 m/s
    set resting = true
  }
}

presentation BallChecks for Ball {
  observe {
    first = elapsed on bounce
    final = (y, v, resting) at t0 + 6 s
  }
}
```

```cases
run settles of Ball with BallChecks {
  until t0 + 6 s
  expect {
    first[1] == 0.451523640985731 s within 1e-8 s    // sqrt(2 * 1 m / g)
    final == (0 m, 0 m/s, true) exactly
  }
}
```

- `zeno settle { ... }` gives the operations applied when the bounces accumulate: the ball comes to rest. `zeno stop` ends the run with a diagnostic instead.
- The compiler decides which events can repeat this way. Leaving out the policy is an error:

```error
// error: MK-E16
model Ball {
  param { e: Real = 0.8 }
  state { y: Length = 1 m; v: Velocity = 0 }
  flow { der(y) = v; der(v) = -9.81 m/s^2 }
  event bounce on falling(y) { set v = -e * v }
}
```

The tank's events are not repeating in this sense: after `full` switches the pump off the level falls, away from `high`. The compiler follows the flows with `pumping` set to `false` to see this (D-038).

RP-03 in the reference programs is the complete bouncing ball, with its expected bounce times and the instant of the Zeno limit.

## Modes with more than two values

A Boolean mode has two values. When a system has more, declare an **enumeration**: a type whose values are a short list of names you choose, such as `rising`, `top`, `falling` (MK-2.2, D-049).

**Goal.** A ball thrown straight up whose stage (climbing, at the top, landed) is named, shown and used by the motion.
**How it is built.** An `enum` lists the stages; a `discrete` value of it is switched by events at the top and at the ground; `match` gives a value for each stage.

```text
model Toss {
  /// The stages of a ball thrown straight up.
  enum Phase { climbing, sinking, landed }
  const { g: Acceleration = 9.81 m/s^2 }
  param { v0: Velocity = 10 m/s }
  state {
    h: Length   = 0 m
    v: Velocity = v0
  }
  discrete { phase: Phase = climbing }
  derived {
    direction: Real = match phase { climbing => 1, sinking => -1, landed => 0 }
  }
  flow {
    der(h) = if phase != landed then v else 0 m/s
    der(v) = if phase != landed then -g else 0 m/s^2
  }
  event apex   on falling(v) if phase == climbing { set phase = sinking }
  event ground on falling(h) if phase == sinking  { set phase = landed; set v = 0 m/s }
}
```

- `enum Phase { climbing, sinking, landed }` declares the type; `phase: Phase = climbing` is discrete state of that type. A case is written by its name, or as `Phase.sinking` where that reads better.
- Cases are compared with `==` and `!=`; they have no order, so `phase < landed` is an error.
- `match phase { ... }` chooses a value by case. It must give every case exactly one arm (`MK-E17`), so adding a case to the enumeration later shows every place that has to decide what it means.
- Two enumerations are different types even if their cases have the same names.
- A label shows the case by name (`phase = sinking`); a formula of `direction` is typeset as one row per case.

```text
presentation TossLab for Toss {
  view height: plot(x: [0 s, 3 s], y: [0 m, 6 m]) {
    series_plot(h every 0.02 s)
  }
  panel status {
    label(phase)
    formula(direction)
  }
  observe {
    top_time  = elapsed on apex
    top       = h on apex
    landed_at = elapsed on ground
    final     = phase at t0 + 3 s
  }
}
```

```cases
run toss of Toss with TossLab {
  until t0 + 3 s
  expect {
    top_time[1]  == 1.01936799184506 s within 1e-6 s    // v0 / g
    top[1]       == 5.09683995922528 m within 1e-6 m    // v0^2 / (2 g)
    landed_at[1] == 2.03873598369011 s within 1e-6 s    // 2 v0 / g
    final == landed exactly
  }
}
```

## Several events adding to one value

**Goal.** Several events that happen at the same instant each add to a score, without one overwriting the other.
**How it is built.** The score is `discrete` state declared with `combine sum`; each event's handler uses `contribute total += ...` instead of `set`.

Two `set` operations on one value at the same instant are a conflict, even when they write the same value: the run cannot know which should win (MK-16.4). `contribute` says that the values are meant to be combined. The binding declares how, with `combine`: `sum` adds, `product` multiplies, `min` and `max` keep the smallest or largest, `any` and `all` combine Booleans. The contributions combine with the value before the instant, so `contribute total += 1` adds one, whatever else happens at that instant (MK-14.9, D-073).

```text
model Score {
  discrete {
    total: Real    = 0      combine sum
    best:  Length  = 0 m    combine max
    hit:   Boolean = false  combine any
  }
  event small on every 1 s { contribute total += 1;  contribute best += 2 m }
  event large on every 1 s { contribute total += 10; contribute best += 0.5 m; contribute hit += true }
}

presentation ScoreChecks for Score {
  observe {
    total2 = total at t0 + 2.5 s
    best2  = best  at t0 + 2.5 s
    hit2   = hit   at t0 + 2.5 s
  }
}
```

```cases
run three_rounds of Score with ScoreChecks {
  until t0 + 3 s
  expect {
    total2 == 33 exactly     // both events at 0 s, 1 s and 2 s: 3 times 11
    best2  == 2 m exactly    // the largest contribution
    hit2   == true
  }
}
```

- A `set` and a `contribute` on the same value at the same instant are still a conflict: in one handler the program is rejected (MK-E19); in two events at one instant, the run stops (or pauses in a lab) with a conflict.
- `combine` goes only on discrete state. Continuous state already sums its contributions through `der(x) += ...` (chapter 4).

A contribution to a value with no combination:

```error
// error: MK-E11
model Tally {
  discrete { n: Real = 0 }
  event tick on every 1 s { contribute n += 1 }
}
```

## Events that carry values, and values from outside

An event can carry a value, its **payload** (MK-15.1, D-050): a kick carries how hard it is, a click carries where it was. And a model can receive values from its environment while it runs: a sensor, a game controller, a host application. These are **inputs** (RC-11.6, D-051).

**Goal.** A cart moved by kicks of chosen strength and by a thrust from outside the model (a slider of a host application, a game controller).
**How it is built.** An event `on request(j: Momentum)` receives the kick's strength as its payload; `input { thrust: Force }` is read by the flow and changed by the run's `input` block or the host's `set_input`.

```text
model Cart {
  const { m: Mass = 2 kg }
  input { thrust: Force = 0 N }
  state {
    x: Length   = 0 m
    v: Velocity = 0 m/s
  }
  discrete {
    pushes: Real     = 0
    last:   Momentum = 0 kg*m/s
  }
  flow {
    der(x) = v
    der(v) = thrust / m
  }
  /// A kick of a given impulse.
  event kick on request(j: Momentum) if j > 0 kg*m/s { set v = v + j / m }
  event remember on kick(j: Momentum) { set last = j }
  event pushed on input(thrust) { set pushes = pushes + 1 }
}
```

- `on request(j: Momentum)` declares the payload: whoever requests `kick` (a timeline's `request kick(3 kg*m/s)`, a host) supplies an impulse, which the condition and handler read as `j`. A request without it is rejected.
- `on kick(j: Momentum)` receives the payload of the event it follows. A handler can also pass a value on explicitly: `emit tally(2 * j)` makes the events `on tally(k: ...)` due with `k = 2 j`.
- A payload may have any type a binding has: a quantity, a vector, a point, a Boolean, or an enumeration (`on request(k: Color)`, requested as `request paint(blue)`; a host names the case, `"blue"`).
- A payload is read only in its own event, and it always comes from somewhere: a request, or an event that carries a payload of the same type (`MK-E24` otherwise).
- `input { thrust: Force = 0 N }` is a value the model does not control: the environment supplies it, and it holds between changes. `0 N` is its default until then; an input without a default needs a starting value from the run.
- `on input(thrust)` happens whenever the environment supplies a new value.
- Inputs are not parameters: the learner cannot set them with a control, and a handler cannot set them.

A case supplies inputs in an `input` block: a value from the start, and values `at` later instants.

```text
presentation CartChecks for Cart {
  observe {
    x_end = x at t0 + 2 s
    v_end = v at t0 + 2 s
    count = pushes at t0 + 2 s
  }
}
```

```cases
run push of Cart with CartChecks {
  input {
    thrust = 4 N
    thrust = 0 N at t0 + 1 s
  }
  until t0 + 2 s
  expect {
    v_end == 2 m/s within 1e-9 m/s    // 4 N on 2 kg for 1 s
    x_end == 3 m within 1e-9 m        // 1 m while pushed, 2 m after
    count == 1 exactly                // the change at 1 s; the starting value is not a change
  }
}
```

A host supplies inputs through its interface (`set_input`, `docs/spec/05-host-interface.md`), at the instant it shows.

A learner requests an event with a payload by pressing a button that gives the value: the event's name with the value after it, as in a timeline's `request` (D-072).

```text
presentation CartLab for Cart {
  panel controls {
    button(kick(2 kg*m/s), label: "Small kick")
    button(kick(6 kg*m/s), label: "Big kick")
    label(v)
  }
}
```

A button for an event that takes a value must give one of the right type (`button(kick)` and `button(kick(2 m))` are rejected, PK-E05 and PK-E04), and a button for an event that takes none gives none.

A payload with no source:

```error
// error: MK-E24
model Wrong {
  state { v: Velocity = 0 m/s }
  event nudge on start { set v = 1 m/s }
  event echo on nudge(j: Momentum) { set v = 0 m/s }
}
```

`nudge` carries no payload, so `echo` has nothing to receive.

A `match` that forgets a case:

```error
// error: MK-E17
model Wrong {
  enum Phase { climbing, sinking, landed }
  discrete { phase: Phase = climbing }
  derived { direction: Real = match phase { climbing => 1, sinking => -1 } }
}
```

The compiler reports `match does not cover landed`.

## Exercises

Solutions to the exercises not solved here are in [chapter 11](11-solutions.md).

1. Add an overflow valve to the tank: an event `spill on rising(level - 1.1 m)` that sets the level back to `1.1 m`. When can it happen?
2. Give the tank a leak that starts at 150 s: a discrete `leaking: Boolean = false`, an event `at t0 + 150 s` that sets it, and a flow that drains twice as fast while it is set. Compute when the level next reaches `low`, and test it.
3. Write a model of a light that switches on for 2 s every 5 s, using two time events and a mode. (`on` is a reserved word, so the mode needs another name.)

<details>
<summary>A solution to exercise 3</summary>

```text
model Blinker {
  state { clock: Real = 0 }
  discrete { lit: Boolean = false }
  flow { der(clock) = 1 / 1 s }
  event light_on  on every 5 s                  { set lit = true }
  event light_off on every 5 s from t0 + 2 s    { set lit = false }
}

presentation BlinkerChecks for Blinker {
  observe {
    at1 = lit at t0 + 1 s
    at3 = lit at t0 + 3 s
    at6 = lit at t0 + 6 s
  }
}
```

```cases
run blinks of Blinker with BlinkerChecks {
  until t0 + 7 s
  expect {
    at1 == true exactly
    at3 == false exactly
    at6 == true exactly
  }
}
```

</details>
