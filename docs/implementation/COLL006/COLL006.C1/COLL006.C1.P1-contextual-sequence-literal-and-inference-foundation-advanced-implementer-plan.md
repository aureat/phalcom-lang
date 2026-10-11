---
id: COLL006.C1.P1
category: COLL
program: COLL006
checkpoint: COLL006.C1
kind: advanced-implementer-plan
title: "Contextual Sequence Literal and Inference Foundation"
status: COMPLETE
completion: IMPLEMENTED
verification: FOCUSED_TESTED
repository: aureat/phalcom-lang
branch: main
audited_revision: 61c8734f752c8ffc252b6d4d4ba378c2bfe2b2b1
audited_at: 2026-10-08
successor_plans:
  - COLL006.C1.P2
  - COLL006.C1.P3
  - COLL006.C1.P4
---

# COLL006.C1.P1 — Contextual Sequence Literal and Inference Foundation

## Advanced-Implementer Implementation Plan

> **This is intentionally not a Luna patch-grade plan.**
>
> It is an implementation-and-reasoning contract for a high-capability implementer that can investigate, model, make bounded mechanical decisions, and diagnose difficult semantic failures. It deliberately gives the implementer more reasoning latitude than a mechanically prescriptive patch plan, while fixing the semantic architecture, ownership model, invariants, and escalation boundaries that must not drift.

---

# 0. Plan thesis

P1 is the **semantic-foundation plan** for the collection program.

It must transform the current language implementation from:

```text
bracket syntax
    == List syntax
    == List semantic synthesis
    == List compiler lowering
```

into:

```text
bracket syntax
    ↓
neutral source representation
    ↓
semantic sequence-resolution authority
    ↓
canonical checked semantic product
    ↓
compiler consumes that product
```

while **preserving the current observable bare-bracket behavior as List for the duration of P1**.

In parallel, P1 must establish `_` as a real inference request and make initializer-producing annotations capable of participating in the canonical inference solver without allowing inference-local variables to become durable type identity.

P1 therefore changes *who owns the meaning* of collection literals before P2/P3/P4 change *which concrete collection meaning is selected*.

The architectural result must be strong enough that:

- P2 can introduce `Array<T>` without changing parser ownership or inventing a second literal checker;
- P3 can introduce `Vector<T,N>` without inventing a second inference/cardinality system;
- P4 can flip runtime/static default families atomically without changing compiler authority or static semantics architecture.

---

# 1. Repository grounding

## 1.1 Audited repository state

This plan was prepared against:

```text
repository: aureat/phalcom-lang
branch:     main
revision:   61c8734f752c8ffc252b6d4d4ba378c2bfe2b2b1
commit:     fix: repair random resource and bootstrap blockers and record continuation
```

The immediately preceding P1 audit baseline was:

```text
ad1db231f5d866650e7ab169c377f28ffa602de9
```

The latest commit does **not** implement the P1 literal/inference work. It does, however:

- commit the COLL006 lifecycle records into `docs/implementation/COLL006/`;
- modify semantic workspace/session behavior for unrelated bootstrap/formal-product correctness;
- make one `CheckingContext` expression-owner stack correction;
- add unrelated random, bytes, slotmap, bootstrap, and test work.

The live P1 ownership seams remain materially as previously diagnosed and were reverified at `61c8734f...`.

## 1.2 Important recent inference change

Recent commit `568c545acd2becfe991b61f317fda626c64cc5ba` changed canonical inference behavior in a way P1 must respect:

- projection normalization rewrites constraints and requests another canonical propagation pass;
- a rewrite is **not itself proof**;
- residual obligations remain until ordinary propagation adjudicates them;
- unresolved declaration equality is distinguished from missing value-producing evidence;
- an unresolved mandatory equality must not publish a call specialization.

P1 must therefore treat:

```text
materializable inference term
```

and:

```text
fully solved/proved inference query
```

as distinct states.

Do not locally “finish” annotation inference merely because an `InferenceTerm` can be materialized if residual obligations remain.

## 1.3 Current lifecycle state

Current committed lifecycle truth:

```text
COLL006
    PROPOSED / NOT_STARTED / UNVERIFIED

COLL006.C1
    PROPOSED / NOT_STARTED / UNVERIFIED

active plan
    none

next action
    author and execute COLL006.C1.P1
```

At implementation start, this plan becomes the active plan and the checkpoint ledger must be updated before substantial source work.

---

# 2. Authorities the implementer must keep separate

The implementer must reason with four independent authorities.

```text
1. Desired language behavior
   accepted specifications + decisions + explicit user rulings

2. Current repository reality
   current main + live tests + current products/interfaces

3. Implementation architecture
   this plan + C1 checkpoint + later recorded consultations/amendments

4. Current work state
   checkpoint record + scoped diff + actual verification evidence
```

A contradiction is not resolved by casually choosing whichever source is easiest.

Examples:

- An old test saying `[...]` is intrinsically List is valid **during P1 compatibility**, but becomes migration evidence at P4.
- A current implementation name such as `ListLiteralExpr` is repository reality, not an architectural mandate.
- A planned helper name is not authoritative if the live repository offers a more canonical existing helper.
- An accepted semantic invariant cannot be weakened because a current implementation path makes it inconvenient.

---

# 3. Telos semantic philosophy for this implementation

P1 touches syntax, typing, inference, static phase, semantic products, incrementality, and compiler lowering. It must therefore follow Telos's deeper semantic model rather than merely “make new syntax compile.”

## 3.1 Syntax records what was written; it does not decide ontology

A bracket literal is source syntax.

It is not intrinsically a List.

The parser may preserve:

```text
whether a head was written
which generic/static arguments were written
where `_` was written
which elements/spreads were written
source ranges
```

The parser must not decide:

```text
runtime Array
static Vector
contextual List
element type
Vector extent
```

Those are semantic propositions.

This is why P1 should use a syntax-oriented name such as:

```text
BracketLiteral
```

and a semantic product name such as:

```text
SequenceLiteralResolution
```

The distinction is intentional.

## 3.2 Context is evidence, not semantic overwrite

Telos uses bidirectional/contextual checking, but an expected type is not permission to replace actual evidence.

Expected information may constrain a literal:

```telos
let xs: List<Number> = [1, 2.5]
```

but it must not make an invalid expression valid by fiat.

Maintain the existing epistemic distinctions:

```text
Known / established
Known / assumed
Unknown
Dynamic
Blocked
Invalid
Cancelled
BudgetExceeded
InternalFailure
```

Do not collapse them because a collection literal needs a result type.

## 3.3 `_` is epistemic uncertainty, not an ontological type

`_` means:

> “Solve this argument from the local semantic evidence available to this inference query.”

It does **not** denote a value-space classifier.

Therefore this is wrong:

```text
Array<_>  -> Array<Dynamic>
```

and this is wrong:

```text
_ -> hidden wildcard TypeId
```

The correct model is:

```text
syntax hole
    ↓
query-local inference variable
    ↓
ordinary constraints
    ↓
solve/prove
    ↓
canonical ordinary type/static term
```

If the evidence is insufficient, the correct result is **underconstrained**, not Dynamic.

## 3.4 Inference variables are temporary proof machinery

An `InferVarId` is not durable semantic identity.

It may exist in:

```text
InferenceSession
InferenceCoordinator
InferenceTerm
ExpectedType::Inference
temporary annotation elaboration
```

It may not exist in:

```text
TypeStore canonical identity
BindingContract
CallableAnalysis published type
metadata
reflection
compiler lowering
runtime type environments
```

The publication boundary is a proof boundary.

## 3.5 Static/runtime phase is an evaluation regime, not a type

Telos separates:

```text
what type a value has
```

from:

```text
whether the expression is evaluated statically or at runtime
```

Do not encode “static” as:

- a fake expected type;
- a collection classifier;
- a const-only Array type;
- a literal syntax property.

P1 must introduce an explicit checker phase fact so nested static expressions inherit the correct evaluation regime.

## 3.6 Semantic authority precedes compiler authority

The compiler is not a second typechecker.

Once P1 is complete, the compiler may see:

```text
BracketLiteral AST
+
SequenceLiteralLoweringSpec
```

It must not ask:

```text
“Is this runtime?”
“What does the expected type say?”
“How many generic parameters were in the head?”
“Should I default this to List?”
```

Those are semantic questions already answered upstream.

## 3.7 Runtime execution state is not language identity

P1 does not add new runtime collection objects, but the principle matters for successors:

```text
TypeId / semantic classifier
    != heap representation
    != bytecode opcode
    != VM-local builder state
```

The P1 semantic product should describe the canonical meaning needed by lowering without encoding VM implementation details.

## 3.8 Fail closed rather than fabricate evidence

When required semantic information is absent:

```text
diagnose
block
or raise an internal compiler error at an impossible post-check boundary
```

Do not:

```text
fall back to List
fall back to Dynamic
guess family from AST
guess parameter kind
silently discard a hole
```

## 3.9 One authority per proposition

For every new fact, ask:

```text
What proposition is this?
Who owns it?
What is its lifetime?
What identifies it?
Who consumes it?
How is it invalidated?
```

If two layers answer the same question independently, the architecture is wrong.

## 3.10 Simplicity at the language surface may require precision internally

The desired source experience is deliberately simple:

```telos
let xs: Array<_> = [...]
```

That simplicity is purchased by precise internal machinery:

```text
kind-aware hole allocation
expected-type propagation
constraint solving
residual obligation handling
canonical publication
semantic lowering projection
incremental fingerprinting
```

Do not push this complexity back onto the user through awkward syntax merely to avoid implementing the correct semantic boundary.

---

# 4. P1 acceptance objective

P1 succeeds when all of the following are true.

## 4.1 Syntax ownership

1. `[...]` is represented by a neutral bracket-literal AST form rather than `Expr::ListLiteral`.
2. The neutral AST can preserve an optional explicit head.
3. `<T>[...]` and `<T,N>[...]` parse as bracket literals with retained head arguments.
4. `_` has a dedicated inference-hole syntax node in generic/static argument position.
5. Existing list-pattern syntax remains a separate pattern grammar and is not renamed into sequence semantics unless independently required.

## 4.2 Inference foundation

6. An inference hole in an authorized initializer-driven annotation becomes a query-local inference variable with the target binder's canonical kind/domain.
7. Strict/durable type formation rejects holes rather than publishing them.
8. Initializer analysis can consume an `ExpectedType::Inference` derived from a partial annotation.
9. After analysis, the inference query is propagated/finished through canonical inference machinery.
10. A binding contract is published only after the annotation term materializes to a fully solved canonical type.
11. Underconstrained/conflicting holes diagnose without Dynamic fallback.

## 4.3 Evaluation phase

12. The ordinary checker has an explicit scoped `Runtime | Static` phase fact.
13. Static-body/static-constant entry points establish Static phase.
14. Child/nested expression analysis inherits phase automatically.
15. P1 does **not** use phase to change bare-bracket default behavior.

## 4.4 Semantic literal authority

16. Semantic analysis publishes a checked sequence-literal resolution keyed by `ExpressionId`.
17. P1 compatibility mode resolves ordinary bare brackets to canonical List semantics.
18. The semantic product contains only canonical/durable terms.
19. Semantic resolution participates in semantic product fingerprinting where it can affect executable lowering.
20. Array/Vector placeholders are not fabricated in order to make future syntax appear implemented.

## 4.5 Compiler boundary

21. Semantic resolution is projected into `ModuleLoweringSemantics`.
22. Compiler bracket-literal code generation consumes that checked product.
23. The compiler does not reconstruct family/default/phase inference.
24. Missing checked resolution at a supposedly valid executable bracket literal fails closed as an internal compiler condition.
25. Existing fixed/spread List bytecodes and runtime behavior remain unchanged.

## 4.6 Compatibility

26. P1 does **not** flip bare runtime literals to Array.
27. P1 does **not** flip static bare literals to Vector.
28. Existing List fixed-literal fast path remains.
29. Existing List spread semantics remain.
30. P1 does not amend the legacy bracket/List TDRs; P4 owns the semantic epoch cutover.

---

# 5. Critical P1 scope clarification: Array and Vector do not get fake identities

The lifecycle records describe P1 as creating the foundation needed for:

```telos
Array<_>
Vector<Int, _>
<T>[...]
<T,N>[...]
```

But P2 and P3 explicitly own the actual `Array<T>` and `Vector<T,N>` collection families.

Therefore P1 must distinguish:

```text
syntax/inference mechanism support
```

from:

```text
concrete family activation
```

## 5.1 Required P1 interpretation

P1 must:

- parse explicit bracket heads;
- preserve their generic/static argument syntax;
- define the semantic family-selection architecture;
- implement inference holes generically;
- prove partial annotation inference using existing canonical generic declarations, especially `List<_>`, plus synthetic test declarations when a static-domain binder must be exercised;
- leave clean extension points for P2/P3.

P1 must **not** introduce:

```text
temporary Array declaration
temporary Vector declaration
fake UniverseKey
placeholder runtime class
temporary Vector N model
compiler-only pseudo TypeId
```

that P2/P3 later replace.

## 5.2 Executability during P1

At P1 completion:

- bare brackets remain executable List literals;
- `List<_>` partial annotations are real executable proof of initializer-driven type-hole inference;
- synthetic semantic tests should prove holes with heterogeneous/static-domain binders;
- explicit future Array/Vector bracket heads may be syntax-valid but semantically unavailable until their owning family lands.

If a clean existing canonical declaration for Array/Vector has appeared on implementation HEAD before P1 execution, verify it and activate it only if doing so does not consume P2/P3 ownership.

If the live tree creates a genuine ambiguity here, record it and consult rather than inventing a transitional semantic identity.

---

# 6. Live architecture map at plan authoring

## 6.1 AST

Current:

```rust
Expr::ListLiteral(Box<ListLiteralExpr>)

struct ListLiteralExpr {
    elements: Vec<ListLiteralElement>,
    range: SourceRange,
}

enum ListLiteralElement {
    Element { ... },
    Expansion { ... },
}
```

Current generic arguments:

```rust
enum GenericArgumentSyntax {
    Single(TypeAnnotation),
    Expansion(TypePackExpansionSyntax),
    Remainder(TypePackRemainderSyntax),
    Value(Box<Expr>),
}
```

No inference-hole variant exists.

## 6.2 Parser

Current `parse_primary` includes:

```text
Token::LBracket    -> parse_list_literal()
Token::Less        -> type lambda only when is_type_lambda_ahead()
Token::Underscore  -> ordinary Expr::Var("_") in expression position
```

`parse_generic_argument` already centralizes:

- ordinary type argument syntax;
- explicit expansion;
- anonymous remainder;
- static value argument.

This parser should remain the canonical grammar for arguments inside an explicit bracket head.

## 6.3 Binding checking

Current `Statement::Let` performs:

```text
resolve full canonical annotation
    ↓
form Proper expected type
    ↓
analyze initializer
    ↓
check relation
    ↓
publish binding
```

That ordering cannot represent `List<_>` correctly because the annotation is required to be canonical before initializer evidence exists.

## 6.4 Expected types

`ExpectedType` already provides:

```rust
None
Proper { ty, origin }
Inference { context, term, origin }
```

This is a strong existing seam.

`collection_element_type` already understands an `InferenceTerm::Applied` expected type for a canonical one-argument collection origin.

P1 should extend and reuse this architecture rather than create collection-specific expected-type state.

## 6.5 Canonical inference

The current solver already has:

```text
kinded fresh variables
InferenceTerm::Var
InferenceTerm::Applied
heterogeneous static argument inference terms
constraint provenance
residual obligations
materialization
underconstrained/conflicting outcomes
```

P1 does not need another solver.

## 6.6 List semantic synthesis

Current `synthesize_list_literal`:

- hardcodes `ctx.core_ids.list`;
- projects an expected List element type;
- analyzes element expressions contextually;
- resolves positional expansion through canonical iteration evidence;
- composes element knowledge;
- returns `List<T>`;
- treats an empty literal without expected List type as `Unknown(NoTypeEvidence)`.

This function contains valuable behavior but the *family ownership* must be generalized.

## 6.7 Semantic publication

`CheckingContext` currently owns deterministic per-expression products including:

```text
for_iteration_lanes
positional_expansions
known_invalid_positional_expansions
associated_resolutions
family_applications
match_resolutions
...
```

`CallableAnalysis` publishes them.

This is the model for `sequence_literals`.

## 6.8 Static checking

Static expressions already use the ordinary checker through:

```text
analyze_static_body
analyze_static_constant_initializer
analyze_static_expression
check_static_expression
```

They enforce phase accessibility by detecting illegal runtime binding reads after ordinary checking, but the checker has no general expression `EvaluationPhase` fact.

P1 should make the phase explicit rather than teaching bracket literals to inspect const ownership ad hoc.

## 6.9 Static constant annotation path

Module static constants have a separate demand-driven path in:

```text
phalcom-semantic/src/static_constant_demand.rs
```

Today it:

1. performs an initializer discovery check;
2. forces discovered static-constant dependencies;
3. resolves the annotation to a complete canonical type;
4. checks the initializer with that expected canonical type;
5. evaluates/publishes the static constant.

This path is relevant if P1 claims inference holes for `const` annotations.

Do not change only `Statement::Let` and then claim all binding annotations support holes.

## 6.10 Compiler/lowering

Compiler currently switches directly on:

```rust
Expr::ListLiteral(...)
```

and emits:

```text
BuildList(n)
```

for fixed literals, or:

```text
BeginListLiteral
ListLiteralAppend
FinishListLiteral
ListTryExpandTuplePositionals
```

for spread-containing literals.

P1 must preserve those runtime mechanics but change the authority that selects them.

## 6.11 Cross-cutting AST consumers

Neutralizing `ListLiteral` impacts at least:

```text
phalcom-ast/src/bin/c4_migrate.rs
phalcom-lsp/src/semantic_tokens.rs
phalcom-semantic/src/advisory/analyzer.rs
phalcom-semantic/src/checker/expression.rs
phalcom-semantic/src/source_index/builder.rs
phalcom-semantic/src/source_index/occurrence.rs
phalcom-core/src/compiler/lib/expr.rs
phalcom-core/src/compiler/attributes.rs
phalcom-core/src/compiler/boundedness.rs
phalcom-core/src/compiler/lib/product_opt.rs
phalcom-core/src/modules/semantic_lowering.rs
tests constructing/matching ListLiteral nodes
```

Most of these are mechanical syntax traversal consumers and must not become new family-selection authorities.

---

# 7. Target P1 architecture

## 7.1 Neutral syntax

Recommended conceptual form:

```rust
Expr::BracketLiteral(Box<BracketLiteralExpr>)

struct BracketLiteralExpr {
    head: Option<BracketLiteralHeadSyntax>,
    elements: Vec<BracketLiteralElement>,
    range: SourceRange,
}

struct BracketLiteralHeadSyntax {
    arguments: Vec<GenericArgumentSyntax>,
    range: SourceRange,
}

enum BracketLiteralElement {
    Element { expr: Expr, range: SourceRange },
    Expansion { expr: Expr, range: SourceRange },
}
```

The exact private Rust names are flexible.

The important rule is semantic:

> The AST records an optional argument head; it does not encode `Array` or `Vector`.

Do **not** name the syntax node:

```text
ArrayLiteralHead
VectorLiteralHead
SequenceLiteralFamily
```

because those names assign semantic meaning too early.

## 7.2 Inference-hole syntax

Recommended:

```rust
GenericArgumentSyntax::InferenceHole {
    range: SourceRange,
}
```

This must be distinguished from:

```text
Pattern::Wildcard
Expr::Var("_")
```

The same `_` token has different grammar roles.

## 7.3 Annotation elaboration

Recommended checker-only conceptual result:

```rust
enum AnnotationElaboration {
    Canonical(TypeId),

    Inference {
        context: InferenceContextId,
        term: InferenceTerm,
        holes: Box<[InferenceHoleOrigin]>,
    },
}

struct InferenceHoleOrigin {
    range: SourceRange,
    parameter: TypeParameterId,
    kind: KindId,
}
```

Equivalent structures are acceptable.

This is **not** a new type system.

It is a temporary bridge between source annotation syntax and the existing inference solver.

## 7.4 Evaluation phase

Recommended:

```rust
enum EvaluationPhase {
    Runtime,
    Static,
}
```

The fact belongs to the checking context, not to the AST.

Default:

```text
ordinary callable/body analysis -> Runtime
static definition/constant analysis -> Static
```

Any nested checker context created from an existing context must preserve the phase unless it deliberately crosses a semantic phase boundary.

## 7.5 Sequence semantic product

Recommended:

```rust
enum SequenceLiteralFamily {
    List,
    Array,
    Vector,
}

enum SequenceConstructionPlan {
    Fixed {
        element_count: u32,
    },
    Expanded,
}

struct SequenceLiteralResolution {
    source_expression: ExpressionId,
    source_range: SourceRange,
    family: SequenceLiteralFamily,
    result_type: TypeId,
    element_type: TypeId,
    extent: Option<TypeId>,
    construction: SequenceConstructionPlan,
}

type SequenceLiteralResolutionIndex =
    BTreeMap<ExpressionId, SequenceLiteralResolution>;
```

P1 executable resolutions are expected to be List-compatible.

The enum may include successor families because it defines the durable vocabulary of the C1 literal architecture, but P1 must not publish an Array/Vector resolution without a canonical result type.

`extent` is reserved for P3's canonical static term.

Do not use `usize` here.

## 7.6 Lowering projection

Recommended conceptual lowering product:

```rust
struct SequenceLiteralLoweringSpec {
    family: SequenceLiteralFamily,
    construction: SequenceConstructionPlan,
}
```

Only carry data code generation actually needs.

The semantic layer owns full type identity. The compiler does not need duplicate type information unless a later runtime contract requires it.

Projection:

```text
ExpressionId resolution
    ↓
look up checked expression source range
    ↓
ModuleLoweringSemantics.sequence_literals[SourceRange]
```

This mirrors existing semantic-product projection patterns.

---

# 8. Advanced implementer reasoning protocol

The implementer is expected to reason actively, but within fixed architectural bounds.

For every substantial subproblem, use this loop.

## 8.1 OBSERVE

Establish current facts from the live tree.

Do not begin from the plan's pseudocode name.

Record:

```text
current owner
current call path
current product type
current tests
current failure state
```

## 8.2 MODEL

State the semantic proposition being implemented.

Example:

> “At a binding annotation site with an initializer, this generic argument is unknown but must be solved as a value of the destination binder's canonical kind.”

If the proposition cannot be stated precisely, do not code yet.

## 8.3 LOCATE AUTHORITY

Ask:

```text
syntax fact?          -> parser/AST
type/inference fact?  -> semantic checker
durable identity?     -> canonical semantic store/product
codegen fact?         -> semantic lowering projection
runtime mechanics?    -> compiler/VM
```

Move the implementation to the owner instead of patching the consumer.

## 8.4 STATE INVARIANTS

Before modifying a cross-layer seam, identify what must remain true.

Example for holes:

```text
fresh per written hole
kind-correct
query-local
fully solved before publication
underconstraint is not Dynamic
```

## 8.5 PREDICT

Before running a test, write the expected distinguishing result mentally or in a scratch note.

Example:

```text
If List<_> is truly checked with an inference expected type,
then [1,2,3] should bind T=Int and the final BindingContract should contain List<Int>.
If the binding still reports underconstrained, expected-type propagation did not reach the element checks.
```

## 8.6 IMPLEMENT A COHERENT SLICE

Make one architecturally meaningful change.

Do not scatter speculative guards through multiple consumers.

## 8.7 DISCRIMINATE

Run the smallest test that can falsify the current causal model.

A broad suite is not a substitute for understanding.

## 8.8 RECORD DURABLE FACTS

Update the checkpoint only when the result establishes a fact later tasks depend on.

Examples:

```text
neutral AST landed
hole cannot escape canonical publication
phase context established
sequence resolution product published
compiler now consumes product
```

Do not use the checkpoint as a minute-by-minute log.

---

# 9. Problem-solving state model

## GREEN — autonomous

Use when:

- architecture matches this plan;
- repository drift is mechanical;
- failures are local/obvious;
- discriminating tests behave according to the causal model.

Continue.

## AMBER — diagnose before editing

Use when:

- behavior is semantically wrong;
- two local hypotheses are plausible;
- an inference outcome differs from expectation;
- an incremental fingerprint changes unexpectedly;
- static and runtime checking diverge unexpectedly.

Before editing, record:

```text
Observed:
Hypothesis:
Evidence:
Prediction:
Discriminating test:
```

Make one serious correction for the same semantic cause.

## RED — stop implementation on the issue

Use when a fixed ownership/identity/semantic boundary appears wrong.

Create an incident packet rather than improvising architecture.

---

# 10. P1 durable invariants

Use these IDs in tests, walkthrough, checkpoint evidence, and incidents.

## Syntax

**P1-INV-SYN-01 — Bracket neutrality**  
`[...]` AST identity does not name List/Array/Vector.

**P1-INV-SYN-02 — Head preservation**  
The parser retains the complete written head argument syntax and source range.

**P1-INV-SYN-03 — No parser family authority**  
Parser argument count does not itself select semantic family.

**P1-INV-SYN-04 — Hole is first-class syntax**  
`_` in generic argument position is not encoded as Dynamic, identifier, missing argument, or static expression.

**P1-INV-SYN-05 — Grammar non-regression**  
Sequence-head lookahead does not steal type lambdas, comparisons, generic applications, or shift-token `>>` handling.

## Inference

**P1-INV-INF-01 — Kinded holes**  
Every hole variable uses the destination binder's canonical `KindId`/static domain.

**P1-INV-INF-02 — Freshness**  
Two written holes are distinct variables unless ordinary constraints later unify them.

**P1-INV-INF-03 — Solver reuse**  
All hole constraints use the canonical inference coordinator/session.

**P1-INV-INF-04 — Residual proof**  
A rewritten/materializable inference relation is not considered solved while mandatory residual obligations remain.

**P1-INV-INF-05 — No durable InferVar**  
No inference-local ID enters canonical/published products.

**P1-INV-INF-06 — Underconstraint honesty**  
Insufficient evidence produces underconstraint/diagnostic, never Dynamic fallback.

**P1-INV-INF-07 — Canonical publication**  
A successful partial annotation becomes an ordinary canonical `TypeId` before it becomes a binding contract.

## Phase

**P1-INV-PHASE-01 — Explicit phase**  
Static/runtime evaluation regime is represented explicitly in the checker.

**P1-INV-PHASE-02 — Phase orthogonality**  
Phase is separate from expected type, binding mutability, and classifier identity.

**P1-INV-PHASE-03 — Nested propagation**  
Nested expressions inherit the active phase unless a real semantic boundary changes it.

**P1-INV-PHASE-04 — No P1 default flip**  
Phase has no effect on bare-bracket family default until P4.

## Sequence semantics

**P1-INV-SEQ-01 — Semantic family authority**  
Semantic analysis is the only authority that publishes an executable collection family selection.

**P1-INV-SEQ-02 — Canonical result**  
Published sequence resolution contains a canonical result type.

**P1-INV-SEQ-03 — Compatibility default**  
P1 bare executable bracket literals resolve to List.

**P1-INV-SEQ-04 — No temporary future types**  
No fake Array/Vector identity is created in P1.

**P1-INV-SEQ-05 — Expansion reuse**  
List spread continues to use canonical positional-expansion iteration evidence rather than a new iterable resolver.

## Lowering

**P1-INV-LOWER-01 — Checked product consumption**  
Compiler lowering consumes sequence-literal semantic resolution.

**P1-INV-LOWER-02 — No semantic reconstruction**  
Compiler does not inspect phase/expected type/head arity to choose collection family.

**P1-INV-LOWER-03 — Fail closed**  
Missing semantic resolution at an executable checked bracket literal is an internal failure, not a List fallback.

**P1-INV-LOWER-04 — Runtime compatibility**  
P1 emits the same List bytecode/runtime behavior for accepted current List cases.

## Incrementality

**P1-INV-INCR-01 — Deterministic product**  
Sequence resolution is deterministic for the same semantic inputs.

**P1-INV-INCR-02 — Meaningful fingerprint**  
A semantic family/type/construction change affects the body product fingerprint; source-range-only movement should not alter semantic meaning unless existing fingerprint policy requires it.

**P1-INV-INCR-03 — Cold parity**  
Incremental and cold analysis agree on final sequence resolution and binding types.

---

# 11. Non-goals

P1 must not expand into:

- `Array<T>` heap/runtime implementation;
- `Vector<T,N>` heap/runtime implementation;
- exact arbitrary-precision Vector N representation work;
- heterogeneous Universe metadata work for Vector;
- static homogeneous sequence value storage;
- const Array runtime freezing;
- new Array/Vector bytecodes;
- Slice/Buffer;
- Bag/OrderedSet/IndexedDict/Bijection/RangeMap;
- a new generic solver;
- a new static evaluator;
- collection-specific Iterable;
- a general declaration-signature `_` feature;
- whole-language wildcard/existential design;
- reworking List storage;
- changing list patterns;
- P4 migration from bare List defaults;
- amendment of TDR-0025/TDR-0028;
- unrelated semantic baseline repair.

If one of these becomes necessary to satisfy a P1 invariant, reassess scope before proceeding.

---

# 12. FIXED / MECHANICALLY FLEXIBLE / VERIFY-FIRST

## 12.1 FIXED

Do not change without recorded architectural consultation:

- neutral parser/AST ownership;
- semantic checker as family-selection authority;
- compiler as consumer of semantic product;
- `_` as solver-local inference;
- no InferVar durable publication;
- phase as independent checker context;
- P1 compatibility bare default = List;
- no temporary Array/Vector identity;
- no phase-based overload tie-break;
- no new inference solver;
- no runtime collection work in P1;
- fail-closed compiler boundary.

## 12.2 MECHANICALLY FLEXIBLE

The implementer may adapt:

- private helper names;
- exact Rust enum/struct names;
- local file decomposition;
- whether a source range lives in the resolution or a companion map;
- how `CheckingContext` phase guards are implemented;
- exact diagnostic wording/code if it preserves semantic specificity;
- exact test module placement under current repository conventions;
- whether construction plan uses `Fixed/Expanded` or an equivalent compact representation;
- whether lowering spec shares an enum type with semantic code or maps it into a core-owned equivalent;
- mechanical updates required by AST exhaustive matches.

## 12.3 VERIFY-FIRST

Check live HEAD before relying on:

- exact inference-context creation/finalization APIs;
- all `CheckingContext` constructors/clones/probes;
- static-constant annotation resolution call graph;
- compiler entry points that may compile without semantic lowering;
- semantic fingerprint function names;
- source index/semantic-token AST walkers;
- test target/filter names;
- any newly landed Array/Vector work after this plan's audit SHA;
- changes to C4 static checking made concurrently.

If only mechanics changed, adapt.

If ownership or semantic architecture changed, enter RED.

---

# 13. Ordered implementation program

The tasks below are coherent implementation units. They are not microstep checklists.

---

## T0 — Establish takeover state and activate P1

### Reasoning objective

Prove that the plan still matches the repository before making architectural edits.

### Required observations

Inspect:

```text
git status --short
current HEAD
COLL006 PROGRAM/STATUS/CHECKPOINT
this P1 plan
recent commits touching AST/checker/inference/lowering
current List literal tests
current semantic test READMEs
```

Re-run focused searches for:

```text
ListLiteral
ListLiteralElement
parse_list_literal
synthesize_list_literal
GenericArgumentSyntax
collection_element_type
positional_expansions
BuildList
BeginListLiteral
```

### Required state update

Mark `COLL006.C1.P1`:

```text
IN_PROGRESS / NOT_STARTED or PARTIAL / UNVERIFIED
```

as repository convention requires.

Record starting revision.

### Baseline tests

Run only enough to establish a useful baseline:

- parser tests directly covering generic arguments/list literal syntax;
- semantic focused tests around List contextual inference and type annotations;
- core fixed/spread List literal focused tests.

Do not run workspace tests.

### Gate G0

Proceed only if:

- no unrecorded concurrent Array/Vector implementation has taken ownership of P1 seams;
- current List literal pipeline still matches the plan materially;
- current inference solver still provides query-local variables and `ExpectedType::Inference`.

---

## T1 — Neutralize bracket syntax across the AST and structural consumers

### Reasoning objective

Remove the ontological claim “bracket syntax is List” while changing no behavior.

### Implementation shape

Conceptually rename:

```text
Expr::ListLiteral        -> Expr::BracketLiteral
ListLiteralExpr          -> BracketLiteralExpr
ListLiteralElement       -> BracketLiteralElement
parse_list_literal       -> parse_bracket_literal
```

At this task, `head` may be introduced as `None` immediately or added in T3.

### Consumer classification

Before editing each exhaustive match, classify it.

#### Syntax traversal consumers

Examples:

```text
source index
occurrence index
semantic tokens
migration utility
purity/attribute walker
boundedness walker
product optimizer visitor
anonymous-product collector
```

These should simply traverse the neutral node.

They do not need collection-family knowledge.

#### Advisory semantic consumers

`phalcom-semantic/src/advisory/analyzer.rs` may currently model bracket values as List shape.

During P1 compatibility this may remain a List-shaped advisory observation, but it must not become formal authority.

Document the compatibility nature if necessary.

#### Formal semantic consumer

`checker/expression.rs` remains temporarily List-compatible, but the function/node naming should become neutral in preparation for T7.

#### Compiler consumer

Compiler may temporarily keep emitting List during T1 while the semantic product is not yet landed.

That intermediate state is acceptable in BUILD MODE.

### Forbidden approach

Do not introduce a giant “sequence abstraction” spanning parser/checker/runtime merely to avoid renaming several exhaustive matches.

This task is syntax neutralization, not a runtime refactor.

### Discriminating tests

Parser snapshots/AST assertions proving:

```text
[1,2,3]
[*xs]
[]
```

retain identical source/evaluation structure.

### Gate G1

- no production `Expr::ListLiteral` remains;
- no semantic behavior has changed;
- current List literal focused runtime behavior remains green;
- list **patterns** remain untouched.

---

## T2 — Add first-class generic inference-hole syntax

### Reasoning objective

Create a source representation that can carry a request for inference without pretending it is a type/value.

### Implementation shape

Add conceptually:

```rust
GenericArgumentSyntax::InferenceHole {
    range: SourceRange,
}
```

Update:

```text
PartialEq
range()
as_single()
all exhaustive AST visitors
source-index visitors
static-key visitors
semantic formation matches
static evaluator matches
test constructors
```

### Context policy

At this stage, strict type formation must **reject** a hole in contexts that require a complete canonical form.

Examples:

```text
class declaration generic application in a public signature
trait requirement
impl header
type alias body if no inference-producing owner exists
metadata-bound declaration contract
```

The exact diagnostic code may reuse an appropriate existing code or add a focused one.

Required semantic message:

> This `_` requires an inference-producing context; this declaration/site requires a complete generic/static argument.

### Hole is not a reference

Source index / semantic tokens must not create a reference target for `_`.

### Hole is not a value expression

Do not convert it to:

```text
Expr::Var("_")
GenericArgumentSyntax::Value(...)
```

### Static evaluator

An inference hole should never reach evaluation as a valid unsolved static argument.

The relevant branch should fail through checked invalidity/unreachable internal state, not accidentally evaluate `_` as a binding.

### Tests

Parser:

```text
List<_>
Foo<_, Int>
Foo<Int, _>
```

Strict semantic negative:

```text
hole at non-inference-producing durable site
```

AST identity test:

```text
Pattern::_ != GenericArgumentSyntax::InferenceHole
```

### Gate G2-A

The repository can carry hole syntax end-to-end without assigning it a canonical meaning outside an authorized inference context.

---

## T3 — Parse explicit bracket heads without assigning family in the parser

### Reasoning objective

Teach source formation:

```text
<T>[...]
<T,N>[...]
```

while preserving the semantic boundary.

### Lookahead requirement

At primary-expression position, a leading `<` may start:

```text
type lambda
future bracket head
```

and angle tokens also participate in comparisons/generic syntax elsewhere.

Implement a non-consuming predicate equivalent to:

> “Does this `<` have a syntactically valid matching generic-argument `>` whose next significant token is `[`?”

The lookahead must respect the parser's existing:

```text
Less / Greater
ShiftLeft / ShiftRight
nested generic application
newlines
```

machinery.

Prefer reusing existing shift-aware angle utilities over writing a second token interpretation.

### Parsing

Once the head is recognized:

1. parse the head's arguments with canonical `parse_generic_argument`;
2. preserve their exact syntax;
3. require `>`;
4. parse the following neutral bracket literal;
5. attach the head.

### Parser forbidden behavior

Do not encode:

```rust
if args.len() == 1 { Array }
if args.len() == 2 { Vector }
```

inside AST construction.

Semantic analysis owns the language interpretation.

### Required parser cases

Positive:

```telos
<Int>[1,2]
<_>[1,2]
<Int, 3>[1,2,3]
<_, _>[1,2,3]
<List<Int>>[...]
<
  Int,
  3
>[
  1,2,3
]
```

Non-regression/negative:

```text
type lambda syntax
nested generic >>
ordinary comparisons
malformed missing >
missing [
empty head if grammar disallows it
too many arguments (parser may preserve; semantic layer diagnoses)
**/*** lane misuse inside bracket element remains current error
```

### Gate G2-B

Head syntax is source-accurate and neutral.

---

## T4 — Build inference-bearing annotation elaboration

### Reasoning objective

Create the missing bridge between:

```text
annotation syntax containing `_`
```

and:

```text
existing canonical inference machinery
```

without weakening ordinary type formation.

### Architectural rule

Do **not** retrofit `resolve_type_annotation` so it sometimes returns a pseudo-canonical type containing inference variables.

Keep strict type formation strict.

Add a checker-owned inference-producing elaboration path.

### Suggested flow

```text
TypeAnnotation
    ↓
contains no inference hole?
    ├─ yes -> delegate to canonical resolve_type_annotation
    │        -> AnnotationElaboration::Canonical
    │
    └─ no  -> inference-aware application elaboration
             -> canonical origin
             -> destination binder metadata
             -> fresh kinded variables for holes
             -> canonical subterms for explicit arguments
             -> InferenceTerm
```

### Key design obligation: reuse generic-argument semantics

Do not duplicate static-domain/type argument formation rules.

For a non-hole argument under a partial application, reuse or factor the existing canonical “argument against expected binder kind/domain” machinery.

The implementation may require extracting a reusable helper from `types/annotation.rs`.

That is preferable to implementing a partial-annotation-specific interpretation of:

```text
Type argument
static Int argument
static data term
record-row/Pack-related argument
```

### Hole kind

The hole's variable kind must come from the destination binder.

Conceptually:

```rust
let param = generic_signature.parameter_at(index);
let kind = store.type_parameter(param).kind;
let var = inference_session.fresh_variable(kind);
```

Never infer the kind from the syntax of sibling arguments.

### Nested holes

The infrastructure may support recursively nested holes when this falls naturally out of the same elaborator.

Do not expand acceptance scope into arbitrary public-signature inference.

### Inference lifecycle

Use the current `InferenceCoordinator`/session lifecycle.

Respect:

```text
propagation
projection normalization
residual retry
constraints
finish outcome
materialization
```

### Tests independent of Array/Vector

Use real existing List:

```telos
let xs: List<_> = [1,2,3]
```

for type-kind inference.

For static-domain hole behavior, use a semantic test declaration with heterogeneous binders rather than creating Vector early.

Example conceptual test owner:

```text
Synthetic<T: Type, N: Int>
```

Prove:

```text
first hole kind = Type
second hole kind = Int static domain
```

### Gate G3-A

A hole can be elaborated into canonical inference terms with exact binder kinds, and strict type formation remains hole-free.

---

## T5 — Integrate partial annotations with initializer checking and canonical publication

This is the core semantic task.

### Reasoning objective

Change the order of information flow from:

```text
annotation must be complete
then initializer is checked
```

to:

```text
annotation may create local unknowns
initializer supplies evidence
solver closes them
only then durable binding contract is published
```

### T5-A — Ordinary `let` / `let mut` binding path

Refactor the `Statement::Let` path conceptually:

```text
annotation
    ↓
AnnotationElaboration

Canonical
    ↓
ExpectedType::Proper
    ↓
analyze initializer
    ↓
existing relation/reconciliation

Inference
    ↓
ExpectedType::Inference
    ↓
analyze initializer
    ↓
propagate/finish canonical inference query
    ↓
Solved?
    ├─ no -> diagnostic/status; no inferred partial contract
    └─ yes -> materialize annotation term to TypeId
              ↓
              existing relation/reconciliation
              ↓
              BindingContract canonical only
```

Preserve the existing causal-invalidity, explanation, and binding-reconciliation architecture.

Do not fork a parallel “inferred annotation binding” flow after solving.

### Expected List proof

The existing `ExpectedType::Inference` + `collection_element_type` path should allow:

```telos
let xs: List<_> = [1, 2, 3]
```

to solve `T = Int`.

This is an important architecture proof because it demonstrates that collection literal checking and annotation inference compose through the existing expected-type domain.

### Empty literal

This must remain underconstrained:

```telos
let xs: List<_> = []
```

unless another legitimate constraint supplies T.

Do not use List's existence as evidence for its element type.

### Conflicting evidence

Example conceptual:

```text
annotation variable T constrained two incompatible ways
```

must produce canonical inference conflict / binding diagnostic, not a widened Dynamic type.

### T5-B — `const` / static constant path

Do not assume `Statement::Let` covers module constants.

The live module constant path resolves annotations in `static_constant_demand.rs` before checking the initializer with an expected canonical type.

P1 must decide and implement a coherent inference-producing static-constant path if `const` holes are claimed.

Required architecture:

```text
constant discovery pass
    ↓
force static dependencies
    ↓
annotation elaboration (canonical or inference)
    ↓
check static initializer with Proper/Inference expectation
    ↓
finish inference
    ↓
canonical expected/result type
    ↓
static evaluation/publication
```

This likely requires generalizing the static expression checker input from:

```text
Option<TypeId>
```

to a representation capable of carrying checker-local inference expectation, or giving the static checker a bounded inference-aware annotation entry point.

### Static constant invariants

- demand scheduling remains canonical;
- no LKG/stale value is used as inference evidence;
- computed Record-key demand logic remains single-owner;
- the published static constant result type is canonical;
- inference variables do not enter `CheckedStaticExpression` durable result type;
- input fingerprints still cover the annotation syntax.

### Computed-key composition boundary

P1 does not need to generalize every complex annotation composition.

If a partial inference hole appears in the same annotation as computed Record-key projection and the current architecture cannot compose them without a new owner/protocol, classify explicitly.

Do not silently skip one feature.

If composition is required for ordinary collection annotations, solve it.

If it represents an unrelated broad generic feature, a precise unsupported diagnostic may be acceptable for P1 pending consultation.

### Gate G3-B

Required evidence:

```text
List<_> nonempty binding -> canonical List<Int>
List<_> empty binding -> underconstrained
two holes remain distinct until constrained
no InferVar in BindingContract
static-domain synthetic binder receives correct kind
const path either fully supports authorized hole inference or is explicitly classified/amended
```

Do not mark P1 complete while claiming `const` support that only works in ordinary statement checking.

---

## T6 — Introduce explicit evaluation phase

### Reasoning objective

Make phase an ordinary semantic context fact before P4 needs it for default selection.

### Implementation shape

Add:

```rust
EvaluationPhase::{Runtime, Static}
```

or equivalent to `CheckingContext`.

Default ordinary context:

```text
Runtime
```

Static entry points establish:

```text
Static
```

before analyzing expressions.

Likely relevant entry points include:

```text
analyze_static_expression_with_demands
analyze_static_body_inner
demand_checking_context or its caller
```

Choose one owner so phase is not redundantly set in many consumers.

### Context propagation audit

Every context clone/derived context must be classified:

```text
with_resolver
flow probes
branch probes
call-specific child contexts
static-demand contexts
test constructors
```

Unless the operation intentionally crosses a phase boundary, preserve the phase.

### Why P1 adds phase even though default remains List

P4 must not infer static-vs-runtime by inspecting `const` parents.

Landing phase now:

- establishes one semantic owner;
- lets P2/P3 use the same infrastructure;
- allows static nested literals to already carry correct phase metadata;
- separates the architectural change from the later semantic epoch cutover.

### Tests

Prove phase inside nested analysis, not merely at root.

Conceptual:

```telos
const x = Some([1,2,3])
```

In P1 the bracket still resolves List for compatibility, but its semantic checker must observe `Static` phase.

### Gate G4

- phase is explicit;
- nested static analysis observes Static;
- ordinary bodies observe Runtime;
- no current List default changes.

---

## T7 — Replace List-specific semantic synthesis with checked sequence resolution

### Reasoning objective

Make semantic analysis the durable owner of literal family/result/construction.

### Function architecture

Replace or generalize:

```text
synthesize_list_literal
```

with a neutral sequence-literal checker.

A clean conceptual decomposition is:

```text
resolve_sequence_literal_family(...)
check_sequence_literal_elements(...)
publish_sequence_literal_resolution(...)
```

Avoid one enormous function if it obscures authority.

### P1 selection policy

P1 compatibility rule:

```text
bare headless literal with no future-family implementation
    -> List

expected canonical List
    -> List

future explicit Array/Vector head
    -> syntax recognized, but no executable resolution unless canonical family exists
```

The P1 function should be shaped so P2/P3 can add families without replacing it.

### Future selection rule must be visible in architecture

The resolution algorithm must be designed for:

```text
1. explicit semantic head
2. unique concrete expected collection family
3. evaluation-phase default
```

but P1's phase-default branch returns legacy List until P4.

Do not write a permanent `match phase { Runtime => List, Static => List }` abstraction that obscures the future transition. Prefer an explicit compatibility policy/temporary branch documented as P4-owned.

### Element inference

Preserve current List behavior:

- expected element type constrains element checking;
- element contributions use canonical composition/join behavior;
- spreads reuse positional-expansion resolution;
- invalid/unknown child causes are not silently dropped.

### Resolution publication

On a semantically accepted executable literal:

```rust
ctx.sequence_literals.insert(expr_id, resolution)
```

The resolution must contain canonical type terms only.

### Static checked products

Because static expressions use the same checker, ensure the sequence-resolution index needed by future P4 static evaluation is retained in static checked products where appropriate.

At minimum inspect and, if required, extend:

```text
CheckedStaticExpression
CheckedStaticBody
static evaluator input products
```

P1 need not evaluate static List as a new static value; it must not throw away the semantic resolution that P4 will need.

### Fingerprint

Add sequence resolutions to the canonical body/static-product fingerprint as appropriate.

Hash semantic meaning:

```text
expression identity
family
result type
element type
extent
construction category/count
```

Avoid making source range part of semantic meaning unless current product fingerprint conventions require it.

### Incremental test

A change that changes literal element/result semantics should change the affected body product.

An unrelated body/module edit should not mutate sequence resolution.

A range-only/source-position edit should preserve canonical semantic meaning.

### Gate G5

Semantic analysis now owns and publishes literal family/result construction.

---

## T8 — Project sequence resolution to core lowering and remove compiler family inference

### Reasoning objective

Complete the authority transfer.

After this task the compiler must execute the semantic decision, not recreate it.

### Module lowering

Add a compact sequence-literal lowering product to:

```text
ModuleLoweringSemantics
```

Project from semantic `ExpressionId` resolution to compiler lookup identity, likely current `SourceRange`.

Follow the existing positional-expansion projection style.

### Compiler

Change neutral bracket codegen:

```text
look up checked sequence-literal spec
    ↓
family == List in P1
    ↓
emit current List construction path
```

The compiler may inspect syntax to compile child expressions in source order.

It may not inspect syntax to decide semantic family.

### Fixed List path

Preserve:

```text
BuildList(n)
```

for no-spread literals.

### Spread List path

Preserve current:

```text
BeginListLiteral
ListLiteralAppend
FinishListLiteral
ListTryExpandTuplePositionals
```

and existing semantic positional-expansion evidence.

Do not generalize VM list builders into an abstract sequence builder in P1 unless an existing core abstraction makes that change trivial and behavior-neutral.

P2/P3 can introduce target-specific construction.

### Missing lowering fact

Production behavior:

```text
checked executable BracketLiteral
+ no sequence lowering spec
= internal compiler error
```

Not:

```text
fallback to List
```

If compiler unit tests bypass semantic lowering, update the harness to provide the checked product or test a lower-level helper.

Do not add a production compatibility fallback solely for tests.

### Compiler-side AST visitors

Update:

```text
boundedness
attributes
product_opt
anonymous product collection
```

to neutral syntax terminology where not already done in T1.

Their syntactic analyses may inspect element shapes but must not infer collection family.

### Gate G6

Disassembly/runtime evidence proves current accepted List cases emit the same opcodes/results, while compiler code no longer owns the List-default decision.

---

## T9 — Stabilize, close P1 records, and hand off to P2/P3

### Reasoning objective

Prove the architectural foundation, not the future collection families.

### Stabilization work

Run the focused suites defined below.

Classify failures A/B/C/D.

Resolve only P1-caused failures.

### Required lifecycle outputs

Update:

```text
docs/implementation/COLL006/COLL006.C1/COLL006.C1-CHECKPOINT.md
docs/implementation/COLL006/STATUS.md
```

Create:

```text
COLL006.C1.P1-walkthrough.md
COLL006.C1.P1-handoff.md
```

The walkthrough must record actual implementation, not restate this plan.

The handoff must tell P2/P3 exactly which interfaces are stable and must not be redesigned.

### Gate G7 — P1 final acceptance

All P1 acceptance criteria in §4 and §25 must be satisfied or explicitly amended.

---

# 14. Testing-surface design

Testing must cover the semantic architecture broadly while execution remains selective.

## 14.1 Parser dimensions

### Bare literals

```text
[]
[1]
[1,2,3]
[*xs]
[1,*xs,2]
```

### Heads

```text
<Int>[1]
<_>[1]
<Int,3>[1,2,3]
<_,_>[1,2,3]
<List<Int>>[...]
multiline heads
nested generic >>
```

### Ambiguity/non-regression

```text
type lambda
comparison operators
generic applications
shift tokens
pattern wildcard
list pattern
postfix indexing
computed product labels using [...]
```

## 14.2 Hole inference dimensions

Positive:

```telos
let xs: List<_> = [1,2,3]
```

Synthetic generic tests:

```text
Type-kind hole
Static-Int-domain hole
multiple independent holes
nested applied term if naturally supported
```

Negative:

```text
List<_> = []
hole at forbidden durable signature
kind mismatch
conflicting constraints
residual equality not proven
```

Publication:

```text
BindingContract contains canonical TypeId
no InferVar in published signature/product
```

## 14.3 Phase dimensions

```text
ordinary runtime literal
static constant root literal
nested literal in static expression
static definition body literal
child context / resolver context preserves phase
```

Remember: expected family remains List in P1 compatibility.

## 14.4 Semantic resolution dimensions

```text
fixed List literal
spread List literal
empty contextual List
empty uncontextual List current behavior
invalid spread
unknown/dynamic spread compatibility
result type
element type
construction plan
source expression identity
```

## 14.5 Lowering dimensions

```text
fixed literal -> BuildList(n)
spread literal -> existing builder sequence
known positional expansion uses checked iterator requirements
invalid expansion fails as before
missing sequence semantic fact fails internally
```

## 14.6 Incremental dimensions

At least one focused case should prove:

```text
cold and incremental sequence resolution equal
meaningful literal edit changes relevant fingerprint/product
unrelated edit preserves unaffected product
```

Do not turn P1 into an incremental-system redesign.

## 14.7 Static constant dimensions

If P1 supports holes in `const` annotation:

```text
const generic hole solved from initializer
underconstrained const hole
static dependency forcing unaffected
canonical result type
cold/incremental parity
```

---

# 15. Verification execution budget

## 15.1 BUILD MODE

Default mode.

Run:

- exact parser regression after T2/T3;
- exact semantic hole test after T4/T5;
- exact phase test after T6;
- exact sequence product test after T7;
- exact compiler/disassembly test after T8.

Use `cargo check` for a directly affected crate when an exhaustive AST enum change makes compiler feedback useful.

Do **not** run:

```text
full workspace tests
workspace clippy
all core tests
all semantic tests
```

after each task.

## 15.2 STABILIZE MODE

At T9/G7, run:

- focused `phalcom-ast` parser tests for changed grammar;
- focused semantic foundations/type-annotation/bidirectional-expression groups;
- focused semantic incremental fingerprint/parity tests added by P1;
- focused core collection literal/spread compiler/runtime tests;
- affected-crate `cargo check`;
- formatting/diff hygiene.

Determine exact filters with `-- --list`; confirm tests actually ran.

## 15.3 CERTIFY MODE

P1 itself does not require full checkpoint/release certification unless repository policy or the user explicitly requests it.

Defer:

```text
workspace test
workspace clippy
release gates
```

to checkpoint/release certification unless evidence exposes a cross-crate problem that genuinely requires broader verification.

---

# 16. Baseline/unrelated failure policy

Classify every unexpected failure.

```text
A — definitely caused by P1
    fix

B — probably caused by P1
    investigate and fix

C — unclear
    one bounded causal classification pass;
    if nonblocking, record and defer

D — clearly baseline/unrelated
    record and continue
```

Do not repair random/bootstrap/math/string/other failures simply because P1 testing encounters them.

Do not spend architectural consultation on category D unless it prevents obtaining required P1 evidence.

---

# 17. Semantic debugging discipline

## 17.1 Mechanical failure

Examples:

```text
new enum arm missing
renamed import
wrong helper signature
test constructor missing new field
borrow cleanup
```

Use a few coherent correction cycles.

## 17.2 Semantic failure

Before changing code:

```text
Observed:
Expected semantic proposition:
Hypothesis:
Evidence:
Prediction:
Smallest discriminating test:
```

Example:

```text
Observed:
List<_> remains underconstrained after [1,2,3].

Expected:
ExpectedType::Inference should project the element InferVar into each literal element.

Hypothesis:
The annotation inference context differs from the context stored in ExpectedType,
or list element extraction is rejecting the applied origin.

Evidence:
...

Prediction:
If context/origin mismatch is repaired, integer element checks bind the variable.

Test:
single List<_> binding regression.
```

Make one serious correction to that causal model.

If the same semantic problem persists, enter RED.

## 17.3 Architectural failure

Do not attempt speculative rewrites.

Examples:

```text
need InferVar in TypeStore to continue
compiler appears to need to redo family inference
static constant scheduler cannot support initializer-dependent annotation without changing declaration ownership
head grammar cannot be made unambiguous under current parser model
```

Stop and consult.

---

# 18. P1-specific STOP AND CONSULT triggers

In addition to repository-global triggers, P1 must stop if:

1. `_` cannot be modeled without storing inference variables in canonical `TypeStore`;
2. the destination binder's kind/domain cannot be obtained canonically at annotation elaboration;
3. partial annotations require a second generic-application semantics implementation rather than factoring/reusing the canonical one;
4. `ExpectedType::Inference` cannot constrain bracket elements without redesigning the general expected-type contract;
5. static constant hole inference requires changing static identity, evaluation, or demand ownership beyond the bounded checker input contract;
6. explicit bracket-head parsing cannot be disambiguated without changing broader expression/type-lambda grammar;
7. a concrete Array/Vector identity appears necessary merely to complete P1;
8. sequence resolution cannot be published through the existing `CheckingContext -> analysis -> lowering` product architecture;
9. compiler tests appear to require a production fallback when semantic lowering is absent;
10. compiler family selection would remain duplicated after T8;
11. phase would need to be inferred from syntax/parent inspection rather than explicit context;
12. phase must change overload ranking to make tests pass;
13. sequence resolution fingerprinting creates global/unrelated invalidation and there is no clear local correction;
14. a hole is accepted in a durable public signature without an explicit language decision;
15. existing List behavior must change before P4 to make the architecture work;
16. the implementation would need to amend TDR-0025/TDR-0028 during P1;
17. the recent inference residual-obligation model would need to be bypassed;
18. the same nontrivial inference/semantic failure survives one serious correction.

The implementer cannot self-waive these triggers.

---

# 19. Expected impact map

## Primary semantic/source write set

```text
phalcom-ast/src/ast.rs
phalcom-ast/src/parser.rs
phalcom-ast/tests/parser.rs

phalcom-semantic/src/types/annotation.rs
phalcom-semantic/src/checker/expected.rs
phalcom-semantic/src/checker/inference.rs        # only if reusable helper/lifecycle support is genuinely needed
phalcom-semantic/src/checker/context.rs
phalcom-semantic/src/checker/analysis.rs
phalcom-semantic/src/checker/expression.rs
phalcom-semantic/src/checker/statement.rs
phalcom-semantic/src/checker/static_body.rs
phalcom-semantic/src/static_constant_demand.rs   # const-hole integration
phalcom-semantic/src/static_constant.rs          # if checked static product gains sequence resolutions
phalcom-semantic/src/db/fingerprint.rs
```

## AST traversal/mechanical consumers

```text
phalcom-ast/src/bin/c4_migrate.rs
phalcom-lsp/src/semantic_tokens.rs
phalcom-semantic/src/advisory/analyzer.rs
phalcom-semantic/src/source_index/builder.rs
phalcom-semantic/src/source_index/occurrence.rs
phalcom-semantic/src/static_key.rs
phalcom-semantic/src/impls.rs
phalcom-semantic/src/static_eval/machine.rs
```

## Core lowering/compiler

```text
phalcom-core/src/modules/semantic_lowering.rs
phalcom-core/src/modules/materialize.rs           # only if new lowering spec requires remapping; likely minimal/none
phalcom-core/src/compiler/lib/expr.rs
phalcom-core/src/compiler/attributes.rs
phalcom-core/src/compiler/boundedness.rs
phalcom-core/src/compiler/lib/product_opt.rs
```

## Tests

Likely:

```text
phalcom-semantic/tests/semantic/foundations/type_annotations.rs
phalcom-semantic/tests/semantic/foundations/bidirectional_calls.rs
phalcom-semantic/tests/semantic/incremental/fingerprints.rs
relevant semantic integration/source-index tests
phalcom-core/tests/core/collections/outgoing_packs_completion.rs
relevant compiler declaration/expansion tests
```

## Explicitly not expected in P1

```text
phalcom-core/src/heap/*
phalcom-core/src/primitive/array.rs
phalcom-core/src/primitive/vector.rs
phalcom-native-meta Universe Array/Vector identities
new Array/Vector .ph source classes
new Array/Vector bytecodes
GC tracing for new collections
```

If those become required, reassess the P1/P2/P3 boundary.

---

# 20. Detailed failure modes to actively guard against

## FM-01 — Hole becomes Dynamic

Symptom:

```text
List<_> accepted as List<Dynamic>
```

Cause:

Treating `_` as a permissive type rather than inference request.

Reject.

## FM-02 — Hole escapes through TypeStore

Symptom:

Canonical type/product contains solver variable identity.

Cause:

Trying to make strict type formation “partially canonical.”

Reject architecture.

## FM-03 — Parser hardcodes family

Symptom:

AST node becomes `ArrayLiteralHead` based on one argument.

Cause:

Confusing syntax arity with semantic classification.

Move decision to semantic layer.

## FM-04 — Compiler defaults missing product to List

Symptom:

Codegen succeeds when semantic sequence resolution is absent.

Cause:

Compatibility fallback retained too long.

Replace with internal failure.

## FM-05 — Phase inferred from `const` parent

Symptom:

Literal checker asks whether enclosing statement is const.

Cause:

No explicit evaluation context.

Use `EvaluationPhase`.

## FM-06 — Partial annotation duplicates generic application

Symptom:

A large new resolver branch separately interprets static/type arguments.

Cause:

Treating hole support as collection-local feature.

Factor/reuse canonical argument formation.

## FM-07 — “Materializable” mistaken for solved

Symptom:

Residual generic equality remains but binding publishes type.

Cause:

Ignoring recent inference retry/residual semantics.

Require solved inference outcome.

## FM-08 — Concrete Array/Vector stubs in P1

Symptom:

Temporary Universe keys/classes exist only so syntax can typecheck.

Cause:

Crossing successor-plan boundary.

Remove and test mechanisms synthetically.

## FM-09 — AST rename causes advisory layer to become formal authority

Symptom:

compiler/checker consults advisory list shape to select family.

Cause:

Convenient existing consumer reused incorrectly.

Formal semantic checker remains authority.

## FM-10 — Static constants remain a hidden gap

Symptom:

`let xs: List<_>` works but `const xs: List<_>` is claimed supported despite strict static annotation resolution.

Cause:

Only ordinary binding path was changed.

Either complete static path or report scope honestly.

## FM-11 — Range-only edit invalidates semantic meaning

Symptom:

sequence resolution fingerprint changes solely because source range moved.

Cause:

Hashing location as semantics.

Follow existing semantic fingerprint policy.

## FM-12 — Test-driven semantic weakening

Symptom:

empty hole gets Dynamic/Object so a corpus test passes.

Cause:

Using test success as authority over inference semantics.

Reject.

---

# 21. What P1 must leave for P2

P2 should inherit, not redesign:

```text
BracketLiteral AST
optional head syntax
InferenceHole syntax
AnnotationElaboration mechanism
EvaluationPhase
SequenceLiteralResolution
sequence literal analysis index
semantic fingerprint participation
ModuleLoweringSemantics sequence-literal projection
compiler checked-product consumption
```

P2 should only need to:

- introduce canonical `Array<T>` family identity/runtime surface;
- teach semantic family resolution that expected/explicit Array is executable;
- teach compiler lowering how Array resolution constructs Array;
- preserve the P1 architecture.

If P2 needs to redesign the parser or partial-annotation solver, P1 was incomplete.

---

# 22. What P1 must leave for P3

P3 should inherit the same foundation and add:

```text
Vector<T,N> canonical family
static N binder/domain
exact cardinality reasoning
extent publication
Vector lowering
```

P3 must not need to:

- introduce `_`;
- add another solver;
- add another literal-head grammar;
- change semantic/compiler authority;
- infer phase from source parents.

The `extent` slot in sequence resolution must be capable of carrying P3's canonical static term without host-width assumptions.

---

# 23. What P1 deliberately leaves for P4

P4 owns the semantic epoch switch.

P1 must leave a clearly identifiable compatibility policy:

```text
headless / no unique family context
    -> legacy List default
```

P4 replaces that policy with:

```text
Runtime -> Array
Static  -> Vector
```

only after Array, Vector, and static sequence materialization exist.

P4 also owns:

- repository migration of bare brackets;
- TDR amendment/supersession;
- static homogeneous sequence representation;
- const Array/Vector materialization;
- final overload/default certification.

P1 must not partially flip semantics.

---

# 24. Lifecycle bookkeeping during implementation

## At plan start

Update checkpoint:

```text
active plan = COLL006.C1.P1
status = IN_PROGRESS
starting revision = actual HEAD
```

## At meaningful architecture gates

Record only durable facts:

- G1 neutral AST;
- G3 hole inference publication boundary;
- G4 phase;
- G5 sequence semantic product;
- G6 compiler authority transfer.

## Consultations

Record:

```text
incident
task/gate
trigger
decision
architecture consequence
plan amendment
tests added
```

Do not paste full adviser conversations.

## At completion

Checkpoint plan ledger should truthfully report:

```text
status
completion
verification
outcome
```

Create walkthrough and handoff.

---

# 25. Final P1 acceptance checklist

## Repository/process

- [ ] P1 started from recorded current revision.
- [ ] checkpoint marked active before substantial implementation.
- [ ] unrelated work preserved.
- [ ] no unauthorized broad repair.

## Syntax

- [ ] `Expr::ListLiteral` production ownership removed.
- [ ] neutral bracket literal AST exists.
- [ ] neutral element expansion syntax exists.
- [ ] optional head syntax exists.
- [ ] `<T>[...]` parses.
- [ ] `<T,N>[...]` parses.
- [ ] `_` has dedicated generic argument node.
- [ ] list patterns unchanged.
- [ ] type lambda/comparison/generic grammar non-regressed.

## Inference

- [ ] strict annotation resolver remains canonical.
- [ ] inference-producing annotation elaborator exists.
- [ ] holes get exact destination binder kind/domain.
- [ ] holes are fresh.
- [ ] `List<_> = [1,2,3]` solves to canonical List<Int>.
- [ ] underconstrained empty case diagnoses.
- [ ] conflicting case diagnoses.
- [ ] mandatory residual obligations cannot be bypassed.
- [ ] no InferVar reaches durable product.
- [ ] static-domain hole behavior has focused proof.

## Binding/static integration

- [ ] ordinary binding contract publication occurs only after solve.
- [ ] existing relation/reconciliation path is reused.
- [ ] const/static binding path has been audited.
- [ ] if const holes are within P1 acceptance, static demand path supports them canonically.
- [ ] static result type is canonical.
- [ ] demand/evaluation ownership remains unchanged.

## Phase

- [ ] EvaluationPhase exists.
- [ ] runtime contexts default Runtime.
- [ ] static expression/body contexts use Static.
- [ ] derived contexts preserve phase.
- [ ] nested static literals observe Static.
- [ ] phase does not flip P1 List default.

## Semantic products

- [ ] SequenceLiteralResolution exists.
- [ ] CheckingContext records it.
- [ ] CallableAnalysis publishes it.
- [ ] static checked product retains it where future static evaluation requires.
- [ ] fingerprint includes semantic resolution meaning.
- [ ] bare P1 literal resolution remains List.
- [ ] no fake Array/Vector result is published.

## Compiler/lowering

- [ ] ModuleLoweringSemantics carries sequence literal lowering spec.
- [ ] compiler consumes spec.
- [ ] compiler does not decide family from AST/phase/expected type.
- [ ] missing valid spec fails closed.
- [ ] fixed List emits existing BuildList path.
- [ ] spread List emits existing builder path.
- [ ] positional expansion evidence remains canonical semantic input.
- [ ] no Array/Vector bytecode/runtime work landed.

## Incrementality/tests

- [ ] exact parser tests pass.
- [ ] exact hole inference tests pass.
- [ ] exact phase tests pass.
- [ ] exact sequence-resolution tests pass.
- [ ] exact compiler/disassembly tests pass.
- [ ] focused cold/incremental parity case passes.
- [ ] affected focused suites pass or failures classified.
- [ ] test filters selected real tests.
- [ ] broad tests deliberately deferred unless actually required.

## Records

- [ ] checkpoint updated with actual outcome.
- [ ] walkthrough created.
- [ ] handoff created.
- [ ] residual issues classified.
- [ ] P2/P3 stable takeover interfaces recorded.

---

# 26. Final implementer guidance

The difficult part of P1 is not renaming `ListLiteral`.

The difficult part is establishing a semantic architecture in which later collection families can be added **without new sources of truth**.

When deciding between two implementation approaches, prefer the one that preserves this pipeline:

```text
source syntax
    ↓
canonical semantic question
    ↓
query-local inference if necessary
    ↓
fully canonical semantic answer
    ↓
durable semantic product
    ↓
projection to lowering
    ↓
mechanical code generation
```

The implementer should continuously ask:

```text
Am I preserving source syntax, or assigning meaning?
Am I carrying uncertainty, or fabricating a type?
Am I using an existing semantic owner, or creating a parallel one?
Am I proving a fact once, or making a consumer infer it again?
Am I changing P1 architecture, or merely adapting mechanics?
```

A strong implementation of P1 makes P2/P3/P4 boring in the right ways.

They should extend the family table and runtime representations—not reopen parser ownership, inference philosophy, phase propagation, or compiler semantic authority.

That is the principal success criterion of this plan.
