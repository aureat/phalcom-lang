# Optional Types (Experimental)

Part of the [Phalcom Language Specification](../README.md). Status: **Experimental —
not committed.** This is an exploratory design note, not a spec part and not an ADR.
Nothing here binds an implementation; no [invariant](../README.md#invariants) is
amended by it. It exists to capture a coherent typing design end-to-end — idea,
model, implementation, edge cases, risks — so the decision can be taken (or
rejected) deliberately rather than improvised.

> **Partially superseded (2026-07-12).** The "flat f64 numbers, Int/Float split open
> (ADR-0005, Q2)" premise is now **closed** by
> [ADR-0024](../../../adr/0024-numeric-surface-split-int-float-and-division.md): exact
> unbounded `Int` + `Float`, with `/` true division and `~/` floor integer division.
> This also settles the "integer division threatens erasure" tension — **§5.9 should be
> revisited against `~/`** (floor division is a distinct selector, not an erasure of
> `/`). Index: [deferred-work.md](../deferred-work.md).

**Anchoring ADRs (constraints, not endorsements):**
[ADR-0002](../../../adr/0002-metaclass-tower-parallel-rule.md) (metaclass tower) ·
[ADR-0004](../../../adr/0004-boolean-as-abstract-bool-with-true-false.md) (Bool tower) ·
[ADR-0005](../../../adr/0005-number-as-flat-f64.md) (flat `f64`) ·
[ADR-0007](../../../adr/0007-option-as-abstract-with-some-none.md) (Option/Some/None) ·
[ADR-0009](../../../adr/0009-handle-arena-heap.md) (handle heap) ·
[ADR-0010](../../../adr/0010-tagged-value-enum.md) (tagged `Value`) ·
[ADR-0012](../../../adr/0012-selector-signature-encoding-and-dispatch.md) (selector identity & dispatch)

Relates to open questions: [Q2 Int/Float](../open-questions.md), [Q3 param labels](../open-questions.md),
[Q4 hierarchy mutability](../open-questions.md), [Q10 traits](../open-questions.md).

---

## 1. The idea

Add types to Phalcom as an **optional, structural, erasable** layer — Strongtalk's
"pluggable types," not TypeScript-as-default and not Typed-Racket-with-contracts.

Three words, each load-bearing:

- **Optional.** Programs run un-annotated exactly as today. Annotations are checked
  where present and never required.
- **Structural.** A type describes *which messages an object understands*, matching
  the message-send object model. Protocols are named entities, satisfied
  structurally (§5.3).
- **Erasable.** Annotations carry **zero runtime semantics.** Stripping every
  annotation yields a byte-identical bytecode program. Types are a *pre-compile
  analysis pass*, never a runtime mechanism.

This shape is not a free choice — it is forced by what Phalcom has already
committed to (§2). Message-send + `doesNotUnderstand` is exactly the language shape
Strongtalk was built to type; the erasable/optional stance is the only one that
leaves the dynamic substrate (ADR-0009/0010/0012) untouched.

### 1.1 Why not the alternatives

| Alternative | Why rejected here |
|---|---|
| Mandatory static types | Kills the dynamic core: `doesNotUnderstand`, `perform`, proxies, runtime hierarchy edits ([Q4](../open-questions.md)) become untypable or illegal. |
| Gradual-with-contracts (Typed Racket) | Inserts blame-tracking contracts at every typed/untyped boundary → real runtime cost, violating the "only `Some`/heap objects allocate" posture (ADR-0010). |
| Types drive dispatch (CLOS/Julia multimethods) | Would make argument types part of selector identity → breaks the one-hashmap-probe `name+labels` invariant (ADR-0012, Invariant 2/3). |

Erasable-optional keeps *all* runtime semantics decided by the existing dispatch
machinery and confines types to a checker that emits diagnostics and then discards
everything it learned.

---

## 2. Design forces (what the committed positions dictate)

The interesting content of this design is not "we chose types" but which *specific*
type system the locked decisions permit. Each row is a constraint the checker must
obey, and the mechanism it forces.

| Committed position | Forces |
|---|---|
| Selector identity = `name+labels`, one probe (ADR-0012, Inv. 2–3) | Types **cannot** be dispatch keys. Must be **erased** (§5.2). Protocol members key on the *same* `encode_selector`. |
| `doesNotUnderstand` / `perform` / proxies are first-class ([method-lookup](../method-lookup.md)) | A static checker cannot see dNU-provided methods → a `Dynamic` (`?`) escape hatch is mandatory (§5.7). |
| Absence is `Option`/`Some`/`None`, `None` a shared singleton (ADR-0007) | No nullable tracking needed, **but** the core library is already generic → generics + a `Nothing` bottom type are mandatory, not optional (§5.4). |
| Tagged `Value`, handle heap, "no new runtime cost" (ADR-0009/0010) | Erasable types honor this; contract systems do not. |
| `Bool` abstract + `True`/`False`, truthiness banned (ADR-0004, [values §3.5](../values-and-absence.md)) | Typed code gets *static* enforcement of the ban; untyped code keeps the runtime floor (§5.10). |
| Parallel metaclass tower (ADR-0002) | Classes are values with type `T class`; constructors/factories must be typed on the class side (§5.6). |
| Flat `f64` numbers, Int/Float split open (ADR-0005, [Q2](../open-questions.md)) <!-- RESOLVED by ADR-0024: split decided (exact `Int` + `Float`); `~/` floor div removes the erasure tension — see §5.9. --> | A surface `Number > Int, Float` split can live in the *type lattice only* — but integer division threatens erasure (§5.9). |

---

## 3. Non-goals (explicit)

- **No gradual guarantee / no soundness at the dynamic boundary.** A fully
  well-typed method can still raise `doesNotUnderstand` at runtime when *untyped*
  code passes it a bad value. There is no blame and no boundary contract — by
  choice, because that is what keeps runtime cost zero. The dNU slow-path
  ([method-lookup §2](../method-lookup.md)) is the only safety net.
- **No type-directed dispatch / overloading.** `foo(Int)` vs `foo(String)` will
  never be resolved by argument type. That is multiple dispatch and is permanently
  precluded by erasure (§9).
- **No union types in v1** (§5.12) — they require flow-narrowing, which Phalcom
  deliberately lacks.
- **No type-driven optimization in v1.** The optimizer never sees types (§5.2);
  inline caches and the sacred-selector inliner keep their deopt guards unchanged.

---

## 4. The type universe

```
                 Any            (top; every value)
                  │
              (classes & protocols, structural + nominal lattice)
                  │
               Nothing          (bottom; subtype of everything; type of None's T)

   Dynamic  (?)  — NOT in the subtype lattice; related by *consistency* only
```

| Form | Written | Meaning |
|---|---|---|
| Class type | `Point`, `String` | Instances of a class (and subclasses). |
| Protocol type | `Drawable` | Any value understanding the protocol's selectors (§5.3). |
| Generic application | `List<Int>`, `Option<T>` | Parameterized type (§5.4). |
| Block type | `[Int, Int] -> Bool` | A `Block` of that arity/result ([blocks](../blocks.md)). |
| Metaclass type | `Point class` | The class object itself (§5.6). |
| Self type | `Self` | "The receiver's own type" (§5.5). |
| Top / bottom | `Any` / `Nothing` | Universal supertype / universal subtype. |
| Dynamic | `?` | The gradual seam; consistent with every type (§5.7). |

`?` is **not** the top type. `Any` is the top of the *subtype* lattice; `?` sits
outside it and relates to other types by the gradual **consistency** relation
(Siek–Taha). Conflating them is the classic gradual-typing bug — `Any` forgets
what a value is (you may only send it universal messages), while `?` *defers* the
check (you may send it anything, checked at runtime).

---

## 5. Specification

### 5.1 Annotation syntax (sketch)

Annotations are optional and appear only at binding boundaries. They never change
parsing of the annotated construct — an annotated program with annotations deleted
must lex and parse identically.

```phalcom
let n: Int = 0
var xs: List<Int> = []

class Point {
  construct new(x: Int, y: Int) { _x = x; _y = y }

  x: Int => _x
  dist(other: Point) -> Float { ... }

  // labels are part of the type, because they are part of the selector:
  move(_: Int, to: Int, duration: Float) -> Self { ... }
}

// generic method: type parameter list before the selector
map<U>(f: [T] -> U) -> List<U> { ... }

protocol Drawable {
  draw() -> Unit
  bounds() -> Rect
}
```

`->` gives a method/block result type; `:` gives a binding/parameter type. `Unit`
is the type of a statement with no meaningful value (renders as the receiver /
`self` where relevant). The grammar reserves these positions; a parser that ignores
everything after `:`/`->` produces today's language exactly.

### 5.2 The erasure invariant (the spine)

> **E.** Deleting all type annotations from a program changes neither its bytecode
> nor its observable behavior.

E is the single invariant the whole design serves. Its consequences:

- The checker is a **pass between parse and compile**; codegen never reads an
  annotation. Erasure is therefore *by construction*, not a separate lowering step.
- Any proposed feature that would make a type annotation change a runtime value is
  **rejected**, and the semantic difference is re-expressed as a *distinct selector*
  instead (see integer division, §5.9).
- The optimizer (inline caches, sacred-selector inliner, [control-flow §3](../control-flow.md))
  runs on erased code and is unaffected. Deopt guards stay.

E is what lets the checker be developed, tested, and shipped behind a flag with
**zero risk to the runtime** — it is a linter that happens to prove things.

### 5.3 Protocols and the conformance rule

A **protocol** is a *named* set of selector → arrow-type entries. Naming (rather than
pure anonymous structural sets) avoids accidental conformance and gives diagnostics
something to print.

Conformance is checked **structurally** — a class need not declare it satisfies a
protocol (Go-interface style); it may declare it (§5.7 trusted case, and to state
intent), in which case the checker still verifies it.

> **Conformance / subtyping rule.** `S <: P` iff for every member
> `m : (a₁ … aₙ) -> r` of `P`, `S` has a method for the *same selector* `m` with
> arrow `(a₁′ … aₙ′) -> r′` where arguments are **contravariant** (`aᵢ <: aᵢ′`) and
> the result is **covariant** (`r′ <: r`). Selectors compare by the existing
> `encode_selector(name, labels)` — labels included (Inv. 2).

Subtyping is thus separated from subclassing: subclassing reuses code; protocol
conformance grants substitutability. A class may conform to protocols unrelated to
its superclass.

### 5.4 Generics, variance, bounds

Generics are **mandatory**, not a later add-on, because `Option`, `Result`, `List`,
and block types are all parametric and all in the kernel.

**Variance is declaration-site** (`out`/`in`, Kotlin/Scala style) and is a property
of *where the parameter appears*, not a blanket choice:

| Type | Variance | Why |
|---|---|---|
| `Option<out T>` | covariant | `T` only in output position (immutable). Lets the shared `None : Option<Nothing>` be a subtype of every `Option<T>` (§5.4.1). |
| `Result<out T, out E>` | covariant | immutable, both params output-only. |
| `List<out T>` | covariant | **read-only** view; no `add`. |
| `MutableList<T>` | **invariant** | has `add(_: T)` → `T` in input position → covariance would be unsound (the Java array-store hole). |
| `[in A] -> out R` (block) | contra in args, co in result | standard function variance. |

> **Correctness note.** A covariant mutable container is unsound. Any container with
> a method taking `T` must be invariant in `T`. The read-only/mutable split is the
> mechanism, not an ergonomic preference.

**Bounded type parameters** are required for generic algorithms:

```phalcom
sort<T: Comparable<T>>(xs: List<T>) -> List<T> { ... }
max<T: Comparable<T>>(a: T, b: T) -> T { ... }
```

`T: Comparable<T>` is **F-bounded polymorphism** — the bound mentions the parameter
itself. This is the same machinery as `Self` (§5.5); ship them together or neither
is usable.

#### 5.4.1 The `Nothing` bottom type

`ADR-0007`'s shared-singleton `None` is only typable with a bottom type:

```
None : Option<Nothing>          Nothing <: T   for all T
                                ⇒ Option<Nothing> <: Option<T>  (covariance)
                                ⇒ the single None value inhabits every Option<T>
```

Without `Nothing`, one `None` value could not simultaneously be an `Option<Int>`
and an `Option<String>`. `Nothing` also types `throw`/non-returning expressions and
the empty-list literal `[] : List<Nothing>`.

### 5.5 Self types

Methods that return "my own type" need a **`Self` type**, or structural subtyping is
either unsound or useless (the binary-method problem, Bruce/Cardelli):

```phalcom
class Number {
  +(other: Self) -> Self { ... }     // Int + Int : Int, not Number
}

protocol Comparable<T> {
  compareTo(other: Self) -> Ordering
}
```

`Self` in a method signature denotes the *dynamic receiver's* type, refined at each
call site: `(1).+(1)` has result `Int` even though `+` is declared on `Number`.
Arithmetic, `Comparable`, `copy`, and fluent/builder APIs all depend on it; because
they are in the bootstrap, `Self` cannot be deferred (§9).

### 5.6 Class-side and metaclass types

Classes are values (ADR-0002), so each class `C` has a **metaclass type `C class`**
carrying the class-side protocol: `construct`s, `static` methods, factories.

```phalcom
Point            : Point class
Point.new(1, 2)  : Point            // new : (Int, Int) -> Point on Point class
```

Generic instantiation flows the type arguments from the class side:

```phalcom
List<Int>()      : List<Int>        // the metaclass application carries <Int>
```

Because the metaclass tower is *parallel* (`(X class).super == (X.super) class`,
ADR-0002), class-side inheritance of `construct`/`static` types works with no extra
rule — the same conformance machinery (§5.3) applies one level up.

### 5.7 The Dynamic seam

Three graded ways code crosses into `?`, weakest guarantee last:

1. **Send to `?`.** Any selector is allowed; the result is `?`. All un-annotated
   code lives here. Formally: `?` is *consistent* with every type, so no send to a
   `?` receiver is ever a type error.
2. **Ascription `e as P`.** The programmer asserts a `?` value satisfies protocol
   `P`. **No runtime check** is inserted (erasure, E) — a pure, deliberate
   soundness surrender at a named point.
3. **Trusted conformance.** A class with a `doesNotUnderstand` handler may *declare*
   `class Proxy : Drawable` and the checker **takes its word** that dNU covers the
   protocol. This is the only way to type a forwarding proxy, and it is explicitly a
   trust boundary.

`perform(#sel)` synthesizes `?` (the selector is not statically known). Precise
typing via singleton-symbol types is a possible extension, not core.

### 5.8 Checking algorithm

**Bidirectional** (Pierce–Turner local type inference), two modes:

- **synth(e)** → infer `e`'s type bottom-up (literals, sends, references).
- **check(e, T)** → verify `e` against an expected `T` (flows types *inward*).

Key rules:

| Construct | Rule |
|---|---|
| Method body | Parameters carry declared types (check-mode entry); body checked against declared result. |
| Send `r.m(args)` | synth `r` → get its protocol → look up selector `m` → **check** each arg against the parameter type → result is the arrow's result. |
| `let x = e` | synth `e`; `let x: T = e` → check `e` against `T`. |
| Block passed to a parameter of block type | **check** the block against that block type → parameter types flow inward, so `xs.map { x => x.name }` infers `x` without annotation. |
| Generic send `xs.map<U>(f)` | **local** constraint solving for `U` only, from the argument types; no propagation across statements. |

The inward flow of expected types into blocks (row 4) is what makes a block-heavy,
control-flow-as-message language tolerable to annotate — it is why bidirectional
beats pure bottom-up inference here. HM/global inference is **not** used: subtyping +
structural types + label-encoded selectors break unification.

### 5.9 Numeric tower typing

A surface `Number > Int, Float` split lives in the **type lattice only**; the
runtime payload stays flat `f64` (ADR-0005 unchanged):

```
Number (abstract)
 ├── Int
 └── Float
```

- `1` synthesizes `Int`; `1.0` synthesizes `Float`.
- `Int + Int -> Int`; any `Float` operand widens the result to `Float`.

> **Erasure hazard — integer division.** If `Int / Int` truncated while
> `Float / Float` did not, the annotation would change the runtime result →
> **violates E.** Resolution: `/` keeps one IEEE semantics regardless of type, and
> integer (flooring) division is a **distinct selector** (`//` or `divFloor`).
> Routing the semantic difference through a different selector, not a different
> type, is the general rule whenever a type distinction wants to change behavior.

This gives [Q2](../open-questions.md) a resolution at the type layer with no boxing
and no new `Value` arm — but only if the division edge is handled as above.

### 5.10 Truthiness enforcement

The [truthiness ban](../values-and-absence.md#35-no-truthiness) is currently
enforced only at runtime (the branch protocol requires `Bool`) plus a syntactic
reject; there is no flow analysis. Types supply the missing static half:

- **Typed code:** `if` / `while` sacred forms are typed `(Bool, [] -> T) -> …`, so
  `if (opt)` is a **static type error** — "condition must be `Bool`, got
  `Option<T>`; did you mean `opt.isSome`?".
- **Untyped code:** the condition is `?` (consistent with `Bool`, so it passes the
  checker) but the **runtime `Bool`-only branch opcode still traps.** The existing
  floor covers exactly the region the checker cannot see.

The two halves compose with no overlap and no new runtime machinery — the type layer
retroactively answers an open enforcement question.

### 5.11 Blocks and non-local return

A block's type is `[A…] -> R` where `R` is its **normal** value. Non-local `return`
([blocks §5](../blocks.md)) is an **effect**, not part of the block type — modeled
like Phalcom's already-unchecked terminating errors (ADR-0008), which the type
system also does not track. The one rule: inside a block, `^expr` / `return expr` is
checked against the **home method's** declared result type (known from lexical
scope), not against `R`.

### 5.12 Why no union types (yet)

Heterogeneous returns (`Int | String`) tempt every dynamic-language type system, but
a union is only *consumable* with **flow-narrowing** (`if (x is Int) { … }`), and
Phalcom deliberately has no flow analysis (same reason the truthiness ban needs a
runtime floor). Adding unions without narrowing yields a type nothing can use.
Therefore: heterogeneous results go to `?` or to a shared protocol/supertype. This
is the *same* decision as §5.10, kept consistent.

---

## 6. Implementation

### 6.1 Where it sits

```
source ──lex──► tokens ──parse──► AST ──[TYPE CHECKER]──► AST (unchanged)
                                          │                    │
                                          └─► diagnostics       └──compile──► bytecode
                                              (miette)
```

The checker is a **read-only AST pass**. It:

1. Builds a **protocol table** per class from method signatures + declared
   conformances + class-side (`C class`) protocol.
2. Resolves annotation syntax to `Type` values (§4).
3. Runs bidirectional checking (§5.8) over each method body and top-level binding.
4. Emits diagnostics through the existing `miette` + `phalcom-common` range
   infrastructure.
5. **Discards all type information.** Nothing reaches `chunk`/`bytecode`.

### 6.2 Data structures (Rust sketch)

```rust
/// A resolved type. Erased before codegen; never stored in a Chunk.
enum Type {
    Class(ClassId, Vec<Type>),   // Point, List<Int>
    Protocol(ProtocolId, Vec<Type>),
    Meta(Box<Type>),             // `T class`
    Block(Vec<Type>, Box<Type>), // [A,B] -> R
    Var(TypeVarId),              // generic parameter / inference var
    SelfType,                    // `Self`
    Any,
    Nothing,
    Dynamic,                     // `?`
}
```

Protocol members are keyed by the **interned selector `Symbol`** — the *same* key
[`encode_selector`](../../../adr/0012-selector-signature-encoding-and-dispatch.md)
produces for dispatch — so there is one source of truth for "what message is this,"
and the checker and VM can never disagree about selector identity.

### 6.3 Reuse, not new machinery

- Selector identity: reuse `encode_selector`.
- Diagnostics: reuse `miette`/ranges.
- No new opcode, no `Value` arm, no `Chunk` field, no bootstrap change.

That the checker requires *no* runtime change is the strongest signal the shape is
right: if typing Phalcom demanded touching the VM, the design would be wrong.

---

## 7. Edge-case catalogue

| # | Case | Resolution |
|---|---|---|
| 1 | `None` used as both `Option<Int>` and `Option<String>` | `None : Option<Nothing>`, `Nothing <: T`, `Option` covariant (§5.4.1). |
| 2 | Mutable list treated covariantly | Rejected; `MutableList<T>` is invariant (§5.4). |
| 3 | `Int + Int` should be `Int`, not `Number` | `Self` type on `+` (§5.5). |
| 4 | `Int / Int` truncating under erasure | Forbidden; use `//` distinct selector (§5.9). |
| 5 | Proxy responding via `doesNotUnderstand` | Trusted `class Proxy : P` conformance (§5.7.3). |
| 6 | `perform(#sel)` result type | `?` (§5.7). |
| 7 | `if (opt)` in typed code | Static type error (§5.10). |
| 8 | `if (x)` where `x : ?` | Passes checker; runtime `Bool` floor still traps (§5.10). |
| 9 | `^expr` inside a block | Checked against the home method's result, not the block's (§5.11). |
| 10 | `List<Int>()` — where does `Int` come from? | Metaclass application on `List class` (§5.6). |
| 11 | Empty literal `[]` | `List<Nothing>`, unifies against expected element type (§5.4.1). |
| 12 | `xs.map { x => … }` block param `x` | Inferred inward from `map`'s parameter type (§5.8 row 4). |
| 13 | Heterogeneous return `Int` or `String` | `?` or shared protocol; no union (§5.12). |
| 14 | Two unrelated classes with matching selectors | Not interchangeable unless typed against a *named* protocol (§5.3). |
| 15 | `super.m()` result type | Superclass's arrow, but `Self` stays the receiver's type (§5.5). |
| 16 | Runtime `superclass=` edit ([Q4](../open-questions.md)) invalidating a checked type | Types are erased; runtime is unaffected. Checker assumes the *static* hierarchy; a doc'd limitation, not unsoundness of the runtime. |

---

## 8. Risks

| Risk | Severity | Mitigation |
|---|---|---|
| **Erasure creep.** A future feature lets an annotation change runtime behavior (e.g. int division). | High | E is an invariant; route every such difference through a distinct selector (§5.9). Add an erasure test: strip-annotations → identical bytecode, in the golden corpus. |
| **`Self` + F-bounds are hard and land late.** | High | They are prerequisites, not extras (§5.5). Building the checker without them produces a system that cannot type its own stdlib. Sequence them first. |
| **Structural diagnostics are unreadable** (the TypeScript giant-structural-type problem). | Medium | Report *selector-level diffs* ("missing `draw()`", "arg 2 expected `Bool`, got `Option<T>`"), never a full protocol dump. Message-send always gives this granularity. |
| **Optional types get relied upon, drift toward mandatory** (the Dart 1→2 migration). | Medium | Keep the non-goals (§3) explicit; never let the optimizer trust a type (§5.2). Re-evaluate soundness only as a *separate opt-in sealed subset*, never by tightening the default. |
| **Bidirectional annotation burden** feels heavy in a block-heavy language. | Medium | Inward flow into blocks (§5.8 row 4) is the primary mitigation; measure on the real corpus before adding inference. |
| **Scope creep into unions / flow-typing.** | Medium | Explicitly out (§5.12); revisit only with a flow-analysis ADR. |
| **REPL typing semantics** (per-line environments, incremental). | Low | Ship checker in batch/compile first; REPL integration is a follow-up. |

---

## 9. What this precludes (the mandatory check)

- **Argument-type overloading / multiple dispatch, forever** — types are erased and
  not in the selector, so `foo(Int)` vs `foo(String)` can never be added as an
  *extension*. Any future multi-dispatch is a *separate* first-class-type-value
  mechanism, not a growth of this system. **Decide erasure now.**
- **Type-driven unguarded optimization** — optional + unsound + mutable hierarchy
  ([Q4](../open-questions.md)) means the inliner/IC may never drop a deopt guard on
  the strength of a declared type. Reversing this needs *sound* types + `sealed`
  classes — a large, breaking commitment.
- **Sound cross-boundary guarantees / blame** — surrendered by choosing erasure over
  contracts (§3). Retrofitting them is contract insertion with real runtime cost.
- **Retrofitting `Self`/bounds after the stdlib is typed** — like default arguments,
  these must exist *before* `Comparable`/arithmetic are annotated, or every
  signature is a breaking change later.

---

## 10. Constraints summary

1. **Erasure (E)** is inviolable (§5.2). Everything else serves it.
2. Protocol members key on `encode_selector` — one selector-identity source (§6.2).
3. Variance follows parameter position; mutable containers are invariant (§5.4).
4. `Nothing`, `Self`, and bounded/F-bounded generics are prerequisites, not extras.
5. `?` is consistency-related, not the top type (§4).
6. No runtime change: no opcode, `Value` arm, `Chunk` field, or bootstrap edit (§6.3).

---

## 11. Typing-specific open questions

| # | Question |
|---|---|
| T1 | `Self` in protocol positions — full F-bounded semantics vs a restricted "receiver-only" form? |
| T2 | Do we surface `Int`/`Float` types (§5.9) *before* resolving runtime [Q2](../open-questions.md), or couple them? |
| T3 | Precise `perform`/`#symbol` singleton-symbol typing — worth the complexity? |
| T4 | Protocol conformance for kernel classes with `doesNotUnderstand`-based combinators — trusted, or hand-written signatures? |
| T5 | Checker delivery: compiler pass always-on vs `--check` flag vs separate tool? |
| T6 | Interaction with runtime hierarchy mutation ([Q4](../open-questions.md)) — document as a checker assumption, or forbid mutation in typed modules? |
| T7 | Exhaustiveness on sealed hierarchies (`Some`/`None`) — do we want it, given Phalcom dispatches rather than pattern-matches? |

---

## 12. Suggested staging

Typing is a **Phase 3+** feature; it is *not* on the current critical path
([U5/U6 control flow](../../../forge/archive/phase2/PHASE2-INDEX.md)) and should not be built until the
generic kernel it would type first — `Option`, `Result`, `List`, `Bool` — has
landed.

1. **T-0 Foundations.** `Type`, the lattice (`Any`/`Nothing`/`?`), `encode_selector`
   reuse, erasure test in the golden corpus.
2. **T-1 Monomorphic core.** Class types, protocols + conformance (§5.3),
   bidirectional checking (§5.8), truthiness enforcement (§5.10).
3. **T-2 Generics.** Parametric types, declaration-site variance, bounds, `Nothing`,
   local type-argument inference (§5.4).
4. **T-3 `Self` + F-bounds** (§5.5) — unblocks typing arithmetic/`Comparable`.
5. **T-4 Class-side / metaclass types + constructors** (§5.6).
6. **T-5 The Dynamic seam polish** — ascription, trusted conformance, `perform`
   (§5.7); numeric tower (§5.9).

---

## 13. Precedent

| Language | Took | Cost / lesson |
|---|---|---|
| **Strongtalk** | Optional, structural, sound-where-present types for Smalltalk. | The canonical "type a message-send language" answer; separates subtyping from subclassing. |
| **TypeScript** | Gradual, structural, erased, unsound (`any`). | Erasure + `?` seam here mirror it; also the source of the diagnostics-readability risk (§8). |
| **Typed Racket** | Sound gradual with boundary contracts + blame. | Runtime cost at the boundary — the reason Phalcom rejects contracts (§1.1). |
| **Kotlin / Scala** | Declaration-site variance, `Nothing` bottom, F-bounds, local inference. | Direct model for §5.4/§5.5/§5.8. |
| **Dart** | Optional (v1) → sound static (v2). | Cautionary tale (§8): optional types relied upon drift toward mandatory. Keep non-goals explicit. |
| **Go / Swift** | Structural-satisfy / nominal protocols. | Model for "named protocol, satisfied structurally" (§5.3). |
```
