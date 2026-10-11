---
id: LANG007.C4.P10
category: LANG
program: LANG007
checkpoint: LANG007.C4
plan: LANG007.C4.P10
kind: implementation-plan
title: "Parallel Structural Classification, Static-Control, and Convergence Foundation"
status: IN_PROGRESS
completion: PARTIAL
verification: UNVERIFIED
repository: aureat/phalcom-lang
branch: main
prepared_against_revision: 419ec6e66ce90cba6003b7627eb4c99fa2345a7a
prepared: 2026-10-05
started_against_revision: 419ec6e66ce90cba6003b7627eb4c99fa2345a7a
worktree_branch: codex/lang007-c4-p10
worktree_path: /Users/altunhasanli/.codex/worktrees/lang007-c4-p10/phalcom
started: 2026-10-05
depends_on:
  - LANG007.C4.P2
  - LANG007.C4.P3
parallel_with:
  - LANG007.C4.P4
follows: LANG007.C4.P3
supersedes: null
---

# LANG007.C4.P10 — Parallel Structural Classification, Static-Control, and Convergence Foundation

## Luna patch-grade implementation plan

# 0. Executor contract

This plan is a bounded parallel implementation tranche inside `LANG007.C4`. It deliberately extracts dependency-independent work from the eventual P5, P6, and P9 scopes so that it can proceed concurrently with active P4 unordered-product work.

It is **not** an early declaration that P5, P6, or P9 have begun or completed.

The implementer:

- may adapt mechanical details to live repository drift after re-reading the named paths;
- must preserve the post-documentation-consolidation normative architecture;
- must treat `main` revision `419ec6e66ce90cba6003b7627eb4c99fa2345a7a` as the planning anchor, not an older P3/P4 snapshot;
- must preserve `LANG007.C4.P4` as the owner of heterogeneous `FieldKey`, Record rows, unordered-product calculus, Record runtime shape, Record patterns, and FieldKey metadata staging;
- must not create a second structural type system, reflection graph, static evaluator, pattern resolver, relation engine, row solver, or semantic identity system;
- must not publish incomplete general `Product` / `UnorderedProduct` capability semantics merely because ordered structural facts can be derived early;
- must use canonical semantic type/declaration products as structural-classification inputs;
- must use the existing ordinary checker to establish legality, binding identity, loop-control legality, and return contracts before the static evaluator executes a body;
- must preserve `QueryBudget` and `CancellationToken` as the sole static-evaluation resource authorities;
- must preserve P3's Pack storage, Symbol-only ordered labels, O(1) views, and fixed-call zero-Pack fast path;
- must keep P4 and P10 source changes isolated until explicitly named synchronization barriers;
- must follow the testing budget and not escalate to broad verification reflexively;
- must classify unrelated failures A/B/C/D under the C4 lifecycle convention;
- must STOP AND CONSULT whenever a fixed architectural premise fails;
- must keep `LANG007.C4-CHECKPOINT.md` truthful;
- must produce a P10 walkthrough and handoff before declaring P10 complete.

Testing is evidence gathering. A source edit alone never justifies a broad test run.

---

# 1. Goal

Implement the dependency-independent structural-classification, ordered-product reflection, shallow Sum/Variant reflection, static control-transfer foundation, explicit static `while` semantics, P1–P3 convergence cleanup, ontology census, and Pack/static-evaluator resource harnesses that can safely advance while P4 independently completes generalized unordered Record semantics.

The architectural output is:

```text
P3 canonical ordered Pack/type facts
existing Tuple/data/enum semantic products
                    │
                    ├───────────────┐
                    ▼               ▼
       shared structural       shallow Sum/Variant
       classification facts     classification facts
                    │               │
                    └──────┬────────┘
                           ▼
                 future P5 capability/
                  public reflection layer


P2 checked static body
        +
ordinary checker control legality
        │
        ▼
explicit EvalControl
return / break / continue
if propagation
explicit semantic while
        │
        ▼
future P6 match/for/proof control


P1–P3 implemented repository
        │
        ├─ docs/spec consistency audit
        ├─ LSP/REPL/tooling cleanup
        ├─ legacy ontology census
        └─ resource harnesses
                  │
                  ▼
             future P9 closure
```

P10 intentionally does **not** implement the P4-owned unordered branch.

---

# 2. Checkpoint acceptance objective

`LANG007.C4` ultimately requires one coherent structural-static system:

```text
canonical semantic structure
        ↓
sealed compiler-derived structural capability
        +
ordinary reflected structural data

ordinary checked Telos constructs
        ↓
bounded semantic static evaluator

first-class Pack
FieldKey Record calculus
static control
recursion/residualization
durable metadata/materialization
public/tooling convergence
```

P10 advances this checkpoint in three independent tracks:

1. **Structural foundation**
  - shared structural-classification framework;
  - ordered-product classification;
  - ordered reflection over finalized Tuple/Pack/ordered data;
  - Sum classification;
  - declaration-ordered shallow Variant reflection.

2. **Static-control foundation**
  - evaluator control-transfer representation;
  - `return`;
  - `break`;
  - `continue`;
  - integration of already-existing static `if`;
  - explicit semantic `while` identity and evaluator execution after the P4 frontend synchronization barrier.

3. **Convergence/resource foundation**
  - post-consolidation P1–P3 specification/implementation consistency audit;
  - bounded LSP/REPL/tooling cleanup;
  - legacy ontology census and P9 migration preparation;
  - Pack and static-evaluator deterministic performance/resource harnesses.

P10 is successful when these foundations are implemented and focused-tested **without requiring P4 unordered semantics** and without pretending the later integrated P5/P6/P9 acceptance objectives are complete.

---

# 3. Repository grounding

## 3.1 Planning revision

```text
repository: aureat/phalcom-lang
branch:     main
revision:   419ec6e66ce90cba6003b7627eb4c99fa2345a7a
date:       2026-10-05
```

The head is the merge of the consolidated C4 documentation branch:

```text
419ec6e Merge pull request #19 from aureat/docs/c4-structural-static-upgrade
ee5a8aed docs(c4): reconcile structural-static specification with implemented P3 and active P4
```

This matters: the consolidated normative files, especially `structural-products.md`, `structural-sums.md`, `static-evaluation.md`, and `type-level-domains.md`, override older planning assumptions where they differ.

The user reports:

```text
local main: clean
remote main: synchronized
dirty files: none
```

T0 must still mechanically verify this before creating the P10 branch/worktree.

## 3.2 Current C4 state

The checkpoint currently records:

```text
P1 COMPLETE / IMPLEMENTED / BASELINE_BLOCKED
P2 COMPLETE / IMPLEMENTED / FOCUSED_TESTED
P3 COMPLETE / IMPLEMENTED / FOCUSED_TESTED
P4 IN_PROGRESS / PARTIAL / UNVERIFIED
P5–P9 PROPOSED / NOT_STARTED
```

P4 code currently integrated into `main` includes:

```text
d0c80c71 feat(semantic): begin canonical FieldKey scalar
80e4e854 refactor(semantic): share exact integer scalar
```

Post-code commits through `419ec6e` are documentation/lifecycle consolidation.

P4 therefore remains early in T1:

```text
implemented:
    FieldKey scalar
    shared StaticInt owner

not yet completed:
    Record AST/parser migration
    Record row migration
    solver/relation migration
    static Record FieldKey migration
    runtime Record migration
    pattern migration
    metadata/incremental/performance closure
```

P10 must not rely on any of those unfinished downstream P4 facts.

## 3.3 Current ordered structural substrate

P3 established:

```text
TypeData::Pack
proper Pack type construction
StaticValueData::Pack
runtime PackRoot / PackSlice
EmptyPack immediate
FinishPack / BuildStaticPack / BuildPack
rest capture -> Pack
spread/forwarding convergence
O(1) lane rest slices
fixed-call no-Pack invariant
```

Current normative architecture preserves:

```text
Pack / Tuple / call labels -> Symbol
Record unordered keys      -> FieldKey
```

This boundary is load-bearing.

## 3.4 Current product declaration authority

The semantic layer already owns:

```rust
DataInfo
DataShape::{Tuple, Record}
DataProductShapeTemplate
DataProductShapeMaterialization
DataProductMaterializedComponent
```

This is the canonical nominal product declaration/specialization authority.

Do not infer product structure from runtime `phalcom-core::ProductShape`.

## 3.5 Current Sum/Variant authority

The semantic layer already owns:

```rust
EnumInfo
VariantInfo
VariantId
VariantFamilyId
VariantFieldId
AlgebraicCasePayloadKind
VariantShape
EnumSemanticTable
```

`phalcom-semantic/src/reflection.rs` already contains protocol-neutral:

```rust
EnumReflection
VariantReflection
VariantFamilyReflection
VariantFieldReflection
SpecializedVariantFieldReflection
ExactCaseTypeReflection
```

P10 must extend/converge this authority rather than create parallel `C4Variant*` metadata.

## 3.6 Current static evaluator

`phalcom-semantic/src/static_eval/machine.rs` already uses:

```text
explicit Work stack
explicit SavedFrame stack
StaticEvalResult::{Value, KindedTerm}
local slots keyed from checked BindingId
StaticValueStore
QueryBudget
CancellationToken
checked static applications
checked static constructions
```

The existing evaluator directly supports `Expr::If`.

Block execution currently accepts:

```text
Statement::Expr
Statement::Let
```

and rejects other statements as `UnsupportedExpression`.

`return`, `break`, and `continue` therefore require evaluator-control semantics rather than isolated statement cases.

## 3.7 Current `while` mismatch

Current parser behavior for plain:

```telos
while condition {
    body
}
```

is still:

```text
parser
  ↓
Expr::MethodCall
    receiver = literal block containing condition
    selector = "whileTrue"
    body     = literal block
```

There is no ordinary `Expr::While`.

`while let`, separately, already has `Expr::WhileLet`.

The consolidated C4 specification now requires:

```text
static while = explicit semantic control construct
user whileTrue = ordinary method
```

Therefore P10 must restore explicit source/semantic identity for plain `while` before static execution can support it.

The runtime compiler must retain the current observable runtime behavior, including the guarded sacred-selector/fallback semantics, unless a separate ratified decision explicitly changes runtime `while`.

## 3.8 Current ontology after documentation consolidation

The merged normative documents explicitly preserve:

```text
TypePack
LabeledTypePack
CompleteTypePack
```

as canonical C3 **kinded pack domains**.

They are **not** generic retirement targets.

They remain distinct from proper first-class:

```text
PositionalPack<T>
LabeledPack<T>
CompletePack<T>
```

The migration/census concern is instead obsolete public dual ontology, especially `RecordFields`-facing or incorrect Pack-vs-kind-domain presentation.

This plan must not mass-delete legitimate kind-domain terminology.

> Re-read all named live paths before editing. Adapt mechanical drift locally. Treat architectural drift as a STOP / CONSULT condition.

---

# 4. Required reads before implementation

Read these before editing:

```text
AGENTS.md

docs/workflow/implementation-record-lifecycle-convention.md
docs/workflow/luna-patch-grade-plan-schema.md

docs/decisions/accepted/0090-structural-static-evaluation-packs-products-sums-and-binding-semantics.md

docs/specs/structural-products.md
docs/specs/structural-sums.md
docs/specs/static-evaluation.md
docs/specs/type-level-domains.md
docs/specs/type-members.md
docs/specs/callables/arguments.md
docs/specs/vm/products.md

docs/implementation/LANG007/LANG007.C4/LANG007.C4-CHECKPOINT.md
docs/implementation/LANG007/LANG007.C4/LANG007.C4-GUIDANCE.md
docs/implementation/LANG007/LANG007.C4/LANG007.C4-requirements-analysis.md

docs/implementation/LANG007/LANG007.C4/LANG007.C4.P2/LANG007.C4.P2-walkthrough.md
docs/implementation/LANG007/LANG007.C4/LANG007.C4.P2/LANG007.C4.P2-handoff.md

docs/implementation/LANG007/LANG007.C4/LANG007.C4.P3/LANG007.C4.P3-walkthrough.md
docs/implementation/LANG007/LANG007.C4/LANG007.C4.P3/LANG007.C4.P3-handoff.md

docs/implementation/LANG007/LANG007.C4/LANG007.C4.P4/LANG007.C4.P4-requirements-analysis.md
docs/implementation/LANG007/LANG007.C4/LANG007.C4.P4/LANG007.C4.P4-luna-patch-grade-implementation-plan.md
```

Re-audit at minimum:

```text
phalcom-semantic/src/data_semantics.rs
phalcom-semantic/src/algebraic_semantics.rs
phalcom-semantic/src/enum_semantics.rs
phalcom-semantic/src/reflection.rs
phalcom-semantic/src/types/store.rs
phalcom-semantic/src/types/substitution.rs

phalcom-semantic/src/static_definition.rs
phalcom-semantic/src/checker/static_body.rs
phalcom-semantic/src/checker/statement.rs
phalcom-semantic/src/checker/expression.rs
phalcom-semantic/src/checker/control.rs
phalcom-semantic/src/checker/flow/**
phalcom-semantic/src/static_eval/**
phalcom-semantic/src/static_value/**

phalcom-ast/src/ast.rs
phalcom-ast/src/parser.rs

phalcom-core/src/compiler/inliner.rs
phalcom-core/src/compiler/lib/expr.rs
phalcom-core/src/compiler/lib/jumps.rs

phalcom-lsp/src/semantic_tokens.rs
phalcom-lsp/src/hover.rs
phalcom-lsp/src/completion.rs

phalcom-repl/src/highlighter.rs
phalcom-repl/src/completer.rs

phalcom-core/tests/core/collections/outgoing_packs_completion.rs
phalcom-semantic/tests/semantic/**
```

Before touching `Expr`, run an exhaustive consumer census rather than relying only on this list.

---

# 5. Normative authority

## 5.1 Structural products

The merged normative model requires:

```telos
trait Product {
    type Kind: ProductKind
    type Shape: ProductShape
}

trait OrderedProduct {
    type OrderedShape: OrderedShape
    type Entries: CompletePack<Type>
    type PositionalEntries: PositionalPack<Type>
    type LabeledEntries: LabeledPack<Type>
}

trait UnorderedProduct {
    type UnorderedShape: UnorderedShape
}
```

Compiler-derived evidence is sealed.

P10 implements **lower ordered structural facts and reflection formation**, not final integrated general Product capability derivation.

## 5.2 Pack ontology

Canonical distinctions:

```text
TypePack / LabeledTypePack / CompleteTypePack
    = compiler kinded domains

PositionalPack<T> / LabeledPack<T> / CompletePack<T>
    = proper first-class Pack specializations
```

Do not collapse these domains.

## 5.3 Sum reflection

Normative public target:

```telos
intrinsic data Variant(
    _ name: Symbol,
    _ shape: ProductShape,
)

type Variants =
    PositionalPack<Variant>
```

Required rules:

```text
variant order = declaration order
same Symbol may name multiple exact variants
VariantId is exact identity
VariantFamilyId owns same-name family grouping
reflection specializes with owning Sum
reflection is shallow
GADT proof environment is not public Variant metadata
```

P10 may establish the complete shallow semantic facts but must defer final `ProductShape::Unordered(...)` projection for Record-shaped cases to P5 after P4.

## 5.4 Static execution

Static evaluation is semantic evaluation, not VM execution.

Allowed control includes:

```text
if
match
for
while
return
break
continue
```

P10 only owns:

```text
if integration
return
break
continue
plain while
```

P10 does not own P6's `match`, structural `for`, structural patterns, or proof predicates.

## 5.5 Static while

Fixed rule:

```text
while is explicit semantic control
whileTrue has no static intrinsic meaning
```

No P10 implementation may recognize a static loop by selector spelling.

## 5.6 Resource authority

Every control step and loop iteration remains bounded by:

```text
QueryBudget
CancellationToken
```

No hidden independent loop count is permitted.

---

# 6. Takeover state

| Concept | Current authority | P10 use | Must not duplicate |
|---|---|---|---|
| proper type identity | `TypeStore` / `TypeId` | structural classification | reflection-local type identity |
| nominal ordered product | `DataInfo`, `DataShape::Tuple` | classify/materialize ordered structure | runtime layout inference |
| Pack type shape | `TypeData::Pack` | ordered classification | second Pack shape model |
| Tuple type shape | existing `TypeData::Tuple` representation | ordered classification | reconstructed Tuple metadata |
| Sum identity | `EnumInfo` | Sum classification | name-based Sum table |
| Variant identity | `VariantId` / `VariantFamilyId` | shallow Variant facts | Symbol-only variant identity |
| specialization | existing generic substitution/materialization | reflected field types | textual substitution |
| GADT proofs | checker/match proof environment | explicitly excluded from reflection | public proof graph |
| static legality | ordinary checker | authorize control/effects | evaluator-side semantic checker |
| static work execution | `StaticEvaluator` | extend | second evaluator |
| evaluator values | `StaticValueStore` | payload only | control encoded as StaticValue |
| query resource | `QueryBudget` / `CancellationToken` | loop/control accounting | hidden iteration cap |
| Pack runtime | `PackRoot` / `PackSlice` / immediate EmptyPack | resource regressions | benchmark-specific representation |
| P4 Record key calculus | active P4 | excluded | P10 unordered row model |

---

# 7. Architecture

# 7.1 Track A — structural classification below capability/reflection

The required authority direction is:

```text
TypeStore
DataInfo / DataShape
EnumInfo / VariantInfo
canonical specialization
        │
        ▼
structural classification
        │
        ├────────────► later compiler-derived capabilities
        │
        └────────────► reflection descriptors
```

Never:

```text
capability evidence
    ↔ reflection
```

and never:

```text
runtime ProductShape
    → semantic Product classification
```

## 7.1.1 Shared classifier

Introduce one pure semantic classification facade, mechanically expected near:

```text
phalcom-semantic/src/structural.rs
or
phalcom-semantic/src/structural_classification.rs
```

Conceptually:

```rust
pub struct StructuralClassifier<'a> {
    types: &'a mut TypeStore,
    data: &'a DataSemanticTable,
    enums: &'a EnumSemanticTable,
}
```

Exact mutability may differ depending on specialization APIs.

The initial public/internal methods should be separable:

```rust
ordered_product(type_id) -> ...
sum(type_id)             -> ...
```

Do **not** force a single enum that pretends P10 already knows complete unordered reflection.

P5 may later compose:

```text
ordered classifier from P10
unordered classifier from P4/P5
sum classifier from P10
```

into the complete capability/reflection layer.

## 7.1.2 No new durable `QueryKey<TypeId>` yet

Do not immediately add:

```rust
QueryKey::StructuralClassification(TypeId)
```

The existing DB keys are based primarily on stable declaration/body identities.

A snapshot/store-local `TypeId` must not become a durable incremental identity merely for convenience.

P10's classification should initially be:

```text
pure snapshot/compiler semantic query
over canonical already-published products
```

Existing `DataDeclaration` / `EnumDeclaration` query products remain the dependency-visible authorities.

If actual evidence demonstrates that a durable classification query is required, STOP AND CONSULT before adding one.

---

# 7.2 Track A — ordered structural facts

P10 recognizes only finalized ordered categories:

```text
Pack
Tuple / Unit
tuple-shaped data
```

The classifier must preserve:

```text
positional lane
labeled Symbol lane
lane order
required entries
open/closed Pack/Tuple remainder semantics where applicable
specialized component TypeIds
nominal owner identity independently of shape identity
```

Record-shaped data is not reflected into an ordered structure.

P10 does not implement `UnorderedShape`.

## 7.2.1 Reflection representation

The classifier should return shallow semantic facts rather than source/runtime objects.

Conceptually:

```rust
pub struct OrderedProductStructure {
    pub ty: TypeId,
    pub origin: OrderedProductOrigin,
    pub positional_entries: ...,
    pub labeled_entries: ...,
    pub state: StructuralShapeState,
}
```

Possible origin cases:

```text
Pack
Tuple
Data(DeclarationId)
```

Reuse existing canonical Pack/Tuple/data entry structures where practical rather than copying them into a second representation.

`StructuralShapeState` may reuse an existing `Concrete` / `Symbolic` state if suitable.

P10 must not invent a public mutable builder or recursive cons representation for reflection.

---

# 7.3 Track A — Sum and Variant shallow structure

Extend/converge `reflection.rs` rather than introducing another reflection subsystem.

The classifier/reflection layer must be able to answer:

```text
is this TypeId a Sum?
what canonical EnumInfo owns it?
what exact specialization does this TypeId represent?
what are its exact VariantIds in declaration order?
for each Variant:
    family identity
    payload kind
    immediate field identities
    immediate specialized field Type references
    result/exact-case relationship
```

Keep existing:

```rust
EnumReflection
VariantReflection
VariantFamilyReflection
VariantFieldReflection
ExactCaseTypeReflection
```

unless concrete repository evidence shows a bounded replacement is cleaner.

Prefer additive specialized descriptors over parallel reimplementation.

A Record-shaped variant payload may be identified as Record/unordered payload at the shallow categorical level, but P10 must not materialize the final P4-dependent `UnorderedShape`.

---

# 7.4 Track B — evaluator control-transfer architecture

Control transfer is not a static value.

Conceptually:

```rust
enum EvalControl {
    Return(StaticEvalResult),
    Break,
    Continue,
}
```

where:

```rust
enum StaticEvalResult {
    Value(StaticValueRef),
    KindedTerm(TypeId),
}
```

may remain the existing result carrier.

Mechanically equivalent decomposition is allowed.

Required rule:

```text
StaticValueStore contains semantic values.
EvalControl contains evaluator execution state.
```

Never intern:

```text
Return
Break
Continue
```

into `StaticValueStore`.

## 7.4.1 Explicit control boundaries

The evaluator must know where control is consumed.

Required semantic boundaries:

```text
root static body
static-definition application
loop
```

Rules:

```text
Return at root
    -> becomes root result

Return in called static definition
    -> consumed by that application boundary
    -> caller receives ordinary result

Break
    -> consumed by nearest loop

Continue
    -> consumed by nearest loop

Break/Continue reaching static-call/root boundary
    -> invalid checked/evaluator state
```

The Work stack may gain explicit boundary markers or mechanically equivalent unwind metadata.

Do not implement control transfer by:

- clearing the entire Work vector blindly;
- using host-language panic/unwind;
- recursing through Rust call frames;
- scanning source syntax for enclosing constructs;
- putting sentinel values on the value stack.

## 7.4.2 Cleanup during control unwind

Control propagation must preserve evaluator cleanup:

```text
trace pops
saved static-application frames
pending argument state
body restoration
active-definition stack
```

If P10 uses Work-stack unwinding, cleanup Work items must execute while ordinary pending computation is skipped.

---

# 7.5 Track B — explicit `while` semantic identity

Plain `while` should become an explicit AST semantic node.

Conceptually:

```rust
pub struct WhileExpr {
    pub condition: Box<Expr>,
    pub body: LexicalBlock,
    pub range: SourceRange,
}

Expr::While(Box<WhileExpr>)
```

Exact field ownership may adapt to current AST conventions.

Parser:

```text
before:
while ... -> synthetic MethodCall("whileTrue")

after:
while ... -> Expr::While
```

`while let` remains its existing separate `Expr::WhileLet` path and is outside P10 static execution unless mechanically affected by exhaustive matching.

## 7.5.1 Runtime compatibility

This AST migration must not silently change ordinary runtime `while` semantics.

The runtime compiler should lower `Expr::While` through the existing guarded sacred-loop/fallback authority or an exactly equivalent helper:

```text
Expr::While
    │
    ├─ static evaluator:
    │      direct semantic loop
    │
    └─ runtime compiler:
           current guarded fast path
              +
           real whileTrue send fallback where current runtime semantics require it
```

Thus:

```text
source identity is explicit
static semantics do not depend on whileTrue
runtime observable behavior remains unchanged
```

Do not replace the existing runtime deopt/fallback semantics with unconditional jump lowering unless separately ratified.

---

# 7.6 Track C — post-consolidation convergence

The documentation merge has already performed substantial normative consolidation.

P10 therefore does **not** reopen broad C4 documentation design.

It verifies and repairs only residual contradictions between:

```text
implemented P1–P3 code
consolidated normative specs
implementation records
tooling presentation
```

The normative specs should only change if P10 itself establishes a new accepted implemented fact that genuinely requires synchronization.

---

# 7.7 Track C — ontology census

Classify legacy-looking names instead of mass-replacing them.

Required categories:

```text
A — canonical current compiler/kinded-domain authority
B — valid compatibility facade
C — obsolete public ontology/documentation
D — obsolete tooling presentation
E — intentional compatibility/regression test
F — durable/internal metadata representation
G — dead/stale reference
```

In particular:

```text
TypePack
LabeledTypePack
CompleteTypePack
```

are usually Category A under the consolidated specification when they denote C3 kinded domains.

They are not automatically migration debt.

`RecordFields` requires contextual classification:

```text
internal kind/domain machinery may remain
public unordered-reflection ontology should converge to
    UnorderedShape / FieldKey / FieldInfo / UnorderedTail
```

P10 records the distinction. P9 performs final public cutover.

---

# 8. Ownership boundaries

## 8.1 P10 owns

```text
shared non-unordered structural classification framework
ordered product classification
ordered structural reflection facts
Sum classification
shallow specialized Variant facts

static EvalControl representation
return propagation
break/continue propagation
static-if control integration
plain semantic while identity
plain static while evaluation

P1–P3 post-consolidation consistency audit
bounded P1–P3 tooling cleanup
legacy ontology census
Pack/static evaluator deterministic resource harnesses
```

## 8.2 P10 does not own

```text
P4 FieldKey design
Record AST heterogeneous key design
Record row migration
row solver FieldKey migration
Record runtime shape
Record patterns
FieldKey metadata publication

general Product capability derivation
UnorderedProduct capability
UnorderedShape
complete ProductShape projection
final Variant ProductShape for Record payloads

static match
static structural for
Pack/product static patterns
Equivalent/Subtype/Conforms
proof residualization

recursive static computation redesign
normalized residual IR
memoization

cross-module static metadata/materialization closure

final public ontology cutover
P4–P8 docs/tooling convergence
final C4 performance certification
checkpoint closure
```

---

# 9. Global invariants

```text
P10-INV-01  Structural classification consumes canonical semantic facts only.

P10-INV-02  Runtime ProductShape/layout is never semantic classification authority.

P10-INV-03  Capability derivation and reflection are downstream siblings of classification;
            neither proves the other.

P10-INV-04  P10 does not publish incomplete general Product/UnorderedProduct semantics.

P10-INV-05  Pack/Tuple/call labels remain Symbol-only.

P10-INV-06  P4 remains the exclusive owner of generalized unordered FieldKey/row calculus.

P10-INV-07  Equivalent reflected shape never erases nominal declaration identity.

P10-INV-08  Variant exact identity is VariantId; repeated Symbol names remain distinct.

P10-INV-09  Variant family identity is VariantFamilyId, not Symbol alone.

P10-INV-10  Variant order is declaration order.

P10-INV-11  Structural reflection is shallow; nested recursive types remain Type references.

P10-INV-12  GADT proof environments remain matcher-internal.

P10-INV-13  TypePack/LabeledTypePack/CompleteTypePack remain valid C3 kinded domains.

P10-INV-14  Proper PositionalPack/LabeledPack/CompletePack remain distinct from kinded domains.

P10-INV-15  EvalControl is evaluator state and never StaticValue.

P10-INV-16  Untaken if branches remain unevaluated.

P10-INV-17  Return is consumed at the current static-definition application boundary.

P10-INV-18  Break and Continue are consumed only by the nearest semantic loop.

P10-INV-19  Plain static while is never recognized through "whileTrue" spelling.

P10-INV-20  Runtime while behavior remains observationally compatible after AST explicitness.

P10-INV-21  Every loop iteration consumes QueryBudget and checks CancellationToken cooperatively.

P10-INV-22  No independent hidden loop-iteration cap exists.

P10-INV-23  Static control remains host-stack independent.

P10-INV-24  Existing checked BindingId/local-slot ownership remains canonical.

P10-INV-25  P10 does not introduce a durable TypeId-keyed incremental DB identity without consultation.

P10-INV-26  P1–P3 documentation changes describe implemented truth only.

P10-INV-27  P10 performs census/preparation, not final P9 ontology deletion.

P10-INV-28  Performance harnesses establish asymptotic/resource evidence, not P9 certification.

P10-INV-29  Fixed calls continue to require zero first-class Pack construction solely for argument transport.

P10-INV-30  Pack rest/view operations preserve O(1) shared-root semantics.
```

---

# 10. Non-goals

Do not expand P10 into:

- P4 Record migration;
- generalized FieldKey use in ordered structures;
- final `Product`, `UnorderedProduct`, or `Sum` trait derivation;
- complete source-facing ProductShape API;
- static structural `for`;
- static `match`;
- proof predicates;
- pattern language extensions;
- `while let` static pattern semantics;
- recursive P7 evaluator/memoization redesign;
- residual IR;
- metadata schema publication;
- LSP-backed REPL architectural work;
- broad Universe ontology deletion;
- final performance benchmark certification;
- general runtime-control-flow redesign.

---

# 11. Expected impact map

## 11.1 Structural semantic source

Expected:

```text
phalcom-semantic/src/structural.rs                    # likely new
or phalcom-semantic/src/structural_classification.rs # likely new

phalcom-semantic/src/reflection.rs
phalcom-semantic/src/lib.rs

phalcom-semantic/src/data_semantics.rs                # only bounded adapter/helper work
phalcom-semantic/src/enum_semantics.rs                # only bounded reflection helper work
phalcom-semantic/src/types/store.rs                   # inspect; avoid broad representation edits
phalcom-semantic/src/types/substitution.rs            # only if specialization helper requires it
```

Prefer new lower-level structural module over embedding classification into conformance/reflection code.

## 11.2 Static evaluator/checker

Expected:

```text
phalcom-semantic/src/static_eval/machine.rs
phalcom-semantic/src/static_eval/mod.rs
phalcom-semantic/src/static_eval/error.rs

phalcom-semantic/src/checker/static_body.rs
phalcom-semantic/src/checker/statement.rs             # ideally consume, not redesign
phalcom-semantic/src/checker/control.rs
phalcom-semantic/src/static_definition.rs             # only if checked-body control facts require additive data
```

## 11.3 Explicit while frontend/compiler — after synchronization barrier only

```text
phalcom-ast/src/ast.rs
phalcom-ast/src/parser.rs
phalcom-ast tests

phalcom-semantic/src/checker/expression.rs
phalcom-semantic/src/checker/flow/graph.rs
phalcom-semantic/src/source_index/**
phalcom-semantic/src/advisory/** where exhaustive Expr matching requires it

phalcom-core/src/compiler/lib/expr.rs
phalcom-core/src/compiler/inliner.rs                  # reuse/refactor helper only
phalcom-core/src/compiler/lib/product_opt.rs
phalcom-core/src/modules/semantic_lowering.rs

phalcom-lsp/src/semantic_tokens.rs                    # AST exhaustive traversal only where needed
```

Run an exhaustive `Expr`-consumer census at T6. Do not assume this list is complete.

## 11.4 Convergence/tooling

Potential:

```text
docs/specs/**
docs/implementation/LANG007/LANG007.C4/**
docs/implementation/LANG007/LANG007.C4/LANG007.C4.P1/**
docs/implementation/LANG007/LANG007.C4/LANG007.C4.P2/**
docs/implementation/LANG007/LANG007.C4/LANG007.C4.P3/**

phalcom-lsp/src/semantic_tokens.rs
phalcom-lsp/src/hover.rs
phalcom-lsp/src/completion.rs
phalcom-lsp/tests/**

phalcom-repl/src/highlighter.rs
phalcom-repl/src/completer.rs
phalcom-repl/tests/**

tools/vsphalcom/syntaxes/phalcom.tmLanguage.json
```

Do not modify a file merely because the census found a legacy string.

## 11.5 Performance/resource tests

Prefer existing test harnesses:

```text
phalcom-core/tests/core/collections/outgoing_packs_completion.rs
phalcom-core/tests/core/**pack**
phalcom-semantic/tests/semantic/**static**
```

Create a new narrow resource test module only when existing organization cannot express the discriminator cleanly.

---

# 12. Parallel execution and branch protocol

This section is load-bearing because P4 remains active.

## 12.1 Initial branch/worktree

At T0:

```sh
git fetch origin
git switch main
git pull --ff-only
git status --short
git rev-parse HEAD
```

Required initial result:

```text
HEAD = current origin/main descendant of 419ec6e
working tree clean
```

Create a dedicated P10 branch/worktree according to repository workflow.

Suggested branch:

```text
codex/lang007-c4-p10
```

Do not reuse the P4 implementation branch/worktree.

## 12.2 Pre-barrier safe write-set

Before P4 completes/integrates its Record AST/parser portion, P10 may implement:

```text
T1 structural classifier
T2 ordered reflection
T3 Sum/Variant reflection
T4 EvalControl / return / if
T5 break/continue control carrier
T7 docs/spec audit
T8 tooling/census
Pack/non-while portion of T9 resource harnesses
```

Avoid editing:

```text
phalcom-ast/src/ast.rs
phalcom-ast/src/parser.rs
Record-specific checker/parser files P4 is actively changing
```

unless P4 has already landed the relevant coherent commit into `main`.

## 12.3 Synchronization barrier S1

Before T6:

1. finish and commit all pre-barrier P10 work;
2. fetch latest `origin/main`;
3. confirm P4's T1 frontend slice has been integrated or obtain an explicit stable handoff revision;
4. rebase/merge P10 according to repository policy;
5. rerun only the directly affected pre-barrier compilation/focused checks;
6. inspect the actual post-P4 Record AST/parser architecture;
7. only then add explicit `Expr::While`.

If P4 has broadened `ProductLabel` or otherwise changed AST architecture in a way that conflicts with the P10 while design, STOP AND CONSULT.

## 12.4 Shared lifecycle file discipline

Both plans may need:

```text
LANG007.C4-CHECKPOINT.md
```

Do not let routine branch merges overwrite the other plan's state.

Checkpoint edits must be semantic merges, not “ours/theirs” conflict resolution.

---

# 13. Implementer decision authority

## 13.1 FIXED

The implementer must not change:

- current consolidated TypePack-vs-Pack ontology;
- Symbol-only ordered labels;
- P4 ownership of unordered structure;
- structural-classification authority direction;
- declaration identity rules;
- VariantId / VariantFamilyId identity;
- shallow reflection;
- GADT proof boundary;
- static evaluator ownership;
- EvalControl-vs-StaticValue separation;
- explicit static-while requirement;
- runtime-while compatibility requirement;
- QueryBudget/CancellationToken resource ownership;
- no hidden iteration cap;
- no durable raw TypeId query key without consultation;
- no P9 final migration/certification claim.

## 13.2 MECHANICALLY FLEXIBLE

The implementer may adapt:

- exact structural module filename;
- private classifier/helper names;
- borrowed versus small owned descriptor layout;
- use of existing materialization structs versus thin adapters;
- exact `EvalControl` Rust representation;
- exact Work boundary marker names;
- exact AST `WhileExpr` field boxing;
- compiler helper decomposition for runtime while lowering;
- exact test filenames/selectors;
- ontology census document location.

## 13.3 VERIFY-FIRST

| Assumption | Verify | If false |
|---|---|---|
| `DataInfo`/`DataShape::Tuple` is sufficient nominal ordered authority | `data_semantics.rs` | STOP if runtime shape must become authority |
| Pack/ Tuple type representations expose canonical lane types | `types/store.rs` | adapt helper; STOP before reconstructing from source |
| specialized data shape can expose canonical component TypeIds without scratch-only identity leakage | `materialize_data_product_shape` and TypeStore semantics | STOP if returned IDs are not safe semantic references |
| EnumInfo variants remain declaration ordered | enum builder + tests | fix at canonical owner if bug; do not sort reflection independently |
| existing reflection descriptors can be extended | `reflection.rs` consumers | bounded additive changes allowed |
| static-body ordinary checker already records legal return/break/continue structure | checker control/flow | use existing products; STOP before adding evaluator-side scope checking |
| runtime while semantics still require sacred guard/fallback | accepted runtime decisions + compiler | preserve them |
| explicit AST while can feed existing compiler lowering without semantic regression | inliner/compiler | refactor helper; STOP before behavior change |
| QueryBudget can deterministically drive resource tests | `types/outcome.rs` | use bounded outcome tests; no wall-clock oracle |
| docs consolidation already resolved normative P1–P3 contradictions | current specs | repair only concrete residual contradictions |
| LSP-backed REPL remains separately deferred | accepted REPL decision | do not absorb into P10 |

---

# 14. Global STOP / CONSULT triggers

STOP AND CONSULT if any of the following occurs:

1. ordered structural classification requires P4 FieldKey/row completion;
2. classification requires runtime layout as semantic authority;
3. ordered classification would need a second Pack/Tuple type representation;
4. correct classification requires publishing an incomplete general Product capability;
5. specialized reflected TypeIds would only be valid in a scratch TypeStore clone;
6. a durable `QueryKey<TypeId>` appears necessary;
7. Variant identity cannot remain VariantId-based;
8. repeated names would have to collapse for reflection;
9. GADT proof state appears necessary in public Variant facts;
10. static control requires control sentinels inside StaticValueStore;
11. evaluator control requires host-language recursion/unwinding;
12. `return` cannot be bounded to the current static application frame;
13. ordinary checker legality cannot be reused for `break`/`continue`;
14. static `while` would need recognition of `whileTrue`;
15. explicit AST `while` would necessarily remove existing runtime fallback/override behavior;
16. P4's current AST work conflicts materially with explicit while representation;
17. loop budgeting requires a second independent resource counter;
18. P1–P3 docs appear materially inconsistent with the newly consolidated normative specs;
19. tooling cleanup requires architecture beyond presentation/parity repair;
20. ontology cleanup would delete valid kinded-domain `TypePack` concepts;
21. a benchmark requires timing-sensitive correctness assertions;
22. a representation change is proposed solely to satisfy a resource harness;
23. the same semantic failure persists after one serious corrective attempt;
24. P10 begins implementing P4/P6/P7/P8 ownership to make local tests pass;
25. any invariant above must be weakened.

---

# 15. Debugging budget

## 15.1 Mechanical failures

Up to three coherent correction cycles where evidence shows progress:

```text
inspect
identify concrete cause
make one coherent correction
rerun smallest discriminator
```

Examples:

```text
exhaustive match compilation
import drift
borrow checker
private helper visibility
test selector drift
```

## 15.2 Semantic failures

Record:

```text
Observed:
Hypothesis:
Evidence:
Prediction:
Discriminating test:
```

Permit one serious corrective attempt for the same underlying semantic issue.

If it persists, STOP AND CONSULT.

## 15.3 Architectural failures

Zero speculative redesign attempts.

Consult immediately.

---

# 16. Testing surface analysis

| Coverage | Dimension | Invariants | Required discriminator |
|---|---|---|---|
| CV-01 | ordered classification | 01–07 | Pack/Tuple/Unit/tuple-data classify correctly |
| CV-02 | ordered lane shape | 05, 07 | positional+labeled Symbol lanes preserved |
| CV-03 | nominality | 07 | equal ordered shapes do not equal nominal owners |
| CV-04 | generic specialization | 01, 11 | generic ordered data reflects specialized Type refs |
| CV-05 | Sum classification | 08–12 | exact Sum owner and declaration-order variants |
| CV-06 | repeated variant names | 08–10 | two same-name exact variants remain separate |
| CV-07 | shallow recursion | 11 | recursive payload retains Type ref, no recursive shape expansion |
| CV-08 | GADT boundary | 12 | reflection contains no branch proof environment |
| CV-09 | return | 15–17 | early return stops remaining block |
| CV-10 | nested return | 16–17 | return through nested if |
| CV-11 | call boundary | 17 | callee return does not return caller |
| CV-12 | if laziness | 16 | untaken branch not evaluated |
| CV-13 | break | 18 | nearest loop consumes break |
| CV-14 | continue | 18 | nearest loop consumes continue |
| CV-15 | nested control | 17–18 | nested loops + return propagation |
| CV-16 | explicit while identity | 19–20 | parser produces While; user whileTrue stays MethodCall |
| CV-17 | while evaluation | 19, 21–23 | zero/multiple iterations, mutable locals |
| CV-18 | while Bool rule | 19 | non-Bool condition invalid |
| CV-19 | budget | 21–22 | infinite/static-long loop hits BudgetExceeded |
| CV-20 | cancellation | 21 | loop cooperatively cancels |
| CV-21 | runtime parity | 20 | ordinary while runtime behavior unchanged |
| CV-22 | ontology | 13–14, 27 | kinded domains not falsely retired |
| CV-23 | tooling | 26–27 | P1–P3 presentation remains canonical |
| CV-24 | Pack resource | 29–30 | fixed call no Pack; slices stay root-backed |
| CV-25 | evaluator resource | 21–23, 28 | bounded work scales via QueryBudget |
| CV-26 | P4 isolation | 05–06 | no unordered/FieldKey implementation imported into P10 |
| CV-27 | lifecycle | 26–28 | checkpoint accurately records P4+P10 parallel state |

---

# 17. Verification execution budget

## BUILD mode

Run only exact discriminators for the active task.

No workspace test.

No broad corpus.

No broad clippy.

## STABILIZE mode

Run:

```text
owning semantic test modules
directly affected AST/compiler tests
affected-crate cargo check
P3 Pack resource regressions
```

## CERTIFY mode

P10 certification means its focused G1–G4 gates.

It does **not** mean:

```text
C4 checkpoint certified
workspace release certified
P9 performance certified
```

---

# 18. Tasks

## T0 — Clean takeover, lifecycle amendment, and P4 concurrency map

### Purpose

Establish the exact post-consolidation baseline, create the isolated P10 execution context, amend checkpoint bookkeeping for parallel P4/P10 activity, and freeze file-collision rules before source edits.

### Preconditions

- user-reported clean synchronized `main`;
- P3 complete;
- P4 partial.

### Produces

```text
real starting revision
clean worktree evidence
P10 branch/worktree
P4/P10 collision map
checkpoint P10 ledger entry
synchronization barrier S1
exact focused test selectors
```

### Required actions

Verify:

```sh
git fetch origin
git status --short
git rev-parse HEAD
git rev-parse origin/main
```

Require local/remote equality at takeover or fast-forward before continuing.

Re-read:

```text
P4 plan T1/T2
current checkpoint
current structural/static specs
current AST/parser
```

Amend checkpoint plans ledger conceptually to include:

```text
P4 active — unordered product calculus
P10 active — safe parallel structural/static-control/convergence foundation
```

Do not change `latest_completed_plan = P3`.

Checkpoint `active_plan` metadata currently assumes one active plan. Follow the lifecycle convention conservatively:

- keep P4 as primary `active_plan`;
- add explicit parallel-active state in body/ledger;
- do not invent conflicting second authoritative metadata field unless repository convention supports it.

### Tests

No feature tests.

Run only cheap baseline selectors necessary to prove the intended starting seams still exist.

### Acceptance

T0 closes when repository state, P4 overlap, branch isolation, and lifecycle state are explicit.

---

## T1 — Shared structural-classification framework

### Purpose

Introduce one lower semantic classification authority that P10 ordered/Sum reflection and later P5 capability/reflection consume.

### Preconditions

T0 complete.

### Consumes

```text
TypeStore
DataSemanticTable
EnumSemanticTable
DataInfo
EnumInfo
```

### Produces

A pure classification facade with no P4/unordered implementation dependency.

### Files

Expected:

```text
phalcom-semantic/src/structural.rs                 # create
phalcom-semantic/src/lib.rs
tests under phalcom-semantic/tests/semantic/foundations or adts
```

### Required shape

Prefer:

```rust
pub struct StructuralClassifier<'a> {
    // canonical semantic authorities
}

impl<'a> StructuralClassifier<'a> {
    pub fn ordered_product(...) -> ...;
    pub fn sum(...) -> ...;
}
```

or mechanically equivalent free functions/context.

The classifier must consume exact `TypeId`, not source names.

It must distinguish:

```text
not structurally supported
ordered product
sum
```

without fabricating unordered reflection.

Do not add a new incremental DB product in T1.

### Forbidden

- runtime `ProductShape` inspection;
- name recognition such as `"Tuple"`/`"Pack"` unless exact intrinsic identity is first resolved canonically;
- automatic `QueryKey::StructuralClassification(TypeId)`;
- capability/conformance lookup as classification input.

### Tests

CV-01 and CV-05 smoke discriminators:

```text
ordinary scalar -> no ordered/sum classification
Pack -> ordered route available
Tuple -> ordered route available
tuple data -> ordered route available
enum/sum -> sum route available
record data -> not accidentally treated as ordered
```

### Acceptance

One shared lower authority exists and no caller needs to rediscover structural category independently.

---

## T2 — Ordered-product classification and reflection facts

### Purpose

Fully classify finalized ordered structural forms and expose shallow ordered semantic facts suitable for later P5 public `OrderedShape`/`Entries` projection.

### Preconditions

T1 complete.

### Consumes

```text
TypeData::Pack
Tuple representation
Unit alias/canonical Tuple<>
DataInfo/DataShape::Tuple
data specialization/materialization
```

### Produces

Canonical ordered structural descriptor/projection for:

```text
Pack
Tuple
Unit
tuple-shaped data
generic/specialized tuple-shaped data
```

### Required implementation shape

Preserve two ordered lanes:

```text
positional entries
labeled (Symbol, TypeId) entries
```

For Pack/Tuple, preserve open remainder state where part of the canonical type.

For nominal tuple data:

- retain owner `DeclarationId`;
- specialize components through existing semantic specialization;
- never equate declaration identity merely because entries match another product.

Reflection is shallow:

```text
entry type = TypeId
```

Never recursively ask `entry_type::OrderedShape`.

### P10 boundary

Do not publish final compiler-derived:

```text
Product
OrderedProduct
ProductShape
```

conformance/type-member evidence in this task.

Produce the facts P5 will attach to those capabilities.

### Tests

CV-01–CV-04:

- `Pack()` classification;
- fixed positional Pack;
- mixed positional/labeled Pack;
- open Pack remainder preservation;
- Unit;
- Tuple;
- labeled Tuple;
- tuple data;
- generic tuple data;
- specialized generic tuple data;
- two nominal data types with identical payload retain distinct owners;
- Record data rejected from ordered reflection.

### Performance

Classification should be O(number of immediate entries), excluding existing canonical specialization cost.

No recursive reflection.

### Acceptance

All finalized ordered forms share one classification/reflection authority and P5 will not need to rediscover ordered shape.

---

## T3 — Sum classification and shallow Variant reflection convergence

### Purpose

Establish the P5-ready Sum/Variant structural facts using existing Enum/Variant identity and reflection infrastructure.

### Preconditions

T1 complete. T2 need not be complete except where tuple-shaped Variant payloads consume its adapter; schedule mechanically as needed.

### Consumes

```text
EnumInfo
VariantInfo
VariantId
VariantFamilyId
VariantFieldId
AlgebraicCasePayloadKind
existing reflection.rs
generic specialization
```

### Produces

```text
Sum classification for exact owner TypeId
declaration-order exact Variant sequence
specialized immediate payload field Type references
family relationship
payload category
shallow recursion-safe reflection
```

### Required shape

Extend `reflection.rs` or add a thin adjacent structural descriptor.

Do not replace existing reflection descriptors solely to obtain C4 naming.

For a specialized:

```telos
Result<Int, String>
```

the immediate reflected Variant fields must contain the specialized `Int` / `String` Type references.

For:

```telos
data Node {
    Next(Node)
}
```

reflection contains:

```text
Type(Node)
```

once. No recursive expansion.

For:

```telos
data Value {
    Number(_ value: Int)
    Number(_ value: Float)
}
```

reflection returns two exact variants in declaration order with separate VariantIds and common/related family identity as currently modeled.

### Payload shape boundary

Allowed now:

```text
Singleton
ExplicitEmpty
Tuple payload category + immediate ordered entries
Record payload category + immediate field identities/types
```

But final:

```text
Variant.shape: ProductShape::Unordered(UnorderedShape(...))
```

is deferred until P4/P5.

Do not create `LegacyRecordProductShape`.

### GADT rule

Tests must prove ordinary structure is available without exposing:

```text
BranchProofEnvironment
branch-local equality proof
matcher refinement state
```

### Tests

CV-05–CV-08.

Prefer extending existing ADT reflection tests instead of creating a parallel C4 reflection suite unless current harness organization clearly requires it.

### Acceptance

P5 can consume P10 Sum/Variant facts directly without name re-resolution or GADT proof reconstruction.

---

## T4 — EvalControl, `return`, and static-`if` consolidation

### Purpose

Add explicit evaluator control transfer, implement block `return`, and make existing `Expr::If` propagate control correctly without evaluating untaken branches.

### Preconditions

T0 complete. Independent of T1–T3.

### Consumes

```text
ordinary checked static body
BindingAnalysisIndex
existing Work stack
existing SavedFrame static application stack
existing Expr::If implementation
```

### Produces

```text
EvalControl
root control boundary
static-application control boundary
return statement execution
control-aware block sequencing
control-aware if
```

### Required implementation shape

Add conceptual:

```rust
enum EvalControl {
    Return(StaticEvalResult),
    Break,
    Continue,
}
```

Maintain pending control separately from static semantic values.

Introduce explicit semantic boundaries or equivalent Work metadata.

At minimum:

```text
root
static application
future loop
```

#### Return statement

Evaluate:

```telos
return
```

as:

```text
Return(Unit)
```

and:

```telos
return expr
```

as:

```text
evaluate expr
Return(result)
```

Use checked expression/denotation facts for kinded/static result forms. Do not resolve return expressions by source name in the evaluator.

#### Block behavior

Once control is pending:

```text
remaining ordinary statements in current block are skipped
```

but evaluator cleanup/boundary work still executes.

#### Static application

Given:

```telos
type Inner(): Int {
    return 1
    999
}

type Outer(): Int {
    let x = Inner<>()
    x + 1
}
```

result:

```text
Outer<> == 2
```

not an outer return of `1`.

`FinishStaticApplication` or its replacement is the callee return boundary.

#### Existing if

Do not redesign branch selection.

Existing:

```rust
Work::AfterIf
```

already ensures only the selected branch is scheduled.

Modify only enough for:

```text
Return from selected branch -> propagate
Return/invalid operation in untaken branch -> never execute
```

### Checked-body verification

Verify the ordinary checker correctly establishes return legality/result expectation for static definitions.

If static kind-result definitions require additional checked return facts, add them at the checker/static-body seam. Do not move kind checking into runtime-style evaluator resolution.

### Tests

CV-09–CV-12.

Required cases:

```text
early return
return without value where Unit valid
nested if return
else return
untaken invalid branch
callee return boundary
nested application return
statements after return not evaluated
budget/cancellation still propagate as terminal outcomes
```

### Acceptance

Static block return semantics work using explicit evaluator control, and existing static if semantics remain lazy.

---

## T5 — `break` / `continue` control carrier and loop-boundary foundation

### Purpose

Complete the evaluator-control representation for loop transfer before the explicit plain-while frontend lands.

### Preconditions

T4 complete.

### Produces

```text
Break
Continue
loop boundary abstraction/Work marker
invalid escaped loop-control handling
```

### Required shape

The evaluator may recognize checked:

```rust
Statement::Break
Statement::Continue
```

and produce the corresponding control.

But positive consumption waits for T6's semantic loop.

A loop boundary must define:

```text
Break    -> consume, exit loop
Continue -> consume, next condition
Return   -> do not consume; propagate outward
```

A static-call boundary encountering Break/Continue is invalid.

A root boundary encountering Break/Continue is invalid.

Normally the ordinary checker should prevent these states; evaluator validation remains a defensive invariant, not a second legality checker.

### Forbidden

- searching the AST upward for nearest loop;
- global loop-depth counters as semantic ownership;
- converting Break/Continue to Unit;
- swallowing Return at loop boundary.

### Tests

Do not manufacture broad user-visible loop tests before T6.

Add only narrow evaluator/unit discriminators if naturally expressible through existing checked-body harnesses.

Re-run T4 return gate after control carrier expansion.

### Acceptance

The evaluator has a complete control vocabulary and an explicit place for the nearest loop to consume loop-only control.

---

## T6 — Synchronization barrier, explicit `while` AST identity, and static while

### Purpose

After P4's frontend work is integrated, make plain `while` an explicit semantic AST construct, preserve runtime behavior, and implement bounded direct static evaluation.

### Preconditions

- T4 and T5 complete;
- synchronization barrier S1 complete;
- latest P4 frontend changes integrated/rebased;
- AST/parser architecture re-audited.

### Step T6.0 — post-P4 AST census

Before editing:

```sh
rg -n "Expr::WhileLet|Expr::If|MethodCallExpr|whileTrue" \
  phalcom-ast phalcom-semantic phalcom-core phalcom-lsp phalcom-repl
```

Build the exact exhaustive-consumer list.

### Step T6.1 — explicit AST

Introduce ordinary:

```rust
Expr::While(...)
```

for plain `while`.

Keep:

```rust
Expr::WhileLet(...)
```

unchanged as a distinct refutable-pattern construct.

Parser must no longer lower plain source `while` to MethodCall.

### Step T6.2 — ordinary checker/flow/source-index migration

Every semantic consumer should see an explicit loop.

Do not duplicate loop analysis.

Reuse existing checker loop-frame/fixpoint infrastructure where appropriate.

The existing checker already has loop-control flow machinery for `for`/`while let`/sacred loop analysis; consolidate rather than creating a second flow algorithm.

### Step T6.3 — runtime compiler parity

Add `compile_while_expr` or equivalent.

Runtime lowering must preserve current behavior.

Preferred direction:

```text
semantic Expr::While
        ↓
compiler-owned while lowering
        ↓
existing sacred while fast-path machinery
        +
existing real-send fallback
```

If the existing inliner only accepts `MethodCallExpr`, factor its while-lowering core into a helper consumable by both:

```text
explicit While
legacy/direct whileTrue call
```

Do not make `Expr::While` unconditional jump-only if that changes existing override/deopt semantics.

### Step T6.4 — static evaluator

Evaluate:

```text
condition
require Bool
false -> Unit / normal loop result
true  -> body
```

After body:

```text
normal      -> evaluate condition again
Continue    -> evaluate condition again
Break       -> loop returns Unit
Return(x)   -> propagate x outward
```

Unless normative source semantics already define another ordinary while result, preserve existing Unit/None-equivalent semantic result as established by the checked source model. Verify exact current semantic expectation before freezing the test oracle.

Every condition evaluation/body iteration must:

```text
charge normal evaluator work
check cancellation
```

Do not introduce a bespoke `MAX_STATIC_WHILE`.

### Step T6.5 — `whileTrue` discriminator

Required negative test:

```telos
// ordinary callable/method named whileTrue
```

must remain an ordinary invocation in static semantics and may be rejected as unsupported runtime dispatch where appropriate.

It must never receive loop semantics simply because of its selector.

### Tests

CV-13–CV-21.

Required cases:

```text
0 iterations
1 iteration
many iterations
mutable local condition
continue
break
nested loop nearest-break
nested loop nearest-continue
return through loop
if + loop control
non-Bool condition
budget exhaustion
pre-cancelled evaluation
cancellation during repeated work
ordinary while runtime behavior parity
whileTrue non-intrinsic static negative
while let remains distinct
```

### Acceptance

Plain `while` has canonical semantic identity; static while executes directly and boundedly; runtime semantics do not regress.

---

## T7 — Post-consolidation P1–P3 documentation/spec convergence audit

### Purpose

Verify that the newly merged documentation consolidation accurately reflects shipped P1–P3 behavior and repair only remaining factual contradictions.

### Preconditions

Can begin after T0 and run independently.

### Authoritative direction

```text
current normative specs
        +
actual P1–P3 implementation
        ↓
repair stale implementation/as-built records
```

Do not resurrect pre-consolidation documents as authorities.

### Required audit domains

P1:

```text
let
let mut
const
definite initialization
immutable fields
explicit stored fields
```

P2:

```text
StaticValue
static definitions
closed const
checked static body
query budget/cancellation
materialization seam
```

P3:

```text
Pack
proper Pack specializations
kinded pack domains
Pack views
rest capture
spreading
fixed-call fast path
Symbol labels
```

### Critical consolidated ontology check

Search:

```text
TypePack
LabeledTypePack
CompleteTypePack
PositionalPack
LabeledPack
CompletePack
RecordFields
```

For each hit, first decide whether it is:

```text
valid kinded-domain terminology
proper Pack terminology
internal implementation
legacy/stale public terminology
```

Do not mechanically replace.

### Normative spec editing rule

`docs/specs/**` and TDR-0090 should remain unchanged unless:

1. the consolidation contains a demonstrable contradiction;
2. current accepted implementation proves the correct rule;
3. the repair does not invent future P4–P9 behavior.

A contradiction triggers consultation if two normative authorities disagree materially.

### Tests

Documentation searches and link/format checks only.

No runtime suite merely because prose changed.

### Acceptance

No known P1–P3 implementation record presents stale semantics as current normative behavior.

---

## T8 — P1–P3 LSP/REPL/tooling cleanup and legacy-ontology census

### Purpose

Align current tooling presentation with already-implemented P1–P3 semantics and produce the durable census P9 will use for final migration.

### Preconditions

T0 complete; may run in parallel with structural/static work.

### LSP scope

Audit:

```text
semantic token classification
keyword hover
completion
source-index-based type/static presentation
Pack terminology
binding terminology
```

Do not implement P4 heterogeneous Record-key UX.

### REPL scope

Audit existing:

```text
highlighter
completer
parser-facing keyword handling
```

Do not un-defer or build the separate LSP-backed REPL architecture.

### VS Code grammar

Repair only concrete stale syntax/presentation rules.

### Ontology census

Create a durable P10 census document, suggested:

```text
docs/implementation/LANG007/LANG007.C4/LANG007.C4.P10/
  LANG007.C4.P10-legacy-ontology-census.md
```

Each entry should record:

```text
symbol/term:
path:
usage:
category: A|B|C|D|E|F|G
current semantic meaning:
P9 action:
keep | migrate presentation | delete facade | investigate
```

At minimum census:

```text
TypePack
LabeledTypePack
CompleteTypePack
RecordFields
RecordRow
TupleEntries
Pack aliases/views
legacy reflection names
old let/const terminology
```

### Fixed census rule

Do not mark internal representation obsolete merely because the public source vocabulary changed.

### Tests

Run focused:

```text
phalcom-lsp directly relevant tests
phalcom-repl directly relevant tests
grammar JSON parse/validation
```

Retain existing classified REPL baseline failures as D unless current edits cause them.

### Acceptance

P1–P3 tooling has no known P10-owned presentation drift and P9 has a classified migration map instead of a blind search/replace list.

---

## T9 — Pack and static-evaluator performance/resource harnesses

### Purpose

Establish deterministic regression harnesses for P3 Pack asymptotics and P2/P10 static evaluator resource behavior without claiming final P9 performance certification.

### Preconditions

Pack half may begin after T0.

Static control/while cases requiring T6 wait for T6.

## T9.1 Pack resource discriminators

Preserve:

```text
fixed calls -> zero Pack machinery
rest/view -> root-backed O(1)
lane slicing -> opposite lane preserved
fused spreads -> one builder/finalization path
no recursive slice chains
```

Useful scale discriminators:

```text
repeated positional rest slicing
repeated labeled rest slicing
alternating lane rest slicing
large fixed Pack shape
large fused multi-spread contribution
```

Prefer structural inspection/assertions over timing.

If PackSlice can point to another PackSlice today despite the intended root rule, classify the live behavior before changing it; do not silently rewrite representation under a harness task.

## T9.2 Static evaluator resource discriminators

Use `QueryBudget` deterministically.

Cases:

```text
linear statement sequence
deep nested if
nested static applications
early return short-circuits later work
many while iterations
infinite while -> BudgetExceeded
cancelled before evaluation
cancelled during long evaluation
```

Required relationship:

```text
more semantic work consumes more query steps
```

Do not require exact wall-clock ratios.

Exact budget-step counts should only be asserted when they are a deliberate stable contract; otherwise assert bounded success/failure around generous/discriminating thresholds.

## T9.3 No benchmark-driven production redesign

If tests reveal asymptotic failure:

```text
classify
identify owning path
consult if architecture changes
```

Do not rewrite core representation merely to make a microbenchmark aesthetically better.

### Acceptance

The repository has deterministic regressions capable of catching:

```text
fixed-call Pack allocation regressions
copying Pack rest
nested Pack slice chains where prohibited
unbounded static while
lost evaluator budget/cancellation checks
obvious superlinear evaluator control handling
```

without claiming final performance certification.

---

## T10 — Stabilization, focused gates, lifecycle closure, walkthrough, handoff

### Purpose

Prove the extracted P10 work coherently, reconcile P4/P10 lifecycle truth, and hand stable foundations to later P5/P6/P9 work.

### Preconditions

T1–T9 complete or explicitly excluded by a recorded consultation.

### Required final review

Audit the final P10 diff against every `P10-INV-*`.

Confirm explicitly:

```text
no P4 unordered implementation
no public incomplete Product capability
no second reflection authority
no second evaluator
no static whileTrue intrinsic
no valid TypePack kinded-domain deletion
no P9 certification claim
```

### Lifecycle update

Checkpoint must record:

```text
P4 current actual state
P10 final actual state
stable P10 interfaces
verification evidence
remaining P5/P6/P9 responsibilities
synchronization revision(s)
```

P10 completion does **not** make P5 automatically complete.

### Deliverables

Create:

```text
docs/implementation/LANG007/LANG007.C4/LANG007.C4.P10/
  LANG007.C4.P10-luna-patch-grade-implementation-plan.md
  LANG007.C4.P10-walkthrough.md
  LANG007.C4.P10-handoff.md
  LANG007.C4.P10-legacy-ontology-census.md
```

The walkthrough records actual shipped behavior.

The handoff tells P5/P6/P9 which P10 facts are ready to consume.

---

# 19. Verification gates

## G1 — Structural classification and reflection gate

### Proves

```text
ordered classification
ordered reflection
Sum classification
Variant identity/order/specialization
shallow recursion boundary
GADT proof boundary
```

### Required coverage

```text
CV-01 through CV-08
CV-26
```

### Expected focused command shape

Use exact live module names established by T0/T1, conceptually:

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' \
cargo test -p phalcom-semantic --test semantic structural_classification -- --nocapture

RUSTFLAGS='' RUSTC_WRAPPER='' \
cargo test -p phalcom-semantic --test semantic adts::reflection -- --nocapture
```

If tests live in existing modules, prefer exact selectors instead of creating umbrella tests.

### Pass means

- P10 structural facts are canonical;
- no P4 dependency leaked in;
- no recursive reflection.

Do not broaden after pass.

---

## G2 — Static control gate

### Proves

```text
return
if propagation
break/continue
explicit while
budget
cancellation
runtime while parity
whileTrue negative
```

### Required coverage

```text
CV-09 through CV-21
```

Expected focused test families:

```text
checker static-body tests
semantic static-definition integration
static evaluator control tests
AST parser while tests
runtime compiler while parity tests
```

Run only selectors directly covering these concerns.

### Pass means

Static control semantics are coherent from parser/checker through evaluator while preserving ordinary runtime behavior.

---

## G3 — Convergence/tooling gate

### Proves

```text
post-consolidation P1–P3 truth
tooling presentation
ontology classification
no accidental valid kind-domain retirement
```

### Required coverage

```text
CV-22
CV-23
CV-27
```

Run focused LSP/REPL tests and documentation/census consistency checks.

Known unrelated baseline failures remain classified rather than absorbed.

---

## G4 — Resource and affected-crate gate

### Proves

```text
P3 Pack performance invariants
static evaluator resource controls
directly affected crates compile together
P4 isolation
```

### Required coverage

```text
CV-24
CV-25
CV-26
```

At minimum re-run the established P3 fixed-call resource discriminator:

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' \
cargo test -p phalcom-core --test core \
static_calls_and_static_tuple_emit_no_pack_machinery -- --nocapture
```

and the directly relevant outgoing Pack/resource selectors.

Run affected-crate check after source stabilizes, expected set:

```sh
RUSTFLAGS='' RUSTC_WRAPPER='' \
cargo check \
  -p phalcom-ast \
  -p phalcom-semantic \
  -p phalcom-core \
  -p phalcom-lsp \
  -p phalcom-repl
```

Only include crates actually touched if the live workspace makes the full set wasteful.

---

# 20. Final focused acceptance matrix

| Coverage | Minimum evidence | Required result |
|---|---|---|
| CV-01 | ordered category tests | PASS |
| CV-02 | lane/reflection tests | PASS |
| CV-03 | nominality test | PASS |
| CV-04 | specialization test | PASS |
| CV-05 | Sum classification | PASS |
| CV-06 | repeated-name Variant test | PASS |
| CV-07 | recursive shallow reflection | PASS |
| CV-08 | no-GADT-proof reflection test/audit | PASS |
| CV-09 | early return | PASS |
| CV-10 | nested-if return | PASS |
| CV-11 | static-call return boundary | PASS |
| CV-12 | untaken branch | PASS |
| CV-13 | break | PASS |
| CV-14 | continue | PASS |
| CV-15 | nested loop/return control | PASS |
| CV-16 | explicit while AST + whileTrue distinction | PASS |
| CV-17 | while iterations | PASS |
| CV-18 | Bool condition | PASS |
| CV-19 | loop budget | PASS |
| CV-20 | loop cancellation | PASS |
| CV-21 | runtime while parity | PASS |
| CV-22 | ontology audit | PASS |
| CV-23 | tooling focused parity | PASS or unrelated baseline classified |
| CV-24 | Pack resource invariants | PASS |
| CV-25 | evaluator resource harness | PASS |
| CV-26 | P4 isolation audit | PASS |
| CV-27 | lifecycle truth | PASS |

Any missing required row keeps P10 `PARTIAL / UNVERIFIED`.

---

# 21. Performance/resource contract

P10 does not define final numerical benchmark thresholds.

It establishes architecture-sensitive evidence.

## 21.1 Ordered reflection

Expected:

```text
classification immediate-shape traversal = O(n)
reflection immediate entries            = O(n)
recursive contained type                = O(1) reference, not recursive expansion
```

## 21.2 Pack

Inherited requirements:

```text
lane length                  O(1)
one-step rest/view           O(1)
finalization                 O(n)
multi-spread construction    O(total contributions)
fixed call                   no required first-class Pack allocation
```

## 21.3 Static evaluator

Expected:

```text
one loop iteration -> bounded finite additional Work/budget charge
control unwind     -> proportional to skipped continuation depth
no host recursive loop execution
no unbudgeted infinite static while
```

A `return` that exits early should not execute or budget-charge ordinary semantic work after the return site, apart from required cleanup/control-boundary processing.

---

# 22. Checkpoint bookkeeping

## 22.1 At P10 start

Record:

```text
P10 PROPOSED -> IN_PROGRESS
completion -> PARTIAL
verification -> UNVERIFIED
starting revision
parallel with P4
P4 remains primary active plan unless lifecycle schema is deliberately amended
```

## 22.2 During P10

Update checkpoint only when:

- structural classifier becomes stable;
- ordered/Sum reflection facts become stable;
- EvalControl architecture becomes stable;
- explicit while lands;
- a coherent gate completes;
- ontology census reveals a durable migration decision;
- a consultation changes architecture;
- a new baseline issue is discovered.

Do not log every commit.

## 22.3 At P10 completion

Record:

```text
final revision
actual implemented tasks
G1–G4 evidence
deferred failures
P4 current status
P5 inputs supplied by P10
P6 inputs supplied by P10
P9 inputs supplied by P10
remaining integration boundaries
```

---

# 23. P10 walkthrough contract

Create:

```text
LANG007.C4.P10-walkthrough.md
```

Required contents:

1. starting and final revisions;
2. branch/worktree and P4 synchronization history;
3. structural-classification architecture actually implemented;
4. ordered forms actually supported;
5. Sum/Variant reflection changes;
6. EvalControl representation;
7. return/break/continue semantics;
8. explicit while AST/compiler/static semantics;
9. runtime while compatibility result;
10. documentation/tooling convergence work actually performed;
11. ontology census summary;
12. Pack/static evaluator resource harnesses;
13. deviations from plan;
14. consultations;
15. tests added;
16. tests actually run;
17. tests deliberately deferred;
18. baseline/unrelated failures;
19. verification classification;
20. exact P5/P6/P9 follow-up boundaries.

---

# 24. P10 handoff contract

Create:

```text
LANG007.C4.P10-handoff.md
```

It must tell the next architect/implementer:

```text
current revision
P4 current state
P10 final state

structural classifier entry points
ordered structural descriptor
Sum/Variant reflection entry points
what remains P4-dependent

EvalControl contract
static application return boundary
loop boundary semantics
explicit While AST contract
runtime compiler while lowering contract

ontology census location
resource harness selectors

P5:
    compose P10 ordered + P4 unordered
    derive sealed capabilities
    finish ProductShape/Variant shape projection

P6:
    consume EvalControl
    add match/for/pattern/proof semantics
    do not redesign while

P9:
    consume census
    complete public/tooling/docs migration
    run final certification
```

Include explicit “do not redesign” guidance.

---

# 25. Completion truth table

| Evidence | Meaning | Completes P10? |
|---|---|---:|
| structural module exists | source slice | No |
| Ordered reflection tests pass | structural track partially proven | No |
| Sum/Variant tests pass | structural track advanced | No |
| EvalControl source compiles | control carrier exists | No |
| return tests pass | static control partial | No |
| explicit while parses | frontend identity exists | No |
| static while works | static-control track advanced | No |
| ontology census exists | P9 prep exists | No |
| resource harnesses exist | evidence mechanisms exist | No |
| G1–G4 pass | focused P10 verification | Potentially |
| walkthrough/handoff complete | lifecycle closure | Required |
| P4 complete | separate plan | Not required for P10 pre-barrier work; required only for the T6 synchronization assumption if frontend collision remains |
| P5 complete | downstream | No |
| P6 complete | downstream | No |
| P9 complete | downstream | No |
| workspace release green | stronger external evidence | Not required |

---

# 26. Downstream scope after P10

## 26.1 P5 after P10 + P4

P5 should become an integration/capability plan:

```text
consume P10 ordered classifier
consume P10 Sum classifier
consume P4 unordered facts
        ↓
derive sealed Product
derive sealed OrderedProduct
derive sealed UnorderedProduct
derive sealed Sum
        ↓
form final ProductShape
form OrderedShape
form UnorderedShape
finish Variant(name, shape)
publish source-facing reflection members
```

P5 must not reimplement P10 classification.

## 26.2 P6 after P10

P6 should consume:

```text
EvalControl
explicit semantic While
ordered/Sum reflection
P4/P5 complete structural facts
```

and focus on:

```text
static match
structural static for
Pack/Product structural patterns
heterogeneous per-iteration refinement
Equivalent
Subtype
Conforms
proof uncertainty/residual behavior
```

P6 must not reintroduce a `whileTrue` static intrinsic.

## 26.3 P9 after P10

P9 receives:

```text
post-consolidation P1–P3 docs baseline
tooling cleanup baseline
classified ontology census
Pack resource harnesses
static evaluator resource harnesses
```

and remains responsible for:

```text
P4–P8 documentation convergence
final public ontology migration
final Universe/core reflection helpers
full LSP/REPL/editor closure
fuzz/regression closure
final performance certification
C4 checkpoint acceptance
```

---

# 27. Plan self-review

Before execution, verify:

- [x] grounded against `419ec6e`;
- [x] post-consolidation specs treated as normative;
- [x] P3 is complete;
- [x] P4 remains independent and active;
- [x] P4 FieldKey/Record ownership preserved;
- [x] TypePack kinded domains are not falsely retired;
- [x] shared classification is below capability/reflection;
- [x] no durable TypeId query key is assumed;
- [x] ordered reflection is bounded to finalized ordered forms;
- [x] nominality is preserved;
- [x] Sum identity uses existing canonical IDs;
- [x] repeated Variant names remain distinct;
- [x] reflection is shallow;
- [x] GADT proof metadata remains private;
- [x] EvalControl is not StaticValue;
- [x] return application boundary is explicit;
- [x] loop control boundary is explicit;
- [x] current static if is reused;
- [x] plain while becomes explicit semantic AST;
- [x] whileTrue is never static-control authority;
- [x] runtime while behavior is preserved;
- [x] AST conflict with P4 is serialized through S1;
- [x] docs task accounts for completed consolidation;
- [x] tooling scope excludes LSP-backed REPL redesign;
- [x] ontology census precedes P9 deletion;
- [x] resource harnesses avoid timing-sensitive semantics;
- [x] P10 does not claim P5/P6/P9 completion;
- [x] walkthrough/handoff obligations exist;
- [x] debugging and escalation rules are bounded.

---

# 28. Dependency graph

```text
                         ┌── T2 Ordered classification/reflection ──┐
                         │                                         │
T0 ─────► T1 classifier ─┤                                         │
                         │                                         ├─────┐
                         └── T3 Sum/Variant reflection ─────────────┘     │
                                                                        │
T0 ─────► T4 EvalControl/return/if ─────► T5 break/continue ──┐          │
                                                              │          │
P4 T1 frontend ────────────────────────────────────────────────┤          │
                                                              ▼          │
                                                         S1 sync         │
                                                              │          │
                                                              ▼          │
                                                         T6 while ───────┤
                                                                        │
T0 ─────────────────────────────► T7 docs convergence ───────────────────┤
                                                                        │
T0 ─────────────────────────────► T8 tooling/census ─────────────────────┤
                                                                        │
P3 ───────────────► T9 Pack resource harnesses ──────────────────────────┤
T4/T6 ────────────► T9 evaluator resource harnesses ─────────────────────┤
                                                                        │
                                                                        ▼
                                                                  T10 closure
```

T1–T5, T7, T8, and part of T9 are deliberately executable while P4 continues.

T6 is deliberately serialized after the P4 frontend synchronization barrier because both plans otherwise need `phalcom-ast/src/{ast,parser}.rs`.

---

# 29. Final execution directive

Implement P10 as **shared foundation extraction**, not as partial uncontrolled execution of P5, P6, and P9.

The three controlling architectural sentences are:

> Structural capability and reflection must both consume one lower canonical semantic classification authority.

> Static `return`, `break`, `continue`, and `while` are evaluator control semantics over already-checked Telos constructs; they are not StaticValues and are not runtime method dispatch.

> Documentation/tooling migration must respect the consolidated distinction between valid C3 kinded Pack domains and proper C4 first-class Pack types, and must prepare rather than prematurely execute P9's final public cutover.

If an implementation shortcut violates any of those directions, stop rather than cementing a second semantic authority.
