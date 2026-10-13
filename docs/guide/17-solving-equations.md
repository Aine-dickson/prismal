# 17. Solving Equations

Until now every value was computed from others in one direction: a derived value from its definition, a rate from its flow. Many problems in science are not written that way. Kirchhoff's laws give a circuit's currents only together; an equilibrium is a condition the amounts must meet. Prismal solves such equations: declare the **unknowns**, write the equations that relate them, and the runtime finds their values at every instant.

> **In plain words.**
> - An **unknown** is a value you do not write a formula for: `unknown { I: Current }`. You state facts about it instead, as equations.
> - An **equation** that reads an unknown is **solved** for it. You need as many equations as unknowns, and each unknown must appear in them.
> - The runtime solves the equations at every instant, as a calculator's solver would, starting from a **guess** you may give with the unknown (`= 1 mA`). For equations of the first degree (most circuits) the guess does not matter; for others it picks which solution you get.
> - An equation may also give a **rate**: `L * der(I) == V0 - R * I` tells how the current changes, without solving for `der(I)` by hand.

## A voltage divider

**Goal.** Two resistors in series across a battery: the learner moves the resistances and sees the current and the voltage across the second resistor follow, from Ohm's law and Kirchhoff's voltage law.
**How it is built.** The current `I` and the voltage `V2` are `unknown`s. Two equations relate them: around the loop the battery's voltage is shared between the resistors (`loop`), and Ohm's law holds for the second resistor (`load`). Neither is a formula for `I` or `V2`; together they determine both. Derived values may read unknowns, as `P2` does.

```text
model Divider {
  param {
    V0: Voltage    = 9 V      in [0 V, 12 V]
    R1: Resistance = 100 Ω    in [10 Ω, 1 kΩ]
    R2: Resistance = 200 Ω    in [10 Ω, 1 kΩ]
  }
  unknown {
    I:  Current
    V2: Voltage
  }
  derived { P2: Power = V2 * I }
  equation loop: V0 == I * R1 + V2
  equation load: V2 == I * R2
}

presentation DividerLab for Divider {
  panel controls {
    slider(V0, range: [0 V, 12 V])
    slider(R1, range: [10 Ω, 1000 Ω])
    slider(R2, range: [10 Ω, 1000 Ω])
  }
  panel results {
    label(I)
    label(V2)
    label(P2)
    equation(loop)
    equation(load)
  }
  observe { current = I at t0 }
}
```

```cases
run shares of Divider with DividerLab {
  expect { current == 0.03 A within 1e-9 A }
}
```

- The equations are written as a physics book writes them. `V0 == I * R1 + V2` is not an assignment: no side has to be a single name.
- `equation(loop)` in a panel shows the equation with its current values, as for any equation.
- `Voltage`, `Resistance`, `Capacitance`, `Inductance` and `Charge` are named types; `V`, `Ω` (or `ohm`), `kΩ`, `F`, `μF` (or `uF`), `H`, `mA` are units.

## A charging capacitor

**Goal.** A capacitor charged through a resistor: the learner watches its voltage rise towards the battery's and the current fall away, both exponentially.
**How it is built.** The capacitor's voltage `Vc` is `state`, with a flow: the current charges it. The current is an unknown, given by the loop equation at every instant, so it changes as `Vc` does. An event fires when the capacitor is half charged.

```text
model Charging {
  param {
    V0: Voltage     = 6 V
    R:  Resistance  = 1 kΩ
    C:  Capacitance = 1000 μF
  }
  state { Vc: Voltage = 0 V }
  unknown { I: Current }
  flow der(Vc) = I / C
  equation kvl: V0 == I * R + Vc
  event half on rising(Vc - V0 / 2)
}

presentation ChargingLab for Charging {
  until t0 + 6 s
  view voltage: plot(x: [0 s, 6 s], y: [0 V, 7 V]) {
    series_plot(Vc every 0.05 s)
  }
  view current: plot(x: [0 s, 6 s], y: [0 A, 0.007 A], y_unit: mA) {
    series_plot(I every 0.05 s)
  }
  observe { at_half = elapsed on half }
}
```

```cases
run time_constant of Charging with ChargingLab {
  until t0 + 2 s
  expect { at_half[1] == 0.6931 s within 0.001 s }
}
```

- The time constant is `R C = 1 s`; the capacitor is half charged after `R C ln 2`, 0.693 s.
- A flow may read an unknown (`I / C`), and an unknown may depend on state (`Vc` in the loop equation): the runtime solves the equations whenever it needs the flows.

## Equations that give rates

**Goal.** A coil switched on to a battery: the current cannot jump, it grows as the coil's back-voltage allows. And a mass on a spring, written as Newton's second law rather than as a formula for the acceleration.
**How it is built.** The current `I` is state with no flow. The equation `faraday` reads `der(I)`: since no flow gives `I` its rate, the equation is solved for it. The same holds for `newton`: `v` has no flow, so `m * der(v) == -k * x` gives its rate.

```text
model Coil {
  param {
    V0: Voltage    = 12 V
    R:  Resistance = 6 Ω
    L:  Inductance = 3 H
  }
  state { I: Current = 0 A }
  equation faraday: L * der(I) == V0 - R * I
}

model Spring {
  param {
    m: Mass = 2 kg
    k: Quantity<M/T^2> = 8 N/m
  }
  state {
    x: Length = 0.1 m
    v: Velocity = 0 m/s
  }
  flow der(x) = v
  equation newton: m * der(v) == -k * x
}

presentation CoilLab for Coil {
  until t0 + 3 s
  view current: plot(x: [0 s, 3 s], y: [0 A, 2.5 A]) {
    series_plot(I every 0.02 s)
  }
  observe { after_one = I at t0 + 1 s }
}
```

```cases
run rises of Coil with CoilLab {
  until t0 + 1 s
  expect { after_one == 1.7293 A within 0.0001 A }
}
```

- The current rises as `V0 / R (1 - e^(-R t / L))`: after 1 s, `2 (1 - e^-2)` A.
- A state with a flow keeps it: an equation that reads `der(x)` of such a state is not solved for it (MK-E35), it can only be checked (`checked within ...`).

## Equations of higher degree

**Goal.** A diode in series with a resistor: the current through a diode grows exponentially with its voltage, so no formula gives the circuit's current directly.
**How it is built.** Two unknowns and two equations, one of them exponential (Shockley's law). The guesses, `1 mA` and `0.6 V`, start the solver near the answer.

```text
model DiodeCircuit {
  param {
    V0: Voltage = 5 V          in [0 V, 10 V]
    R:  Resistance = 1 kΩ
    Is: Current = 1e-12 A      // the diode's saturation current
    Vt: Voltage = 25.85 mV     // the thermal voltage at room temperature
  }
  unknown {
    I:  Current = 1 mA
    Vd: Voltage = 0.6 V
  }
  equation resistor: V0 - Vd == I * R
  equation shockley: I == Is * (exp(Vd / Vt) - 1)
}

presentation DiodeLab for DiodeCircuit {
  panel controls { slider(V0, range: [0 V, 10 V]) }
  panel results {
    label(I)
    label(Vd)
  }
  observe { drop = Vd at t0 }
}
```

```cases
run conducts of DiodeCircuit with DiodeLab {
  expect { drop in [0.5 V, 0.7 V] }
}
```

- Whatever the battery, the diode takes about 0.6 V and the resistor the rest: the learner sees `Vd` hardly move while `I` follows `V0`.
- Equations of the first degree are solved exactly whatever the guess. Others are solved by improving the guess step by step (Newton's method), and an equation with several solutions gives the one the guess leads to: a guess is part of the problem's statement, as it is on paper.

## How equations are solved

- Each instant the runtime needs a value, it solves all the unknowns together, by Newton's method from the guesses (0 where none is given), with the equations' derivatives estimated by small changes of each unknown. It starts from the guesses every time, so a run is the same however often it is computed.
- The equations solved are those that read an unknown, directly or through derived values, and are not checks. There must be as many of them as unknowns (a vector equation counts once per axis), and each unknown must be determined by an equation of its own (MK-E34): `x + y == 1` and `2 x + 2 y == 2` are two equations but say one thing, and the run cannot start ("dependent").
- If no solution is found from the guesses, the value is not available and the run stops with a message naming the unknowns; better guesses usually help.

## Exercises

1. Add a third resistor `R3` in parallel with `R2` in the divider. Which unknowns and equations do you need? (Hint: one more current, and the current law at the junction.)
2. In the charging capacitor, make the battery switch off at 3 s (an event that sets a discrete `on` to false, and `V0 * (if on then 1 else 0)` in the loop equation). What does the current do?
3. Reverse the battery of the diode circuit: let `V0` and its slider run from -10 V to 10 V. What current flows when `V0` is negative, and why does `Vd` now follow `V0` instead of staying near 0.6 V? (Hint: `exp(Vd / Vt)` for a negative `Vd`, and `Is`.)
