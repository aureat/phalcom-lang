# Derive decorators — `@data`, `@get`/`@set`, `@sealed`/`@variant` (+ `@construct` residue)

- Status: **Built (U-ANNOT-LAYOUT)**, with three defects (DEF-7, DEF-8, DEF-9)
  and one scheduled deletion (`@construct` → [placement.md](placement.md)).
  As-built truth: [data.md](../v0.2/decorators/data.md),
  [accessors.md](../v0.2/decorators/accessors.md),
  [sealed.md](../v0.2/decorators/sealed.md),
  [construct.md](../v0.2/decorators/construct.md).
- Tier: Compile / generate. All derives emit ordinary members through the
  normal selector-encoding path — philosophy check 2 (selector identity)
  passes by construction, and every generated body is sends-on-the-floor
  (`ifFalse`, `at(_)`, `+`), no hidden primitives.

## Verified design summary

- **`@data`** derives constructor (unless a `new` constructor exists), getter
  backfill, `==`/`hash` (together or not at all — the equality-ladder
  invariant, enforced as `attr.accessor_collision` when exactly one is
  hand-written), `toString`, shallow `with(...)`. The three documented
  divergences from the draft pseudocode (folded `*31+` hash, `+`-chain
  toString, ternary-shaped `with`) are all consequences of real floor limits
  (no `hash.combine`, no interp node at codegen, `orElse` needs an `Option`)
  — verified sound, keep.
- **`@get`/`@set`** derive accessor pair from a field; sibling-member
  emission forces driver-level implementation (the expander trait can't
  append members) — same structural honesty as the subtractive pair.
- **`@sealed`/`@variant`** derive sibling variant classes (each `@data`,
  `extends` the sealed root) plus a keyword-labeled visitor
  (`shape.match(circle: {...}, rect: {...})`) whose exhaustiveness falls out
  of ADR-0012 keyword-selector dispatch — a missing arm is a missing keyword,
  an ordinary dispatch failure. This is the correct Phalcom answer to
  exhaustiveness-without-a-type-checker, and any future real `match` syntax
  must desugar to this visitor rather than reinvent it (open-Q7 residue).

## Defects and fix plans

### `@get(priv)` (DEF-7)

The argument parses and does nothing — an API that accepts and ignores.
Two honest options: enforce it (derive a `_`-prefixed/module-private getter
once `_`-privacy is enforced — but ADR-0045 records `_`-privacy itself as
specified-unenforced), or reject the argument until it can mean something.
**Recommendation: reject now** via mechanism.md Plan §4's `AttrArity`
(`@get` declares no-args until privacy enforcement exists), and record
`(priv)` as the reserved future form. An ignored argument is worse than a
rejected one; this is the same reasoning that made unknown attributes a hard
error.

### `@set` on `const` fields (DEF-8)

Post-U-BINDINGS, `const _x` + `@set` derives a setter whose write either
trips `field.const_write` pointing at code the user never wrote, or bypasses
the syntactic const enforcement entirely (the check is
constructor-flag-based). Fix at derive time: `derive_accessors` consults
`FieldDef.mutable`; `@set` on an immutable field ⇒ new error
`attr.set_on_const` (same family as `attr.accessor_collision`). `@get` on
`const` remains legal. Fixture pair, positive and negative. This is the
deferred-doc's own recommendation, adopted — it keeps const enforcement
syntactic (no flow analysis), which is the ADR-0064 posture.

### `@sealed` reachability (DEF-9)

`attr.sealed_violation` is dead code for user classes: `extends` can't name
an imported class (parse limitation + ADR-0045 whole-module binding), so no
cross-unit subclass site exists to reject. Within a unit, subclassing a
`@sealed` class in the same file is rejected; the decorator's only other live
effect is gating `@variant`. Disposition: **keep the enforcement code** — it
is the correct shape for the day module-qualified `extends` lands — but
sealed.md's honesty note must travel to any doc citing `@sealed` as a
guarantee. The dual-representation cleanup (`sealed_by_attr` vs
`sealed_by_table`, DEFERRED #35) belongs to the class-sealing follow-ups, not
this tree; noted, not re-planned. Related invariant tests
(`sealed_classes` key/value redundancy) are recorded in
docs/deferred/class-sealing-followups.md item 4 and remain unlanded.

### `@construct` residue transferring to `@constructor`

Two behaviors must be decided (not silently inherited) when
`derive_constructor` absorbs `derive_construct` in U-CTOR:

1. **Keyword-only parameters** — `Point.new(x: 3, y: 4)` works,
   `Point.new(3, 4)` doesn't. Keep keyword-only: it is field-order-robust
   (reordering fields doesn't silently break positional call sites — the
   field-order-is-API hazard both data.md and the Phaldoc table flag), and
   consistent with ADR-0043's no-defaults idiom.
2. **Own-fields-only, no super chaining** — a derived constructor on a
   subclass ignores inherited fields. The test-strategy draft specced
   super-inference (`construct.super_ambiguous` etc.); as-built punts.
   U-CTOR should adopt the specced inference (single-constructor parent →
   chain; ambiguous → error) rather than freeze the punt; the fixture names
   already exist in the archived [test-strategy.md](../../../../work/pending/ctor/notes/test-strategy.md).

## Evaluated extensions

- **`@data` deep `with`/clone** — no. Shallow is documented, cheap, and
  matches slot-vector copy (ADR-0011). Deep-copy semantics on an object graph
  with identity is a tar pit (Ruby's `dup` vs `clone` confusion); a user who
  needs deep copy writes it.
- **`@variant` payload immutability** — variant fields are mutable today
  (they're ordinary `@data` classes). A future `const`-field `@variant` form
  becomes attractive once U-BINDINGS' const fields are routine; park until a
  use case forces it.
- **Field-level `@default(v)`** — the persistence draft wants it; as a
  *general* derive (seed value for `@construct`-omitted params) it already
  exists via field initializers. No second mechanism.

## What this precludes

- A `match`-syntax exhaustiveness checker separate from the visitor — the
  keyword-selector visitor **is** the exhaustiveness mechanism (guards ⊗
  exhaustiveness hazard: sealed set + unguarded arms is exactly what the
  visitor encodes for free).
- Hand-written `==` without `hash` on a `@data` class (and vice versa) —
  enforced, and the enforcement is the spec.
