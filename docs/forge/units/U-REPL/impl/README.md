# U-REPL — implementation spec (branch-isolated)

Companion to [`../plan.md`](../plan.md) (§D-series, evaluation substrate) and
[`../surface.md`](../surface.md) (§S-series, what the user sees). Those two say **what**
and **why**, and neither is superseded here. This directory says **where, in what order,
and on which branch** — at the grain U-CLASSNS and U-CLASSCLOSE's implementation specs
set: named files, line numbers, and a gate per stage.

Written to be built on a **side branch while `main` carries the class work**
(U-CLASSNS → U-CLASSCLOSE), then consolidated. The branch protocol is not an appendix;
it is [§00](00-branch-protocol.md), and it changes the stage order.

## Status — three stages already landed

| Stage | Spec | Commit | State |
|---|---|---|---|
| §S8 — delete the rustyline stack | surface.md §S8 | `380461c` | **landed** |
| §D2 — source binds to the artifact | plan.md §D2 | `16b3760` | **landed** |
| §D7 parser half — EOF → `UnrecognizedEof` | plan.md §D7 | `2fe6aba` | **landed** |
| Stage 0 — crate wiring & public VM surface | §01 | `3e118ab` | **landed** |
| Stage 1 — session & unwinding cell loop | §02 | `3e118ab` | **landed** |
| Stage 2 — two-set immutability & cross-cell rebind | §03 | `3e118ab` | **landed** |
| Stage 3 — continuation validator & trailing backslash | §04 | current | **landed** |
| Stage 4 — snapshot oracle & selectors | §05 | current | **landed** |
| Stage 5 — reedline surface (completer, highlighter, prompt) | §06 | current | **landed** |
| Stage 6 — REPL commands & `:reload` | §07 | current | **landed** |
| Consolidation — workspace tests & tracker updates | §08 | current | **landed** |

Everything below covers what remains. Do not re-implement the three above; do read
§D2's outcome, because [§02](02-session-and-cells.md) builds directly on `Chunk.source_id`.

## Read this before anything else: four deltas between the spec and the tree

The plan and surface docs were written against an earlier tree. Each item below was
**re-verified on 2026-07-19 at `dcd0567`**. An implementer who trusts the prose over
this table will write code that does not compile, or worse, compiles and means the
wrong thing.

**1. `phalcom-repl` does not depend on the language.** Its `Cargo.toml` lists
`reedline`, `regex`, `once_cell`, `unicode-segmentation`, `nu-ansi-term` — and nothing
else. There is no `phalcom-core`, no `phalcom-ast`, no `phalcom-lsp`. `grep -rn
"phalcom_core\|phalcom_ast\|phalcom_lsp" phalcom-repl/src/` returns **nothing**. The
REPL today is a standalone editor shell that evaluates a counter.

Neither plan.md nor surface.md states this. §D1, §D3, §D8, §D9, §S1, §S2, §S3, §S5-L1
and §S5-L2 are all unreachable until it changes. That is why [§01](01-wiring.md) exists
and why it is stage 0.

**2. `CompileMode` already exists, and it does not mean what §D3 means.**
`compiler::attributes::CompileMode` is ADR-0052's **contract-weaving** axis —
`Debug` / `Release` / `Unchecked`, governing whether `@requires` / `@ensures` /
`@invariant` guards are woven or stripped. It is stored once, globally, at
`vm/mod.rs:232`.

§D3 asks for CPython's `"single"` compile mode: suppress the trailing `Pop` on a final
expression statement. Adding a `Repl` variant to that enum would force every contract
expander to answer "how do contracts weave in Repl mode?", which is not a question.
It would also make an inherently **per-cell** property share a field with a **global**
one. [§02 §2.2](02-session-and-cells.md) specs a separate, orthogonal type and keeps
`CompileMode` untouched. The name in plan.md §D3 is the casualty; the design is not.

**3. `unwind_to` is `pub(crate)`.** `vm/dispatch.rs:110`. §D10 requires the cell loop to
call it, and the cell loop lives in `phalcom-repl` — a different crate. It is currently
uncallable from there. [§02 §4](02-session-and-cells.md) specs the public surface.

**4. surface.md's concurrent-edit note is stale.** It warns that another session held an
uncommitted 2-line change to `completer.rs::builtin_keywords()`. That change landed as
`ebc0a63` (adding `const`, which U-BINDINGS made a real keyword). There is nothing
uncommitted to land after. §S5 rewrites `completer.rs` wholesale regardless.

## Stage map

Ordered for **branch isolation**, which is not plan.md's stage order. [§00](00-branch-protocol.md)
explains the reordering: everything touching `phalcom-core` is pulled forward into one
small, early, `main`-landed batch so the side branch can be pure `phalcom-repl/**` and
conflict-free against the class work.

| Stage | File | Scope | Crate footprint | Conflicts with class work |
|---|---|---|---|---|
| 0 | [§01](01-wiring.md) | crate deps; public VM surface | `phalcom-repl/Cargo.toml`, `phalcom-core` (3 visibility changes) | **yes — land on `main` first** |
| 1 | [§02](02-session-and-cells.md) | §D1 + §D10 + §D3 — session module, unwinding cell loop, echo | `phalcom-core` (compiler), `phalcom-repl` | **yes — split; see §00** |
| 2 | [§03](03-immutability.md) | §D4 — two-set immutability, cross-cell rebind | `phalcom-core` (compiler) | **yes — split; see §00** |
| 3 | [§04](04-continuation.md) | §D7 REPL half — validator, `\`, `...`, escapes | `phalcom-repl` only | no |
| 4 | [§05](05-oracle-and-selectors.md) | §D8 + §D9 — live oracle, structured selectors | `phalcom-repl` only | no |
| 5 | [§06](06-surface.md) | §S1–§S7 — completion, hints, highlighting, latency | `phalcom-repl` only | no |
| 6 | [§07](07-commands.md) | §S9 — `:reload` | `phalcom-repl` only | no |
| — | [§08](08-consolidation.md) | merge protocol, test matrix, exit criteria | — | — |

Stages 3–6 are the bulk of the unit and touch **only `phalcom-repl/**`**. That is the
whole reason this unit is safe to branch: its conflict surface is concentrated in
stages 0–2, which are small.

## Decisions

**Nothing is open.** DEC-REPL-A/B/C are all CLOSED (plan.md §"Decisions to flag"), and
decision 0065 ruling 6 settles class redefinition. An implementer may not reopen them.

Two new questions surfaced while grounding this spec. Both are **ruled here**, not
deferred, because leaving them open would block stage 0:

- **The §D3 type is new and orthogonal, not a `CompileMode` variant.** Ruled in
  [§02 §2.2](02-session-and-cells.md). Rationale is delta 2 above.
- **`unwind_to` is exposed via a named `VM` method, not by making the existing one
  `pub`.** Ruled in [§02 §4](02-session-and-cells.md).

## Invariant this unit must not break

Three independent rulings key off `Compiler` being constructed **per cell**: §D4's
two-set immutability, U-BINDINGS' same-scope redeclaration ban
(`compiler/lib/scope.rs:118`, `:170`), and decision 0066's registration of class
declarations in the same `global_bindings` map. One undocumented lifetime carries all
three. The cross-cell regression test in [§03](03-immutability.md) is the only guard.
Do not weaken it, and do not make `Compiler` session-lived as an "optimization".
