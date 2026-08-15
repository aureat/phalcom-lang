# U-GC — Work order: non-moving mark-sweep collector + Object-size win + fiber-stack pool

> ## ⚠ Steps 0–2 are DONE. For steps 3–5, read [`IMPL-SPEC-steps-3-5.md`](IMPL-SPEC-steps-3-5.md) instead of §3.4/§5.
>
> | Step | State | Commit |
> |---|---|---|
> | 0 — regenerate roots + edge table | **done** | `f0a8a1d` |
> | 1 — Win A (280 B → 40 B; `for.ph` −43%, `skynet` −34%) | **done** | `7480d75` |
> | 2 — collector behind `force_gc` + 6 GC tests | **done** | `e9fdd96` |
> | 3–5 | **TODO** → [`IMPL-SPEC-steps-3-5.md`](IMPL-SPEC-steps-3-5.md) | — |
>
> **§3.4 below is wrong about where the hazard is, and following it wastes the unit's budget.**
> It calls for auditing "≈46 re-entrant sites, 40 alloc sites" and giving each a
> `push_temp_root`/`pop_temp_root` scope, calling this "the substantive work." **That audit has
> since been run in full: the intersection is EMPTY — zero sites need a temp root**, because all
> 6 `run_until` call sites in the crate push their receiver and args onto `vm.stack` before
> re-entering. Building `temp_roots` now would be dead scaffolding.
>
> The hazard that *does* exist is a fresh handle held across a subsequent **allocation** (not
> across a re-entrant send) — pervasive in `new_message`, `forward_does_not_understand`,
> `call_method`'s varargs prologue, and four dispatch opcodes — and it is neutralised entirely by
> the safepoint latch. **`Heap::alloc` must latch and never collect** is therefore step 4's real
> deliverable. Evidence, sites, and the replacement spec: `IMPL-SPEC-steps-3-5.md` §2.
>
> §3.4 is retained below for provenance only.

_Self-contained implementation plan for **one** implementer. Runtime/heap unit — no surface, no parser,
no `core.ph` protocol. **Reviewer ON** (touches spine files `heap.rs`, `vm.rs`) — hand the diff to
`phalcom-reviewer`; do not self-approve. Green gate: `./scripts/verify.sh` exits 0 +
`cargo doc --workspace --no-deps` clean. Grounded in **[ADR-0050](../../../adr/0050-non-moving-mark-sweep-collector.md)**
and normative **[memory-management.md](../../../spec/current/memory-management.md) §1–§7**. Governing ADR:
ADR-0050 (Proposed — ratify before merge, or land behind the flag; see §8 DEC-GC-A)._

> **Provenance.** Design verified against HEAD 2026-07-13: `Heap = SlotMap<ObjRef, Object>` (`slotmap = "1"`
> → `retain`/`SecondaryMap` available), **zero `impl Drop`** on the object graph, `size_of::<Object>()` =
> **256 B measured**, roots fully reified (`VM::stack`/`frames` are owned `Vec`s), fibers carry
> heap-resident `Vec` stacks (no native stack). Does **not** edit `docs/forge/PHASE2-INDEX.md`,
> `docs/forge/STATE.md`, `system.md`, or `core.ph` — all shared/concurrent files.
>
> **Re-verified against HEAD 2026-07-14 (step 0) — five drifts, read before touching code:**
>
> 1. **Paths.** `heap.rs` → **`heap/`** (15 files: `mod.rs`, `object.rs`, `class.rs`, `fiber.rs`,
>    `instance.rs`, `accessors.rs`, …); `vm.rs` → **`vm/`** (`mod.rs`, `dispatch.rs`, `send.rs`, `api.rs`,
>    `bootstrap.rs`); `value.rs` → **`value/`**. The §4 write-set table below still names the old
>    single-file paths — read it as directories.
> 2. **`size_of::<Object>()` = 280 B, not 256 B — it grew.** `ClassObject` gained `attributes: Vec<Value>`
>    + `attributes_frozen` (U-ANNOT) and now *is* the whole 280 B. Win A is **six** variants, not "the
>    driver(s)" — see the measured ladder in [memory-management.md §7](../../../spec/current/memory-management.md),
>    which supersedes ADR-0050 §9's list. Boxing `Instance` (24 B) would be counterproductive.
> 3. **§2.3's edge table was stale and is now regenerated** (memory-management.md §2.3, 16 variants,
>    field-level). It had **two missed edges that would each free a live object**: `Block.closure` (the
>    variant was absent entirely) and `Upvalue::Open.fiber` (the spec asserted an `Open` cell was "already
>    traced as a root" — false: its slot may live on a *parked* fiber's stack). It also claimed two edges
>    that do not exist (`Class`'s "name string" is a Rust `String`; `Method` does not own a chunk — its
>    `MethodKind::Closure(ObjRef)` does). Five new edges landed post-plan: `Class.attributes`,
>    `Method.attributes`, `Method.contracts`, `Module.attributes`, `Fiber.checking`.
> 4. **§2.1's root set missed two roots:** `VM::sealed_classes: HashMap<Symbol, ObjRef>` (U-ANNOT-LAYOUT)
>    and `VM::checking: HashSet<ObjRef>` (U-ANNOT-CONTRACTS). Both now in the spec table. A missed root
>    frees a live object — the unit's one fatal failure mode.
> 5. **Audit scope grew:** alloc sites in `primitive/` **31 → 40**; re-entrant sites still 46. §3.4's
>    stated scope is updated accordingly.
>
> Preconditions re-confirmed green on HEAD: `slotmap = "1"` ✓ · zero `impl Drop` ✓ · roots reified
> (`VM::stack`/`frames`/`current`/`open_upvalues` are `pub(crate)` owned `Vec`s on `VM`) ✓ · fibers carry
> no native stack ✓ · `cargo build` clean ✓ · `ClassId` is a type alias for `ObjRef` ✓ (one handle type).

> **Empirical motivation** ([`benchmarks/wren-suite/for.ph`](../../../../benchmarks/wren-suite/for.ph),
> measured 2026-07-13, release build): a `for (x in list)` loop over a 1M-element `List` costs **~11-12s**
> regardless of loop body (empty-increment and `sum +=` both land there) — isolated by bisection against
> `list.add(i)` ×1M alone (0.42s, fine) and a plain scalar `while`-sum ×1M with no `List` (0.34s, fine). Root
> cause: `List#iterate(cursor)` ([core.ph:313](../../../../phalcom-core/core/core.ph)) allocates a fresh
> `Option`/`Some` instance (2+ heap objects) plus 2 block-call closures **every step**, and with **no GC at
> all today**, every one of those 1M throwaway objects is retained forever — `sample`-profiling this run
> showed 56-77% *system* time (allocator/mmap pressure), not CPU, vs. `build_only`'s clean 93% CPU. This is
> the single most convincing real-workload case that reclamation is overdue, and doubles as a ready-made
> stress fixture for §7 below (a workload that specifically exercises per-step-Option-churn, not just
> reachable-graph size).
>
> **Fibers make the same underlying gap visible fastest, per-object rather than per-iteration.** No GC is a
> whole-VM property, not a `Fiber` defect — but a `FiberObject` pins its entire `stack`/`frames`/
> `open_upvalues` (every `Value` and closure the fiber ever touched) for the *life of the process* once
> created, `Done`/`Failed` or not (U-FIBER's own §Rubric flagged this pre-emptively: "keep fiber stacks
> inside the arena object... lets a future collector reach them" — the collector is this unit). A
> `benchmarks/wren-suite/fibers.ph`-shaped workload (100k short-lived fibers) leaks 100k full stacks with
> zero reuse (see U-GC §3.7 Win B) — one `Fiber.new` in a loop is a more direct GC-urgency demonstration
> than the `for`-loop's incidental `Option`-churn above. A related, now-fixed correctness gap in the same
> area: the fiber-floor failure cascade (`vm.rs` `run_until`) used to clear only a `Failed` intermediate
> resumer's `.frames`, leaving `.stack`/`.open_upvalues` populated — harmless pre-GC (everything leaks
> regardless), but it would have retained dead state past what even this collector's own reachability
> analysis expected. Fixed in `94487af` (`.stack`/`.open_upvalues` now cleared alongside `.frames`) — worth
> knowing this class of bug exists so §7's "suspended fiber roots its stack" test also asserts a *Failed*
> fiber's parked state is fully gone, not just its frames.

---

## 1. Mission (one sentence)
Land Phalcom's first real reclamation — a **non-moving, precise, stop-the-world mark-sweep** collector on
the existing `SlotMap` heap ([memory-management.md §3](../../../spec/current/memory-management.md)), wire
`System.gc` to it, discharge the **safepoint / temp-root** obligation for native code (§4, the only way the
collector can lose a live object), and ship the two orthogonal, independently-green memory wins — **`Box`
the fat `Object` variants** (measured 256 B → ~24–32 B) and **pool fiber stacks** — with **zero change to
the `ObjRef`/`ClassId`/`Value` surface** and **no `unsafe`**.

## 2. Preconditions (verify on actual HEAD — do not assume)
- **ADR-0009 heap is the substrate** — `Heap { objects: SlotMap<ObjRef, Object> }`; confirm `alloc`/`get`/
  `get_mut`/`retain` are the only slot mutators and that no code caches a `&Object` across an `alloc`.
- **`slotmap = "1"`** in `phalcom-core/Cargo.toml` → `SlotMap::retain` and `SecondaryMap` exist. Confirm.
- **Zero `impl Drop`** on any object-graph type (`rg "impl Drop" phalcom-core/src` → empty). If a `Drop`
  has appeared, **stop** — Invariant M4 (no finalization) is violated and the design must be revisited.
- **Roots reified** — `VM::stack: Vec<Value>`, `VM::frames: Vec<CallFrame>`, `VM::open_upvalues`,
  `VM::current`, `VM::modules`/`main_module`/`last_imported_module`, `VM::classes`, `VM::universe`. Confirm
  the field names on HEAD (the root set of [memory-management.md §2.1](../../../spec/current/memory-management.md)
  is normative — a *missed* root frees live objects; an *extra* non-root over-retains but is safe).
- **`Chunk::constants: Vec<Value>`** exists (const pools are a trace edge). Confirm.
- **`FiberObject`** owns `stack`/`frames`/`open_upvalues`/`resumer`/`result`/`entry`. Confirm — these are
  the parked-fiber trace edges.
- **Re-entrant drivers** — `send_dynamic`, `block_call`, `invoke_method_object` are the only calls that
  recursively drive `run_until` from a primitive (they bump `native_reentry_depth`). Confirm the set; §3.4
  audits handle-holding across them.
- Baseline `./scripts/verify.sh` green before the first edit. Re-run `graphify affected "heap.rs"` and
  `graphify affected "vm.rs"` and **check for concurrent `heap.rs`/`vm.rs` editors** (§4.1).

## 3. Design (realise memory-management.md — do not re-litigate the algorithm)

### 3.1 Marks in a side table — no struct churn ([spec §3](../../../spec/current/memory-management.md))
Add a `SecondaryMap<ObjRef, ()>` to `Heap` (or a throwaway per-collection local). **No `mark` field on any
`Object` variant.** Cleared each cycle. This is the non-invasive core: 16 object structs untouched.

### 3.2 Precise trace — one exhaustive match ([spec §2.3](../../../spec/current/memory-management.md))
`fn trace_object(obj: &Object, push: &mut impl FnMut(ObjRef))` — an **exhaustive** `match` over `Object`
(the compiler then forces every future variant to declare its edges) yielding every child handle per the
§2.3 table. Visit `Value` children through a **single `Value::as_obj() -> Option<ObjRef>` accessor**, never
by matching `Value`'s tags — this is the seam that keeps the collector NaN-box-agnostic (add the accessor
to `value.rs` if absent). Mark with an **explicit worklist** `Vec<ObjRef>`, never Rust recursion (a deep
`List`/`Instance` chain must not overflow the native stack).

### 3.3 Collect = mark then `retain` ([spec §3](../../../spec/current/memory-management.md))
```rust
pub fn collect(&mut self, roots: &Roots) {
    let mut marks = SecondaryMap::new();
    let mut gray = Vec::new();
    roots.each_handle(&mut |h| if marks.insert(h, ()).is_none() { gray.push(h); });
    while let Some(h) = gray.pop() {
        // Object::get(h) then trace_object; mark-and-push each unmarked child.
    }
    self.objects.retain(|k, _| marks.contains_key(k));   // sweep: free list + generation bump, free
}
```
`Roots::each_handle` lives on the VM (it alone knows the root set §2.1) and **must** include the kernel
(`universe`) and the `temp_roots` stack. Non-moving: surviving objects keep their key (Invariant M1).

### 3.4 Safepoint latch + temp roots — the real work ([spec §4](../../../spec/current/memory-management.md))
- `Heap::alloc` accrues live size and **latches** `gc_pending` past a self-tuning threshold
  (`next_gc = live * grow`, grow ≈ 1.5, floored) — it does **not** collect in place.
- The dispatch loop services `gc_pending` **only at a back-edge safepoint**, where `VM::stack`/`frames`
  are the complete root truth. Collection therefore never runs mid-primitive.
- Add `VM::temp_roots: Vec<ObjRef>` with `push_temp_root`/`pop_temp_root`.
- **Audit (the substantive task, not the mark loop):** enumerate every primitive that holds a *fresh*
  `alloc_*`/`heap.alloc` handle in a Rust local across a `send_dynamic`/`block_call`/`invoke_method_object`.
  Scope: **46 re-entrant sites, 40 alloc sites** in `primitive/` (re-measured HEAD 2026-07-14; the plan's
  original "≈31 alloc" is stale) — the intersection is smaller.
  Each such handle gets a `push_temp_root`/`pop_temp_root` scope. Handles already on the operand stack
  (receiver/args) need none. **List this audit explicitly in the return contract** — a missed site is a
  silently-lost live object (Invariant M3).

### 3.5 `System.gc` ([spec §5](../../../spec/current/memory-management.md))
Wire the `gc` primitive (currently a spec'd no-op stub, `system.md` §`gc`) to `Heap::collect` at the
safepoint, returning `None`. No finalizers, no compaction, handles stable.

### 3.6 Companion win A — `Box` the fat variants ([spec §7](../../../spec/current/memory-management.md), **independent, ship first**)
`size_of::<Object>()` = **280 B measured on HEAD 2026-07-14** (not the 256 B in ADR-0050 — `ClassObject`
grew by `attributes: Vec<Value>` + `attributes_frozen` under U-ANNOT, and now *is* the whole 280 B). The
`SlotMap` slot is sized to the fattest variant, so every `Str`/`Range`/small instance pays it, taxing the
hot `heap.get` path.

The per-variant ladder is **already measured** — see the table in
[memory-management.md §7](../../../spec/current/memory-management.md); do not re-derive it. Box **six**
variants — `Class` (280), `Fiber` (176), `Module` (168), `Closure` (160), `Method` (88), `Map`/`Set` (72) —
so `Range` (40) becomes the cap and the `<= 48` bound in §7 holds. `Object::Class(Box<ClassObject>)`, etc.
**Do not box `Instance`** (24 B): it is already below the floor and is the most-allocated variant, so a
`Box` buys nothing and costs an indirection + an allocation on the hottest path. ADR-0050 §9's variant
list predates the measurement.

Fully behind the enum; touches only the variant decl + its constructors/accessors. **No collector
dependency — land it as the first commit for a standalone measurable win**, then re-measure
`size_of::<Object>()` and record the number in the return contract. Re-measure with
`cargo +nightly rustc -p phalcom-core --lib -- -Zprint-type-sizes` (the method used for the ladder).

### 3.7 Companion win B — fiber-stack pool ([spec §7](../../../spec/current/memory-management.md), **independent**)
Pool and reuse `FiberObject::stack`/`frames` `Vec`s across fiber deaths instead of fresh-allocating per
fiber (Skynet allocates ~1M fibers). A free-list of `Vec<Value>`/`Vec<CallFrame>` on the VM, handed out at
fiber creation and returned when a fiber reaches `Finished`/`Failed`. **Zero observable semantic change.**
Optional within this unit — split to a follow-on if the write-set gets hot (§4.1).

### 3.8 Decorators / attributes — no GC-specific machinery, one forward obligation
Established in step 0 (2026-07-14). **Landed:** the Compile tier (`@construct`/`@get`/`@set`/`@data`/
`@sealed`/`@variant`, contracts) + M-ATTR-ROOT retention/reflection (`Attribute`/`On` at
[core.ph:1008](../../../../phalcom-core/core/core.ph), the `attributes` stores on `Class`/`Method`/`Module`,
`Behavior#attributes`). **Absent:** the Install (`wrap(_)`), Dispatch (`resolveMissing`), Runtime
(`aroundSend`/`Invocation`), and Layout (`finalizeLayout`) tiers — grep for all four across
`phalcom-core/src` returns zero.

For the collector this means:

- **The retention stores are ordinary edges, not a special case.** `attributes: Vec<Value>` holds plain
  `Object::Instance` handles; `Method.contracts: Option<Vec<(Symbol, Value)>>` likewise. One arm each in
  `trace_object` — already in the regenerated §2.3 table. Nothing about decorators needs GC-specific code.
- **`attributes_frozen` is a free gift to a *future* generational collector**: it makes each store
  append-only-then-frozen, so a frozen class needs no write barrier. Note it under §9, do not act on it.
- **The one forward obligation is the *unbuilt* Runtime tier.** `aroundSend` allocates an `Invocation` per
  intercepted send **and** re-enters the interpreter — exactly the §3.4 hazard shape (fresh handle in a
  Rust local across a re-entrant send). Land `push_temp_root`/`pop_temp_root` in this unit and record in
  `DEFERRED.md` that M-RUNTIME must be written against it. Layout-tier reserved slots will live in
  `InstanceObject.slots` — already traced, no new edge.

The exhaustive `match` (§3.2) is what forces every future tier to declare its edges. Keep it exhaustive;
that discipline — not a decorator-aware tracer — is the defence.

### Rubric — hazards & preclusion (mandatory)
- **Safepoint ⊗ re-entrant native handle (THE load-bearing check, [spec §4](../../../spec/current/memory-management.md)).**
  The collector is sound **iff** no live object is reachable only from a Rust local across a re-entrant send
  at a safepoint. The §3.4 audit + temp-root scopes discharge this. Guard with a **stress test**: a
  primitive path that allocs-then-re-enters under a forced `System.gc` between the two, asserting the object
  survives (see §7).
- **Moving ⊗ handle stability (why non-moving, [spec §3](../../../spec/current/memory-management.md)).** IC tags
  (ADR-0012), `==` identity (`value_eq`), and suspended-fiber `Value`s all assume a handle names the same
  object forever. Non-moving mark-sweep preserves this (M1). Do **not** compact in this unit.
- **Finalizer ⊗ unwind (absent by construction, M4).** No `impl Drop` on the graph → GC runs no user code →
  no resurrection, no `ensure`-vs-GC ordering. The precondition check (§2) fails the unit if a `Drop` has
  appeared.
- **Kernel cycle (M5).** `Metaclass`-instance-of-itself must not trap the marker (the mark bit stops the
  loop) and must never be swept (pinned via `universe` roots). Assert post-GC kernel liveness in
  `verify_invariants()`.
- **Deep-graph recursion.** Marking must be worklist-based; a recursive tracer would overflow on a long
  `List`. Pin a deep-chain collection test.
- **Representation/dispatch impact:** none to the surface. No `Value` tag change, no selector-encoding
  change, no opcode change. `Box`-ing changes `Object`'s internal layout only.
- **Precedent:** Wren's single-threaded non-moving mark-sweep + `nextGC = live * heapGrowFactor` + the
  `wrenPushRoot`/`wrenPopRoot` temp-root API (the direct model). Rejected alternatives (refcount — kernel
  cycle + `Copy`-`Value` tax; moving/copying — breaks handle stability; immediate generational/incremental —
  write-barrier invasive, no measured pause) are in [ADR-0050 §Alternatives](../../../adr/0050-non-moving-mark-sweep-collector.md).

## 4. Confirmed write-set (tight & disjoint; re-validate with `graphify affected` on HEAD)
| File | Why | Slice |
|---|---|---|
| `phalcom-core/src/heap.rs` **(SPINE — reviewer ON)** | `SecondaryMap` marks; `trace_object`; `Heap::collect` (mark+`retain`); alloc size-accrual + `gc_pending` latch; `Box`-ing the fat `Object` variants + accessor fixups | collector + repr |
| `phalcom-core/src/vm.rs` **(SPINE — reviewer ON)** | `Roots::each_handle` (root enumeration §2.1); `temp_roots` + push/pop; safepoint service of `gc_pending` at the loop back-edge | roots + safepoint |
| `phalcom-core/src/value.rs` | `Value::as_obj()` accessor (the NaN-box seam) if absent | seam |
| `phalcom-core/src/primitive/system.rs` | wire the `gc` primitive to `Heap::collect` → `None` | System.gc |
| `phalcom-core/src/primitive/*.rs` (audited subset only) | `push_temp_root`/`pop_temp_root` at the handle-across-re-entrant-send sites found in §3.4 | temp roots |
| `phalcom-core/src/primitive/fiber.rs` (+ `vm.rs`) | fiber-stack pool return-on-death (win B) | pool |
| `phalcom-core/tests/` (**new `gc` label** or Rust integration tests) | mark/sweep, kernel-liveness, deep-chain, cycle-collected, temp-root stress, `System.gc`, `size_of` regression | all |

**Deliberately NOT in scope:** any `Value` tag change / NaN-boxing (deferred, ADR-0010); write barriers;
generational/incremental/compacting collection (spec §7 open); `bytecode.rs`, `compiler/`, `phalcom-ast`,
`core.ph` (no surface, no protocol); `PHASE2-INDEX.md`/`u0-state.md`/`system.md` (shared files — the reviewer
or a separate doc-sync commit updates the ledger, never this unit's code diff).

### 4.1 Write-set collision risk (flag, don't resolve)
- **`heap.rs` and `vm.rs` are spine files** — the busiest in the tree (dispatch, fibers, IC-readiness all
  live here). **Serialize:** U-GC must hold both alone; it cannot share a parallel wave with any unit that
  edits `vm.rs`/`heap.rs` (e.g. U-HOTPATH IC work, any fiber unit). Check `graphify affected` + live
  worktrees before dispatch.
- **`primitive/*.rs`** — the temp-root edits are surgical and scattered; confirm no concurrent primitive
  unit holds the same files.
- **Win A (`Box`) and Win B (pool) are separable** — if the spine is contended, land Win A first (it needs
  only the `Object` decl) and split Win B to a follow-on.

## 5. Build order (small, independently-green diffs)
1. **Step 0 — re-ground the normative tables against HEAD. DONE 2026-07-14** (see Provenance): §2.1 root set
   and §2.3 edge table regenerated in [memory-management.md](../../../spec/current/memory-management.md); §7
   size ladder measured; this plan's drifts annotated. Doc-only, no code. **Do not start step 1 from the
   pre-step-0 tables** — they carry two live free-a-live-object bugs (`Block.closure`, `Upvalue::Open.fiber`).
2. **Win A — `Box` fat variants.** Ladder already measured (§3.6) — box the six named variants, not
   `Instance`; fix constructors/accessors; re-measure `size_of::<Object>()`. Green + record the number.
   *(Standalone; no collector.)*
3. **Trace + collect (no scheduling yet).** `Value::as_obj`; `trace_object`; `Heap::collect`;
   `Roots::each_handle`. Drive it from a **test-only** `vm.force_gc()` — no automatic triggering yet.
   Goldens: allocate-then-drop-root frees; a cycle is collected; the kernel survives; a deep chain
   collects without stack overflow. Green.
4. **`System.gc`.** Wire the `gc` primitive to `force_gc`; `.ph` test that `System.gc` returns `None` and a
   dropped object count drops. Green.
5. **Safepoint automation + temp-root audit.** `gc_pending` latch in `alloc`; service at the loop back-edge;
   `temp_roots` + the §3.4 audited push/pop scopes; the **temp-root stress test** (forced GC across a
   re-entrant send). `verify_invariants()` post-GC kernel assert. Green — **this is the gating commit.**
6. **Win B — fiber-stack pool** (or split to follow-on). Pool return-on-death; a Skynet-shaped allocation
   test shows reuse; semantics unchanged. Green.

Each step is a self-verifiable commit; never commit a non-compiling tree.

## 6. Mandatory rules
- **Docs:** `///` on every new fn/field/type (`trace_object`, `collect`, `Roots`, `temp_roots`,
  `Value::as_obj`, the `gc_pending` field, each boxed-variant accessor) citing ADR-0050 /
  memory-management.md §. `cargo doc --workspace --no-deps` adds no warnings.
- **Green gate:** `./scripts/verify.sh` exits 0; no new clippy; **no `unsafe`** (the collector is safe Rust
  — this is a hard design constraint, ADR-0009/ADR-0050). Follow `rust-best-practices`.
- **Reviewer ON** (spine files) — `phalcom-reviewer` gates the diff; the writer never self-approves. Also
  run the `rust-sanitizers-miri` lane over the collector (no UB in the mark/sweep worklist).

## 7. Test strategy (the green gate must assert) — new `gc` label / integration tests
- **Reclaims garbage (PASS):** allocate an object reachable from one root; drop the root; `force_gc`; assert
  its handle is now stale (resolves to the diagnostic, Invariant M6) and the live count dropped.
- **Collects cycles (PASS):** two instances referencing each other, no external root; `force_gc` frees both
  (proves mark-sweep, not refcount).
- **Kernel survives (PASS — M5):** `force_gc` with an empty user stack; every `CoreClasses` handle still
  resolves; `verify_invariants()` passes; the `Metaclass` apex is intact.
- **Deep chain (PASS):** a length-100k `List`/linked instance chain collects (or survives if rooted) with
  **no native-stack overflow** — proves the worklist marker.
- **Temp-root stress (PASS — THE §4 guard):** a primitive path that `alloc`s then re-enters
  (`send_dynamic`/`block_call`) with a forced GC in between; assert the freshly-alloc'd object **survives**.
  A variant **without** the temp-root proves the hazard is real (documents why the scope exists).
- **Suspended fiber roots its stack (PASS):** a parked fiber whose saved `stack` holds the only reference to
  an object X; `force_gc`; X survives while the fiber is reachable, and is collected once the fiber is not.
- **`System.gc` (PASS):** returns `None`; observable free after dropping references.
- **`size_of::<Object>()` regression (PASS):** assert the post-`Box` size is at/below a pinned bound (e.g.
  `<= 48`), so a future fat un-boxed variant is caught.
- **Allocation-churn workload (PASS — the `for.ph` motivating case, see Provenance above):** run
  [`benchmarks/wren-suite/for.ph`](../../../../benchmarks/wren-suite/for.ph) (or the isolated `for`-over-1M-
  `List` loop it bisects to) with automatic collection enabled; assert wall time drops materially from the
  pre-GC ~11-12s baseline and live-object count stays bounded across the loop instead of growing linearly
  with iteration count — proves the collector actually reclaims per-step `Option`/`Some` churn from
  `List#iterate`, not just externally-rooted garbage.
- **NEGATIVE / robustness:** using a handle after its object is swept yields the defined stale-handle
  diagnostic (never UB, never a wrong object) — run under miri.

## 8. Decisions flagged (flag, don't pick)
| ID | Decision | Options | Architect recommendation |
|---|---|---|---|
| **DEC-GC-A** | **Ratify ADR-0050 before merge, or land behind a `gc_enabled` flag?** | **(A)** ratify ADR-0050 first, ship collection on by default; **(B)** land the machinery with automatic collection behind a default-off flag, ratify after soak. | **(A)** if the reviewer + a green temp-root stress test give confidence; else **(B)** — the collector is the one change that can *lose* data, so a soak flag is cheap insurance. Win A (`Box`) ships unconditionally either way. |
| **DEC-GC-B** | **Collection trigger threshold.** | **(A)** count-based (`objects.len()`); **(B)** byte-based via per-variant `size_of`. | **(A)** to start — simplest, and post-`Box` slot size is uniform enough. Move to (B) only if count-based over/under-collects in practice. |
| **DEC-GC-C** | **Fiber-stack pool (Win B) in this unit or follow-on?** | **(A)** in-unit; **(B)** split to U-GC-POOL follow-on. | **(B)** — now settled by measurement, not preference. Win B was **built, A/B'd, and reverted as a null result** (finding F5): Skynet RSS is dominated by ~1M immortal `FiberObject` shells, so pooling their buffers cannot move it. It has no collector dependency in *mechanism* but a hard one in *sequence* — it is only measurable once sweeping frees the shells. Split it out, land the collector, then re-measure against a high-turnover workload. **The reverted code is unrecoverable** (never staged; absent from the object DB — `git fsck` confirms), but the design is fully recorded in F5 and is ~1h to rebuild: bounded free-list on the VM, acquire in `new_fiber_ref` (`primitive/fiber.rs`), recycle at `FiberStatus::Done` (`vm/dispatch.rs`) before the resumer's `load_live_from` drops the buffers; `Done` path only. |
| **DEC-GC-D** | **`Heap` owns a persistent `SecondaryMap` marks field, or a per-collection local?** | **(A)** persistent (reused, cleared each cycle); **(B)** fresh local each `collect`. | **(B)** for simplicity first (no stale-marks invariant to hold between cycles); switch to (A) only if allocation of the map shows up in profiles. |

## 9. Must-not-preclude check
- **Generational collection (spec §7):** *served, not precluded* — a nursery + remembered set slots in
  behind the same handle API; the one enabler to add **now** is funnelling field/element mutation through
  choke-point methods so a future write barrier has a single home. Flag any mutation site that bypasses
  them.
- **Incremental / tri-color (spec §7):** not precluded — the `SecondaryMap` marks generalise from `()` to a
  color; the worklist becomes the gray set. No surface change.
- **Compaction / moving (spec §7, ADR-0050 §Alternatives):** kept reversibly open — it would need a
  handle→slot indirection to preserve M1; this unit neither builds nor blocks it.
- **NaN-boxing (ADR-0010):** not precluded — the collector visits `Value` children only through
  `Value::as_obj()`, so a `Value` repr change touches that one accessor, never the mark/sweep.
- **Inline caches (ADR-0012, U-HOTPATH):** actively *protected* — non-moving collection keeps `ClassId`/
  `ObjRef` IC tags valid across a GC (M1); U-HOTPATH can populate caches without a GC-invalidation story.
- **`superclass=` (Q4):** not precluded — stop-the-world collection needs no barrier, so runtime hierarchy
  mutation stays implementable.

## 10. Return contract (report to `phalcom-reviewer`)
The `Object` variants boxed + **before/after `size_of::<Object>()`** (measured) · the `trace_object` edge
coverage vs [spec §2.3](../../../spec/current/memory-management.md) (confirm every variant's handles/Values
visited) · the exact `Roots::each_handle` root set vs [spec §2.1](../../../spec/current/memory-management.md)
(confirm kernel + temp_roots included; confirm `field_layouts`/symbol maps correctly excluded) · **the
full §3.4 temp-root audit list** — every primitive site that holds a fresh handle across a re-entrant send
and how it was protected (the load-bearing deliverable) · the safepoint wiring (where `gc_pending` is
serviced) · `System.gc` returns `None` + observable free · the `verify_invariants()` post-GC kernel assert ·
the temp-root stress + cycle + deep-chain + suspended-fiber-roots test results · confirmation **no `unsafe`,
no surface/`Value`/opcode change** · miri lane tail · how DEC-GC-A/B/C/D resolved · any `DEFERRED.md`
pointers (Win B follow-on, generational barrier, NaN-box) and a note that `system.md` §`gc` /
`PHASE2-INDEX.md` / `u0-state.md` need a **separate** ledger-sync commit (not this code diff).
