# Closures & Control Flow

> Generic design-space layer of the `language-design` skill — matrices + hazards, no textbook prose. Phalcom's committed choice on any axis here: see [../phalcom/overlay.md](../phalcom/overlay.md).
> **Load when:** designing closures, blocks, non-local return, iteration, tail calls, or control-flow-as-message semantics.

## Contents
- Closure environment representation
- Capture semantics & loop-variable trap
- Non-local return
- Control flow: message/library vs keyword vs macro
- Tail-call optimization
- Iteration protocol
- First-class continuations

## Closure environment representation
| Option | Langs | Consequence |
|---|---|---|
| Flat copied captures | Rust (`move`), Lua closed upvalues, OCaml | O(1) var access; captured set fixed at closure creation |
| Linked env / parent-pointer chain | Scheme, early Lisp, JS (naive) | O(depth) lookup; whole frame kept alive; simple to build |
| Open/closed upvalue cells | Lua, Wren | Shared mutable var stays live-on-stack then heap-promoted on close |
| Display (array of frame ptrs) | Pascal-lineage, some Forth | O(1) at fixed nesting; breaks under first-class/escaping funcs |
| Boxed cell per captured var | JS `let`, Smalltalk mutable temps | Each mutated capture is a heap box; unmutated stay flat |

**Impl.** Lua open-upvalue list keyed by stack slot, closed to heap on scope exit · Scheme parent-pointer env frames · JS/Smalltalk per-var heap boxes. → recipes.md#lua-upvalues

**Hazard — over-capture keeps frames alive.** A parent-pointer/whole-frame capture pins every local (and its transitive heap) for the closure's lifetime, not just the vars used — a memory leak that flat per-var capture avoids. → recipes.md#lua-upvalues

## Capture semantics & loop-variable trap
| Option | Langs | Consequence |
|---|---|---|
| Capture by reference (shared cell) | JS `var`, Python, Go <1.22 | All closures in a loop see the final value — the classic bug |
| Fresh binding per iteration | JS `let`/`const`, Go ≥1.22, Scheme `do` | Each closure captures its own copy; intuitive |
| Capture by value at creation | Rust `move`, C++ `[=]` | Snapshot semantics; later mutation invisible to closure |
| Explicit box/ref opt-in | OCaml (`ref`), Rust (`&`/`RefCell`) | Sharing is visible in the type; no accidental aliasing |

**Syntax.** JS `var` (one shared cell) vs `let` (fresh per-iter) · Go `x := x` re-bind in body · Rust `move ||` · Python `lambda x=x: …` default-arg snapshot.

**Hazard — loop var ⊗ by-reference capture.** One binding shared across iterations + deferred closures (callbacks, goroutines) = every closure reads the last value. Per-iteration fresh binding is the fix; retrofitting it is a breaking semantic change. → overlay

## Non-local return
| Option | Langs | Consequence |
|---|---|---|
| `^` returns from home method | Smalltalk, Ruby (`return` in block) | Block can exit its defining method; needs live home frame |
| `return` = block-local only | JS arrow, Rust closure, OCaml | No non-local exit; use explicit result/`?`/exceptions |
| `proc` vs `lambda` split | Ruby | `proc` return exits encloser; `lambda` returns locally — subtle |
| Escape via exception/condition | Java, Python, Scheme (`call/cc`) | Non-local exit modeled as unwinding, not a return |
| Labeled break to outer | Kotlin (`return@label`), Rust loop labels | Statically-scoped exit; no dead-home problem, no first-class escape |

**Syntax.** Smalltalk `[:x | ^x]` · Ruby `foo { return x }` / `next x` · JS/Rust `return` (block-local) · Kotlin `return@forEach`.
**Impl.** unwind stack to home frame's saved return address; if home already returned, raise dead-home error. → recipes.md#non-local-return

**Hazard — non-local return ⊗ escaping block (CROWN JEWEL).** A block with `^` stored and invoked after its home method has returned has no frame to return to. Options: raise `BlockContext`/dead-home error (Smalltalk), or forbid escaping via lifetimes (Rust). Ignoring it = corrupt unwind / UB. → overlay

## Control flow: message/library vs keyword vs macro
| Option | Langs | Consequence |
|---|---|---|
| Control = message + blocks | Smalltalk (`ifTrue:`), Self | Uniform, user-extensible; naive dispatch is catastrophically slow |
| Built-in keyword | C-family, Python, Rust `if`/`match` | Fast, non-overridable; control flow not first-class |
| Macro / special form | Scheme, Lisp, Rust `macro_rules!` | User-defined control at zero runtime cost; hygiene burden |
| Lazy args / call-by-need | Haskell (`if` as function), Scala by-name | Conditionals expressible as ordinary functions |

**Syntax.** Smalltalk `c ifTrue: [a] ifFalse: [b]` · Scheme `(if c a b)` / `(cond …)` · Rust `if c {a} else {b}` · Haskell `if c then a else b` (lazy fn).
**Impl.** compiler special-cases sacred selectors into inline branch/jump bytecode; everything else is a full message send. → recipes.md#sacred-inline

**Hazard — sacred selectors must be inlined.** If `ifTrue:`/`whileTrue:`/`and:` are real message sends, the compiler MUST special-case ("inline") them or every branch pays full dispatch + block alloc. But inlining assumes nobody overrode them — see the deopt hazard in vm.md. → overlay

## Tail-call optimization
| Option | Langs | Consequence |
|---|---|---|
| Guaranteed proper tail calls | Scheme (mandated), Lua, some ML | Loops-as-recursion run in O(1) space; a language guarantee |
| Best-effort / opt-in | OCaml, LuaJIT, Kotlin (`tailrec`) | Works when compiler proves it; silent fallback otherwise |
| None | JVM, CPython, most Ruby | Deep recursion overflows the stack; must use explicit loops |
| Explicit trampoline | Clojure (`recur`/`trampoline`) | TCO surfaced as a construct, not implicit |
| Self-tail-call only | Erlang/BEAM, Kotlin `tailrec` | Only direct self-recursion optimized; mutual recursion still grows |

**Syntax.** Scheme tail position (implicit) · Clojure `(recur …)` · Kotlin `tailrec fun` · Lua `return f(x)`.
**Impl.** overwrite current frame's args in place and jump to entry instead of pushing a new frame; guaranteed vs best-effort per proof.

**Hazard — TCO ⊗ stack traces/debugging.** Reusing the caller's frame erases it from the backtrace; the culprit call site vanishes and step-debugging loses intermediate frames. Guaranteed-TCO languages accept degraded traces; JVM refused TCO partly to preserve them. → overlay

## Iteration protocol
| Option | Langs | Consequence |
|---|---|---|
| Internal (`each:`/`do:` + block) | Smalltalk, Ruby, Self | Collection drives; simple, but no easy early-stop/zip without extra protocol |
| External iterator object | Java, Rust (`Iterator`), C++ | Consumer drives; composable, lazy, stoppable; more boilerplate |
| Generators / coroutines | Python (`yield`), Lua, JS `function*` | Suspendable producer; needs frame save/restore or stackful coro |
| Lazy stream / thunk | Haskell, Scheme streams, Clojure seq | Infinite sequences; space leaks from unforced thunks |
| Push/reducer (`fold` transducer) | Clojure transducers, Java Stream | Fusible, allocation-free pipelines; awkward to pause mid-stream |

**Syntax.** Smalltalk `coll do: [:x | …]` · Ruby `coll.each { |x| … }` · Rust `for x in it` · Python `def g(): yield x`.
**Impl.** external iterators = `next()` state object; generators/coroutines save & restore the producer frame. → recipes.md#coroutine-switch

**Hazard — internal iteration ⊗ early exit.** `each:`-style internal loops can't `break`/`return` from the caller without non-local return or a sentinel — so this axis forces the non-local-return decision above. Generators sidestep it but demand suspendable frames (vm.md). → overlay

## First-class continuations
| Option | Langs | Consequence |
|---|---|---|
| Full `call/cc` | Scheme, Ruby (`callcc`) | Any control operator expressible; captures whole stack — costly, re-entrant |
| Delimited `shift`/`reset` | Racket, Scala, OCaml effects | Composable, bounded capture; easier to reason about than full call/cc |
| One-shot / escape-only | exceptions, Java, Go `panic` | Cheap upward-only unwind; can't resume or re-enter |
| Algebraic effect handlers | OCaml 5, Koka, Eff | Resumable, typed effects; generalizes generators + exceptions |
| None | Rust, most imperative | Control is second-class; async needs a state-machine transform |

**Syntax.** Scheme `(call/cc (lambda (k) …))` · Racket `(reset (… (shift k …)))` · OCaml 5 `perform E` + `effect E, k -> …` handler.
**Impl.** full call/cc copies/reifies the whole stack; delimited operators capture only up to the enclosing prompt. → recipes.md#coroutine-switch

**Hazard — call/cc ⊗ native/heap frames.** Multi-shot continuations require copying/reifying the whole stack; incompatible with frames holding raw native pointers, C interop, or `unwind`-only runtimes. This constrains the frame-layout choice in vm.md. → overlay
