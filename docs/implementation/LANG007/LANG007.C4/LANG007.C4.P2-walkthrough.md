---
id: LANG007.C4.P2-walkthrough
kind: walkthrough
plan: LANG007.C4.P2
checkpoint: LANG007.C4
status: IN_PROGRESS
completion: PARTIAL
verification: UNVERIFIED
created: 2026-10-04
---

# LANG007.C4.P2 — Walkthrough (Partial, T0–T6 Complete, T7 In Progress)

This walkthrough records what actually shipped through the initial T7
evaluator slice. It will be extended as later tasks complete. When
P2 is fully done this header must be updated to
`status: COMPLETE`, `completion: IMPLEMENTED`, and an appropriate
`verification` classification.

---

## 1. Plan identity and state at interruption

**Plan**: LANG007.C4.P2 — Static Value Model, Static Definitions, Evaluator Foundation, and Closed `const`
**Status**: IN_PROGRESS
**Completion**: PARTIAL (T0–T7 implemented and focused-tested; T8–T11 pending)
**Verification**: UNVERIFIED for P2 overall (focused T7 evaluator foundation tests pass; no evaluator/materialization vertical gate yet)

---

## 2. Revision

```text
branch:  main
started: c2a938c7c195b1098891ecf0cd6940e89e32ee17   (P2 plan baseline)
current: 71bde0270af06f67098deb00af276d985d3b7278 (uncommitted shared worktree)
```

No merge or reset has occurred. The shared worktree contains P2 changes and
pre-existing adjacent work as uncommitted modifications and untracked files.

---

## 3. Architecture actually landed

### T0 — Takeover state established

- Confirmed P1 administrative closure (LANG007.C4.P1 complete, walkthrough/handoff present).
- Confirmed checkpoint named P2 as active.
- Census of stale `type X = ...` result-before-generics syntax: 0 occurrences.
- Plan initialized to IN_PROGRESS.

### T1 — Shared static domain model (phalcom-semantic/src/static_domain.rs)

New file `phalcom-semantic/src/static_domain.rs` introduces:

```rust
pub enum StaticValueDomain {
    Kind(KindId),
    ValueType(TypeId),
}
impl StaticValueDomain { is_kind(), is_value_type(), as_kind(), as_value_type(), validate(), to_type_member_domain(), from_type_member_domain() }

pub fn resolve_static_domain(store, declarations, resolver, site, syntax, diagnostics) -> TypeFormationOutcome<StaticValueDomain>
pub fn validate_proper_type(store, declarations, resolver, site, ann, diagnostics) -> TypeFormationOutcome<TypeId>
```

Adapter methods added to `TypeMemberDomain` in `phalcom-semantic/src/types/type_member.rs`.

AST carrier:

```rust
// phalcom-ast/src/ast.rs
pub enum StaticDomainSyntax {
    ProperType,
    Named(TypeAnnotation),
}
```

Focused test suite: `phalcom-semantic/tests/semantic/foundations/static_domains.rs` — **6/6 passed**.

### T2 — Generalized type-definition AST/grammar

Changed `phalcom-ast/src/ast.rs`:

```rust
// Renamed: TypeDefinition -> TypeDefinitionDef
// Added: StaticParameterSyntax (name, domain: Option<StaticDomainSyntax>, range)
// Added: StaticDefinitionBody::{Expression(Box<Expr>), Block(LexicalBlock)}
// Changed: TypeDefinitionDef.parameters: Vec<TypeParameterSyntax> -> Vec<StaticParameterSyntax>
// Added: TypeDefinitionDef.result_domain: Option<StaticDomainSyntax>
// Added: TypeDefinitionDef.body: StaticDefinitionBody  (replaces direct TypeAnnotation body)
// Added: GenericArgumentSyntax::Value(Expr)
```

Parser in `phalcom-ast/src/parser.rs`: `type Name<P: Domain>: Result = body` grammar,
falling back to `type Name = body` for legacy aliases.

All downstream match consumers updated across:
- `phalcom-semantic/src/checker/body.rs`
- `phalcom-semantic/src/checker/flow/graph.rs`
- `phalcom-semantic/src/source_index/builder.rs`
- `phalcom-semantic/src/source_index/occurrence.rs`
- `phalcom-semantic/src/types/annotation.rs`
- `phalcom-semantic/src/types/type_member.rs`
- `phalcom-semantic/src/semantic_shard.rs`
- `phalcom-core/src/compiler/attributes.rs`
- `phalcom-core/src/compiler/lib/mod.rs`
- `phalcom-core/src/compiler/lib/product_opt.rs`
- `phalcom-core/src/modules/semantic_lowering.rs`
- `phalcom-lsp/src/completion.rs`
- `phalcom-lsp/src/semantic_tokens.rs`
- `phalcom-modules/src/interface.rs`
- `phalcom-semantic/tests/semantic/adts/exact_cases.rs`
- `phalcom-semantic/tests/semantic/incremental/t12_6.rs`

Focused test suite: **288/288 AST tests passed**.

### T3 — StaticDefinition signatures/products + alias projection

New file `phalcom-semantic/src/static_definition.rs`:

```rust
pub struct StaticParameter {
    pub name: String,
    pub domain: StaticValueDomain,
    pub index: u32,
    pub type_parameter_id: Option<TypeParameterId>,   // Some only for Kind parameters
    pub range: SourceRange,
}

pub struct StaticDefinitionSignature {
    pub parameters: Box<[StaticParameter]>,
    pub result_domain: StaticValueDomain,
    pub generic_signature: Option<GenericSignature>,  // kinded lane only
    pub where_clause: Option<WhereClauseSyntax>,
}

pub enum StaticDefinitionBodyRepr {
    TypeForm(TypeAnnotation),
    Expression(Expr),
    Block(LexicalBlock),
}

pub struct StaticDefinition {
    pub id: DeclarationId,
    pub signature: StaticDefinitionSignature,
    pub body: StaticDefinitionBodyRepr,
    pub source_range: SourceRange,
}

pub struct StaticDefinitionProduct {
    pub definition: Arc<StaticDefinition>,
    pub diagnostics: Arc<[SemanticDiagnostic]>,
    pub alias_info: Option<Arc<TypeAliasInfo>>,   // Some for Type-valued kinded-only definitions
}

pub struct StaticDefinitionTable { ... }

pub fn resolve_static_definition(
    store, declarations, resolver, site, def, module_id, dependencies, diagnostics
) -> StaticDefinitionProduct
```

Key decisions made:
- `TypeParameterId` allocated **only** for kinded (`StaticValueDomain::Kind`) parameters.
- Ordinary value parameters (`Int`, `Bool`, etc.) get `type_parameter_id: None`; they do not enter TypeStore parameter identity.
- Where-clause constraints mentioning value parameters are diagnosed with `DiagnosticCode::AnnotationUnsupported` and filtered out before `resolve_generic_signature` to avoid cascading errors.
- `TypeAliasInfo` is projected only when: result domain is `Kind(...)`, body is `TypeForm`, and all parameters are kinded.
- For type lambda bodies, `alias_info.kind` stores the full constructor arrow kind (for C3 alias normalization), while the domain check compares `expected_kind` against `lambda.result_kind`.

Updated `phalcom-semantic/src/db/key.rs`:
```rust
QueryKey::StaticDefinition(DeclarationId)
```

Updated `phalcom-semantic/src/db/product.rs`:
```rust
SemanticProduct::StaticDefinition(Arc<StaticDefinitionProduct>)
fn as_static_definition(&self) -> Option<&Arc<StaticDefinitionProduct>>
```

Updated `phalcom-semantic/src/db/fingerprint.rs`:
- `static_definition_input_fingerprint`
- `static_definition_product_fingerprint`

Updated `phalcom-semantic/src/db/query.rs`:
- `query_static_definition(db, product)`

Updated `phalcom-semantic/src/db/mod.rs` + `phalcom-semantic/src/session.rs`:
- `query_key_module` and `query_key_module_for_worklist` route `StaticDefinition` keys.

Updated `phalcom-semantic/src/snapshot.rs`:
- `pub static_definitions: Arc<StaticDefinitionTable>` field
- `with_static_definitions(...)` builder method
- `static_definitions()` / `static_definition(...)` accessors

Updated `phalcom-semantic/src/session.rs`:
- `recompute_structural_shards` tracks `static_definitions_table`
- Alias resolution loop replaced with unified `resolve_static_definition` + `query_static_definition`; projects to `type_aliases` when `alias_info` is present
- Snapshot receives `with_static_definitions(static_definitions_table)`

Updated `phalcom-semantic/src/lib.rs`: re-exports `static_definition` module.

Focused test suites:
- `phalcom-semantic/tests/semantic/foundations/static_definitions.rs` — **5/5 passed**
- `phalcom-semantic/tests/semantic/foundations/static_domains.rs` (regression) — **6/6 passed**
- `phalcom-semantic/tests/semantic/foundations/type_annotations.rs` (regression) — **61/61 passed**
- `phalcom-semantic/tests/semantic/incremental::db` (regression) — **14/14 passed**
- `cargo check -p phalcom-semantic` — clean (0 errors)
- `cargo check -p phalcom-core` — clean (0 errors)

---

## 4. Changed components summary

| Component | Change |
|---|---|
| `phalcom-ast/src/ast.rs` | `TypeDefinitionDef`, `StaticParameterSyntax`, `StaticDefinitionBody`, `GenericArgumentSyntax::Value`, `StaticDomainSyntax` |
| `phalcom-ast/src/parser.rs` | `type Name<P: Domain>: Result = body` grammar |
| `phalcom-semantic/src/static_domain.rs` | **NEW** — `StaticValueDomain`, `resolve_static_domain`, `validate_proper_type` |
| `phalcom-semantic/src/static_definition.rs` | **NEW** — `StaticParameter`, `StaticDefinitionSignature`, `StaticDefinition`, `StaticDefinitionTable`, `resolve_static_definition` |
| `phalcom-semantic/src/db/{key,product,query,fingerprint,mod}.rs` | `QueryKey::StaticDefinition`, `SemanticProduct::StaticDefinition` and routing |
| `phalcom-semantic/src/snapshot.rs` | `static_definitions` field + builder/accessor |
| `phalcom-semantic/src/session.rs` | Unified alias/static-definition resolution loop |
| `phalcom-semantic/src/types/{annotation,type_member}.rs` | AST adapters, domain adapters |
| `phalcom-semantic/src/lib.rs` | Module exports |
| `phalcom-semantic/src/checker/{body,flow/graph}.rs` | Downstream match updates for new AST |
| `phalcom-semantic/src/source_index/{builder,occurrence}.rs` | Downstream match updates |
| `phalcom-semantic/src/semantic_shard.rs` | Downstream match updates |
| `phalcom-core/src/{compiler/attributes,compiler/lib/{mod,product_opt},modules/semantic_lowering}.rs` | Downstream match updates |
| `phalcom-lsp/src/{completion,semantic_tokens}.rs` | Downstream match updates |
| `phalcom-modules/src/interface.rs` | Downstream match updates |
| `phalcom-semantic/tests/semantic/foundations/{static_domains,static_definitions}.rs` | **NEW** test suites |
| `phalcom-semantic/src/static_value/{mod,store}.rs` | **NEW** snapshot-safe primitive static-value arena |
| `phalcom-semantic/src/types/store.rs` | read-only canonical nominal type lookup for domain reporting |
| `phalcom-semantic/tests/semantic/foundations/static_values.rs` | **NEW** primitive value, equality/hash, type-bridge, and lineage tests |

---

## 5. Tests added

| Suite | Tests | Result |
|---|---|---|
| `foundations::static_domains` | 6 | PASS |
| `foundations::static_definitions` | 5 | PASS |
| Regression: `foundations::type_annotations` | 61 | PASS |
| Regression: `incremental::db` | 14 | PASS |
| `foundations::static_values` | 1 focused test covering primitive semantics and store lineage | PASS |

---

## 6. Tests deferred

The remaining T7–T11 gates are pending. Specifically:

- `static_eval` — T7 explicit-frame evaluator
- `static_constants` — T8 closed const query/dependency/cycles
- `static_materialization` and `core::compiler::static_const` — T9 descriptors and M2 materialization
- G1 integration suite, T10 incremental/resource, G2/G3 crate-level stabilization

### T4 — Snapshot-safe primitive StaticValues

Added `StaticValueStore` with a globally unique lineage ID, append-only local
indices, store-qualified `StaticValueRef`, and clone-preserved prefixes. The
session owns its `Arc<StaticValueStore>` and publishes the same lineage on each
semantic snapshot. Primitive nodes cover Unit, Bool, exact normalized BigInt,
bit-preserving Float, content-interned UTF-8 String and Symbol, proper Type,
and non-proper C3 StaticTerm values.

`semantic_eq`/`semantic_hash` implement Telos primitive value semantics. Float
storage preserves signed zero and NaN payload bits; value equality makes both
zeros equal and every NaN unequal, while semantic hashing normalizes equal
values. `content_fingerprint` retains exact Float bits for representation
identity. Type and StaticTerm admission checks the supplied TypeStore, and
domain lookup uses that paired TypeStore to report canonical built-in type IDs
and C3 kinds.

Focused gate:
`RUSTFLAGS='' cargo test -p phalcom-semantic --test semantic foundations::static_values -- --nocapture`
selected one test and passed.

### T5 — Immutable structural StaticValues and type projection

Extended the T4 store with store-local IDs for Tuple, Record, Data, and Variant
payload nodes. Tuple children preserve source order. P2 Records sort their
string keys and reject duplicates. Every child must already be present in the
same store lineage, so constructors can form shared immutable DAGs without a
backpatch path. Empty tuple construction canonicalizes to Unit.

Data values retain the exact declaration and constructor identities, and their
payload is reordered through canonical `DataComponentId`s. Variant values
retain the exact `VariantId`; the result must be the matching exact-case type.
Both reject mismatched payload shape. Type projection derives Tuple and Record
types through `TypeStore`, and validates nominal metadata before returning
stored canonical Data/Variant result types. Value result checking delegates to
the existing assignability relation.

Focused gate:
`RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic foundations::static_values_structural -- --nocapture`
selected one test and passed. It covers tuple ordering, record key-order
canonicalization and duplicate rejection, shared DAGs, foreign-child rejection,
canonical data component mapping, same-name exact variants, wrong arity,
ordinary type projection, and a relation-engine mismatch.

One API boundary is explicit: `domain_of(ref, &TypeStore)` continues to serve
primitive values. Structural ordinary-value typing uses `static_value_type`
with the snapshot's canonical Data/Enum tables, since generic record/data/case
types cannot be reconstructed from the `TypeStore` alone.

### T6 — Exact callable authorization slice

Added `StaticIntrinsicRegistry`, keyed by complete canonical `CallableId`, and
an initial `NumberAdd` operation sourced from the canonical Universe
`Number.+(_)` native surface row. The surface selector is used only to build
the exact registry key. `resolve_ready_expressions` grants a resolution only
when ordinary `ExpressionAnalysis` is ready and contains that exact selected
callable; unlisted callables and invalid expressions receive no operation
fact. This crate-private projection preserves the exact target selected by
direct or reflected dispatch and awaits trusted static-body integration.

Focused gate:
`RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic foundations::static_intrinsics -- --nocapture`
selected one test and passed. It covers canonical operation membership,
same-spelling user identity rejection, and another canonical native identity
rejection. The focused unit test
`RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic static_intrinsic::tests::ready_analysis_projection_keeps_exact_callable_and_skips_invalid_sites -- --nocapture`
also passed 1/1, covering exact target retention and invalid-expression
suppression. Expression analysis additionally records the exact `BindingId`
selected by local variable reads, synchronizes it into published products, and
includes it in callable body product fingerprints. The focused
`foundations::binding_init::read_after_assignment_is_proven` regression passed
1/1 with an assertion for that exact identity; `cargo check -p phalcom-semantic`
completed. The focused incremental fingerprint regression also passed 1/1,
including a changed-binding identity discriminator.

The standalone `analyze_static_body` helper was added at
`phalcom-semantic/src/checker/static_body.rs`. It reuses ordinary expression
and executable-region checking; binds value-domain parameters with ordinary
binding contracts; checks Kind and ValueType results with existing semantic
interfaces; marks disallowed outer reads invalid by exact `BindingId`; and
publishes parameter IDs, local slots, dependencies, expression analyses, and
exact allowlisted operation resolutions. The focused selector
`checker::static_body::tests` selected two unit tests and both passed: a static
parameter read publishes its identity, and a runtime outer-binding read is a
phase error. This gate covers the helper only; no session or query integration
is proved.

At this point in the chronological walkthrough, the query and session wiring
was still in progress. Its completed implementation and focused evidence are
recorded in the T6 execution update below.

After consultation, added `ModuleStaticBindingIndex` in
`phalcom-semantic/src/module_bindings.rs` and published it on
`SemanticSnapshot`. It is assembled from the final ordinary `<main>` analysis
and top-level binding declarations. Its `SymbolId` keys map to the owning main
callable, exact source `BindingId`, checked `BindingState`, and a phase category
selected from the declaration kind. Linked import aliases follow their
canonical `LinkedReadSpec::Binding(SymbolId)` targets. The new
`analyze_static_body_with_module_bindings` entry point seeds checked facts under
fresh local IDs, makes only const bindings statically readable, records stable
constant reads, and carries the module-index fingerprint. A checked-binding
insertion path preserves current/contract/consistency facts without repeating
contract reconciliation. A separate `CheckedStaticBody` query now tracks the
module binding and captured semantic dependencies. The final session pass
checks or reuses each body and composes it into the snapshot-facing definition
product; the early definition query remains body-free. No build or test gate
has run for this publication/composition path, so it remains implementation-
only.

---

## 7. Invariant status at interruption

Established invariants (T1–T5 scope):
- **P2-INV-03**: kinded terms and ordinary compile-time values are distinct domains ✓
- **P2-INV-04**: C3 Type/static-term identity reused, not duplicated ✓
- **P2-INV-11**: definitions are declaration-identified ✓
- **P2-INV-12**: ordinary static value parameters do not masquerade as TypeParameters ✓
- **P2-INV-13**: existing Type aliases retain optimized C3 semantics ✓
- **P2-INV-14**: StaticDefinition is the generalized source-definition authority ✓
- **P2-INV-02**: StaticValue contains no runtime `Value`/`ObjRef`/VM identity ✓
- **P2-INV-05**: ordinary static Int is exact/unbounded ✓
- **P2-INV-06**: Float bits and Telos equality/hash are separate ✓
- **P2-INV-07**: static String equality uses content ✓
- **P2-INV-08**: static Symbol identity uses stable spelling ✓
- **P2-INV-09**: published value references are store-qualified ✓
- **P2-INV-10**: the store clone lineage preserves prior IDs ✓

Not yet established (T6 onward):
- P2-INV-01, 15–45 (closed `const`, evaluator, materialization, and remaining fingerprint invariants)

---

## 8. Residual and baseline failures

No new failures were introduced by T0–T5. All pre-existing failures remain
Category D (classified in P1 walkthrough and inherited by P2).

---

## 9. Deviations and consultations

T1–T3 decisions remained within implementer authority:
- kind vs result_kind disambiguation for type lambdas in TypeAliasInfo projection
- value-parameter where-clause filtering strategy

T4 adapted `domain_of` to accept the paired `TypeStore` and return a checked
result. The value domain contains canonical `TypeId`/`KindId`, and primitive
type identity must come from `TypeStore`; no alternate registry was introduced.

An architectural consultation was requested for T6 query ownership. It found
that the early `StaticDefinitionProduct` query cannot safely be enriched under
the same key because its input fingerprint is declaration-only, it records no
semantic dependencies, and a cache hit returns the earlier product. The active
plan was amended to use a distinct `CheckedStaticBody` query keyed by
`StaticBodyId`, with source/signature/readable-binding inputs, precise semantic
dependency edges, a fingerprint of checked output, and composition into the
snapshot-facing static-definition view. The module static-constant binding
input has a second recorded decision: ordinary module checking must publish a
`ModuleStaticBindingIndex` keyed by stable `SymbolId`/`StaticConstantId`, with
the source-body-qualified local `BindingId`, checked type/contract, source site,
and a declaration-time static/runtime phase category. Static bodies seed
fresh local identities from this product and record stable const reads for T8.

---

## T6 execution update — checked static bodies and exact operations

Completed T6's ordinary-checker body analysis and publication path:

- Published `ModuleStaticBindingIndex` facts from final module analysis, with
  stable `SymbolId`, body-qualified source `BindingId`, checked state, and
  static-vs-runtime phase. Linked import aliases retain canonical target
  identity.
- Added `binding_write` to ordinary expression analysis so static-phase
  checking can reject an outer runtime mutation without pretending it was a
  read.
- Added `CheckedStaticBody` as a separate SemanticDb query with source/module
  inputs, semantic dependency edges, and checked-product fingerprints. The
  session composes this cached product into the snapshot-facing static
  definition only for modules with active ordinary source analysis; canonical
  Universe definitions without such a handoff are not checked in this pass.
- Added a narrow checker bridge for ambiguous bare-name body syntax: with a
  value result domain, a single name resolves through the seeded ordinary
  binding scope; kind-result type forms continue through type-form checking.
- Confirmed selected `Number.+(_)` creates `StaticOperationResolution` in a
  checked body. Existing exact-key registry tests reject same-spelling user
  and non-whitelisted canonical identities.

Focused evidence:

```text
RUSTFLAGS='' RUSTC_WRAPPER='' cargo check -p phalcom-semantic — passed
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --lib checker::static_body::tests -- --nocapture — 4 passed
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic foundations::static_ -- --nocapture — 19 passed
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic integration::static_definition_bodies -- --nocapture — 3 passed
```

The first session regression exposed a blocked body query for canonical
Universe definitions without module binding handoff; the session now checks
only active source-module definitions in this composition pass. The second
exposed the parser's neutral bare-name bridge; semantics now resolves that
name from checked binding identity. Focused regressions pass after both
corrections. The three integration cases also cover imported alias identity and
body invalidation across revisions. Broader C1–C3 compatibility remains for
later gates.

## T7 execution update — evaluator foundation slice

Added `phalcom-semantic/src/static_eval/` with an explicit work queue,
checker-backed parameter and local reads, ordered expression blocks, lazy
`if`/`and`/`or`, primitive literals including canonical Symbol spellings,
exact-authorized integer addition, and positional Tuple/bare-label Record
construction.
Mixed kinded/value parameter positions are retained and argument kinds/counts
are checked before execution. Unsupported syntax still fails closed.

The checked-body unit selector passed 4 tests, including parameter reduction,
local initialization/read by checked binding identity, argument count/kind
rejection, budget exhaustion, and cancellation. The session integration
selector passed 3 tests, including exact-authorized integer addition
(`1 + 2` reduces to `3`), selected `if` branch reduction, paired lazy
`and`/`or` paths where an unsupported comparison is skipped only while
untaken, and positional Tuple/bare-label Record construction.
The same integration reduces a mutable local after assignment to its new
checked value.
`cargo check -p phalcom-semantic` also completed. This is focused
evidence for the initial evaluator slice, not T7 completion. At that point,
SymbolPattern values, static applications, recursion outcomes, Data/Variant
construction, diagnostic traces, and remaining execution budgets were still
required; later T7.10–T7.12 progress is recorded below. Mutable local
assignment and Tuple/Record construction are covered by this slice. Labeled/spread
Tuples, Record expansions/computed labels, and TypeForm roots remain unsupported
in this evaluator slice.

### T7.10 checked application product and publication blocker (resolved)

Before adding nested static applications, live-source analysis found no exact
application-target/argument product for the evaluator to consume: expression
analysis publishes ordinary call and binding facts, while type-form application
resolution returns canonical `TypeId`s. The active plan requires exact resolved
signature authority. The checkpoint records the narrow decision request; no
source-name lookup workaround was added.

The user resolved the architecture question in favor of an ordinary-checker
published checked static-application product carrying exact target/signature/
domain and argument identities. T7 value-parameter applications remain in
scope; C3 `TypeAliasInfo` stays an optimized compatibility projection. This is
a plan amendment. The product seam is now implemented: `TypeResolver` exposes
the exact static definition by declaration identity, the checker records
`CheckedStaticApplication` in `CheckedStaticBody`, and a formal
`SemanticDependency::StaticDefinition` is replayed by the checked-body query.
`SemanticDenotation::StaticApplication` links the checker expression fact to
the stored application product. The explicit-frame evaluator now consumes it.

Focused regression attempt (2026-10-04):

```text
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic integration::static_definition_bodies -- --nocapture — FAILED
```

Adding `type Literal: Int = 3` and `type Applied: Int = Literal<>` to the
previously passing fixture made its existing `ReadBase` lookup fail because
the snapshot static-definition table was empty. The focused consultation
confirmed that `resolve_type_name` recorded an unreplayable `DeclarationShell`
edge for these value-domain targets. The localized
`resolve_static_definition_name` resolver preserves `LinkedName` and
`PublicExport` dependencies and records the canonical `StaticDefinition` edge
without that invalid shell edge. No evaluator spelling lookup or replay
exception was added.

The focused integration selector passed 3/3 after this correction and again
after evaluator application frames and the mixed kinded/value regression were
added. Verified evaluator cases include zero-argument, value-parameter,
directly resolved kinded-parameter, mixed kinded/value, nested nonrecursive,
and self-reentry applications. The checked-constructor product now retains
body-local associated resolutions, exact target/result type, argument
expression IDs, and stable payload IDs. The evaluator consumes borrowed
snapshot TypeStore/Data/Enum tables. Reordered labeled Data payloads reduce to
canonical component order, and same-named Variant cases under distinct owners
remain distinct. The focused integration selector passed 3/3 with these cases.
T7 remains partial: kinded-expression applications, bounded diagnostic traces,
and remaining budget/cancellation boundaries still require implementation
and focused evidence. T8 must not begin until T7 acceptance is complete.

*This walkthrough will be completed and finalized at T11.*

### T7.12 bounded trace and recursion/resource evidence

`StaticEvaluator::trace()` now exposes definition, exact application, and
source-expression frames. Each trace push charges the existing `QueryBudget`
and checks cancellation; no independent trace-depth limit was introduced.
The focused `integration::static_definition_bodies` selector passed 3/3 with
mutual A→B→A reentry, application step-budget exhaustion, and trace-path
assertions. The evaluator unit test also distinguishes pre-cancelled
execution. `Recursive`, `BudgetExceeded`, and `Cancelled` remain separate
outcomes.

T7 remains `IN_PROGRESS / PARTIAL / UNVERIFIED`. Kinded-expression
applications and remaining resource/value-node boundaries are open. Do not
advance to T8 until T7 acceptance is complete.


### T7 focused acceptance — 2026-10-04

The final focused command passed:

```text
RUSTFLAGS='' RUSTC_WRAPPER='' cargo test -p phalcom-semantic --test semantic integration::static_definition_bodies -- --nocapture
3 passed, 0 failed
```

Added discriminators for false and nested `if`, nested arithmetic, source-order
mutable writes, uninitialized local read, and immutable reassignment rejection.
Together with prior focused cases, the selector covers checked applications
(including the amended TypeForm result subset), exact Data/Variant construction,
recursion/trace outcomes, lazy positive/negative paths, step-budget exhaustion,
and cancellation. A bounded Data-construction budget case confirms the value
node is not interned after exhaustion. `QueryBudget` has no distinct value-node
quota, so the plan's optional quota test is not enabled. T7 is
IMPLEMENTED / FOCUSED_TESTED; this does not complete P2. T8 is next.


### T8 source/result architecture decision — 2026-10-04

Before T8 implementation, a focused consultation resolved the initializer
ownership seam. A module const needs its own `StaticConstant(SymbolId)` query
and constant-specific checked initializer product. The ordinary module binding
handoff must retain the initializer source site; parsed source is accessed by
that site and stable identity. The current static-definition checked-body
owner is not reused. The shared ordinary checker analyzes the exact initializer
in static-body context, preserving one checker/resolution authority. The
checkpoint and plan record the targeted invalidation discriminator and exact
next slice. No T8 source code has been implemented or verified yet.
