# 21. Algorithms

Every model so far changed by flows and by events whose operations all happen at once. Some things worth learning are **procedures**: sorting a list, searching it, finding a greatest common divisor. A procedure is a sequence of instructions with loops and choices, and what a learner needs to see is each step of it. This chapter writes procedures as **algorithms**: statements run in order, paused after each step so a view can show it (D-101).

> **In plain words.**
> - An **algorithm** is a list of instructions carried out one after another, like a recipe. `while` repeats instructions as long as something holds; `for` repeats them for each number in a range; `if` chooses.
> - `step` is a pause. Each time the algorithm's trigger happens (every half second, or when the learner presses a button), the algorithm carries on from where it paused until the next `step`.
> - The values an algorithm works on are ordinary model values: its counters and its temporary values are `discrete` state, and a list of values is a collection of objects, one per item. So everything it does can be drawn, observed and checked.

## Sorting a list

**Goal.** Bubble sort of eight numbers, drawn as bars: each half second the next pair of neighbours is compared and swapped if they are out of order, the two bars being compared in a colour of their own.
**How it is built.** Each bar is a member of a collection `bars: Bar[8]`; its value `v` starts from a list, `(5, 2, 7, 1, 8, 4, 6, 3)[index]`. The algorithm `sort` runs `on every 0.5 s`: two `for` loops over the discrete counters `i` and `j`, an `if` that swaps through a temporary value `held`, and a `step` after each comparison. A plot view draws one segment per bar, coloured by conditions on `j`.

```text
model Sorting {
  object Bar {
    param { k: Real = 0 }
    discrete { v: Real = 0 }
  }
  parts {
    bars: Bar[8] {
      k = index
      v = (5, 2, 7, 1, 8, 4, 6, 3)[index]
    }
  }
  discrete {
    i:           Real    = 0
    j:           Real    = 0
    held:        Real    = 0
    comparisons: Real    = 0
    swaps:       Real    = 0
    sorted:      Boolean = false
  }
  algorithm sort on every 0.5 s {
    for i in [1, 7] {
      for j in [1, 8 - i] {
        set comparisons = comparisons + 1
        if bars[j].v > bars[j + 1].v {
          set held = bars[j].v
          set bars[j].v = bars[j + 1].v
          set bars[j + 1].v = held
          set swaps = swaps + 1
        }
        step
      }
    }
    set sorted = true
  }
}

presentation SortLab for Sorting {
  until t0 + 16 s
  view chart: plot(x: [0, 9], y: [0, 9]) {
    for b in bars {
      segment((b.k, 0), (b.k, b.v), color: gray, width: 8, when: not sorted and b.k != j and b.k != j + 1) as bar
      segment((b.k, 0), (b.k, b.v), color: orange, width: 8, when: not sorted and (b.k == j or b.k == j + 1)) as compared
      segment((b.k, 0), (b.k, b.v), color: green, width: 8, when: sorted) as done
    }
  }
  panel counts {
    label(comparisons)
    label(swaps)
    button(restart)
  }
  observe {
    first      = bars[1].v at t0 + 15 s
    last       = bars[8].v at t0 + 15 s
    compared   = comparisons at t0 + 15 s
    exchanged  = swaps at t0 + 15 s
    early_swap = bars[2].v at t0
    finished   = sorted at t0 + 15 s
  }
}
```

```cases
run sort_eight of Sorting with SortLab {
  until t0 + 15 s
  expect {
    early_swap == 5 exactly       // the first comparison, at t0, swapped 5 and 2
    first == 1 exactly
    last == 8 exactly
    compared == 28 exactly        // 7 + 6 + ... + 1 pairs
    exchanged == 14 exactly       // one swap per pair out of order in the starting list
    finished == true
  }
}
```

- `algorithm sort on every 0.5 s { ... }` is written like an event (chapter 5), with any trigger an event can have, and an `if c` condition if wanted. Its block holds **statements**.
- The first occurrence, at `t0`, runs from the top: `i` takes 1, `j` takes 1, the first pair is compared and swapped, and `step` ends the work of that occurrence. The next occurrence, half a second later, carries on after the `step`: `j` grows to 2, and so on.
- After the last comparison (`i = 7`, `j = 1`, at 13.5 s), the occurrence at 14 s runs what follows the loops, `set sorted = true`, and reaches the end. A finished algorithm does not occur again; `button(restart)` starts the run, and the algorithm, from the beginning.
- `swaps` ends at 14: bubble sort swaps exactly the pairs that are out of order in the starting list, and `(5, 2, 7, 1, 8, 4, 6, 3)` has 14 of them.

## How an algorithm runs

- Inside an algorithm, statements run **in order**, and each reads the values the statements before it wrote. That is why the swap needs `held`: after `set bars[j].v = bars[j + 1].v`, the old value of `bars[j].v` is gone. (An event's operations are different: they all read the values from before the event, chapter 5.)
- What one occurrence changes is committed at once when it reaches `step` or the end, as one transition: constraints are checked on the result, other events at the same instant combine with it, and a conflict with them is reported as for events.
- The statements are `set`, `emit`, `create` and `destroy` (as in events), `if c { ... } else { ... }` (with `else if`), `while c { ... }`, `for i in [a, b] { ... }`, and `step`. `contribute` belongs to events: an algorithm writes with `set`.
- `for i in [a, b]` sets `i` to `a`, and while `i` is at most `b` it runs the block and adds 1 to `i`. `b` is read again before each turn, so `for j in [1, 8 - i]` shrinks as `i` grows. After the loop, `i` is `b + 1`.
- A loop's counter is a `discrete` number of the model, declared like any other. So are an algorithm's temporary values. Nothing is hidden: a view can show `i`, `j` and `held`.
- `bars[j]` is the member numbered `j` while the run goes on; a number outside `1` to `8` stops the run with a model failure that names it. `(5, 2, 7)[k]` is the `k`-th of a list of values.
- An algorithm that loops a million times without reaching `step` or its end stops the run: almost always a `while` whose condition never becomes false.
- `floor(x)`, `ceil(x)` and `round(x)` give whole numbers of a number, `mod(a, b)` the remainder `a - b floor(a / b)`.

## Searching, by hand or on a clock

**Goal.** Binary search for a value in a sorted list of fifteen: each look halves the part of the list that can still hold it. The learner steps it with a button, or lets it run.
**How it is built.** The algorithm `search` follows the event `look` (`on look`). `look` happens when the learner presses `button(look)`, and when a clock `tick` emits it while `auto` is on. `lo` and `hi` bound the part still searched, `mid` is the middle of it, read with `floor`. The view colours the searched part, the middle and the rest differently.

```text
model Search {
  object Cell {
    param { k: Real = 0 }
    discrete { v: Real = 0 }
  }
  parts {
    cells: Cell[15] {
      k = index
      v = (2, 5, 8, 12, 16, 23, 38, 41, 56, 72, 77, 85, 91, 95, 99)[index]
    }
  }
  param {
    target: Real    = 72 in [0, 100]
    auto:   Boolean = true
  }
  discrete {
    lo:    Real    = 1
    hi:    Real    = 15
    mid:   Real    = 0
    looks: Real    = 0
    found: Boolean = false
  }
  event look on request
  event tick on every 1 s if auto { emit look }
  algorithm search on look {
    while lo <= hi and not found {
      set mid = floor((lo + hi) / 2)
      set looks = looks + 1
      if cells[mid].v == target {
        set found = true
      } else if cells[mid].v < target {
        set lo = mid + 1
      } else {
        set hi = mid - 1
      }
      step
    }
  }
}

presentation SearchLab for Search {
  until t0 + 8 s
  view list: plot(x: [0, 16], y: [0, 100]) {
    for c in cells {
      segment((c.k, 0), (c.k, c.v), color: gray, width: 6, when: c.k < lo or c.k > hi) as out
      segment((c.k, 0), (c.k, c.v), color: blue, width: 6, when: c.k >= lo and c.k <= hi and c.k != mid) as left
      segment((c.k, 0), (c.k, c.v), color: orange, width: 6, when: c.k == mid) as middle
    }
  }
  panel controls {
    slider(target, range: [0, 100])
    toggle(auto)
    button(look, label: "Look")
    label(looks)
    label(found)
  }
  observe {
    first_mid  = mid at t0
    second_mid = mid at t0 + 1 s
    third_mid  = mid at t0 + 2 s
    count      = looks at t0 + 8 s
    hit        = found at t0 + 8 s
  }
}
```

```cases
run find_72 of Search with SearchLab {
  until t0 + 8 s
  expect {
    first_mid == 8 exactly     // 41 < 72: the search goes on in 9 to 15
    second_mid == 12 exactly   // 85 > 72: 9 to 11
    third_mid == 10 exactly    // 72 found
    count == 3 exactly
    hit == true
  }
}

run absent of Search with SearchLab {
  param target = 50
  until t0 + 8 s
  expect {
    count == 4 exactly         // 41, 85, 72, 56: then lo > hi
    hit == false
  }
}
```

- `algorithm search on look` follows an event, as `event e on look` would. Here two things make `look` happen: the button and the clock. Pressing the button while the clock runs at the same instant still makes one look.
- Fifteen values need at most four looks: each halves what is left, and `2^4 = 16 > 15`.
- With `auto` off, nothing happens until the learner presses Look. Moving `target` during a search changes what the next look compares with; `button(restart)` (as in the sort lab) would start over.

## A loop with no list

**Goal.** Euclid's algorithm: the greatest common divisor of two numbers by repeated remainders.
**How it is built.** One `while` loop on two discrete numbers, `mod` for the remainder.

```text
model Euclid {
  discrete {
    a: Real = 1071
    b: Real = 462
    r: Real = 0
  }
  algorithm gcd on every 1 s {
    while b != 0 {
      set r = mod(a, b)
      set a = b
      set b = r
      step
    }
  }
}

presentation EuclidChecks for Euclid {
  observe {
    a1 = a at t0
    a3 = a at t0 + 2 s
    b3 = b at t0 + 2 s
  }
}
```

```cases
run gcd_1071_462 of Euclid with EuclidChecks {
  until t0 + 5 s
  expect {
    a1 == 462 exactly      // 1071 = 2 × 462 + 147
    a3 == 21 exactly       // 462 = 3 × 147 + 21, 147 = 7 × 21
    b3 == 0 exactly
  }
}
```

A counter that is not a discrete number of the model:

```error
// error: MK-E37
model Wrong {
  state { x: Real = 0 }
  algorithm count on start {
    for x in [1, 3] { step }
  }
}
```

`x` is continuous state; a counter is declared in `discrete { ... }`.

## Exercises

1. Change the sort to **selection sort**: for each position `i`, find the smallest value from `i` to 8 (a `for` with a discrete `smallest` holding its number), then swap it into place. Count its swaps for the same list. Why are there at most 7?
2. Give the sort lab a toggle and a button like the search's, so a learner can step the comparisons by hand.
3. Write **insertion sort** with a `while` that moves a value left while its left neighbour is larger. Which list makes it compare the most?
4. Euclid's algorithm with subtraction instead of `mod`: replace the larger number by the difference until they are equal. How many steps does it take for 1071 and 462?
