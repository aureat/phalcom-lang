# Execution visualizer — UI specification

The visual design. Every choice carries its justification and cites the requirement it serves from
[`REQUIREMENTS.md`](REQUIREMENTS.md). Where a choice rejects an obvious alternative, the rejection is
recorded — those are the load-bearing decisions.

**Committed 2026-07-19** (options A3 / B3+B1 / C1+C3 / D-horizontal / E-card-rail / F2).

---

## 1 · The four decisions everything else follows from

| # | Decision | Serves | Rejected alternative, and why |
|---|---|---|---|
| **D1** | **The tape is one flat horizontal array; frames are brackets over regions** | N2, R-SPINE | Per-frame boxes — the universal debugger rendering, and *it is the misconception itself*. `stack_offset` indexes a shared `Vec`; boxes assert otherwise |
| **D2** | **A switch is three stoppable cursor positions: take → hole → install** | N4, N7, R-STATIC | A single smooth tween. Lua/Wren swap a pointer and genuinely have no hole; a tween would render Phalcom as if it were Lua |
| **D3** | **Fibers are cards; the host stack is a bolted-down rail** | N1 | Same vocabulary for both — would render Phalcom as stackful coroutines, the model ADR-0030 rejected |
| **D4** | **Editorial register, not IDE register** | R-SILENT, N2 | Debugger chrome. Every debugger shows a call-stack list plus a per-frame variables panel — adopting the register imports N2 for free and primes viewers to hunt for breakpoints |

Everything below is downstream of these.

---

## 2 · Layout

Topology **A3 — centre tape, satellites.**

```
┌─ source ──────────────────────────┬─ bytecode · Counter#tick(_) ─────┐
│   class Counter {                 │    0006  GetLocal   1            │
│ ▸   tick(n) { return n + 1 }      │  ▸ 0008  Invoke     +(_)         │
│   }                               │    0010  Return                  │
├──────┬────────────────────────────┴────────────────────┬─────────────┤
│ HOST │                                                 │   FIBERS    │
│      │        ┌── f0 ────────────┐┌── f1 ──────┐       │             │
│  ▓▓  │        │ self   n=3   ·   ││ self   7   │       │ ▐ F1 ●run ▌ │
│  ▓▓  │  TAPE  ├────┬────┬────┬───┴┼────┬───────┤       │             │
│  ░░  │        │ ⟨C⟩│  3 │  · │  · │ ⟨C⟩│   7   │       │ ▐ F2 ○susp▌ │
│      │        └────┴─▲──┴────┴────┴────┴───────┘       │ ▕▓▓▓▓▏      │
│ d=1  │   cells      ╵ ●n                               │             │
│      │                                                 │ F0─call─▶F1 │
├──────┴─────────────────────────────────────────────────┴─────────────┤
│  ◀◀  ◀  ▮  ▶  ▶▶     ●────────────────────────   event 84 / 150      │
└──────────────────────────────────────────────────────────────────────┘
```

**Region budget** at 1280 px, the envelope minimum (§7):

| Region | Size | Note |
|---|---|---|
| source + bytecode | full width × 180 px | two panes, 50/50 |
| host gutter | 64 px × flex | fused to the tape's left edge |
| **tape** | flex × ~260 px | the spine; gets every pixel the satellites don't need |
| fiber rail | 200 px × flex | right |
| transport | full width × 56 px | bottom |

**Justification for A3 over the alternatives.** Both rejected options satisfy §3's co-visibility table,
so co-visibility does not decide it — *direction of motion* does. In A1 (horizontal bands) the lockers
sit below the tape, so parking reads as **falling**, and falling connotes discard. A park is
preservation. In A3 the lockers are a right rail and the tape slides **sideways into a drawer**, which
is what a park is. A2 (code left) additionally puts the tape in competition with the fiber panel for
one half of the screen, violating R-SPINE.

The host gutter is **fused to the tape's left edge, not floating** — legality is a property *of the
tape's current state*, and separating them costs eye travel on the one relationship E3 exists to teach.

---

## 3 · The tape

The spine. Everything else is a satellite — **except the bytecode pane, see §3.0.**

### 3.0 The bytecode pane is not a satellite

Corrected 2026-07-19, after the D1 prototype. The first cut of this spec treated the bytecode pane as
later work and the prototype shipped without it. It failed immediately: **the tape's slot indices were
unexplained numbers.**

A slot index has two halves and they live in different places:

| Half | Where it is decided | Where it is visible |
|---|---|---|
| the **index** | compile time — `add_local` pushes onto `func.locals`, and a local's slot *is* its position in that Vec (`compiler/lib/scope.rs:132`; `resolve_local`'s own doc at `:194` says the index "doubles as the runtime stack slot") | **the instruction operand** — `GetLocal(u16)`, `bytecode.rs:80` |
| the **base** | runtime — the frame's `stack_offset` | the frame bracket |

The access is one addition, `stack_offset + slot` (`vm/dispatch.rs:721`). No name lookup, no hash, no
map — **the name does not exist at runtime at all.**

So the bytecode pane is where the index's *origin* is visible. Without it the tape shows an anonymous
array and the viewer has no way to learn why slot 3 is `n`. `GetLocal 1` · `offset=2` · `slot 3` is
**one fact rendered across three panes**, and dropping any of the three breaks it.

**Requirement added: R-LINK** — when the current instruction resolves a local, the operand, the frame's
offset, and the target slot are marked as one unit, and the arithmetic is spelled out literally.

This also supplies the cleanest available motivation for upvalues: a captured local **cannot** be
`stack_offset + N`, because the capturing frame's base is gone. `is_captured` (`scope.rs:141`) routes it
to `add_upvalue` instead. Upvalues are not a closure feature bolted on — they are what is needed when
`stack_offset + N` stops being answerable. Rendering §3.0 well is what makes §6 land.

### 3.1 Slots

A horizontal row of fixed-width cells, `28 px` wide at the fine zoom, mono-font values, absolute index
printed below every fourth slot.

| Slot state | Render |
|---|---|
| holds a value | value text, centered, mono |
| holds `nil` (VM sentinel) | `·` in muted foreground |
| beyond stack top | empty cell, hairline border only |
| changed at this cursor | persistent accent outline (§7) |

Values render short: `3`, `"hi"`, `<Counter>`, `None`, `⟨blk⟩`. **No nested inspection** (§7 envelope).
A value too long to fit truncates with a middle ellipsis and gives the full text on hover (R-POINT).

### 3.2 Frame brackets

Frames render as **brackets in lanes above the slots**, depth 0 nearest the tape, deeper frames stacked
upward. Each bracket spans its frame's window, starting at `stack_offset`.

Lanes rather than a single row because a callee's window **begins at the receiver the caller pushed** —
the regions share a boundary and conceptually overlap. A single row of adjacent brackets would assert a
clean partition that does not exist. Lanes render the overlap honestly without reintroducing boxes.

Each bracket carries, in its label strip: chunk name · `offset=N` · `gen=N`. A **block** frame
additionally shows `home ▸ f0` — a method frame has no `home_frame_token`, a block frame does, and that
difference is the whole of non-local return. Rendering it always means the `DeadFrameError` example
needs no new machinery.

Clicking a bracket loads that frame's chunk into the bytecode pane (R-POINT).

### 3.3 Fiber identity is carried by colour

The tape carries the **hue of the fiber that owns it**, as a 3 px left-edge band and a tint on the
bracket labels. When it parks, that hue travels into the locker and the incoming fiber's hue arrives.

This is the colour system's primary job: **hue = fiber identity**, everywhere it appears (tape edge,
locker, chain node, upvalue connector). It makes the move legible in the coarse read (R-ZOOM) — you see
*a colour move*, before you read a single value.

### 3.4 Scroll

Beyond ~40 slots the tape scrolls horizontally, always keeping the **current frame's window** in view.
Auto-scroll on step, manual override, auto re-engages on the next switch.

---

## 4 · Host gutter

A vertical rail on the tape's left edge. Height segments = `native_reentry_depth` (`vm/mod.rs:97`).

- **Rendering** — flat, hatched, **no shadow, no rounded corners, flush with the chrome.** It is the one
  element that must read as *part of the machine* rather than as an object on it.
- **Persistent label** — `machine stack · cannot park`, four words, always visible.
- **Depth readout** — `d=N` at the base.
- **At `d > 0`** the entire gutter takes the warning tint and the tape's left edge gains a matching
  hairline: *yield is illegal right now.*

**Justification for the persistent label.** D3's card/rail contrast carries the semantics, but showing
alone will not land the point in the 5 seconds R-ZOOM allows. Four words of text buy the single most
important non-obvious fact on the screen. This is a deliberate exception to show-don't-tell.

**Always present, in every example** — including the two that never raise it. Introducing the gutter only
in E3 would make it read as a special case bolted on for one lesson, when it is a permanent property of
every instant of execution.

---

## 5 · Fiber rail

Right rail. Three stacked zones.

### 5.1 Fiber cards

One card per live fiber. **Rounded corners, drop shadow, offset from the background** — visibly a thing
that could be picked up (D3). Contains:

```
▐ F1  ● running                    ▌   ← hue band + status
▐ ▕frames▕stack▕upvals▕chk▏        ▌   ← the four compartments
▐ mode: call   gen carried: —      ▌
```

The **four compartments are always drawn**, full when parked and hollow when running. This is the whole
of T3: a viewer reads what a `FiberObject` contains directly off the card, and reads that a *running*
fiber's card is **empty** — because its buffers are in the VM. That inversion is the grip, rendered.

Status glyphs: `● running` · `○ suspended` · `◍ done` · `✗ failed` (`FiberStatus`, `heap/fiber.rs:12`).

### 5.2 Resumer chain

Small directed chain beneath the cards: `F0 ─call─▶ F1 ─try─▶ F2`.

Edge label is `FiberResumeMode` (`fiber.rs:37`) and edge **style** encodes it: `call` solid, `try`
double-stroked. During an unwind the edge being traversed lights; a `try` edge **visibly halts it**.
That is the fiber floor with no prose, and it is why the chain must be co-visible with status (§3).

### 5.3 Spawn hopper

A small, deliberately peripheral strip at the rail's base: `hopper: [F4][F5]`.

**Rendered small on purpose** (N3). `ready_queue` (`vm/mod.rs:140`) holds only fibers never yet started;
a yielding fiber does *not* re-enter it, it goes to its `resumer`. Giving the hopper visual weight
comparable to the chain would teach a round-robin scheduler that does not exist. Labelled
**`spawn hopper`**, never "ready queue" or "scheduler."

---

## 6 · Upvalue cells — the hardest rendering, and the best lesson

A thin **cell strip directly beneath the tape**. Each open upvalue is a cell in the strip with a short
vertical connector rising to its slot.

| Upvalue state | Render |
|---|---|
| `Open { fiber, slot }` | cell in the strip, connector to slot, connector in the owning fiber's hue |
| `Closed(value)` | connector retracts, value appears **inside** the cell |
| two closures sharing one local | **one** cell, two closure references — not two cells |

### The moment that earns the whole design

**Cells do not move when a fiber parks. The tape moves out from under them.**

Cells are heap objects; they are not among the four fields a switch takes. So on a park the tape slides
right into the locker and **the connector stretches to follow it** — unbroken, now reaching into the
card.

That single rendered fact *is* the answer to why `Upvalue::Open` carries a `fiber` field at all
(`heap/upvalue.rs:34-36`, N5). The referent relocated, so the reference must name *which buffer* to look
in, not an address. Under the Lua representation — a raw `TValue*` — that field would be meaningless,
and under a pointer-swap rendering it would look like noise.

**The bug this makes visible.** In the abort trace the fiber dies without closing its upvalues: the tape
is dropped, the locker empties, and the connector is left **reaching into a hollow card**. Rendered, not
narrated (T4).

---

## 7 · Colour, type, and delta

### Palette — four semantic roles, then hues

| Role | Use | Constraint |
|---|---|---|
| **fiber hues** | up to 4, identity only (§3.3) | must survive next to the semantic colours |
| **accent** | delta marking, one colour only | never used decoratively |
| **warn** | `d > 0`, illegal state | reserved; never a fiber hue |
| **error** | raised error, dangling reference, invariant violation | reserved |
| **neutral hatch** | host gutter, pinned state | flat, no shadow, ever |
| **dashed neutral** | the hole | used at exactly one moment (§8) |

Both light and dark themes. Fiber hues chosen for distinguishability under both, and under the common
colour-vision deficiencies — status is *never* carried by hue alone, always hue **plus** glyph.

### Type

- **Sans** (system stack) for labels, prose, callouts, chunk names.
- **Mono** for every value, slot index, bytecode operand, offset and generation number.

The split is the B3+B1 decision made concrete: **editorial outside, schematic inside.** Mono is a
promise of exactness; using it for prose spends that signal on nothing, and using sans for a slot value
withdraws it where it matters.

### Delta marking (R-DELTA)

The element changed at this cursor gets a **persistent accent outline**, decaying over the following two
steps. Plus a **callout** — a short sans annotation with a leader line — on the single most significant
change.

**Persistent, never a flash.** A flash is invisible when scrubbing quickly and absent from a still,
which fails R-STATIC. The callout is what discharges R-SILENT: it is the narration, printed.

At most **one** callout per cursor position. Two competing annotations is panel soup at the sentence level.

---

## 8 · The switch — three beats

Rendering **C1 live**, **C3 for docs stills**. One trace, two renderers.

### Live: three cursor positions

| Beat | Cursor stop | Render |
|---|---|---|
| **1 · take** | `fiber_switch{phase:'take'}` | tape translates right into the outgoing card; card's four compartments fill; VM region empties |
| **2 · hole** | `fiber_switch{phase:'hole'}` | **VM region is a dashed empty outline.** Both cards full. Callout: *"the VM holds nothing — this is a move, not a swap"* |
| **3 · install** | `fiber_switch{phase:'install'}` | incoming card's compartments empty; tape translates left into the VM region; hue is now the incoming fiber's |

**Beat 2 is a stoppable cursor position, not a transition** (R-BOUND, R-STATIC). You can park on it,
screenshot it, and point at it.

**The image at beat 2 is the point of the entire tool:** two full cards and an empty centre. That state
is exactly what `store_live_into` followed by `load_live_from` produces, and it is a state Lua and Wren
**cannot enter** — a pointer swap has no interval where nothing is current. Annotate that contrast at
beat 2; it is the cleanest available proof that Phalcom's switch is not the ancestor's switch.

### Docs still: triptych

The same three beats side by side in one frozen figure for `docs/learn` embeds — before / hole / after,
with the current beat marked. Same trace data, different renderer, no extra authoring.

### Motion budget

- **Permitted**: tape translate on park/unpark (≈ 400 ms, ease-out), connector stretch, cursor movement.
- **Forbidden**: state cross-fades (N7 — a half-faded tape is a state the VM never holds), decorative
  transitions, any reveal that exists only during autoplay (R-NODECOR, R-STATIC).
- **`prefers-reduced-motion`**: all translation becomes an instant cut. **Nothing is lost** — because the
  three beats are *cursor positions* rather than animation keyframes, the reduced-motion rendering is
  exactly as informative. That the design degrades this cleanly is a consequence of D2, and a check that
  D2 is right.

---

## 9 · Page structure — one page per example

Navigation **F2**. Each example is its own page:

```
1 · framing prose          what this program does, what to watch
2 · the program            full source, syntax-highlighted, static
3 · PREDICT gate           2–3 answers, must choose before the player unlocks
4 · the player             §2 layout
5 · takeaway               what you just saw, and the one sentence to keep
6 · forward pointer        the ADR / docs-learn doc that goes deeper
```

### Predict-then-check (R-GATE)

Gates are **authored into the trace**, not inferred: one additional event type,
`{t:'gate', question, options[], answer, because}`. The player halts, the viewer commits, then it
reveals and continues.

Gates are what make this a teaching tool rather than a debugger. Committing to a wrong answer *first* is
the mechanism; a viewer who merely watches learns materially less. Gates are skippable on a second pass
(`g`), never on the first.

Placement, one per example, at the moment the requirement names:

| Example | Gate | Test |
|---|---|---|
| E1 ping-pong | *where is the tape after this yield?* | T1 |
| E2 upvalue across a park | *does the connector break, or follow?* | T4 groundwork |
| E3 legal vs illegal | *which of these two programs cannot work?* | T2 |

---

## 10 · Transport, keys, and edge states

### Transport

`◀◀ ◀ ▮ ▶ ▶▶` plus a scrub track with **switch events marked as ticks** on the track itself, so a viewer
can jump between switches without hunting. Readout: `event 84 / 150`.

### Keyboard (R-KEYS)

| Key | Action |
|---|---|
| `←` `→` | step one event |
| `shift` + `←` `→` | jump to previous/next `fiber_switch` |
| `space` | play / pause |
| `home` `end` | first / last event |
| `1`–`3` | switch example |
| `g` | skip gate (second pass only) |
| `?` | key help |

### Edge states

| State | Render |
|---|---|
| **invariant violation** | full-width **red banner**, naming the failed check and the event index. On by default. Failure mode #1 is a hand-authored trace that lies plausibly; this is the only defence |
| **trace truncated** | explicit strip: *"trace stops at 150 events"* — never silent |
| **end of trace** | transport disables forward, final state stays rendered |
| **error raised** | the raising element takes the error colour; error text in a callout, not a modal |

### Invariant checks (run on every fold step)

1. frame brackets do not overlap illegally or exceed the tape length
2. `upvalue_open` slot is within the current tape
3. no pop from an empty tape
4. every `frame_pop` matches a prior `frame_push`
5. `fiber_switch` phases arrive in `take → hole → install` order
6. no `fiber_switch` while `native_reentry_depth > 0`

Check 6 is the encoded form of the language restriction. A hand-authored trace violating it would be
drawing an illegal machine.

---

## 11 · Deliberately absent

| Absent | Why |
|---|---|
| heap panel | v1 needs only cells, which live in the tape's own strip (§6). A general heap panel is the GC tenant's requirement, not this one |
| class chain / lookup | dispatch is a later tenant; adding it now buys a panel and costs the spine |
| call-site cache state | inline caches, later tenant |
| AST pane | compile-time substrate, a different tool |
| breakpoints, watches | this is not a debugger (§9 non-requirements) |
| autoplay-first presentation | R-MANUAL — a viewer who cannot stop cannot predict |

Each is a **later tenant on the same timeline**, not a v1 compromise (tiebreaker 7). The event schema
and the layout both leave room: new tenants add event types and satellites, never a new spine.

---

## 12 · Build order

1. Trace format + fold reducer + keyframe snapshots (R-SCRUB) + the six invariant checks
2. Tape with brackets and delta marking — **verify D1 reads correctly before anything else is built**
3. Host gutter, fiber rail, transport, keys
4. The three-beat switch (C1), then the triptych renderer (C3)
5. Cell strip and connectors — the stretch-into-locker moment
6. Page shell, gates, prose for E1 → E2 → E3

Step 2 is the go/no-go. If the flat tape with bracket lanes does not read at a glance, D1 is wrong and
everything downstream needs rethinking — so it gets built and judged first, against T1, before the
satellites exist to prop it up.
