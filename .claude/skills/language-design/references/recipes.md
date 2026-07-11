# Implementation Recipes

> How-to-actually-build-it layer of the `language-design` skill. Terse algorithms, not concepts. Load when implementing one of these mechanisms.

## Contents
- [lua-upvalues](#lua-upvalues) — open/closed upvalue closing
- [nan-boxing](#nan-boxing) — NaN-boxing & pointer tagging
- [inline-cache](#inline-cache) — mono→poly→megamorphic call-site caches
- [sacred-inline](#sacred-inline) — speculative control-selector inlining + deopt
- [non-local-return](#non-local-return) — frame-token home-context return
- [option-niche](#option-niche) — niche-encoded Option/None, no alloc
- [coroutine-switch](#coroutine-switch) — stackful fiber context switch

## lua-upvalues
Lua-style open→closed upvalue closing. *Phalcom uses this → overlay.*
**Seen in:** Lua, Wren, Crafting Interpreters clox
1. Each running frame owns an intrusive list of **open upvalues**, sorted by the stack slot they point at.
2. Capture: to close over slot `s`, walk the open list; if an upvalue already points at `s`, reuse it (sharing), else insert a new open upvalue holding `&stack[s]`.
3. An open upvalue is a cell whose `location` points **into the live stack**; reads/writes go through it, so the variable is still shared.
4. `close_upvalues(level)` on scope/frame exit: for every open upvalue with slot ≥ `level`, copy `*location` into the cell's own inline storage, repoint `location` at that inline slot, unlink from the list.
5. Closures hold ref-counted/GC'd pointers to the cells; after close they keep working off heap storage. Multiple closures sharing a var share one cell → mutation is visible to all.

## nan-boxing
Pack every value into one 64-bit word.
**Seen in:** LuaJIT, JavaScriptCore, SpiderMonkey, Crafting Interpreters clox
1. Doubles are stored raw. All non-double values hide in the payload of a **quiet NaN** (exponent all-1s, top mantissa bit set) — 51 free payload bits.
2. Tag scheme: use a few high mantissa bits (or a sign-bit + low bits) to encode a type tag; pointers fit because real heap addrs use ≤48 bits — canonicalize/mask the top 16.
3. Singletons (`nil`/`true`/`false`) = distinct constant NaN payloads. Small ints = fixnum tag + payload.
4. Decode: `if !is_nan(bits) → double; else match tag bits`. Pointer decode masks tag, sign-extends to 48-bit canonical form.
5. Tradeoffs: NaN-boxing = 8 bytes/value, fast double math, but pointer masking cost + platform pointer-width assumptions. **Tagged enum** (Rust) = larger, no masking, exhaustive matching. **Low-bit fixnum tagging** = pointers 8-aligned so low 3 bits tag; ints shifted — cheap but no unboxed doubles.

## inline-cache
Speed message dispatch by caching lookup at the call site.
**Seen in:** Smalltalk-80 (Deutsch-Schiffman), Self, V8, JavaScriptCore
1. Each call site starts **uninitialized**. First send: do full method lookup, then store `(receiver_class_id → method*)` inline at the site — now **monomorphic**.
2. Subsequent sends: guard `receiver.class_id == cached_id`; hit → jump straight to `method*`, skipping the dictionary.
3. Miss: re-lookup. Promote to **polymorphic** — a small (≤N, e.g. 4) array of `(class_id → method*)` pairs checked linearly.
4. Overflow N distinct classes → **megamorphic**: fall back to the global method cache / hashtable; stop caching per-site.
5. Invalidation: any class reshape or method (re)definition bumps a **version/epoch** (global or per-class). Guards also check the epoch, or a reshape actively clears sites keyed on that class. Without this, monkey-patch dispatches stale.

## sacred-inline
Speculatively inline control-flow selectors (`ifTrue:`, `whileTrue:`, `and:`, `to:do:`). *Phalcom uses this → overlay.*
**Seen in:** Smalltalk-80, Pharo, Self, Squeak
1. Compiler recognizes the sacred selector syntactically **and** the arg is a literal block; emit native branch/loop bytecode inlining the block body — no send, no block allocation.
2. This assumes nobody overrode the selector on the receiver's class. Record that assumption: a global **override-epoch** or per-selector "not-overridden" assumption flag.
3. Emit a cheap guard (or rely on the epoch) so the inlined fast path is only valid while the assumption holds.
4. On any redefinition of a sacred selector: bump the epoch → outstanding guards fail.
5. **Deopt**: guard failure discards the inlined path and re-executes as a real message send (materialize the block, dispatch normally). Correct-but-slow fallback preserves overridability.

## non-local-return
`^expr` returning from the home method even when run inside an escaped block. *Phalcom uses this → overlay.*
**Seen in:** Smalltalk-80, Pharo, Ruby (block `return`), Newspeak
1. Every method activation gets a unique **frame token** (monotonic id or the frame pointer itself), stored in the frame.
2. When a block literal is created, it captures its **home token** = the token of the enclosing method activation (alongside its upvalues).
3. Executing `^expr` inside a block: unwind frames from current top, running `ensure`/`finally` handlers, until the frame whose token == the block's home token; deliver `expr` as that method's return value; pop it too.
4. If no live frame matches (home already returned): raise a **dead-home / BlockContext** error — do not corrupt the stack.
5. Method-local `^` (not in a block) is just the trivial case where home == current frame.

## option-niche
Represent `Option`/`None` with no allocation and no extra tag word.
**Seen in:** Rust (null-pointer opt), Swift, OCaml (unboxed), Haskell strictness
1. Pick a **niche**: a bit pattern the payload type can never legitimately hold (null pointer, a reserved NaN payload, an out-of-range discriminant).
2. `Some(x)` = the raw value `x` when `x` cannot collide with the niche; `None` = the niche pattern. No box, no wrapper word (Rust null-pointer optimization).
3. If `x`'s domain is full (could equal the niche), fall back to boxing `Some` or spending a discriminant word.
4. **Bootstrap cycle:** a stdlib `Option` defined in the language needs the object model, but the VM's own value representation (e.g. `nil` as a niche) must exist *before* the stdlib loads. So `None`/`nil` gets **VM-blessed** as a primitive niche value rather than an ordinary library class. → overlay

## coroutine-switch
Context switch for stackful coroutines/fibers.
**Seen in:** Lua, Ruby Fiber, Kotlin, Go
1. Each fiber owns a **separate stack** (heap-allocated region or mmap'd, with a guard page).
2. `resume(f)`: save current **callee-saved registers + SP + IP** into the current context; load `f`'s saved SP/IP/registers; jump. `yield`: symmetric save/restore back to the resumer's context.
3. Transfer is a small asm/`ucontext` trampoline; the first `resume` of a fresh fiber sets SP to the top of its stack and IP to the entry fn.
4. Suspended state = the saved SP + the untouched stack region; a fiber is "paused mid-call" for free.
5. **GC interaction:** every fiber stack is a root set the collector must scan; a **moving** GC must also rewrite pointers found inside each suspended stack, and know the guard-page bounds. Cross-ref concurrency.md execution-unit hazard. → overlay
