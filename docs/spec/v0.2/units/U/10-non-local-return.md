# U10 — Non-Local Return (as-built)

- **Status:** ✅ Landed — `4e2ec73` (`U10: non-local return via `return` inside blocks`). In-tree on `main`, no worktree.
- **Realizes:** [ADR-0013](../../../../adr/0013-closure-upvalues-and-frame-token-return.md) (frame-token non-local return — consumes the infrastructure U4 stood up). Spec: [blocks.md](../../blocks.md) §5, [functions.md](../../functions.md) §2, [object-model.md](../../object-model.md) §4 (`DeadFrameError`).
- **Reviewer gate:** OFF per STATE.md review policy (U10 is not in the load-bearing set U1/U2/U4/U6) — self-verified on the green gate, including the dead-frame path and upvalue-across-unwind promotion.

## Mission
Make `return` inside a block unwind to the **enclosing method activation** (not just the block's own frame) by consuming U4's frame token — comparing the block's `home_frame_token` generation against the live frame stack — and raise **`DeadFrameError`** when the home frame is already gone.

## Surface / behavior
A `return` inside a braced block exits the method that lexically created the block, even across intermediate calls (e.g. an `each` iteration). Calling an escaped block whose home method has already returned is a clean runtime error, not undefined behavior.

```phalcom
class Finder {
  findNegative(nums) {
    nums.each { n => if (n < 0) { return n } }   // returns from findNegative, not from the block
    return 0
  }
}

class Maker {
  make() { return { return 1 } }                 // returns an escaping block that captures a return
}
let b = Maker.new().make()
b.call()                                          // DeadFrameError — Maker.make's frame is gone
```

## Implementation
- `bytecode.rs` — new `Bytecode::ReturnNonLocal` (no operand: the unwind target is read off the executing frame, not the instruction stream). It deliberately does **not** `return Ok(value)` out of `run_until` the way `Bytecode::Return` does. Disassembly needs no change (`disasm.rs` is pure `Debug` formatting).
- `compiler/lib.rs` — `FunctionState` gains `is_block`, set to `!is_method` in `compile_block`. `Statement::Return` emits `ReturnNonLocal` when the enclosing function is a block literal, and the ordinary `Bytecode::Return` for method/constructor bodies. (`compile_block`'s existing `is_method` parameter is the compile-time discriminant; the single `is_method=false` call site is `Expr::Block`.)
- `frame.rs` — `CallFrame.home_frame_token: Option<FrameToken>`, `None` for ordinary method/closure calls, `Some` only for a block invocation. `FrameToken` is `Copy`, so `Option<FrameToken>` keeps `CallFrame` `Copy` (load-bearing — the VM holds frames in a plain `Vec`).
- `primitive/block.rs` — `resolve_callable` now surfaces the block's `home_frame_token` alongside the closure handle (`None` for a bare `Object::Closure`, e.g. a `Method`'s callable used reflectively as a `Function` — those have no lexical home frame). `block_call` stamps the pushed `CallFrame` with that token (post-construction assignment, keeping `new_call_frame`'s signature stable).
- `error.rs` — `RuntimeError::DeadFrameError`, a plain `thiserror` variant with no span and no miette (matching every existing `RuntimeError` neighbor; the codebase has no spanned diagnostics today).
- `vm.rs` — the `Bytecode::ReturnNonLocal` handler and a guard in `call_method`'s `MethodKind::Primitive` arm.

The eager-unwind design (the one architectural fact of this unit): every block invocation runs through `block_call`, which re-enters `VM::run_until` **recursively** on the Rust call stack, all sharing one `vm.frames` `Vec`. A block's home frame is therefore always in an *outer, suspended* `run_until`, never the innermost one executing the `return`. The `ReturnNonLocal` handler unwinds the whole thing in one shot at the point `return` executes:
1. Read the executing frame's `home_frame_token` off `self.frames.last()`.
2. Search `self.frames` for the frame at `token.frame_index` whose `.generation == token.generation`. **Not found / generation mismatch** → the home method already returned; raise `DeadFrameError` **before** mutating any state (a partial mutation followed by an error would corrupt state for whatever catches it).
3. **Found** → evaluate the return expression, then `close_upvalues_from(home_frame.stack_offset)` (one call closes every open upvalue at or above that offset for every popped frame) **before** truncating; `stack.truncate(home_frame.stack_offset)`; push the return value (surfaced through `surface_absence`, so a bare `return` yields `None`); `frames.truncate(token.frame_index)`.

The handler does **not** return a value out of the match arm — the surrounding `loop { }` continues, and the **unmodified** top-of-loop drain check (`if self.frames.len() <= base_frames { pop stack, return Ok(...) }`) in whichever nested `run_until` first finds its floor at or above the shrunk frame count picks the value up.

The Primitive-arm guard (`call_method`) is the second, mandatory half: snapshot `frames_before = self.frames.len()` immediately before calling `native_fn`; if the frame count shrank afterward, a non-local return unwound past this call site — so **skip the stale `truncate(receiver_idx)` and re-push** the returned value (rather than the plan's "skip both": `run_until`'s drain check *pops* the value the handler pushed, so it must be re-established for the resuming outer frame). Each unwound level's push balances its drain-pop exactly — no duplicate, no loss. Verified specifically on the multi-level `each`-calling-`.call()` case; a single-level `{ return x }.call()` never crosses more than one `run_until` boundary and would not have caught this.

## Invariants & tests
- `blocks/blocks_non_local_return.ph` — multi-level `each` unwind → `-5` (PASS); crosses a re-entrant `block_call`.
- `blocks/blocks_non_local_return_bare.ph` — value-less `return` in a block surfaces `<None instance>` (PASS).
- `runtime-errors/runtime_non_local_return_dead_frame.ph` — escaped block called after its home method returned → `DeadFrameError` (NEGATIVE, substring match).
- `blocks/blocks_escape.ph` still passes byte-identical — upvalue promotion survives the new unwind path.
- `verify.sh` exit 0; `cargo doc --workspace --no-deps` clean.

## Deviations & deferrals
- **`error.rs` uses plain `thiserror`, not miette** — the plan called for a miette diagnostic with spans, but no `RuntimeError` variant carries a span and miette is unused in the tree; `DeadFrameError` matches its neighbors (corrects U10-plan §3 / implementation-spec §0.1).
- **`disasm.rs` untouched** — it is `Debug`-only, so the new opcode disassembles automatically (corrects U10-plan write-set).
- **Primitive-arm guard re-pushes the value** rather than skipping the push, correcting U10-implementation-spec §2 pt3 (see above).
- **Pending fixtures rewritten off real `List`:** `pending/blocks_non_local_return.ph` (used unparseable `[3,-5,8]` list-literal syntax) was rewritten against `List.new()`/`.add(_)` and promoted; `pending/blocks_argument_to_method.ph` stays pending — it needs `List.reduce`, which is U-STD's job, **not** a U10 blocker ([`docs/forge/phase-next/DEFERRED.md`](../../../../forge/phase-next/DEFERRED.md) #25).
- **No exception machinery.** ADR-0008 notes `throw`/`return`/`abort` unify as one unwind primitive; U10 implements only the `return` slice and does not preclude that reuse. `break`/`continue` are out of scope (loop sugar, U5).
- **Concurrency-forward:** the unwind operates only on the current fiber's `frames`/`stack`; nothing assumes a single global call stack, so a future per-fiber stack is not foreclosed. See [deferred-work.md](../../deferred-work.md).

## Sources
- Forge work orders (`U10-plan.md` and the superseding `U10-implementation-spec.md`) folded into this spec (see git history); landing record: [`docs/forge/archive/phase2/STATE.md`](../../../../forge/archive/phase2/STATE.md) "U10 — LANDED".
- Code: `phalcom-core/src/{bytecode,error,frame,vm}.rs`, `phalcom-core/src/compiler/lib.rs`, `phalcom-core/src/primitive/block.rs`.
