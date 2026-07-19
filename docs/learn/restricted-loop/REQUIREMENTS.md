# REQUIREMENTS — C1, "The restricted loop"

Phase 2 of [AUTHORING.md](../AUTHORING.md). Target doc ships to
`docs/learn/concurrency/restricted-loop.md`. Plan: [CONCURRENCY-PLAN.md](../CONCURRENCY-PLAN.md) §3.
Grounding: [recon.md](recon.md) — **read it before this file**; every section here is downstream of
its five findings.

---

## 1. The obligation

> After reading, the reader can re-derive *why* Phalcom's fibers cannot yield from inside
> `list.each { }` — from the constraints alone, without being told the rule.

Stronger form, which is the real test: hand the reader (a) the fact that a switch is a buffer swap,
and (b) the fact that a nested native call holds an index into those buffers, and they should
predict the restriction before it is stated. If they can only *recall* the rule, the doc failed.

## 2. The reader

Per [AUTHORING §9](../AUTHORING.md#9-reference-the-standing-method): knows PL design, not fluent in
implementation; cannot hold moving-state mechanisms in their head without stable notation. They have
read the VM track. They have almost certainly hit "cannot yield across a C-call boundary" in Lua, or
function coloring in JS, without having a model for *why* such a rule exists at all.

## 3. Doc kind — **fork**, and unusually a real one

Per recon §3, ADR-0030 records four genuinely rejected branches with bills. **This is the first doc
in the course whose design space is not a pedagogical reconstruction.**

> **Structural rule, non-negotiable:** do **not** carry over the VM track's standing caveat that the
> design-space walk is reconstruction. Here it would be a false statement. Say the opposite,
> explicitly and once: this space was deliberated, and here is the record.

## 4. The grip (copied from [recon §2](recon.md), grounded)

> **A fiber switch is not a jump, and not a scheduler decision. It is `mem::take` on four VM fields —
> and the dispatch loop is never told it happened. That is why a switch costs O(1); it is also
> exactly why a switch is illegal whenever a native `run_until` is sitting on the Rust stack holding
> an index into the buffers about to be swapped.**

The doc's spine is that **the cheapness and the restriction are one fact, not two.** Every section
should be placeable on that sentence. If a section cannot be, cut it.

## 5. The design space

| Branch | Occupant | Buys | Costs / forecloses | Weight |
|---|---|---|---|---|
| **A — restricted re-entrant loop** | Lua 5.1, Phalcom | tiny VM change; no collector rewrite; O(1) switch | the callback generator (`yield` under a native combinator) | **heavy — it is the doc** |
| **B — full trampoline** | Lua 5.2+ (via `lua_yieldk`), CPS interpreters | yield anywhere | rewrite of the whole primitive/callback protocol | **heavy — the additivity claim is the decision's spine** |
| **C — stackful coroutines** | Go, Ruby `Fiber`, `corosensei` | yield across native frames, no protocol change | `unsafe` stack switching; **permanently constrains the GC** — every parked native stack is a root a moving collector must scan and relocate | **heavy — the bill is GC-shaped and that is the point** |
| **Preemptive / OS threads** | Go, JVM | true parallelism | a memory model + locks throughout the object model | one paragraph — the outer boundary |
| **Resumable (Smalltalk) suspension** | Smalltalk | resume a failed computation | orthogonal; ADR-0008 propagation is terminating | one sentence |

**The asymmetry is the argument:** A→B is purely additive; A→C is irreversible. A doc that walks
five branches evenly has missed the decision. Expect to cut ~30% from the walk (AUTHORING §5.5).

## 6. Comparison filter — the cast, and the cuts

| Language | Test passed | Job in this doc |
|---|---|---|
| **Lua 5.1** | (4) ancestor — *named in ADR-0030 itself* | Same branch, and **the same error message**: `attempt to yield across a C-call boundary`. The reader may have hit it personally. |
| **Lua 5.2/5.3** | (2) scar → shipped fix | The A→B lift, **actually shipped by the ancestor**: `lua_yieldk` + continuations. This is the highest-value comparison in the doc — it makes "additively reachable" concrete instead of aspirational. |
| **Go** | (1) other branch, bill attached | Branch C+preemption: real stacks, growable, a scheduler. Bill: `unsafe`-equivalent machinery, a race detector, and a GC that must cope with many stacks. |
| **JS** | (3) **names what Phalcom does anonymously** | *Function coloring.* Phalcom's restriction is a coloring rule that the language never gives a name to; JS's `async`/`await` supplies the vocabulary. Highest-value-by-filter-rank. |
| **Wren** | (1) same branch, and *our own evidence* | The `concurrency_fiber_wren_*` fixture family means Phalcom validated its semantics against Wren directly — rare first-party evidence, not borrowed anecdote. |

**Cut, and say so in the doc:** Erlang (processes are a different unit of isolation — a whole other
argument), Ruby `Fiber` (branch C, but Go already carries that bill better), Python `asyncio` (JS
already makes the coloring point), C#/Kotlin `async`/`suspend` (same, and coloring is JS's job here).

## 7. Tensions to surface

1. **Cheapness ⊗ restriction.** The spine. One representational choice buys both.
2. **The optimizer sets the language's rules.** `while` is legal to yield from and `each` is not —
   and the difference is *entirely* whether ADR-0018's inliner lowered it inside one chunk. A
   user-visible language restriction falling out of an optimizer decision is the doc's sharpest
   point. Cite Doc 5; do not re-teach the inliner.
3. **The guard is stricter than the spec, knowingly** (recon F3). ADR §4 forecloses yield; HEAD also
   forecloses resume, documented as "a deliberately wider, sound over-restriction."
4. **A guard general beyond the machine that runs it** (recon F4). `floor_depth` is provably always
   `0` at HEAD, so the relative check is currently equivalent to `!= 0`. Interesting *because* it is
   written for the narrower world F3 might someday restore — but it must be stated as currently
   equivalent, never as a live two-case distinction.

## 8. Structural rules

- **Must pay, by name:** Doc 4 ([message-send.md](../vm/message-send.md)) *Lie #2*'s
  `switch_pending` branch — "a fiber switch firing inside a primitive," explicitly deferred to "the
  concurrency doc." Quote `call_method`'s three-way `Primitive` arm and destroy the lie outright.
- **Must not restate — forbidden list** (highest-overlap risk; handle as Doc 6 handled
  `upvalues.md`):
  - Doc 1 ([execution-loop.md](../vm/execution-loop.md)) §"One honest paragraph about fibers"
    already explains why the hoist guards on `closure_id` and not `ip`. **Cite it as paid; do not
    re-derive it.** C1 may add only the half Doc 1 could not: *why* the swap is wholesale.
  - Doc 5 ([caches-and-fusion.md](../vm/caches-and-fusion.md)) owns the inliner. Cite for tension 2.
  - Doc 3 ([frames.md](../vm/frames.md)) owns `CallFrame`-as-a-value.
  - **C2 owns** the four-field `mem::take` *as mechanism* — what each field is, why
    `next_frame_generation` stays VM-global, fiber-stack pooling, GC rooting. C1 uses the swap as a
    **fact with one quoted line**, and forward-points. Do not pre-empt C2.
  - **C3 owns** failure: `capture_error_value`, the `Call`/`Try` cascade, the fiber floor's
    teardown. C1 may name `CannotYieldAcrossNativeFrame` as a *catchable* error and stop.
- **Anchors symbol-first:** `file.rs::Type::method` @ ~Lxxx.
- **Mark every lie** with a forward pointer.

## 9. The predict-then-check moment (mandatory, AUTHORING §5.4)

Placed **before** the restriction is stated. The material is unusually good and already verified
live (recon F5):

> Two fiber bodies. One is legal; one raises. Which — and why?
> ```phalcom
> Fiber.new { var n = 0; while (true) { Fiber.yield(n); n = n + 1 } }
> Fiber.new { list.each { x => Fiber.yield(x) } }
> ```

The reader has, by that point, been given the swap and the held index. That is sufficient to derive
the answer. Reveal only after.

> **Transcription rule (recon F5):** use `var n`, not the ADR's `let n`. The ADR predates the
> `let`/`var` split and its snippet **does not compile at HEAD** — verified: `Cannot reassign
> immutable 'let' binding 'n'`. Copying the ADR verbatim would ship a broken program in a doc whose
> selling point is that it is grounded.

## 10. The hard trace (AUTHORING §5.5)

Trace the **illegal** program, not the legal one. Follow `list.each { x => Fiber.yield(x) }` down:
`each` → `block_call` → `native_reentry_depth += 1` → re-entrant `run_until(base_frames)` → the
`Fiber.yield` primitive finding the depth changed → raise. The legal `while` case is the reader's
intuition already working and earns at most a sentence of contrast.

Show what *would* break if the guard were absent: `base_frames` is an index into a vector that no
longer belongs to this fiber. Name the corruption concretely — that is what makes the guard
inevitable rather than arbitrary.

## 11. Claims ledger discipline (AUTHORING §5.3)

- "O(1) switch" — a structural claim (four `mem::take`s, no per-element work). State it structurally.
  **No timing number is available and none may be implied.** If any cost language creeps in, label
  it **unmeasured** — `perf-log/SCOREBOARD.md` is the only source of numbers in this repo.
- The ADR's *reasons* for rejecting C (GC constraint) are the ADR's, and are cited as such — not
  restated as a measured fact.
- Recon F2: never quote ADR §5's "`ControlFlow` value out of the primitive" as describing HEAD.
- Recon F4: `floor_depth`'s always-zero status is a **read-verified** claim; state it as verified.
- Per plan §7: `vm-trace` is `LevelFilter::OFF` and `disasm` walks only the top-level chunk, so any
  dynamic/frame-level claim not backed by observed program output must be labelled **INFERRED**.
- Every markdown link resolves to a file or real anchor.

## 12. Build sequence

1. Reconcile A vs B (AUTHORING §5.1) — expect A to be strongest on Lua/coloring history and to have
   no idea what Phalcom's signal is. Expect the table to have rows.
2. Honesty pass on F2/F3/F5 — three separate ADR-vs-HEAD gaps; none may be smoothed over.
3. Draft against §8's forbidden list open beside you.
4. Insert §9 before the reveal; write §10's trace from B's observed output.
5. Cut the design-space walk to §5's weights.
6. Gate (AUTHORING §6). No box unchecked.

## 13. Open risks

| Risk | If wrong, the doc… |
|---|---|
| Recon F4 (`floor_depth` always 0) rests on one dominance argument in one function. | …asserts dead generality that is actually live. **Agent B must adversarially re-check every writer** (brief item 2). If B finds a nonzero path, tension 4 is deleted, not softened. |
| The inliner claim (`while` frameless, `each` through `block_call`) is quoted from ADR-0018 and comments, not yet observed. | …rests the doc's sharpest tension on an unverified mechanism. B must confirm mechanically or the claim ships **INFERRED**. |
| `ready_queue` / `System.schedule` / the root-drive pump sit inside `run_until` and are visible while reading it. | …drifts into C4's subject and doubles in length. Named in B's brief as a scope check: report, do not expand. |
| Overlap with C2 on the swap. | …spends C2's subject. §8 forbidden list is the control; re-check at the gate. |
| ADR-0030 is one document doing double duty (fibers *and* futures), and futures are a quarter built. | …repeats spec text as shipped. Plan §7 predicts this is the track's recurring honesty note. Verify each §-citation against HEAD before quoting. |
