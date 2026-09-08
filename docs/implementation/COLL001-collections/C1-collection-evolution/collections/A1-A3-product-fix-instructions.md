# Phalcom A1-A3 Product Implementation Fix Instructions

## Scope

This document contains actionable fixes identified from review of the implemented A1-A3 product syntax, runtime representation, and construction pipeline.

Each item contains:
- exact file
- target location
- issue
- required change
- verification

---

# P1 — Clarify and enforce product construction invariants

## File

`phalcom-core/src/product.rs`

## Location

`finish_tuple` and `finish_record`

## Issue

Empty product normalization is implemented, but the invariant is only partially documented.

The compiler currently prevents empty products from reaching construction, while runtime construction also normalizes them. The two layers have the same rule but different responsibilities.

## Change

Add comments documenting:

- compiler normalization is an optimization;
- runtime normalization is an invariant boundary;
- no heap allocated empty TupleObject or RecordObject may exist.

Do not remove either check.

## Verification

Add tests proving:
- `()` evaluates to Unit.
- `{}` evaluates to Unit.
- no product allocation occurs for empty products.

---

# P1 — Protect tuple labeled suffix invariant

## File

`phalcom-core/src/product.rs`

## Location

`finish_tuple`

## Issue

Tuple construction relies on the compiler emitting:

1. positional values
2. labeled values

The runtime assumes this ordering but does not explain the contract.

## Change

Add a comment above `finish_tuple` documenting the bytecode contract:

```
values contains positional entries followed by labeled entries.
labels describe only the labeled suffix.
```

Keep the representation unchanged.

## Verification

Add nested tuple tests containing labeled fields.

---

# P1 — Audit remaining Tuple.fromList surface

## Files

Search results indicate references in:

`phalcom-core/core/core.ph`

and collection tests.

## Issue

A3 moved literal construction away from method-based construction. Remaining references must be intentional compatibility APIs only.

## Change

Audit every `Tuple.fromList` usage.

Allowed:
- compatibility APIs;
- tests verifying compatibility behavior.

Not allowed:
- compiler lowering;
- literal construction;
- VM construction paths.

## Verification

Search:

```
Tuple.fromList
```

and confirm no compiler/runtime literal path depends on it.

---

# P2 — Add explicit record order-independence tests

## File

`phalcom-core/tests/lang/collections/`

## Issue

Record representation preserves insertion order. Equality and hashing must not accidentally depend on insertion order.

## Change

Add tests:

```
#{a: 1, b: 2} == #{b: 2, a: 1}
```

and verify equal hashes.

## Verification

Run collection test suite.

---

# P2 — Add Symbol canonicalization tests

## File

`phalcom-core/tests/lang/collections/`

## Issue

Product labels depend on canonical Symbols.

## Change

Add tests verifying equivalent labels use the same Symbol identity.

Examples:

```
(a: 1)
#{a: 1}
```

must use the same label Symbol.

## Verification

Run VM tests.

---

# P2 — Replace assertion-only internal invariant checks if required

## Files

Potential locations:

- `phalcom-core/src/heap/tuple.rs`
- `phalcom-core/src/heap/record.rs`

## Issue

Review whether invariant checks disappear in release builds.

## Change

For impossible VM states prefer explicit panic paths over debug-only assertions.

## Verification

Build release mode and run invariant tests.

---

# P3 — Add regression coverage for nested products

## Add tests covering:

```
((1,2), #{a:3})
#{a:(1,2), b:#{c:3}}
((a:1),)
```

## Purpose

These exercise:

- recursive construction;
- stack ordering;
- GC interaction;
- printing;
- equality.

---

# Completion checklist

- [ ] Construction invariants documented
- [ ] Tuple suffix-label contract documented
- [ ] Tuple.fromList usage audited
- [ ] Record order-independent equality tested
- [ ] Symbol canonicalization tested
- [ ] Nested product regression tests added
