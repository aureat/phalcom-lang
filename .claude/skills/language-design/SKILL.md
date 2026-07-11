---
name: language-design
description: >-
  Programming-language design mastery — the design SPACE (axes, precedents in
  syntax + implementation, and how features fight), not concept tutorials. Use
  when designing, critiquing, or implementing a language or its compiler: object
  model / metaclasses / prototypes, dispatch / selectors / multimethods /
  keyword/default/variadic args, closures / non-local return / control-flow / tail
  calls, value repr (tagged/NaN-box/niche) / absence (Option/nil) / truthiness /
  numeric tower, typing (static/dynamic/gradual, inference, variance), error
  handling (exceptions/Result/conditions/panics), concurrency
  (fibers/coroutines/async/actors), compiler / IR / bootstrapping / self-hosting,
  core & standard library, VM/bytecode (inline caches, speculative inlining,
  deopt), performance & GC, security & robustness, or design best-practices. Use
  before adding a feature to check what it PRECLUDES. Built for Phalcom
  (Smalltalk-style bytecode VM in Rust): generic layer in references/, committed
  positions in phalcom/overlay.md.
---

# Language Design

A design-space reference for building and evolving programming languages. It assumes the reader already knows *what* a closure/metaclass/inline-cache is. It supplies what a smart model re-derives (and sometimes gets subtly wrong) under time pressure: **the axes of choice, how ~15 real languages resolved each and at what cost, and — the crown jewel — which features are in tension.** Most design mistakes are not "this feature is bad" but "this feature, added to a language that already committed to *that* feature, is unsound or combinatorial." This skill is organized to catch exactly that.

**Two layers.** `references/*.md` = the generic space (any language). `phalcom/overlay.md` = what *Phalcom* has already committed to, keyed to the same axes, with ADR/spec citations. The generic layer lists options; the overlay says which one was taken and why it's locked. Never restate the overlay's content as if undecided, and never invent Phalcom's position — read the overlay.

## Operating procedure

Run this loop on any "design / add / critique / plan feature X" task. It is the actual skill — the facts are just fuel for it.

1. **Name the axis.** Locate X in the reference matrices (nav below). State the axis precisely ("upvalue representation", not "closures"). If X spans several axes, list them.
2. **Scan for interaction hazards.** Before evaluating X on its own merits, check the **Hazard catalog** below and the per-axis hazards in the reference file against features the language has *already committed to*. This is where most real bugs live. A feature that is fine in isolation can be unsound given a prior commitment (see the canonical case: default args ⊗ selector-identity dispatch).
3. **Cite precedent with consequence.** Name 2–3 languages that took each option and the *cost* they pay — not "Ruby does X" but "Ruby does X, which forces Y." Precedent without consequence is trivia; the consequence is the argument.
4. **Reconcile with the committed position.** For Phalcom work, read `phalcom/overlay.md` (and the cited ADR/spec §) for what is already locked, open, or explicitly deferred. Do not reopen a settled question or design atop an open one without flagging it.
5. **Recommend, and state what it precludes.** Give one recommendation. Then name what choosing it forecloses (future features, optimizations, invariants). This "what must this not preclude" check is mandatory — it is how a local decision avoids a global regret.

## Interaction-hazard catalog (crown jewel)

Generalized cross-feature traps. Each is a *pattern*; the reference files carry the domain-specific instances, and `phalcom/overlay.md` records the ones Phalcom has already hit. When step 2 fires, match against these shapes first.

- **Identity-dispatch ⊗ optional arity.** If method identity is `name+arity+kind` (so `foo` and `foo(_)` coexist), then *anything that varies effective arity* — default arguments, optional trailing args, implicit-self elision — produces a *different selector* and misses the defined method. Resolutions are all bad: arity-family expansion (combinatorial) or static-callee knowledge (unavailable under dynamic dispatch). Decide *before* selector identity becomes load-bearing. (`dispatch.md`)
- **Bootstrap cycle in self-hosted absence/objects.** If "absence" (`None`/`nil`) is an ordinary object, and object fields *default to absence*, then constructing the absence value needs a class whose fields already default to absence → cycle. Forces VM-blessing / niche-encoding the primitive rather than defining it in the library. Same shape hits any "everything is an object" primitive (true/false, the metaclass root). (`values.md`, `object-model.md`)
- **Escape ⊗ non-local return.** The moment a closure/block can outlive its defining frame (stored, returned, sent to another fiber), a non-local `^`/return through the dead home frame must *trap*, not corrupt the stack. Design the dead-frame error *when blocks can escape*, not when you add `^`. (`closures-control.md`, recipe `non-local-return`)
- **Speculative inlining ⊗ late binding.** Inlining "sacred" selectors (`ifTrue:`, `whileTrue:`, `+`) assumes they aren't overridden. In a language where *any* method can be redefined at runtime, that assumption needs a guard + deopt path (override epoch / assumption invalidation), or the optimization is unsound. (`vm.md`, recipe `sacred-inline`)
- **Inline cache ⊗ mutable hierarchy.** Monomorphic/polymorphic inline caches assume class shape and method bindings are stable. If the language permits reshaping the hierarchy or redefining methods at runtime, every such mutation must invalidate caches (version/epoch bump). Mutable hierarchy also blocks fixed slot-layout assumptions. (`vm.md`, `object-model.md`)
- **Enforcement without static analysis.** "Ban truthiness" / "reject `if(option)`" / "no implicit conversion" are static-sounding rules in a language with no type/flow analysis. They can only be enforced by (a) a runtime protocol floor (the branch opcode *requires* Bool; Option never implements the branch protocol) plus (b) compile-time rejection of the *syntactically obvious* cases. Know which half you're getting. (`values.md`)
- **Function coloring ⊗ higher-order APIs.** `async`/`await` splits the function universe in two; every higher-order API (map, callbacks, iterators) must be duplicated or bridged across the color boundary. Transparent green threads/fibers (Go, BEAM, Loom) avoid the split at the cost of a runtime scheduler. Choosing async/await colors the *entire* standard library. (`concurrency.md`)
- **Stackful fibers ⊗ moving GC / native pointers.** Stackful coroutines keep live pointers on a separate native stack the GC must find; a moving/compacting collector or a handle/arena heap constrains how fiber stacks and captured upvalues may hold references. (`concurrency.md`, `vm.md`)
- **Cleanup ordering ⊗ unwinding.** `ensure`/`finally`/`defer`/`Drop` must run in a defined order relative to a non-local return, exception, or cross-fiber unwind — and resumable (condition/restart) vs terminating unwind decides the *entire* stack discipline. Pick resumable-vs-terminating first; it is not retrofittable. (`errors.md`)
- **Keyword-message selectors ⊗ evaluation order & currying.** Keyword-part selectors (`at:put:`) bind arity into the name and fix argument grouping; they interact with default args, partial application, and external-vs-internal labels. Reserve the signature field for labels *before* identity is load-bearing even if you don't use it yet. (`dispatch.md`)
- **Speculative optimization ⊗ observable semantics.** Any type-feedback inlining, fast path, or unboxing must deopt to *exactly* the slow-path result — an optimized path that differs on overflow, a side effect, a redefinition, or a subclass override is a correctness bug wearing a performance hat. Every fast path needs a guard that provably implies the slow path. (`performance.md`, `vm.md`)
- **Dynamic power ⊗ untrusted input.** `eval`, deserialization, reflection/`doesNotUnderstand`, and *unverified* loaded bytecode each turn metaprogramming reach into an injection or type-confusion primitive the instant input is attacker-controlled. A VM must validate external bytecode, cap recursion/allocation, randomize hashing, and convert every malformed input into a *defined* error — never UB, never a raw `panic!`. (`security.md`)
- **Primitive/library boundary ⊗ bootstrap order.** A class the runtime secretly depends on — the kernel `List` behind `Message.args`/rest-params, `Bool` behind control flow, `Option` behind absence — must be built *before* the features that use it, or the image fails to load. Decide native-vs-library and the kernel dependency DAG up front; a "just a library class" that `doesNotUnderstand`/variadics need is on the critical path, not deferrable. (`bootstrapping.md`, `stdlib.md`)
- **Gradual typing ⊗ soundness & performance.** Retrofitting optional/gradual types onto a dynamic language is either *unsound* (TypeScript erases and lies at the typed↔untyped boundary) or *sound but slow* (contracts/casts at every crossing — the "gradual guarantee" tax). Pick which; never assume a bolted-on annotation actually constrains a runtime that still dispatches on tags. (`typing.md`)

## Design-review rubric

When *critiquing* a proposed feature or diff (feeds the forge auditor/reviewer lenses), score it on:

1. **Soundness** — is there an input/state where it produces a wrong result or corrupts an invariant? (Name the state.)
2. **Dispatch impact** — does it change selector identity, method lookup, or inline-cache validity?
3. **Representation impact** — does it force a boxing, a niche, a new tag, or an allocation on a hot path?
4. **Preclusion** — what future feature/optimization does it foreclose? (The mandatory step-5 check.)
5. **Precedent** — has a real language done this? What did it cost them? If *no* language has, why not — is it novel or is it a known trap?
6. **Spec reconciliation** — does it contradict a committed ADR/spec §, silently resolve an open question, or depend on one still open? (Phalcom: cite the overlay.)

A "no problems found" verdict must still answer 1, 2, and 4 explicitly — silence on them is not a pass.

## Navigation

Load the one reference the axis lives in; don't pull them all.

| File | Axes it covers |
|---|---|
| [references/object-model.md](references/object-model.md) | class vs prototype, metaclass strategies, Behavior/method-dict, inheritance & traits, hierarchy mutability, instance layout, identity/equality |
| [references/dispatch.md](references/dispatch.md) | message-send vs vtable, selector/signature identity, single vs multiple dispatch, keyword/default/variadic args & labels, MRO/super, doesNotUnderstand/proxies |
| [references/closures-control.md](references/closures-control.md) | closure/upvalue representation, capture semantics, non-local return, control-flow-as-message, TCO, iteration/generators, continuations |
| [references/values.md](references/values.md) | value representation (tagged/NaN-box/niche), absence, truthiness, numeric model, immutability, symbols, equality ladder |
| [references/typing.md](references/typing.md) | static/dynamic/gradual, strong/weak, nominal/structural, inference (HM/bidirectional), generics & erasure, variance, soundness/escape hatches, RTTI/reflection |
| [references/errors.md](references/errors.md) | exceptions vs Result vs conditions/restarts vs panics, resumable vs terminating, bridging, cleanup/RAII, error data, failure in constructors |
| [references/concurrency.md](references/concurrency.md) | threads/green/fibers/coroutines, sym vs asym, async/await & coloring, scheduling, actors/isolation/STM, structured concurrency, memory model |
| [references/vm.md](references/vm.md) | stack vs register bytecode, frames, inline caches, speculative inlining + deopt, upvalue closing, value repr for the VM, GC interaction |
| [references/compiler.md](references/compiler.md) | pipeline shape (single/multi/query), CST/AST, IR choice, name resolution, desugaring/lowering, pass ordering, error recovery, compile-time/macros |
| [references/bootstrapping.md](references/bootstrapping.md) | bootstrap strategy, self-hosting stages, kernel-in-the-language, metaclass/absence bootstrap cycle, image vs source, trusting-trust, kernel load order |
| [references/stdlib.md](references/stdlib.md) | primitive-vs-library boundary, core/std split, batteries vs minimal, prelude/auto-import, collection library, modules/namespaces, API stability |
| [references/performance.md](references/performance.md) | dispatch cost, boxing/representation, GC strategy, interpreter loop & JIT, speculative opt/deopt, collections/strings, warmup, allocation/escape analysis |
| [references/security.md](references/security.md) | memory-safety model, panic-vs-UB, sandboxing/capabilities, eval/deserialization, integer safety, resource-exhaustion/DoS, FFI/unsafe boundary, bytecode verification |
| [references/best-practices.md](references/best-practices.md) | least surprise, orthogonality, small core, make-illegal-states-unrepresentable, errors-as-UX, evolution/compat, explicit-over-implicit, spec-first, Rust-VM impl hygiene |
| [references/recipes.md](references/recipes.md) | how-to-build-it algorithms: lua-upvalues, nan-boxing, inline-cache, sacred-inline, non-local-return, option-niche, coroutine-switch |
| [references/reading.md](references/reading.md) | canonical primary sources per domain (Blue Book, Self, AMOP, CLtL conditions, Appel, TAPL, Lua/Wren, Crafting Interpreters …) |
| [phalcom/overlay.md](phalcom/overlay.md) | **Phalcom's committed positions + open questions + hazards-already-hit**, keyed to the axes above, with ADR/spec citations |
