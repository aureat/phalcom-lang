# 008 — superinstructions: fuse `(GetLocal | Constant) -> Invoke` (Tier 3)

Status: **landed** `1d2baea` · Unit: [U-IC](../units/U-IC/plan.md) (superinstructions) · Finding: [F19](findings.md#f19--a-dispatch-costs-33-ns-and-that-is-what-a-fusion-buys-h13) (the price), [F16](findings.md#f16--superinstructions-are-premature-no-opcode-histogram-and-the-inliner-already-covers-the-classic-win) (**verdict overturned**) · Behavior-invariant (no ADR, no floor change)

The F16 re-ask, run and answered: **superinstructions pay.** Fusing the two
highest-ranked pair shapes removes 13–20% of every hot program's dispatches and
measures **−8.1% `string_equals`, −5.1% `for`, −4.7% `variadic_send`, −4.2%
`bare_send`, −3.9% `fib`**.

## Why the re-ask flipped the verdict

F16 deferred on three reasons. **All three are now gone**, and the third was not
retired — it was *false*:

| F16 reason | Status |
|---|---|
| 1. "They would pay for a bug, not a cost — do S1 first, the case may evaporate" | **Retired.** S1a landed (cut 007). The case did **not** evaporate: 007 deleted the per-opcode *re-derivation*, but the per-opcode *fixed cost* it leaves (safepoint, 96 B frame copy, guard compare, bounds-checked `code[ip]`, `ip` re-index, jump-table branch) is still ~3.3 ns, and that is exactly what a fusion deletes |
| 2. "The pairs cannot be chosen" | **Retired** (`5516504`) — the statically-adjacent pair counter |
| 3. "The inliner already covers the classic arithmetic win" | **FALSE, and never checked.** `compiler/inliner.rs`'s sacred set is `ifTrue(_)`, `ifFalse(_)`, `ifTrue(_:ifFalse:)`, `and(_)`, `or(_)`, `whileTrue(_)` — **control flow only**. It has never inlined arithmetic. `1 + 2` still compiles `Constant, Constant, Invoke`, which the histogram confirms: `arith_send` retires `Invoke` at 19% and `Constant -> Invoke` at 20%. **The classic arithmetic win was never covered** |

Reason 3 was the one this session was told to check before fusing arithmetic. It
does not survive reading the recognizer.

## The cost, and the price that sizes it

A fusion does not remove an opcode's *work* — it removes one **dispatch**. Sizing
it therefore needs the price of a dispatch, which was hole **H13** and is the
input F16 never had. Two independent measurements, both at HEAD (see
[F19](findings.md#f19--a-dispatch-costs-33-ns-and-that-is-what-a-fusion-buys-h13)):

- **Differential** (two programs differing by a known, histogram-verified
  6,000,000 instructions of near-zero body): **3.56–3.68 ns** per added
  instruction, linear at 4× (measured 4.14 against an ideal 4.00). That is
  `dispatch + body`, so an **upper bound** on the dispatch alone.
- **This cut, read backwards**: Δwall ÷ dispatches removed, over the rows whose
  A/B is unanimous: **3.05–3.86 ns**. Bodies are preserved by a fusion, so this
  is the dispatch **alone**.

Two instruments, from opposite directions, landing on the same ~3.3 ns. **8.8% of
instructions is not 8.8% of time** — the trap this cut was warned about — and the
ceiling the price predicts (`removed × 3.6 ns ÷ wall`) matched what shipped on
every row.

## The cut

Two fused opcodes (`bytecode.rs`), both 8 B — `Bytecode` does **not** grow, since
`SuperSend(u8, u16, u16)` already sets the width:

```rust
InvokeLocal(u16, u8, u16),   // slot,  arity, selector
InvokeConst(u16, u8, u16),   // const, arity, selector
```

A post-compile peephole (`Chunk::fuse_superinstructions`, called at the two
`Callable` construction sites in `compiler/lib/mod.rs`) rewrites each
statically-adjacent pair **in place**.

### The in-place rewrite is the whole trick

The fused opcode replaces the pair's **first** instruction; the original `Invoke`
is left at `p + 1` as dead code and the fused arm advances `ip` by **2**.
`code.len()` never changes, so:

- every jump offset in the chunk stays correct — **no re-layout pass**;
- `spans`, `caches` and `gcaches` (all `ip`-indexed, parallel to `code`) stay
  aligned with no re-indexing;
- the fused arm reads its IC and span at **`ip + 1`** — the dead `Invoke`'s own
  slots — so it probes the same cache and reports the same span as the pair it
  replaced. `chunk.rs`'s invariant ("only `Invoke` indices are ever non-`None`"
  in `caches`) stays literally true.

Compacting the array instead would mean rewriting every branch offset and
re-indexing three side tables **for the same number of saved dispatches**.

The send itself is not duplicated: `Invoke`'s body was extracted verbatim into
`VM::invoke_at(callable, cache_ip, arity, selector)`, and all three arms call it.
IC probe, refill, variadic probe and the `doesNotUnderstand(_)` forward are
therefore shared code, not three copies that can drift.

### Soundness — why a jump target forbids the fusion

The rewrite is sound only if `p + 1` is unreachable. If a branch targets the
`Invoke` directly, that entry point must keep finding a real `Invoke` there — so
`branch_targets()` collects every `Jump`/`JumpIfFalse`/`JumpIfNone`/`Loop`/
`GuardBool`/`GuardBlock` target (conservatively, taken or not) and such pairs are
skipped. The fallback is the unfused pair: correct, just not fast.

**The guard fires in 0 chunks across `core.ph`, `for.ph` and 60 lang fixtures** —
i.e. it is defensive surface that the corpus does not exercise. Per
[F10](findings.md#f10--the-fiber-pool-is-not-neutral-it-is-negative--and-f5-measured-the-wrong-workload)'s
lesson (unmeasured surface is not free) it is covered by three unit tests in
`chunk.rs`, one of which constructs the jump-into-`Invoke` case and **fails
without the guard**.

## Method

Alternating same-session A/B vs `b9a3048`, best-of-7 (best-of-3 for skynet under
`/usr/bin/time -l`), both binaries built **before** any timing, stdout
byte-compared on every run. Counts from a separate `--features opcode-histogram`
build; **no timing was ever read from a counting build**.

## Result

| Benchmark | dispatches removed | % of instrs | base | fused | Δ | pairs |
|---|---|---|---|---|---|---|
| `string_equals` | 15,000,001 | 15.2% | 0.899 s | 0.826 s | **−8.1%** | `-------` |
| `for` | 9,000,001 | 13.2% | 0.575 s | 0.546 s | **−5.1%** | `+------` |
| `variadic_send` | 8,000,001 | 16.0% | 0.566 s | 0.539 s | **−4.7%** | `-------` |
| `bare_send` | 400,001 | 12.5% | 0.033 s | 0.031 s | **−4.2%** | `----+--` |
| `fib` | 10,284,581 | 14.8% | 0.678 s | 0.651 s | **−3.9%** | `--+----` |
| `binary_trees` | 4,619,021 | 7.5% | 0.614 s | 0.596 s | **−3.0%** | `-+-----` |
| `arith_send` | 600,001 | 20.0% | 0.027 s | 0.026 s | −1.6% | `---+---` |
| `method_call` | 1,200,001 | 2.4% | 0.440 s | 0.433 s | −1.5% | `--+--+-` |
| `skynet` | — | — | 1.63 s `user` | 1.60 s | −1.8% | `-+-` |
| `map_numeric` | 18,000,003 | 14.3% | 3.459 s | 3.453 s | **−0.2%** | `++-+---` |

RSS unchanged on skynet (1.242 GB both sides) — a dispatch cut allocates nothing
less, and unlike cut 007 it did not move the number.

**`for` 68,000,706 → 59,000,705 instructions retired, exactly −9,000,001** = 6M
`InvokeLocal` + 3M `InvokeConst`, matching the pair counter's prediction to the
digit. That the *predicted* pair counts and the *realized* fusion counts agree
exactly is what says the pass fused what the instrument said it would.

### The two rows that did not move, and why they are the finding's proof

**`map_numeric` removed 18.0M dispatches — the most of any row — and measured
−0.2%.** It is not a failure; it is [F17](findings.md#f17--an-instruction-costs-1013-ns-and-the-28-spread-is-per-instruction-work-not-instruction-count)
being right. Its instructions cost **27.6 ns** against `for`'s 8.9: they are
individually heavy (hashing, allocation, GC), so removing a ~3.3 ns dispatch from
one is noise. `skynet` (27.7 ns/instr) and `fiber_churn` (−0.1%) say the same
thing. **A fusion buys dispatch, and only workloads whose time *is* dispatch can
spend it.** The ceiling predicted 1.2% for `map_numeric` before the code was
written, and 1.2% is what it is worth.

**`method_call` −1.5%** because its top pair is `GetSelf -> GetField` (12.8%),
which this cut does not fuse — only 2.4% of its instructions were reachable. It
is the obvious next fusion.

## Caveats

- **`arith_send` and `binary_trees` swing across runs** (arith_send read −5.4% /
  −1.8% / −1.6% over three sessions). Both are dominated by rows where the ~7.7 ms
  bootstrap is a large share (`arith_send` is a 0.027 s program — bootstrap is
  ~28% of it), so their percentages understate and their signs are the noisiest.
  **`string_equals`, `variadic_send` and `for` are the load-bearing rows**: all
  ~0.5–0.9 s and unanimous or near-unanimous.
- **×-vs-Wren was not refreshed.** No `wren_test` binary is present on this machine
  this session, so `compare-wren.py` timed Phalcom only and **§1's ratios are not
  re-derived here**. The behavior evidence is the base-vs-fused stdout byte-compare
  on every A/B run (stronger for this cut than the Wren diff) plus the exact
  instruction-count delta.
- **This cut and S1b compete for the same bytes.** Both attack the per-opcode fixed
  cost: fusion removes whole dispatches, S1b (hoist `ip`/`stack_offset` behind a
  frame-identity guard) makes each one cheaper. **Doing both pays less than the sum**
  — every dispatch this cut deleted is one S1b can no longer speed up. Re-derive
  S1b's estimate at *this* commit, not at 007's.
- Bootstrap is **7.35 ms** fused vs 7.81 ms base: the pass costs nothing
  measurable (it also *runs* `core.ph`, which is now faster). Tripwire passes at
  8.1 ms against its 20 ms ceiling.

## Write-set

- `phalcom-core/src/bytecode.rs` — `InvokeLocal`/`InvokeConst` variants, `BYTECODE_NAMES`, `index()`, `VARIANTS` 35 → **37**.
- `phalcom-core/src/chunk.rs` — `fuse_superinstructions`, `branch_targets`, 3 unit tests.
- `phalcom-core/src/vm/dispatch.rs` — `invoke_at` helper extracted from the `Invoke` arm; two fused arms.
- `phalcom-core/src/compiler/lib/mod.rs` — call the pass at both `Callable` construction sites.

No new goldens (behavior-invariant). Floor: +0. No `unsafe`. `cargo test --workspace` green (26 targets), `cargo clippy --workspace` clean, `cargo doc` clean.
