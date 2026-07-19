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
| [`../../tools/viz/check.mjs`](../../tools/viz/check.mjs) | Trace validator. `node tools/viz/check.mjs` |
| [`../../tools/viz/prototype-tape.html`](../../tools/viz/prototype-tape.html) | The D1 go/no-go prototype. Kept as the record of why the tape reads the way it does. |

## Running it

```sh
open tools/viz/index.html      # the player
node tools/viz/check.mjs       # validate every trace, exit 1 on any problem
```

## The three examples

| | Teaches |
|---|---|
| **E1 · Ping-pong** | What a switch *is*: `mem::take` on four fields, take → **hole** → install. Also that module-level `let` is a global, not a slot. |
| **E2 · Upvalue across a park** | Why `Upvalue::Open` names a `{ fiber, slot }`. Cells are heap objects and do not travel — the tape moves out from under them. |
| **E3 · Legal vs illegal yield** | `native_reentry_depth == 0`. Two loops that look equally reasonable; the only visible difference is the host gutter. |

## The fidelity rule

Traces are **hand-authored**, and a hand-authored trace can be silently wrong — an off-by-one
`stack_offset` draws a confident lie, which is worse than no picture (REQUIREMENTS §8, failure mode 1).
Two defences, and both must stay:

1. **`check.mjs` runs structural *and* semantic checks.** The load-bearing semantic one is
   `TWO-FULL-CARDS`: at every hole other than a fiber's first resume, both fibers must be parked and
   the VM empty. That frame is the entire reason the tool exists. Structural invariants cannot catch
   its loss; this can. It has already caught five real bugs.
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

Two examples are specced but unbuilt, and they are where this stops being a stack visualizer and
starts being an argument about language design: **E4 `DeadFrameError`** (closures ⊗ frame lifetime)
and **E5 `call` vs `try`** (errors ⊗ concurrency). Both run on the existing panels.
