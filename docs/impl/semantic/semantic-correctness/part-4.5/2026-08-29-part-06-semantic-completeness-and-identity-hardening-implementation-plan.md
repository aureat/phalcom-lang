# Part 06 — Semantic Completeness and Identity Hardening Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Close the six-part semantic correctness program by removing remaining syntax- and spelling-based shortcuts, fixing canonical TypeStore/TypeId transport defects, routing operation syntax through real semantic evidence, adding contextual empty-collection typing without authority laundering, auditing every formal proof-construction boundary, and proving clean/incremental/editor publication stability.

**Architecture:** Part 06 intentionally adds no new general type-system architecture. It hardens existing semantic identities and funnels residual expression forms through the same dispatch/evidence/relation machinery established by Parts 01–05. Canonical core privilege is represented by exact `DeclarationId`/`CallableId`, generic-supertype specialization materializes into the active `TypeStore`, comparison/membership syntax fails closed unless its operation is semantically established, and a repository-wide authority audit becomes executable through focused regression tests and a maintained audit ledger.

**Tech Stack:** Rust, `phalcom-semantic`, `phalcom-ast`, `phalcom-modules`, canonical `DeclarationId` / `CallableId`, `TypeStore` / `TypeId`, `TypeKnowledge`, `RelationOutcome`, canonical dispatch, semantic `Fixture`, incremental `SemanticWorkspaceSession`, immutable `SemanticSnapshot`, Cargo test/check/fmt.

**Spec:** This is Part 06, the final plan in the six-part semantic hardening series. It consumes Parts 01–05 and closes the remaining gaps identified by source archaeology against `docs/impl/semantic/semantic-correctness/part-4/2026-08-27-semantic-capability-gap-closure-implementation-plan.md` and current implementation behavior.

## Global Constraints

- Historical grounding source for this plan: `main` at `24fc9fd98f3c3c534c4d52b613962a39b9374185`. Rebase against the actual post-Part-05 HEAD before implementation.
- `phalcom-semantic` remains the only static semantic engine. `phalcom-lsp` may render/query compiler products but must not gain type/dispatch/identity logic.
- Semantic privilege is identity-based, never spelling-based. A user declaration named `Object`, `Function`, `Closure`, `Bool`, `Int`, `List`, or another core name receives no compiler privilege by name alone.
- A `TypeId` is meaningful only in the canonical active `TypeStore` arena that produced it. Do not materialize a new `TypeId` in a cloned store and consume it in the original store.
- `RelationOutcome::Proven` proves a relation; it never upgrades the authority of either premise.
- `Established` must have an explicit compiler-owned evidence source. Syntax shape alone does not establish the result of an overridable/dispatch-dependent operation.
- Formal unknown remains unknown unless a valid formal contract is allowed to supply an Assumed fact under Part 01 rules.
- Advisory products never become formal evidence.
- Unknown/unsupported operation semantics fail closed. Do not invent a selector or runtime protocol in the semantic checker.
- Comparison-chain middle operands are analyzed once.
- Empty collection literals may consume compatible expected type context, but contextual evidence inherits the context's authority; empty syntax does not prove an element/key/value type.
- Do not redesign `TypeId` representation or embed a store ID into every `TypeId` in this part. Repair canonical-store ownership at the APIs that materialize/transport IDs.
- Do not build a general builtin registry. Add only a small canonical core-identity helper needed to eliminate spelling privilege.
- No new general constant evaluator, range analysis, theorem prover, or alternate IR.
- Close each class of bug with a law-level test and a real-source/incremental composition test.

---

# 1. Fresh Repository Grounding and Confirmed Defects

This plan is rebuilt from source. The following are concrete observations at the grounding commit.

## 1.1 Generic-supertype specialization has a real TypeStore domain hazard

In `phalcom-semantic/src/types/relation.rs`, generic-supertype relation checking currently performs the equivalent of:

```rust
let specialized_super =
    TypeView::new(template.supertype, env).materialize(&mut store.clone());

check_subtype_impl(store, hierarchy, specialized_super, sup, ...);
```

`TypeStore::clone()` preserves the store ID and copies the arena, but interning a specialization in the clone can allocate a new numeric `TypeId` that does not exist in the original arena—or whose numeric slot later denotes a different type there. The resulting ID is then interpreted through `store`, the original.

This is not merely a hypothetical design concern. Part 06 must remove clone-materialization from relation solving and add regressions where specialization requires a type not already interned in the original store.

## 1.2 Relation code contains spelling-based core privilege

Current relation logic contains name tests equivalent to:

```rust
sup_decl.name.as_ref() == "Function"
sup_decl.name.as_ref() == "Closure"
sup_decl.name.as_ref() == "Object"
```

to prove callable/object subtyping.

A non-core declaration with the same spelling can therefore acquire special relation behavior. This violates the repository's own canonical-identity architecture.

## 1.3 Literal/collection semantic roots use ordinary name resolution

Current expression synthesis resolves primitive/container semantic owners via calls such as:

```rust
ctx.resolve_type_name("Int")
ctx.resolve_type_name("Float")
ctx.resolve_type_name("String")
ctx.resolve_type_name("Bool")
ctx.resolve_type_name("List")
ctx.resolve_type_name("Set")
ctx.resolve_type_name("Map")
ctx.resolve_type_name("Object")
```

Ordinary source name resolution is the right tool for source references; it is not the right authority for compiler-owned literal and builtin semantics if shadowing can redirect the name.

## 1.4 Comparison chains invent `Established<Bool>`

`Expr::ComparisonChain` currently analyzes operands and then returns established Bool without proving each relation operation.

The repository already has a better mechanism:
- binary operators resolve through canonical call/dispatch machinery;
- `checker/call.rs` already supports `ApplicationArgument::PreAnalyzed`, specifically so a previously analyzed operand can participate in a call without a second traversal.

Part 06 should reuse that machinery and preserve one evaluation per chain operand.

## 1.5 Membership nodes also invent `Established<Bool>`

`Expr::Membership` and `Expr::IsMembership` currently analyze operands/candidates and then stamp established Bool.

The fresh repository pass confirms explicit AST nodes, but does not establish a canonical selector/protocol that static semantics may safely invent. Therefore the closure rule is:

> Remove the false proof immediately. Precise membership typing is permitted only through a canonical compiler/runtime semantic operation that exists independently of this checker patch. If no such operation is present after rebase, the formal result remains Unknown rather than silently defining language semantics here.

This is a deliberate soundness boundary, not unfinished implementation.

## 1.6 Empty collections ignore usable expected element context

Collection synthesis computes expected element/key/value context but returns `Unknown(NoTypeEvidence)` for empty literals rather than deriving a contextual collection fact.

Example target:

```phalcom
let xs: List<Int> = []
```

The empty literal can be usable under the `List<Int>` contract, but its authority is contextual/assumed; the literal itself did not observe an `Int`.

## 1.7 Production code has many `established(...)` construction sites

A fresh repository search finds establishment sites across:
- declaration types;
- calls;
- field lifecycle;
- body/control;
- declaration signatures;
- composition;
- context;
- statement/expression synthesis;
- native surfaces.

Part 06 must classify every production site and make the classification reviewable.

## 1.8 Current architecture already supplies the right global boundaries

Repository architecture docs state:
- one semantic world in `phalcom-semantic`;
- canonical semantic identities, not spelling;
- one persistent TypeStore per semantic session;
- immutable coherent snapshots;
- advisory/formal authority separation;
- clean/incremental equivalence as a correctness criterion.

Part 06 enforces those principles locally rather than adding another layer.

---

# 2. Normative Closure Laws

### C1 — Canonical identity law

Compiler-owned semantic privilege requires exact canonical identity. Name equality is never sufficient.

### C2 — Core shadowing law

A user declaration named like a core declaration must behave as an ordinary user declaration unless explicitly referenced by its canonical module identity.

### C3 — TypeStore materialization law

Every materialized `TypeId` consumed by a relation/analysis must be interned in the active canonical store used to interpret it.

### C4 — Clone isolation law

A retained snapshot/store clone may preserve historical denotations, but no analysis may export a newly interned ID from a clone back into the live store domain.

### C5 — Relation non-strengthening law

A proven relation can validate compatibility, not establish the authority of its input facts.

### C6 — Syntax non-proof law

Parser recognition of a comparison, membership operation, subscript, property, or message-like construct does not by itself establish the operation's result type if the operation depends on semantic resolution.

### C7 — Single-evaluation chain law

Each source operand of a comparison chain receives exactly one formal expression analysis and one runtime evaluation semantics.

### C8 — Chain conjunction law

A comparison chain is formally known only to the strength supported by all required link operations. One unknown/invalid link prevents a clean established chain result.

### C9 — Membership fail-closed law

Membership syntax cannot publish Established Bool unless a canonical semantic operation proves a Bool-producing result. Unknown protocol → Unknown formal result.

### C10 — Contextual-empty law

An empty collection may use a compatible expected applied collection type as contextual evidence, but the resulting known fact is no stronger than the expected context.

### C11 — Wrong-context law

Expected `Map<...>` cannot make `[]` a `List<...>`; expected types constrain compatible constructors only.

### C12 — Non-empty evidence precedence law

When literal elements exist, value evidence remains primary. Expected context constrains/checks it; it does not overwrite contradictory established element evidence.

### C13 — Establishment accountability law

Every production `TypeKnowledge::established` / `TypedExpression::established` construction site must be classifiable under one approved evidence category.

### C14 — Unknown quarantine law

A recovery value may retain useful actual type information for diagnostics, but invalid/blocked/recursive evidence cannot be republished as an established formal fact.

### C15 — Incremental equivalence law

After hardening, clean recomputation and incremental recomputation produce semantically equivalent formal products and canonical identities.

### C16 — Presentation non-authority law

Hover, diagnostics, source index, and LSP projection consume formal products. They cannot repair or reinterpret an unknown semantic result as known.

### C17 — No hidden language design law

If the repository has no ratified/compiler-owned operation semantics for a syntax form, the checker fails closed and the missing language contract is surfaced explicitly; the checker does not create that contract implicitly.

---

# 3. Target Architecture

## 3.1 Canonical core identities

Create:

`phalcom-semantic/src/core_surface/identity.rs`

```rust
use crate::identity::{CallableId, DeclarationId, DispatchSide};
use crate::ModuleId;
use phalcom_common::selector::{Selector, SelectorSlot};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CoreDeclarationIds {
    pub object: DeclarationId,
    pub bool_: DeclarationId,
    pub int: DeclarationId,
    pub float: DeclarationId,
    pub string: DeclarationId,
    pub symbol: DeclarationId,
    pub number: DeclarationId,
    pub list: DeclarationId,
    pub set: DeclarationId,
    pub map: DeclarationId,
    pub function: DeclarationId,
    pub closure: DeclarationId,
}

impl Default for CoreDeclarationIds {
    fn default() -> Self {
        let module = ModuleId::core();
        Self {
            object: DeclarationId::new(module.clone(), "Object".into()),
            bool_: DeclarationId::new(module.clone(), "Bool".into()),
            int: DeclarationId::new(module.clone(), "Int".into()),
            float: DeclarationId::new(module.clone(), "Float".into()),
            string: DeclarationId::new(module.clone(), "String".into()),
            symbol: DeclarationId::new(module.clone(), "Symbol".into()),
            number: DeclarationId::new(module.clone(), "Number".into()),
            list: DeclarationId::new(module.clone(), "List".into()),
            set: DeclarationId::new(module.clone(), "Set".into()),
            map: DeclarationId::new(module.clone(), "Map".into()),
            function: DeclarationId::new(module.clone(), "Function".into()),
            closure: DeclarationId::new(module, "Closure".into()),
        }
    }
}

impl CoreDeclarationIds {
    pub fn is_object(&self, declaration: &DeclarationId) -> bool {
        declaration == &self.object
    }

    pub fn is_callable_supertype(&self, declaration: &DeclarationId) -> bool {
        declaration == &self.function
            || declaration == &self.closure
            || declaration == &self.object
    }
}
```

If the actual canonical universe uses a different exact owner for one of these declarations after rebase, construct IDs from the session's canonical bootstrap declarations instead of changing names ad hoc. The important rule is exact `DeclarationId`.

Export the module through `core_surface/mod.rs`.

## 3.2 CheckingContext builtin access

Add one immutable field:

```rust
pub core_ids: CoreDeclarationIds,
```

or a zero-allocation accessor:

```rust
pub fn core_ids(&self) -> CoreDeclarationIds;
```

Prefer a stored value if it avoids repeatedly constructing boxed declaration names in hot expression paths.

Add:

```rust
pub(crate) fn core_type(&mut self, declaration: &DeclarationId) -> Option<TypeId> {
    self.declarations
        .form(declaration)
        .or_else(|| self.declaration_info(declaration).map(|info| info.instance_type))
}
```

Use the actual current declaration-table API; do not call ordinary source resolution for compiler-owned literal semantics.

## 3.3 Mutable canonical relation API

The generic-supertype specialization fix requires relation materialization into the live store.

Target signatures:

```rust
pub fn check_subtype_bounded(
    store: &mut TypeStore,
    hierarchy: &dyn TypeHierarchy,
    sub: TypeId,
    sup: TypeId,
    budget: &mut QueryBudget,
    cancellation: &CancellationToken,
) -> RelationOutcome;

fn check_subtype_impl(
    store: &mut TypeStore,
    ...
) -> RelationOutcome;
```

and propagate `&mut TypeStore` through public assignability/knowledge relation functions that can recurse into subtype specialization.

Before recursive matching:

```rust
let sub_data = store.get(sub).clone();
let sup_data = store.get(sup).clone();
```

so the immutable borrow does not overlap materialization.

Replace:

```rust
materialize(&mut store.clone())
```

with:

```rust
materialize(store)
```

No clone-generated TypeId crosses the relation boundary.

## 3.4 Pre-analyzed operation links

`checker/call.rs` already has:

```rust
ApplicationArgument::PreAnalyzed {
    label: Option<&str>,
    typed: &TypedExpression,
    range: SourceRange,
}
```

Use it.

Extract binary-operation application from `synthesize_binary_expr` so both ordinary binary syntax and comparison-chain links can call:

```rust
pub(crate) fn apply_binary_operation_from_typed(
    ctx: &mut CheckingContext<'_>,
    left_expr: &Expr,
    left: &TypedExpression,
    op: BinaryOp,
    right_expr: &Expr,
    right: &TypedExpression,
    range: SourceRange,
) -> TypedExpression;
```

This helper resolves direct/reflected dispatch exactly once without re-analyzing `left` or `right`.

## 3.5 Comparison-chain result composition

Analyze all operands once, left-to-right:

```rust
let operands: Vec<TypedExpression> = chain
    .operands
    .iter()
    .map(|expr| analyze_expression(ctx, expr, &ExpectedType::None))
    .collect();
```

For each `(left, op, right)` link, apply a semantic operation using pre-analyzed arguments.

For binary relation operators, reuse binary dispatch.

For `RelationOp::Matches` / `RelationOp::Understands`, route through the canonical operation implementation already used by non-chain syntax if present. If the rebase still has no canonical formal operation, that link is `Unknown(UncheckedExpression)`; do not stamp Bool.

Compose required link knowledge:

```text
all links clean known Bool:
    result authority = minimum link authority
any Unknown:
    Unknown(joined reason)
any Dynamic:
    Dynamic(joined reason)
invalid causal dependency:
    keep recovery type if useful, but result status/causal invalidity prevents formal proof publication
```

## 3.6 Membership resolution boundary

Introduce a focused helper in `expression.rs`:

```rust
fn synthesize_membership_expr(
    ctx: &mut CheckingContext<'_>,
    membership: &MembershipExpr,
) -> TypedExpression;

fn synthesize_is_membership_expr(
    ctx: &mut CheckingContext<'_>,
    membership: &IsMembershipExpr,
) -> TypedExpression;
```

Implementation rule:

1. analyze source operands once;
2. ask only an existing canonical semantic operation/dispatch implementation for a result;
3. propagate that operation's return knowledge/authority/causal invalidity;
4. apply syntactic negation as Bool-preserving only when the positive operation is formally Bool-known;
5. if no canonical operation exists in the post-Part-05 repository, return `Unknown(UncheckedExpression)` plus child dependencies; never fabricate `Established<Bool>`.

Do not define a `contains`/`includes` selector inside this task unless compiler/runtime source already defines that exact contract.

## 3.7 Contextual empty collections

For expected compatible applied collection type:

```text
expected: List<Int>
source: []
```

derive:

```rust
TypeKnowledge::assumed(list_int, EvidenceOrigin::ContextualDerivation)
```

if the expected contract itself is Assumed, or preserve Established only when the expected context is compiler-established rather than merely developer-supplied.

The cleanest API is to retain expected authority:

```rust
impl ExpectedType {
    pub(crate) fn contextual_knowledge(&self, ty: TypeId) -> Option<TypeKnowledge>;
}
```

Do not reconstruct an expected `TypeId` as established.

---

# 4. File Change Map

| File | Responsibility in Part 06 |
| --- | --- |
| `phalcom-semantic/src/core_surface/identity.rs` | **create** canonical core declaration identities |
| `phalcom-semantic/src/core_surface/mod.rs` | export identity helpers |
| `phalcom-semantic/src/checker/context.rs` | canonical core identity/type access |
| `phalcom-semantic/src/types/relation.rs` | remove spelling privilege; materialize generic supertypes in live store |
| `phalcom-semantic/src/types/environment.rs` | change only if materialization API needs an explicit live-store contract |
| `phalcom-semantic/src/checker/binding.rs` | propagate mutable relation store signatures; preserve authority |
| `phalcom-semantic/src/checker/flow/transfer.rs` | mutable relation call adaptation only; no new predicate architecture |
| `phalcom-semantic/src/checker/expression.rs` | canonical literals/collections; comparison chains; membership fail-closed/operation routing |
| `phalcom-semantic/src/checker/call.rs` | expose/reuse pre-analyzed operation application if needed; no second call engine |
| `phalcom-semantic/src/checker/expected.rs` | contextual expected-type evidence helper |
| `phalcom-semantic/src/checker/composition.rs` | authority-preserving composition only if empty collection helper belongs here |
| `phalcom-semantic/src/db/fingerprint.rs` | include semantic results changed by hardened identities if missing |
| `phalcom-semantic/tests/semantic/foundations/authority_boundaries.rs` | **create** cross-cutting proof-law tests |
| `phalcom-semantic/tests/semantic/capabilities/structural.rs` | contextual collections |
| `phalcom-semantic/tests/semantic/capabilities/dispatch.rs` or current dispatch module | core-shadow/operation dispatch tests |
| `phalcom-semantic/tests/semantic/capabilities/generics.rs` | generic-supertype relation regression |
| `phalcom-semantic/tests/semantic/incremental/type_store_revisions.rs` | active-store specialization stability |
| `phalcom-semantic/tests/semantic/incremental/fingerprints.rs` | clean/incremental semantic identity |
| `phalcom-semantic/tests/semantic/COVERAGE_LEDGER.md` | final closure ledger |
| `docs/impl/semantic/semantic-correctness/part-4/2026-08-29-semantic-authority-audit.md` | **create** production establishment/relation audit |

---

# 5. Implementation Tasks

## Task 0 — Post-Part-05 Baseline and Closure RED Probes

**Files:**
- Create: `phalcom-semantic/tests/semantic/foundations/authority_boundaries.rs`
- Modify: `phalcom-semantic/tests/semantic/foundations/mod.rs`
- Modify targeted capability tests

**Interfaces:**
- Consumes: complete Parts 01–05.
- Produces: failing tests for every concrete Part-06 defect before implementation.

- [ ] **Step 1: Record actual base and run full semantic baseline**

```bash
git rev-parse HEAD
git status --short
cargo fmt --all -- --check
cargo check -p phalcom-semantic
cargo test -p phalcom-semantic
```

- [ ] **Step 2: Add RED core-shadow tests**

Create source containing user declarations named:
- `Object`
- `Function`
- `Closure`
- `Bool`
- `Int`
- `List`

Assert ordinary user declarations do not receive core subtype/literal/container privilege.

At minimum:

```rust
#[test]
fn user_object_name_is_not_universal_supertype() {
    let f = Fixture::new(
        r#"
class Object {}
class Unrelated {}
class Probe {
  @class
  run(_ value: Unrelated) {
    let x: Object = value
  }
}
"#,
    );
    f.assert_diagnostic(DiagnosticCode::BindingInitializerMismatch);
}
```

Use module scoping/import syntax as needed so the fixture can distinguish user `Object` from core `Object`.

- [ ] **Step 3: Add RED generic-supertype live-store test**

Build a generic subtype relation whose specialized parent type is not pre-interned in the active store before the relation check. Assert:
- relation is correct;
- no invalid `TypeId` lookup/panic;
- active store type count increases only by canonical specialization interning;
- repeat relation does not increase count again.

- [ ] **Step 4: Add RED comparison-chain operation test**

Use a class whose `<` selector is absent or refuted. Assert `a < b < c` is not a clean established Bool.

- [ ] **Step 5: Add RED single-evaluation chain test**

Use method-call operands with distinct expression sites. Assert every chain operand has one public `ExpressionAnalysis`, even though middle operands participate in two links.

- [ ] **Step 6: Add RED membership proof test**

For a source membership expression whose right-hand type has no established canonical membership operation, assert result is not `Established<Bool>`.

- [ ] **Step 7: Add RED contextual-empty tests**

```phalcom
let xs: List<Int> = []
let ys = []
```

Expected:
- `xs` is usable as `List<Int>` with contextual/assumed authority;
- `ys` remains `Unknown(NoTypeEvidence)`.

Add Set/Map equivalents.

- [ ] **Step 8: Run RED tests**

```bash
cargo test -p phalcom-semantic semantic::foundations::authority_boundaries -- --nocapture
cargo test -p phalcom-semantic semantic::capabilities::generics -- --nocapture
cargo test -p phalcom-semantic semantic::capabilities::structural -- --nocapture
```

- [ ] **Step 9: Commit tests**

```bash
git add phalcom-semantic/tests/semantic
git commit -m "test(semantic): pin final authority and identity closure laws"
```

---

## Task 1 — Canonical Core Declaration Identity

**Files:**
- Create: `phalcom-semantic/src/core_surface/identity.rs`
- Modify: `phalcom-semantic/src/core_surface/mod.rs`
- Modify: `phalcom-semantic/src/checker/context.rs`
- Test: `phalcom-semantic/tests/semantic/foundations/authority_boundaries.rs`

**Interfaces:**
- Produces:
  - `CoreDeclarationIds`
  - exact identity predicates
  - canonical type access from `CheckingContext`

- [ ] **Step 1: Add low-level identity tests**

Assert:

```rust
let core = CoreDeclarationIds::default();
assert!(core.is_object(&DeclarationId::new(ModuleId::core(), "Object".into())));
assert!(!core.is_object(&DeclarationId::new(user_module, "Object".into())));
```

Add Function/Closure tests.

- [ ] **Step 2: Implement `CoreDeclarationIds`**

Use exact `ModuleId::core()` + declaration identity from the canonical universe. Keep this file free of type checking/dispatch logic.

- [ ] **Step 3: Export from `core_surface/mod.rs`**

```rust
mod identity;
pub use identity::CoreDeclarationIds;
```

- [ ] **Step 4: Attach canonical IDs to checking context**

Initialize them once in all `CheckingContext` constructors.

- [ ] **Step 5: Add canonical type lookup**

Use declaration tables/bootstrap products, not ordinary source-name resolution.

- [ ] **Step 6: Run**

```bash
cargo test -p phalcom-semantic semantic::foundations::authority_boundaries -- --nocapture
cargo check -p phalcom-semantic
```

- [ ] **Step 7: Commit**

```bash
git add phalcom-semantic/src/core_surface \
        phalcom-semantic/src/checker/context.rs \
        phalcom-semantic/tests/semantic/foundations/authority_boundaries.rs
git commit -m "feat(semantic): centralize canonical core identities"
```

---

## Task 2 — Repair Generic-Supertype TypeStore Ownership

**Files:**
- Modify: `phalcom-semantic/src/types/relation.rs`
- Modify callers:
  - `phalcom-semantic/src/checker/context.rs`
  - `phalcom-semantic/src/checker/binding.rs`
  - `phalcom-semantic/src/checker/flow/transfer.rs`
  - any compiler-owned relation callers found by `cargo check`
- Test:
  - `phalcom-semantic/tests/semantic/capabilities/generics.rs`
  - `phalcom-semantic/tests/semantic/incremental/type_store_revisions.rs`

**Interfaces:**
- Consumes: one active mutable `TypeStore`.
- Produces: relation APIs that materialize specialized supertypes in that same store.

- [ ] **Step 1: Add direct unit regression in `types/relation.rs`**

Create a generic declaration/template where `Sub<Int>` specializes to a parent applied type that is absent from the original store before the check. Record `type_count()`.

Assert subtype succeeds and `store.get(specialized_id)` is the expected applied parent.

- [ ] **Step 2: Make subtype solver accept `&mut TypeStore`**

Change the bounded and recursive subtype functions. Clone `TypeData` before recursive calls:

```rust
let sub_data = store.get(sub).clone();
let sup_data = store.get(sup).clone();
```

- [ ] **Step 3: Remove clone materialization**

Replace:

```rust
TypeView::new(template.supertype, env).materialize(&mut store.clone())
```

with:

```rust
TypeView::new(template.supertype, env).materialize(store)
```

- [ ] **Step 4: Propagate mutability through assignability APIs**

Update:
- `check_assignability_bounded`
- `check_knowledge_against_type_bounded`
- convenience wrappers that can traverse generic supertypes
- checker callers

Do not clone `TypeStore` to avoid signature changes.

- [ ] **Step 5: Preserve read-only callers where provably safe**

If a public query currently has only `&TypeStore` and cannot materialize, either:
- route it through a caller that owns `&mut TypeStore`; or
- split a genuinely read-only fast path from the materializing path.

Do not use interior mutability merely to preserve an old signature.

- [ ] **Step 6: Add idempotence regression**

Run the same generic subtype relation twice. Assert the second run does not grow `type_count()`.

- [ ] **Step 7: Add wrong-specialization regression**

`Sub<Int>` must not subtype an incompatible `Parent<String>` if variance/arguments refute it.

- [ ] **Step 8: Run**

```bash
cargo test -p phalcom-semantic types::relation -- --nocapture
cargo test -p phalcom-semantic semantic::capabilities::generics -- --nocapture
cargo test -p phalcom-semantic semantic::incremental::type_store_revisions -- --nocapture
cargo check -p phalcom-semantic
```

- [ ] **Step 9: Commit**

```bash
git add phalcom-semantic/src/types/relation.rs \
        phalcom-semantic/src/checker \
        phalcom-semantic/tests/semantic/capabilities/generics.rs \
        phalcom-semantic/tests/semantic/incremental/type_store_revisions.rs
git commit -m "fix(semantic): materialize generic supertypes in canonical store"
```

---

## Task 3 — Remove Spelling-Based Relation Privilege

**Files:**
- Modify: `phalcom-semantic/src/types/relation.rs`
- Test: `phalcom-semantic/tests/semantic/foundations/authority_boundaries.rs`

**Interfaces:**
- Consumes: `CoreDeclarationIds`.
- Produces: exact core identity checks for universal/callable supertypes.

- [ ] **Step 1: Replace name comparisons**

Replace every semantic branch equivalent to:

```rust
decl.name.as_ref() == "Object"
```

with exact canonical identity.

For relation code without `CheckingContext`, instantiate/pass the small `CoreDeclarationIds` value or use exact helper functions from `core_surface::identity`.

- [ ] **Step 2: Harden callable supertype relation**

Only canonical:
- core Function;
- core Closure;
- core Object

receive the special callable-supertype relation.

- [ ] **Step 3: Harden universal Object relation**

Only canonical core Object is the universal nominal supertype under this rule.

- [ ] **Step 4: Search for additional semantic spelling privilege**

Run:

```bash
rg -n '\.name(\.as_ref\(\))?\s*==|name\.as_str\(\)\s*==' phalcom-semantic/src
rg -n 'resolve_type_name\("(Object|Function|Closure|Bool|Int|Float|String|List|Set|Map|Number)"\)' phalcom-semantic/src
```

Classify each hit:
- source-level lookup: allowed;
- compiler-owned semantic privilege: replace with canonical identity;
- presentation-only formatting: allowed.

- [ ] **Step 5: Run core-shadow regressions**

```bash
cargo test -p phalcom-semantic semantic::foundations::authority_boundaries -- --nocapture
cargo test -p phalcom-semantic types::relation -- --nocapture
```

- [ ] **Step 6: Commit**

```bash
git add phalcom-semantic/src/types/relation.rs \
        phalcom-semantic/tests/semantic/foundations/authority_boundaries.rs
git commit -m "fix(semantic): require canonical identity for core privilege"
```

---

## Task 4 — Canonical Literal and Collection Roots

**Files:**
- Modify: `phalcom-semantic/src/checker/expression.rs`
- Modify: `phalcom-semantic/src/checker/context.rs` only for helper access
- Test: `phalcom-semantic/tests/semantic/foundations/authority_boundaries.rs`
- Test: `phalcom-semantic/tests/semantic/capabilities/structural.rs`

**Interfaces:**
- Consumes: Task-01 core IDs.
- Produces: literal/container types that cannot be hijacked by source shadowing.

- [ ] **Step 1: Add literal-shadow RED tests**

Where source/module syntax permits a user declaration with the same spelling, prove integer/string/bool literals still denote canonical core types rather than the shadowing declaration.

- [ ] **Step 2: Replace primitive literal source-name resolution**

For `Expr::Int`, `Float`, `String`, `Boolean`, resolve canonical declaration IDs directly.

Preserve `EvidenceOrigin::Syntax`.

- [ ] **Step 3: Harden Symbol fallback**

If Symbol is canonical and present, use canonical Symbol. If the language deliberately falls back to canonical String in bootstrap mode, make that exact fallback explicit; do not use a shadowable source `String`.

- [ ] **Step 4: Replace collection constructor roots**

`ListLiteral`, `SetLiteral`, `MapLiteral` use canonical List/Set/Map declaration forms.

- [ ] **Step 5: Replace compiler-owned Object fallback**

Block/callable construction that needs the top type uses canonical core Object, not source name resolution.

- [ ] **Step 6: Preserve source references**

Do **not** replace ordinary `Expr::Var { value: "List" }` resolution. A source identifier should still resolve lexically. This task affects compiler-owned semantics, not source-name lookup.

- [ ] **Step 7: Run**

```bash
cargo test -p phalcom-semantic semantic::foundations::authority_boundaries -- --nocapture
cargo test -p phalcom-semantic semantic::capabilities::structural -- --nocapture
```

- [ ] **Step 8: Commit**

```bash
git add phalcom-semantic/src/checker/expression.rs \
        phalcom-semantic/src/checker/context.rs \
        phalcom-semantic/tests/semantic
git commit -m "fix(semantic): anchor literals and collections to core identity"
```

---

## Task 5 — Comparison Chains Use Real Operation Evidence

**Files:**
- Modify: `phalcom-semantic/src/checker/expression.rs`
- Modify: `phalcom-semantic/src/checker/call.rs` only if the pre-analyzed helper needs visibility/refactoring
- Test: `phalcom-semantic/tests/semantic/capabilities/dispatch.rs` or the repository's actual dispatch test module
- Test: `phalcom-semantic/tests/semantic/foundations/authority_boundaries.rs`

**Interfaces:**
- Consumes:
  - current binary direct/reflected dispatch;
  - `ApplicationArgument::PreAnalyzed`.
- Produces:
  - `apply_binary_operation_from_typed(...)`
  - operation-backed `ComparisonChain`.

- [ ] **Step 1: Extract binary application from traversal**

Refactor `synthesize_binary_expr`:

```rust
fn synthesize_binary_expr(ctx: &mut CheckingContext<'_>, binary: &BinaryExpr) -> TypedExpression {
    let left = analyze_expression(ctx, &binary.left, &ExpectedType::None);
    let right = analyze_expression(ctx, &binary.right, &ExpectedType::None);
    apply_binary_operation_from_typed(
        ctx,
        &binary.left,
        &left,
        binary.op.clone(),
        &binary.right,
        &right,
        binary.range,
    )
}
```

Preserve the current bilateral/reflected dispatch rule and `from` labeling exactly.

- [ ] **Step 2: Add ordinary-binary parity test**

Before changing comparison chains, assert `1 < 2` still resolves the same callable/knowledge/causal status as before.

- [ ] **Step 3: Analyze chain operands once**

Store one `TypedExpression` per source operand.

- [ ] **Step 4: Apply each link from pre-analyzed operands**

For every `RelationOp::Binary(op)`, call the extracted helper.

No middle operand is traversed a second time.

- [ ] **Step 5: Handle `Matches` / `Understands` without invention**

Route through their existing canonical formal operation if present after rebase. If formal resolution is absent, produce an unknown link rather than assumed/established Bool.

- [ ] **Step 6: Compose all required link results**

Use `compose_required_knowledge`-style weakest-premise semantics. The chain result can be Established Bool only if every link is a clean Established Bool-producing semantic operation.

If a resolved operator returns a non-Bool type contrary to the language's relation contract, emit/use the existing operation/type diagnostic rather than force-casting it to Bool.

- [ ] **Step 7: Preserve causal invalidity**

Join causal invalidity from all source operands and all link applications. Invalid recovery knowledge cannot turn into a clean chain proof.

- [ ] **Step 8: Run**

```bash
cargo test -p phalcom-semantic semantic::foundations::authority_boundaries -- --nocapture
cargo test -p phalcom-semantic semantic::capabilities::dispatch -- --nocapture
```

Use the actual dispatch module name shown by `tests/semantic/capabilities/mod.rs` if it differs.

- [ ] **Step 9: Commit**

```bash
git add phalcom-semantic/src/checker/expression.rs \
        phalcom-semantic/src/checker/call.rs \
        phalcom-semantic/tests/semantic
git commit -m "fix(semantic): derive comparison chains from operator dispatch"
```

---

## Task 6 — Membership Fails Closed Unless Canonical Semantics Exist

**Files:**
- Modify: `phalcom-semantic/src/checker/expression.rs`
- Test: `phalcom-semantic/tests/semantic/foundations/authority_boundaries.rs`
- Test: a dedicated `membership.rs` capability module **only if** post-rebase compiler/runtime semantics expose a canonical membership operation

**Interfaces:**
- Produces:
  - `synthesize_membership_expr`
  - `synthesize_is_membership_expr`
  - no invented selector.

- [ ] **Step 1: Pin current compiler contract with a repository search**

Run:

```bash
rg -n "Membership|IsMembership|not in|is! in|is not in" \
    phalcom-ast phalcom-core phalcom-semantic docs
```

Record the exact compiler/runtime semantic operation if one exists after Part 05.

At the grounding commit, explicit AST/checker nodes exist but no canonical operation was established by this pass. Therefore the default implementation branch below is fail-closed.

- [ ] **Step 2: Replace unconditional Bool stamping**

Move membership cases to the focused helpers. Analyze operands once and propagate their causal dependencies.

- [ ] **Step 3: Implement the canonical-operation branch only if independently present**

If the search identifies an existing compiler/runtime operation:
- construct/use its exact canonical selector/identity;
- resolve it through the existing call engine;
- preserve return authority;
- require formal Bool result for Bool-producing membership syntax;
- apply `negated` without strengthening authority.

The operation name comes from the compiler/runtime contract, not from this plan.

- [ ] **Step 4: Implement the grounded fail-closed branch**

When no canonical operation exists:

```rust
let mut result = TypedExpression::unknown(UnknownReason::UncheckedExpression);
result.causal_invalidity =
    left.causal_invalidity.join(right_or_candidates.causal_invalidity);
crate::checker::composition::propagate_required_dependencies(
    &mut result,
    &[left, right_or_candidates],
);
result
```

Use the existing dependency propagation helper's actual signature.

- [ ] **Step 5: Verify no false proof**

Assert unsupported membership is Unknown, not Dynamic and not Established Bool. Dynamic is reserved for deliberate dynamic escapes, not missing implementation.

- [ ] **Step 6: Verify negation does not create proof**

`not in` / negated `is in` over an unknown positive membership operation remains Unknown.

- [ ] **Step 7: Run**

```bash
cargo test -p phalcom-semantic semantic::foundations::authority_boundaries -- --nocapture
cargo check -p phalcom-semantic
```

- [ ] **Step 8: Commit**

```bash
git add phalcom-semantic/src/checker/expression.rs \
        phalcom-semantic/tests/semantic/foundations/authority_boundaries.rs
git commit -m "fix(semantic): stop fabricating membership result proofs"
```

---

## Task 7 — Contextual Typing for Empty Collections

**Files:**
- Modify: `phalcom-semantic/src/checker/expected.rs`
- Modify: `phalcom-semantic/src/checker/expression.rs`
- Modify: `phalcom-semantic/src/checker/composition.rs` only if shared applied-type projection belongs there
- Test: `phalcom-semantic/tests/semantic/capabilities/structural.rs`
- Test: `phalcom-semantic/tests/semantic/foundations/authority_boundaries.rs`

**Interfaces:**
- Produces:
  - authority-preserving expected contextual knowledge
  - empty List/Set/Map typing.

- [ ] **Step 1: Add RED List tests**

Cases:
1. `let xs: List<Int> = []` → current value usable as `List<Int>`, no error, authority no stronger than source contract.
2. `let xs = []` → Unknown NoTypeEvidence.
3. expected `Set<Int>` does not type `[]` as List.
4. an established compiler-owned expected List context remains established if the context itself is established.
5. an assumed developer context remains assumed.

- [ ] **Step 2: Add RED Set/Map tests**

Map must preserve both key/value generic arguments. Empty map cannot infer either without context.

- [ ] **Step 3: Implement `ExpectedType::contextual_knowledge`**

Do not turn a bare expected `TypeId` into Established. Carry expected authority/provenance under `EvidenceOrigin::ContextualDerivation`.

If `ExpectedType` currently stores only a `TypeId` + origin and lacks authority, extend it with the minimum evidence status needed to preserve Part-01 law. Update constructors:
- declaration/source contract → Assumed;
- compiler-established expectation → Established;
- dynamic/unknown expectations remain non-concrete.

- [ ] **Step 4: Match expected constructor identity**

For empty list:
- expected type must be an application whose origin is canonical List form.
For Set/Map analogous.

Do not rely on declaration name.

- [ ] **Step 5: Build empty result from the compatible expected applied type**

Do not synthesize an element union from zero members. The contextual fact is the entire expected applied collection type.

- [ ] **Step 6: Preserve non-empty behavior**

Non-empty collection literals continue to derive element/key/value types from actual value evidence. Expected context only checks/constrains; it does not overwrite a conflicting established literal member.

- [ ] **Step 7: Run**

```bash
cargo test -p phalcom-semantic semantic::capabilities::structural -- --nocapture
cargo test -p phalcom-semantic semantic::foundations::authority_boundaries -- --nocapture
```

- [ ] **Step 8: Commit**

```bash
git add phalcom-semantic/src/checker/expected.rs \
        phalcom-semantic/src/checker/expression.rs \
        phalcom-semantic/src/checker/composition.rs \
        phalcom-semantic/tests/semantic
git commit -m "feat(semantic): type empty collections from contextual evidence"
```

---

## Task 8 — Production `Established` Site Audit

**Files:**
- Create: `docs/impl/semantic/semantic-correctness/part-4/2026-08-29-semantic-authority-audit.md`
- Modify production files only where audit finds an unjustified establishment
- Modify: `phalcom-semantic/tests/semantic/foundations/authority_boundaries.rs`

**Interfaces:**
- Consumes: Parts 01–07.
- Produces: a complete reviewable inventory of formal proof construction.

- [ ] **Step 1: Generate the establishment inventory**

Run:

```bash
rg -n \
  'TypeKnowledge::established|TypedExpression::established|EvidenceStatus::Established' \
  phalcom-semantic/src \
  > /tmp/phalcom-established-sites.txt
```

Exclude test-only sites from the production audit.

- [ ] **Step 2: Classify every site in the audit document**

For each production hit, record:
- file:line;
- expression/function;
- type established;
- authority source;
- approved category;
- invariant test.

Approved categories:

```text
literal/compiler syntax semantics
canonical declaration semantics
constructor semantics
trusted native signature
validated callable publication
established generic inference
authority-preserving composition from all-established premises
trusted runtime/type-test observation
validated field lifecycle
canonical structural builtin with compiler-owned semantics
```

A site that does not fit one category is not automatically accepted.

- [ ] **Step 3: Fix unclassified sites**

For each unclassified site:
- replace with authority-preserving composition;
- downgrade to Assumed;
- return Unknown/Dynamic where appropriate;
- or route through canonical semantic resolution.

Do not create a new category merely to justify existing code.

- [ ] **Step 4: Add a negative invariant test for each repaired class**

Examples:
- assumed premise cannot produce established result;
- invalid recovery cannot produce established result;
- unknown operation cannot produce established result;
- spelling match cannot produce established result.

- [ ] **Step 5: Re-run inventory after repairs**

The audit document must cover every remaining production establishment site at the final Part-06 commit.

- [ ] **Step 6: Commit**

```bash
git add docs/impl/semantic/semantic-correctness/part-4/2026-08-29-semantic-authority-audit.md \
        phalcom-semantic/src \
        phalcom-semantic/tests/semantic/foundations/authority_boundaries.rs
git commit -m "audit(semantic): account for every established proof site"
```

---

## Task 9 — `RelationOutcome::Proven` Consumer Audit

**Files:**
- Extend: `docs/impl/semantic/semantic-correctness/part-4/2026-08-29-semantic-authority-audit.md`
- Modify relation consumers only where a law violation is found
- Test: `phalcom-semantic/tests/semantic/foundations/authority_boundaries.rs`

**Interfaces:**
- Produces: proof that relation success cannot launder evidence authority.

- [ ] **Step 1: Inventory relation-success consumers**

Run:

```bash
rg -n 'RelationOutcome::Proven|\.is_proven\(\)|is_subtype\(' phalcom-semantic/src
```

- [ ] **Step 2: Classify each consumer**

Allowed:
- compatibility validation;
- control contradiction against established premises;
- dispatch applicability;
- structural normalization that preserves premise status.

Forbidden:
- reconstructing an assumed known type as Established;
- replacing recovery current knowledge with declared contract solely because assignability is proven;
- establishing a return/field/binding from relation success without an independent established premise.

- [ ] **Step 3: Add direct law test**

At minimum:

```rust
#[test]
fn proven_subtyping_does_not_upgrade_assumed_actual() {
    // assumed Int, relation Int <: Number is proven
    // resulting reconciled current evidence remains Assumed Int
}
```

- [ ] **Step 4: Repair violations**

Use Part-01 `EvidenceStatus::meet`/weakest-premise helper under its actual final name. Do not add local status-min functions in each consumer.

- [ ] **Step 5: Run**

```bash
cargo test -p phalcom-semantic semantic::foundations::authority_boundaries -- --nocapture
cargo test -p phalcom-semantic
```

- [ ] **Step 6: Commit**

```bash
git add docs/impl/semantic/semantic-correctness/part-4/2026-08-29-semantic-authority-audit.md \
        phalcom-semantic/src \
        phalcom-semantic/tests/semantic/foundations/authority_boundaries.rs
git commit -m "audit(semantic): close relation proof laundering paths"
```

---

## Task 10 — Syntax Result-Invention Audit

**Files:**
- Extend authority audit document
- Modify: `phalcom-semantic/src/checker/expression.rs`
- Modify other expression helpers only where found
- Test relevant capability modules

**Interfaces:**
- Produces: no remaining “analyze children, then stamp a known result” paths without formal justification.

- [ ] **Step 1: Search expression result constructors**

Run:

```bash
rg -n 'analyze_expression\(.*\);|TypedExpression::established|TypeKnowledge::established' \
  phalcom-semantic/src/checker/expression.rs
```

Review every syntax arm that returns a known type after child analysis.

- [ ] **Step 2: Separate intrinsic syntax from semantic operations**

Intrinsic examples:
- integer/string/bool literal;
- canonical tuple/record construction whose constituents are all proved;
- compiler-owned Unit completion.

Operation-dependent examples:
- comparison;
- membership;
- message send;
- property/index access;
- reflective relation.

Operation-dependent results must come from their resolver/dispatch/structural semantic rule.

- [ ] **Step 3: Audit comparison/membership repairs**

Verify Tasks 05–06 removed the known unconditional Bool sites.

- [ ] **Step 4: Audit Range and other miscellaneous nodes**

If a node currently returns broad Object simply because its precise semantics are absent, determine whether that Object is a formally valid compiler-owned supertype fact or merely a convenience guess. If it is a guess, return Unknown instead.

- [ ] **Step 5: Add regressions for each repair**

Each repaired syntax form gets:
- valid known case;
- invalid/refuted case;
- unknown case;
- dynamic case where the language explicitly supports Dynamic;
- authority assertion.

- [ ] **Step 6: Commit**

```bash
git add phalcom-semantic/src/checker/expression.rs \
        phalcom-semantic/tests/semantic \
        docs/impl/semantic/semantic-correctness/part-4/2026-08-29-semantic-authority-audit.md
git commit -m "audit(semantic): remove residual syntax result invention"
```

---

## Task 11 — Canonical Identity and TypeStore Incremental Stability

**Files:**
- Modify:
  - `phalcom-semantic/tests/semantic/incremental/type_store_revisions.rs`
  - `phalcom-semantic/tests/semantic/incremental/fingerprints.rs`
  - `phalcom-semantic/tests/semantic/incremental/callable_dependencies.rs`
- Modify `db/fingerprint.rs` only for missing semantic fields

**Interfaces:**
- Consumes: canonical identity + relation fixes.
- Produces: clean/incremental equivalence proof.

- [ ] **Step 1: Add generic-specialization revision test**

Revision 1 introduces a generic subclass relation; revision 2 changes an unrelated body; revision 3 exercises the same relation again.

Assert:
- one session keeps one `TypeStoreId`;
- old snapshot denotations remain intact;
- live relation specializations are stable;
- unchanged specialization does not allocate a semantically new type.

- [ ] **Step 2: Add core-shadow edit test**

Add/remove a user `Object`/`List` declaration across revisions. Assert core literal/subtyping semantics and canonical IDs are unchanged.

- [ ] **Step 3: Add comparison-chain dependency test**

Change the signature of one resolved comparison operator. Assert only dependent callable products recompute and chain formal knowledge changes accordingly.

- [ ] **Step 4: Add contextual-empty edit test**

Change a binding contract `List<Int>` → `List<String>` while source remains `[]`. Assert the literal/contextual formal product invalidates and republishes with the new assumed type.

- [ ] **Step 5: Verify fingerprints include semantic authority**

A change from Established ↔ Assumed or Known ↔ Unknown is semantic and must affect the relevant product fingerprint even when `TypeId` stays the same.

- [ ] **Step 6: Run**

```bash
cargo test -p phalcom-semantic semantic::incremental::type_store_revisions -- --nocapture
cargo test -p phalcom-semantic semantic::incremental::fingerprints -- --nocapture
cargo test -p phalcom-semantic semantic::incremental::callable_dependencies -- --nocapture
```

- [ ] **Step 7: Commit**

```bash
git add phalcom-semantic/tests/semantic/incremental \
        phalcom-semantic/src/db/fingerprint.rs
git commit -m "test(semantic): prove hardened identity stable incrementally"
```

---

## Task 12 — Formal/Advisory/Editor Boundary Regression

**Files:**
- Test:
  - `phalcom-semantic/tests/semantic/integration/advisory_analysis.rs`
  - `phalcom-semantic/tests/semantic/integration/presentation.rs`
  - `phalcom-semantic/tests/semantic/integration/source_index.rs`
  - `phalcom-semantic/tests/semantic/integration/editor_type_hints.rs`
- No production changes unless a test reveals projection drift

**Interfaces:**
- Consumes: final formal semantics.
- Produces: proof downstream projections do not re-launder Unknown/Assumed into formal established facts.

- [ ] **Step 1: Add unknown-membership presentation test**

If membership remains fail-closed, assert formal presentation reports Unknown while advisory context, if any, stays explicitly advisory.

- [ ] **Step 2: Add contextual-empty presentation test**

Assert hover/type-hint projection can show the contextual `List<Int>` fact and its formal status without presenting it as compiler-established when source contract supplied the assumption.

- [ ] **Step 3: Add core-shadow source-index test**

A source reference to a user `Object` targets that user declaration; compiler literal semantics still target canonical core type identity. Source index must keep both identities distinct.

- [ ] **Step 4: Run**

```bash
cargo test -p phalcom-semantic semantic::integration::advisory_analysis -- --nocapture
cargo test -p phalcom-semantic semantic::integration::presentation -- --nocapture
cargo test -p phalcom-semantic semantic::integration::source_index -- --nocapture
cargo test -p phalcom-semantic semantic::integration::editor_type_hints -- --nocapture
```

- [ ] **Step 5: Commit**

```bash
git add phalcom-semantic/tests/semantic/integration
git commit -m "test(semantic): lock formal projection after hardening"
```

---

## Task 13 — Final Six-Part Closure Gate

**Files:**
- Modify:
  - `phalcom-semantic/tests/semantic/COVERAGE_LEDGER.md`
  - `phalcom-semantic/tests/semantic/capabilities/BASELINE_LEDGER.md`
  - authority audit document
- No production changes unless verification exposes a defect

**Interfaces:**
- Consumes: entire Parts 01–06 program.
- Produces: final semantic correctness closure record.

- [ ] **Step 1: Re-run production authority searches**

```bash
rg -n \
  'TypeKnowledge::established|TypedExpression::established|EvidenceStatus::Established' \
  phalcom-semantic/src

rg -n \
  'RelationOutcome::Proven|\.is_proven\(\)|is_subtype\(' \
  phalcom-semantic/src

rg -n \
  '\.name(\.as_ref\(\))?\s*==|resolve_type_name\("(Object|Function|Closure|Bool|Int|Float|String|List|Set|Map|Number)"\)' \
  phalcom-semantic/src
```

Every production hit must be represented in the audit classification or explicitly source/presentation-only.

- [ ] **Step 2: Search for cloned-store materialization**

```bash
rg -n 'materialize\(&mut .*clone\(\)|store\.clone\(\)' phalcom-semantic/src/types phalcom-semantic/src/checker
```

Expected: no path materializes a TypeId in a cloned store and transports it to live relation/checker code.

- [ ] **Step 3: Run formatting/checks**

```bash
cargo fmt --all -- --check
cargo check -p phalcom-semantic
cargo check -p phalcom-modules
cargo check -p phalcom-lsp --lib
```

- [ ] **Step 4: Run full semantic crate**

```bash
cargo test -p phalcom-semantic
```

- [ ] **Step 5: Run module/LSP architectural gates**

```bash
cargo test -p phalcom-modules
cargo test -p phalcom-lsp
cargo test -p phalcom-lsp --test semantic_boundary
```

- [ ] **Step 6: Run import/navigation regression anchors**

```bash
cargo test -p phalcom-lsp --test module_navigation
cargo test -p phalcom-lsp --test imported_binding_resolution
```

Use registered test names from the current `Cargo.toml` if the harness requires an explicit target alias.

- [ ] **Step 7: Update coverage ledger by semantic law**

The final ledger must include direct tests for:
- evidence creation;
- evidence weakening;
- relation validation;
- callable publication;
- field validity;
- branch joins;
- predicate refinement/contradiction;
- abrupt exits;
- loop fixed points/exhaustion;
- core identity;
- generic TypeStore specialization;
- comparison operation resolution;
- membership fail-closed/canonical resolution;
- contextual empty collections;
- clean/incremental equivalence.

- [ ] **Step 8: Review unresolved Unknowns**

List remaining `UnknownReason::UncheckedExpression` production paths. For each, record whether it is:
- intentionally unsupported language semantics;
- blocked on a separate ratified language design;
- recovery boundary;
- implementation gap outside this six-part typing closure.

Do not convert them to known types merely to make the ledger look complete.

- [ ] **Step 9: Commit final closure**

```bash
git add docs/impl/semantic/semantic-correctness/part-4/2026-08-29-semantic-authority-audit.md \
        phalcom-semantic/tests/semantic
git commit -m "docs(semantic): close six-part semantic hardening program"
```

---

# 6. Authority Audit Acceptance Table

Each final known proof must fit one row.

| Evidence source | May produce Established? | Conditions |
| --- | --- | --- |
| Primitive literal syntax | Yes | canonical compiler-owned literal semantics |
| Tuple/record/collection composition | Yes | all required constituents Established and constructor semantics canonical |
| Developer annotation | No by itself | supplies Assumed contract/evidence only |
| Relation success | No by itself | validates relation only |
| Exact source method return annotation | No by itself | Part-01 body certification may later establish public contract |
| Constructor semantics | Yes | canonical constructor rule |
| Native signature | Yes | trusted canonical native surface |
| Generic inference | Yes | all required constraints/premises established |
| Flow refinement | Yes/Assumed | may not exceed prior + predicate authority |
| Field lifecycle | Yes | Part-02 initialization + validated contract + clean causality |
| Contextual empty collection | Depends on context | never stronger than expected-context authority |
| Comparison chain | Yes | every semantic link clean Established Bool |
| Membership | Yes only with canonical semantic operation | otherwise Unknown |
| Advisory analysis | Never formal | presentation only |

---

# 7. TypeStore Ownership Acceptance Matrix

| Operation | Store behavior |
| --- | --- |
| canonical interning | mutate active session store |
| retained snapshot | clone/snapshot store for immutable historical reads |
| relation specialization | materialize in active store |
| type-lambda beta materialization during analysis | materialize in active store |
| speculative loop probe | may intern in active store, canonical/idempotent |
| clone used for preview/debug | IDs must not escape clone |
| cached product | carries IDs from the session store domain that owns the product |
| cross-session comparison | compare exported semantic forms/identity, not bare TypeId numbers |

---

# 8. Non-Goals

Part 06 does not:

- add a second semantic database;
- move static semantics to LSP;
- redesign the language's membership protocol;
- invent an unratified membership selector;
- add value-range comparison reasoning;
- add dependent/refinement types;
- add a general theorem prover;
- embed `TypeStoreId` into every `TypeId`;
- replace current interning architecture;
- rewrite dispatch;
- rewrite source index;
- execute the CFG;
- reopen loop fixed-point architecture without a concrete regression;
- use advisory shapes to fill formal Unknown;
- make every `UncheckedExpression` known merely for “completeness.”

The final plan defines completeness as:

> Every implemented language semantic feature uses one sound canonical proof path, and every unimplemented/unratified semantic feature fails closed without fabricating formal evidence.

---

# 9. Final Completion Criteria

The six-part semantic hardening effort is complete when:

1. There is one evidence-authority model across declarations, calls, fields, flow, and generics.
2. Developer annotations cannot override contradictory established proofs.
3. Invalid recovery facts cannot become formal established publication.
4. Fields distinguish initialization from contract validity.
5. Abrupt control has one canonical exit model.
6. Branch execution uses executable regions, not closure construction.
7. Predicate refinement preserves authority and contradictions prune only established impossibilities.
8. Loops use bounded semantic fixed points with correct edge topology.
9. Generic-supertype specialization never exports clone-generated TypeIds into the live store.
10. Core privilege uses exact canonical identity, never declaration spelling.
11. Primitive/container compiler semantics cannot be hijacked by user shadowing.
12. Comparison chains derive results from actual operation semantics and evaluate middle operands once.
13. Membership never manufactures Bool proof in the absence of canonical semantics.
14. Empty collections can consume compatible contextual types without authority laundering.
15. Every production establishment site is accounted for in the authority audit.
16. Every `RelationOutcome::Proven` consumer has been reviewed for non-strengthening.
17. Clean and incremental analysis are semantically equivalent.
18. Formal/advisory/presentation boundaries remain intact.
19. `phalcom-lsp` remains a protocol/presentation adapter.
20. `cargo test -p phalcom-semantic`, module tests, and LSP semantic-boundary tests all pass.

---

# 10. End-State Architectural Contract

After Part 06, the intended semantic pipeline is:

```text
source syntax
    │
    ▼
canonical declaration / operation identity
    │
    ▼
formal expression evidence
    │
    ├───────────────┐
    ▼               ▼
contracts       trusted predicates
    │               │
    └──────┬────────┘
           ▼
        FlowState
           │
     ┌─────┴─────┐
     ▼           ▼
 branches      loops
     │           │
     └─────┬─────┘
           ▼
 normal / abrupt exits
           │
           ▼
 contract certification
           │
           ▼
 formal publication
           │
           ▼
 immutable SemanticSnapshot
           │
     ┌─────┴────────┐
     ▼              ▼
 editor queries   advisory presentation
     │              │
     └──────┬───────┘
            ▼
           LSP
```

Two invariants govern every arrow:

1. **Reachability decides which evidence can contribute.**
2. **Evidence authority decides how strongly the checker may claim the result.**

Canonical identity decides *what* the evidence is about; the active TypeStore decides *which type arena* gives every TypeId its meaning.

No downstream layer gets to repair those three decisions.
