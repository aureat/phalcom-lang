# Recon — `caches-and-fusion.md` (VM track, Doc 5)

Phase-1 scout. Not the survey (that is Agent B). Answers the four required questions and arms both
briefs. All line anchors verified at HEAD this session.

**Governing hazard for this doc:** [[landed-state-claims-go-stale]]. There is a `U-IC` unit *plan*
(`docs/forge/units/U-IC/plan.md`, still `Status: PLANNED`) describing a **grander** cache than the
one that actually runs. Recon's job here is to separate *what landed* from *what was planned*. HEAD
is the truth; the plan is a plan.

---

## 1. Architecture vs representation

**Architecture (the shape).** *Call-site memoization with version-stamped lazy invalidation.* The
send Doc 4 described (`resolve` = walk the receiver's class chain) is paid **once per call site**,
not once per call. Two independent, structurally-identical instances of the same idea run at HEAD:

1. **Method inline cache** — each `Invoke` site owns one slot recording *(receiver class, resolved
   method, version)*. A send compares the receiver's class and a global version stamp against the
   slot; a match uses the cached method with **no walk**; a mismatch re-runs `lookup_method` and
   refills. Probe + refill live in `invoke_at` (`vm/dispatch.rs::VM::invoke_at` @ L408–427 — the
   *hit path* Doc 4 deferred as Lie #1).
2. **Global-variable cache** — each `GetGlobal`/`SetGlobal` site owns a slot recording *(module,
   slot index, module's globals-version)*, so a resolved global name indexes straight into
   `ModuleObject::globals` instead of re-probing a `name_to_slot` SipHash map (F12, commit
   `39d9042`).

Plus a compile-time layer that *removes dispatches entirely* rather than caching them:
**superinstruction fusion** (`Chunk::fuse_superinstructions`, chunk.rs @ L116; cut 008) rewrites
adjacent `(GetLocal|Constant) → Invoke` pairs into `InvokeLocal`/`InvokeConst`.

**Representation (what the live state holds — where the consequences live).**

- The method cache is a **side table**, not an operand: `Chunk::caches: Vec<Cell<Option<InlineCache>>>`
  (chunk.rs @ L50), **parallel to `code`**, indexed by the instruction's own `ip` (`cache_ip`).
  `InlineCache = { class: ClassId, method: ObjRef, version: u64 }` (chunk.rs @ L10). The `Cell`
  buys interior mutability so a refill works through a shared `&Chunk` borrow. Nothing is stored in
  the `Invoke(u8, u16)` operand — the operand is still just arity + selector index (Doc 4).
- **The version stamp is a single *global* `world_version: u64`** on the VM (`vm/mod.rs` @ L116).
  A mismatch means "*some* method was (re)defined *somewhere* since this slot was filled" — not
  "this class changed." It is bumped **unconditionally** on every method install
  (`Bytecode::Method` arm, dispatch.rs @ L927 static / L930 instance). One counter; one bump
  invalidates **every** cache slot in the entire program at once — lazily, on each slot's next
  stamp check. The VM never walks the world to *find* stale slots.
- The global-variable cache is a **separate** parallel table, `Chunk::gcaches` (chunk.rs @ L55),
  keyed on a per-module `globals_version` (chunk.rs @ L36) — deliberately split from `caches`
  because the two never occupy the same instruction and a union would widen every slot.
- Fusion is an **in-place peephole**: the fused opcode overwrites the pair's *first* instruction and
  the original `Invoke` is left at `ip+1` as **dead code**, so `code.len()` never changes — every
  jump offset and every `ip`-indexed side table (`spans`/`caches`/`gcaches`) stays aligned with no
  re-layout. The fused arm reads its IC slot at `ip+1` (the dead `Invoke`'s own slot), so it probes
  *the same cache* the unfused pair would have.

**The one-line representation fact that settles the doc:** *the cache is a side table keyed by
instruction position, and its invalidation stamp is one global counter — so a hit is a `ClassId` +
`u64` compare, and the price of that simplicity is that defining any method anywhere flushes every
cache in the program, lazily.*

## 2. The grip, grounded

> **The VM resolves a selector once per call site, not once per call.** Each `Invoke` site owns a
> one-entry memo — *(receiver class, method, version)* — and trusts it until a single global
> `world_version` counter says the world changed. One bump invalidates every site at once; the VM
> never hunts for stale caches, it lets each one fail its own stamp check the next time it runs.

Corollary (the honesty knot, see §3): the stamp is **global, not per-class**. Defining a method on
`Foo` invalidates the cache at a call site that only ever sends to `Bar`. That coarseness is the
representation choice, and it was the *fallback* option in the plan, not the recommended one.

## 3. What was actually deliberated (and the landed-vs-planned split)

This is the doc's honesty spine (§5.2). Four distinct provenances, and they must not be blurred:

- **The IC *seam* was deliberated — ADR-0012.** The `caches` slot on every `Invoke` was reserved at
  ADR-0012 time as an "IC-ready shape at zero present cost," population explicitly deferred. Doc 4's
  Lie #1 is exactly this seam. That the seam exists is a real, documented decision.
- **The IC *population* landed as an incremental cut, NOT via the `U-IC` plan.** The plan
  (`U-IC/plan.md`, `PLANNED`) proposes a selector-only interner (Change 1), a **per-class epoch**
  (DEC-IC-A recommended), design-B own-method arrays (Change 2), and operand-free `LOAD_LOCAL_0..15`
  superinstructions (Change 3). **None of that is what runs.** HEAD has: a *global* `world_version`
  (the plan's DEC-IC-A *fallback*, "acceptable v1"), the still-mixed `Symbol` space (Change 1 not
  done — **Agent B must confirm** `Symbol` is still vars/fields/selectors mixed), the still-`IndexMap`
  method dict (Change 2 not done), and cut-008 `InvokeLocal`/`InvokeConst` fusion (a *different*
  superinstruction than Change 3's operand-free loads). **Honest framing:** the fine-grained
  per-class epoch was *planned and not built*; the coarse global counter is an **absence of that
  machinery** that happens to be correct, not a reasoned choice of coarseness. Do not flatter it as
  "chose global invalidation for simplicity" — no doc says that; the plan recommends the opposite.
- **Fusion was deliberated AND measured — perf-log 008.** This is the doc's strongest honest
  evidence. F16 first *deferred* superinstructions ("premature; the inliner covers arithmetic") and
  the re-ask **overturned its own verdict** — reason 3 was found *false* (the inliner's sacred set is
  control-flow only; `1 + 2` was never inlined). Cut 008 measures a dispatch at **~3.3 ns** (two
  independent instruments) and ships **−8.1% `string_equals`, −5.1% `for`, −4.7% `variadic_send`,
  −4.2% `bare_send`, −3.9% `fib`** — while `map_numeric` removed the *most* dispatches (18M) and
  moved **−0.2%**, because its instructions cost 27.6 ns each (F17). "A fusion buys dispatch, and
  only workloads whose time *is* dispatch can spend it." Real numbers, real scar. Quote them.
- **Guard opcodes — ADR-0018 (sacred-selector inliner + override guard).** `GuardBool`/`GuardBlock`
  are pristine-flag fast paths; `note_method_installed` (dispatch.rs @ L935) dirties the pristine
  flag when a sacred selector is redefined on kernel `Bool`/`Block`. Recon steer: **mention as a
  third fast-path family, keep brief** — the sacred-inliner mechanism is its own future topic, not
  this doc's spine.

**Honesty flag for synthesis.** The coarse *fork* a design-space walk wants (no cache / monomorphic
IC / polymorphic IC) was **not** bench-raced at HEAD — monomorphic-with-global-stamp is what the
seam got populated with, and the plan keeps the slot layout "extensible to a small PIC without a
bytecode change" (DEC-IC-D) rather than shipping one. Present the coarse fork as scaffolding; land
the two genuinely-deliberated things: the *seam* (ADR-0012) and *fusion* (perf-log 008, measured).

## 4. Brief-steering

**Agent A (theory, no source, not told Phalcom's branch).** Emphasis:
- Go **deep** on the cache fork: **no cache** (walk every send) → **monomorphic inline cache** (one
  entry per site, the common case is a loop hammering one receiver class) → **polymorphic IC (PIC)**
  (a few entries per site, for megamorphic sites) → **megamorphic fallback** (give up, walk). Each
  genuinely tempting; each with its bill (hit rate vs slot size vs the cost of a wrong guess).
- Go **deep** on **invalidation** — the hard half of caching. The two poles: a **per-site / per-class
  epoch** (fine; only affected caches die; bookkeeping to bump the right epochs) vs a **single global
  version counter** (coarse; one bump flushes everything lazily; trivially correct, no bookkeeping,
  but a class-unrelated mutation still costs you). Name the classic soundness bug: a cache that
  serves a method removed or overridden after caching. This is the doc's spine alongside the fork.
- Go **deep** on **superinstructions / bytecode fusion** as a *different* lever than caching — it
  removes the *dispatch*, not the *lookup*. What a "dispatch" costs and why removing one only helps
  dispatch-bound code (the reader needs the "8.8% of instructions ≠ 8.8% of time" idea).
- **One section** on the polymorphic-inline-cache history (Deutsch & Schiffman, Smalltalk-80, 1984 —
  the origin of inline caching; SELF's PICs, Hölzle/Ungar, and the line from PICs to type feedback
  and JIT deopt). This is where the vocabulary lives.
- Distinguishing program: a call site in a loop over one receiver class (monomorphic — cache wins
  big) vs the same site fed many classes (megamorphic — cache thrashes); and a program that mutates
  a class mid-loop, to make invalidation observable.
- Cast to consider (A names the cut list): **Smalltalk/SELF** (inline caches + PICs, the ancestors,
  name-givers), **V8/JS engines** (hidden classes / shapes + IC + the monomorphic→polymorphic→
  megamorphic ladder; the deopt story), **JVM** (`invokevirtual` monomorphic-call-site profiling,
  bimorphic inlining), **CPython** (PEP 659 adaptive specializing interpreter — `LOAD_ATTR`
  specialization + version tags, 3.11 — a *very* close analogue of exactly this doc's mechanism),
  **Forth/threaded code** (superinstructions / token threading — the fusion ancestor). Expect ~4–5
  to survive.

**Agent B (source map, graphify-led).** Headline question first: *is the method inline cache keyed
on a **per-class epoch** or a **single global `world_version`**, and is it a **side table** or an
operand?* Must confirm with lines:
- `InlineCache` + `GlobalCache` + `Chunk::{caches,gcaches}` type defs (chunk.rs @ L10/L30/L50/L55) —
  quote in full. Confirm side-table-keyed-by-ip, `Cell` interior mutability.
- `invoke_at` hit path (dispatch.rs @ L408–427): the probe (`slot.class == receiver_class &&
  slot.version == self.world_version`) and the refill. **Confirm the U-IC hazard**: `receiver_class`
  and `world_version` are read *after* `lookup_method`, not before (why? — a re-entrant send during
  lookup could bump the world; caching a pre-lookup version would be unsound). Quote the ordering.
- `world_version` (vm/mod.rs @ L116) is a **single global `u64`**; bumped unconditionally at
  dispatch.rs @ L927 (static) and L930 (instance) on every `Bytecode::Method` install. Confirm there
  is **no per-class epoch** at HEAD.
- **Confirm the landed-vs-planned split** (the doc's honesty spine): `U-IC/plan.md` is `PLANNED`;
  its Change-1 selector-only interner did NOT land (confirm `Symbol` is still a mixed space —
  interner.rs); its per-class epoch (DEC-IC-A recommended) did NOT land (global counter runs);
  Change-3 operand-free `LOAD_LOCAL_0..15` did NOT land (cut-008 `InvokeLocal`/`InvokeConst` did
  instead). State plainly what runs vs what is planned.
- `fuse_superinstructions` + `branch_targets` (chunk.rs @ L116/L137): the in-place rewrite, the dead
  `Invoke` at `ip+1`, the jump-target guard. Confirm the fused arms in dispatch.rs (`InvokeLocal` @
  ~L1036, `InvokeConst` @ ~L1046) advance `ip` by 2 and read the IC/span at `ip+1`.
- `GetGlobal`/`SetGlobal` fast paths (dispatch.rs ~L632 / ~L685): the `gcaches` probe on
  `globals_version`; confirm `SetGlobal` has no core-module fallback (recon claim).
- Guard opcodes `GuardBool`/`GuardBlock` (dispatch.rs ~L1184/~L1191) + `note_method_installed`
  (dispatch.rs @ L935): confirm they are the sacred-selector pristine-flag fast path (ADR-0018);
  **brief** — mark as adjacent, not this doc's core.
- **Run fixtures live** (`phalcom -i '<src>'`): (a) a method sent in a loop — works (can't see cache
  from stdout, note that). (b) **the invalidation proof** — send `x.foo()` in a loop, redefine `foo`
  (reopen the class) partway, assert the *new* body runs (Doc 4's reopen fixture already shows v1→v2;
  frame it here as "the world_version bump the cache honors"). (c) a megamorphic site (many receiver
  classes through one call site) — correct results, no crash. Report observed output verbatim.
- Bounded ADR read: ADR-0012 (the seam — already summarized in Doc 4's source-map), ADR-0051 (Tier-3
  perf strategy), ADR-0018 (guards), ADR-0041 (hierarchy-stability — what mutations must invalidate).
  Decision + Alternatives only. Perf-log 008 is the measured-fusion source — cite its numbers.

## 5. Predict-then-check candidates (pick in synthesis)

- **Primary (from the track spec):** "A site runs `x.foo()` in a loop; the cache is warm. Someone
  defines `foo` on `x`'s class mid-loop. Does the cached site notice — or keep running the old
  method?" → It notices. The `Method` install bumps `world_version`; the next probe's
  `slot.version == world_version` fails and re-resolves. Reader predicts hit vs re-resolve.
- **Secondary (the coarseness knot):** "You define a method on class `Foo`. A call site elsewhere
  only ever sends to `Bar`, and `Bar` is untouched. Is `Bar`'s cached site invalidated?" → **Yes** —
  the stamp is global, so every site in the program is invalidated at once (lazily). This is the
  price of one counter instead of per-class epochs, and it teaches the representation.

## 6. Lies to mark forward / defer

1. **Per-class epoch** — the fine-grained invalidation the `U-IC` plan recommends (DEC-IC-A) is
   **planned, not built**. Present global invalidation as HEAD-truth; note the epoch as the planned
   refinement, clearly labelled unbuilt. (Not a "lie the next doc pays off" — a genuine
   unbuilt-future; mark it as intent, cite the plan as intent per AUTHORING truth-basis.)
2. **Polymorphic IC (PIC)** — HEAD is monomorphic (one entry). PIC is A's theory and the plan's P4
   ("slot layout must not preclude PIC"); mark as not-built.
3. **Selector-only interner / design-B arrays** — U-IC Changes 1–2, not built. Mention only as the
   planned redesign; do not describe as current.
4. **Sacred-selector inliner mechanism** (ADR-0018) — `GuardBool`/`GuardBlock` get a brief mention as
   a third fast-path family; the inliner's full mechanism (`compile_sacred_call`, override-epoch
   deopt) is **its own future topic**, deferred.
5. **`SuperSend` is uncached** (DEC-IC-B, DEFERRED) — a statically-known target, left out of the IC
   in v1. One sentence, ties back to Doc 4's SuperSend forward-pointer.

## 7. Doc kind

**Mechanism + fork.** Mechanism spine: the two-move memo (probe → hit, or miss → resolve → refill)
and the version-stamp invalidation, applied twice (method IC, global cache) plus the orthogonal
fusion lever. Fork: the invalidation-granularity axis (global counter vs per-class epoch) and the
cache-shape axis (no cache / monomorphic / PIC / megamorphic-fallback). **Stateful ⇒ a trace earns
its place**, and the hard case to trace (§5.5) is **invalidation**, not the hit: a warm cache, a
mid-loop method (re)definition, the `world_version` bump, and the next probe missing its own stamp —
that is the moment a reader's "the cache just remembers the answer" model breaks. The textbook hit is
the case intuition already handles.

**Merge-candidate note (track spec §knob).** Doc 5 was flagged a possible merge with Doc 4. Recon's
verdict: it stands alone — IC + global `world_version` + `gcaches` + cut-008 fusion + the guard
family is a full doc, and its honesty spine (landed-vs-planned) is distinct from Doc 4's (the
deliberated selector-encoding). Keep separate.

## 8. Anchors gathered this session (for the briefs; B verifies + extends)

- `phalcom-core/src/chunk.rs` — `InlineCache` @ L10, `GlobalCache` @ L30, `Chunk::caches` @ L50,
  `gcaches` @ L55, `fuse_superinstructions` @ L116, `branch_targets` @ L137.
- `phalcom-core/src/vm/dispatch.rs` — `invoke_at` probe/refill @ L408–427; `world_version += 1` @
  L927/L930; `note_method_installed` @ L935; `GetGlobal`/`SetGlobal` ~L632/~L685; `GuardBool`/
  `GuardBlock` ~L1184/~L1191; fused arms ~L1036/~L1046.
- `phalcom-core/src/vm/mod.rs::world_version` @ L116 (global `u64`).
- `docs/forge/units/U-IC/plan.md` (PLANNED — the grander unbuilt form; DEC-IC-A..D).
- `docs/forge/perf-log/008-fuse-invoke-pairs.md` (measured fusion: ~3.3 ns/dispatch, the result
  table, the F16 verdict flip, the `map_numeric` non-result).
- ADR-0012 (IC seam), ADR-0051 (tiered perf), ADR-0018 (sacred guards), ADR-0041 (hierarchy
  stability / what must invalidate).
