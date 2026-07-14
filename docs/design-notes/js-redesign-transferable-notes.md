# Cross-language design notes — JS redesign session (2026-07-14)

**Status: UNMEASURED. Hypotheses only.** These came out of an abstract discussion
about redesigning JavaScript from scratch, not from profiling Phalcom. Nothing here
is a finding.

Under [ADR-0051](../adr/accepted/0051-performance-strategy-measure-first-tiered-optimization.md)
(measure-first, tiered, behavior-invariant) and perf-log law **P1** (no oral numbers),
none of this may land as an optimization without a U-BENCH before/after. Deliberately
kept out of [`../forge/perf-log/`](../forge/perf-log/README.md), which is a measured
ledger — filing speculation there would corrupt it.

Purpose: record the transferable reasoning while it's fresh, and mark which items
**cross-validate**, **contradict**, or are **orthogonal to** what the perf-log already
measured.

---

## The load-bearing thesis: mechanism vs policy

JS's core defect isn't `==` or `this`. It shipped **policy where it needed mechanism**.

`Promise`/`async`/`await` is policy. It baked one concurrency shape into the language
and colored every function, forcing a duplicate higher-order universe (`map` vs async
map, iterator vs async iterator). The mechanism it needed was stack switching — then
`Promise` is a library and a wrong `Promise` is fixable.

Prototype chains are mechanism, but *unrestricted* mechanism. Because any object's
shape can change at any time, V8 had to build hidden classes + inline caches + deopt:
an entire speculative machine to re-derive at runtime what the programmer already knew
and had no way to state.

**Phalcom bearing.** Phalcom already took the mechanism road on concurrency — U-FIBER
is stackful (`fiber call/yield/try/current/abort`). The note to self is defensive: do
**not** later add `async`/`await` sugar over fibers. That would re-introduce coloring
on top of a primitive that exists specifically to avoid it. Scheduling policy belongs
in the library; the fiber stays the mechanism.

---

## Items that cross-validate existing measured findings

### Allocation is the mechanism, not dispatch — confirms F1

The JS analysis lands independently on the same place perf-log **F1** measured:
no amount of speculation removes an *allocation*. V8 can reach ~2× native on
monomorphic numeric code and still lose to the allocation firehose (every object
literal, every closure, every VDOM node).

Phalcom measured this from the other direction: malloc/free is the **#1 attributed
mechanism** (arith 19.7%, Skynet 28.2%), *above* tracing span and dispatch lookup.

Two independent routes to the same ranking. Raises confidence in F1's re-ranking of
`perf.md` §2, and in **F7**'s framing that the `Object` 280 B → ~40 B arena-density
target is the shape of win that actually pays.

### Niche-encoded absence — confirms F2

The redesign argument says: one absence, niche-encoded, no `null`-and-`undefined`.
Phalcom already has this, and **F2** measured the consequence — `List/Map.at` is
already zero-alloc via the `None` singleton, which is exactly why the `Option`-escape
optimization premise was falsified. Nothing to do. Recorded because the JS reasoning
predicts F2's result, which is a small validity check on the reasoning.

---

## Items that are genuinely new and unmeasured

### N1 — Bootstrap image / precompiled `core.ph` bytecode (startup lever)

**Precedent with a number.** Hermes (no JIT, AOT bytecode, ship-precompiled) beat JSC
on React Native cold start. The claim generalizes: for short-lived processes, *warmup
dominates peak throughput*, and a JIT is a net loss. BinAST tried to bring a binary
format to the web and died of back-compat; a language with no back-compat constraint
just ships it.

**Phalcom bearing.** `phalcom-core/core/core.ph` is lexed, parsed, and compiled on
**every** startup. That's a fixed cost paid by every CLI invocation, every REPL start,
and every golden test in the suite. An image or serialized-bytecode format would skip
it entirely.

**Unmeasured and unranked.** No idea what fraction of startup this is. Before anyone
builds it:

1. Measure cold-start wall time, split parse / compile / execute for `core.ph`.
2. Check it against the golden suite's total runtime — if the suite runs `core.ph`
   bootstrap N times, the lever is worth N×.
3. Only then decide. Could easily be noise.

Note this is a **startup** lever, not a throughput lever. The U-BENCH harness
(`benchmarks/vm/`) measures steady-state sends; it would not see this at all. Would
need a new bench lane. That cost counts against the lever.

### N2 — Macros rewrite syntax; they do not inform codegen

From working through Swift's `@Observable` in detail. It is an attached macro
(`memberAttribute` + `member` + `extension` roles) that rewrites every stored property
into a computed property backed by an underscored twin, injects an
`ObservationRegistrar`, and adds `access(keyPath:)` / `withMutation(keyPath:)` calls
into the getter and setter.

The critical property: **the compiler learns nothing about the dependency graph.**
Every edge is discovered at runtime by executing a read that mutates a lock-protected
dictionary through a thread-local access list. The macro saved typing. It bought zero
codegen knowledge. Per-read cost is a computed getter + `access()` call + thread-local
lookup + lock acquire + dictionary insert.

**Phalcom bearing, and this one is live.** The recent attribute-expansion work
(`expand_class_attributes`, `derive_construct`, `derive_accessors`, the `@get`/`@set`
expanders) is exactly this shape: expanders produce AST, that AST compiles normally.

So: **no perf claim may be attached to an expander.** `@get`-derived accessors are not
faster than hand-written ones — they *are* hand-written ones, generated. If derived
accessors should be fast, the win has to come from the compiler or VM recognizing the
accessor shape (direct slot load, no send), never from the expansion step. Worth
stating explicitly before someone benchmarks `@get` and reports a delta that is really
just noise or a difference in what got generated.

### N3 — Sealing as an IC precondition (check against F4)

The JS lesson: ICs are the tax you pay for not knowing shape statically. Phalcom pays
that tax by design — Smalltalk semantics, method lookup on signature symbols, mutable
hierarchy. Correct, and not up for renegotiation.

But perf-log **F4** lists U-IC's unmet preconditions (`Symbol` is one mixed namespace
and needs a selector-only interner first; the IC seam is a comment only; no
`ClassObject` epoch or global `world_version` yet). And the U-GC root work in **F6**
surfaced `sealed_classes` as a root, so *some* sealing concept already exists in the
tree.

**Open question, not a plan:** does the existing sealing let some call sites skip the
guard entirely rather than merely cache-and-verify? If a sealed class cannot be
reshaped or have methods redefined, a site whose receiver class is provably sealed
needs no epoch check. That would make sealing an IC *accelerant*, not just a GC root.

Requires reading what `sealed_classes` actually guarantees today. Do not design on
this until F4's preconditions land — per the hazard **inline cache ⊗ mutable
hierarchy**, an IC without an invalidation story is unsound, and sealing is only a
shortcut *after* the epoch mechanism exists, not instead of it.

---

## WASM: what transfers and what doesn't

Follow-on question in the same session: does WebAssembly already fix this? Answer that
survived: it fixes the **representation and format** axes and none of the mechanism
axes. It's an abstract machine sitting *below* this whole design space. Three things
fall out that bear on Phalcom.

### Cross-validation: one heap (confirms ADR-0009)

WASM's worst structural failure in the browser is **two heaps** — linear memory on one
side, JS/DOM objects on the other, and no cross-heap cycle collector. An `Rc` holding a
JS closure holding the Rust struct leaks forever. There is no fix, only discipline.

Phalcom's handle/arena `Heap` ([ADR-0009](../adr/0009-handle-arena-heap.md)) is one heap
for the whole object graph. Not a lever — a hazard already dodged by construction.
Recorded so that nobody later proposes a native side-table for some hot structure and
reintroduces a second heap for a perf reason. That trade has a known, unbounded cost.

### N4 — predictable vs peak is a real axis, and N1 sits on it

This sharpens **N1** and is the most useful thing the WASM detour produced.

Two facts, both true:

- Hermes (AOT bytecode, no JIT, ship-precompiled) beat JSC on React Native cold start.
  That's N1's premise.
- WASM (AOT, statically typed, no deopt, no warmup) is routinely *matched or beaten* by
  warm TurboFan on non-numeric workloads — because TurboFan has **runtime type feedback**
  and WASM's compiler has none.

Reconciliation: AOT trades peak for predictability. It wins where warmup dominates (short
process, cold start) and loses where a JIT's feedback would have specialized. N1 is
already filed correctly as a *startup* lever; the WASM evidence is *why* it can never be
re-sold as a throughput one. A precompiled `core.ph` image must not be expected to speed
up steady-state sends, and U-BENCH (`benchmarks/vm/`, steady-state sends) is blind to it
by construction.

Second-order, and more interesting: **Phalcom is already AOT-shaped.** No JIT, no type
feedback, every send walks the dict (ADR-0012). So the WASM comparison is the relevant
one, not the Hermes one — the ceiling on a bytecode VM with no feedback is a real ceiling,
and the sacred-selector inliner (control-flow §3) plus IC population (ADR-0012, deferred)
are precisely the feedback mechanisms that would raise it. That reframes IC from
"nice-to-have someday" to "the thing that buys back what AOT already gave up."

Still unmeasured. **F4**'s preconditions still gate U-IC and nothing here changes that.

### N5 — the boundary tax is a design constraint, not an implementation detail

WASM's worst *practical* cost isn't codegen, it's crossings. Every string crossing is a
copy plus a transcode (JS is UTF-16; WASM has no string type at all). `stringref` stalled;
JS String Builtins shipped to patch the wound. The generalizable lesson: **a boundary
between two representations is paid per call, and if the workload *is* boundary calls,
the speed of the fast side stops mattering.**

Phalcom bearing, stated as a question because I did not read the string primitives:

The native/library split (ADR-0006 — VM-native arithmetic, `Object`, `Bool`, block `call`,
absence, dispatch; the rest self-defined in `core.ph`) is a boundary. The overlay already
names its trade — "smaller native surface = more auditable, slower hot ops." What is
*not* recorded anywhere is whether any **representation** changes across it. If a Rust
primitive and a `.ph` method exchange the same `Value` enum, the crossing is nearly free
and there is no tax to find. If anything transcodes — strings are the obvious candidate —
then there's a WASM-shaped cost hiding in the split that no dispatch optimization touches.

Cheap to check, worth checking **before** anyone moves a hot op across that boundary in
either direction, since that's the decision the answer would change.

### The target/source distinction

WASM inherits whatever the source language chose. Rust-on-WASM is still colored, because
*Rust* is colored. Compile any language to WASM and you still write a signals library
for reactivity, because WASM has no opinion on dependency graphs. It unbundled "the
browser's language" from "the language you write" — that's its real contribution, and it
makes the design question *askable*, not answered.

Two bearings:

1. **Reinforces the mechanism-vs-policy note at the top of this file.** Phalcom's fibers
   are stackful and uncolored (concurrency §1–2, U-FIBER). A stackful *target* does not
   protect a colored *source*. So "we have fibers" is not a defense against later adding
   `async`/`await` sugar. Only not adding it is.
2. **Phalcom's bytecode is a target.** Any argument opening "the bytecode should…" must
   first say whether it's a source-semantics claim or a target claim; they have different
   burdens of proof. Related: there's no external bytecode loader today, so no verifier is
   needed *yet* — but one becomes mandatory the moment anything loads a *compiled* unit
   rather than compiling from source. Modules landed as
   [ADR-0027](../adr/0027-modules-as-files-with-public-by-default-imports.md)
   (files + public-by-default imports); whether that ever admits precompiled units is a
   separate question I did not check. WASM's validate-before-run is the precedent for what
   the verifier has to look like if it does.

---

## Design principles worth keeping

### Speculative optimization ⊗ observable semantics

Any fast path must deopt to *exactly* the slow-path result. An optimized path that
differs on overflow, on a side effect, on a redefinition, or on a subclass override is
a correctness bug wearing a performance hat. Every fast path needs a guard that
provably implies the slow path.

The reactivity version of this trap: memoizing a `derived` is unsound unless the
computation is checkably pure, because a side effect inside it becomes observable
through the memo. Swift's `@Observable` sidesteps it by never memoizing at all —
computed properties re-run on every read, transparently *tracked* but never *cached*,
and no purity check exists. That is a real cost, honestly paid.

Phalcom bearing: any memoization or caching added to the VM needs a purity or
invalidation story stated up front, not retrofitted.

### The workarounds don't compose

The strongest argument for a clean redesign was never that any individual fix is
better. It's that each existing fix is **capped by its weakest neighbor**, and the
taxes compound: TypeScript's types can't inform V8's inline caches (same information
computed twice, one copy discarded); Svelte's compile-time knowledge dies at the module
boundary; the bundler can't trust `sideEffects` because `eval` might exist three
dependencies down; React Compiler bails on aliasing *because* the language has no value
types.

A coherent design pays once and collects several times: static shape feeds codegen feeds
dead-code elimination feeds the binary format feeds startup.

Phalcom bearing: this is an argument for spec-first coherence over local patches, which
the forge method already encodes. Recorded as a sharper articulation of why, not as a
new practice.

### The honest counterevidence

Worth writing down because it cuts against the whole redesign instinct. The experiment
was run. Google shipped **Dartium** — a JS-replacement VM in Chrome with sound types
and AOT. It lost, on ecosystem politics, not merit; Dart survives via Flutter, i.e. by
abandoning the DOM entirely. **Elm** took nearly the same bet — sound, no runtime
exceptions, own the architecture — and stayed tiny, killed by interop cost.

Meanwhile the things that *did* win the web (CoffeeScript on syntax, TypeScript on
types, JSX on templating) all won by **compiling to JS**, i.e. by accepting the
semantics they were fixing.

The lesson that transfers: the set of problems only a redesign solves is exactly the
set no compile-down tool can reach — which is exactly why those problems are still
open. Being right about the design is not sufficient and never was.

---

## What this session did **not** produce

- No measurements. Not one number about Phalcom.
- No spec change, no ADR, no unit.
- Nothing that may enter [`../forge/perf-log/`](../forge/perf-log/README.md) until a
  U-BENCH before/after exists.

Actionable residue, ranked:

1. **N2** — a live guard against a false perf claim on the attribute expanders. Costs
   nothing, applies now.
2. **N5** — one bounded look at whether any representation transcodes across the
   native/`core.ph` boundary. Cheap, and it's the thing that would change a decision.
3. **N1 + N4 as a pair** — a real but unranked startup lever, *plus* the constraint that
   it may never be re-sold as a throughput lever. N4 also reframes why IC (F4) matters:
   an AOT VM with no type feedback has a ceiling, and IC is what buys back the peak that
   AOT traded away.
4. **N3** — an open question, gated behind F4's preconditions.

Cross-validations (no action, confidence only): allocation-is-the-mechanism confirms
**F1**; niche absence confirms **F2**; one-heap confirms **ADR-0009**.
