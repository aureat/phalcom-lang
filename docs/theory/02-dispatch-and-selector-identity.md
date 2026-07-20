# 02 — Dispatch and selector identity

> **Thesis:** the encoding of a method name is not a naming convention. It is the primary
> design lever of a message-passing language. What you fold into the dispatch key determines
> which features are expressible, which are impossible, and which arrive free without anyone
> designing them. Phalcom's key bought pattern-match exhaustiveness for nothing and made
> default arguments unimplementable — from the same decision.

---

## 1. What a call site actually names

**`[V]`** A Phalcom call site compiles to `Invoke(arity: u8, selector_const_idx: u16)`. That is
the whole instruction. There is no method handle, no vtable offset, no resolved target — nothing
is bound at compile time. Late binding is total.

The lookup ladder at runtime has four rungs, and each is worth a note:

1. **Inline-cache probe.** A per-site cached `(class, method, version)` triple.
2. **Exact `lookup_method`** on the receiver's class, walking the superclass chain.
3. **Variadic probe** — a second key shape for rest-parameter methods.
4. **Reify and forward** — build a first-class `Message` and send `doesNotUnderstand(_)`, which
   user code may override.

**`[V]`** Two structural facts that a reader of the dispatch loop should not have to discover the
hard way. First, **not every send pushes a frame**: `call_method` forks on
`MethodKind::{Primitive, Closure}`, and only the closure arm builds a `CallFrame`. Primitive sends
run in Rust and never appear in the guest frame stack — which is exactly why they are invisible to
a guest-level traceback and why they create the native-reentrancy hazard discussed in
[`01`](01-coroutines-and-the-suspension-problem.md). Second, **`doesNotUnderstand` is itself a
send**, so it must not re-enter on a class that lacks it; the recursion guard is not optional
decoration.

**`[R]`** The lineage: Smalltalk contributed `doesNotUnderstand` and `perform`; Objective-C made
the same idea a C function (`objc_msgSend`) with a documented forwarding chain; Ruby's
`method_missing` is the same mechanism and, notably, comes with the same optimization bill.
C++ vtables are the road not taken — an offset resolved at compile time, fast and closed.

---

## 2. Selector encoding as a design lever

**`[V]`** ADR-0012 rejected arity-only dispatch for a concrete reason: it cannot distinguish
`move(to:duration:)` from `move(_,_)`. The chosen key encodes name *and* labels *and* kind. The
canonical forms, per `SignatureKind`:

| Kind | Encoded selector | Note |
|---|---|---|
| getter | `size` | deliberately **not** `size()`, so a property is distinct from a 0-arg method |
| method | `name(_,label)` | comma-canonical; labels are part of identity |
| setter | `name=(_)` | |
| subscript | `[slots]` | the name is ignored — the bracket *is* the identity |
| variadic | `name(*)` | arity payload deliberately **not** spelled into the selector |

Each row encodes a decision. Making a getter's selector bare rather than 0-arity means properties
and no-argument methods occupy different namespaces and can coexist. Making the bracket its own
identity means subscript is a real selector rather than sugar — a position this project reached
only after retiring the opposite one (ADR-0055 retired, superseded by ADR-0060).

**`[V]`** Three real defects came from having *two* encoders — compiler and runtime — that could
drift: a dropped `Result` that silently swallowed errors, a mis-tagged 0-argument `new()`, and a
divergent encoder that interned a stray space. All three were fixed by forcing both paths through
a single `encode_selector` helper.

**The transferable rule:** any canonical string computed independently in two places is a latent
bug factory, and the bugs it produces are *silent lookup misses* rather than crashes — the
hardest class to notice, because a missed lookup falls through to a plausible fallback path.

---

## 3. The same decision, two opposite consequences

This is the most instructive pairing in the whole project, and it is worth presenting as a
single phenomenon rather than two unrelated decisions.

### Free exhaustiveness

**`[V]`** From `docs/design-notes/eliminator-convention-and-sacred-match.md`:

> Phalcom's dispatch-key design (name+arity+kind selector identity, ADR-0012) gives eliminator
> totality **without** a Maranget-style usefulness algorithm. `match(ok:)` and `match(ok:, err:)`
> are different selectors — a caller who forgets an arm doesn't get silent fallthrough, they get a
> missed method lookup and `doesNotUnderstand`.

Because labels are load-bearing in the key, an eliminator method `match(some:, none:)` cannot be
called with only one arm. The arity-and-label-complete call is a *different selector* from the
incomplete one, and the incomplete one is simply not installed. This lands remarkably close to
ML's compile-time totality with **zero static analysis** — and Phalcom has no flow analysis at
all.

**`[R]`** The comparison table, which is what makes the claim meaningful rather than a boast:

| Language | Exhaustiveness | Cost |
|---|---|---|
| Ruby `case/in` + `deconstruct` | none, ever | partial matches silent until an input hits the gap |
| Scala `unapply` | only over `sealed`; extractor arms opaque to the checker | `MatchError` at runtime even in "checked" code |
| Rust / OCaml / Haskell | total | closed-world: a new variant is a breaking edit to every match |
| Smalltalk | nothing native (`caseOf:` was a Squeak hack, later dropped) | you nest, and it is ugly |
| **Phalcom** | **total for eliminators, via selector identity** | **only for closed variant sets; no destructuring** |

### The impossibility of default arguments

**`[V]`** The identical property makes default arguments unimplementable. Omitting a defaulted
argument produces a *different selector*, so lookup misses the full-arity method entirely. The
only available repairs were combinatorial arity-family expansion at install time, or static
knowledge of the callee that a dynamically dispatched language does not have.

ADR-0043 resolves it **by declining the feature**. Arity is fixed and 1:1 with signature
identity; manual arity overloading (`foo`, `foo(_)`, `foo(_,_)` as separate methods) is the
idiom. The stated cost is repetitive overloads; the stated benefit is that the single-probe
lookup is preserved with no signature aliasing at install and no arity-fold at the call site.

**`[V]`** A later amendment closes a loophole in a way worth copying as a documentation practice.
The original record left "aliasing versus call-site fold" as an open choice; the amendment
forbids call-site fold permanently and adds:

> a superseding ADR inherits that constraint; it does not get to re-open it.

Without that sentence, a future record could clear the stated bar while doing precisely what the
original meant to prevent. **A decision that forbids an outcome should say whether it also binds
its own successors** — otherwise "superseded" becomes a laundering mechanism.

### Why the pairing matters

Same key, two consequences, opposite valence. There is no version of this design where you get
the free exhaustiveness and the default arguments; they are the same property viewed from two
sides. That is what a real design axis looks like, and it is why "add default arguments" was never
a small feature request. **`[V]`** The overlay catalogues it as a canonical hazard:
*default args ⊗ selector-identity dispatch*.

---

## 4. Inline caching: the lineage and the honest gap

**`[R]`** The canonical lineage is Deutsch and Schiffman (1984), which introduced the monomorphic
call-site cache alongside bytecode-to-native translation for Smalltalk-80, and Hölzle, Chambers,
and Ungar (1991), which generalized it to polymorphic inline caches — several receiver classes
cached per site, with the cache doubling as type-feedback for an optimizing compiler. Both are
listed in `references/reading.md`; neither has been read against a primary source in this
project.

**`[V]`** What actually shipped in Phalcom:

```rust
InlineCache = { class: ClassId, method: ObjRef, version: u64 }
Chunk::caches: Vec<Cell<Option<InlineCache>>>
```

Three implementation choices worth extracting. The cache is a **side table keyed by instruction
pointer**, not inlined into the operand — so the instruction encoding stays untouched and the
cache can be added, resized, or removed without a bytecode change. `Cell` supplies interior
mutability so a refill can occur through a shared `&Chunk` borrow, which is what makes the cache
compatible with sharing compiled artifacts across closures. Global-variable caches live in a
*separate* vector with a **per-module `globals_version`** rather than the global stamp, so the
two invalidation domains never widen the same slot.

**`[V]`** And the honest part, which is the reason to read this section: the plan recommended
**per-class epochs** as the primary invalidation scheme with a global counter as fallback. HEAD
shipped **the fallback**. Any method definition anywhere lazily invalidates every cache slot in
the program. The record explicitly notes this was never framed as a deliberated simplicity
choice — it is the *absence* of the per-class machinery.

**The generalizable lesson**, and it applies far beyond caches: *shipped is not the same as
designed*. When a plan names a primary and a fallback, and the fallback ships, the record must
say so explicitly or every future reader will assume the primary is what exists. This is the
same class of error as the citation incident — a true artifact (the cache works) attached to a
false account of its provenance (that it is what was designed).

---

## 5. Speculation without a cache: the pristine-flag scaffold

**`[V]`** Before any general inline cache, the VM had exactly two booleans:
`bool_sacred_pristine` and `block_sacred_pristine`. Each is cleared *forever* by
`note_method_installed()` if the installed selector falls in a fixed set — `and(_)`, `or(_)`,
`not()`, `ifTrue(_)`, `ifFalse(_)`, `ifTrue(_,ifFalse:)` for Bool; `whileTrue(_)` for Block.

The compiler inlines those six shapes to `Jump`/`Loop`/`GuardBool`/`GuardBlock` opcodes when the
block arguments are **literal block expressions at the call site** — not variables holding
blocks. `GuardBool` requires *both* a `Bool` receiver *and* a live flag; on failure it jumps to a
fallback real-send path the compiler emitted alongside the fast path.

Three things make this a good miniature of speculative optimization in general:

**The deopt target is emitted eagerly, not reconstructed.** There is no "materialize a frame from
metadata" machinery. The compiler simply emitted both paths. **`[V]`** The cost is a genuine
one and it bit: sacred calls emit their arms twice, so without suppression, the fallback copy
recursively inlines its own nested conditionals and code size doubles per nesting level. A
14-deep conditional in a string method took ~200 ms to compile and added a **fixed 175 ms to
every process startup**. The fix — a `deopt_fallback_depth` counter suppressing inlining inside
fallback copies — is safe *precisely because* the inliner is a guarded optimization rather than a
semantic requirement: a non-inlined conditional compiles to the same program. Anyone emitting
dual fast/slow paths from one AST node needs that counter, and will discover it as a startup
regression rather than as a compile-time one.

**The generalization path was stated in advance.** Replace the two booleans with a
`world_version: u64`, bump it on every install, store the version in each cache entry. The
scaffold was designed to be thrown away.

**`[X]` A belief about its coverage was false and survived for weeks.** The stated reason for
deferring an arithmetic-inlining unit was "the inliner already covers arithmetic." It does not.
The sacred set is control-flow only; `+`, `-`, `*`, `/`, `<`, `<=` are ordinary message sends, and
`1 + 2` still compiles to `Constant, Constant, Invoke` — about 20% of instructions in the
arithmetic benchmark. A deferral *reason* survived unexamined because it sounded right. Deferral
rationales are claims, and they decay like any other.

---

## 6. `super` is a different question

**`[V]`** `SuperSend(argc, selector, defining_class_name)` bakes the *defining class name* at
compile time and starts lookup at that class's superclass. Note what this is not: it is not
"start at the receiver's class's superclass," which is the naive reading and is wrong in the
presence of a three-level hierarchy where a method is inherited — that version loops forever.

**`[V]`** The invalidation story shows how one decision can retire another's obligation. The
original record noted that a cached `SuperSend` must invalidate when a superclass is reassigned.
Once ADR-0026/0041 sealed reparenting, that half became moot; only the override-epoch half
(method redefinition) still applies. **Sealing one axis of mutability deletes an entire
invalidation case rather than making it cheaper** — which is the strongest available argument for
sealing, and a better one than the usual performance framing.

---

## 7. What it costs to have no static shape

**`[M]`** Dispatch in this VM costs about **3.3–3.6 ns**, established by two independent
instruments that converged: a differential measurement over two programs differing by a
histogram-verified 6 M near-empty instructions, and reading a shipped optimization's results
backwards as (Δwall ÷ dispatches removed).

The decisive negative result is the one to remember. A row that removed **18 M dispatches** — the
most of any measured row — moved **−0.2%**, because its instructions cost 27.6 ns each (hashing,
allocation, collection). Stated as a rule:

> A fusion buys dispatch, and only workloads whose time *is* dispatch can spend it.

**`[M]`** And the counterpoint that shows where protocol design meets dispatch cost: building a
list of 1 M elements takes 0.42 s, a scalar loop over 1 M takes 0.34 s, and building-plus-iterating
takes **12.21 s** — against 0.76 s if the costs were independent. The cursor protocol
(`iterate` → `iteratorValue` → sentinel check, three dispatches per element) is roughly 30× a
scalar loop. **`[V]`** The protocol was chosen anyway, with the ~46% figure stated up front,
because it makes `for-in` compile to the same Invoke-only loop as a user-defined iterable and
therefore makes user iterables first-class rather than second-class.

That is the whole trade of a message-passing language in one measurement: uniformity is bought
with dispatch, and dispatch is what an inline cache exists to give back.
