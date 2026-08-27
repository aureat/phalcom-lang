# Phalcom Semantic Capability Gap Closure — Repository-Grounded Continuation Plan

**Repository:** `aureat/phalcom-lang`  
**Verified baseline:** `main` at `e1c8764bb85f4e9d9dcab89e2da06da3a03881b9`  
**Date:** 2026-08-27  
**Status:** Continuation implementation plan after the first three landed capability slices

---

## 1. Executive summary

The current `main` branch contains the first three capability slices, but the continuation work described during the interrupted analysis is still unlanded.

The remaining work is not one isolated bug. It is a connected set of semantic-product and incremental-ownership gaps:

1. **Abrupt-flow publication is incomplete.**
   - `throw` does not publish a throw exit.
   - callable finalization synthesizes return-flow information from the final context instead of retaining exact exit snapshots.
   - `BodyExitFacts::throws` is always empty in the current finalizer.

2. **Field lifecycle semantics are not implemented.**
   - `FlowState` tracks lexical bindings and predicates only.
   - there is no `FieldState`, initialization lattice, or constructor-to-method lifecycle product.
   - the staged field capability test remains ignored.

3. **Pattern decomposition is only partially closed.**
   - statement declaration patterns gained list/rest decomposition helpers.
   - the expression-side binder used by `if let` / `while let` still handles only name and tuple patterns.
   - the list/rest capability scenario is still ignored.

4. **Incremental callable-body reuse has two distinct problems.**
   - current `main` still contains refresh-side invalidation/recompute behavior that conflicts with the DB’s existing reuse model.
   - after that is corrected, callable body input identity is still position-sensitive because it hashes absolute ranges and AST `Debug` representations containing ranges.

5. **Source position ownership is entangled with `CallableAnalysis`.**
   - `CallableAnalysis` stores absolute `body_range`, expression ranges, binding ranges, diagnostics, and provenance.
   - `CallableSourceAttachment::from_analysis` reconstructs source attachments from those ranges.
   - therefore reusing the exact same `Arc<CallableAnalysis>` after a pure positional shift would publish stale source locations.

6. **Canonical imported identity is already substantially implemented.**
   - source indexing resolves imported bindings to the declaring module’s canonical declaration identity.
   - imported class inference preserves declaring-module identity.
   - this area needs regression hardening and integration with incremental/source movement work rather than a parallel identity model.

The core architectural recommendation is:

> Keep semantic identity and semantic DB reuse authoritative, but explicitly separate reusable position-independent semantic payload from current-revision source attachment/presentation data.

For fields:

> Treat constructor field-lifecycle summaries as compiler-owned semantic products with dependency edges, not session-local side tables.

For abrupt flow:

> Make exact exit snapshots a prerequisite to field lifecycle, because constructor definite-initialization proof depends on which field states survive all reachable normal exits.

---

# 2. Current repository state

## 2.1 Flow state contains no field lifecycle domain

Current flow state is binding-centric. The published summary currently contains:

```rust
pub struct FlowStateSummary {
    pub bindings: BTreeMap<BindingId, FlowBindingSummary>,
    pub fact_count: usize,
}
```

There is no field-state map.

Required conclusion:

```text
field lifecycle semantics are not merely unfinished plumbing;
the formal state domain itself does not yet model them.
```

The field capability test remains gated:

```rust
#[ignore = "GATED: formal field read/write publication is staged in the capability ledger"]
fn field_facts_survive_constructor_and_general_writes()
```

---

## 2.2 Abrupt exits are not formally published with enough fidelity

`check_statement` currently handles:

```rust
Statement::Return(...)
Statement::Throw { ... }
Statement::Break { ... }
Statement::Continue { ... }
```

but only a direct `return` produces a returned `TypeKnowledge`.

`throw` currently evaluates its operand and returns `None`.

The callable finalizer currently constructs:

```rust
BodyExitFacts {
    returns: ...,
    normal_return_values,
    throws: Vec::new(),
    unreachable: false,
}
```

This is insufficient for:

- exact branch exit reasoning;
- throw-path exclusion;
- constructor lifecycle proof;
- future effect/termination analysis.

The semantic coverage ledger itself already admits this limitation: deeper `BodyExitFacts` trace assertions remain a prerequisite.

---

## 2.3 List/rest pattern support is split across two binders

`statement.rs` has a declaration-pattern binder that now supports:

- name,
- tuple,
- list,
- list rest.

It uses:

```rust
decompose_list_element(...)
decompose_list_rest(...)
```

However, `expression.rs` still has a separate `bind_pattern(...)` used by `if let` / `while let`, and it only supports:

- name,
- tuple.

Therefore the language currently has two pattern semantic engines with divergent capability.

Recommendation:

> Collapse both paths onto one canonical recursive pattern decomposition API.

Do not add list/rest handling independently to both implementations.

---

## 2.4 Higher-order callable invocation is implemented

The higher-order capability tests are enabled, including:

- explicit `.call()`,
- direct block invocation,
- nominal `call` not using the structural shortcut.

This area should be treated as landed capability, not rewritten as part of the continuation.

---

# 3. Abrupt-flow closure plan

## 3.1 Goal

Replace the current synthetic final exit model with explicit recorded exits.

The checker must be able to distinguish:

```text
normal fallthrough
explicit return
throw
break
continue
unreachable
```

The callable-level product should publish only callable exits:

```text
normal return
throw
unreachable / diverging
```

Loop-local `break` and `continue` remain loop-frame facts.

---

## 3.2 Introduce explicit exit capture in `CheckingContext`

Add checker-owned collections such as:

```rust
pub(crate) struct NormalExit {
    pub flow: FlowState,
    pub value: TypeKnowledge,
}

pub(crate) struct ThrowExit {
    pub flow: FlowState,
    pub thrown: TypeKnowledge,
}
```

and context fields:

```rust
normal_exits: Vec<NormalExit>,
throw_exits: Vec<ThrowExit>,
```

Preferred implementation location:

```text
phalcom-semantic/src/checker/context.rs
```

Do not make these query/session-side structures.

---

## 3.3 Record exits at the statement boundary

In:

```text
phalcom-semantic/src/checker/statement.rs
```

change direct `return` handling from “return an optional type to the body walker” into:

```text
evaluate expression
check declared return contract
snapshot current FlowState
record NormalExit
mark current flow unreachable
```

Similarly for `throw`:

```text
evaluate thrown expression
snapshot current FlowState
record ThrowExit
mark current flow unreachable
```

The body walker should no longer need to infer exit semantics from `Option<TypeKnowledge>` alone.

---

## 3.4 Remove synthetic final-exit publication

`CheckingContext::finalize_with_normal_returns(...)` should stop manufacturing:

```rust
returns: vec![entry_flow.clone()]
throws: Vec::new()
```

Instead:

```text
BodyExitFacts.returns
    = summaries of explicitly recorded normal exits
      plus final reachable fallthrough

BodyExitFacts.normal_return_values
    = corresponding normal values

BodyExitFacts.throws
    = summaries of explicitly recorded throw exits

BodyExitFacts.unreachable
    = whether no normal fallthrough remains / body is proven divergent,
      according to the exact callable semantics
```

---

## 3.5 Required tests

Enable or add source-level tests for:

```text
returning_branch_does_not_contribute_value_to_continuing_join
throwing_branch_is_excluded_from_reachable_value_join
refined_branch_with_abrupt_else_publishes_only_normal_value
nested_return_records_exact_exit_flow
nested_throw_records_exact_throw_flow
throw_only_callable_has_no_normal_return_value
```

Also add direct product assertions against `BodyExitFacts`.

---

# 4. Field lifecycle product

## 4.1 Do not implement fields as ordinary lexical bindings

A field has a different identity and lifetime from a local binding.

Use canonical:

```rust
FieldId {
    owner: DeclarationId,
    name: Box<str>,
    side: DispatchSide,
}
```

already defined in:

```text
phalcom-semantic/src/identity.rs
```

Field flow state should therefore be keyed by `FieldId`.

---

## 4.2 Add a field-state domain

Recommended model:

```rust
pub enum FieldInitialization {
    Uninitialized,
    MaybeInitialized,
    DefinitelyInitialized,
}

pub struct FieldState {
    pub field: FieldId,
    pub contract: TypeKnowledge,
    pub current: TypeKnowledge,
    pub initialization: FieldInitialization,
    pub causal_invalidity: CausalInvalidity,
    pub explanation: Option<ExplanationId>,
}
```

Integrate into:

```rust
FlowState {
    bindings: ...,
    fields: BTreeMap<FieldId, FieldState>,
    facts: ...,
    reachable: ...
}
```

and:

```rust
FlowStateSummary {
    bindings: ...,
    fields: BTreeMap<FieldId, FlowFieldSummary>,
    fact_count: usize,
}
```

Recommended files:

```text
phalcom-semantic/src/checker/flow/state.rs
phalcom-semantic/src/checker/analysis.rs
```

---

## 4.3 Field initialization join law

At control-flow joins:

```text
Definitely + Definitely -> Definitely
Definitely + Uninitialized -> Maybe
Maybe + anything reachable -> Maybe
Uninitialized + Uninitialized -> Uninitialized
```

Current type knowledge joins independently using the existing epistemic join rules.

Do not convert “declared field type” into established current knowledge when initialization is absent.

A field contract is a constraint, not proof of initialization.

---

## 4.4 Seed field state at callable entry

For instance-side checking, the context needs the declaration’s field surface.

At callable entry:

```text
ordinary instance method:
    seed from class lifecycle product

constructor:
    seed declaration defaults / uninitialized fields
```

Class-side fields are a separate lifecycle domain and should not be conflated with instance construction.

---

## 4.5 Default field initializers

Existing session code calls:

```rust
check_class_field_initializers(...)
```

This validates initializer expressions but does not produce a lifecycle state.

Convert field defaults into formal constructor-entry facts:

```text
field with valid default:
    current = initializer fact
    initialization = DefinitelyInitialized

field without default:
    current = Unknown(MissingInitializer or field-specific absence state)
    initialization = Uninitialized
```

Prefer a dedicated field-initialization absence representation over abusing local-binding `MissingInitializer` if diagnostics/provenance require distinction.

---

# 5. Constructor lifecycle summary as a semantic product

## 5.1 Why a product is required

Ordinary instance methods need to know what constructors establish.

That is cross-callable semantic consumption.

Therefore lifecycle is not a `HashMap` hidden inside `SemanticWorkspaceSession`.

It must participate in incremental dependency tracking.

---

## 5.2 Add a query/product

Recommended canonical key:

```rust
QueryKey::DeclarationFieldLifecycle(DeclarationId)
```

or, if constructor-specific composition proves cleaner:

```rust
QueryKey::ConstructorFieldLifecycle(CallableId)
```

Then compose a declaration-level lifecycle product.

Recommended product:

```rust
pub struct DeclarationFieldLifecycle {
    pub declaration: DeclarationId,
    pub fields: BTreeMap<FieldId, FieldLifecycleFact>,
}
```

with:

```rust
pub struct FieldLifecycleFact {
    pub initialization: FieldInitialization,
    pub knowledge: TypeKnowledge,
}
```

A declaration-level product is preferable because ordinary methods consume the class invariant, not a particular constructor implementation.

---

## 5.3 Constructor aggregation semantics

For each constructor:

```text
collect every reachable normal exit
join its field states
```

Then across constructors:

```text
field is DefinitelyInitialized for the class
iff every successful constructor definitely initializes it
```

If a class has a default initializer and all constructors preserve it, it remains definite.

Throw-only constructor paths do not weaken successful-instance lifecycle because no instance is produced on those paths.

This is why exact abrupt-exit publication is a prerequisite.

---

## 5.4 Dependency integration

Extend:

```rust
SemanticDependency
```

with a field-lifecycle dependency, e.g.:

```rust
DeclarationFieldLifecycle(DeclarationId)
```

and map it to the corresponding `QueryKey`.

When an ordinary instance method seeds field state from the class lifecycle product, record that dependency.

Then:

```text
constructor field semantics change
        ↓
lifecycle product fingerprint changes
        ↓
ordinary methods that read field initialization facts invalidate
```

No session-specific invalidation rule is needed.

---

# 6. Field read/write analysis

## 6.1 Bare field syntax

Current expression handling recognizes field reads/writes and surface field types.

Upgrade those paths to use:

```text
surface declaration = field contract
flow field state     = current knowledge + initialization
```

Reading a definitely initialized field:

```text
returns field.current
```

Reading maybe/uninitialized field:

```text
must not fabricate the declared type as Established
```

Possible outcomes depend on language policy:

- error for definite-initialization violation;
- assumed contract only if Phalcom explicitly permits unsafe reads;
- unknown if evidence is incomplete.

Given the existing “checker is authoritative when proof exists” model, a source error is the stronger recommendation.

---

## 6.2 Writes

A write:

```phalcom
_value = expr
```

should:

1. analyze `expr`;
2. check it against the field contract;
3. preserve actual value fact;
4. update `FieldState.current`;
5. set initialization to `DefinitelyInitialized` on the current path;
6. retain refutation causality without replacing the value fact with the contract.

This mirrors binding reconciliation, but must operate on `FieldId`.

---

## 6.3 Property syntax

`receiver.property` and `receiver.property = value` currently resolve direct fields before getter/setter dispatch.

For `self` / current receiver where identity is known, route exact field access through the flow field state.

For arbitrary external receivers:

```text
the checker normally has no per-object construction state
```

so use declaration-level lifecycle knowledge plus field contract.

Do not pretend to track object-identity-sensitive heap mutation globally in this slice.

That would require alias analysis.

---

# 7. Pattern decomposition closure

## 7.1 Eliminate duplicate recursive binders

Current duplication:

```text
statement.rs::bind_declaration_pattern
expression.rs::bind_pattern
```

Introduce a canonical decomposition function that can produce pattern leaves:

```rust
pub struct PatternLeaf {
    pub pattern: ...,
    pub fact: ValueSemanticFact,
}
```

or a recursive callback API.

Recommended file:

```text
phalcom-semantic/src/checker/pattern.rs
```

Both declaration patterns and `if let` / `while let` should call it.

---

## 7.2 Supported first closure set

Unify at least:

```text
Name
Tuple
List
List rest
```

Then keep record/map/variant patterns explicitly gated until their formal decomposition contracts are implemented.

Do not silently accept unsupported patterns with `_ => {}`.

Unsupported formal pattern shapes should produce a structured incomplete/blocking result.

---

## 7.3 Promote the capability test

After canonical list/rest decomposition is wired through the source paths, remove:

```rust
#[ignore = "GATED: list/rest pattern lowering is not formal yet"]
```

from:

```text
phalcom-semantic/tests/semantic/capabilities/patterns.rs
```

and promote S06 in the coverage ledger.

---

# 8. Incremental repair — phase 1: remove session-side cache policy

## 8.1 Existing DB behavior is already correct in principle

`SemanticDb::validate_reuse(...)` requires:

```text
same direct input fingerprint
+
dependencies validated this revision
+
dependency product fingerprints unchanged
```

It updates only `validated_revision`.

It deliberately permits reuse of an older computation revision when the semantic product is unchanged.

That is the desired incremental law.

---

## 8.2 The session should not override it

The session currently performs additional callable refresh behavior around inferred returns and recomputation.

The continuation should ensure:

```text
DB owns query reuse
session orchestrates publication
```

not:

```text
DB reuse
+
session-local refresh invalidation policy
```

The fixed-point inferred-return pass may remain as a semantic algorithm, but it must use query product dependencies instead of discarding/recomputing products as an external cache protocol.

---

# 9. Incremental repair — phase 2: position-independent callable input identity

## 9.1 Current problem

Callable body input identity currently includes absolute source positions.

Even removing the direct `body_range` hash is insufficient if the fingerprint hashes:

```rust
format!("{statement:?}")
```

because the AST’s derived `Debug` output contains `SourceRange` fields throughout the tree.

Therefore:

> A semantic callable-body input fingerprint must be computed from position-independent syntax structure.

---

## 9.2 Add a semantic AST fingerprint visitor

Implement a dedicated walker that hashes:

```text
node kind
names
operators
literal values
labels
parameter structure
annotations
nested expression structure
control structure
```

and deliberately ignores:

```text
SourceRange.start
SourceRange.end
token locations
presentation-only offsets
```

Recommended location:

```text
phalcom-semantic/src/db/fingerprint.rs
```

or a reusable AST fingerprint module in `phalcom-ast` if other compiler stages need it.

Do not use `Debug` as a semantic hash format.

---

## 9.3 Separate body semantic fingerprint from presentation fingerprint

The repository already introduced the same distinction for source indexes:

```rust
SourceIndexFingerprints {
    semantic,
    presentation,
}
```

Apply the same architecture to callable body input.

For example:

```rust
CallableBodyInputFingerprints {
    semantic,
    presentation,
}
```

Query reuse uses `semantic`.

Source attachment refresh uses current presentation/source data.

---

# 10. `CallableAnalysis` range entanglement

## 10.1 Why direct `Arc<CallableAnalysis>` reuse across source movement is unsound

Current `CallableAnalysis` stores:

```text
body_range
ExpressionAnalysis.range
BindingState.range
diagnostic ranges
provenance ranges
internal incident ranges
```

`CallableSourceAttachment::from_analysis(...)` then matches:

```text
binding.name
binding.declaration_range == state.range
callable owner
```

and builds expression sites directly from:

```rust
expression.range
```

Thus an old analysis object contains old source coordinates.

---

## 10.2 Required architectural split

Introduce a position-independent semantic core.

Recommended direction:

```rust
pub struct CallableSemanticAnalysis {
    pub callable: CallableId,
    pub expressions: ...
    pub bindings: ...
    pub flow_graph: ...
    pub entry_flow: ...
    pub exits: ...
    pub dependencies: ...
    pub semantic_dependencies: ...
    pub status: ...
}
```

where semantic expression/binding identities do not require absolute ranges.

Then a current-revision wrapper:

```rust
pub struct CallableAnalysis {
    pub semantic: Arc<CallableSemanticAnalysis>,
    pub source: Arc<CallableAnalysisSource>,
}
```

with:

```rust
pub struct CallableAnalysisSource {
    pub body_range: SourceRange,
    pub expression_ranges: BTreeMap<ExpressionId, SourceRange>,
    pub binding_ranges: BTreeMap<BindingId, SourceRange>,
    pub diagnostics: Arc<[SemanticDiagnostic]>,
    ...
}
```

The exact shape can vary, but the ownership rule should not.

---

## 10.3 Pointer-identity acceptance criterion

Do not require:

```text
same Arc<CallableAnalysis>
```

after a positional-only source edit.

Require:

```text
same Arc<CallableSemanticAnalysis>
```

while allowing a new current-revision source wrapper.

This is the only sound way to combine:

```text
semantic reuse
+
fresh source positions
```

under the current product requirements.

---

# 11. Source attachment repair

## 11.1 Stop using absolute range equality as semantic binding identity

Current attachment logic searches for:

```text
same name
same declaration range
same callable owner
```

That makes presentation coordinates part of the attachment contract.

Use stable local semantic IDs instead.

The source index already gives each callable a local `SourceSiteLocalId` namespace.

The checker also has stable-in-body:

```text
BindingId
ExpressionId
```

Create an explicit current-parse mapping from semantic local IDs to source sites.

---

## 11.2 Recommended attachment model

During source indexing, assign deterministic callable-local structural ordinals for:

```text
parameters
local binding declarations
expressions
```

Then the checker and source index can attach through a structural key such as:

```rust
CallableLocalSourceKey {
    kind,
    ordinal,
}
```

or by sharing a compiler-owned traversal identity.

The exact implementation should preserve the existing law:

```text
SourceRange is attachment metadata, not semantic identity.
```

This law is already documented in `SourceSiteId`.

---

# 12. Import identity closure

## 12.1 What is already correct

The repository already proves:

```text
imported binding use
    -> exported declaration target
    -> declaring module identity
```

and imported class expressions infer the class object of the original declaration.

Do not replace this with name-based local aliases.

---

## 12.2 Required additional tests

Add or strengthen tests for:

```text
same leaf declaration name in multiple imported modules
selective import alias preserves original declaration
module import member access preserves declaring module
re-export preserves ultimate canonical declaration identity
incremental import target change invalidates dependent semantic product
renaming local import alias does not change canonical imported target
```

These belong under:

```text
phalcom-semantic/tests/semantic/integration/
phalcom-semantic/tests/semantic/incremental/
```

---

# 13. Debug cleanup required before closure

Current `main` contains development logging such as:

```rust
eprintln!("trusted condition typed callable=...")
eprintln!("control={} recv={:?}", ...)
eprintln!("then predicate=...")
eprintln!("core object fallback ...")
```

These must be removed or routed through an explicit opt-in tracing facility before the capability slice is considered complete.

Semantic analysis should not emit unsolicited stderr output in normal compiler/LSP operation.

---

# 14. Recommended implementation order

## Phase A — close already-landed slices

1. Remove semantic debug `eprintln!`.
2. Canonicalize pattern decomposition.
3. Enable list/rest capability test.
4. Add explicit abrupt-exit capture.
5. Enable abrupt-flow composed capability tests.

Reason:

```text
field lifecycle depends on exact exits
```

and should not be built on the current synthetic exit model.

---

## Phase B — field flow state

1. Add `FieldInitialization`.
2. Add `FieldState`.
3. Extend `FlowState`.
4. Extend join semantics.
5. Extend `FlowStateSummary`.
6. Route current-receiver field reads/writes through field flow.
7. Add field-specific diagnostics/provenance.

---

## Phase C — lifecycle query product

1. Add lifecycle product type.
2. Add query key.
3. Add product variant/accessor.
4. Add DB query.
5. Aggregate constructor normal-exit field states.
6. Record lifecycle dependency in ordinary instance bodies.
7. Re-enable field lifecycle capability test.
8. Add constructor branch/throw/default-init tests.

---

## Phase D — incremental DB cleanup

1. Remove refresh-side invalidation that duplicates DB reuse ownership.
2. Make inferred-return fixed point operate through semantic dependencies.
3. Re-run existing callable dependency tests.
4. Promote the currently ignored remove/re-add composed test once green.

---

## Phase E — semantic/presentation split

1. Implement position-independent AST semantic fingerprints.
2. Split callable semantic payload from source attachment.
3. Reuse semantic payload across positional shifts.
4. rebuild source attachment for current snapshot.
5. Add whitespace/prepend/move tests.

---

# 15. Acceptance tests

A continuation implementation should not be considered complete until these categories are green and enabled.

## Abrupt flow

```text
return branch excluded from continuing join
throw branch excluded from continuing join
nested abrupt branch preserves surviving type refinement
throws published in BodyExitFacts
```

## Patterns

```text
list head/rest binding
nested list/tuple decomposition
if-let list decomposition
while-let list decomposition
```

## Fields

```text
default initializer establishes field
constructor write establishes field
all constructor branches initialize -> definite
one constructor branch omits write -> maybe/uninitialized diagnostic
throw-only branch does not weaken produced instance
ordinary method reads lifecycle-proven field
mutation updates current field fact
wrong write refutes relation without overwriting actual value fact
```

## Incremental

```text
callee body-only edit reuses caller
signature edit invalidates caller
remove/re-add removes stale products
unrelated source insertion before callable reuses semantic payload
whitespace-only movement refreshes source positions
current source attachment contains new positions
no stale old ranges survive in snapshot
```

## Imports

```text
canonical declaration identity through import
canonical identity through alias
canonical identity through re-export
same leaf names remain distinct
```

---

# 16. Key architectural decisions

## Decision 1

**Abrupt exits are first-class checker products.**

They are not inferred later from the final flow state.

## Decision 2

**Field lifecycle is flow state plus a query-visible declaration product.**

It is not a session-local side table.

## Decision 3

**Field contracts and field initialization are independent axes.**

Knowing `_x: Int` does not prove that `_x` has been initialized.

## Decision 4

**Pattern decomposition has one recursive semantic implementation.**

Declaration patterns and conditional patterns must not drift.

## Decision 5

**Semantic DB reuse remains authoritative.**

Session orchestration must not maintain a second invalidation policy.

## Decision 6

**Semantic callable-body fingerprints ignore presentation coordinates.**

No `Debug`-based AST hashing.

## Decision 7

**Reusable semantic payload and current source attachment are separate products.**

Exact old `Arc<CallableAnalysis>` reuse across source movement is not a valid goal while that object owns absolute ranges.

## Decision 8

**Canonical import identity remains declaration/module based.**

Aliases affect source bindings, not the underlying semantic target.

---

# 17. Files expected to change

Primary files:

```text
phalcom-semantic/src/checker/analysis.rs
phalcom-semantic/src/checker/body.rs
phalcom-semantic/src/checker/context.rs
phalcom-semantic/src/checker/expression.rs
phalcom-semantic/src/checker/statement.rs
phalcom-semantic/src/checker/composition.rs
phalcom-semantic/src/checker/flow/state.rs
phalcom-semantic/src/checker/flow/transfer.rs
phalcom-semantic/src/checker/flow/join.rs
phalcom-semantic/src/db/key.rs
phalcom-semantic/src/db/product.rs
phalcom-semantic/src/db/query.rs
phalcom-semantic/src/db/fingerprint.rs
phalcom-semantic/src/session.rs
phalcom-semantic/src/source_index/mod.rs
phalcom-semantic/src/source_index/builder.rs
phalcom-semantic/src/source_index/occurrence.rs
```

Likely new files:

```text
phalcom-semantic/src/checker/pattern.rs
phalcom-semantic/src/checker/field.rs
phalcom-semantic/src/field_lifecycle.rs
```

Tests:

```text
phalcom-semantic/tests/semantic/capabilities/flow_branches.rs
phalcom-semantic/tests/semantic/capabilities/patterns.rs
phalcom-semantic/tests/semantic/capabilities/fields.rs
phalcom-semantic/tests/semantic/incremental/callable_dependencies.rs
phalcom-semantic/tests/semantic/integration/imported_resolution.rs
```

Ledgers:

```text
phalcom-semantic/tests/semantic/capabilities/BASELINE_LEDGER.md
phalcom-semantic/tests/semantic/COVERAGE_LEDGER.md
```

---

# 18. Final recommendation

Do not proceed directly to field initialization by patching field reads and writes.

The correct sequence is:

```text
exact abrupt exits
    ↓
canonical pattern decomposition closure
    ↓
field state in flow
    ↓
constructor lifecycle product
    ↓
ordinary-method lifecycle consumption
    ↓
incremental dependency wiring
    ↓
semantic/presentation split for stable reuse
```

That sequence keeps Phalcom’s existing semantic philosophy intact:

```text
proof before publication
unknown remains unknown
contracts constrain but do not fabricate evidence
canonical identity survives surface aliases
query products own semantic dependencies
presentation coordinates do not define semantic identity
```

The most important architectural correction is the last one: a callable analysis product that contains absolute source positions cannot simultaneously be the unit of position-independent semantic reuse. The reusable semantic core must be made explicit.
