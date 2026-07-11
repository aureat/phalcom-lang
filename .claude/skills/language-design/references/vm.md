# Bytecode VM & Execution

> Generic design-space layer of the `language-design` skill — matrices + hazards, no textbook prose. Phalcom's committed choice on any axis here: see [../phalcom/overlay.md](../phalcom/overlay.md).
> **Load when:** designing bytecode format, call frames, method-lookup caching, inlining/deopt, upvalue closing, value representation, GC interaction, or dispatch fast-paths.

## Contents
- Stack- vs register-machine bytecode
- Call frame & operand-stack layout
- Method-lookup caching
- Speculative inlining & deopt guards
- Upvalue closing mechanics
- Value representation
- GC interaction
- Dispatch fast-paths & threading

## Stack- vs register-machine bytecode
| Option | Langs | Consequence |
|---|---|---|
| Stack machine | CPython, JVM, Smalltalk, Wren | Compact, trivial codegen; more dispatches, lots of push/pop traffic |
| Register (virtual) machine | Lua, LuaJIT, Dalvik, Erlang BEAM | Fewer/larger instrs, fewer dispatches; harder register alloc in compiler |
| Hybrid / accumulator | V8 Ignition (accumulator) | One implicit reg cuts operand encoding; simplifies the interpreter loop |
| Closure/graph reduction | Haskell STG, combinator machines | Suits laziness/purity; alien to strict imperative dispatch |

**Impl.** stack: CPython `LOAD_FAST a; LOAD_FAST b; BINARY_OP +` · register: Lua `ADD R2 R0 R1` (dst,src,src) · accumulator: V8 `Ldar a; Add b`.

**Hazard — instruction count ⊗ dispatch cost.** Register VMs win by amortizing per-dispatch overhead over bigger ops, so the stack-vs-register choice is really a bet on whether dispatch (branch mispredict) or code size dominates — measure before committing; a naive stack VM's push/pop can double dispatch count. → overlay

## Call frame & operand-stack layout
| Option | Langs | Consequence |
|---|---|---|
| Frames on native C stack | CPython (pre-3.11 ceval), Lua | Fast; recursion depth bounded by OS stack; no cheap coroutines |
| Frames on heap / VM stack | JVM, CPython 3.11+, BEAM | Enables coroutines/continuations/green threads; alloc + GC pressure |
| Contiguous operand+locals region | Lua registers, LuaJIT | Locals and temps share one window; cache-friendly slicing per call |
| Spaghetti/linked frames | Smalltalk `MethodContext` | Frames are first-class reifiable objects; slow unless optimized away |

**Impl.** frame = fixed locals slots + operand region + saved return-pc + caller link; heap-allocated frames chain to suspend/resume for coroutines; Smalltalk contexts are ordinary heap objects. → recipes.md#coroutine-switch

**Hazard — native-stack frames ⊗ suspendable control.** Generators, `call/cc`, async, and stackful coroutines all need frames that outlive the C call that created them; a native-stack design forecloses them without a CPS/state-machine rewrite. Ties directly to closures-control.md continuations. → overlay

## Method-lookup caching
| Option | Langs | Consequence |
|---|---|---|
| No cache (walk hierarchy) | naive Smalltalk, early Ruby | Correct, dead simple, dispatch is O(depth × dict-probe) — slow |
| Global method cache (hash of class×sel) | Ruby (pre-ICs), older Smalltalk | One shared table; any redefinition flushes globally |
| Monomorphic inline cache | Self, V8, Wren | Caches last {class→method} at call site; near-zero when stable |
| Polymorphic IC (small n) | Self, V8, HotSpot | Handles few receiver types per site; falls to megamorphic beyond cap |
| Megamorphic / vtable fallback | V8, HotSpot | Too-polymorphic sites use a shared stub; give up on speculation |

**Impl.** per-call-site slot caches {receiver-class, resolved method, class-epoch}; hit checks class+epoch, miss re-resolves and refills; megamorphic sites fall back to a shared dispatch stub. → recipes.md#inline-cache

**Hazard — IC ⊗ hierarchy mutability (CROWN JEWEL).** Every cached call site embeds an assumption about a class's method table. Adding/removing/redefining a method, reparenting, or reshaping layout must invalidate matching ICs — via a per-class/global epoch counter checked on hit, or eager cache sweep. Miss this and dispatch calls stale code. → recipes.md#inline-cache → overlay

## Speculative inlining & deopt guards
| Option | Langs | Consequence |
|---|---|---|
| Inline sacred selectors unconditionally | many Smalltalks (`ifTrue:` etc.) | Fast branches; WRONG if user overrode the selector |
| Inline + guard + deopt | V8, HotSpot, Self | Speculate common case, bail to generic on guard failure |
| Override-epoch / assumption registry | HotSpot `Dependencies`, V8 | Overriding a speculated method invalidates compiled code globally |
| Never speculate | CPython (pre-3.13), Wren | Predictable, slower; no deopt machinery to maintain |

**Impl.** emit the inlined fast body behind a class/assumption guard, and register the site in a dependency table so a later override bumps the epoch and deopts the site back to a generic send. → recipes.md#sacred-inline

**Hazard — sacred-selector inlining ⊗ method override / deopt (CROWN JEWEL).** Inlining `ifTrue:`/`+`/`whileTrue:` assumes `Boolean`/`Integer` didn't override them. If the object model lets a user (or subclass) redefine those selectors, inlined call sites now skip the override — you MUST either forbid overriding sacred selectors or install deopt guards that invalidate on redefinition. → recipes.md#sacred-inline → overlay

## Upvalue closing mechanics
| Option | Langs | Consequence |
|---|---|---|
| Open-upvalue list keyed by stack slot | Lua, Wren | Shared vars point into stack; `close` on scope exit copies to heap cell |
| Immediate heap boxing on capture | JS `let`, Smalltalk temps | No open/close bookkeeping; every captured var is a heap alloc |
| Flat copy at closure creation | Rust `move`, OCaml | No sharing, no close step; can't reflect later mutation |
| Escape analysis → stack-keep | HotSpot, some V8 | Non-escaping captures never boxed; falls back to heap when proven to escape |

**Impl.** VM holds a sorted open-upvalue list pointing into live stack slots; on scope exit (or unwind) `close` copies each slot into its own heap cell and redirects the upvalue pointer at it. → recipes.md#lua-upvalues

**Hazard — upvalue close ⊗ non-local exit.** An upvalue must be closed (promoted stack→heap) the instant its slot leaves scope — but `break`, exceptions, and non-local `^` unwind past the normal exit point. The unwinder itself must run the close, or a still-open upvalue dangles into a reused stack slot. → recipes.md#lua-upvalues → overlay

## Value representation
| Option | Langs | Consequence |
|---|---|---|
| Tagged pointer (low bits) | OCaml (1-bit int), V8 SMI, Lisp | Cheap int/ptr discrimination; shrinks int range, mask on every deref |
| NaN-boxing | LuaJIT, older SpiderMonkey, JSC | All values in 64 bits; doubles native, ptrs in NaN payload; 48-bit ptr limit |
| Tagged union / struct | CPython (`PyObject*`), Wren `Value` | Simple, portable; boxing overhead, poor cache density |
| Niche / unboxed enum | Rust `Option<&T>`, Kotlin | Zero-cost when a spare bit-pattern exists; language-assisted only |

**Impl.** tagged ptr steals low/high bits for a type tag (mask on deref); NaN-boxing packs every value in a 64-bit double's NaN space; niche reuses an invalid bit-pattern as the discriminant. → recipes.md#nan-boxing · recipes.md#option-niche

**Hazard — NaN-boxing ⊗ moving GC & 64-bit ptrs.** NaN-boxing assumes pointers fit in 48 bits and that raw bit patterns are stable — hostile to pointer-compression, tagged moving collectors, and platforms with high canonical addresses. Deep recipe & bit layouts: → recipes.md#nan-boxing → overlay

## GC interaction
| Option | Langs | Consequence |
|---|---|---|
| Non-moving mark-sweep + raw ptrs | CPython (refcount+cycle), Boehm | Interior/raw pointers stay valid; fragmentation, no compaction |
| Moving/compacting + handles | JVM, V8, .NET | Compacts heap; every retained ptr must be a GC-visible root/handle |
| Generational + write barriers | V8, HotSpot, LuaJIT | Fast young collection; mutation needs barriers on every store |
| Arena / region | some Rust, MLKit | Bulk-free, no per-object GC; lifetimes must nest |

**Impl.** moving GC needs a precise root set + per-slot pointer updates (or a handle indirection) on relocation; non-moving pins raw pointers; generational adds a write barrier on every reference store.

**Hazard — moving GC ⊗ inline caches & open upvalues.** A compacting collector relocates objects, so ICs caching raw class/method addresses and upvalues holding raw stack/heap pointers become dangling unless every such site is a tracked root updated on move. Non-moving GC dodges this at the cost of compaction. → overlay

## Dispatch fast-paths & threading
| Option | Langs | Consequence |
|---|---|---|
| Switch-based interpreter loop | CPython (pre-3.11), portable VMs | Portable; big branch that mispredicts often |
| Direct/indirect threaded code | GForth, CPython 3.11+ computed-goto | Threads op→op, better branch prediction; needs `&&label` GNU ext |
| Superinstructions | Forth, Factor, CPython specializer | Fuse hot op pairs into one dispatch; combinatorial op-table growth |
| Inline-cache-specialized bytecodes | CPython 3.11+ (`LOAD_ATTR_INSTANCE`) | Rewrite generic op to a fast variant after warmup; quicken/deopt path |

**Impl.** switch loop = one `switch(op)`; threaded code = table of `&&label` computed-gotos jumping at each op's tail; superinstructions fuse hot pairs; quickening rewrites an op to a specialized variant in place.

**Hazard — threaded code ⊗ portability & superinstruction blowup.** Computed-goto threading depends on a GNU extension (no standard C fallback but the switch loop), and fusing superinstructions grows the dispatch table multiplicatively — each fast-path variant is more code to maintain and blow the I-cache. → overlay
