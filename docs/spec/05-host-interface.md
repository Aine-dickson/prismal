# 05 Host Interface

- **Version:** v0 (draft), 2026-09-30
- **Part:** how a host system embeds Prismal: loading and editing programs, running presentations, and receiving what to render (D-044, D-045). It adds no meaning to `01` to `04`.
- **Scope:** the operations every host needs, their data, and the bindings through which they are offered. The reference implementation is the `prismal-host` crate (`crates/prismal-host`).

---

## 1. Principles

- **Decisions:** D-018, D-019, D-036, D-037, D-044, D-045, D-047.
- **Prior art:** follows the Language Server Protocol (one JSON protocol, transport left to the host, capabilities declared) and FMI (a simulation unit embedded by any tool through a small, versioned interface). Follows embeddable engines such as SQLite and Lua in keeping storage, files and user interface with the host.

- **HI-1.1** A **host** is any system that embeds Prismal: an authoring studio, a learning platform, a document viewer, a test harness, a command-line tool. Prismal knows no particular host (D-044). The reference web player (`web/`) is a host like any other and uses only this interface.
- **HI-1.2** The interface has three layers: **authoring** (programs as text or IR, checked and identified, section 3), **execution** (presentations opened as instances, driven by the learner's inputs and the host's clock, section 4) and **output** (layouts, frame descriptions and observations, section 5).
- **HI-1.3** Authoring by text and authoring by a visual tool are equal (D-019, D-036). A host may send source text, an IR document, or both over time; Prismal returns the IR, the canonical text and diagnostics either way.
- **HI-1.4** **Division of responsibility.** The host owns storage, files, projects, assets, the editor and its edit history, users and collaboration, the choice of renderer, its windows and the capture of input from its devices (pointer, touch, keyboard). Prismal owns the meaning of programs: checking, identities, runs, sessions (including undo of the learner's interventions, RC section 11), lesson playback, frame descriptions, and the interpretation of the input the host forwards (HI-4.5): what a gesture targets, how a view is framed, zoomed and panned, and which representation has keyboard focus. Prismal never attaches to a host's event loop or registers listeners in it.
- **HI-1.5** The same operations are offered through every binding (section 6), with the same data. A host never needs a binding-specific feature.

---

## 2. Engine and handles

- **HI-2.1** An **engine** holds documents and instances, each named by a **handle** chosen by the engine (an opaque string). A host may hold several documents and several open instances at once; instances of one document are independent.
- **HI-2.2** Handles stay valid until the host closes them. Closing a document closes its instances. Nothing leaks when a host loads, edits and closes documents repeatedly.
- **HI-2.3** An engine is used from one thread at a time. A host that needs parallel work uses several engines.

---

## 3. Authoring

- **HI-3.1** `load` takes source text in the working syntax, a Markdown document whose `text` and `cases` blocks form a program, or an IR document (04). It returns a document handle, or diagnostics when the program does not compile. A document that compiles but fails its checks (MK, PK diagnostics) is still loaded, so that an editor can show the diagnostics against it.
- **HI-3.2** `update` replaces a document's content with new text or IR. The new IR keeps the identities of elements that match the previous version by declaration path (`reconcile`, D-036); `update` returns the identities that disappeared, so that a host can report broken references (PK-2.3).
- **HI-3.3** `rename` renames a declared element by identity and keeps its identity (D-036). It is the operation a visual editor uses; a text edit that renames is a removal and an addition (HI-3.2).
- **HI-3.4** A document answers with its **IR** (04), its **canonical text** (working syntax section 1.4), its **catalogue** (models, presentations with their kinds, cases) and its **diagnostics**. Each diagnostic has a code, a message, the identity of the element it concerns, and a source span when the document came from text.
- **HI-3.5** Cases (`run` blocks) run headless on request and report each expectation (PK-4.2).

---

## 4. Execution

- **HI-4.1** `open` opens a presentation of a checked document as an **instance**: a **lesson** when it has a timeline (PK sections 8, 9; in the interactive or the video medium, PK-9.10), otherwise an **interactive session** (PK section 10). It returns the instance's handle and layout (HI-5.1).
- **HI-4.2** The host owns the clock. A lesson is asked for the frame at a presentation instant; an interactive session is told which simulation instant to show (`seek`, RC section 12). Frames are deterministic (PK-8.7): the same instant gives the same frame.
- **HI-4.3** Learner inputs name representations by identity or author name: set a control, press a button, click a representation that requests an event (`click`, PK-10.5b), click a point of a view in view coordinates (`click_at` with `view`, `x`, `y`, PK-10.5c), pointer down, move and up on a draggable representation, a keyboard step (PK-11.2), cancel, undo and redo. In a lesson, inputs carry their presentation instant and are refused outside explore beats (D-025, PK-9.8a); `press` presses an explore beat's button (PK-9.8b). In a session, `press` on a runtime-control button resets, undoes or redoes (PK-6.3d). Each answers with what happened: applied, previewed (with validity and reason, PK-10.6) or refused (with reason).
- **HI-4.3a** **Environment inputs** (D-051). `set_input` supplies a new value of one of the model's `input` bindings, by name or identity, at the instant an interactive session shows; it is logged, replayed and undone as interventions are, and makes `on input(i)` events due (RC-11.6). It is how a host feeds a model from its own world: a sensor, a controller, another simulation.
- **HI-4.3b** **Requests** (D-050, D-059). `request` makes an event declared `on request` happen at the instant an interactive session shows, by name or identity, with its `payload`: a number in coherent SI units (0 and 1 for a Boolean), an array of numbers for a vector, the coordinates in metres of a point in its space's standard axes, the name of an enumeration's case (`"blue"`), a member named by its collection and number (`"balls[2]"`), and several payloads as an array with one entry each (`["balls[2]", [0, 2]]`). It is logged, replayed and undone as interventions are; a request the event refuses (no payload, a member not alive, a condition that does not hold) answers `ok: false` with the reason.
- **HI-4.4** `restart` replays a lesson without the learner's inputs; `reset` starts an interactive session's run again with an empty log.
- **HI-4.5** **Raw input** (D-047). A host that does not target input itself forwards what it captured, in its own terms, and the engine interprets it the same way for every host:
  - `pointer`: a phase (`down`, `move`, `up`, `cancel`), the view, the position in pixels from the top left of the view as the host drew it, the size it drew the view at (`width`, `height`), the kind of pointer (`mouse`, `pen`, `touch`; a finger reaches farther, PK-10.2a), and an optional `id` that tells several pointers apart. A second touch that comes down while a first pans a spatial view pinches it, where the presentation permits `zoom`: the framing scales by the ratio of the touches' first distance to their current one, and the view point under their first midpoint stays under their midpoint; lifting either touch ends the pinch, and `cancel` returns to the framing before it. Pressing on a draggable part starts a drag (in an interactive session); a press and release on a clickable representation without moving past the pointer's reach clicks it, and a clickable representation that can also be dragged is dragged only once the pointer moves further (PK-10.5b, D-059); pressing elsewhere clicks the point pressed, if the view requests an event for such clicks (PK-10.5c, D-060), when the release is within the pointer's reach of the press, and otherwise pans a spatial view if the presentation permits `pan`; moving without a gesture answers what is under the pointer (`hover`), so that the host can show that it can be grabbed or clicked (`drag`, `click`).
  - `wheel`: a scroll step (`delta`, pixels, positive away) over a view zooms a spatial view about the pointer if the presentation permits `zoom`. `view_reset` returns a view to its own framing.
  - `key_down`: a key named as in the W3C `KeyboardEvent.key` values, with `shift`. `Tab` moves keyboard focus through the focus order (PK-11.2b) and gives focus back to the host past either end; arrow keys step the focused representation or control (PK-11.2); `Enter` and space press a focused button, flip a focused toggle or click a focused clickable representation; `Escape` cancels a drag or a pan. `focus` sets or clears focus directly, for a host whose own focus system moves it (a browser's, a screen reader's).
  - Each answers whether the engine `handled` it, the `action` taken (`drag`, `click`, `pan`, `zoom`, `focus`, `step`, `press`, `toggle`, `cancel`), its `target`, and the outcome as the semantic inputs answer it. An input the engine does not use is answered `handled: false`, so that the host may use it (play and pause on the space bar, for example).
  - In a lesson, raw input carries its presentation instant (`time`); drags are not offered (D-025), and keyboard steps of explore controls are lesson inputs (PK-9.8a).
  - Raw and semantic inputs (HI-4.3) may be mixed: a host may target its own native widgets (a slider, an accessibility tree) and forward the rest raw.

---

## 5. Output

- **HI-5.1** A **layout** lists an instance's views with their coordinate systems (spatial scale and orientation, plot ranges and units, and `x_elapsed`: whether a plot's time axis is the run's elapsed time, false where a histogram counts durations, D-089), the extent of their content over the run (for an initial viewport), the learner's permissions, and for a lesson its beats, captions, explore windows and continue points (`waits`: `beat`, `at`, `end`, `pending`, `explore`; a host stops its clock at a pending one until the learner continues, PK-9.2e).
- **HI-5.1a** **Caption cues** (D-053). Each caption of a lesson's layout has `cue` (its name, PK-9.2d), `start`, `end` and `text`; a frame lists the texts of the captions shown. A host that voices narration plays the recording named after the cue, or synthesized speech, from the cue's start; the cue's timing is the layout's whatever the sound lasts. Sound is the host's: the engine neither loads nor plays audio.
- **HI-5.2** A **frame** is the frame description of PK-12.1, which holds everything any renderer needs to draw it without the model (PK-12.1a): labels, the part a representation is dragged by, a control's symbol and display unit, the author's color and line by name (PK-6.6a, D-061), and for formulas and equations the symbolic IR and a layout drawable with text and lines (PK-6.5a, D-046). The host interface adds nothing specific to one medium; a binding for a medium may add to frames what that medium uses (the WebAssembly binding adds MathML).
- **HI-5.2a** **Viewports** (D-047). Each view of a frame carries the viewport every renderer draws it with, and the engine maps raw input with: `box` (x, y, width, height in view coordinates) and `size` (its natural drawn size in pixels). A spatial view's box is its framing (the extent of its content with room around it; in a session grown to keep what it shows in view), changed by the learner's zoom and pan, or the timeline's camera (D-042); it is fitted into the drawn area, uniformly scaled and centred. A plot's box is its ranges, filling the drawn area less `margin` pixels on every side, `y` upwards. A frame also names the representation with keyboard focus (`focus`).
- **HI-5.3** A host may draw frames with its own renderer, or use a reference renderer: `web/` draws frames in any web view, interactively; `prismal-svg` draws each frame as a standalone SVG document (still images, vector documents, image sequences for video). A renderer needs only layouts and frames.
- **HI-5.4** **Observations** (PK section 3) of an instance's current run are available by name, as values and as text.

---

## 6. Bindings and protocol

- **HI-6.1** **Rust API.** The `prismal-host` crate offers the engine as a Rust type. Native hosts in Rust (a desktop studio, a server, a command-line tool) link it directly.
- **HI-6.2** **Protocol.** Every operation is also a JSON request answered by a JSON response, through one entry point: `handle(request) -> response`. The protocol is independent of transport: a host may call it in process, over WebAssembly, over a pipe or a socket.

  ```json
  { "protocol": 1, "op": "open", "document": "d1", "presentation": "ProjectileLab" }
  { "ok": { "instance": "i1", "layout": { ... } } }
  { "error": [ { "code": "HI-E02", "message": "no presentation `Lab`" } ] }
  ```

- **HI-6.3** **WebAssembly.** The `prismal-web` crate exports the engine's protocol to JavaScript, for browsers and web views.
- **HI-6.4** **Process.** The `prismal-stdio` program serves the protocol over standard input and output, for hosts in any language that can start a process: one request per line of JSON, one response per line, in order (JSON Lines). A request MAY carry an `id` of any JSON value, which its response repeats, so that a host can match responses to requests. Nothing but responses is written to standard output; the process ends when its input ends. `crates/prismal-stdio/client.py` is a host in Python.
- **HI-6.4a** Further bindings (a C ABI for languages that link libraries) are added when a host needs them; they carry the protocol unchanged.
- **HI-6.5** **Versions.** Requests carry the protocol version. `capabilities` answers the protocol and IR versions (D-037), the representation kinds and timeline actions the implementation supports, the input it takes (semantic and raw operations, pointer kinds, keys), and its limits, so that a host can adapt without trial and error.

### 6.1 Operations

| Operation | Takes | Answers |
|---|---|---|
| `capabilities` | - | protocol and IR versions, supported kinds and actions |
| `load` | `text` or `ir` | `document`, `catalogue`, `diagnostics` |
| `update` | `document`, `text` or `ir` | `catalogue`, `diagnostics`, `removed` identities |
| `rename` | `document`, `element`, `name` | `diagnostics` |
| `ir`, `text` | `document` | the IR, the canonical text |
| `locate` | `document`, `element` | its source span |
| `run_cases` | `document` | each case's expectations |
| `close` | `document` or `instance` | - |
| `open` | `document`, `presentation`, `medium`? | `instance`, `layout` |
| `layout`, `frame` | `instance`, `time`?, `dt`? | the layout, the frame |
| `seek`, `reset`, `restart` | `instance`, `time`? | the clock or the layout |
| `set_control`, `press`, `click`, `key`, `pointer_down`, `pointer_move`, `pointer_up`, `cancel`, `undo`, `redo`, `continue` | `instance`, `rep`, value, `time` in a lesson | the outcome |
| `set_input` | `instance`, `input`, `value` | the outcome |
| `request` | `instance`, `event`, `payload`? | the outcome |
| `pointer` | `instance`, `phase`, `view`, `x`, `y`, `width`?, `height`?, `pointer`?, `time` in a lesson | `handled`, `action`, `target` or `hover`, the outcome |
| `wheel` | `instance`, `view`, `x`, `y`, `width`?, `height`?, `delta`, `time` in a lesson | `handled`, `action` |
| `key_down` | `instance`, `key`, `shift`?, `time` in a lesson | `handled`, `action`, `focus` or `target`, the outcome |
| `focus` | `instance`, `rep` or null, `time`? | `focus` |
| `view_reset` | `instance`, `view` | `handled` |
| `observations` | `instance` | observations by name |

### 6.2 Diagnostics of the interface

| Code | Meaning |
|---|---|
| HI-E01 | malformed request: not JSON, unknown operation, a missing or mistyped field, an unsupported protocol version |
| HI-E02 | unknown handle, presentation, representation or element |
| HI-E03 | the operation does not apply: an interactive input to a lesson, a lesson input to a session, an instance of a document with errors |

---

## History

- 2026-09-30 written with D-044 and D-045.
- 2026-09-30 HI-5.2: frames carry formula layouts; MathML moved to the WebAssembly binding (D-046).
- 2026-09-30 HI-5.3: the SVG renderer named as a second reference renderer.
- 2026-09-30 HI-6.4: the process binding `prismal-stdio` with request ids.
- 2026-09-30 HI-4.3a: environment inputs (D-051); the event log shows payloads (D-050).
- 2026-10-01 HI-4.3b: requests with payloads, members named `balls[2]` (D-059).
- 2026-10-01 HI-4.3, HI-4.5: clicks on representations, told from drags by movement; Enter and space activate them (D-059).
- 2026-10-01 HI-4.3b: points and enumeration cases as payloads.
- 2026-10-01 HI-5.2: frames carry the author's color and line by name (D-061).
- 2026-10-01 HI-4.5: pointer `id`; two touches pinch a view.
- 2026-10-01 HI-4.3, HI-4.5: `click_at`, and presses on an empty point of a view that takes clicks, told from pans by movement (D-060).
- 2026-09-30 HI-1.4, HI-4.5, HI-5.2a, HI-6.5: the host captures input and Prismal interprets it; raw input, viewports and focus in frames (D-047).
- 2026-09-30 HI-5.1a: caption cues named for voicing narration (D-053).
- 2026-10-01 HI-5.1: continue points in a lesson's layout (D-067).
- 2026-10-01 HI-4.3: `press` in lessons, runtime-control buttons (D-069).
- 2026-10-13 HI-5.1: `x_elapsed` of plot views (D-089).
