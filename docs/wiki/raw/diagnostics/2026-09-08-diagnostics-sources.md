# Raw source snapshot: diagnostics

> Captured: 2026-09-08
> Source area: traceback/rendering specifications and DIAG001 program boundary

--- docs/spec/current/traceback/index.md ---
# Traceback & diagnostic rendering

Specification for how Phalcom reports a failure to a human: runtime tracebacks, compile and
syntax diagnostics, execution tracing, and disassembly.

- [`u22-seq-spec.md`](implementation-spec.md) — the dispatch-ready implementation spec
  (renderer ruling, walk/styler architecture, capture, native/@native frames, style guide,
  did-you-mean, observability).
- [`err-plan.md`](plan.md) — dependency-ordered units with write-sets, edges, and gates.
- [`verification-2026-07-20.md`](verification-2026-07-20.md) — adversarial re-verification of
  the audit; lists every claim that did not survive.
- [`output-catalog.md`](output-catalog.md) — every rendering surface, by example.
- [`color.md`](color.md) — the color scheme: semantic roles, palette discipline, per-surface use.
- [`../../deferred/tracing.md`](../../deferred/tracing.md) — the U-TRACE audit and continuation
  prompt: what exists, what is unwired, what is broken.
- [`../../deferred/error-handling-followups.md`](../../deferred/error-handling-followups.md) —
  unowned error-handling defects found alongside.

**Status:** specified — implementation not started. §3.1 (renderer) is **ruled** —
[PDR-0014](../../pdr/0014-diagnostics-renderer-is-in-house.md), Accepted. The capture row
in §2 below is **superseded by
[PDR-0010](../../pdr/0010-errors-carry-structure-and-cheap-origin.md) §3, ratified
2026-07-20**; the normative `kind` table is
[`u22-seq-spec.md`](implementation-spec.md) §8.1. Parts of `tracing.md` went stale when
PDR-0008 landed; trust [`verification-2026-07-20.md`](verification-2026-07-20.md) over it where
they disagree.

---

## 1. Scope

One primitive with several consumers:

```
              ┌── traceback renderer (human)
StackWalk ────┼── traceback renderer (json)
              ├── fiber switch log
              └── REPL `where`          (deferred — see catalog §8)

Chunk walk ───── disassembler
```

The walk is the primitive. Every formatter is a consumer. Building the formatter *as* the
primitive is the failure mode — it forces a rewrite the moment a second consumer appears.

Not in scope: `Error.stackTrace` surface reflection (backlogged; rides on the compact capture
record), a debugger protocol, profiler integration.

---

## 2. Decisions locked

Carried from the U-TRACE design session. Do not re-litigate.

| Decision | Ruling |
|---|---|
| Frame ordering | Python's — most-recent-call-last, error at the bottom |
| Caret block | Innermost frame only, never all frames |
| Core frames | Elided by default with a count; `--trace-core` expands |
| Fiber boundary | Traceback **chains** across the floor with a spawn-site link; does not stop |
| Primitive shape | Walkable live stack object; formatter is a consumer |
| Capture timing | ~~Compact record at **raise**~~ **Superseded by PDR-0010 §3 (ratified 2026-07-20)**: capture at the first `on` boundary / per-hop in the fiber cascade; record holds Symbols + line, never `ObjRef`s (PDR-0010 §4) |
| Frame granularity | **Logical** frames, 1:many expansion from day one |
| Trace stability | Golden fixtures assert fields via JSON stream; human layout explicitly unstable |
| Fiber switch log | No `cfg` gate — cold path |

Two of these carry non-obvious rationale worth restating:

**Capture at raise, resolve at render.** `.attempt()` and `on(_)` handlers already exist
(`core.ph:1427`). Walk-time-only capture loses the origin of every *caught* error, because the
frames are gone by the time anyone asks. Capturing compact — ids and offsets, no strings, no
source resolution — is cheap enough to leave always-on, and it removes the
`module_source.unwrap()` panic by construction: there is no source to unwrap at capture time.

**Logical frames from day one.** There is no inlining and no TCO in the tree today, which is
exactly why the API is free to model 1:many now and expensive to retrofit later. V8 and the JVM
both retrofitted inline-frame expansion and both found it invasive. Precedent that the concern is
real here: superinstruction fusion already had to solve span-fidelity-under-transformation
([dispatch.rs:538-543](../../../phalcom-core/src/vm/dispatch.rs)).

---

## 3. Open decisions

### 3.1 Renderer — miette or color-print  ✅ RULED

**Ruled 2026-07-20: option (b), extended with a named style layer; miette leaves the
workspace.** Full grounds in [`u22-seq-spec.md`](implementation-spec.md) §1 — the short
version: most surfaces (frame lines, fiber log, disasm, JSON) are not miette-shaped, so (a)
still means two renderers; `color.md`'s palette discipline is easier to own than to impose on
miette; and the genuinely hard part (multi-label spans) is bounded at two labels. The catalog's
`╭─ │ · ╰──` style is kept and implemented in-house. Original analysis preserved below.

- `miette` is a declared workspace dependency and **nothing imports it**. Zero `use miette` /
  `miette::` across the repo. `CLAUDE.md` names "thiserror + miette" as the convention;
  that half of the convention is aspirational.
- The incumbent is `color_print::ceprintln` with inline markup
  ([diagnostics.rs:2](../../../phalcom-core/src/diagnostics.rs)), driving hand-rolled
  `print_line_information`.

The catalog's `╭─ │ · ╰──` look is miette's house style. Two paths:

- **(a) Wire miette for real.** Get multi-label spans, help/note slots, and severity for free.
  Cost: changes how every diagnostic prints, moves the entire negative-fixture corpus, and adds
  a second color system alongside `color-print`.
- **(b) Extend `print_line_information`.** Keeps one color system and one corpus. Cost: hand-roll
  multi-label spans, which is the part of miette that is genuinely hard.

Any prior doc or decision asserting diagnostics are "rendered as miette labels" is describing
something that does not exist. Decision 0066 did, and was amended for it (`bb4f365`).

### 3.2 Hint provenance

Catalog §2 shows `receiver is None — \`first\` on an empty List` — a hint explaining *why* the
receiver is `None`, not merely that it is. That requires the VM to know which expression produced
the receiver and reason about it. Specify what class of hint is derivable from spans plus the
opcode alone, and what would need dataflow the compiler does not currently keep. Do not promise
the hard ones in v1.

### 3.3 Remaining opens carried from `tracing.md`

- Does U-TRACE get its own ADR, or ride an existing one? If new: flip `docs/adr/STATUS.md` the
  same pass.
- Which paths leave `ModuleObject::source` as `None` (REPL? `-i` inline? core.ph bootstrap?) —
  determines where source echo degrades to bare `file:line`.
- Does the cross-fiber spawn-site link collide with U-FIBER's floor-capture ownership (DEC-FIB-A)?
- `spans` is 16 bytes/instruction, one per instruction. Record as named debt with a migration
  shape and consume it through an accessor, not `spans[ip]` at N call sites.

---

--- docs/spec/current/traceback/color.md ---
# Color scheme

How diagnostics use color. Companion to [`README.md`](README.md) §4.1, which covers the
mechanics (`--color`, `NO_COLOR`, TTY detection); this document covers what gets colored.

**Status:** design target. Gated on README §3.1 — the renderer decision (miette vs the incumbent
`color_print`) determines how this is expressed, not what it says.

---

## 1. The rule that governs everything else

**Color is emphasis, never information.**

Every distinction a reader needs must survive `--color=never`. Structure is carried by glyphs and
position — the `×` marker, the box rails, the caret, the `help:` prefix, indentation depth. Color
makes a correct rendering faster to scan; it never makes an uncolored one ambiguous.

Three reasons this is not negotiable:

- Roughly 8% of men have some form of red-green color deficiency. An error/success distinction
  carried only by hue is invisible to them.
- Output is piped into files, CI logs, and test fixtures constantly. Those are the uncolored path.
- Terminal themes vary enormously. A color that reads as "alarming red" in one theme is muddy
  brown in another.

Test for this: render every catalog example with color off and confirm nothing is lost. If
something is, the glyph layer is wrong — fix that, do not reach for a second color.

---

## 2. Semantic roles

Colors attach to **roles**, not to literal elements. A role gets one color everywhere it appears,
across tracebacks, compile errors, trace logs, and disassembly. This is what makes the whole tool
look like one program.

| Role | Applies to | ANSI | Weight |
|---|---|---|---|
| `severity.error` | `error:`, `×` marker | red | bold |
| `severity.warn` | `warning:` | yellow | bold |
| `severity.help` | `help:`, `note:` prefixes | cyan | bold |
| `location` | `shop.ph:3:48`, frame `file:line` | blue | — |
| `identifier` | frame names, selectors, class names | default | bold |
| `rail` | box drawing, gutters, `│ ╭ ╰ ·` | dim default | — |
| `line-number` | the ` 3 │` gutter number | dim default | — |
| `source` | the echoed source line | default | — |
| `span.primary` | the underline under the failing span | red | bold |
| `span.secondary` | a second label's underline | blue | — |
| `label` | text hanging off a caret | matches its span | — |
| `elision` | `[2 core frames elided …]` | dim default | italic |
| `chain` | `⤷ raised inside fiber #3 …` | magenta | — |

### Notes on specific choices

**`source` stays default.** The strongest instinct is to syntax-highlight the echoed line. Don't.
The span underline is the only thing that should draw the eye there; highlighting the whole line
competes with it and makes the actual error harder to find. The REPL highlights source because
the user is *writing* it — a diagnostic echoes source because the user is *locating* something in
it. Different jobs.

**`span.primary` is red and `span.secondary` is blue** — not two shades of red. Under
deuteranopia those would collapse; red/blue survives. The primary is also bold, so the ranking
holds with color off.

**`chain` is magenta** because a fiber boundary is genuinely a different kind of event from both a
frame and an error, and reusing either color makes it read as a subheading of the wrong thing.

**`elision` is dim and italic** so skipped content recedes without vanishing. A reader scanning
for their own code should skim past elided core frames; a reader debugging core should still spot
the line telling them how to expand it.

---

## 3. Palette discipline

**Use the 16 ANSI indices. Not 256, not truecolor.**

The indices are what the user's terminal theme remaps. Emitting `#d75f5f` overrides Solarized,
Nord, Gruvbox, and every high-contrast accessibility theme with one author's taste. Emitting
"index 1, bold" lets each of those render *their* red. The output looks native everywhere instead
of correct in one place.

Practical consequences:

- Never emit pure white or pure black. Both are unreadable on one of the two common backgrounds.
  "Default foreground" is the color that adapts.
- No background fills. They fight every theme and break selection and copy-paste.
- "Dim" means the SGR dim attribute, not a darker color. It composes with the user's theme.
- Bold is a real signal here and should be spent sparingly — severity, primary span, identifiers.
  Bold everywhere is bold nowhere.

---

## 4. Worked example

Catalog §1, annotated with roles:

```
Traceback (most recent call last):          ← severity.error (bold red), header only
  shop.ph:7   in <main>                     ← location (blue) + identifier (bold)
      cart.total                            ← source (default)
  shop.ph:2   in Cart.total
      total { self.sum(_items) }
  shop.ph:3   in Cart.sum(_)
      sum(items) { items.fold(initial: 0, using: { acc, it => acc + it.price }) }
  [2 core frames elided — pass --trace-core to expand]    ← elision (dim italic)
  shop.ph:3   in <block in Cart.sum(_)>

  × 1 does not understand 'price'           ← × is severity.error; 'price' is identifier
   ╭─[shop.ph:3:48]                         ← rail (dim) + location (blue)
 3 │   sum(items) { items.fold(initial: 0, using: { acc, it => acc + it.price }) }
   ·                                                ─────┬────   ← span.primary (bold red)
   ·                                                     ╰── Number has no method 'price'
   ╰────                                                          ← label (red, matches span)
  help: did you mean 'floor'?               ← severity.help (bold cyan) + identifier
```

With `--color=never` this is the same text, unchanged. Nothing above depends on hue.

---

## 5. Per-surface application

**Traceback** — as above. The innermost frame's caret block is the only place `span.*` appears;
outer frames are location + identifier + source only.

**Fiber switch log** — `[fiber]` prefix in `chain` magenta so trace lines are separable from
program stdout at a glance. Fiber ids in `identifier` bold. `spawn`/`switch`/`yield`/`done` in
default; `fail` in `severity.error`.

--- docs/spec/current/traceback/output-catalog.md ---
# Traceback & diagnostic output catalog

Every user-visible rendering surface U-TRACE is responsible for, specified by example.

**Status:** design target, not implemented. See [`README.md`](README.md) for scope, locked
decisions, and the open decisions that gate several of these renderings.

**These are targets, not fixtures.** Golden tests assert *structure* — frame sequence, event
fields — never byte-exact layout. See README §"Testability without freezing the format".

---

## 1. Runtime traceback — the base case

Source `shop.ph`:

```phalcom
1  class Cart {
2    total { self.sum(_items) }
3    sum(items) { items.fold(initial: 0, using: { acc, it => acc + it.negatd }) }
4  }
5
6  let cart = Cart.new()
7  cart.total
```

Rendering:

```
Traceback (most recent call last):
  shop.ph:7   in <main>
      cart.total
  shop.ph:2   in Cart.total
      total { self.sum(_items) }
  shop.ph:3   in Cart.sum(_)
      sum(items) { items.fold(initial: 0, using: { acc, it => acc + it.negatd }) }
  [2 core frames elided — pass --trace-core to expand]
  shop.ph:3   in <block in Cart.sum(_)>

  × 1 does not understand 'negatd'
   ╭─[shop.ph:3:48]
 3 │   sum(items) { items.fold(initial: 0, using: { acc, it => acc + it.negatd }) }
   ·                                                ─────┬────
   ·                                                     ╰── Number has no method 'negatd'
   ╰────
  help: did you mean 'negated'?
```

Rules this encodes:

- **Ordering is Python's**: most-recent-call-last. Error message sits at the bottom, adjacent
  to the innermost frame, because that is where the terminal leaves the reader's eye.
- **Frame line format**: `  <module>:<line>   in <name>` then the source line indented under it.
  Two lines per frame, no box drawing.
- **Frame names**: `<main>` for module top level, `Cart.total` for a nullary method,
  `Cart.sum(_)` for arity-1 (selector shape, not a bare name — `foo` and `foo(_)` are different
  methods and the trace must distinguish them), `<block in Cart.sum(_)>` for a block, naming its
  enclosing method.
- **Core frames elide by default** with a count and the flag that expands them. A reader
  debugging `shop.ph` does not want `List.fold`'s internals. Ruby and Rust both do this; Python
  does not and its tracebacks are worse for it.
- **Only the innermost frame gets the caret block.** A forty-frame caret render is unreadable.
  Everything above is the cheap two-line form.
- **`help:` is optional** and appears only when a suggestion clears the confidence threshold
  (README §"did-you-mean").

---

## 2. Runtime traceback — across a fiber boundary

Source `job.ph`:

```phalcom
1  let worker = Fiber.new {
2    let rows = load()
3    rows.first.parse()
4  }
5  worker.call()
```

Rendering:

```
Traceback (most recent call last):
  job.ph:5   in <main>
      worker.call()

  ⤷ raised inside fiber #3, spawned at job.ph:1

Traceback (most recent call last):
  job.ph:3   in <block in <main>>
      rows.first.parse()

  × None does not understand 'parse'
   ╭─[job.ph:3:14]
 3 │   rows.first.parse()
   ·              ──┬──
   ·                ╰── receiver is None — `first` on an empty List
   ╰────
```

Rules this encodes:

- **The chain crosses the fiber floor.** It does not stop there. Precedent is Python 3's
  `__context__`/`__cause__` chaining, which prints a second traceback under a linking sentence.
- **The link line carries the fiber id and its spawn site**, so the reader can find where the
  fiber came from without tracing on.
- **Each side is a complete traceback** with its own header. Only the innermost side carries the
  caret block and message.
- **This is the fibers-track payoff**: a fiber switch becomes visible *in the error itself*, with
  no tracing flag set. Today it is invisible at every level.
- **N-deep chains** repeat the link line. See README §"Deep and repetitive stacks" for the
  truncation rule once a chain exceeds the frame budget.

The label on the caret (`receiver is None — \`first\` on an empty List`) is a **second-order
hint**: it explains why the receiver is `None`, not merely that it is. **Aspirational — not v1.**
`None` is immediate, so origin-tracking would require a separate provenance wrapper; v1 renders the first-order
`receiver is None` only. See [`implementation-spec.md`](implementation-spec.md) §10 (hint
provenance classes A/B/C).

---

## 3. Disassembly — recursive

Today `disasm` prints only the top-level chunk; nested closures sit in `constants` as
`Value::Obj` and render as `{:?}` handles, so every method and block body is invisible
([disasm.rs:11](../../../phalcom-core/bin/phalcom/disasm.rs)).

Target, same `shop.ph`:


--- docs/implementation/DIAG001-result-error-surface/PROGRAM.md ---
---
id: DIAG001
category: DIAG
kind: completion-and-correction
status: IN_PROGRESS
completion: PARTIAL
verification: UNVERIFIED
---

# DIAG001 — result error surface

This program owns result/error implementation and traceback diagnostics,
including corrective and historical records.
