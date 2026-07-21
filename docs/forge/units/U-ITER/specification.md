# U-ITER — Specification: cursor iteration + `for` / `break` / `continue`

> **Status:** Normative surface (deepens the ratified spec). This document is the
> unit-scoped, full-detail specification for U-ITER. It **extends** the already-normative
> [`iteration.md`](../../../spec/current/iteration.md) (ratified by
> [ADR-0035](../../../adr/0035-iteration-protocol-cursor.md)) — that document stays the
> terse normative index; this one adds the surface grammar, the exact operational
> desugars at the bytecode level, the loop-control lowering, the error surface,
> cross-feature interactions (chiefly the `for`-generator seam with
> [[U-FIBER]](../U-FIBER/specification.md)), worked examples, and machine-checkable
> conformance points. **Nothing here overrides a ratified claim.** Every normative
> statement cites its governing ADR §/spec §; where this deepens the index, it says so.
>
> **Governing sources.** [ADR-0035](../../../adr/0035-iteration-protocol-cursor.md)
> (the protocol + the three lowering rules); [`iteration.md`](../../../spec/current/iteration.md)
> §1–§7 (the promoted spec); [ADR-0018](../../../adr/0018-sacred-selector-inliner-and-override-guard.md)
> (the sacred-selector inliner the loop scaffold rides on);
> [ADR-0007](../../../adr/0007-option-as-abstract-with-some-none.md) (`Option` as the
> "more?" signal); [ADR-0020](../../../adr/0020-kernel-list-native-array-protocol.md)
> (`List` as the reference iterable);
> [ADR-0030](../../../adr/0030-fibers-and-futures-cooperative-concurrency.md) §4 (the
> restricted-yield model that the `for` lowering must serve — the load-bearing
> preclusion check).
>
> **Ratified constraints honored verbatim (2026-07-12).** `for` lowers to an inlined
> cursor `while`, **never** to `.each` ([ADR-0035](../../../adr/0035-iteration-protocol-cursor.md)
> §2). `iterate(_)`/`iteratorValue(_)` are **ordinary sends, never inlined**
> (ADR-0035 §4). Combinator migration onto the protocol is **out of scope** —
> **DEC-ITER-A resolved to a U-STD follow-on** (ADR-0035 §5). Zero new floor
> primitives (ADR-0035 §Consequences).

---

## 1. Surface syntax and grammar

Three loop-control forms enter the surface. Their keyword tokens **already lex**
(`token.rs` `While`/`For`/`Break`/`Continue`/`In`; `lexer.rs:256-263`) but only `while`
is parsed today (§ implementation-spec §1); U-ITER adds the `for`/`break`/`continue`
productions.

### 1.1 Grammar (added productions)

```ebnf
statement   ::= … | for-stmt | break-stmt | continue-stmt

for-stmt    ::= "for" "(" binding "in" expr ")" brace-block
binding     ::= IDENT                            (* the loop variable *)

break-stmt      ::= "break"
continue-stmt   ::= "continue"
```

- **`in` is a contextual keyword** — special *only* in the `for (… in …)` header
  (**DEC-ITER-B → (A) contextual**, plan §8). It does not become a reserved word, so
  an existing identifier `in` elsewhere keeps working. (The `Token::In` variant exists;
  the parser consumes it only inside `parse_for`.)
- **`break`/`continue` take no operand** and no label (unlabelled forms only; a
  labelled-break future is left open, §10).
- The loop variable `binding` is a fresh immutable binding in the loop body's scope,
  re-bound each iteration (it behaves as `let x` per step, matching the desugar in §3).

### 1.2 Where the forms are legal

- `for`/`while` are **expressions that are used as statements** — consistent with the
  existing `while`, which parses to an `Expr` (`parser.rs:1338`). Their value is
  unspecified/`None`; they are consumed for effect. (Iteration is not an
  expression-producing comprehension in v0.2; see §10.)
- `break`/`continue` are **statements legal only inside a lexically-enclosing
  `for`/`while` body.** Outside any loop they are a **compile error** (§6, §9 C-ITER-7).

---

## 2. The cursor protocol (deepened)

*Deepens [`iteration.md`](../../../spec/current/iteration.md) §1.* A value is **iterable**
iff it answers two ordinary selectors:

| Selector | Given | Returns | Notes |
|---|---|---|---|
| `iterate(_)` | the previous cursor, or `None` to start | `Some(nextCursor)`, or `None` when exhausted | The **only** "more?" channel. |
| `iteratorValue(_)` | a cursor | the element at that cursor | Called only with a cursor extracted from a `Some` returned by `iterate`. |

### 2.1 Contract obligations (normative, ADR-0035 §1)

1. **The cursor is an ordinary value** — no iterator object is allocated. For `List`
   it is an integer index; for a tree it may be a node handle; for a user type it is
   whatever the two methods agree on.
2. **`None` is the exhaustion signal** (ADR-0007). A conforming `iterate(_)` returns
   `Some(cursor)` while elements remain and `None` at the end. Because the "more?" bit
   rides in `Option`, **no surface `nil` ever appears** ([Invariant 4](../../../spec/current/README.md)).
3. **`iteratorValue(_)` is total on live cursors** — it is only ever called with a
   cursor that `iterate(_)` just wrapped in `Some`, never with `None` and never with a
   past-the-end cursor. A type need not defend against out-of-range cursors *from the
   `for` desugar*; a direct caller that violates this gets that type's own out-of-range
   behavior (e.g. `List#at` bounds handling).
4. **Purity is not required, but re-entrancy must be safe** — `for` calls `iterate`
   then `iteratorValue` once each per step (§3); a type whose cursor advance has side
   effects sees exactly that call pattern.

### 2.2 The reference iterable — `List` (ADR-0020, ADR-0035 §Consequences)

`List` is the reference implementation. Its cursor is the integer index; the two
selectors are written **purely in `.ph` over the existing `size`/`at(_)` floor**, adding
**zero** primitives:

```phalcom
iterate(cursor) {
  let next = cursor.map { c => c + 1 }.unwrapOr(0)   // None → 0; Some(i) → i+1
  return (next < self.size).ifTrue { Some(next) }.ifNone { None }
}
iteratorValue(cursor) => self.at(cursor)
```

(Exact `Option`/`ifTrue`/`ifNone` spellings are pinned against the landed U6/U-CORE-2
surface in the implementation-spec §3.1.)

### 2.3 User iterables

A user type opts into `for` — and, once the U-STD follow-on lands, every combinator —
by defining the two selectors. The canonical example (from
[`iteration.md`](../../../spec/current/iteration.md) §1) drives `for` through nothing but
its own two `.ph` methods:

```phalcom
class Countdown {
  construct from(n:) { _n = n }
  iterate(cursor) {
    let next = cursor.map { c => c - 1 }.unwrapOr(_n)
    return (next >= 0).ifTrue { Some(next) }.ifFalse { None }
  }
  iteratorValue(cursor) => cursor
}
for (x in Countdown.from(n: 3)) { System.print(x) }   // 3 2 1 0
```

---

## 3. Operational semantics — the exact desugar

*Deepens [`iteration.md`](../../../spec/current/iteration.md) §2 to the bytecode level.*

### 3.1 `for` without `break`/`continue` — the plain cursor `while` (ADR-0035 §2)

`for (x in coll) { body }` is defined to mean, evaluating `coll` **exactly once** into a
fresh temporary:

```phalcom
let _coll = coll                 // evaluate the iterable expression once
var _c   = _coll.iterate(None)
while (_c.isSome) {
  let x = _coll.iteratorValue(_c.unwrap)
  body
  _c = _coll.iterate(_c)
}
```

**This supersedes the `for ≡ coll.each{…}` sketch in
[`control-flow.md`](../../../spec/current/control-flow.md) §1** (ADR-0035 §2).

**Lowering.** The `while` scaffold and the `isSome` test are realized as the *inlined
`while` skeleton* — the same `Jump`/`JumpIfFalse`/`Loop` jump structure the sacred
inliner emits for `whileTrue` (ADR-0018; `inliner.rs` `compile_while_true`). The two
protocol calls `iterate(_)` and `iteratorValue(_)`, and `isSome`/`unwrap` on the cursor,
are **ordinary sends inside that skeleton** (ADR-0035 §4). `iterate`/`iteratorValue` are
**never inlined** — a user type's implementations must be reachable by normal dispatch.
`isSome` is inliner-eligible (iteration.md §4) but its inlining is not required for
correctness.

> **The single load-bearing property:** the emitted `for` scaffold contains **no
> `block_call` / `call(_)` on the taken (protocol) path** — the body runs under jumps,
> not under a native combinator callback. This is what lets a `for` inside a fiber body
> suspend freely (§7.1, the [[U-FIBER]](../U-FIBER/specification.md) seam). Lowering
> `for` through `.each` — which *does* interpose `f.call(x)` → `block_call` → a native
> Rust frame ([[U-FIBER]](../U-FIBER/specification.md#the-crown-jewel), ADR-0033 Context)
> — is therefore **forbidden**, not merely discouraged.

### 3.2 `for` / `while` *containing* `break`/`continue` — the dedicated jump loop (ADR-0035 §3)

A `for`/`while` whose body lexically contains a `break` or `continue` (at this loop's
level) compiles to a **direct jump-based loop** — condition, body, a `break` target at
the loop exit, a `continue` target at the step — **bypassing the overridable
`whileTrue(_)` send entirely**:

```
        <eval iterable once → _coll>          ; for only
        _c = _coll.iterate(None)              ; for only
loop:   <cond>                                ; for: _c.isSome ; while: the condition
        JumpIfFalse → end
        <bind x = _coll.iteratorValue(_c.unwrap)>   ; for only
        <body>                                ;   break    → Jump end
                                              ;   continue → Jump step
step:   _c = _coll.iterate(_c)                ; for: the cursor advance
        Jump → loop
end:
```

**Why bypass `whileTrue` here** (ADR-0035 §3): the inlined `whileTrue` carries an
override-epoch **deopt fallback** (`GuardBlock` → a real `whileTrue(_)` send). If a
`break`/`continue` jump targeted a label inside the inlined fast path and the guard then
deopted to the fallback send, the jump targets would be invalid. Emitting the loop
directly means **there is no deopt path to fall back to** — the jump targets are always
valid. `continue` re-runs the cursor's `iterate(_)` step (jumps to `step:`, not to
`loop:`), so the loop variable advances correctly.

- **`continue`** → unconditional `Jump` to the innermost loop's `step:` label. For a
  `for`, `step:` is the `_c = _coll.iterate(_c)` advance, so the next `iterate` runs;
  for a bare `while`, `step:` is the condition re-test.
- **`break`** → unconditional `Jump` to the innermost loop's `end:` label.

Both reuse the existing unconditional `Bytecode::Jump(i32)` (bytecode.rs:130) — **no new
opcode** (DEC-ITER-C resolved against HEAD, implementation-spec §1/§6).

### 3.3 Evaluation and scoping guarantees

- **`coll` is evaluated once** — bound to `_coll` before the loop; a side-effecting
  iterable expression runs exactly one time (§9 C-ITER-3).
- **Empty iterable** — if `_coll.iterate(None)` is `None`, the body runs **zero times**
  (§9 C-ITER-2).
- **Loop variable freshness** — `x` is re-bound each step; a closure captured in the
  body over `x` observes that step's value (consistent with block-closure capture,
  [blocks.md](../../../spec/current/blocks.md)).
- **Nesting** — `break`/`continue` bind to the **innermost** enclosing loop (§4).

---

## 4. `break` / `continue` and the loop-context stack

*Deepens [`iteration.md`](../../../spec/current/iteration.md) §3.*

`break`/`continue` are **loop-control keywords, not sends and not floor primitives**
(ADR-0035 §3). Their resolution is a purely lexical, compile-time affair:

- The compiler maintains a **loop-context stack**. Entering a `for`/`while` body pushes a
  frame carrying that loop's `end:` and `step:` label slots; leaving it pops the frame.
- `break` resolves to the top frame's `end:`; `continue` to the top frame's `step:`.
- **Innermost binding** falls out of "top of stack" — in nested loops, an inner
  `break`/`continue` leaves/steps only the inner loop (§9 C-ITER-6).
- A `break`/`continue` compiled with an **empty loop-context stack** is a **compile
  error** with a span at the offending keyword (§6, §9 C-ITER-7). Because this is a
  compile-time check, no runtime `break`/`continue` value or opcode exists.

**Interaction with non-local `return`.** `break`/`continue` are *not* `return`. A `return`
inside a `for`/`while` body still compiles to `Bytecode::Return`/`ReturnNonLocal`
([blocks.md](../../../spec/current/blocks.md) §5) and unwinds the enclosing **method/block**,
not just the loop. `break` leaves only the loop. This distinction is exactly why `for`
lowers to a jump-`while` and not to `coll.each { … }`: a block handed to `.each` could
only express early exit as a non-local `return` (which would leave the whole method), not
as `break` (ADR-0035 §2, Alternatives).

---

## 5. Dispatch and the inliner (deepened)

*Deepens [`iteration.md`](../../../spec/current/iteration.md) §4; grounded in
[ADR-0018](../../../adr/0018-sacred-selector-inliner-and-override-guard.md).*

| Element of a `for` loop | Inlined? | Mechanism |
|---|---|---|
| the `while` skeleton (condition test + back-edge) | **yes** | `Jump`/`JumpIfFalse`/`Loop` (the `whileTrue` skeleton, or emitted directly for the break/continue form) |
| `Option#isSome` on the cursor | eligible | inliner may lower to a jump; not required |
| `iterate(_)` | **no** | ordinary send — type-specific, overridable |
| `iteratorValue(_)` | **no** | ordinary send — type-specific, overridable |
| `unwrap` on the cursor `Some` | ordinary send | not a control selector |

**Why `iterate`/`iteratorValue` must stay non-inlined** (ADR-0035 §4): they are open,
type-specific methods ([ADR-0026](../../../adr/0026-class-hierarchy-mutability.md)).
Inlining them would freeze `List`'s implementation into every `for` call-site and break
the "a user type opts in by defining two methods" contract — `Countdown` (§2.3) would no
longer drive `for`. Only the *fixed* `Bool`/`Block` control selectors are sacred and
inlinable (ADR-0018); the iteration protocol selectors are deliberately not.

A `for` loop is therefore, at runtime: **an inlined `while` skeleton driving two regular
protocol sends per step** — cheap control flow, fully generic iteration.

---

## 6. Error surface

Iteration is defined entirely over ordinary sends + compile-time loop control, so its
error surface is small and inherited:

| Situation | Result | Where |
|---|---|---|
| `break`/`continue` outside any loop | **Compile error** (span at the keyword) | compiler loop-context stack (§4) |
| receiver of `for` does not answer `iterate(_)` | ordinary **`doesNotUnderstand`** miss → surface `MessageNotUnderstood` (U-CORE-6) at the first `_coll.iterate(None)` send | dispatch, [error surface of U-CORE-6](../U-CORE-6/as-built.md) |
| `iterate(_)` returns a non-`Option` | whatever `.isSome`/`.unwrap` do to that value — typically a further `doesNotUnderstand` miss | dispatch |
| `iteratorValue(_)` raises | the error propagates out of the loop through the ordinary unwind (`RuntimeError::Raise`, U-CORE-6) | unified unwind |
| a fiber body's `for` yields under a **combinator callback** (`.each { yield }`, not `for`) | **`CannotYieldAcrossNativeFrame`** — but this is the combinator's callback, not `for` itself (§7.1) | [[U-FIBER]](../U-FIBER/specification.md#the-crown-jewel) |

`for` introduces **no new error type**. `break`/`continue` misuse is caught at compile
time, never at runtime.

---

## 7. Cross-feature interactions

### 7.1 `for` ⊗ the `Fiber` generator — the load-bearing seam {#fiber-generator-seam}

*Cross-links [[U-FIBER §4.3]](../U-FIBER/specification.md#yield-guard)
and [`iteration.md`](../../../spec/current/iteration.md) §6;
[ADR-0030](../../../adr/0030-fibers-and-futures-cooperative-concurrency.md) §4,
[ADR-0035](../../../adr/0035-iteration-protocol-cursor.md) §5,
[ADR-0033](../../../adr/0033-amend-fiber-execution-trampolined-block-callsite.md) Context.*

The cursor protocol needs **no** `Fiber` (ADR-0035 §5). But the *reason* `for` lowers to
an inlined `while` rather than to `.each` is precisely to serve the restricted-yield
model of the (later) `Fiber` unit:

```phalcom
Fiber.new { for (x in coll) { Fiber.yield(x) } }   // ✅ suspends freely — inlined while
Fiber.new { coll.each { x => Fiber.yield(x) } }    // ✗ CannotYieldAcrossNativeFrame
```

- The `for` body runs under **jumps** (§3), so between the fiber floor and the
  `Fiber.yield` send there is **no native Rust frame** — the yield integrates with the
  top-level `run_until` and suspends ([[U-FIBER §4.3]](../U-FIBER/specification.md#yield-guard)).
- `.each { yield }` interposes `each`'s `f.call(x)` → the `block_call` primitive → a
  **re-entrant `run_until`** (a native frame), so the yield raises
  `CannotYieldAcrossNativeFrame` ([[U-FIBER]](../U-FIBER/specification.md), ADR-0030 §4).
- **`for` is the idiomatic v0.2 generator body** and supersedes the older
  "rewrite with index iteration" advice ([`concurrency.md`](../../../spec/current/concurrency.md)
  §1). It also gives `break`/`continue`, which a block handed to `.each` cannot express.

This interaction is **served, not precluded**, by U-ITER; it is verified by a PENDING
cross-unit fixture that graduates when [[U-FIBER]](../U-FIBER/specification.md) lands
(§9 C-ITER-8, implementation-spec §4). The residual `.each { yield }` lift is the
**Deferred** [ADR-0033](../../../adr/0033-amend-fiber-execution-trampolined-block-callsite.md)
— orthogonal to U-ITER, which adds no block-call path.

### 7.2 `for` ⊗ combinators (`.each`/`.map`/`.filter`/`.reduce`)

Both mechanisms are **correct in parallel**. `for` is the **loop-control** form
(supports `break`/`continue`); `.each` is the **full-traversal** form (no early exit).
Today the combinators sit on `size`/`at` directly (core.ph); ADR-0035 §5 wants them
rewritten as `.ph` defaults over `iterate`/`iteratorValue`. **That migration is
explicitly a U-STD follow-on (DEC-ITER-A), not part of U-ITER** — U-ITER's only `core.ph`
edit is adding `List#iterate`/`iteratorValue`. Until the follow-on lands, `.each` etc.
keep their current definitions and remain fully functional.

### 7.3 `for` ⊗ other collections (`Map`/`Set`/`Tuple`/`Range`)

Not precluded and not delivered here. Each conforms **later** by implementing the two
selectors; `for` and every combinator then fall out with **no further compiler work**
(ADR-0035 §Consequences). U-ITER ships only `List#iterate`/`iteratorValue` as the
reference.

---

## 8. Worked examples

### 8.1 `List` traversal, order and empty case

```phalcom
for (x in [10, 20, 30]) { System.print(x) }   // 10 20 30
for (x in []) { System.print("never") }        // (no output; body runs 0 times)
```

### 8.2 `break` and `continue`

```phalcom
for (x in xs) {
  if (x < 0)   { continue }     // skip negatives → re-runs iterate for the next x
  if (x > 100) { break }        // stop at the first > 100
  process(x)
}
```

### 8.3 Nested loops — innermost binding

```phalcom
for (row in rows) {
  for (cell in row) {
    if (cell.isBlank) { continue }   // continues the INNER loop only
    if (cell.isStop)  { break }      // breaks the INNER loop only
    emit(cell)
  }
}
```

### 8.4 User iterable driving `for` (non-`List` proof)

See `Countdown` (§2.3): `for (x in Countdown.from(n: 3))` prints `3 2 1 0`, driven purely
through the two `.ph` selectors — proving `iterate`/`iteratorValue` stay non-inlined and
overridable (§5).

### 8.5 `for` as a fiber generator (graduates with U-FIBER)

```phalcom
let g = Fiber.new { for (x in [1, 2, 3]) { Fiber.yield(x) } }
g.call()   // 1
g.call()   // 2
g.call()   // 3
```

---

## 9. Conformance points (machine-checkable)

| ID | Requirement | How verified |
|---|---|---|
| **C-ITER-1** | `for` over `[10,20,30]` visits `10,20,30` in order. | golden |
| **C-ITER-2** | `for` over `[]` runs the body zero times. | golden |
| **C-ITER-3** | The iterable expression is evaluated **once** (side-effecting-receiver golden). | golden |
| **C-ITER-4** | A `for` chunk's taken path emits `Jump`/`Loop` + `iterate`/`iteratorValue`/`isSome` sends and **no `call(_)`/`block_call`** (the §7.1 guard). | disasm golden |
| **C-ITER-5** | `Countdown` (§2.3) drives `for` purely via its two `.ph` selectors — `iterate`/`iteratorValue` are not inlined. | golden |
| **C-ITER-6** | In nested loops, `break`/`continue` bind to the **innermost** loop. | golden |
| **C-ITER-7** | `break`/`continue` outside any loop is a **compile error** with a clear span. | negative |
| **C-ITER-8** | *(PENDING → graduates with [[U-FIBER]](../U-FIBER/specification.md))* `Fiber.new { for (x in [1,2,3]) { Fiber.yield(x) } }` suspends and yields `1,2,3`. | pending golden |
| **C-ITER-9** | Protocol round-trips: `[7,8].iterate(None) == Some(0)`; `.iterate(Some(0)) == Some(1)`; past-end `== None`; `.iteratorValue(0) == at(0)`. | golden |
| **C-ITER-10** | **Net floor delta = 0** — no new `(class, selector)` binding, no new primitive fn. | census audit |

---

## 10. Non-goals and reserved shapes

- **External (pull) iterators / `Stream`** as the primitive — rejected (ADR-0035
  §Alternatives): this protocol is internal/cursor-based; a lazy `Stream` layer, if ever
  wanted, builds on [[U-FIBER]](../U-FIBER/specification.md), not on this protocol.
- **Mutation-during-iteration safety** — not guaranteed by the protocol; a collection
  may later add fail-fast via a modification counter (a separate decision,
  iteration.md §7).
- **Comprehensions / `for` as a value-producing expression** — not in v0.2; `for` is a
  statement consumed for effect (§1.2).
- **`for`-`else` and labelled `break`** — not shipped; the **loop-context stack (§4) is
  the natural extension point** for both, so neither is precluded.
- **Combinator migration onto the protocol** — **DEC-ITER-A: U-STD follow-on**, not this
  unit (§7.2).
