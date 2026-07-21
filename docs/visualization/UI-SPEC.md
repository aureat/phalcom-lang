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

A horizontal row of equal-width cells on a **9-column grid** (≈ 48 px+ at the envelope minimum),
mono-font values, and a label row beneath.

> **Amended after the build (2026-07-19).** This section originally specified `28 px` slots with the
> absolute index printed below every fourth. Both were wrong in practice:
> - `28 px` cannot hold `<Counter>`. Values are the reason the tape exists, so the slot sizes to the
>   value, not the reverse. Cost: the visible envelope drops from ~40 slots to ~9. Acceptable — no
>   teaching trace has exceeded 7.
> - The label row now shows the **innermost frame's local name** for slots inside its window
>   (`self`, `n`, `bump`) and the absolute index elsewhere. This is strictly better: it renders
>   R-LINK's payload continuously instead of only when an instruction fires. It also produces a
>   genuine lesson for free — inside `bump`, slot 1 shows as `1` and not as `n`, because from that
>   frame's base the name is *not reachable*, which is precisely why a cell exists (§6).

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
additionally shows `home ▸ fN genM` — a method frame has no `home_frame_token`, a block frame does, and
that difference is the whole of non-local return.

**Generations are minted by the engine from the pinned counter, never authored** (§5.4). That is what
makes the dead-frame marker *derived* rather than scripted: a home token is stale precisely when
`frames[home.index].gen !== home.gen`, computed from live state every render. E4 needed no new
machinery beyond this, exactly as predicted.

**Metadata degrades; it never vanishes.** An early build suppressed `offset`/`gen`/`home` on brackets
narrower than three slots to avoid clipping. E4's tape is *two slots wide*, so the rule hid the exact
labels that example exists to show — a narrow bracket is when generations matter most, not least.
Below three slots the strip now drops to a compact form (`gen=3`, `▸f1g2 ✗ DEAD`) and the full text
moves to the tooltip.

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
▐ F1                     failed   ▌   ← hue band + status
▐ ▕frames▕stack▕upvals▕chk▏       ▌   ← the four compartments
▐ result: <Error boom>            ▌   ← only when set
```

The **four compartments are always drawn**, full when parked and hollow when running. This is the whole
of T3: a viewer reads what a `FiberObject` contains directly off the card, and reads that a *running*
fiber's card is **empty** — because its buffers are in the VM. That inversion is the grip, rendered.

`result` appears only once set, and carries the error tint on a `failed` fiber — that slot holding an
`Error` instead of a return value *is* the fiber floor's outcome (§5.2, E5).

Status: `running` · `suspended` · `done` · `failed` (`FiberStatus`, `heap/fiber.rs:12`), always as a
word, never hue alone (§7).

**A trace must model the resumer parking.** A running child implies its resumer's buffers are sitting
in the resumer's object; a trace that starts a fiber without that step renders one full card at the
hole instead of two and silently destroys the crown-jewel frame (§8). `check.mjs` enforces this as
`TWO-FULL-CARDS`.

### 5.2 Resumer chain

Small directed chain beneath the cards: `F0 ─call─▶ F1 ─try─▶ F2`.

Edge label is `FiberResumeMode` (`fiber.rs:37`) and edge **style** encodes it: `call` solid, `try`
double-stroked. During an unwind the edge being traversed lights; a `try` edge **visibly halts it**
("unwind stops here — the fiber floor") while a `call` edge shows it crossing. That is the fiber floor
with no prose, and it is why the chain must be co-visible with status (§3).

The mode is stored on the fiber but **set by the resume call**, so it is properly an *edge* property.
Rendering it on the edge rather than on the card is what makes E5's lesson visible: the two variants
differ in exactly one glyph, and containment turns out to be the caller's decision.

### 5.4 Pinned state

A small readout at the rail's base: `next_frame_generation = N`, styled distinctly from the cards —
dashed, flush, not liftable.

It exists because E1's takeaway asserts *"movable is position, pinned is identity"* and the first build
showed only the movable half. The readout **must not change when the tape moves**; that non-movement
is the claim. E4 then reads this counter directly: it advances on every frame push, never on a pop,
and never travels on a switch — which is what makes a `FrameToken` unique across fibers rather than
merely within one.

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

**Built 2026-07-20** as an SVG overlay (`#upvsvg`) spanning the whole machine row, redrawn from live DOM
geometry on every `render()` — never cached across cursors, so a still frame is correct by construction
and the connector cannot desync from the state it is drawn from. Solid to the slot when live, dashed
into the fiber's card when parked or holed; nothing drawn for a closed cell.

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

**Built 2026-07-20** as `tools/viz/triptych.html`, a second renderer over the same `EXAMPLES` data —
`trace-data.js` was split out of `index.html`'s inline script (byte-identical move, diffed before
editing) so both pages, and `check.mjs`, load the identical trace/engine code with no re-authoring.
`?ex=&var=&switch=` picks the trace and which take/hole/install triple to freeze; `&mark=1|2|3` applies
`.beat.current` for prose that wants to point at one beat. Built ahead of an actual `docs/learn` need,
by explicit request — the gate this section states was not met at build time.

### During the hole there is no current instruction

A consequence of D2 worth stating, because it looked like a bug and is not. With no frames there is no
top frame, hence no `ip` and no chunk: the bytecode pane reads **"no frame — nothing is executing"**.

This is not a gap in the rendering. `ip` lives on the `CallFrame` (`frame.rs:72`), so it parks with
`frames`; between take and install the VM genuinely holds no instruction pointer at all. A pane that
kept showing the outgoing chunk would be asserting execution that is not happening.

### Motion budget

- **Permitted**: tape translate on park/unpark (≈ 400 ms, ease-out), connector stretch, cursor movement.
- **Forbidden**: state cross-fades (N7 — a half-faded tape is a state the VM never holds), decorative
  transitions, any reveal that exists only during autoplay (R-NODECOR, R-STATIC).
- **`prefers-reduced-motion`**: all translation becomes an instant cut. **Nothing is lost** — because the
  three beats are *cursor positions* rather than animation keyframes, the reduced-motion rendering is
  exactly as informative. That the design degrades this cleanly is a consequence of D2, and a check that
  D2 is right.

**Tween built 2026-07-20.** The beats are still cursor stops — `render()` remains instant and correct
for every navigation path, exactly as before. `step()` (prev/next, plain arrow keys, autoplay) lays a
clone of the tape over that already-correct render and animates only the clone's position: `tw-out` on
take (tape exits toward the outgoing card), `tw-in` on install (tape enters from the incoming one),
~400ms ease-out. Scrub, jump-to-switch, and first/last stay on the untouched instant path, so fast
scrubbing still shows no smearing. `prefers-reduced-motion` short-circuits both tween branches, so the
reduced-motion rendering is unchanged from the pre-tween build — the argument above still holds, it is
just no longer the *only* rendering.

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

Gates are **authored per example**, not inferred: `{q, opts[{t, ok}], because}`, rendered above the
player. The viewer commits, then the answer and its reasoning reveal.

Gates are what make this a teaching tool rather than a debugger. Committing to a wrong answer *first* is
the mechanism; a viewer who merely watches learns materially less.

**The gate collapses once answered** — unchosen options hide, leaving the choice made and the
explanation. A gate sits above the player deliberately (commit before seeing), but left expanded it
pushes the machine below the fold and costs the 5-second coarse read (R-ZOOM).

**Locking the transport.** Built 2026-07-20: transport buttons and the scrub track disable while a
gate is unanswered, and `go()` itself clamps the cursor to 0 in that state — so a keyboard shortcut or
a programmatic `go()` call can't bypass the disabled buttons either. Unlocks on answer via the existing
`GATED=false` path. Examples without a gate are unaffected.

Placement, one per example, at the moment the requirement names:

| Example | Gate | Test |
|---|---|---|
| E1 ping-pong | *where does the fiber's stack go on a yield?* | T1, T5 |
| E2 upvalue across a park | *does the reference break, or follow?* | T4 |
| E3 legal vs illegal | *which program cannot work, and why?* | T2 |
| E4 dead frame | *what happens when a block returns through a dead home?* | — |
| E5 call vs try | *what decides whether the host survives a fiber's failure?* | — |

**E4 and E5 were later tenants that arrived early.** Both were specced as "later" on the grounds that
they needed no new panels — which held. E4 needed only `home ▸ fN genM` on block brackets (§3.2) plus
engine-minted generations; E5 needed only the mode on the chain edge (§5.2) and a `result` line on the
card (§5.1). They are where the tool stops being a stack visualiser and becomes an argument about
language design, because each sits on a **feature collision** rather than a feature:
*closures ⊗ frame lifetime*, and *errors ⊗ concurrency*.

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
| `1`–`5` | switch example |
| `t` | toggle light / dark |

### Edge states

| State | Render |
|---|---|
| **invariant violation** | full-width **red banner**, naming the failed check and the event index. On by default. Failure mode #1 is a hand-authored trace that lies plausibly; this is the only defence |
| **trace truncated** | explicit strip: *"trace stops at 150 events"* — never silent |
| **end of trace** | transport disables forward, final state stays rendered |
| **error raised** | the raising element takes the error colour; error text in a callout, not a modal |

### Validation

Two layers. The player runs the **structural** checks on every fold step and shows a red banner; a
headless runner, `tools/viz/check.mjs`, runs those plus **semantic** checks and exits non-zero, so a
commit can be gated on it.

**Structural** — is this a possible machine?

1. frame brackets do not overlap illegally or exceed the tape length
2. an open cell's slot is within the tape of the fiber it names
3. no pop from an empty tape; every `frame_pop` matches a prior `frame_push`
4. switch phases arrive in `take → hole → install` order, never interleaved
5. no switch while `native_reentry_depth > 0`
6. `ip` is within the current chunk; every named chunk exists; every line is within the source
7. a `framePush` starts the callee at `ip 0`

**Semantic** — is this still the machine we meant to show?

8. **`TWO-FULL-CARDS`** — at every hole other than a fiber's first resume, both fibers are parked and
   the VM is empty
9. **`DEAD-HOME`** — if a home token is captured, it must actually go stale somewhere
10. a hole leaves no tape and no frames behind

Check 5 is the language restriction encoded; a trace violating it draws an illegal machine.

Checks 8–10 exist because structural validity is not the real risk. A trace can be entirely
well-formed and still have quietly stopped teaching anything — the two-full-cards frame is the whole
reason the tool exists, and losing it produces no error, no visual glitch, and no reason to look.
Check 7 is there because the same authoring mistake recurred three times; it is silently in-bounds
whenever the callee's chunk is longer than the caller's ip.

> **The rule this encodes:** when a rendering carries a lesson, assert the lesson, not just the
> well-formedness. Anything a reviewer would notice only by remembering to look, a check should notice
> instead.

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

That claim has now been tested once, and held: E4 and E5 were filed here as later tenants and both
landed **without a new panel** — E4 needed a label on an existing bracket, E5 an attribute on an
existing edge (§9). Both are collision examples, which is the class this design most wants and the
class that turns out to be cheapest.

---

## 12 · Build order

| | | |
|---|---|---|
| 1 | Trace format, fold reducer, structural checks | **done** |
| 2 | Tape with brackets and delta marking — **the D1 go/no-go** | **done**, `tools/viz/prototype-tape.html` |
| 3 | Host gutter, fiber rail, transport, keys | **done** |
| 4 | The three-beat switch (C1) | **done** as cursor stops; no tween (§8) |
| 5 | Cell strip — the stretch-into-locker moment | **done**, text + drawn connector (§6) |
| 6 | Page shell, gates, prose, E1 → E2 → E3 | **done** |
| 7 | E4 dead frame, E5 call vs try, semantic checks | **done** |
| 11 | Triptych renderer (C3) | **done**, `tools/viz/triptych.html` |
| 8 | Transport lock (R-GATE) | **done** |
| 9 | Tape tween on park/unpark (C1 spatial slide) | **done** |
| 10 | Drawn connector for upvalue cells | **done** |

Step 2 was the go/no-go: if the flat tape with bracket lanes had not read at a glance, D1 was wrong and
everything downstream needed rethinking. It was built and judged first, against T1, before any
satellite existed to prop it up — **and it passed**, which is why the rest of this document still
stands.

## 13 · What the build changed about the spec

Recorded because a spec that quietly absorbs its own mistakes is worth less than one that keeps them.

| Spec said | Build found | Now |
|---|---|---|
| bytecode pane is a later satellite | slot indices become unexplained numbers without it | §3.0 — it is half the R-LINK fact |
| 28 px slots, index every 4th | `<Counter>` does not fit; naming every slot is better | §3.1 |
| suppress bracket metadata when narrow | E4's tape is 2 slots wide; the rule hid the lesson | §3.2 — degrade, never vanish |
| `ip` is VM state | it is a `CallFrame` field, so it parks | engine; §8 "no current instruction" |
| the hole shows two full cards | only true after a fiber's *first* resume | §8, and `TWO-FULL-CARDS` allows the exception |
| gate locks the player | not built at first cut; built 2026-07-20 (`go()` clamp + disabled controls) | §9 |

The through-line: **every one of these was found by building or by a check, not by re-reading.** The
two that would have survived review indefinitely — `ip` placement and the one-full-card hole — were
both caught by `check.mjs`, which is the argument for §10's semantic layer.
