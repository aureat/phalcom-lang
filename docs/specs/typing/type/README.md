# Phalcom Type Algebra Registry

**Status:** Normative registry meta-specification

## Purpose

This directory is the semantic law registry for Phalcom's type system. Individual files define coherent law families. The registry is intended to serve simultaneously as language specification, solver conformance authority, property-test source, and architecture vocabulary.

## Identifier ordering

Every normative law has a global identifier:

```phalcom
@id(TXXXX)
@name(type.semantic.path)
LawName<A, B> {
    ...
}
```

Lower numeric identifiers are more foundational. Normative dependencies must point only to lower identifiers. Derived laws therefore appear after the axioms/theorems they consume, even when this places a cross-family law later than its most obvious subject file would otherwise suggest.

IDs in this coherence revision supersede the provisional IDs in the initial draft corpus. Once implementation/tests begin citing the revised IDs, they should be treated as stable and changed only through an explicit registry migration.

## Law parameter quantification

Generic law parameters are implicitly universally quantified over values/entities for which the law expression is well formed. A law does not authorize ill-kinded substitution.

## Semantic relations

- `A == B` means semantic identity of kind-compatible type-level entities. It is not AST equality, pointer identity, source equality, or overloadable runtime value equality.
- `A <: B` is the semantic subtype relation and is defined only for proper types of kind `Type`.
- `staticEquivalent(a, b)` is identity in the static-value universe. It is distinct from overloadable runtime equality.
- `valueEqual(a, b)` or `runtimeEqual(a, b)` denotes value-level equality when a law explicitly needs it. Neither relation defines type identity.
- `kind(X)` classifies a static entity. Proper types have kind `Type`; admissible static values have the proper type they inhabit as their kind.

## Kind-domain model

Phalcom uses a unified static classification model:

```text
kind(Int) == Type
kind(20) == Int          // when 20 is admitted as a static value
kind(Bool) == Type
kind(true) == Bool
```

Every proper type can therefore act as a kind domain for its admissible static inhabitants. `type data` builds on this directly: the declaration defines a proper type/static kind domain, and its cases are static values of that kind.

## Logical notation

- `if P { Q }` means semantic implication.
- `P iff Q` means biconditional equivalence.
- `not P` means semantic negation.
- `exists x ...` is existential quantification.
- `illFormed(E)` means `E` must be rejected before ordinary semantic solving.
- `notEntailsFrom(P, Q)` means `P` alone is insufficient to derive `Q`; it does not assert `not Q`.
- `doesNotImply` is descriptive shorthand for the same non-entailment relation and should not be interpreted as Boolean negation.
- `Proven`, `Disproven`, and `Unknown` are solver proof states, not semantic types.

## Law kinds

The prose accompanying a law classifies it as one of:

- **Semantic law:** part of the language's denotational type semantics.
- **Derived semantic law:** theorem following from lower laws.
- **Well-formedness invariant:** constrains legal type-level expressions.
- **Representation invariant:** constrains canonical/compiler representation without defining meaning.
- **Solver invariant:** constrains proof behavior.
- **Cross-subsystem coherence invariant:** requires multiple implementations/views of the same semantics to agree.

## Semantic law versus rewrite rule

An equality in the registry does not automatically authorize eager rewriting in either direction. In particular, distributive equivalence and recursive/projection reductions may be semantically valid while intentionally remaining symbolic to avoid exponential expansion or nontermination. `type.normalization` separately specifies canonicalization policy.

## Completeness boundary

A solver may be incomplete outside fragments explicitly declared closed/decidable. `Unknown` is therefore distinct from `Disproven`. Soundness is global; completeness is scoped.

## Registry dependency invariant

For every explicit normative dependency `A -> B` where law `A` depends on law `B`:

```text
id(B) < id(A)
```

The coherence validator for this corpus checks unique IDs, unique semantic paths, valid references, and backward dependency ordering.

## ID allocation after coherence pass

| File | Range |
|---|---:|
| `type.identity.md` | `T0001–T0003` |
| `type.kind.md` | `T0050–T0060` |
| `type.subtype.md` | `T0100–T0106` |
| `type.never.md` | `T0150–T0156` |
| `type.any.md` | `T0200–T0205` |
| `type.static-value.md` | `T0250–T0259` |
| `type.unit.md` | `T0300–T0307` |
| `type.exact.md` | `T0350–T0363` |
| `type.boolean.md` | `T0400–T0412` |
| `type.union.md` | `T0450–T0465` |
| `type.intersection.md` | `T0500–T0515` |
| `type.algebra.md` | `T0550–T0565` |
| `type.function.md` | `T0600–T0617` |
| `type.tuple.md` | `T0650–T0668` |
| `type.record-row.md` | `T0700–T0721` |
| `type.nominal.md` | `T0750–T0766` |
| `type.generic-constructor-variance.md` | `T0800–T0824` |
| `type.data.md` | `T0850–T0875` |
| `type.projection-type-function.md` | `T0900–T0924` |
| `type.normalization.md` | `T0950–T0975` |
| `type.inference-solver.md` | `T1000–T1036` |
| `type.reflection-pattern.md` | `T1050–T1078` |
