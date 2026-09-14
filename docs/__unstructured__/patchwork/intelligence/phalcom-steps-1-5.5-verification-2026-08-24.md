# Phalcom Incremental Semantic Architecture — Steps 1–5.5 Verification

**Verification date:** 2026-08-24
**Repository:** `aureat/phalcom-lang`
**Remote branch inspected:** `main`
**Remote HEAD observed:** `06f6bcd2375a7e62c46eabee967da95fa99652cf` (`feat(semantic): add stable incremental invalidation`)

## Verification status

### Important repository-state limitation

The user reports that Steps 1–5 plus Step 5.5 are implemented and committed.

The connected GitHub repository currently exposes `main` at:

```text
06f6bcd2375a7e62c46eabee967da95fa99652cf
```

This is the Step-5 commit inspected previously. The Step-5.5 commit is **not visible on the remote default branch at verification time**.

Therefore this report separates:

- **remote-verified implementation:** Steps 1–5 as represented by `06f6bcd`;
- **Step-5.5 expected gate:** requirements from the Step-5.5 specification that must be confirmed in the user's local/newer commit before Step 6 implementation;
- **Step-6 archaeology:** work that remains structurally necessary regardless of the exact Step-5.5 commit SHA.

No GitHub Actions workflow runs or commit status checks were exposed for `06f6bcd`, so executable compilation/test verification is not independently available through GitHub.

---

# 1. Remote-verified Step 1–5 architecture

## 1.1 Query validity and current-revision validation exist

`QueryState::Ready` distinguishes:

```rust
revision
validated_revision
input_fingerprint
product_fingerprint
```

This is the required distinction between computation revision and the newest revision in which the cached result was proven reusable.

`SemanticDb::record_dependency` requires dependency products to be validated in the current revision before they can be consumed.

Verdict:

```text
PASS — core Step-1 validity model exists remotely.
```

---

## 1.2 Input and product fingerprints are distinct

The DB publishes:

```text
InputFingerprint
ProductFingerprint
```

separately, and dependency edges retain observed product fingerprints.

Dedicated structural hashing exists in:

```text
phalcom-semantic/src/db/fingerprint.rs
```

for interfaces, declaration surfaces, callable signatures, callable bodies, diagnostics and semantic components.

Verdict:

```text
PASS — Step-2 fingerprint separation exists remotely.
```

Caveat:

Step 5.5 is expected to further remove source/presentation data from the callable-body *semantic product* fingerprint.

---

## 1.3 Semantic body dependency capture exists

Body checking records semantic dependencies for:

```text
CallableSignature
DeclarationSurface
HierarchyEdge
LinkedInterface
```

and uses tracking wrappers for resolver/hierarchy reads.

Dispatch resolution records visited owners rather than only the final found method.

Constructor bodies correctly distinguish the instance-side implementation identity from the class-side public constructor signature identity.

Verdict:

```text
PASS — Step-3 dependency capture architecture exists remotely.
```

Caveat:

Step 5.5 is expected to add:

```text
DeclarationShell
```

for declaration form/kind/generic metadata reads.

---

## 1.4 Hierarchy/surface/signature products are DB query products

The remote DB has query functions for:

```text
HierarchyEdge
DeclarationSurface
CallableSignature
CallableBody
```

and tests in:

```text
phalcom-semantic/tests/formal_query_ownership.rs
```

verify that hierarchy and surface direct syntax are not modeled as broad `ParsedModule` semantic dependencies.

Verdict:

```text
PASS — Step-4 query ownership is substantially present.
```

Important qualification:

“query-owned product” is not yet the same as “single runtime authority.” The session still constructs independent compatibility tables from those products. That is the next architectural gap.

---

## 1.5 Product-stability propagation is implemented

Ordinary query recomputation uses:

```rust
discard_for_recompute(...)
```

which preserves incoming dependents.

The dependent remains cached and is later revalidated against the dependency's new product fingerprint.

Destructive reverse-closure invalidation remains for removal/disappearance.

Workspace regressions cover:

```text
body-only edit
signature edit
superclass edit
unrelated callable reuse
```

Verdict:

```text
PASS — Step-5 product-stability mechanism is present remotely.
```

---

# 2. Step-5.5 verification gate

Step 5.5 cannot be source-verified from the currently exposed remote HEAD.

Before implementing Step 6 in the user's actual checkout, run:

```bash
git rev-parse HEAD
git log -n 3 --oneline
```

Then verify the following expected Step-5.5 invariants.

## 2.1 Generic DB reuse law

```bash
rg "declaration_surface_source_prerequisite|DeclarationSurface.*ParsedModule" \
  phalcom-semantic/src/db
```

Expected:

```text
no query-specific reuse exception
```

`SemanticDb::is_reusable()` must uniformly require every recorded dependency product fingerprint to equal its observed fingerprint.

---

## 2.2 DeclarationShell is active

```bash
rg "DeclarationShell" phalcom-semantic/src
```

Expected production coverage:

```text
db product
db query
checker semantic dependency
checking context tracking
session publication/current validation
```

The expected typed product is based on:

```rust
DeclarationTypeInfo
```

rather than a second declaration-metadata authority.

---

## 2.3 DeclarationSurface reuse is pre-resolution

```bash
rg "declaration_surface_query_input_fingerprint" phalcom-semantic
```

Expected:

```text
no production cache lookup derived from a candidate computed surface
```

A surface cache hit must be decidable before:

```text
register_class_surface
resolve_type_annotation
resolve_generic_signature
```

runs.

---

## 2.4 CallableBody does not require prewarming

```bash
rg "callable body requires ready" phalcom-semantic
```

Expected:

```text
no matches
```

A body query must ensure/request its complete signature prerequisite or propagate an ordinary query outcome.

---

## 2.5 CallableBody product fingerprint is semantic-only

Run the Step-5.5 fingerprint regressions.

Expected:

```text
body-range-only movement        -> same semantic product fingerprint
expression-range-only movement  -> same
binding-range-only movement     -> same
flow-node-range-only movement   -> same
diagnostic wording/range only   -> same
explanation presentation only   -> same

binding type change             -> different
expression denotation change    -> different
flow semantic change            -> different
callee dependency change        -> different
analysis status change          -> different
```

---

# 3. What remains after Step 5.5

The original architectural completion specification defines the next unit as:

```text
Task 6 — Make hierarchy and declaration materializations
         snapshot projections, not independent authorities
```

That task remains necessary.

## 3.1 Current session still rebuilds a declaration table

The update path currently begins source semantics with:

```rust
let mut declarations = self.base_declarations.clone();
```

and directly populates/enriches it from source AST.

Even after Step 5.5 publishes `DeclarationShell`, this mutable table must become only a temporary producer input. Downstream body checking and snapshots must switch to a table materialized from current DB products.

---

## 3.2 Current session still rebuilds hierarchy

The update path currently begins with:

```rust
let mut hierarchy = self.base_hierarchy.clone();
```

and then inserts `HierarchyEdge` query results into it.

That makes the query product and the mutable map two representations of source truth.

Step 6 must turn the map into a read projection of current query products.

---

## 3.3 Current session still rebuilds dispatch and signature tables

Current source flow uses:

```rust
let mut dispatch = self.base_dispatch.clone();
let mut callable_signatures = self.base_callable_signatures.clone();
```

then inserts DB-produced declaration surfaces and callable signatures.

This is closer to projection than the old implementation, but the session itself still owns the materialization algorithm ad hoc and rebuilds these maps every revision.

Step 6 should formalize this as one read-model materializer with structural sharing.

---

## 3.4 Snapshot receives independently assembled structures

`SemanticSnapshot` currently accepts independently passed:

```text
surfaces
dispatch
callable_signatures
declarations
hierarchy
```

This API permits inconsistent combinations.

Step 6 should make the production snapshot constructor consume one coherent formal projection bundle/stamp set.

Compatibility constructors may remain for tests if necessary.

---

# 4. Newly identified hierarchy defect

`MapTypeHierarchy::insert_template(...)` currently does two things:

```text
insert GenericSupertypeTemplate
insert direct superclass = core.generic_super
```

The latter is not a semantic superclass relation.

Repository search found no other intended production role for the synthetic:

```text
core.generic_super
```

Step 6 must change `insert_template` so it only installs the template.

Direct superclass edges must come exclusively from:

```text
HierarchyEdgeProduct
```

This is especially important once snapshots are built directly from query products.

---

# 5. Future Step-7 work already partially present

Several pieces originally planned for compiler-owned module lifecycle already exist remotely:

- `OverlaySourceProvider`;
- `SourceOverlay`;
- `ImportResolutionTrace`;
- `resolve_import_with_trace`;
- package-interface trace dependencies in `query_resolved_imports`;
- `ModuleQueryProducts`;
- `ModuleQueryFacade::import_root_entries`;
- `module_children`;
- `external_import_children`;
- `resolve_relative_prefix`;
- workspace input model types.

These should not be reimplemented in Step 6.

However lifecycle ownership is still not in `SemanticWorkspaceSession`.

Also, the existing overlay provider still warrants a later correctness repair:

```text
set_overlay lock order:
    module map -> source map

read lock order:
    source map -> module map
```

and replacing one module overlay with a new `SourceId` does not remove the old reverse-source mapping.

Those are Step-7 concerns, not Step 6.

---

# 6. Verification verdict

## Remote-visible Steps 1–5

```text
Architecture: substantially verified.
Compilation/CI: not independently verified from GitHub.
```

## Step 5.5

```text
Implementation: reported complete by user.
Remote verification: unavailable because remote main still exposes Step-5 HEAD.
Required local pre-Step-6 gate: mandatory.
```

## Step 6 readiness

The next implementation unit should be:

```text
DB-projected formal read model and snapshot authority
```

not module-lifecycle ownership yet.

The reason is architectural sequencing:

```text
Step 5.5
  DB dependency/fingerprint substrate sound
        |
        v
Step 6
  DB products become sole source-state authority
  compatibility tables become immutable projections
        |
        v
Step 7
  session owns project/source/module lifecycle
```

This keeps module lifecycle work from being built on top of a still-duplicated formal semantic authority.
