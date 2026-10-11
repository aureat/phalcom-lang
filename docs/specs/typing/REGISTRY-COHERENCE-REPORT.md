# Phalcom Type Algebra Registry — Coherence Pass Report

**Status:** `STRUCTURAL RULES RECONCILED / MECHANICALLY VALIDATED`
**Scope:** Existing Type Algebra Registry corpus plus foundational repairs, including the current Tuple/Record/Pack cross-specification reconciliation.
**Resulting specification files:** 22
**Resulting laws:** 375

## 1. Objective

The pass converted the initial law collection into a dependency-ordered semantic registry suitable for implementation references and solver-conformance work. The pass targeted contradictions, over-strong laws, category errors, forward dependencies, and ambiguity in the registry's conceptual notation. It did not attempt to add every still-missing Phalcom type-system family.

The original report described only registry-internal mechanical checks. Those checks
did not establish agreement with newer feature-specific normative specifications.
The current result adds targeted cross-specification assertions for high-risk
structural rules, concrete variadic kind domains, and conservative runtime
reflection. It still does not claim that every current or future language
specification has been semantically reconciled.

## 2. Registry-level changes

### 2.1 Added a normative registry meta-specification

`type/README.md` now defines:

- the meaning of `@id(TXXXX)` and `@name(...)`;
- implicit universal quantification of law parameters;
- semantic type identity versus static-value identity versus runtime equality;
- the unified kind-domain model;
- `iff`, negation, non-entailment, well-formedness, and solver proof-state notation;
- semantic law versus normalization/rewrite policy;
- scoped solver completeness;
- the invariant that every explicit normative dependency must point to a lower law ID.

### 2.2 Globally renumbered the law corpus

The initial IDs were provisional and did not consistently respect foundational dependency order. The revised IDs are globally partitioned into ascending semantic-family ranges from `T0001` through `T1078`.

`ID-MIGRATION.md` contains the complete old-to-revised mapping.

### 2.3 Added mechanical validation

`tools/validate_registry.py` validates:

- unique law IDs;
- unique semantic `@name` paths;
- valid semantic path syntax;
- no dangling `TXXXX` references;
- matching file range headers;
- ascending IDs within files;
- backward-only explicit normative dependencies;
- balanced Markdown code fences.
- the 11 canonical Tuple, Record, `FieldKey`, and Pack-rest invariants listed
  in §2.4 across the registry, structural owner/companion specifications, and
  callable specifications;
- the current variadic lane-domain names used by T0058; and
- the reflection registry's conservative-information law and its runtime API
  contract.

Current result:

```text
REGISTRY VALIDATION: PASSED
specification files: 22
laws: 375
unique IDs: 375
unique semantic paths: 375
dangling references: 0
forward/self explicit dependencies: 0
cross-spec canonical assertions: 13
```

### 2.4 Cross-specification conformance assertions

The registry validator now fails if the registry, the structural semantic owner
or syntax companion, or the callable rest specification cease to agree on:

- `Tuple<> == Unit`;
- negative Tuple indexing;
- closed Tuple rejection of implicit width subtyping;
- open Tuple remainder subtyping;
- required-only fixed Tuple entries;
- closed Record rejection of implicit width subtyping;
- open Record width compatibility;
- Record optional-field presence;
- the `FieldKey` domain;
- Pack-native Closure rest capture; and
- `Pack() != Unit`.

These are maintained assertions over stable normative markers, not a claim that
the validator proves the complete prose semantics.

The two added assertions keep T0058 tied to the distinct concrete lane domains
(`TypePack`, `LabeledTypePack`, and `CompleteTypePack`) and ensure T1062's
reflection-conservatism requirement is also stated by the runtime API owner.

```text
registry mechanical validation
!=
whole-spec semantic coherence
```

## 3. Semantic corrections

### 3.1 `Any` is no longer compared to solver `Unknown`

The initial corpus contained a category error equivalent to `Any != Unknown`, even though `Unknown` was defined as a solver knowledge state rather than a semantic type. The revised registry states instead that `Any` is a semantic proper type and is not a knowledge state. Solver proof states are outside the operand domain of semantic type identity.

### 3.2 Added a static-value identity layer

A new `type.static-value.md` family separates compile-time/static identity from arbitrary runtime equality.

The revised model establishes:

```text
staticValue(v)
staticEquivalent(a, b)
kind(v) == A      when v is a static inhabitant of A
```

Proper types are themselves static values of kind `Type`. Runtime equality does not define static identity, and runtime values are not implicitly promoted into the static universe.

This repairs the earlier exact-type model, which otherwise allowed overloaded/general value equality to leak into type identity.

### 3.3 Exact types now require static values

`exact(v)` is well formed only for values admitted to the static-value universe. Exact-type injectivity is defined through `staticEquivalent`, not arbitrary runtime `==`.

The `Unit == exact(())` relationship was moved into the exact-type family so its dependency points backward correctly.

### 3.4 `type data` now defines user-authored static kind domains

The initial corpus modeled `type data` cases as ordinary proper types and related them to the parent through `<:`. That conflated a static value with its singleton type.

The revised model is:

```text
kind(Direction) == Type
kind(Direction::North) == Direction

kind(Bound<Int>) == Type
kind(Bound<Int>::Included) == Int -> Bound<Int>
kind(Bound<Int>::Included<20>) == Bound<Int>
```

A nullary/static case is a static value. Its singleton proper type is `exact(D::V)`, and therefore:

```text
exact(D::V) <: D
```

A `type data` declaration is closed by constructor family, but it is not assumed finite when constructors carry payloads. Only finite nullary static domains receive a finite union decomposition into exact singleton types.

### 3.5 Nominal generic identity now includes static arguments

The initial law `decl(A) == decl(B) => A == B` incorrectly collapsed distinct generic specializations from the same declaration.

Nominal identity is now defined by both:

```text
declaration identity
+
ordered static argument identity
```

Thus `List<Int>` and `List<String>` may share a declaration while remaining distinct nominal types.

### 3.6 Generic constructor laws no longer overclaim

The pass narrowed several laws that were valid for nominal generic constructors but not arbitrary reducible type functions:

- distinct constructors remain distinct only under nominal-constructor identity rules;
- invariance means argument subtyping does not induce covariance or contravariance; it does not prohibit unrelated separately declared subtype relations;
- arity mismatch is a well-formedness error rather than a semantic inequality;
- parameter-order identity is asserted for nominal constructors, where ordered arguments are part of nominal identity;
- constructor injectivity remains explicit rather than assumed.

### 3.7 Distributivity is now scoped

The global proper-type universe is specified as a bounded lattice. Distributive laws apply only where `distributiveTypeFragment(...)` is established.

This preserves strong set-theoretic algebra where valid without prematurely forcing future overload intersections, quantified types, unresolved projections, or other non-extensional constructs into a distributive model.

Normalization remains explicitly free to avoid distributive expansion even where semantic distributivity holds.

### 3.8 Semantic normalization is separated from solver proof-state behavior

`type.normalization` now states semantic subtype preservation:

```text
A <: B iff normalize(A) <: normalize(B)
```

The stronger condition that the canonical solver entrypoint returns the same proof state before and after explicit normalization remains in `type.inference-solver`, where it is properly an implementation/solver contract.

### 3.9 Inference received explicit bound algebra

The solver family now specifies:

- lower/upper-bound consistency;
- lower-bound join behavior;
- upper-bound meet behavior;
- final solution containment between effective bounds.

This connects inference directly to the same union/intersection lattice used by ordinary subtype semantics.

### 3.10 Pattern exhaustiveness now distinguishes finite domains

The initial pattern laws could be read as treating every closed `type data` constructor family as a finite value domain. The revised registry distinguishes constructor closure from finite cardinality.

Finite nullary `type data` domains support exhaustive exact-case enumeration. Parameterized static domains remain closed by constructor family but are not assumed finite.

## 4. Dependency corrections

The pass removed the major forward dependency classes identified in the audit:

- `type.unit` → future `type.exact`;
- `type.function` → future `type.tuple`;
- `type.nominal` → future `type.generic`;
- `type.projection` → future `type.normalization`;
- `type.normalization` → future `type.solver`.

Cross-family derived laws were moved conceptually to locations/ranges where their prerequisites are already defined, or rewritten so the foundational family states only the semantic fact it owns.

## 5. What this pass does not claim

The original registry pass was mechanically validated only against its
then-existing internal checks. The current pass reconciles the listed structural
invariants and verifies them mechanically across their owner, companion, and
registry views. Neither result alone proves whole-spec semantic coherence, and
the corpus is still not a complete specification of every Phalcom type-system
feature.

High-priority semantic families still requiring dedicated specification include:

- trait/interface conformance, conditional conformance, evidence coherence, and associated-type selection;
- metatypes, `.type`, `.class`, class objects, metaclasses, and the two `Self` contexts;
- type/value packs and pack algebra;
- generic quantification, rigid variables, generalization/instantiation, and existential constraints;
- Phalcom method/selector callable semantics and type-directed method polymorphism;
- explicit error/poison/recovery-state semantics if those states are observable to type solving;
- GADT result refinement and pattern-driven existential refinement;
- open/polymorphic rows if adopted.

Those are extension work. They are no longer needed to repair the internal contradictions of the existing core registry.

## 6. Recommended status

The revised corpus is suitable as the **core Type Algebra Registry baseline**
for the laws it actually covers, with the current structural rules reconciled
against their feature-specific owners. Its status remains mechanical validation
plus scoped cross-specification conformance, not a blanket semantic-coherence
certificate.

New type-system features should either reference an existing law ID or extend the registry before implementation semantics are considered authoritative.
