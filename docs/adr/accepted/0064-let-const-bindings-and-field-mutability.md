# 64. `let`/`const` bindings; unkeyworded mutable fields

- Status: **Accepted** (ratified by the user 2026-07-15, DEC-CTOR-I/I2)
- Date: 2026-07-15
- **Supersedes [ADR-0014](0014-let-and-var-bindings.md)** (*let and var bindings*).
  ADR-0014's *semantics* survive unchanged; only the spellings move, plus one new rule
  for fields.
- Related: [ADR-0063](0063-constructors-are-ordinary-class-side-methods.md)
  (`@class`/`@constructor`; its field grammar defers to this ADR — **U-BINDINGS lands
  first**) · [ADR-0011](0011-static-instance-slot-layout.md) (instance slot layout;
  read-before-write) · [ADR-0017](0017-class-side-stored-static-fields.md) (class-side
  storage) · [ADR-0007](0007-option-as-abstract-with-some-none.md) (an unset binding
  reads `None`)

## Context

ADR-0014 gave Phalcom `let` (immutable) and `var` (mutable). Two problems have
accumulated.

**1. The common case carries the longer, less obvious keyword.** `var` is the mutable
binding and is what most code wants locally, while `let` — the word most languages
(Rust, Swift, JS) use for the *ordinary* binding — is reserved for the immutable one.
Corpus: 728 `let` vs 352 `var`. The naming does not pay for itself.

**2. `let` is not enforced on fields — it is decorative.** ADR-0014 says reassignment
of a `let` is a compile error. On a *field* that is simply not true. Measured:

```phalcom
class K {
  let _n
  construct new(n) { _n = n }
  clobber(v) { _n = v }      // second write, different method
  n => _n
}
K.new(1).clobber(99)   // → 99, no error
```

So today `let _n` promises immutability and delivers nothing. That is worse than not
offering the distinction: it misleads. It is also how a draft of ADR-0063's own example
ended up mutating a `let` field without anyone noticing.

Separately, ADR-0063 introduces `@class` on fields, so the field grammar is being
touched anyway. Settling mutability now avoids migrating the same declarations twice.

## Decision

### 1. Bindings — `let` is mutable, `const` is immutable

```
binding := ("let" | "const") IDENT [ "=" expr ]
```

| Form | Meaning |
|---|---|
| `let x` | mutable, uninitialized → reads `None` (ADR-0007) |
| `let x = e` | mutable |
| `const x = e` | immutable |
| `const x` | **compile error** — a `const` must be given its one value up front |

`var` ceases to be a keyword and becomes an ordinary identifier.

**This is a pure rename of ADR-0014's semantics**: old `var` → `let`, old `let` →
`const`. Every rule carries over, including "an uninitialized mutable binding reads
`None`" and "an immutable binding requires an initializer". No new analysis, no
behavior change.

### 2. Fields — mutable is unkeyworded, `const` is immutable

```
field_decl := { attribute } [ "const" ] FIELD [ "=" expr ]
```

| Form | Meaning |
|---|---|
| `_x` | mutable field |
| `_x = e` | mutable field, declaration default |
| `const _x = e` | immutable, defined at declaration |
| `const _id` | immutable, **assignable only inside a constructor** |

Mutable fields take **no keyword**. This is the deliberate asymmetry with §1: fields
are already `_`-prefixed, already private, and already implicitly declared by
assignment (`classes.md` §2) — a keyword adds nothing a reader cannot see. `let _x`
becomes a parse error, so the misleading form from Context is unspellable rather than
merely discouraged.

`@class` composes: `@class _total = 0`, `@class const _limit = 10`.

**`const` fields relax §1's initializer requirement** — `const _id` with no `=` is
legal, unlike `const x` at binding position — because a constructor is the field's
definition site. `const _id = 5` *and* a constructor write is not: the declaration
default is the definition, so a later write is the same error as any other.

### 3. `const` field enforcement is syntactic, not flow-sensitive

> A write to a `const` field from anywhere other than a `@constructor` body is
> `field.const_write` — a compile error.

Keyed purely on **which member the write appears in**. Phalcom has no flow analysis
(overlay: "enforced as written contract + golden-test snapshot, not static analysis"),
and this rule deliberately needs none.

What that buys and what it does not:

- ✅ Catches the Context case — `clobber(v) { _n = v }` is rejected.
- ❌ Does **not** catch two writes *inside* one constructor. Accepted; it would need
  definite-assignment analysis.
- ❌ Does **not** require that a constructor actually assigns the field. A `const _id`
  never written reads `None` **forever** — no method may repair it.

That last point is the sharp edge, and it is reachable: ADR-0063 §7 rules that `new()`
is an ordinary inherited method, so `Factory.new()` bypasses every constructor and
yields an object whose `const` fields are permanently `None`. **This is specified, not
a bug** — it composes two ratified rules, and it is consistent with `classes.md` §2's
"a field declared but not yet assigned reads `None`". A class that cannot tolerate it
should declare a default (`const _id = …`) or not use `const`.

## Consequences

### The migration is context-sensitive and is a swap — it is not a `sed`

1080 declarations across 395 files (352 `var`, 728 `let`). Two traps:

**It is a swap.** `var`→`let` and `let`→`const` in two passes turns every original
`var` into `const`. A **single pass** over tokens is required.

**The mapping depends on position, not on the token.** The same word maps differently
by context:

| Position | Old | New |
|---|---|---|
| statement | `var x` | `let x` |
| statement | `let x` | `const x` |
| class body | `var _x` | `_x` |
| class body | `let _x` | `const _x` |

A blind textual rewrite gets the class-body cases wrong. The codemod must parse, or at
minimum be class-body aware.

### `const` is free; `var` is released

`const` appears nowhere in the corpus and has no `Token::Const` — the name is
available. `var` becomes an ordinary identifier, usable as a method or variable name.

### ADR-0014's status flips

ADR-0014 → **Superseded by 0064**, with `STATUS.md` synced in the same pass
([[adr-status-two-way-sync]]). Its content is not wrong, only respelled; the
superseding note should say exactly that, so a reader chasing an ADR-0014 citation is
not misled into thinking the *semantics* changed.

### Sequencing: U-BINDINGS before U-CTOR

ADR-0063's `@class` field work sits on the field grammar this ADR rewrites. Landing
U-CTOR first would migrate the same declarations twice and force the field codemod to
understand two grammars. **U-BINDINGS is the first unit.**

### What this must not preclude (P4)

- **Definite-assignment analysis.** §3 is deliberately weaker than "a `const` field is
  always assigned". If Phalcom ever grows flow analysis, §3 can tighten without a
  surface change — the syntax already expresses the intent.
- **ADR-0011 read-before-write.** Untouched. That check is about *any* field read with
  no write anywhere in the class; §3 is about *where* a write may appear. They compose.
- **`@class` on fields (ADR-0063 §2.1).** The `[ "const" ]` slot sits inside
  `field_decl`, after attributes, so `@class const _limit = 10` parses with no further
  grammar change.
- **A future `const` on parameters or methods.** Not specified here; the keyword is not
  spent on any other position.

## Alternatives considered

- **Keep `let`/`var`, just enforce `let` on fields.** Rejected: fixes the honesty
  problem but leaves the common case (`var`) carrying the unusual keyword, and burns
  the migration budget without buying the naming improvement. If the corpus is moving
  anyway, move it once.
- **`let`/`var` for fields too, mutable = `var`.** Rejected: fields are already
  `_`-prefixed, private, and implicitly declared by assignment. Requiring a keyword on
  the mutable (overwhelmingly common) case is ceremony for no information.
- **`const _x` requires an initializer, like `const x`.** Rejected: it would force
  every constructor-computed immutable field to first take a dead default, which is
  both wasteful and a lie about the field's real definition site.
- **Runtime write-once enforcement** (a `const` field is writable while unset,
  regardless of member). Rejected: needs a per-write runtime check on a hot path, and
  turns "immutable" into "write-once-whenever" — weaker *and* less predictable than a
  syntactic rule.
- **Two-pass codemod.** Rejected — it is a swap; see Consequences.
