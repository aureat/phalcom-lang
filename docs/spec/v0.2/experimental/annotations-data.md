# Annotations — `@data`/`@sealed`/`@variant` (structural records, closed hierarchies)

- Status: **Proposed** (experimental; not ratified)
- Date: 2026-07-13
- Depends on: [annotations-core.md](annotations-core.md) (the `@` desugar
  pass), [annotations-construct.md](annotations-construct.md) (field-decl
  syntax, `@construct` — `@data` composes with it), [annotations-legality-grammar.md](annotations-legality-grammar.md)
  (`Target` already carries `@data`/`@sealed`/`@variant` rows — this doc fills
  in the expansion semantics the grammar note left open)
- Related: [annotation-paradigm-bridges.md](annotation-paradigm-bridges.md)
  (Bridge A — the origin of this design, extracted into its own doc so it does
  not depend on the still-gated [decorators.md](../decorators/README.md)),
  object-model.md §8 (value types override `==`), [ADR-0011](../../../adr/0011-static-instance-slot-layout.md)
  (fixed slot layout — what makes `with(...)` cheap)

## Context

`annotation-paradigm-bridges.md`'s Bridge A sketches `@data`/`@sealed`/
`@variant` as the algebraic/functional bridge, and `decorators-stdlib.md`
gives a concrete `@data` expansion example — but both live in documents that
depend on [decorators.md](../decorators/README.md), which remains gated
([ADR-0054](../../../adr/0054-two-speed-ratification-annotation-decorator-tiers.md)).
`@data`/`@sealed`/`@variant` need no runtime hook — they are pure Compile-tier
derives, same shape as `@get`/`@set`/`@construct` — so they are extracted here
into a standalone draft that depends only on the already-ratified Compile/
Layout mechanism, clearing them for ratification independent of Install/
Dispatch/Runtime.

`annotations-legality-grammar.md`'s `Target` table already carries these rows
(ratified as part of the Compile/Layout tier):

| Attribute | Legal on |
|-----------|----------|
| `@data`, `@sealed` | Class |
| `@variant` | Class-nested variant decl |

This doc specifies what they expand to.

## Decision

### `@data` — Compile, composes with `@construct`

`@data` implies the same field-to-constructor derivation `@construct`
performs (annotations-construct.md), plus structural `==`, consistent `hash`,
a default `toString`, and a functional-update `with(...)`. If a class
declares both `@construct` and `@data`, `@data`'s generate-phase step is a
no-op for the constructor (it reuses the one `@construct` already produced —
no double-derivation, no collision, since both target the same `new`
selector via the same field list); a class declaring `@data` alone gets the
constructor derivation "for free," as part of `@data`'s own generate step.

```phalcom
@data class Money { var _cents; var _currency }
// generate phase, same field-to-param derivation as @construct:
//   construct new(cents:, currency:) { _cents = cents; _currency = currency }
// generate phase, @data's own additions:
//   ==(other)   { return _cents == other.cents and _currency == other.currency }
//   hash        => _cents.hash.combine(_currency.hash)
//   toString    => "Money(\(_cents), \(_currency))"
//   with(cents: None, currency: None) {
//     return Money.new(
//       cents:    cents.orElse { _cents },
//       currency: currency.orElse { _currency }
//     )
//   }
```

`==`/`hash` are derived **together, never one without the other** — a lone
derived `==` would break the equality ladder (two structurally-equal
instances hashing differently). `with(...)` is a **shallow** copy: it
allocates one new instance and copies the unchanged slots verbatim (ADR-0011
fixed layout makes this a slot-vector copy, not a general clone), so a
`with(...)`-produced instance shares any heap-object field values with its
source — standard functional-update semantics (Rust struct-update syntax has
the same shallow-copy shape), not a deep clone.

### `@sealed` — Compile, closes the subclass set at finalization

`@sealed` on a class freezes its subclass set: after the compiler finishes
processing the compilation unit (file/module) containing the `@sealed`
class, no further subclass may be declared. This is enforced **per
compilation unit**, not whole-program — Phalcom has no ratified cross-module
closed-world check yet (`import` semantics are open-Q8), so a `@sealed`
class's subclass set is guaranteed closed only within the file that declares
it and any subclass declared in the same file. A subclass declared in a
different module is a compile error at the subclass's own definition site
(`attr.sealed_violation`), not a silent extension of the sealed set.

`@sealed` is Phalcom's **only** exhaustiveness mechanism — there is no static
type checker to prove a `match`-style dispatch is total by any other means
(overlay: "Pattern matching / destructuring — None yet," open-Q7). Declaring
`@sealed` is what lets the generated visitor (below) be checked exhaustive at
compile time; without it, a class hierarchy is open by default and no
exhaustiveness claim can be made.

### `@variant` — Compile, one arm of a sealed hierarchy

```phalcom
@data @sealed
class Shape {
  @variant Circle(radius:)
  @variant Rect(w:, h:)
}
```

Each `@variant Name(labels...)` inside a `@sealed` class body is sugar for an
ordinary top-level class declaration, generated in the **generate** phase
before the enclosing class finalizes:

```phalcom
// @variant Circle(radius:) inside class Shape expands to:
@data
class Circle extends Shape { var _radius }
```

**Draft 0.1 scoping — no nested namespace.** `Circle` is registered as an
ordinary **global** class name (extending `Shape`, counted toward `Shape`'s
sealed set), not a name scoped under `Shape.Circle`. Phalcom has no
nested-class or namespace syntax specified yet; inventing one here would
smuggle a second, unrelated feature into this draft. This is a deliberate
Draft 0.1 simplification — a future namespacing feature can rescope variant
names without changing the sealed/exhaustiveness semantics.

### Visitor dispatch — a generated method, not new `match` grammar

`annotation-paradigm-bridges.md`'s original sketch used `shape.match { Circle(r) => ... }`
pattern-matching syntax. That syntax does not exist in Phalcom and is a
**separate, larger feature** (overlay: pattern matching/destructuring is
open-Q7, explicitly unspecified, with its own guards-vs-exhaustiveness
tension to resolve). Bundling it into this draft would exceed Compile-tier
annotation scope and require new grammar beyond `Token::At`.

Instead, `@sealed @data` generates an ordinary double-dispatch visitor method
taking one **keyword-labeled block argument per variant**, using only
existing message-send syntax — zero new grammar:

```phalcom
shape.match(circle: { c => 3.14 * c.radius * c.radius }, rect: { r => r.w * r.h })
```

```phalcom
// generated on Shape (the generate phase, once all @variant arms are known):
match(circle:, rect:) {
  return self.__matchArm(circle, rect)   // double-dispatch: overridden per variant
}
// generated on each variant (Circle, Rect):
__matchArm(circle, rect) { return circle.call(self) }   // Circle overrides to call circle.call(self); Rect calls rect.call(self)
```

The keyword-argument list is exactly the declared `@variant` names, in
declaration order — a call site omitting an arm, or naming an arm that
doesn't exist, is an ordinary missing-keyword-argument dispatch failure
(existing selector-identity machinery, ADR-0012), which is exhaustiveness
enforcement **for free**, entirely from `@sealed` fixing the arm set and
ordinary keyword-selector matching doing the rest. No new exhaustiveness
checker is needed or built.

## Hazards

- **`==`/`hash` derived together only** — already the rule; a class
  hand-writing one and deriving the other via `@data` is a compile error
  (`attr.accessor_collision`, the same collision diagnostic `@get`/`@set`/
  `@construct` already use).
- **`@sealed` cross-module gap** — the per-unit enforcement means a `@sealed`
  class's true closed-world guarantee is only as strong as Phalcom's module
  system, which is itself open (open-Q8). Documented, not silently assumed
  total.
- **`with(...)` field-order sensitivity** — like `@construct`, `with(...)`'s
  generated keyword-argument list is positionally tied to field declaration
  order; reordering fields changes `with(...)`'s call shape the same way it
  changes the constructor's (annotations-construct.md's existing R3 caveat
  extends here unchanged).

## What this precludes

Real `match`-with-patterns syntax (guards, nested destructuring, or-patterns)
is not built here and is not precluded by the keyword-argument visitor given
above — when open-Q7 resolves and true `match` syntax is designed, it can
desugar to the same generated `__matchArm` visitor this draft already
produces, so the exhaustiveness mechanism carries forward unchanged. Nested/
namespaced variant naming is deferred, not foreclosed — `Circle` staying a
global name is a Draft 0.1 simplification that a future namespace feature can
tighten without touching `@sealed`/`@variant`'s semantics.
