---
id: LANG007.C4.P6
kind: implementation-plan
title: "Static Control Flow, Structural Iteration, Matching, and Proof Predicates"
status: PROPOSED
completion: NOT_STARTED
verification: UNVERIFIED
branch: main
prepared_against_revision: 73ea73f4622417510c39850e61664cf8c2c4a900
prepared: 2026-10-12
planner_grade: luna-patch-grade
---

# LANG007.C4.P6 — Luna Patch-Grade Implementation Plan

## 0. Mission

Implement `LANG007.C4.P6` on `main` as the **static-control consumer layer** over the semantic authorities already established by P2/P3/P4/P5/P10 and LANG005.C6.

P6 owns:

```text
ordinary static match
finite static structural for
static pattern binding/destructuring
branch proof-environment transport
proof predicates:
    Equivalent
    Subtype
    Conforms
P6-level unresolved propagation
budget/cancellation/resource behavior
incremental publication of the new checked products
```

P6 does **not** own:

```text
a second pattern resolver
a second GADT engine
a second structural classifier
runtime Iterator execution for static structural for
recursive generic static computation
normalized residual IR
durable residual serialization
cross-module residual metadata
VM execution of static code
reimplementation of while
P4's unfinished generalized Record-pattern authority
```

The implementation law is:

> **Static control executes ordinary Telos semantics over canonical semantic static values and canonical proof products. It does not create a second structural language.**

---

# 1. Baseline and source of truth

## 1.1 Pinned repository baseline

Implement against:

```text
repository: aureat/phalcom-lang
branch:     main
revision:   73ea73f4622417510c39850e61664cf8c2c4a900
commit:     feat(LANG007): implement Product/Sum reflection and compiler-derived structural capabilities (P5)
```

Before editing, rerun:

```bash
git status --short
git rev-parse HEAD
git rev-parse origin/main
git log -1 --oneline
```

Expected starting revision:

```text
73ea73f4622417510c39850e61664cf8c2c4a900
```

If `main` has moved, re-audit every file/symbol named in this plan before applying the patch. Do not mechanically implement against stale offsets or stale struct layouts.

## 1.2 Lifecycle inconsistency to repair first

`docs/implementation/LANG007/LANG007.C4/LANG007.C4-CHECKPOINT.md` has stale front matter that still names P4 as active even though the body ledger and live source record P5 as complete.

Treat as current truth:

```text
P5 = COMPLETE / IMPLEMENTED / FOCUSED_TESTED
P6 = PROPOSED / NOT_STARTED / UNVERIFIED
```

T0 must update the lifecycle record before or with the first P6 implementation commit.

Do not rewrite historical P4/P5 records except where the lifecycle convention requires a current pointer.

---

# 2. Read-first implementation authority

Read these before production edits:

```text
AGENTS.md
docs/workflow/implementation-record-lifecycle-convention.md

docs/decisions/accepted/
  0090-structural-static-evaluation-packs-products-sums-and-binding-semantics.md

docs/specs/
  static-evaluation.md
  type-level-domains.md
  type-members.md
  structural-products.md
  structural-sums.md
  generic-constraints.md
  callables/arguments.md

docs/implementation/LANG007/LANG007.C4/
  LANG007.C4-CHECKPOINT.md
  LANG007.C4-GUIDANCE.md
  LANG007.C4.P5-handoff.md
  LANG007.C4.P5-walkthrough.md

docs/implementation/LANG007/LANG007.C4/LANG007.C4.P10/
  LANG007.C4.P10-handoff.md
  LANG007.C4.P10-walkthrough.md
```

Also inspect the current P4 completion state before claiming Record-pattern acceptance.

---

# 3. Stable takeover contracts

P6 must consume these APIs as established authorities.

## 3.1 Structural classification

```text
phalcom-semantic/src/structural.rs
```

Stable takeover symbols:

```rust
StructuralClassifier
StructuralClassification

OrderedProductForm
UnorderedProductForm

OrderedProductStructure
OrderedProductEntry

UnorderedProductStructure
UnorderedProductEntry

SpecializedVariantStructure
SumStructure
```

Canonical categories:

```rust
pub enum StructuralClassification {
    Unsupported,
    OrderedProduct(OrderedProductForm),
    UnorderedProduct(UnorderedProductForm),
    Sum { owner: DeclarationId },
}
```

P6 must not rediscover structure from source syntax or runtime layout.

## 3.2 Structural evidence

```text
phalcom-semantic/src/generic_evidence.rs
```

Stable takeover symbols:

```rust
GenericConformanceKey
GenericConformanceEvidence
GenericConformanceProof
GenericEquivalenceProof
GenericObligationProof

CompilerDerivedConformanceKind
CompilerDerivedConformanceEvidence

ConformanceProofContext
prove_constraint
prove_conformance_with_context
```

Compiler-derived Product/Sum evidence is already integrated into normal generic conformance.

P6 must not fabricate `ImplId`s or introduce a structural-proof side channel.

## 3.3 Member projection

```text
phalcom-semantic/src/member_projection.rs
phalcom-semantic/src/structural_reflection.rs
```

Stable outcome law:

```rust
pub enum StaticMemberProjectionOutcome {
    ReducedValue(StaticValueRef),
    ReducedKinded(TypeId),
    Stuck(MemberProjection),
    Invalid(Box<str>),
    Unknown(UnknownReason),
    Blocked(BlockReason),
    Dynamic(DynamicBoundaryObligation),
    Cancelled,
    BudgetExceeded(BudgetReport),
    InternalFailure(Box<str>),
}
```

Required invariant:

```text
Stuck  != false
Unknown != false
Blocked != false
Dynamic != false
Stuck  != empty iteration
```

## 3.4 Static values

```text
phalcom-semantic/src/static_value/store.rs
```

The existing store is sufficient:

```rust
StaticValueData::Unit
StaticValueData::Bool
StaticValueData::Char
StaticValueData::Int
StaticValueData::Float
StaticValueData::String
StaticValueData::Symbol
StaticValueData::Type
StaticValueData::StaticTerm
StaticValueData::Tuple
StaticValueData::Sequence
StaticValueData::Record
StaticValueData::Data
StaticValueData::Variant
StaticValueData::Pack
```

P6 must not create another static-value arena.

## 3.5 Pattern/match authority

```text
phalcom-semantic/src/match_semantics.rs
phalcom-semantic/src/checker/pattern.rs
phalcom-semantic/src/checker/expression.rs
```

Stable products:

```rust
MatchResolution
MatchArmResolution
MatchResolutionIndex

PatternResolution
PatternSiteResolution
PatternSiteResolutionIndex

ResolvedVariantPattern
ResolvedVariantCandidate
ResolvedFieldPattern

PatternBindingResolution
BranchProofEnvironment

PatternUsefulness
ExhaustivenessResult
```

P6 is an evaluator consumer of these products.

It must not introduce:

```text
StaticPattern
StaticVariantResolver
StaticGadtEnvironment
```

## 3.6 P10 control machinery

```text
phalcom-semantic/src/static_eval/machine.rs
```

Reuse:

```rust
EvalControl
ControlBoundary
ControlDisposition
```

`while`, `break`, `continue`, and `return` already have an explicit static-control architecture. Extend it; do not redesign it.

---

# 4. Current concrete gaps

## 4.1 Checked static products do not publish match semantics

Current:

```rust
pub struct CheckedStaticBody {
    pub expressions: Arc<ExpressionAnalysisIndex>,
    pub bindings: Arc<BindingAnalysisIndex>,
    pub static_operations: Arc<StaticOperationResolutionIndex>,
    pub static_applications: Arc<[CheckedStaticApplication]>,
    pub static_constructions: Arc<BTreeMap<ExpressionId, CheckedStaticConstruction>>,
    pub associated_resolutions: Arc<AssociatedResolutionIndex>,
    pub sequence_literals: Arc<SequenceLiteralResolutionIndex>,
    ...
}
```

Missing:

```text
match_resolutions
pattern_site_resolutions
static structural iteration resolutions
```

`CheckingContext` already owns:

```text
match_resolutions
pattern_site_resolutions
for_iteration_lanes
```

so P6 must publish existing match/pattern products, not recompute them.

## 4.2 Checked static constants have the same match-publication gap

`CheckedStaticExpression` currently publishes expression/binding/static-operation/application/construction/member/sequence facts, but not match products.

Therefore:

```telos
const x = match value { ... }
```

cannot become correct merely by changing `CheckedStaticBody`.

P6 must update **both** checked static carriers:

```text
CheckedStaticBody
CheckedStaticExpression
```

and the `EvaluatorBody` accessors that abstract over them.

## 4.3 The evaluator cannot execute match or for

Current static evaluator supports the P2/P10 subset, including explicit while/control transfer.

It does not execute:

```text
Expr::Match
Statement::For
```

Those currently terminate in the unsupported-expression path.

## 4.4 Ordinary semantic `for` is runtime-protocol based

Current `checker/statement.rs::Statement::For`:

1. analyzes the source expression;
2. calls `resolve_for_iteration_lane`;
3. publishes `ForIterationLaneResolution`;
4. records exact Iterable/Iterator requirement/evidence;
5. checks the body from the runtime iteration item type.

That product is explicitly runtime iteration authority:

```rust
pub struct ForIterationLaneResolution {
    pub source_type: TypeId,
    pub item_type: TypeId,
    pub iterator_type: TypeId,
    pub iterator: ForIterationRequirementResolution,
    pub next: ForIterationRequirementResolution,
}
```

Static structural iteration must not reinterpret this as its own meaning.

## 4.5 Static proof operations are not authorized

Current:

```rust
pub enum StaticIntrinsicOperation {
    NumberAdd,
}
```

There is no current exact static-operation implementation for:

```text
Equivalent
Subtype
Conforms
```

Their canonical source/semantic identity must be established during T0/T9; raw spelling is forbidden.

---

# 5. Target architecture

## 5.1 Static match

```text
source match
    ↓
ordinary checker
    ↓
MatchResolution + PatternResolution + BranchProofEnvironment
    ↓
CheckedStaticBody / CheckedStaticExpression
    ↓
StaticEvaluator
    ↓
resolved pattern × StaticValueStore
    ↓
selected branch only
    ↓
Reduced / Symbolic / terminal
```

P5 reflection follows the exact same route:

```text
T::Shape
    ↓
StructuralReflectionProjector
    ↓
StaticValueData::Data / Variant / Pack / Record
    ↓
ordinary static matcher
```

There is no reflection-specific matcher.

## 5.2 Static structural `for`

```text
source for
    ↓
ordinary syntax/pattern analysis
    ↓
static-phase structural iteration resolution
    ↓
finite semantic structural cursor
    ↓
exact iteration value/refinement
    ↓
ordinary resolved pattern binding
    ↓
body execution
```

There is no:

```text
runtime Iterable
runtime Iterator
iterator()
next()
VM object
runtime native call
```

in this path.

## 5.3 Proof predicates

```text
Equivalent
    -> canonical equivalence proof authority

Subtype
    -> canonical relation authority

Conforms
    -> GenericConformanceKey
    -> prove_conformance_with_context
```

Only semantic refutation becomes public `false`.

---

# 6. Proposed P6 semantic products

Names below are recommended patch names. Rename only if a current repository convention clearly dictates a better name.

## 6.1 Publish existing match products

Add to `CheckedStaticBody`:

```rust
pub match_resolutions: Arc<crate::match_semantics::MatchResolutionIndex>,
pub pattern_site_resolutions: Arc<crate::match_semantics::PatternSiteResolutionIndex>,
```

Add the same fields to `CheckedStaticExpression`.

Extraction rule in `checker/static_body.rs`:

```text
take only products whose expression/range belongs to the current checked static body/expression;
do not clone unrelated callable/module entries.
```

Prefer filtering by `ExpressionId` owner / the existing body expression set. For pattern sites without a direct expression key, filter by the checked body's source range and binding ownership only if the existing index has no stronger owner identity.

Do not invent source-range identity when an existing canonical key is available.

## 6.2 Add a static structural-iteration product

Do **not** reuse `ForIterationLaneResolution`.

Recommended shape:

```rust
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StaticIterationLaneResolution {
    pub source_expression: ExpressionId,
    pub source_range: SourceRange,
    pub source_type: TypeId,
    pub mode: StaticIterationMode,
    pub item_domain: StaticIterationItemDomain,
}

pub type StaticIterationLaneResolutionIndex =
    BTreeMap<ExpressionId, StaticIterationLaneResolution>;
```

Recommended closed semantic mode:

```rust
pub enum StaticIterationMode {
    PackValues {
        lane: StaticPackLane,
    },

    TypePackTerms {
        lane: StaticPackLane,
    },

    OrderedProductValues {
        origin: OrderedProductOrigin,
        lane: StaticPackLane,
    },

    OrderedProductTypes {
        origin: OrderedProductOrigin,
        lane: StaticPackLane,
    },

    UnorderedProductValues,

    UnorderedProductFields,

    ReflectedPack,
}
```

Do **not** add `SumVariants` unless implementation evidence proves it is necessary. `Sum::Variants` is already a Pack and should use the Pack path.

A smaller enum is preferred if the live checker can express all required behavior with fewer cases.

### Item-domain representation

Static iteration must be able to preserve exact heterogeneous alternatives while validating the body soundly.

Recommended:

```rust
pub enum StaticIterationItemDomain {
    Uniform(TypeKnowledge),
    FiniteExact(Box<[TypeKnowledge]>),
    Unknown(TypeKnowledge),
}
```

Rules:

```text
Uniform
    one item type applies to every iteration.

FiniteExact
    checker knows a finite heterogeneous set of possible iteration item types.

Unknown
    structure is statically legal, but exact per-entry type is not known yet;
    body must remain valid under the conservative knowledge supplied.
```

Do not publish actual `StaticValueRef`s in the checked product. Checked products describe semantics; evaluator values belong to a concrete evaluation request.

## 6.3 Checked carriers

Add `static_iteration_lanes` to:

```text
CheckedStaticBody
```

A constant expression has no statement-level `for`, so `CheckedStaticExpression` does not require statement iteration products unless current syntax provides an inline lexical statement expression that can contain `for`. Verify before omitting.

---

# 7. T0 — Rebaseline, lifecycle, and open-boundary gate

## Objective

Freeze exact entry conditions before changing semantics.

## Files

```text
docs/implementation/LANG007/LANG007.C4/LANG007.C4-CHECKPOINT.md
docs/implementation/LANG007/LANG007.C4/LANG007.C4.P6-*.md
docs/specs/static-evaluation.md
docs/specs/generic-constraints.md
phalcom-ast/src/ast.rs
phalcom-ast/src/parser.rs
```

## Steps

### T0.1 Rebaseline

Record:

```bash
git status --short
git rev-parse HEAD
git rev-parse origin/main
```

### T0.2 Repair lifecycle front matter

Set current plan pointers to P6 only after the P6 plan record exists.

Do not alter the checkpoint-wide status beyond what implementation evidence supports.

### T0.3 Re-audit P4 Record-pattern status

Check whether generalized `RecordKeySyntax` has converged into `PatternResolution`.

Current red flag:

```rust
ResolvedRecordFieldPattern {
    label: Box<str>,
    child: Box<PatternResolution>,
}
```

If this still cannot represent heterogeneous `FieldKey`, record P4 Record-pattern completion as an external P6 gate.

P6 must not add `field_key` support in a P6-private pattern type.

### T0.4 Freeze structural pattern syntax

Audit current normative syntax for the minimum Pack/product tail/capture forms.

Do not take examples from old planning records as normative merely because they exist.

If exact grammar is not ratified, implement P6 core match/iteration without new Pack-pattern syntax first and mark the structural-pattern task blocked for consultation.

### T0.5 Freeze proof-predicate carrier identity

Trace how source syntax for:

```telos
Equivalent<A, B>
Subtype<A, B>
Conforms<T, Trait>
```

is represented after parsing/checking.

Required outcome: one canonical semantic identity per operation.

Forbidden:

```rust
if name == "Subtype" { ... }
```

If no canonical declaration/operation identity exists, introduce one through the compiler-owned static semantic surface; do not encode the operation through source text.

## Gate G0

Proceed only when:

```text
baseline revision recorded
lifecycle pointer corrected
P4 dependency classified
structural-pattern syntax classified
proof-operation identity carrier classified
```

---

# 8. T1 — Publish match and pattern products into checked static carriers

## Objective

Make existing ordinary match semantics directly consumable by the evaluator.

## Files

```text
phalcom-semantic/src/static_definition.rs
phalcom-semantic/src/static_constant.rs
phalcom-semantic/src/checker/static_body.rs
phalcom-semantic/src/static_eval/machine.rs
phalcom-semantic/src/db/fingerprint.rs
phalcom-semantic/tests/semantic/integration/static_definition_bodies.rs
phalcom-semantic/tests/semantic/incremental/fingerprints.rs
```

## Patch

### T1.1 `CheckedStaticBody`

Add:

```rust
pub match_resolutions: Arc<MatchResolutionIndex>,
pub pattern_site_resolutions: Arc<PatternSiteResolutionIndex>,
```

### T1.2 `CheckedStaticExpression`

Add the same fields.

This is required for expression-level static match in constants.

### T1.3 Extraction

In both static-checker publication paths:

```text
collect current body's/expression's MatchResolution entries;
collect current body's/expression's PatternSiteResolution entries;
preserve canonical ExpressionId / BindingId / VariantId / TypeId identities;
do not resolve source again.
```

### T1.4 Evaluator abstraction

Extend `EvaluatorBody` accessors:

```rust
fn match_resolutions(&self) -> &MatchResolutionIndex
fn pattern_site_resolutions(&self) -> &PatternSiteResolutionIndex
```

### T1.5 Fingerprints

`checked_static_body_product_fingerprint` currently hashes `Debug` for the complete body product. The new fields will therefore affect it automatically, but add explicit regression tests.

Also ensure the static-constant checked-product fingerprint path reflects the new checked semantics where applicable.

Do not add a second manually maintained hash path if the product's existing complete hash already owns it.

## Tests

1. `CheckedStaticBody` contains the exact `MatchResolution`.
2. `CheckedStaticExpression` contains the exact `MatchResolution`.
3. repeated-symbol variant candidate identity survives publication.
4. edit to a match pattern changes the checked product fingerprint.
5. unrelated source does not spuriously change the product.
6. cache hit returns identical canonical match products.

## Gate G1

No evaluator execution yet. This gate proves publication only.

---

# 9. T2 — Static structural-iteration checker product

## Objective

Stop using runtime `Iterable` semantics as the only checker meaning of `for` in static phase.

## Files

```text
phalcom-semantic/src/checker/analysis.rs
phalcom-semantic/src/checker/context.rs
phalcom-semantic/src/checker/statement.rs
phalcom-semantic/src/checker/static_body.rs
phalcom-semantic/src/static_definition.rs
phalcom-semantic/src/db/fingerprint.rs
phalcom-semantic/src/lib.rs
phalcom-semantic/src/checker/mod.rs
```

## Patch

### T2.1 Add `StaticIterationLaneResolution*`

Place the public semantic product near other checked iteration products.

Do not overload `ForIterationLaneResolution` because that type explicitly contains:

```text
iterator_type
iterator requirement
next requirement
runtime conformance evidence
```

which are forbidden static-structural semantics.

### T2.2 Add context index

```rust
pub static_iteration_lanes: StaticIterationLaneResolutionIndex
```

Propagate through context constructors/clones exactly as other analysis indexes are propagated.

### T2.3 Split `Statement::For` by evaluation phase

Current code unconditionally calls runtime:

```rust
resolve_for_iteration_lane(...)
```

Patch law:

```rust
match ctx.phase {
    EvaluationPhase::Runtime => {
        existing runtime behavior unchanged
    }

    EvaluationPhase::Static => {
        resolve_static_iteration_lane(...)
    }
}
```

Do not try static structural resolution and then silently fall back to runtime Iterator dispatch.

If a source is not legal static structure, emit/retain a static semantic failure.

### T2.4 Static resolver

Introduce a helper under checker/static semantics, not runtime lowering.

Recommended file:

```text
phalcom-semantic/src/checker/static_iteration.rs
```

Input:

```text
checked source expression
canonical source TypeId/TypeKnowledge
TypeStore
DataSemanticTable
EnumSemanticTable
StructuralClassifier
static-domain information
```

Output:

```text
StaticIterationLaneResolution
or rich semantic unavailability
```

### T2.5 Classification rules

At minimum classify:

```text
Pack values / pack-shaped static domains
TypePack/LabeledTypePack/CompleteTypePack where static source syntax permits them
ordered structural values
ordered structural Type values
unordered structural values
unordered structural Type values
already-reflected Pack values such as Sum::Variants
```

Prefer P5 `StructuralClassifier` over hand-written type-name tables.

### T2.6 Body checking

Keep the existing loop flow/fixpoint infrastructure where possible.

Change only the source/item semantic fact.

For `FiniteExact` item domains:

```text
validate the loop body against every reachable exact item knowledge;
join resulting flow states using existing flow machinery;
do not accept an operation solely because one iteration type permits it.
```

For uniform domains:

```text
one ordinary body analysis is sufficient.
```

For unresolved generic domains:

```text
check under the conservative declared/known item domain;
do not invent exact entries.
```

### T2.7 Pattern site publication

Continue using the ordinary `resolve_pattern(...)`.

Record:

```text
PatternSiteKind::ForLane
PatternResolution
bindings
usefulness
```

Do not create a static-only pattern resolution.

### T2.8 Fingerprinting

Publish `static_iteration_lanes` into `CheckedStaticBody`.

Add it to any explicit product hash where the full checked body is not already hashed structurally.

## Diagnostics

Prefer existing semantic diagnostics where they already describe the failure.

Add static-specific diagnostics only for genuinely static iteration laws, for example:

```text
static.iteration_not_structural
static.iteration_open_tail_not_finite
```

Do not duplicate match usefulness/exhaustiveness diagnostics under `static.*`.

## Gate G2

At this gate:

```text
static for checks through a static structural product;
runtime for behavior is unchanged;
the evaluator still need not execute the loop.
```

---

# 10. T3 — General resolved-pattern × static-value matcher

## Objective

Implement one evaluator-side matching engine over canonical `PatternResolution` and `StaticValueStore`.

## Files

Recommended:

```text
phalcom-semantic/src/static_eval/pattern.rs        # new
phalcom-semantic/src/static_eval/mod.rs
phalcom-semantic/src/static_eval/machine.rs
phalcom-semantic/src/static_eval/error.rs
```

Do not add resolver logic here.

## Recommended result type

```rust
pub(crate) enum StaticPatternMatchOutcome {
    Matched(StaticPatternBindings),
    NoMatch,
    Symbolic,
    Invalid(StaticEvalError),
    Cancelled,
    BudgetExceeded(BudgetReport),
}
```

`StaticPatternBindings` should map canonical binding IDs to static evaluator results/values.

For P6 initial scope, ordinary pattern binders that bind runtime/value-domain values should bind `StaticValueRef`.

If a current pattern can bind a kinded static term, represent it through the evaluator's existing `StaticEvalResult` rather than forcing it into `StaticValueRef`.

## Pattern support

Implement the existing resolved pattern family in priority order:

```text
Wildcard
Binding
Char
Tuple
Variant
Or
Record        after/within the P4 authority boundary
List          only if source value is already a supported static sequence
Map           only if static Map value semantics already exist
```

Do not add unsupported runtime values merely to complete a pattern enum arm.

### Wildcard

Always matches. No binding.

### Binding

Always matches and records the canonical `BindingId`.

### Char

Match only `StaticValueData::Char`.

### Tuple

Require canonical tuple static value and exact child arity under ordinary tuple pattern law.

Recurse into children.

### Variant

Use `ResolvedVariantCandidate.variant` and `StaticVariantValue.variant`.

Critical rule:

```text
VariantId equality is authority.
printed selector/base name is not authority.
```

For the matching candidate, project fields by canonical `VariantFieldId`, then recursively match child resolutions.

### Or

Try alternatives in semantic order.

Bindings must obey the already-checked ordinary or-pattern binding contract.

Do not synthesize new binding equivalence.

### Data/reflection values

Reflection data are ordinary `StaticValueData::Data` / `Variant`.

Do not special-case:

```text
ProductShape
Variant reflection object
FieldLookup
UnorderedShapeView
```

by name. Match through canonical declaration/variant identities already retained in the static value.

### Record

Consume the current canonical resolved Record pattern product.

If P4 has not converged heterogeneous keys, do not widen P6's private matcher beyond the published product.

## Resource behavior

Charge budget proportional to visited pattern/value nodes.

Check cancellation during recursive/iterative descent.

Avoid unbounded host recursion for adversarial deep pattern trees if current semantic pattern depth can exceed safe Rust recursion. If necessary, use a local explicit matcher stack.

Do not introduce a new arbitrary depth limit.

## Gate G3

Focused tests prove the matcher independently of `Expr::Match`.

---

# 11. T4 — Execute `Expr::Match` in the explicit static evaluator

## Objective

Integrate match into `StaticEvaluator` without host-recursive evaluation.

## File

```text
phalcom-semantic/src/static_eval/machine.rs
```

## Work-item states

Add explicit work items conceptually equivalent to:

```text
AfterMatchScrutinee
TryMatchArm
EnterMatchArm
FinishMatchArm
```

Use names consistent with current `Work`/continuation enum naming.

## Algorithm

For:

```telos
match scrutinee {
    p0 => b0
    p1 => b1
}
```

execute:

```text
1. resolve current expression ID;
2. obtain exact MatchResolution from EvaluatorBody::match_resolutions();
3. evaluate scrutinee exactly once;
4. iterate semantic arms in source order;
5. call the general static matcher with the arm's PatternResolution;
6. on NoMatch: continue;
7. on Matched:
       save old local values affected by pattern bindings;
       install bindings;
       install arm proof environment;
       evaluate only that arm body;
       restore scoped state;
8. propagate arm result/control;
9. never evaluate later arms.
```

## Resolution failure law

Missing checked `MatchResolution` for an otherwise supported static match is an internal fail-closed error.

Do not re-run checker resolution in the evaluator.

Add a dedicated evaluator error if useful:

```rust
MissingMatchResolution(ExpressionId)
```

rather than reporting a generic unsupported expression.

## Unresolved law

If matching requires an unresolved static operand/projection:

```text
return P6-level Symbolic
```

Do not select an arm.

Do not create residual match IR in P6.

## Untaken branches

Add a regression where an untaken arm contains an otherwise unsupported/invalid static operation.

Expected:

```text
selected arm reduces successfully;
untaken arm is not executed.
```

This is mandatory.

## Constants

Verify:

```telos
const x = match ...
```

through `CheckedStaticExpression`.

Do not certify only block-bodied `type` definitions.

## Gate G4

Static match works for closed concrete values with ordinary resolved patterns.

---

# 12. T5 — Branch proof environment transport

## Objective

Ensure static operations in a matched GADT/refined arm execute under the exact proof facts used by the checker.

## Existing authority

```rust
pub struct BranchProofEnvironment {
    pub bindings: BTreeMap<TypeParameterId, TypeId>,
    pub equalities: Box<[TypeConstraint]>,
    pub local_bindings: BTreeMap<TypeParameterId, LocalType>,
    pub local_equalities: Box<[LocalConstraint]>,
}
```

## Patch

Add an evaluator-local proof frame/reference.

Recommended state:

```rust
active_branch_proofs: Vec<&'a BranchProofEnvironment>
```

or a composed lightweight environment if existing proof APIs require composition.

Do not clone `TypeStore`.

Do not rewrite global canonical types.

When entering a matched arm:

```text
push arm proof environment
```

When leaving:

```text
pop it
```

Proof predicate/member-projection/static-application paths that consult generic evidence must see the active branch facts where semantically applicable.

## Tests

1. GADT arm equality enables the same static relation the checker accepted.
2. sibling arm does not inherit the proof.
3. nested match composes proof scopes.
4. exiting arm restores prior proof environment.
5. no proof is reconstructed from variable names.

## Gate G5

Static evaluator branch semantics and ordinary checker branch semantics agree on proof context.

---

# 13. T6 — Pack structural `for` execution

## Objective

Implement the first complete static-for runtime: finite Pack traversal.

## Files

```text
phalcom-semantic/src/static_eval/machine.rs
phalcom-semantic/src/static_value/store.rs   # read-only helpers only if needed
phalcom-semantic/src/static_eval/error.rs
```

## Cursor law

Use the existing flat/slice-backed Pack representation.

Do not implement iteration as recursive rest construction.

Recommended evaluator state:

```rust
StaticForCursor::Pack {
    pack: StaticPackId,
    positional_index: u32,
    labeled_index: u32,
    mode: ...
}
```

or equivalent borrowing/index state.

## Work items

Conceptually:

```text
StartStaticFor
AdvanceStaticFor
EnterStaticForBody
FinishStaticForBody
```

Reuse:

```rust
ControlBoundary::Loop
control_disposition(...)
```

## Semantics

For each finite entry:

```text
1. obtain exact StaticValueRef;
2. obtain the checked ForLane PatternSiteResolution;
3. match/bind through the general static pattern matcher;
4. bind optional `at` index if supported by the existing `ForStatement`;
5. execute body;
6. Continue -> advance;
7. Break -> finish loop;
8. Return -> propagate;
9. normal completion -> advance.
```

## Complexity

Required:

```text
O(n) traversal
O(1) cursor state
no O(n) suffix allocation per iteration
```

## Tests

```text
empty Pack
one element
many elements
break
continue
return
nested Pack loops
pattern binder
wildcard binder
index binder
budget exhaustion
cancellation
```

## Gate G6

The strongest first vertical should reduce a source equivalent to:

```telos
type Count<*P>: Int {
    let mut n = 0
    for _ in P {
        n += 1
    }
    n
}
```

with a concrete finite Pack/static Pack argument.

---

# 14. T7 — Ordered structural traversal and `Sum::Variants`

## Objective

Exploit P5 ordinary reflection values rather than adding new loop engines.

## Ordered products

P5 exposes ordered reflection members as Packs:

```text
Entries
PositionalEntries
LabeledEntries
```

Preferred semantic route:

```text
projection -> StaticValueData::Pack -> T6 Pack iterator
```

Do not add an `OrderedProductIterator`.

## Sum

P5 defines:

```telos
type Variants = PositionalPack<Variant>
```

Therefore:

```text
Sum::Variants -> Pack -> T6
```

Do not add:

```text
StaticIterationMode::SumVariants
```

unless a concrete repository limitation makes Pack reuse impossible and that limitation is documented.

## Tests

```text
ordered tuple/data Entries
positional entries
labeled entries
Sum::Variants declaration order
repeated variant Symbol preserved
generic/reflection projection that reduces concretely
```

## Gate G7

No Product/Sum-specific evaluator iteration authority exists.

---

# 15. T8 — Linear unordered structural traversal

## Objective

Implement deterministic finite unordered traversal without leaking P5's suffix-materialization cost into an O(n²) loop.

## Current risk

`StructuralReflectionProjector::UnorderedShapeView` currently forms a rest shape by copying the remaining fixed fields.

Repeated:

```text
View -> rest.View -> rest.View -> ...
```

can perform:

```text
n + (n-1) + ... + 1
```

field-copy work.

That is not acceptable as the internal P6 loop cursor.

## Required semantics

Observable order must equal the canonical fixed-field order already used by P5.

Open tail semantics must remain explicit.

## Preferred implementation

When the static-for source is a reduced `UnorderedShape` value:

```text
1. inspect the canonical Data value once;
2. obtain its fixed Record field collection once;
3. iterate fields by index/cursor;
4. construct only the current exposed ProductField/FieldInfo value needed by source semantics;
5. retain the original tail value separately;
6. never rebuild every suffix shape merely to advance the cursor.
```

If static `for` source syntax instead iterates an unordered Type directly:

```text
use StructuralClassifier::unordered_product(...) once;
iterate the fixed entries directly;
preserve tail metadata;
```

Do not call runtime Record iteration.

## Open-tail law

Anonymous/open structure is not a finite invented sequence.

Distinguish:

```text
operation explicitly defined as fixed-field iteration
    -> fixed entries may be traversed, tail remains metadata

operation meaning entire structural sequence
    -> open tail is not finite; return unresolved/diagnose as specified
```

Do not silently drop the tail unless the source operation's normative semantics are fixed-field traversal.

## Tests

```text
0 fields
1 field
many fields
Symbol key
String key
Int key
Bool key
canonical order
closed tail
typed/open tail
unconstrained tail
generic unresolved tail
large 1k-field traversal complexity probe
```

## Gate G8

Unordered traversal is O(n) in fixed fields and has no runtime iterator dependency.

---

# 16. T9 — Proof predicate operation identity and checker publication

## Objective

Introduce exact semantic authorization for:

```text
Equivalent
Subtype
Conforms
```

without raw source-name dispatch.

## Files likely involved

```text
phalcom-semantic/src/static_intrinsic.rs
phalcom-semantic/src/checker/static_body.rs
phalcom-semantic/src/static_definition.rs
phalcom-semantic/src/static_constant.rs
phalcom-semantic/src/generic_evidence.rs
phalcom-semantic/src/types/relation.rs
```

Exact files depend on the T0 carrier audit.

## Required design

One compiler-owned checked product must identify:

```text
expression/application identity
proof operation kind
canonical operands
required evidence/proof environment
```

Recommended if the source resolves through ordinary expression call identity:

```rust
pub enum StaticIntrinsicOperation {
    NumberAdd,
    Equivalent,
    Subtype,
    Conforms,
}
```

Use this only if the checker actually produces canonical callable identities for those forms.

If proof syntax resolves as static/generic semantic application rather than a callable, add a dedicated checked proof-operation index instead of pretending it is a runtime callable.

## Forbidden

```text
spelling match
selector-only match
user-shadowable declaration
runtime native call
runtime Typing reflection API
```

## Gate G9A

Each proof expression has one exact checked semantic operation identity before evaluator work begins.

---

# 17. T10 — Evaluate `Equivalent`

## Objective

Map canonical generic equivalence proof outcomes to public Bool / evaluator terminal states.

## Existing authority

```rust
GenericEquivalenceProof {
    Proven,
    NotProven,
    Ambiguous,
    Blocked,
    Dynamic,
    Recursive,
    Cancelled,
    BudgetExceeded,
    InternalFailure,
}
```

## Mapping

```text
Proven
    -> Bool(true)

NotProven
    -> Bool(false)

Ambiguous
    -> Symbolic/unresolved in an open static context;
       forcing diagnostic in a closed concrete context if it cannot resolve

Blocked
    -> Symbolic/unresolved or forcing diagnostic

Dynamic
    -> Symbolic only where static context permits

Recursive
    -> P6 unresolved/recursive terminal as appropriate;
       do not fabricate false

Cancelled
    -> StaticEvalOutcome::Cancelled

BudgetExceeded
    -> StaticEvalOutcome::BudgetExceeded

InternalFailure
    -> fail closed / compiler failure path
```

Do not treat `Ambiguous` as semantic refutation.

## Tests

```text
identical canonical terms -> true
known non-equivalent terms -> false
branch proof establishes equality -> true
blocked normalization -> not false
budget -> BudgetExceeded
cancellation -> Cancelled
```

---

# 18. T11 — Evaluate `Subtype`

## Objective

Use the canonical relation engine.

## Existing outcome family

Use `RelationOutcome<()>`.

Required mapping:

```text
Proven
    -> true

Refuted
    -> false

Blocked
    -> unresolved/forcing diagnostic

DynamicBoundary
    -> unresolved where static context permits

Cancelled
    -> Cancelled

BudgetExceeded
    -> BudgetExceeded

InternalFailure
    -> fail closed
```

Do not collapse `DynamicBoundary` to false.

Ensure relation work uses the existing parent `QueryBudget`/cancellation authority.

## Tests

```text
Int <: Number -> true
known non-subtype -> false
generic unresolved relation -> not false
GADT branch fact -> correct result
budget
cancellation
```

---

# 19. T12 — Evaluate `Conforms`

## Objective

Route all conformance—including Product/Sum capability proofs—through canonical generic evidence.

## Required call path

```text
GenericConformanceKey
    ↓
ConformanceProofContext
    ↓
prove_conformance_with_context(...)
```

## Mapping

```text
Proven(Exact)
Proven(Assumed)
Proven(Derived)
Proven(CompilerDerived)
    -> true

Missing
    -> false only when the proof engine means definitive semantic absence

Ambiguous
Incomplete
CoherenceConflict
Unknown
Blocked
Dynamic
    -> never false merely because proof is unavailable;
       preserve unresolved/diagnostic semantics

Cancelled
    -> Cancelled

BudgetExceeded
    -> BudgetExceeded

InternalFailure
    -> fail closed
```

For ambiguous/coherence outcomes, use the language's existing conformance error policy; do not silently residualize a semantically invalid coherence state.

## Required structural tests

```text
Conforms<Tuple<...>, Product>
Conforms<Record<...>, Product>
Conforms<ordered data, OrderedProduct>
Conforms<record data, UnorderedProduct>
Conforms<enum, Sum>
```

Each must prove through:

```text
GenericConformanceEvidence::CompilerDerived
```

not a P6-specific branch.

Also test:

```text
source exact evidence
assumed generic evidence
derived conditional evidence
definitive missing
unknown
blocked
dynamic
```

## Gate G12

`Conforms<T, Product>` and ordinary `Conforms<T, UserTrait>` use the same proof entry point.

---

# 20. T13 — P6-level unresolved propagation and P7 boundary

## Objective

Ensure open/generic valid static control does not become false, invalid, or eager residual IR.

## P6 law

For unresolved but semantically valid:

```text
member projection
match scrutinee
structural iteration source
proof predicate
```

P6 may terminate with the current coarse:

```rust
StaticEvalOutcome::Symbolic
```

provided the cause is retained long enough to produce correct closed-context diagnostics and terminal mapping.

## P7 boundary

Do not add:

```text
ResidualMatch
ResidualFor
ResidualProof
ResidualBranch
serialized evaluator frames
normalized symbolic DAG
```

Those are P7.

## Required test pair

Generic:

```text
evaluate open definition
    -> Symbolic
```

Concrete re-evaluation:

```text
same checked definition + concrete arguments
    -> Reduced
```

Do not require stored residual specialization in P6.

## Gate G13

Unknown proof/control is preserved without implementing P7.

---

# 21. T14 — Minimum structural Pack/Product pattern forms

## Dependency

This task is conditional on the T0 grammar audit.

## Objective

Add only the structural pattern forms required by current normative C4 semantics.

## Rules

1. extend the ordinary `Pattern` AST;
2. extend the ordinary parser;
3. extend the ordinary pattern resolver;
4. extend coverage/usefulness if required;
5. static evaluator consumes the resulting ordinary `PatternResolution`;
6. runtime matcher/lowering must consume the same semantic form where the pattern is also legal at runtime.

Do not add static-only syntax or resolution.

## Candidate required capabilities

Depending on current normative grammar:

```text
Pack fixed lanes
positional rest capture
labeled rest capture
complete Pack capture
product tail/capture
```

Do not blindly implement historical spellings such as:

```telos
Pack(_head, *rest)
Pack(debug: T, **rest)
```

unless T0 confirms them as the current accepted source form.

## Gate G14

One pattern language remains true.

---

# 22. T15 — P4 generalized Record-pattern convergence gate

## Objective

Integrate P6 with the **owning** generalized Record-pattern implementation.

## Do not implement privately in P6

If current canonical pattern resolution still contains:

```rust
ResolvedRecordFieldPattern {
    label: Box<str>,
    ...
}
```

and therefore cannot represent canonical heterogeneous `FieldKey`, P6 must not patch a separate static-only alternative.

## Acceptance once P4 product exists

Test:

```text
Symbol key
String key with same spelling as Symbol
Int key
Bool key
rest capture
duplicate canonical key
static match parity with ordinary match semantics
```

## Gate G15

P6 can close only after its declared Record-pattern acceptance is either:

```text
completed through the P4-owned canonical product
```

or explicitly removed from P6 acceptance by an architect/lifecycle ruling.

---

# 23. Error and diagnostic requirements

Prefer existing checker diagnostics when the semantic error is already known during checking.

Evaluator errors are for fail-closed execution mismatches and terminal static-evaluation failures.

Recommended additions only where needed:

```rust
StaticEvalError::MissingMatchResolution(ExpressionId)
StaticEvalError::MissingPatternSiteResolution(...)
StaticEvalError::MissingStaticIterationResolution(ExpressionId)
StaticEvalError::StaticPatternProjectionFailed(...)
StaticEvalError::StaticIterationNotFinite
StaticEvalError::UnmatchedExhaustiveStaticMatch
```

Do not duplicate source-level match exhaustiveness diagnostics if the checker already owns them.

Potential user-facing static diagnostics:

```text
static.iteration_not_structural
static.iteration_open_tail_not_finite
static.proof_unknown
static.proof_blocked
static.proof_dynamic
```

Use the repository's actual `DiagnosticCode` naming convention; do not add stringly diagnostic IDs outside it.

---

# 24. Incremental and fingerprint plan

## 24.1 Checked product dependencies

A static match depends on:

```text
checked static source
MatchResolution
PatternResolution
referenced enum/data declaration products
branch proof environment
static scrutinee dependencies
P5 member projection/evidence dependencies where used
```

A static structural loop depends on:

```text
checked static source
StaticIterationLaneResolution
source structural type/value dependencies
Pack/type-pack structure
Product/Sum classification where relevant
reflection result where explicitly evaluated
generic evidence/proof environment
```

Proof predicates depend on:

```text
canonical relation operands
generic evidence environment
trait surface
conformance index/rules
structural classifier inputs when compiler-derived evidence participates
```

## 24.2 Must invalidate

```text
pattern changes
match arm changes
scrutinee static value changes
variant declaration/case changes
data component changes
Pack contents/shape changes
Record FieldKey/presence/tail changes
Sum variant order changes
static-for source shape changes
generic constraint/evidence changes
structural classification changes
```

## 24.3 Should not invalidate

```text
unrelated runtime method body
unrelated declaration
unrelated impl body
runtime layout-only change with identical semantic structural products
unrelated static definition
```

## Tests

Add focused cases under:

```text
phalcom-semantic/tests/semantic/incremental/
```

Prefer extending existing fingerprint/query suites unless a P6-specific module materially improves organization.

---

# 25. Performance and resource gates

## 25.1 Required asymptotics

| Operation | Target |
|---|---:|
| Simple static variant match | O(pattern depth) after checked resolution |
| Scan A match arms | O(A × tested-pattern cost) worst case |
| Pack traversal | O(n) |
| `Sum::Variants` traversal | O(v) |
| Ordered reflected Pack traversal | O(n) |
| Unordered fixed-field traversal | O(n) |
| Pack rest/slice creation | O(1) using existing backing |
| Proof operation | existing relation/proof-query complexity; no new source scan |
| Loop cursor state | O(1) aside from evaluator locals/control |

## 25.2 Explicit regressions to prevent

### Repeated `UnorderedShape::View`

Do not implement static-for as repeated suffix materialization.

### Pack suffix rebuilding

Do not allocate a new Pack for each iteration.

### Re-resolving variants

Do not run source-name candidate resolution per match arm execution.

### Re-materializing reflection

Evaluate one reflection scrutinee once.

### Proof engine reinitialization

Reuse normal query/proof context and memoization. Do not rebuild the conformance world for every iteration.

### TypeStore cloning

Never clone the whole canonical TypeStore to install branch proof facts or heterogeneous loop refinements.

## 25.3 Budget

Use the existing:

```text
QueryBudget
CancellationToken
```

Charge at least:

```text
loop advance
pattern traversal work
branch/body evaluation
proof queries through existing authority
reflection/projection work through existing authority
```

No wall-clock timeout.

No magic separate maximum loop count.

---

# 26. Test implementation map

## 26.1 Static definitions / evaluator

Primary:

```text
phalcom-semantic/tests/semantic/integration/static_definition_bodies.rs
```

Add or split a sibling only if this file becomes unwieldy.

Required groups:

```text
static_match_basic
static_match_selected_arm_only
static_match_repeated_variant_symbol_identity
static_match_gadt_branch_proof
static_match_reflection_data
static_match_generic_stuck

static_for_pack_empty
static_for_pack_heterogeneous
static_for_break_continue_return
static_for_nested
static_for_budget
static_for_cancellation

static_for_variants_pack
static_for_ordered_reflection
static_for_unordered_fields
static_for_open_tail

static_proof_equivalent
static_proof_subtype
static_proof_conforms
static_proof_unknown_not_false
```

## 26.2 Match parity

Existing matching suites:

```text
phalcom-semantic/tests/semantic/adts/matching/
```

Add assertions only where canonical product behavior changes.

Do not duplicate the entire match suite under static integration.

## 26.3 Structural evidence/reflection regressions

Retain:

```text
phalcom-semantic/tests/semantic/capabilities/structural
```

P6 adds integration tests proving reuse, not a second P5 test matrix.

## 26.4 Incremental

```text
phalcom-semantic/tests/semantic/incremental/fingerprints.rs
```

or current closest query-cache suite.

## 26.5 AST/parser

Only if T14 changes grammar:

```text
phalcom-ast/tests/
```

Use existing parser/pattern suites.

## 26.6 Core/runtime regression

P6 should not require a VM static-control feature, but rerun P10 runtime while/control tests and ordinary match/for compiler tests to prove the static-phase split did not change runtime behavior.

---

# 27. Verification commands

Use exact repository selectors after confirming with `cargo test ... -- --list`.

Minimum focused gates:

```bash
RUSTFLAGS='' cargo check -p phalcom-semantic

cargo test -p phalcom-semantic --test semantic \
  integration::static_definition_bodies -- --nocapture

cargo test -p phalcom-semantic --test semantic \
  capabilities::structural -- --nocapture

cargo test -p phalcom-semantic --test semantic \
  foundations::static_domains -- --nocapture

cargo test -p phalcom-semantic --test semantic \
  foundations::type_members -- --nocapture
```

Run matching suites using their current selectors:

```bash
cargo test -p phalcom-semantic --test semantic \
  adts::matching -- --nocapture
```

If that module-prefix selector does not match the harness, list tests and run the exact existing matching submodules rather than weakening verification.

Run incremental focused tests:

```bash
cargo test -p phalcom-semantic --test semantic \
  incremental -- --nocapture
```

Again, prefer exact narrower selectors after listing.

If T14 changes syntax:

```bash
cargo test -p phalcom-ast --tests
```

Affected-crate checks:

```bash
RUSTFLAGS='' cargo check -p phalcom-ast
RUSTFLAGS='' cargo check -p phalcom-semantic
RUSTFLAGS='' cargo check -p phalcom-core
```

Final hygiene:

```bash
cargo fmt --check
git diff --check
git status --short
```

Do not claim workspace-wide green if inherited baseline failures remain. Classify every failure against baseline and record the evidence.

---

# 28. Patch sequencing and commit discipline

Recommended commit slices:

```text
1. docs: activate/rebaseline LANG007.C4.P6

2. feat(semantic): publish checked match/pattern products for static evaluation

3. feat(semantic): add static structural iteration resolution

4. feat(static-eval): add resolved static-value pattern matcher

5. feat(static-eval): execute static match and branch proof environments

6. feat(static-eval): execute Pack structural for

7. feat(static-eval): integrate ordered/Sum/unordered structural traversal

8. feat(static-eval): add Equivalent/Subtype/Conforms proof operations

9. feat(patterns): add ratified structural Pack/Product pattern forms
   # only if T14 is unblocked

10. test(LANG007): add incremental/resource/performance regressions

11. docs(LANG007): close P6 lifecycle records
```

Do not create large mixed commits combining:

```text
pattern grammar
proof predicates
iteration engine
unrelated runtime compiler changes
```

unless atomicity truly requires it.

---

# 29. Detailed STOP / CONSULT conditions

Stop implementation and request an architectural ruling if any occurs.

## S1 — P4 authority conflict

P6 would need a private heterogeneous Record-pattern representation because canonical pattern semantics still cannot represent `FieldKey`.

## S2 — second matcher pressure

Implementation appears to require:

```text
StaticPattern
StaticVariantResolver
StaticGadtEnvironment
```

rather than consuming existing match products.

## S3 — structural-specific match engine

P5 reflection Data/Variant values cannot be matched through ordinary resolved patterns without a Product/Sum-specific engine.

## S4 — runtime iteration dependency

Static structural `for` requires:

```text
Iterable
Iterator
iterator
next
runtime method dispatch
```

## S5 — uncertainty collapse

Any implementation path maps:

```text
Stuck
Unknown
Blocked
Dynamic
```

to:

```text
false
empty traversal
NoMatch
```

without semantic proof.

## S6 — P7 leakage

Correct P6 behavior appears to require normalized residual Match/For/Proof IR.

## S7 — P8 leakage

Correct P6 behavior appears to require durable residual serialization or exported evaluator state.

## S8 — fake structural evidence

Structural `Conforms` requires synthetic impl evidence or bypasses `GenericConformanceProof`.

## S9 — source-name proof intrinsic

Proof operations can only be recognized by `"Equivalent"`, `"Subtype"`, or `"Conforms"` spelling.

## S10 — unratified grammar

Structural Pack/Product patterns require choosing new syntax not established by current normative docs.

## S11 — duplicated coverage/GADT logic

Static matching would need to reimplement usefulness, exhaustiveness, exact-case selection, or GADT equality derivation.

## S12 — global type mutation

Heterogeneous loop refinement requires global TypeStore mutation or whole-store cloning.

## S13 — O(n²) unordered traversal requires public P5 breakage

An O(n) static traversal cannot be implemented without breaking the public P5 reflection contract.

## S14 — proof evidence collapse

Implementation loses distinctions among:

```text
Exact
Assumed
Derived
CompilerDerived
Missing
Unknown
Blocked
Dynamic
```

---

# 30. Acceptance gates

## G0 — Baseline/lifecycle

- [ ] exact starting revision recorded;
- [ ] checkpoint active-plan metadata corrected;
- [ ] P4 Record-pattern dependency classified;
- [ ] structural-pattern grammar status classified;
- [ ] proof-operation identity carrier classified.

## G1 — Checked product publication

- [ ] `CheckedStaticBody` publishes match/pattern products;
- [ ] `CheckedStaticExpression` publishes expression match products;
- [ ] evaluator body exposes them;
- [ ] fingerprints/incremental tests pass.

## G2 — Static iteration checker

- [ ] static-phase `for` no longer requires runtime Iterator semantics;
- [ ] runtime-phase `for` is unchanged;
- [ ] static iteration product is canonical/fingerprinted;
- [ ] ordinary pattern resolver remains the binder authority.

## G3 — General matcher

- [ ] matcher consumes `PatternResolution`;
- [ ] exact `VariantId` identity used;
- [ ] reflection Data/Variant values need no special matcher;
- [ ] binding IDs are canonical;
- [ ] budget/cancellation behavior is bounded.

## G4 — Static match

- [ ] block `type` match reduces;
- [ ] `const x = match ...` reduces;
- [ ] selected arm only is evaluated;
- [ ] unresolved valid scrutinee becomes Symbolic;
- [ ] no source re-resolution in evaluator.

## G5 — Branch proofs

- [ ] GADT/equality branch facts visible in selected arm;
- [ ] proof scope does not leak;
- [ ] no TypeStore clone/global mutation.

## G6 — Pack static for

- [ ] finite Pack traversal works;
- [ ] break/continue/return work;
- [ ] nested loops work;
- [ ] O(n), no suffix rebuild;
- [ ] budget/cancellation pass.

## G7 — Ordered/Sum traversal

- [ ] ordered reflected Packs reuse Pack iterator;
- [ ] `Sum::Variants` reuses Pack iterator;
- [ ] repeated variant Symbol order/identity preserved;
- [ ] no Product/Sum-specific loop engine.

## G8 — Unordered traversal

- [ ] canonical deterministic order;
- [ ] heterogeneous `FieldKey` values preserved where P4 supports them;
- [ ] open tail law respected;
- [ ] O(n) fixed-field traversal;
- [ ] no repeated suffix-shape copying.

## G9 — Proof operation identity

- [ ] proof predicates authorized by canonical semantic identity;
- [ ] no raw spelling recognition.

## G10 — Equivalent

- [ ] proven -> true;
- [ ] refuted -> false;
- [ ] ambiguous/blocked/dynamic not false;
- [ ] budget/cancellation propagated.

## G11 — Subtype

- [ ] proven -> true;
- [ ] refuted -> false;
- [ ] blocked/dynamic not false;
- [ ] branch proof facts honored.

## G12 — Conforms

- [ ] exact/assumed/derived/compiler-derived -> true;
- [ ] definitive missing -> false;
- [ ] unknown/blocked/dynamic not false;
- [ ] Product/Sum uses normal compiler-derived evidence query.

## G13 — P7 boundary

- [ ] open generic valid control returns Symbolic;
- [ ] concrete re-evaluation reduces;
- [ ] no normalized residual IR added.

## G14 — Shared structural patterns

- [ ] only if ratified/unblocked;
- [ ] one AST/resolver;
- [ ] static and runtime semantics share the resolved product.

## G15 — P4 Record integration

- [ ] canonical heterogeneous Record pattern product consumed;
- [ ] no P6-private FieldKey matcher.

## G16 — Incremental/resource

- [ ] cold/warm parity;
- [ ] precise invalidation;
- [ ] unrelated edits do not invalidate;
- [ ] deterministic budget behavior;
- [ ] cancellation responsive.

## G17 — Regression

- [ ] P2 static-definition gates;
- [ ] P3 Pack gates;
- [ ] P4 FieldKey/row gates;
- [ ] P5 structural gates;
- [ ] P10 while/control gates;
- [ ] LANG005.C6 evidence gates;
- [ ] TYPE001 match/GADT gates;
- [ ] affected crates check.

---

# 31. Exit criteria

P6 may be marked:

```text
COMPLETE / IMPLEMENTED / FOCUSED_TESTED
```

only when all of the following are true.

1. Static `match` executes through the semantic evaluator.
2. `const` expression match and block static-definition match both consume canonical checked match products.
3. The evaluator does not resolve variant names or GADT facts from source.
4. Selected-arm-only execution is proven.
5. Branch proof environments are preserved.
6. Static `for` uses compiler-known structural iteration rather than runtime Iterator dispatch.
7. Pack traversal is finite, bounded, O(n), and control-transfer correct.
8. `Sum::Variants` reuses Pack iteration.
9. Ordered structural reflection reuses ordinary Pack iteration where represented as Pack.
10. Unordered structural traversal is deterministic and O(n), not repeated `View` suffix reconstruction.
11. Open tails are never invented as finite elements.
12. Heterogeneous exact iteration refinement is preserved to the extent available from checked structural products, while body validity remains sound for all reachable iterations.
13. `Equivalent`, `Subtype`, and `Conforms` use canonical semantic relation/evidence authorities.
14. Only proven predicates yield `true`; only semantic refutation yields `false`.
15. `Stuck`, `Unknown`, `Blocked`, and `Dynamic` are not collapsed to false or empty control.
16. Product/Sum conformance uses P5 compiler-derived evidence through the normal generic proof query.
17. P6 does not introduce normalized residual IR.
18. P6 does not require VM execution or arbitrary runtime/native calls.
19. Shared structural pattern extensions, if implemented, remain one ordinary pattern language.
20. P4-owned heterogeneous Record-pattern semantics are consumed rather than duplicated.
21. Focused incremental/resource/regression gates pass or every inherited baseline failure is explicitly classified.
22. Lifecycle walkthrough/handoff/checkpoint records match the actual source and verification evidence.

---

# 32. Final implementer directive

Implement P6 by extending the existing semantic product/evaluator pipeline in the smallest coherent vertical slices.

The preferred dependency order is:

```text
checked match publication
    ↓
general static matcher
    ↓
static match
    ↓
branch proofs

static iteration resolution
    ↓
Pack for
    ↓
ordered/Sum Pack reuse
    ↓
linear unordered traversal

canonical proof-operation identity
    ↓
Equivalent
Subtype
Conforms

then:
structural pattern grammar, if ratified
P4 Record-pattern integration
incremental/resource certification
lifecycle closure
```

Do not begin with grammar expansion, reflection-specific match logic, or a generalized residual system.

The P6 architecture should end with this property:

> **Everything P6 executes has already been semantically classified or resolved by an existing canonical authority. The evaluator decides only how to execute that checked meaning over immutable static values.**
