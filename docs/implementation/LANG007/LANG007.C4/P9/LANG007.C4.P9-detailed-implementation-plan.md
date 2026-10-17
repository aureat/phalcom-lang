---
id: LANG007.C4.P9
category: LANG
program: LANG007
checkpoint: LANG007.C4
kind: implementation-plan
title: "LANG007.C4.P9 — Public Ontology Migration, Tooling, Documentation, Performance Certification, and Checkpoint Closure"
status: COMPLETE
completion: IMPLEMENTED
verification: FOCUSED_TESTED
repository: aureat/phalcom-lang
branch: main
prepared_against_revision: cb8a068cd4ed7db2b9d17c0a4e096839c2a90ac0
prepared: 2026-10-13
requires:
  - LANG007.C4.P4
  - LANG007.C4.P5
  - LANG007.C4.P6
  - LANG007.C4.P7
  - LANG007.C4.P8
inherits:
  - LANG007.C4.P10
  - LANG007.C4.P11
  - LANG007.C4.P12
  - LANG007.C4.P13
  - LANG007.C4.P14
  - LANG007.C4.P15
  - LANG007.C4.P16
handoff_to:
  - LANG007.C4 checkpoint closure
supersedes: null
---

# LANG007.C4.P9 — Detailed Implementation Plan

## Public Ontology Migration, Tooling, Documentation, Performance Certification, and Checkpoint Closure

---

# 0. Executive implementation directive

`LANG007.C4.P9` is the final convergence and certification plan for the C4 Structural Static Upgrade.

P9 does **not** design a new semantic subsystem.

It consumes the architecture established by P1–P8 and proves that the entire repository presents, tools, documents, persists, executes, and benchmarks that architecture as one coherent language model.

The plan therefore has five primary implementation goals:

```text
1. converge public structural ontology;
2. replace obsolete Record-row reflection presentation with Product/UnorderedShape;
3. prove representative structural algorithms can live in ordinary Telos source;
4. converge tooling/docs/generated surfaces on the same semantic authority;
5. perform broad correctness + performance certification and close C4 lifecycle truth.
```

The plan must preserve the ratified `INC-P3-2` amendment.

The following are **canonical and must remain** unless a new architecture decision explicitly supersedes them:

```text
TypePack
LabeledTypePack
CompleteTypePack
```

as C3 compiler/static kinded domains.

The current `RecordFields` name likewise remains valid in its established **Record-context binder-domain** role unless a new decision changes that role.

The migration target is **not** "delete every symbol containing TypePack or RecordFields".

The migration target is the obsolete **competing reflection ontology**, principally:

```text
Record::Fields
RecordFields::Map     // as a public reflection facade
RecordRow terminology exposed as public structural reflection
stale tooling/docs/examples that teach the row model as the canonical reflection API
```

The canonical public unordered reflection model is:

```text
UnorderedProduct
UnorderedShape
FieldKey
FieldInfo
Presence
ProductField
UnorderedTail
FieldLookup
UnorderedShapeView
```

The implementation rule for all P9 work is:

> **Retire competing presentation; preserve valid semantic machinery.**

---

# 1. Prepared-against repository state

## 1.1 Revision

This plan was prepared against:

```text
repository: aureat/phalcom-lang
branch:     main
revision:   cb8a068cd4ed7db2b9d17c0a4e096839c2a90ac0
subject:    fix(semantic): validate static residual specialization
```

Before starting implementation, T0 must re-read `main` and either:

- confirm this revision or a descendant whose relevant interfaces still match; or
- rebase the write set and acceptance gates.

Do not apply this plan mechanically to a materially different P7/P8 tree.

## 1.2 Live C4 plan state at preparation

The checkpoint currently records:

```text
P1   COMPLETE / IMPLEMENTED / BASELINE_BLOCKED
P2   COMPLETE / IMPLEMENTED / FOCUSED_TESTED
P3   COMPLETE / IMPLEMENTED / FOCUSED_TESTED
P4   IN_PROGRESS / PARTIAL / UNVERIFIED
P10  COMPLETE / IMPLEMENTED / FOCUSED_TESTED
P5   COMPLETE / IMPLEMENTED / FOCUSED_TESTED
P6   COMPLETE / IMPLEMENTED / FOCUSED_TESTED
P7   IN_PROGRESS / PARTIAL / UNVERIFIED
P8   PROPOSED / NOT_STARTED / UNVERIFIED
P9   PROPOSED / NOT_STARTED / UNVERIFIED
P11–P16 complete/focused-tested with individual lifecycle qualifications
```

Therefore this plan is executable only after its predecessor gate is satisfied.

---

# 2. Architectural authorities P9 must preserve

P9 must reuse these authorities.

## 2.1 Type/domain authority

```text
TypeStore
KindId
StaticValueDomain
TypeMemberDomain
TypePack / LabeledTypePack / CompleteTypePack kind identities
RecordFields / RecordRow kind identity where applicable
```

P9 must not create a new "public-only" kind system.

## 2.2 Structural row authority

```text
RecordRow
RecordRowId
row solver
canonical FieldKey ordering
row relation/inference machinery
```

These may remain internal after public ontology cleanup.

## 2.3 Reflection authority

```text
Product
OrderedProduct
UnorderedProduct
Sum
ProductShape
OrderedShape
UnorderedShape
Variant
Variants
```

P9 must not build a second reflection database in tooling.

## 2.4 Static evaluation authority

```text
StaticValue
StaticResidual
bounded evaluator
P6 static control
P7 recursion/memoization/specialization
```

Source-defined helpers must use this path.

## 2.5 Durability authority

```text
P8 metadata schema
static-definition products
static-value/residual metadata DAG
materialization descriptor
artifact-only import/specialization
incremental semantic database
```

P9 certifies these products; it does not redesign them.

## 2.6 Tooling authority

LSP/REPL/editor tooling must consume canonical semantic products and source indices.

No tooling-specific reconstruction of:

```text
Record rows
Product capabilities
static constants
type-member result domains
```

is allowed.

---

# 3. P9 scope

P9 owns:

```text
ontology census refresh
public reflection surface migration
Universe source convergence
public compatibility disposition
source-defined structural/reflection helpers
LSP integration closure
REPL closure
formatter/printer closure where repository provides such authority
VS Code/TextMate/generated table convergence
diagnostic/presentation terminology cleanup
spec/ADR/doc/example convergence
fuzz/property regression closure
metadata/incremental end-to-end certification
runtime/static performance certification
broad workspace certification
residual failure classification
checkpoint/program/status lifecycle closure
```

P9 explicitly does **not** own:

```text
new Pack representation
new FieldKey semantics
new Product/Sum identity
new type-member result-domain architecture
new static evaluator
new residual representation
new pattern matcher
new proof engine
new metadata identity model
new materializer architecture
new compiler call semantics
```

If P9 reaches one of those boundaries, STOP AND CONSULT.

---

# 4. Non-negotiable invariants

## P9-INV-01 — `INC-P3-2` is authoritative

The following remain canonical kinded domains:

```text
TypePack
LabeledTypePack
CompleteTypePack
```

P9 must not delete, rename, or alias them to proper `Pack` specializations.

## P9-INV-02 — Pack proper types are distinct from kinded Pack domains

```text
PositionalPack<Type> != TypePack
LabeledPack<Type>    != LabeledTypePack
CompletePack<Type>   != CompleteTypePack
```

They may correspond structurally but are not the same semantic category.

## P9-INV-03 — Record binder semantics are not reflection semantics

`RecordFields` may remain a valid context-specific generic binder domain.

Its presence does not justify exposing `RecordFields` as the canonical public reflection model.

## P9-INV-04 — `RecordRow` remains internal

No new public docs/tooling APIs expose `RecordRow` or `RecordRowId` as user structural reflection concepts.

## P9-INV-05 — canonical unordered reflection is `UnorderedShape`

Public introspection flows through:

```text
UnorderedProduct::UnorderedShape
UnorderedShape::View
UnorderedShape::Lookup<K>
UnorderedShape::Tail
```

or equivalent canonical members supplied by P5.

## P9-INV-06 — public cleanup does not delete optimized internals

Row IDs, TypePack nodes, Pack backing IDs, compiler caches, and metadata pool nodes may remain private optimizations.

## P9-INV-07 — source-defined algorithms remain source-defined

Do not move ordinary recursive structural algorithms into Rust/native because implementation is easier there.

## P9-INV-08 — no compile-time VM

P9 may not reintroduce bytecode/runtime execution for `const` or static definitions.

## P9-INV-09 — fixed calls remain Pack-allocation-free

Ordinary fixed calls must not allocate a `Pack` merely because `Pack` is a first-class language type.

## P9-INV-10 — Pack rest/view remains O(1)

Repeated Pack slicing/view decomposition must not copy the remainder.

## P9-INV-11 — tooling uses semantic identity

Hover/navigation/completion must target canonical source declarations and semantic result domains.

## P9-INV-12 — historical docs do not override current authority

Old C4 handoffs may remain as history but must be clearly marked as superseded where `INC-P3-2` changed the ontology.

## P9-INV-13 — broad certification is actual evidence

Zero-selected tests, stale historic runs, or "appears fine" are not final P9 evidence.

## P9-INV-14 — lifecycle truth is not aspirational

C4 is marked complete only after every C4-owned unresolved item has a real final disposition.

---

# 5. Execution topology

The recommended serialized order is:

```text
T0  Freeze P9 base + predecessor readiness
 |
 v
T1  Refresh ontology / public-surface census
 |
 v
T2  Freeze compatibility and cutover decisions
 |
 +-----------------------+
 |                       |
 v                       v
T3 Universe source       T4 semantic public-member cleanup
 |                       |
 +-----------+-----------+
             |
             v
T5 source-defined structural helper certification
             |
             v
T6 LSP semantic/tooling convergence
             |
             v
T7 REPL / formatter / VS Code convergence
             |
             v
T8 docs/spec/ADR/example convergence
             |
             v
T9 fuzz/property/regression closure
             |
             v
T10 performance baseline + certification
             |
             v
T11 broad workspace/checkpoint certification
             |
             v
T12 residual failure classification / repair loop
             |
             v
T13 lifecycle closure
```

T3 and T4 may be implemented in one patch if the public declaration removal and semantic operation cleanup are mechanically coupled.

Do not combine T10–T13 prematurely; performance and broad correctness need independent evidence.

---

# 6. T0 — Freeze the live P9 base and prove predecessor readiness

## Objective

P9 must start from a state where P4, P7, and P8 have supplied the semantics P9 is expected to integrate.

## Read set

```text
docs/implementation/LANG007/LANG007.C4/LANG007.C4-CHECKPOINT.md
docs/implementation/LANG007/LANG007.C4/LANG007.C4-GUIDANCE.md
docs/implementation/LANG007/PROGRAM.md
docs/implementation/LANG007/STATUS.md

P4 walkthrough/handoff
P5 walkthrough/plan
P6 walkthrough/plan
P7 walkthrough/handoff/plan
P8 walkthrough/handoff/plan
P10 ontology census + walkthrough/handoff
P11–P16 lifecycle records
```

## Mechanical repository checks

Record:

```bash
git status --short
git rev-parse HEAD
git rev-parse origin/main
git log -1 --oneline
```

No P9 production edit should start from an unidentified dirty state.

## Predecessor acceptance checklist

### P4

Must be able to demonstrate:

```text
general FieldKey semantics complete
unordered row operations use canonical FieldKey
patterns/relations/runtime shape agree
```

### P7

Must be able to demonstrate:

```text
recursive static applications
normalized residualization
memo reuse
specialization
representative generic structural recursion
```

### P8

Must be able to demonstrate:

```text
cross-module static product import
durable static values/residuals
artifact-only specialization
materialization without initializer re-execution
cold/incremental parity
```

## STOP conditions

Stop before T1 production migration if:

```text
P4 still has unresolved C4-owned unordered-shape semantics
P7 cannot run the planned source helper
P8 cannot preserve the helper/const across module/artifact boundaries
```

## Gate G0

Pass only when the predecessor matrix is written into the P9 plan/walkthrough or checkpoint and no required semantic dependency is ambiguous.

---

# 7. T1 — Refresh the ontology and public-surface census

## Objective

Re-run the P10 legacy ontology census against the actual P9 base.

Do not reuse the old census blindly.

## Search set

At minimum:

```bash
rg -n \
  "TypePack|LabeledTypePack|CompleteTypePack|RecordFields|RecordRow|Record::Fields|RecordFields::Map|PositionalTypePackView|LabeledTypePackView|RecordProduct|RecordField|FieldPresence|TypePackView" \
  phalcom-ast \
  phalcom-semantic \
  phalcom-type-meta \
  phalcom-modules \
  phalcom-core \
  phalcom-lsp \
  phalcom-repl \
  tools \
  docs
```

Also search:

```bash
rg -n "Positionals|LabeledEntries|PositionalEntries|ProductShape|UnorderedShape|FieldInfo|UnorderedTail|VariantNames" .
```

## Required classification

Every hit relevant to current semantics must be assigned one category:

```text
A  canonical public structural API
B  canonical public kind/binder domain
C  compatibility facade
D  obsolete public API
E  tooling/test regression guard
F  internal implementation
G  historical/superseded documentation
H  unrelated coincidental spelling
```

## Required initial classifications

The live repository already establishes these defaults:

| Symbol | Default classification |
|---|---|
| `TypePack` | B |
| `LabeledTypePack` | B |
| `CompleteTypePack` | B |
| `RecordFields` binder | B |
| `Pack` | A |
| `PositionalPack<T>` | A |
| `LabeledPack<T>` | A |
| `CompletePack<T>` | A |
| `UnorderedShape` | A |
| `RecordRow` / `RecordRowId` | F |
| `Record::Fields` | D |
| `RecordFields::Map` public declaration | D unless fresh audit disproves |
| `RecordRowMap` internal operation | F |
| `PositionalTypePackView` | C pending usage audit |
| `LabeledTypePackView` | C pending usage audit |

## Deliverable

Update or create:

```text
docs/implementation/LANG007/LANG007.C4/LANG007.C4.P9/
    LANG007.C4.P9-ontology-census.md
```

If repository convention prefers updating P10's census in place with a P9 addendum, do that instead.

There must be one authority, not two diverging censuses.

## Gate G1

Pass when:

- every public-looking legacy symbol is classified;
- no planned deletion is based only on spelling;
- kind-domain and reflection-domain uses are distinguished.

---

# 8. T2 — Freeze compatibility and cutover decisions

## Objective

Before deleting public declarations, decide exactly which compatibility names survive P9 and why.

## Required decision table

Create a final disposition table with columns:

```text
name
semantic category
canonical replacement
public after P9?
deprecated?
warning?
tooling visibility?
metadata impact?
planned removal checkpoint?
```

## Mandatory decisions

### `Record::Fields`

Default plan decision:

```text
public after P9: no
replacement: UnorderedProduct::UnorderedShape / UnorderedShape reflection
internal row projection helper: may remain if compiler needs it
```

If actual downstream source packages require a compatibility window, STOP AND CONSULT rather than silently keeping a permanent dual API.

### `RecordFields::Map`

Default plan decision:

```text
public after P9: no
internal row mapping normalization: keep if required
```

The compiler may preserve `IntrinsicTypeOperation::RecordRowMap` or rename it internally without changing source semantics.

### `PositionalTypePackView`

Audit usage.

Preferred outcomes:

```text
unused compatibility alias -> remove
clear discoverability role  -> retain and document
widely used compatibility   -> deprecate with bounded exit
```

### `LabeledTypePackView`

Same rule.

### Kinded domain names

Explicitly mark:

```text
TypePack             canonical; not deprecated
LabeledTypePack      canonical; not deprecated
CompleteTypePack     canonical; not deprecated
RecordFields binder  canonical in Record binder contexts unless amended
```

## Gate G2

Pass when compatibility policy is frozen and documented before source deletion.

---

# 9. T3 — Converge Universe source on the final public ontology

## Objective

Make canonical Universe declarations reflect the final P9 model.

## Primary write set

```text
phalcom-core/core/universe/src/collections/record.ph
phalcom-core/core/universe/src/collections/pack.ph
phalcom-core/core/universe/src/collections/tuple.ph
phalcom-core/core/universe/src/reflection/structural.ph
Universe export/prelude modules discovered during T1
```

## 9.1 `record.ph`

Current relevant surface:

```telos
intrinsic type data RecordFields

impl RecordFields {
  intrinsic type Map<F: Type -> Type>: RecordFields
}

intrinsic data Record<**Fs> {
  **Fs
}

impl<**Fs: RecordFields> Record<**Fs> {
  intrinsic type Fields: RecordFields
  intrinsic type Value: Type
  intrinsic type Map<F: Type -> Type>: Type
}
```

### Target

Preserve:

```telos
intrinsic type data RecordFields
```

if it remains the canonical Record binder kind.

Preserve `Record::Value` and `Record::Map` only if their semantics remain independently valid.

Remove or deprecate:

```telos
Record::Fields
RecordFields::Map
```

as public reflection APIs according to T2.

Do not remove internal operations needed by `Record::Map`.

## 9.2 `pack.ph`

Preserve canonical first-class Pack surface:

```text
Pack
PositionalPack<T>
LabeledPack<T>
CompletePack<T>
PositionalPackView<T>
LabeledPackView<T>
```

Apply the T2 disposition for:

```text
PositionalTypePackView
LabeledTypePackView
```

Do not rename `CompleteTypePack` import merely to make Pack terminology visually uniform.

It is a valid kind constraint.

## 9.3 `tuple.ph`

Preserve:

```text
TypePack
LabeledTypePack
CompleteTypePack
Tuple
Tuple::Positionals
Tuple::Labeled
Tuple::Entries
```

unless a specific member has a separately documented migration.

P9 is not a blanket Tuple type-member cleanup.

## 9.4 `reflection/structural.ph`

Ensure canonical declarations include the final public reflection surface:

```text
Product
OrderedProduct
UnorderedProduct
ProductKind
ProductShape
OrderedTail
PositionalTail
LabeledTail
OrderedShape
Presence
FieldInfo
ProductField
FixedFields
UnorderedTail
UnorderedShape
FieldLookup
UnorderedShapeView
Sum
Variant
Variants
VariantNames
```

Ensure exports are coherent and sufficient for source-defined structural utilities.

## 9.5 Export graph

Audit:

```text
universe root exports
reflection package exports
collections exports
prelude exposure
generated core tables
```

Avoid accidental compatibility through stale re-exports after source declarations are removed.

## Tests

Add/modify source-level Universe tests proving:

```text
kind-domain names still resolve
RecordFields binder still resolves
old reflection facade no longer resolves or warns according to policy
new reflection path resolves
UnorderedShape members have correct result domains
```

## Gate G3

Pass when Universe exposes one reflection model while preserving valid kind/binder domains.

---

# 10. T4 — Remove obsolete public semantic member identity without deleting internal row operations

## Objective

Align semantic member registration, lookup, projection, diagnostics, metadata presentation, and tests with the T3 source surface.

## Expected primary write set

```text
phalcom-semantic/src/types/type_member.rs
phalcom-semantic/src/types/projection.rs
phalcom-semantic/src/declarations.rs
phalcom-semantic/src/static_domain.rs
associated-surface/type-member registration files discovered during T1
phalcom-type-meta/** if public source-member metadata requires migration
```

Do not edit all of these unless the actual P9 base requires it.

## 10.1 `IntrinsicTypeOperation`

Current enum includes:

```rust
RecordFields,
RecordRowMap,
UnorderedShapeLookup,
UnorderedShapeView,
UnorderedShapeTail,
```

The final internal design may legitimately keep:

```text
RecordFields
RecordRowMap
```

if they are synthetic/internal operations.

What must disappear is a source/public identity that claims:

```text
Record::Fields
RecordFields::Map
```

is canonical reflection.

### Preferred implementation

Separate:

```text
internal semantic primitive identity
```

from:

```text
source selected declaration identity
```

Do not keep a dead source declaration merely to retain an internal operation enum.

## 10.2 `IntrinsicTypeOwner`

Current owners include:

```text
Record
RecordRow
UnorderedShape
```

Keeping `RecordRow` internally is valid.

The source lookup path must no longer surface retired declarations.

## 10.3 Projection helper

Current `selected_record_fields_row(...)` is documented around a source-selected:

```text
Record::Fields
```

projection.

After public retirement, either:

1. keep an internal helper but remove its dependence on public source declaration identity; or
2. remove it if no internal caller remains.

Do not rewrite row extraction by reparsing source types.

## 10.4 Type-member tests

Primary test:

```text
phalcom-semantic/tests/semantic/foundations/type_members.rs
```

Current tests assert old public behavior.

Rewrite them to distinguish:

```text
internal row operation remains exact
public structural reflection uses UnorderedShape
valid kind-domain Map operations remain valid
```

## 10.5 Metadata compatibility

If P8 metadata stores source member declaration identity for the retired public member:

- follow P8 schema compatibility strategy;
- decode old metadata only through an explicit versioned compatibility mapping;
- do not silently reinterpret arbitrary old row projections as new `UnorderedShape` values.

If P8 final schema already removed the old public identity, no extra P9 schema bump is needed.

## Gate G4

Pass when:

- source lookup cannot select retired public reflection declarations;
- internal row normalization still works;
- new canonical reflection has exact semantic identity;
- metadata behavior is explicit.

---

# 11. T5 — Implement and certify source-defined structural/reflection helpers

## Objective

Prove that the C4 architecture can express nontrivial structural algorithms as ordinary Telos source.

This is a core P9 acceptance requirement, not decorative library work.

## 11.1 Selection rule

Choose a minimal representative set that exercises:

```text
Pack traversal
Sum reflection
UnorderedShape traversal
ProductShape branching
generic recursion/specialization
cross-module import
```

Do not implement every imaginable helper.

## 11.2 Recommended helper 1 — `VariantsNamed`

Normative source already exists in:

```text
docs/specs/structural-sums.md
```

Use the canonical form:

```telos
type VariantsNamed<
    Vs: Variants,
    Name: Symbol,
>: Variants {
    let mut result = Pack()

    for variant in Vs {
        match variant {
            Variant(name, _) =>
                if name == Name {
                    result = Pack(*result, variant)
                }
        }
    }

    result
}
```

### Acceptance cases

```text
zero matching variants
one matching variant
multiple variants in same family
generic Sum specialization
cross-module helper import
artifact-only helper import after P8
```

## 11.3 Recommended helper 2 — Pack count

A simple Pack count proves static structural iteration without reflection complexity.

Required sizes:

```text
0
1
100
1000
```

This helper also supplies P10 performance input.

## 11.4 Recommended helper 3 — recursive `UnorderedShape` walk

Use:

```text
UnorderedShape::View
```

to traverse finite fixed fields.

Acceptance must prove:

```text
canonical FieldKey order
required/optional FieldInfo preserved
tail returned separately
anonymous open tail not iterated
captured tail handled according to P5/P6 semantics
```

## 11.5 Recommended helper 4 — ProductShape predicate/transform

Use:

```text
ProductShape::Ordered(...)
ProductShape::Unordered(...)
```

in static `match`.

This proves public reflection is sufficient without direct `RecordRow` inspection.

## 11.6 Performance rule

If any helper is O(n²) only because immutable Pack result construction repeatedly copies prefixes:

STOP.

Do not change the source API to require a public mutable builder.

Use existing P7 specialization/fusion and Pack representation optimizations, or route a missing optimization to the appropriate earlier plan.

## 11.7 Cross-module acceptance

At least one helper must be:

```text
defined in provider module
imported by consumer
specialized by consumer
```

and then repeated from P8 durable metadata/artifact state without re-running source initialization semantics.

## Gate G5

Pass when representative structural algorithms execute correctly from ordinary Telos source at acceptable asymptotic cost.

---

# 12. T6 — LSP semantic convergence

## Objective

Remove stale ontology/tooling assumptions and prove editor semantics consume canonical compiler products.

## Primary write set

```text
phalcom-lsp/src/hover.rs
phalcom-lsp/src/completion.rs
phalcom-lsp/src/semantic_tokens.rs
phalcom-lsp navigation/reference/rename modules discovered in T1
phalcom-lsp/tests/module_navigation.rs
phalcom-lsp/tests/semantic_completion.rs
other C4-specific LSP tests
```

## 12.1 Kind completion

Keep valid kind-domain completions:

```text
TypePack
LabeledTypePack
CompleteTypePack
RecordFields
```

in appropriate kind contexts.

Keep internal:

```text
RecordRow
```

hidden.

## 12.2 Runtime/proper-type completion

Do not offer kind-only declarations as runtime values.

Offer:

```text
Pack
PositionalPack
LabeledPack
CompletePack
Product/Sum reflection declarations
```

where visible.

## 12.3 Legacy reflection completion

After T3/T4:

```text
Record::Fields
```

must not remain a canonical completion item.

If T2 chooses a deprecation window, it may remain but must be clearly marked deprecated and ordered behind the canonical API.

## 12.4 Hover

Binding semantics:

```text
let      runtime immutable binding
let mut  runtime mutable binding
const    compile-time constant
```

Reflection members must show exact result domains.

Examples:

```text
Product::Shape          : ProductShape
UnorderedProduct::UnorderedShape : UnorderedShape
Sum::Variants           : Variants
UnorderedShape::Lookup  : FieldLookup
```

Do not render every type member as `Type`.

## 12.5 Const hover

Where semantic products expose a compact reduced constant:

show:

```text
declared/inferred type
reduced static value
```

with deterministic truncation for large Pack/shape values.

Do not trigger static evaluation opportunistically from hover.

Consume the existing checked/frozen semantic product.

## 12.6 Definition / references

Current `module_navigation.rs` asserts navigation to:

```text
Record::Fields
```

Migrate this test.

New navigation coverage should include:

```text
UnorderedProduct::UnorderedShape
UnorderedShape
FieldKey
Variant
ProductShape
```

as applicable.

Navigation target must be the authored/intrinsic Universe declaration, not a synthetic mystery source location.

## 12.7 Rename

Verify:

```text
user static definition rename works
const rename works
reflection intrinsic declaration is not renamed as user-owned source
compiler-derived capability identity is not forged/renamed
```

## Gate G6

Pass when LSP presents the final C4 ontology and no stale old-reflection navigation test remains.

---

# 13. T7 — REPL, formatter/printer, VS Code, and generated editor surface

## Objective

Close non-LSP user tooling.

---

## 13.1 REPL

### Primary paths

Discover actual files under:

```text
phalcom-repl/src/**
phalcom-repl/tests/**
```

### Required semantics

```text
const     evaluated by semantic static evaluator
let       runtime immutable
let mut   runtime mutable
```

`const` must not be executed through ordinary runtime bytecode just because REPL is an execution environment.

### Persistence

If REPL uses a persistent module/session model, certify:

```text
const result persists
dependent later submission sees same semantic constant
invalidating edit/submission behaves according to existing session rules
```

### Highlighter/completion

Ensure keyword tables agree with parser/LSP.

No independent legacy:

```text
let = mutable
```

presentation may remain.

---

## 13.2 Formatter / printer

First discover the repository's canonical formatter/printer authority.

If no general formatter exists, record:

```text
NOT_APPLICABLE: no canonical formatter subsystem
```

and validate parser/display printers that actually exist.

If one exists, cover:

```text
let mut
mut field
block-bodied type without =
Pack rest/expansion
Record FieldKey syntax
static match/for/while
Product/Sum examples
```

Round-trip:

```text
parse -> format -> parse
```

must preserve semantic AST.

---

## 13.3 VS Code / TextMate

Audit:

```text
tools/vsphalcom/src/generated/core-table.json
tools/vsphalcom/syntaxes/phalcom.tmLanguage.json
tools/vsphalcom/manual-test/**
```

Update generator source of truth first:

```text
phalcom-core/bin/gen-core-table/**
```

if generated core declarations changed.

Do not hand-edit generated declaration tables.

### Grammar coverage

Verify highlighting for:

```text
mut
const
intrinsic
type data
block-bodied type
* / ** / ***
Record key brackets
static control forms
```

JSON parse must pass.

---

## Gate G7

Pass when REPL/editor surfaces match canonical parser/semantic behavior and generated artifacts are regenerated from source.

---

# 14. T8 — Documentation, ADR, specification, example convergence

## Objective

Make repository documentation internally coherent and aligned with the actual post-P9 architecture.

## 14.1 Mandatory current authorities

Audit and update as needed:

```text
docs/decisions/accepted/0090-structural-static-evaluation-packs-products-sums-and-binding-semantics.md
docs/specs/type-level-domains.md
docs/specs/type-members.md
docs/specs/structural-products.md
docs/specs/structural-sums.md
docs/specs/static-evaluation.md
docs/specs/bindings-and-fields.md
docs/specs/universe.md
docs/specs/vm/products.md
docs/specs/callables/arguments.md
```

## 14.2 C4 planning/guidance reconciliation

Update:

```text
docs/implementation/LANG007/LANG007.C4/LANG007.C4-GUIDANCE.md
docs/implementation/LANG007/LANG007.C4/LANG007.C4-requirements-analysis.md
docs/implementation/LANG007/LANG007.C4/LANG007.C4-structural-static-upgrade-handoff.md
```

Do not rewrite historical events.

Instead add explicit amendment notes where older text conflicts with `INC-P3-2`.

## 14.3 Required final ontology statement

Current docs must clearly teach:

```text
TypePack family
    = kinded static domains

Pack family
    = first-class proper structural values/types

RecordFields
    = Record binder domain where applicable

UnorderedShape
    = public unordered structural reflection

RecordRow
    = internal compiler row calculus
```

## 14.4 Remove stale reflection vocabulary

Search and repair current normative examples containing:

```text
RecordProduct
RecordField
FieldPresence
TypePackView::Item
LabeledTypePackView::Item
P::Positionals
P::Labeled
raw OrderedShape match against Product::Shape
```

using canonical forms.

## 14.5 Binding/field semantics

No current normative docs may claim:

```text
let is mutable
const is runtime immutable
bare field is mutable
const field is immutable
```

Historical decisions may retain old text only with explicit supersession.

## 14.6 Source-defined helper examples

Synchronize normative examples with code actually certified in T5.

Do not document hypothetical syntax that no longer parses.

## Gate G8

Pass when a repository doc audit finds no current normative contradictions.

---

# 15. T9 — Fuzz, property, and regression closure

## Objective

Convert important C4 invariants into durable automated evidence.

## 15.1 Syntax corpus

Add/update cases for:

```text
let mut
mut field
block-bodied type
Pack construction
Pack spread
Pack rest
heterogeneous Record FieldKey
Record pattern
static match
static for
static while
nested pattern
repeated variant family
open/captured tail
```

## 15.2 Property tests

Implement or retain direct properties.

### P9-PROP-01 — unordered source-order invariance

Source field order permutation does not change canonical unordered semantic shape.

### P9-PROP-02 — Pack view/recompose

Decomposing and reconstructing preserves Pack structural equality.

### P9-PROP-03 — Pack root-backing stability

Repeated alternating positional/labeled rest slicing retains root backing and composes offsets.

### P9-PROP-04 — metadata round-trip

P8 exported C4 values/residuals decode to semantically equivalent products.

### P9-PROP-05 — cold/incremental equivalence

Equivalent final source graph reached through different edit sequences yields equal:

```text
semantic static value
diagnostics
reflection order
metadata fingerprint where specified
runtime result
```

### P9-PROP-06 — specialized reflection parity

Generic reflection specialized later equals direct specialized reflection where semantics say they should coincide.

### P9-PROP-07 — fixed-call no-Pack lowering

Compiler IR/bytecode for fixed calls contains no Pack allocation path solely due to C4.

### P9-PROP-08 — repeated variant families

Variants sharing one Symbol remain separate exact variants in reflection.

### P9-PROP-09 — canonical FieldKey order

Mixed:

```text
Symbol
String
Int
Bool
```

keys sort according to the language-defined order, not host enum discriminants or intern IDs.

### P9-PROP-10 — static cancellation not cached

Cancelled/budget-exceeded static work cannot become a reusable successful result.

## Gate G9

Pass when all checkpoint-critical properties have direct regression coverage.

---

# 16. T10 — Establish performance baseline and certify C4

## Objective

Measure C4's runtime and compile-time cost and protect critical asymptotic/runtime invariants.

## 16.1 Harness

Reuse existing infrastructure:

```text
phalcom-core/benches/vm_bench.rs
benchmarks/**
Criterion under phalcom-core `benchmarks` feature
semantic timing/instrumentation where already available
```

Add focused benchmarks where coverage is missing.

Do not invent a parallel benchmark framework.

## 16.2 Baseline protocol

For each benchmark record:

```text
benchmark name
baseline revision
P9 revision
machine
OS/toolchain
build profile
warmup
sample count
median/mean
variance/error
allocation count if measured
threshold
result
```

## 16.3 Runtime benchmark set

### P9-BENCH-R1 — fixed positional call

Protect:

```text
no Pack allocation
no meaningful throughput regression
```

### P9-BENCH-R2 — fixed labeled call

Same requirement.

### P9-BENCH-R3 — dynamic/rest call

Measure new Pack path separately.

### P9-BENCH-R4 — Tuple construction/access

Ensure C4 did not regress unrelated ordered product runtime.

### P9-BENCH-R5 — Record construction/access

Include heterogeneous FieldKey where runtime supports it.

### P9-BENCH-R6 — ordinary non-static program

Protect code that never uses const/reflection.

### P9-BENCH-R7 — module startup

Measure unused/used large constants where P8 materialization policy matters.

## 16.4 Static/semantic benchmark set

### P9-BENCH-S1 — simple closed const

### P9-BENCH-S2 — simple static type definition

### P9-BENCH-S3 — Pack count 100

### P9-BENCH-S4 — Pack count 1000

### P9-BENCH-S5 — VariantsNamed over 1k variants

### P9-BENCH-S6 — large UnorderedShape traversal

### P9-BENCH-S7 — recursive static computation

Measure:

```text
cold
memo reuse
specialized
```

### P9-BENCH-S8 — residual specialization

### P9-BENCH-S9 — metadata decode

### P9-BENCH-S10 — artifact-only imported specialization

### P9-BENCH-S11 — incremental one-file edit

### P9-BENCH-S12 — editor cancellation latency

## 16.5 Allocation/complexity checks

At minimum prove:

```text
Pack rest/view      O(1), no O(n) copy
Pack traversal      O(n)
Pack finalize       O(n)
multi-spread        O(total entries)
fixed call          no Pack allocation
metadata decode     O(nodes + edges)
source helper       no accidental O(n²) immutable append pathology
```

## 16.6 Threshold policy

Set explicit thresholds before final run.

Minimum:

```text
fixed-call throughput:
    no statistically meaningful material regression

fixed-call allocations:
    exactly no new Pack allocation

Pack view:
    constant-time scaling between representative sizes

Pack traversal:
    approximately linear scaling

metadata:
    growth attributable to exported semantic products, not duplicate caches

editor cancellation:
    bounded according to existing query/cancellation infrastructure
```

If noise prevents a precise percentage threshold, document variance and use repeated comparison rather than lowering the requirement to subjective judgment.

## Gate G10

Pass when performance evidence is complete and no critical invariant has unexplained regression.

---

# 17. T11 — Broad workspace/checkpoint certification

## Objective

Run the broad validation intentionally deferred by earlier plans.

## 17.1 Core crate gates

Use exact repository convention at execution time.

Minimum coverage:

```bash
cargo test -p phalcom-ast
cargo test -p phalcom-modules
cargo test -p phalcom-type-meta
cargo test -p phalcom-semantic
cargo test -p phalcom-core
cargo test -p phalcom-lsp
cargo test -p phalcom-repl
```

If workspace nextest is canonical:

```bash
cargo nextest run --workspace
```

or the repository's documented equivalent.

## 17.2 Static quality gates

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features
```

If inherited format drift exists, prove it against the clean predecessor and record it.

Do not silently reformat unrelated files in P9.

## 17.3 Language corpus

Run:

```text
bindings
fields
types
records
patterns
static evaluation
Pack
reflection
strings/text if C4-owned follow-ons require it
random/parse follow-on gates where P9 retained certification obligation
```

Use repository selectors rather than speculative command names.

## 17.4 Metadata/import

Run:

```text
schema compatibility
cross-module static definitions
cross-module constants
artifact-only specialization
materialization
cold/incremental parity
```

## 17.5 Tooling

Run:

```text
LSP integration
REPL integration
VS Code grammar JSON/golden/manual checks
generated core-table diff/check
```

## 17.6 GC/runtime stress

Only run GC stress where C4 runtime materialization/Pack/Record ownership is relevant.

Do not add broad unrelated concurrency/GC work without evidence.

## Gate G11

Pass when broad checkpoint evidence is recorded with selected/pass/fail counts.

---

# 18. T12 — Residual failure classification and repair loop

## Objective

No broad test failure may remain unclassified at checkpoint closure.

## 18.1 Classification model

Use repository-standard categories; if a new local table is needed:

```text
A  P9 regression
B  earlier C4 regression revealed by P9
C  proven unrelated baseline failure
D  explicitly deferred outside C4
```

## 18.2 Revalidation rule

Historic baseline classifications must be rechecked.

A previous failure cannot automatically stay baseline if C4 intentionally changed its subsystem.

Examples:

```text
binding-flow failure
    cannot remain baseline if P1 changed that exact flow

Record reflection failure
    cannot remain baseline if P4/P5/P9 changed the member

static const failure
    cannot remain baseline if P2/P7/P8 changed static semantics
```

## 18.3 Repair loop

For A/B failures:

```text
reproduce narrow
identify owner
fix at canonical layer
run narrow regression
run affected crate
return to failed broad gate
```

Maximum debugging-cycle guidance follows repository C4 policy; escalate if the failure reveals a broken architectural premise.

## 18.4 P1 historical baseline failures

Re-run the relevant suites.

If failures still exist and remain unrelated, record clean predecessor evidence.

## 18.5 P14 / parse complete-package obligation

The repository explicitly assigned P9 a later complete-package certification obligation for previously blocked parse/Text consumer paths.

P9 must:

```text
run the final package gate if now unblocked
or
record proof that the remaining failure belongs to another owner/checkpoint
```

Do not omit it from final ledger.

## 18.6 Random follow-on

The checkpoint records a partially accepted Random follow-on.

P9 must classify whether unresolved work blocks C4.

Do not infer from naming alone.

## 18.7 Algebraic-literals follow-on

The checkpoint records a blocked algebraic-literals follow-on.

P9 must decide:

```text
C4-owned blocker
or
explicit non-C4 deferred follow-on
```

and document why.

## Gate G12

Pass when every residual failure/follow-on has a final disposition.

---

# 19. T13 — Lifecycle and checkpoint closure

## Objective

Make lifecycle records accurately represent what actually shipped.

## 19.1 P9 walkthrough

Create:

```text
docs/implementation/LANG007/LANG007.C4/LANG007.C4.P9/
    LANG007.C4.P9-walkthrough.md
```

Record:

```text
prepared revision
final revision
source changes
ontology decisions
tooling migration
docs migration
source-defined helper evidence
metadata evidence
performance evidence
broad test evidence
residual failure classifications
stop/consult incidents
```

## 19.2 P9 handoff

Create:

```text
LANG007.C4.P9-handoff.md
```

If P9 fully closes C4, this is a checkpoint/program handoff, not another C4 plan handoff.

It should summarize:

```text
frozen final semantics
remaining non-C4 deferred work
future optimization opportunities
compatibility removal checkpoints
```

## 19.3 C4 checkpoint

Update:

```text
docs/implementation/LANG007/LANG007.C4/LANG007.C4-CHECKPOINT.md
```

The checkpoint should contain final durable truth, not every edit.

Update:

```text
P4/P7/P8 final states
P9 final state
P10/P11–P16 final dispositions
verification ledger
consultation/amendment ledger
baseline/deferred ledger
next action
```

## 19.4 Program status

Update:

```text
docs/implementation/LANG007/PROGRAM.md
docs/implementation/LANG007/STATUS.md
```

These are currently stale and still describe early C4/P4 state.

Final program records must not omit completed P5/P6/P10/P11–P16 work.

## 19.5 Verification status

Do not automatically write:

```text
RELEASE_COMPLETE
```

Possible final checkpoint state:

```yaml
status: COMPLETE
completion: IMPLEMENTED
verification: FOCUSED_TESTED
```

or stronger only if the final broad gates satisfy the repository's release-complete standard.

## Gate G13

Pass when lifecycle files are truthful and internally consistent.

---

# 20. Exact public ontology migration matrix

| Current surface | P9 action | Final state |
|---|---|---|
| `TypePack` | keep | canonical kinded domain |
| `LabeledTypePack` | keep | canonical kinded domain |
| `CompleteTypePack` | keep | canonical kinded domain |
| `RecordFields` binder | keep unless new ratified decision | Record-context kinded domain |
| `RecordRow` | keep internal | compiler row representation |
| `RecordRowId` | keep internal | stable internal semantic identity |
| `Pack` | keep | canonical proper Pack |
| `PositionalPack<T>` | keep | canonical first-class Pack specialization |
| `LabeledPack<T>` | keep | canonical first-class Pack specialization |
| `CompletePack<T>` | keep | canonical first-class Pack specialization |
| `PositionalPackView<T>` | keep | canonical Pack view |
| `LabeledPackView<T>` | keep | canonical Pack view |
| `PositionalTypePackView` | audit | explicit keep/deprecate/remove decision |
| `LabeledTypePackView` | audit | explicit keep/deprecate/remove decision |
| `Record::Fields` | retire/migrate | replaced by UnorderedProduct/UnorderedShape reflection |
| `RecordFields::Map` public facade | retire/migrate | internal row map may remain |
| `Record::Value` | re-audit independently | keep if semantically valid |
| `Record::Map<F>` | re-audit independently | keep if semantically valid |
| `UnorderedShape` | keep | canonical public unordered reflection |
| `FieldKey` | keep | canonical heterogeneous key |
| `FieldInfo` | keep | canonical field metadata |
| `Presence` | keep | canonical required/optional |
| `UnorderedTail` | keep | canonical tail reflection |
| `Product` / `OrderedProduct` / `UnorderedProduct` | keep | sealed compiler-derived capabilities |
| `Sum` / `Variant` / `Variants` | keep | canonical sum reflection |

---

# 21. Semantic source write-set guidance

This is an expected write set, not a mandate to touch every file.

## Strongly expected

```text
phalcom-core/core/universe/src/collections/record.ph
phalcom-core/core/universe/src/reflection/structural.ph

phalcom-semantic/src/types/type_member.rs
phalcom-semantic/tests/semantic/foundations/type_members.rs

phalcom-lsp/tests/module_navigation.rs
phalcom-lsp/tests/semantic_completion.rs

docs/implementation/LANG007/LANG007.C4/LANG007.C4-GUIDANCE.md
docs/implementation/LANG007/LANG007.C4/LANG007.C4-CHECKPOINT.md
docs/implementation/LANG007/PROGRAM.md
docs/implementation/LANG007/STATUS.md
```

## Likely

```text
phalcom-core/core/universe/src/collections/pack.ph
phalcom-semantic/src/types/projection.rs
phalcom-lsp/src/hover.rs
phalcom-lsp/src/completion.rs
phalcom-lsp/src/semantic_tokens.rs
phalcom-repl/**
tools/vsphalcom/**
docs/specs/**
docs/decisions/accepted/0090-...
```

## Conditional

```text
phalcom-type-meta/**
phalcom-modules/**
compiler lowering
VM runtime
```

These should change only if P8 or P9 integration proves a real need.

If P9 requires broad compiler/runtime semantic changes, investigate whether an earlier plan is incomplete.

---

# 22. Verification ladder

Use the smallest discriminating test while implementing, then broaden deliberately.

## T3/T4 local loop

```text
source parse/check for changed Universe module
semantic type-member focused test
semantic structural reflection focused test
LSP focused navigation/completion test
```

## T5 source-helper loop

```text
single source helper regression
generic specialization regression
cross-module regression
artifact-only regression
scaling probe
```

## T6/T7 tooling loop

```text
single LSP test
LSP integration target
single REPL test
REPL integration target
grammar/generator checks
```

## T8 docs

```text
rg guards
link/check tooling if repository has it
source-example compile tests where supported
```

## T9 property

```text
individual property
module suite
affected crate
```

## T10 benchmark

Run benchmark independently from broad test suite.

## T11 final

Run broad certification once the source tree is otherwise stable.

---

# 23. `rg` guardrail searches for P9 completion

Run at T1 and again at T13.

## Ontology

```bash
rg -n "Record::Fields|RecordFields::Map|RecordProduct|RecordField|FieldPresence|TypePackView::Item|LabeledTypePackView::Item" \
  phalcom-* tools docs
```

Every remaining occurrence must be:

```text
internal
historical
negative migration test
or intentionally retained compatibility
```

## Kind-domain safety

```bash
rg -n "TypePack|LabeledTypePack|CompleteTypePack|RecordFields" \
  phalcom-core phalcom-semantic phalcom-lsp docs/specs
```

The result must **not** be zero.

These names remain valid in specific roles.

## Runtime/static architecture

```bash
rg -n "static.*VM|VM.*static|runtime.*const.*initializer|const.*runtime.*initializer" \
  phalcom-semantic phalcom-core
```

Inspect rather than blindly asserting every textual hit is bad.

## Fixed-call Pack path

Search compiler lowering for:

```text
NewArgumentPack
FinishPack
PackExpand
```

on fixed-call paths.

Architecture tests are stronger than grep.

---

# 24. Performance acceptance table

| Requirement | Evidence |
|---|---|
| fixed calls do not allocate Pack | lowering/bytecode invariant + allocation benchmark |
| Pack rest O(1) | backing identity test + size-scaling microbenchmark |
| Pack traversal O(n) | 100/1000 scaling |
| Pack construction not accidental O(n²) | source helper benchmark |
| unordered canonicalization bounded | large mixed FieldKey benchmark |
| recursive static memo reuse | step/work counters + timing |
| imported specialization reusable | P8 artifact benchmark |
| metadata not duplicated | size comparison + pool/node counts |
| cancellation bounded | editor/static cancellation benchmark |
| ordinary runtime unaffected | representative VM benchmark baseline |

---

# 25. Tooling acceptance table

| Tool | Required proof |
|---|---|
| LSP completion | correct kind vs value namespace |
| LSP hover | C4 binding semantics + exact reflection result domain |
| LSP definition | canonical Universe declarations |
| LSP references | stable semantic declaration identity |
| LSP rename | no synthetic reflection corruption |
| REPL | const static + let/let mut runtime semantics |
| semantic tokens | contextual `mut`, structural syntax |
| VS Code grammar | valid generated/current grammar |
| formatter/printer | round-trip if canonical formatter exists |

---

# 26. Documentation acceptance table

| Topic | Final statement |
|---|---|
| TypePack | canonical kinded domain |
| PositionalPack | proper first-class Pack specialization |
| RecordFields | Record binder domain, not reflection ontology |
| RecordRow | compiler/internal |
| Record reflection | `UnorderedShape` |
| Pack rest | O(1) view/slice |
| fixed calls | no Pack allocation requirement |
| const | compile-time static |
| let | runtime immutable |
| let mut | runtime mutable |
| Product/Sum | compiler-derived sealed |
| static control | semantic evaluator, no VM |
| recursive static | bounded, memoized/residualized |
| metadata | stable semantic identities, no process-local IDs |

---

# 27. Final closeout evidence packet

P9 completion should leave a compact evidence packet.

Recommended records:

```text
LANG007.C4.P9-ontology-census.md
LANG007.C4.P9-performance-certification.md
LANG007.C4.P9-broad-verification.md
LANG007.C4.P9-walkthrough.md
LANG007.C4.P9-handoff.md
```

If this would duplicate the checkpoint excessively, merge:

```text
performance
broad verification
residual classification
```

into the walkthrough and checkpoint.

The important rule is one clear authority per kind of evidence.

---

# 28. STOP / CONSULT matrix

## STOP-P9-01 — deleting TypePack seems necessary

Reason:

```text
conflicts with INC-P3-2
```

## STOP-P9-02 — deleting RecordFields binder seems necessary

Reason:

```text
changes generic Record kind semantics
```

## STOP-P9-03 — `UnorderedShape` cannot replace public `Record::Fields`

Reason:

```text
P4/P5 API incomplete
```

## STOP-P9-04 — source helper needs new control/evaluator semantics

Reason:

```text
P7 incomplete
```

## STOP-P9-05 — source helper fails cross-module/artifact use

Reason:

```text
P8 incomplete
```

## STOP-P9-06 — tooling needs a new semantic identity graph

Reason:

```text
earlier architecture incomplete
```

## STOP-P9-07 — performance requires changing public Pack semantics

Reason:

```text
architecture change, not P9 optimization
```

## STOP-P9-08 — fixed-call regression cannot be repaired locally

Reason:

```text
C4 violates core runtime invariant
```

## STOP-P9-09 — cold/incremental parity mismatch

Reason:

```text
semantic DB/P8 correctness defect
```

## STOP-P9-10 — metadata compatibility requires silent semantic reinterpretation

Reason:

```text
invalid durability migration
```

## STOP-P9-11 — final docs have conflicting authoritative ontology

Reason:

```text
authority decision unresolved
```

## STOP-P9-12 — broad failure overlaps changed C4 semantics but is marked baseline

Reason:

```text
classification invalid
```

## STOP-P9-13 — final closure still has unresolved C4-owned plan work

Reason:

```text
cannot close checkpoint truthfully
```

---

# 29. Gate summary

| Gate | Acceptance |
|---|---|
| G0 | P4/P7/P8 readiness and P9 base frozen |
| G1 | refreshed ontology census complete |
| G2 | compatibility/cutover decisions frozen |
| G3 | Universe source converged |
| G4 | semantic public-member surface converged |
| G5 | representative source-defined helpers certified |
| G6 | LSP converged |
| G7 | REPL/formatter/editor generated surfaces converged |
| G8 | docs/spec/ADR convergence complete |
| G9 | fuzz/property regression coverage complete |
| G10 | performance certification passes |
| G11 | broad workspace/checkpoint verification recorded |
| G12 | every residual failure/follow-on classified |
| G13 | lifecycle records closed truthfully |

P9 is complete only when G0–G13 have real evidence.

---

# 30. Expected final C4 architecture

After successful P9:

```text
C3 kind domains
    TypePack
    LabeledTypePack
    CompleteTypePack
    RecordFields (Record binder context)
          |
          | compiler semantic domains
          v

C4 first-class structural values/types
    Pack
      |- PositionalPack<T>
      |- LabeledPack<T>
      `- CompletePack<T>

C4 reflection
    Product
      |- OrderedProduct -> OrderedShape
      `- UnorderedProduct -> UnorderedShape

    Sum -> Variants -> PositionalPack<Variant>

C4 static semantics
    StaticValue
    StaticResidual
    bounded evaluator
    static match/for/while
    recursion/memo/specialization

C4 durability
    metadata DAGs
    imported static definitions
    imported constants
    materialization descriptors

Tooling/docs
    consume and present the same graph
```

There is no second public row reflection ontology.

There is no compile-time VM.

There is no permanent compatibility layer that forces compiler hot paths through obsolete APIs.

---

# 31. Definition of done

`LANG007.C4.P9` is **COMPLETE / IMPLEMENTED** only when all of the following hold.

## Semantics

- `TypePack`, `LabeledTypePack`, and `CompleteTypePack` remain valid kind domains.
- `RecordFields` remains only in intentionally ratified binder/domain roles.
- public unordered reflection is `UnorderedShape` based.
- `RecordRow` is internal.
- obsolete `Record::Fields` reflection is removed/deprecated exactly according to T2.
- no source-visible dual reflection model remains.

## Source-defined metaprogramming

- at least one Pack algorithm is ordinary Telos source;
- `VariantsNamed` or equivalent Sum algorithm is ordinary Telos source;
- unordered reflection can be traversed through public `UnorderedShape` API;
- generic/cross-module/static durability path works;
- no ordinary helper is moved native solely for convenience.

## Tooling

- LSP presents canonical semantics;
- REPL respects static/runtime phase distinctions;
- generated/editor grammar is synchronized;
- formatter/printer, if present, round-trips C4 forms.

## Documentation

- `INC-P3-2` is reflected everywhere current semantics are taught;
- old binding/field semantics are superseded;
- old row reflection is not taught as canonical;
- examples compile against the actual API.

## Durability

- metadata and artifact-only import pass;
- cold/incremental parity passes;
- constants materialize without initializer re-execution.

## Performance

- fixed-call fast path is protected;
- Pack rest/view is O(1);
- structural algorithms scale as intended;
- metadata/editor/static paths have recorded measurements;
- no unexplained material regression remains.

## Verification

- broad relevant crates pass or failures have proven disposition;
- no zero-test filter is counted;
- historic baseline classifications have been revalidated;
- P14/parse retained certification obligations have final disposition;
- Random/algebraic follow-ons have explicit blocking/non-blocking status.

## Lifecycle

- P9 walkthrough exists;
- P9 handoff exists;
- C4 checkpoint is current;
- LANG007 PROGRAM is current;
- LANG007 STATUS is current;
- checkpoint status does not overclaim release completeness.

---

# 32. Implementer operating rule

P9 is not successful because old names disappeared.

P9 is successful when the repository has **one coherent semantic story**:

```text
valid compiler kind domains remain,
proper Pack values remain distinct,
unordered reflection uses Product/UnorderedShape,
ordinary Telos can compute over those structures,
metadata preserves them,
tooling presents them,
docs teach them,
benchmarks protect them,
and lifecycle records prove the checkpoint is actually closed.
```

If implementation begins deleting canonical kind domains, inventing tooling semantics, moving ordinary structural algorithms native, or treating focused tests as checkpoint certification, it has left the P9 architecture and must stop.
