# Where Does a Chain Break?

A complete example: a chain of beads and links hanging between two hooks, as a lab where the learner hangs weights and pulls the hooks apart, and as a narrated lesson that breaks it. Every program on this page is compiled and its cases run by `cargo test` (`crates/prismal-web/tests/examples.rs`).

**Goal.** A learner finds that a hanging chain pulls hardest at its hooks, not under the weight, and that pulling a chain straight makes it pull far harder than its weight: a taut chain breaks where a sagging one holds.
**How it is built.**

- *the beads*: an object type `Bead` with a position, a velocity and a load, in a collection of nine; the first and last are the hooks;
- *the links*: an undirected relation `Link` between two beads, which pulls when stretched past its rest length and never pushes;
- *the forces*: the model sums, for each bead, the pulls of the links on it, its weight and a damping that lets the chain come to rest; the hooks glide to where the parameter `spread` puts them;
- *breaking*: an event for each link, `snap`, that disconnects it when its pull reaches the chain's strength;
- *the learner's actions*: a click on a bead hangs 1 kg on it (`hang`, an event whose payload is a member); a slider sets the spread;
- *the lab*: the chain drawn link by link, each with its pull written beside it and turning red near the breaking point;
- *the lesson*: the chain at rest, a weight hung, the hooks pulled apart until a link snaps, then the learner's own spread.

## The model

```text
space Plane = euclidean(2)

model Chain in Plane {
  object Bead {
    param {
      m:    Mass = 0.1 kg
      side: Real = 0          // -1 for the left hook, 1 for the right one, 0 for a bead
    }
    state {
      pos: Point            = origin
      vel: Vector<Velocity> = 0
    }
    discrete { load: Mass = 0 kg }     // the weights hung on it
    flow { der(pos) = vel }
  }
  undirected relation Link(a in beads, b in beads) {
    param {
      k:    Quantity<M/T^2> = 500 N/m
      rest: Length          = 0.5 m
    }
    derived {
      stretch: Length = |b.pos - a.pos| - rest
      pull:    Force  = if stretch > 0 m then k * stretch else 0 N     // a link pulls, it never pushes
    }
  }
  param {
    g:        Acceleration = 9.81 m/s^2
    strength: Force        = 20 N                       // a link breaks at this pull
    spread:   Length       = 3 m    in [2 m, 4.2 m]     // the distance between the hooks
    tau:      Time         = 0.3 s                      // how quickly motion dies away
  }
  parts {
    beads: Bead[9] {
      pos  = origin + ((index - 5) * 0.375 m, 0 m)
      side = if index == 1 then -1 else if index == 9 then 1 else 0
    }
    links: Link[8, max 8] {
      a = beads[index]
      b = beads[index + 1]
    }
  }
  flow {
    for o in beads {
      der(o.vel) = if o.side != 0 then ((origin + (o.side * spread / 2, 0 m)) - o.pos) / tau^2 - 2 * o.vel / tau else
        sum(s.pull * (s.other(o).pos - o.pos) / |s.other(o).pos - o.pos| for s in links if s.has(o)) / (o.m + o.load) +
          (0 m/s^2, -g) - o.vel / tau
    }
  }
  for s in links {
    event snap on rising(s.pull - strength) { disconnect s }
  }
  event hang on request(o in beads) if o.side == 0 { set o.load = o.load + 1 kg }
  derived {
    intact:  Real  = count(links)
    biggest: Force = max(s.pull for s in links) otherwise 0 N
    weight:  Force = sum((o.m + o.load) * g for o in beads if o.side == 0)
    held:    Force = -sum(s.pull * (s.other(beads[1]).pos - beads[1].pos).y / |s.other(beads[1]).pos - beads[1].pos| for s in links if s.has(beads[1])) -
                     sum(s.pull * (s.other(beads[9]).pos - beads[9].pos).y / |s.other(beads[9]).pos - beads[9].pos| for s in links if s.has(beads[9]))
    left:    Real  = count(s for s in links if s.has(beads[1]))
    right:   Real  = count(s for s in links if s.has(beads[9]))
  }
}
```

- The chain is 4 m long (eight links of 0.5 m) and starts as a straight line 3 m wide, so its links are slack and it falls into a sag.
- A bead is pulled by every link it is on, towards the bead at the other end: `s.other(o)` is that bead, and `s.has(o)` picks the links on `o`. The links are undirected: a link from bead 3 to bead 4 is the same as one from 4 to 3.
- The hooks are beads too, so that links can end at them, but no link moves them: each glides to its place, `spread / 2` from the middle, and stays there. Moving the slider moves them smoothly, never in one jump.
- `- o.vel / tau` damps the motion, as air and friction would, so the chain comes to rest within a few seconds.
- `hang` takes the bead clicked as its payload. Its condition refuses the hooks.
- `held` is the upward pull of the links on the two hooks: what the hooks hold up. At rest it equals `weight`, the beads and their loads.

## The lab

```text
presentation ChainLab for Chain {
  title "Where does a chain break?"
  view scene: spatial(Plane, scale: 1 m -> 120 px, y: up) {
    title("Click a bead to hang 1 kg on it")
    for s in links {
      segment(s.a.pos, s.b.pos, color: gray, when: s.pull < 0.75 * strength) as link
      segment(s.a.pos, s.b.pos, color: red, when: s.pull >= 0.75 * strength) as strained
      text("{s.pull:.1}", at: s.a.pos + (s.b.pos - s.a.pos) / 2 + (-0.55 m, -0.15 m), when: s.a.pos.x + s.b.pos.x < 0 m)
      text("{s.pull:.1}", at: s.a.pos + (s.b.pos - s.a.pos) / 2 + (0.1 m, -0.15 m), when: s.a.pos.x + s.b.pos.x >= 0 m)
    }
    for o in beads {
      marker(o.pos, label: none, color: blue, when: o.side == 0) as bead { on click request hang(o) }
      marker(o.pos, label: none, color: ink, when: o.side != 0) as hook
      text("{o.load:.0}", at: o.pos + (0 m, -0.25 m), when: o.load > 0 kg)
    }
  }
  panel controls {
    title("The chain")
    slider(spread, range: [2 m, 4.2 m], ends: ("slack", "taut"))
    button(reset, label: "Start again")
    text("{intact} of 8 links hold.")
    text("Strongest pull: {biggest:.1}. A link breaks at {strength:.0}.")
    text("The hooks hold up {held:.1}; the beads and weights weigh {weight:.1}.")
  }
  layout row(scene, controls)
  permit learner { timeline_controls }
  observe {
    hook_pull   = links[1].pull at t0 + 10 s
    middle_pull = links[4].pull at t0 + 10 s
    held_10     = held          at t0 + 10 s
    weight_10   = weight        at t0 + 10 s
  }
}
```

- Each link is drawn twice, in gray and in red, and `when:` shows one of the two: red once its pull reaches three quarters of the strength.
- The pull of each link is written beside its middle, with one decimal, outside the curve: to the left of the links on the left half, to the right on the right half. A text starts at its point, so a text on the left is placed half a metre further out. A link that has snapped is no longer in `links`, so neither its line nor its number is drawn.
- `on click request hang(o)` on each bead's marker hangs a weight on that bead. The hooks are drawn by a second marker, which takes no click.
- The number under a bead is its load, shown only once it has one.

At rest, the links at the hooks pull hardest, the middle one least, and the hooks hold up exactly the chain's weight, `7 × 0.1 kg × 9.81 m/s²`:

```cases
run at_rest of Chain with ChainLab {
  until t0 + 10 s
  expect {
    hook_pull   == 4.0262 N within 1e-3 N
    middle_pull == 2.1590 N within 1e-3 N
    weight_10   == 6.867 N within 1e-9 N
    held_10     == 6.867 N within 1e-4 N
  }
}
```

## The lesson

```text
presentation ChainLesson for Chain {
  title "Where does a chain break?"
  view scene: spatial(Plane, scale: 1 m -> 120 px, y: up) {
    title("Pulls in newtons")
    for s in links {
      segment(s.a.pos, s.b.pos, color: gray, when: s.pull < 0.75 * strength) as link
      segment(s.a.pos, s.b.pos, color: red, when: s.pull >= 0.75 * strength) as strained
      text("{s.pull:.1}", at: s.a.pos + (s.b.pos - s.a.pos) / 2 + (-0.55 m, -0.15 m), when: s.a.pos.x + s.b.pos.x < 0 m)
      text("{s.pull:.1}", at: s.a.pos + (s.b.pos - s.a.pos) / 2 + (0.1 m, -0.15 m), when: s.a.pos.x + s.b.pos.x >= 0 m)
    }
    for o in beads {
      marker(o.pos, label: none, color: blue, when: o.side == 0) as bead
      marker(o.pos, label: none, color: ink, when: o.side != 0) as hook
      text("{o.load:.0}", at: o.pos + (0 m, -0.25 m), when: o.load > 0 kg)
    }
  }
  panel numbers {
    title("The chain")
    text("Hooks {spread:.1} apart.")
    text("Strongest pull: {biggest:.1}. A link breaks at {strength:.0}.")
    text("{intact} of 8 links hold.")
  }
  layout row(scene, numbers)
  permit learner { timeline_controls }
  observe {
    rest_pull   = links[1].pull at t0 + 12.9 s
    loaded_pull = links[1].pull at t0 + 27 s
    left_after  = left          at t0 + 41 s
    right_after = right         at t0 + 41 s
    hanging     = biggest       at t0 + 41 s
  }

  timeline {
    scene hanging {
      beat intro { run rate 1; narrate "A chain of nine beads hangs between two hooks. The number beside each link is how hard it pulls, in newtons." for 8 s }
      beat ends  { narrate "The links at the hooks pull hardest: they hold up everything below them." for 5 s }
    }
    scene weight {
      beat load { request hang(beads[4]); narrate "Now hang a one kilogram weight on the fourth bead." for 4 s }
      beat look { narrate "Every link pulls harder, and the link at the nearer hook most of all." for 5 s }
    }
    scene taut {
      beat warn { narrate "Now pull the hooks apart to straighten the chain. The weight stays the same." for 5.5 s }
      beat pull { intervene { set spread = 4.1 m }; narrate "It breaks at the hook, not beside the weight. A straight chain pulls much harder than its weight." for 7 s }
      beat dangle { narrate "What is left hangs from one hook, which now holds the whole weight: less than the taut chain pulled." for 7 s }
    }
    scene yours {
      beat again {
        reset
        explore limit 30 s keep spread {
          slider(spread, range: [2 m, 4.2 m], ends: ("slack", "taut"))
        } fallback {
          sequence { intervene { set spread = 2.5 m }; wait 3 s }
        }
      }
      beat close { narrate "A sagging chain holds more. That is why power lines and rope bridges are hung with a sag." for 7 s }
    }
  }
}
```

- The run starts in the first beat and is never held: the chain keeps moving under the narration, so presentation time and simulation time agree until the reset.
- `request hang(beads[4])` hangs the weight as a click on the fourth bead would.
- `intervene { set spread = 4.1 m }` moves the hooks as the slider would; they glide apart over about a second, and the left link snaps while the narration says so.
- `reset` starts the run again from the beginning, with a whole chain, for the learner's own spread.

The weight is hung at 13 s and the hooks are pulled apart at 27.5 s, as the narration says where the chain breaks. Each narration is as long as a system voice takes to say it. The left link snaps, and the rest hangs from the right hook, whose link now holds up all the weight: `(0.7 kg + 1 kg) × 9.81 m/s²`.

```cases
run lesson of Chain with ChainLesson {
  expect {
    rest_pull   == 4.0262 N within 1e-3 N
    loaded_pull == 12.437 N within 1e-2 N
    left_after  == 0 within 1e-12
    right_after == 1 within 1e-12
    hanging     == 16.677 N within 1e-2 N
    end of dangle == 41.5 s exactly
  }
}
```

## Exporting the lesson

```sh
cargo run --release -p prismal-media -- examples/chain.md ChainLesson chain.mp4 --speech system
```

The video plays the explore beat's fallback: the chain again, with the hooks 2.5 m apart.
