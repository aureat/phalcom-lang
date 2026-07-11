# Errors, Failure & Cleanup

> Generic design-space layer of the `language-design` skill — matrices + hazards, no textbook prose. Phalcom's committed choice on any axis here: see [../phalcom/overlay.md](../phalcom/overlay.md).
> **Load when:** deciding the error model, unwind vs resume, exception↔Result bridging, cleanup/RAII, panic/FFI boundaries, error payloads, or failing constructors.

## Contents
- Axis 1 — Error model
- Axis 2 — Resumable vs terminating unwind
- Axis 3 — Exceptions ⊗ Result bridging
- Axis 4 — Cleanup discipline
- Axis 5 — Panics/aborts vs recoverable errors
- Axis 6 — Error carrying data
- Axis 7 — Failure in constructors

## Axis 1 — Error model
| Option | Langs | Consequence |
|---|---|---|
| Unchecked exceptions | Java (RuntimeEx), Python, Ruby, JS, Smalltalk | Ergonomic; invisible control flow; every call may throw |
| Checked exceptions | Java (checked) | Signature-documented failure; boilerplate; `throws` leaks/`catch{}` swallows |
| Result / Either values | Rust, OCaml, Haskell, Go(ish), Swift `throws` | Explicit, in the type; noisy propagation without sugar (`?`/`do`) |
| Conditions & restarts | Common Lisp | Handle without unwinding; powerful; rare, mental-model heavy |
| Multi-value return `(v, err)` | Go | Uniform, visible; `if err != nil` everywhere; easy to ignore |
| Panic / abort | Rust `panic!`, Zig, Go `panic` | For bugs/unrecoverable; unwinds or aborts; not for expected errors |

**Syntax.** Java `try{}catch(E e){}finally{}` · Python `try/except/finally` · Rust `Result<T,E>`/`foo()?` · Go `v, err := f()` · CL `(handler-case … (error (c) …))` · Ruby `begin/rescue/ensure`.

**Hazard — checked/typed errors ⊗ higher-order functions.** A `map`/callback that can fail forces its error type onto every combinator; without polymorphism over effects (or `?`-style sugar) the error type infects and fragments generic APIs. → overlay
**Hazard — two-tier (exceptions + Result) ⊗ discipline.** Offering both invites inconsistency: libraries split arbitrarily between throwing and returning, so callers must handle both mechanisms for one logical failure. → overlay

## Axis 2 — Resumable vs terminating unwind
| Option | Langs | Consequence |
|---|---|---|
| Terminating unwind (stack discarded) | Java, Python, Rust, C++, most | Simple; handler runs after frames gone; context lost at raise site |
| Resumable conditions (stack preserved) | Common Lisp, Smalltalk (`resume:`) | Handler runs atop live stack, can resume/retry in place |
| Restart-based (named recovery points) | Common Lisp | Signaler offers strategies; handler picks; ultimate flexibility |

**Syntax.** CL `(restart-case … (retry () …))` + `(invoke-restart 'retry)` · Smalltalk `[…] on: Error do: [:e | e resume: v]` · terminating: `throw`/`raise` everywhere else.
**Impl.** Resumption keeps the signaling frame live and runs handlers atop it (no eager pop); terminating models unwind while searching (see recipes.md#non-local-return).

**Hazard — resumable-vs-terminating decides the whole stack discipline.** Resumption requires the signaling frame to stay live while the handler runs, so handlers execute *before* unwinding and unwinding is a separate later step. This forbids the eager "pop frames as you search for a handler" model and dictates VM stack layout, cleanup timing, and every language it can host. Choose it first — it is not retrofittable. → overlay
**Hazard — resume ⊗ cleanup ordering.** If handlers run before unwinding, `ensure`/`finally` blocks between raise and handler must NOT run until the handler decides to unwind — otherwise resources are freed under a handler that may resume into freed state. → overlay

## Axis 3 — Exceptions ⊗ Result bridging
| Option | Langs | Consequence |
|---|---|---|
| Result-only, panics for bugs | Rust | `?` propagates; `panic`/`catch_unwind` a separate rare escape hatch |
| Exceptions modeled as Result sugar | Swift `throws`/`try`, Haskell `ExceptT` | `try` = auto-`?`; typed-ish; still stack-unwinding under hood |
| Convert at layer boundary | Go `errors`, Rust `From`/`?` | Wrap/`map_err` at each layer; boundary is where classification happens |
| Reify exceptions as values | `Result::catch`, `try/except`→object | Uniform handling; loses cheap throw; must materialize trace |

**Syntax.** Swift `try foo()` / `func f() throws` · Rust `foo()?` on `Result` / `catch_unwind` · Haskell `runExceptT`/`ExceptT` · Go `errors.Wrap`/`fmt.Errorf("…%w")`.

**Hazard — `throw`↔`Result` round-trip ⊗ information loss.** Catching an exception into a `Result` (or `?`-ing a `Result` into a throw) at every layer boundary strips or re-wraps context; without a cause-chain the original raise site and stack trace vanish by the time it's handled. → overlay
**Hazard — `?`/try sugar ⊗ cleanup.** Sugar that turns a value into an early return/throw executes cleanup as it unwinds; a `?` buried in an expression can skip code the reader assumed ran — cleanup must be tied to scope exit, not statement order. → overlay

## Axis 4 — Cleanup discipline
| Option | Langs | Consequence |
|---|---|---|
| `try`/`finally` | Java, JS, Python | Explicit block; runs on all exits; easy to forget/nest badly |
| `ensure` (block form) | Ruby, Smalltalk | Same as finally; ties cleanup to a lexical block |
| `defer` (stack of thunks) | Go, Zig, Swift | Registered at use site, LIFO at scope exit; loop-defer pitfalls |
| RAII / `Drop` (destructor) | Rust, C++ | Automatic, deterministic on scope/unwind; ownership required |
| Bracket / `with` / RAII combinator | Haskell `bracket`, Python `with` | Acquire+release paired as one HOF; exception-safe by construction |

**Syntax.** Java/JS `try…finally` · Ruby `begin…ensure` · Go `defer f()` (LIFO) · Rust `impl Drop for T` · Haskell `bracket acq rel use` · Python `with`.
**Impl.** Cleanup registered per scope, run in reverse on every exit; block/closure non-local return must unwind through them (see recipes.md#non-local-return).

**Hazard — RAII/`ensure` ⊗ non-local return/unwind ordering.** A non-local exit (exception, `return` from block, generator close) must run *all* pending destructors/`ensure`/`defer` in strict reverse order while passing through. If an `ensure` itself raises, or a `Drop` panics mid-unwind, you get lost exceptions or double-unwind — define the "error during cleanup" rule explicitly. → overlay
**Hazard — cleanup ⊗ resumable handlers.** With resumable conditions (Axis 2), "scope exit" is ambiguous: the handler may resume, so cleanup can't fire on raise — only on the eventual real unwind. RAII-style eager release is incompatible with resumption. → overlay

## Axis 5 — Panics/aborts vs recoverable errors
| Option | Langs | Consequence |
|---|---|---|
| Panic unwinds, catchable at boundary | Rust `catch_unwind`, Go `recover` | Isolate faults per task/request; unwind-safety burden on shared state |
| Panic aborts process | Rust `panic=abort`, Zig | No cleanup, no corruption risk; whole process dies |
| "Let it crash" per-process supervision | Erlang/OTP | Faults kill one process; supervisor restarts; needs isolated heaps |
| No panics; all failure recoverable | pure Result langs | Uniform; forces modeling truly-impossible states as errors |

**Syntax.** Rust `panic!()` / `catch_unwind` · Go `panic()` / `recover()` · Zig `@panic("…")` / `unreachable` · Erlang `exit(reason)` + supervisor restart.

**Hazard — panic ⊗ FFI/task boundary.** Unwinding across an FFI frame (into C) or across a thread/task boundary is undefined or forbidden; a panic must be caught and converted to an error code/value *before* it reaches the boundary, or it aborts/corrupts. → overlay
**Hazard — catch-unwind ⊗ shared mutable state.** Catching a panic and continuing means state mutated before the panic may be left half-updated (broken invariants); "unwind-safety" must be reasoned about or the recovered process operates on corrupt data. → overlay

## Axis 6 — Error carrying data
| Option | Langs | Consequence |
|---|---|---|
| Exception class hierarchy | Java, Python, Ruby, Smalltalk | Catch by type/supertype; open-ended; catch-too-broad risk |
| Typed error enum / sum | Rust, OCaml, Haskell | Exhaustive `match`; closed set; adding a variant breaks callers (good) |
| String / code | C `errno`, Go sentinel `errors.New` | Trivial; no structure; string-matching for classification is fragile |
| Error with cause-chain | Java `getCause`, Rust `source()`, Go `%w` | Preserves layered context; must thread wrapping through every boundary |

**Syntax.** Java `class E extends Exception` + `getCause()` · Rust `enum E { … }` + `#[from]`/`source()` · Go `fmt.Errorf("…: %w", err)` · Python `raise X from cause`.

**Hazard — stack-trace capture cost.** Capturing a full backtrace at every `throw`/construction (JVM/Python default) makes exceptions costly enough that they can't be used for expected control flow; if errors *are* control flow (Result, Go), traces must be opt-in or deferred, trading debuggability for speed. → overlay
**Hazard — open hierarchy ⊗ exhaustiveness.** An open exception hierarchy can't be exhaustively matched; a closed enum can but can't be extended by third parties without a breaking change — you can't have both extensibility and exhaustiveness. → overlay

## Axis 7 — Failure in constructors
| Option | Langs | Consequence |
|---|---|---|
| Constructor may throw | Java, C++, Python `__init__` | Partially-built object may leak; destructor/`Drop` on half-state |
| Factory returns Result/Option | Rust (`new()->Result`), OCaml | No partial objects; "no invalid instance exists" guarantee |
| Failable initializer | Swift `init?`/`init throws` | Returns `nil`/throws; language nulls out the partial instance |
| Two-phase init (construct then `open`) | many C libs, Objective-C | Object exists invalid until step 2; every method must guard |

**Syntax.** Swift `init?` / `init() throws` · Rust `fn new() -> Result<Self, E>` · Java `throw` inside ctor · C++ throwing ctor (member RAII) · Obj-C two-phase `alloc`/`init`.

**Hazard — throwing constructor ⊗ RAII/cleanup.** If a constructor throws after acquiring some resources/fields, the object never fully exists so its destructor may not run — each acquired sub-resource must be independently owned (RAII members) or leaks; base-then-derived init makes the half-constructed window worse. → overlay
**Hazard — partial object ⊗ absence/nil defaults.** A field left unset by a failed/two-phase init defaults to nil/zero; combined with nil-punning (see values.md) the broken object answers messages instead of erroring, hiding the construction failure until far away. → overlay
