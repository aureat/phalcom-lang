# Phalcom Overlay

> Phalcom's *committed* design positions, keyed to the generic axes in [../references/](../references/). This is the authoritative "what we chose" layer — when the generic reference lists options, this says which one Phalcom took and why it's locked. Update when an ADR lands or an open question resolves.

Source of truth: `docs/adr/0001…0016`, `docs/spec/*.md`, open register in `docs/spec/open-questions.md` + `docs/forge/PHASE2-INDEX.md` (DEC-A…F).

## Committed positions
| Axis (→ generic ref) | Phalcom's choice | ADR / spec § | Consequence / what it locks |
|---|---|---|---|
| Metaclass strategy (→ object-model) | Parallel metaclass tower; `(X class).super == (X.super) class`, root `(Object class).super == Class` | ADR-0002, object-model §5 | Class-side `static`/`construct` inherit correctly |
| Kernel shared super (→ object-model) | `Behavior` abstract super of `Class` & `Metaclass`, under `Object` | ADR-0003, object-model §4 | Method-dict/lookup/alloc protocol lives in one place |
| Inheritance model | Single inheritance; `Object` is root; one `superclass` per class | object-model §1.5 | Traits/mixins/MI explicitly out (open-Q10) |
| Bootstrap ordering | Allocate-then-wire; `Metaclass` instance-of itself; `verify_invariants()` guard | ADR-0002, object-model §6 | Cyclic apex expressed as handle patches, not `new_cyclic` |
| Dispatch key (→ method-lookup) | Label-encoded selector symbol (name+labels); one hashmap probe | ADR-0012, messages §2, method-lookup §1 | `foo`, `foo()`, `foo(_)`, `move(to,duration)` all distinct |
| Selector spelling | Canonical **comma** form `move(_,to,duration)` | ADR-0012 (amended), selectors §1 | Compiler & every runtime builder share one `encode_selector` |
| Dispatch shape | Inline-cache-ready (per-site monomorphic slot keyed by `ClassId`); IC population deferred | ADR-0012 | Adding IC later is not a redesign |
| Value representation (→ values) | Tagged Rust `enum`: `Number(f64)`,`Bool(bool)`,`Obj(ObjRef)`,`Symbol`, private `Nil` | ADR-0010 | NaN-boxing deferred behind same enum API |
| Object ownership | Handle/arena `Heap`; `Copy` `ObjRef`/`ClassId`; no `Rc`/`RefCell` | ADR-0009 | No borrow-panic surface; GC-ready arena |
| Absence (→ values-and-absence) | Abstract `Option` + `Some(_value)`/`None`; `None` shared singleton; no surface `nil` | ADR-0007, values-and-absence §3 | Dispatch replaces branching; only `Some` allocates |
| `nil` | Private VM sentinel only — no class, no literal, cannot leak into `Some` (Invariant 4) | ADR-0010, values-and-absence §2 | Standing obligation of Option work |
| Truthiness | **Banned** — `if (opt)` is a compile error; conditions must be `Bool` | ADR-0007 §3.5 | Signposted JS deviation (Invariant 6) |
| Bool tower | Abstract `Bool` + singleton `True`/`False`; `and`/`or`/`ifTrue` dispatch by class | ADR-0004 | No new `Value` arm — class selected from `Bool(bool)` payload |
| Numeric type | Single flat `Number` backed by `f64` | ADR-0005, object-model §4 | Int/Float **surface** split still open (open-Q2) |
| Callable root | Abstract `Function`; `Block` & `Method` siblings under it | ADR-0006, functions §1 | One `ClosureObject`; `Fiber`/`Future` take any `Function` |
| Closures / capture | Lua-style open→closed upvalues; cells live in heap | ADR-0013, blocks §5 | Escaping blocks + shared mutation of captured `var` |
| Non-local return | `return` in block unwinds to home method via frame token (ptr+generation) | ADR-0013, blocks §5 | Dead home frame → `DeadFrameError`, not UB |
| Unwind primitive | `return`/`throw`/fiber `abort` are one stack-unwind; `ensure` fires on any | ADR-0008 §4, ADR-0013 | `ensure`/`finally` on non-local return too |
| Control flow (→ control-flow) | `if`/`while`/`for` = keyword sugar over block sends; short-circuit args are blocks | control-flow §1–2 | Laziness falls out of object model |
| Inliner | Sacred selectors (`ifTrue(_)`,`and(_)`,`whileTrue(_)`,…) inlined w/ deopt type guard | control-flow §3, ADR-0017 (drafting, U5) | Zero closure alloc on hot path; must land early (Invariant 5) |
| Bindings | `let` immutable / `var` mutable; `var x` w/o init = `None`; `let x` w/o init rejected | ADR-0014, open-Q1 | One absence rule for fields + bindings |
| Error handling | Layered: unwinding `throw`/`Error` + value `Result`(`Ok`/`Err`); bridges; **terminating** | ADR-0008, error-handling | Rejects Smalltalk `resume:`; only `Error` subclasses throwable |
| `try`/`catch`/`finally` | Pure sugar over `Block` protocol `on(_)(_)`/`ensure(_)`/`attempt()` | error-handling §2 | No semantics beyond block sends |
| doesNotUnderstand / reflection | Failed send reified as `Message`; slow path re-sends `doesNotUnderstand(_)`; `perform`/`SEND_DYNAMIC` | method-lookup §2–3, messages §5, ADR-0012 | Proxies/DSLs/`respondsTo` for free; `perform` selector-symbols only |
| Variadics | Rest `*xs` positional-only, last param; interns `sum(*)`; variadic-table fallback probe | messages §4, ADR-0012 | No `**kwargs` (labels are identity) — take a `Map` |
| Instance layout / construct | Fixed per-class slot vector; `GetField/SetField(slot)`; `construct` keyword on metaclass | ADR-0011, classes §1–2 | Read-before-write = compile error; fields private, non-inherited |
| Default toString | Instance renders `"<ClassName>"`; class `toString`=own name; no `printString` | ADR-0015 | Fixes F4 (class reported metaclass name) |
| Lexer/parser | Hand-written byte lexer + recursive-descent/Pratt parser; panic-mode recovery; LALRPOP removed | ADR-0016 | Newlines are tokens; multi-error diagnostics |
| String interpolation | `{expr}` desugars to `toString` + concat; `\{` escapes | lexical §5 | Sigil choice still open (open-Q5 / DEC-F) |
| Symbols / method refs | `#name`/`#sel(_,..)` atomic tokens; `::` Open/Pinned `Family`; `@` attributes | lexical §10–12, selectors §2–4 | `#`≠`@`≠`::`, no JS `#priv` fields |
| Concurrency | Cooperative single-threaded; `Fiber` sole primitive; `Future` = state machine over it | concurrency §1–2 | No preemption, no data races; ADR pending |

## Open / undecided (do NOT design as if settled)
| Question | Status | Where |
|---|---|---|
| Int/Float surface split | OPEN (f64 settled underneath; surface split open) | open-Q2, ADR-0005 |
| External vs internal param names | OPEN (field reserved in `Signature`) | open-Q3, ADR-0012 |
| Class-hierarchy mutability (runtime `superclass=`) | OPEN (heap keeps it implementable; policy undecided) | open-Q4, ADR-0009 |
| String-interpolation sigil (`{}`/`${}`/`\(...)`) | OPEN (rec: `{expr}`; needs ADR) | open-Q5, DEC-F |
| Set literal syntax (`Set(..)` vs `#{..}`) | OPEN | open-Q6 |
| Destructuring (`let (a,b)=…`, `[first,*rest]`) | OPEN (not specified) | open-Q7 |
| Modules / imports semantics | OPEN (`import` token exists, no semantics) | open-Q8 |
| Traits / mixins / multiple inheritance | OPEN (single inheritance is current invariant) | open-Q10 |
| Default arguments | OPEN — **decide before shipping** (fights selector identity) | open-Q12, selectors §7.3 |
| `Option` bootstrap (VM-blessed vs stdlib class) | OPEN | open-Q13, selectors §7.4 |
| `Family` reflective introspection | OPEN | open-Q14, selectors §7.5 |
| `Family` vs `Method.bind` unification | OPEN (two routes coexist) | functions §3 |
| `@construct`/`@get`/`@set` vs hand-written `construct`/accessors | PLANNED, relationship TBD | classes §1, §3 |
| Kernel `List`/collections unit | OPEN — unscheduled but hard dep of dNU/`Message.args` & rest-params | DEC-A |
| Variadic dispatch-table key | OPEN — spec key `(name,min_arity)` not implementable; rec: bare name + reject 2nd variadic | DEC-B, messages §4 |
| `if(opt)` compile-error mechanism | OPEN — no flow analysis; rec: runtime `Bool`-only branch + reject syntactically-literal Option conds | DEC-C |
| Class-side *stored* static fields | OPEN — rec: descope; ADR-0011 "static"=instance layout (naming collision) | DEC-D |
| Owner of `if`/`while`/`for` surface parsing | OPEN — rec: U5 owns parse-time desugar (no CF AST node today) | DEC-E |
| Structured concurrency / cancellation, `select`/`race`, scheduler fairness | OPEN | concurrency §3 |
| `retry` on protected block | DEFERRED (out of Draft 0.1; block stays live) | error-handling §3 |
| Inline-cache population; NaN-boxing | DEFERRED optimizations behind committed APIs | ADR-0012, ADR-0010 |

## Hazards Phalcom has already hit (canonical case studies)
- **default args ⊗ selector-identity dispatch** — omitting a defaulted arg produces a *different* selector → lookup misses the full-arity method. Only fixes are combinatorial arity-family expansion or unavailable static callee knowledge. Status: **open-Q12 / selectors §7.3, unresolved — decide before shipping.**
- **Option bootstrap cycle** — fields default to `None`, but constructing `None` needs a class whose fields default to `None` → `Option` must be VM-blessed / niche-encoded in `Value`. Status: **open-Q13 / selectors §7.4.**
- **`ifTrue`/`ifFalse` → `Option` breaks chaining** — `cond.ifTrue{a}.ifFalse{b}` sends `ifFalse` to an `Option`, not a `Bool`; `ifTrue{None}` is indistinguishable from the branch not taken. Rec: paired `ifTrue(_)ifFalse(_)` primary. Status: **open-Q1 note / selectors §7.2, not adopted.**
- **`var x` = `None` reintroduces `nil` under a new name** — every var becomes `T | None`. Alternative floated: VM `Uninit` sentinel that traps on read. Status: **open-Q1 note / selectors §7.1, resolution stands but flagged.**
- **truthiness ban without flow analysis** — banning `if(opt)` has no static/flow analyzer to enforce it; general static detection is impossible. Rec: runtime no-coercion floor + reject only syntactically-literal Option conditions. Status: **DEC-C, unresolved mechanism.**
- **variadic dispatch key unimplementable as written** — messages §4 keys the variadic table by `(name, min_positional_arity)`, but a call of arity K needs `min ≤ K`; an exact-tuple hash can't answer that. Rec: key by bare name, reject a 2nd same-name variadic at definition. Status: **DEC-B, needs ADR-0012 amendment.**
- **kernel `List` is an unscheduled hard dependency** — `Message.args` (dNU) and rest-params both require `List`, which no unit builds yet and which collides with U-STD in the same wave. Status: **DEC-A, elevate onto critical path.**
- **`static` naming collision** — ADR-0011 uses "static" for the instance slot layout, but class-side *stored* fields (also "static") are unspecified. Status: **DEC-D, rec descope class-side storage to its own ADR/unit.**
- **non-local `return` across a fiber boundary** — a block's home frame lives on one fiber's stack; returning across fibers would unwind the wrong stack. *Handled by construction*: generation mismatch raises `DeadFrameError`. Ref: ADR-0013, concurrency §3.
- **`Rc<RefCell>` cycle-leak + double-borrow panic (F5/F1-class)** — the old cycle-breaker was inert (kernel never freed) and `RefCell` re-borrow-during-send double-panicked. *Removed by construction* via the handle heap. Ref: ADR-0009 (F5).
- **audit findings folded into ADRs, not patched piecemeal** — F1 (`Invoke` swallowed a primitive `Result` → silent exit-0), F7 (0-arg `new` registered as `Method(1)`), F8 (malformed `">( _)"` operator selector) all fixed inside ADR-0012; F4 (class `toString`→metaclass name) fixed in ADR-0015; F9/F10 (parse-error panic, trailing-newline fail) in ADR-0016.
