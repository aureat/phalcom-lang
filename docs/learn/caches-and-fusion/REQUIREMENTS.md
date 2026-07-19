# `docs/learn` — caches & fusion: requirements, approach, checklist

Working folder. Scratch. The shipped doc is `docs/learn/vm/caches-and-fusion.md` (VM track, Doc 5);
everything here is state used to build it. Grip and design space copied from `recon.md`, grounded.

## 0. The obligation

One test, and it is the whole spec:

> **After reading, the reader could re-derive Phalcom's choice from the constraints alone.**

Delete the source. Hand the reader the pressures. Could they rebuild it? A doc that only describes
what `invoke_at` and `fuse_superinstructions` do has failed, however accurate.

Corollary: every branch not taken must be made **genuinely tempting** before it is rejected. A
strawman teaches Phalcom's answer without teaching the question.

## 1. Reader

Knows PL design — has used dynamic dispatch, virtual methods, maybe Smalltalk or Ruby; has heard
"inline cache" and "JIT deopt" but cannot say what a cache slot *holds*. Not fluent in runtime
implementation. Stated weakness: **cannot hold moving-state mechanisms in their head**, lacks stable
notation, so complexity accretes until the thread is lost. Caching *is* moving state (a slot that
fills, goes stale, refills) — so this doc is squarely in the reader's weak spot and must hand over a
**grip**, not completeness.

Inherited truth from Docs 1–4 (assume the reader has them): the VM is one `while` loop over a
`match` (Doc 1); the compiled artifact is a `Chunk` inside a `Callable`, and the `Chunk` already
carries `caches`/`gcaches` side tables *named but deferred* (Doc 2); a call runs in a `CallFrame`
(Doc 3); **a send resolves a selector by walking the receiver's class chain** — Doc 4's spine — and
Doc 4 explicitly left **the hit path** as its Lie #1 ("the walk happens every send") with a forward
pointer to *this* doc. **This doc owns Doc 4's Lie #1: what happens on the *second* send, once the
walk's answer can be remembered.**

## 2. Doc kind

**Mechanism + fork** (recon §7).

- **Mechanism** — the two-move memo: **probe** (compare receiver class + version stamp to the slot →
  hit, use cached method, no walk) or **miss** (re-run Doc 4's `lookup_method`, refill the slot).
  Applied *twice* with identical shape (method IC + global-variable cache), plus an orthogonal
  compile-time lever (superinstruction fusion) that removes *dispatches* rather than caching *lookups*.
- **Fork** — two live axes: **invalidation granularity** (single global version counter vs per-class
  epoch) and **cache shape** (no cache / monomorphic / polymorphic IC / megamorphic fallback).
- **Stateful ⇒ a trace earns its place** — but trace the **counterintuitive** case: not the textbook
  hit (reader's intuition already handles "it just remembers the answer"), but **invalidation** — a
  warm cache, a mid-loop method redefinition, the `world_version` bump, and the next probe missing
  its *own* stamp. That is the moment the "cache = remembered answer" model breaks.

## 3. The grip (grounded — from recon §2)

> **The VM resolves a selector once per call *site*, not once per call.** Each `Invoke` site owns a
> one-entry memo — *(receiver class, method, version)* — and trusts it until a single global
> `world_version` counter says the world changed. One bump invalidates every site at once; the VM
> never hunts for stale caches, it lets each one fail its own stamp check the next time it runs.

Corollary (the honesty knot, §5.2 territory): the stamp is **global, not per-class**. Defining a
method on `Foo` invalidates the cache at a site that only ever sends to `Bar`. That coarseness is the
representation choice — and, crucially, it was the *fallback* option in the `U-IC` plan (DEC-IC-A
"acceptable v1"), **not** the recommended per-class epoch. It is an **absence of the planned
machinery**, not a reasoned choice of coarseness. This is the secondary predict-then-check.

The one-line representation fact that settles it: *the cache is a side table (`Chunk::caches`) keyed
by instruction position, and its invalidation stamp is one global `u64` counter — so a hit is a
`ClassId` + `u64` compare, and the price of that simplicity is that defining any method anywhere
flushes every cache in the program, lazily.*

## 4. The design space (walked, not listed)

The problem Doc 4 leaves open: a send is a hash + superclass chain-walk *per send*. A loop sending
`x foo` a million times pays that walk a million times, though the answer never changes. What can be
remembered, and how do you know when the memory has gone stale?

### 4a. Cache shape — how much does a site remember?

| Branch | Occupants | The bill |
|---|---|---|
| **No cache** — walk the dictionary chain on every send | naive Smalltalk-80, a tree-walker | Simplest; always correct; no invalidation problem at all. Bill: the walk is the interpreter's dominant cost — a hash + a per-superclass-level probe, every send. |
| **Monomorphic inline cache** — one slot per call site: *(class, method, stamp)* | early Smalltalk (Deutsch & Schiffman 1984), Phalcom | A hit is a class-identity + stamp compare; the walk runs only on the first send and after invalidation. Bill: a site fed *many* receiver classes (megamorphic) thrashes — every send misses, refills, and you pay the walk *plus* the failed compare. And you must detect staleness (invalidation). |
| **Polymorphic inline cache (PIC)** — a few slots per site, linear-scanned | SELF (Hölzle/Ungar 1991), V8, HotSpot | Handles a site with 2–N stable receiver classes without thrashing; the type feedback it gathers is what a JIT later inlines on. Bill: bigger slot, a small scan per hit, and a policy for when to give up (spill to megamorphic). |
| **Megamorphic fallback** — a site past N classes stops caching, walks (or hits a global method cache) | V8/HotSpot's giving-up state | Bounds the worst case: a wildly polymorphic site degrades to the no-cache cost, not worse. Bill: it *is* the no-cache cost for that site — the cache bought nothing there, and you spent slots discovering it. |

### 4b. Invalidation — how does a cache know the world changed?

The hard half. A cache that serves a method *removed or overridden after it was cached* is the classic
inline-cache soundness bug. The two poles:

| Branch | The bill |
|---|---|
| **Per-class / per-site epoch** — each class carries a version; a mutation bumps only the affected class's (and its subtree's) epoch; a slot stores the class epoch it was filled at | Fine-grained: redefining a method on `Foo` invalidates only caches that resolved *through* `Foo`; an unrelated `Bar` site keeps its cache. Bill: bookkeeping — you must bump the *right* epochs up the affected subtree on every mutation, and missing one site is a live-staleness bug. |
| **Single global version counter** — one `u64` on the VM; *any* method install bumps it; every slot stores the global value it was filled at | Trivially correct and zero bookkeeping: one increment invalidates the entire program's caches at once, lazily (each slot notices on its next probe). Bill: coarse — a method defined on `Foo` flushes a cache at a site that only ever touches `Bar`. In a workload that rarely mutates classes after warmup, the coarseness costs nothing; in one that mutates often, every mutation re-warms the whole program. |

**Phalcom runs the global counter** (`world_version`). Honesty framing (recon §3): the `U-IC` plan
*recommends the per-class epoch* (DEC-IC-A) and lists the global counter as the *fallback* — so the
doc must present global invalidation as **HEAD-truth and an absence of the planned fine-grained
machinery**, not as "chose coarse for simplicity." No document claims the latter; the plan says the
opposite.

### 4c. Fusion — a different lever (remove the dispatch, not the lookup)

Orthogonal to caching. A cache makes a *lookup* cheap; **superinstruction fusion** removes a whole
*dispatch* (a fetch-decode-execute turn of Doc 1's loop) at compile time by merging two adjacent
opcodes into one. `(GetLocal | Constant) → Invoke` becomes `InvokeLocal` / `InvokeConst`. The reader
needs the idea that **8.8% of instructions removed ≠ 8.8% of time saved** — a fusion buys *dispatch*,
and only dispatch-bound code can spend it (perf-log 008; recon §3).

## 5. Comparison filter

A language enters **only** if it (recon §4 cast): (1) took another branch with the bill attached;
(2) has a scar; (3) names something Phalcom does anonymously; (4) is an ancestor. Expect ~4–5 to
survive. Name the cut list.

Vocabulary to import (deliverable in itself): *inline cache* (monomorphic / polymorphic / megamorphic);
*call-site specialization*; *type feedback*; *hidden class / shape / map* (V8); *deoptimization*
(the guard-failure return to the interpreter); *version tag / epoch* (the staleness stamp); *global
method cache* (the megamorphic backstop); *superinstruction / token threading* (fusion's ancestor);
*peephole optimization*; *quickening* (rewriting a generic opcode to a specialized one after first use).

Provisional cast, subject to earning it:

- **Smalltalk-80 / Deutsch & Schiffman (1984)** — *the* origin of inline caching; names *inline
  cache*, the monomorphic slot. The ancestor. **Deep.**
- **SELF / Hölzle & Ungar (1991)** — invents the *polymorphic* inline cache and *type feedback*; the
  line from PICs to JIT inlining. Names what Phalcom leaves as the plan's PIC-extensible slot.
  **Deep, one section.**
- **V8 (JS)** — hidden classes/shapes + the monomorphic→polymorphic→megamorphic ladder + *deopt*; the
  scar the reader has personally hit (a "megamorphic" perf cliff, a deopt loop). **Medium.**
- **CPython PEP 659 (3.11 adaptive specializing interpreter)** — *very* close analogue: per-instruction
  specialization guarded by a **version tag**, `LOAD_ATTR`/`LOAD_GLOBAL` specialization, "quickening."
  This is the closest living relative of exactly this doc's mechanism (a bytecode interpreter, not a
  JIT, caching at the instruction). **Medium–deep.**
- **Forth / threaded code** — the *superinstruction* ancestor; fusion of common opcode pairs is a
  1970s threaded-code technique. Names fusion's lineage. **Short.**

Likely **cut** (name them): **JVM/HotSpot** as a *separate* entry (its monomorphic/bimorphic call-site
story overlaps V8's ladder for this axis — fold a line into V8 or SELF rather than a standalone
section); **Ruby** (its open-class invalidation cost is the *motivation* Doc 4 already forward-pointed
— reference it, don't re-teach it); anything about JIT compilation proper (Phalcom has no JIT; PICs
here buy interpreter speed and would buy type feedback *if* a JIT existed — say that once, don't
survey JITs).

## 6. Tensions to surface

- **Cache ⊗ the dynamic object model** — the whole reason invalidation is hard is that Phalcom lets
  you reopen a class and redefine a method at runtime (Doc 4's reopen fixture). A static language
  never faces this; the cache exists *because* binding is late, and must be *undone* because binding
  stays late forever. Ties directly to Doc 4.
- **Global stamp ⊗ unrelated mutation** — the coarseness knot (grip corollary): one counter means a
  mutation to `Foo` costs `Bar`'s cache. This is the secondary predict-then-check and the honesty
  spine's payoff.
- **Fusion ⊗ jump targets** — the in-place rewrite (dead `Invoke` left at `ip+1`) is sound *only* if
  no branch targets the second instruction; `branch_targets()` guards it. A tension between "keep
  `code.len()` stable so side tables stay aligned" and "a jump could land mid-superinstruction."
- **Fusion ⊗ the IC slot** — the fused arm reads its cache slot at `ip+1` (the dead `Invoke`'s own
  slot), so fusion and caching *compose* — the fused send probes the same IC the unfused pair would.
  A nice tie: the two levers were designed to coexist.
- **Two caches, one shape, deliberately split** — `caches` (method IC) and `gcaches` (global-var
  cache) are the same idea (side table + version stamp) but kept as separate parallel tables because
  the two opcodes never share an instruction and a union would widen every slot. Names *why* there
  are two.

## 7. Structural rules (constraints, not a skeleton)

- **Structure follows the theory.** No imposed heading set. Bottoms out where the theory bottoms out.
- **No checkbox comparative table.** Comparison is a weapon aimed at one named confusion.
- **Trace invalidation, and that as the hard case.** The hit is the reader's intuition; the miss-on-
  stale-stamp (warm cache → `Method` install bumps `world_version` → next probe fails its own stamp →
  re-resolve) is the strange one. Trace from real observed output (the reopen fixture).
- **Present the cache-shape fork honestly as scaffolding**: HEAD did not bench-race no-cache /
  monomorphic / PIC; the seam (ADR-0012) got populated monomorphic-with-global-stamp. Land the two
  genuinely-deliberated things: the **seam** (ADR-0012) and **fusion** (perf-log 008, *measured*).
- **The landed-vs-planned split is the honesty spine** — state plainly what runs (global counter,
  mixed `Symbol`, `IndexMap` dict, cut-008 fusion) vs what `U-IC` plans (selector interner, per-class
  epoch, design-B arrays, operand-free loads). Cite the plan as *intent*, HEAD as *truth*.
- **Mermaid only where the shape is the point** — the probe/hit-or-miss/refill two-move, or the
  invalidation timeline. Not decoration.
- **Source anchors: symbol first, line second** (`vm/dispatch.rs::VM::invoke_at` @ ~L408). Bare line
  numbers rot.
- **HEAD as-implemented.** Cite spec/ADR/plan intent as intent, clearly labelled.
- **Mark every simplification as a lie with a forward pointer.** (Recon §6 list, §8 below.)
- **Perf claims quote a number or say "unmeasured."** perf-log 008 has real numbers — use them; the
  ~3.3 ns dispatch, the result table, the `map_numeric` non-result. Do not invent a hit-rate.

## 8. Lies to mark forward / defer (recon §6)

1. **Per-class epoch** — the fine-grained invalidation `U-IC` recommends (DEC-IC-A) is **planned, not
   built**. Present global invalidation as HEAD-truth; mark the epoch as the planned refinement,
   clearly labelled unbuilt. Not a "lie the next doc destroys" — a genuine unbuilt-future; cite the
   plan as intent per AUTHORING truth-basis.
2. **Polymorphic IC (PIC)** — HEAD is monomorphic (one entry). PIC is A's theory and the plan's P4
   ("slot layout must not preclude PIC"). Mark as not-built; it is where the SELF/V8 vocabulary lands
   even though Phalcom doesn't run it.
3. **Selector-only interner / design-B own-method arrays** — `U-IC` Changes 1–2, not built. Mention
   only as the planned redesign; do not describe as current. (B confirms `Symbol` still mixed.)
4. **Sacred-selector inliner mechanism** (ADR-0018) — `GuardBool`/`GuardBlock` get a brief mention as
   a *third* fast-path family; the inliner's full mechanism (`compile_sacred_call`, override-epoch
   deopt) is **its own future topic**, deferred.
5. **`SuperSend` is uncached** (DEC-IC-B, DEFERRED) — a statically-known target left out of the IC in
   v1. One sentence; ties back to Doc 4's SuperSend forward-pointer.

## 9. Checklist (gate before shipping — maps to AUTHORING §6)

- [ ] Grip stated early, one sentence ("resolve once per *site*, not per call"), *earned* by the end.
- [ ] Call site shown to hold an **empty cache slot** that fills on first send — from the `caches`
      side table, keyed by `ip`, not the `Invoke` operand.
- [ ] Every rejected cache-shape branch made tempting before it is set aside.
- [ ] The **invalidation fork** (global counter vs per-class epoch) walked; global framed as
      HEAD-truth *and* absence-of-planned-machinery, per-class epoch marked planned-unbuilt.
- [ ] The two genuinely-deliberated things landed: the **seam** (ADR-0012) and **fusion** (perf-log
      008, with the real numbers and the F16-verdict-flip / `map_numeric` non-result scar).
- [ ] ≥1 predict-then-check moment (primary: redefine `foo` mid-loop — does the warm site notice?;
      secondary: define on `Foo` — is unrelated `Bar`'s cache flushed?).
- [ ] **Invalidation** traced step by step from real observed output (reopen fixture v1→v2), as the
      hard case — not the textbook hit.
- [ ] The **landed-vs-planned split** stated plainly; plan cited as intent, HEAD as truth.
- [ ] The **U-IC hazard** shown: `receiver_class`/`world_version` read *after* `lookup_method`, and
      *why* (a re-entrant send during lookup could bump the world).
- [ ] Fusion's in-place rewrite explained: dead `Invoke` at `ip+1`, `code.len()` stable, side tables
      stay aligned, `branch_targets` guards jump-into-superinstruction.
- [ ] `gcaches` covered as the same shape on a *different* stamp (`globals_version`); why split from
      `caches`.
- [ ] Guard opcodes (ADR-0018) mentioned as a third fast-path family — **brief**, marked as its own
      future topic.
- [ ] Every language present passes §5; named cut list.
- [ ] Vocabulary imported and findable.
- [ ] Anchors symbol-first and exist at HEAD.
- [ ] Every lie marked with a forward pointer.
- [ ] Claims ledger clean: perf claims quote a number (perf-log 008) or say "unmeasured"; comparative
      claims cited or cut; all links resolve.
- [ ] Reader could re-derive the design. (§0)

## 10. Build sequence

| # | Deliverable | Who | Path |
|---|---|---|---|
| 1 | `recon.md` | me | done |
| 2 | This file | me | `REQUIREMENTS.md` |
| 3 | Theory draft — no source access | sonnet A | `draft-concept.md` |
| 4 | Source map — graphify-led, fixtures run live | sonnet B | `source-map.md` |
| 5 | The doc — synthesis, my judgment over A's bulk + B's ground truth | me | `../vm/caches-and-fusion.md` |

## 11. Open risk

Recon is bounded; Agent B goes deeper. Assumptions the doc rests on that B must confirm or the doc
bends:

1. **The method IC is keyed on a single global `world_version`, not a per-class epoch.** The entire
   grip + honesty spine rest on it. If B finds a per-class epoch at HEAD, §3 and §4b flip. *(Recon
   read `world_version: u64` at `vm/mod.rs` ~L116 and the unconditional bump at dispatch.rs
   ~L927/L930 — high confidence, but B verifies and confirms there is no per-class epoch.)*
2. **The cache is a side table (`Chunk::caches`) keyed by `ip`, not stored in the `Invoke` operand.**
   The representation fact §3 rests on it. If B finds the cache inline in the bytecode, §3's "side
   table" framing is wrong. *(Recon read `caches: Vec<Cell<Option<InlineCache>>>` at chunk.rs ~L50.)*
3. **`receiver_class`/`world_version` are read *after* `lookup_method`, and this ordering is
   load-bearing (the U-IC hazard).** If B finds they are read before, the "re-entrant send during
   lookup could bump the world" beat is wrong. B must quote the ordering in `invoke_at`.
4. **The `U-IC` plan did NOT land — `Symbol` is still a mixed space, the method dict is still an
   `IndexMap`, and the per-class epoch was never built.** The landed-vs-planned honesty spine rests
   on this. B must confirm each of Changes 1–3 is unbuilt and that the global counter (the plan's
   fallback) is what runs.
5. **Fusion is behaviour-invariant and composes with the IC** — the fused arm reads the same IC slot
   at `ip+1` the unfused pair would. If B finds the fused arm allocates a fresh slot or skips the
   cache, the "they compose" tension (§6) is wrong. *(Recon read perf-log 008's account; B confirms
   against `dispatch.rs` fused arms and `chunk.rs::fuse_superinstructions`.)*
6. **A method redefined after compile takes effect on an already-warm call site** (the invalidation
   proof). B must **run** the reopen fixture and report observed v1→v2 output, not read it off code.
