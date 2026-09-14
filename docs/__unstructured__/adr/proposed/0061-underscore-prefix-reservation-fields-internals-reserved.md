# 61. Underscore prefixes are reserved: `_` fields, `_$` language internals, `__` reserved

- Status: Retired — superseded by PDR-0032 on 2026-08-08
- Date: 2026-07-14
- Superseded by: [PDR-0032](../../pdr/0032-transition-1-language-surface-convergence.md)
- Related: [ADR-0052](../accepted/0052-annotation-contracts.md) (`@construct`/`@get`/`@variant`
  derivation — this ADR makes `strip_leading_underscore` total);
  [ADR-0002](../accepted/0002-metaclass-tower.md) (why `Object#__attach` is registered once,
  instance-side, and reaches class/method/module receivers);
  [`docs/spec/current/syntax/README.md`](../../spec/current/syntax/README.md) (identifier grammar);
  `attribute-classes.md` (M-ATTR-ROOT — the mechanism whose selectors this ADR renames)

## Context

> Historical proposal. PDR-0032 assigns `__name` to implementation fields,
> keeps `_$name` for implementation selectors, and uses `@private`/`@protected`
> for source visibility. Those later rules are authoritative.

Phalcom has no implicit-`self` message send. A bare `foo(...)` in a method body
parses as a *variable/field read* followed by a `.call` send
([`parser.rs:1985`](../../../phalcom-ast/src/parser.rs)), never as dispatch to
`self.foo`. Separately, `parse_primary` treats **any** identifier with a leading
`_` as a field reference:

```rust
if value.starts_with('_') { Expr::Field { .. } } else { Expr::Var { .. } }
```
([`parser.rs:2387`](../../../phalcom-ast/src/parser.rs))

The rule counts *presence*, not *arity*, of underscores. The M-ATTR-ROOT
mechanism names its native selectors `Object#__attach(_)`, `Object#__attributes`,
`Object#__freezeAttributes()` ([`primitive/attribute.rs`](../../../phalcom-core/src/primitive/attribute.rs))
on the double-underscore convention that these are internal. Those two facts
collide:

- `obj.__attach(x)` works — the dot path goes through `parse_property_name`
  ([`parser.rs:2109`](../../../phalcom-ast/src/parser.rs)), which is a bare
  `Token::Identifier(name) => Ok(name)` with no prefix check.
- bare `__attach(x)` **fails at compile time** — it parses as
  `Expr::Field{"__attach"}`, and lowering
  ([`compiler/lib/expr.rs:264`](../../../phalcom-core/src/compiler/lib/expr.rs))
  misses the field layout and returns `CompilerError::ReadBeforeWrite("__attach")`.
  The diagnostic names a field the author never wrote.

So the double-underscore convention is *only* a convention. It buys no grammar,
no enforcement, and one actively misleading error. Three further gaps compound it:

1. **Underscore selectors are user-definable.** `parse_method_name`
   ([`parser.rs:1374`](../../../phalcom-ast/src/parser.rs)) is
   `Token::Identifier(n) => n.clone()` — nothing stops `method __attach(...)`
   in user source, shadowing or colliding with the native mechanism.
2. **Field declarations do not require the prefix that field *access* requires.**
   `parse_field_decl` ([`parser.rs:1156`](../../../phalcom-ast/src/parser.rs)) takes
   any identifier, so `let foo = 1` declares a field named `foo` that can never be
   read — bare `foo` in a body parses as `Expr::Var`. Silently unreachable state.
3. **`$` is not a lexical character at all.** It appears nowhere in
   [`lexer.rs`](../../../phalcom-ast/src/lexer.rs); `scan_identifier` is
   `[A-Za-z_][A-Za-z0-9_]*`.

We want a prefix that marks language internals, is callable without ceremony,
and is closed to user definition — and we want `__` held back for a future
purpose rather than spent on today's internals.

## Decision

Three reserved prefixes, with distinct grammar:

| Prefix | Meaning | Definable in `.ph` | Bare in a method body |
|---|---|---|---|
| `_name` | instance/static field | yes — field decls only | field access (unchanged) |
| `_$name` | language-internal selector | **no** — native registration only | implicit-`self` send |
| `__name` | reserved, no assigned meaning | **no** | compile error |

### 1. Lexical

```
identifier  := internal | ordinary
internal    := "_$" [A-Za-z] [A-Za-z0-9_]*
ordinary    := [A-Za-z_] [A-Za-z0-9_]*
```

`$` is legal **only** as the second byte of an identifier whose first byte is
`_`. `$foo`, `a$b`, `_$$x`, `__$x` stay invalid — the surface is not opened to a
general `$` identifier character. Both forms still emit `Token::Identifier`: no
new token kind, no downstream match arms change.

This rule applies to **`scan_identifier` and `scan_symbol_name` alike**
([`lexer.rs:252`](../../../phalcom-ast/src/lexer.rs),
[`lexer.rs:476`](../../../phalcom-ast/src/lexer.rs)). They share a continuation
set today; they must share the `_$` opening via one helper, or they drift.
Omitting the symbol path would make internals *unreferenceable as symbols* —
`::#_$attach(_)` (pinned method ref) and `perform(#_$attach)` would stop being
expressible, where `#__attach` works today.

### 2. Definition ban

`parse_method_name` rejects any identifier with a leading `_`, uniformly —
`_attach`, `__attach`, **and** `_$attach`:

> `selector.reserved_prefix: selector names may not begin with '_' ('_' is reserved for fields, '_$' for language internals)`

No `core.ph` exemption is needed: `core.ph` only *calls* these selectors, never
defines one. Native registration goes through the `primitive!` macro
([`universe/primitives.rs`](../../../phalcom-core/src/universe/primitives.rs))
and bypasses the parser entirely, so `_$` selectors remain installable from Rust
with **no escape hatch in the surface grammar**.

### 3. `parse_primary` arm ordering (load-bearing)

`_$attach` also starts with `_`, so the existing `starts_with('_')` test would
swallow it as a field. Order is a correctness constraint, not style:

```rust
Token::Identifier(value) => {
    self.advance();
    if value.starts_with("_$") {          // 1. internal send
        // peek '(' → MethodCall { object: SelfVar, method: value, args }
        // else     → GetProperty { object: SelfVar, property: value }
    } else if value.starts_with("__") {   // 2. reserved → error
    } else if value.starts_with('_') {    // 3. field (unchanged)
        Ok(Expr::Field { value, range })
    } else {
        Ok(Expr::Var { value, range })
    }
}
```

The `_$` arm **must consume its own argument list** rather than returning a node
and letting `parse_call`'s postfix loop run. That loop turns a trailing `(` into
`MethodCall { method: "call" }`, which would lower `_$attach(x)` to
`self._$attach.call(x)` — wrong. Handling both shapes inside the arm keeps
chaining intact: `_$attributes.filter { ... }` yields `GetProperty`, then the
loop picks up `.filter` normally.

Both shapes are required because the internals span two signature kinds:
`_$attributes` is `SignatureKind::Getter` (no parens), `_$attach` is `Method(1)`,
`_$freezeAttributes` is `Method(0)`.

**This is the first and only implicit-`self` send in the grammar.** Scoping it to
`_$` — a prefix closed to user definition — keeps "no implicit-`self` sends"
intact for all user-writable code.

### 4. `__name` rejection is parse-time

Rejected in the arm above, not in `Expr::Field` lowering:

> `identifier.reserved_prefix: '__' is reserved for future language use; language internals use the '_$' prefix (did you mean '_$attach'?)`

Parse-time matters because AST-synthesized field nodes (`@get`/`@set` derivation,
[`attributes.rs:924,1032,1144`](../../../phalcom-core/src/compiler/attributes.rs))
bypass the parser. A compiler-level check would have to reason about provenance;
a parser-level one never sees them.

### 5. Property-position ban

`parse_property_name` rejects a leading `_` **except** `_$`:

```rust
if name.starts_with('_') && !name.starts_with("_$") { return Err(/* reserved_prefix */); }
```

The asymmetry with §2 is intentional: definitions ban **all** leading `_`
including `_$`; call sites ban leading `_` **except** `_$`. You cannot define an
internal, but you must be able to call one — the existing
`Engine.__attach(Author.new("Bob"))`
([`runtime_attribute_store_frozen.ph:18`](../../../phalcom-core/tests/lang/runtime-errors/runtime_attribute_store_frozen.ph))
proves internals take **arbitrary receivers**, not just `self`. `_$` is a normal
selector that *additionally* has bare implicit-`self` sugar.

Single-underscore is covered too: `obj._foo` is meaningless today regardless —
`Expr::Field` is always implicit-`self`, there is no `obj._field` syntax — so it
could only ever have been a `doesNotUnderstand`. Now it is a parse error with a
real message.

`parse_property_name` also serves `::` open method refs
([`parser.rs:1950`](../../../phalcom-ast/src/parser.rs)), so `obj::__foo` becomes
a parse error and `obj::_$attach` stays legal. Intended.

### 6. Field declarations

```
field_name := "_" [A-Za-z] [A-Za-z0-9_]*
```

Rejects `foo` (missing), `__foo` (§4), `_$foo` (§2), bare `_` (no name). Internal
underscores stay legal — `_foo_bar` is fine; only the leading run is constrained.

> `field.name_prefix: field names must begin with exactly one '_'`

The `[A-Za-z]`-after-underscore requirement mirrors §1 and incidentally rejects
`_1foo`, which `scan_identifier` accepts today. Named here rather than left implicit.

### 7. Rename

`__attach` → `_$attach`, `__attributes` → `_$attributes`,
`__freezeAttributes` → `_$freezeAttributes`.

## Consequences

**`strip_leading_underscore` becomes total.** It is
`strip_prefix('_').unwrap_or(name)` today
([`attributes.rs:663`](../../../phalcom-core/src/compiler/attributes.rs)) — a
silent no-op when the prefix is absent. Post-§6 the prefix is guaranteed, so the
fallback is dead. Convert to `expect`, so a future regression surfaces loudly
instead of deriving a wrong label in silence.

**The ban is a footgun guard, not a capability boundary — state this plainly.**
`Object#perform(_)`
([`primitive/object.rs:129`](../../../phalcom-core/src/primitive/object.rs))
reflectively sends a selector *value*. `obj.perform(#__attach)` never touches
`parse_property_name`. §1's symbol-path check catches the *literal* form, but a
runtime-computed symbol is uncatchable at parse time **by construction**. A real
boundary would have to live in dispatch — deliberately out of scope here; do not
read this ADR as providing one.

**`@variant` manufactures a reserved name from legal input.** The synthesizer
([`attributes.rs:1292`](../../../phalcom-core/src/compiler/attributes.rs)) builds
`FieldDef { name: format!("_{}", label) }` — already single-underscore-prefixed,
so it satisfies §6 for free. But labels come from
`expect_identifier(&["variant label"])` with no prefix check, so
`@variant Foo(_bar:)` synthesizes field `__bar` — **two** underscores, violating
§4, from a source line that never wrote `__`. `parse_variant_decl` must ban
leading `_` on variant labels. The same check likely belongs on `@construct`
param labels; only the `@variant` instance was confirmed this pass.

**Migration is one fixture, and call sites do not move.**
[`class_attribute_construct_get_set.ph:7-9`](../../../phalcom-core/tests/lang/classes/class_attribute_construct_get_set.ph)
(`var x`/`var y`/`@get var label` → `var _x`/`var _y`/`@get var _label`) is the
only `.ph` break, and it is already `status: PENDING` — no green test moves.
`@construct` derives labels via `strip_leading_underscore`, so field `_x` yields
label `x` exactly as `var x` did: `Point.new(x: 3, y: 4, label: "origin")` and
`p.label` keep working verbatim. Only declaration lines change. `core.ph` is
already compliant (`var _targets`/`var _tier`,
[`core.ph:991`](../../../phalcom-core/core/core.ph)).

**All existing internal call sites use explicit receivers**, so the rename is
mechanical: `self.__attributes` ([`core.ph:1020-1029`](../../../phalcom-core/core/core.ph)),
`Engine.__attach(...)` (the test above), and two compiler-synthesized sends
(`make_signature("__attach", …)` / `make_signature("__freezeAttributes", …)`,
[`class_decl.rs:785,798`](../../../phalcom-core/src/compiler/lib/class_decl.rs)).
Post-rename, `self.__attributes` → `_$attributes` — the `self.` may drop.

Touch-set: `lexer.rs` (§1, both scanners) · `parser.rs` §§2-6
(`parse_method_name`, `parse_primary`, `parse_property_name`, `parse_field_decl`,
`parse_variant_decl`) · `universe/primitives.rs` (3 `primitive!` literals) ·
`class_decl.rs:785,798` (2 `make_signature` literals) ·
`invariants.rs:709-711` (exact selector-string asserts; `NEW_ATTR_ROOT = 3`
unchanged) · doc comments in `primitive/attribute.rs`, `method/object.rs:50-55`,
`class_decl.rs`, `core.ph:962`.

New negative fixtures (negative lane, per the golden-test lane split, or the
suite reddens): reserved-prefix selector definition (×3 prefixes), bare `__foo`
reference, `let __x` / `let foo` field decls, `#__attach` symbol literal,
`@variant Foo(_bar:)`.

**`__` is spent deliberately.** It buys nothing today by design — it is held for
a future purpose. Anyone reaching for a "clearly internal" prefix before that
purpose is ruled should use `_$`.

## Alternatives considered

- **Keep `__` for internals, add implicit-`self` sugar to it.** Rejected: spends
  the prefix we want to reserve, and `__` reads as "private by convention" in
  Python/C++ — a connotation we would then have to fight.
- **A general `$` identifier character (`$attach`).** Rejected: opens the whole
  surface (`a$b`, `$foo`) to buy one prefix, and `$`-leading names have no
  existing meaning in the grammar to anchor on.
- **Enforce in the compiler rather than the parser.** Rejected: AST-synthesized
  nodes from `@get`/`@set`/`@variant` bypass the parser, so a compiler check must
  distinguish synthesized from authored nodes — provenance the AST does not carry.
- **Make the prefix ban a real capability boundary.** Rejected for v0.2:
  `perform` takes runtime values, so it must be enforced in dispatch, which is a
  larger change with a per-send cost. Named in Consequences so no future reader
  mistakes the guard for a boundary.
