# 002 — U-GC Win A: box the six fat `Object` variants (Tier 4 representation)

Commit `7480d75`. Grounded in [ADR-0050 §Decision 9](../../adr/accepted/0050-non-moving-mark-sweep-collector.md)
+ [memory-management.md §7](../../spec/current/memory-management.md). U-GC step 1 of 5;
standalone, no collector dependency.

## The cost

`Heap` is a `SlotMap<ObjRef, Object>`. A `SlotMap` sizes **every slot to the fattest
enum variant**, so an unboxed `ClassObject` set the price of every object in the
arena — a 16 B `Tuple` and a 32 B `Str` each cost 280 B.

`size_of::<Object>()` measured **280 B** on HEAD 2026-07-14 —
[F7](findings.md#f7--size_ofobject-grew-to-280-b-win-a-is-six-variants-not-the-driver),
up from the 256 B ADR-0050 recorded, because `ClassObject` gained
`attributes: Vec<Value>` (+24 B) under U-ANNOT. `ClassObject` alone *was* the 280 B.

## The cut

Box the six fat payloads — `Class` (280), `Fiber` (176), `Module` (168), `Closure`
(160), `Method` (88), `Map`/`Set` (72) — leaving `RangeObject` (40 B) as the cap:

```rust
Class(Box<ClassObject>),
Method(Box<MethodObject>),
Module(Box<ModuleObject>),
Closure(Box<ClosureObject>),
Fiber(Box<FiberObject>),
Map(Box<MapObject>),
Set(Box<MapObject>),
```

**`Instance` is deliberately not boxed.** At 24 B it is already under the `Range`
floor, and it is the most-allocated variant — a `Box` would add an indirection *and*
an allocation for zero size win. The obvious-looking move is the wrong one, and
ADR-0050 §9's variant list (which predates the measurement) named it.

**`size_of::<Object>()`: 280 B → 40 B (7×).**

The diff is 18 construction sites across 11 files. Every *read* site was absorbed by
deref coercion (`&Box<T>` → `&T` at the return of the centralized
`heap/accessors.rs` accessors) — which is why a 7× representation change costs 283
diff lines and no API change.

## Measured (release, 2 runs each, alternating boxed/unboxed)

| Workload | | wall | `sys` | peak RSS |
|---|---|---|---|---|
| **`for.ph`** (1M `Some` churn) | nobox | 8.69 s, 9.83 s | 3.34, 2.78 | 2.80 GB, 5.07 GB |
| | boxed | **5.93 s, 4.57 s** | **1.40, 0.59** | 3.43 GB, 3.45 GB |
| **`skynet`** (1M fibers) | nobox | 18.63 s, 16.75 s | 7.21, 7.02 | 4.31 GB, 4.08 GB |
| | boxed | **11.75 s, 11.61 s** | **4.22, 3.64** | 3.90 GB, 5.76 GB |

**`for.ph` −43% wall · `skynet` −34% wall.** Boxed wins 4/4 comparisons, margins far
above the noise floor.

**The mechanism is `sys` time, and it is not what ADR-0050 predicted.** The ADR
hypothesised a `heap.get` cache-density win; `user` time barely moves (−8% on
`for.ph`). What moves is `sys`: 3.34/2.78 → 1.40/0.59 on `for.ph` (~4×), 7.21/7.02 →
4.22/3.64 on `skynet` (~2×). The cause is **`SlotMap` backing-array growth**: at
40 B/slot instead of 280 B/slot, a million objects cost 40 MB of contiguous arena
growth instead of 280 MB — and every realloc along the way memmoved the whole array.
This is [F3](findings.md#f3--memmove-206-skynet-is-vec-growth-not-memtake)'s memmove
and [F1](findings.md#f1--measured-baseline-supersedes-the-oral-29)'s malloc
attribution, hit from an unanticipated angle.

**No RSS claim is made.** Peak RSS is a wash and the samples are not even internally
consistent (nobox `for.ph` measured 2.80 GB then 5.07 GB across two identical runs).
This is expected: boxing **relocates** bytes from the arena to malloc rather than
deleting them. A boxed fiber is a 40 B slot *plus* a 176 B box = 216 B, only 23% off
280 B — not 7×. Reclamation, not representation, is still the RSS lever (F5).

## Counter-evidence (recorded, not hidden)

`bare_send` regresses **~5%** — boxed was slower in 6/6 measurements across the
session. Real: it is one extra pointer chase on the dispatch path. The effect is
under this machine's noise floor, and it is vastly outweighed by the allocation-path
win on both real workloads. Recorded so a future `bare_send` investigation does not
rediscover it as a mystery.

## Verification

- `cargo build` (workspace) + `cargo doc -p phalcom-core --no-deps` clean.
- **Clippy delta vs the pre-change tree is empty** — 11 warnings before, the same 11
  after, all in files this cut does not touch.
- Tests 39/41. The 2 failures (`indexing`, `indexing_negative`) were confirmed
  identical at `f0a8a1d` with the change stashed — a `[]()` subscript gap, orthogonal
  to a representation change.

## Method note — the micro-benches were the wrong instrument

This cut was nearly abandoned. Criterion said `bare_send` **+8.8% regressed
(p = 0.00)**; a re-run of the *same binary* against the *same saved baseline* then
said **+1.3%, no change (p = 0.29)**. Criterion's p-value covers within-run variance,
not a machine whose clock drifts between runs — on this hardware it certified noise
as significance twice.

Worse, a hypothesis was built on the first run ("boxing `Closure` puts a pointer
chase on the per-opcode `heap.closure(id).callable.chunk` fetch at
`dispatch.rs:391`"), tested, and *appeared* confirmed — then the config it supposedly
beat measured identically. It was **fitted to noise** and discarded.

The error was reaching for a per-*send* instrument to measure a change whose cost is
per-send but whose benefit is per-*allocation*. **Lesson for the remaining tiers: a
representation change must be measured on an allocation-heavy real workload
(`for.ph`, `skynet`) with `/usr/bin/time -l`, not on criterion micro-benches.** An
A/B harness must also never run `cargo build` inside its measurement loop — two runs
of the 4-pair A/B came back at 2× normal from build contention.
