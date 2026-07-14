# 006 — F14 S2: drop `spans[ip]` from the dispatch read-decode (Tier 2)

Status: **landed** `916be0a` · Unit: [U-HOTPATH](../units/U-HOTPATH/plan.md) Change 1 family (Tier 2) · Finding: [F14 S2](findings.md#f14--the-dispatch-loop-re-derives-every-frame-field-on-every-opcode) · Behavior-invariant (no ADR, no floor change)

The first of [F14](findings.md#f14--the-dispatch-loop-re-derives-every-frame-field-on-every-opcode)'s
four candidate cuts (S2), landed **before** S1 ([007](007-hoist-rc-callable.md)) on
purpose — see [Why S2 first](#why-s2-first).

## The cost

`vm/dispatch.rs`'s `run_until_inner` read **two** parallel arrays per opcode:

```rust
let (opcode, source_range) = {
    let chunk = &self.heap.closure(closure_id).callable.chunk;
    (chunk.code[ip], chunk.spans[ip])          // <- spans[ip] discarded on the happy path
};
```

`Chunk.spans: Vec<SourceRange>` (`chunk.rs:46`) runs parallel to `Chunk.code`
(`chunk.rs:44`) — one span per instruction, for diagnostics. **Only two of ~35 opcode
arms ever read it**, and both only to hand it onward for the *error* path:

| consumer | symbol | passes it to |
|---|---|---|
| `Bytecode::Invoke` | `VM::call_method`, `VM::forward_does_not_understand` | error reporting |
| `Bytecode::SuperSend` | same | error reporting |

Every other opcode — `GetLocal` (19% of `for`'s mix), `Constant` (25% of
`string_equals`'s), `Pop`, `Jump`, … — paid a bounds-checked load of a 2×`u32`
`SourceRange` it discarded. `SourceRange` is `Copy`, so the load is cheap; it is the
**per-instruction frequency** that made it worth removing, not its unit cost.

## The cut

Read `code[ip]` alone in the loop head (`dispatch.rs:458` at HEAD):

```rust
let opcode = callable.chunk.code[ip];
```

Re-read `spans[ip]` in each of the two consumers. **`Invoke` pays nothing**, because
it folds the read into the borrow its IC probe already takes (`dispatch.rs:947-953`):

```rust
let (cached, source_range) = {
    let chunk = &self.heap.closure(closure_id).callable.chunk;
    let cached = chunk.caches[ip].get().filter(|slot| {
        slot.class == receiver_class && slot.version == self.world_version
    }).map(|slot| slot.method);
    (cached, chunk.spans[ip])                  // <- rides the existing borrow
};
```

`SuperSend` reads it directly (`dispatch.rs:782`) — a send is expensive enough that
one extra lookup on that arm is not measurable, and `SuperSend` is 0% of every hot
benchmark's mix.

**Shape note.** F14 proposed "re-derive from `(closure, ip)` on the error path". The
shipped shape is cheaper and simpler: the *frame already carries both*, so no
derivation is needed — the consumer just reads the array it was already borrowing.
The estimate held (−3–8% predicted, −2.8–6.8% measured) but the mechanism was one step
simpler than the finding assumed.

## Method

Alternating same-session A/B (`REPS=5`), both binaries built **before** any timing
(README §Method: never `cargo build` inside a measurement loop), stdout byte-compared
on every run. Read the **sign across pairs**, not a p-value — criterion has certified
noise at `p = 0.00` on this hardware twice.

## Result

| Benchmark | base `user` | S2 `user` | Δ | pair signs |
|---|---|---|---|---|
| `for` | 0.694 s | 0.647 s | **−6.8%** | `-----` |
| `method_call` | 0.502 s | 0.473 s | **−5.6%** | `-----` |
| `variadic_send` | 0.666 s | 0.632 s | −5.2% | `-----` |
| `arith_send` | 0.031 s | 0.030 s | −3.0% | `-----` |
| `bare_send` | 0.035 s | 0.034 s | −2.8% | `+----` |
| `skynet` | 1.80 s | 1.75 s | **−2.8%** | `---` |

**24 of 25 pairs negative.** RSS unchanged (skynet 1.304 → 1.316 GB, noise).
`cargo test --workspace` green; golden diff **none**.

**Read the two 0.03 s rows with care.** `bare_send`/`arith_send` are ~0.03 s programs
in which the ~7.7 ms bootstrap is ~15–25%, so their percentages **understate** the
effect and their signs are the noisiest — the single `+` in the table is `bare_send`'s
first pair. `for` (−6.8%) and `method_call` (−5.6%) are the load-bearing rows: ~0.5–0.7 s
each, unanimous.

## Why S2 first

F14 ranked S1 (*est* −30–45%) far above S2 (*est* −3–8%) but said to land **S2 first,
as an isolated A/B, before S1 makes it unmeasurable**. That advice was correct and is
worth generalizing: S1 ([007](007-hoist-rc-callable.md)) hoists the whole chunk access
into a loop local, which subsumes the `spans[ip]` load entirely. Had they landed
together, S2's contribution would have been **permanently unattributable** — folded
into S1's number with no way to separate them. Landing the small one first costs one
extra A/B and buys a number that stays true.

## Write-set

- `phalcom-core/src/vm/dispatch.rs` — loop head; `Bytecode::Invoke` arm; `Bytecode::SuperSend` arm.

No other file. No new goldens (behavior-invariant). Floor: +0.
