# Performance & Speed

> Generic design-space layer of the `language-design` skill — matrices + hazards, no textbook prose. Phalcom's committed choice on any axis here: see [../phalcom/overlay.md](../phalcom/overlay.md).
> **Load when:** designing/critiquing anything speed-sensitive — dispatch cost, allocation, GC, interpreter loop, representation, warmup.

## Contents
- Dispatch cost
- Value representation & boxing
- GC strategy
- Interpreter dispatch loop
- Speculative optimization & deopt
- Collection & string representation
- Startup vs peak / warmup
- Allocation discipline & escape analysis

## Dispatch cost
| Option | Langs | Consequence |
|---|---|---|
| Full method-dict walk | naive Smalltalk, early Ruby | Correct, trivial; O(depth × probe) per send — slow |
| Monomorphic inline cache | Self, V8, Wren | Near-zero when receiver type stable at site |
| Polymorphic IC (small n) | Self, V8, HotSpot | Handles few types/site; degrades to megamorphic past cap |
| Megamorphic / vtable | V8, HotSpot, C++ virtual | Shared stub or indirect load; no per-site speculation |
| Static / monomorphized | Rust, C, MLton | Direct call, inlinable; no runtime dispatch, no late binding |

**Syntax.** `obj.foo()` (dynamic send) · Rust `Trait::method` static · `dyn Trait` (vtable) · `@sealed`/`final` hints permitting devirtualization.
**Impl.** per-call-site slot caches {receiver-class, resolved method, class-epoch}; hit checks class+epoch, miss re-resolves. → recipes.md#inline-cache
**Hazard — inline-cache ⊗ polymorphism (CROWN JEWEL).** A site hammered with many receiver types exceeds the poly cap and goes megamorphic, which cannot cache — dispatch falls off a cliff to a shared stub. Hot loops over heterogeneous collections are the classic trigger. → recipes.md#inline-cache → overlay

## Value representation & boxing
| Option | Langs | Consequence |
|---|---|---|
| Boxed everything | CPython (`PyObject*`) | Uniform, simple; every int is a heap object, poor cache density |
| Tagged pointer (fixnum) | V8 SMI, OCaml, Lisp | Cheap int/ptr split; shrinks int range, mask on deref |
| NaN-boxing | LuaJIT, JSC, old SpiderMonkey | All values in 64 bits; doubles native, ptrs in 48-bit NaN payload |
| Unboxed native int | OCaml, Rust, C | Zero-alloc arithmetic; no reflection/identity on the value |
| Autoboxed on demand | Java `Integer`, C# | Unboxed in locals; boxed at generic/collection boundaries |
| Small-int cache | CPython (−5..256), JVM (−128..127) | Interns common ints; == identity holds only inside the window |

**Syntax.** Java `int` vs `Integer` · Rust `i64` vs `Box<dyn Any>` · unboxed-array types `int[]` vs `Integer[]` · `@packed`/`@unboxed`.
**Impl.** tagged ptr steals low bits (mask on deref); NaN-boxing packs value in a double's NaN space. → recipes.md#nan-boxing · recipes.md#option-niche
**Hazard — boxing ⊗ hot numeric loops (CROWN JEWEL).** Autoboxing inside a tight loop (`Integer` accumulator, boxed generic collection) allocates per iteration and dominates runtime — the arithmetic is free, the box churn and GC are not. Unboxed locals + specialized collections are the fix. → overlay

## GC strategy
| Option | Langs | Consequence |
|---|---|---|
| Reference counting | CPython, Swift, PHP | Prompt free, low pause; leaks cycles, refcount traffic + atomics |
| Tracing mark-sweep (non-moving) | Boehm, Ruby, old CPython | Reclaims cycles; stop-the-world pauses, fragmentation |
| Generational | V8, HotSpot, LuaJIT | Cheap young collection; write barrier on every ref store |
| Moving / compacting | JVM, V8, .NET | Defrags, bump allocation; every retained ptr must be a tracked root |
| Arena / region | MLKit, some Rust | Bulk-free, no per-object cost; lifetimes must nest |
| No GC (ownership) | Rust, C++ RAII | Deterministic, no runtime; borrow-checker/manual burden on author |

**Impl.** refcount adds inc/dec on every assignment (+ atomics if threaded); moving GC needs a precise root set and per-slot pointer updates on relocation; generational adds a store write barrier.
**Hazard — refcount ⊗ cycles.** Pure reference counting never reclaims a reference cycle (parent↔child, closures capturing their own frame) — you leak until process exit unless you bolt on a tracing cycle collector (CPython) or demand `weak` refs at cycle points (Swift). → overlay

## Interpreter dispatch loop
| Option | Langs | Consequence |
|---|---|---|
| Switch-based loop | CPython (pre-3.11), portable VMs | Portable C; one big branch, mispredicts often |
| Direct/indirect threaded | GForth, CPython 3.11+ | op→op computed-goto, better prediction; needs GNU `&&label` |
| Template / baseline JIT | V8 Sparkplug, JSC baseline | Emits straight-line machine code per op; fast warmup, modest peak |
| Tracing JIT | LuaJIT, PyPy | Compiles hot linear traces + guards; superb loops, trace explosion risk |
| Method / optimizing JIT | HotSpot C1/C2, V8 TurboFan | Whole-method opt + inlining; long warmup, deopt machinery |
| AOT | Rust, Go, GraalVM native | No warmup, predictable; no runtime type feedback, big binaries |

**Impl.** switch loop = `switch(op)`; threaded = table of `&&label` gotos jumping at each op's tail; template JIT stitches per-op code stencils; tracing records a hot path then compiles it behind guards.
**Hazard — tracing JIT ⊗ irregular control flow.** Tracing bets the hot path is linear; deeply polymorphic branches or megamorphic sends spawn many side-traces / trace aborts, so a branchy interpreter can run *slower* warmed than cold. Loop shape, not loop count, decides the payoff. → overlay

## Speculative optimization & deopt
| Option | Langs | Consequence |
|---|---|---|
| Inline sacred selectors unconditionally | many Smalltalks (`ifTrue:`) | Fast branches; WRONG if the selector was overridden |
| Type-feedback inline + guard + deopt | V8, HotSpot, Self | Speculate observed type, bail to generic on guard miss |
| Assumption/dependency registry | HotSpot `Dependencies`, V8 | Overriding a speculated method deopts dependent code globally |
| Never speculate | CPython (pre-3.13), Wren | Predictable, slower; no deopt bookkeeping |

**Syntax.** JVM `-XX:+PrintInlining` / `final` · V8 `%OptimizeFunctionOnNextCall` · `@inline`/`@specialize` hints.
**Impl.** emit inlined fast body behind a class/assumption guard; register the site in a dependency table so a later override bumps the epoch and deopts back to a generic send. → recipes.md#sacred-inline
**Hazard — speculative inlining ⊗ correctness (CROWN JEWEL).** Deopt must reconstruct exact interpreter state — locals, stack, pending sends — at the guard's bailout point. If the on-stack-replacement mapping drops a value or mis-times a side effect, deopt silently changes program semantics: an optimization bug, not a slowdown. → recipes.md#sacred-inline → overlay

## Collection & string representation
| Option | Langs | Consequence |
|---|---|---|
| Contiguous array / dynarray | Lua tables (array part), JS packed | O(1) index, cache-friendly; resize copies, holes deopt to dict |
| Hidden-class / shape objects | V8, JSC | Field access as fixed offset + shape guard; shape churn kills it |
| Hashmap-backed object | Python `__dict__`, Ruby | Flexible dynamic fields; per-access hash, poor density |
| String interning | Java, Lua, Smalltalk symbols | O(1) identity compare; intern-table pressure, dedup cost |
| Small-string / rope | Rust `SmallString`, ropey, JS cons-strings | Elides alloc / cheap concat; branchy access, rope rebalancing |
| Immutable persistent | Clojure, Scala | Structural sharing, safe to share across threads; pointer-chasing, GC load |

**Syntax.** unboxed `int[]` vs boxed `Object[]` · Rust `&str`/`String`/`Rc<str>` · Clojure `[]`/`{}` literals are persistent.
**Impl.** shape objects map field→offset via a transition tree; adding fields in inconsistent order forks shapes and demotes sites to megamorphic/dictionary mode.
**Hazard — hidden-class ⊗ inconsistent field order.** Two "same" objects built by adding fields in different orders get *different* hidden classes, so a call/property site sees polymorphic shapes and can't cache the offset. Deleting a field or adding after construction demotes the object to slow dictionary mode. → overlay

## Startup vs peak / warmup
| Option | Langs | Consequence |
|---|---|---|
| Pure interpreter | CPython, MRI | Instant start, flat (low) peak; no warmup, no JIT wins |
| Interpret-then-JIT | HotSpot, V8, LuaJIT | High peak eventually; cold code slow, tiering + profiling cost |
| Heap snapshot / image | V8 snapshots, Smalltalk images, CRaC | Skips init/bootstrap replay; snapshot must match VM+platform |
| AOT compile | Go, Rust, GraalVM native-image | Zero warmup, small footprint; no runtime feedback, closed-world |

**Impl.** V8 serializes a warmed heap (builtins, parsed core) into a snapshot blob mmap'd at boot; tiering JITs start interpreted and promote per method once profile counters trip.
**Hazard — JIT warmup ⊗ short-lived processes.** A serverless/CLI process that exits before methods tier up pays all the profiling and interpretation cost and reaps none of the peak — AOT or a snapshot beats a tiering JIT for start-and-die workloads. Benchmark the whole process lifetime, not steady state. → overlay

## Allocation discipline & escape analysis
| Option | Langs | Consequence |
|---|---|---|
| Heap-allocate everything | naive VMs, CPython | Simple, uniform; alloc + GC pressure on every temporary |
| Stack allocation for non-escaping | HotSpot EA, Go escape analysis | No GC cost when proven local; conservative analysis over-heaps |
| Scalar replacement | HotSpot C2, TurboFan | Explodes a non-escaping object into registers; guard-fragile |
| Open-upvalue elision | Lua, Wren | Captured locals stay in stack slots until they actually escape |
| Move/flat capture | Rust `move`, OCaml | No shared cell, no close step; can't observe later mutation |

**Syntax.** Go `//go:noescape` · `new` vs stack literal · Rust `move ||` closures · `@stackalloc`.
**Impl.** VM holds a sorted open-upvalue list into live stack slots; on scope exit `close` copies each into its own heap cell and redirects the pointer. → recipes.md#lua-upvalues
**Hazard — escape analysis ⊗ inlining order.** Scalar replacement only fires once inlining proves the callee doesn't leak the object, so a call the compiler declined to inline (too big, megamorphic, behind a guard) forces the allocation back onto the heap. EA wins are contingent on inlining wins — one deopt can un-elide the alloc. → recipes.md#lua-upvalues → overlay
