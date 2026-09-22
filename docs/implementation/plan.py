from pathlib import Path
from textwrap import dedent

out = Path("LANG007/LANG007.C3.P3-requirements-analysis.md")

doc = dedent(r'''
---
id: LANG007.C3.P3.requirements-analysis
category: LANG
program: LANG007
checkpoint: LANG007.C3
plan: LANG007.C3.P3
kind: requirements-analysis
status: PROPOSED
completion: NOT_STARTED
verification: UNVERIFIED
title: "Intrinsic Structural Surface, Universe Migration, Tooling, Certification, and C3 Closure"
repository: aureat/phalcom-lang
branch: main
prepared_against_revision: 60a27f79d69aa3e79c22e3d19df221466c7db920
prepared: 2026-09-22
depends_on:
  - LANG007.C3.P2
  - LANG007.C3.P1
  - LANG007.C2
---

# LANG007.C3.P3 — Requirements Analysis

## Intrinsic Structural Surface, Universe Migration, Tooling, Certification, and C3 Closure

## 0. Executive conclusion

`LANG007.C3.P3` is the last planned implementation work unit of checkpoint
`LANG007.C3 — Type Data and Unified Algebraic Static Domains`.

It is not merely a source migration and it is not merely a test/certification pass.

P3 has four distinct responsibilities that must be executed in dependency order:

```text
1. EXPOSE
   bind compiler-owned structural static/runtime domains to canonical source declarations

2. MIGRATE
   move Universe and repository-owned active source from legacy enum/class facades
   to the accepted data/type-data surface without changing semantic identity

3. TOOL
   make source index, LSP, reflection/presentation, diagnostics, and editor syntax
   consume the final C3 semantic identities and terminology

4. CERTIFY
   validate P1+P2+P3 together, close authoritative specifications, prove runtime
   isolation/persistence/incremental/performance invariants, and close C3 truthfully
```

The accepted P3 endpoint includes source-visible declarations equivalent to:

```phalcom
intrinsic type data TypePack
intrinsic type data LabeledTypePack
intrinsic type data TupleEntries
intrinsic type data RecordRow

intrinsic data Unit()
intrinsic data Tuple<***Es>
intrinsic data Record<R: RecordRow>
```

and structural members equivalent to:

```phalcom
impl Tuple<***Es> {
    intrinsic type Entries: TupleEntries
    intrinsic type Positionals: TypePack
    intrinsic type Labeled: LabeledTypePack
    intrinsic type At<I: Int>: Type
    intrinsic type Map<F: Type -> Type>: Type
}
```

with the non-negotiable semantic bridge:

```text
TupleEntries
    ==
existing C2 CompleteTypePack semantic domain
```

and:

```text
*T   == T::Positionals
**T  == T::Labeled
***T == T::Entries
```

where the operations are valid.

The most important architectural requirement for P3 is that these declarations are
**facades over already-canonical semantics**, not new semantic owners.

For static structural domains:

```text
source declaration TypePack
    -> existing KindId::TYPE_PACK / TypeData::TypePack authority

source declaration LabeledTypePack
    -> existing KindId::LABELED_TYPE_PACK / TypeData::LabeledTypePack authority

source declaration TupleEntries
    -> existing KindId::COMPLETE_TYPE_PACK / TypeData::CompleteTypePack authority

source declaration RecordRow
    -> existing KindId::RECORD_ROW / RecordRowId + row solver authority
```

For runtime structural domains:

```text
source declaration Unit
    -> existing canonical Unit type/runtime identity

source declaration Tuple
    -> existing structural Tuple type formation + specialized Tuple runtime object/class

source declaration Record
    -> existing RecordRow-backed Record type formation + specialized Record runtime object/class
```

P3 must **not** produce:

```text
new StaticDomainId for TypePack
new TypeData::TupleEntries
nominal Applied<Tuple, CompleteTypePack> replacing structural Tuple
nominal Applied<Record, RecordRow> replacing TypeData::Record
generic DataObject storage for Tuple/Record
runtime values for TypePack/LabeledTypePack/TupleEntries/RecordRow
```

The second major requirement is final migration away from `enum`.

The accepted C3 language model is already decided:

```text
data
    sole canonical runtime algebraic declaration keyword

type data
    static-only algebraic/static domain declaration
```

Therefore final P3 language acceptance must no longer accept `enum` as a declaration
form. A lexer/parser token may temporarily survive during the migration stage only as a
diagnostic aid; the final P3 state must contain no independent semantic or accepted
source declaration path for `enum`.

The live pre-P3 repository baseline makes the migration scope concrete:

- the canonical Universe still declares `Option`, `Result`, and `Ordering` as
  `native enum`;
- additional Universe source files still contain `enum`;
- the current Tuple/Record/Unit source anchors are `native class`, not
  `intrinsic data`;
- `Tuple` exposes `Positionals`, `Labeled`, `At`, and `Map`, but not `Entries`;
- `TupleEntries` exists only in planning/older specification material and has no live
  semantic source declaration;
- `TypePack`, `LabeledTypePack`, and `RecordRow` do not yet have canonical Universe
  source declarations;
- the live type-member intrinsic lookup still recognizes domain names by strings such
  as `"Tuple"`, `"TypePack"`, and `"RecordRow"`;
- the source/native Universe census currently models classes plus sum-data through
  `SourceEnumRecord`/`UniverseClassRow`-style structures;
- `SourceDeclarationKind` still contains `Enum`;
- LSP semantic tokens still classify `Token::Enum` as a keyword;
- the VS Code TextMate grammar still contains enum-specific declaration/variant rules;
- the current LSP supports definition/references/hover/completion but has no
  `rename`/`prepareRename` request implementation;
- the authoritative `docs/specs/` corpus still describes `enum` as the canonical sum
  declaration and `data` as product-only data objects.

Those facts are **current baseline observations**, not permission to execute P3 now.
The repository at the time of this analysis is still at a partial P1 revision. P3 must
start only from a completed P2 revision that satisfies the P2 acceptance contract.

The final closure rule is:

> **P3 may close C3 only if it consumes P1/P2 architecture as already complete, exposes
> compiler-owned structural semantics without duplication, completes canonical source
> and tooling migration, validates the entire C3 semantic matrix, updates authoritative
> specifications, and leaves the runtime/static/proper-type boundaries unchanged.**

If P3 discovers that it needs a new static-term carrier, a new associated-domain identity
model, a new generic argument representation, or a new inference architecture, P2 is
incomplete. The correct action is STOP/CONSULT and corrective predecessor work, not a
P3 redesign.

---

# Part I — Analysis authority and repository state

## 1. Purpose of this document

This document performs requirements analysis for the final C3 plan:

```text
LANG007.C3.P3
```

It defines:

- the predecessor contract P3 may assume;
- the exact semantic meaning of intrinsic structural source facades;
- the Universe migration requirements;
- the TupleEntries/pack bridge requirements;
- the final legacy `enum` retirement requirements;
- corpus/spec/tooling migration scope;
- metadata/interface/incremental certification requirements;
- runtime representation and runtime-negative certification requirements;
- performance/resource requirements;
- final C3 validation breadth;
- closure truth and lifecycle requirements;
- risks and mandatory escalation triggers.

This is not the P3 patch-grade implementation plan.

A later implementation plan should turn this analysis into ordered implementation tasks
and verification gates.

---

## 2. Repository baseline used for analysis

The live repository inspected for this analysis is:

```text
repository: aureat/phalcom-lang
branch:     main
revision:   60a27f79d69aa3e79c22e3d19df221466c7db920
subject:    feat(semantic): type data foundation
parent:     62a6082ab463387c812d558e5561508267ecc63c
date:       2026-09-22
```

At this revision:

```text
LANG007.C3
    IN_PROGRESS / PARTIAL / UNVERIFIED

active implementation
    partial P1

P2
    not landed in the repository

P3
    future
```

Therefore this document uses two evidence classes.

### 2.1 Live repository facts

These are directly true at `60a27f79...`.

### 2.2 Required predecessor facts

These are the facts P1/P2 must establish before P3 can execute. They are requirements
from the accepted C3 architecture and P2 plan, not claims about the current revision.

P3 must re-audit the actual completed-P2 revision before implementation.

---

## 3. Normative sources

The primary C3 planning authorities are:

```text
docs/implementation/LANG007/LANG007.C3/
    LANG007.C3-type-data-unified-algebraic-static-domains-spec-and-requirements-analysis.md
    LANG007.C3-requirements-analysis.md
    LANG007.C3-CHECKPOINT.md
    LANG007.C3-GUIDANCE.md

LANG007.C3.P1 implementation plan
LANG007.C3.P2 requirements analysis
LANG007.C3.P2 implementation plan
```

The authoritative language/platform specification corpus is:

```text
docs/specs/
```

per `docs/specs/README.md`.

Historical material under:

```text
docs/spec/
docs/implementation/
docs/decisions/
...
```

does not override `docs/specs/` once P3 closes the accepted semantic migration.

P3 must finish updating the owning `docs/specs/` documents before checkpoint closure.

---

# Part II — Hard predecessor contract

## 4. P3 must not begin from partial P2 semantics

P3 is a migration and certification plan.

It is not an architecture laboratory.

P3 may begin only after P2 has established all of the following.

### 4.1 Canonical static term/application substrate

```text
[ ] TypeId remains canonical for non-row static terms
[ ] ProperTypeId still proves KindId::TYPE
[ ] StaticDomainId is canonical for user nominal static domains
[ ] StaticArgument is the canonical heterogeneous application envelope
[ ] TypeData::Applied preserves heterogeneous arguments
[ ] RecordRow remains row-owned
[ ] generic binder domains use one KindId classification system
```

### 4.2 Static inference/substitution

```text
[ ] non-row static inference reuses InferVarId(kind)
[ ] type-data unification is structural/first-order
[ ] checked substitution/materialization re-canonicalizes
[ ] static aliases/result domains work
[ ] type-member Int/user-domain semantics use canonical static terms
```

### 4.3 Associated domains

```text
[ ] associated type-data declarations exist
[ ] associated-domain declaration identity is stable
[ ] generic-owner specialization is declaration + canonical environment
[ ] AssociatedSurface owns :: collision/lookup semantics
[ ] source index can attach associated static identities
```

### 4.4 Persistence

```text
[ ] static domains/terms/applications round-trip through stable metadata
[ ] imported static domains preserve identity
[ ] incremental and cold P2 products converge
[ ] query-local IDs do not enter durable products
```

### 4.5 Runtime boundary

```text
[ ] type-data terms have no runtime inhabitants
[ ] no VM static solver exists
[ ] arbitrary runtime code is not executed for static identity
[ ] runtime proper-Type consumers still validate ProperType
```

If any item is false, P3 must stop.

The missing work belongs to P1/P2 corrective closure.

---

# Part III — P3 semantic objective

## 5. Final P3 objective

P3 must turn the C3 semantic model into the canonical public platform surface and prove
that all layers agree.

The required final architecture is:

```text
SOURCE
│
├─ data
│   ├─ ordinary runtime algebraic products
│   ├─ ordinary runtime closed sums
│   └─ intrinsic runtime structural declaration facades
│
└─ type data
    ├─ nominal user static domains
    └─ intrinsic compiler structural-domain facades
            │
            ▼
SEMANTIC AUTHORITIES
├─ algebraic declaration authority
├─ TypeStore / KindId / StaticArgument
├─ RecordRow authority
├─ AssociatedSurface
└─ source semantic index
            │
            ├───────────────┐
            ▼               ▼
      STABLE METADATA    LOWERING/RUNTIME
                           only Runtime data
            │               │
            ▼               ▼
     IMPORTS / LSP      specialized representations
```

The intrinsic declaration surface must make compiler-owned semantics discoverable through
the same declaration/reflection/tooling model while preserving the existing underlying
authorities.

---

# Part IV — Intrinsic static-domain facades

## 6. Required public structural static domains

P3 must expose source declarations equivalent to:

```phalcom
intrinsic type data TypePack
intrinsic type data LabeledTypePack
intrinsic type data TupleEntries
intrinsic type data RecordRow
```

These declarations are **not nominal user static domains** in the P1/P2 sense.

They are source/reflection anchors for compiler structural domains that already exist.

This distinction is essential.

---

## 7. Canonical mapping requirements

The final mapping must be exactly one-to-one.

| Public source facade | Existing canonical authority | Forbidden duplicate |
|---|---|---|
| `TypePack` | `KindId::TYPE_PACK`, `TypeData::TypePack`, pack concat/projection machinery | new `StaticDomainId(TypePack)` term universe |
| `LabeledTypePack` | `KindId::LABELED_TYPE_PACK`, `TypeData::LabeledTypePack` | new nominal static-data representation |
| `TupleEntries` | `KindId::COMPLETE_TYPE_PACK`, `TypeData::CompleteTypePack` | `TypeData::TupleEntries` |
| `RecordRow` | `KindId::RECORD_ROW`, `RecordRowId`, row arena/solver | fake row `TypeId` or nominal row domain |

### 7.1 Domain resolution

After P3:

```phalcom
P: TypePack
L: LabeledTypePack
E: TupleEntries
R: RecordRow
```

must classify to:

```text
KindId::TYPE_PACK
KindId::LABELED_TYPE_PACK
KindId::COMPLETE_TYPE_PACK
KindId::RECORD_ROW
```

respectively.

The source declaration name is a stable public anchor.

It does not replace the internal domain identity.

### 7.2 Reflection/presentation

The user-facing reflective name of `KindId::COMPLETE_TYPE_PACK` may be:

```text
TupleEntries
```

but internal diagnostic/debug formatting may retain `CompleteTypePack` where the output is
explicitly implementation-oriented.

Public language-facing formatting should converge to `TupleEntries`.

---

## 8. Do not allocate nominal `StaticDomainId`s for structural builtins

P1/P2 nominal user `type data` domains use dynamic:

```rust
KindData::StaticDomain(StaticDomainId)
```

P3 structural builtins must not be forced through that representation merely because
their source facade uses `type data`.

The accepted architecture already gives these domains dedicated canonical kinds.

Creating:

```text
StaticDomainId(TypePack)
StaticDomainId(LabeledTypePack)
StaticDomainId(TupleEntries)
StaticDomainId(RecordRow)
```

as the semantic authority would duplicate C1/C2.

A source-declaration-to-builtin-domain binding table is appropriate.

A second term representation is not.

---

## 9. Intrinsic static-domain registration

P3 needs a canonical semantic association between source declaration identity and the
compiler-owned domain.

Conceptually:

```rust
enum IntrinsicStaticDomain {
    TypePack,
    LabeledTypePack,
    TupleEntries,
    RecordRow,
}

struct IntrinsicStaticDomainBinding {
    declaration: DeclarationId,
    domain: IntrinsicStaticDomain,
}
```

Exact naming is flexible.

The binding must resolve to existing `KindId`/row authority.

It must not become an alternate type store.

### 9.1 Ownership

Recommended ownership:

```text
canonical Universe source declaration
    owns public declaration/source identity

semantic intrinsic-domain registry
    owns source-declaration -> compiler-domain association

TypeStore / row subsystem
    own actual static term identity and normalization
```

### 9.2 Runtime prohibition

No runtime Universe class is required for these domains.

Do not add VM classes merely to make reflection convenient.

If public reflection wants descriptors, they are static-domain reflection descriptors,
not runtime value classes.

---

# Part V — Bodiless intrinsic declarations

## 10. P3 must define intrinsic declaration-only semantics

The accepted P3 examples use bodiless forms:

```phalcom
intrinsic type data TypePack
intrinsic type data LabeledTypePack
intrinsic type data TupleEntries
intrinsic type data RecordRow

intrinsic data Tuple<***Es>
intrinsic data Record<R: RecordRow>
```

The current partial-P1 parser proves `intrinsic type data` with an explicit body but does
not yet establish these declaration-only structural facades.

P3 therefore requires a precise declaration-only intrinsic rule.

### 10.1 Required meaning

A bodiless intrinsic algebraic declaration says:

> This canonical source declaration presents a domain whose full representation/algebra
> is supplied by compiler-owned intrinsic semantics.

It does not mean:

```text
empty sum
zero-field product
unknown user declaration
abstract trait-like requirement
native runtime body
```

### 10.2 Scope restriction

Declaration-only `data` / `type data` should be permitted only where an accepted
implementation-origin rule supplies the semantics.

At minimum:

```text
intrinsic data ...
intrinsic type data ...
```

Ordinary user:

```phalcom
type data Shape
```

without a shape/body must remain invalid unless separately specified.

### 10.3 Semantic binding requirement

Every declaration-only intrinsic must resolve to one registered compiler authority.

Unknown/unregistered:

```phalcom
intrinsic type data UserMystery
```

must fail semantic validation in ordinary user source if user-authored intrinsic
declarations are not allowed by implementation-origin rules.

No “opaque unknown kind” fallback.

---

# Part VI — Intrinsic runtime structural data

## 11. Required public runtime structural facades

P3 must expose declarations equivalent to:

```phalcom
intrinsic data Unit()
intrinsic data Tuple<***Es>
intrinsic data Record<R: RecordRow>
```

These declarations are **runtime-visible nominal/source anchors** whose storage and
structural type formation remain specialized.

They must not be lowered as ordinary P1 generic data objects.

---

## 12. Unit bridge

Current semantic/type invariants already include:

```text
empty tuple type == canonical Unit type
```

P3 must bind:

```phalcom
intrinsic data Unit()
```

to the existing canonical Unit identity.

Required:

```text
source Universe declaration identity preserved
runtime UniverseKey::Unit association preserved
TypeStore unit identity preserved
empty tuple canonicalization preserved
runtime class relation Unit -> Object preserved
existing Unit behavior preserved
```

Forbidden:

```text
new nominal data TypeId distinct from TypeData::Unit
new DataObject runtime representation for Unit
```

---

## 13. Tuple bridge

The required declaration:

```phalcom
intrinsic data Tuple<***Es>
```

is a source-level facade over the existing structural Tuple family.

It must **not** redefine Tuple as an ordinary nominal generic constructor:

```text
Applied(Nominal Tuple, [CompleteTypePack])
```

if the canonical structural type remains:

```text
TypeData::Tuple(...)
TypeData::TupleFromPacks { ... }
```

### 13.1 Required identity separation

Keep:

```text
source declaration identity:
    universe.collections.tuple::Tuple

structural tuple type identity:
    TypeData::Tuple(...)
    TypeData::TupleFromPacks(...)

runtime dispatch class:
    canonical Tuple runtime class / UniverseKey::Tuple
```

These are related projections of one language/platform concept, not interchangeable IDs.

### 13.2 Runtime hierarchy

The live runtime relation currently establishes:

```text
Tuple -> Collection
```

P3 must preserve it.

The fact that ordinary data objects are non-inheritable does not authorize breaking the
compiler-owned intrinsic Tuple's established runtime supertype relation.

The intrinsic source facade may rely on the compiler/native Universe relation if ordinary
`data` syntax intentionally does not expose superclass syntax.

The source/runtime census must not falsely claim the superclass was authored in source
if it is supplied by intrinsic platform metadata.

### 13.3 Runtime representation

Tuple values remain specialized Tuple objects/representations.

No generic DataObject migration.

---

## 14. Record bridge

The required declaration:

```phalcom
intrinsic data Record<R: RecordRow>
```

is the source facade over existing:

```text
TypeData::Record(RecordRowId)
RecordRow canonical identity
specialized Record runtime object/class
```

Do not implement it as:

```text
Applied(Nominal Record, [row])
```

if that would duplicate or replace the existing canonical Record type.

Preserve:

```text
Record runtime class identity
Record -> Object relation
Record structural equality/hash semantics
row-domain canonicalization
Record::Map behavior
```

---

# Part VII — Behavior migration for Tuple/Record/Unit

## 15. Representation in declarations; behavior in `impl`

Current Universe source places Tuple and Record behavior inside `native class` bodies.

C3's algebraic model requires representation-bearing declaration syntax to remain
separate from behavior.

P3 should therefore migrate behavior to `impl` blocks around the intrinsic data anchors.

Conceptually:

```phalcom
intrinsic data Tuple<***Es>

impl Tuple<***Es> {
    // intrinsic type members
    // native runtime operations
    // Phalcom methods
}
```

and:

```phalcom
intrinsic data Record<R: RecordRow>

impl Record<R> {
    ...
}
```

Exact generic target syntax must follow completed P2 semantics.

Unit may similarly move behavior into:

```phalcom
impl Unit { ... }
```

if the data declaration grammar does not admit behavior.

### 15.1 Native methods remain native

A declaration being `intrinsic data` does not mean every runtime operation becomes a
compiler intrinsic.

Examples such as:

```text
rawAt
rawSize
rawLabelAt
rawValueAt
```

may remain `native` runtime members.

The declaration implementation origin and individual behavior implementation origin are
orthogonal.

---

# Part VIII — Native/Universe source indexing implications

## 16. Current source census assumptions are too class/enum-specific

The live pre-P3 Universe source index models:

```text
UniverseClassRow
SourceEnumRecord
SourceEnumVariantRecord
SourceDeclarationRecord::{Class, Enum}
```

and indexes:

```text
Statement::Class
Statement::Data if sum
Statement::Impl
```

That is insufficient for final C3.

P3 must make source/native census categories represent final semantic source forms.

### 16.1 Required census categories

The census must distinguish at least:

```text
runtime class-like declaration presentation
runtime algebraic data presentation
type-only static-domain presentation
behavior/member presentation
```

without reviving `enum` as a source language category.

The exact internal types may stay compatibility-named if private, but exported/public
records and user-facing reflection should prefer:

```text
Data
Case
StaticDomain
```

rather than:

```text
Enum
Variant
```

### 16.2 Runtime class relation attachment

Intrinsic runtime `data` anchors for Unit/Tuple/Record still need association with
`UniverseKey` and runtime class relations.

Type-only domains must not be inserted into:

```text
UNIVERSE_CLASS_RELATIONS
VM class tables
runtime class census
```

### 16.3 TypeOnly declarations and bootstrap roots

The live `NativeSourceIndex::bootstrap_roots()` currently treats modules containing
non-export statements as runtime bootstrap roots.

P3 must explicitly ensure that semantic-only `type data` declarations do not force
runtime module execution solely because they are declarations.

Required invariant:

```text
TypeOnly declaration discovery
    may affect semantic/source catalog

TypeOnly declaration discovery
    must not by itself require runtime initialization
```

This is especially important if intrinsic structural static-domain declarations live in
a module with no other runtime declarations.

---

# Part IX — TupleEntries bridge

## 17. TupleEntries is public naming, not new semantics

P3 must expose:

```text
TupleEntries
```

as the public static domain representing a complete two-lane Tuple entry shape.

The canonical internal authority remains:

```text
KindId::COMPLETE_TYPE_PACK
TypeData::CompleteTypePack
project_complete
project_positionals
project_labeled
```

Do not add:

```rust
TypeData::TupleEntries(...)
```

Do not add a second solver.

Do not add another metadata term kind for TupleEntries when CompleteTypePack already
represents the term.

---

## 18. Tuple `Entries` type member

P3 must add a canonical intrinsic member equivalent to:

```phalcom
intrinsic type Entries: TupleEntries
```

to the Tuple structural family.

Current live pre-P3 type-member operations include:

```text
TuplePositionals
TupleLabeled
TupleAt
TupleMap
```

P3 needs the complete-lane operation, conceptually:

```text
TupleEntries
```

whose normalization delegates to existing:

```text
project_complete
```

or exact equivalent.

No alternate projection.

---

## 19. Pack/member equivalence certification

P3 must prove semantic identity/equivalence, not merely equal formatting.

Required:

```text
*T
    ==
T::Positionals

**T
    ==
T::Labeled

***T
    ==
T::Entries
```

where `T` is a subject that supports the relevant projection.

### 19.1 Concrete cases

Test concrete structural Tuple types:

```phalcom
(Int, String, name: Symbol)
```

### 19.2 Symbolic cases

Test valid symbolic terms so equivalence survives:

```text
generic substitution
type lambda/scoped representation
metadata round trip
inference materialization
```

### 19.3 Invalid cases

Wrong-domain projection remains invalid.

P3 must not broaden pack projection applicability merely for surface symmetry.

---

# Part X — Structural type-member source anchors

## 20. Source declarations must reflect canonical domain-native members

The current authoritative `type-members.md` already says compiler structural domains may
have canonical Universe anchors for discovery/documentation/reflection.

P3 should make the source surface describe the actual domain-native operations.

At minimum the final source/reflection surface must cover:

```text
Tuple
    Entries
    Positionals
    Labeled
    At / subscript identity
    Map

Record
    Map

TypePack
    Map

LabeledTypePack
    Map

RecordRow
    Map
```

where those operations already exist in C1/C2.

### 20.1 One member identity

The source declaration must register the existing compiler intrinsic member.

It must not create a second `TypeMemberId`.

For `At`:

```text
T[I]
T::At<I>
```

remain one canonical subscript type-member identity with `At` as presentation alias.

### 20.2 Remove string-name semantic ownership

The live pre-P3 intrinsic lookup contains logic equivalent to:

```rust
lookup_intrinsic_type_operation(target_domain: &str, selector: ...)
```

matching string names such as:

```text
"Tuple"
"TypePack"
"LabeledTypePack"
"RecordRow"
```

P3 should remove source spelling as semantic authority.

Final intrinsic lookup should be keyed by canonical domain/declaration identity.

Presentation strings may remain for diagnostics.

---

# Part XI — Core closed-sum migration

## 21. Mandatory Universe sum migration

P3 must migrate:

```text
Option
Result
Ordering
```

from legacy:

```phalcom
native enum ...
```

to canonical:

```phalcom
native data ...
```

or the exact completed implementation-origin spelling accepted by the source-declaration
spec.

The implementation origin should remain `native` where runtime behavior/representation is
host-supplied.

Do not change it to `intrinsic` merely because the declaration is compiler/runtime
special.

---

## 22. Preserve Option/Result/Ordering identities

Migration must preserve:

```text
DeclarationId
UniverseKey association
generic signature
case identity
selector identity
exact-case identity
associated family identity
native runtime representation
reflection owner identity
module ownership
prelude/export visibility
```

Keyword migration alone must not change semantic product fingerprints.

### 22.1 Option representation

The VM/runtime may have specialized `Option`/`Some`/`None` representation.

P3 must preserve it.

`data` syntax is not permission to replace it with generic ADT heap objects.

### 22.2 Result/Ordering

Same rule.

Source semantic unification is above runtime representation.

---

## 23. Migrate all active repository-owned Phalcom `enum` source

The current pre-P3 repository contains at least 22 `.ph` files with `enum` source
occurrences, including Universe, examples, and test fixtures.

P3 must perform a fresh completed-P2 census.

Required migration scope:

```text
canonical Universe .ph source
active examples
active benchmarks where they are source compatibility fixtures
test fixture .ph files
golden editor examples
generated/embedded source fixtures owned by the repository
```

### 23.1 Do not blindly replace every textual `enum`

Global text replacement is forbidden.

Reasons:

```text
Rust uses enum as a language keyword
historical docs must stay historical
some tests intentionally test removed syntax
migration diagnostics may mention "enum"
file/module names may be historical or intentionally retained
```

Use syntax-aware/source-file census.

---

# Part XII — Final `enum` compatibility retirement

## 24. Final language rule

The accepted C3 decision is:

> `enum` is removed as a canonical Phalcom declaration form. Closed runtime sums use
> `data`.

Therefore final P3 must not keep:

```text
enum -> Runtime + Sum semantic compatibility
```

as an accepted language route.

The compatibility parser introduced for migration must be retired after source/corpus
migration.

---

## 25. Lexer/parser retirement choices

There are two implementation choices for the final parser surface.

### Option A — true lexical retirement

Remove:

```text
Token::Enum
lexer enum keyword recognition
parse_enum compatibility path
legacy enum AST compatibility field if no longer needed
```

Then `enum` lexes as an ordinary identifier where identifier grammar permits it.

This is the cleanest interpretation of “keyword removed”.

### Option B — reserved obsolete token with hard syntax diagnostic

Keep a lexer token only to emit:

```text
`enum` declarations were removed; use `data`
```

but do not produce any semantic declaration.

This keeps `enum` reserved.

### Recommendation

Prefer Option A unless the accepted lexical specification explicitly keeps `enum`
reserved.

The user-ratified language decision is that `enum` is removed, not merely deprecated.

A targeted parser diagnostic can still be implemented contextually for the source pattern:

```text
enum Name { ... }
```

without preserving a semantic compatibility path, if desired.

### Closure invariant

Regardless of lexical choice:

```text
no Statement/AlgebraicDef produced from legacy enum declaration syntax
no legacy enum semantic product
no metadata compatibility path based on source keyword
```

---

## 26. Remove final semantic dependence on legacy keyword provenance

During P1 compatibility, AST may retain:

```text
legacy_enum_keyword_range
```

for migration diagnostics.

P3 final closure should remove or quarantine it if no longer useful.

It must not remain in:

```text
semantic identity
metadata
fingerprints
incremental products
reflection
```

If retained as syntax recovery/presentation-only state, prove that it has no semantic
effect.

---

# Part XIII — Terminology migration boundary

## 27. Public terminology

Final current-language terminology should use:

```text
data
sum
case
exact case
data constructor
```

where describing source-language concepts.

Avoid current public documentation/tooling referring to:

```text
enum declaration
variant
enum root
```

as the active language category.

### 27.1 Internal Rust names may remain

C3 does not require mass-renaming private implementation machinery such as:

```text
EnumInfo
VariantId
enum_decl.rs
Bytecode::Enum
AdtCaseObject
```

when those are internal representation names and renaming creates risk without semantic
benefit.

### 27.2 Public/stable metadata and reflection

New public/stable semantic descriptors should prefer neutral/final terminology.

If a stable serialized field/API already uses legacy terminology and changing it has a
compatibility cost, P3 must make an explicit compatibility decision rather than silently
rename it.

---

# Part XIV — Authoritative specification closure

## 28. Current authoritative specs are not C3-final

At the live baseline:

```text
docs/specs/objects/data-objects.md
```

defines `data` as product-shaped data objects and explicitly distinguishes it from enum.

Meanwhile:

```text
docs/specs/objects/enums.md
```

defines `enum` as the canonical closed-sum declaration.

That is incompatible with the accepted C3 endpoint.

C3 cannot close with this contradiction.

---

## 29. Required specification ownership after P3

The authoritative specification must describe one runtime algebraic declaration keyword:

```text
data
    Product
    Sum
```

and:

```text
type data
    TypeOnly Product/Sum/static domain
```

### 29.1 Recommended spec restructuring

A coherent final ownership model is:

```text
objects/data-objects.md
    product-shaped runtime data semantics

objects/algebraic-data.md
or renamed former enums.md
    closed-sum data cases, exact cases, GADTs

type-level-domains.md
    type data and compiler static-domain semantics

type-members.md
    structural members + TupleEntries

associated-types.md
    explicit distinction from associated type data

objects/impl.md
    behavior for data roots/exact cases

structural-products.md
    Tuple/Record structural static semantics

universe.md
    intrinsic structural declaration anchors
```

The exact filenames are flexible.

The ownership map in `docs/specs/README.md` must be coherent.

### 29.2 No parallel current specs

Do not leave:

```text
objects/enums.md
    says enum is canonical

objects/data-objects.md
    says data is product-only
```

and merely add a third C3 note.

That would preserve contradictory current authority.

---

## 30. Dependent specification updates

P3 must audit and update current `docs/specs/` references in:

```text
objects/data-objects.md
objects/enums.md or replacement
objects/impl.md
objects/values.md
objects/traits.md where case examples exist
type-level-domains.md
type-members.md
associated-types.md
generic-constraints.md
structural-products.md
universe.md
callables/families.md / references.md where associated constructors are discussed
VM specs only where they incorrectly describe source semantics
README.md ownership map
```

### 30.1 Do not rewrite historical specification trees

Do not mass-edit:

```text
docs/spec/
docs/implementation/
historical ADRs
walkthroughs/handoffs
```

to look current.

History should remain history.

Only update historical docs if they are incorrectly linked as active authority.

---

# Part XV — Tooling and source-index closure

## 31. Source index final model

The live pre-P3 source index exposes:

```rust
SourceDeclarationKind {
    Class,
    Enum,
    Data,
    Trait,
    TypeAlias,
}
```

Final C3 should not preserve `Enum` as a current source declaration category.

A better final model is orthogonal, for example:

```text
SourceDeclarationKind::Data {
    phase: Runtime | TypeOnly
    shape: Product | Sum | IntrinsicStructural
}
```

or equivalent normalized categories.

Exact implementation is flexible.

Required:

```text
runtime sum data
type data
intrinsic static domain
intrinsic runtime structural data
```

must have source identity without reconstructing meaning from syntax strings.

---

## 32. Semantic targets

P2 should already add targets for static/associated domains.

P3 must verify the final target graph can represent:

```text
runtime data declaration
runtime case
data component
type-data declaration/domain
type-data case/constructor
associated static domain
associated static term
type member
intrinsic structural domain/member
```

LSP tooling must consume these targets.

Do not reconstruct target identity from rendered names.

---

# Part XVI — LSP closure

## 33. Required C3 LSP baseline

P3 requirements explicitly include:

```text
hover
go-to-definition
completion
rename
diagnostics
source indexing
```

for the new/final C3 constructs.

The live pre-P3 LSP currently has:

```text
definition
references
completion
hover
semantic tokens
```

but no `rename` or `prepareRename` request implementation.

Therefore rename is genuine P3 work unless P2 lands it earlier.

---

## 34. Hover requirements

Hover should distinguish:

```text
data <Name>
type data <Name>
intrinsic data <Name>
intrinsic type data <Name>
case <Name>
static term/case
type member and result domain
```

It should display canonical semantic facts from the snapshot.

No source-text reconstruction.

### 34.1 TupleEntries

Hover for:

```text
TupleEntries
Tuple::Entries
```

must reflect the public domain name while remaining backed by CompleteTypePack.

---

## 35. Definition requirements

Go-to-definition must resolve:

```text
Shape
Shape::Vector
Parser::State
Parser::State::Done
TupleEntries
Tuple::Entries
TypePack
RecordRow
Option::Some
```

to canonical source anchors where a source anchor exists.

Compiler-only terms without an independent source declaration should navigate to the
canonical facade declaration/member, not a synthetic generated file.

---

## 36. Completion requirements

Completion must expose appropriate C3 entities:

```text
associated data cases
associated static domains
associated static terms
intrinsic type members
static-domain names
```

using semantic availability/access rules.

No token-only candidate synthesis for semantically ambiguous `::`.

---

## 37. Rename requirements

P3 must add or certify semantic rename.

The implementation should be built on:

```text
semantic target at cursor
source/reference index
canonical declaration identity
```

not text search.

### 37.1 Required rename targets

At minimum for user-owned source:

```text
data declaration
data case
type-data declaration
type-data case
associated type-data declaration
ordinary references to those identities
```

### 37.2 Rename exclusions

Reject or decline rename for:

```text
compiler-only synthetic identity
read-only canonical Universe declarations
intrinsic structural domains in bundled Universe
targets whose cross-project edit ownership cannot be guaranteed
```

unless the workspace/source transport explicitly supports editing them.

### 37.3 Multi-file consistency

Rename must update all semantic references in the current editable workspace and must not
rename unrelated same-spelling entities.

### 37.4 Prepare rename

If the LSP supports `prepareRename`, use it to validate:

```text
target is canonical
target is source-backed
target is editable
cursor covers the declaration/reference spelling
```

---

## 38. Diagnostics requirements

Final diagnostics must:

```text
reject removed enum declarations
name the canonical replacement data
use case rather than variant in new source-facing messages where appropriate
preserve domain-specific P2 diagnostics
distinguish runtime data from type data
```

No compatibility warning is sufficient if `enum` remains accepted after final closure.

---

## 39. Semantic tokens / syntax highlighting

P3 must update:

```text
phalcom-lsp semantic token classifier
tools/vsphalcom TextMate grammar
IDE golden examples
```

### 39.1 `enum`

If Token::Enum is removed, it must no longer be highlighted as a current keyword.

### 39.2 `data` / `type data`

Runtime data, type data, and intrinsic declaration modifiers should be highlighted
correctly.

### 39.3 Case declarations

The TextMate grammar should stop naming final language concepts as enum variants where
that leaks into user-facing scopes/golden descriptions.

Internal scope names may remain if extension compatibility depends on them, but new
grammar organization should prefer data/case terminology.

---

# Part XVII — Reflection and presentation closure

## 40. Canonical Universe source/reflection identity

The Universe specification requires one canonical source module/declaration identity per
builtin declaration.

P3 must preserve:

```text
ModuleId
DeclarationId
UniverseKey association where runtime-backed
source URI
source range
public reflection target
native/intrinsic implementation association
```

during source-category migration.

### 40.1 No synthetic aggregate document

Do not create a synthetic generated “type domain module” merely to expose TypePack etc.

Add canonical authored Universe source anchors to the normal module graph.

---

## 41. Public reflection terminology

Where reflection exposes algebraic declaration/case categories, final presentation should
speak in final C3 terms.

Internal runtime ADT metadata may retain old field names.

User-visible reflection should not claim:

```text
Option is an enum because the runtime uses AdtCaseObject
```

It is source/semantic `data` with sum shape.

---

# Part XVIII — Universe source placement requirements

## 42. Static structural domain module placement

P3 must choose canonical Universe modules for:

```text
TypePack
LabeledTypePack
TupleEntries
RecordRow
```

The placement must satisfy:

- stable canonical module identity;
- discoverability;
- no synthetic root flattening;
- no circular bootstrap dependence;
- no unnecessary runtime execution root;
- type-member source anchors can refer to the domains without source-string hacks.

A likely placement near Tuple/Record/type-level structural facilities is acceptable.

The exact module path should be fixed in the P3 implementation plan after live Universe
catalog audit.

---

## 43. Package catalog/exposure

If P3 adds new Universe source files/modules:

```text
UNIVERSE_NODES
package expose declarations
root/prelude re-exports
physical corpus/catalog census
canonical URI mapping
```

must remain consistent.

If P3 places declarations in existing modules, avoid adding modules solely for
presentation convenience.

---

# Part XIX — Corpus migration

## 44. Corpus categories

P3 needs a source migration census partitioned into:

```text
A. canonical Universe source
B. first-party examples
C. active test fixtures
D. editor golden fixtures
E. benchmarks/first-party sample packages
F. active authoritative specs/docs
G. historical records
H. Rust implementation identifiers
```

Only A–F are ordinary migration targets.

G and H are not global text-replacement targets.

---

## 45. Final source census requirement

Before removing enum compatibility, P3 must prove:

```text
no active repository-owned .ph source requires enum declaration syntax
```

Exceptions may include a deliberately named negative fixture testing that `enum`
is rejected.

Such fixtures should be clearly negative and should not pass through semantic
compatibility.

---

## 46. Migration identity requirement

Source spelling changes:

```text
enum X { ... }
    ->
data X { ... }
```

must preserve:

```text
DeclarationId
case/selector identity
exact-case identity
associated family identity
metadata structural identity
incremental semantic fingerprint where source spelling is nonsemantic
```

Input/source fingerprints may change.

Semantic product fingerprints must not change solely because `enum` became `data`.

---

# Part XX — Metadata/interface certification

## 47. P3 should not invent a new term schema merely for facades

If P2 already serializes:

```text
TypePack
LabeledTypePack
CompleteTypePack
RecordRow
StaticArgument
associated static domains
```

P3 should reuse those nodes.

`TupleEntries` is presentation/declaration mapping for `CompleteTypePack`, not a new
term node.

---

## 48. Intrinsic source declaration metadata

P3 may need stable metadata/source-index records associating canonical declaration
anchors with compiler domains.

Required stable identity is:

```text
StableDeclarationRef
+
intrinsic structural-domain key
```

or equivalent.

Do not serialize raw:

```text
KindId
StaticDomainId
TypeId
RecordRowId
```

as stable declaration identity.

---

## 49. Interface fingerprint certification

P3 must distinguish:

```text
source/input fingerprint
semantic/public interface fingerprint
runtime representation fingerprint where separately defined
```

Required property:

```text
enum -> data keyword migration
    may change source/input fingerprint
    must not change semantic declaration/case identity
    should not change semantic interface product where no semantics changed
```

Structural facade introduction is a real public surface change and may legitimately
change interface fingerprints.

That change must be intentional and deterministic.

---

## 50. Metadata compatibility

P3 final certification must round-trip at least:

```text
runtime Product data
runtime Sum data
GADT cases
top-level type data
static term arguments
associated type data
intrinsic TypePack facade
intrinsic TupleEntries facade -> CompleteTypePack
Tuple::Entries projection
RecordRow structural domain
```

Schema compatibility behavior must remain explicit.

Do not silently reinterpret an older metadata node as a new facade declaration.

---

# Part XXI — Incremental certification

## 51. Cold/incremental parity matrix

P3 must certify final-state equality for edits covering all C3 layers.

At minimum:

```text
runtime data product component edit
runtime data sum case add/remove
GADT result edit
type-data case edit
static-domain generic argument edit
associated type-data edit
intrinsic type-member source anchor edit in test fixture
enum->data source migration in compatibility differential fixture
Tuple Entries use edit
import of intrinsic static domain
```

Final semantic products and diagnostics must equal a cold analysis of the same final
source.

---

## 52. Migration-specific invalidation

Migrating:

```text
enum X -> data X
```

should invalidate source/parser input as expected.

It must not cause dependent semantic identity churn if the declaration semantics are
otherwise equal.

P3 should include a focused incremental regression proving this property before final
compatibility removal, or use stable predecessor metadata/fingerprint fixtures to prove
the same fact after removal.

---

# Part XXII — Runtime representation certification

## 53. Runtime storage freedom remains intact

P3 must explicitly prove that final source declarations do not dictate one universal
runtime storage class.

Allowed final implementation may still include:

```text
DataObject
AdtCaseObject
TupleObject
RecordObject
specialized Option representation
immediates/special values
```

The source declaration category is semantic.

The runtime representation remains specialized.

---

## 54. Tuple/Record/Unit negative audit

After migration to `intrinsic data`, verify:

```text
Tuple values still use Tuple runtime representation
Record values still use Record runtime representation
Unit keeps canonical Unit representation
no DataObject construction is inserted for those values
```

Compiler lowering must recognize the intrinsic facade and preserve specialized handling.

---

## 55. Type-data/static-domain negative audit

Verify:

```text
TypePack has no runtime instance
LabeledTypePack has no runtime instance
TupleEntries has no runtime instance
RecordRow has no runtime instance
type-data user terms do not allocate
associated type-data terms do not allocate
```

Search bytecode/runtime changes.

There must be no new:

```text
StaticDataObject
TypePackObject for type-system packs
TupleEntriesObject
RecordRowObject solely for static typing
static-domain VM opcode
static unifier opcode
```

---

# Part XXIII — Bounded normalization certification

## 56. No general evaluator

P3 final certification must prove C3 still uses closed compiler normalization.

No path should execute:

```text
user method
runtime ==
runtime hash
closure
I/O
clock
randomness
runtime match body
ordinary runtime constructor
```

to establish static identity.

---

## 57. Static-term complexity

Final operations should remain approximately linear in reachable structure.

Required architecture:

```text
StaticArgument application
    O(argument count + child canonicalization)

StaticData interning
    hash-consed

TupleEntries projection
    existing CompleteTypePack projection complexity

associated lookup
    indexed by owner/base

metadata
    graph encoding

incremental invalidation
    dependency keyed, not global scan
```

---

# Part XXIV — Universe bootstrap/resource certification

## 58. Bootstrap must not regress because of static facades

Intrinsic type-data declarations are semantic/catalog entities.

They must not force:

```text
VM class allocation
runtime module execution
eager native initialization
runtime global binding materialization
```

unless an ordinary runtime declaration in the same module independently requires it.

### 58.1 Measurement

Reuse existing Universe bootstrap measurement infrastructure.

Record:

```text
discovered units
root reachable units
executed units
```

before/after P3 where meaningful.

Any increase caused solely by TypeOnly declaration presence requires explanation.

---

## 59. Source census remains exact

The existing native/reflection census verifies:

```text
source native selectors == generated surface descriptors == runtime descriptors
```

P3 must adapt this certification to intrinsic data source anchors without weakening it.

Required:

```text
every runtime native descriptor has canonical source owner where specified
intrinsic runtime data anchors map to canonical UniverseKey
type-only domains do not enter runtime descriptor census
native members moved into impl remain source-indexed
```

---

# Part XXV — LSP rename architecture analysis

## 60. Rename can reuse existing compiler-owned source identity

The live LSP already exposes:

```text
go-to-definition
references
compiler target lookup
compiler reference locations
```

Therefore C3 rename should be implemented as a thin mutation projection over the existing
semantic target/reference index rather than a new resolution engine.

Conceptually:

```text
cursor
    -> canonical SemanticTargetId
    -> declaration source site + reference sites
    -> validate editability/name category
    -> WorkspaceEdit
```

This is architecturally bounded.

### 60.1 No text-search fallback

If semantic target resolution fails, rename should fail/decline.

Do not rename all matching strings.

### 60.2 C3-specific target completeness

P2/P3 must ensure `SemanticTargetId` or equivalent can identify the new static-domain
categories.

If rename requires adding a missing target ID, add it to the semantic identity model,
not as an LSP-only ID.

---

# Part XXVI — Enum retirement impact map

## 61. Expected front-end changes

Final removal likely affects:

```text
phalcom-ast/src/token.rs
phalcom-ast/src/lexer.rs
phalcom-ast/src/parser.rs
phalcom-ast/src/ast.rs
phalcom-ast/src/error.rs

phalcom-ast/tests/lexer.rs
phalcom-ast/tests/enum_syntax.rs
phalcom-ast/tests/data_syntax.rs
phalcom-ast/tests/parser.rs
```

`enum_syntax.rs` may be converted into a removal/negative suite rather than deleted
without replacement.

---

## 62. Expected semantic/tooling cleanup

Likely:

```text
phalcom-semantic/src/source_index/**
phalcom-semantic/src/core_surface/source.rs
phalcom-semantic/src/semantic_shard.rs
phalcom-semantic/src/declarations.rs

phalcom-lsp/src/semantic_tokens.rs
phalcom-lsp/src/hover.rs
phalcom-lsp/src/completion.rs
phalcom-lsp/src/backend.rs
tools/vsphalcom/syntaxes/phalcom.tmLanguage.json
editor golden fixtures
```

Private runtime files such as:

```text
enum_decl.rs
enum_semantics.rs
VariantId
```

need not be renamed merely for terminology.

---

# Part XXVII — C3 final certification matrix

## 63. P3 is a broader certification plan

P1/P2 use focused BUILD/STABILIZE testing.

P3 closes the checkpoint and therefore should deliberately run a broader, C3-specific
matrix.

The goal is not “run everything blindly”.

The goal is to prove every C3 acceptance invariant across the full affected stack.

---

## 64. Syntax and algebraic-shape certification

Required final cases:

```text
data tuple product
data record product
data singleton sum
data comma singleton shorthand
data tuple-payload sum
data record-payload sum
data explicit zero-arg constructor
data empty product
data empty sum
GADT data sum
type data equivalents
intrinsic data facades
intrinsic type data facades
mixed-body rejection
enum declaration rejection
```

---

## 65. Identity certification

Required:

```text
two structurally equal runtime data declarations remain nominally distinct
two structurally equal type-data declarations remain distinct domains
equal static terms canonicalize
unequal constructors/domains do not
associated domain specialization includes owner environment
Option/Result/Ordering identities survive source migration
TupleEntries maps to CompleteTypePack
```

---

## 66. ADT/GADT behavior certification

Re-run focused mature behavior:

```text
exact-case construction
exact-case impl
case-specific conformance
GADT result specialization
branch refinement
exhaustive matching
non-exhaustive diagnostics
redundant branch diagnostics
root requirement/default completeness
case visibility
```

Migration to `data` must not regress any of these.

---

## 67. Static application/inference certification

Re-run representative P2 examples:

```phalcom
type data Shape {
    Scalar
    Vector(_ size: Int)
}

data Tensor<Element: Type, S: Shape>()

type V: Type = Tensor<Float, Shape::Vector(3)>
```

Required:

```text
wrong-domain rejection
ConstInt argument
associated-domain argument
RecordRow argument
pack arguments
partial application
static aliases
first-order inference
constructor/domain refutation
symbolic preservation
```

---

## 68. Associated-domain certification

Required:

```phalcom
class Parser {
    type data State {
        Start
        Done
    }

    type Initial: State = State::Start
}
```

Certify:

```text
Parser::State
Parser::State::Done
Parser::Initial
generic-owner associated domain
collision behavior
trait-associated type distinction
```

---

## 69. Structural-domain certification

Required:

```text
TypePack source anchor
LabeledTypePack source anchor
TupleEntries source anchor
RecordRow source anchor

Tuple::Entries
Tuple::Positionals
Tuple::Labeled
Tuple::At
Tuple::Map

TypePack::Map
LabeledTypePack::Map
RecordRow::Map
Record::Map
```

and equivalences:

```text
*T   == T::Positionals
**T  == T::Labeled
***T == T::Entries
```

---

## 70. Persistence/import certification

Required:

```text
metadata round trip
schema validation
cross-module type-data import
cross-module associated domain
intrinsic static-domain import/reference
stable source declaration references
no raw IDs
```

---

## 71. Incremental certification

Required final-state cold parity for the matrix in Part XXI.

---

## 72. Tooling certification

Required LSP cases:

```text
hover
definition
references
completion
rename
diagnostics
semantic tokens
workspace symbols where relevant
```

over:

```text
data root
data case
type data
type-data case
associated type data
TupleEntries
Tuple::Entries
removed enum syntax diagnostic
```

---

## 73. Runtime certification

Required:

```text
runtime product behavior unchanged
runtime sum behavior unchanged
special Option representation unchanged
Tuple/Record specialized runtime representation unchanged
Unit identity unchanged
no static runtime objects
no static VM solver
```

---

# Part XXVIII — Final validation breadth

## 74. Mandatory final compile breadth

Because P3 removes a public AST/token compatibility surface and migrates Universe/tooling,
one final whole-workspace compile/check is justified.

Recommended mandatory closure compile gate:

```sh
cargo fmt --all -- --check
RUSTFLAGS='' RUSTC_WRAPPER='' cargo build --workspace --all-targets
```

or a workspace `check --all-targets` if repository policy prefers it, followed by
explicit builds required for extension/LSP integration.

This does **not** imply release certification.

---

## 75. Mandatory C3 test breadth

The final P3 plan should include at least:

```text
phalcom-ast full crate tests

phalcom-type-meta full crate tests

phalcom-semantic:
    C3 algebraic/data suites
    type-model/type-member/static application suites
    inference/substitution
    associated resolution
    metadata
    source index
    incremental

phalcom-core:
    Universe source/native census
    algebraic-data runtime suites
    native ADT runtime
    Option/Result/Ordering behavior
    Tuple/Record/Unit behavior
    relevant language corpus

phalcom-lsp:
    full crate tests plus C3-specific requests

VS Code extension:
    grammar/unit or e2e lane if maintained and available
```

---

## 76. Workspace test vs checkpoint certification

`cargo test --workspace --all-targets` is valuable but is not automatically required to
claim:

```text
C3 COMPLETE / IMPLEMENTED / FOCUSED_TESTED
```

because the repository has known unrelated baseline failures and checkpoint closure is
not release certification.

Recommended policy:

### C3 closure mandatory

```text
workspace build/check
all C3-owned focused/broad affected suites
C3-specific corpus gate
LSP/tooling gate
metadata/incremental gate
runtime-negative audit
```

### Optional release-strength evidence

```text
cargo test --workspace --all-targets
cargo clippy --workspace --all-targets -- -D warnings
```

If run, classify unrelated failures.

Passing them may support a stronger release record, but P3 must not silently label C3
`RELEASE_COMPLETE` unless the repository release convention explicitly permits it.

---

# Part XXIX — Baseline failure handling at closure

## 77. Inherited known baselines

Historically C2 handed off:

```text
BASELINE-001
semantic::capabilities::traits::
trait_default_replays_external_nominal_callable_dependencies

BASELINE-002
semantic::capabilities::traits::
trait_defaults_reject_unknown_members_storage_and_super_without_concrete_capabilities
```

P3 must read the completed P2 handoff for the current baseline ledger.

Do not assume these exact failures still exist.

---

## 78. Closure classification

Unexpected final failures remain classified:

```text
A — definitely caused by C3
B — probably caused by C3
C — unclear
D — unrelated/baseline
```

### A/B

C3 cannot close until fixed or architecturally escalated.

### C

Perform one bounded causal classification.

If still unclear and the failing surface overlaps C3 acceptance, closure remains blocked.

If demonstrably outside C3 acceptance, record and defer.

### D

Record in checkpoint and continue.

Do not weaken C3 tests to hide baseline failures.

---

# Part XXX — Performance/resource requirements

## 79. Required complexity evidence

C3 closure must preserve:

```text
case duplicate detection
    expected O(n)

static term interning
    hash-consed

StaticArgument application
    proportional to argument graph

TupleEntries projection
    same order as CompleteTypePack projection

associated lookup
    indexed owner/base

metadata
    graph traversal, no stringify/reparse

incremental
    dependency-directed

pack terms
    C2 flatness
```

---

## 80. Universe bootstrap evidence

Record and compare:

```text
number of discovered units
number of root-reachable units
number of executed units
```

No major increase should be caused solely by static declaration facades.

If migration moves methods from class declarations to impl blocks, bootstrap execution
semantics should remain behaviorally equivalent.

---

# Part XXXI — Security/soundness boundary audit

## 81. ProperTypeId

Final C3 audit must prove:

```text
TypePack term
LabeledTypePack term
TupleEntries term
RecordRow
type-data term
associated type-data term
```

cannot enter runtime proper-Type APIs unless the operation specifically yields
`KindId::TYPE`.

---

## 82. Static identity

No final static identity may depend on:

```text
runtime object address
runtime hash
runtime equality
mutable state
environment variables
filesystem
clock
randomness
network
native runtime call
```

---

## 83. Stable IDs

No query-local/session-local:

```text
InferVarId
TypeId
StaticDomainId
RecordRowId
```

may be serialized as portable identity.

---

# Part XXXII — High-risk migration seams

## 84. Intrinsic `Tuple` is not ordinary data storage

Severity: critical.

Failure mode:

```text
intrinsic data Tuple<***Es>
    accidentally lowers through general DataObject constructor
```

Prevention:

```text
explicit source-facade -> existing structural Tuple mapping
runtime-negative construction tests
```

---

## 85. `TupleEntries` accidentally duplicates CompleteTypePack

Severity: critical.

Failure mode:

```text
KindData::TupleEntries
TypeData::TupleEntries
new solver
```

Prevention:

```text
source facade maps to KindId::COMPLETE_TYPE_PACK
Tuple::Entries uses project_complete
metadata reuses CompleteTypePack nodes
```

---

## 86. Intrinsic static domains accidentally become runtime classes

Severity: critical.

Failure mode:

```text
UniverseKey::TypePack
VM class TypePack
runtime object instances
```

Prevention:

```text
semantic-only intrinsic static-domain catalog
no class relation
bootstrap-root test
```

---

## 87. Enum removal happens before corpus migration

Severity: high.

Failure mode:

```text
Universe/examples/tests stop parsing en masse
```

Prevention:

```text
migrate source first
run census
then remove compatibility parser
```

---

## 88. Enum migration changes stable identity

Severity: critical.

Failure mode:

```text
Option DeclarationId/cases/native associations change
```

Prevention:

```text
same module/name/selector identity
semantic fingerprint checks
native census checks
```

---

## 89. Native source census assumes only classes

Severity: high.

Failure mode:

```text
Tuple/Record/Unit no longer attach native members/runtime class relation
```

Prevention:

```text
generalize source presentation categories
keep UniverseKey association
index impl-owned native members
```

---

## 90. TypeOnly declarations become runtime bootstrap roots

Severity: high.

Failure mode:

```text
adding TypePack source declaration increases executed Universe modules
```

Prevention:

```text
bootstrap root policy aware of runtime-vs-TypeOnly declaration phase
```

---

## 91. Tooling recreates enum/static identity from text

Severity: high.

Failure mode:

```text
hover/rename/definition disagree with compiler
```

Prevention:

```text
semantic source targets only
```

---

## 92. Rename grows into second semantic engine

Severity: high.

Failure mode:

```text
LSP parses/re-resolves names itself
```

Prevention:

```text
canonical SemanticTargetId + ReferenceIndex -> WorkspaceEdit
```

---

# Part XXXIII — P3 requirements by implementation domain

## 93. Front-end requirements

P3 must support/finalize:

```text
bodiless intrinsic data/type-data facade declarations
final data sum syntax only
final enum rejection
intrinsic structural generic binder surface
```

The parser must not infer structural semantics from names alone.

Intrinsic registration validates the declaration.

---

## 94. Semantic declaration requirements

P3 must attach intrinsic facades to existing semantic owners.

The algebraic declaration authority should expose enough information for:

```text
phase
shape/presentation
implementation provenance
intrinsic structural binding
source identity
```

without constructing duplicate structural terms.

---

## 95. TypeStore requirements

P3 should need minimal TypeStore semantic additions:

```text
Tuple Entries projection convenience/intrinsic op
public-domain presentation mapping
intrinsic source declaration association
```

A large new term representation is a predecessor failure.

---

## 96. Compiler requirements

Compiler changes should:

```text
recognize intrinsic runtime structural data anchors
preserve existing Tuple/Record/Unit lowering
ignore TypeOnly facades for runtime codegen
consume final algebraic semantic products
```

No new static executor.

---

## 97. Runtime requirements

Runtime should mostly remain unchanged.

Expected changes are:

```text
source/native presentation association
possibly reflection labels/descriptors
```

not:

```text
object layout migration
bytecode redesign
GC model redesign
```

---

## 98. Metadata requirements

P3 may add source-facade/reflection association records.

It must not create new semantic term encodings for aliases of existing C2 domains.

---

## 99. Source-index requirements

Source index must represent final categories and targets.

No `Enum` category as active language declaration.

---

## 100. LSP requirements

Use compiler semantic snapshot for:

```text
hover
definition
references
completion
rename
diagnostics
semantic tokens
```

---

# Part XXXIV — Recommended P3 plan decomposition

## 101. Why P3 should be split internally into coherent tasks

Although P3 is one numbered plan, it should not be implemented as one migration sweep.

A safe dependency order is:

```text
A. predecessor certification + migration census
B. intrinsic static-domain facade registration
C. TupleEntries + structural type-member anchors
D. Unit/Tuple/Record intrinsic runtime facade migration
E. Option/Result/Ordering + remaining active enum source migration
F. tooling/source-index/rename/terminology closure
G. final enum parser retirement
H. authoritative spec closure
I. metadata/incremental/runtime/resource certification
J. checkpoint closure records
```

The patch-grade plan should preserve this ordering.

---

## 102. Why parser retirement must come late

Removing `enum` early would break the corpus before semantic migration is complete.

Therefore:

```text
migrate source
    ↓
prove active-source census clean
    ↓
remove accepted enum syntax
    ↓
convert old enum syntax tests to rejection tests
    ↓
run final corpus certification
```

---

## 103. Why intrinsic source facades must come before Tuple/Record migration

Tuple/Record class-to-data migration should happen only after the semantic compiler knows
how to bind:

```text
intrinsic data declaration
    ->
existing structural runtime/type authority
```

Otherwise the migration risks temporarily creating ordinary DataInfo/DataObject semantics.

---

# Part XXXV — Final checkpoint closure contract

## 104. Required plan lifecycle state

At successful P3 completion:

```yaml
LANG007.C3.P1:
  status: COMPLETE
  completion: IMPLEMENTED
  verification: FOCUSED_TESTED

LANG007.C3.P2:
  status: COMPLETE
  completion: IMPLEMENTED
  verification: FOCUSED_TESTED

LANG007.C3.P3:
  status: COMPLETE
  completion: IMPLEMENTED
  verification: FOCUSED_TESTED
```

Then and only then may:

```yaml
LANG007.C3:
  status: COMPLETE
  completion: IMPLEMENTED
  verification: FOCUSED_TESTED
```

be recorded.

---

## 105. `RELEASE_COMPLETE` is separate

C3 checkpoint closure does not automatically mean:

```text
verification: RELEASE_COMPLETE
```

Release completion requires the repository's broader release evidence.

P3 must not overclaim.

---

## 106. C3 closure checklist

C3 closes only if all are true.

### Language surface

```text
[ ] data is canonical runtime Product/Sum keyword
[ ] type data is canonical TypeOnly static-domain form
[ ] enum declaration syntax no longer accepted
[ ] intrinsic data/type-data facades are source-visible
```

### Static domains

```text
[ ] TypePack facade maps to TypePack authority
[ ] LabeledTypePack facade maps to LabeledTypePack authority
[ ] TupleEntries facade maps to CompleteTypePack
[ ] RecordRow facade maps to row authority
[ ] no duplicate static domains created
```

### Structural runtime data

```text
[ ] Unit facade preserves Unit identity/runtime
[ ] Tuple facade preserves structural Tuple identity/runtime
[ ] Record facade preserves RecordRow-backed identity/runtime
[ ] no generic DataObject migration
```

### Type members

```text
[ ] Tuple::Entries exists
[ ] Entries == complete pack projection
[ ] Positionals/Labeled/At/Map preserved
[ ] structural domain Map members preserved
[ ] source anchors use canonical member identity
```

### Runtime sums

```text
[ ] Option migrated
[ ] Result migrated
[ ] Ordering migrated
[ ] all active Universe enums migrated
[ ] active first-party .ph corpus migrated
[ ] exact-case/GADT semantics unchanged
```

### P2 semantics

```text
[ ] heterogeneous application still passes
[ ] static inference still passes
[ ] aliases/type members still pass
[ ] associated type data still passes
[ ] trait unsupported boundary still explicit
```

### Persistence

```text
[ ] metadata round trip passes
[ ] stable declaration identity passes
[ ] interface fingerprints are intentional
[ ] no raw IDs
```

### Incremental

```text
[ ] cold/incremental parity passes
[ ] enum->data migration causes no semantic identity churn
```

### Tooling

```text
[ ] source-index final categories
[ ] hover
[ ] definition
[ ] references
[ ] completion
[ ] rename
[ ] diagnostics
[ ] semantic tokens
[ ] VS Code grammar
```

### Specs

```text
[ ] docs/specs language ownership map coherent
[ ] no active authoritative enum-as-keyword specification remains
[ ] type-level/type-member/Universe specs reflect final C3
```

### Runtime/resource

```text
[ ] no runtime TypeOnly inhabitants
[ ] no static VM solver/opcodes
[ ] Tuple/Record/Option representations preserved
[ ] Universe bootstrap resource evidence acceptable
[ ] ordinary static operations remain bounded
```

### Lifecycle

```text
[ ] P3 walkthrough complete
[ ] P3 handoff complete
[ ] C3 checkpoint record truthful
[ ] PROGRAM.md / STATUS.md truthful
[ ] next LANG007 action recorded
```

---

# Part XXXVI — Required P3 walkthrough/handoff content

## 107. P3 walkthrough

The final walkthrough must record:

```text
final revision
intrinsic static-domain declarations added
intrinsic runtime structural declarations migrated
TupleEntries mapping
structural member registrations
Universe closed-sum migrations
remaining active enum census
parser/lexer compatibility removal
source-index/LSP changes
rename implementation
VS Code grammar changes
specification migration
metadata/schema changes, if any
incremental evidence
runtime-negative evidence
Universe bootstrap/resource evidence
commands actually run
baseline failures
consultations/amendments
final P3 classification
final C3 classification
```

---

## 108. P3 handoff

Because P3 closes C3, the handoff is not P3→P4.

It should freeze the C3 substrate for the next accepted LANG007 work.

It must state:

```text
data/type-data final syntax
canonical term/domain architecture
TupleEntries mapping
StaticArgument contract
associated-domain contract
final enum compatibility state
metadata schema state
runtime representation boundaries
tooling/source-index state
known deferred baselines
next accepted LANG007 objective
```

---

# Part XXXVII — STOP / CONSULT triggers

## 109. Mandatory P3 escalation conditions

Stop and consult if any of the following occurs.

1. Completed P2 does not satisfy the hard predecessor contract.
2. `TupleEntries` cannot map exactly onto CompleteTypePack semantics.
3. Exposing TypePack/LabeledTypePack/TupleEntries requires new canonical term variants.
4. Exposing RecordRow requires moving row identity into TypeId.
5. `intrinsic data Tuple` requires ordinary generic DataObject representation.
6. `intrinsic data Record` requires replacing `TypeData::Record`.
7. `intrinsic data Unit` creates a second Unit type identity.
8. Intrinsic static-domain source anchors appear to require VM classes.
9. TypeOnly source declarations force runtime initialization and no local bootstrap fix
   preserves the accepted module architecture.
10. Moving Tuple/Record/Unit behavior to impl blocks changes runtime/native member identity.
11. Option/Result/Ordering migration changes declaration/case/native identity.
12. Removing enum compatibility breaks accepted data/GADT behavior after one serious
    correction.
13. Final enum lexical status conflicts with an authoritative accepted lexical decision.
14. LSP rename cannot be implemented from canonical source/reference targets without a
    second semantic resolver.
15. Source-index identity for an associated/static target is missing because P2 failed to
    publish one.
16. Stable metadata needs a new static term representation solely for TupleEntries.
17. Runtime representation must be redesigned to satisfy source facade semantics.
18. A VM static operation becomes necessary.
19. Cold/incremental equality cannot be restored without global invalidation.
20. A C1/C2/C3 invariant must be weakened for tests to pass.
21. Two plausible intrinsic-domain registration architectures have materially different
    semantic ownership consequences.
22. The same nontrivial semantic failure survives one serious correction.
23. Authoritative specs and accepted C3 decisions materially conflict.
24. Checkpoint closure would require claiming unrun verification as passed.

---

# Part XXXVIII — Non-goals

## 110. P3 must not expand into

```text
release-wide unrelated refactoring
renaming every Rust Enum/Variant identifier
runtime type-data reification
general compile-time evaluator
trait-owned type-data requirements
trait generic static-domain redesign beyond P2 boundary
new pack features
new row solver
new Tuple/Record runtime storage
new Option runtime representation
GC redesign
VM bytecode cleanup unrelated to C3
global historical documentation rewrite
formatter/linter redesign unless required for enum removal
```

---

# Part XXXIX — Requirements verdict

## 111. P3 is primarily an integration/closure plan, but it still contains real feature work

The remaining feature work is:

```text
intrinsic structural source facades
TupleEntries source/member surface
final enum syntax removal
semantic rename tooling
```

Everything else should mostly consume P1/P2.

If P3 becomes dominated by new type-system architecture, something is wrong upstream.

---

## 112. Recommended architecture for intrinsic static domains

Use:

```text
canonical source declaration
    +
intrinsic semantic binding descriptor
    ->
existing compiler domain
```

not:

```text
canonical source declaration
    ->
new nominal StaticDomain
    ->
adapter back to existing domain
```

The latter creates duplicate identity.

---

## 113. Recommended architecture for intrinsic runtime structural data

Use:

```text
intrinsic data source facade
    ->
existing structural type/runtime owner
    ->
existing specialized compiler/runtime lowering
```

not:

```text
intrinsic data source facade
    ->
ordinary DataInfo constructor
    ->
generic DataObject
```

---

## 114. Recommended final enum policy

Final language acceptance should contain no `enum` declaration syntax.

Prefer true lexical retirement if no contrary lexical decision exists.

Convert legacy syntax tests into explicit rejection/migration tests.

Keep private runtime Enum/Variant naming where harmless.

---

## 115. Recommended certification strength

For C3 closure:

```text
workspace compile/build once
+
broad C3-specific semantic/runtime/tooling corpus
+
metadata/incremental parity
+
runtime-negative/resource audits
```

is appropriate.

Do not require unrelated workspace test/clippy perfection merely to call C3
`FOCUSED_TESTED`.

Do not call C3 `RELEASE_COMPLETE` without release evidence.

---

# Part XL — Final C3 architecture after closure

## 116. Static/domain tower

```text
KindId
├─ Type
├─ RecordRow
├─ TypePack
├─ LabeledTypePack
├─ CompleteTypePack   <- public facade: TupleEntries
├─ Arrow(...)
└─ StaticDomain(id)   <- user/nominal type data domains
```

```text
TypeId
├─ proper Types
├─ packs
├─ complete packs
├─ static constants
├─ type-data terms
├─ symbolic projections
└─ heterogeneous Applied terms

RecordRowId
└─ dedicated row identity
```

---

## 117. Algebraic source model

```text
data
├─ Product
└─ Sum
    ├─ singleton cases
    ├─ tuple-payload cases
    ├─ record-payload cases
    ├─ zero-arg constructor cases
    └─ GADT cases

type data
├─ nominal static algebra
└─ intrinsic compiler structural facade
```

---

## 118. Structural facade model

```text
intrinsic type data TypePack
    -> KindId::TYPE_PACK

intrinsic type data LabeledTypePack
    -> KindId::LABELED_TYPE_PACK

intrinsic type data TupleEntries
    -> KindId::COMPLETE_TYPE_PACK

intrinsic type data RecordRow
    -> KindId::RECORD_ROW
```

```text
intrinsic data Unit()
    -> canonical Unit

intrinsic data Tuple<***Es>
    -> structural Tuple type family + Tuple runtime class

intrinsic data Record<R: RecordRow>
    -> structural Record type + Record runtime class
```

---

## 119. One associated/static member model

```text
Tuple::Entries
Tuple::Positionals
Tuple::Labeled
Tuple::At
Tuple::Map

TypePack::Map
LabeledTypePack::Map
RecordRow::Map
Record::Map
```

all resolve through canonical type-member identity and compiler normalization.

---

## 120. One current sum terminology

```text
data Result<T, E> {
    Ok(_ value: T)
    Error(_ error: E)
}
```

No current source `enum` category.

Runtime may still internally use ADT/variant implementation terms.

---

## 121. One closure criterion

C3 is done when every layer says the same thing:

```text
parser
AST
semantic identity
type store
inference
associated lookup
metadata
incremental DB
Universe source
compiler/runtime
source index
LSP
active specs
```

and no layer has to reconstruct or reinterpret the C3 model independently.

That is the core P3 requirement.
''').lstrip()

out.write_text(doc, encoding="utf-8")
print(f"Wrote {out}")
print(f"{len(doc.splitlines())} lines; {len(doc):,} characters")
