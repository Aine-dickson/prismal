# 19. Shared Definitions and Modules

Chapter 9 wrote an object type inside the model that uses it. A kind of thing worth describing once (a body moved by forces, a spring, a formula for kinetic energy) is often needed by several models, and by other authors. This chapter writes definitions **outside** any model, so that every model of the file can use them, and puts them in a **module** that other files import with `use` (D-099).

> **In plain words.**
> - A **shared definition** is an `object`, `enum` or `fn` written at the top level of a file, not inside a model. Every model in the file can use it by name.
> - A **module** is a file of shared definitions with a name, such as `mechanics`. `use mechanics` at the top of another file makes its definitions visible there.
> - Shared definitions work in a model of any space: a `Point` in a shared object type is a point of whichever space the model using it is in, a plane or a room.
> - Prismal ships with modules of its own, the **first-party library**, written in Prismal like any author's module. `mechanics` is the first; `optics` (chapter 22) is another.

## Definitions for every model of a file

**Goal.** One description of a point mass, used by a model in a plane and by another in a three-dimensional room, with a shared formula for its energy.
**How it is built.** `object Body`, `fn kinetic` and `enum Heading` are written at the top level, after the spaces. Each model declares a `Body` as a part, as in chapter 9, and connects its `force` input. The plane's force has two components, the room's three: the object type does not say which space it lives in.

```text
space Plane = euclidean(2)
space Room = euclidean(3)

/// A point mass moved by the force on it.
object Body {
  param { m: Mass = 1 kg }
  input { force: Vector<Force> = 0 }
  state {
    pos: Point            = origin
    vel: Vector<Velocity> = 0
  }
  derived { energy: Energy = kinetic(m, |vel|) }
  flow {
    der(pos) = vel
    der(vel) = force / m
  }
}

fn kinetic(m: Mass, v: Velocity): Energy = m * v^2 / 2

enum Heading { up, down }

model Throw in Plane {
  parts { ball: Body { m = 2 kg; vel = (3 m/s, 4 m/s); force = (0 N, -2 kg * 9.81 m/s^2) } }
  derived { heading: Heading = if ball.vel.y > 0 m/s then up else down }
}

model Drop in Room {
  parts { ball: Body { force = (0 N, 0 N, -9.81 N) } }
}

presentation ThrowLab for Throw {
  observe {
    start_energy = ball.energy at t0
    half_way     = ball.pos.x at t0 + 0.5 s
  }
}

presentation DropLab for Drop {
  observe { fallen = ball.pos.z at t0 + 1 s }
}
```

```cases
run throw of Throw with ThrowLab {
  until t0 + 0.5 s
  expect {
    start_energy == 25 J within 1e-9 J          // 2 kg · (5 m/s)² / 2
    half_way == 1.5 m within 1e-9 m             // 3 m/s for 0.5 s
  }
}

run drop of Drop with DropLab {
  until t0 + 1 s
  expect { fallen == -4.905 m within 1e-9 m }   // -g t² / 2 with m = 1 kg
}
```

- A shared definition reads nothing of the models that use it, like an object type (chapter 9): what it needs comes in through its inputs, parameters and arguments.
- A model's own definition of the same name wins inside that model.
- Object types written outside models cannot contain relation types between a model's collections (chapter 9, Relations): a relation names the collections of one model, so it is written in that model.

## Importing a module

**Goal.** The same throw, without writing the body at all: it comes from the first-party `mechanics` module, with its formulas.
**How it is built.** `use mechanics` at the top of the file. `Body`, `kinetic_energy`, `spring_force` and `pendulum_period` are then visible by name. The module's `Body` also has `kinetic` and `momentum` values.

```text
use mechanics

space Plane = euclidean(2)

model Swing in Plane {
  param {
    L: Length       = 2 m
    g: Acceleration = 9.81 m/s^2
  }
  parts { ball: Body { m = 0.5 kg; vel = (2 m/s, 0 m/s); force = (0 N, -0.5 kg * g) } }
  derived { period: Time = pendulum_period(L, g) }
}

presentation SwingFacts for Swing {
  observe {
    T       = period at t0
    kinetic = ball.kinetic at t0
  }
}
```

```cases
run facts of Swing with SwingFacts {
  expect {
    T == 2.8371 s within 1e-4 s                 // 2 π sqrt(L / g)
    kinetic == 1 J within 1e-12 J               // 0.5 kg · (2 m/s)² / 2
  }
}
```

- `use mechanics.{Body, kinetic_energy}` imports only the names listed.
- Two modules that define one name cannot both give it: the second `use` is an error (SX-E09) that says which modules clash. Import the name from one of them, `use a.{X}`, and the others' other names, `use b.{Y, Z}`.
- A name the file defines itself and a module also defines is an error too: a name means one thing in a file.
- A module may `use` other modules, but not in a circle (SX-E08).

## Writing a module

- A module is a file of shared definitions. Its name is its path without the extension: the file `physics/waves.prismal` (or `physics/waves.md`, whose program is its code blocks) is the module `physics.waves`.
- The `prismal` command and the formatter look for modules in the folder of the file they read, then in the first-party library (the repository's `lib` folder). A host that embeds Prismal says where its modules are (chapter 12, Embedding).
- `use` imports only shared definitions. A module's spaces, models and presentations, if any, stay in the module: they are its own examples.
- A compiled document holds the definitions of every module it uses, so it runs without the module files: a lesson sent to a learner does not need the library that built it.

## Exercises

1. Move the `Ball` object type of chapter 9's row of balls out of its model, so that a second model can drop balls on the Moon with it. What has to become an input?
2. Write a module `snell` with a function `refraction_angle(n1: Real, n2: Real, θ: Angle): Angle` from Snell's law, and use it in a model of a light ray. (Hint: `asin` is not built in; `atan2` and `sqrt` are.) Compare it with the first-party `optics` module of chapter 22.
