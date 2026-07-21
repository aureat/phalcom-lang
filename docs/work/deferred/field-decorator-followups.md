# Deferred: field-decorator follow-ups (`@get`/`@set`) — unowned

Surfaced 2026-07-19 while reconstructing how `@get`/`@set` worked *before* U-BINDINGS.
Three items: the field-declaration syntax U-BINDINGS gives the decorators (item 1, which has
a live hazard in it), plus two long-standing gotchas from U-ANNOT-LAYOUT that never got an
owning unit (items 2 and 3).

Provenance is mixed and marked per claim: **[tree]** = verified against the working tree
2026-07-19, **[memory]** = recorded at the time by claude-mem and *not* re-verified.

> **Working-tree caveat.** The field-grammar dispatch described in item 1 is **not in
> `HEAD`** — `phalcom-ast/src/parser.rs` is dirty (55 insertions) from a concurrent session
> mid-U-BINDINGS. `git show HEAD:phalcom-ast/src/parser.rs` does not contain
> `field.no_mutable_keyword`. Re-diff before acting; the line numbers below will move.

---

## 1. The new field syntax, and the `@set`-on-`const` hole

### What the decorators attach to now

Per [ADR-0064](../adr/accepted/0064-let-const-bindings-and-field-mutability.md) and
`docs/forge/units/U-BINDINGS/implementation-spec.md` §4:

```
field_decl := { attribute } [ "const" ] IDENT_leading_underscore [ "=" expr ]
```

Attributes precede the optional `const` (spec line 354), so both forms parse:

```phalcom
@get _label              // mutable field, no keyword
@get @set const _x = 1   // immutable field with default
```

Dispatch in `parse_class_member` (`phalcom-ast/src/parser.rs:1264-1290` **[tree]**):
`Token::Const` → field; `Token::Let` → **hard error** `field.no_mutable_keyword`
("mutable fields take no keyword; write `_name` instead of `let _name`"); a leading-`_`
identifier with lookahead ∈ {newline, `}`, EOF, `=`} → field. There is no third form.

### What this breaks

- **`@get var label` is gone.** The pre-U-BINDINGS fixture form **[memory]** no longer
  exists; `class_attribute_construct_get_set.ph` is already migrated to `@get _label`
  **[tree]**. The spec counts **16** corpus `var _x` sites needing the keyword dropped and
  calls it "the only hard case" (implementation-spec line 160). `let _x` at field position
  is **0** occurrences, so that error path is a guard, not a migration.
- **`FieldDef.mutable` flips its source.** `parse_field_decl` now sets `mutable: !is_const`
  (`parser.rs:1196` **[tree]**). Anything still reasoning "`var` ⇒ mutable, `let` ⇒
  immutable" is inverted — including prose in older unit plans.
- Two of the 16 sites are in `benchmarks/annotations/showcase.ph:69,71` (`@get @set var _x`)
  — the showcase is a decorator demo, so it is also documentation.

### The hazard: nothing owns `@set` on a `const` field

`derive_accessors` does not consult `FieldDef.mutable` **[memory]** — it derives a Setter
from `@set` unconditionally. U-BINDINGS §5 makes a `const` write outside a constructor
`field.const_write`, enforced by a **new** `Compiler::in_constructor` flag checked at the
`Bytecode::SetField` emission site (`compiler/lib/expr.rs:329,333`). A derived setter is a
`Setter` member, not a constructor, so `@set const _x` lands in one of two bad states:

1. the check fires and the diagnostic's span points at **compiler-synthesized code** the
   user never wrote, or
2. the check misses derived members and `const` is silently violated through the accessor.

The spec asks for a fixture `field_attributes_before_const.ph` — `@get @set const _x = 1`
(§9, line 544) — but only to *prove the attribute-before-`const` parse*. It does not say
what the derived setter should do. That is the gap.

**Recommendation:** reject `@set` on a `const` field at derive time, inside
`derive_accessors`, with an `attr.*` diagnostic that names the decorator and points at the
user's `@set` — same shape as the existing `attr.accessor_collision`. Do this in the same
unit that lands const enforcement, or the fixture above ships as a booby trap. `@get` on
`const` is fine and needs no change.

---

## 2. Mixed declared / inferred fields in one class (DEC-ANNOT-H)

Field inference by assignment is still live — U-BINDINGS' own probe P7 reproduces it: "a
class with zero field declarations compiles and runs" (implementation-spec line 45). So a
class can carry `FieldDef`s *and* implicit-by-assignment fields at once, and their relative
**order is ambiguous** — `FieldDef`s are position-known at parse time, inferred fields are
discovered in assignment order (U-ANNOT-LAYOUT plan lines 267-269).

Order is not cosmetic here: field declaration order is API (R3, `selectors.md` §1) and fixes
the generated parameter list of every layout-derive decorator.

**DEC-ANNOT-H is recorded as "flagged, not resolved"** (plan line 428), with a recommended
policy: *any `FieldDef` present ⇒ inference off for that class*, matching how
`field_layouts`' reopen-guard already treats first-definition-wins. The unit's return
contract (plan line 462) asserts the policy "was implemented as specified" — **that
assertion has not been independently confirmed against the tree.** Confirm before relying
on it; if it is real, DEC-ANNOT-H should be closed rather than left flagged.

---

## 3. `derive_construct` emits keyword-labeled parameters only

**[memory]**, recorded 2026-07-13 during the `@get`/`@set` session (commit `60db152`),
flagged out of scope then and **never tracked since** — it appears in neither
`docs/forge/DEFERRED.md` nor the U-CTOR plan **[tree]**.

A `@construct`-derived constructor labels every parameter, so:

```phalcom
Point.new(x: 3, y: 4)   // works
Point.new(3, 4)         // fails — no positional form
```

Consistent with the surviving fixture, which calls
`Point.new(x: 3, y: 4, label: "origin")` and is still marked `status: PENDING` **[tree]**.

**Owner candidate: U-CTOR-3.** That step replaces `@construct` with a target-polymorphic
`@constructor` and adds `derive_constructor` next to `derive_construct`
(`docs/forge/units/U-CTOR/plan.md:152-163`), codemodding 148 sites. Whether derived
constructors gain a positional form is a decision that step should make explicitly rather
than inherit by accident — and if the answer is "labeled-only, by design", say so and drop
the fixture's `PENDING`.
