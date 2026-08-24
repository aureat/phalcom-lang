# Phalcom Semantic Incrementality Hardening — Step 5.5 Implementation Specification

> **Status:** Proposed implementation specification
> **Purpose:** Hardening gate between Step 5 and Step 6
> **Repository:** `aureat/phalcom-lang`
> **Verified baseline:** `main` at `06f6bcd2375a7e62c46eabee967da95fa99652cf` — `feat(semantic): add stable incremental invalidation`
> **Parent baseline:** `3ff22158a60db3a323a4a64e8b6ab3957de02408`
> **Pinned Rust toolchain:** `nightly-2026-07-10`

---

## 0. Executive decision

Step 5 established the correct core invalidation model:

```text
input change
    -> recompute only the affected query
    -> preserve cached dependents
    -> republish a semantic product fingerprint
    -> unchanged product fingerprint stops propagation
    -> changed product fingerprint makes exact dependents non-reusable
```

That mechanism should be retained.

Step 5.5 is a **hardening phase**, not a new feature phase. It must correct the remaining cases where the current implementation either:

1. performs semantic work before deciding whether a query can be reused;
2. records the wrong semantic product as a dependency;
3. relies on query-order prewarming rather than requesting an immediate prerequisite;
4. lets presentation/source movement change a dependency-visible semantic product fingerprint; or
5. weakens the generic DB reuse invariant with query-specific exceptions.

The central architectural outcome of Step 5.5 is:

```text
direct syntax/input identity
        +
dependency-visible semantic identities
        |
        v
generic SemanticDb reuse law
        |
        v
semantic computation only on a real miss
```

Step 6 must not begin until this specification's acceptance gates pass.

---

# 1. Repository-grounded current state

This specification was re-grounded against `main` at:

```text
06f6bcd2375a7e62c46eabee967da95fa99652cf
```

The important current implementation facts are as follows.

## 1.1 Query state now distinguishes computation and validation revisions

Current `phalcom-semantic/src/db/state.rs` has:

```rust
Ready {
    revision: SemanticRevision,
    validated_revision: SemanticRevision,
    input_fingerprint: InputFingerprint,
    product_fingerprint: ProductFingerprint,
    value: QueryValue,
}
```

This is correct and MUST remain.

`revision` means:

> the semantic revision that actually computed and published the stored product.

`validated_revision` means:

> the newest semantic revision in which the stored product was proven reusable.

Step 5.5 MUST NOT collapse these two concepts.

---

## 1.2 Product-stability propagation is already the correct primitive

Current `phalcom-semantic/src/db/mod.rs` provides:

```rust
pub fn discard_for_recompute(&mut self, key: &QueryKey) -> bool
```

It deletes the query's current state/product and outgoing dependencies while deliberately preserving incoming reverse edges from dependents.

That is correct for ordinary recomputation.

Current:

```rust
pub fn invalidate(...)
```

still performs destructive reverse-closure invalidation and is used for disappearance/removal.

That distinction MUST remain:

```text
query will be replaced
    -> discard_for_recompute

query disappeared and has no replacement
    -> invalidate reverse closure
```

---

## 1.3 `DeclarationSurface` currently computes before cache validation

Current `query_declaration_surface(...)` in:

```text
phalcom-semantic/src/db/query.rs
```

does this, in order:

```text
find ClassDef
    ->
construct CheckingContext
    ->
register_class_surface(...)
    ->
resolve annotations
    ->
allocate/intern types
    ->
collect diagnostics
    ->
compute declaration-surface input fingerprint
    ->
validate_reuse(...)
```

This means a cache hit still performs the expensive semantic work.

It also makes the query's supposed *input* fingerprint depend on the product that was just computed.

This is structurally wrong.

After Step 5.5, `DeclarationSurface` MUST calculate its direct input fingerprint **before** resolving any type annotation or building a candidate surface.

---

## 1.4 `DeclarationSurface` is currently used as a proxy for declaration type metadata

Current `TrackingTypeResolver` in:

```text
phalcom-semantic/src/checker/context.rs
```

records:

```rust
SemanticDependency::DeclarationSurface(declaration)
```

when a type name resolves.

But `resolve_type_form(...)` in:

```text
phalcom-semantic/src/types/annotation.rs
```

actually consumes:

```rust
declarations.form(&decl)
```

and therefore indirectly consumes:

- canonical declaration form;
- declaration kind;
- generic parameter identity/kinds;
- generic constraints;
- declaration type metadata.

Those facts live in:

```rust
DeclarationTypeInfo
```

from:

```text
phalcom-semantic/src/declarations.rs
```

They are **not** represented by `DeclarationSurface`.

This creates a real stale-reuse hole.

Example:

```phalcom
class Higher<F: Type -> Type> {}
```

changes to:

```phalcom
class Higher<F: Type> {}
```

while another unchanged declaration contains a type annotation referring to `Higher`.

Step 5 correctly versions the canonical `TypeId`/`TypeParameterId`, but a dependent query that records only `DeclarationSurface(Higher)` can miss the change if `Higher`'s member surface is otherwise identical.

Step 5.5 MUST make declaration form/kind/generic metadata dependency-visible.

---

## 1.5 `QueryKey::DeclarationShell` already exists but has no typed product

Current:

```text
phalcom-semantic/src/db/key.rs
```

already contains:

```rust
DeclarationShell(DeclarationId)
```

but current:

```text
phalcom-semantic/src/db/product.rs
```

has no `DeclarationShell` product variant.

Step 5.5 MUST activate this existing key rather than creating another parallel declaration-metadata cache.

---

## 1.6 `CallableBody` still requires its signature to be pre-warmed

Current `query_callable_body(...)` checks that the complete consumed signature:

```text
QueryKey::CallableSignature(...)
```

is already:

```text
Ready
+
validated in current revision
+
backed by a typed CallableSignature product
```

Otherwise it publishes `Failed`.

This makes success depend on orchestration order.

A callable-body query should request/ensure its immediate formal prerequisite instead of failing because some caller did not prewarm it.

Step 5.5 MUST remove that order-dependent failure mode.

---

## 1.7 Callable-body product fingerprints still contain presentation data

Current:

```text
phalcom-semantic/src/db/fingerprint.rs
```

includes source/presentation details in:

```rust
callable_body_product_fingerprint(...)
```

including, among other things:

- `analysis.body_range`;
- expression ranges;
- binding ranges;
- flow-node ranges;
- `TypeKnowledge` provenance;
- diagnostics and their source ranges/messages/notes;
- explanation source spans/content.

This means source movement can change a dependency-visible **semantic** product fingerprint even when inferred meaning is unchanged.

That violates the established input/product separation.

Step 5.5 MUST make callable-body product identity semantic-only while preserving source-sensitive input identity and the full source-rich payload.

---

## 1.8 `SemanticDb::is_reusable()` contains a query-specific exception

Current `phalcom-semantic/src/db/mod.rs` contains special logic equivalent to:

```text
if dependent is DeclarationSurface
and dependency is ParsedModule
    ignore dependency product-fingerprint mismatch
```

A generic dependency engine MUST NOT know special semantic relationships between particular query-key variants.

If a dependency is irrelevant, it must not be recorded.

If it is recorded, its product fingerprint must be honored.

Step 5.5 MUST remove this exception.

---

## 1.9 Step 5 also fixed persistent kind identity; preserve it

Current `TypeStore` now interns by:

```rust
HashMap<(TypeData, KindId), TypeId>
```

and versions `TypeParameterId` when semantic binder data changes.

That is correct.

Step 5.5 MUST add regressions that rely on this behavior, but MUST NOT revert to in-place mutation of old canonical type identities.

---

# 2. Scope

Step 5.5 consists of six tightly related changes.

## 2.1 Required work

1. Activate a dependency-visible `DeclarationShell` product.
2. Record declaration-shell dependencies for form/kind/generic metadata reads.
3. Replace declaration-surface candidate-before-reuse with a cheap source-contract input fingerprint.
4. Capture the semantic dependencies actually consumed while rebuilding a declaration surface.
5. Make callable-body evaluation request its immediate callable-signature prerequisite.
6. Redefine callable-body product fingerprinting as semantic-only.
7. Remove the query-specific `is_reusable()` exception.
8. Add regressions proving all of the above.

---

## 2.2 Explicit non-goals

Step 5.5 MUST NOT:

- move `ProjectUniverse` ownership into `SemanticWorkspaceSession`;
- move `ModuleResolver` or `ModuleLinker` lifecycle ownership;
- replace externally supplied `LinkedProgram`;
- modify LSP completion/navigation;
- delete `run_static_workspace_analysis`;
- redesign import/export products;
- implement cold-vs-incremental full workspace equivalence;
- redesign constructor `Self`;
- add new language syntax;
- change module identity semantics;
- introduce a second declaration metadata table;
- introduce a second callable-signature cache;
- redesign all source syntax hashing in the compiler.

Those belong to later steps.

---

# 3. Non-negotiable Step 5.5 invariants

After Step 5.5, all of these statements MUST be true.

## 3.1 One generic reuse law

For every query key:

```text
reusable =
    stored Ready product exists
    AND direct input fingerprint is unchanged
    AND every recorded dependency is current-validated
    AND every recorded dependency has the same product fingerprint observed before
```

No query-key-specific exception is permitted inside `SemanticDb::is_reusable()`.

---

## 3.2 Direct input must be computable without computing the query result

A query MUST NOT have to construct its semantic product merely to calculate the direct input fingerprint used to decide whether that product can be reused.

In particular:

```text
DeclarationSurface input fingerprint
```

MUST be computable from source declaration contract syntax and stable non-query invocation data.

It MUST NOT require:

```text
register_class_surface(...)
resolve_type_annotation(...)
candidate DeclarationSurface
candidate diagnostics
```

---

## 3.3 Declaration type identity is its own dependency product

Any checker/query path that consumes:

- `DeclarationTypeInfo`;
- declaration `form`;
- declaration `kind`;
- declaration generic signature;
- declaration supertype template when semantically relevant;

must observe:

```text
QueryKey::DeclarationShell(declaration)
```

for query-owned declarations.

`DeclarationSurface` MUST NOT stand in for this information.

---

## 3.4 Member surface and declaration shell remain distinct concepts

Use:

```text
DeclarationShell
```

for:

```text
nominal form
kind
generic signature
generic constraints
supertype template
class-object type identity
```

Use:

```text
DeclarationSurface
```

for:

```text
instance fields
class fields
instance callable signatures
class callable signatures
member-level declared contracts
constructor Self signature surface
```

A query may depend on both.

Do not merge these products.

---

## 3.5 Source movement refreshes payload without propagating semantic invalidation

If source ranges move but semantic meaning is unchanged:

```text
query direct input may change
    ->
query may recompute to refresh ranges/provenance
    ->
stored payload becomes source-current
    ->
semantic ProductFingerprint remains unchanged
    ->
semantic dependents reuse
```

This law applies especially to `CallableBody`.

---

## 3.6 Missing prerequisite is not an internal failure

If an ordinary query prerequisite has not yet been computed/current-validated, do not turn that ordering condition into:

```rust
QueryOutcome::Failed(...)
```

when it can be evaluated or represented as a normal dependency block.

The query layer should:

```text
ensure prerequisite
OR
propagate prerequisite Cancelled/BudgetExceeded/Blocked/Failed
```

A missing prerequisite caused only by call order is not a semantic failure.

---

# 4. Target query topology after Step 5.5

The formal portion of the graph should become:

```text
ParsedModule
    |
    v
UnlinkedInterface

externally supplied linked workspace (still transitional in Step 5.5)
    |
    v
LinkedInterface

source declaration predeclaration/enrichment
    |
    v
DeclarationShell
    |
    +-----------------------------+
    |                             |
    v                             v
HierarchyEdge              DeclarationSurface
                                  |
                                  v
                           CallableSignature
                                  |
                                  v
                           CallableBody
```

Additional dynamic dependencies include:

```text
DeclarationSurface
    -> DeclarationShell(referenced type)
    -> LinkedInterface(current module on imported/unresolved lookup)

CallableBody
    -> DeclarationShell(type metadata read)
    -> DeclarationSurface(member/field/dispatch surface read)
    -> CallableSignature(exact consumed callable)
    -> HierarchyEdge(exact traversed hierarchy edges)
    -> LinkedInterface(import/name-resolution dependency)
```

The transitional externally supplied linked workspace remains in place until Step 6.

---

# 5. Activate `DeclarationShell` as a typed DB product

## 5.1 Files

Modify:

```text
phalcom-semantic/src/db/product.rs
phalcom-semantic/src/db/fingerprint.rs
phalcom-semantic/src/db/query.rs
phalcom-semantic/src/checker/analysis.rs
phalcom-semantic/src/checker/context.rs
phalcom-semantic/src/session.rs
```

`phalcom-semantic/src/db/key.rs` already contains the required key and should not gain another equivalent key.

---

## 5.2 Product representation

Add to `SemanticProduct`:

```rust
DeclarationShell(Arc<DeclarationTypeInfo>),
```

Add:

```rust
pub fn as_declaration_shell(&self) -> Option<&Arc<DeclarationTypeInfo>>
```

Update `to_query_value()` with:

```text
declaration-shell
```

Do not duplicate the `DeclarationTypeInfo` fields into a second struct unless compilation/ownership constraints strictly require a wrapper.

The existing canonical data type is:

```rust
pub struct DeclarationTypeInfo {
    pub declaration: DeclarationId,
    pub form: TypeId,
    pub class_object_type: TypeId,
    pub kind: KindId,
    pub generic_signature: Option<GenericSignature>,
    pub supertype_template: Option<GenericSupertypeTemplate>,
}
```

That is the semantic shell payload.

---

## 5.3 Fingerprints

Add:

```rust
pub fn declaration_shell_input_fingerprint(
    info: &DeclarationTypeInfo,
) -> InputFingerprint

pub fn declaration_shell_product_fingerprint(
    info: &DeclarationTypeInfo,
) -> ProductFingerprint
```

Hash semantically:

```text
declaration
form
class_object_type
kind
generic_signature
supertype_template
```

`GenericSignature` hashing must retain:

```text
owner
parameter IDs
constraints
```

The TypeStore's Step-5 parameter versioning guarantees a semantic binder kind/name/variance change receives a new parameter identity.

Do not hash source-only `TypeParameterData.source` into the shell product.

The shell itself contains no source-range payload, so source movement alone does not require a new shell product.

---

## 5.4 Query function

Add:

```rust
pub fn query_declaration_shell(
    db: &mut SemanticDb,
    info: Arc<DeclarationTypeInfo>,
) -> QueryOutcome<Arc<DeclarationTypeInfo>>
```

Behavior:

```text
key = DeclarationShell(info.declaration)

calculate direct input fingerprint
    ->
validate_reuse
    ->
if reusable:
    return stored typed product
    ->
otherwise discard_for_recompute(key)
    ->
publish current DeclarationShell product
```

For Step 5.5 this is intentionally a **projection query** over the declaration table already reconstructed by `SemanticWorkspaceSession`.

It does not yet move declaration predeclaration/generic enrichment into the DB.

That ownership move may happen later, but dependency visibility must exist now.

---

## 5.5 Session publication order

After the current session has finished:

```text
predeclaration
generic-signature resolution
supertype-template resolution
DeclarationTypeTable enrichment
```

and before `HierarchyEdge`, `DeclarationSurface`, or `CallableBody` consumption, ensure every query-owned source declaration has a current:

```text
DeclarationShell(decl)
```

product.

Core/bootstrap declaration metadata remains an immutable session seed and need not receive synthetic DB query products.

---

# 6. Extend semantic dependency vocabulary

## 6.1 Add the dependency variant

Modify:

```text
phalcom-semantic/src/checker/analysis.rs
```

Add:

```rust
SemanticDependency::DeclarationShell(DeclarationId),
```

The full enum becomes conceptually:

```rust
pub enum SemanticDependency {
    DeclarationShell(DeclarationId),
    DeclarationSurface(DeclarationId),
    CallableSignature(CallableId),
    HierarchyEdge(DeclarationId),
    LinkedInterface(ModuleId),
}
```

---

## 6.2 Centralize conversion to `QueryKey`

The mapping from `SemanticDependency` to `QueryKey` is currently embedded in callable-body publication.

Extract one DB-layer helper, for example:

```rust
fn semantic_dependency_query_key(
    dependency: &SemanticDependency,
) -> QueryKey
```

or an equivalent private helper.

Both declaration-surface and callable-body publication must use the same conversion.

Do not duplicate two independent `match` statements that can drift when new dependency kinds are added.

---

# 7. Correct declaration metadata tracking

Modify:

```text
phalcom-semantic/src/checker/context.rs
```

## 7.1 Introduce shell dependency helpers

Add:

```rust
fn record_declaration_shell_dependency(
    dependencies: &SharedSemanticDependencies,
    declaration: &DeclarationId,
)
```

Use the existing `is_query_owned_module(...)` policy so immutable builtin `core` seed reads do not generate impossible staged DB dependencies.

---

## 7.2 `TrackingTypeResolver` must record shell, not surface, for type identity

Change successful type-name resolution from:

```text
DeclarationSurface(target)
```

to:

```text
DeclarationShell(target)
```

because annotation resolution consumes the target declaration's canonical form/kind.

Continue recording:

```text
LinkedInterface(current_module)
```

when an imported resolution is involved, because changing the current module's import binding can change which canonical declaration is selected.

For an unresolved type name, continue recording:

```text
LinkedInterface(current_module)
```

so a later import/export resolution change can invalidate the previous negative lookup.

---

## 7.3 Declaration metadata accessors must record shell

Change these helpers:

```rust
declaration_info(...)
declaration_generic_signature(...)
nominal_type_of(...)
```

to record:

```text
DeclarationShell(declaration)
```

rather than `DeclarationSurface`.

Member reads such as:

```rust
get_surface(...)
get_field(...)
dispatch owner traversal
```

continue to record `DeclarationSurface`.

---

## 7.4 Generic substitution must expose the shell read

Current dispatch specialization calls:

```rust
substitution_for_applied(
    self.declarations,
    self.store,
    receiver,
)
```

which reads a declaration generic signature directly.

Before or while calling this helper, record:

```text
DeclarationShell(origin declaration)
```

for the nominal origin of an applied receiver.

The preferred design is a context helper, e.g.:

```rust
fn substitution_for_applied_receiver(
    &self,
    receiver: TypeId,
) -> Option<TypeSubstitution>
```

that:

1. discovers the nominal origin;
2. records `DeclarationShell(origin)`;
3. delegates to `substitution_for_applied(...)`.

Do not scatter an ad-hoc record call around every caller.

---

## 7.5 `Self` specialization must expose declaration-form reads

`specialize_self_type(...)` can call:

```rust
declarations.form(...)
```

for the concrete receiver declaration or `Self` owner.

Before this operation, the checking context must record shell dependencies for the declaration forms that may be consumed.

This can be done by a context-owned specialization wrapper.

The raw substitution helper may remain pure; the query-aware context owns dependency recording.

---

# 8. Cheap declaration-surface direct input fingerprint

## 8.1 Replace result-derived input identity

Delete the architectural use of:

```rust
declaration_surface_query_input_fingerprint(
    &computed_surface,
    &computed_diagnostics,
)
```

as the direct cache key for `query_declaration_surface`.

The helper may be deleted entirely if no other consumer needs it.

Replace it with a source-contract fingerprint computable before semantic resolution.

Suggested API:

```rust
pub fn declaration_surface_source_input_fingerprint(
    unit: &ParsedModuleUnit,
    declaration: &DeclarationId,
    class_def: &ClassDef,
) -> InputFingerprint
```

---

## 8.2 What the direct surface input MUST hash

Hash only source facts that can alter the *declared member surface* or its source-rich annotation diagnostics.

### Class identity

Hash:

```text
DeclarationId
```

Do not hash the entire `ClassDef::range`.

### Field members

Hash:

```text
member kind = field
effective side
field name
presence + source text/range of type annotation
```

Do NOT hash:

```text
field default expression
field initializer body syntax
whole field declaration range
irrelevant attributes
```

A field default belongs to initializer/body analysis, not the declared field type surface.

### Method members

Hash:

```text
member kind = method
method name
effective side
constructor status
ordered parameters:
    local name
    external label
    rest mode
    optional annotation source slice/range
optional return annotation source slice/range
method generic parameter source slices/ranges
method where-clause source slice/range
```

Do NOT hash:

```text
method body
whole MethodDef::range
unrelated attribute argument expressions
```

Hash the semantic presence of:

```text
@class
@constructor
```

where those attributes affect `member_side()` / constructor lowering.

### Getter members

Hash:

```text
member kind = getter
name
effective side
optional return annotation source slice/range
```

Do not hash the getter body.

### Setter members

Hash exactly the facts currently consumed by `register_class_surface(...)`:

```text
member kind = setter
name
effective side
parameter local name
parameter label/rest if semantically represented
parameter annotation source slice/range
```

The current surface builder returns `Unit` for setter return type. Do not invent a new setter-return contract in this hardening patch.

### Index members

Hash:

```text
member kind = index
effective side
get vs set accessor
ordered index parameters
labels/rest/local names
parameter annotation source slices/ranges
getter return annotation when applicable
setter put parameter + annotation
```

Do not hash the index body.

---

## 8.3 Source-slice helper

Do not use:

```rust
format!("{node:?}")
```

for declaration-surface direct input hashing.

Add a bounded helper such as:

```rust
fn hash_source_region(
    source: &str,
    range: SourceRange,
    hasher: &mut impl Hasher,
)
```

It should hash:

```text
range start/end
source bytes within the range
```

If a recovered/bad range is outside the source string:

- hash a deterministic invalid-range marker;
- hash the numeric range;
- do not panic.

This fingerprint is intentionally source-sensitive because source/range changes may require a new diagnostics/provenance payload.

---

# 9. Refactor `query_declaration_surface` into a real cache lookup

The target algorithm is:

```text
find source ClassDef
    |
    v
compute declaration_surface_source_input_fingerprint(...)
    |
    v
ensure static prerequisites are current:
    DeclarationShell(self)
    LinkedInterface(module)
    |
    v
validate_reuse(key, input_fp)
    |
    +-- reusable --> return stored product
    |
    v
discard_for_recompute(key)
    |
    v
build surface + diagnostics
    |
    v
capture semantic dependencies actually read
    |
    v
record all dependencies fail-closed
    |
    v
publish surface payload
```

Semantic resolution must happen only below the cache-miss branch.

---

## 9.1 Expose semantic dependencies from `CheckingContext`

The declaration surface builder currently uses a `CheckingContext`, which already tracks semantic reads internally.

Add a read-only snapshot accessor such as:

```rust
pub(crate) fn semantic_dependencies_snapshot(
    &self,
) -> BTreeSet<SemanticDependency>
```

or equivalent.

Do not consume/destroy the context merely to get dependencies.

---

## 9.2 Surface query dependency rules

A successful `DeclarationSurface` rebuild MUST record all query-owned semantic dependencies that were actually consumed.

Expected common dependencies:

```text
DeclarationShell(self)
DeclarationShell(referenced type declarations)
LinkedInterface(current module) for imported/unresolved type lookup
```

It must not depend on the whole:

```text
ParsedModule(module)
```

product.

It must not record:

```text
DeclarationSurface(self)
```

as a self-cycle.

With declaration metadata reads converted to `DeclarationShell`, no legitimate self-surface edge should be necessary.

If a self-surface dependency still appears, treat that as an implementation bug; do not hide it with reuse exceptions.

---

## 9.3 Fail closed

If a captured semantic dependency maps to a query product that is not current/Ready:

```text
do not publish DeclarationSurface Ready
```

Return/propagate an appropriate normal query outcome.

Never discard dependency-recording errors.

---

# 10. Activate current declaration shells before surfaces

Modify:

```text
phalcom-semantic/src/session.rs
```

After the declaration table is fully enriched for the current revision, run:

```text
query_declaration_shell
```

for every query-owned source declaration.

This must happen before:

```text
DeclarationSurface
CallableSignature
CallableBody
```

queries can consume those shell products.

The compatibility `DeclarationTypeTable` remains available because current checker/type APIs consume it directly, but its semantically observable entries now have a DB product identity.

---

# 11. Make callable-body signature prerequisites demand-driven

## 11.1 Remove the prewarm-only readiness check

Current `query_callable_body(...)` manually checks:

```text
CallableSignature key is Ready
AND validated in current revision
AND typed product exists
```

and returns `Failed` otherwise.

Delete that order-dependent logic.

---

## 11.2 Immediate prerequisite evaluation

When `signature_consumed_by_body(...)` reports a complete source signature:

```text
CallableBody
    ->
ensure CallableSignature(signature_id)
```

At minimum, `query_callable_body` MUST invoke:

```rust
query_callable_signature(...)
```

rather than inspecting query state itself.

It must propagate:

```text
Ready
Blocked
Cancelled
BudgetExceeded
Failed
```

from the signature query.

---

## 11.3 Formal prerequisite helper

To remove the remaining false dependency on caller ordering, introduce a lightweight formal-query input view that **borrows** the current transitional workspace inputs without owning module lifecycle.

Recommended shape:

```rust
pub struct FormalQueryInputs<'a> {
    pub sources: &'a BTreeMap<ModuleId, Arc<ParsedModuleUnit>>,
    pub linked: &'a LinkedProgram,
    pub hierarchy: &'a dyn TypeHierarchy,
    pub base_resolver: &'a dyn TypeResolver,
    pub declarations: &'a DeclarationTypeTable,
}
```

Exact field visibility may remain private.

This struct MUST NOT:

- own a second `TypeStore`;
- construct a `ProjectUniverse`;
- construct a `ModuleResolver`;
- construct a `ModuleLinker`;
- mutate workspace revision;
- perform filesystem I/O.

It is only a borrowed lookup view over inputs the current session already owns for the revision.

---

## 11.4 Ensure helpers

Add internal helpers with behavior equivalent to:

```rust
ensure_declaration_shell(...)
ensure_linked_interface(...)
ensure_declaration_surface(...)
ensure_callable_signature(...)
```

`ensure_callable_signature(...)` should:

```text
ensure DeclarationShell(owner)
    ->
ensure LinkedInterface(owner.module)
    ->
ensure DeclarationSurface(owner)
    ->
query CallableSignature(callable)
```

`query_callable_body(...)` uses this helper for its own consumed complete signature.

This makes body evaluation independent of whether the session happened to populate the signature table earlier.

---

## 11.5 Missing formal input

If a requested prerequisite cannot be found in the current formal input view, represent it as a normal dependency problem where possible:

```rust
QueryOutcome::Blocked(
    BlockReason::UnresolvedDependency(query_key)
)
```

Do not report a generic internal `Failed("requires ready ...")` merely because the prerequisite was not prewarmed.

Actual invariant corruption may still be `Failed`.

---

# 12. Redefine callable-body product identity as semantic-only

## 12.1 Keep source-sensitive input identity

`callable_body_input_fingerprint(...)` remains the mechanism deciding whether the stored source-rich body payload must refresh.

It may continue to include:

- body source shape;
- body range;
- source-sensitive syntax identity.

Step 5.5 does not require a complete AST hashing rewrite for body direct inputs.

---

## 12.2 Semantic body product fingerprint

Change:

```rust
callable_body_product_fingerprint(...)
```

so it answers only:

> Did the semantic result consumed by downstream semantic queries change?

It MUST NOT answer:

> Did any source location, diagnostic wording, provenance path or explanation presentation detail move?

---

## 12.3 Include these semantic facts

At minimum hash:

### Callable identity

```text
analysis.callable
```

### Expression semantics

For each expression in deterministic identity order:

```text
ExpressionId
TypeKnowledge without provenance
SemanticDenotation
AnalysisStatus
```

Do not hash expression source range.

Do not hash explanation ID merely because it changed.

Prefer canonical callable/dependency identity over local call-resolution arena identity when representing call semantics.

### Binding semantics

For each binding in deterministic identity order:

```text
BindingId
declared type
current TypeKnowledge without provenance
mutable
version
```

The binding source name/range are presentation/source facts and should not make downstream semantic queries recompute when all actual semantic facts are unchanged.

### Flow semantics

Hash:

```text
flow-node identity
flow-node semantic kind
predecessor/successor topology
edge source/target
edge semantic kind
predicate identity/facts
entry flow summary
exit facts
unreachable state
```

Do NOT hash flow-node source ranges.

### Canonical callable dependencies

Hash:

```text
analysis.dependencies
```

because downstream effect/control/termination queries may care which canonical callees are reachable.

The list must be deterministic.

### Analysis status

Hash:

```text
Complete
Partial
Blocked
...
```

where a successful product can carry such state.

---

## 12.4 Exclude these presentation/graph-maintenance facts

The semantic product fingerprint MUST exclude:

```text
body_range
expression ranges
binding ranges
TypeKnowledge provenance ranges/descriptions
diagnostic source ranges
diagnostic notes/help/fix wording
explanation source spans
explanation presentation graph
semantic_dependencies as dependency-graph metadata
dependency_fingerprint itself
```

Diagnostics and explanations remain stored in the full `CallableAnalysis` payload.

A source edit still refreshes that payload through the **input** fingerprint.

The semantic product fingerprint only determines propagation to semantic dependents.

---

## 12.5 Clarify `dependency_fingerprint`

Current `CallableAnalysis` has:

```rust
pub dependency_fingerprint: ProductFingerprint
```

but the query writes the callable body's own semantic product fingerprint into this field.

Recommended:

```rust
pub semantic_product_fingerprint: ProductFingerprint
```

if repository-wide usage is small and the change is mechanical.

Acceptable transitional alternative: keep the field name but document that it is the semantic product fingerprint, not a hash of dependency edges.

---

# 13. Remove the `DeclarationSurface -> ParsedModule` reuse exception

Modify:

```text
phalcom-semantic/src/db/mod.rs
```

Delete all logic equivalent to:

```rust
let declaration_surface_source_prerequisite =
    matches!(key, QueryKey::DeclarationSurface(_))
    && matches!(&edge.dependency, QueryKey::ParsedModule(_));
```

The dependency loop must become generic:

```rust
for edge in deps {
    dependency must be Ready
    dependency.validated_revision == db.revision()
    dependency.product_fingerprint == edge.observed_fingerprint
}
```

No query-key pattern matching belongs in `is_reusable()`.

---

## 13.1 Remove obsolete dependencies rather than weakening validation

After the surface refactor, `DeclarationSurface` MUST NOT record a broad:

```text
ParsedModule(module)
```

product dependency.

Its source declaration contract is represented by the direct input fingerprint.

This is why no special-case reuse rule is required.

---

## 13.2 Consider deleting unused `invalidate_roots`

Current `SemanticDb::invalidate_roots(...)` appears redundant with the now-clear distinction between:

```text
discard_for_recompute
invalidate
```

If repository search confirms it has no consumers, delete it in Step 5.5.

Do not keep three overlapping invalidation APIs without distinct semantics.

---

# 14. File-by-file implementation map

## `phalcom-semantic/src/db/product.rs`

- add `DeclarationShell(Arc<DeclarationTypeInfo>)`;
- add typed accessor;
- add query-value tag;
- preserve source-rich `DeclarationSurfaceProduct`.

## `phalcom-semantic/src/db/fingerprint.rs`

- add declaration-shell input/product fingerprint helpers;
- add declaration-surface **source contract** input fingerprint;
- add bounded source-region hashing helper;
- stop deriving surface query input from computed surface/diagnostics;
- redefine callable-body product fingerprint as semantic-only;
- split/rename flow/explanation helpers as necessary so semantic hashing cannot accidentally include source ranges.

Do not change module/link fingerprint semantics in this step.

## `phalcom-semantic/src/db/mod.rs`

- remove `DeclarationSurface`/`ParsedModule` special-case from `is_reusable()`;
- retain `validated_revision` law;
- retain `discard_for_recompute`;
- retain destructive `invalidate` for disappearance;
- delete `invalidate_roots` only if confirmed unused.

## `phalcom-semantic/src/db/query.rs`

- add `query_declaration_shell`;
- centralize semantic-dependency -> query-key mapping;
- add formal prerequisite ensure helpers;
- refactor `query_declaration_surface` to validate before semantic construction;
- capture and publish its real dynamic dependencies;
- refactor `query_callable_body` to request its immediate signature prerequisite;
- replace prewarm-order failure with prerequisite evaluation/propagation.

## `phalcom-semantic/src/checker/analysis.rs`

- add `SemanticDependency::DeclarationShell`;
- optionally clarify/rename body semantic fingerprint field.

## `phalcom-semantic/src/checker/context.rs`

- add shell-dependency recorder;
- change type-name resolution dependency from surface to shell;
- change declaration metadata accessors to shell;
- add query-aware wrappers for generic substitution / `Self` specialization metadata reads;
- expose a read-only semantic-dependency snapshot for non-body query publication.

## `phalcom-semantic/src/types/annotation.rs`

Prefer no architectural rewrite. Review only to ensure every direct `DeclarationTypeTable` semantic read is preceded by resolver/context tracking in query-aware paths.

`types/annotation.rs` should remain a semantic resolution library, not become a DB client.

## `phalcom-semantic/src/types/substitution.rs`

Prefer to keep substitution functions pure. Add dependency-aware wrappers in `CheckingContext`.

## `phalcom-semantic/src/session.rs`

- publish/current-validate `DeclarationShell` products after declaration enrichment;
- use formal ensure helpers where appropriate;
- preserve current externally supplied linked workspace;
- do not reintroduce broad source invalidation;
- do not add new module resolver/linker ownership.

---

# 15. Required regression suite

Implementation MUST be test-first.

## 15.1 Generic DB reuse invariant has no query exceptions

File:

```text
phalcom-semantic/tests/semantic_db_incremental.rs
```

Add:

```rust
#[test]
fn dependency_product_mismatch_is_never_ignored_for_specific_query_kinds()
```

Construct manually:

```text
ParsedModule leaf
DeclarationSurface dependent
```

Record the dependency, republish the leaf with a different product fingerprint in a newer revision, and verify the `DeclarationSurface` dependent cannot validate reuse.

This test must fail with the current special-case.

---

## 15.2 Declaration shell product changes on kind/generic changes

File:

```text
phalcom-semantic/tests/semantic_fingerprints.rs
```

Add:

```text
declaration_shell_product_changes_when_kind_changes
declaration_shell_product_changes_when_generic_parameter_version_changes
declaration_shell_product_changes_when_supertype_template_changes
```

---

## 15.3 Type-name resolution records declaration shell

File:

```text
phalcom-semantic/tests/checker_dependency_tracking.rs
```

Add:

```text
resolved_type_name_records_declaration_shell_not_member_surface
```

Expected dependency:

```text
DeclarationShell(Target)
```

not `DeclarationSurface(Target)` unless the test also performs an actual member-surface read.

---

## 15.4 Imported and negative type lookup retain linked-interface dependency

Add/retain:

```text
imported_type_resolution_records_current_module_linked_interface
unresolved_type_lookup_records_current_module_linked_interface
```

---

## 15.5 Generic substitution records declaration shell

Add:

```text
generic_receiver_substitution_records_origin_declaration_shell
```

Analyze a body that dispatches on an applied generic nominal type.

Assert the body semantic dependencies include:

```text
DeclarationShell(GenericOwner)
```

---

## 15.6 Surface cache hit does not invoke semantic resolver

File:

```text
phalcom-semantic/tests/formal_query_ownership.rs
```

Use a counting or panic-on-use `TypeResolver`.

Test:

1. build the surface once;
2. begin a new semantic revision;
3. current-validate prerequisites;
4. invoke `query_declaration_surface` with identical direct syntax;
5. use a resolver that panics or increments a forbidden-call counter if semantic resolution executes.

Expected:

```text
cached surface returned
resolver not invoked
surface computation revision remains old
validated_revision becomes current
```

---

## 15.7 Annotation-only source movement refreshes payload but not semantic product

Add a source edit that moves an unresolved annotation range without changing annotation meaning.

Expected:

```text
DeclarationSurface recomputes because direct source input changed
new diagnostics carry new range
DeclarationSurface ProductFingerprint unchanged
dependent CallableBody remains reusable if semantic facts are unchanged
```

---

## 15.8 Generic-kind edit invalidates dependent surface through `DeclarationShell`

Add a workspace regression where a declaration's kind/form changes while an unchanged dependent declaration references it.

Expected:

```text
DeclarationShell(target) product changes
dependent surface cannot reuse through old shell fingerprint
current annotation resolution sees new form/kind
no stale TypeId/KindId survives
```

---

## 15.9 Callable body ensures its own complete signature

Add a low-level query test that deliberately does **not** prewarm:

```text
CallableSignature
```

but provides the formal query inputs needed to derive it.

Invoke `query_callable_body`.

Expected products become current in dependency order:

```text
DeclarationShell(owner)
LinkedInterface(module)
DeclarationSurface(owner)
CallableSignature(callable)
CallableBody(callable)
```

The body query must not fail with:

```text
"requires ready CallableSignature"
```

---

## 15.10 Missing prerequisite is Blocked, not order-dependent Failed

Construct an input view missing a required formal prerequisite.

Expected:

```rust
QueryOutcome::Blocked(
    BlockReason::UnresolvedDependency(...)
)
```

where appropriate.

---

## 15.11 Callable body semantic fingerprint ignores range-only movement

File:

```text
phalcom-semantic/tests/semantic_fingerprints.rs
```

Add:

```text
callable_body_product_ignores_body_range_only_change
callable_body_product_ignores_expression_range_only_change
callable_body_product_ignores_binding_range_only_change
callable_body_product_ignores_flow_node_range_only_change
callable_body_product_ignores_type_evidence_provenance_only_change
```

---

## 15.12 Callable body semantic fingerprint ignores diagnostic/explanation presentation

Replace the current expectation that diagnostic notes alter the body semantic product.

Add:

```text
callable_body_product_ignores_diagnostic_presentation_only_change
callable_body_product_ignores_explanation_presentation_only_change
```

`ModuleDiagnostics` fingerprint tests MUST continue to prove diagnostic details/ranges matter to the diagnostics product itself.

---

## 15.13 Callable body semantic changes still alter product fingerprint

Retain/add inequality tests for:

```text
binding current type
expression TypeKnowledge
expression denotation
expression analysis status
flow topology / exit facts
canonical callee dependency
callable analysis status
```

---

# 16. Required performance/structural tests

## 16.1 No semantic surface construction on true reuse

A declaration-surface cache hit with unchanged direct contract syntax and unchanged dependency products must not call:

```text
resolve_type_annotation
resolve_generic_signature
register_class_surface
```

or mutate the live type store as part of a candidate build.

## 16.2 Body source movement does not propagate to semantic dependents

Create a synthetic dependent query if no production `CallableEffects` query is currently wired.

Record:

```text
Dependent -> CallableBody
```

Recompute `CallableBody` with only source-range/presentation movement.

Expected:

```text
CallableBody input changes
CallableBody recomputes
CallableBody semantic ProductFingerprint stays the same
Dependent validates reuse
```

## 16.3 Generic-kind change propagates despite unchanged member surface

Record a dependent on `DeclarationShell`, change shell kind/form while keeping member surface stable, and verify shell consumers recompute.

---

# 17. Error/outcome semantics

## `Ready`

Only publish when:

```text
all required dependencies were current
all dependency edges were recorded successfully
semantic computation completed
product is internally coherent
```

## `Blocked`

Use for normal unresolved formal prerequisites or suppressed dependencies where analysis cannot yet proceed.

## `Cancelled`

Propagate without publishing a new Ready product. Preserve last known good.

## `BudgetExceeded`

Propagate without replacing last known good Ready product.

## `Failed`

Reserve for:

```text
internal invariant violation
impossible typed product shape
stale publication contract violation
corrupt/mismatched formal query inputs
```

Do not use `Failed` for:

```text
the caller did not prewarm this prerequisite
```

---

# 18. TDD implementation sequence

Do not implement Step 5.5 as one giant edit.

## Slice 5.5-A — Generic DB law cleanup

Tests first:

```text
dependency_product_mismatch_is_never_ignored_for_specific_query_kinds
```

Implementation: remove query-specific `is_reusable` logic.

Gate:

```bash
cargo test -p phalcom-semantic --test semantic_db_incremental
```

## Slice 5.5-B — DeclarationShell product and tracking

Tests first: shell fingerprint and checker dependency tests.

Implementation:

```text
SemanticProduct::DeclarationShell
query_declaration_shell
SemanticDependency::DeclarationShell
shell tracking in CheckingContext
```

Publish shell products from session.

Gate:

```bash
cargo test -p phalcom-semantic --test semantic_fingerprints
cargo test -p phalcom-semantic --test checker_dependency_tracking
cargo test -p phalcom-semantic --test type_store_revisions
```

## Slice 5.5-C — Surface direct input and pre-resolution reuse

Tests first: panic/counting resolver cache-hit test.

Implementation:

```text
declaration_surface_source_input_fingerprint
semantic-dependency snapshot
query_declaration_surface pre-resolution validate path
dynamic dependency publication
```

Gate:

```bash
cargo test -p phalcom-semantic --test formal_query_ownership
cargo test -p phalcom-semantic --test product_stability_invalidation
```

## Slice 5.5-D — Demand-driven callable signature prerequisite

Tests first: low-level body query without prewarmed signature.

Implementation: formal input/ensure helpers and replacement of prewarm readiness check.

Gate:

```bash
cargo test -p phalcom-semantic --test callable_dependency_invalidation
cargo test -p phalcom-semantic --test formal_query_ownership
```

## Slice 5.5-E — Semantic-only callable body fingerprint

Tests first: invert/add range/provenance/diagnostic tests.

Implementation: refactor body semantic hashing.

Gate:

```bash
cargo test -p phalcom-semantic --test semantic_fingerprints
cargo test -p phalcom-semantic --test product_stability_invalidation
```

## Slice 5.5-F — Integrated incremental regressions

Add:

```text
generic-kind dependent invalidation
range-only body dependent reuse
surface hit avoids semantic rebuild
no order-dependent callable-signature failure
```

Then run the entire semantic crate.

---

# 19. Executable verification gate

Use:

```text
nightly-2026-07-10
```

## 19.1 Format

```bash
cargo fmt --check
```

## 19.2 Focused Step 5.5 tests

```bash
RUST_MIN_STACK=33554432 cargo test -p phalcom-semantic --test semantic_db_incremental -- --nocapture
RUST_MIN_STACK=33554432 cargo test -p phalcom-semantic --test semantic_fingerprints -- --nocapture
RUST_MIN_STACK=33554432 cargo test -p phalcom-semantic --test checker_dependency_tracking -- --nocapture
RUST_MIN_STACK=33554432 cargo test -p phalcom-semantic --test formal_query_ownership -- --nocapture
RUST_MIN_STACK=33554432 cargo test -p phalcom-semantic --test callable_dependency_invalidation -- --nocapture
RUST_MIN_STACK=33554432 cargo test -p phalcom-semantic --test product_stability_invalidation -- --nocapture
RUST_MIN_STACK=33554432 cargo test -p phalcom-semantic --test type_store_revisions -- --nocapture
```

## 19.3 Semantic crate

```bash
cargo test -p phalcom-semantic
```

## 19.4 Workspace compile/lint

```bash
cargo check --workspace
cargo clippy --workspace --all-targets -- -D warnings
```

## 19.5 Full workspace

```bash
cargo test --workspace
```

No Step 6 work should begin if any focused Step 5.5 regression fails.

---

# 20. Static deletion/audit gates

## 20.1 No query-specific reuse exception

```bash
rg "DeclarationSurface.*ParsedModule|declaration_surface_source_prerequisite"   phalcom-semantic/src/db
```

Expected: no matches.

## 20.2 No computed-surface cache input

```bash
rg "declaration_surface_query_input_fingerprint" phalcom-semantic
```

Expected: no production query use; prefer complete deletion if unused.

## 20.3 No body prewarm failure text

```bash
rg "callable body requires ready" phalcom-semantic
```

Expected: no matches.

## 20.4 Shell dependencies exist

```bash
rg "DeclarationShell" phalcom-semantic/src
```

Expected production coverage includes:

```text
db product
db query
semantic dependency
checking context
session publication
```

## 20.5 Ordinary recomputation still avoids destructive invalidation

```bash
rg "\.invalidate\(" phalcom-semantic/src
```

Every remaining destructive invalidation call must be justified by disappearance/removal.

---

# 21. Repository hygiene note: `target-cpu=native`

The Step-5 commit also moved:

```text
-C target-cpu=native
```

into:

```text
.cargo/config.toml
```

This is unrelated to semantic incrementality.

Step 5.5 MUST NOT mix build-profile policy changes into the semantic hardening patch.

If portability/reproducibility policy requires removing `target-cpu=native`, do so in a separate explicitly named commit after deciding whether local developer builds or distributable builds should own that flag.

---

# 22. Expected post-Step-5.5 architecture

After this work:

```text
Declaration type metadata:
    DeclarationShell product

Declared member contracts:
    DeclarationSurface product

Body semantic meaning:
    CallableBody semantic ProductFingerprint

Source-rich body presentation:
    full CallableAnalysis payload refreshed by InputFingerprint
```

A body-only edit:

```text
ParsedModule recomputes
UnlinkedInterface may recompute
DeclarationSurface direct contract input unchanged
DeclarationSurface reuses without rebuilding
CallableSignature reuses
edited CallableBody recomputes
unaffected CallableBody reuses
```

A range-only body move:

```text
CallableBody input changes
CallableBody payload recomputes with new ranges
CallableBody semantic product fingerprint unchanged
semantic dependents reuse
```

A generic declaration kind edit:

```text
DeclarationShell product changes
dependent annotation/surface queries cannot reuse stale form/kind
unrelated member-surface consumers remain reusable where semantically valid
```

A callable-body direct request:

```text
body requests its complete signature prerequisite
signature ensures owner formal products
no correctness dependence on prewarming order
```

---

# 23. Acceptance checklist

## DB semantics

- [ ] `is_reusable()` has no query-key-specific semantic exception.
- [ ] `validated_revision` remains distinct from computation `revision`.
- [ ] dependency edges always compare dependency product fingerprints.
- [ ] ordinary recomputation preserves incoming dependents.

## Declaration shell

- [ ] `SemanticProduct::DeclarationShell` exists.
- [ ] every source declaration has a current shell product before formal consumers run.
- [ ] shell fingerprint includes form/kind/generic/supertype semantics.
- [ ] type-name resolution records shell, not surface, for type identity.
- [ ] generic substitution / Self specialization record shell metadata reads.

## Declaration surface

- [ ] direct input fingerprint is computable before semantic surface construction.
- [ ] body/default expressions are excluded from direct surface input.
- [ ] surface cache hit performs no annotation resolution.
- [ ] rebuilt surface publishes every dynamic semantic dependency it consumed.
- [ ] no self-surface dependency exists.
- [ ] no broad ParsedModule dependency exists.

## Callable body

- [ ] body actively requests its complete callable-signature prerequisite.
- [ ] body does not fail solely because signature was not prewarmed.
- [ ] semantic body product fingerprint excludes source/presentation-only data.
- [ ] source-rich payload still refreshes when direct input changes.
- [ ] semantic body changes still change product fingerprint.

## Regressions

- [ ] generic kind edit invalidates exact declaration-shell consumers.
- [ ] body-only edit avoids surface reconstruction.
- [ ] range-only body edit does not invalidate semantic dependents.
- [ ] diagnostic/explanation-only body presentation changes do not alter semantic product identity.
- [ ] module diagnostics continue to fingerprint their own presentation details.
- [ ] existing Step 1–5 regressions remain green.

## Workspace gates

- [ ] `cargo fmt --check`
- [ ] focused Step 5.5 tests
- [ ] `cargo test -p phalcom-semantic`
- [ ] `cargo check --workspace`
- [ ] `cargo clippy --workspace --all-targets -- -D warnings`
- [ ] `cargo test --workspace`

---

# 24. What Step 6 may assume after this gate

Only after Step 5.5 passes may Step 6 assume:

1. `SemanticDb` reuse semantics are generic and exception-free.
2. declaration type identity has a dependency-visible product.
3. declaration surfaces can be cache-hit without rebuilding.
4. callable bodies do not rely on signature prewarming order.
5. source/presentation movement does not masquerade as semantic callable-body change.
6. generic kind/form changes propagate through explicit semantic dependencies.
7. the remaining large architectural problem is module/source lifecycle ownership rather than correctness debt inside the formal invalidation substrate.

Step 6 can then focus on:

```text
safe overlay provider
persistent workspace roots/source inputs
compiler-owned module lifecycle substrate
```

without simultaneously repairing the semantic DB underneath it.

---

# 25. Recommended commit boundary

Keep Step 5.5 as one reviewable semantic-hardening commit or a short ordered series such as:

```text
1. fix(semantic-db): enforce generic reuse and add declaration-shell dependencies
2. perf(semantic): make declaration-surface reuse pre-resolution
3. fix(semantic): make callable prerequisites demand-driven
4. fix(semantic): separate callable-body semantic and presentation fingerprints
5. test(semantic): add step-5.5 incremental hardening regressions
```

If delivered as a single `.patch`, preserve those logical sections in the verification report.

Do not combine Step 6 overlay/module lifecycle changes into the same patch.
