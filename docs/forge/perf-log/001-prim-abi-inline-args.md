# 001 — U-PRIM-ABI: on-stack argument buffer (Tier 2 allocation cut)

Status: **landed** · Unit: U-PRIM-ABI (Tier 2) · Spec: [performance.md §4 Tier 2](../../spec/v0.2/performance.md), [ADR-0051](../../adr/proposed/0051-performance-strategy-measure-first-tiered-optimization.md) · Behavior-invariant (no ADR, no floor change)

## The cost

`VM::call_method`'s `Primitive` arm (`phalcom-core/src/vm/send.rs`) built the
primitive's argument slice with `self.stack[receiver_idx + 1..].to_vec()` — a
fresh heap `Vec` **per send**. The primitive path pushes no `CallFrame` (only the
`Closure` arm does), so this `to_vec()` was the *only* per-send heap allocation on
the native fast path.

U-BENCH attribution (see [`findings.md`](findings.md) F1) put **malloc/free as the
single largest mechanism** on the arithmetic micro-bench — every `1 + 2` send
heap-allocated a one-element argument `Vec`, then freed it.

## The cut

`Value` is `Copy` (16 bytes), so the receiver+args window is copied into a fixed
on-stack `[Value; 8]` and the primitive is handed a slice of it. A rare wider call
(variadic spread > 8 args) falls back to the heap `Vec`. The primitive still
receives an identical `&[Value]` — `PrimitiveFn`'s signature is unchanged, so **no
primitive migration** was needed.

```rust
const INLINE_ARGS: usize = 8;
let result = if arity <= INLINE_ARGS {
    let mut args = [Value::Nil; INLINE_ARGS];
    args[..arity].copy_from_slice(&self.stack[receiver_idx + 1..]);
    native_fn(self, &receiver, &args[..arity])
} else {
    let args: Vec<Value> = self.stack[receiver_idx + 1..].to_vec();
    native_fn(self, &receiver, &args)
};
```

## Measured (criterion, 100 samples, p < 0.05)

| Bench | Before | After | Δ |
|-------|--------|-------|---|
| `arith_send` (200k Number `+` sends) | 72.1 ms (~361 ns/send) | 42.2 ms (~211 ns/send) | **−41.5%** |
| `bare_send` (200k dispatch-bound sends) | 65.7 ms (~329 ns/send) | 43.5 ms (~218 ns/send) | **−33.8%** |

Ratifies the attribution: removing one per-send `Vec` alloc took ~40% off
arithmetic send wall-time.

## Verification

- **Zero golden diff.** `cargo test -p phalcom-core` — 39/41 lang+invariant tests
  pass; the 2 failures (`indexing`, `indexing_negative`) are pre-existing on clean
  HEAD (a concurrent `[]`-subscript parse WIP) and are a *parse* error a
  `vm/send.rs` edit cannot cause.
- `cargo build` + `cargo clippy -p phalcom-core` clean on the changed lines (no new
  warnings from `send.rs`).
- Write-set: one runtime file, `phalcom-core/src/vm/send.rs`. No new dependency.

## Scope decision

This captures U-PRIM-ABI's **entire measured goal**. DEC-PRIM-B is resolved by the
result — the allocation cut alone won ~41% on arithmetic, so the guarded arithmetic
superinstruction is **deferred to U-IC** (its remaining cost is dispatch lookup,
Tier 3), not shipped here. The plan's full Wren-style window-status ABI (migrate
~70 primitives) would buy only a ~128-byte on-stack memcpy over this cut — not
worth the churn. **U-PRIM-ABI is considered met at the allocation cut**; the
migration is not pursued unless a re-measure demands it.
