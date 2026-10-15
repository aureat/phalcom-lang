# PERF003.C2.P7 — Audit Slice 2 Report

**Subtitle:** Canonical semantic-authority fidelity, callable owner graphs, source/formal attachments, static checked bodies, trait and witness reconstruction, and compiler-consumer readiness  
**Audit date:** 2026-10-15  
**Repository:** [`aureat/phalcom-lang`](https://github.com/aureat/phalcom-lang)  
**Branch:** `perf/universe-characterization`  
**Immutable audited revision:** [`2939e9d41640dc1eca3fbbc9b8e20529ab05ef6a`](https://github.com/aureat/phalcom-lang/commit/2939e9d41640dc1eca3fbbc9b8e20529ab05ef6a)  
**Relationship to preceding audit:** [Slice 1 report](PERF003.C2.P7-Audit-Slice-1-2026-10-15.md), covering the overall artifact pipeline and its hard stops.  
**Assessment:** **IN_PROGRESS / PARTIAL / UNVERIFIED**. No valid, generally usable persisted Universe compiler image is established; P8 takeover remains blocked.  
**Audit mode:** Read-only inspection of the pinned GitHub branch and comparison with the supplied P7 handoff, execution plan, checkpoint, walkthrough and continuation guide. **No Rust tests, benchmarks, child processes or local worktree validations were executed in this audit.** All findings below explicitly distinguish source-proven conclusions from proposed tests and prior implementer reports.

> **Scope boundary:** Slice 2 reviews the *semantic owner graph* rather than re-auditing Slice 1's whole container path. It inspects the native analyzer and semantic owner types; current `SemanticWorldV2` schema and validation; `CallableAnalysis` export, roots and import; formal/source attachment authority consumed by core lowering; `CheckedStaticBody` and operational templates; trait surfaces and conformance witness plans; derived indices; and directly relevant regression tests. Deep allocator census, malicious binary decoding, type/static graph interning, query-family fingerprint closure, disk concurrency and actual cold-process package execution remain for subsequent slices. Some cross-slice dependencies are called out where they block the semantic work.

---

## 1. Executive verdict

The current P7 semantic persistence implementation is **not merely an almost-complete serializer awaiting test certification**. It contains a number of deliberate and appropriate fail-closed guards, but the underlying transport is still missing compiler-required owner fields, and some importer code that sits behind those guards substitutes **incorrect, default, or stale semantic facts** for canonical source facts. A legal, semantically equivalent artifact cannot be produced by removing guards, promoting `SnapshotStatus::Complete`, or making the completeness predicate less demanding.

### Highest-impact discoveries in this slice

1. **The just-pushed tree contradicts its own previously passing wire-codec tests.** `SemanticWorldV2::decode()` calls the budgeted validator, which unconditionally returns `UnaccountedBoundary`. Existing positive tests call `SemanticWorldV2::decode(...).expect(...)`. Therefore these tests **cannot currently pass** under the inspected code path if selected and executed; this is a source-proven control-flow contradiction, not an independently executed test failure. The earlier reported focused passes were recorded before or outside this exact latest tree. [World decode/validator](https://github.com/aureat/phalcom-lang/blob/2939e9d41640dc1eca3fbbc9b8e20529ab05ef6a/phalcom-compiler-artifact/src/semantic/world.rs#L601-L669), [unconditional stop](https://github.com/aureat/phalcom-lang/blob/2939e9d41640dc1eca3fbbc9b8e20529ab05ef6a/phalcom-compiler-artifact/src/semantic/world.rs#L1524-L1536), [positive tests](https://github.com/aureat/phalcom-lang/blob/2939e9d41640dc1eca3fbbc9b8e20529ab05ef6a/phalcom-compiler-artifact/tests/semantic_world_callable_analysis_test.rs#L129-L180).
2. **The field-level `CallableAnalysis` contract and its wire record do not match.** The manifest labels expression facts, binding facts, iteration evidence, match/pattern resolution, exit/return facts, diagnostics and explanations, semantic dependencies and many more as `Wire`, but `CallableAnalysisRecordV1` carries only a limited subset. There is no complete callable-analysis materializer; the importer explicitly refuses all nonempty analyses, then creates an empty table. [Native owner](https://github.com/aureat/phalcom-lang/blob/2939e9d41640dc1eca3fbbc9b8e20529ab05ef6a/phalcom-semantic/src/checker/analysis.rs#L672-L720), [manifest](https://github.com/aureat/phalcom-lang/blob/2939e9d41640dc1eca3fbbc9b8e20529ab05ef6a/phalcom-semantic/src/artifact/manifest.rs#L35-L65), [wire record](https://github.com/aureat/phalcom-lang/blob/2939e9d41640dc1eca3fbbc9b8e20529ab05ef6a/phalcom-compiler-artifact/src/semantic/world.rs#L298-L322), [import guard](https://github.com/aureat/phalcom-lang/blob/2939e9d41640dc1eca3fbbc9b8e20529ab05ef6a/phalcom-semantic/src/artifact/import.rs#L630-L637).
3. **Compiler-lowering attachments are missing in an operationally important way.** Core lowering does *not* merely need callable IDs and resolved types. It reads `snapshot.formal_projection.binding_attachment_range`, owner-qualified `binding_origins`, expression reads/writes and `match_resolutions`. The current importer initializes empty `formal_projection`, `source_index` and callable-analysis tables. The wire's formal attachment retains a binding ordinal plus a **range**, not the exact canonical source-site identity. This is not a safe basis for reconstructing full source-owned attachment parity. [Lowering consumer](https://github.com/aureat/phalcom-lang/blob/2939e9d41640dc1eca3fbbc9b8e20529ab05ef6a/phalcom-core/src/modules/semantic_lowering.rs#L1155-L1215), [source-site owning relationship](https://github.com/aureat/phalcom-lang/blob/2939e9d41640dc1eca3fbbc9b8e20529ab05ef6a/phalcom-semantic/src/source_index/mod.rs#L120-L265), [imported empty owners](https://github.com/aureat/phalcom-lang/blob/2939e9d41640dc1eca3fbbc9b8e20529ab05ef6a/phalcom-semantic/src/artifact/import.rs#L2229-L2280).
4. **Checked static applications already belong to `CheckedStaticBody`, but the portable operational node loses their `StaticApplicationId` and full checked signature.** Producer ownership is the enclosing `StaticBodyId` plus owner-local ordinal. Operational export currently retains just target and recursively encoded arguments. Treat the current document's wording about `CallableAnalysis` retention as **stale for this real producer**; do not duplicate the checked-body authority to conform to prose. [Producer and identity](https://github.com/aureat/phalcom-lang/blob/2939e9d41640dc1eca3fbbc9b8e20529ab05ef6a/phalcom-semantic/src/checker/context.rs#L1613-L1635), [native row](https://github.com/aureat/phalcom-lang/blob/2939e9d41640dc1eca3fbbc9b8e20529ab05ef6a/phalcom-semantic/src/static_definition.rs#L124-L160), [operational wire](https://github.com/aureat/phalcom-lang/blob/2939e9d41640dc1eca3fbbc9b8e20529ab05ef6a/phalcom-compiler-artifact/src/semantic/operational.rs#L38-L69).
5. **A concrete, separately actionable fingerprint bug is present in the witness importer.** It constructs `ConformanceAssociatedTypePlan::empty(impl_id)` (which computes a fingerprint of **empty** bindings), then mutates `associated_type_plan.bindings = bindings` without recomputing the fingerprint. The source owner computes the product fingerprint from the actual binding map. Imported nonempty associated-type plans therefore carry a stale internal fingerprint, with possible downstream fingerprint/invalidation and witness consistency failures. [Importer](https://github.com/aureat/phalcom-lang/blob/2939e9d41640dc1eca3fbbc9b8e20529ab05ef6a/phalcom-semantic/src/artifact/import.rs#L2138-L2162), [empty constructor](https://github.com/aureat/phalcom-lang/blob/2939e9d41640dc1eca3fbbc9b8e20529ab05ef6a/phalcom-semantic/src/impls.rs#L591-L612), [fingerprint computation](https://github.com/aureat/phalcom-lang/blob/2939e9d41640dc1eca3fbbc9b8e20529ab05ef6a/phalcom-semantic/src/impls.rs#L1190-L1209).
6. **Full trait/witness ownership remains materially lossy.** The trait DTO omits per-member signatures, visibility, source/default provenance and associated-type domains; the importer uses synthetic `KindId(0)` and public/default member attributes. It also appears to ignore exported `super_traits`. Witness DTOs reduce rich proof states to a `u8`, omit requirement views and failures, and imports silently manufacture substitute proof payloads. [Trait DTO](https://github.com/aureat/phalcom-lang/blob/2939e9d41640dc1eca3fbbc9b8e20529ab05ef6a/phalcom-compiler-artifact/src/semantic/world.rs#L357-L379), [native trait members](https://github.com/aureat/phalcom-lang/blob/2939e9d41640dc1eca3fbbc9b8e20529ab05ef6a/phalcom-semantic/src/traits.rs#L150-L191), [importer](https://github.com/aureat/phalcom-lang/blob/2939e9d41640dc1eca3fbbc9b8e20529ab05ef6a/phalcom-semantic/src/artifact/import.rs#L1890-L1972).

**Strategic conclusion:** The implementer should maintain a *single field-level source-of-truth and consumer-read manifest*, not multiple ad hoc partial DTO expansions. Complete native owner → stable wire → validated fresh relocation → owning snapshot/index → compiler consumer as one vertical slice. The underlying P7 architecture remains viable and worth preserving: shared type/static graph relocation, explicit loaded-image injection, source-denied external module authority, budget-aware frozen `FlowGraph`, and fail-closed publication are appropriate foundations.

---

## 2. Evidence and severity conventions

- **CONFIRMED / source:** Read in the pinned remote tree; reproducible by opening the linked file/lines.
- **CONFIRMED / control flow:** Follows necessarily from unconditional checked branches or assignment/constructor behavior in the pinned source. Runtime is not independently executed.
- **LATENT:** Code behind a currently rejecting guard is incorrect or incomplete. It must be fixed *before* guard removal; current production loader does not normally expose the erroneous published image.
- **HIGH-RISK / not measured:** A well-founded algorithmic, determinism, or malformed-input risk that needs a targeted reproducer before calling it an observed failure.
- **REPORTED:** Earlier implementer records show a particular focused test result; not a result independently rerun in this audit.

| ID | Severity | Evidence | Finding | Contract / owning location |
|---|---|---|---|---|
| **S2-01** | **BLOCKER** | CONFIRMED / control flow | Positive semantic-world `decode()` test paths unconditionally error due budget guard. | `phalcom-compiler-artifact::semantic::world`; R1/R7; test truthfulness |
| **S2-02** | **BLOCKER** | CONFIRMED / source | `CallableAnalysis` field manifest advertises missing wire families; partial DTO and no full import. | T7.B/R3; `analysis.rs`, `world.rs`, `export.rs`, `import.rs` |
| **S2-03** | **BLOCKER** | CONFIRMED / source | Imported callable/formal/source attachments are absent, incompatible with compiler lowering's required reads. | T7.B/T8/T12; `semantic_lowering.rs`, `source_index`, `import.rs` |
| **S2-04** | **BLOCKER** | CONFIRMED / source | Checked operational templates are detached; static definitions/constants/cases/module bindings are not reconstructed, and source checked bodies are not reliably exported as operational templates. | T7.B/T8; `operational.rs`, `export.rs`, `import.rs` |
| **S2-05** | **HIGH** | CONFIRMED / source, LATENT | Checked-body `StaticApplication` wire drops owner-qualified ID and exact checked signature; unsupported nested kinded-expression argument path. | `CheckedStaticBody`, operational transport; R3 |
| **S2-06** | **HIGH** | CONFIRMED / source, LATENT | Trait import reconstructs synthetic domains/member visibility/default-source facts and omits supertrait reattachment. | Trait headers/surfaces; R3/R4 |
| **S2-07** | **HIGH** | CONFIRMED / source, LATENT | Conformance witness proof states, requirement views, applicability and failure details lose authority in wire/import. | `ConformanceWitnessPlan`; R4 |
| **S2-08** | **HIGH** | CONFIRMED / control flow, LATENT | Imported associated-type-plan fingerprint is computed before bindings are installed and is not recomputed. | `ConformanceAssociatedTypePlan`; R4/T8 |
| **S2-09** | **HIGH** | CONFIRMED / source, LATENT | Imported dispatch callable knowledge is fabricated as assumed `TypeId(0)`. | `DeclarationSurface` / `SurfaceDispatchResolver`; R3 |
| **S2-10** | **HIGH** | CONFIRMED / source, LATENT | Effective callable definition provenance and inherent context are replaced with `PrimaryDeclaration/0/None`. | `EffectiveCallableDefinition`; R3/T8 |
| **S2-11** | **HIGH** | CONFIRMED / source | Required derived conformance, trait dispatch and module/static owner indexes are not rebuilt or attached. | T8; `snapshot.rs`, `trait_dispatch.rs`, `import.rs` |
| **S2-12** | **HIGH** | CONFIRMED / source, LATENT | Semantic-world structural validator does not establish complete witness/trait/conformance cross-owner and uniqueness invariants; unchecked raw `u8` tags are mapped to valid semantic categories. | `world.rs`, `import.rs`; integrity |
| **S2-13** | **MEDIUM/HIGH** | HIGH-RISK / not measured | Root registration orders some type environments by native `TypeParameterId`, potentially changing stable root ordinals across independent allocation histories. | `roots.rs`; R2 deterministic export |
| **S2-14** | **MEDIUM** | CONFIRMED / algorithm | Denotation owner validation uses repeated linear searches through semantic-family collections, potentially quadratic in large family/capture workloads. | `world.rs`; bounded validation performance |
| **S2-15** | **HIGH** | CONFIRMED / test design | Focused wire and source-index tests do not establish real artifact source/import parity; many core tests do not reach intended assertions. | G7.3, G7.5/6, G7.8–11 |

These findings refine, not negate, Slice 1. S2-04/06/07/09/10/11 identify particular ways Slice 1's broad `S1-04/S1-05` failures arise; S2-01, S2-05, S2-08, S2-12–14 add new concrete discriminators.

---

## 3. Full callable-analysis field inventory and first implementation contract

### 3.1 Canonical owner requirements

`checker::analysis::CallableAnalysis` is a semantic *owner graph*, not an execution summary. Fields are not independently replaceable by inferred guesses: expression status and `TypeKnowledge`, causal invalidity and diagnostic roots, lexical binding origin/versions, call and trait evidence, flow facts, context-owned denotations, match/pattern decisions, construction/iteration resolutions, owner-local explanation IDs, call/semantic dependency records and compiler attachments jointly describe what the checker actually proved. [Native owner definition](https://github.com/aureat/phalcom-lang/blob/2939e9d41640dc1eca3fbbc9b8e20529ab05ef6a/phalcom-semantic/src/checker/analysis.rs#L672-L720).

`CallableAnalysisFieldManifest` already classifies these fields as wire, except internal incidents (approved omission only when actually empty/rejected) and the body product fingerprint (derived **after relocation**). The problem is that the manifest is aspirational: it is not verified to correspond to implemented DTO fields and compiler reconstruction. [Manifest](https://github.com/aureat/phalcom-lang/blob/2939e9d41640dc1eca3fbbc9b8e20529ab05ef6a/phalcom-semantic/src/artifact/manifest.rs#L35-L65).

| Native family/field | Pinned wire/export state | Import/consumer state | Required correction |
|---|---|---|---|
| `callable`, `body_range`, `body_source_owner`, overall status | Present as stable callable/source owner, span and status | Import currently rejects analyses; module-body owner relation must survive | Validate actual owner and module-site authority, including synthetic `<main>` without inventing a signature |
| `expressions` | Only `expression_count`, sparse denotations and typed call-specialization rows | Full `ExpressionAnalysis` judgments, read/write targets, statuses and causal links absent | Export/import all checked expression rows with owner-local IDs and exact references |
| `bindings` | Sparse denotations, origins and range attachments; no complete `BindingState` rows | No reconstructed binding table | Preserve parameter linkage, contract, current evidence, causal state and exact body/local/global storage distinction |
| `binding_origins` | Explicit `BodyLocal` / `ModuleValue {module,name,import_binding}` | No full body reconstruction and no exercising compiler binding graph | Validate module symbol/import alias against loaded linker authority; use qualified lowering identity |
| `for_iteration_lanes`, positional and invalid expansions | Type roots registered for many fields; no full wire DTO | Missing consumer evidence after cold import | Transport selected signatures/generic evidence, shape, source and exact execution plan |
| `sequence_literals` | Roots cover result, element and extent; no owner record | Missing | Preserve family/shape/exact-expansion construction plans |
| `associated_resolutions`, callable reference and family applications | Root registration and denotation helpers partially established | No installed per-expression resolution map | Preserve checker-captured target/member/order/dispatch evidence, no lookup at load |
| `match_resolutions`, `pattern_site_resolutions` | Absent in current analysis DTO and cited root census | Absent from lowering | Preserve all arm/pattern decisions and site-to-target identities |
| `flow_graph` | Portable nodes, edges, entry and exits; budgeted frozen importer exists as helper | Not attached to reconstructed analyses | Validate and restore one graph per callable with owner-relative IDs and source/frozen read parity |
| `entry_flow`, exit return/throw flows, return validation | Only `exits_unreachable` flag | Missing binding/field flow facts and return-causal state | Persist all normal/abrupt exits and structured contract validation |
| diagnostics / causal explanations / `return_explanation` | Closure maps/explanation validation groundwork, no full DTO | Missing | Preserve exact invalid outcomes, cause sharing, provenance and explanation graph; never synthesize Ready |
| `dependencies`, `semantic_dependencies` | No complete callable DTO | Missing exact query/refresh evidence | Persist stable owner evidence, reconstruct dependency fingerprints after relocation |
| constructor-super initializer | Range + stable callable in DTO | No installed `CallableAnalysis` to consume it | Preserve exact prologue site and target with graph identity checks |
| `dependency_fingerprint` | Manifest says derived after relocation | No implemented complete imported-body recomputation | Hash restored semantic facts/typed call arguments, not only copied source hash |
| internal incidents | Correctly classified omitted only when absent | Source artifact preflight asserts zero for canonical image | Keep hard rejection and ensure no incident is concealed by other fallback |

The native-to-wire disparity is visible in the partial [`CallableAnalysisRecordV1`](https://github.com/aureat/phalcom-lang/blob/2939e9d41640dc1eca3fbbc9b8e20529ab05ef6a/phalcom-compiler-artifact/src/semantic/world.rs#L298-L322). The actual [`export_callable_analysis_record`](https://github.com/aureat/phalcom-lang/blob/2939e9d41640dc1eca3fbbc9b8e20529ab05ef6a/phalcom-semantic/src/artifact/export.rs#L948-L1162) exports just those fields. [`ensure_complete_callable_analysis_payload`](https://github.com/aureat/phalcom-lang/blob/2939e9d41640dc1eca3fbbc9b8e20529ab05ef6a/phalcom-semantic/src/artifact/import.rs#L630-L637) refuses nonempty input, and [`import.rs#L1873-L1890`](https://github.com/aureat/phalcom-lang/blob/2939e9d41640dc1eca3fbbc9b8e20529ab05ef6a/phalcom-semantic/src/artifact/import.rs#L1873-L1890) initializes an empty `CallableAnalysisTable`.

### 3.2 Owner-local identities and cross-reference closure

An owner-qualified artifact reference must distinguish `(<stable callable>, expression ordinal)`, `(<stable callable>, binding ordinal)`, flow-node/edge ordinal, explanation ordinal, and cause ordinal. Two distinct callable analyses can legitimately assign the same local ordinal. Native `BindingId`/`ExpressionId` numbers are not global portable identities. `CallableAnalysisOrdinalMap::new` performs useful existing work: it enumerates owner-local expressions/bindings/flow nodes/edges/causes, checks edge adjacency consistency, and ensures expression binding/explanation and diagnostic links reference present owner-local IDs. [Ordinal-map checks](https://github.com/aureat/phalcom-lang/blob/2939e9d41640dc1eca3fbbc9b8e20529ab05ef6a/phalcom-semantic/src/artifact/export.rs#L780-L948).

**Retain and extend this single map** rather than creating parallel ordinal allocators per DTO family. The importer needs the reverse map and must reject dangling, duplicate, wrong-owner and inconsistent references *before publication*. The map should preserve exact sharing and structural meaning, not merely produce a dense count that happens to fit a range.

**Current gap:** `expression_count` and sparse references cannot reconstruct the original `ExpressionAnalysisIndex`; knowing an ordinal is within `[0, expression_count)` does not prove a complete owning record exists at that ordinal. The current validation checks sparse denotation/specialization ordinals, but has no complete expression row to which those references are attached. [Wire validation](https://github.com/aureat/phalcom-lang/blob/2939e9d41640dc1eca3fbbc9b8e20529ab05ef6a/phalcom-compiler-artifact/src/semantic/world.rs#L849-L957).

### 3.3 Typed generic call specializations are good groundwork, not complete transport

The partial call-specialization row correctly distinguishes `Term(TypeId)`, `RecordRow(RecordRowId)`, and `DynamicType`, with parameter roots and category checks. The exporter refuses nonempty `specialization.conformances`, and `import_callable_call_specializations` restores an empty `conformances` array. That is an intentional **fail-closed partial contract**. [Export of call specializations](https://github.com/aureat/phalcom-lang/blob/2939e9d41640dc1eca3fbbc9b8e20529ab05ef6a/phalcom-semantic/src/artifact/export.rs#L1050-L1128), [import helper](https://github.com/aureat/phalcom-lang/blob/2939e9d41640dc1eca3fbbc9b8e20529ab05ef6a/phalcom-semantic/src/artifact/import.rs#L802-L846).

**Do not collapse** `DynamicType` into missing binding, fabricate `TypeId(0)`, or substitute static proof for a Dynamic-dependent conformance. Finish the existing typed environment through checked call resolution, conformance evidence, body/query fingerprints and runtime lowering. Cover exact `Box<Int>` / `Box<Dynamic>`, forwarding, a nested `List<T>`, row arguments, zero proof slots when genuinely Dynamic, and cold/import parity.

---

## 4. Core lowering and `<main>` formal-source ownership

### 4.1 Specific consumer contract

Core lowering builds a per-module range-to-binding attachment index and distinguishes these qualified identities:

```rust
LoweringBindingIdentity::BodyLocal { callable, binding }
LoweringBindingIdentity::ModuleValue { symbol }
```

For callable-local declaration bindings it reads `snapshot.formal_projection.binding_attachment_range(callable, binding)`. For reads/writes it iterates `analysis.expressions` and uses `binding_read` / `binding_write`. It additionally consumes `analysis.match_resolutions` and other checked operations. A missing attachment becomes a `ProjectionError::MissingFormalBindingAttachment`; collisions between different identities on a single range become `AmbiguousBindingAttachment`. [Binding identity and collision behavior](https://github.com/aureat/phalcom-lang/blob/2939e9d41640dc1eca3fbbc9b8e20529ab05ef6a/phalcom-core/src/modules/semantic_lowering.rs#L803-L857), [consumer call path](https://github.com/aureat/phalcom-lang/blob/2939e9d41640dc1eca3fbbc9b8e20529ab05ef6a/phalcom-core/src/modules/semantic_lowering.rs#L1155-L1215).

`SourceScopeIndex` source attachments are actually indexed by `SourceSiteId`, owned by an explicit `SourceOwner::Callable` or `SourceOwner::Module`, and the normal source-building algorithm uses binding/parameter site identity and carefully handled fallback/rebasing. The `<main>` case must **not** use the callable count/order rebase shortcut: a module shard can contain imports, declarations and nested closures. [Source index owner logic](https://github.com/aureat/phalcom-lang/blob/2939e9d41640dc1eca3fbbc9b8e20529ab05ef6a/phalcom-semantic/src/source_index/mod.rs#L140-L268).

### 4.2 Why the current portable attachment is insufficient

Current `CallableBindingSourceAttachmentV1` contains only `{ binding_ordinal, range }`; the serializer obtains that range from the source index. A source range is a **locator**, not the identity of the exact declaration site. Without full owner-held formal graph/site identity, constructing the native formal projection by scanning for matching ranges/name is not a faithful import. A reliable representation must preserve a stable, owner-qualified canonical source-site identity (or an equivalent independently validated attachment authority), and reconnect that identity to the imported callable graph without opening source syntax. [Wire DTO](https://github.com/aureat/phalcom-lang/blob/2939e9d41640dc1eca3fbbc9b8e20529ab05ef6a/phalcom-compiler-artifact/src/semantic/world.rs#L224-L241), [source export](https://github.com/aureat/phalcom-lang/blob/2939e9d41640dc1eca3fbbc9b8e20529ab05ef6a/phalcom-semantic/src/artifact/export.rs#L1021-L1050).

**Evidence-sensitive qualification:** The present range row could form *one component* of an accepted compiler-only formal attachment if the complete owner graph guarantees that no ambiguous identity exists and the formal reader is materialized from that owner. The issue is that neither such invariant nor the materialized formal reader currently exists. The report does not claim that every non-overlapping range fails in isolation.

### 4.3 `<main>` and ambient module binding obligations

The adopted amendment correctly separates:

- `body_source_owner`: module namespace for synthetic top-level body facts versus source-callable namespace for regular bodies;
- `storage origin`: callable-local binding versus module-owned `SymbolId` with optional exact import provenance;
- lexical scope and exact `SourceSiteId`/occurrence linkage;
- canonical callable body identity and source declaration owner.

Do not invent a normal callable signature row for synthetic `<main>` or identify it by selector spelling during import. A top-level lambda parameter is a **body-local** binding even though its source site belongs to a module. Two parameters named `text` in distinct top-level lambdas must remain distinct; two body-local handles seeded from the *same* canonical module symbol must resolve to the same `ModuleValue` storage identity where applicable. The exporter already rejects absent storage-origin mappings and invalid module-value formal attachments, which is appropriate. [Export's body-attachment construction](https://github.com/aureat/phalcom-lang/blob/2939e9d41640dc1eca3fbbc9b8e20529ab05ef6a/phalcom-semantic/src/artifact/export.rs#L1971-L2003), [source-index module owner test](https://github.com/aureat/phalcom-lang/blob/2939e9d41640dc1eca3fbbc9b8e20529ab05ef6a/phalcom-semantic/tests/semantic/integration/source_index.rs#L1183-L1415), [adopted plan](PERF003.C2.P7-remaining-execution-plan.md).

**Required discriminator:** Build the real Universe source image, select `universe:json` synthetic top-level body and two top-level lambdas with `text` parameters, then compare the imported *formal binding graph* and generated lowering identities with the source graph. Include empty and documentation-bearing module preambles. A test that only sees both sources' textual ranges is inadequate.

---

## 5. Static definitions, checked applications and actual operational authority

### 5.1 Correct ownership decision from live source

The checked application identity is:

```rust
StaticApplicationId {
    body: StaticBodyId,  // enclosing *checked* body owner
    ordinal: u32,        // body-local structural application ordinal
}
```

`CheckingContext::record_checked_static_application` uses `current_static_body_owner` to construct the row, and `checker/static_body.rs` moves the accumulated rows into the owning `CheckedStaticBody`. This is the correct existing owner boundary, including two bodies both using ordinal zero. It is **not** the selected callee's `DeclarationId`, and ordinary callable finalization is a separate producer. [Producer](https://github.com/aureat/phalcom-lang/blob/2939e9d41640dc1eca3fbbc9b8e20529ab05ef6a/phalcom-semantic/src/checker/context.rs#L1613-L1635), [static body publication](https://github.com/aureat/phalcom-lang/blob/2939e9d41640dc1eca3fbbc9b8e20529ab05ef6a/phalcom-semantic/src/checker/static_body.rs#L447-L463), [checked row](https://github.com/aureat/phalcom-lang/blob/2939e9d41640dc1eca3fbbc9b8e20529ab05ef6a/phalcom-semantic/src/static_definition.rs#L124-L161).

**Architectural recommendation:** Correct the amended execution document's inaccurate sentence about `CallableAnalysis` retention to name the actual `CheckedStaticBody` owner. This is an evidence-backed reconciliation of the accepted identity principle, not a request to duplicate or relocate the native producer state.

### 5.2 Current portable operational node is lossy

`OperationalOpV1::StaticApplication` encodes `{target, arguments}`; the native `OperationalNode` also contains target/arguments only. Neither carries `StaticApplicationId`, the exact selected `StaticDefinitionSignature`, original application source site, or full structured argument parameter matching. The denotation codec independently rejects `SemanticDenotationV1::StaticApplication` because no checked application row exists to resolve it. [Portable node](https://github.com/aureat/phalcom-lang/blob/2939e9d41640dc1eca3fbbc9b8e20529ab05ef6a/phalcom-compiler-artifact/src/semantic/operational.rs#L38-L69), [projection](https://github.com/aureat/phalcom-lang/blob/2939e9d41640dc1eca3fbbc9b8e20529ab05ef6a/phalcom-semantic/src/artifact/operational.rs#L603-L616), [denotation rejection](https://github.com/aureat/phalcom-lang/blob/2939e9d41640dc1eca3fbbc9b8e20529ab05ef6a/phalcom-compiler-artifact/src/semantic/world.rs#L994-L1006).

Transport either the owner-held checked application rows plus validated ordinal references in operations, or an equivalent fully checked operational product that preserves the exact facts consumers require. **Never** reconstruct application meaning from the callee static definition or re-run static application checking at import time. Keep the fail-closed guard until exact source/import parity and evaluated-use parity exist.

`OperationalTemplate::from_checked_body` also explicitly rejects `CheckedStaticApplicationArgument::KindedExpression` with `unsupported kinded expression in static application`; for a root `TypeForm`, it uses the **first** checked application as a shortcut. This is an implementation limitation and potential wrong-association risk when there are multiple nested applications, not proof of a manifested test failure. [Root lowering](https://github.com/aureat/phalcom-lang/blob/2939e9d41640dc1eca3fbbc9b8e20529ab05ef6a/phalcom-semantic/src/artifact/operational.rs#L38-L100), [nested TypeForm path](https://github.com/aureat/phalcom-lang/blob/2939e9d41640dc1eca3fbbc9b8e20529ab05ef6a/phalcom-semantic/src/artifact/operational.rs#L164-L211). Add tests for two applications in a body and a nested kinded argument, each paired to its source expression/target/owner.

### 5.3 Templates are not exported or installed as complete static definitions

The exporter enumerates `snapshot.static_definitions` but pushes a template only if `checked_body.root` is **already** `StaticDefinitionBodyRepr::Operational`; the ordinary checker also accepts source `Expression`, `Block`, and `TypeForm` representations, and there is a separately defined `OperationalTemplate::from_checked_body` transform. The current exporter does **not** invoke that transform at the viewed seam. Thus, absent some upstream conversion proven separately, a checked source body would be omitted from `checked_definitions` even if the existing static-family coverage guard were simply removed. [Export filter](https://github.com/aureat/phalcom-lang/blob/2939e9d41640dc1eca3fbbc9b8e20529ab05ef6a/phalcom-semantic/src/artifact/export.rs#L1854-L1870), [accepted source representations](https://github.com/aureat/phalcom-lang/blob/2939e9d41640dc1eca3fbbc9b8e20529ab05ef6a/phalcom-semantic/src/static_definition.rs#L90-L109), [lowerer](https://github.com/aureat/phalcom-lang/blob/2939e9d41640dc1eca3fbbc9b8e20529ab05ef6a/phalcom-semantic/src/artifact/operational.rs#L30-L115).

On import, `import_checked_static_definitions` returns a **detached `Vec<OperationalTemplate>`**. `template_to_static_definition` exists and can build a source-independent checked definition from a template/signature, but the semantic-world import leaves `snapshot.static_definitions` empty and the signature/product attachment is not composed. [Import helper](https://github.com/aureat/phalcom-lang/blob/2939e9d41640dc1eca3fbbc9b8e20529ab05ef6a/phalcom-semantic/src/artifact/operational.rs#L798-L870), [current snapshot construction](https://github.com/aureat/phalcom-lang/blob/2939e9d41640dc1eca3fbbc9b8e20529ab05ef6a/phalcom-semantic/src/artifact/import.rs#L2240-L2280).

**Required correction:** Include complete static-definition signatures (parameter names/domains/modes/type params/results/normalized constraints) and compiler-usable checked templates, construct genuine owning `StaticDefinitionTable` products, restore constant products and module-owned binding catalogs, and validate cross-table ID authority before publishing. If the evaluator still needs a raw `WhereClauseSyntax`/AST node for an imported definition, normalize once at export into source-independent constraints instead of shipping source syntax or rechecking on load. Failure to represent a supported static feature must block publication; do not turn its definition into a no-op.

---

## 6. Trait surfaces, conformance proof graphs, associated plans and indexes

### 6.1 Trait surfaces lose native contract dimensions

`TraitSurfaceRecordV1` carries a declaration, optional generics, supertrait list, lists of required/default callable IDs, and associated-type names. The native `TraitSurfaceMember` retains a signature, visibility, source and default-source provenance, while `TraitAssociatedTypeRequirement` retains an exact `TypeMemberDomain` and source. These are independent semantic fields: a string name and a default/required boolean do not represent them. [DTO](https://github.com/aureat/phalcom-lang/blob/2939e9d41640dc1eca3fbbc9b8e20529ab05ef6a/phalcom-compiler-artifact/src/semantic/world.rs#L357-L365), [native contracts](https://github.com/aureat/phalcom-lang/blob/2939e9d41640dc1eca3fbbc9b8e20529ab05ef6a/phalcom-semantic/src/traits.rs#L150-L191).

The importer creates associated requirements with **`TypeMemberDomain::Kind(KindId(0))`**, reconstructs member visibility as Public, creates default source spans, sets default member source to `None`, and skips a member entirely if its callable signature lookup fails. It does not reattach `rec.super_traits` within this loop. [Importer](https://github.com/aureat/phalcom-lang/blob/2939e9d41640dc1eca3fbbc9b8e20529ab05ef6a/phalcom-semantic/src/artifact/import.rs#L1890-L1973). Those defaults are **not** equivalent to the original trait surface and can affect trait checking, dispatch, generic conformance and source diagnostics.

**Repair:** Transport exact member/associated requirement product fields from their canonical owners, relocate type/domain references, validate IDs, preserve visibility and default-authority/source, reattach supertrait contracts, and require every referenced callable row to exist or fail the entire artifact. The source/import discriminator must compare generic trait + associated type requirements, nonpublic/default members and inherited supertrait obligations at field level, not merely nonempty table counts.

### 6.2 Witness plan records reduce rich proof states to hints

The native [`ConformanceWitnessPlan`](https://github.com/aureat/phalcom-lang/blob/2939e9d41640dc1eca3fbbc9b8e20529ab05ef6a/phalcom-semantic/src/impls.rs#L1894-L1912) contains instantiated `requirement_views`, selected `requirements`, associated-type binding plans (including failures and diagnostics), `invalid_explicit_members`, exact completeness status with reason payloads, and a semantic fingerprint. The `WitnessPlanV1` schema retains selected callable identities, an ordered layout digest, associated bindings and `completeness: u8` but omits those other facts. [DTO](https://github.com/aureat/phalcom-lang/blob/2939e9d41640dc1eca3fbbc9b8e20529ab05ef6a/phalcom-compiler-artifact/src/semantic/world.rs#L35-L72).

On import, incomplete status becomes `Incomplete { failures: [] }`; Unknown is forced to `NoTypeEvidence`; Blocked is forced to `RecursiveFixpoint`; Dynamic gets a generic `"dynamic"` reason; budget exhaustion constructs `Steps, 0, 0`; `InternalFailure` gets fabricated text; `requirement_views` is always empty; `invalid_explicit_members` is always false; associated plan failures/diagnostics disappear. [Importer](https://github.com/aureat/phalcom-lang/blob/2939e9d41640dc1eca3fbbc9b8e20529ab05ef6a/phalcom-semantic/src/artifact/import.rs#L2052-L2164). The exporter currently rejects a subset of such cases, which prevents silent publication today but does not complete the contract. [Export guard](https://github.com/aureat/phalcom-lang/blob/2939e9d41640dc1eca3fbbc9b8e20529ab05ef6a/phalcom-semantic/src/artifact/export.rs#L2202-L2234).

**Repair:** Preserve exact proof state and payloads, specialized requirement views, selected conditional/associated applicability, invalid explicit member flags and associated failure records; reconstitute them in the proper owning witness table without recomputing checker selection. Recalculate native fingerprints **after** relocation and compare the relocated semantics against the wire's portable/source identity where the contract requires agreement. For genuinely unsupported proof variants, preserve fail-closed behavior rather than fabricate a Complete plan.

### 6.3 Specific stale associated-plan fingerprint: deterministic bug

The source `ConformanceAssociatedTypePlan::empty()` computes `plan.fingerprint = associated_type_plan_fingerprint(&plan)`. The fingerprint function hashes the plan `impl_id`, **each binding's requirement/value/source impl**, and failures. The importer's assignment sequence is:

```rust
let mut associated_type_plan = ConformanceAssociatedTypePlan::empty(impl_id.clone());
associated_type_plan.bindings = bindings;
// No fingerprint recomputation after the mutation.
```

For a nonempty binding set, this violates the native plan's fingerprint invariant even if every individual imported binding is structurally correct. Furthermore, the imported witness's parent `fingerprint` is copied from the source payload, while the genuine witness fingerprint function incorporates `plan.associated_type_plan.fingerprint`, view selections, and invalidity. Thus both internal consistency and downstream invalidation parity require more than preserving the original `u64`. [Importer](https://github.com/aureat/phalcom-lang/blob/2939e9d41640dc1eca3fbbc9b8e20529ab05ef6a/phalcom-semantic/src/artifact/import.rs#L2138-L2160), [fingerprint owner](https://github.com/aureat/phalcom-lang/blob/2939e9d41640dc1eca3fbbc9b8e20529ab05ef6a/phalcom-semantic/src/impls.rs#L1190-L1209), [parent witness fingerprint](https://github.com/aureat/phalcom-lang/blob/2939e9d41640dc1eca3fbbc9b8e20529ab05ef6a/phalcom-semantic/src/impls.rs#L2318-L2355).

**Immediate isolated regression:** Build a witness with one nontrivial associated type binding. In a helper or a repaired full import path, assert `restored.associated_type_plan.fingerprint == associated_type_plan_fingerprint(&restored.associated_type_plan)` and check that modifying its binding changes the plan fingerprint and dependent witness fingerprint. The current assignment violates the first assertion. Recomputing a fingerprint alone is **not** enough to close R4 while the other witness fields are missing.

### 6.4 Derived conformance and trait-dispatch authority

`TraitDispatchIndex::build(&ConformanceIndex, &TraitSurfaceTable)` already provides a deterministic derivation from owner inputs, sorting/deduplicating bucket contributions; this is the accepted direction for a **derived** index. However, import initializes `TraitDispatchIndex::default()` instead of calling the builder after validated trait and conformance reconstruction. It also creates an empty `DerivedConformanceRuleIndex` although the manifest classifies derived conformance rules as required primary wire authority. [Builder](https://github.com/aureat/phalcom-lang/blob/2939e9d41640dc1eca3fbbc9b8e20529ab05ef6a/phalcom-semantic/src/trait_dispatch.rs#L122-L157), [manifest classifications](https://github.com/aureat/phalcom-lang/blob/2939e9d41640dc1eca3fbbc9b8e20529ab05ef6a/phalcom-semantic/src/artifact/manifest.rs#L195-L216), [import defaults](https://github.com/aureat/phalcom-lang/blob/2939e9d41640dc1eca3fbbc9b8e20529ab05ef6a/phalcom-semantic/src/artifact/import.rs#L2230-L2280).

A synthesized empty derived index is **not** an acceptable reconstruction, even if every current test fixture happens to use a trait that does not traverse that bucket. Populate primary rules from portable owner rows, then build derived buckets from completed conformance/trait input and compare exact bucket membership/fingerprints to the source image. Avoid computing witness proof decisions during import; use already transported proof state.

### 6.5 Effective definitions and dispatch callable knowledge

Importer reconstructs each `EffectiveCallableDefinition` from a signature as `PrimaryDeclaration`, member index zero, and no `InherentBodyContext`. The real owner distinguishes primary, inherent impl and conformance witness provenance and retains generic covering/body-context information. Those facts affect source-relative `Self`, accepted body context and provenance-sensitive compiler consumers. [Native owner](https://github.com/aureat/phalcom-lang/blob/2939e9d41640dc1eca3fbbc9b8e20529ab05ef6a/phalcom-semantic/src/impls.rs#L4447-L4515), [import default](https://github.com/aureat/phalcom-lang/blob/2939e9d41640dc1eca3fbbc9b8e20529ab05ef6a/phalcom-semantic/src/artifact/import.rs#L1867-L1885).

Separately, imported `DeclarationSurface` callable entries get `TypeKnowledge::assumed(TypeId(0), EvidenceOrigin::Flow)` rather than the owner's actual canonical callable type evidence. `TypeId(0)` is a native arena coordinate, **not a justified imported callable result**. This can poison type-dependent dispatch and future compiler overlays if the guard is removed. [Import surface loop](https://github.com/aureat/phalcom-lang/blob/2939e9d41640dc1eca3fbbc9b8e20529ab05ef6a/phalcom-semantic/src/artifact/import.rs#L1974-L2022).

**Repair:** Define exact owner-held definition provenance, source-member identity and inherent body context wire forms and materialize them into the true `CallableDefinition` owner. Persist callable type knowledge from canonical surfaces or reconstruct it only from fully validated equivalent owner facts, using the *existing* TypeRelocation. Add tests with a genuine inherent method, class-side type context, explicit conformance witness method, and changed physical native type allocation history.

---

## 7. Semantic-world validator hardening — defects hiding behind fail-closed guards

`SemanticWorldV2::validate_inner` checks global family counts; performs extensive enum requirement and callable-analysis checks; and validates denotation graph edges. Those are useful. However, the currently inspected validator has **no corresponding exhaustive iteration** over witness plans, trait surfaces and conformance heads that establishes their uniqueness, ordering, required cross-owner references, or all raw numeric category tags. It checks family length in this region, then concentrates on signature, enum and callable records before returning. [Validation body](https://github.com/aureat/phalcom-lang/blob/2939e9d41640dc1eca3fbbc9b8e20529ab05ef6a/phalcom-compiler-artifact/src/semantic/world.rs#L671-L960).

Specific latent failures to address:

- `ConformanceCompleteness` has a portable `u8`, rather than a closed tagged enum with field-bearing variants. Import maps any unrecognized value to a fabricated internal failure, not a precisely rejected invalid tag; exact failure reports are absent. [DTO](https://github.com/aureat/phalcom-lang/blob/2939e9d41640dc1eca3fbbc9b8e20529ab05ef6a/phalcom-compiler-artifact/src/semantic/world.rs#L60-L72), [import mapping](https://github.com/aureat/phalcom-lang/blob/2939e9d41640dc1eca3fbbc9b8e20529ab05ef6a/phalcom-semantic/src/artifact/import.rs#L2126-L2144).
- Imported witness/field/constructor `side` and `origin` values are decoded from raw `u8`; importer branches of the form `0 => Instance, _ => Class` and `0,1,2 => ... , _ => DefaultOnly` can silently give unknown wire tags legitimate semantic meaning unless the validator rejects those exact values first. [Importer](https://github.com/aureat/phalcom-lang/blob/2939e9d41640dc1eca3fbbc9b8e20529ab05ef6a/phalcom-semantic/src/artifact/import.rs#L2070-L2101), [constructor mapping](https://github.com/aureat/phalcom-lang/blob/2939e9d41640dc1eca3fbbc9b8e20529ab05ef6a/phalcom-semantic/src/artifact/import.rs#L2166-L2211).
- Insertion into `witness_plans_map` uses `BTreeMap::insert` without checking a duplicate `impl_id`. A valid outer codec must reject duplicate/wrong-owner primary rows rather than let last-write-wins overwrite a witness. [Import insertion](https://github.com/aureat/phalcom-lang/blob/2939e9d41640dc1eca3fbbc9b8e20529ab05ef6a/phalcom-semantic/src/artifact/import.rs#L2144-L2163). The validator's relevant family count check alone is insufficient.
- Denotation validation repeatedly calls `validate_declaration_reference`, `associated_surface`, and `validate_associated_member_owner` through linearly scanned collections. Correctness checks are worthwhile but should use charged, validated, sorted/lookup indexes on the actual load path rather than increase worst-case validation complexity with every captured denotation. [Denotation validation](https://github.com/aureat/phalcom-lang/blob/2939e9d41640dc1eca3fbbc9b8e20529ab05ef6a/phalcom-compiler-artifact/src/semantic/world.rs#L990-L1149).

**Qualification:** These importer paths are unreachable in normal public load while the independent accounting guards remain. They are nonetheless required correctness fixes for the valid artifact importer, not mere future hardening; the guard's existence does not make current defaulting code acceptable when the blocker is removed.

### Complexity consideration

If each of `K` captured associated denotations validates owner/member authority by scanning a family of `D` declarations or records, the validation cost approaches **O(K × D)**, and may multiply further for member-in-family searches. P7 targets a substantial canonical Universe with thousands of semantic facts; prefer validated owner indices or sorted vectors plus binary search, construct them through the same bounded allocation transaction, and avoid duplicating full worlds. Do not remove the validation for speed: improve its data access shape.

---

## 8. Deterministic root registration — specific unverified counterexample

The present root registerer walks `TypeEnvironment` hash-backed bindings by collecting `(TypeParameterId, value)` pairs into a vector, sorting by **native `TypeParameterId`** and then issuing roots whose `product_subkey` is a sequential counter (`Roots::key` initializes at 100). This removes randomized HashMap enumeration, which is an improvement, but native parameter arena ordinals are not necessarily stable across independent stores with different allocation histories. [Root order and ordinal construction](https://github.com/aureat/phalcom-lang/blob/2939e9d41640dc1eca3fbbc9b8e20529ab05ef6a/phalcom-semantic/src/artifact/roots.rs#L105-L206).

**Failure hypothesis:** Semantically identical environments `{A ↦ T1, B ↦ T2}` built in two independent stores may number `A/B` in opposite native order. Sorting native IDs will then assign earlier/later portable root keys to different semantic parameters, potentially changing reachable graph traversal/order and final artifact-local numbering and bytes. **This has not been reproduced here**; it is a targeted determinism risk. The plan requires independence from physical root/allocation history.

**Discriminator:** Construct two fresh stores with identical declaration/parameter owner identities and equivalent generic environments but reversed interning/allocation history, including unreachable historical parameters. Register all canonical roots and compare the **entire artifact bytes and section digests**. If differing, order by validated stable *owner/parameter position* rather than native arena IDs, and ensure the type graph's root traversal itself is deterministic. Do not solve only by sorting final outer DTO rows after artifact IDs have already been assigned.

---

## 9. Testing audit — what currently proves what

### 9.1 Focused tests are valuable, but not an integrated owner parity gate

The pushed repository has material codec regressions, including:

- [`semantic_world_callable_analysis_test.rs`](https://github.com/aureat/phalcom-lang/blob/2939e9d41640dc1eca3fbbc9b8e20529ab05ef6a/phalcom-compiler-artifact/tests/semantic_world_callable_analysis_test.rs) — nine source-defined tests for status, owner-local flow, constructor range, Dynamic argument, owner mismatch, associated denotation ownership and captured behavioral targets;
- [`semantic_world_enum_requirements_test.rs`](https://github.com/aureat/phalcom-lang/blob/2939e9d41640dc1eca3fbbc9b8e20529ab05ef6a/phalcom-compiler-artifact/tests/semantic_world_enum_requirements_test.rs) — schema/status/order/owner/range tests;
- [`source_index.rs` integration tests](https://github.com/aureat/phalcom-lang/blob/2939e9d41640dc1eca3fbbc9b8e20529ab05ef6a/phalcom-semantic/tests/semantic/integration/source_index.rs#L1066-L1550) — useful source-built owner and formal-site discriminators, including synthetic module body, shared module values and shadowing;
- [`static_definition_bodies.rs` tests](https://github.com/aureat/phalcom-lang/blob/2939e9d41640dc1eca3fbbc9b8e20529ab05ef6a/phalcom-semantic/tests/semantic/integration/static_definition_bodies.rs#L1-L200) — source checked-body and static application coverage;
- [`canonical_artifact.rs` core tests](https://github.com/aureat/phalcom-lang/blob/2939e9d41640dc1eca3fbbc9b8e20529ab05ef6a/phalcom-core/tests/core/modules/canonical_artifact.rs#L1-L120) — intended full build/write/load/inline smoke path, currently blocked before loading.

**Crucial revision-sensitive contradiction:** The positive `semantic_world_callable_analysis_test` cases invoke `SemanticWorldV2::decode(...).expect("... decodes")`; on this exact pinned revision, `decode → decode_with_budget → decode_with_allocation_budget → validate_with_allocation_budget → require_semantic_validation_allocation_accounting` returns `UnaccountedBoundary` unconditionally. The analogous enum codec tests invoke the same path. Thus the previously recorded test pass counts cannot be considered a passing check *after this latest change*. [Positive tests at lines 137–138, 179–180, 243–244, 583–584, 675–676](https://github.com/aureat/phalcom-lang/blob/2939e9d41640dc1eca3fbbc9b8e20529ab05ef6a/phalcom-compiler-artifact/tests/semantic_world_callable_analysis_test.rs#L129-L180), [decoder code](https://github.com/aureat/phalcom-lang/blob/2939e9d41640dc1eca3fbbc9b8e20529ab05ef6a/phalcom-compiler-artifact/src/semantic/world.rs#L601-L669).

**Classification:** Guaranteed error from inspected control flow **if executed under the inspected code**; not an assertion that we executed Cargo and observed its exit status. This is a strong reason to run these two narrow suites immediately under the real checked-out head, report exact failures, and make the budget-aware validator safe and usable before making new codec pass claims. Do not introduce test-only decoding bypasses or water down `decode`'s contract to make synthetic tests green.

### 9.2 Missing discriminators

There are no demonstrated fresh-store **full** `CallableAnalysis` round-trips; no positive source-free full `SemanticSnapshot` load; no source/import compiler lowering parity for actual checked sources; no full static definition and constant persistence; no source/import complete conformance witness view and conditional applicability parity; no independent fingerprint recomputation proof for restored conformance; no stress tests validating worst-case family cross-reference cost; and no cold child process compiling a standalone real package with Universe source acquisition denied. These remain **NOT VERIFIED**, even where source-only or helper-level focused tests passed earlier.

### 9.3 Proposed executable gate matrix

| Gate | Test intent and required oracle | Expected current frontier | Owner / order |
|---|---|---|---|
| **S2.G1** | `SemanticWorldV2::decode` valid-world positive + exact-budget validation; assert typed failure and no debit at `N−1` | **Source-predicted fail at unconditional validator guard** | Artifact crate; run first |
| **S2.G2** | Real callable-analysis payload maps every field; owner-local graph closure, two owners each with expression/binding ordinal 0 | Guard rejects nonempty; schema incomplete | Semantic exporter/importer |
| **S2.G3** | `<main>` top-level lambdas, imports, local shadowing and same-range cases: exact restored formal sites and lowering target identity | Missing imported owner graph/formal projection | Semantic/Core |
| **S2.G4** | Native full `ExpressionAnalysis` incl. blocked/invalid/diagnostic/explanation/cause; source/import parity | DTO missing fields | Semantic |
| **S2.G5** | Source/frozen CFG attached to imported analyses, preserving entry/exits/fingerprint and rejecting mutation | Helper has focused tests; full attachment absent | Semantic |
| **S2.G6** | Nested `CheckedStaticBody` applications, repeated ordinals across body owners, exact checked signature and kinded-expression argument | Operational wire missing ID/signature, unsupported argument | Static evaluator/artifact |
| **S2.G7** | Static definition/constant/catalog/binding source→artifact→loaded owner parity and actual evaluator use | Detached template/empty snapshot tables | Semantic/Core |
| **S2.G8** | Trait with nonpublic/default members, specific associated type domain, inherited supertrait and complete member references | Source wire omits facts; synthetic import | Traits |
| **S2.G9** | Generic/conditional conformance witness with requirement views, failure payloads, associated bindings, invalidity, exact completeness | Guard/DTO missing proof dimensions | Witness owner |
| **S2.G10** | Associated plan fingerprint equals recomputation and changes when binding changes | **Source-predicted fail for nonempty binding after current importer assignment** | Witness importer |
| **S2.G11** | Rebuilt `TraitDispatchIndex` and derived conformance rules equal source bucket membership/fingerprints | Empty imported owners | T8 |
| **S2.G12** | Malformed raw enum tags/duplicate witness/trait/conformance owner rows reject before insertion | Insufficient validation/overwriting risk | Artifact validator |
| **S2.G13** | Root registration independent of reversed native parameter numbering/unreachable history; entire bytes identical | **Unverified determinism risk** | Roots/T11 |
| **S2.G14** | Scaling: 1×/2×/4× denotation captures and owner families, record validation time/request bytes | Potential repeated linear scans | Artifact validator |
| **S2.G15** | Fresh-process novel project and standalone package (`parse`, `slotmap`, `similarity` consumer) compilation/execution using prebuilt Universe; fail-fast on canonical source acquisition | End-to-end blocked before loader | T12/T13/T14 |

Every test should assert that it reached its intended stage. A generic `is_err()` after an earlier unavoidable guard is not evidence for a later malformed-owner or runtime failure. Positive tests must inspect owner fields or consumed behavior, not just collection sizes.

---

## 10. Closure-critical implementation sequence

The following is an **implementation ordering recommendation**, not a claim that items have been completed. Keep P7's already accepted separation between compiler artifact (P7), portable executable artifact (P8) and automatic artifact cache/freshness/incremental build (P9). The user's stronger practical success criterion—**compilation and existing-runtime use of standalone packages**—should be included in P7 acceptance without conflating it with P8's serialized VM executables.

### Phase 0 — truth and exact revision checks

- Read live `AGENTS.md`, checkpoint, the current detailed P7 plan and the attached execution/amendment state. Pin the *actual checked-out worktree* and distinguish committed from uncommitted work. Do not reset or wipe ongoing implementer edits.
- Re-run **only** `phalcom-compiler-artifact --test semantic_world_callable_analysis_test` and `--test semantic_world_enum_requirements_test` on the actual head; record whether the source-predicted unconditional decode error manifests. Do not cite historic passes against the current tree.
- Build a field-to-source, wire, root, validation, materializer, owner insertion, fingerprint and compiler-reader manifest. Keep the `Wire` label as **required disposition**, not a completed status.

### Phase 1 — coherent full callable-analysis owner graph

- Complete the typed DTO for every native field, with one canonical ordinal allocation source; preserve checked evidence, invalidity and exact denotations, runtime/iteration/dispatch resolutions, CFG + flow state + return facts, dependencies, diagnostics and explanations.
- Reconstruct into `CallableAnalysisTable` only after *all* rows and links validate; update fresh-store type/static relocation and recompute body fingerprints after relocation. No unqualified ID, source-name lookup, or guessed type.
- Carry the formal/source attachment that core lowering actually reads, preserving stable source-site identity and the explicit `SourceOwner::Module` relation for synthetic `<main>`. Module-valued handles attach to canonical global symbols, not callable-local declaration slots. Add field-level parity and exact lowering tests before removing analysis-family guards.

### Phase 2 — complete trait, witness and compiler owner restoration

- Persist accepted callable-definition provenance and inherent body context; restore actual declaration surface callable knowledge rather than assumed `TypeId(0)`.
- Persist every trait member/associated requirement dimension and supertraits, plus complete witness requirement views, selections, applicability, associated binding plans, proof-status payloads and diagnostic/invalidity state.
- Fix the specific stale associated-plan fingerprint assignment; reconstruct/verify both associated-plan and witness fingerprints after graph relocation and exact owner installation. Add duplicate/malformed tag and cross-table authority validation.
- Load derived conformance **primary** rules and rebuild trait dispatch from validated owner products; attach actual module/static indexes, with no repeated global scans.

### Phase 3 — static operational authority and loaded-image composition

- Reconcile the documentation with native checked-body ownership. Persist exact checked static applications under `StaticBodyId + ordinal`, including checked target/signature/argument correspondence; keep the denotation guard until importer resolver and evaluator parity exist.
- Replace the operational-only export filter with an exporter that consumes every valid checked source representation and produces a complete source-independent template, or proves that an accepted upstream normalization has already done so. Support nested kinded expressions or fail publication for an explicitly unsupported profile with a precise error.
- Insert static definitions/constants/case catalogs/module binding authority into their true snapshot owners. Rebuild the required `module_products`/secondary indexes from validated module/semantic authority. Preserve exact consumer behavior before `SnapshotStatus::Complete`.

### Phase 4 — validation/accounting and acceptance release

- Resolve Slice 1's independent unconditional semantic-validator and semantic-import allocation-accounting barriers *by exact, fallible, measured accounting*, including the owner graph introduced above. A temporary unrestricted decode path is not acceptable production safety.
- Make the native semantic-world validator comprehensive: duplicate/wrong-owner checks, closed discriminants, exact type/trait/witness cross-links and bounded lookup complexity. Require full owner-complete validation before writer publication and loader image publication.
- Only after the real artifact is buildable: run fresh-store source/import field parity, full canonical root fingerprints, cross-process deterministic bytes, cold process/source-denied imported program and standalone package, P5/P6 non-regression, allocator-request accounting and performance gates. Record exact program exits and artifact identities.

**Dependency caveat:** The phases must be implemented at coherent owner seams: new DTO fields require corresponding root registration and validation before graph closure; new import owners require chargeable/fallible allocation before public decode/import guards can be lifted. It is reasonable to develop owner-specific synthetic tests while the global guard remains in place. It is not reasonable to claim release completion from those helper tests or to bypass safety for the full pipeline.

---

## 11. Remaining open questions and explicit audit limits

1. **Formal site identity design:** What minimal owner-qualified portable `SourceSiteId` or equivalent compiler-formal attachment product is sufficient to reproduce *exact* source-built lowering without importing parsed modules or rebuilding the entire editor source index? Requires a direct consumer/producer proof; do not guess an AST range-based relation.
2. **Static checked-body row schema:** Should the operational template reference a dedicated `CheckedStaticApplicationV1` table by body-local ordinal, or carry a fully validated inline checked application record at each operation? Either can be correct if sharing, duplicate/cycle validation, signatures and exact parameter mapping are retained. The actual native authority stays `CheckedStaticBody`.
3. **Dispatch/trait kinded domains:** Which native type/visibility/default facts have independent source authority and must be stored, versus which are derived safely from *complete* validated signature/trait tables? The current `KindId(0)` and visibility shortcuts are not evidence of derivability.
4. **Budgeted native structures:** The trait/witness/callable tables currently allocate maps, interners, strings, arcs and scratch through owners not proved fallible/budgeted. The detailed allocation census belongs to Slice 3, but these owner APIs must be designed with accounting seams now; otherwise new importer fields will perpetuate `UnaccountedBoundary`.
5. **Cross-store stable semantic fingerprints:** Several product fingerprint routines hash native-semantic structures and debug representations (`format!("{:?}", ...)`). Determine which fingerprints are portable identity ingredients versus freshly derived loaded-process evidence. Do not copy process-local numerical fingerprints into an eternal wire ABI without a source/import equality strategy. [Witness hash routine](https://github.com/aureat/phalcom-lang/blob/2939e9d41640dc1eca3fbbc9b8e20529ab05ef6a/phalcom-semantic/src/impls.rs#L2322-L2355).
6. **Latest-test truth:** The checked-in `rustc-ice-2026-10-15T09_58_51-58574.txt` records a nightly compiler incremental verification ICE, not proof of a Telos semantic failure. It should be classified separately from the source-proven decode test contradiction. [ICE file](https://github.com/aureat/phalcom-lang/blob/2939e9d41640dc1eca3fbbc9b8e20529ab05ef6a/rustc-ice-2026-10-15T09_58_51-58574.txt).
7. **Standalone package surface:** P7 should demonstrate compilation and actual existing-runtime execution of realistic language features via the loaded Universe. The explicit `ProgramAnalyzer` path already branches for project, package and module entry selections; this has not yet been validated with a loaded, owner-complete artifact. [Compiler entry choices](https://github.com/aureat/phalcom-lang/blob/2939e9d41640dc1eca3fbbc9b8e20529ab05ef6a/phalcom-core/src/modules/compile.rs#L2330-L2508). P7 does **not** have to deliver P8's independently serialized executable runtime image. The `similarity` sample has a separate `project.toml` entry/layout discrepancy recorded in Slice 1 and should be handled as a fixture or package issue, not concealed by modifying P7's semantics.

### Not performed in this turn

- No native Rust compile/test/fuzz/benchmark execution: no reachable checked-out worktree and the repository was inspected through connected GitHub read APIs. **Source-predicted test failures are not reported as observed Cargo failures.**
- No access to the implementer's private dirty worktree: the report reflects the pushed commit `2939e9d`, not unpushed local modifications.
- No byte-level malicious artifact mutation, end-to-end allocator measurement, read-race simulation or package smoke run. Those are explicitly deferred to dedicated slices/implementer validation.
- No commit, GitHub mutation, user code patch or plan-status edit. This is an audit and implementer guidance artifact only.

---

## 12. Slice 2 closure decision and next audit target

**Decision:** **P7 remains OPEN / BLOCKED FOR COMPLETE SEMANTIC EXPORT, IMPORT AND REAL SOURCE-FREE COMPILER ACCEPTANCE.** The next implementer should prioritize `S2-01` test-truth/decoder boundary and `S2-02/S2-03` complete callable graph plus formal attachment ownership; in parallel within a coherent owner interface, fix `S2-06–S2-12` trait/witness/static/defaulting defects. Do not rush to remove global guards or mark historical helper passes as final acceptance.

**Slice 3 recommendation:** a dedicated deep audit of type/static graph relocation; source/portable identity and root determinism; metadata and semantic native allocation ownership; validator scratch and nested container growth; worst-case complex input handling; writer validation and atomic overwrite safety. Audit all exact budget/transaction paths under the final selected source revision, then leave the acceptance/performance/source-denied project/package system slice for Slice 4.

**Practical acceptance invariant to carry forward:** A fresh process must load a deterministic `universe.telosc`, deny every canonical Universe source acquisition, compile a genuinely new project and standalone package exercising real Telos features, and hand the output to the existing runtime correctly, without recomputing canonical analysis, witness planning or presentation. Those are **P7 compiler-image** guarantees, independent from P8's portable executable artifact and P9's automatic cache/freshness system.
