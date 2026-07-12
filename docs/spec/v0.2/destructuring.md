# Destructuring `let`/`var` Bindings

Part of the [Phalcom Language Specification](README.md). Status: Draft 0.1.

**Governing ADR:** [ADR-0046](../../adr/0046-destructuring-bindings.md)
(irrefutable tuple + list destructuring, the `at(_)` accessor protocol).

`let`/`var` ([ADR-0014](../../adr/0014-let-and-var-bindings.md)) can bind more
than a single bare name — its left-hand side is a **pattern**: a name, a
tuple pattern, or a list pattern — open-question [Q7](open-questions.md).

## 1. Tuple destructuring

```phalcom
let (a, b) = point
let (q, r) = divmod(17, 5)
```

Binds each name positionally against a `Tuple`'s elements. The pattern's
arity must match the scrutinee's `size` **exactly** — `(a, b)` against a
3-`Tuple` raises a runtime error (§3).

## 2. List destructuring, with an optional rest

```phalcom
let [first, second] = pair
let [first, *rest]  = list
```

Binds each name positionally against a `List`'s elements, the same as a
tuple pattern. A trailing `*rest` (reusing U9's rest-parameter spelling
verbatim, [messages-and-selectors.md §5](messages-and-selectors.md)) collects
everything from that position onward into a fresh `List`; without `*rest`,
the pattern requires an exact-length scrutinee, same as a tuple pattern. A
rest sub-pattern **must be the pattern's last element** — `[*rest, last]` is
a parse error, mirroring [`functions.md`](functions.md)'s rest-parameter rule.

## 3. Irrefutable — a shape mismatch raises

Both forms are **irrefutable**: there is no partial bind and no boolean test.
A shape mismatch — wrong arity, or (for a `*rest` pattern) too few elements —
raises a clean `Error` at runtime rather than truncating silently. `List`'s
`at(_)` is otherwise *total* (an out-of-range read answers `None`, per
[ADR-0020](../../adr/0020-kernel-list-native-array-protocol.md)); the
destructuring lowering's own arity guard is what turns a shape mismatch into
a visible error instead of a silently `None`-padded partial bind.

There is no `match`/`if let` construct yet to receive a genuinely *refutable*
failure branch (see §5); the `Pattern` node this feature introduces is
designed so a future refutable evaluator can reuse it without reshaping it.

## 4. Nesting

Patterns nest recursively:

```phalcom
let ((a, b), c) = ((1, 2), 3)
```

A nested sub-pattern applies its own arity guard at its own level — a shape
mismatch anywhere in the tree raises at the level it occurs.

## 5. Desugaring

A destructuring binding evaluates the initializer **exactly once**, into a
compiler-internal scratch local, then reads each sub-pattern's slice through
the *same* `at(_)` selector `List`/`Tuple` already expose — no separate
accessor protocol:

```text
let (a, b) = point
```
behaves as:
```text
let $t = point
let a  = $t.at(0)
let b  = $t.at(1)
```

See [ADR-0046](../../adr/0046-destructuring-bindings.md) for the full
desugaring, the arity-guard shape, and the rest-tail construction.

## 6. `var` vs `let`

Unchanged from [ADR-0014](../../adr/0014-let-and-var-bindings.md): `let`
produces immutable leaf bindings; `var` produces mutable ones, threaded
through every leaf including nested and rest sub-patterns. A destructuring
pattern always requires an initializer, for both `let` and `var` — unlike a
bare-name `var x` (still legal, still reads `None`).

## 7. Not yet: pattern matching

A `match`/`if let` construct with genuinely refutable pattern arms, map
patterns, and guard clauses is future work — see
[Deferred & Future Work](deferred-work.md). This spec covers only the
irrefutable `let`/`var` case.
