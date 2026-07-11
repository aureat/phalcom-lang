# Handoff — U4: blocks/closures (Lua-style open/closed upvalues + frame-token infra)

## Context

`main @ 037da3d` has U-FE/U0/U3/U1/**U2** landed (U2 = metaclass tower parallel rule +
`Behavior` kernel + `verify_invariants()`, committed directly to `main`, no independent
`phalcom-reviewer` gate this pass — see [`U2-progress.md`](U2-progress.md)). Per
`docs/forge/remaining-work-handoff.md`'s U2→U4→U5 spine, U4 is next: first-class blocks,
Lua-style open/closed upvalues, and the frame-token *infrastructure* U10 will later consume
for non-local return (U4 ships **zero** non-local-return behavior).

`docs/forge/U4-plan.md` is a complete, pre-written work order (grounded in ADR-0013,
ADR-0006, `blocks.md`, `functions.md`). I re-verified its "Preconditions" section (§2)
against the current `main` HEAD (post-U2) — **nothing has drifted**, since U2 only touched
`universe.rs`/`vm.rs`/`tests/invariants.rs` bootstrap wiring, orthogonal to closures:

- `Expr` (`phalcom-ast/src/ast.rs:83`) has **no `Block` arm** — confirmed. Variants today:
  `Number/String/Boolean/Nil/Var/Field/SelfVar/SuperVar/Assignment/Unary/Binary/MethodCall/
  GetProperty/SetProperty`.
- `Token::FatArrow` is consumed **only** in `parse_method_block` (`phalcom-ast/src/parser.rs:648`),
  i.e. only in method/getter body position — not as an expression literal. Confirmed.
- `compiler/lib.rs::compile_block` (L118) still hardcodes `num_upvalues: 0` at both call sites
  (L175, L212) with the `// TODO: Calculate num_upvalues` marker — confirmed, this is the stub
  U4 replaces.
- `Callable` (`phalcom-core/src/callable.rs`) has `num_upvalues: usize` but **no capture
  descriptors** (`is_local`/`index`) yet — confirmed, needs adding.
- `ClosureObject` (`phalcom-core/src/closure.rs`) has `upvalues: Vec<Value>` — plain values,
  not heap-owned `Upvalue` cells. Confirmed this needs replacing per ADR-0013 (open cells must
  outlive the popped frame).
- `CallFrame` (`phalcom-core/src/frame.rs`) has no generation counter / frame-token field —
  confirmed, needs adding (infra only, per the U4/U10 boundary).
- `Bytecode` (`phalcom-core/src/bytecode.rs`) has no `Closure`/`GetUpvalue`/`SetUpvalue`/
  `CloseUpvalue` opcodes — confirmed, all four are new.
- `Value` (`phalcom-core/src/value.rs`) variants are `Nil/Bool/Number/Symbol/Obj` — no `Block`
  arm yet, confirmed.
- `CoreClasses` (`phalcom-core/src/universe.rs`, post-U2) has no `function_class`/`block_class`
  field — confirmed, U4 must add both as new kernel classes under `Object`, siblings of the
  existing `method_class`, without touching the U2-corrected parallel-superclass wiring or
  `verify_invariants()`.

No reconciliation deltas needed — `U4-plan.md` can be followed as written. Read it in full
before starting; this handoff does not repeat its design decisions (§4), write-set (§3), or
build order (§5) verbatim.

## Working model — precedent set by U2, follow unless told otherwise

U2 landed **directly on `main`, no worktree, no `phalcom-architect` pass, no
`phalcom-reviewer` gate** (explicit user instruction: "just pure coding and implementation").
That worked cleanly and is the default for U4 too:

1. Work **directly on `main`** — do not create a worktree/branch unless asked.
2. **No `phalcom-architect` reconciliation pass** — this handoff already did that (see above).
3. **No `phalcom-reviewer` gate** — implement, self-verify, and stop. Flag any risk in a new
   `docs/forge/U4-progress.md` (mirror `U2-progress.md`'s structure), same as U2.
4. Build/test bar per the plan's §7 "Green gate", *minus* the strict ceremony U2 also skipped:
   - `cargo build --workspace` clean.
   - `cargo test -p phalcom-core` — including un-pending `blocks()` in `tests/lang.rs` and the
     new block goldens the plan calls for (§3, §5 step 8) — these are deliverables of the unit
     itself, not optional verification, so they stay in scope even though the *reviewer* gate
     is skipped.
   - `cargo doc --workspace --no-deps` clean (no new warnings).
   - `cargo clippy` and the strict byte-identical golden-baseline ceremony are **not** required
     this pass — flag as deferred verification risk in `U4-progress.md`, same framing as U2.
   - Run `cargo run -p phalcom-core --bin phalcom examples/core_new.ph` (or similar) once at the
     end to confirm the interpreter still boots and runs end-to-end.
5. **Respect the U4/U10 boundary exactly as the plan states it** (§4, §1 guardrails): frame-token
   *infrastructure* only (generation counter + token mint/compare helper); no `ReturnNonLocal`
   opcode, no unwind logic, no non-local-return test. If a `return` inside a braced block reaches
   the compiler during implementation, do not invent the non-local path — note it for U10.
6. If the unit turns out large enough to risk ballooning context, slice it (fresh sub-context per
   slice, commit + update `U4-progress.md` between slices) rather than grinding one pass to
   ~176k tokens (the U1 lesson, see memory `subagent-context-handoff`) — but attempt it in one
   pass first, the way U2 fit comfortably.
7. Commits: conventional format, e.g. `feat(u4): first-class blocks + open/closed upvalues +
   frame-token infra`, ending `Co-Authored-By: Claude Sonnet 5 <noreply@anthropic.com>`.
8. On landing: update `docs/forge/STATE.md`'s phase log (U2 ✅ → U4 ✅, note reviewer gate
   skipped, next = U5) and `docs/forge/PHASE2-INDEX.md`'s U4 roster row, same as done for U2.
9. Do **not** push to `origin` — stays the user's call.
10. graphify-first per the repo's `CLAUDE.md`/hook rule: `graphify explain "ClosureObject"`,
    `graphify affected "CallFrame"`, `graphify affected "compile_block"` before editing; rebuild
    (`graphify update . --no-cluster`) after.

## Files touched (from `U4-plan.md` §3 — reproduced for quick reference, not authoritative;
defer to the plan if this drifts)

`phalcom-ast/src/ast.rs`, `phalcom-ast/src/parser.rs`, `phalcom-ast/src/{token,lexer}.rs`
(only if a token is missing), `phalcom-core/src/value.rs`, `phalcom-core/src/block.rs` (new),
`phalcom-core/src/upvalue.rs` (new), `phalcom-core/src/closure.rs`, `phalcom-core/src/callable.rs`,
`phalcom-core/src/frame.rs`, `phalcom-core/src/bytecode.rs`, `phalcom-core/src/chunk.rs`,
`phalcom-core/src/compiler/lib.rs`, `phalcom-core/src/vm.rs`, `phalcom-core/src/primitive/block.rs`
(new) + `primitive/mod.rs`, `phalcom-core/src/{universe,class}.rs`, `phalcom-core/core/core.ph`
(only if needed), `phalcom-core/bin/phalcom/disasm.rs`, `phalcom-core/tests/lang.rs`,
`phalcom-core/tests/fixtures/golden/` + `golden.rs`.

## Verification

- After implementation: `cargo build --workspace` && `cargo test -p phalcom-core` clean
  (including previously-`#[ignore]`d `blocks()`), `cargo doc --workspace --no-deps` clean.
- Final sanity: run the interpreter on a `.ph` program that defines and calls a block (a new
  golden fixture satisfies this).
