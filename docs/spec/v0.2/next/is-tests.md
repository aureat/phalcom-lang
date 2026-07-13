# `is` / `is!` / `is not` — kind-of and exact-instance test operators

- Status: **Proposed** (experimental; not ratified — exploratory)
- Date: 2026-07-13
- Depends on:
  [object-model.md](../object-model.md) (`isA(_)`, the metaclass tower, `class`) ·
  [method-lookup.md](../method-lookup.md) (send, `doesNotUnderstand`) ·
  [values-and-absence.md](../values-and-absence.md) (`Bool`, `not`, the branch protocol) ·
  [selectors.md](../selectors.md) (selector identity, the `#`-adjacency rule)
- Related:
  [proxy.md](proxy.md) (a proxy overrides `is` to masquerade; `isExactly` stays honest) ·
  [callables.md](callables.md) · [implicit-self.md](implicit-self.md) (keyword-desugar precedent)

## Context

Type testing today is a plain message: `x.isA(T)` (`Object#isA(_)`, landed U-CORE-1,
`.ph` over the floor — [object-model.md §8](../object-model.md)). This note adds a small
family of **surface operators** that desugar to two overridable ("magic") methods,
giving the two membership questions a language and either polarity a spelling:

- **kind-of** — does the receiver's class chain **contain** the named class? (Subclasses
  count.) The hierarchy relationship.
- **exact** — is the receiver **currently** a direct instance of the named class?
  (Subclasses do **not** count.) The live, direct-class question.

## Decision

An `is` core, with **two independent, orthogonal suffixes**:

- an optional contiguous **trailing `!`** strictifies (kind-of → exact);
- an optional trailing **`not` keyword** negates (logical `not`), Python-style.

Negation is **the word `not`**, not a leading bang. There is no `!is` / `!is!`.

| Surface | Reads | Desugars to |
|---|---|---|
| `x is T`      | x is a kind-of T         | `x.is(T)` |
| `x is! T`     | x is **exactly** a T     | `x.isExactly(T)` |
| `x is not T`  | x is **not** a kind-of T | `x.is(T).not` |
| `x is! not T` | x is **not exactly** a T | `x.isExactly(T).not` |

All four yield a `Bool`. Only **two magic methods** exist — `is(_)` and `isExactly(_)`;
negation is a compile-time `.not` wrap on the result, not a separate selector. So an
override of `is(_)` automatically governs both `is` and `is not` with consistent polarity.

`is not` carries the conventional English/Python meaning ("is not a kind-of"); *strict*
keeps its own trailing suffix `!`. The two suffixes are independent bits, so `is! not`
("not exactly") exists as their composition.

### Negation surface: the `not` keyword

`not` is the single negation concept in the language (see [values-and-absence.md](../values-and-absence.md)):

- `not x` — prefix boolean negation, an expression (lowers to `x.not`, `Bool#not`).
- `x.not` — the method form, unchanged.
- `x is not T` — the same `not` keyword as the negation particle of the is-operator.

There is **no prefix `!`**. General boolean negation is `not x`; inequality stays `!=`
(the `!=` token is unaffected — it is a distinct two-character token, not prefix `!`).

> Implementation note: landed by U-NEG. `not` is wired as the sole prefix-negation
> operator (`Token::Not` → `UnaryOp::Not` in `parse_unary`); prefix `!` (`Token::Bang`)
> is retired as an expression operator and now survives only inside the lexer's `!=`
> (`Token::BangEqual`) disambiguation. All `!x` sites in `core.ph` migrated to `not x`.

### Grammar & lexing

```
comparison := shift ( is_op "not"? shift )?
is_op      := "is" | "is!"
```

- **`is!` trailing-bang adjacency.** A `!` binds to `is` only when **contiguous** — the
  same adjacency rule selectors.md §2 uses for `#move` vs `#move (…)`:
  - `x is! T` / `x is!T` → **strict** (`is!` is the token).
  - `x is T` → kind-of.
  - There is no `is !T` reading: prefix `!` no longer exists, so a `!` after `is` is
    only ever the strict suffix (or, with a space, a lex error).
- **`not` is a compound-operator particle, not a prefix on the RHS.** When `not`
  immediately follows `is` / `is!`, it is **always** the negation particle of the
  operator — `x is not T` is `(x.is(T)).not`, never `x.is(not T)`. This mirrors Python's
  `is not` exactly. To kind-of-test against a negated *value* (rare, and nonsensical for a
  class RHS), parenthesize: `x is (not T)`.
- **Non-associative, relational precedence** — same tier as `==`/`!=`, looser than
  arithmetic, tighter than `and`/`or`.
- **Non-chaining** — `a is B is C` is a compile error (the left result is a `Bool`).
  Parenthesize to force it.
- **RHS is an expression** evaluated to a class: `x is Number`, `x is! someClassVar`. If
  the RHS is not a class, the test simply returns `false` — the chain walk never matches a
  non-class value (I-4, resolved: `false`, not raise).
- **LHS is any expression.**

### The magic methods (`core.ph`, over the floor)

```phalcom
class Object {
  // kind-of: walk the receiver's class chain; true if `cls` is reached.
  // This IS the existing isA(_) semantics — isA becomes an alias (below).
  // No RHS-is-a-class guard: a non-class `cls` simply never matches any `c`
  // in the chain, so the walk returns false (I-4). A guard cannot live here —
  // `cls.isA(…)` would re-enter `is` through the alias and recurse forever.
  is(cls) {
    var c = self.class
    while (c != None) {
      (c == cls).ifTrue { return true }
      c = c.superclass          // None at the root terminates the walk
    }
    return false
  }

  // exact: the receiver's CURRENT direct class, identity-compared. No chain walk.
  isExactly(cls) => self.class == cls

  // retained name for the landed kernel method; now an alias over `is`.
  isA(cls) => self.is(cls)
}
```

`is(_)` subsumes the landed `isA(_)`; `isA(_)` stays as an alias so U-CORE-1 code and
fixtures (`3.isA(Number)`) keep working. New code uses the operators; `isA` is not
deprecated, just no longer canonical.

Negation is not a method: the compiler lowers a trailing `not` particle to `.not` on the
`Bool` result (`Bool#not`, core.ph). No `isNot` selector exists.

### Semantics — the kind-of / exact split

Sharpest on classes themselves (classes are objects; their direct class is their
metaclass, [object-model.md](../object-model.md)):

```phalcom
3 is Number            // true  — Int chain reaches Number
3 is! Number           // false — 3's direct class is Int, not Number
3 is! Int              // true  — 3's direct class IS Int
3 is not Number        // false — negation of kind-of (3 IS a kind-of Number)
3 is! not Number       // true  — 3 is not *exactly* a Number

class Dog extends Animal {}
let d = Dog.new
d is Animal            // true  — kind-of
d is! Animal           // false — exact class is Dog, not Animal
d is not Animal        // false — "is not a kind-of Animal" — but it IS
d is! Dog              // true
d is! not Dog          // false — it IS exactly a Dog

Point is Class         // true  — a class is a kind-of Class
Point is! Class        // false — Point's direct class is its metaclass, not Class itself
```

"**Currently**" is load-bearing: `isExactly` reads the *live* `self.class`. If the object
model admits `become:` / conditional re-parent (U-CORE-3), an object's direct class can
change at runtime, and `is!` reflects the class *right now*, not at construction. `is`
likewise re-walks the *current* chain on every call — neither result is cached across a
reparent.

### Override semantics (why they are "magic")

Because both cores are sends, a class controls its own membership answers, and negation
follows automatically:

```phalcom
// Protocol-based membership: answer `is Drawable` structurally.
class Shape {
  is(cls) { (cls == Drawable).ifTrue { return true };  return super.is(cls) }
}
// now `s is Drawable` → true and `s is not Drawable` → false, for free.

// Proxy masquerade (proxy.md P-2): a transparent wrapper claims its target's kind,
// so it substitutes — but a boundary proxy stays HONEST about exact identity.
class Trace : Proxy {
  is(cls)        => _target.is(cls)        // kind-of transparent: `view is Order` → true
  isExactly(cls) => _target.isExactly(cls) // a Capability would NOT forward this
}
```

Policy (mirrors [proxy.md P-2](proxy.md)): a *transparency* proxy (`Trace`, `Lazy`)
forwards **both** so it fully substitutes; a *boundary* proxy (`Capability` membrane)
forwards `is` (passes kind-of checks) but answers `isExactly` **as itself** — the
boundary genuinely *is* a different identity, and security code that must not be fooled
tests with `is!` / `is! not`.

## Interaction hazards

- **`is!` reads as an assertion, not a test.** A trailing `!` in many languages (Swift
  force-unwrap, Ruby bang-methods) connotes "do it or blow up." Here it means *strict
  equality of class*, and the whole expression is still a pure `Bool` test — it never
  raises on a false result. Document that `is!` is a **predicate**, not a forced cast; it
  never raises — a non-class RHS returns `false` (I-4), same as `is`.
- **`is not` vs `is (not …)`.** Because `not` is also the prefix boolean-negation
  keyword, `x is not T` is disambiguated by fiat: a `not` directly after `is`/`is!` is the
  operator particle (Python's `is not`), never a prefix on the RHS. This is the one place
  the unified `not` keyword needs a parser rule; it is a fixed, well-precedented rule, not
  a per-site judgement. (Fork B — see open question I-7 — dodges this by making `not`
  *only* an is-particle and keeping `!x` for boolean negation.)
- **Truthiness stays clean** (enforcement-without-static-analysis). All four forms return
  `Bool` and nothing else, so `if (x is T)` feeds the branch protocol a `Bool` — no
  Option/nil reaches the branch. Consistent with the no-truthiness floor
  ([values-and-absence.md](../values-and-absence.md)).
- **Inline cache ⊗ mutable hierarchy.** `is`/`isExactly` read `self.class` and the
  superchain live; a reparent/`become:` bumps the class generation so any IC on the
  `is(_)`/`isExactly(_)` send invalidates — the discipline every send already needs. The
  *result* is never cached; recomputed per call.
- **Override ⊗ security.** `is` being overridable means a hostile object can *claim*
  `is Number`. Code guarding a trust boundary uses `is!` / `is! not` (exact, and by policy
  un-forwarded across a membrane), or checks the raw target before wrapping. `isExactly`'s
  default (`self.class == cls`) cannot be spoofed without overriding it, and a boundary
  proxy is specified not to forward it.

## What this precludes

- **`is` and `not` as identifiers.** `is` becomes a reserved keyword; `let is = …` and a
  parameter named `is` are illegal. `not` is *already* a reserved token and now does
  double duty (prefix boolean negation + `is not` particle), so it too cannot be an
  identifier. The *methods* `.is(_)` / `.isExactly(_)` are still callable in dot form and
  as `#is(_)` / `#isExactly(_)`; only the bareword operator positions are reserved.
- **Prefix `!` as boolean negation.** Retired in favour of `not x`. `!=` survives as its
  own token. This is the single negation concept the language commits to.
- **Smart-casting / flow narrowing.** In statically-typed languages `x is T` narrows `x`
  in the then-branch (Kotlin smart cast). Phalcom is dynamic: the operators return a
  `Bool`, bind nothing, narrow nothing. A future typed layer could add narrowing; these
  operators do not presuppose it.
- **A separate negation selector.** Negation is compile-time `.not`, so there is no
  `isNot(_)` magic method to override or get out of sync with `is(_)`. Overriding `is`
  governs `is not` by construction — this is foreclosed from drifting.
- **A single fused "instanceof".** Kind-of and exact stay distinct operators; the caller
  chooses subclass-inclusive vs exact at the call site.

## Examples

```phalcom
// dispatch on kind
draw(shape) {
  (shape is Circle).ifTrue { return self.drawCircle(shape) }
  (shape is Polygon).ifTrue { return self.drawPolygon(shape) }
  UnsupportedShape.new(shape).raise()
}

// exact-class fast path (skip the subclass-general branch)
render(node) {
  (node is! TextNode).ifTrue { return node.text }   // exactly TextNode, no subclass hook
  return self.renderGeneric(node)
}

// negation reads naturally
guard(x) { (x is not Number).ifTrue { TypeError.new(need: Number).raise() } }

// "a T but not the base T itself"
onlySubclass(x) { return x is Widget and x is! not Widget }  // kind-of Widget, not exactly Widget
```

## Open questions

| # | Question |
|---|---|
| I-2 | Should `is` fall through to `super.is(cls)` by default so overrides compose (shown), or is the default `is` final and non-overridable-for-lying, with only proxies granted the hook? |
| I-3 | Does `isExactly` compare by class **identity** (`==`) or by class **name** (to survive class reloading / image migration where a class object is re-created)? Identity is stricter; name survives reload. |
| I-4 | *(closed — `false`)* A non-class RHS returns `false`: the chain walk never matches a non-class value, so this needs no guard. A raising variant would need a **non-recursive** native class-predicate (a plain guard `cls.isA(…)` re-enters `is` via the alias and recurses forever) — deferred until such a primitive exists; it would also break U-IS's floor-0. Cost accepted: `x is 3` typos read as `false` rather than erroring. |
| I-5 | Do `is!` / `is! not` earn their keep, or ship only `is` + `is not` (kind-of ± negation) and leave *exact* to the method `x.isExactly(T)`? The two-suffix grammar is orthogonal but the strict form is rarely reached for. |
| I-6 | Interaction with `match`/destructuring ([destructuring.md](../destructuring.md)): may a pattern arm use `is T` as a guard, and if so does it bind/narrow, or stay a plain `Bool` guard? |
| I-7 | *(closed — Fork A)* Negation is the single `not` keyword: `not x`, `x.not`, and the `is not` particle all share it; prefix `!` retires (`!=` survives); `x is not T` uses the compound-operator disambiguation rule above. The alternative (keep `!x`, make `not` only an is-particle) is rejected — one negation concept over two spellings. |
