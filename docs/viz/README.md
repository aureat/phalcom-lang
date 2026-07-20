# Execution visualizer

A replay player for Phalcom execution — fibers, frames, the value stack, locals and upvalues, and
how they collide. Built so the runtime can be understood at the level where design decisions live,
not at tutorial level.

## The files

| | |
|---|---|
| [`REQUIREMENTS.md`](REQUIREMENTS.md) | The operating manual. Tests T1–T5, prohibitions N1–N7, requirements R-*, ranked tiebreakers. **Read first** — every design decision resolves against it. |
| [`UI-SPEC.md`](UI-SPEC.md) | The visual design, with the justification and the rejected alternative for each choice. |
| [`../../tools/viz/index.html`](../../tools/viz/index.html) | The player. Open it directly — no build step. |
| [`../../tools/viz/trace-data.js`](../../tools/viz/trace-data.js) | `EXAMPLES` and the trace-application engine — DOM-independent, shared verbatim by `index.html`, `triptych.html`, and `check.mjs`. |
| [`../../tools/viz/triptych.html`](../../tools/viz/triptych.html) | Frozen before/hole/after figure for a `docs/learn` embed. `?ex=&var=&switch=&mark=`; same trace data, no build step. |
| [`../../tools/viz/check.mjs`](../../tools/viz/check.mjs) | Trace validator. `node tools/viz/check.mjs` |
| [`../../tools/viz/prototype-tape.html`](../../tools/viz/prototype-tape.html) | The D1 go/no-go prototype. Kept as the record of why the tape reads the way it does. |

## Running it

```sh
open tools/viz/index.html      # the player
node tools/viz/check.mjs       # validate every trace, exit 1 on any problem
```

## The five examples

The first three are mechanisms. The last two sit on **feature collisions**, which is where language
design actually lives and where most real bugs come from.

| | Teaches | Collision |
|---|---|---|
| **E1 · Ping-pong** | What a switch *is*: `mem::take` on four fields, take → **hole** → install. Also that module-level `let` is a global, not a slot. | — |
| **E2 · Upvalue across a park** | Why `Upvalue::Open` names a `{ fiber, slot }`. Cells are heap objects and do not travel — the tape moves out from under them. | closures ⊗ fibers |
| **E3 · Legal vs illegal yield** | `native_reentry_depth == 0`. Two loops that look equally reasonable; the only visible difference is the host gutter. | optimiser ⊗ user-visible semantics |
| **E4 · Dead frame** | A block outlives its home and returns through it. The escaping block lands at the *same frame index* its home had — only the generation tells them apart. | closures ⊗ frame lifetime |
| **E5 · call vs try** | The same failing fiber resumed two ways. `FiberResumeMode` is an edge property, so containment is the caller's decision. | errors ⊗ concurrency |

Keys: `←` `→` step · `shift`+`←` `→` jump to the next switch · `space` play/pause · `1`–`5` example ·
`Home`/`End` · `t` theme.

## The fidelity rule

Traces are **hand-authored**, and a hand-authored trace can be silently wrong — an off-by-one
`stack_offset` draws a confident lie, which is worse than no picture (REQUIREMENTS §8, failure mode 1).
Two defences, and both must stay:

1. **`check.mjs` runs structural *and* semantic checks**, and has caught seven real bugs so far.
   The semantic layer is the important half: a trace can be perfectly well-formed and have quietly
   stopped teaching anything, with no error and no visual glitch to prompt a second look.
   - `TWO-FULL-CARDS` — at every hole other than a fiber's first resume, both fibers parked, VM empty
   - `DEAD-HOME` — a captured home token must actually go stale somewhere
   - a `framePush` must start the callee at `ip 0` (this authoring slip recurred three times)

   The rule: **assert the lesson, not just the well-formedness.**
2. **The fidelity note is rendered in the page itself**, naming what is verified and what is inferred.

What is verified: the opcode vocabulary (`bytecode.rs`), module-level sequences (checked with
`phalcom disasm`), `stack_offset + slot` (`vm/dispatch.rs:721`), the four moved fields,
`Upvalue::Open { fiber, slot }` (`heap/upvalue.rs:34`), `native_reentry_depth == 0` (`vm/mod.rs:97`).

What is **inferred**: block and method chunk *sequences*. `disasm` walks only the top-level chunk, so
nested chunks were reconstructed by hand from the real opcode set.

> **Highest-value follow-up:** make `disasm` recurse into nested chunks (~20 lines in
> `bin/phalcom/disasm.rs`). It would let every trace be checked against real output instead of
> reconstructed, and the concurrency track's docs need the same tool — `CONCURRENCY-PLAN.md` §7
> lists "no live tracing" as an open risk against a track whose subject is *switching*.

Any future emitter must copy `phalcom-core/src/opcode_stats.rs` — feature-gated, thread-local, dumped
at exit. **Not** `tracing`: `vm-trace` is double-gated and its subscriber is hardcoded to
`LevelFilter::OFF`, so it emits nothing even when compiled in.

## Not built yet

The player covers execution only. Deliberately absent, each a later tenant on the same timeline
rather than a v1 compromise: dispatch and selector identity, inline caches and `world_version`
invalidation, sacred-selector deopt, GC and parked-fibers-as-roots, the metaclass tower (structural —
wrong renderer), compilation (needs an AST pane).

The four smaller, additive deferrals this section used to list — the triptych renderer, a drawn
connector for upvalue cells, a tape tween on park/unpark, and locking the transport until a gate is
answered — are all built now (see the build-order table in [`UI-SPEC.md`](UI-SPEC.md#12--build-order)).
