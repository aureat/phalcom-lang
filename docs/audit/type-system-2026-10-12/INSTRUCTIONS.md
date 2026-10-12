# TELOS — COMPREHENSIVE TYPE SYSTEM AND COMPILER SEMANTICS AUDIT
## Repository Architecture, Correctness Requirements, Failure Taxonomy, Investigation Priorities, and Verification Strategy

**Document type:** Technical Audit Requirements Analysis
**Repository:** `aureat/phalcom-lang`
**Target branch:** `main`
**Reference revision:** `5942666901b387519c5cd502bed2611cf97b53c4`
**Assessment date:** 2026-10-12
**Scope:** Type system, semantic analysis, inference, checker, compiler, semantic lowering, runtime typing, incremental analysis, diagnostics, and supporting infrastructure.
**Audit classification:** Cross-layer semantic correctness and implementation reliability.
**Implementation authorization:** None. Read-only investigation and audit.
**Evidence status:** Initial repository inspection completed; exhaustive code-level validation pending.

---

# 1. EXECUTIVE ASSESSMENT

## 1.1 Primary Objective

Conduct a comprehensive, adversarial, implementation-level audit of the Telos type system and every compiler subsystem capable of influencing source-language typing semantics.

The audit must investigate:

1. Incorrect acceptance of invalid programs.
2. Incorrect rejection of valid programs.
3. Compiler or semantic analyzer crashes on valid or invalid source.
4. Type inference failures, divergence, ambiguity, and underconstraint.
5. Incorrect generic specialization or substitution.
6. Incorrect nominal, structural, and algebraic subtyping.
7. Conformance and trait resolution inconsistencies.
8. Incorrect associated-type and type-member projection.
9. Unsound flow-sensitive reasoning.
10. Incomplete initialization and mutability verification.
11. Incorrect overload, method, family, and callable resolution.
12. Missing or inconsistent semantic evidence.
13. Incorrect compiler lowering of valid semantic products.
14. Semantic information silently discarded between compilation stages.
15. Runtime behavior inconsistent with statically established types.
16. Incorrect diagnostic suppression and error recovery.
17. Incremental analysis returning results different from clean analysis.
18. Incorrect publication or reuse of cached semantic products.
19. Unhandled recursive or cyclic semantic dependencies.
20. Missing source-language constructs and combinations in test coverage.

The investigation must establish both:

- Whether individual implementations satisfy their contracts.
- Whether the entire compilation pipeline preserves those contracts.

The second requirement is particularly important.

A locally correct type solver can still participate in an incorrect compiler when its results are projected improperly, attached to the wrong source expression, specialized with an incorrect generic environment, or consumed after becoming stale.

## 1.2 Architectural Assessment

The repository implements a comparatively sophisticated static semantic architecture.

The principal responsibilities are divided among:

- `phalcom-ast`: lexical and syntactic representation.
- `phalcom-type-syntax`: type syntax infrastructure.
- `phalcom-modules`: project identities, namespaces, imports, linking, and module graphs.
- `phalcom-semantic`: canonical type system and static semantic authority.
- `phalcom-type-meta`: versioned semantic metadata and persistence schemas.
- `phalcom-core`: compilation, semantic lowering, bytecode, VM, runtime typing, and native integration.
- `phalcom-lsp`: editor-facing semantic consumers.
- `phalcom-diagnostics`: diagnostic infrastructure.

Within `phalcom-semantic`, the architecture further separates:

- Canonical type representations.
- Kinds and generic applications.
- Subtyping and assignability.
- Inference and constraints.
- Declaration registration.
- Callable checking.
- Conformance resolution.
- Associated-type reasoning.
- Static evaluation.
- Semantic database queries.
- Immutable semantic snapshots.
- Editor/advisory semantic products.

This separation is beneficial, but it introduces important consistency obligations.

Every architectural boundary represents a potential location where a valid semantic result can become incomplete, incorrectly specialized, incorrectly transported, or detached from its original identity.

## 1.3 Initial Risk Assessment

The audit should initially concentrate on six areas.

| Rank | Area | Priority | Principal risk |
|---|---|---|---|
| 1 | Semantic evidence and compiler lowering | P0 | Correctly checked programs compiled incorrectly |
| 2 | Generic inference and constraint solving | P0 | Incorrect inference, rejection, or unsound acceptance |
| 3 | Conformance, traits, associated types, families | P0 | Invalid proof selection or missing specialization |
| 4 | Semantic error recovery and invariant enforcement | P0 | Panics, inconsistent state, false-success analysis |
| 5 | Canonical type relations and type formation | P0 | Incorrect foundational typing judgments |
| 6 | Incremental database and snapshot consistency | P0 | Stale or contradictory semantic results |
| 7 | Flow-sensitive typing and initialization | P1 | Unsafe reads, invalid narrowing, inconsistent contracts |
| 8 | Call and overload resolution | P1 | Incorrect candidate selection or dispatch identity |
| 9 | Algebraic types, patterns, GADTs | P1 | Incorrect branch proofs or exhaustiveness |
| 10 | Static evaluation and type-level computation | P1 | Incorrect normalization, residualization, or divergence |
| 11 | Compiler and VM runtime typing | P1 | Violation of established semantic contracts |
| 12 | Modules, imports, and cross-module identity | P1 | Incorrect symbol ownership or invalid cache reuse |
| 13 | Diagnostics and editor products | P2 | Misleading or inconsistent semantic presentation |
| 14 | Performance, determinism, scalability | P2 | Excessive complexity or unstable analysis |
| 15 | Public API and architecture hygiene | P3 | Maintainability problems and future correctness risk |

These are provisional risk priorities, not findings that these systems are defective.

---

# 2. AUDIT PHILOSOPHY AND CORRECTNESS MODEL

## 2.1 Distinguish Five Classes of Failure

The audit must classify every discovered problem precisely.

### A. Type-system soundness failure

An invalid typing judgment is accepted, potentially permitting execution that violates a language-level type guarantee.

Examples:

- Treating an unconstrained generic type as conforming to a trait.
- Accepting an invalid subtype conversion.
- Using a branch-local GADT equality outside its branch.
- Assigning a value to an incompatible typed binding.
- Treating an unresolved associated-type equality as established.

### B. Type-system completeness failure

A valid program is rejected because the implementation cannot establish a typing judgment that follows from the language specification.

Examples:

- Failure to infer a generic variant constructor under an expected type.
- Failure to propagate an expected return type through a callable.
- Failure to recognize inherited generic conformance.
- Failure to apply an existing associated-type equality.
- Failure to recognize valid structural record compatibility.

### C. Compiler realization failure

Semantic analysis reaches a valid conclusion, but the compiler produces incorrect executable behavior.

Examples:

- Incorrectly specialized invocation.
- Missing runtime type-environment bindings.
- Invalid trait witness selection.
- Incorrect method-family identity.
- Lost generic arguments.
- Incorrect field or variant layout.

### D. Robustness failure

The analyzer, compiler, or tooling fails operationally instead of producing a controlled result.

Examples:

- Panic on malformed source.
- Unbounded recursion.
- Stack overflow.
- Invalid indexing.
- Unexpected `unwrap()` or `expect()`.
- Arithmetic overflow.
- Nontermination during inference.
- Semantic database corruption.
- Incorrect recovery after cancellation.

### E. Semantic consistency failure

Different analysis paths reach incompatible conclusions about the same source.

Examples:

- Cold analysis succeeds while incremental analysis fails.
- Semantic checking accepts a call while compiler-side validation rejects it.
- LSP diagnostics disagree with CLI diagnostics.
- A type normalizes differently depending on its construction history.
- The same trait obligation resolves differently depending on query order.

The audit must not merge these categories into a generic description such as "typechecker bug."

## 2.2 Fundamental Invariants

The following invariants should guide the entire investigation.

**Invariant T1 — Canonical authority**

All authoritative typing judgments originate in `phalcom-semantic`.

Consumers must not independently recreate formal typing, member, dispatch, conformance, or proof facts.

**Invariant T2 — Type identity**

Canonical type identity must remain stable and meaningful within its declared scope.

Two different types must not become indistinguishable through incorrect interning, substitution, metadata encoding, or runtime projection.

**Invariant T3 — Inference isolation**

Inference-local variables must never escape into canonical type storage or published semantic snapshots.

**Invariant T4 — Evidence integrity**

Evidence used for specialization, conformance, dispatch, or lowering must correspond to the exact semantic judgment established by analysis.

**Invariant T5 — Sound uncertainty**

An unknown, blocked, incomplete, cancelled, or budget-exhausted result must not silently become a successful proof.

**Invariant T6 — Lowering preservation**

A successfully checked expression must lower according to the semantic facts attached to that expression.

**Invariant T7 — Incremental equivalence**

For identical source and dependency inputs, incremental analysis and fresh analysis must produce equivalent formal semantic results.

**Invariant T8 — Failure containment**

Malformed source or an isolated analyzer failure must not corrupt unrelated semantic products.

**Invariant T9 — Determinism**

Equivalent source, dependencies, and compiler configuration must produce equivalent diagnostics, semantic resolutions, and executable behavior.

**Invariant T10 — Runtime consistency**

Runtime specialization, dispatch, and type representation must preserve the guarantees established statically.

---

# 3. REPOSITORY ARCHITECTURE AND AUDIT OWNERSHIP

## 3.1 Workspace Structure

The inspected root `Cargo.toml` declares the following workspace members:

| Crate | Primary responsibility | Audit relevance |
|---|---|---|
| `phalcom-ast` | Lexer, parser, AST | Source validity and syntax-to-semantics consistency |
| `phalcom-common` | Shared infrastructure | Identities, ranges, selectors, shared utilities |
| `phalcom-type-syntax` | Type syntax representations | Type grammar, type arguments, annotation structure |
| `phalcom-native-meta` | Native metadata | Native type and declaration consistency |
| `phalcom-native-decl` | Native declaration infrastructure | Native declaration contracts |
| `phalcom-native-macros` | Native procedural macros | Generated declaration correctness |
| `phalcom-native-surface` | Native surface representation | Source/native contract alignment |
| `phalcom-native-surface-gen` | Native surface generation | Generated surface integrity |
| `phalcom-core` | Compiler, bytecode, VM, Universe bootstrap | Semantic lowering and runtime correctness |
| `phalcom-repl` | CLI/REPL | Incremental interactive semantics |
| `phalcom-lsp` | Language server | Editor and incremental consistency |
| `phalcom-modules` | Module/project resolution | Cross-module typing and identity |
| `phalcom-semantic` | Canonical static semantics | Primary audit target |
| `phalcom-test-support` | Shared testing support | Audit harness |
| `phalcom-diagnostics` | Diagnostic infrastructure | Error propagation and presentation |
| `phalcom-type-meta` | Portable semantic metadata | Serialization and semantic identity |

The repository additionally contains independently organized development resources, such as fuzzing infrastructure.

The root workspace manifest, rather than a directory listing alone, must determine which crates belong to the active build graph.

The audit should also inspect:

- Feature-dependent crate inclusion.
- Workspace-excluded crates.
- Package-local manifests.
- Build scripts.
- Generated source files.
- Macro-generated semantic declarations.
- Integration test executables.
- Conditional compilation.

## 3.2 Primary Semantic Execution Pipeline

The principal execution model to reconstruct is:

```text
Telos source
    |
    v
phalcom-ast
    |
    | Lexer / Parser / AST
    v
phalcom-modules
    |
    | Module identity
    | Imports
    | Linking
    | Dependency topology
    v
phalcom-semantic
    |
    +-- Declaration registration
    |
    +-- Canonical type formation
    |
    +-- Generic and constraint analysis
    |
    +-- Expression / statement checking
    |
    +-- Trait and conformance resolution
    |
    +-- Flow-sensitive analysis
    |
    +-- Static evaluation
    |
    +-- Semantic evidence
    |
    +-- Semantic database
    |
    v
SemanticSnapshot
    |
    | Canonical semantic products
    v
phalcom-core/modules/semantic_lowering.rs
    |
    | Compiler-facing lowering specifications
    v
phalcom-core/compiler
    |
    | Bytecode
    v
phalcom-core/VM
    |
    | Executable values and runtime typing
    v
Program behavior
```

This diagram is an initial architectural model.

The audit must construct a verified call graph identifying actual orchestration order, early exits, alternate entry points, and error paths.

In particular, determine whether all compiler entry points require a valid semantic snapshot, or whether some may compile source with incomplete semantic information.

---

# 4. SEMANTIC CRATE: DETAILED RESPONSIBILITY MAP

## 4.1 Canonical Type System

Primary directory:

`phalcom-semantic/src/types/`

The inspected `types/mod.rs` declares these modules:

```text
annotation.rs
application.rs
case_environment.rs
case_instantiation.rs
constraint.rs
denotation.rs
environment.rs
evidence.rs
family.rs
id.rs
instantiation.rs
kind.rs
native.rs
outcome.rs
parameter.rs
projection.rs
relation.rs
rigid.rs
row.rs
row_solver.rs
specialization.rs
store.rs
substitution.rs
type_lambda.rs
type_member.rs
variance.rs
```

These files form the foundation of the type system.

### 4.1.1 `store.rs`

**Responsibility:** Canonical type representation, storage, and type-data identity.

Investigate:

- Canonical type interning.
- Type equality and hash consistency.
- Type-store ownership.
- Duplicate canonical representations.
- Recursive type construction.
- Type identity across distinct stores.
- Type normalization before interning.
- Invalid type references.
- Structural-type canonicalization.
- Applied generic types.
- Callable representations.
- Tuple and record representations.
- Union construction and normalization.
- Exact variant representations.

Required checks:

1. Equivalent canonical types receive equivalent identity.
2. Non-equivalent types cannot collide.
3. Type construction preserves kind correctness.
4. Store-local identifiers cannot be confused across stores.
5. No inference-local variable is published as a canonical type.
6. Recursive type construction cannot recurse indefinitely.

### 4.1.2 `relation.rs`

**Responsibility:** Subtyping, assignability, and related type relations.

This is one of the highest-priority files in the audit.

The public interface includes:

```rust
is_subtype(...)
check_subtype_bounded(...)
check_assignability(...)
check_assignability_bounded(...)
check_knowledge_against_type(...)
```

Investigate the distinction between:

- Type equivalence.
- Nominal inheritance.
- Structural compatibility.
- Subtyping.
- Assignability.
- Dynamic conversion.
- Constraint satisfaction.
- Type knowledge.

Verify foundational relation laws where applicable:

```text
Reflexivity:
    A <: A

Transitivity:
    A <: B and B <: C
    implies A <: C

Equality compatibility:
    A == B
    implies A <: B and B <: A
```

Additional tests:

- Generic variance.
- Union subtype relations.
- Callable variance.
- Structural record width and depth.
- Tuple positional and labeled compatibility.
- Exact variants.
- Applied superclass substitution.
- Rigid type variables.
- Associated-type projections.
- Higher-kinded applications.
- Recursive generic bounds.

Pay special attention to bounded relation queries.

A budget-exhausted relation must not be interpreted as success.

### 4.1.3 `application.rs`, `kind.rs`, `parameter.rs`

**Responsibility:** Type application, kind checking, and generic parameter semantics.

Investigate:

- Generic arity.
- Default arguments.
- Higher-kinded arguments.
- Type parameters versus static-value parameters.
- Generic constraints.
- Pack parameters.
- Partial application.
- Incorrect argument domains.
- Incorrect propagation of generic binders.
- Kind preservation through specialization.

A generic application must be invalid when the supplied argument does not inhabit the required domain.

### 4.1.4 `substitution.rs`, `instantiation.rs`, `specialization.rs`

**Responsibility:** Substitution, materialization, and context-dependent specialization.

Potential failure categories:

- Incomplete substitutions.
- Incorrect parameter ownership.
- Capture of unrelated parameters.
- Generic argument order mismatches.
- Self-type specialization failures.
- Nested generic substitution failures.
- Recursive substitution loops.
- Substitution of associated projections.
- Inherited generic specialization.

Required property:

```text
specialize(specialize(T, S1), S2)
```

must agree with a correctly composed substitution whenever composition is semantically defined.

Test nested and inherited generic cases explicitly.

### 4.1.5 `type_lambda.rs`

**Responsibility:** Scoped type abstraction and reduction.

Investigate:

- Alpha-equivalence.
- Beta-reduction.
- Variable capture.
- Nested binder shadowing.
- Invalid reduction arity.
- Termination and reduction budgets.
- Recursive type-level applications.
- Normalization stability.

### 4.1.6 `projection.rs`, `type_member.rs`

**Responsibility:** Type-level projection and normalization.

Investigate:

- Projection ownership.
- Valid projection subjects.
- Domain checking.
- Result kind.
- Generic specialization.
- Unresolved projections.
- Cyclic projection definitions.
- Projection equality.
- Static argument substitution.
- Nested projections.
- Projection under abstract generic constraints.

Distinguish:

```text
A projection is syntactically valid.
A projection is semantically resolvable.
A projection is provably equal to another type.
```

These are separate judgments.

### 4.1.7 `row.rs`, `row_solver.rs`

**Responsibility:** Structural record rows and row unification.

Investigate:

- Duplicate fields.
- Open row tails.
- Closed records.
- Row-variable unification.
- Occurs checks.
- Field type disagreements.
- Tail constraints.
- Lacks constraints.
- Cyclic row solutions.
- Row materialization.
- Canonicalization after solving.

Potential critical failure:

An incomplete row solution is materialized as a concrete closed record.

### 4.1.8 `evidence.rs`, `outcome.rs`

**Responsibility:** Representation of semantic knowledge and proof outcomes.

This subsystem requires especially careful auditing.

Verify that the implementation preserves distinctions among:

- Proven.
- Refuted.
- Unknown.
- Blocked.
- Dynamic.
- Cancelled.
- Budget-exhausted.
- Internally invalid.

Determine the exact state model used by each operation rather than assuming these labels correspond one-to-one with Rust variants.

The fundamental requirement is that a non-proof state cannot be silently strengthened into proof.

### 4.1.9 `variance.rs`

**Responsibility:** Generic variance analysis.

Test:

- Covariant occurrences.
- Contravariant occurrences.
- Invariant occurrences.
- Nested callable types.
- Mutable storage.
- Inherited generic parameters.
- Higher-kinded applications.
- Recursive type declarations.

Investigate whether a variance declaration can be accepted despite an incompatible occurrence.

---

# 5. TYPE CHECKER AND INFERENCE ARCHITECTURE

## 5.1 Checker Organization

Primary directory:

`phalcom-semantic/src/checker/`

The inspected module declarations include:

```text
algebraic_declaration.rs
algebraic_realization.rs
analysis.rs
annotation_elaboration.rs
associated.rs
binding.rs
body.rs
call.rs
causal.rs
composition.rs
context.rs
control.rs
coverage.rs
data_declaration.rs
declaration.rs
declaration_signature.rs
enum_behavior.rs
enum_declaration.rs
exhaustiveness.rs
expected.rs
expression.rs
field_lifecycle.rs
flow.rs
gadt_proof.rs
incident.rs
inference.rs
iteration.rs
loop_analysis.rs
pattern.rs
proof_predicate.rs
result.rs
row_inference.rs
sequence_literal.rs
sequence_publication.rs
statement.rs
static_body.rs
static_iteration.rs
type_member_declaration.rs
typed_expr.rs
```

This is the primary implementation area for source-level semantic correctness.

## 5.2 `inference.rs` — Highest-Priority Investigation

The inspected source identifies inference variables as solver-local entities:

```rust
// Law: InferVarId != TypeId.
```

This invariant is fundamental.

The module contains an `InferenceTerm` representation including:

- Canonical types.
- Flexible inference variables.
- Rigid variables.
- Generic applications.
- Static-data constructions.
- Exact enum cases.
- Unions.
- Tuples.
- Callables.
- Records.
- Families.
- Associated projections.
- Type-member projections.
- Type packs.
- Pack concatenations.
- Pack projections.

### Investigation A: Variable lifecycle

Determine:

1. Where inference variables originate.
2. How constraints are attached.
3. How variables are unified.
4. How solutions are propagated.
5. How unsolved variables are classified.
6. How inferred results are materialized.
7. Whether variables can escape their owning session.

### Investigation B: Constraint propagation

Test whether all supported constraints propagate in both necessary directions.

Examples:

```text
Expected result -> generic constructor arguments

Known argument -> generic parameter

Known return type -> callable specialization

Known receiver -> associated type

Known associated type -> constrained generic

Inherited generic superclass -> member specialization
```

### Investigation C: Solver soundness

Investigate:

- Occurs checks.
- Cyclic constraints.
- Conflicting constraints.
- Contradictory bounds.
- Rigid-variable assignments.
- Projection equality.
- Equality under generic substitution.
- Partial solutions.
- Constraint prioritization.
- Solver result ordering.

### Investigation D: Solver completeness

Test cases where a valid solution requires combining evidence from multiple sources.

Examples:

```telos
data Box<T> {
    value: T
}
```

Investigate representative uses involving:

- Explicit generic arguments.
- Contextually inferred arguments.
- Nested generic constructors.
- Generic return types.
- Generic superclass members.
- Associated-type equality.
- Collections.
- Variants.
- Callables.

All examples must be adjusted to the current accepted grammar before execution.

### Investigation E: Determinism

The solver should not produce different successful solutions merely because constraints were inserted in different orders when the constraints are semantically equivalent.

Construct permutation tests for:

- Argument order.
- Constraint insertion order.
- Declaration order.
- Import order.
- Candidate enumeration order.

Where source order is intentionally semantically significant, record the governing specification rule.

## 5.3 `row_inference.rs`

Audit the integration between generic inference and record-row solving.

In particular:

- Can generic inference leave row variables unresolved?
- Can a row solution contradict an earlier generic assignment?
- Are row constraints preserved across nested callable inference?
- Is row materialization performed only after appropriate solving?
- Are open rows inadvertently treated as closed?
- Do row constraints survive expected-type propagation?

## 5.4 `expected.rs`, `expression.rs`

**Responsibility:** Expected-type propagation and expression synthesis/checking.

Investigate bidirectional typing.

Important distinction:

```text
Synthesis:
    expression -> inferred type

Checking:
    expression + expected type -> compatibility judgment
```

Audit all expression families to determine whether they support both paths where required.

Potential omissions:

- Expected type not propagated into a nested expression.
- Expected result ignored for generic constructors.
- Expected type applied too late.
- Expected context propagated where prohibited.
- A child expression is finalized before parent constraints become available.
- Expected type from an invalid ancestor is incorrectly treated as proof.

## 5.5 `call.rs`, `associated.rs`

**Responsibility:** Callable invocation, candidate resolution, and associated-member selection.

Audit:

- Positional arguments.
- Labeled arguments.
- Rest arguments.
- Generic arguments.
- Optional/default arguments where supported.
- Method overloads.
- Callable values.
- Method references.
- Family applications.
- Class-side versus instance-side dispatch.
- Inherited methods.
- Trait-provided methods.
- Conditional implementations.
- Associated callable identity.

A particularly important test category is:

```text
The correct callable is selected,
but the wrong callable identity is published.
```

This may not be visible to a local typechecker test.

The compiler may later lower the call to an incorrect executable target.

## 5.6 `binding.rs`, `flow.rs`, `field_lifecycle.rs`

**Responsibility:** Persistent binding contracts, flow-sensitive facts, and initialization.

Audit:

- Definite assignment.
- Reassignment.
- Immutable bindings.
- Mutable bindings.
- Field initialization.
- Constructor initialization.
- Branch merges.
- Loop fixed points.
- Early returns.
- Exceptions.
- Captured variables.
- Nested closures.
- Pattern bindings.
- Shadowing.
- Unreachable branches.

Important requirement:

Flow-sensitive knowledge must not be allowed to contradict the persistent type contract of a binding.

For example:

```text
Declared binding: x: A

Branch-local fact: x has type B

After the branch:
    x retains its declared binding contract.
```

Narrowing may refine what is known about the value at a program point, but must not silently redefine the persistent binding contract.

## 5.7 `pattern.rs`, `exhaustiveness.rs`, `gadt_proof.rs`

**Responsibility:** Pattern semantics, coverage analysis, and branch-local type proofs.

Investigate:

- Exhaustive enum matches.
- Missing variants.
- Nested patterns.
- Structural patterns.
- Generic variants.
- GADT constructors.
- Branch-local equalities.
- Inaccessible branches.
- Guarded patterns.
- Redundant patterns.
- Pattern-bound variable specialization.
- Branch-local proof escape.

Critical invariant:

A proof derived from matching one variant must not become available in unrelated control-flow branches.

## 5.8 `sequence_literal.rs`, `sequence_publication.rs`

Audit:

- Empty sequence inference.
- Homogeneous element inference.
- Contextual collection-family selection.
- Nested collections.
- Generic collection constructors.
- Literal type widening.
- Associated element types.
- Sequence literals in generic call arguments.
- Sequence literals under trait constraints.
- Publication of selected collection and element types.

Special attention should be paid to the interaction between canonical `Iterable` inference and generic collection specialization.

## 5.9 `static_body.rs`, `static_iteration.rs`, `proof_predicate.rs`

Audit static evaluation and proof-directed semantic analysis.

Important questions:

- Are static and runtime contexts correctly distinguished?
- Are unsupported static expressions rejected?
- Are unresolved static results represented soundly?
- Can static evaluation cause unbounded recursion?
- Do static proofs depend on runtime-only information?
- Are compiler budgets enforced?
- Can cancellation leave partially published results?

---

# 6. TRAIT, CONFORMANCE, AND EVIDENCE AUDIT

## 6.1 Relevant Modules

Prioritize inspection of:

```text
phalcom-semantic/src/impls/
phalcom-semantic/src/traits/
phalcom-semantic/src/trait_dispatch/
phalcom-semantic/src/generic_evidence/
phalcom-semantic/src/derived_conformance/
phalcom-semantic/src/associated/
phalcom-semantic/src/member_projection/
phalcom-semantic/src/hierarchy_product/
phalcom-semantic/src/dispatch/
```

The actual file and submodule layout must be enumerated before detailed investigation.

## 6.2 Conformance Resolution

Audit:

1. Direct conformance.
2. Inherited conformance.
3. Generic conformance.
4. Conditional conformance.
5. Nested conditional conformance.
6. Conformance through applied generic superclasses.
7. Associated-type requirements.
8. Default implementations.
9. Conflicting implementations.
10. Recursive conformance dependencies.
11. Conformance visibility.
12. Conformance identity across module boundaries.

## 6.3 Evidence Provenance

Every successful conformance judgment should identify the evidence establishing it.

The audit must determine whether evidence retains:

- Required trait identity.
- Target type identity.
- Generic arguments.
- Selected implementation identity.
- Associated-type bindings.
- Nested prerequisite proofs.
- Source/caller specialization.
- Relevant dependency identities.

### Example risk

A generic implementation requires:

```text
T conforms to One
T conforms to Zero
```

The enclosing callable may be selected correctly, while one or both prerequisite proofs fail to reach the compiler's invocation environment.

This exact class of problem has appeared in the project's historical mathematical implementation work.

The audit should treat that history as justification for a regression investigation, not evidence that the current implementation remains defective.

## 6.4 Derived and Conditional Evidence

Audit:

- Exact evidence.
- Derived evidence.
- Assumed evidence.
- Conditional source selection.
- Nested prerequisite evidence.
- Evidence specialization.
- Negative evidence.
- Contradictory evidence.
- Blocked evidence.

Investigate whether a derived conformance can be consumed without preserving its antecedents.

## 6.5 Trait Dispatch

For every trait-based invocation, verify:

```text
Source requirement
    ->
Resolved trait application
    ->
Selected implementation
    ->
Conformance evidence
    ->
Specialized callable
    ->
Compiler invocation specification
    ->
Runtime target
```

An audit test should compare semantic and runtime identities rather than merely checking whether the program terminates.

---

# 7. SEMANTIC DATABASE AND INCREMENTAL ANALYSIS

## 7.1 Relevant Modules

```text
phalcom-semantic/src/db/
phalcom-semantic/src/session/
phalcom-semantic/src/snapshot.rs
phalcom-semantic/src/invalidation/
phalcom-semantic/src/workspace/
phalcom-semantic/src/workspace_inputs/
phalcom-semantic/src/semantic_shard/
phalcom-semantic/src/source_index/
```

The inspected database implementation includes:

```text
budget.rs
dependency.rs
fingerprint.rs
key.rs
metrics.rs
product.rs
query.rs
scheduler.rs
state.rs
```

The semantic database maintains:

- Query states.
- Cached semantic products.
- Previous known-good products.
- Dependency indexes.
- Query scheduling.
- Semantic revisions.
- Recomputed and revalidated query sets.

These mechanisms require a dedicated audit because incremental bugs may be invisible to a conventional compile-once test suite.

## 7.2 Fundamental Incremental Equivalence Test

For a source mutation sequence:

```text
S0 -> S1 -> S2 -> ... -> Sn
```

Compare:

```text
IncrementalAnalysis(S0 ... Sn)
```

against:

```text
FreshAnalysis(Sn)
```

Require equivalence for:

- Formal declaration identities.
- Type judgments.
- Callable resolutions.
- Conformance results.
- Associated-type bindings.
- Diagnostics.
- Static evaluation products.
- Semantic lowering specifications.

Do not require byte-for-byte equality for intentionally transient internal identifiers.

Normalize outputs according to stable semantic identity.

## 7.3 Dependency Invalidation

Audit mutation categories:

| Mutation | Expected invalidation |
|---|---|
| Change callable body | Body-dependent semantic products |
| Change callable signature | Callers and relevant dependent declarations |
| Change generic constraint | Generic applications and proofs |
| Change trait implementation | Relevant conformance consumers |
| Change superclass | Hierarchy and inherited-member consumers |
| Change associated-type binding | Projection and dependent call consumers |
| Change import | Affected name-resolution products |
| Change project manifest | Affected module topology and imports |
| Remove declaration | Consumers and relevant failed lookups |
| Add declaration | Previously unresolved names and candidates |
| Change static constant | Dependent static evaluation products |
| Change field declaration | Constructor and field-access consumers |

The audit must pay particular attention to **negative dependencies**.

A query that previously failed to find a declaration must be invalidated when that declaration becomes available.

Similarly, a failed trait or member lookup must not remain cached after the relevant declaration or implementation changes.

## 7.4 Failure and Cancellation

Test cancellation during:

- Generic inference.
- Recursive type normalization.
- Conformance resolution.
- Source publication.
- Incremental invalidation.
- Static evaluation.
- Compiler lowering.

Verify that cancellation cannot publish half-constructed authoritative results.

## 7.5 Last-Known-Good Products

The database retains last-known-good semantic products.

Determine:

1. Which consumers may access these products.
2. Whether they are explicitly marked stale.
3. Whether stale products may influence formal semantic conclusions.
4. Whether editor recovery can accidentally affect compilation.
5. Whether dependency changes invalidate their authority.

Last-known-good information may be useful for editor presentation.

It must not silently substitute for current authoritative proof.

---

# 8. COMPILER AND SEMANTIC LOWERING AUDIT

## 8.1 Compiler Organization

Primary directory:

`phalcom-core/src/compiler/`

The inspected compiler module structure includes:

```text
compiler/
    attributes
    boundedness
    inliner
    lib/
        associated.rs
        class_decl.rs
        data_decl.rs
        enum_decl.rs
        error.rs
        expr.rs
        impl_decl.rs
        jumps.rs
        loops.rs
        match_expr.rs
        patterns.rs
        product_opt.rs
        scope.rs
        state.rs
```

The compiler transforms parsed source into executable bytecode and runtime objects.

The audit must distinguish between:

- Semantic decisions that must already be established.
- Backend-specific realization decisions.
- Defensive compiler checks.
- Legacy compiler-side semantic behavior.

## 8.2 Semantic-to-Lowering Bridge

Highest-priority file:

`phalcom-core/src/modules/semantic_lowering.rs`

The inspected implementation explicitly describes its purpose as bridging formal `SemanticSnapshot` products into immutable compiler-facing lowering specifications.

It defines lowering-site identities using:

```rust
pub struct LoweringSite {
    pub source: SourceId,
    pub range: SourceRange,
    pub kind: LoweringSiteKind,
}
```

This design deserves targeted verification.

### Investigation A: Site identity

Determine:

- Whether two different expressions can share the same lowering key.
- Whether nested expressions can collide.
- Whether recovered syntax ranges can be ambiguous.
- Whether generated syntax receives stable source identity.
- Whether source modifications invalidate outdated attachments.
- Whether compiler consumers request the correct site kind.

### Investigation B: Product completeness

For every lowering specification, identify its semantic origin.

Check whether the projection retains all relevant:

- Generic bindings.
- Trait proofs.
- Associated-type specializations.
- Callable identities.
- Receiver specializations.
- Constructor identities.
- Runtime type arguments.
- Conditional implementation prerequisites.

### Investigation C: Missing attachments

Determine what happens when a compiler expects a lowering specification but none exists.

Acceptable behavior must be explicitly established.

Possible outcomes include:

- Controlled compilation error.
- Valid fallback for a formally specified case.
- Internal incident with useful context.

An unverified fallback that guesses the intended callable or type is unacceptable.

### Investigation D: Duplicate authority

Search the compiler for independent implementations of:

- Type compatibility.
- Method resolution.
- Generic specialization.
- Trait selection.
- Type-member lookup.
- Associated-type resolution.

Determine whether each implementation is:

1. A permitted backend check.
2. A projection of canonical semantics.
3. Legacy duplicated semantic authority.

Category 3 requires investigation and potentially a later consolidation plan.

## 8.3 Bytecode and VM

Relevant areas:

```text
phalcom-core/src/chunk.rs
phalcom-core/src/bytecode/
phalcom-core/src/vm/
phalcom-core/src/typing/
phalcom-core/src/heap/
phalcom-core/src/value/
```

Verify actual submodule layouts before creating file-level work assignments.

Audit:

- Runtime type environments.
- Generic application representations.
- Runtime type descriptors.
- Specialized method calls.
- Trait dispatch.
- Associated family invocation.
- Variant construction.
- Data construction.
- Field access.
- Dynamic casts.
- Closure captures.
- Runtime argument packs.
- Type reflection.

Critical requirement:

The runtime implementation must not silently weaken the guarantees established by semantic analysis.

---

# 9. PANIC, CRASH, AND UNHANDLED-STATE AUDIT

## 9.1 Objective

Find every user-reachable execution path capable of terminating semantic analysis or compilation unexpectedly.

The search must cover:

```rust
panic!(...)
unreachable!(...)
todo!(...)
unimplemented!(...)
unwrap()
expect(...)
assert!(...)
assert_eq!(...)
debug_assert!(...)
```

Also investigate less obvious failure surfaces:

- Indexing operations.
- Integer conversions.
- Stack recursion.
- Interior mutability.
- Recursive graph traversal.
- Hash-map assumptions.
- RefCell borrow failures.
- Lock poisoning.
- Invalid enum-state transitions.
- Out-of-memory amplification.

## 9.2 Classify Every Panic Surface

Use the following classification:

| Category | Meaning | Required action |
|---|---|---|
| U0 | Reachable from malformed source | P0 defect |
| U1 | Reachable from valid source | P0 defect |
| U2 | Reachable through stale/inconsistent semantic products | P0/P1 investigation |
| U3 | Internal invariant assertion with established proof | Document and test invariant |
| U4 | Test-only panic | Exclude from production risk |
| U5 | Startup/bootstrap invariant | Verify configuration and input reachability |

Do not assume every `unwrap()` is defective.

Do not assume every panic is unreachable merely because it is described as an invariant.

## 9.3 Semantic Incident Infrastructure

The inspected repository contains:

`phalcom-semantic/src/checker/incident.rs`

It defines structured internal incidents with categories including:

- Flow invariant violations.
- Relation invariant violations.
- Inference invariant violations.
- Identity invariant violations.
- Database invariant violations.

It also defines containment and fail-fast policies.

The audit should investigate whether this infrastructure is consistently used.

Questions:

1. Are internal failures recorded before analysis exits?
2. Can affected queries be contained safely?
3. Are unrelated products preserved?
4. Are incident identifiers deterministic?
5. Can an incident be incorrectly converted into an ordinary source diagnostic?
6. Can an incident accidentally disappear during snapshot publication?
7. Is fail-fast mode restricted to appropriate developer/test contexts?

## 9.4 Fuzzing Requirements

Use parser-aware generation and mutation.

Generate:

- Nested generics.
- Recursive type aliases.
- Deeply nested records.
- Unresolved names.
- Contradictory constraints.
- Invalid kind applications.
- Cyclic inheritance.
- Nested associated projections.
- Malformed patterns.
- Duplicate declarations.
- Incomplete callable declarations.
- Incorrectly labeled calls.
- Large unions.
- Deep subtype hierarchies.

Every fuzz input should satisfy at least:

```text
Analyzer must not panic.
Compiler must not panic.
Analysis must terminate within configured limits.
Diagnostics must remain structurally valid.
```

For successfully checked programs, additional differential execution tests are required.

---

# 10. HIGH-PRIORITY CROSS-FEATURE TEST MATRIX

A major weakness in traditional typechecker testing is that features are tested independently.

Many difficult bugs occur only when individually correct mechanisms are composed.

The audit should therefore prioritize the following interaction matrix.

| Feature A | Feature B | Required investigation |
|---|---|---|
| Generic inference | Inheritance | Applied superclass specialization |
| Generic inference | Variants | Contextual constructor specialization |
| Generic inference | Collections | Expected element and family propagation |
| Generic inference | Associated types | Solving through projection constraints |
| Generic inference | Callable values | Higher-order inference |
| Generic inference | Structural records | Row-variable interactions |
| Generic inference | Static arguments | Kind and domain consistency |
| Generic inference | Overloading | Candidate-specific inference isolation |
| Traits | Inheritance | Inherited conformance and member identity |
| Traits | Associated types | Evidence-based projection resolution |
| Traits | Conditional implementations | Antecedent proof transport |
| Traits | Method families | Callable identity preservation |
| GADTs | Pattern matching | Branch-local equalities |
| GADTs | Generic inference | Constructor result specialization |
| GADTs | Exhaustiveness | Reachable constructor sets |
| Flow analysis | Closures | Captured mutable bindings |
| Flow analysis | Loops | Fixed-point correctness |
| Flow analysis | Pattern matching | Narrowing and branch merge |
| Flow analysis | Constructors | Field initialization proofs |
| Structural types | Subtyping | Width/depth/variance laws |
| Structural types | Generics | Row inference and materialization |
| Static evaluation | Type members | Reduction and projection normalization |
| Static evaluation | Generic constraints | Proof-dependent type computation |
| Module imports | Conformance | Cross-module evidence lookup |
| Incremental DB | Generic inference | Invalidation of dependent solutions |
| Incremental DB | Trait implementations | Negative proof invalidation |
| Compiler lowering | Trait dispatch | Evidence-to-runtime consistency |
| Compiler lowering | Generic families | Callable and type-binding consistency |
| Runtime typing | Static generics | Specialization fidelity |

Each entry should receive positive, negative, and incremental regression cases.

---

# 11. SPECIFICATION CONFORMANCE AUDIT

## 11.1 Authority

The repository establishes `docs/specs/` as the authoritative specification corpus.

Historical implementation plans and checkpoints are useful evidence, but they do not override accepted language semantics.

The audit must maintain three distinct states:

```text
Specified behavior
Implemented behavior
Observed behavior
```

Disagreement between these states must be recorded explicitly.

## 11.2 Priority Specifications

Begin with:

```text
docs/specs/README.md

docs/specs/generic-constraints.md
docs/specs/associated-types.md
docs/specs/type-level-domains.md
docs/specs/type-members.md
docs/specs/structural-products.md
docs/specs/structural-sums.md
docs/specs/tuple-and-record-typing.md
docs/specs/sequence-literals.md
docs/specs/static-evaluation.md
docs/specs/bindings-and-fields.md
```

Also investigate:

```text
docs/specs/typing/
docs/specs/objects/
docs/specs/callables/
docs/specs/vm/
```

## 11.3 Traceability Requirements

Every audited semantic feature should receive a traceability record:

| Field | Description |
|---|---|
| Feature | Semantic capability |
| Specification | Authoritative rule |
| Implementation | Owning function/module |
| Consumers | Compiler/runtime/editor dependents |
| Tests | Existing verification |
| Missing cases | Identified coverage gaps |
| Risk | Failure impact |
| Evidence | Source references and reproductions |
| Status | Confirmed, suspected, or cleared |

No feature should be marked compliant based solely on the existence of implementation code.

---

# 12. REPOSITORY-WIDE INVESTIGATION PROCEDURE

## 12.1 Establish Baseline

Before investigating individual defects:

```bash
git status --short
git branch --show-current
git rev-parse HEAD

cargo metadata --format-version 1 --no-deps
cargo check --workspace --all-targets
cargo test --workspace
```

Record:

- Exact commit SHA.
- Rust toolchain version.
- Enabled features.
- Platform.
- Test failures.
- Known baseline incidents.
- Worktree state.

Do not modify or clean unrelated work.

## 12.2 Establish File Ownership

Generate a code ownership inventory using the live repository:

```bash
find phalcom-semantic/src -type f -name '*.rs' | sort
find phalcom-core/src -type f -name '*.rs' | sort
find phalcom-modules/src -type f -name '*.rs' | sort
```

For each file, record:

- Functional responsibility.
- Public interfaces.
- Internal dependencies.
- Callers.
- Semantic authority.
- Error paths.
- Invariants.
- Test coverage.

Where a module is too large for one audit task, divide it by public entry points and internal responsibility rather than arbitrary line ranges.

## 12.3 Search for Failure Surfaces

```bash
rg -n \
  'panic!|unreachable!|todo!|unimplemented!|\.unwrap\(\)|\.expect\(|assert!' \
  phalcom-semantic/src \
  phalcom-core/src \
  phalcom-modules/src

rg -n \
  'unsafe\s*\{|get_unchecked|transmute|from_raw_parts' \
  phalcom-semantic/src \
  phalcom-core/src \
  phalcom-modules/src
```

Classify findings rather than automatically reporting all matches as defects.

## 12.4 Identify Semantic Authority Duplication

Search compiler and runtime code for type-system operations.

Example investigation terms:

```text
subtype
assignable
conformance
specialize
infer
resolve_method
resolve_member
type_relation
trait_dispatch
associated
projection
```

For each occurrence, determine whether it consumes a semantic product or independently derives a semantic judgment.

## 12.5 Construct Call Graphs

For major entry points, document:

```text
Entry point
    ->
Semantic query
    ->
Owning algorithm
    ->
Evidence/result
    ->
Publication
    ->
Consumer
```

Required entry-point families:

1. Whole-program checking.
2. Callable body checking.
3. Expression inference.
4. Generic application.
5. Trait resolution.
6. Associated-type projection.
7. Pattern matching.
8. Static evaluation.
9. Semantic snapshot publication.
10. Compiler lowering.
11. VM execution.
12. Incremental workspace update.
13. LSP semantic queries.

## 12.6 Review Historical Incidents

Inspect implementation records relevant to:

```text
LANG005
LANG007
COLL006
```

Prioritize documented incidents involving:

- Canonical Iterable inference.
- Generic variant inference.
- Applied superclass resolution.
- Associated callable identity.
- Conditional source realizations.
- Missing nested conformance evidence.
- Static type-member evaluation.
- Semantic/lowering disagreement.

The purpose is to identify recurring failure mechanisms and determine whether the underlying architectural issue is comprehensively covered by regressions.

Do not reopen resolved issues without current evidence.

---

# 13. TESTING STRATEGY

## 13.1 Testing Levels

The audit must assess five distinct test layers.

### Level 1 — Unit tests

Test individual algorithms:

- Subtyping.
- Substitution.
- Constraint solving.
- Row unification.
- Kind validation.
- Type normalization.

### Level 2 — Semantic integration tests

Parse source and verify canonical typing decisions.

Test both successful and unsuccessful programs.

### Level 3 — Compiler integration tests

Verify that canonical semantic products are transformed into correct lowering specifications.

### Level 4 — End-to-end execution tests

Compile and execute valid source.

Compare runtime outcomes with the specified semantics.

### Level 5 — Incremental and adversarial tests

Test mutation sequences, malformed source, cancellation, deep recursion, and resource limits.

All five layers are necessary.

A unit test passing in `phalcom-semantic` does not establish the correctness of executable behavior.

## 13.2 Property-Based Testing

Prioritize properties that can be stated independently of implementation details.

Examples:

```text
Subtype reflexivity

Subtype transitivity

Substitution identity

Substitution composition

Type normalization idempotence

Canonicalization idempotence

Equivalent constraint permutation consistency

Cold/incremental analysis equivalence

Semantic/lowering identity agreement
```

Use valid generated type terms rather than unrestricted random IDs.

Property generators must respect the domain of each operation.

## 13.3 Metamorphic Testing

Construct semantics-preserving transformations.

Examples:

- Alpha-rename a local binding.
- Rename a generic parameter.
- Insert irrelevant declarations.
- Reorder independent declarations.
- Replace an inferred generic argument with its explicitly resolved form.
- Extract an expression into a typed local.
- Add explicit type annotations equivalent to inferred types.
- Replace a direct generic use with an equivalent type alias where supported.

Expected result:

The relevant typing judgment and runtime semantics remain equivalent.

## 13.4 Differential Testing

Compare:

```text
Fresh semantic analysis
vs.
Incremental semantic analysis
```

```text
Semantic callable selection
vs.
Compiler lowering target
```

```text
Compiler-generated runtime specialization
vs.
Canonical semantic specialization
```

```text
CLI semantic diagnostics
vs.
LSP semantic diagnostics
```

These comparisons should use normalized semantic identities.

## 13.5 Regression Test Requirements

Every confirmed defect must produce:

1. Minimal reproducer.
2. Failing test before correction.
3. Root-cause explanation.
4. Test at the owning layer.
5. Relevant cross-layer regression.
6. Negative companion test.
7. Verification after correction.

Avoid fixes verified only by the original large program.

---

# 14. PRIORITY-RANKED AUDIT WORKSTREAMS

## P0-A — Panic and Internal Invariant Audit

**Objective:** Establish whether source input can crash the analyzer or compiler.

**Primary owners:**

- `phalcom-semantic/src/checker/`
- `phalcom-semantic/src/types/`
- `phalcom-semantic/src/db/`
- `phalcom-core/src/compiler/`
- `phalcom-core/src/modules/semantic_lowering.rs`

**Tasks:**

1. Enumerate production panic surfaces.
2. Establish source reachability.
3. Investigate invariant preconditions.
4. Generate malformed-source probes.
5. Test cancellation and bounded execution.
6. Identify silent internal-failure recovery.
7. Verify structured incident publication.

**Gate:** All identified user-reachable crashes are reproduced or formally ruled out.

## P0-B — Canonical Type Relations

**Objective:** Verify foundational type relations and canonical type formation.

**Primary owners:**

- `types/store.rs`
- `types/relation.rs`
- `types/application.rs`
- `types/variance.rs`
- `types/substitution.rs`

**Tasks:**

1. Audit type identity.
2. Audit subtyping laws.
3. Audit assignability.
4. Audit generic variance.
5. Audit structural compatibility.
6. Audit exact variant relationships.
7. Audit recursive relation termination.

**Gate:** No unexplained discrepancy between specified and implemented type relations.

## P0-C — Generic Inference and Solver

**Objective:** Verify inference soundness, completeness, and termination.

**Primary owners:**

- `checker/inference.rs`
- `checker/row_inference.rs`
- `checker/expected.rs`
- `checker/expression.rs`
- `types/row_solver.rs`

**Tasks:**

1. Reconstruct solver state machine.
2. Verify variable ownership.
3. Audit occurs checks.
4. Audit rigid variables.
5. Audit expected-type propagation.
6. Audit associated projections.
7. Audit materialization.
8. Test contradictory and cyclic constraints.
9. Test candidate-order independence.

**Gate:** All supported inference-term categories have positive, negative, and failure-path coverage.

## P0-D — Conformance and Evidence

**Objective:** Verify that trait and implementation evidence is sound and complete.

**Primary owners:**

- `impls/`
- `traits/`
- `trait_dispatch/`
- `generic_evidence/`
- `derived_conformance/`

**Tasks:**

1. Map conformance resolution.
2. Audit inherited and conditional evidence.
3. Audit associated-type bindings.
4. Audit nested prerequisite proofs.
5. Audit negative dependency tracking.
6. Audit evidence specialization.
7. Audit dispatch selection.

**Gate:** Every successful trait invocation is traceable to the correct evidence and executable target.

## P0-E — Semantic Lowering Consistency

**Objective:** Verify preservation of semantic decisions during compilation.

**Primary owners:**

- `phalcom-semantic/src/snapshot.rs`
- `phalcom-core/src/modules/semantic_lowering.rs`
- `phalcom-core/src/compiler/lib/`

**Tasks:**

1. Inventory lowering-site kinds.
2. Trace source identity.
3. Verify product attachment.
4. Audit missing-product behavior.
5. Audit generic call environments.
6. Audit associated callable identity.
7. Audit runtime target selection.
8. Search for duplicate compiler-side semantics.

**Gate:** Every compiler lowering path consuming formal semantic information has verified provenance and consistency tests.

## P0-F — Incremental Semantic Integrity

**Objective:** Establish cold/incremental equivalence.

**Primary owners:**

- `db/`
- `snapshot.rs`
- `invalidation/`
- `session/`
- `workspace/`

**Tasks:**

1. Audit query fingerprints.
2. Audit dependency recording.
3. Audit negative dependencies.
4. Audit revision transitions.
5. Audit cancellation.
6. Audit last-known-good behavior.
7. Run differential mutation tests.
8. Audit deterministic publication.

**Gate:** Representative source-edit sequences produce equivalent fresh and incremental formal results.

---

## P1-A — Flow, Bindings, and Constructors

Audit definite assignment, field initialization, branch merges, narrowing, closure captures, and constructor lifecycle proofs.

## P1-B — Callable and Overload Semantics

Audit overload ranking, argument labels, generic specialization, method references, callable values, and family identity.

## P1-C — Algebraic Types and Pattern Matching

Audit enum/data declarations, GADT branch proofs, pattern-bound types, and exhaustiveness.

## P1-D — Static Evaluation and Type-Level Programming

Audit kinded static domains, type members, const-related semantics, normalization, budgets, and cyclic evaluation.

## P1-E — Module and Cross-Project Semantics

Audit namespace ownership, imported generic declarations, inherited implementations, stable identities, and dependency invalidation.

## P1-F — Runtime Typing and Dispatch

Audit generic runtime descriptors, type-environment recipes, specialized call execution, and dynamic boundaries.

---

## P2-A — Diagnostics and Recovery

Audit diagnostic quality, duplicate suppression, source ranges, internal incident distinction, and invalid-source recovery.

## P2-B — LSP and Editor Consistency

Audit semantic snapshots consumed by hover, completion, definition, references, and diagnostics.

## P2-C — Performance and Complexity

Audit pathological solver complexity, deeply nested type expressions, recursive graphs, and excessive invalidation.

## P2-D — Metadata Serialization and Compatibility

Audit stable identities, metadata validation, cross-version encoding, and reconstruction of semantic types and generic contexts.

---

# 15. AUDIT EXECUTION ORDER

The investigation should proceed in phases.

## Phase 0 — Baseline and Architecture Inventory

Deliver:

- Repository revision record.
- Workspace/crate inventory.
- Source ownership map.
- Semantic execution call graph.
- Existing test inventory.
- Historical incident index.
- Preliminary risk register.

## Phase 1 — Crash and Failure-Surface Census

Deliver:

- Panic inventory.
- Source reachability classification.
- Internal incident assessment.
- Resource-exhaustion probes.
- Confirmed crash reproductions.

## Phase 2 — Canonical Type System

Deliver:

- Type representation audit.
- Subtyping and assignability audit.
- Kind and application audit.
- Substitution and variance audit.
- Property-based regression candidates.

## Phase 3 — Inference and Constraint Solving

Deliver:

- Solver architecture diagram.
- Constraint propagation map.
- Inference completeness matrix.
- Cyclic/contradictory constraint probes.
- Generic specialization findings.

## Phase 4 — Traits, Evidence, and Dispatch

Deliver:

- Conformance proof-flow diagram.
- Associated-type specialization audit.
- Conditional evidence audit.
- Callable identity audit.
- Nested proof-transport regressions.

## Phase 5 — Checker and Flow Semantics

Deliver:

- Expression-category coverage matrix.
- Statement/control-flow coverage matrix.
- Constructor initialization findings.
- Pattern/GADT correctness assessment.
- Diagnostic recovery findings.

## Phase 6 — Compiler Lowering and Runtime

Deliver:

- Semantic-to-lowering traceability matrix.
- Missing or inconsistent product findings.
- Runtime specialization audit.
- End-to-end semantic fidelity regressions.

## Phase 7 — Incremental and Tooling Consistency

Deliver:

- Dependency invalidation assessment.
- Cold/incremental differential results.
- Cancellation tests.
- Snapshot freshness analysis.
- LSP/CLI consistency findings.

## Phase 8 — Consolidation and Remediation Planning

Deliver:

- Confirmed defect register.
- Root-cause clusters.
- Architecture-level corrective recommendations.
- Priority-ranked implementation backlog.
- Regression coverage requirements.
- Release-blocking findings.
- Deferred investigation register.

The phases may share investigation resources, but foundational semantic conclusions should precede consumer-specific fixes.

---

# 16. DEFECT REPORTING REQUIREMENTS

Every finding must follow a consistent structure.

```text
Finding ID:
Severity:
Confidence:
Status:

Subsystem:
Owning file:
Owning function:
Affected consumers:

Specification:
Expected behavior:
Actual behavior:

Minimal reproducer:
Reproduction command:
Observed output:
Expected output:

Root cause:
Violated invariant:
Failure classification:

Affected feature combinations:
Related historical incidents:

Recommended correction layer:
Required regression tests:
Potential collateral effects:

Verification status:
```

## 16.1 Severity Model

### Critical / P0

- Soundness violation.
- User-reachable compiler/analyzer panic.
- Incorrect executable behavior after successful semantic checking.
- Incorrectly established conformance proof.
- Cross-workspace or cross-revision semantic contamination.
- Silent conversion of failed analysis into successful proof.

### High / P1

- Valid program rejected.
- Incorrect generic specialization.
- Incorrect branch-local type reasoning.
- Major incremental inconsistency.
- Unbounded semantic analysis on ordinary source.
- Incorrect callable or associated-member selection.

### Medium / P2

- Diagnostic inconsistencies.
- Poor error recovery.
- Missing invalidation causing editor-only stale presentation.
- Pathological but bounded performance.
- Missing regression coverage for high-risk features.

### Low / P3

- Noncritical duplication.
- Maintainability concerns.
- Unclear invariants.
- Missing documentation.
- Low-risk API inconsistencies.

Severity and confidence must remain separate.

A severe hypothetical failure is not a confirmed P0 defect without evidence.

---

# 17. ARCHITECTURAL ANTI-PATTERNS TO INVESTIGATE

## 17.1 Duplicate Semantic Authority

Example pattern:

```text
Semantic analyzer resolves a callable.
Compiler independently resolves the callable again.
```

Risk:

The two resolutions can disagree.

Preferred architecture:

```text
Canonical semantic resolution
    ->
Stable resolution product
    ->
Compiler realization
```

## 17.2 Silent Fallback to Weaker Typing

Example:

```text
Cannot prove A <: B
    ->
Accept because relation was inconclusive
```

Risk:

Unknown becomes success.

All fallback paths must be classified and justified.

## 17.3 Incomplete Generic Environments

Example:

```text
Callable specialization contains T = Int
but loses the proof that Int conforms to RequiredTrait.
```

Risk:

The compiler realizes an invocation without the required semantic environment.

## 17.4 Consumer-Specific Inference Fixes

Example:

```text
Collection literal cannot infer T.
Compiler injects a special-case inference rule.
```

Risk:

A second inference authority emerges.

The correct fix should normally occur in the canonical inference implementation.

## 17.5 Incomplete Incremental Dependencies

Example:

```text
Generic inference depends on an associated type.

Associated binding changes.

Generic inference product is not invalidated.
```

Risk:

Stale semantic products persist.

## 17.6 Unverified Internal Assumptions

Example:

```rust
let selected = candidates.first().unwrap();
```

The audit must determine whether an empty candidate set is structurally impossible.

A nearby assertion or explanatory comment is not sufficient evidence by itself.

## 17.7 Lossy Semantic Projection

Example:

```text
Canonical resolution:
    implementation identity
    caller specialization
    nested proof environment

Compiler projection:
    implementation identity only
```

Risk:

The executable selection lacks information required for correct specialization.

## 17.8 Inconsistent Error Recovery

Example:

```text
Earlier expression contains invalid type information.

Later expression consumes a partially initialized checker state.

Later expression panics or receives a fabricated type.
```

Risk:

An ordinary source error causes unrelated analyzer corruption.

---

# 18. SPECIAL INVESTIGATION: RECURRENT TELOS FAILURE PATTERNS

Historical implementation work suggests several particularly valuable regression themes.

These should receive concentrated investigation.

## 18.1 Generic Result Constructor Inference

Test:

- Result-like generic variants.
- Expected return type supplied by enclosing declaration.
- Generic error subclasses.
- Contextually constructed error values.
- Nested generic enum constructors.
- Multiple constructor candidates.

Determine whether expected result evidence reaches payload analysis early enough.

## 18.2 Generic Superclass Resolution

Test:

```text
Derived<T> inherits Base<T>
```

with:

- Inherited methods.
- Inherited fields.
- Generic substitutions.
- Trait implementations.
- Static/class members.
- Multiple inheritance levels.

Verify that all consumers use the same canonical superclass relation.

## 18.3 Iterable and Associated Item Inference

Test generic collection values under:

- Iterable constraints.
- Associated Item equalities.
- Expected collection types.
- Nested generic arguments.
- Method return types.
- Generic callable results.

## 18.4 Associated Family Callable Identity

Test:

- Captured family values.
- Generic family applications.
- Associated method references.
- Trait-provided callable families.
- Runtime invocation through captured callable values.

Ensure semantic selection and executable identity agree.

## 18.5 Conditional Source Evidence Projection

Test nested prerequisite proofs and type-binding transport.

Ensure compiler invocation recipes preserve the complete evidence required by the selected implementation.

## 18.6 Static Type-Level Evaluation

Test:

- Recursive type members.
- Static generic parameters.
- Nested type applications.
- Projection normalization.
- Residualized static computations.
- Budget exhaustion.
- Invalid static-domain operations.

These investigations should be used to construct generalized regressions rather than repeating only the historical failing examples.

---

# 19. AUDIT ACCEPTANCE CRITERIA

The audit is complete only when the following conditions are satisfied.

## Architecture

- [ ] Every relevant workspace crate is mapped.
- [ ] Primary type-system modules have documented responsibilities.
- [ ] Authoritative semantic entry points are identified.
- [ ] Compiler consumers of semantic products are identified.
- [ ] Cross-layer semantic data flows are documented.

## Foundational Correctness

- [ ] Canonical type representation is audited.
- [ ] Subtyping and assignability are audited.
- [ ] Generic application and kinding are audited.
- [ ] Substitution and variance are audited.
- [ ] Structural typing and row inference are audited.
- [ ] Type-member normalization is audited.

## Inference

- [ ] Inference-variable lifecycle is verified.
- [ ] Occurs checks are audited.
- [ ] Expected-type propagation is covered.
- [ ] Generic constructor inference is covered.
- [ ] Associated-type inference is covered.
- [ ] Solver termination is tested.
- [ ] Materialization is verified.

## Traits and Evidence

- [ ] Direct conformance is audited.
- [ ] Inherited conformance is audited.
- [ ] Conditional conformance is audited.
- [ ] Nested evidence preservation is verified.
- [ ] Associated-type specialization is verified.
- [ ] Callable identity preservation is verified.

## Robustness

- [ ] Panic census is completed.
- [ ] User-reachable panic paths are classified.
- [ ] Malformed-source fuzzing is executed.
- [ ] Cancellation behavior is tested.
- [ ] Budget exhaustion is tested.
- [ ] Internal incident containment is verified.

## Compiler

- [ ] Semantic lowering products are inventoried.
- [ ] Lowering-site identities are audited.
- [ ] Missing-product behavior is verified.
- [ ] Generic call environments are audited.
- [ ] Runtime dispatch identities are verified.
- [ ] Cross-layer semantic consistency tests exist.

## Incremental Analysis

- [ ] Dependency tracking is audited.
- [ ] Negative dependencies are audited.
- [ ] Cold/incremental equivalence is tested.
- [ ] Snapshot freshness is verified.
- [ ] Last-known-good behavior is audited.
- [ ] Cancellation cannot publish partial authority.

## Final Reporting

- [ ] Every confirmed issue has a minimal reproducer.
- [ ] Each issue identifies its owning implementation layer.
- [ ] Findings distinguish bugs from coverage gaps.
- [ ] Findings distinguish current defects from historical incidents.
- [ ] Root-cause clusters are identified.
- [ ] Remediation work is prioritized.
- [ ] Residual risks are explicitly documented.

---

# 20. REQUIRED FINAL DELIVERABLES

The audit should produce the following durable records.

## D1 — Repository Architecture Atlas

Crate ownership, file inventory, semantic entry points, dependency relationships, and execution diagrams.

## D2 — Type System Correctness Report

Detailed findings covering canonical types, relations, generics, inference, kinding, substitution, and structural typing.

## D3 — Semantic Analyzer Reliability Report

Panics, internal invariant violations, error recovery, cancellation, resource bounds, and invalid-source robustness.

## D4 — Conformance and Evidence Integrity Report

Trait resolution, associated types, generic implementation selection, conditional evidence, and proof transport.

## D5 — Compiler Semantic Fidelity Report

Semantic snapshot projection, executable lowering, runtime type environments, and dispatch identity preservation.

## D6 — Incremental Consistency Report

Query dependency tracking, invalidation, fingerprints, snapshots, cold/incremental equivalence, and tooling consistency.

## D7 — Coverage Gap Matrix

A feature-by-feature assessment of existing tests, missing interactions, untested failure modes, and required new regressions.

## D8 — Consolidated Defect Register

Confirmed defects ranked by severity, confidence, impact, and architectural ownership.

## D9 — Remediation Roadmap

A dependency-ordered implementation backlog that addresses foundational problems before consumer-specific symptoms.

---

# 21. FINAL RECOMMENDATION

This investigation should be treated as a compiler-wide semantic integrity audit, not merely a review of `phalcom-semantic/src/checker`.

The most important question is not whether the inference solver, subtyping implementation, trait resolver, or compiler works independently.

The important question is whether they agree on one authoritative semantic interpretation and preserve that interpretation through every subsequent phase.

The recommended execution strategy is:

1. Establish the exact repository architecture and baseline.
2. Inventory crashes and semantic invariant failures.
3. Audit canonical type representations and relations.
4. Audit generic inference and constraint solving.
5. Audit conformance, associated types, and evidence provenance.
6. Audit semantic snapshots and compiler lowering.
7. Audit flow, callable, algebraic, and static semantics.
8. Verify cold/incremental equivalence.
9. Construct cross-feature and adversarial regressions.
10. Consolidate findings into root-cause-driven remediation plans.

Do not begin by applying scattered fixes to individual failing examples.

First identify the authoritative semantic subsystem responsible for each failure, establish the violated invariant, and determine whether related failures share the same architectural cause.

The intended outcome is not simply fewer compiler bugs.

It is a type-system implementation whose semantic judgments are:

- Sound.
- Complete for the supported language.
- Deterministic.
- Explicitly evidenced.
- Resistant to malformed input.
- Consistent across compilation phases.
- Stable under incremental updates.
- Faithfully realized at runtime.
- Continuously protected by targeted regression and property-based testing.

That should be the acceptance standard for Telos's compiler and type-system audit.