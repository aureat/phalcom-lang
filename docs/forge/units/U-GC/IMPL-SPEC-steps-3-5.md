# U-GC — implementation spec for steps 3–5

_Companion to [`plan.md`](plan.md), written 2026-07-14 against HEAD `bbc12d6`. Steps 0–2 are
**done and committed**; this spec covers **only** what remains. It is deliberately more
prescriptive than the plan: exact signatures, exact insertion points, exact constants, and the
**already-completed audit result** — so the implementer writes code rather than re-deriving
analysis._

> **Read this before `plan.md` §3.4/§5.** The plan's framing of step 4 is **wrong about where the
> hazard is**, and following it would produce a large pointless audit plus an unused API. §2 below
> replaces it. `plan.md`'s §3.4 is retained for provenance, not for execution.

## 0. Status — what is already done

| Step | State | Commit |
|---|---|---|
| 0 — regenerate roots + edge table | **done** | `f0a8a1d` |
| 1 — Win A, box six fat variants (280 B → 40 B) | **done** | `7480d75` |
| 2 — `trace_object` / `Heap::collect` / `VM::collect_roots` / `force_gc` + 6 GC tests | **done** | `e9fdd96` |
| **3 — `System.gc`** | **TODO** | §1 |
| **4 — safepoint latch** | **TODO** | §2 |
| **5 — fiber pool re-measure** | **TODO** | §3 |

DEC-GC-A is **resolved by the user: option A** — ADR-0050 is Accepted, collection ships **on by
default**, no `gc_enabled` soak flag. There is no flag to hide a missed root behind.
DEC-GC-B → **(A) count-based**. DEC-GC-C → **(B) split** (§3). DEC-GC-D → **(B) per-collection local** (done).

Existing API to build on (all landed, all documented):

```rust
Heap::collect(&mut self, roots: &[ObjRef]) -> usize   // objects swept
Heap::live_count(&self) -> usize
Heap::try_get(&self, id) -> Option<&Object>           // None == swept (M6)
VM::collect_roots(&self, out: &mut Vec<ObjRef>)       // exhaustive destructure — DO NOT bypass
VM::force_gc(&mut self) -> usize                      // unconditional collect
```

---

## 1. Step 3 — `System.gc`

**Correction to `plan.md` §3.5:** it says `gc` is "currently a spec'd no-op stub." **It is not a
stub — it does not exist.** `phalcom-core/src/primitive/system.rs` has no `gc` function and
`universe/primitives.rs` has no `gc` registration. Sending `System.gc` today raises
`doesNotUnderstand`. This step *adds* the primitive; it does not rewire one.

Spec: [`system.md:65`](../../../spec/v0.2/system.md) — "`gc` | request a garbage collection;
returns `None`". Note `System`'s primitives are **static** (`System.gc`, not `aSystem.gc`).

### 1.1 Write

`phalcom-core/src/primitive/system.rs` — mirror the shape of the neighbouring
`system_next_scheduled`:

```rust
/// Signature: `System.gc` — forces one full mark-sweep and returns `None`
/// (`system.md` §`gc`, [ADR-0050](../../../docs/adr/0050-non-moving-mark-sweep-collector.md) §8).
///
/// Runs **no finalizers**, performs **no compaction**, and changes **no handle**
/// (Invariant M1) — a surviving object keeps its `ObjRef`. Deterministic and safe
/// to call from `.ph` code: a primitive runs at a dispatch safepoint by
/// construction, where `VM::stack`/`frames` are the complete root truth
/// ([memory-management.md §4](../../../docs/spec/v0.2/memory-management.md)).
///
/// Returns `None` rather than the swept count because `system.md` §`gc` says so;
/// the count is available to Rust via [`VM::force_gc`].
///
/// # Errors
///
/// Infallible; returns [`PhResult`] to match the primitive ABI.
pub fn system_gc(vm: &mut VM, _receiver: &Value, _args: &[Value]) -> PhResult<Value> {
    vm.force_gc();
    Ok(vm.none_value())
}
```

`phalcom-core/src/universe/primitives.rs` — register beside the other `system_cls` statics
(~line 249), and add `system_gc` to the `use crate::primitive::system::{…}` list at ~line 31:

```rust
primitive_static!(vm, system_cls, "gc", SignatureKind::Getter, system_gc);
```

**`SignatureKind::Getter` is correct, not `Method(0)`** — `System.gc` is written without
parentheses, matching `nextScheduled` (line 249), which is the precedent to copy. Getting this
wrong makes `System.gc` a dNU while `System.gc()` works.

### 1.2 Why this is safe without temp roots

A primitive is invoked from `call_method`, which is invoked from the dispatch loop — so
`vm.stack`/`frames` already hold every live handle, and after cut 001 the receiver and args
remain **on** `vm.stack` for the primitive's whole duration (they are *copied* into an on-stack
buffer, not popped). `system_gc` allocates nothing and holds nothing. See §2.2.

### 1.3 Test

Add to `phalcom-core/tests/gc.rs`, plus a golden `.ph` fixture if the lane has one for `System`:

- `System.gc` returns `None` — assert against `vm.none_value()`, i.e. handle-identity with the
  `none_singleton`, **not** a rendered string.
- Observable free: from `.ph`, build unreachable garbage in a loop, call `System.gc`, assert
  `live_count` dropped. Drive via `interpret_source` and read `vm.heap.live_count()` around it.
- `System.gc` twice in a row is a no-op the second time (idempotent; nothing left to sweep).

**Baseline trap:** use the `settled_vm()` helper already in `tests/gc.rs`. A fresh VM is **not**
garbage-free — `core.ph`'s top-level closure is unreachable the moment bootstrap returns, so the
first collection on any VM legitimately sweeps one object (finding F8). Four step-2 tests failed
off-by-one on exactly this.

---

## 2. Step 4 — the safepoint latch (**the gating commit**)

### 2.1 The audit is done, and its result inverts the plan

`plan.md` §3.4 calls for auditing "≈46 re-entrant sites, 40 alloc sites … each such handle gets a
`push_temp_root`/`pop_temp_root` scope," and calls this "the substantive work of the unit."

**That audit has been run in full. The intersection is EMPTY. Zero sites need a temp root.**

Method and evidence:

- **All 6 `run_until` call sites in the crate** — `interpret.rs:270`, `primitive/block.rs:159`,
  `vm/dispatch.rs:202`, `vm/send.rs:234`, `vm/send.rs:277` (+ the `run` entry). These are the
  *only* places a safepoint can fire beneath a Rust frame. Every one of them **pushes its
  receiver and arguments onto `vm.stack` before re-entering**:
  - `block_call` — `vm.stack.push(*receiver); vm.stack.extend_from_slice(args);` then `run_until`.
  - `send_dynamic` / `invoke_method_object` — identical shape.
  - `import_module` — `module_id` is inserted into `universe.module_registry` (a root) *before*
    compiling, and `closure` is written into `ModuleObject::closure` **and** into the pushed
    `CallFrame` before `run_until`.
  - `run` — top level, nothing held.
- **Every function in `phalcom-core/src` containing both an allocation and a re-entrant call** —
  exactly two in `primitive/` (`bool_if_true`, `bool_if_false`), and in both the allocation is
  `wrap_some` **after** `block_call(…)?` returns. Nothing is held across.

So the hazard class the plan anticipated **does not occur**. Do not build `VM::temp_roots`,
`push_temp_root`, or `pop_temp_root` in this step: with zero call sites they are dead scaffolding,
and dead scaffolding is what this project already reranked `world_version` out of the schedule for.

### 2.2 The hazard that *does* exist, and why the latch is load-bearing correctness

There is a real, pervasive unrooted window — but it is **fresh handle across a subsequent
ALLOCATION**, not across a re-entrant send. Representative sites, all in Rust:

| Site | The window |
|---|---|
| `vm/send.rs` `forward_does_not_understand` | `let args = stack[i+1..].to_vec(); stack.truncate(i+1);` — **arguments are now off the stack** — then `new_message(selector, &args)` **allocates ~6 times** before the result is pushed. |
| `vm/send.rs` `new_message` | `alloc_string_value(name)` → N× `alloc_string_value(label)` → `alloc_list` → `alloc_list` → `alloc(Instance)`. Every intermediate lives only in a Rust local across the next allocation. |
| `vm/send.rs` `call_method` (varargs prologue) | `let rest = self.stack.split_off(…)` — **split_off removes them from the stack** — then `alloc_list(rest)`, then push. |
| `vm/dispatch.rs` `Bytecode::Closure` | `alloc(Closure)` → `alloc(Block)` → push. `new_closure` is unrooted across the second allocation. |
| `vm/dispatch.rs` `capture_error_value` | `inst.slots[0] = alloc_string_value(…)` on a **Rust-stack** `InstanceObject`, then `alloc(Instance)`. |
| `vm/dispatch.rs` `Bytecode::WrapSome` | `stack.pop()` → `wrap_some` (allocates) → push. |
| `vm/dispatch.rs` `Bytecode::MakeFamily` | `stack.pop()` → lookups → `alloc(Family)` → push. |

Each is safe **if and only if `Heap::alloc` never collects**. This is the central invariant of
this step:

> **L. `Heap::alloc` latches; it never collects.** Allocation may only *set* `gc_pending`.
> Collection happens exclusively at the dispatch-loop back-edge safepoint.

If collection ever moves into `alloc` — as an "optimization," or by someone calling `force_gc`
from a new allocation helper — every row above breaks **simultaneously and silently**: the
`doesNotUnderstand` path would build a `Message` out of dangling handles. Treat L as the
deliverable of this step. It is not hygiene; it is the whole reason the audit came out empty.

### 2.3 Write

**`phalcom-core/src/heap/mod.rs`** — add to `Heap`:

```rust
pub struct Heap {
    objects: SlotMap<ObjRef, Object>,
    /// Live-object count at which `alloc` next latches `gc_pending`.
    next_gc: usize,
    /// Set by `alloc` when `objects.len()` crosses `next_gc`; **serviced only** at
    /// the dispatch back-edge safepoint. See Invariant L.
    gc_pending: bool,
}
```

- `Heap::new()` / `Default` → `next_gc: INITIAL_GC_THRESHOLD`, `gc_pending: false`.
- `const INITIAL_GC_THRESHOLD: usize = 4096;` — comfortably above the ~694 objects a bootstrapped
  VM settles at (measured), so bootstrap never trips a collection.
- `const GC_GROW_FACTOR: f64 = 1.5;` — Wren's `nextGC = live * heapGrowFactor`.
- In `alloc` **and every `alloc_*` helper** (they call `objects.insert` directly today — route
  them all through one private `fn insert(&mut self, o: Object) -> ObjRef` so the latch cannot be
  bypassed by a future helper):

  ```rust
  fn insert(&mut self, object: Object) -> ObjRef {
      let id = self.objects.insert(object);
      if self.objects.len() >= self.next_gc {
          self.gc_pending = true;   // LATCH ONLY — never collect here (Invariant L)
      }
      id
  }
  ```
- `pub fn gc_pending(&self) -> bool`.
- At the end of `collect`, retune: `self.next_gc = max(INITIAL_GC_THRESHOLD, (live as f64 * GC_GROW_FACTOR) as usize); self.gc_pending = false;`

**`phalcom-core/src/vm/gc.rs`** — add:

```rust
/// Services a latched `gc_pending` — **safepoint only**.
///
/// Call this exclusively from the dispatch-loop back-edge, where `VM::stack`/
/// `frames` are the complete root truth. Never from `Heap::alloc` (Invariant L,
/// memory-management.md §4), and never mid-opcode: several opcodes have a window
/// where a value is popped or `split_off` the stack and held only in a Rust local.
pub(crate) fn service_gc_safepoint(&mut self) {
    if self.heap.gc_pending() {
        self.force_gc();
    }
}
```

**`phalcom-core/src/vm/dispatch.rs`** — service at the loop head of `run_until_inner`. The exact
insertion point is **immediately after** the `frames.len() <= base_frames` exit check and
**before** `let frame = *self.frames.last().unwrap();` (currently ~line 384):

```rust
// Safepoint (memory-management.md §4): the *only* place collection runs. Here
// `stack`/`frames` are coherent — no opcode is mid-flight with a value popped
// into a Rust local. Servicing before reading `frame` is deliberate: a
// non-moving collector cannot invalidate the `CallFrame` we are about to copy,
// but keeping the whole read-decode-execute sequence GC-free is what makes that
// independent of the collector's future shape.
self.service_gc_safepoint();

let frame = *self.frames.last().unwrap();
```

**This placement is correct for re-entrancy without further thought**: `run_until_inner` is
re-entrant, but `vm.stack`/`vm.frames` are *one shared pair* across all nesting levels, so an
inner back-edge sees the outer frames' roots too. It is also correct across a fiber switch: at the
loop head the mirror is coherent (`vm.current` and `vm.stack`/`frames` agree), because switches
complete inside a primitive and set `switch_pending` before returning to the loop.

### 2.4 Tests (add to `phalcom-core/tests/gc.rs`)

1. **Invariant L — `alloc` never collects (THE test).** Allocate one unrooted object; keep its
   handle; then allocate `> INITIAL_GC_THRESHOLD` further objects (crossing the latch) **without
   running any bytecode**; assert the first object is **still alive** (`try_get(h).is_some()`) and
   `gc_pending()` is `true`. This encodes L directly: it fails the instant someone makes `alloc`
   collect. Name it so the failure is self-explaining, e.g. `alloc_latches_but_never_collects`.
2. **The safepoint fires.** Run a `.ph` loop that churns unreachable objects past the threshold
   via `interpret_source`; assert `live_count()` stays **bounded** (does not grow linearly with
   iterations) and `gc_pending()` is `false` afterwards.
3. **Bounded churn, real workload.** `benchmarks/wren-suite/for.ph` shape — a `for (x in list)`
   over a large `List` allocates a `Some` per step (this is still true; Route B is unlanded).
   Assert live count is bounded across the loop instead of growing with iteration count. This is
   the proof the collector reclaims per-step `Option` churn, not just externally-rooted garbage.
4. **`System.gc` still works** post-latch, at any safepoint.
5. **Suspended fiber roots its stack.** A parked fiber whose saved `stack` holds the only
   reference to X: force a collection; X survives while the fiber is reachable, and dies once the
   fiber is not. (Deferred from step 2 — it needs real fiber execution, which needs the loop.)
6. **`verify_invariants()` post-GC kernel assert** — already covered by
   `kernel_survives_collection`; extend it to run *after* an automatic collection rather than only
   a forced one.

### 2.5 Verification gate

- `./scripts/verify.sh` exits 0; `cargo doc --workspace --no-deps` adds no warnings.
- **Clippy delta must be empty** vs the pre-change tree — compare sorted `^warning:` lines, not a
  raw count. There are 11 pre-existing warnings, all in files this unit does not touch.
- Test suite green **except** `indexing` / `indexing_negative`, which are pre-existing (a `[]()`
  subscript gap) and confirmed identical with the change stashed. Verify that, don't assume it.
- **No `unsafe`.** Hard constraint (ADR-0009/0050).
- Run the `rust-sanitizers-miri` lane over mark/sweep.
- **Reviewer ON** (`vm/`, `heap/` are spine files) — `phalcom-reviewer` gates; do not self-approve.

### 2.6 Perf check — expect a small regression, measure it honestly

The safepoint adds one predictable branch per opcode. Measure `bare_send`/`arith_send`
(`benchmarks/vm/`, criterion) **and** `for.ph`/`skynet` (`/usr/bin/time -l`, watch `sys`).

**Read the perf-log's method section first** (`docs/forge/perf-log/README.md`) — it exists because
cut 002 was nearly abandoned on a criterion micro-bench that was measuring the wrong axis:

- Criterion's p-value covers **within-run variance only**. On this machine it certified noise as
  `p = 0.00` twice; the *same binary* against the *same saved baseline* reported +8.8% then +1.3%.
- For effects under ~10%, use alternating same-session A/B and read the **sign across pairs**.
- **Never run `cargo build` inside a measurement loop** — it contends with the bench.
- The collector's real payoff is on allocation-heavy workloads (`for.ph`, `skynet`), where it
  should now *win* on wall time by bounding the arena instead of growing it forever. Report both
  axes; do not suppress the micro-bench regression.

---

## 3. Step 5 — fiber-stack pool re-measure

DEC-GC-C is settled at **(B) split**, on evidence rather than preference. The pool was built,
A/B'd against Skynet, and **reverted as a null result** (finding F5): Skynet RSS was dominated by
~1M immortal `FiberObject` shells that only reclamation can free, so pooling their buffers could
not move it.

**Do not re-land it blind.** The sequence dependency is the whole point: it is only measurable
once sweeping frees the shells. Now that the collector exists:

1. Re-measure the *unpooled* baseline on `skynet` post-collector first. The collector may already
   have taken the win, in which case the pool has nothing left to buy.
2. Only if a gap remains, rebuild from the F5 design (the code is **unrecoverable** — never
   staged, absent from the object DB; ~1h to rebuild from the writeup): a bounded free-list on the
   VM; acquire in `new_fiber_ref` (`primitive/fiber.rs`); recycle at `FiberStatus::Done`
   (`vm/dispatch.rs`) **before** the resumer's `load_live_from` drops the buffers; `Done` path
   only, leaving park/`Failed` alone.
3. Gate on a **high-fiber-turnover** benchmark (rapid spawn→Done→respawn) on a quiet machine. No
   current benchmark exercises that shape; `fiber_spawn` in `benchmarks/vm/` is the closest.
4. Note the post-collector trade changed: sweeping now frees a dead fiber's `Vec`s, so the pool
   competes against **malloc**, not against a leak. That is a much smaller prize than F5 assumed.

If it does not measure, **do not ship it** — record a second null result. That is the same call
F5 made and it was right.

---

## 4. What must this not preclude (re-checked at `bbc12d6`)

- **Generational collection** — *served, not precluded*. The one enabler to add **now** is
  funnelling allocation through the single private `Heap::insert` (§2.3), which gives a future
  write barrier one home on the alloc side. Flag any mutation site that bypasses the accessors.
- **Incremental / tri-color** — not precluded; `SecondaryMap<ObjRef, ()>` generalises from `()` to
  a colour, the worklist becomes the gray set.
- **Compaction / moving** — reversibly open; would need a handle→slot indirection to preserve M1.
  This step neither builds nor blocks it.
- **NaN-boxing (ADR-0010)** — not precluded. `Value::as_obj()` is the sole seam and is already
  landed; the collector never matches `Value`'s arms.
- **Inline caches (ADR-0012, U-IC)** — actively *protected*: non-moving collection keeps
  `ClassId`/`ObjRef` IC tags valid across a collection (M1), so U-IC needs no GC-invalidation story.
- **`superclass=` (Q4)** — not precluded; stop-the-world needs no write barrier.
- **The Runtime decorator tier (M-RUNTIME)** — **this is where `temp_roots` will actually be
  needed.** `aroundSend` allocates an `Invocation` per intercepted send *and* re-enters the
  interpreter: the first genuine instance of the hazard class §2.1 found empty today. Record in
  `DEFERRED.md` that M-RUNTIME must introduce `push_temp_root`/`pop_temp_root` **with** its first
  call site, and must not be written against the assumption that the audit stays empty.

## 5. Return contract

Report: the `System.gc` registration (`SignatureKind` used) · the exact safepoint insertion point ·
confirmation that **every** `alloc_*` helper routes through the single latching `insert` · the
Invariant-L test and what it asserts · live-count-bounded results on the `.ph` churn loop and on
`for.ph` · before/after `bare_send`/`arith_send` **and** `for.ph`/`skynet` wall+`sys` · the
`verify_invariants` post-automatic-GC assert · confirmation of **no `unsafe`, no surface/`Value`/
opcode change** · miri lane tail · the step-5 measurement and whether the pool shipped or produced
a second null result · a `DEFERRED.md` pointer for M-RUNTIME temp-roots · and a note that
`system.md` §`gc` / `PHASE2-INDEX.md` / `STATE.md` need a **separate** ledger-sync commit, never
this code diff.

## 6. Standing hazards for whoever implements this

1. **Do not reintroduce a hand-audit.** `VM::collect_roots`, `Universe::each_handle` and
   `CoreClasses::each_handle` are **exhaustive destructures**; `trace_object` is a **wildcard-free
   match**. A new field must fail the build. Three roots (`sealed_classes`, `checking`,
   `ready_queue`) were missed by hand-auditing the spec table — the last of them *after* a
   dedicated audit pass, because the audit's grep matched `pub` but not `pub(crate)`. If you find
   yourself adding `_ => {}` or reading fields individually, stop.
2. **A missed root frees a live object** (M3) — silent, arbitrarily delayed. An extra non-root
   merely over-retains. **When unsure, root it.**
3. **An off-by-one live count is ambiguous.** It looks identical to a missed root. The only way to
   tell correct collection from an M3 violation is to identify the object and prove nothing holds
   it — reverse-scan the arena with `Heap::iter_handles_for_test` / `kind_of_for_test`, tracing
   every live object and asking who points at the suspect. That probe found F8; rebuild it rather
   than assuming your baseline is wrong.
4. **The `main` branch has live concurrent sessions.** Commit narrow explicit paths; never
   `git add -A`. `vm/` and `heap/` are spine files — check `git worktree list` for a dirty
   worktree touching them before starting.
