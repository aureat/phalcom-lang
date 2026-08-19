# Phalcom Numeric Specification Set

> **Status:** Ratified architecture; normative consolidation dated **2026-07-29**.
>
> **Authority:** This set records the project-owner ratification of the numeric architecture developed during the review of the original numeric specifications. It supersedes conflicting language in the earlier `numeric-tower.md`, `float-protocol.md`, `numeric-literals.md`, `text-and-errors.md`, `bitwise.md`, and numeric index. Repository ADR/PDR status files must still be amended so the source tree reflects this ratification.
>
> **Version:** `NUMERIC-SPEC-2026-07-29`

## 1. Purpose

This directory defines Phalcom's numeric language contract, implementation architecture, conformance requirements, migration consequences, and unresolved decisions.

The set intentionally separates permanent language semantics from repository-specific implementation instructions. A source path, dependency version, primitive count, commit hash, or worktree condition cannot silently become part of the language contract.

## 2. Document map

| File | Status | Purpose |
|---|---|---|
| [`numeric-tower.md`](numeric-tower.md) | Normative | Public numeric types, arithmetic, comparison, division, remainder, conversion, keys, and laws. |
| [`float-protocol.md`](float-protocol.md) | Normative, with named open tables | Binary64 behavior, Float protocol, rounding, power architecture, NaN, signed zero, and total-order requirement. |
| [`numeric-literals.md`](numeric-literals.md) | Normative | Source literal grammar, candidate boundaries, classification, oversized constants, and compiler diagnostics. |
| [`text-and-errors.md`](text-and-errors.md) | Normative | Text constructors, canonical rendering, runtime error taxonomy, structured fields, and traceback rules. |
| [`bitwise.md`](bitwise.md) | Normative | Infinite-two's-complement Int operations, precedence, huge-count behavior, laws, and errors. |
| [`implementation.md`](implementation.md) | Implementation contract | Runtime representation, semantic kernel, constant pool, GC, hashing, resource controls, dispatch, primitive floor, and landing order. |
| [`conformance.md`](conformance.md) | Normative conformance contract | Reference model, test matrices, properties, edge corpus, differential rules, and ship gates. |
| [`migration.md`](migration.md) | Normative change inventory; release mechanism open | Breaking changes, compatibility consequences, source migration, and release requirements. |
| [`open-decisions.md`](open-decisions.md) | Open-decision register | The remaining unresolved names, algorithms, tables, defaults, and release choices. |
| [`amendment-map.md`](amendment-map.md) | Informative | Exact corrections and improvements relative to the original uploaded files. |
| [`NUMERIC-SPEC-COMBINED.md`](NUMERIC-SPEC-COMBINED.md) | Generated convenience edition | All documents concatenated in reading order. The individual files remain authoritative. |

## 3. Normative hierarchy

When documents overlap, apply this order:

1. A closed decision in this set overrides conflicting prose in an earlier numeric ADR, PDR, or specification until those records are amended.
2. `numeric-tower.md`, `float-protocol.md`, `numeric-literals.md`, `text-and-errors.md`, and `bitwise.md` define public semantics.
3. `conformance.md` defines what an implementation must prove.
4. `implementation.md` constrains architecture without exposing internal representation as public behavior.
5. `open-decisions.md` identifies deliberately unsettled points. An open item must not be inferred from examples or host-library behavior.
6. `migration.md` describes compatibility impact; the release mechanism remains open under **OD-NUM-010**.

The words **must**, **must not**, **shall**, and **shall not** are normative. **Should** records a strong default that may be departed from only with an explicit project decision. **May** grants permission.

## 4. Ratified decision index

| ID | Decision |
|---|---|
| NUM-001 | Public tower is `Number` with concrete `Int` and `Float`; `Int` is exact and unbounded. |
| NUM-002 | Small and large Int representations are private and canonicalized through one normalizer. |
| NUM-003 | Numeric behavior is implemented through operation-specific semantic paths, not one universal promotion helper. |
| NUM-004 | Float-producing mixed arithmetic may round; exact comparison, key equality, hashing, and `~/` may not round an Int to Float. |
| NUM-005 | Mixed Int/Float equality and order compare exact mathematical values. |
| NUM-006 | Finite Int-to-Float conversion rounds ties-to-even and raises on finite-range overflow. |
| NUM-007 | Float-to-Int conversion is explicit; `Int.new(Float)` always rejects. |
| NUM-008 | `/` always returns Float. |
| NUM-009 | `~/` returns exact Int and floors exact represented values. |
| NUM-010 | `%` follows floor-division semantics for Int and Float; a named `fmod` is deferred. |
| NUM-011 | Int nonnegative power is exact; Float-domain power has a special-case table and one-ULP finite-result bound. |
| NUM-012 | `rounded` uses nearest, ties to even. |
| NUM-013 | Public Float equality remains IEEE-like; a separate total-order operation shall exist. |
| NUM-014 | Map/Set numeric keys merge equal Int/Float values, signed zeroes, and all NaNs while preserving the first key representative. |
| NUM-015 | Numeric hashing is one coherent mathematical model and accepts arbitrary Int results from user-defined `hash`. |
| NUM-016 | Literal and constructor grammars share productions but remain separate entry grammars. |
| NUM-017 | Oversized integer constants are heap-independent compiler constants, not live VM object references. |
| NUM-018 | Numeric resource failures are deterministic policy failures, distinct at compile time and runtime. |
| NUM-019 | Int bitwise semantics are infinite two's complement; huge nonnegative counts remain semantically valid. |
| NUM-020 | Numeric operations remain ordinary selector dispatch; optimized paths are guarded and deoptimizable. |
| NUM-021 | Integer-only boundaries require Int; integral Float is not accepted. |
| NUM-022 | Runtime numeric failures are structured language `Error` values sent through the ordinary raise path. |
| NUM-023 | Conformance uses an independent mathematical model plus targeted differential oracles. |
| NUM-024 | The numeric specification is split into semantics, implementation, conformance, migration, and open decisions. |

## 5. Closed architectural invariants

The following are no longer open:

```text
1.class == Int
1.0.class == Float
Int.is(Number)
Float.is(Number)
Number is allocator-abstract
Bool is not numeric
Int is exact and unbounded
6 / 2 has class Float
~/ returns Int
Int and equal integral Float compare equal
Map and Set merge equal numeric keys
Float indices are rejected
```

## 6. Remaining decisions

No foundational tower decision remains open. The unresolved items are bounded and named:

- **Blocking public-semantics items:** OD-NUM-001 through OD-NUM-006.
- **Blocking implementation selections:** OD-NUM-007 through OD-NUM-009.
- **Blocking release-policy item:** OD-NUM-010.
- **Deferred extensibility item:** OD-NUM-011.

See [`open-decisions.md`](open-decisions.md). Open items must be resolved before the corresponding surface or subsystem ships; they do not reopen the ratified architecture.

## 7. Non-goals of this revision

This set does not introduce Decimal, Rational, Complex, fixed-width integer classes, unsigned integers, SIMD values, bitwise Float operations, implicit user-defined numeric coercions, or persistent hash values. It deliberately leaves a future numeric-extension protocol open.
