# 007 — F14 S1a: hoist the frame's `Rc<Callable>` out of the dispatch loop (Tier 2)

Status: **landed** `5254586` · Unit: [U-HOTPATH](../units/U-HOTPATH/plan.md) **Change 1** (Tier 2) · Spec: [implementation-spec.md §4](../units/U-HOTPATH/implementation-spec.md) · Finding: [F14 S1](findings.md#f14--the-dispatch-loop-re-derives-every-frame-field-on-every-opcode) · Behavior-invariant (no ADR, no floor change)

U-HOTPATH's Change 1 — the one its own spec calls **"do LAST — riskiest"** and the
one [cut 004](004-hotpath-rc-callable.md) left a measured **+5–7% send-path
regression** waiting on. Lands the **callable half** (S1a). `ip` is deliberately not
hoisted; see [The guard](#the-guard--why-closure_id-and-not-the-frame).

## The cost

`run_until_inner` re-derived the executing chunk from `self.heap` on **every opcode**:

```rust
let opcode = self.heap.closure(closure_id).callable.chunk.code[ip];
```

`heap.closure(id)` is a `SlotMap` lookup — bounds check + generation check + enum
match — followed by an `Rc` deref to reach `.chunk`. **And 19 arm bodies re-did the
same chase**, because each needed `constants`, `caches` or `gcaches`:

| arm | what it re-derived |
|---|---|
| `Constant`, `Closure`, `Method`, `DefineGlobal`, `Class`, `Import`, … | `chunk.constants[idx]` |
| `GetGlobal`, `SetGlobal` | `chunk.constants[idx]` **and** `chunk.gcaches[ip]` (×2 each: probe + refill) |
| `Invoke` | `chunk.caches[ip]` probe, `chunk.constants[sel]`, `caches[ip]` refill |
| `SuperSend` | `chunk.constants` ×2, `chunk.spans[ip]` |

So a `GetGlobal` paid the chase **three times** in one instruction. This is F14's items
**#3 and #6**, and it is why `run_until_inner` has held 33–39% of leaf ticks across
every cut and every workload: a switch-dispatch loop collapses every arm into one
frame, so `sample` books the cost there and can never name it.

Wren, by contrast, hoists `ip`/`fn`/`stackStart`/`fn->code` into C locals via
`LOAD_FRAME()`/`STORE_FRAME()` and reloads **only on call/return**
(`wren_vm.c:832-862`).

## The borrow problem, and why cut 004 is the reason this is `unsafe`-free

`u22-seq-spec.md` §4 names the obstacle exactly: **a `&Chunk` borrowed out of
`self.heap` cannot live across the `&mut self` calls the arms make.** It lists three
resolutions and recommends (a): hold an **`Rc<Callable>` clone in a local**.

That works because the borrow is then of a *local*, not of `self.heap` — the two are
disjoint, so the arms keep full `&mut self`. The `Rc` also keeps the `Callable` alive
independently of the heap, so a GC cycle mid-frame cannot invalidate it.

**This is cut 004's payoff.** 004 changed `ClosureObject.callable` to `Rc<Callable>`
(`heap/closure.rs`) and booked a **+5–7% send-path regression** for a per-instruction
`Rc` hop, explicitly noting "Change 1 (chunk hoist) is what repays it." It does:
`Rc::clone` here is **per frame change**, not per opcode — a refcount bump on call and
return only.

Options (b) index-based access and (c) raw `*const Chunk` were not needed. **No
`unsafe`** (spec §7: "no `unsafe` without explicit sign-off" — none required).

## The cut

Declare the hoist above the loop (`dispatch.rs:403`):

```rust
let mut hoisted: Option<(ObjRef, Rc<Callable>)> = None;
```

Refresh it with a one-compare guard per instruction (`dispatch.rs:444-450`):

```rust
let callable = match &hoisted {
    Some((id, callable)) if *id == closure_id => callable,
    _ => {
        let callable = Rc::clone(&self.heap.closure(closure_id).callable);
        &hoisted.insert((closure_id, callable)).1
    }
};
let opcode = callable.chunk.code[ip];
```

Then all 19 arm sites became `callable.chunk.…` (21 uses at HEAD). `caches`/`gcaches`
still write through the shared borrow — they are `Cell<Option<…>>` (`chunk.rs:49,54`),
so interior mutability works through `&Chunk` and the IC/global-cache refills compile
unchanged.

New import: `use crate::callable::Callable;` (`dispatch.rs:4`).

## The guard — why `closure_id`, and not the frame

**`ip` and `stack_offset` are still read from the live frame every opcode**
(`dispatch.rs:421`). That is not an oversight, it is the correctness argument:

> The guard is sound **only because the hoisted state is a pure function of the
> closure.** A chunk is. `ip` is not.

A `ClosureObject`'s chunk is fixed at construction, so *any* path arriving with the
same `closure_id` — **including a fiber switch, which swaps `self.frames` wholesale** —
is entitled to the same chunk. `ip` is re-read from whatever frame is now live, so a
switch into the *same* closure at a *different* `ip` still executes correctly.

**Two fibers suspended in the same closure at different `ip`s compare EQUAL under this
guard.** Hoisting `ip` behind it is precisely the stale-across-fiber-switch bug
`u22-seq-spec.md` §4 calls "the classic bug this unit could ship". S1b needs a
**frame-identity** guard — `CallFrame.generation` / `FrameToken` already exist
(`frame.rs`, ADR-0013) — not this one. The comment at the guard site says so.

### Soundness precondition, verified

The guard assumes `closure_id` determines the callable. Checked at HEAD, all zero:

| probe | hits |
|---|---|
| writes to a `ClosureObject`'s `.callable` field | **0** |
| `heap.closure_mut` (no such accessor exists) | **0** |
| `Rc::get_mut` / `Rc::make_mut` anywhere in `phalcom-core` | **0** |

So a `Callable` is never mutated post-construction and a closure's callable is never
reassigned — spec §7's reviewer condition ("nothing mutates a `Callable`
post-construction", 004's whole soundness argument) still holds.

## Method

Alternating same-session A/B vs `5516504` (`REPS=5`; 3 for the fiber workloads), both
binaries built before any timing, stdout byte-compared every run.

## Result

| Benchmark | base `user` | S1a `user` | Δ | pairs |
|---|---|---|---|---|
| `arith_send` | 0.030 s | 0.023 s | **−22.3%** | `-----` |
| `bare_send` | 0.034 s | 0.029 s | **−16.7%** | `-----` |
| `for` | 0.654 s | 0.569 s | **−12.9%** | `-----` |
| `variadic_send` | 0.630 s | 0.557 s | −11.6% | `-----` |
| `method_call` | 0.481 s | 0.430 s | −10.5% | `-----` |
| `skynet` | 1.74 s | 1.62 s | **−6.9%** | `---` |
| `fiber_churn` | 0.21 s | 0.20 s | −4.8% | `---` |

**25 of 25 pairs negative** on the send/loop rows, 3 of 3 on both fiber workloads.
All 9 wren-suite rows re-verified `ok` — stdout byte-identical to Wren's. Suite band
**1.5–13.7× → 1.1–10.7×**; skynet **2.9× → 2.7×** Wren.

**Fiber-switch evidence.** skynet (1.11M fibers) and `fiber_churn` (500k spawn→drop)
both produce byte-identical output, and `tests/` fiber+concurrency suites are green.
Those are the workloads that would expose a stale hoist across a switch.

**Instruction counts are unchanged, to the digit** — `for` 68,000,706, `method_call`
49,334,099, `bare_send` 3,200,683, identical to the counts recorded at `45ffe76`. That
is stronger behavior-invariance evidence than the stdout diff: identical output says
two runs agreed on the answer; identical counts say they agreed on **every step taken
to reach it**. A dispatch cut that accidentally altered control flow would have to be
invisible at both.

## Caveats

**RSS moved and is not claimed.** skynet fell 1.280 → 1.192 GB (−6.9%). A dispatch
hoist allocates nothing less, so this has no mechanism on its face; suspect GC
*scheduling* (a faster loop moves safepoints; F11's yield-adaptive threshold sizes
`next_gc` from reclaim yield). Filed as **H14** — the number is a measurement, the
mechanism is a hypothesis.

**It falsifies F15's identity claim.** F15 reads skynet's RSS ratio as *being*
`Value`'s 2.0× ("a fiber is a stack of `Value`s… not a coincidence to be re-derived").
This cut changed no representation — `Value` is still 16 B vs Wren's 8 B — and the
ratio moved to ~1.8× anyway. `Value` sets a **floor** on fiber density; it does not
determine peak RSS. See [SCOREBOARD §2](SCOREBOARD.md#2-memory--peak-rss-vs-wren).

**`fiber_spawn` barely moved** (−4.6% on criterion, where every other row moved
14–28%). Expected: its ~569 ns is fiber *construction* — allocation, buffer growth —
not dispatch. Consistent with [F17](findings.md#f17--an-instruction-costs-1013-ns-and-the-28-spread-is-per-instruction-work-not-instruction-count):
fiber rows cost 24–25 ns/instr against tight loops' 10–13, because their instructions
do more *work*. **A loop cut cannot reach per-instruction work** — which is also why
[F18](findings.md#f18--presizing-the-fiber-vecs-is-negative-and-f3h9s-memmove-lever-is-spent-206--30)'s
fiber-allocation lever had to be attacked separately (and failed).

## What remains of S1

F14 sized S1 at *est* −30–45%/send for items #2+#3+#5+#6 together. S1a booked
−10.5…−22.3% and took **both heap-chase items (#3, #6)**. What is left is the cheap
half: the 96 B `CallFrame` copy (#2, `dispatch.rs:421`) and the `frames` re-index
(#5). **Do not expect the arithmetic remainder** — the estimate was never re-derived
at HEAD, and F18 is this session's demonstration of what an un-re-derived estimate is
worth (*est* −10–15% → **+2.4%** measured, a sign error).

## Write-set

- `phalcom-core/src/vm/dispatch.rs` — `use crate::callable::Callable`; `run_until_inner` hoist + guard; 19 arm sites.

No other file. No new goldens (behavior-invariant). Floor: +0. No `unsafe`.
