# Evaluation Strategy & Effects

> Generic design-space layer of the `language-design` skill — matrices + hazards, no textbook prose. Phalcom's committed choice on any axis here: see [../phalcom/overlay.md](../phalcom/overlay.md).
> **Load when:** designing/critiquing evaluation order, laziness/strictness, purity, or an effect system (monads / algebraic effects).

## Contents
- Axis 1 — Parameter passing / evaluation strategy
- Axis 2 — Strict vs lazy default
- Axis 3 — Argument evaluation order
- Axis 4 — Short-circuit & user-defined laziness
- Axis 5 — Purity & referential transparency
- Axis 6 — Effect tracking / systems
- Axis 7 — Lazy data structures & memoization
- Axis 8 — Effects unify control

## Axis 1 — Parameter passing / evaluation strategy
| Option | Langs | Consequence |
|---|---|---|
| Call-by-value | C, Rust, ML, Go | Arg evaluated once before call; caller's binding isolated |
| Call-by-reference | C++ `&`, Pascal `var` | Callee writes caller's variable; aliasing is explicit |
| Call-by-sharing | Python, Java, Ruby, JS objects | Value = shared reference; mutation visible, rebinding not |
| Call-by-name | Algol-60, Scala `=> T` | Arg re-evaluated at each use; no caching |
| Call-by-need (lazy) | Haskell, R promises | Evaluated at most once on first force, then cached |

**Syntax.** C `f(x)` · C++ `f(int& x)` · Scala by-name `def f(c: => Int)` · Haskell implicit thunk · Smalltalk block `[expr]` sent `value`.
**Impl.** by-value copies/moves the evaluated result; by-name/need pass a thunk (closure over the unevaluated expr), need adds a memo cell flipped on first force. → recipes.md#lua-upvalues
**Hazard — call-by-sharing ⊗ mutation surprises.** "Pass by value of a reference": mutating the shared object leaks across the call, but reassigning the param doesn't — callers can't tell which a function does from its signature. Defensive copies or immutability are the only guards. → overlay

## Axis 2 — Strict vs lazy default
| Option | Langs | Consequence |
|---|---|---|
| Strict, opt-in laziness | ML, Scheme, Rust, most | Predictable timing; laziness is explicit (`delay`/thunk/block) |
| Lazy, opt-in strictness | Haskell, Miranda | Infinite data & fine control; timing/space hard to reason about |
| Strict with lazy-`val` sugar | Scala, Swift `lazy` | Per-binding deferral without going lazy language-wide |

**Syntax.** Scheme `(delay e)` / `(force p)` · Smalltalk `[e]` then `value` · Haskell strictness via `seq e2` / `!x` bang pattern / `!`-fields · Scala `lazy val`.
**Impl.** strict evaluates at bind/apply; lazy compiles every binding to a thunk + update-frame, `seq`/bang inject forced evaluation to break thunk chains. → closures-control.md
**Hazard — laziness ⊗ space leaks (CROWN JEWEL).** Unforced thunks accumulate instead of collapsing to a value: `foldl (+) 0 [1..n]` builds an O(n) thunk chain that overflows on force. Fix is strict fold / `seq` / bang patterns — but the leak is invisible in source; it lives in the evaluation order. → overlay

## Axis 3 — Argument evaluation order
| Option | Langs | Consequence |
|---|---|---|
| Left-to-right guaranteed | Java, JS, C#, Swift | Side-effect order deterministic and portable |
| Unspecified / impl-defined | C, C++ (pre-17), Scheme | Compiler may reorder; effect order not portable |
| Right-to-left | OCaml (in practice) | L-to-R code that relies on order silently misbehaves |
| Sequenced-but-unordered | C++17 | Each arg fully evaluated before another starts; order still free |
| Single argument (curried) | Haskell, ML | One arg per application; order question dissolves |

**Syntax.** all `f(g(), h())` — semantics differ · C++17 fixed *indeterminate-but-sequenced* (still unordered) · OCaml `f (g ()) (h ())`.
**Impl.** codegen picks a push order; a guarantee pins it, "unspecified" lets the optimizer reorder for register/spill efficiency.
**Hazard — unspecified arg-eval-order ⊗ side effects (CROWN JEWEL).** In C, `f(g(), h())` where `g`/`h` both mutate shared state (or `i++ + i++`) has implementation-defined result — a portability and soundness trap that compiles clean and passes some compilers' tests. Pick a guaranteed order or forbid effectful arg positions. → overlay

## Axis 4 — Short-circuit & user-defined laziness
| Option | Langs | Consequence |
|---|---|---|
| Built-in short-circuit operators | C, Java, Python `and`/`or` | `&&`/`||`/`?:` skip RHS; hardwired, not overridable |
| Lazy args via by-name/thunk | Scala `=> T`, Haskell | User functions get short-circuit semantics for free |
| Blocks/thunks as messages | Smalltalk, Self | `and:`/`ifTrue:` are library methods over unevaluated blocks |
| Macros / special forms | Lisp, Scheme | `and`/`or`/`cond` expand to nested `if`; zero-cost laziness |

**Syntax.** C `a && b` · Smalltalk `a and: [b]` · `c ifTrue: [x] ifFalse: [y]` · Scheme `(and a b)` · Scala `def or(l: => Bool)`.
**Impl.** operator lowers to a branch that skips the RHS bytecode; message form passes an unevaluated block and only sends it `value`/`call` on the chosen path — see sacred-selector inlining. → closures-control.md
**Hazard — library laziness ⊗ dispatch cost.** `and:`/`ifTrue:` built as ordinary message sends over blocks cost a send + block alloc per branch unless the compiler inlines the sacred selectors; correctness is fine, but hot control flow is catastrophically slow without special-casing. → overlay

## Axis 5 — Purity & referential transparency
| Option | Langs | Consequence |
|---|---|---|
| Pure by default, effects in type | Haskell, Elm, PureScript | Equational reasoning, free memoization/parallelism; `IO` ceremony |
| Impure, effects ambient | C, Java, Python, Smalltalk, most | Any expression may do IO; convenient; no reasoning guarantees |
| Pure core + controlled effect zone | Roc, Koka, Unison | Purity where it pays, escape hatch that's still tracked |
| Mutation-restricted / linear | Rust (`&mut` uniqueness), Clojure | Effects allowed but aliasing/sharing constrained |
| Pure-veneer escape hatch | Haskell `unsafePerformIO` | Types say pure, runtime isn't — reasoning silently unsound |

**Syntax.** Haskell `pure x` / `x <- action` in `do` · Elm managed effects via `Cmd` · Clojure `atom`/`swap!` isolates mutation.
**Impl.** purity is a typing/discipline property, not runtime — `IO a` is a value describing an action, run only by the runtime's `main`; the compiler may reorder/CSE/memoize pure subexpressions freely.
**Hazard — memoization/CSE ⊗ hidden effects.** A compiler that assumes purity will cache or elide a call; if that call secretly does IO (impure language pretending, or `unsafePerformIO`), the effect runs once, zero times, or out of order. Purity must be enforced, not assumed, before you lean on it. → overlay

## Axis 6 — Effect tracking / systems
| Option | Langs | Consequence |
|---|---|---|
| Untracked / ambient | Python, Java, Ruby, Smalltalk, most | Signature says nothing about what a fn does; max flexibility |
| Monads + transformers | Haskell (`mtl`) | Effects in the type; stacking composes them; ordering boilerplate |
| Algebraic effects & handlers | Koka, Eff, OCaml 5, Unison | Effects declared, handlers interpret; composable, resumable |
| Capabilities / effect polymorphism | Scala 3 (caps), Koka rows | Fn is polymorphic over the effects it's given; least authority |

**Syntax.** Haskell `f :: a -> IO b` / `ExceptT e (StateT s IO)` · Koka `fun f() : <exn,st> a` · OCaml 5 `perform E` + `match … with effect E k -> …`.
**Impl.** monads thread a hidden state/continuation via `>>=`; effect handlers capture a delimited continuation at `perform` and hand it to the nearest matching handler (generalizes exceptions + generators). → closures-control.md
**Hazard — monad-transformer stacking ⊗ composition.** `StateT s (ExceptT e IO)` vs `ExceptT e (StateT s IO)` differ in whether state survives an error; the stack order is semantically load-bearing, `lift` noise grows with depth, and two libraries' stacks don't compose without adapters. Effect rows/handlers were invented to escape this. → overlay

## Axis 7 — Lazy data structures & memoization
| Option | Langs | Consequence |
|---|---|---|
| Lazy `val` / `Lazy<T>` | Scala, Swift, C# `Lazy<T>` | Deferred, computed-once per binding; needs thread-safety story |
| Streams / lazy seqs | Haskell, Clojure, Scheme | Infinite/self-referential data; head realized on demand |
| Generators / promises | Python, JS, Lua | Producer suspends; pull-based; explicit `yield`/`await` |
| Explicit thunk + memo cell | ML `Lazy.t`, hand-rolled | Full control; caller manages force + caching |
| Memoized pure function | Clojure `memoize`, table cache | Result cached by args; only sound if the fn is pure |

**Syntax.** Scala `lazy val x = e` · Clojure `(lazy-seq …)` · Haskell `let xs = 1 : xs` · Python `def g(): yield …` · OCaml `lazy e` / `Lazy.force`.
**Impl.** a memo thunk = closure + `state` (unforced | forcing | value); first force runs and overwrites, re-entry during `forcing` signals a cyclic-dependency error. → closures-control.md
**Hazard — lazy memo ⊗ concurrency.** A shared `Lazy<T>` forced from two threads either double-computes (benign only if pure) or needs a lock/once-guard; the `forcing` state also makes cyclic forces (self-referential lazy val) deadlock or throw. Effectful lazy init is the sharp case. → concurrency.md

## Axis 8 — Effects unify control
| Option | Langs | Consequence |
|---|---|---|
| Separate ad-hoc constructs | C, Java, Go | Exceptions, iterators, async each hand-built and disjoint |
| One-shot delimited continuations | async/await, generators desugared | Suspend/resume once; powers `yield` + `await` uniformly |
| Multi-shot continuations | Scheme `call/cc`, Racket | Resume many times; backtracking, amb, re-entrant control |
| Algebraic effect handlers | Koka, OCaml 5, Eff, Unison | Exceptions, async, generators, state = one `perform`/handler |

**Syntax.** JS `function*`/`yield` + `async`/`await` (distinct sugars) · OCaml 5 one handler expresses both · Racket `shift`/`reset`.
**Impl.** all four are delimited continuations under different names: a handler captures the stack slice from `perform` to the handler; resuming zero times = exception, once = async/generator, many = backtracking. → errors.md · concurrency.md
**Hazard — effect handlers ⊗ resource cleanup / multi-shot resumption (CROWN JEWEL).** A handler that resumes a captured continuation *twice* re-runs everything after the `perform` — including code already past an `ensure`/`finally`, or a continuation that escapes and resumes after its resource scope closed. One-shot restriction (OCaml 5) or linear continuations tame it; unrestricted multi-shot ⊗ `defer`/`Drop`/`ensure` gives double-cleanup or use-after-free. Ties errors.md cleanup ordering and closures-control non-local-return. → overlay

## Phalcom anchor
Control flow is **blocks (thunks) sent as messages** — explicit call-by-need *for control only*: `and:`/`or:`/`ifTrue:` are lazy because the argument block isn't evaluated until sent `call`/`value`, and the compiler inlines these sacred selectors (Axis 4 hazard). Everything else is **strict call-by-sharing** (Axis 1) over object references. There is **no effect system**: effects are untracked/ambient (Axis 6, row 1) — no purity guarantee, so no free memoization/reordering (Axis 5 hazard applies to any future optimizer). An effect system would buy composable, typed control (unify exceptions + non-local return + future async as one handler mechanism, Axis 8) but costs a static effect discipline that a dynamic message-send language has no place to record — every send would need an effect row the runtime can't check, so it stays ambient by design. → overlay
