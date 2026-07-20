# 06 — Mechanism versus policy

> **Thesis:** most durable language defects are not wrong features. They are *right features
> shipped at the wrong altitude* — a policy baked into the language where a mechanism belonged,
> or an unrestricted mechanism exposed where a constrained one belonged. Policy in the language
> cannot be fixed by a library; unrestricted mechanism cannot be optimized without building a
> speculative machine to re-derive what the programmer already knew.

**`[V]`** The material in this file is drawn from `docs/design-notes/js-redesign-transferable-notes.md`
(2026-07-14), whose own status banner reads **"UNMEASURED. Hypotheses only."** That banner is
load-bearing and is preserved here: nothing below is a measurement about Phalcom. It is
transferable reasoning, and its home is a theory directory precisely because it may not enter
the measured perf ledger.

---

## 1. The load-bearing claim

**`[V]`** From the design note:

> JS's core defect isn't `==` or `this`. It shipped **policy where it needed mechanism**.

The worked example is the one everyone has lived through. `Promise`/`async`/`await` is *policy*:
it commits the language to one concurrency shape and, crucially, **colors every function**. Once
`async` is a property of a function's type, a duplicate higher-order universe becomes necessary —
`map` and async `map`, iterator and async iterator, every combinator twice — because a
synchronous higher-order function cannot accept an asynchronous callback without becoming
asynchronous itself, virally.

The mechanism the language actually needed was **stack switching**. Given stack switching,
`Promise` is an ordinary library; a wrong `Promise` is a fixable library bug; a second, better
concurrency model can coexist with the first; and no function needs a color, because suspension
is a property of the *runtime*, not of the *signature*.

The mirror-image failure is in the same language. Prototype chains are mechanism, but
*unrestricted* mechanism: any object's shape may change at any moment. Because the programmer
had no way to state "this object's shape is fixed" — and would often have been happy to — V8 had
to construct hidden classes, inline caches, and deoptimization: an entire speculative apparatus
whose only job is to *re-derive at runtime what the programmer already knew and could not
express*. The tax is paid forever, by every program, including the ones that never needed the
freedom.

**The general shape.** Ask of any feature: *is this a mechanism the user composes, or a policy
the language commits to on the user's behalf?* Then ask the sharper follow-up: *if this policy
turns out to be wrong, can it be replaced without a new language version?* If the answer is no,
it needed to be a mechanism. And symmetrically: *if this mechanism is unrestricted, who pays to
recover the guarantees the restriction would have given?* Someone always does — usually the
optimizer, and therefore the user, permanently.

---

## 2. Where Phalcom sits, and the standing obligation this creates

**`[V]`** Phalcom took the mechanism road on concurrency. The fiber is the sole primitive;
`Future`, `async`/`await`, generators, and the scheduler all derive from it, and ADR-0030's
Consequences state that `Future` "adds **no** VM mechanism beyond `Fiber` + a ready-queue."

The design note then records a defensive obligation, and it is worth quoting because it is a
commitment about the *future*, not a description of the present:

> The note to self is defensive: do **not** later add `async`/`await` sugar over fibers. That
> would re-introduce coloring on top of a primitive that exists specifically to avoid it.
> Scheduling policy belongs in the library; the fiber stays the mechanism.

This is subtle and easy to get wrong, so it is worth being explicit about what is and is not
forbidden. A library that *uses* fibers to implement future-like combinators is fine — that is
the point of a mechanism. What is forbidden is a *language-level* `async` marker on a function
declaration, because the marker is what colors the type, and coloring is what forces the
duplicate universe. The distinction is not "does the word `await` appear" but "does the
function's *signature* now carry a concurrency property that its callers must propagate."

**`[R]`** The reinforcing observation from the WASM section of the same note: a stackful
*target* does not protect a colored *source*. Rust compiled to WebAssembly is still colored,
because Rust is colored — WASM's stack machine buys nothing back. Therefore "we have fibers" is
not, by itself, a defense against adding coloring later. Only not adding it is.

---

## 3. The boundary tax

**`[V]`** The most portable lesson from the WASM analysis, stated generally:

> **a boundary between two representations is paid per call, and if the workload *is* boundary
> calls, the speed of the fast side stops mattering.**

WASM's worst practical cost in the browser is not codegen quality but crossings: every string
crossing is a copy plus a transcode, since JavaScript strings are UTF-16 and WASM has no string
type at all. The proposed fix (`stringref`) stalled; a narrower patch (JS String Builtins)
shipped instead. No amount of making the WASM side faster addresses a cost that is incurred at
the seam.

**`[O]`** The open question this raises for Phalcom is specific and cheap to answer, and it has
not been answered. The native-versus-`.ph` split *is* a boundary. The design note frames it
exactly right:

> If a Rust primitive and a `.ph` method exchange the same `Value` enum, the crossing is nearly
> free and there is no tax to find. If anything transcodes — strings are the obvious candidate —
> then there's a WASM-shaped cost hiding in the split that no dispatch optimization touches.

Worth checking **before** anyone moves a hot operation across that boundary in either direction,
since that is the decision the answer would change.

**`[V]`** Related, and a genuinely elegant piece of design hygiene: the floor-admission rule of
ADR-0019 says admission requires proof that a capability *cannot be expressed in `.ph` at all* —
and that **speed is explicitly never sufficient**. The named counter-move to hot-path cost is
"fund an inline cache or JIT *above* the floor." It is a one-way door: native→`.ph` is always
allowed and needs no record; `.ph`→native needs a superseding decision. That rule is a policy
about where mechanism may live, and it exists to stop the boundary from creeping outward one
justified exception at a time.

---

## 4. The workarounds do not compose

**`[V]`** The strongest argument in the note for coherence over patching, and it is not the
argument one expects:

> each existing fix is **capped by its weakest neighbor**, and the taxes compound: TypeScript's
> types can't inform V8's inline caches (same information computed twice, one copy discarded);
> Svelte's compile-time knowledge dies at the module boundary; the bundler can't trust
> `sideEffects` because `eval` might exist three dependencies down; React Compiler bails on
> aliasing *because* the language has no value types.

The claim is not that any individual fix is bad. Each is locally excellent. The claim is that
information cannot flow between them, so the same knowledge is derived repeatedly and discarded
repeatedly, and the ceiling is set by whichever layer knows least. A coherent design pays once
and collects several times — static shape feeds codegen feeds dead-code elimination feeds the
binary format feeds startup.

This is the actual argument for spec-first design, and it is stronger than the usual one
("planning is good"). It is an argument about *information flow across layers*, which predicts
where patching will plateau and roughly at what altitude.

---

## 5. The honest counterevidence

**`[V]`** The design note ends its own argument by attacking it, which is why the file is worth
reading rather than summarizing:

> The experiment was run. Google shipped **Dartium** — a JS-replacement VM in Chrome with sound
> types and AOT. It lost, on ecosystem politics, not merit; Dart survives via Flutter, i.e. by
> abandoning the DOM entirely. **Elm** took nearly the same bet — sound, no runtime exceptions,
> own the architecture — and stayed tiny, killed by interop cost.
>
> Meanwhile the things that *did* win the web (CoffeeScript on syntax, TypeScript on types, JSX
> on templating) all won by **compiling to JS**, i.e. by accepting the semantics they were fixing.

And the conclusion, which is the sentence to keep:

> the set of problems only a redesign solves is exactly the set no compile-down tool can reach —
> which is exactly why those problems are still open. **Being right about the design is not
> sufficient and never was.**

The transferable content is a selection effect on the historical record. The problems that
remain unsolved in a mature ecosystem are, almost by definition, the ones incremental tools
cannot reach; observing that they are unsolved is therefore not evidence that a redesign would
succeed, only that the incremental route has been exhausted. Adoption is a separate axis from
correctness, and the correlation between them is weaker than designers reflexively assume.

---

## 6. Two corollaries worth stating separately

### Speculative optimization ⊗ observable semantics

**`[V]`** A fast path must deopt to *exactly* the slow-path result. A path that differs on
overflow, on a side effect, on a redefinition, or on a subclass override is a correctness bug
wearing a performance hat. Every fast path needs a guard that provably implies the slow path.

**`[V]`** Phalcom's committed answer is ADR-0018's sacred-selector inliner with a per-family
override-epoch flag: `ifTrue(_)`, `and(_)`, `whileTrue(_)` and friends inline to `Jump`/`Loop`
opcodes, and if any sacred method is ever redefined the flag flips and the site deopts to a real
send. The overlay records the property that makes it sound: "Guard failure is *observably
identical* to the slow path."

**`[R]`** The instructive contrast is Swift's `@Observable`, which sidesteps the memoization
version of this trap by *never memoizing at all* — computed properties re-run on every read,
transparently tracked but never cached, because no purity check exists that would make caching
sound. A real cost, honestly paid, rather than an unsound optimization.

### Macros rewrite syntax; they do not inform codegen

**`[V]`** From the same note, and this one is live for Phalcom:

> The critical property: **the compiler learns nothing about the dependency graph.** […] The
> macro saved typing. It bought zero codegen knowledge.

The consequence, stated as a standing rule: **no performance claim may be attached to an
expander.** Phalcom's attribute expansion (`expand_class_attributes`, `derive_construct`,
`derive_accessors`, the `@get`/`@set` expanders) produces AST that compiles normally, so
derived accessors are not faster than hand-written ones — they *are* hand-written ones,
generated. If derived accessors should be fast, the win must come from the compiler or VM
recognizing the accessor *shape*, never from the expansion step. Stated in advance so that
nobody benchmarks `@get` and reports a delta that is really noise.
