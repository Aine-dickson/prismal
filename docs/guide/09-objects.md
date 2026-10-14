# 9. Systems of objects

Many systems are made of several things of one kind: balls in a box, planets around a star, masses on a chain of springs. This chapter declares a kind of thing once, as an **object type**, and puts objects of that type in a model: one at a time, or as a **collection** of members (D-055).

> **In plain words.**
> - An **object type** is a description of one kind of thing, written once: what a ball has (a position, a speed) and how it behaves. Each **object** made from it is one actual ball.
> - A **collection** is a numbered group of objects of one type, like a row of balls; each one is a **member**, named `row[1]`, `row[2]`, ...
> - An **aggregate** combines a value over all members: `sum`, `count`, `max` (the highest ball, the number at rest).
> - A **relation** connects members, like a spring between two balls or a bond between two atoms. It can have its own values (the spring's stiffness).

## Object types

An object type is declared inside a model with `object`, and has the body of a model: parameters, inputs, state, derived values, flows, events.

```prismal
object Ball {
  param { r: Length = 0.1 m }
  input { g: Acceleration = 9.81 m/s^2 }
  state {
    pos: Point            = origin + (0 m, 1 m)
    vel: Vector<Velocity> = 0
  }
  flow {
    der(pos) = vel
    der(vel) = (0 m/s^2, -g)
  }
}
```

- An object reads nothing of the model around it. What it needs from outside is an **input** (`g` here), which the model connects; an input nobody connects keeps its default.
- An object type is written once and used by as many objects as the model declares. It lives in the model's space: `Point` is a point of the model's plane.

## Parts

The model declares its objects in a `parts` block:

| Part | Declares |
|---|---|
| `ball: Ball` | one object of type `Ball` |
| `ball: Ball { g = g }` | one object, with a value for one of its bindings |
| `row: Ball[3] { pos = origin + (index * 1 m, 2 m) }` | a collection of three members, numbered 1 to 3 |

- A value in braces is an **override**: a starting value for a member's state or parameter, or a **connection** for an input. It is written in the model's scope: `g = g` connects the member's input `g` to the model's parameter `g`, which it then follows.
- In the overrides of a collection, `index` is the member's number, so each member can start in its own place.
- Members are numbered in order; `row[2]` is the second.

## Reading members

| Expression | Reads |
|---|---|
| `ball.pos` | a binding of an object |
| `row[2].pos` | a binding of one member; the number is a constant |
| `sum(b.m for b in row)` | a sum over the members, each named `b` |
| `min(...)`, `max(...)`, `any(...)`, `all(...)` | the smallest, the largest, whether one or every member satisfies a condition |
| `count(row)`, `count(b for b in row if c)` | how many members, or how many satisfy `c` |
| `sum(... for o in row if o != b)` | a filter; `o != b` compares members, not values |

`sum` over no member is `0`, `any` is false and `all` true. `min` and `max` take only filters that compare members.

## A row of balls

**Goal.** Several bouncing balls that each behave the same way, written once, plus summary values across all of them (the highest ball, how many have landed).
**How it is built.** One `object Ball { ... }` holds everything a single ball needs (it is written like a small model). The model's `parts` block makes three of them as a collection `row: Ball[3]` and one more, `moon`. `derived` values with `max(... for b in row)` and `count(...)` combine the members.

Three balls dropped from different heights bounce on a floor at their own radius; a fourth ball bounces on the Moon.

```text
space Plane = euclidean(2)

model Drops in Plane {
  object Ball {
    param { r: Length = 0.1 m }
    input { g: Acceleration = 9.81 m/s^2 }
    state {
      pos: Point            = origin + (0 m, 1 m)
      vel: Vector<Velocity> = 0
    }
    discrete { resting: Boolean = false }
    flow {
      der(pos) = vel
      der(vel) = if resting then 0 else (0 m/s^2, -g)
    }
    event bounce on falling(pos.y - r) {
      set vel = (vel.x, -0.8 * vel.y)
    } zeno settle {
      set vel = 0
      set pos = origin + (pos.x, r)
      set resting = true
    }
  }
  param { g: Acceleration = 9.81 m/s^2 }
  parts {
    row: Ball[3] {
      pos = origin + (index * 1 m, index * 1 m)
      g = g
    }
    moon: Ball { g = 1.62 m/s^2 }
  }
  derived {
    highest: Length = max(b.pos.y for b in row)
    landed:  Real   = count(b for b in row if b.resting)
  }
}
```

- `row` has three balls, the first 1 m to the right and 1 m up, the next 2 m and 2 m, the last 3 m and 3 m. Each follows the model's `g`.
- `moon` is one ball, its input connected to a constant.
- Each ball has its own `bounce` event: the balls bounce independently, and each settles when its bounces accumulate (chapter 5).
- `highest` follows the highest ball; `landed` counts the balls at rest.

A `for` in a view draws one representation per member:

```text
presentation DropsView for Drops {
  view scene: spatial(Plane, scale: 1 m -> 60 px, y: up) {
    axes
    for b in row { marker(b.pos) as ball }
    marker(moon.pos) as moon_ball
  }
  panel numbers {
    label(highest)
    label(landed)
  }
  observe {
    second = row[2].pos.y live
    top    = highest live
    rested = landed live
    moon_y = moon.pos.y live
  }
}
```

- The markers are named `ball[1]`, `ball[2]` and `ball[3]`; each has its own text alternative. `highlight ball` in a timeline highlights all three.

After 0.2 s every ball has fallen `g t² / 2 = 0.1962 m`, the Moon's `0.0324 m`; after 10 s the three balls rest on the floor:

```cases
run first_moments of Drops with DropsView {
  until t0 + 0.2 s
  expect {
    second == 1.8038 m within 1e-9 m
    top    == 2.8038 m within 1e-9 m
    moon_y == 0.9676 m within 1e-9 m
  }
}

run at_rest of Drops with DropsView {
  until t0 + 10 s
  expect {
    second == 0.1 m within 1e-9 m
    rested == 3 within 1e-12
  }
}
```

## Forces between members

An object cannot see the others: a ball does not know the row it is in. Forces between members are written by the model, which contains them all, with a `for` in its `flow` block:

**Goal.** Three bodies pulling on one another by gravity, with their total momentum conserved.
**How it is built.** An object type `Body` and a collection of three; the model's flow gives each body the sum of the others' pulls with `for b in bodies` and a filtered `sum`.

```text
space Plane = euclidean(2)

model Stars in Plane {
  object Body {
    param { m: Mass = 1e24 kg }
    state {
      pos: Point            = origin
      vel: Vector<Velocity> = 0
    }
    flow { der(pos) = vel }
  }
  const { G: Quantity<L^3/M/T^2> = 6.674e-11 m^3/kg/s^2 }
  parts {
    bodies: Body[3] {
      pos = origin + (index * 1e7 m, 0 m)
      vel = (0 m/s, index * 100 m/s)
    }
  }
  flow {
    for b in bodies {
      der(b.vel) = sum(G * o.m * (o.pos - b.pos) / |o.pos - b.pos|^3 for o in bodies if o != b)
    }
  }
  derived {
    momentum: Vector<Momentum> = sum(b.m * b.vel for b in bodies)
  }
}
```

- `for b in bodies { der(b.vel) = ... }` gives each body its acceleration: the sum of the others' pulls, Newton's law of gravitation. `o != b` leaves the body itself out.
- The body's own flow moves it (`der(pos) = vel`); the model's flow sets how its velocity changes. A model may write the state of the objects it contains; an object never writes another's.
- `momentum` adds the bodies' momenta. The pulls between two bodies are equal and opposite, so the total never changes: the bodies start with momentum `(1 + 2 + 3) × 100 m/s × 1e24 kg = 6e26 kg m/s` upwards, and keep it.

```text
presentation Sky for Stars {
  view sky: spatial(Plane, scale: 1e6 m -> 10 px, y: up) {
    for b in bodies {
      marker(b.pos) as star
      trace(b.pos every 100 s)
    }
  }
  observe {
    px = momentum.x live
    py = momentum.y live
  }
}
```

```cases
run conserved of Stars with Sky {
  until t0 + 5000 s
  expect {
    px == 0 kg*m/s within 1e15 kg*m/s
    py == 6e26 kg*m/s within 1e15 kg*m/s
  }
}
```

The tolerance is a relative `2e-12`: the sums are computed in floating point, and the bodies' speeds grow to kilometres per second.

## Members that come and go

The collections so far keep their members for the whole run. A fountain does not: it throws a drop every half second, and each drop is gone when it falls back. A collection whose membership changes declares the most members it makes in one run, with `max` (D-057):

**Goal.** A fountain that throws a drop every half second, each drop gone when it lands.
**How it is built.** A collection with a capacity, `Drop[max ...]`; a time event `create`s a drop; an event repeated per drop `destroy`s it at the ground.

```text
space Plane = euclidean(2)

model Fountain in Plane {
  object Drop {
    state {
      pos: Point            = origin
      vel: Vector<Velocity> = 0
    }
    flow {
      der(pos) = vel
      der(vel) = (0 m/s^2, -9.81 m/s^2)
    }
  }
  param { speed: Velocity = 5 m/s }
  parts { drops: Drop[max 40] }
  event spray on every 0.5 s {
    create drops { vel = (1 m/s, speed) }
  }
  for d in drops {
    event land on falling(d.pos.y) { destroy d }
  }
  derived {
    flying:  Real   = count(drops)
    highest: Length = max(d.pos.y for d in drops) otherwise 0 m
  }
}
```

- `drops: Drop[max 40]` starts empty and makes at most 40 drops in a run. `Drop[3, max 40]` would start with three.
- `create drops { vel = (1 m/s, speed) }` in a handler makes the next drop, with a starting value for its velocity; its position starts where `Drop` says, at the origin. The values are read when the event happens, so they may use the model's state or the event's payload.
- `for d in drops { event land ... }` gives every drop its own `land` event, which reads the drop as `d`. When a drop falls back through zero height, `destroy d` removes it.
- A drop that is not made yet, or already destroyed, does nothing: it does not move, its events do not happen, and aggregates leave it out. `count(drops)` is the number of drops in the air.
- `max` over no drop has no value (at the start, before the first spray, and between drops if the fountain stops), so `highest` says what to use then with `otherwise`.

| Form | Does |
|---|---|
| `c: T[max m]`, `c: T[n, max m]` | a collection that starts with none or `n` members and makes at most `m` in a run |
| `c: T[max inf]`, `c: T[n, max inf]` | a collection whose members come and go without a limit |
| `create c { x = e ... }` | makes the next member, with starting values; several `create` in one handler make several members |
| `destroy b` | removes the member `b`: a loop variable, a contained member, or `c[k]` |
| `for b in c { event ... }` | an event of the model for each member |
| `set b.x = e` | in such an event, writes the member's binding (a model may write its members' state) |

The `k`-th drop made is `drops[k]` for the whole run: `drops[1]` is the first drop, which lands after `2 × 5 / 9.81 = 1.0194 s`. Its number is never given to another drop, so its name, its marker and its trace belong to one drop. Read by number, a drop holds its starting values before it is made and its last values after it is destroyed.

```text
presentation Spray for Fountain {
  view scene: spatial(Plane, scale: 1 m -> 50 px, y: up) {
    axes
    for d in drops { marker(d.pos) as drop }
  }
  panel numbers {
    label(flying)
    label(highest)
  }
  observe {
    n     = flying live
    top   = highest live
    first = drops[1].pos.y live
  }
}
```

A marker is drawn for each drop in the air, and only while it is there. At 1.2 s the first drop has landed; the second, thrown at 0.5 s, is at `5 × 0.7 - 4.905 × 0.7² = 1.09655 m`, and the third, thrown at 1 s, is lower:

```cases
run early of Fountain with Spray {
  until t0 + 1.2 s
  expect {
    n     == 2 within 1e-12
    top   == 1.09655 m within 1e-9 m
    first == 0 m within 1e-9 m
  }
}
```

The capacity is part of the model: it says how long the fountain can run. 40 drops, one every half second, last until 19.5 s; a run past 20 s stops there with the diagnostic that `drops.capacity` is exceeded. Choose a capacity that covers the longest run the presentation shows.

When no limit belongs in the model, say so: `drops: Drop[max inf]` (D-066). Prismal then chooses room for the members itself and makes more whenever a run needs it, in tests, sessions and lessons alike, so a fountain left running in a lab never stops. A run is the same as with the room it ended up with written as the capacity, and runs again identically. The room grows by elaborating the model again, so a lab with thousands of members made over a long session pays for it with a pause each time the room doubles; a capacity written in the model costs nothing at run time.

## Relations between members

A spring joins two balls; a thread ties one bead to another. The spring is not a ball, and it is not a line drawn between them: it is a **relation**, with its own values (a stiffness, a rest length) and two **endpoints**, each a member of a collection (MK-8.5, D-058).

**Goal.** Two balls joined by a spring that can be cut and joined again.
**How it is built.** A `relation Spring(a in balls, b in balls)` with its stiffness and rest length; a relation set `springs`; the model's flow adds each spring's pull at its two ends; requested events `disconnect` and `connect` it.

```text
space Plane = euclidean(2)

model Pair in Plane {
  object Ball {
    param { m: Mass = 1 kg }
    state {
      pos: Point            = origin
      vel: Vector<Velocity> = 0
    }
    flow { der(pos) = vel }
  }
  relation Spring(a in balls, b in balls) {
    param {
      k:    Quantity<M/T^2> = 2 N/m
      rest: Length          = 1 m
    }
    derived {
      stretch: Length        = |b.pos - a.pos| - rest
      pull:    Vector<Force> = k * stretch * (b.pos - a.pos) / |b.pos - a.pos|
    }
  }
  parts {
    balls: Ball[2, max 3] { pos = origin + ((2 * index - 3) * 1 m, 0 m) }
    springs: Spring[1, max 4] {
      a = balls[1]
      b = balls[2]
    }
  }
  flow {
    for o in balls {
      der(o.vel) = (sum(s.pull for s in springs if s.a == o) - sum(s.pull for s in springs if s.b == o)) / o.m
    }
  }
  event cut on request { disconnect springs[1] }
  event join on request { connect springs(balls[1], balls[2]) { k = 4 N/m } }
  derived {
    links: Real   = count(springs)
    gap:   Length = balls[2].pos.x - balls[1].pos.x
  }
}
```

- `relation Spring(a in balls, b in balls) { ... }` declares a relation type: its endpoints `a` and `b` are members of `balls`, and its body has the bindings of each spring. Inside it, `a.pos` is the position of the ball at `a`.
- `springs: Spring[1, max 4]` holds the springs, like a collection: one at the start, at most four made in a run. The overrides of the starting spring name its endpoints, `a = balls[1]`.
- The balls move by the springs' pulls. A ball cannot see the springs; the model writes each ball's acceleration as a sum over the springs that end at it: `s.a == o` holds when the spring's endpoint `a` is the ball `o`. A spring pulls its `a` end towards `b`, and its `b` end the other way.
- `connect springs(balls[1], balls[2]) { k = 4 N/m }` makes a spring between two members, with a starting value; `disconnect springs[1]` removes one. Destroying a ball disconnects every spring that ends at it.

| Form | Does |
|---|---|
| `relation R(a in c, b in d) { ... }` | a relation type; its endpoints are members of `c` and `d` |
| `rs: R[n, max m] { a = c[1] ... }` | a set of relations, like a collection; starting relations name their endpoints |
| `s.a`, `s.a.pos` | the member at an endpoint, and one of its bindings |
| `s.a == o` | whether the endpoint is the member `o` |
| `connect rs(x, y) { k = e }` | makes a relation between the members `x` and `y` |
| `disconnect s` | removes a relation |

The endpoints of a relation are chosen while the model runs, so `s.a.pos` is the position of whichever ball the spring holds at that moment.

```text
presentation Springs for Pair {
  view scene: spatial(Plane, scale: 1 m -> 50 px, y: up) {
    for o in balls { marker(o.pos) as ball }
    for s in springs { segment(s.a.pos, s.b.pos) as spring }
  }
  observe {
    gap   = gap live
    links = links live
  }
}
```

The balls start 2 m apart on a spring of rest length 1 m. Each ball has mass `m`, so the gap `d` obeys `d'' = -2 k (d - 1 m) / m`: it oscillates as `1 m + cos(ω t) × 1 m` with `ω = sqrt(2 k / m) = 2 rad/s`, and after 1 s it is `1 + cos 2 = 0.583853 m`.

```cases
run oscillates of Pair with Springs {
  until t0 + 1 s
  expect {
    gap   == 0.583853 m within 1e-6 m
    links == 1 within 1e-12
  }
}
```

An event can be repeated for each relation, like an event for each member. A thread snaps when stretched by 1 m:

**Goal.** Beads tied by threads that snap when stretched by more than a metre.
**How it is built.** A relation `Thread` with a derived `stretch`, and an event repeated per thread that disconnects it when its stretch passes 1 m.

```text
space Plane = euclidean(2)

model Threads in Plane {
  object Bead {
    state {
      pos: Point            = origin
      vel: Vector<Velocity> = 0
    }
    flow { der(pos) = vel }
  }
  relation Thread(a in beads, b in beads) {
    derived { stretch: Length = |b.pos - a.pos| - 1 m }
  }
  parts {
    beads: Bead[2, max 2] {
      pos = origin + ((index - 1.5) * 1 m, 0 m)
      vel = ((2 * index - 3) * 1 m/s, 0 m/s)
    }
    threads: Thread[1, max 1] {
      a = beads[1]
      b = beads[2]
    }
  }
  for s in threads {
    event snap on rising(s.stretch - 1 m) { disconnect s }
  }
  derived { held: Real = count(threads) }
}
```

```text
presentation Watch for Threads {
  observe {
    early = held at t0 + 0.4 s
    late  = held at t0 + 0.6 s
  }
}
```

The beads start 1 m apart and separate at 2 m/s, so the thread is stretched by 1 m after 0.5 s:

```cases
run snaps of Threads with Watch {
  until t0 + 1 s
  expect {
    early == 1 within 1e-12
    late  == 0 within 1e-12
  }
}
```

A relation that is not made yet, or already disconnected, has no values: `springs[3].stretch` before a third spring is connected is not a number. The count, sums and representations leave it out, as they leave out members that are not alive.

### Relations without a direction

The spring above has a direction: its pull is towards `b` at its `a` end and the other way at its `b` end, so the model sums the pulls at each end separately. A spring has no real direction, though, and nothing should change if its endpoints are swapped. An **undirected** relation says so (D-064):

**Goal.** A chain of balls linked by springs whose two ends are alike.
**How it is built.** An `undirected relation Link`; each ball's pull is summed over the links that have it, `s.has(o)`, towards the ball at the other end, `s.other(o)`.

```text
space Plane = euclidean(2)

model Chain in Plane {
  object Ball {
    param { m: Mass = 1 kg }
    state {
      pos: Point            = origin
      vel: Vector<Velocity> = 0
    }
    flow { der(pos) = vel }
  }
  undirected relation Link(a in balls, b in balls) {
    param {
      k:    Quantity<M/T^2> = 2 N/m
      rest: Length          = 1 m
    }
    derived { stretch: Length = |b.pos - a.pos| - rest }
  }
  parts {
    balls: Ball[3] { pos = origin + ((index - 1) * 2 m, 0 m) }
    links: Link[2, max 2] {
      a = balls[index + 1]
      b = balls[index]
    }
  }
  flow {
    for o in balls {
      der(o.vel) = sum(s.k * s.stretch * (s.other(o).pos - o.pos) / |s.other(o).pos - o.pos| for s in links if s.has(o)) / o.m
    }
  }
  derived {
    span:   Length = balls[3].pos.x - balls[1].pos.x
    middle: Real   = count(s for s in links if s.has(balls[2]))
  }
}
```

- `s.has(o)` holds when `o` is either endpoint of `s`; `s.other(o)` is the endpoint that is not `o`. Each ball is pulled towards the other end of every link it is on, in one sum.
- The links are written with their endpoints in either order (`a = balls[index + 1]`): an undirected relation's result does not depend on it.
- An undirected relation has its two endpoints in one collection. Outside its body, the model reads its endpoints only with `has` and `other`: `s.a == o` is refused (MK-E26), since an undirected relation has no first end. A presentation may still draw `segment(s.a.pos, s.b.pos)`.
- `has` and `other` work on any relation with two endpoints in one collection, directed or not.

```text
presentation ChainView for Chain {
  view scene: spatial(Plane, scale: 1 m -> 40 px, y: up) {
    for o in balls { marker(o.pos) as ball }
    for s in links { segment(s.a.pos, s.b.pos) as link }
  }
  observe {
    span   = span at t0 + 1 s
    middle = middle live
  }
}
```

The three balls start 2 m apart on links of rest length 1 m. The pulls on the middle ball cancel, so it stays; each outer ball oscillates on one link from a fixed point with `ω = sqrt(k / m) = sqrt 2 rad/s`, and the span is `2 (1 + cos(sqrt 2 t))` m.

```cases
run symmetric of Chain with ChainView {
  until t0 + 1 s
  expect {
    span   == 2.311887 m within 1e-6 m
    middle == 2 within 1e-12
  }
}
```

### Relations across containers

An endpoint's collection may belong to a contained object: a bond between atoms of two different cells is held by the model that contains both cells (D-065). The collection is named by its path, `left.atoms`, and so are its members and loops:

**Goal.** Two cells of atoms, with bonds from an atom of one cell to an atom of the other.
**How it is built.** An object `Cell` with its own collection of atoms; the model's relation has endpoints in `left.atoms` and `right.atoms`, paths through the contained cells.

```text
space Plane = euclidean(2)

model Cells in Plane {
  object Atom {
    param { m: Mass = 1 kg }
    state {
      pos: Point            = origin
      vel: Vector<Velocity> = 0
    }
    flow { der(pos) = vel }
  }
  object Cell {
    param { x0: Length = 0 m }
    parts {
      atoms: Atom[2, max 2] { pos = origin + (x0, (index - 1) * 1 m) }
    }
    event release on at t0 + 1.5 s { destroy atoms[1] }
  }
  relation Bond(a in left.atoms, b in right.atoms) {
    param {
      k:    Quantity<M/T^2> = 2 N/m
      rest: Length          = 1 m
    }
    derived { pull: Vector<Force> = k * (|b.pos - a.pos| - rest) * (b.pos - a.pos) / |b.pos - a.pos| }
  }
  parts {
    left:  Cell { x0 = -1 m }
    right: Cell { x0 = 1 m }
    bonds: Bond[1, max 2] {
      a = left.atoms[1]
      b = right.atoms[1]
    }
  }
  flow {
    for o in left.atoms  { der(o.vel) = sum(s.pull for s in bonds if s.a == o) / o.m }
    for o in right.atoms { der(o.vel) = -sum(s.pull for s in bonds if s.b == o) / o.m }
  }
  derived {
    gap:   Length = right.atoms[1].pos.x - left.atoms[1].pos.x
    links: Real   = count(bonds)
  }
}
```

- `relation Bond(a in left.atoms, b in right.atoms)`: the endpoints are atoms of the cell `left` and of the cell `right`. A path goes through contained objects, never through a collection: `cells[1].atoms` is not a collection of the model.
- `left.atoms[1]`, `for o in left.atoms` and `count(left.atoms)` name members and collections of contained objects the same way.
- Each cell lets go of its first atom at 1.5 s, in an event of the cell. Destroying an atom disconnects the bonds that end at it, wherever they are held.

```text
presentation Bonds for Cells {
  view scene: spatial(Plane, scale: 1 m -> 50 px, y: up) {
    for o in left.atoms { marker(o.pos) as l }
    for o in right.atoms { marker(o.pos) as r }
    for s in bonds { segment(s.a.pos, s.b.pos) as bond }
  }
  observe {
    gap    = gap at t0 + 1 s
    before = links at t0 + 1 s
    after  = links at t0 + 2 s
  }
}
```

The bonded atoms start 2 m apart on a bond of rest length 1 m, like the spring pair: their gap after 1 s is `1 + cos 2` m.

```cases
run bonded of Cells with Bonds {
  until t0 + 2 s
  expect {
    gap    == 0.583853 m within 1e-6 m
    before == 1 within 1e-12
    after  == 0 within 1e-12
  }
}
```

### Making members of a contained object

**Goal.** A model adds atoms to one of its cells from its own events, without the cell knowing why.
**How it is built.** The cell holds the collection (`atoms: Atom[1, max 4]`); the model's event writes `create left.atoms { ... }`, naming the collection by its path through the contained object `left` (D-074).

```text
space Plane = euclidean(2)

model Feed in Plane {
  object Atom { state { pos: Point = origin } }
  object Cell {
    parts { atoms: Atom[1, max 4] { pos = origin } }
  }
  parts { left: Cell }
  state { clock: Time = 0 s }
  flow { der(clock) = 1 }
  event add on every 1 s from t0 + 1 s if count(left.atoms) < 3 {
    create left.atoms { pos = origin + (count(left.atoms) * 1 m, 0 m) }
  }
  derived {
    atoms: Real   = count(left.atoms)
    last:  Length = left.atoms[3].pos.x
  }
}

presentation FeedChecks for Feed {
  observe {
    n    = atoms at t0 + 3.5 s
    last = last  at t0 + 3.5 s
  }
}
```

```cases
run fed of Feed with FeedChecks {
  until t0 + 4 s
  expect {
    n    == 3 exactly      // one atom to start, one made at 1 s and one at 2 s
    last == 2 m exactly    // made when the cell had two atoms
  }
}
```

- The starting values (`pos = ...`) are read in the scope where `create` is written, the model's: `count(left.atoms)` is the count before the atom is made.
- The member is made where the collection is held: it is `left.atoms[3]`, with the cell's capacity.
- A cell's own event and the model's event creating atoms of one collection at the same instant conflict, as two creates from two events always do (MK-16.6): the run stops, or pauses in a lab.
- `connect` takes paths the same way. In v0 relation sets are held by the model that declares the relation type (D-065), so its paths name sets of the model itself.

## Labs with members

A lab with many objects lets the learner act on one of them: drag this planet, remove that one. The learner's action has to say **which** member it is about. An event can receive a member as its payload (D-059):

**Goal.** A lab of planets: the learner drags a planet, clicks one to remove it, adds one with a button or a click.
**How it is built.** Events take a planet as their payload (`on request(p in planets)`); representations repeated per planet carry a drag that proposes its position and a click that requests the removal.

```text
space Plane = euclidean(2)

model Orbits in Plane {
  object Planet {
    state {
      pos: Point            = origin + (1 m, 0 m) intervenable
      vel: Vector<Velocity> = (0 m/s, 1 m/s)
    }
    flow { der(pos) = vel }
  }
  param { mu: Quantity<L^3/T^2> = 1 m^3/s^2 }
  parts {
    planets: Planet[2, max 6] {
      pos = origin + (index * 1 m, 0 m)
      vel = (0 m/s, sqrt(mu / (index * 1 m)))
    }
  }
  flow {
    for p in planets {
      der(p.vel) = -mu * (p.pos - origin) / |p.pos - origin|^3
    }
  }
  event remove on request(p in planets) { destroy p }
  event add on request {
    create planets {
      pos = origin + (0 m, -2 m)
      vel = (sqrt(mu / (2 m)), 0 m/s)
    }
  }
  event place on request(q: Point) if |q - origin| > 0.5 m {
    create planets {
      pos = q
      vel = sqrt(mu / |q - origin|^3) * (-(q - origin).y, (q - origin).x)
    }
  }
  derived { n: Real = count(planets) }
}
```

- The planets circle a sun at the origin; `mu` is the sun's gravitational parameter. Each starts on a circular orbit: at distance `r`, the speed `sqrt(mu / r)`.
- `on request(p in planets)` declares a payload that is a member of `planets`. Whoever requests `remove` says which planet; the handler reads it as `p`, like a loop variable: `destroy p`, `p.pos`, `set p.vel = ...`.
- The event happens only for a planet that is alive. A request for a planet already removed, or not made yet, is refused with the reason (`planets[2]` is not alive).
- `add` takes no payload: it makes the next planet, on a circular orbit of radius 2 m.
- `place` takes a point and makes a planet there, on a circular orbit through it: the velocity is the direction from the sun turned a quarter turn, `(-(q - origin).y, (q - origin).x)`, at the circular speed. Its condition keeps new planets away from the sun.
- `pos` is `intervenable`, so the learner may move a planet (D-023).

An event may take a member and a value together: `event kick on request(p in planets, j: Vector<Momentum>) { set p.vel = p.vel + j / 1 kg }` is requested as `kick(planets[1], (0 kg*m/s, 1 kg*m/s))`. `emit E(p)` passes a member on to the events that follow `E`, which receive it as `on E(q in planets)`.

In a lab, the learner acts on the planet under the pointer:

```text
presentation Sky for Orbits {
  view sky: spatial(Plane, scale: 1 m -> 80 px, y: up) {
    on click as q request place(q)
    marker(origin) as sun
    for p in planets { trace(p.pos every 0.05 s) }
    for p in planets {
      marker(p.pos) as planet {
        on drag as q { propose p.pos = q }
        on click request remove(p)
      }
    }
  }
  panel controls { button(add, label: "Add a planet") }
}
```

- `on drag as q { propose p.pos = q }` in a representation repeated per member moves that member: dragging `planet[2]` proposes a new position for `planets[2]`. The binding must be intervenable in the object type.
- `on click request remove(p)` requests `remove` with the planet clicked. A click is a press and release that does not move; moving further makes it a drag. A click only requests an event: what happens is written in the model.
- With the keyboard, Tab gives focus to each planet in turn; arrow keys move it and Enter or space activates its click. The text alternative says so: `planet[2] ..., activate: remove`.
- `on click as q request place(q)`, written in the view itself, acts on a click where no representation takes it: an empty point of the sky. `q` is the point clicked, in the view's space, and the view requests `place` with it (D-060). A click on a planet removes the planet; a click beside it places a new one. When the presentation also permits `pan`, a press that moves pans the view and a press that does not move clicks.
- The button adds a planet. A host that has no pointer requests the same events with the member named: `remove` with payload `"planets[2]"`.

| Form | Does |
|---|---|
| `on request(b in c)` | an event requested for one member of `c`, read as `b` in its condition and handler |
| `on request(b in c, x: T)` | several payloads: a member and a value |
| `request E(c[k])`, `request E(c[k], v)` | a timeline's request for a member |
| `emit E(b)` | passes a member to the events that follow `E` |
| `on drag as q { propose b.x = q }` | in `for b in c { ... }`: dragging moves that member |
| `on click request E(b)` | in `for b in c { ... }`: clicking requests `E` for that member |
| `on click as q request E(q)` | in a view: clicking an empty point requests `E` with the point |

A lesson requests for members as a learner would. The case checks the effect of each request:

```text
presentation Tour for Orbits {
  view sky: spatial(Plane, scale: 1 m -> 80 px, y: up) {
    marker(origin) as sun
    for p in planets { marker(p.pos) as planet }
  }
  observe {
    before = n at t0 + 0.5 s
    after  = n at t0 + 1.5 s
    later  = n at t0 + 2.5 s
    x1     = planets[1].pos.x at t0 + 2.5 s
    x3     = planets[3].pos.x at t0 + 2.5 s
  }
  timeline {
    scene sky {
      beat watch { run rate 1; wait 1 s }
      beat fewer { request remove(planets[2]); wait 1 s }
      beat more  { request add; wait 1 s }
    }
  }
}
```

Planet 1 goes round once in `2π` s, so at 2.5 s it is at `cos 2.5 = -0.801144 m`. Planet 2 is removed at 1 s. The planet added at 2 s is `planets[3]`, since a number is never given twice; on its orbit of radius 2 m it turns at `sqrt(mu / r^3) = 0.353553` rad/s, so after 0.5 s its x is `2 sin(0.176777) = 0.351715 m`.

```cases
run tour of Orbits with Tour {
  expect {
    before == 2 within 1e-12
    after  == 1 within 1e-12
    later  == 2 within 1e-12
    x1     == -0.801144 m within 1e-5 m
    x3     == 0.351715 m within 1e-5 m
  }
}
```

## Names in results

Each member's bindings and events have names built from the member: `row[2].pos`, `moon.bounce`. They appear in diagnostics, event logs and text alternatives. A mistake in the object type is reported once for each member, and located at the line in the object type.

A relation's bindings are named like a member's: `springs[2].stretch`.

## Mistakes

```error
// error: MK-E26
space Plane = euclidean(2)
model M in Plane {
  object Ball {
    state { pos: Point = origin }
  }
  parts { row: Ball[3] }
  derived { y4: Length = row[4].pos.y }
}
```

`row` has members 1 to 3; there is no `row[4]`.

```error
// error: SX-E08
space Plane = euclidean(2)
model M in Plane {
  object Ball {
    state { pos: Point = origin }
  }
  parts { row: Ball[3] }
  derived { y: Length = row.pos.y }
}
```

A collection has no `pos`; one of its members has. Write `row[1].pos.y`, or an aggregate: `max(b.pos.y for b in row)`.

```error
// error: SX-E03
space Plane = euclidean(2)
model M in Plane {
  object Ball {
    state { pos: Point = origin }
  }
  parts { ball: Ball { g = 1.62 m/s^2 } }
}
```

`Ball` has no `g` to give a value to: an override names a binding of the object.

```error
// error: MK-E26
space Plane = euclidean(2)
model M in Plane {
  object Ball {
    state { pos: Point = origin }
  }
  parts { row: Ball[3] }
  event more on every 1 s { create row }
}
```

`row` has a fixed membership. To make members during a run, declare how many it can make: `row: Ball[3, max 10]`.

```error
// error: SX-E08
space Plane = euclidean(2)
model M in Plane {
  param { speed: Velocity = 1 m/s }
  event halt on every 1 s { destroy speed }
}
```

`destroy` removes a member of a collection; `speed` is a binding.

```error
// error: MK-E26
space Plane = euclidean(2)
model M in Plane {
  object Ball {
    state { pos: Point = origin }
  }
  relation Link(a in balls, b in balls) {}
  parts {
    balls: Ball[2]
    links: Link[max 3]
  }
  event tie on every 1 s { create links }
}
```

`links` holds relations: they are made with `connect links(balls[1], balls[2])`, which names the endpoints.

```error
// error: MK-E26
space Plane = euclidean(2)
model M in Plane {
  object Ball {
    state { pos: Point = origin }
  }
  parts { balls: Ball[3, max 3] }
  event remove on request(b in balls) { destroy b }
  event clear on every 1 s { emit remove(2) }
}
```

The payload of `remove` is a member: give one, `emit remove(balls[2])`, not its number.

## Exercises

Solutions to the exercises not solved here are in [chapter 11](11-solutions.md).

1. Add a fourth ball to `row`. Which expectations change?
2. Give each ball in `row` a different radius with an override (`r = index * 0.05 m`), and show the radius as a `circle(b.pos, b.r)` in the view.
3. Chain three masses with springs: each mass is pulled towards its neighbours. Write the force on member `b` as a sum over the others with a filter that keeps only neighbours. (Hint: give each member its number as a parameter, `n = index`, and compare numbers.)
4. Make the fountain throw two drops at each spray, one to each side (`vel = (1 m/s, speed)` and `vel = (-1 m/s, speed)`). How many drops are in the air at 1.2 s?
5. Give `Orbits` an event `kick` that takes a planet and an impulse (`on request(p in planets, j: Vector<Momentum>)`). A representation has one click, so draw at each planet a second marker, a little above it, whose click requests `kick` for that planet with a fixed impulse.

<details>
<summary>A solution to exercise 2</summary>

In `parts`, add `r = index * 0.05 m` to the overrides of `row`; in the view, draw each ball as a circle of its own radius:

```prismal
for b in row { circle(b.pos, b.r) as ball }
```

A ball's `bounce` compares `pos.y` with its own `r`, so each ball now rests on the floor at its own radius.

</details>
