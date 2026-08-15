# Negative result — presizing the fiber `stack`/`frames` `Vec`s

Status: **built, measured, reverted — do not re-attempt as written** · Probed at `5254586`, 2026-07-14 · Finding: [F18](findings.md#f18--presizing-the-fiber-vecs-is-negative-and-f3h9s-memmove-lever-is-spent-206--30) · Closes [H9](SCOREBOARD.md#6-open-holes--what-is-empty-and-how-to-fill-it)

This is **not** a landed cut. It is recorded at cut-level detail for one reason:
[F5](findings.md#f5--fiber-stack-pool-implemented-measured-reverted-null-result) lost
its reverted implementation entirely — *"never staged, so it never entered the git
object DB… Only the design survives — ~1h to rebuild from it."* The probe below is
~4 lines and is reproduced verbatim so nobody pays that hour, or re-runs the
experiment believing it is new.

## What was claimed

[README lever 7](README.md#next-measured-levers) ranked this **"the highest
gain-per-effort item on the whole list"**: ~10 lines, *est* **−10–15% skynet `user`**,
open since origin. [F15](findings.md#f15--value-is-2-wrens-and-objref-blocks-nan-boxing)
carried it as the top rung of its fiber ladder. The premise, from
[F3](findings.md#f3--memmove-206-skynet-is-vec-growth-not-memtake):

> every fiber starts `stack: Vec::new()` / `frames: Vec::new()` (capacity 0), and each
> push past current capacity triggers a `memmove` … growth 4→8→16 = **two reallocs +
> two `memmove`s per fiber, 1.11M times**.

Both halves of that premise are **true and still true at HEAD**:

| claim | verified at `5254586` |
|---|---|
| fibers start at capacity 0 | `heap/fiber.rs:129-130` (`new_entry`), `:172-173` (`root`) — `Vec::new()` |
| the compiler knows max stack depth | `Callable.max_slots` exists (`callable.rs:25`) and is unused by fiber construction |
| `memmove` comes from `Vec` growth | profile shows `_platform_memmove` under `_realloc` → `xzm_realloc` |

**The premise was never the problem. The size was.**

## Nail 1 — the re-profile, which H9 existed to demand

`sample` skynet @ `5254586`, leaf frames, 677 listed ticks (≥5, collapsed):

| mechanism | ticks | share | at origin (F1) |
|---|---|---|---|
| `vm::dispatch::run_until_inner` | 264 | 39% | 27.7% |
| `heap::trace::trace_object` | 107 | 16% | — |
| `vm::send::call_method` | 76 | 11% | 3.7% |
| malloc/free family | ~96 | ~14% | 28.2% |
| `Value::class` | 25 | 4% | — |
| **`_platform_memmove`** | **20** | **~3.0%** | **20.6%** |
| `slotmap::SlotMap::try_insert_with_key` | 18 | 3% | 2.2% |

**`memmove`: 20.6% → 3.0%.** Cuts 001/002/004 + U-GC rebuilt the allocation landscape
around it. The −10–15% estimate was extrapolated from origin's 20.6% and **never
re-derived across the four cuts that invalidated it** — which is exactly what H9 said
("its current share is unknown") and exactly why it was open. **The ceiling was ~3%
before a line was written.** One `sample` run, ~2 minutes, would have said so at any
point in the preceding four cuts.

## Nail 2 — the probe (verbatim, so it is not lost)

The constant probe that kills F3's 4→8→16 growth. Applied to
`FiberObject::new_entry`, `phalcom-core/src/heap/fiber.rs:128-130`:

```diff
         #[cfg(not(feature = "fiber-pool"))]
         Self {
-            stack: Vec::new(),
-            frames: Vec::new(),
+            stack: Vec::with_capacity(16),
+            frames: Vec::with_capacity(4),
             open_upvalues: BTreeMap::new(),
             status: FiberStatus::Suspended,
```

`root()` (`:172-173`) was left alone — the root fiber's live buffers are the VM's own
`frames`/`stack` mirror, so its fields stay empty while it runs.

A/B vs `5254586`, best-of-3, `/usr/bin/time -l`, output byte-identical both arms:

| workload | base `user` | presized | Δ `user` | base RSS | presized | Δ RSS | pairs |
|---|---|---|---|---|---|---|---|
| `skynet` | 1.650 s | 1.690 s | **+2.4%** | 1306 MB | 1233 MB | −5.6% | `+++` |
| `fiber_churn` | 0.200 s | 0.240 s | **+20.0%** | 263 MB | **581 MB** | **+121.3%** | `+++` |
| `fibers` | 0.080 s | 0.090 s | **+12.5%** | 114 MB | 141 MB | **+23.2%** | `+++` |

**Slower on all three, catastrophic on memory under turnover.** Every pair positive.
Reverted (`git checkout -- phalcom-core/src/heap/fiber.rs`).

## Mechanism — this is F10's failure, reached from the other side

Presizing buys **~640 B per fiber eagerly** (16 × 16 B `Value` + 4 × 96 B `CallFrame`)
where growth previously fitted actual need. A fiber shell is **GC-lifetime**: it
outlives its own run until the collector sweeps it. So the capacity is *retained*, not
reused.

[F10](findings.md#f10--the-fiber-pool-is-not-neutral-it-is-negative--and-f5-measured-the-wrong-workload)
measured the identical shape at **~450 B/fiber, +72–86% RSS, +37% `user`** at 1M
fibers, and named the mechanism: *"recycled capacity is being retained per fiber, not
reused… a buffer drawn from the pool carries a previous fiber's grown capacity into a
shell that outlives its own run."*

F10 handed a shell a **pooled** buffer's capacity. This hands it a **presized** one.
**The shell does not care where the capacity came from.**

⇒ **Any per-fiber eager allocation is negative on this object model while shells are
GC-lifetime.** Measured twice now, by two independent mechanisms. This is the
generalization neither F5, F10 nor F3 reached — each blamed its own lever. **The
problem was never the pool, and it is not the presize: it is that a fiber shell
outlives its run.**

Why it is *slower* too, not merely fatter: two eager `malloc`s of larger blocks per
fiber, against a growth path that for a short fiber may allocate once or not at all —
plus the page-fault cost of touching them (F10 saw the same `sys` inflation).

**skynet's −5.6% RSS is the lone non-negative cell and does not rescue it.** Skynet's
fibers are all live, so presizing over-allocates uniformly rather than churning, and
the GC schedule shifts under it — cf. **H14**, where a *reverted-change-free* dispatch
cut moved skynet RSS by the same ~6% with no allocation mechanism at all. Treat
single-digit skynet RSS deltas as unattributed until H14 resolves.

## What a revival would have to be

Not this, and not a better constant:

- **`max_slots`-sized presize is buildable** (`callable.rs:25`; the entry closure is
  reachable from `primitive::fiber::new_fiber_ref`) and would over-allocate less. It
  **does not rescue the item**: the time ceiling is `memmove`'s ~3%, and the RSS
  mechanism is per-fiber *retention*, which right-sizing shrinks but does not remove.
  Not worth a unit at a ~3% ceiling.
- **The only lever that changes the answer is the shell's lifetime** — release or
  shrink `stack`/`frames` at `FiberStatus::Done`, before the sweep, so capacity dies
  with the run instead of with the object. That is a GC/lifetime change, not a
  capacity tweak, and it is unexercised.

## Transferable

**An estimate is not a measurement, and a stale profile is not a profile.** *est*
−10–15% → **+2.4%** measured is a **sign error, not a magnitude error**. It survived
four cuts as "highest gain-per-effort item on the whole list" because the ranking was
never re-derived — and the hole that said so (H9) sat open next to it the whole time.

F14's S1–S4 estimates are the same species of number. S1a and S2 have now been
measured ([007](007-hoist-rc-callable.md), [006](006-dispatch-drop-spans.md)); **S3
and S4 have not.**
